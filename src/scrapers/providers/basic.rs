use crate::config::ScraperConfig;
use crate::error::{Result, TextGenError};
use crate::scrapers::HTTP_CLIENT;
use crate::scrapers::scraper_trait::Scraper;

use async_trait::async_trait;
use log::{info, warn};
use once_cell::sync::OnceCell;
use scraper::Selector;
use serde_json::json;
use std::io::{self, BufRead};
use std::time::Duration;

const DEFAULT_REQUEST_TIMEOUT_SECS: u64 = 30;

const MAX_HTML_SIZE_BYTES: usize = 10 * 1024 * 1024; // 10MB

pub struct BasicScraper {
    url: String,
    selector: String,
    request_timeout: Duration,
    selector_cache: OnceCell<Selector>,
}

impl BasicScraper {
    pub fn new(url: String, selector: String) -> Self {
        Self {
            url,
            selector,
            request_timeout: Duration::from_secs(DEFAULT_REQUEST_TIMEOUT_SECS),
            selector_cache: OnceCell::new(),
        }
    }

    pub fn with_request_timeout(mut self, timeout_secs: u64) -> Self {
        self.request_timeout = Duration::from_secs(timeout_secs);
        self
    }

    pub fn interactive_config() -> Box<dyn Scraper> {
        println!("-- Basic Scraper Config --");
        println!("Enter URL to scrape:");
        let stdin = io::stdin();
        let mut url = String::new();
        stdin.lock().read_line(&mut url).unwrap();

        println!("Enter CSS selector for text extraction (default: body):");
        let mut selector = String::new();
        stdin.lock().read_line(&mut selector).unwrap();

        let selector = selector.trim();
        let selector = if selector.is_empty() {
            "body"
        } else {
            selector
        };

        println!("Enter request timeout in seconds (default: 30):");
        let mut timeout_input = String::new();
        stdin.lock().read_line(&mut timeout_input).unwrap();

        let request_timeout = timeout_input
            .trim()
            .parse::<u64>()
            .unwrap_or(DEFAULT_REQUEST_TIMEOUT_SECS);

        let config = ScraperConfig {
            scraper_type: "basic".to_string(),
            settings: json!({
                "url": url.trim(),
                "selector": selector,
                "request_timeout": request_timeout,
            }),
        };

        if let Err(e) = crate::config::save_interactive_config(&config) {
            warn!("Failed to save config: {}", e);
        }

        Box::new(
            BasicScraper::new(url.trim().to_string(), selector.to_string())
                .with_request_timeout(request_timeout),
        )
    }

    pub fn from_config(settings: &serde_json::Value) -> Box<dyn Scraper> {
        let url = settings["url"].as_str().unwrap_or_default().to_string();
        let selector_str = settings["selector"].as_str().unwrap_or_default();
        let selector = if selector_str.is_empty() {
            "body"
        } else {
            selector_str
        }
        .to_string();
        let request_timeout = settings["request_timeout"]
            .as_u64()
            .unwrap_or(DEFAULT_REQUEST_TIMEOUT_SECS);

        Box::new(BasicScraper::new(url, selector).with_request_timeout(request_timeout))
    }

    fn get_selector(&self) -> Result<&Selector> {
        self.selector_cache.get_or_try_init(|| {
            Selector::parse(&self.selector).map_err(|e| {
                TextGenError::Scraper(format!("Invalid CSS selector '{}': {}", self.selector, e))
            })
        })
    }
}

#[async_trait]
impl Scraper for BasicScraper {
    async fn fetch_text(&self) -> Result<String> {
        let selector = self.get_selector()?;

        info!("Fetching content from URL: {}", self.url);

        let response = HTTP_CLIENT
            .get(&self.url)
            .timeout(self.request_timeout)
            .send()
            .await
            .map_err(|e| TextGenError::Http(e))?;

        if !response.status().is_success() {
            return Err(TextGenError::Scraper(format!(
                "Failed to fetch content from {}: HTTP {}",
                self.url,
                response.status()
            )));
        }

        if let Some(length) = response.content_length() {
            if length > MAX_HTML_SIZE_BYTES as u64 {
                return Err(TextGenError::Scraper(format!(
                    "HTML content too large: {} bytes (max: {} bytes)",
                    length, MAX_HTML_SIZE_BYTES
                )));
            }
        }

        let html = response.text().await.map_err(|e| TextGenError::Http(e))?;

        if html.len() > MAX_HTML_SIZE_BYTES {
            return Err(TextGenError::Scraper(format!(
                "HTML content too large: {} bytes (max: {} bytes)",
                html.len(),
                MAX_HTML_SIZE_BYTES
            )));
        }

        let document = scraper::Html::parse_document(&html);
        let mut extracted_text = String::new();

        for element in document.select(selector) {
            let text = element.text().collect::<Vec<_>>().join(" ");
            if !text.trim().is_empty() {
                extracted_text.push_str(&text);
                extracted_text.push('\n');
            }
        }

        if extracted_text.is_empty() {
            return Err(TextGenError::Scraper(format!(
                "No text found using selector '{}' at {}",
                self.selector, self.url
            )));
        }

        Ok(extracted_text)
    }
}

impl Clone for BasicScraper {
    fn clone(&self) -> Self {
        Self {
            url: self.url.clone(),
            selector: self.selector.clone(),
            request_timeout: self.request_timeout,
            selector_cache: OnceCell::new(),
        }
    }
}
