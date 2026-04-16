use crate::models::*;
use std::collections::HashMap;

/// Convert a RequestData into a cURL command string.
/// `variables` and `base_path` are used to resolve the final URL;
/// pass empty if the raw (unresolved) form is desired.
pub fn to_curl(
    data: &RequestData,
    variables: &HashMap<String, String>,
    base_path: &str,
) -> String {
    let url = super::interpolation::resolve_url(base_path, &data.url, variables);
    let url = super::interpolation::interpolate(&url, variables);

    // Resolve :param segments from path_params
    let path_param_pairs: Vec<(String, String)> = data
        .path_params
        .iter()
        .filter(|p| p.enabled && !p.key.is_empty())
        .map(|p| (p.key.clone(), super::interpolation::interpolate(&p.value, variables)))
        .collect();
    let url = super::interpolation::resolve_path_params(&url, &path_param_pairs);

    let mut parts: Vec<String> = vec!["curl".to_string()];

    // Method (omit for GET, it's the default)
    if data.method != HttpMethod::GET {
        parts.push(format!("-X {}", data.method.as_str()));
    }

    // URL with query params baked in
    let mut final_url = url;
    let qp: Vec<(String, String)> = data
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
    if !qp.is_empty() {
        let sep = if final_url.contains('?') { '&' } else { '?' };
        let qs: Vec<String> = qp
            .iter()
            .map(|(k, v)| {
                if v.is_empty() {
                    url_encode(k)
                } else {
                    format!("{}={}", url_encode(k), url_encode(v))
                }
            })
            .collect();
        final_url = format!("{final_url}{sep}{}", qs.join("&"));
    }
    parts.push(shell_quote(&final_url));

    // Headers
    for h in &data.headers {
        if h.enabled && !h.key.is_empty() {
            let k = super::interpolation::interpolate(&h.key, variables);
            let v = super::interpolation::interpolate(&h.value, variables);
            parts.push(format!("-H {}", shell_quote(&format!("{k}: {v}"))));
        }
    }

    // Body
    match data.body_type {
        BodyType::Json => {
            let body = super::interpolation::interpolate(&data.body, variables);
            // Only add Content-Type if not already in headers
            let has_ct = data
                .headers
                .iter()
                .any(|h| h.enabled && h.key.eq_ignore_ascii_case("content-type"));
            if !has_ct {
                parts.push("-H 'Content-Type: application/json'".to_string());
            }
            if !body.is_empty() {
                parts.push(format!("-d {}", shell_quote(&body)));
            }
        }
        BodyType::FormUrlEncoded => {
            let body = super::interpolation::interpolate(&data.body, variables);
            let has_ct = data
                .headers
                .iter()
                .any(|h| h.enabled && h.key.eq_ignore_ascii_case("content-type"));
            if !has_ct {
                parts.push("-H 'Content-Type: application/x-www-form-urlencoded'".to_string());
            }
            if !body.is_empty() {
                parts.push(format!("-d {}", shell_quote(&body)));
            }
        }
        BodyType::Raw => {
            let body = super::interpolation::interpolate(&data.body, variables);
            if !body.is_empty() {
                parts.push(format!("-d {}", shell_quote(&body)));
            }
        }
        BodyType::Multipart => {
            // Represent multipart as -F fields in the cURL output
            for field in &data.multipart_fields {
                if !field.enabled || field.key.is_empty() {
                    continue;
                }
                let k = shell_quote(&field.key);
                if field.is_file {
                    parts.push(format!("-F {}=@{}", k, shell_quote(&field.file_path)));
                } else {
                    let v = super::interpolation::interpolate(&field.value, variables);
                    parts.push(format!("-F {}={}", k, shell_quote(&v)));
                }
            }
        }
        BodyType::None => {}
    }

    parts.join(" \\\n  ")
}

