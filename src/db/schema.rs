use rusqlite::Connection;

pub fn initialize(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch("PRAGMA journal_mode=WAL;")?;
    conn.execute_batch("PRAGMA foreign_keys=ON;")?;

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

        CREATE TABLE IF NOT EXISTS request_executions (
            id            TEXT PRIMARY KEY,
            version_id    TEXT NOT NULL REFERENCES request_versions(id) ON DELETE CASCADE,
            request_id    TEXT NOT NULL REFERENCES requests(id) ON DELETE CASCADE,
            response_json TEXT NOT NULL,
            latency_ms    INTEGER NOT NULL,
            executed_at   TEXT NOT NULL
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

    // Migration: add headers_config column to existing databases
    let _ = conn.execute_batch(
        "ALTER TABLE collections ADD COLUMN headers_config TEXT;",
    );

    // Migrate old collection_variables → new split tables
    migrate_collection_variables(conn);

    // Global app settings (key-value store for JSON blobs)
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS app_settings (
            key   TEXT PRIMARY KEY,
            value TEXT NOT NULL DEFAULT ''
        );",
    )?;

    Ok(())
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
