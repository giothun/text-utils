pub mod app;
pub mod cli;
pub mod config;
pub mod error;
pub mod model;
pub mod output;
pub mod scrapers;
pub mod text;

// Re-export commonly used types for convenience
pub use app::run_app;
pub use config::{load_config, save_interactive_config};
pub use error::{Result, TextGenError};
pub use model::{Generator, ModelStats, NGramModel, Trainer};
pub use output::insights::show_text_insights;
pub use output::wordcloud::generate_wordcloud;
pub use scrapers::{Scraper, ScraperConfig};
pub use text::{TokenizerOptions, normalize_text, tokenize};
