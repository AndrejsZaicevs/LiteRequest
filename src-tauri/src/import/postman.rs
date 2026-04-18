use crate::db::Database;
use crate::models::*;
use serde::{Deserialize, Serialize};

// ── Postman v2.1 collection format (minimal structs) ─────────────

#[derive(Deserialize)]
pub struct PostmanCollection {
    pub info: PostmanInfo,
    pub item: Option<Vec<PostmanItem>>,
}

#[derive(Deserialize)]
pub struct PostmanInfo {
    pub name: String,
}

#[derive(Deserialize)]
pub struct PostmanItem {
    pub name: Option<String>,
    /// Presence of `item` means this is a folder
    pub item: Option<Vec<PostmanItem>>,
    pub request: Option<PostmanRequest>,
}

#[derive(Deserialize)]
pub struct PostmanRequest {
    pub method: Option<String>,
    pub header: Option<Vec<PostmanHeader>>,
    pub url: Option<PostmanUrlField>,
    pub body: Option<PostmanBody>,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum PostmanUrlField {
    String(String),
    Object(PostmanUrlObject),
}

#[derive(Deserialize)]
pub struct PostmanUrlObject {
    pub raw: Option<String>,
    pub query: Option<Vec<PostmanParam>>,
}

#[derive(Deserialize)]
pub struct PostmanHeader {
    pub key: String,
    pub value: String,
    #[serde(default)]
    pub disabled: bool,
}

#[derive(Deserialize)]
pub struct PostmanParam {
    pub key: Option<String>,
    pub value: Option<String>,
    #[serde(default)]
    pub disabled: bool,
}

#[derive(Deserialize)]
pub struct PostmanBody {
    pub mode: Option<String>,
    pub raw: Option<String>,
    pub urlencoded: Option<Vec<PostmanFormField>>,
    pub formdata: Option<Vec<PostmanFormData>>,
    pub options: Option<PostmanBodyOptions>,
}

#[derive(Deserialize)]
pub struct PostmanFormField {
    pub key: Option<String>,
    pub value: Option<String>,
    #[serde(default)]
    pub disabled: bool,
}

#[derive(Deserialize)]
pub struct PostmanFormData {
    pub key: Option<String>,
    pub value: Option<String>,
    #[serde(rename = "type")]
    pub field_type: Option<String>,
    pub src: Option<String>,
    #[serde(default)]
    pub disabled: bool,
}

#[derive(Deserialize)]
pub struct PostmanBodyOptions {
    pub raw: Option<PostmanRawOptions>,
}

#[derive(Deserialize)]
pub struct PostmanRawOptions {
    pub language: Option<String>,
}

// ── Import result ─────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct ImportSummary {
    pub collection_name: String,
    pub folders: usize,
    pub requests: usize,
}

// ── Export (Postman v2.1 output) ──────────────────────────────

#[derive(Serialize)]
struct ExportCollection {
    info: ExportInfo,
    item: Vec<ExportItem>,
}

#[derive(Serialize)]
struct ExportInfo {
    #[serde(rename = "_postman_id")]
    postman_id: String,
    name: String,
    schema: &'static str,
}

#[derive(Serialize)]
#[serde(untagged)]
enum ExportItem {
    Folder(ExportFolder),
    Request(ExportRequest),
}

#[derive(Serialize)]
struct ExportFolder {
    name: String,
    item: Vec<ExportItem>,
}

#[derive(Serialize)]
struct ExportRequest {
    name: String,
    request: ExportRequestBody,
}

#[derive(Serialize)]
struct ExportRequestBody {
    method: String,
    header: Vec<ExportHeader>,
    url: ExportUrl,
    #[serde(skip_serializing_if = "Option::is_none")]
    body: Option<ExportBody>,
}

#[derive(Serialize)]
struct ExportHeader {
    key: String,
    value: String,
}

#[derive(Serialize)]
struct ExportUrl {
    raw: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    query: Vec<ExportParam>,
}

#[derive(Serialize)]
struct ExportParam {
    key: String,
    value: String,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    disabled: bool,
}

#[derive(Serialize)]
struct ExportBody {
    mode: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    raw: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<ExportBodyOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    urlencoded: Option<Vec<ExportFormField>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    formdata: Option<Vec<ExportFormData>>,
}

#[derive(Serialize)]
struct ExportBodyOptions {
    raw: ExportRawOptions,
}

#[derive(Serialize)]
struct ExportRawOptions {
    language: String,
}

#[derive(Serialize)]
struct ExportFormField {
    key: String,
    value: String,
}

#[derive(Serialize)]
struct ExportFormData {
    key: String,
    #[serde(rename = "type")]
    field_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    src: Option<String>,
}

