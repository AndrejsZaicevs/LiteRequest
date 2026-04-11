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

/// The mutable data of a request at a point in time
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RequestData {
    pub method: HttpMethod,
    pub url: String,
    pub headers: Vec<KeyValuePair>,
    pub query_params: Vec<KeyValuePair>,
    pub body: String,
    pub body_type: BodyType,
}

impl Default for RequestData {
    fn default() -> Self {
        Self {
            method: HttpMethod::GET,
            url: String::new(),
            headers: vec![KeyValuePair::default()],
            query_params: vec![KeyValuePair::default()],
            body: String::new(),
            body_type: BodyType::None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BodyType {
    None,
    Json,
    FormUrlEncoded,
    Raw,
}

impl BodyType {
    pub fn as_str(&self) -> &str {
        match self {
            BodyType::None => "None",
            BodyType::Json => "JSON",
            BodyType::FormUrlEncoded => "Form URL Encoded",
            BodyType::Raw => "Raw",
        }
    }

    pub fn all() -> &'static [BodyType] {
        &[BodyType::None, BodyType::Json, BodyType::FormUrlEncoded, BodyType::Raw]
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
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseData {
    pub status: u16,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub size_bytes: u64,
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
}
