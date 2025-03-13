use log::debug;
use once_cell::sync::Lazy;
use reqwest::Client;

const DEFAULT_USER_AGENT: &str = "text-gen-ngram/1.0";

pub static HTTP_CLIENT: Lazy<Client> = Lazy::new(|| {
    debug!("Initializing shared HTTP client");
    Client::builder()
        .user_agent(DEFAULT_USER_AGENT)
        .pool_max_idle_per_host(10)
        .build()
        .unwrap_or_else(|_| Client::new())
});
