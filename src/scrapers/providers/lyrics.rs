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

pub struct LyricsScraper {
    api_token: String,
    artists: Vec<String>,
    max_songs_per_artist: Option<usize>,
    request_timeout: Duration,
}

impl LyricsScraper {
    pub fn new(
        api_token: String,
        artists: Vec<String>,
        max_songs_per_artist: Option<usize>,
    ) -> Self {
        Self {
            api_token,
            artists,
            max_songs_per_artist,
            request_timeout: Duration::from_secs(DEFAULT_REQUEST_TIMEOUT_SECS),
        }
    }

    pub fn with_request_timeout(mut self, timeout_secs: u64) -> Self {
        self.request_timeout = Duration::from_secs(timeout_secs);
        self
    }

    pub fn interactive_config() -> Box<dyn Scraper> {
        println!("-- Lyrics Scraper Config --");
        println!("Enter Genius API token:");
        let stdin = io::stdin();
        let mut api_token = String::new();
        stdin.lock().read_line(&mut api_token).unwrap();

        println!("Enter artist names (comma-separated):");
        let mut artists_input = String::new();
        stdin.lock().read_line(&mut artists_input).unwrap();

        let artists: Vec<String> = artists_input
            .trim()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        println!("Enter maximum songs per artist (optional, press Enter for no limit):");
        let mut max_songs_input = String::new();
        stdin.lock().read_line(&mut max_songs_input).unwrap();

        let max_songs_per_artist = if max_songs_input.trim().is_empty() {
            None
        } else {
            max_songs_input.trim().parse::<usize>().ok().or_else(|| {
                eprintln!("Invalid number, using no limit");
                None
            })
        };

        println!("Enter request timeout in seconds (default: 30):");
        let mut timeout_input = String::new();
        stdin.lock().read_line(&mut timeout_input).unwrap();

        let request_timeout = timeout_input
            .trim()
            .parse::<u64>()
            .unwrap_or(DEFAULT_REQUEST_TIMEOUT_SECS);

        let config = ScraperConfig {
            scraper_type: "lyrics".to_string(),
            settings: json!({
                "api_token": api_token.trim(),
                "artists": artists,
                "max_songs_per_artist": max_songs_per_artist,
                "request_timeout": request_timeout,
            }),
        };

        if let Err(e) = crate::config::save_interactive_config(&config) {
            warn!("Failed to save config: {}", e);
        }

        Box::new(
            LyricsScraper::new(api_token.trim().to_string(), artists, max_songs_per_artist)
                .with_request_timeout(request_timeout),
        )
    }

    pub fn from_config(settings: &serde_json::Value) -> Box<dyn Scraper> {
        let api_token = settings["api_token"]
            .as_str()
            .unwrap_or_default()
            .to_string();

        let artists = if let Some(artists_array) = settings["artists"].as_array() {
            artists_array
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        } else {
            Vec::new()
        };

        let max_songs_per_artist = settings["max_songs_per_artist"]
            .as_u64()
            .map(|n| n as usize);

        let request_timeout = settings["request_timeout"]
            .as_u64()
            .unwrap_or(DEFAULT_REQUEST_TIMEOUT_SECS);

        Box::new(
            LyricsScraper::new(api_token, artists, max_songs_per_artist)
                .with_request_timeout(request_timeout),
        )
    }

    async fn search_artist(&self, artist_name: &str) -> Result<u64> {
        let url = format!(
            "https://api.genius.com/search?q={}",
            urlencoding::encode(artist_name)
        );

        let response = HTTP_CLIENT
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .timeout(self.request_timeout)
            .send()
            .await
            .map_err(|e| TextGenError::Http(e))?;

        if !response.status().is_success() {
            return Err(TextGenError::Scraper(format!(
                "Failed to search for artist '{}': HTTP {}",
                artist_name,
                response.status()
            )));
        }

        let data: serde_json::Value = response.json().await.map_err(|e| TextGenError::Http(e))?;

        let hits = data["response"]["hits"].as_array().ok_or_else(|| {
            TextGenError::Scraper(format!(
                "Unexpected response format when searching for artist '{}'",
                artist_name
            ))
        })?;

        for hit in hits {
            if let Some(primary_artist) = hit["result"]["primary_artist"].as_object() {
                if primary_artist["name"]
                    .as_str()
                    .map(|name| name.to_lowercase() == artist_name.to_lowercase())
                    .unwrap_or(false)
                {
                    return Ok(primary_artist["id"].as_u64().ok_or_else(|| {
                        TextGenError::Scraper(format!("Artist ID not found for '{}'", artist_name))
                    })?);
                }
            }
        }

        Err(TextGenError::Scraper(format!(
            "Artist '{}' not found",
            artist_name
        )))
    }

