use async_trait::async_trait;
use text_gen_ngram::error::{Result, TextGenError};
use text_gen_ngram::model::{Generator, Trainer};
use text_gen_ngram::scrapers::Scraper;

// A mock scraper that returns predefined text
struct MockScraper {
    text: String,
}

impl MockScraper {
    fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
        }
    }
}

#[async_trait]
impl Scraper for MockScraper {
    async fn fetch_text(&self) -> Result<String> {
        Ok(self.text.clone())
    }
}

// A mock scraper that simulates errors
struct ErrorScraper {
    error_message: String,
}

impl ErrorScraper {
    fn new(error_message: &str) -> Self {
        Self {
            error_message: error_message.to_string(),
        }
    }
}

#[async_trait]
impl Scraper for ErrorScraper {
    async fn fetch_text(&self) -> Result<String> {
        Err(TextGenError::Scraper(self.error_message.clone()))
    }
}

#[tokio::test]
async fn test_mock_scraper_success() {
    let text = "This is a test text for the mock scraper.";
    let scraper = MockScraper::new(text);

    let result = scraper.fetch_text().await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), text);
}

#[tokio::test]
async fn test_mock_scraper_error() {
    let error_message = "Simulated scraper error";
    let scraper = ErrorScraper::new(error_message);

    let result = scraper.fetch_text().await;

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(error, TextGenError::Scraper(_)));
    assert!(error.to_string().contains(error_message));
}

#[tokio::test]
async fn test_end_to_end_with_mock() {
    let text = "The quick brown fox jumps over the lazy dog. The fox is quick and brown.";
    let scraper = MockScraper::new(text);

    let scraped_text = scraper.fetch_text().await.unwrap();

    let trainer = Trainer::new(2);
    let model = trainer.train_from_text(&scraped_text).unwrap();

    let generator = Generator::new(&model);
    let seed = Some(vec!["The".to_string(), "fox".to_string()]);
    let generated = generator.generate(seed, 10);

    assert!(generated.starts_with("The fox"));
    assert!(!generated.is_empty());
}
