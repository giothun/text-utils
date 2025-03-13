use crate::error::{Result, TextGenError};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScraperConfig {
    pub scraper_type: String,
    pub settings: serde_json::Value,
}

pub fn save_interactive_config(config: &ScraperConfig) -> Result<String> {
    let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
    let filename = format!(
        "interactive_config_{}_{}.json",
        config.scraper_type, timestamp
    );

    let file = File::create(&filename)
        .map_err(|e| TextGenError::Config(format!("Failed to create config file: {}", e)))?;

    serde_json::to_writer_pretty(file, config)
        .map_err(|e| TextGenError::Config(format!("Failed to write config: {}", e)))?;

    Ok(filename)
}

pub fn load_config<P: AsRef<Path>>(path: P) -> Result<ScraperConfig> {
    let file = File::open(path)
        .map_err(|e| TextGenError::Config(format!("Failed to open config file: {}", e)))?;

    let reader = BufReader::new(file);
    let config = serde_json::from_reader(reader)
        .map_err(|e| TextGenError::Config(format!("Failed to parse config: {}", e)))?;

    Ok(config)
}
