use serde::{Deserialize, Serialize};

/// A standalone script entity (metadata only — content lives in ScriptVersion).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Script {
    pub id: String,
    pub collection_id: String,
    pub folder_id: Option<String>,
    pub name: String,
    pub current_version_id: Option<String>,
    pub sort_order: i32,
    pub created_at: String,
    pub updated_at: String,
}

/// A versioned snapshot of script content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptVersion {
    pub id: String,
    pub script_id: String,
    /// TypeScript source as written by the user
    pub content_ts: String,
    /// Compiled JavaScript (transpiled from TS by Monaco)
    pub content_js: String,
    pub created_at: String,
}

/// Result of a single script execution (post-exec or standalone).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptRun {
    pub id: String,
    /// NULL for post-execution scripts
    pub script_id: Option<String>,
    /// Version that was executed (NULL for post-exec)
    pub version_id: Option<String>,
    /// The request that triggered it (post-exec only)
    pub request_id: Option<String>,
    /// The request execution that triggered it (post-exec only)
    pub execution_id: Option<String>,
    /// "success", "error", "timeout"
    pub status: String,
    /// JSON array of log entries
    pub logs: String,
    /// JSON map of variables that were set
    pub variables_set: String,
    /// Snapshot of the script source at execution time
    pub script_source: String,
    pub error: Option<String>,
    pub duration_ms: u64,
    pub executed_at: String,
}

/// The result returned to the frontend after running a script.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptResult {
    pub status: String,
    pub logs: Vec<String>,
    pub variables_set: std::collections::HashMap<String, String>,
    pub error: Option<String>,
    pub duration_ms: u64,
    /// For post-exec scripts: optionally transformed response body
    pub transformed_response: Option<String>,
}
