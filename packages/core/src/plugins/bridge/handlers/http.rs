//! HTTP request handling functionality for the service bridge

use std::collections::HashMap;

use log::{debug, error};

use crate::plugins::interface::HttpMethod;

/// Handle HTTP request
pub async fn handle_http_request(
    plugin_id: String,
    _request_id: String,
    _callback_fn_name: String,
    method: HttpMethod,
    url: String,
    headers: HashMap<String, String>,
    body: Option<Vec<u8>>,
) -> Result<action_items_native::HttpResponseData, String> {
    debug!(
        "Processing HTTP {:?} request to {} for plugin {}",
        method, url, plugin_id
    );

    // Create HTTP client
    let client = reqwest::Client::new();

    // Build request
    let mut request_builder = match method {
        HttpMethod::Get => client.get(&url),
        HttpMethod::Post => client.post(&url),
        HttpMethod::Put => client.put(&url),
        HttpMethod::Delete => client.delete(&url),
        HttpMethod::Patch => client.patch(&url),
        HttpMethod::Head => client.head(&url),
    };

    // Add headers
    for (key, value) in headers {
        request_builder = request_builder.header(&key, &value);
    }

    // Add body if present
    if let Some(body_data) = body {
        request_builder = request_builder.body(body_data);
    }

    // Execute request
    match request_builder.send().await {
        Ok(response) => {
            let status = response.status().as_u16();
            let headers: HashMap<String, String> = response
                .headers()
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
                .collect();

            match response.bytes().await {
                Ok(body_bytes) => {
                    debug!(
                        "HTTP request successful for plugin {} (status: {})",
                        plugin_id, status
                    );

                    let http_response = action_items_native::HttpResponseData {
                        status,
                        headers,
                        body: body_bytes.to_vec(),
                    };

                    // Return the HttpResponseData directly
                    Ok(http_response)
                },
                Err(e) => {
                    error!(
                        "Failed to read HTTP response body for plugin {}: {}",
                        plugin_id, e
                    );
                    Err(format!("Failed to read response body: {e}"))
                },
            }
        },
        Err(e) => {
            error!("HTTP request failed for plugin {}: {}", plugin_id, e);
            Err(format!("HTTP request failed: {e}"))
        },
    }
}
