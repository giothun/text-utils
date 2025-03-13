use serde_json::json;
use text_gen_ngram::scrapers::{
    Scraper, ScraperConfig, load_scraper_from_config,
    providers::{BasicScraper, WikipediaScraper},
    scraper_trait::ScraperExt,
};

#[tokio::test]
async fn test_basic_scraper() {
    let scraper = BasicScraper::new("https://httpbin.org/html".to_string(), "html".to_string());

    let result = scraper.fetch_text().await;

    assert!(result.is_ok(), "Failed to fetch text: {:?}", result.err());
    let text = result.unwrap();
    assert!(!text.is_empty());
    assert!(text.contains("Herman Melville"));
}

#[tokio::test]
async fn test_wikipedia_scraper() {
    let scraper = WikipediaScraper::new(vec!["Rust_(programming_language)".to_string()]);

    let result = scraper.fetch_text().await;

    assert!(
        result.is_ok(),
        "Failed to fetch Wikipedia text: {:?}",
        result.err()
    );
    let text = result.unwrap();
    assert!(!text.is_empty());
    assert!(text.contains("Rust"));
}

#[test]
fn test_scraper_from_config() {
    let config = ScraperConfig {
        scraper_type: "basic".to_string(),
        settings: json!({
            "url": "https://example.com",
            "selector": "body"
        }),
    };

    let result = load_scraper_from_config(&config);
    assert!(
        result.is_ok(),
        "Failed to load scraper from config: {:?}",
        result.err()
    );

    let invalid_config = ScraperConfig {
        scraper_type: "nonexistent".to_string(),
        settings: json!({}),
    };

    let result = load_scraper_from_config(&invalid_config);
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("Unknown scraper type"));
    }
}

#[test]
fn test_basic_scraper_from_config() {
    let settings = json!({
        "url": "https://example.com",
        "selector": "body"
    });
    let _scraper = BasicScraper::from_config(&settings);

    let minimal_settings = json!({});
    let scraper = BasicScraper::from_config(&minimal_settings);

    assert!(!scraper.is_null());
}