pub fn export_collection(collection_id: &str, db: &Database) -> Result<String, String> {
    let collections = db.list_collections().map_err(|e| e.to_string())?;
    let collection = collections
        .into_iter()
        .find(|c| c.id == collection_id)
        .ok_or_else(|| format!("Collection {collection_id} not found"))?;

    let all_folders = db
        .list_folders_by_collection(collection_id)
        .map_err(|e| e.to_string())?;
    let all_requests = db
        .list_requests_by_collection(collection_id)
        .map_err(|e| e.to_string())?;

    // Resolve all current versions upfront
    let mut versions: std::collections::HashMap<String, RequestData> =
        std::collections::HashMap::new();
    for req in &all_requests {
        if let Some(vid) = &req.current_version_id {
            if let Ok(v) = db.get_version(vid) {
                versions.insert(req.id.clone(), v.data);
            }
        }
    }

    let items = build_export_items(
        None,
        &all_folders,
        &all_requests,
        &versions,
        &collection.base_path,
    );

    let export = ExportCollection {
        info: ExportInfo {
            postman_id: collection_id.to_string(),
            name: collection.name.clone(),
            schema: "https://schema.getpostman.com/json/collection/v2.1.0/collection.json",
        },
        item: items,
    };

    serde_json::to_string_pretty(&export).map_err(|e| e.to_string())
}

fn build_export_items(
    parent_folder_id: Option<&str>,
    all_folders: &[Folder],
    all_requests: &[Request],
    versions: &std::collections::HashMap<String, RequestData>,
    base_path: &str,
) -> Vec<ExportItem> {
    let mut items: Vec<ExportItem> = Vec::new();

    // Folders at this level
    for folder in all_folders.iter().filter(|f| f.parent_folder_id.as_deref() == parent_folder_id) {
        let sub_items = build_export_items(Some(&folder.id), all_folders, all_requests, versions, base_path);
        items.push(ExportItem::Folder(ExportFolder {
            name: folder.name.clone(),
            item: sub_items,
        }));
    }

    // Requests at this level
    for req in all_requests.iter().filter(|r| r.folder_id.as_deref() == parent_folder_id) {
        let data = versions.get(&req.id).cloned().unwrap_or_default();
        items.push(ExportItem::Request(export_request(&req.name, &data, base_path)));
    }

    items
}

fn export_request(name: &str, data: &RequestData, base_path: &str) -> ExportRequest {
    let raw_url = if base_path.is_empty() {
        data.url.clone()
    } else {
        format!("{}{}", base_path.trim_end_matches('/'), data.url)
    };

    let mut raw_with_query = raw_url.clone();
    let enabled_params: Vec<&KeyValuePair> = data
        .query_params
        .iter()
        .filter(|p| p.enabled && !p.key.is_empty())
        .collect();
    if !enabled_params.is_empty() {
        let qs: String = enabled_params
            .iter()
            .map(|p| format!("{}={}", p.key, p.value))
            .collect::<Vec<_>>()
            .join("&");
        raw_with_query = format!("{}?{}", raw_url, qs);
    }

    let query: Vec<ExportParam> = data
        .query_params
        .iter()
        .filter(|p| !p.key.is_empty())
        .map(|p| ExportParam {
            key: p.key.clone(),
            value: p.value.clone(),
            disabled: !p.enabled,
        })
        .collect();

    let header: Vec<ExportHeader> = data
        .headers
        .iter()
        .filter(|h| h.enabled && !h.key.is_empty())
        .map(|h| ExportHeader { key: h.key.clone(), value: h.value.clone() })
        .collect();

    let body = export_body(data);

    ExportRequest {
        name: name.to_string(),
        request: ExportRequestBody {
            method: data.method.as_str().to_string(),
            header,
            url: ExportUrl { raw: raw_with_query, query },
            body,
        },
    }
}

fn export_body(data: &RequestData) -> Option<ExportBody> {
    match &data.body_type {
        BodyType::None => None,
        BodyType::Json => Some(ExportBody {
            mode: "raw".to_string(),
            raw: Some(data.body.clone()),
            options: Some(ExportBodyOptions {
                raw: ExportRawOptions { language: "json".to_string() },
            }),
            urlencoded: None,
            formdata: None,
        }),
        BodyType::Raw => Some(ExportBody {
            mode: "raw".to_string(),
            raw: Some(data.body.clone()),
            options: Some(ExportBodyOptions {
                raw: ExportRawOptions { language: "text".to_string() },
            }),
            urlencoded: None,
            formdata: None,
        }),
        BodyType::FormUrlEncoded => {
            let fields: Vec<ExportFormField> = data
                .body
                .split('&')
                .filter(|p| !p.is_empty())
                .map(|pair| {
                    if let Some(eq) = pair.find('=') {
                        ExportFormField { key: pair[..eq].to_string(), value: pair[eq + 1..].to_string() }
                    } else {
                        ExportFormField { key: pair.to_string(), value: String::new() }
                    }
                })
                .collect();
            Some(ExportBody {
                mode: "urlencoded".to_string(),
                raw: None,
                options: None,
                urlencoded: Some(fields),
                formdata: None,
            })
        }
        BodyType::Multipart => {
            let fields: Vec<ExportFormData> = data
                .multipart_fields
                .iter()
                .filter(|f| f.enabled && !f.key.is_empty())
                .map(|f| {
                    if f.is_file {
                        ExportFormData {
                            key: f.key.clone(),
                            field_type: "file".to_string(),
                            value: None,
                            src: Some(f.file_path.clone()),
                        }
                    } else {
                        ExportFormData {
                            key: f.key.clone(),
                            field_type: "text".to_string(),
                            value: Some(f.value.clone()),
                            src: None,
                        }
                    }
                })
                .collect();
            Some(ExportBody {
                mode: "formdata".to_string(),
                raw: None,
                options: None,
                urlencoded: None,
                formdata: Some(fields),
            })
        }
    }
}

