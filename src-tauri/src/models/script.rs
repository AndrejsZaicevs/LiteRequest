use serde::{Deserialize, Serialize};

/// A persisted record of one post-execution script run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptRun {
    pub id: String,
    /// The request whose post-execution script ran.
    pub request_id: String,
    /// The request execution that triggered it.
    pub execution_id: String,
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
}
