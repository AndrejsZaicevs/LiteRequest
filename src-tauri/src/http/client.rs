use crate::models::*;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;

/// Bodies larger than this are cut off. The whole body travels through IPC
/// as a JSON string and is held in the webview, so it has to stay bounded.
pub const MAX_BODY_BYTES: usize = 50 * 1024 * 1024;

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
    // Replace :paramName segments with path_params values
    let path_param_pairs: Vec<(String, String)> = data.path_params
        .iter()
        .filter(|p| p.enabled && !p.key.is_empty())
        .map(|p| (p.key.clone(), super::interpolation::interpolate(&p.value, variables)))
        .collect();
    let url = super::interpolation::resolve_path_params(&url, &path_param_pairs);

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
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {e}"))?;

    let builder = build_request(&client, data, variables, &url)?;

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

    let mut body_bytes: Vec<u8> = Vec::new();
    let mut truncated = false;
    let mut response = response;
    while let Some(chunk) = response.chunk().await.map_err(|e| format!("Failed to read body: {e}"))? {
        let room = MAX_BODY_BYTES - body_bytes.len();
        if chunk.len() > room {
            body_bytes.extend_from_slice(&chunk[..room]);
            truncated = true;
            break;
        }
        body_bytes.extend_from_slice(&chunk);
    }
    let size_bytes = body_bytes.len() as u64;

    // Detect binary content-type so we can preserve the bytes as base64
    let is_binary = headers.get("content-type")
        .map(|ct| {
            let ct = ct.to_lowercase();
            ct.starts_with("image/")
                || ct.starts_with("audio/")
                || ct.starts_with("video/")
                || ct == "application/octet-stream"
                || ct.starts_with("application/pdf")
                || ct.starts_with("application/zip")
                || ct.starts_with("application/x-tar")
                || ct.starts_with("application/gzip")
                || ct.starts_with("font/")
        })
        .unwrap_or(false);

    let body = if is_binary {
        BASE64.encode(&body_bytes)
    } else {
        String::from_utf8_lossy(&body_bytes).to_string()
    };

    Ok((
        ResponseData {
            status,
            status_text,
            headers,
            body,
            size_bytes,
            is_binary,
            truncated,
        },
        latency_ms,
    ))
}

/// Apply headers, query params and body from `data` to a request for `url`.
/// Split out from `execute_request` so it can be tested without a network.
fn build_request(
    client: &reqwest::Client,
    data: &RequestData,
    variables: &HashMap<String, String>,
    url: &str,
) -> Result<reqwest::RequestBuilder, String> {
    let method = match data.method {
        HttpMethod::GET => reqwest::Method::GET,
        HttpMethod::POST => reqwest::Method::POST,
        HttpMethod::PUT => reqwest::Method::PUT,
        HttpMethod::PATCH => reqwest::Method::PATCH,
        HttpMethod::DELETE => reqwest::Method::DELETE,
        HttpMethod::HEAD => reqwest::Method::HEAD,
        HttpMethod::OPTIONS => reqwest::Method::OPTIONS,
    };

    let mut builder = client.request(method, url);

    // Add headers. Remember whether the user set Content-Type themselves so
    // the body handling below does not append a second one (reqwest's
    // `header()` appends rather than replaces).
    let mut user_content_type = false;
    for h in &data.headers {
        if h.enabled && !h.key.is_empty() {
            let key = super::interpolation::interpolate(&h.key, variables);
            let val = super::interpolation::interpolate(&h.value, variables);
            if key.eq_ignore_ascii_case("content-type") {
                user_content_type = true;
            }
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

    let default_content_type = |b: reqwest::RequestBuilder, ct: &str| {
        if user_content_type { b } else { b.header("Content-Type", ct) }
    };

    // Add body
    match data.body_type {
        BodyType::Json => {
            let body = super::interpolation::interpolate(&data.body, variables);
            builder = default_content_type(builder, "application/json").body(body);
        }
        BodyType::FormUrlEncoded => {
            let body = super::interpolation::interpolate(&data.body, variables);
            builder = default_content_type(builder, "application/x-www-form-urlencoded").body(body);
        }
        BodyType::Raw => {
            let body = super::interpolation::interpolate(&data.body, variables);
            builder = builder.body(body);
        }
        BodyType::Multipart => {
            let mut form = reqwest::multipart::Form::new();
            for field in &data.multipart_fields {
                if !field.enabled || field.key.is_empty() {
                    continue;
                }
                let key = super::interpolation::interpolate(&field.key, variables);
                if field.is_file {
                    if field.file_path.is_empty() {
                        continue;
                    }
                    let bytes = std::fs::read(&field.file_path)
                        .map_err(|e| format!("Failed to read file '{}': {e}", field.file_path))?;
                    let file_name = Path::new(&field.file_path)
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "file".to_string());
                    let mime = mime_guess::from_path(&field.file_path)
                        .first_or_octet_stream()
                        .to_string();
                    let part = reqwest::multipart::Part::bytes(bytes)
                        .file_name(file_name)
                        .mime_str(&mime)
                        .map_err(|e| format!("Invalid MIME type: {e}"))?;
                    form = form.part(key, part);
                } else {
                    let value = super::interpolation::interpolate(&field.value, variables);
                    form = form.text(key, value);
                }
            }
            builder = builder.multipart(form);
        }
        BodyType::None => {}
    }

    Ok(builder)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn json_post(headers: Vec<KeyValuePair>) -> RequestData {
        RequestData {
            method: HttpMethod::POST,
            body_type: BodyType::Json,
            body: "{}".into(),
            headers,
            ..RequestData::default()
        }
    }

    fn content_types(data: &RequestData) -> Vec<String> {
        let client = reqwest::Client::new();
        let req = build_request(&client, data, &HashMap::new(), "https://example.test/x")
            .unwrap()
            .build()
            .unwrap();
        req.headers()
            .get_all("content-type")
            .iter()
            .map(|v| v.to_str().unwrap().to_string())
            .collect()
    }

    #[test]
    fn default_content_type_added_for_json_body() {
        assert_eq!(content_types(&json_post(vec![])), vec!["application/json"]);
    }

    #[test]
    fn user_content_type_is_not_duplicated() {
        let data = json_post(vec![KeyValuePair {
            key: "content-type".into(),
            value: "application/json; charset=utf-8".into(),
            enabled: true,
        }]);
        assert_eq!(content_types(&data), vec!["application/json; charset=utf-8"]);
    }

    #[test]
    fn disabled_content_type_header_does_not_count() {
        let data = json_post(vec![KeyValuePair {
            key: "Content-Type".into(),
            value: "text/plain".into(),
            enabled: false,
        }]);
        assert_eq!(content_types(&data), vec!["application/json"]);
    }

    #[test]
    fn host_pattern_matching() {
        assert!(host_matches("api.example.com", "api.example.com"));
        assert!(host_matches("*.example.com", "api.example.com"));
        assert!(!host_matches("*.example.com", "example.com"));
        assert!(!host_matches("*.example.com", "evilexample.com"));
    }
}
