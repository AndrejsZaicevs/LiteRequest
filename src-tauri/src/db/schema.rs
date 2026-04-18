use rusqlite::Connection;

pub fn initialize(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch("PRAGMA journal_mode=WAL;")?;
    conn.execute_batch("PRAGMA foreign_keys=ON;")?;

    // Schema version tracking
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        );"
    )?;

    let current_version: i64 = conn
        .query_row("SELECT COALESCE(MAX(version), 0) FROM schema_version", [], |r| r.get(0))
        .unwrap_or(0);

    // Version 0: base schema (always idempotent via IF NOT EXISTS)
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS collections (
            id              TEXT PRIMARY KEY,
            name            TEXT NOT NULL,
            base_path       TEXT NOT NULL DEFAULT '',
            auth_config     TEXT,
            headers_config  TEXT,
            created_at      TEXT NOT NULL,
            updated_at      TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS folders (
            id               TEXT PRIMARY KEY,
            collection_id    TEXT NOT NULL REFERENCES collections(id) ON DELETE CASCADE,
            parent_folder_id TEXT REFERENCES folders(id) ON DELETE CASCADE,
            name             TEXT NOT NULL,
            path_prefix      TEXT NOT NULL DEFAULT '',
            auth_override    TEXT,
            sort_order       INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS requests (
            id                 TEXT PRIMARY KEY,
            collection_id      TEXT NOT NULL REFERENCES collections(id) ON DELETE CASCADE,
            folder_id          TEXT REFERENCES folders(id) ON DELETE SET NULL,
            name               TEXT NOT NULL,
            current_version_id TEXT,
            sort_order         INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS request_versions (
            id         TEXT PRIMARY KEY,
            request_id TEXT NOT NULL REFERENCES requests(id) ON DELETE CASCADE,
            data_json  TEXT NOT NULL,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS response_bodies (
            hash TEXT PRIMARY KEY,
            body TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS request_executions (
            id            TEXT PRIMARY KEY,
            version_id    TEXT NOT NULL REFERENCES request_versions(id) ON DELETE CASCADE,
            request_id    TEXT NOT NULL REFERENCES requests(id) ON DELETE CASCADE,
            response_json TEXT NOT NULL,
            latency_ms    INTEGER NOT NULL,
            executed_at   TEXT NOT NULL,
            environment_id TEXT NOT NULL DEFAULT '',
            body_hash      TEXT NOT NULL DEFAULT ''
        );

        CREATE TABLE IF NOT EXISTS environments (
            id         TEXT PRIMARY KEY,
            name       TEXT NOT NULL,
            is_active  INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS env_variables (
            id             TEXT PRIMARY KEY,
            environment_id TEXT NOT NULL REFERENCES environments(id) ON DELETE CASCADE,
            key            TEXT NOT NULL,
            value          TEXT NOT NULL DEFAULT '',
            is_secret      INTEGER NOT NULL DEFAULT 0
        );

        CREATE INDEX IF NOT EXISTS idx_folders_collection ON folders(collection_id);
        CREATE INDEX IF NOT EXISTS idx_requests_collection ON requests(collection_id);
        CREATE INDEX IF NOT EXISTS idx_requests_folder ON requests(folder_id);
        CREATE INDEX IF NOT EXISTS idx_versions_request ON request_versions(request_id);
        CREATE INDEX IF NOT EXISTS idx_executions_request ON request_executions(request_id);
        CREATE INDEX IF NOT EXISTS idx_executions_version ON request_executions(version_id);
        CREATE INDEX IF NOT EXISTS idx_env_vars_env ON env_variables(environment_id);
        CREATE INDEX IF NOT EXISTS idx_executions_env ON request_executions(environment_id);
        CREATE INDEX IF NOT EXISTS idx_executions_body_hash ON request_executions(body_hash);
        CREATE INDEX IF NOT EXISTS idx_versions_request_fp ON request_versions(request_id, fingerprint);
        CREATE INDEX IF NOT EXISTS idx_executions_req_date ON request_executions(request_id, executed_at DESC);

        CREATE TABLE IF NOT EXISTS collection_variables (
            id             TEXT PRIMARY KEY,
            collection_id  TEXT NOT NULL REFERENCES collections(id) ON DELETE CASCADE,
            environment_id TEXT NOT NULL REFERENCES environments(id) ON DELETE CASCADE,
            key            TEXT NOT NULL,
            value          TEXT NOT NULL DEFAULT '',
            is_secret      INTEGER NOT NULL DEFAULT 0
        );
        CREATE INDEX IF NOT EXISTS idx_coll_vars_coll_env ON collection_variables(collection_id, environment_id);

        CREATE TABLE IF NOT EXISTS collection_var_defs (
            id             TEXT PRIMARY KEY,
            collection_id  TEXT NOT NULL REFERENCES collections(id) ON DELETE CASCADE,
            key            TEXT NOT NULL,
            sort_order     INTEGER NOT NULL DEFAULT 0
        );
        CREATE INDEX IF NOT EXISTS idx_var_defs_coll ON collection_var_defs(collection_id);

        CREATE TABLE IF NOT EXISTS collection_var_values (
            id             TEXT PRIMARY KEY,
            def_id         TEXT NOT NULL REFERENCES collection_var_defs(id) ON DELETE CASCADE,
            environment_id TEXT NOT NULL REFERENCES environments(id) ON DELETE CASCADE,
            value          TEXT NOT NULL DEFAULT '',
            is_secret      INTEGER NOT NULL DEFAULT 0,
            UNIQUE(def_id, environment_id)
        );
        CREATE INDEX IF NOT EXISTS idx_var_values_def ON collection_var_values(def_id);
        ",
    )?;

    // ── Versioned migrations ────────────────────────────────────
    // Each migration only runs once; version is recorded on success.

    if current_version < 1 {
        migrate_collection_variables(conn);
        migrate_add_version_fingerprint(conn);
        migrate_add_execution_request_data(conn);
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS app_settings (
                key   TEXT PRIMARY KEY,
                value TEXT NOT NULL DEFAULT ''
            );",
        )?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS env_var_defs (
                id         TEXT PRIMARY KEY,
                key        TEXT NOT NULL,
                sort_order INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE IF NOT EXISTS env_var_values (
                id             TEXT PRIMARY KEY,
                def_id         TEXT NOT NULL REFERENCES env_var_defs(id) ON DELETE CASCADE,
                environment_id TEXT NOT NULL REFERENCES environments(id) ON DELETE CASCADE,
                value          TEXT NOT NULL DEFAULT '',
                is_secret      INTEGER NOT NULL DEFAULT 0,
                UNIQUE(def_id, environment_id)
            );
            CREATE INDEX IF NOT EXISTS idx_env_var_values_def ON env_var_values(def_id);
            CREATE INDEX IF NOT EXISTS idx_env_var_values_env ON env_var_values(environment_id);
            ",
        )?;
        migrate_env_variables(conn);
        migrate_environment_sort_order(conn);
        migrate_add_soft_delete(conn);
        migrate_var_type(conn);
        conn.execute("INSERT INTO schema_version (version) VALUES (1)", [])?;
    }

    // Future migrations go here:
    // if current_version < 2 { ... conn.execute("INSERT INTO schema_version (version) VALUES (2)", [])?; }

    if current_version < 2 {
        migrate_add_scripting(conn)?;
        conn.execute("INSERT INTO schema_version (version) VALUES (2)", [])?;
    }

    Ok(())
}

/// Add `deleted_at` column to collections, folders, and requests for soft-delete support.
fn migrate_add_soft_delete(conn: &Connection) {
    for ddl in &[
        "ALTER TABLE collections ADD COLUMN deleted_at TEXT",
        "ALTER TABLE folders ADD COLUMN deleted_at TEXT",
        "ALTER TABLE requests ADD COLUMN deleted_at TEXT",
    ] {
        // Ignore "duplicate column" errors — column already exists
        let _ = conn.execute_batch(ddl);
    }
    // Indexes for soft-delete filtered queries (idempotent via IF NOT EXISTS)
    let _ = conn.execute_batch(
        "CREATE INDEX IF NOT EXISTS idx_collections_deleted ON collections(deleted_at);
         CREATE INDEX IF NOT EXISTS idx_folders_deleted ON folders(deleted_at, collection_id);
         CREATE INDEX IF NOT EXISTS idx_requests_deleted ON requests(deleted_at, collection_id);",
    );
}

/// Migrate data from the legacy `collection_variables` table into the
/// new `collection_var_defs` + `collection_var_values` split tables.
/// Only runs if the old table has data and the new table is empty.
fn migrate_collection_variables(conn: &Connection) {
    // Check if old table has rows and new table is empty
    let old_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM collection_variables", [], |r| r.get(0))
        .unwrap_or(0);
    let new_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM collection_var_defs", [], |r| r.get(0))
        .unwrap_or(0);

    if old_count == 0 || new_count > 0 {
        return;
    }

    // Read all old rows
    let mut stmt = conn
        .prepare("SELECT collection_id, environment_id, key, value, is_secret FROM collection_variables ORDER BY key")
        .unwrap();
    let rows: Vec<(String, String, String, String, bool)> = stmt
        .query_map([], |row| {
            let is_secret: i32 = row.get(4)?;
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                is_secret != 0,
            ))
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    // Build def map: (collection_id, key) → def_id
    let mut def_map: std::collections::HashMap<(String, String), String> =
        std::collections::HashMap::new();
    let mut sort = 0i32;

    for (cid, _eid, key, _val, _secret) in &rows {
        if key.is_empty() {
            continue;
        }
        let map_key = (cid.clone(), key.clone());
        if !def_map.contains_key(&map_key) {
            let def_id = uuid::Uuid::new_v4().to_string();
            let _ = conn.execute(
                "INSERT INTO collection_var_defs (id, collection_id, key, sort_order) VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![def_id, cid, key, sort],
            );
            def_map.insert(map_key, def_id);
            sort += 1;
        }
    }

    // Insert values
    for (cid, eid, key, val, secret) in &rows {
        if key.is_empty() {
            continue;
        }
        let map_key = (cid.clone(), key.clone());
        if let Some(def_id) = def_map.get(&map_key) {
            let val_id = uuid::Uuid::new_v4().to_string();
            let _ = conn.execute(
                "INSERT OR IGNORE INTO collection_var_values (id, def_id, environment_id, value, is_secret) VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![val_id, def_id, eid, val, *secret as i32],
            );
        }
    }
}

/// Add `fingerprint` column to `request_versions` and backfill from data_json.
fn migrate_add_version_fingerprint(conn: &Connection) {
    // Check if column already exists
    let has_col: bool = conn
        .prepare("SELECT fingerprint FROM request_versions LIMIT 0")
        .is_ok();
    if has_col {
        return;
    }

    let _ = conn.execute_batch(
        "ALTER TABLE request_versions ADD COLUMN fingerprint TEXT NOT NULL DEFAULT '';"
    );

    // Backfill fingerprints from existing data_json
    let mut stmt = conn
        .prepare("SELECT id, data_json FROM request_versions")
        .unwrap();
    let rows: Vec<(String, String)> = stmt
        .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    for (id, data_json) in rows {
        if let Ok(data) = serde_json::from_str::<crate::models::RequestData>(&data_json) {
            let fp = data.fingerprint();
            let _ = conn.execute(
                "UPDATE request_versions SET fingerprint=?2 WHERE id=?1",
                rusqlite::params![id, fp],
            );
        }
    }
}

/// Add `request_data_json` column to `request_executions`.
fn migrate_add_execution_request_data(conn: &Connection) {
    let has_col: bool = conn
        .prepare("SELECT request_data_json FROM request_executions LIMIT 0")
        .is_ok();
    if has_col {
        return;
    }

    let _ = conn.execute_batch(
        "ALTER TABLE request_executions ADD COLUMN request_data_json TEXT NOT NULL DEFAULT '';"
    );
}

/// Migrate legacy `env_variables` (per-env rows) into the split
/// `env_var_defs` (global keys) + `env_var_values` (per-env values) tables.
/// Only runs once — when env_var_defs is empty but env_variables has data.
fn migrate_env_variables(conn: &Connection) {
    let new_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM env_var_defs", [], |r| r.get(0))
        .unwrap_or(0);
    if new_count > 0 {
        return; // already migrated
    }

    let old_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM env_variables", [], |r| r.get(0))
        .unwrap_or(0);
    if old_count == 0 {
        return; // nothing to migrate
    }

    let mut stmt = conn
        .prepare("SELECT environment_id, key, value, is_secret FROM env_variables WHERE key != '' ORDER BY key")
        .unwrap();
    let rows: Vec<(String, String, String, bool)> = stmt
        .query_map([], |row| {
            let is_secret: i32 = row.get(3)?;
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, is_secret != 0))
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    // Build def map: key → def_id
    let mut def_map: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let mut sort = 0i32;

    for (_eid, key, _val, _secret) in &rows {
        if !def_map.contains_key(key) {
            let def_id = uuid::Uuid::new_v4().to_string();
            let _ = conn.execute(
                "INSERT INTO env_var_defs (id, key, sort_order) VALUES (?1, ?2, ?3)",
                rusqlite::params![def_id, key, sort],
            );
            def_map.insert(key.clone(), def_id);
            sort += 1;
        }
    }

    for (eid, key, val, secret) in &rows {
        if let Some(def_id) = def_map.get(key) {
            let val_id = uuid::Uuid::new_v4().to_string();
            let _ = conn.execute(
                "INSERT OR IGNORE INTO env_var_values (id, def_id, environment_id, value, is_secret)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![val_id, def_id, eid, val, *secret as i32],
            );
        }
    }
}

/// Add `var_type` column to `collection_var_defs` (idempotent).
fn migrate_var_type(conn: &Connection) {
    let has_col: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('collection_var_defs') WHERE name='var_type'",
            [],
            |r| r.get::<_, i64>(0),
        )
        .unwrap_or(0)
        > 0;
    if has_col {
        return;
    }
    let _ = conn.execute_batch(
        "ALTER TABLE collection_var_defs ADD COLUMN var_type TEXT NOT NULL DEFAULT 'regular';",
    );
}

/// Add sort_order column to environments table (idempotent).
fn migrate_environment_sort_order(conn: &Connection) {
    let has_col: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('environments') WHERE name='sort_order'",
            [],
            |r| r.get::<_, i64>(0),
        )
        .unwrap_or(0)
        > 0;
    if has_col {
        return;
    }
    let _ = conn.execute_batch(
        "ALTER TABLE environments ADD COLUMN sort_order INTEGER NOT NULL DEFAULT 0;
         UPDATE environments SET sort_order = rowid;",
    );
}

/// Add scripting tables: post_script on requests, scripts + script_versions + script_runs.
fn migrate_add_scripting(conn: &Connection) -> rusqlite::Result<()> {
    // post_script column on requests (for post-execution scripts)
    let _ = conn.execute_batch(
        "ALTER TABLE requests ADD COLUMN post_script TEXT NOT NULL DEFAULT '';"
    );

    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS scripts (
            id                 TEXT PRIMARY KEY,
            collection_id      TEXT NOT NULL REFERENCES collections(id) ON DELETE CASCADE,
            folder_id          TEXT REFERENCES folders(id) ON DELETE SET NULL,
            name               TEXT NOT NULL,
            current_version_id TEXT,
            sort_order         INTEGER NOT NULL DEFAULT 0,
            created_at         TEXT NOT NULL,
            updated_at         TEXT NOT NULL,
            deleted_at         TEXT
        );
        CREATE INDEX IF NOT EXISTS idx_scripts_collection ON scripts(collection_id);
        CREATE INDEX IF NOT EXISTS idx_scripts_deleted ON scripts(deleted_at, collection_id);

        CREATE TABLE IF NOT EXISTS script_versions (
            id          TEXT PRIMARY KEY,
            script_id   TEXT NOT NULL REFERENCES scripts(id) ON DELETE CASCADE,
            content_ts  TEXT NOT NULL DEFAULT '',
            content_js  TEXT NOT NULL DEFAULT '',
            created_at  TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_script_versions_script ON script_versions(script_id);

        CREATE TABLE IF NOT EXISTS script_runs (
            id             TEXT PRIMARY KEY,
            script_id      TEXT,
            version_id     TEXT,
            request_id     TEXT,
            execution_id   TEXT,
            status         TEXT NOT NULL,
            logs           TEXT NOT NULL DEFAULT '[]',
            variables_set  TEXT NOT NULL DEFAULT '{}',
            script_source  TEXT NOT NULL DEFAULT '',
            error          TEXT,
            duration_ms    INTEGER NOT NULL DEFAULT 0,
            executed_at    TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_script_runs_script ON script_runs(script_id);
        CREATE INDEX IF NOT EXISTS idx_script_runs_version ON script_runs(version_id);
        CREATE INDEX IF NOT EXISTS idx_script_runs_request ON script_runs(request_id);
        CREATE INDEX IF NOT EXISTS idx_script_runs_execution ON script_runs(execution_id);
        ",
    )?;
    Ok(())
}
