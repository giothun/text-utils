use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::config::ScraperConfig;
use crate::error::{Result, TextGenError};
use crate::scrapers::{
    providers::{BasicScraper, LyricsScraper, WikipediaScraper},
    scraper_trait::Scraper,
};

pub struct ScraperFactory {
    pub interactive: Option<fn() -> Box<dyn Scraper>>,
    pub from_config: fn(settings: &serde_json::Value) -> Box<dyn Scraper>,
}

pub static SCRAPER_REGISTRY: Lazy<HashMap<&'static str, ScraperFactory>> = Lazy::new(|| {
    let mut m = HashMap::new();

    m.insert(
        "basic",
        ScraperFactory {
            interactive: Some(BasicScraper::interactive_config),
            from_config: BasicScraper::from_config,
        },
    );

    m.insert(
        "wikipedia",
        ScraperFactory {
            interactive: Some(WikipediaScraper::interactive_config),
            from_config: WikipediaScraper::from_config,
        },
    );

    m.insert(
        "lyrics",
        ScraperFactory {
            interactive: Some(LyricsScraper::interactive_config),
            from_config: LyricsScraper::from_config,
        },
    );

    m
});

pub fn load_scraper_from_config(config: &ScraperConfig) -> Result<Box<dyn Scraper>> {
    if let Some(factory) = SCRAPER_REGISTRY.get(config.scraper_type.as_str()) {
        Ok((factory.from_config)(&config.settings))
    } else {
        Err(TextGenError::Config(format!(
            "Unknown scraper type '{}'",
            config.scraper_type
        )))
    }
}

pub fn get_scraper_interactive(scraper_type: &str) -> Result<Box<dyn Scraper>> {
    if let Some(factory) = SCRAPER_REGISTRY.get(scraper_type) {
        if let Some(interactive_fn) = factory.interactive {
            Ok(interactive_fn())
        } else {
            Err(TextGenError::Config(format!(
                "Interactive config is not implemented for '{}'",
                scraper_type
            )))
        }
    } else {
        Err(TextGenError::Config(format!(
            "Unknown scraper type '{}'",
            scraper_type
        )))
    }
}
