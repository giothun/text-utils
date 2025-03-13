use serde_json::json;
use std::fs;
use std::path::Path;
use text_gen_ngram::config::{ScraperConfig, load_config, save_interactive_config};

#[test]
fn test_scraper_config_serialization() {
    let config = ScraperConfig {
        scraper_type: "test".to_string(),
        settings: json!({
            "url": "https://example.com",
            "selector": "body",
            "max_items": 10
        }),
    };

    let json = serde_json::to_string(&config).unwrap();

    let deserialized: ScraperConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.scraper_type, "test");
    assert_eq!(deserialized.settings["url"], "https://example.com");
    assert_eq!(deserialized.settings["selector"], "body");
    assert_eq!(deserialized.settings["max_items"], 10);
}

#[test]
fn test_save_and_load_config() {
    let _temp_file = "test_config_temp.json";

    let config = ScraperConfig {
        scraper_type: "test".to_string(),
        settings: json!({
            "url": "https://example.com",
            "selector": "body"
        }),
    };

    let result = save_interactive_config(&config);
    assert!(result.is_ok());

    let saved_path = result.unwrap();
    assert!(Path::new(&saved_path).exists());

    let loaded_result = load_config(&saved_path);
    assert!(loaded_result.is_ok());

    let loaded_config = loaded_result.unwrap();
    assert_eq!(loaded_config.scraper_type, config.scraper_type);
    assert_eq!(loaded_config.settings["url"], config.settings["url"]);

    fs::remove_file(saved_path).unwrap_or_default();

    let result = load_config("nonexistent_file.json");
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Failed to open config file")
    );
}

#[test]
fn test_config_with_invalid_json() {
    let temp_file = "invalid_config.json";
    fs::write(temp_file, "{ invalid json }").unwrap();

    let result = load_config(temp_file);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Failed to parse config")
    );

    fs::remove_file(temp_file).unwrap_or_default();
}
