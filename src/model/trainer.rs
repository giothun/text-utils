use crate::error::Result;
use crate::model::NGramModel;
use crate::scrapers::scraper_trait::Scraper;
use crate::text::TokenizerOptions;
use crate::text::processing::{normalize_text, tokenize_large_text};

pub struct Trainer {
    n: usize,
    tokenizer_options: TokenizerOptions,
}

impl Trainer {
    pub fn new(n: usize) -> Self {
        Self {
            n,
            tokenizer_options: TokenizerOptions::default(),
        }
    }

    pub fn with_tokenizer_options(mut self, options: TokenizerOptions) -> Self {
        self.tokenizer_options = options;
        self
    }

    pub async fn train(&self, scraper: &dyn Scraper) -> Result<NGramModel> {
        let text = scraper.fetch_text().await?;
        self.train_from_text(&text)
    }

    pub fn train_from_text(&self, text: &str) -> Result<NGramModel> {
        let normalized_text = normalize_text(text);

        let tokens = tokenize_large_text(&normalized_text, &self.tokenizer_options)?;

        let mut model = NGramModel::new(self.n);
        model.add_tokens(&tokens);
        Ok(model)
    }
}