// ── Main entry point ──────────────────────────────────────────

pub fn import_from_path(path: &str, db: &Database) -> Result<ImportSummary, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file: {e}"))?;
    import_from_json(&content, db)
}

pub fn import_from_json(json: &str, db: &Database) -> Result<ImportSummary, String> {
    let col: PostmanCollection =
        serde_json::from_str(json).map_err(|e| format!("Invalid Postman JSON: {e}"))?;

    let now = chrono::Utc::now().to_rfc3339();
    let collection_id = uuid::Uuid::new_v4().to_string();

    let collection = Collection {
        id: collection_id.clone(),
        name: col.info.name.clone(),
        base_path: String::new(),
        auth_config: None,
        headers_config: None,
        created_at: now.clone(),
        updated_at: now.clone(),
    };
    db.insert_collection(&collection)
        .map_err(|e| format!("DB error: {e}"))?;

    let mut counters = (0usize, 0usize); // (folders, requests)

    walk_items(
        db,
        col.item.as_deref().unwrap_or(&[]),
        &collection_id,
        None,
        &now,
        &mut counters,
    )?;

    Ok(ImportSummary {
        collection_name: col.info.name,
        folders: counters.0,
        requests: counters.1,
    })
}

// ── Recursive item walker ─────────────────────────────────────

fn walk_items(
    db: &Database,
    items: &[PostmanItem],
    collection_id: &str,
    parent_folder_id: Option<&str>,
    now: &str,
    counters: &mut (usize, usize),
) -> Result<(), String> {
    for (i, item) in items.iter().enumerate() {
        let name = item.name.as_deref().unwrap_or("Unnamed").to_string();

        if let Some(sub_items) = &item.item {
            // Folder
            let folder_id = uuid::Uuid::new_v4().to_string();
            let folder = Folder {
                id: folder_id.clone(),
                collection_id: collection_id.to_string(),
                parent_folder_id: parent_folder_id.map(String::from),
                name,
                path_prefix: String::new(),
                auth_override: None,
                sort_order: i as i32,
            };
            db.insert_folder(&folder).map_err(|e| format!("Failed to insert folder: {e}"))?;
            counters.0 += 1;
            walk_items(db, sub_items, collection_id, Some(&folder_id), now, counters)?;
        } else if let Some(req) = &item.request {
            // Request
            let request_id = uuid::Uuid::new_v4().to_string();
            let version_id = uuid::Uuid::new_v4().to_string();

            let data = map_request_data(req);
            let fingerprint = data.fingerprint();

            let request = Request {
                id: request_id.clone(),
                collection_id: collection_id.to_string(),
                folder_id: parent_folder_id.map(String::from),
                name,
                current_version_id: None, // set by insert_version below
                sort_order: i as i32,
            };
            let version = RequestVersion {
                id: version_id,
                request_id: request_id.clone(),
                data,
                fingerprint,
                created_at: now.to_string(),
            };

            // Insert request first (FK: request_versions.request_id → requests.id)
            db.insert_request(&request).map_err(|e| format!("Failed to insert request: {e}"))?;
            // insert_version also runs UPDATE requests SET current_version_id=...
            db.insert_version(&version).map_err(|e| format!("Failed to insert version: {e}"))?;
            counters.1 += 1;
        }
    }
    Ok(())
}

// ── Request data mapper ───────────────────────────────────────

