use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    pub id: String,
    pub name: String,
    pub is_active: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVariable {
    pub id: String,
    pub environment_id: String,
    pub key: String,
    pub value: String,
    pub is_secret: bool,
}

/// Per-collection, per-environment variable (the "Matrix" model).
/// Collection variables override global env variables with the same key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionVariable {
    pub id: String,
    pub collection_id: String,
    pub environment_id: String,
    pub key: String,
    pub value: String,
    pub is_secret: bool,
}
