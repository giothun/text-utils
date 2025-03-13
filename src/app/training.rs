use crate::{
    cli::CliArgs,
    config::load_config,
    error::{Result as TextGenResult, TextGenError},
    model::{NGramModel, Trainer},
    scrapers::{Scraper, get_scraper_interactive, load_scraper_from_config},
    text::TokenizerOptions,
};

use chrono::Utc;
use log::{debug, info, warn};
use tokio::fs;

use super::utils::ask_user;

pub async fn train_new_model(args: &CliArgs) -> TextGenResult<NGramModel> {
    let source_text = if let Some(input_file) = &args.input_file {
        load_text_from_file(input_file).await?
    } else {
        fetch_text_from_scraper(args).await?
    };

    let tokenizer_options = TokenizerOptions {
        lowercase: args.lowercase,
        preserve_punctuation: args.preserve_punctuation,
        preserve_sentence_boundaries: args.preserve_sentence_boundaries,
    };

    info!("Training model with n-gram size: {}", args.n);
    debug!(
        "Tokenizer options: lowercase={}, preserve_punctuation={}, preserve_sentence_boundaries={}",
        tokenizer_options.lowercase,
        tokenizer_options.preserve_punctuation,
        tokenizer_options.preserve_sentence_boundaries
    );

    let trainer = Trainer::new(args.n).with_tokenizer_options(tokenizer_options);
    let model = trainer.train_from_text(&source_text)?;
    info!("Model training complete");

    Ok(model)
}

async fn load_text_from_file(input_file: &std::path::Path) -> TextGenResult<String> {
    info!("Loading source text from file: {}", input_file.display());
    fs::read_to_string(input_file)
        .await
        .map_err(|e| TextGenError::Io(e))
}

async fn fetch_text_from_scraper(args: &CliArgs) -> TextGenResult<String> {
    let scraper: Box<dyn Scraper> = if let Some(config_path) = &args.scraper_config {
        info!(
            "Loading scraper configuration from: {}",
            config_path.display()
        );
        match load_config(config_path) {
            Ok(config) => load_scraper_from_config(&config)?,
            Err(e) => {
                warn!(
                    "Error loading config file: {}. Using interactive config instead.",
                    e
                );
                get_scraper_interactive(&args.scraper_type)?
            }
        }
    } else {
        info!(
            "Using interactive configuration for scraper type: {}",
            args.scraper_type
        );
        get_scraper_interactive(&args.scraper_type)?
    };

    info!("Fetching text using scraper...");
    let scraped_text = scraper.fetch_text().await?;
    info!(
        "Successfully fetched {} characters of text",
        scraped_text.len()
    );

    if ask_user("Do you want to save scraped data? (y/n): ") {
        let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
        let filename = format!("scraped_data_{}.txt", timestamp);

        info!("Saving scraped data to {}", filename);
        fs::write(&filename, &scraped_text)
            .await
            .map_err(|e| TextGenError::Io(e))?;
        info!("Scraped data saved successfully");
    }

    Ok(scraped_text)
}
