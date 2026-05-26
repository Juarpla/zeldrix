use std::time::Duration;
use reqwest::Client;

/// Check if llama-server is healthy by calling /v1/models
pub async fn check_health(port: u16) -> bool {
    let client = Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .ok();

    let Some(client) = client else {
        return false;
    };

    let url = format!("http://127.0.0.1:{}/v1/models", port);
    let response = client.get(&url).send().await;

    match response {
        Ok(resp) => resp.status().is_success(),
        Err(_) => false,
    }
}