    async fn get_artist_songs(&self, artist_id: u64) -> Result<Vec<(String, String)>> {
        let mut page = 1;
        let mut all_songs = Vec::new();
        let per_page = 50;

        loop {
            let url = format!(
                "https://api.genius.com/artists/{}/songs?per_page={}&page={}",
                artist_id, per_page, page
            );

            let response = HTTP_CLIENT
                .get(&url)
                .header("Authorization", format!("Bearer {}", self.api_token))
                .timeout(self.request_timeout)
                .send()
                .await
                .map_err(|e| TextGenError::Http(e))?;

            if !response.status().is_success() {
                return Err(TextGenError::Scraper(format!(
                    "Failed to get songs for artist {}: HTTP {}",
                    artist_id,
                    response.status()
                )));
            }

            let data: serde_json::Value =
                response.json().await.map_err(|e| TextGenError::Http(e))?;

            let songs = data["response"]["songs"].as_array().ok_or_else(|| {
                TextGenError::Scraper(format!(
                    "Unexpected response format when getting songs for artist {}",
                    artist_id
                ))
            })?;

            if songs.is_empty() {
                break;
            }

            for song in songs {
                if let (Some(title), Some(url)) = (song["title"].as_str(), song["url"].as_str()) {
                    all_songs.push((title.to_string(), url.to_string()));
                }
            }

            if let Some(max_songs) = self.max_songs_per_artist {
                if all_songs.len() >= max_songs {
                    all_songs.truncate(max_songs);
                    break;
                }
            }

            page += 1;
        }

        Ok(all_songs)
    }

    async fn get_song_lyrics(&self, song_url: &str) -> Result<String> {
        let response = HTTP_CLIENT
            .get(song_url)
            .timeout(self.request_timeout)
            .send()
            .await
            .map_err(|e| TextGenError::Http(e))?;

        if !response.status().is_success() {
            return Err(TextGenError::Scraper(format!(
                "Failed to get lyrics from {}: HTTP {}",
                song_url,
                response.status()
            )));
        }

        let html = response.text().await.map_err(|e| TextGenError::Http(e))?;

        let document = scraper::Html::parse_document(&html);

        let selector =
            scraper::Selector::parse("div[class*='Lyrics__Container']").map_err(|_| {
                TextGenError::Scraper("Failed to parse selector for lyrics container".to_string())
            })?;

        let mut lyrics = String::new();
        for element in document.select(&selector) {
            let text = element.text().collect::<Vec<_>>().join("");
            lyrics.push_str(&text);
            lyrics.push('\n');
        }

        if lyrics.is_empty() {
            return Err(TextGenError::Scraper(format!(
                "No lyrics found at {}",
                song_url
            )));
        }

        Ok(lyrics)
    }
}

#[async_trait]
impl Scraper for LyricsScraper {
    async fn fetch_text(&self) -> Result<String> {
        let mut all_lyrics = Vec::new();

        for artist_name in &self.artists {
            info!("Searching for artist: {}", artist_name);
            let artist_id = match self.search_artist(artist_name).await {
                Ok(id) => id,
                Err(e) => {
                    warn!("Failed to find artist '{}': {}", artist_name, e);
                    continue;
                }
            };

            info!("Getting songs for artist: {}", artist_name);
            let songs = match self.get_artist_songs(artist_id).await {
                Ok(s) => s,
                Err(e) => {
                    warn!("Failed to get songs for artist '{}': {}", artist_name, e);
                    continue;
                }
            };

            info!("Found {} songs for artist: {}", songs.len(), artist_name);

            let futures = songs.iter().map(|(title, url)| {
                let artist_name = artist_name.clone();
                let title_clone = title.clone();
                let url_clone = url.clone();

                async move {
                    match self.get_song_lyrics(&url_clone).await {
                        Ok(lyrics) => {
                            info!("Successfully fetched lyrics for '{}'", title_clone);
                            Ok(format!("# {} - {}\n\n{}", artist_name, title_clone, lyrics))
                        }
                        Err(e) => {
                            warn!("Failed to get lyrics for '{}': {}", title_clone, e);
                            Err(e)
                        }
                    }
                }
            });

            let results = join_all(futures).await;

            let artist_lyrics: Vec<String> = results
                .into_iter()
                .filter_map(|result| result.ok())
                .collect();

            all_lyrics.extend(artist_lyrics);
        }

        if all_lyrics.is_empty() {
            return Err(TextGenError::Scraper(
                "No lyrics were successfully fetched".to_string(),
            ));
        }

        Ok(all_lyrics.join("\n\n"))
    }
}

impl Clone for LyricsScraper {
    fn clone(&self) -> Self {
        Self {
            api_token: self.api_token.clone(),
            artists: self.artists.clone(),
            max_songs_per_artist: self.max_songs_per_artist,
            request_timeout: self.request_timeout,
        }
    }
}
