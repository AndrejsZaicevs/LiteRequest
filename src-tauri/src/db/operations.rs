use crate::models::*;
use rusqlite::{params, Connection, OptionalExtension};
use sha2::{Digest, Sha256};
use std::path::Path;

fn uuid() -> String {
    uuid::Uuid::new_v4().to_string()
}

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
        self.conn.execute(
            "UPDATE collections SET deleted_at=datetime('now') WHERE id=?1",
            params![id],
        )?;
        Ok(())
    }

    pub fn list_collections(&self) -> rusqlite::Result<Vec<Collection>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, base_path, auth_config, headers_config, created_at, updated_at
             FROM collections WHERE deleted_at IS NULL ORDER BY name",
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
             FROM folders WHERE collection_id=?1 AND deleted_at IS NULL ORDER BY sort_order",
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
        self.conn.execute(
            "UPDATE folders SET deleted_at=datetime('now') WHERE id=?1",
            params![id],
        )?;
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
        self.conn.execute(
            "UPDATE requests SET deleted_at=datetime('now') WHERE id=?1",
            params![id],
        )?;
        Ok(())
    }

    pub fn list_requests_by_collection(&self, collection_id: &str) -> rusqlite::Result<Vec<Request>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, collection_id, folder_id, name, current_version_id, sort_order
             FROM requests WHERE collection_id=?1 AND deleted_at IS NULL ORDER BY sort_order",
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
             FROM requests WHERE folder_id=?1 AND deleted_at IS NULL ORDER BY sort_order",
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
             FROM requests WHERE collection_id=?1 AND folder_id IS NULL AND deleted_at IS NULL ORDER BY sort_order",
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
        let fingerprint = if v.fingerprint.is_empty() { v.data.fingerprint() } else { v.fingerprint.clone() };
        self.conn.execute(
            "INSERT INTO request_versions (id, request_id, data_json, fingerprint, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![v.id, v.request_id, data_json, fingerprint, v.created_at],
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
            "SELECT id, request_id, data_json, fingerprint, created_at FROM request_versions WHERE id=?1",
            params![id],
            |row| {
                let data_json: String = row.get(2)?;
                let data: RequestData = serde_json::from_str(&data_json)
                    .unwrap_or_default();
                Ok(RequestVersion {
                    id: row.get(0)?,
                    request_id: row.get(1)?,
                    fingerprint: row.get(3)?,
                    data,
                    created_at: row.get(4)?,
                })
            },
        )
    }

    pub fn list_versions_by_request(&self, request_id: &str) -> rusqlite::Result<Vec<RequestVersion>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, request_id, data_json, fingerprint, created_at
             FROM request_versions WHERE request_id=?1 ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map(params![request_id], |row| {
            let data_json: String = row.get(2)?;
            let data: RequestData = serde_json::from_str(&data_json)
                .unwrap_or_default();
            Ok(RequestVersion {
                id: row.get(0)?,
                request_id: row.get(1)?,
                fingerprint: row.get(3)?,
                data,
                created_at: row.get(4)?,
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

    /// Overwrite a version's data, fingerprint, and timestamp in place.
    pub fn update_version_data(&self, version_id: &str, data: &RequestData, created_at: &str) -> rusqlite::Result<()> {
        let data_json = serde_json::to_string(data).unwrap_or_default();
        let fingerprint = data.fingerprint();
        self.conn.execute(
            "UPDATE request_versions SET data_json=?2, fingerprint=?3, created_at=?4 WHERE id=?1",
            params![version_id, data_json, fingerprint, created_at],
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

    /// All-in-one save: decides whether to update an existing version in
    /// place, reuse a previous one, or create a new one.
    /// Returns the resulting version.
    ///
    /// Rules:
    ///  1. No current version → create.
    ///  2. Data identical to current version → no-op, return current.
    ///  3. Current version has NO executions → overwrite in place (draft).
    ///  4. Same fingerprint as current → overwrite in place (value-only).
    ///  5. Different fingerprint → look for an older version with the same
    ///     fingerprint and reuse it (update its data, make it current).
    ///  6. No matching version at all → create new.
    pub fn save_version(&self, request_id: &str, data: &RequestData) -> rusqlite::Result<RequestVersion> {
        self.conn.execute_batch("BEGIN IMMEDIATE")?;
        match self.save_version_inner(request_id, data) {
            Ok(v) => {
                self.conn.execute_batch("COMMIT")?;
                Ok(v)
            }
            Err(e) => {
                let _ = self.conn.execute_batch("ROLLBACK");
                Err(e)
            }
        }
    }

    fn save_version_inner(&self, request_id: &str, data: &RequestData) -> rusqlite::Result<RequestVersion> {
        let now = chrono::Utc::now().to_rfc3339();
        let new_fp = data.fingerprint();

        // Find current version id
        let current_vid: Option<String> = self.conn.query_row(
            "SELECT current_version_id FROM requests WHERE id=?1",
            params![request_id],
            |row| row.get(0),
        )?;

        if let Some(ref vid) = current_vid {
            let current = self.get_version(vid)?;
            let current_json = serde_json::to_string(&current.data).unwrap_or_default();
            let new_json = serde_json::to_string(data).unwrap_or_default();

            // Identical → no-op
            if current_json == new_json {
                return Ok(current);
            }

            let has_exec = self.version_has_executions(vid);

            if !has_exec {
                // Draft — always overwrite
                self.update_version_data(vid, data, &now)?;
                return self.get_version(vid);
            }

            let cur_fp = if current.fingerprint.is_empty() {
                current.data.fingerprint()
            } else {
                current.fingerprint.clone()
            };

            if cur_fp == new_fp {
                // Same structure, value-only change — overwrite
                self.update_version_data(vid, data, &now)?;
                return self.get_version(vid);
            }

            // Different fingerprint — try to reuse an older version
            let existing: Option<String> = self.conn.query_row(
                "SELECT id FROM request_versions
                 WHERE request_id=?1 AND fingerprint=?2 AND id!=?3
                 ORDER BY created_at DESC LIMIT 1",
                params![request_id, new_fp, vid],
                |row| row.get(0),
            ).optional()?;

            if let Some(reuse_id) = existing {
                // Update its data to latest values and make it current
                self.update_version_data(&reuse_id, data, &now)?;
                self.conn.execute(
                    "UPDATE requests SET current_version_id=?2 WHERE id=?1",
                    params![request_id, reuse_id],
                )?;
                return self.get_version(&reuse_id);
            }
        }

        // Create new version
        let version = RequestVersion {
            id: uuid::Uuid::new_v4().to_string(),
            request_id: request_id.to_string(),
            data: data.clone(),
            fingerprint: new_fp,
            created_at: now,
        };
        self.insert_version(&version)?;
        Ok(version)
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
        let request_data_json = match &e.request_data {
            Some(rd) => serde_json::to_string(rd).unwrap_or_default(),
            None => String::new(),
        };
        self.conn.execute(
            "INSERT INTO request_executions (id, version_id, request_id, environment_id, response_json, latency_ms, executed_at, body_hash, request_data_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![e.id, e.version_id, e.request_id, e.environment_id, response_json, e.latency_ms, e.executed_at, body_hash, request_data_json],
        )?;
        Ok(())
    }

    pub fn list_executions_by_request(&self, request_id: &str) -> rusqlite::Result<Vec<RequestExecution>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, version_id, request_id, response_json, latency_ms, executed_at, environment_id, body_hash, request_data_json
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
                    is_binary: false,
                });
            let body_hash: String = row.get(7)?;
            let request_data_json: String = row.get(8)?;
            let request_data: Option<RequestData> = if request_data_json.is_empty() {
                None
            } else {
                serde_json::from_str(&request_data_json).ok()
            };
            Ok((
                RequestExecution {
                    id: row.get(0)?,
                    version_id: row.get(1)?,
                    request_id: row.get(2)?,
                    environment_id: row.get(6)?,
                    response,
                    latency_ms: row.get(4)?,
                    executed_at: row.get(5)?,
                    request_data,
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
            "INSERT INTO environments (id, name, is_active, created_at, sort_order)
             VALUES (?1, ?2, ?3, ?4, COALESCE((SELECT MAX(sort_order)+1 FROM environments), 0))",
            params![e.id, e.name, e.is_active, e.created_at],
        )?;
        Ok(())
    }

    pub fn list_environments(&self) -> rusqlite::Result<Vec<Environment>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, is_active, created_at, sort_order FROM environments ORDER BY sort_order, name",
        )?;
        let rows = stmt.query_map([], |row| {
            let is_active: i32 = row.get(2)?;
            Ok(Environment {
                id: row.get(0)?,
                name: row.get(1)?,
                is_active: is_active != 0,
                created_at: row.get(3)?,
                sort_order: row.get(4)?,
            })
        })?;
        rows.collect()
    }

    pub fn reorder_environments(&self, ordered_ids: &[String]) -> rusqlite::Result<()> {
        for (i, id) in ordered_ids.iter().enumerate() {
            self.conn.execute(
                "UPDATE environments SET sort_order=?2 WHERE id=?1",
                params![id, i as i32],
            )?;
        }
        Ok(())
    }

    pub fn set_active_environment(&self, id: &str) -> rusqlite::Result<()> {
        self.conn.execute("UPDATE environments SET is_active=0", [])?;
        self.conn.execute("UPDATE environments SET is_active=1 WHERE id=?1", params![id])?;
        Ok(())
    }

    pub fn rename_environment(&self, id: &str, name: &str) -> rusqlite::Result<()> {
        self.conn.execute("UPDATE environments SET name=?2 WHERE id=?1", params![id, name])?;
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

    /// Get all variables for the currently active environment (from new split tables)
    pub fn get_active_variables(&self) -> rusqlite::Result<Vec<EnvVariable>> {
        let mut stmt = self.conn.prepare(
            "SELECT d.id, e.id, d.key, COALESCE(v.value, '') as value, COALESCE(v.is_secret, 0) as is_secret
             FROM env_var_defs d
             CROSS JOIN environments e
             LEFT JOIN env_var_values v ON v.def_id = d.id AND v.environment_id = e.id
             WHERE e.is_active = 1
             ORDER BY d.sort_order, d.key",
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
            "INSERT INTO collection_var_defs (id, collection_id, key, sort_order, var_type)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![d.id, d.collection_id, d.key, d.sort_order, d.var_type],
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

    pub fn update_var_def_type(&self, def_id: &str, var_type: &str) -> rusqlite::Result<()> {
        self.conn.execute(
            "UPDATE collection_var_defs SET var_type=?2 WHERE id=?1",
            params![def_id, var_type],
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
            "SELECT id, collection_id, key, sort_order, COALESCE(var_type, 'regular')
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
                var_type: row.get(4)?,
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
            "SELECT d.id, d.key, v.value, v.is_secret, v.id, COALESCE(d.var_type, 'regular')
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
                var_type: row.get(5)?,
            })
        })?;
        rows.collect()
    }

    /// Load only operative VarRows for a collection + environment (for the request inspector).
    pub fn load_operative_var_rows(
        &self,
        collection_id: &str,
        environment_id: &str,
    ) -> rusqlite::Result<Vec<VarRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT d.id, d.key, v.value, v.is_secret, v.id, COALESCE(d.var_type, 'regular')
             FROM collection_var_defs d
             LEFT JOIN collection_var_values v
               ON v.def_id = d.id AND v.environment_id = ?2
             WHERE d.collection_id = ?1 AND COALESCE(d.var_type, 'regular') = 'operative'
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
                var_type: row.get(5)?,
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

    // ── Env Variable Defs (global keys, shared across envs) ──────────

    pub fn insert_env_var_def(&self, d: &EnvVarDef) -> rusqlite::Result<()> {
        self.conn.execute(
            "INSERT INTO env_var_defs (id, key, sort_order) VALUES (?1, ?2, ?3)",
            params![d.id, d.key, d.sort_order],
        )?;
        Ok(())
    }

    pub fn update_env_var_def_key(&self, def_id: &str, key: &str) -> rusqlite::Result<()> {
        self.conn.execute(
            "UPDATE env_var_defs SET key=?2 WHERE id=?1",
            params![def_id, key],
        )?;
        Ok(())
    }

    pub fn delete_env_var_def(&self, def_id: &str) -> rusqlite::Result<()> {
        self.conn.execute(
            "DELETE FROM env_var_defs WHERE id=?1",
            params![def_id],
        )?;
        Ok(())
    }

    pub fn list_env_var_defs(&self) -> rusqlite::Result<Vec<EnvVarDef>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, key, sort_order FROM env_var_defs ORDER BY sort_order, key",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(EnvVarDef { id: row.get(0)?, key: row.get(1)?, sort_order: row.get(2)? })
        })?;
        rows.collect()
    }

    pub fn upsert_env_var_value(
        &self,
        val_id: &str,
        def_id: &str,
        environment_id: &str,
        value: &str,
        is_secret: bool,
    ) -> rusqlite::Result<()> {
        let updated = self.conn.execute(
            "UPDATE env_var_values SET value=?3, is_secret=?4
             WHERE def_id=?1 AND environment_id=?2",
            params![def_id, environment_id, value, is_secret as i32],
        )?;
        if updated == 0 {
            self.conn.execute(
                "INSERT INTO env_var_values (id, def_id, environment_id, value, is_secret)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![val_id, def_id, environment_id, value, is_secret as i32],
            )?;
        }
        Ok(())
    }

    /// Load VarRows for an environment (joins defs with values).
    pub fn load_env_var_rows(&self, environment_id: &str) -> rusqlite::Result<Vec<VarRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT d.id, d.key, v.value, v.is_secret, v.id
             FROM env_var_defs d
             LEFT JOIN env_var_values v
               ON v.def_id = d.id AND v.environment_id = ?1
             ORDER BY d.sort_order, d.key",
        )?;
        let rows = stmt.query_map(params![environment_id], |row| {
            let is_secret: Option<i32> = row.get(3)?;
            Ok(VarRow {
                def_id: row.get(0)?,
                key: row.get(1)?,
                value: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
                is_secret: is_secret.unwrap_or(0) != 0,
                value_id: row.get(4)?,
                var_type: "regular".to_string(),
            })
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

    pub fn reorder_folders(&self, ordered_ids: &[String]) -> rusqlite::Result<()> {
        for (i, id) in ordered_ids.iter().enumerate() {
            self.conn.execute(
                "UPDATE folders SET sort_order=?2 WHERE id=?1",
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

    pub fn get_db_stats(&self) -> rusqlite::Result<DbStats> {
        let page_count: i64 = self.conn.query_row("PRAGMA page_count", [], |r| r.get(0))?;
        let page_size: i64 = self.conn.query_row("PRAGMA page_size", [], |r| r.get(0))?;
        let version_count: i64 = self.conn.query_row("SELECT COUNT(*) FROM request_versions", [], |r| r.get(0))?;
        let execution_count: i64 = self.conn.query_row("SELECT COUNT(*) FROM request_executions", [], |r| r.get(0))?;
        let oldest_execution: Option<String> = self.conn
            .query_row("SELECT MIN(executed_at) FROM request_executions", [], |r| r.get(0))
            .optional()?.flatten();
        let oldest_version: Option<String> = self.conn
            .query_row("SELECT MIN(created_at) FROM request_versions", [], |r| r.get(0))
            .optional()?.flatten();
        Ok(DbStats {
            db_size_bytes: page_count * page_size,
            version_count,
            execution_count,
            oldest_execution,
            oldest_version,
        })
    }

    /// Delete executions before cutoff_date (ISO 8601) and orphaned non-current versions
    /// before cutoff_date, then VACUUM. Returns deleted counts.
    pub fn cleanup_old_data(&self, cutoff_date: &str) -> rusqlite::Result<CleanupResult> {
        let executions_deleted = self.conn.execute(
            "DELETE FROM request_executions WHERE executed_at < ?1",
            params![cutoff_date],
        )?;
        // Delete versions older than cutoff that are NOT the current_version_id of any request
        let versions_deleted = self.conn.execute(
            "DELETE FROM request_versions
             WHERE created_at < ?1
               AND id NOT IN (SELECT current_version_id FROM requests WHERE current_version_id IS NOT NULL)",
            params![cutoff_date],
        )?;
        self.conn.execute_batch("VACUUM")?;
        Ok(CleanupResult { versions_deleted, executions_deleted })
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

    // ── Full-text search ─────────────────────────────────────────

    /// Extract a short context snippet around the first occurrence of `query` in `text`.
    fn extract_snippet(text: &str, query: &str, context_chars: usize) -> String {
        let text_lower = text.to_lowercase();
        let query_lower = query.to_lowercase();
        if let Some(byte_pos) = text_lower.find(&query_lower) {
            let char_pos = text[..byte_pos].chars().count();
            let total_chars = text.chars().count();
            let query_char_len = query.chars().count();
            let half = context_chars / 2;
            let start = char_pos.saturating_sub(half);
            let end = (char_pos + query_char_len + half).min(total_chars);
            let snippet: String = text.chars().skip(start).take(end - start).collect();
            let mut result = String::new();
            if start > 0 { result.push('…'); }
            result.push_str(&snippet);
            if end < total_chars { result.push('…'); }
            result
        } else {
            let snippet: String = text.chars().take(80).collect();
            if text.chars().count() > 80 { format!("{}…", snippet) } else { snippet }
        }
    }

    /// Find the first field in `RequestData` that matches `query` and return
    /// (match_field_label, context_snippet).
    fn find_match_in_data(data: &RequestData, query: &str) -> (String, String) {
        let q = query.to_lowercase();

        if data.url.to_lowercase().contains(&q) {
            return ("URL".to_string(), Self::extract_snippet(&data.url, query, 60));
        }
        for h in &data.headers {
            if h.key.to_lowercase().contains(&q) || h.value.to_lowercase().contains(&q) {
                let val_preview: String = h.value.chars().take(40).collect();
                return ("Header".to_string(), format!("{}: {}", h.key, val_preview));
            }
        }
        for p in &data.query_params {
            if p.key.to_lowercase().contains(&q) || p.value.to_lowercase().contains(&q) {
                return ("Query Param".to_string(), format!("{}={}", p.key, p.value));
            }
        }
        for p in &data.path_params {
            if p.key.to_lowercase().contains(&q) || p.value.to_lowercase().contains(&q) {
                return ("Path Param".to_string(), format!(":{}={}", p.key, p.value));
            }
        }
        if !data.body.is_empty() && data.body.to_lowercase().contains(&q) {
            return ("Body".to_string(), Self::extract_snippet(&data.body, query, 60));
        }
        for f in &data.multipart_fields {
            if f.key.to_lowercase().contains(&q) {
                return ("Multipart".to_string(), format!("{}", f.key));
            }
            if !f.is_file && f.value.to_lowercase().contains(&q) {
                return ("Multipart".to_string(), format!("{}={}", f.key, f.value));
            }
            if f.is_file {
                // Match against file name only, not full path
                let file_name = std::path::Path::new(&f.file_path)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_lowercase())
                    .unwrap_or_default();
                if file_name.contains(&q) || f.file_path.to_lowercase().contains(&q) {
                    let name = std::path::Path::new(&f.file_path)
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| f.file_path.clone());
                    return ("Multipart File".to_string(), format!("{}: {}", f.key, name));
                }
            }
        }
        (String::new(), String::new())
    }

    /// Find the first field in a response that matches `query`.
    fn find_match_in_response(response: &ResponseData, body: &str, query: &str) -> (String, String) {
        let q = query.to_lowercase();
        let status_str = format!("{} {}", response.status, response.status_text);
        if status_str.to_lowercase().contains(&q) {
            return ("Status".to_string(), status_str);
        }
        for (k, v) in &response.headers {
            if k.to_lowercase().contains(&q) || v.to_lowercase().contains(&q) {
                let val_preview: String = v.chars().take(40).collect();
                return ("Response Header".to_string(), format!("{}: {}", k, val_preview));
            }
        }
        if !body.is_empty() && body.to_lowercase().contains(&q) {
            return ("Response Body".to_string(), Self::extract_snippet(body, query, 60));
        }
        (String::new(), String::new())
    }

    /// Full-text search across requests, all versions, and execution history.
    pub fn search_all(&self, query: &str, limit: usize) -> rusqlite::Result<Vec<SearchHit>> {
        if query.trim().is_empty() {
            return Ok(vec![]);
        }
        let q_pat = format!("%{}%", query.to_lowercase());
        let mut hits: Vec<SearchHit> = Vec::new();

        // ── 1. Request names ──────────────────────────────────────
        {
            let mut stmt = self.conn.prepare(
                "SELECT r.id, r.name, r.collection_id, c.name, r.current_version_id
                 FROM requests r
                 JOIN collections c ON r.collection_id = c.id
                 WHERE LOWER(r.name) LIKE ?1
                 LIMIT 20",
            )?;
            let rows = stmt.query_map(params![q_pat], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, Option<String>>(4)?,
                ))
            })?;
            for (req_id, req_name, col_id, col_name, cur_vid) in rows.filter_map(|r| r.ok()) {
                let (method, url) = if let Some(ref vid) = cur_vid {
                    self.get_version(vid)
                        .map(|v| (Some(v.data.method.to_string()), Some(v.data.url)))
                        .unwrap_or((None, None))
                } else {
                    (None, None)
                };
                hits.push(SearchHit {
                    result_type: "request".to_string(),
                    request_id: req_id,
                    request_name: req_name.clone(),
                    collection_id: col_id,
                    collection_name: col_name,
                    version_id: cur_vid,
                    execution_id: None,
                    match_field: "Name".to_string(),
                    match_context: req_name,
                    method,
                    url,
                    executed_at: None,
                    status: None,
                });
            }
        }

        // ── 2. Collection names ───────────────────────────────────
        {
            let mut stmt = self.conn.prepare(
                "SELECT id, name, base_path FROM collections
                 WHERE LOWER(name) LIKE ?1 OR LOWER(base_path) LIKE ?1
                 LIMIT 10",
            )?;
            let rows = stmt.query_map(params![q_pat], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?))
            })?;
            for (col_id, col_name, base_path) in rows.filter_map(|r| r.ok()) {
                let (mf, mc) = if col_name.to_lowercase().contains(&query.to_lowercase()) {
                    ("Name".to_string(), col_name.clone())
                } else {
                    ("Base Path".to_string(), Self::extract_snippet(&base_path, query, 60))
                };
                hits.push(SearchHit {
                    result_type: "collection".to_string(),
                    request_id: String::new(),
                    request_name: col_name.clone(),
                    collection_id: col_id,
                    collection_name: col_name,
                    version_id: None,
                    execution_id: None,
                    match_field: mf,
                    match_context: mc,
                    method: None,
                    url: None,
                    executed_at: None,
                    status: None,
                });
            }
        }

        // ── 3. Version data (URL, headers, params, body) ──────────
        {
            let mut stmt = self.conn.prepare(
                "SELECT v.id, v.request_id, v.data_json,
                        r.name, r.current_version_id, r.collection_id, c.name
                 FROM request_versions v
                 JOIN requests r ON v.request_id = r.id
                 JOIN collections c ON r.collection_id = c.id
                 WHERE LOWER(v.data_json) LIKE ?1
                 ORDER BY (CASE WHEN v.id = r.current_version_id THEN 0 ELSE 1 END),
                          v.created_at DESC
                 LIMIT 60",
            )?;
            let rows = stmt.query_map(params![q_pat], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                ))
            })?;

            let mut req_version_count: std::collections::HashMap<String, usize> =
                std::collections::HashMap::new();

            for (vid, req_id, data_json, req_name, cur_vid, col_id, col_name) in
                rows.filter_map(|r| r.ok())
            {
                let cnt = req_version_count.entry(req_id.clone()).or_insert(0);
                if *cnt >= 3 { continue; }

                let data: RequestData = match serde_json::from_str(&data_json) {
                    Ok(d) => d,
                    Err(_) => continue,
                };

                let (match_field, match_context) = Self::find_match_in_data(&data, query);
                if match_context.is_empty() { continue; }

                *cnt += 1;
                let is_current = cur_vid.as_deref() == Some(vid.as_str());
                hits.push(SearchHit {
                    result_type: if is_current { "version".to_string() } else { "version_old".to_string() },
                    request_id: req_id,
                    request_name: req_name,
                    collection_id: col_id,
                    collection_name: col_name,
                    version_id: Some(vid),
                    execution_id: None,
                    match_field,
                    match_context,
                    method: Some(data.method.to_string()),
                    url: Some(data.url),
                    executed_at: None,
                    status: None,
                });
            }
        }

        // ── 4. Execution responses + request data ──────────────────
        {
            let mut stmt = self.conn.prepare(
                "SELECT e.id, e.version_id, e.request_id, e.response_json, e.executed_at,
                        COALESCE(rb.body, '') as body,
                        r.name, r.collection_id, c.name,
                        e.request_data_json
                 FROM request_executions e
                 JOIN requests r ON e.request_id = r.id
                 JOIN collections c ON r.collection_id = c.id
                 LEFT JOIN response_bodies rb ON e.body_hash = rb.hash
                 WHERE LOWER(e.response_json) LIKE ?1
                    OR LOWER(COALESCE(rb.body, '')) LIKE ?1
                    OR LOWER(e.request_data_json) LIKE ?1
                 ORDER BY e.executed_at DESC
                 LIMIT 60",
            )?;
            let rows = stmt.query_map(params![q_pat], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, String>(7)?,
                    row.get::<_, String>(8)?,
                    row.get::<_, String>(9)?,
                ))
            })?;

            let mut req_exec_count: std::collections::HashMap<String, usize> =
                std::collections::HashMap::new();

            for row in rows.filter_map(|r| r.ok()) {
                let (exec_id, vid, req_id, response_json, executed_at, body, req_name, col_id, col_name, request_data_json) = row;
                let cnt = req_exec_count.entry(req_id.clone()).or_insert(0);
                if *cnt >= 5 { continue; }

                let response: ResponseData = serde_json::from_str(&response_json)
                    .unwrap_or_else(|_| ResponseData {
                        status: 0,
                        status_text: String::new(),
                        headers: Default::default(),
                        body: String::new(),
                        size_bytes: 0,
                        is_binary: false,
                    });

                // Check response match
                let (match_field, match_context) =
                    Self::find_match_in_response(&response, &body, query);

                // If no response match, check execution request data
                let (match_field, match_context) = if match_context.is_empty() && !request_data_json.is_empty() {
                    if let Ok(req_data) = serde_json::from_str::<RequestData>(&request_data_json) {
                        let (mf, mc) = Self::find_match_in_data(&req_data, query);
                        if mc.is_empty() { continue; }
                        (format!("Exec {}", mf), mc)
                    } else {
                        continue;
                    }
                } else if match_context.is_empty() {
                    continue;
                } else {
                    (match_field, match_context)
                };

                *cnt += 1;
                let status = response.status;
                // Try to get method/url from execution request data
                let (method, url) = if !request_data_json.is_empty() {
                    if let Ok(rd) = serde_json::from_str::<RequestData>(&request_data_json) {
                        (Some(rd.method.to_string()), Some(rd.url))
                    } else {
                        (None, None)
                    }
                } else {
                    (None, None)
                };
                hits.push(SearchHit {
                    result_type: "execution".to_string(),
                    request_id: req_id,
                    request_name: req_name,
                    collection_id: col_id,
                    collection_name: col_name,
                    version_id: Some(vid),
                    execution_id: Some(exec_id),
                    match_field,
                    match_context,
                    method,
                    url,
                    executed_at: Some(executed_at),
                    status: Some(status),
                });
            }
        }

        hits.truncate(limit);
        Ok(hits)
    }
}

