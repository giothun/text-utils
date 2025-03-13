pub mod http;
pub mod providers;
pub mod registry;
pub mod scraper_trait;

pub use crate::config::ScraperConfig;
pub use http::HTTP_CLIENT;
pub use registry::{get_scraper_interactive, load_scraper_from_config};
pub use scraper_trait::Scraper;
