use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    PATCH,
    DELETE,
    HEAD,
    OPTIONS,
}

impl HttpMethod {
    pub fn as_str(&self) -> &str {
        match self {
            HttpMethod::GET => "GET",
            HttpMethod::POST => "POST",
            HttpMethod::PUT => "PUT",
            HttpMethod::PATCH => "PATCH",
            HttpMethod::DELETE => "DELETE",
            HttpMethod::HEAD => "HEAD",
            HttpMethod::OPTIONS => "OPTIONS",
        }
    }

    pub fn all() -> &'static [HttpMethod] {
        &[
            HttpMethod::GET,
            HttpMethod::POST,
            HttpMethod::PUT,
            HttpMethod::PATCH,
            HttpMethod::DELETE,
            HttpMethod::HEAD,
            HttpMethod::OPTIONS,
        ]
    }

    pub fn color(&self) -> [u8; 3] {
        match self {
            HttpMethod::GET => [97, 175, 254],
            HttpMethod::POST => [73, 204, 144],
            HttpMethod::PUT => [252, 161, 48],
            HttpMethod::PATCH => [80, 227, 194],
            HttpMethod::DELETE => [249, 62, 62],
            HttpMethod::HEAD => [144, 119, 255],
            HttpMethod::OPTIONS => [13, 121, 230],
        }
    }
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KeyValuePair {
    pub key: String,
    pub value: String,
    pub enabled: bool,
}

impl Default for KeyValuePair {
    fn default() -> Self {
        Self {
            key: String::new(),
            value: String::new(),
            enabled: true,
        }
    }
}

// ── Client certificate configuration ────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CertType {
    Pem,
    Pkcs12,
}

impl CertType {
    pub fn as_str(&self) -> &str {
        match self {
            CertType::Pem => "PEM (CRT + KEY)",
            CertType::Pkcs12 => "PKCS12 (PFX)",
        }
    }
    pub fn all() -> &'static [CertType] {
        &[CertType::Pem, CertType::Pkcs12]
    }
}

impl Default for CertType {
    fn default() -> Self {
        CertType::Pem
    }
}

/// Per-host client certificate configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClientCertEntry {
    pub enabled: bool,
    /// Host pattern: exact (`api.example.com`) or wildcard (`*.example.com`)
    pub host: String,
    pub cert_type: CertType,
    /// PEM: path to certificate file. PKCS12: path to .pfx/.p12 file.
    pub cert_path: String,
    /// PEM only: path to private key file.
    pub key_path: String,
    /// Optional CA certificate path (PEM format).
    pub ca_path: String,
    /// Passphrase for PKCS12 file or encrypted private key.
    pub passphrase: String,
}

impl Default for ClientCertEntry {
    fn default() -> Self {
        Self {
            enabled: true,
            host: String::new(),
            cert_type: CertType::Pem,
            cert_path: String::new(),
            key_path: String::new(),
            ca_path: String::new(),
            passphrase: String::new(),
        }
    }
}

/// A single field in a multipart/form-data body.
/// Can be either a text value or a file attachment.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MultipartField {
    pub key: String,
    /// Text value (used when `is_file` is false)
    pub value: String,
    pub is_file: bool,
    /// Absolute path to the file (used when `is_file` is true)
    pub file_path: String,
    pub enabled: bool,
}

impl Default for MultipartField {
    fn default() -> Self {
        Self {
            key: String::new(),
            value: String::new(),
            is_file: false,
            file_path: String::new(),
            enabled: true,
        }
    }
}

/// The mutable data of a request at a point in time
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RequestData {
    pub method: HttpMethod,
    pub url: String,
    pub headers: Vec<KeyValuePair>,
    pub query_params: Vec<KeyValuePair>,
    #[serde(default)]
    pub path_params: Vec<KeyValuePair>,
    pub body: String,
    pub body_type: BodyType,
    #[serde(default)]
    pub multipart_fields: Vec<MultipartField>,
}

impl RequestData {
    /// Compute a structural fingerprint that only changes when the
    /// "shape" of the request changes (method, URL template, param/header
    /// keys, body type). Value-only edits (body content, param values,
    /// header values, path-param values) produce the same fingerprint.
    pub fn fingerprint(&self) -> String {
        let mut qp_keys: Vec<&str> = self.query_params.iter()
            .filter(|p| p.enabled && !p.key.is_empty())
            .map(|p| p.key.as_str())
            .collect();
        qp_keys.sort();

        let mut h_keys: Vec<String> = self.headers.iter()
            .filter(|h| h.enabled && !h.key.is_empty())
            .map(|h| h.key.to_lowercase())
            .collect();
        h_keys.sort();

        // For multipart, fingerprint on field keys (like FormUrlEncoded on param keys)
        let mut mp_keys: Vec<&str> = self.multipart_fields.iter()
            .filter(|f| f.enabled && !f.key.is_empty())
            .map(|f| f.key.as_str())
            .collect();
        mp_keys.sort();

        format!(
            "{}|{}|{}|{}|{:?}|{}",
            self.method.as_str(),
            self.url,
            qp_keys.join(","),
            h_keys.join(","),
            self.body_type,
            mp_keys.join(","),
        )
    }
}

