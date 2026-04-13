use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    pub id: String,
    pub name: String,
    pub base_path: String,
    pub auth_config: Option<String>,    // JSON blob for auth settings
    pub headers_config: Option<String>, // JSON blob for collection-level headers
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Folder {
    pub id: String,
    pub collection_id: String,
    pub parent_folder_id: Option<String>,
    pub name: String,
    pub path_prefix: String,
    pub auth_override: Option<String>,
    pub sort_order: i32,
}

/// Represents a node in the collection tree (for UI rendering)
#[derive(Debug, Clone)]
pub enum TreeNode {
    Collection(Collection, Vec<TreeNode>),
    Folder(Folder, Vec<TreeNode>),
    Request(super::Request),
}
