use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs::File;
use std::io::Write;
#[derive(Debug, Deserialize, Serialize)]
pub struct ScraperConfig {
    pub scraper_type: String,
    pub settings: Value,
}
pub fn save_interactive_config(config: &ScraperConfig) -> Result<()> {
    let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
    let filename = format!(
        "interactive_config_{}_{}.json",
        config.scraper_type, timestamp
    );
    let json_str = serde_json::to_string_pretty(config)?;
    let mut file =
        File::create(&filename).with_context(|| format!("Unable to create file: {}", filename))?;
    file.write_all(json_str.as_bytes())?;
    println!("Interactive config saved as: {}", filename);
    Ok(())
}
