use crate::models::*;
use std::collections::HashMap;
use std::time::Instant;

/// Execute an HTTP request and return the execution result
pub async fn execute_request(
    data: &RequestData,
    variables: &HashMap<String, String>,
    base_path: &str,
) -> Result<(ResponseData, u64), String> {
    let url = super::interpolation::resolve_url(base_path, &data.url, variables);
    let url = super::interpolation::interpolate(&url, variables);

    let client = reqwest::Client::builder()
        .brotli(true)
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {e}"))?;

    let method = match data.method {
        HttpMethod::GET => reqwest::Method::GET,
        HttpMethod::POST => reqwest::Method::POST,
        HttpMethod::PUT => reqwest::Method::PUT,
        HttpMethod::PATCH => reqwest::Method::PATCH,
        HttpMethod::DELETE => reqwest::Method::DELETE,
        HttpMethod::HEAD => reqwest::Method::HEAD,
        HttpMethod::OPTIONS => reqwest::Method::OPTIONS,
    };

    let mut builder = client.request(method, &url);

    // Add headers
    for h in &data.headers {
        if h.enabled && !h.key.is_empty() {
            let key = super::interpolation::interpolate(&h.key, variables);
            let val = super::interpolation::interpolate(&h.value, variables);
            builder = builder.header(&key, &val);
        }
    }

    // Add query params
    let query_pairs: Vec<(String, String)> = data
        .query_params
        .iter()
        .filter(|p| p.enabled && !p.key.is_empty())
        .map(|p| {
            (
                super::interpolation::interpolate(&p.key, variables),
                super::interpolation::interpolate(&p.value, variables),
            )
        })
        .collect();
    if !query_pairs.is_empty() {
        builder = builder.query(&query_pairs);
    }

    // Add body
    match data.body_type {
        BodyType::Json => {
            let body = super::interpolation::interpolate(&data.body, variables);
            builder = builder
                .header("Content-Type", "application/json")
                .body(body);
        }
        BodyType::FormUrlEncoded => {
            let body = super::interpolation::interpolate(&data.body, variables);
            builder = builder
                .header("Content-Type", "application/x-www-form-urlencoded")
                .body(body);
        }
        BodyType::Raw => {
            let body = super::interpolation::interpolate(&data.body, variables);
            builder = builder.body(body);
        }
        BodyType::None => {}
    }

    let start = Instant::now();
    let response = builder.send().await.map_err(|e| format!("Request failed: {e}"))?;
    let latency_ms = start.elapsed().as_millis() as u64;

    let status = response.status().as_u16();
    let status_text = response
        .status()
        .canonical_reason()
        .unwrap_or("Unknown")
        .to_string();

    let mut headers = HashMap::new();
    for (key, value) in response.headers().iter() {
        if let Ok(v) = value.to_str() {
            headers.insert(key.to_string(), v.to_string());
        }
    }

    let body_bytes = response.bytes().await.map_err(|e| format!("Failed to read body: {e}"))?;
    let size_bytes = body_bytes.len() as u64;
    let body = String::from_utf8_lossy(&body_bytes).to_string();

    Ok((
        ResponseData {
            status,
            status_text,
            headers,
            body,
            size_bytes,
        },
        latency_ms,
    ))
}
