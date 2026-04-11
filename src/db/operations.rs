use crate::models::*;
use rusqlite::{params, Connection};
use sha2::{Digest, Sha256};
use std::path::Path;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open(path: &Path) -> rusqlite::Result<Self> {
        let conn = Connection::open(path)?;
        super::schema::initialize(&conn)?;
        Ok(Self { conn })
    }

    pub fn open_in_memory() -> rusqlite::Result<Self> {
        let conn = Connection::open_in_memory()?;
        super::schema::initialize(&conn)?;
        Ok(Self { conn })
    }

    // ── Collections ──────────────────────────────────────────────

    pub fn insert_collection(&self, c: &Collection) -> rusqlite::Result<()> {
        self.conn.execute(
            "INSERT INTO collections (id, name, base_path, auth_config, headers_config, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![c.id, c.name, c.base_path, c.auth_config, c.headers_config, c.created_at, c.updated_at],
        )?;
        Ok(())
    }

    pub fn update_collection(&self, c: &Collection) -> rusqlite::Result<()> {
        self.conn.execute(
            "UPDATE collections SET name=?2, base_path=?3, auth_config=?4, headers_config=?5, updated_at=?6 WHERE id=?1",
            params![c.id, c.name, c.base_path, c.auth_config, c.headers_config, c.updated_at],
        )?;
        Ok(())
    }

    pub fn delete_collection(&self, id: &str) -> rusqlite::Result<()> {
        self.conn.execute("DELETE FROM collections WHERE id=?1", params![id])?;
        Ok(())
    }

    pub fn list_collections(&self) -> rusqlite::Result<Vec<Collection>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, base_path, auth_config, headers_config, created_at, updated_at FROM collections ORDER BY name",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Collection {
                id: row.get(0)?,
                name: row.get(1)?,
                base_path: row.get(2)?,
                auth_config: row.get(3)?,
                headers_config: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })?;
        rows.collect()
    }

    // ── Folders ──────────────────────────────────────────────────

    pub fn insert_folder(&self, f: &Folder) -> rusqlite::Result<()> {
        self.conn.execute(
            "INSERT INTO folders (id, collection_id, parent_folder_id, name, path_prefix, auth_override, sort_order)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![f.id, f.collection_id, f.parent_folder_id, f.name, f.path_prefix, f.auth_override, f.sort_order],
        )?;
        Ok(())
    }

    pub fn list_folders_by_collection(&self, collection_id: &str) -> rusqlite::Result<Vec<Folder>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, collection_id, parent_folder_id, name, path_prefix, auth_override, sort_order
             FROM folders WHERE collection_id=?1 ORDER BY sort_order",
        )?;
        let rows = stmt.query_map(params![collection_id], |row| {
            Ok(Folder {
                id: row.get(0)?,
                collection_id: row.get(1)?,
                parent_folder_id: row.get(2)?,
                name: row.get(3)?,
                path_prefix: row.get(4)?,
                auth_override: row.get(5)?,
                sort_order: row.get(6)?,
            })
        })?;
        rows.collect()
    }

    pub fn delete_folder(&self, id: &str) -> rusqlite::Result<()> {
        self.conn.execute("DELETE FROM folders WHERE id=?1", params![id])?;
        Ok(())
    }

    // ── Requests ─────────────────────────────────────────────────

    pub fn insert_request(&self, r: &Request) -> rusqlite::Result<()> {
        self.conn.execute(
            "INSERT INTO requests (id, collection_id, folder_id, name, current_version_id, sort_order)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![r.id, r.collection_id, r.folder_id, r.name, r.current_version_id, r.sort_order],
        )?;
        Ok(())
    }

    pub fn update_request_version(&self, request_id: &str, version_id: &str) -> rusqlite::Result<()> {
        self.conn.execute(
            "UPDATE requests SET current_version_id=?2 WHERE id=?1",
            params![request_id, version_id],
        )?;
        Ok(())
    }

    pub fn rename_request(&self, id: &str, name: &str) -> rusqlite::Result<()> {
        self.conn.execute(
            "UPDATE requests SET name=?2 WHERE id=?1",
            params![id, name],
        )?;
        Ok(())
    }

    pub fn delete_request(&self, id: &str) -> rusqlite::Result<()> {
        self.conn.execute("DELETE FROM requests WHERE id=?1", params![id])?;
        Ok(())
    }

    pub fn list_requests_by_collection(&self, collection_id: &str) -> rusqlite::Result<Vec<Request>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, collection_id, folder_id, name, current_version_id, sort_order
             FROM requests WHERE collection_id=?1 ORDER BY sort_order",
        )?;
        let rows = stmt.query_map(params![collection_id], |row| {
            Ok(Request {
                id: row.get(0)?,
                collection_id: row.get(1)?,
                folder_id: row.get(2)?,
                name: row.get(3)?,
                current_version_id: row.get(4)?,
                sort_order: row.get(5)?,
            })
        })?;
        rows.collect()
    }

    pub fn list_requests_by_folder(&self, folder_id: &str) -> rusqlite::Result<Vec<Request>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, collection_id, folder_id, name, current_version_id, sort_order
             FROM requests WHERE folder_id=?1 ORDER BY sort_order",
        )?;
        let rows = stmt.query_map(params![folder_id], |row| {
            Ok(Request {
                id: row.get(0)?,
                collection_id: row.get(1)?,
                folder_id: row.get(2)?,
                name: row.get(3)?,
                current_version_id: row.get(4)?,
                sort_order: row.get(5)?,
            })
        })?;
        rows.collect()
    }

    /// Requests not in any folder (top-level in collection)
    pub fn list_orphan_requests(&self, collection_id: &str) -> rusqlite::Result<Vec<Request>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, collection_id, folder_id, name, current_version_id, sort_order
             FROM requests WHERE collection_id=?1 AND folder_id IS NULL ORDER BY sort_order",
        )?;
        let rows = stmt.query_map(params![collection_id], |row| {
            Ok(Request {
                id: row.get(0)?,
                collection_id: row.get(1)?,
                folder_id: row.get(2)?,
                name: row.get(3)?,
                current_version_id: row.get(4)?,
                sort_order: row.get(5)?,
            })
        })?;
        rows.collect()
    }

    // ── Request Versions ─────────────────────────────────────────

    pub fn insert_version(&self, v: &RequestVersion) -> rusqlite::Result<()> {
        let data_json = serde_json::to_string(&v.data).unwrap_or_default();
        self.conn.execute(
            "INSERT INTO request_versions (id, request_id, data_json, created_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![v.id, v.request_id, data_json, v.created_at],
        )?;
        // Update current version pointer
        self.conn.execute(
            "UPDATE requests SET current_version_id=?2 WHERE id=?1",
            params![v.request_id, v.id],
        )?;
        Ok(())
    }

    pub fn get_version(&self, id: &str) -> rusqlite::Result<RequestVersion> {
        self.conn.query_row(
            "SELECT id, request_id, data_json, created_at FROM request_versions WHERE id=?1",
            params![id],
            |row| {
                let data_json: String = row.get(2)?;
                let data: RequestData = serde_json::from_str(&data_json)
                    .unwrap_or_default();
                Ok(RequestVersion {
                    id: row.get(0)?,
                    request_id: row.get(1)?,
                    data,
                    created_at: row.get(3)?,
                })
            },
        )
    }

    pub fn list_versions_by_request(&self, request_id: &str) -> rusqlite::Result<Vec<RequestVersion>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, request_id, data_json, created_at
             FROM request_versions WHERE request_id=?1 ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map(params![request_id], |row| {
            let data_json: String = row.get(2)?;
            let data: RequestData = serde_json::from_str(&data_json)
                .unwrap_or_default();
            Ok(RequestVersion {
                id: row.get(0)?,
                request_id: row.get(1)?,
                data,
                created_at: row.get(3)?,
            })
        })?;
        rows.collect()
    }

    /// Check whether a version has any executions linked to it.
    pub fn version_has_executions(&self, version_id: &str) -> bool {
        self.conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM request_executions WHERE version_id=?1)",
            params![version_id],
            |row| row.get::<_, bool>(0),
        ).unwrap_or(false)
    }

    /// Overwrite a version's data and timestamp in place.
    pub fn update_version_data(&self, version_id: &str, data: &RequestData, created_at: &str) -> rusqlite::Result<()> {
        let data_json = serde_json::to_string(data).unwrap_or_default();
        self.conn.execute(
            "UPDATE request_versions SET data_json=?2, created_at=?3 WHERE id=?1",
            params![version_id, data_json, created_at],
        )?;
        Ok(())
    }

    /// Delete a version by id (used to clean up empty drafts during dedup).
    pub fn delete_version(&self, version_id: &str) -> rusqlite::Result<()> {
        self.conn.execute(
            "DELETE FROM request_versions WHERE id=?1",
            params![version_id],
        )?;
        Ok(())
    }

    // ── Request Executions ───────────────────────────────────────

    fn body_hash(body: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(body.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    fn get_or_create_response_body(&self, body: &str) -> rusqlite::Result<String> {
        let hash = Self::body_hash(body);
        self.conn.execute(
            "INSERT OR IGNORE INTO response_bodies (hash, body) VALUES (?1, ?2)",
            params![hash, body],
        )?;
        Ok(hash)
    }

    pub fn insert_execution(&self, e: &RequestExecution) -> rusqlite::Result<()> {
        let body_hash = self.get_or_create_response_body(&e.response.body)?;
        let mut stripped_response = e.response.clone();
        stripped_response.body = String::new();
        let response_json = serde_json::to_string(&stripped_response).unwrap_or_default();
        self.conn.execute(
            "INSERT INTO request_executions (id, version_id, request_id, environment_id, response_json, latency_ms, executed_at, body_hash)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![e.id, e.version_id, e.request_id, e.environment_id, response_json, e.latency_ms, e.executed_at, body_hash],
        )?;
        Ok(())
    }

    pub fn list_executions_by_request(&self, request_id: &str) -> rusqlite::Result<Vec<RequestExecution>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, version_id, request_id, response_json, latency_ms, executed_at, environment_id, body_hash
             FROM request_executions WHERE request_id=?1 ORDER BY executed_at DESC",
        )?;
        let rows = stmt.query_map(params![request_id], |row| {
            let response_json: String = row.get(3)?;
            let response: ResponseData = serde_json::from_str(&response_json)
                .unwrap_or_else(|_| ResponseData {
                    status: 0,
                    status_text: "Parse Error".to_string(),
                    headers: Default::default(),
                    body: String::new(),
                    size_bytes: 0,
                });
            let body_hash: String = row.get(7)?;
            Ok((
                RequestExecution {
                    id: row.get(0)?,
                    version_id: row.get(1)?,
                    request_id: row.get(2)?,
                    environment_id: row.get(6)?,
                    response,
                    latency_ms: row.get(4)?,
                    executed_at: row.get(5)?,
                },
                body_hash,
            ))
        })?;
        let pairs: Vec<(RequestExecution, String)> = rows.collect::<rusqlite::Result<_>>()?;
        let mut result = Vec::with_capacity(pairs.len());
        for (mut exec, hash) in pairs {
            if let Ok(body) = self.conn.query_row(
                "SELECT body FROM response_bodies WHERE hash=?1",
                params![hash],
                |r| r.get::<_, String>(0),
            ) {
                exec.response.body = body;
            }
            result.push(exec);
        }
        Ok(result)
    }

    // ── Environments ─────────────────────────────────────────────

    pub fn insert_environment(&self, e: &Environment) -> rusqlite::Result<()> {
        self.conn.execute(
            "INSERT INTO environments (id, name, is_active, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![e.id, e.name, e.is_active, e.created_at],
        )?;
        Ok(())
    }

    pub fn list_environments(&self) -> rusqlite::Result<Vec<Environment>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, is_active, created_at FROM environments ORDER BY name",
        )?;
        let rows = stmt.query_map([], |row| {
            let is_active: i32 = row.get(2)?;
            Ok(Environment {
                id: row.get(0)?,
                name: row.get(1)?,
                is_active: is_active != 0,
                created_at: row.get(3)?,
            })
        })?;
        rows.collect()
    }

    pub fn set_active_environment(&self, id: &str) -> rusqlite::Result<()> {
        self.conn.execute("UPDATE environments SET is_active=0", [])?;
        self.conn.execute("UPDATE environments SET is_active=1 WHERE id=?1", params![id])?;
        Ok(())
    }

    pub fn delete_environment(&self, id: &str) -> rusqlite::Result<()> {
        self.conn.execute("DELETE FROM environments WHERE id=?1", params![id])?;
        Ok(())
    }

    // ── Environment Variables ────────────────────────────────────

    pub fn insert_env_variable(&self, v: &EnvVariable) -> rusqlite::Result<()> {
        self.conn.execute(
            "INSERT INTO env_variables (id, environment_id, key, value, is_secret)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![v.id, v.environment_id, v.key, v.value, v.is_secret],
        )?;
        Ok(())
    }

    pub fn list_env_variables(&self, environment_id: &str) -> rusqlite::Result<Vec<EnvVariable>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, environment_id, key, value, is_secret
             FROM env_variables WHERE environment_id=?1 ORDER BY key",
        )?;
        let rows = stmt.query_map(params![environment_id], |row| {
            let is_secret: i32 = row.get(4)?;
            Ok(EnvVariable {
                id: row.get(0)?,
                environment_id: row.get(1)?,
                key: row.get(2)?,
                value: row.get(3)?,
                is_secret: is_secret != 0,
            })
        })?;
        rows.collect()
    }

    pub fn update_env_variable(&self, v: &EnvVariable) -> rusqlite::Result<()> {
        self.conn.execute(
            "UPDATE env_variables SET key=?2, value=?3, is_secret=?4 WHERE id=?1",
            params![v.id, v.key, v.value, v.is_secret],
        )?;
        Ok(())
    }

    pub fn delete_env_variable(&self, id: &str) -> rusqlite::Result<()> {
        self.conn.execute("DELETE FROM env_variables WHERE id=?1", params![id])?;
        Ok(())
    }

    /// Get all variables for the currently active environment
    pub fn get_active_variables(&self) -> rusqlite::Result<Vec<EnvVariable>> {
        let mut stmt = self.conn.prepare(
            "SELECT v.id, v.environment_id, v.key, v.value, v.is_secret
             FROM env_variables v
             JOIN environments e ON v.environment_id = e.id
             WHERE e.is_active = 1
             ORDER BY v.key",
        )?;
        let rows = stmt.query_map([], |row| {
            let is_secret: i32 = row.get(4)?;
            Ok(EnvVariable {
                id: row.get(0)?,
                environment_id: row.get(1)?,
                key: row.get(2)?,
                value: row.get(3)?,
                is_secret: is_secret != 0,
            })
        })?;
        rows.collect()
    }

    // ── Collection Variables (split def/value model) ───────────────

    pub fn insert_var_def(&self, d: &VarDef) -> rusqlite::Result<()> {
        self.conn.execute(
            "INSERT INTO collection_var_defs (id, collection_id, key, sort_order)
             VALUES (?1, ?2, ?3, ?4)",
            params![d.id, d.collection_id, d.key, d.sort_order],
        )?;
        Ok(())
    }

    pub fn update_var_def_key(&self, def_id: &str, key: &str) -> rusqlite::Result<()> {
        self.conn.execute(
            "UPDATE collection_var_defs SET key=?2 WHERE id=?1",
            params![def_id, key],
        )?;
        Ok(())
    }

    pub fn delete_var_def(&self, def_id: &str) -> rusqlite::Result<()> {
        // CASCADE deletes associated var_values
        self.conn.execute(
            "DELETE FROM collection_var_defs WHERE id=?1",
            params![def_id],
        )?;
        Ok(())
    }

    /// List variable definitions for a collection (ordered by sort_order).
    pub fn list_var_defs(&self, collection_id: &str) -> rusqlite::Result<Vec<VarDef>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, collection_id, key, sort_order
             FROM collection_var_defs
             WHERE collection_id=?1
             ORDER BY sort_order, key",
        )?;
        let rows = stmt.query_map(params![collection_id], |row| {
            Ok(VarDef {
                id: row.get(0)?,
                collection_id: row.get(1)?,
                key: row.get(2)?,
                sort_order: row.get(3)?,
            })
        })?;
        rows.collect()
    }

    /// Upsert a variable value for a specific (def, environment) pair.
    pub fn upsert_var_value(
        &self,
        val_id: &str,
        def_id: &str,
        environment_id: &str,
        value: &str,
        is_secret: bool,
    ) -> rusqlite::Result<()> {
        // Try update first by (def_id, environment_id) composite key
        let updated = self.conn.execute(
            "UPDATE collection_var_values SET value=?3, is_secret=?4
             WHERE def_id=?1 AND environment_id=?2",
            params![def_id, environment_id, value, is_secret as i32],
        )?;
        if updated == 0 {
            self.conn.execute(
                "INSERT INTO collection_var_values (id, def_id, environment_id, value, is_secret)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![val_id, def_id, environment_id, value, is_secret as i32],
            )?;
        }
        Ok(())
    }

    /// Load VarRows for a collection + environment (joins defs with values).
    pub fn load_var_rows(
        &self,
        collection_id: &str,
        environment_id: &str,
    ) -> rusqlite::Result<Vec<VarRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT d.id, d.key, v.value, v.is_secret, v.id
             FROM collection_var_defs d
             LEFT JOIN collection_var_values v
               ON v.def_id = d.id AND v.environment_id = ?2
             WHERE d.collection_id = ?1
             ORDER BY d.sort_order, d.key",
        )?;
        let rows = stmt.query_map(params![collection_id, environment_id], |row| {
            let is_secret: Option<i32> = row.get(3)?;
            Ok(VarRow {
                def_id: row.get(0)?,
                key: row.get(1)?,
                value: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
                is_secret: is_secret.unwrap_or(0) != 0,
                value_id: row.get(4)?,
            })
        })?;
        rows.collect()
    }

    /// Get collection variable key-value pairs for the active environment (used at request time).
    pub fn get_active_collection_variables(
        &self,
        collection_id: &str,
    ) -> rusqlite::Result<Vec<(String, String)>> {
        let mut stmt = self.conn.prepare(
            "SELECT d.key, v.value
             FROM collection_var_defs d
             JOIN collection_var_values v ON v.def_id = d.id
             JOIN environments e ON v.environment_id = e.id
             WHERE d.collection_id = ?1 AND e.is_active = 1
             ORDER BY d.sort_order",
        )?;
        let rows = stmt.query_map(params![collection_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        rows.collect()
    }

    // ── Move / Rename helpers ────────────────────────────────────

    pub fn move_request(&self, id: &str, collection_id: &str, folder_id: Option<&str>) -> rusqlite::Result<()> {
        self.conn.execute(
            "UPDATE requests SET collection_id=?2, folder_id=?3 WHERE id=?1",
            params![id, collection_id, folder_id],
        )?;
        Ok(())
    }

    pub fn move_folder(&self, id: &str, collection_id: &str, parent_folder_id: Option<&str>) -> rusqlite::Result<()> {
        self.conn.execute(
            "UPDATE folders SET collection_id=?2, parent_folder_id=?3 WHERE id=?1",
            params![id, collection_id, parent_folder_id],
        )?;
        Ok(())
    }

    pub fn reorder_requests(&self, ordered_ids: &[String]) -> rusqlite::Result<()> {
        for (i, id) in ordered_ids.iter().enumerate() {
            self.conn.execute(
                "UPDATE requests SET sort_order=?2 WHERE id=?1",
                params![id, i as i32],
            )?;
        }
        Ok(())
    }

    pub fn rename_folder(&self, id: &str, name: &str) -> rusqlite::Result<()> {
        self.conn.execute(
            "UPDATE folders SET name=?2 WHERE id=?1",
            params![id, name],
        )?;
        Ok(())
    }

    pub fn rename_collection(&self, id: &str, name: &str) -> rusqlite::Result<()> {
        self.conn.execute(
            "UPDATE collections SET name=?2 WHERE id=?1",
            params![id, name],
        )?;
        Ok(())
    }

    // ── Maintenance ──────────────────────────────────────────────

    pub fn prune_old_executions(&self, days: i64) -> rusqlite::Result<usize> {
        let cutoff = chrono::Utc::now() - chrono::Duration::days(days);
        let cutoff_str = cutoff.to_rfc3339();
        self.conn.execute(
            "DELETE FROM request_executions WHERE executed_at < ?1",
            params![cutoff_str],
        )
    }

    // ── App Settings (key-value store) ───────────────────────────

    pub fn get_app_setting(&self, key: &str) -> rusqlite::Result<Option<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT value FROM app_settings WHERE key=?1")?;
        let mut rows = stmt.query_map(params![key], |row| row.get::<_, String>(0))?;
        match rows.next() {
            Some(Ok(val)) => Ok(Some(val)),
            _ => Ok(None),
        }
    }

    pub fn set_app_setting(&self, key: &str, value: &str) -> rusqlite::Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO app_settings (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;
        Ok(())
    }
}
