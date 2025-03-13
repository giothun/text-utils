use crate::config::ScraperConfig;
use crate::error::{Result, TextGenError};
use crate::scrapers::HTTP_CLIENT;
use crate::scrapers::scraper_trait::Scraper;

use async_trait::async_trait;
use futures::future::join_all;
use log::{info, warn};
use serde_json::json;
use std::io::{self, BufRead};
use std::time::Duration;

const DEFAULT_REQUEST_TIMEOUT_SECS: u64 = 30;

pub struct WikipediaScraper {
    topics: Vec<String>,
    request_timeout: Duration,
}

impl WikipediaScraper {
    pub fn new(topics: Vec<String>) -> Self {
        Self {
            topics,
            request_timeout: Duration::from_secs(DEFAULT_REQUEST_TIMEOUT_SECS),
        }
    }

    pub fn with_request_timeout(mut self, timeout_secs: u64) -> Self {
        self.request_timeout = Duration::from_secs(timeout_secs);
        self
    }

    pub fn interactive_config() -> Box<dyn Scraper> {
        println!("-- Wikipedia Scraper Config --");
        println!("Enter topics to fetch (comma-separated):");
        let stdin = io::stdin();
        let mut topics_input = String::new();
        stdin.lock().read_line(&mut topics_input).unwrap();

        let topics: Vec<String> = topics_input
            .trim()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        println!("Enter request timeout in seconds (default: 30):");
        let mut timeout_input = String::new();
        stdin.lock().read_line(&mut timeout_input).unwrap();

        let request_timeout = timeout_input
            .trim()
            .parse::<u64>()
            .unwrap_or(DEFAULT_REQUEST_TIMEOUT_SECS);

        let config = ScraperConfig {
            scraper_type: "wikipedia".to_string(),
            settings: json!({
                "topics": topics,
                "request_timeout": request_timeout,
            }),
        };

        if let Err(e) = crate::config::save_interactive_config(&config) {
            warn!("Failed to save config: {}", e);
        }

        Box::new(WikipediaScraper::new(topics).with_request_timeout(request_timeout))
    }

    pub fn from_config(settings: &serde_json::Value) -> Box<dyn Scraper> {
        let topics = if let Some(topics_array) = settings["topics"].as_array() {
            topics_array
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        } else {
            Vec::new()
        };

        let request_timeout = settings["request_timeout"]
            .as_u64()
            .unwrap_or(DEFAULT_REQUEST_TIMEOUT_SECS);

        Box::new(WikipediaScraper::new(topics).with_request_timeout(request_timeout))
    }

    async fn fetch_wikipedia_summary(&self, topic: &str) -> Result<String> {
        let url = format!(
            "https://en.wikipedia.org/api/rest_v1/page/summary/{}",
            urlencoding::encode(topic)
        );

        let response = HTTP_CLIENT
            .get(&url)
            .timeout(self.request_timeout)
            .send()
            .await
            .map_err(|e| TextGenError::Http(e))?;

        if !response.status().is_success() {
            return Err(TextGenError::Scraper(format!(
                "Failed to fetch Wikipedia summary for '{}': HTTP {}",
                topic,
                response.status()
            )));
        }

        let data: serde_json::Value = response.json().await.map_err(|e| TextGenError::Http(e))?;

        let title = data["title"].as_str().unwrap_or(topic);
        let extract = data["extract"].as_str().ok_or_else(|| {
            TextGenError::Scraper(format!("No extract found for topic '{}'", topic))
        })?;

        Ok(format!("# {}\n\n{}", title, extract))
    }
}

#[async_trait]
impl Scraper for WikipediaScraper {
    async fn fetch_text(&self) -> Result<String> {
        if self.topics.is_empty() {
            return Err(TextGenError::Scraper("No topics specified".to_string()));
        }

        let futures = self.topics.iter().map(|topic| {
            let topic_clone = topic.clone();
            async move {
                match self.fetch_wikipedia_summary(&topic_clone).await {
                    Ok(summary) => {
                        info!("Successfully fetched summary for '{}'", topic_clone);
                        Ok(summary)
                    }
                    Err(e) => {
                        warn!("Failed to get summary for '{}': {}", topic_clone, e);
                        Err(e)
                    }
                }
            }
        });

        let results = join_all(futures).await;

        let summaries: Vec<String> = results
            .into_iter()
            .filter_map(|result| result.ok())
            .collect();

        if summaries.is_empty() {
            return Err(TextGenError::Scraper(
                "No Wikipedia summaries were successfully fetched".to_string(),
            ));
        }

        Ok(summaries.join("\n\n"))
    }
}

impl Clone for WikipediaScraper {
    fn clone(&self) -> Self {
        Self {
            topics: self.topics.clone(),
            request_timeout: self.request_timeout,
        }
    }
}
