use std::io;
use text_gen_ngram::error::{Result, TextGenError};

#[test]
fn test_error_conversion() {
    let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
    let text_gen_error = TextGenError::Io(io_error);

    assert!(matches!(text_gen_error, TextGenError::Io(_)));
    assert!(text_gen_error.to_string().contains("I/O error"));
    assert!(text_gen_error.to_string().contains("file not found"));

    let scraper_error = TextGenError::Scraper("Failed to scrape".to_string());
    assert!(matches!(scraper_error, TextGenError::Scraper(_)));
    assert!(scraper_error.to_string().contains("Failed to scrape"));

    let model_error = TextGenError::Model("Invalid model".to_string());
    assert!(matches!(model_error, TextGenError::Model(_)));
    assert!(model_error.to_string().contains("Invalid model"));
}

#[test]
fn test_result_type() {
    let ok_result: Result<i32> = Ok(42);
    assert_eq!(ok_result.unwrap(), 42);

    let err_result: Result<i32> = Err(TextGenError::Tokenization("Bad token".to_string()));
    assert!(err_result.is_err());
    assert!(matches!(
        err_result.unwrap_err(),
        TextGenError::Tokenization(_)
    ));
}

#[test]
fn test_error_propagation() {
    fn returns_error() -> Result<()> {
        Err(TextGenError::Config("Bad config".to_string()))
    }

    fn propagates_error() -> Result<()> {
        returns_error()?;
        Ok(())
    }

    let result = propagates_error();
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), TextGenError::Config(_)));
}