fn map_request_data(req: &PostmanRequest) -> RequestData {
    let method = parse_method(req.method.as_deref().unwrap_or("GET"));

    let (url, query_params) = match &req.url {
        None => (String::new(), vec![]),
        Some(PostmanUrlField::String(s)) => parse_url_string(s),
        Some(PostmanUrlField::Object(obj)) => {
            let raw = obj.raw.as_deref().unwrap_or("").to_string();
            let (base, _) = parse_url_string(&raw);
            let params: Vec<KeyValuePair> = obj
                .query
                .as_deref()
                .unwrap_or(&[])
                .iter()
                .filter(|q| !q.disabled && q.key.as_deref().unwrap_or("") != "")
                .map(|q| KeyValuePair {
                    key: q.key.clone().unwrap_or_default(),
                    value: q.value.clone().unwrap_or_default(),
                    enabled: true,
                })
                .collect();
            (base, params)
        }
    };

    let headers: Vec<KeyValuePair> = req
        .header
        .as_deref()
        .unwrap_or(&[])
        .iter()
        .filter(|h| !h.disabled && !h.key.is_empty())
        .map(|h| KeyValuePair {
            key: h.key.clone(),
            value: h.value.clone(),
            enabled: true,
        })
        .collect();

    let (body_type, body, multipart_fields) = match &req.body {
        None => (BodyType::None, String::new(), vec![]),
        Some(b) => map_body(b),
    };

    RequestData {
        method,
        url,
        query_params,
        headers,
        body_type,
        body,
        path_params: vec![],
        multipart_fields,
    }
}

fn parse_method(s: &str) -> HttpMethod {
    match s.to_uppercase().as_str() {
        "GET" => HttpMethod::GET,
        "POST" => HttpMethod::POST,
        "PUT" => HttpMethod::PUT,
        "PATCH" => HttpMethod::PATCH,
        "DELETE" => HttpMethod::DELETE,
        "HEAD" => HttpMethod::HEAD,
        "OPTIONS" => HttpMethod::OPTIONS,
        _ => HttpMethod::GET,
    }
}

/// Split URL into base (no query string) + query params extracted from the string.
fn parse_url_string(raw: &str) -> (String, Vec<KeyValuePair>) {
    if let Some(idx) = raw.find('?') {
        let base = raw[..idx].to_string();
        let query = &raw[idx + 1..];
        let params = query
            .split('&')
            .filter(|s| !s.is_empty())
            .map(|pair| {
                if let Some(eq) = pair.find('=') {
                    KeyValuePair {
                        key: pair[..eq].to_string(),
                        value: pair[eq + 1..].to_string(),
                        enabled: true,
                    }
                } else {
                    KeyValuePair {
                        key: pair.to_string(),
                        value: String::new(),
                        enabled: true,
                    }
                }
            })
            .collect();
        (base, params)
    } else {
        (raw.to_string(), vec![])
    }
}

fn map_body(body: &PostmanBody) -> (BodyType, String, Vec<MultipartField>) {
    match body.mode.as_deref().unwrap_or("") {
        "raw" => {
            let text = body.raw.as_deref().unwrap_or("");
            // Detect JSON: explicit language hint OR successful parse
            let explicit_json = body
                .options
                .as_ref()
                .and_then(|o| o.raw.as_ref())
                .and_then(|r| r.language.as_ref())
                .map(|l| l.eq_ignore_ascii_case("json"))
                .unwrap_or(false);

            if explicit_json || (!text.trim().is_empty() && serde_json::from_str::<serde_json::Value>(text).is_ok()) {
                (BodyType::Json, text.to_string(), vec![])
            } else {
                (BodyType::Raw, text.to_string(), vec![])
            }
        }
        "urlencoded" => {
            let pairs: Vec<KeyValuePair> = body
                .urlencoded
                .as_deref()
                .unwrap_or(&[])
                .iter()
                .filter(|f| !f.disabled && f.key.as_deref().unwrap_or("") != "")
                .map(|f| KeyValuePair {
                    key: f.key.clone().unwrap_or_default(),
                    value: f.value.clone().unwrap_or_default(),
                    enabled: true,
                })
                .collect();
            let encoded = pairs
                .iter()
                .map(|p| format!("{}={}", p.key, p.value))
                .collect::<Vec<_>>()
                .join("&");
            (BodyType::FormUrlEncoded, encoded, vec![])
        }
        "formdata" => {
            let fields: Vec<MultipartField> = body
                .formdata
                .as_deref()
                .unwrap_or(&[])
                .iter()
                .filter(|f| !f.disabled && f.key.as_deref().unwrap_or("") != "")
                .map(|f| MultipartField {
                    key: f.key.clone().unwrap_or_default(),
                    value: if f.field_type.as_deref() == Some("file") {
                        String::new()
                    } else {
                        f.value.clone().unwrap_or_default()
                    },
                    is_file: f.field_type.as_deref() == Some("file"),
                    file_path: if f.field_type.as_deref() == Some("file") {
                        f.src.clone().unwrap_or_default()
                    } else {
                        String::new()
                    },
                    enabled: !f.disabled,
                })
                .collect();
            (BodyType::Multipart, String::new(), fields)
        }
        _ => (BodyType::None, String::new(), vec![]),
    }
}