impl Database {
    // ── Trash (soft-delete) ──────────────────────────────────────

    /// Returns all soft-deleted items across collections, folders, and requests.
    /// Folders/requests inside an already-deleted collection are excluded (they
    /// will be restored/purged together with their parent collection).
    pub fn list_trash(&self) -> rusqlite::Result<Vec<TrashedItem>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, 'collection' AS item_type, name, '' AS parent_name, deleted_at
               FROM collections
              WHERE deleted_at IS NOT NULL
             UNION ALL
             SELECT f.id, 'folder', f.name, COALESCE(c.name,''), f.deleted_at
               FROM folders f
               LEFT JOIN collections c ON c.id = f.collection_id
              WHERE f.deleted_at IS NOT NULL
                AND (c.deleted_at IS NULL)
             UNION ALL
             SELECT r.id, 'request', r.name, COALESCE(c.name,''), r.deleted_at
               FROM requests r
               LEFT JOIN collections c ON c.id = r.collection_id
               LEFT JOIN folders fo ON fo.id = r.folder_id
              WHERE r.deleted_at IS NOT NULL
                AND (c.deleted_at IS NULL)
                AND (fo.id IS NULL OR fo.deleted_at IS NULL)
             ORDER BY deleted_at DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(TrashedItem {
                id: row.get(0)?,
                item_type: row.get(1)?,
                name: row.get(2)?,
                parent_name: row.get(3)?,
                deleted_at: row.get(4)?,
            })
        })?;
        rows.collect()
    }

    pub fn restore_item(&self, item_type: &str, id: &str) -> rusqlite::Result<()> {
        let sql = match item_type {
            "collection" => "UPDATE collections SET deleted_at=NULL WHERE id=?1",
            "folder"     => "UPDATE folders SET deleted_at=NULL WHERE id=?1",
            "request"    => "UPDATE requests SET deleted_at=NULL WHERE id=?1",
            _            => return Ok(()),
        };
        self.conn.execute(sql, params![id])?;
        Ok(())
    }

    pub fn purge_item(&self, item_type: &str, id: &str) -> rusqlite::Result<()> {
        let sql = match item_type {
            "collection" => "DELETE FROM collections WHERE id=?1",
            "folder"     => "DELETE FROM folders WHERE id=?1",
            "request"    => "DELETE FROM requests WHERE id=?1",
            _            => return Ok(()),
        };
        self.conn.execute(sql, params![id])?;
        Ok(())
    }

    pub fn empty_trash(&self) -> rusqlite::Result<()> {
        self.conn.execute_batch(
            "DELETE FROM requests    WHERE deleted_at IS NOT NULL;
             DELETE FROM folders     WHERE deleted_at IS NOT NULL;
             DELETE FROM collections WHERE deleted_at IS NOT NULL;",
        )?;
        Ok(())
    }

    // ── Clone ─────────────────────────────────────────────────────

    /// Clone a request and its current version into the same collection/folder.
    /// Returns the new request id.
    pub fn clone_request(&self, source_id: &str) -> rusqlite::Result<String> {
        // Load source request
        let source = self.conn.query_row(
            "SELECT id, collection_id, folder_id, name, current_version_id, sort_order
             FROM requests WHERE id=?1 AND deleted_at IS NULL",
            params![source_id],
            |row| Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, i32>(5)?,
            )),
        )?;

        let new_req_id = uuid();
        let new_name = format!("{} Copy", source.3);

        self.conn.execute(
            "INSERT INTO requests (id, collection_id, folder_id, name, current_version_id, sort_order)
             VALUES (?1, ?2, ?3, ?4, NULL, ?5)",
            params![new_req_id, source.1, source.2, new_name, source.5 + 1],
        )?;

        // Clone current version if present
        if let Some(ver_id) = &source.4 {
            let ver = self.get_version(ver_id)?;
            let new_ver_id = uuid();
            let data_json = serde_json::to_string(&ver.data).unwrap_or_default();
            let fingerprint = ver.data.fingerprint();
            self.conn.execute(
                "INSERT INTO request_versions (id, request_id, data_json, fingerprint, created_at)
                 VALUES (?1, ?2, ?3, ?4, datetime('now'))",
                params![new_ver_id, new_req_id, data_json, fingerprint],
            )?;
            self.conn.execute(
                "UPDATE requests SET current_version_id=?2 WHERE id=?1",
                params![new_req_id, new_ver_id],
            )?;
        }

        Ok(new_req_id)
    }

    /// Clone a folder (and all its sub-folders and requests) into the same parent.
    /// Returns the new folder id.
    pub fn clone_folder(&self, source_id: &str) -> rusqlite::Result<String> {
        self.clone_folder_into(source_id, None, None)
    }

    fn clone_folder_into(
        &self,
        source_id: &str,
        new_collection_id: Option<&str>,
        new_parent_id: Option<&str>,
    ) -> rusqlite::Result<String> {
        let source = self.conn.query_row(
            "SELECT id, collection_id, parent_folder_id, name, path_prefix, auth_override, sort_order
             FROM folders WHERE id=?1 AND deleted_at IS NULL",
            params![source_id],
            |row| Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, i32>(6)?,
            )),
        )?;

        let col_id = new_collection_id.unwrap_or(&source.1).to_string();
        let parent_id: Option<String> = match new_parent_id {
            Some(p) => Some(p.to_string()),
            None => source.2.clone(),
        };
        let new_folder_id = uuid();
        let new_name = if new_parent_id.is_none() && new_collection_id.is_none() {
            format!("{} Copy", source.3)
        } else {
            source.3.clone()
        };

        self.conn.execute(
            "INSERT INTO folders (id, collection_id, parent_folder_id, name, path_prefix, auth_override, sort_order)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![new_folder_id, col_id, parent_id, new_name, source.4, source.5, source.6 + 1],
        )?;

        // Clone sub-folders
        let sub_folders: Vec<String> = {
            let mut stmt = self.conn.prepare(
                "SELECT id FROM folders WHERE parent_folder_id=?1 AND deleted_at IS NULL ORDER BY sort_order",
            )?;
            let rows = stmt.query_map(params![source_id], |r| r.get::<_, String>(0))?;
            rows.collect::<rusqlite::Result<Vec<_>>>()?
        };
        for sf_id in sub_folders {
            self.clone_folder_into(&sf_id, Some(&col_id), Some(&new_folder_id))?;
        }

        // Clone requests in this folder
        let req_ids: Vec<String> = {
            let mut stmt = self.conn.prepare(
                "SELECT id FROM requests WHERE folder_id=?1 AND deleted_at IS NULL ORDER BY sort_order",
            )?;
            let rows = stmt.query_map(params![source_id], |r| r.get::<_, String>(0))?;
            rows.collect::<rusqlite::Result<Vec<_>>>()?
        };
        for req_id in req_ids {
            let new_req_id = self.clone_request(&req_id)?;
            // Re-parent to new folder and collection
            self.conn.execute(
                "UPDATE requests SET folder_id=?2, collection_id=?3 WHERE id=?1",
                params![new_req_id, new_folder_id, col_id],
            )?;
        }

        Ok(new_folder_id)
    }
}
