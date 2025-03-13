use crate::error::Result;
use async_trait::async_trait;
use std::time::Duration;
use tokio::time::sleep;

#[async_trait]
pub trait Scraper: Send + Sync {
    /// Fetches text from the source
    ///
    /// This method is the main entry point for scrapers. It should:
    /// 1. Connect to the source (web, API, etc.)
    /// 2. Download the raw content
    /// 3. Extract the relevant text
    /// 4. Return the cleaned text
    ///
    /// # Returns
    ///
    /// A Result containing either the scraped text or an error
    async fn fetch_text(&self) -> Result<String>;

    /// Applies rate limiting by sleeping for the specified duration
    ///
    /// This is a utility method that can be used by implementations to
    /// respect rate limits of the services they're scraping.
    ///
    /// # Arguments
    ///
    /// * `duration` - The duration to sleep for
    async fn rate_limit(&self, duration: Duration) -> Result<()> {
        sleep(duration).await;
        Ok(())
    }
}

pub trait ScraperExt {
    fn is_null(&self) -> bool;
}

impl ScraperExt for Box<dyn Scraper> {
    fn is_null(&self) -> bool {
        false
    }
}
