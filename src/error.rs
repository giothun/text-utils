use thiserror::Error;

#[derive(Error, Debug)]
pub enum TextGenError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Image error: {0}")]
    Image(String),

    #[error("Scraper error: {0}")]
    Scraper(String),

    #[error("Model error: {0}")]
    Model(String),

    #[error("Tokenization error: {0}")]
    Tokenization(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, TextGenError>;