impl Default for RequestData {
    fn default() -> Self {
        Self {
            method: HttpMethod::GET,
            url: String::new(),
            headers: vec![KeyValuePair::default()],
            query_params: vec![KeyValuePair::default()],
            path_params: Vec::new(),
            body: String::new(),
            body_type: BodyType::None,
            multipart_fields: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BodyType {
    None,
    Json,
    FormUrlEncoded,
    Raw,
    Multipart,
}

impl BodyType {
    pub fn as_str(&self) -> &str {
        match self {
            BodyType::None => "None",
            BodyType::Json => "JSON",
            BodyType::FormUrlEncoded => "Form URL Encoded",
            BodyType::Raw => "Raw",
            BodyType::Multipart => "Multipart",
        }
    }

    pub fn all() -> &'static [BodyType] {
        &[BodyType::None, BodyType::Json, BodyType::FormUrlEncoded, BodyType::Raw, BodyType::Multipart]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub id: String,
    pub collection_id: String,
    pub folder_id: Option<String>,
    pub name: String,
    pub current_version_id: Option<String>,
    pub sort_order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestVersion {
    pub id: String,
    pub request_id: String,
    pub data: RequestData,
    pub fingerprint: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseData {
    pub status: u16,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub size_bytes: u64,
    /// True when the response body is base64-encoded binary data
    #[serde(default)]
    pub is_binary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestExecution {
    pub id: String,
    pub version_id: String,
    pub request_id: String,
    pub environment_id: String,
    pub response: ResponseData,
    pub latency_ms: u64,
    pub executed_at: String,
    /// Snapshot of the request data at send time (None for legacy executions)
    #[serde(default)]
    pub request_data: Option<RequestData>,
    /// Operative variable values used at send time (None for legacy executions)
    #[serde(default)]
    pub operative_variables: Option<HashMap<String, String>>,
}

/// A single hit returned by the full-text search across all stored data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit {
    /// "request" | "version" | "execution"
    pub result_type: String,
    pub request_id: String,
    pub request_name: String,
    pub collection_id: String,
    pub collection_name: String,
    pub version_id: Option<String>,
    pub execution_id: Option<String>,
    /// What field matched: "Name" | "URL" | "Header" | "Query Param" | "Path Param" | "Body" | "Status" | "Response Header" | "Response Body"
    pub match_field: String,
    /// Short context snippet showing what matched
    pub match_context: String,
    pub method: Option<String>,
    pub url: Option<String>,
    pub executed_at: Option<String>,
    pub status: Option<u16>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DbStats {
    pub db_size_bytes: i64,
    pub version_count: i64,
    pub execution_count: i64,
    pub oldest_execution: Option<String>,
    pub oldest_version: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CleanupResult {
    pub versions_deleted: usize,
    pub executions_deleted: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_request(method: HttpMethod, url: &str) -> RequestData {
        RequestData {
            method,
            url: url.to_string(),
            ..RequestData::default()
        }
    }

    #[test]
    fn test_fingerprint_basic() {
        let data = make_request(HttpMethod::GET, "https://example.com/api");
        let fp = data.fingerprint();
        assert!(fp.starts_with("GET|"));
        assert!(fp.contains("https://example.com/api"));
    }

    #[test]
    fn test_fingerprint_stable_for_value_changes() {
        let mut a = make_request(HttpMethod::POST, "/users");
        a.body = r#"{"name": "Alice"}"#.to_string();
        a.body_type = BodyType::Json;
        a.headers = vec![KeyValuePair { key: "Auth".to_string(), value: "Bearer aaa".to_string(), enabled: true }];

        let mut b = a.clone();
        b.body = r#"{"name": "Bob"}"#.to_string();
        b.headers[0].value = "Bearer bbb".to_string();

        assert_eq!(a.fingerprint(), b.fingerprint(), "Value-only changes should not change fingerprint");
    }

    #[test]
    fn test_fingerprint_changes_on_method() {
        let a = make_request(HttpMethod::GET, "/users");
        let b = make_request(HttpMethod::POST, "/users");
        assert_ne!(a.fingerprint(), b.fingerprint());
    }

    #[test]
    fn test_fingerprint_changes_on_url() {
        let a = make_request(HttpMethod::GET, "/users");
        let b = make_request(HttpMethod::GET, "/posts");
        assert_ne!(a.fingerprint(), b.fingerprint());
    }

    #[test]
    fn test_fingerprint_changes_on_new_header_key() {
        let mut a = make_request(HttpMethod::GET, "/api");
        a.headers = vec![KeyValuePair { key: "Accept".to_string(), value: "application/json".to_string(), enabled: true }];

        let mut b = a.clone();
        b.headers.push(KeyValuePair { key: "X-Custom".to_string(), value: "yes".to_string(), enabled: true });

        assert_ne!(a.fingerprint(), b.fingerprint());
    }

    #[test]
    fn test_fingerprint_changes_on_new_query_param_key() {
        let mut a = make_request(HttpMethod::GET, "/api");
        a.query_params = vec![KeyValuePair { key: "page".to_string(), value: "1".to_string(), enabled: true }];

        let mut b = a.clone();
        b.query_params.push(KeyValuePair { key: "limit".to_string(), value: "10".to_string(), enabled: true });

        assert_ne!(a.fingerprint(), b.fingerprint());
    }

    #[test]
    fn test_fingerprint_changes_on_body_type() {
        let mut a = make_request(HttpMethod::POST, "/api");
        a.body_type = BodyType::Json;

        let mut b = a.clone();
        b.body_type = BodyType::Raw;

        assert_ne!(a.fingerprint(), b.fingerprint());
    }

    #[test]
    fn test_fingerprint_ignores_disabled_params() {
        let mut a = make_request(HttpMethod::GET, "/api");
        a.query_params = vec![KeyValuePair { key: "page".to_string(), value: "1".to_string(), enabled: true }];

        let mut b = a.clone();
        b.query_params.push(KeyValuePair { key: "ignored".to_string(), value: "yes".to_string(), enabled: false });

        assert_eq!(a.fingerprint(), b.fingerprint(), "Disabled params should not affect fingerprint");
    }

    #[test]
    fn test_fingerprint_ignores_empty_key_params() {
        let mut a = make_request(HttpMethod::GET, "/api");
        a.query_params = vec![KeyValuePair { key: "page".to_string(), value: "1".to_string(), enabled: true }];

        let mut b = a.clone();
        b.query_params.push(KeyValuePair { key: "".to_string(), value: "ghost".to_string(), enabled: true });

        assert_eq!(a.fingerprint(), b.fingerprint(), "Empty-key params should not affect fingerprint");
    }

    #[test]
    fn test_fingerprint_header_keys_are_case_insensitive() {
        let mut a = make_request(HttpMethod::GET, "/api");
        a.headers = vec![KeyValuePair { key: "Content-Type".to_string(), value: "text/plain".to_string(), enabled: true }];

        let mut b = make_request(HttpMethod::GET, "/api");
        b.headers = vec![KeyValuePair { key: "content-type".to_string(), value: "application/json".to_string(), enabled: true }];

        assert_eq!(a.fingerprint(), b.fingerprint(), "Header key case should not affect fingerprint");
    }

    #[test]
    fn test_fingerprint_query_param_order_irrelevant() {
        let mut a = make_request(HttpMethod::GET, "/api");
        a.query_params = vec![
            KeyValuePair { key: "a".to_string(), value: "1".to_string(), enabled: true },
            KeyValuePair { key: "b".to_string(), value: "2".to_string(), enabled: true },
        ];

        let mut b = make_request(HttpMethod::GET, "/api");
        b.query_params = vec![
            KeyValuePair { key: "b".to_string(), value: "2".to_string(), enabled: true },
            KeyValuePair { key: "a".to_string(), value: "1".to_string(), enabled: true },
        ];

        assert_eq!(a.fingerprint(), b.fingerprint(), "Param order should not affect fingerprint");
    }

    #[test]
    fn test_fingerprint_multipart_keys() {
        let mut a = make_request(HttpMethod::POST, "/upload");
        a.body_type = BodyType::Multipart;
        a.multipart_fields = vec![MultipartField { key: "file".to_string(), ..MultipartField::default() }];

        let mut b = a.clone();
        b.multipart_fields.push(MultipartField { key: "desc".to_string(), ..MultipartField::default() });

        assert_ne!(a.fingerprint(), b.fingerprint());
    }

    #[test]
    fn test_default_request_data() {
        let d = RequestData::default();
        assert_eq!(d.method, HttpMethod::GET);
        assert!(d.url.is_empty());
        assert_eq!(d.body_type, BodyType::None);
        assert!(d.body.is_empty());
        assert!(d.path_params.is_empty());
        assert!(d.multipart_fields.is_empty());
    }

    #[test]
    fn test_http_method_as_str_roundtrip() {
        for method in HttpMethod::all() {
            let s = method.as_str();
            assert!(!s.is_empty());
            assert_eq!(format!("{}", method), s);
        }
    }

    #[test]
    fn test_body_type_as_str() {
        assert_eq!(BodyType::None.as_str(), "None");
        assert_eq!(BodyType::Json.as_str(), "JSON");
        assert_eq!(BodyType::FormUrlEncoded.as_str(), "Form URL Encoded");
        assert_eq!(BodyType::Raw.as_str(), "Raw");
        assert_eq!(BodyType::Multipart.as_str(), "Multipart");
    }
}
