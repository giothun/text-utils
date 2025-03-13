use reqwest::Client;
use text_gen_ngram::scrapers::HTTP_CLIENT;

#[test]
fn test_http_client_initialization() {
    let client = &*HTTP_CLIENT;

    assert!(client.get("https://example.com").build().is_ok());
}

#[tokio::test]
async fn test_http_client_request() {
    let response = HTTP_CLIENT.get("https://httpbin.org/get").send().await;

    assert!(
        response.is_ok(),
        "HTTP request failed: {:?}",
        response.err()
    );

    let response = response.unwrap();
    assert!(response.status().is_success());

    let json = response.json::<serde_json::Value>().await;
    assert!(json.is_ok(), "Failed to parse JSON: {:?}", json.err());
}

#[test]
fn test_client_builder_defaults() {
    let client = Client::builder().user_agent("test-agent").build();

    assert!(client.is_ok());

    let client = client.unwrap();
    assert!(client.get("https://example.com").build().is_ok());
}
