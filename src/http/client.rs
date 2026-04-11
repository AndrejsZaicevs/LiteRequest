use crate::models::*;
use std::collections::HashMap;
use std::time::Instant;

/// Check whether `pattern` matches `host`.
/// Supports exact match and leading wildcard `*.example.com`.
fn host_matches(pattern: &str, host: &str) -> bool {
    if pattern == host {
        return true;
    }
    if let Some(suffix) = pattern.strip_prefix("*.") {
        // *.example.com matches foo.example.com but not example.com
        if host.ends_with(suffix) && host.len() > suffix.len() {
            let prefix = &host[..host.len() - suffix.len()];
            return prefix.ends_with('.');
        }
    }
    false
}

/// Build a reqwest `Identity` from a `ClientCertEntry`, reading files from disk.
fn build_identity(
    entry: &ClientCertEntry,
) -> Result<reqwest::Identity, String> {
    match entry.cert_type {
        CertType::Pem => {
            let cert_bytes = std::fs::read(&entry.cert_path)
                .map_err(|e| format!("Failed to read cert file '{}': {e}", entry.cert_path))?;
            let key_bytes = std::fs::read(&entry.key_path)
                .map_err(|e| format!("Failed to read key file '{}': {e}", entry.key_path))?;
            // reqwest Identity::from_pem expects cert + key concatenated
            let mut pem = cert_bytes;
            pem.push(b'\n');
            pem.extend_from_slice(&key_bytes);
            reqwest::Identity::from_pem(&pem)
                .map_err(|e| format!("Invalid PEM identity: {e}"))
        }
        CertType::Pkcs12 => {
            let pfx_bytes = std::fs::read(&entry.cert_path)
                .map_err(|e| format!("Failed to read PFX file '{}': {e}", entry.cert_path))?;
            reqwest::Identity::from_pkcs12_der(&pfx_bytes, &entry.passphrase)
                .map_err(|e| format!("Invalid PKCS12 identity: {e}"))
        }
    }
}

/// Execute an HTTP request and return the execution result
pub async fn execute_request(
    data: &RequestData,
    variables: &HashMap<String, String>,
    base_path: &str,
    client_certs: &[ClientCertEntry],
) -> Result<(ResponseData, u64), String> {
    let url = super::interpolation::resolve_url(base_path, &data.url, variables);
    let url = super::interpolation::interpolate(&url, variables);

    // Find matching client cert for this URL's host
    let parsed_host = url::Url::parse(&url)
        .ok()
        .and_then(|u| u.host_str().map(|h| h.to_string()));

    let mut builder = reqwest::Client::builder().brotli(true);

    if let Some(ref host) = parsed_host {
        if let Some(entry) = client_certs
            .iter()
            .find(|c| c.enabled && host_matches(&c.host, host))
        {
            let identity = build_identity(entry)?;

            match entry.cert_type {
                CertType::Pem => {
                    builder = builder.use_rustls_tls().identity(identity);
                }
                CertType::Pkcs12 => {
                    // native-tls backend needed for PKCS12
                    builder = builder.use_native_tls().identity(identity);
                }
            }

            // Add custom CA certificate if configured
            if !entry.ca_path.is_empty() {
                let ca_bytes = std::fs::read(&entry.ca_path)
                    .map_err(|e| format!("Failed to read CA cert '{}': {e}", entry.ca_path))?;
                let ca_cert = reqwest::Certificate::from_pem(&ca_bytes)
                    .map_err(|e| format!("Invalid CA certificate: {e}"))?;
                builder = builder.add_root_certificate(ca_cert);
            }
        }
    }

    let client = builder
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
