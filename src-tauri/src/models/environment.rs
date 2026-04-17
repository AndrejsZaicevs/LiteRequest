use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    pub id: String,
    pub name: String,
    pub is_active: bool,
    pub sort_order: i32,
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

/// Global env variable definition (key shared across all environments).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVarDef {
    pub id: String,
    pub key: String,
    pub sort_order: i32,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VarDef {
    pub id: String,
    pub collection_id: String,
    pub key: String,
    pub sort_order: i32,
    /// "regular" (default) or "operative" (shown in request inspector for quick editing)
    #[serde(default = "VarDef::default_var_type")]
    pub var_type: String,
}

impl VarDef {
    pub fn default_var_type() -> String { "regular".to_string() }
}

/// Per-environment value for a variable definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VarValue {
    pub id: String,
    pub def_id: String,
    pub environment_id: String,
    pub value: String,
    pub is_secret: bool,
}

/// UI row combining a variable definition with its value for a particular environment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VarRow {
    pub def_id: String,
    pub key: String,
    pub value: String,
    pub is_secret: bool,
    /// ID of the VarValue row, if one exists for this env
    pub value_id: Option<String>,
    /// Inherited from VarDef — "regular" or "operative"
    #[serde(default = "VarDef::default_var_type")]
    pub var_type: String,
}

/// Legacy model kept for migration only.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionVariable {
    pub id: String,
    pub collection_id: String,
    pub environment_id: String,
    pub key: String,
    pub value: String,
    pub is_secret: bool,
}
