use crate::db::Database;
use crate::models::*;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// .lreq file format — JSON-based export of a collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LreqFile {
    pub format: String,
    pub version: String,
    pub collection: LreqCollection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LreqCollection {
    pub name: String,
    pub base_path: String,
    pub auth_config: Option<String>,
    pub folders: Vec<LreqFolder>,
    pub requests: Vec<LreqRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LreqFolder {
    pub name: String,
    pub path_prefix: String,
    pub requests: Vec<LreqRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LreqRequest {
    pub name: String,
    pub data: RequestData,
}

/// Export a collection (latest version of each request, no secrets)
pub fn export_collection(db: &Database, collection_id: &str) -> Result<String, String> {
    let collections = db.list_collections().map_err(|e| e.to_string())?;
    let collection = collections
        .iter()
        .find(|c| c.id == collection_id)
        .ok_or("Collection not found")?;

    let folders = db
        .list_folders_by_collection(collection_id)
        .map_err(|e| e.to_string())?;
    let requests = db
        .list_requests_by_collection(collection_id)
        .map_err(|e| e.to_string())?;

    // Build folder data
    let mut lreq_folders = Vec::new();
    for folder in &folders {
        let folder_reqs: Vec<LreqRequest> = requests
            .iter()
            .filter(|r| r.folder_id.as_deref() == Some(&folder.id))
            .filter_map(|r| {
                r.current_version_id.as_ref().and_then(|vid| {
                    db.get_version(vid).ok().map(|v| LreqRequest {
                        name: r.name.clone(),
                        data: v.data,
                    })
                })
            })
            .collect();

        lreq_folders.push(LreqFolder {
            name: folder.name.clone(),
            path_prefix: folder.path_prefix.clone(),
            requests: folder_reqs,
        });
    }

    // Top-level requests
    let top_requests: Vec<LreqRequest> = requests
        .iter()
        .filter(|r| r.folder_id.is_none())
        .filter_map(|r| {
            r.current_version_id.as_ref().and_then(|vid| {
                db.get_version(vid).ok().map(|v| LreqRequest {
                    name: r.name.clone(),
                    data: v.data,
                })
            })
        })
        .collect();

    let lreq = LreqFile {
        format: "lreq".to_string(),
        version: "1.0".to_string(),
        collection: LreqCollection {
            name: collection.name.clone(),
            base_path: collection.base_path.clone(),
            auth_config: collection.auth_config.clone(),
            folders: lreq_folders,
            requests: top_requests,
        },
    };

    serde_json::to_string_pretty(&lreq).map_err(|e| e.to_string())
}

/// Export and write to a file
pub fn export_to_file(db: &Database, collection_id: &str, path: &Path) -> Result<(), String> {
    let json = export_collection(db, collection_id)?;
    std::fs::write(path, json).map_err(|e| format!("Failed to write file: {e}"))
}

/// Import from a .lreq file
pub fn import_from_file(db: &Database, path: &Path) -> Result<String, String> {
    let content = std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {e}"))?;
    import_collection(db, &content)
}

/// Import a collection from JSON string. Returns the new collection ID.
pub fn import_collection(db: &Database, json: &str) -> Result<String, String> {
    let lreq: LreqFile = serde_json::from_str(json).map_err(|e| format!("Invalid .lreq format: {e}"))?;

    let now = chrono::Utc::now().to_rfc3339();
    let collection_id = uuid::Uuid::new_v4().to_string();

    let collection = Collection {
        id: collection_id.clone(),
        name: lreq.collection.name,
        base_path: lreq.collection.base_path,
        auth_config: lreq.collection.auth_config,
        created_at: now.clone(),
        updated_at: now.clone(),
    };
    db.insert_collection(&collection).map_err(|e| e.to_string())?;

    // Import folders with their requests
    for (fi, folder) in lreq.collection.folders.iter().enumerate() {
        let folder_id = uuid::Uuid::new_v4().to_string();
        let f = Folder {
            id: folder_id.clone(),
            collection_id: collection_id.clone(),
            parent_folder_id: None,
            name: folder.name.clone(),
            path_prefix: folder.path_prefix.clone(),
            auth_override: None,
            sort_order: fi as i32,
        };
        db.insert_folder(&f).map_err(|e| e.to_string())?;

        for (ri, req) in folder.requests.iter().enumerate() {
            import_request(db, &collection_id, Some(&folder_id), req, ri as i32, &now)?;
        }
    }

    // Import top-level requests
    for (ri, req) in lreq.collection.requests.iter().enumerate() {
        import_request(db, &collection_id, None, req, ri as i32, &now)?;
    }

    Ok(collection_id)
}

fn import_request(
    db: &Database,
    collection_id: &str,
    folder_id: Option<&str>,
    lreq_req: &LreqRequest,
    sort_order: i32,
    now: &str,
) -> Result<(), String> {
    let request_id = uuid::Uuid::new_v4().to_string();
    let version_id = uuid::Uuid::new_v4().to_string();

    let request = Request {
        id: request_id.clone(),
        collection_id: collection_id.to_string(),
        folder_id: folder_id.map(|s| s.to_string()),
        name: lreq_req.name.clone(),
        current_version_id: Some(version_id.clone()),
        sort_order,
    };
    db.insert_request(&request).map_err(|e| e.to_string())?;

    let version = RequestVersion {
        id: version_id,
        request_id,
        data: lreq_req.data.clone(),
        created_at: now.to_string(),
    };
    db.insert_version(&version).map_err(|e| e.to_string())?;

    Ok(())
}
