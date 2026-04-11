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
        ",
    )?;

    // Migration: add headers_config column to existing databases
    let _ = conn.execute_batch(
        "ALTER TABLE collections ADD COLUMN headers_config TEXT;",
    );

    // Global app settings (key-value store for JSON blobs)
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS app_settings (
            key   TEXT PRIMARY KEY,
            value TEXT NOT NULL DEFAULT ''
        );",
    )?;

    Ok(())
}