/// Parse a cURL command string into a `RequestData`.
/// Handles: -X, -H, -d/--data/--data-raw/--data-binary, URL.
pub fn parse_curl(input: &str) -> Result<RequestData, String> {
    let tokens = tokenize(input)?;
    if tokens.is_empty() {
        return Err("Empty input".to_string());
    }

    let mut method: Option<HttpMethod> = None;
    let mut url: Option<String> = None;
    let mut headers: Vec<KeyValuePair> = Vec::new();
    let mut body = String::new();
    let mut body_type = BodyType::None;

    let mut i = 0;
    while i < tokens.len() {
        let tok = &tokens[i];
        match tok.as_str() {
            "curl" => {
                i += 1;
                continue;
            }
            "-X" | "--request" => {
                i += 1;
                if i < tokens.len() {
                    method = Some(parse_method(&tokens[i])?);
                }
            }
            "-H" | "--header" => {
                i += 1;
                if i < tokens.len() {
                    if let Some((k, v)) = tokens[i].split_once(':') {
                        headers.push(KeyValuePair {
                            key: k.trim().to_string(),
                            value: v.trim().to_string(),
                            enabled: true,
                        });
                    }
                }
            }
            "-d" | "--data" | "--data-raw" | "--data-binary" | "--data-ascii" => {
                i += 1;
                if i < tokens.len() {
                    body = tokens[i].clone();
                }
            }
            "--compressed" | "-s" | "--silent" | "-S" | "--show-error" | "-k"
            | "--insecure" | "-L" | "--location" | "-v" | "--verbose" | "-i"
            | "--include" => {
                // skip standalone flags
            }
            _ => {
                // Not a recognized flag — treat as URL if we don't have one yet
                if !tok.starts_with('-') && url.is_none() {
                    url = Some(tok.clone());
                }
            }
        }
        i += 1;
    }

    let url = url.ok_or("No URL found in cURL command")?;

    // Parse query params from the URL
    let (base_url, query_params) = extract_query_params(&url);

    // Determine body type from Content-Type header
    if !body.is_empty() {
        let ct = headers
            .iter()
            .find(|h| h.key.eq_ignore_ascii_case("content-type"))
            .map(|h| h.value.to_lowercase());
        body_type = match ct.as_deref() {
            Some(ct) if ct.contains("application/json") => BodyType::Json,
            Some(ct) if ct.contains("x-www-form-urlencoded") => BodyType::FormUrlEncoded,
            _ => {
                // If body looks like JSON, assume JSON
                if body.trim_start().starts_with('{') || body.trim_start().starts_with('[') {
                    BodyType::Json
                } else {
                    BodyType::Raw
                }
            }
        };
    }

    // Infer method if not explicitly set
    let method = method.unwrap_or(if body.is_empty() {
        HttpMethod::GET
    } else {
        HttpMethod::POST
    });

    // Remove Content-Type from headers (the body_type will set it)
    let headers: Vec<KeyValuePair> = headers
        .into_iter()
        .filter(|h| !h.key.eq_ignore_ascii_case("content-type"))
        .collect();

    // Ensure at least one empty row for headers and query_params
    let headers = if headers.is_empty() {
        vec![KeyValuePair::default()]
    } else {
        headers
    };
    let query_params = if query_params.is_empty() {
        vec![KeyValuePair::default()]
    } else {
        query_params
    };

    Ok(RequestData {
        method,
        url: base_url,
        headers,
        query_params,
        path_params: Vec::new(),
        body,
        body_type,
        multipart_fields: Vec::new(),
    })
}

// ── helpers ──────────────────────────────────────────────────────

fn parse_method(s: &str) -> Result<HttpMethod, String> {
    match s.to_uppercase().as_str() {
        "GET" => Ok(HttpMethod::GET),
        "POST" => Ok(HttpMethod::POST),
        "PUT" => Ok(HttpMethod::PUT),
        "PATCH" => Ok(HttpMethod::PATCH),
        "DELETE" => Ok(HttpMethod::DELETE),
        "HEAD" => Ok(HttpMethod::HEAD),
        "OPTIONS" => Ok(HttpMethod::OPTIONS),
        _ => Err(format!("Unknown HTTP method: {s}")),
    }
}

fn extract_query_params(url: &str) -> (String, Vec<KeyValuePair>) {
    let Some(q_start) = url.find('?') else {
        return (url.to_string(), Vec::new());
    };
    let base = url[..q_start].to_string();
    let query = &url[q_start + 1..];
    let params: Vec<KeyValuePair> = query
        .split('&')
        .filter(|s| !s.is_empty())
        .map(|pair| {
            let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
            KeyValuePair {
                key: url_decode(k),
                value: url_decode(v),
                enabled: true,
            }
        })
        .collect();
    (base, params)
}

/// Minimal URL decoding for query parameter values.
fn url_decode(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            if hex.len() == 2 {
                if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                    result.push(byte as char);
                    continue;
                }
            }
            result.push('%');
            result.push_str(&hex);
        } else if c == '+' {
            result.push(' ');
        } else {
            result.push(c);
        }
    }
    result
}

/// Minimal URL encoding for query string keys/values.
fn url_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => {
                out.push_str(&format!("%{:02X}", b));
            }
        }
    }
    out
}

/// Shell-quote a string with single quotes, escaping internal single quotes.
fn shell_quote(s: &str) -> String {
    if s.contains('\'') {
        // Replace ' with '\'' (end quote, escaped quote, start quote)
        format!("'{}'", s.replace('\'', "'\\''"))
    } else {
        format!("'{s}'")
    }
}

/// Tokenize a cURL command, respecting single/double quotes and backslash line
/// continuations. Returns the list of tokens.
fn tokenize(input: &str) -> Result<Vec<String>, String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut chars = input.chars().peekable();
    let mut in_single_quote = false;
    let mut in_double_quote = false;

    while let Some(c) = chars.next() {
        if in_single_quote {
            if c == '\'' {
                in_single_quote = false;
            } else {
                current.push(c);
            }
            continue;
        }
        if in_double_quote {
            if c == '\\' {
                // In double quotes, backslash only escapes certain chars
                if let Some(&next) = chars.peek() {
                    if matches!(next, '"' | '\\' | '$' | '`') {
                        current.push(chars.next().unwrap());
                        continue;
                    }
                }
                current.push(c);
            } else if c == '"' {
                in_double_quote = false;
            } else {
                current.push(c);
            }
            continue;
        }
        match c {
            '\'' => in_single_quote = true,
            '"' => in_double_quote = true,
            '\\' => {
                // Line continuation or escaped char
                if let Some(&next) = chars.peek() {
                    if next == '\n' {
                        chars.next(); // skip newline
                    } else {
                        current.push(chars.next().unwrap());
                    }
                }
            }
            ' ' | '\t' | '\n' | '\r' => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(c),
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_get() {
        let data = parse_curl("curl https://api.example.com/users").unwrap();
        assert_eq!(data.method, HttpMethod::GET);
        assert_eq!(data.url, "https://api.example.com/users");
        assert!(data.body.is_empty());
    }

    #[test]
    fn test_post_json() {
        let input = r#"curl -X POST https://api.example.com/users \
  -H 'Content-Type: application/json' \
  -d '{"name": "John"}'
"#;
        let data = parse_curl(input).unwrap();
        assert_eq!(data.method, HttpMethod::POST);
        assert_eq!(data.body_type, BodyType::Json);
        assert_eq!(data.body, r#"{"name": "John"}"#);
    }

    #[test]
    fn test_query_params_extracted() {
        let data = parse_curl("curl 'https://api.example.com/users?page=1&limit=10'").unwrap();
        assert_eq!(data.url, "https://api.example.com/users");
        assert_eq!(data.query_params.len(), 2);
        assert_eq!(data.query_params[0].key, "page");
        assert_eq!(data.query_params[0].value, "1");
    }

    #[test]
    fn test_headers_preserved() {
        let input = r#"curl https://api.example.com -H 'Authorization: Bearer tok123' -H 'Accept: application/json'"#;
        let data = parse_curl(input).unwrap();
        assert_eq!(data.headers.len(), 2);
        assert_eq!(data.headers[0].key, "Authorization");
        assert_eq!(data.headers[0].value, "Bearer tok123");
    }

    #[test]
    fn test_roundtrip() {
        let data = RequestData {
            method: HttpMethod::POST,
            url: "https://api.example.com/data".to_string(),
            headers: vec![KeyValuePair {
                key: "Authorization".to_string(),
                value: "Bearer xyz".to_string(),
                enabled: true,
            }],
            query_params: vec![KeyValuePair {
                key: "page".to_string(),
                value: "1".to_string(),
                enabled: true,
            }],
            path_params: Vec::new(),
            body: r#"{"test": true}"#.to_string(),
            body_type: BodyType::Json,
        };
        let curl = to_curl(&data, &HashMap::new(), "");
        let parsed = parse_curl(&curl).unwrap();
        assert_eq!(parsed.method, HttpMethod::POST);
        assert_eq!(parsed.body_type, BodyType::Json);
        assert!(parsed.body.contains(r#""test": true"#));
    }
}
