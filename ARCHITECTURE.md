# LiteRequest — Architecture & Developer Guide

This document gives a new contributor (human or AI agent) all the context needed to understand, navigate and modify the codebase.

---

## 1. High-Level Architecture

LiteRequest is a **Tauri v2** desktop app: a Rust backend manages all data and HTTP execution, a React/TypeScript frontend handles the UI, and the two communicate through Tauri's IPC invoke system.

```
┌──────────────────────────────────────────────────┐
│  WebView (React 19 + TypeScript + Tailwind v4)   │
│  src-web/                                        │
│    App.tsx ─── components/ ─── lib/              │
└──────────────────┬───────────────────────────────┘
                   │  tauri::invoke("command", args)
                   │  (JSON serialized over IPC)
┌──────────────────▼───────────────────────────────┐
│  Rust Backend (Tauri v2)                         │
│  src-tauri/src/                                  │
│    lib.rs ──── commands.rs ──── db/ ──── http/   │
│                                 models/  import/ │
└──────────────────────────────────────────────────┘
                   │
           ┌───────┴────────┐
           │  SQLite (WAL)  │  literequest.db
           │  + OS Keychain │  (secrets only)
           └────────────────┘
```

### Data flow for a typical "Send request"

1. Frontend calls `api.executeRequest(...)` → `invoke("execute_request", { ... })`
2. `commands::execute_request` receives args, acquires `AppState.db` lock
3. Variables are interpolated (`http::interpolation`), auth headers injected
4. `http::client::execute` sends the request via `reqwest`
5. Response is stored as a `RequestExecution` row in SQLite
6. Large response bodies are deduplicated via SHA-256 hash into `response_bodies`
7. The command returns `(ResponseData, latency_ms, execution_id)` to the frontend
8. Frontend updates state → UI re-renders

---

## 2. Repository Layout

```
liteRequest/
├── src-tauri/                  # Rust backend (Tauri)
│   ├── Cargo.toml              # Rust dependencies
│   ├── tauri.conf.json         # Tauri config (window, bundle, CSP)
│   ├── build.rs                # Tauri build script
│   └── src/
│       ├── main.rs             # Entry point (calls lib::run())
│       ├── lib.rs              # AppState, command registration, DB init
│       ├── commands.rs         # All #[tauri::command] handlers (815 lines)
│       ├── db/
│       │   ├── schema.rs       # CREATE TABLE + all migrations
│       │   └── operations.rs   # All SQL queries (Database struct, 1500 lines)
│       ├── models/
│       │   ├── collection.rs   # Collection, Folder, TrashedItem, TreeNode
│       │   ├── request.rs      # Request, RequestVersion, RequestData, ResponseData,
│       │   │                   #   RequestExecution, SearchHit, HttpMethod, BodyType,
│       │   │                   #   KeyValuePair, ClientCertEntry, MultipartField
│       │   └── environment.rs  # Environment, EnvVariable, EnvVarDef, VarDef,
│       │                       #   VarValue, VarRow, CollectionVariable (legacy)
│       ├── http/
│       │   ├── client.rs       # reqwest-based HTTP executor
│       │   ├── curl.rs         # cURL import/export parser
│       │   └── interpolation.rs# {{variable}} resolution engine
│       ├── import/
│       │   └── postman.rs      # Postman collection import/export
│       └── utils/
│           └── debounce.rs     # Debounce utility
│
├── src-web/                    # React frontend
│   ├── main.tsx                # React root mount
│   ├── index.html              # HTML entry point
│   ├── index.css               # Global CSS + Tailwind imports
│   ├── App.tsx                 # Root component — ALL app state lives here (829 lines)
│   ├── lib/
│   │   ├── types.ts            # TypeScript types mirroring Rust models
│   │   ├── api.ts              # All invoke() wrappers (one per Tauri command)
│   │   └── dynamicVars.ts      # {{$randomEmail}} etc. dynamic variable generators
│   └── components/
│       ├── layout/
│       │   ├── Sidebar.tsx     # Collection tree, context menus, DnD, rename
│       │   └── Inspector.tsx   # Right panel: params, headers, versions, executions
│       ├── editor/
│       │   ├── UrlBar.tsx      # Method selector + URL input + Send button
│       │   ├── RequestEditor.tsx# Body tabs (JSON, Form, Raw, Multipart), cURL
│       │   └── CodeEditor.tsx  # CodeMirror wrapper: syntax HL, var tooltips, autocomplete
│       ├── response/
│       │   └── ResponseView.tsx# Response body (CM JSON viewer), headers, search, download
│       ├── inspector/
│       │   └── KvTable.tsx     # Reusable key-value table (query params, headers)
│       ├── search/
│       │   └── GlobalSearch.tsx# Ctrl+K modal — searches everything
│       ├── settings/
│       │   ├── AppSettings.tsx # Global settings: envs, vars, headers, certs, cleanup, trash
│       │   └── CollectionConfig.tsx # Per-collection: base URL, auth, headers, variables
│       └── shared/
│           ├── CollapsibleSection.tsx # Reusable collapsible header
│           ├── VarDefTable.tsx       # Variable definition table (used in settings + config)
│           ├── VariableInput.tsx     # Input with {{var}} highlighting
│           └── TooltipPortal.tsx     # App-wide tooltip portal (portals to document.body)
│
├── package.json                # Frontend deps + scripts
├── vite.config.ts              # Vite config
├── tsconfig.json               # TS config
└── README.md                   # User-facing documentation
```

---

## 3. Database Schema

SQLite with WAL mode. Single file at `~/.local/share/LiteRequest/literequest.db` (Linux) or equivalent platform dir.

### Core Tables

| Table | Purpose | Key columns |
|---|---|---|
| `collections` | Top-level grouping | `id, name, base_path, auth_config (JSON), headers_config (JSON), deleted_at` |
| `folders` | Nested within collections | `id, collection_id, parent_folder_id, name, sort_order, deleted_at` |
| `requests` | Individual API requests | `id, collection_id, folder_id, name, current_version_id, sort_order, deleted_at` |
| `request_versions` | Immutable request snapshots | `id, request_id, data_json (JSON), fingerprint, created_at` |
| `request_executions` | Execution history | `id, version_id, request_id, environment_id, response_json, latency_ms, body_hash, request_data_json` |
| `response_bodies` | Deduplicated response bodies | `hash (PK), body` |

### Variable System (Split Model)

Variables use a definition/value split so keys are shared across environments:

| Table | Purpose |
|---|---|
| `env_var_defs` | Global environment variable keys |
| `env_var_values` | Per-environment values for global vars |
| `collection_var_defs` | Per-collection variable keys (has `var_type: regular\|operative`) |
| `collection_var_values` | Per-environment values for collection vars |

### Other Tables

| Table | Purpose |
|---|---|
| `environments` | Named environments (`dev`, `prod`) with `is_active` flag and `sort_order` |
| `app_settings` | Key-value store for JSON blobs (global headers, cert config, UI prefs) |
| `collection_variables` | **Legacy** — migrated to split tables on startup |
| `env_variables` | **Legacy** — migrated to split tables on startup |

### Migrations

All migrations are in `schema.rs` and are **idempotent** — they check for column/data existence before altering. They run on every app start inside `initialize()`. Pattern:

```rust
fn migrate_something(conn: &Connection) {
    // Check if already migrated (pragma_table_info, row count, etc.)
    // If not, ALTER TABLE / INSERT / etc.
}
```

---

## 4. Key Patterns & Conventions

### State Management

**All app state lives in `App.tsx`** — there is no Redux, Zustand, or context. State is passed down as props. Callbacks flow up.

Key state groups in `App.tsx`:
- **Data caches**: `collections`, `folders`, `requests`, `versions`, `executions`, `environments`, `envVariables`
- **Selection**: `centerView`, `currentRequest`, `selectedVersionId`, `selectedExecutionId`
- **Editor**: `editorData` (RequestData), `dirty` flag
- **Response**: `currentResponse`, `currentLatency`, `isLoading`
- **Panel sizing**: `sidebarWidth`, `inspectorWidth`, `splitRatio` (all persisted to localStorage)

### Versioning (Fingerprint-Based)

Versions are created automatically on structural changes. The "structure" = method + URL + enabled param/header keys + body type + multipart field keys. Value-only changes (editing a body, changing a param value) don't create new versions.

```
fingerprint = "POST|/api/users|name,email|content-type|Json|"
```

Both Rust (`RequestData::fingerprint()`) and TypeScript (`computeVersionFingerprint()`) compute the same fingerprint. When saving, if the fingerprint matches the current version, `data_json` is updated in place. If different, a new version row is created.

### Variable Resolution Order

When sending a request, variables are resolved in this order (later wins):

1. **Dynamic variables** — `{{$randomEmail}}`, `{{$timestamp}}`, etc. (generated at send time)
2. **Global variables** — defined in App Settings
3. **Collection variables** — defined per-collection, per-environment
4. **Environment variables** — from the active environment
5. **Built-in**: `{{requestName}}` → current request's name

Resolution happens in `http::interpolation::interpolate()` on the Rust side. The frontend also resolves for display purposes (URL bar preview, tooltips).

### Secret Storage

Variables marked `is_secret` store their values in the **OS keychain** (via the `keyring` crate), not in SQLite. The database stores an empty string; the actual value is fetched from the keychain at read time.

### Soft Delete

Collections, folders, and requests have a `deleted_at` column. Deletion sets this timestamp instead of removing the row. All list queries filter `WHERE deleted_at IS NULL`. The Trash UI in App Settings shows soft-deleted items and allows restore or permanent purge.

### Response Body Deduplication

Response bodies are stored in a separate `response_bodies` table keyed by SHA-256 hash. `request_executions.body_hash` references this. Identical responses across executions share one body row.

---

## 5. Command/API Layer

Every Tauri command follows this pattern:

```rust
// commands.rs
#[tauri::command]
pub fn some_command(state: State<AppState>, arg1: String) -> CmdResult<ReturnType> {
    state.db.lock().unwrap().some_operation(&arg1).map_err(map_err)
}
```

```typescript
// api.ts
export const someCommand = (arg1: string) =>
  invoke<ReturnType>("some_command", { arg1 });
```

**Naming convention**: Rust uses `snake_case` commands, TypeScript uses `camelCase` wrappers. Tauri auto-converts between them.

Commands are registered in `lib.rs` → `invoke_handler(generate_handler![...])`. **All commands must be listed there** or they won't be callable from the frontend.

### Command Groups (as listed in lib.rs)

| Group | Commands |
|---|---|
| Collections | `list_collections`, `insert_collection`, `update_collection`, `delete_collection`, `rename_collection` |
| Folders | `list_folders`, `insert_folder`, `delete_folder`, `rename_folder`, `move_folder` |
| Requests | `list_requests_by_collection`, `list_requests_by_folder`, `list_orphan_requests`, `insert_request`, `rename_request`, `delete_request`, `move_request`, `update_request_version`, `reorder_*` |
| Versions | `insert_version`, `get_version`, `list_versions`, `update_version_data`, `delete_version`, `save_version`, `version_has_executions` |
| Executions | `insert_execution`, `list_executions` |
| Environments | `list_environments`, `insert_environment`, `set_active_environment`, `rename_environment`, `delete_environment` |
| Env Variables | `list_env_variables`, `insert_env_variable`, `update_env_variable`, `delete_env_variable`, `get_active_variables` |
| Env Var Defs | `list_env_var_defs`, `insert_env_var_def`, `update_env_var_def_key`, `delete_env_var_def`, `upsert_env_var_value`, `load_env_var_rows` |
| Collection Vars | `insert_var_def`, `update_var_def_key`, `update_var_def_type`, `delete_var_def`, `list_var_defs`, `upsert_var_value`, `load_var_rows`, `load_operative_var_rows`, `get_active_collection_variables` |
| Settings | `get_app_setting`, `set_app_setting` |
| HTTP | `execute_request`, `cancel_request` |
| cURL | `to_curl`, `parse_curl` |
| Interpolation | `interpolate`, `resolve_url`, `extract_path_params` |
| Search | `search_all` |
| Maintenance | `get_db_stats`, `cleanup_old_data`, `prune_old_executions` |
| Trash | `list_trash`, `restore_item`, `purge_item`, `empty_trash` |
| Clone | `clone_request`, `clone_folder` |
| Import/Export | `import_postman_collection`, `export_collection_to_postman` |
| File I/O | `save_file`, `copy_to_clipboard` |
| Fingerprint | `compute_fingerprint` |

---

## 6. Frontend Component Map

### Layout

```
App.tsx
├── Sidebar.tsx ────────── Left panel: collection tree, DnD, context menus
│   └── (recursive TreeNode rendering with depth guides)
├── Center area (depends on centerView):
│   ├── UrlBar.tsx ─────── Method dropdown + URL input + Send button
│   ├── RequestEditor.tsx ─ Body tabs, cURL import/export
│   │   └── CodeEditor.tsx ─ CodeMirror: JSON editing, var tooltips, autocomplete
│   ├── ResponseView.tsx ── Response body (CM JSON viewer), headers, search
│   ├── CollectionConfig.tsx ─ Collection settings panel
│   └── AppSettings.tsx ──── Global settings panel
├── Inspector.tsx ──────── Right panel: operative vars, params, headers,
│   │                       path vars, versions, executions
│   └── KvTable.tsx ─────── Reusable key-value table
└── GlobalSearch.tsx ───── Ctrl+K search modal (portalled)
```

### Shared Components

| Component | Used by | Purpose |
|---|---|---|
| `CollapsibleSection` | Inspector, AppSettings, CollectionConfig | Expandable section header with count badge |
| `VarDefTable` | AppSettings, CollectionConfig | Variable definition rows with key/value/secret/operative toggle |
| `VariableInput` | KvTable | Text input with `{{var}}` syntax highlighting |
| `TooltipPortal` | UrlBar, CodeEditor | App-wide tooltip that portals to `document.body` |
| `KvTable` | Inspector | Key-value pair table (params, headers) |

### CodeMirror Integration

`CodeEditor.tsx` wraps `@uiw/react-codemirror` with:
- Custom dark theme matching the app palette
- `makeVarPlugin()` — Decorations that highlight `{{var}}` tokens (green=resolved, orange=unresolved, purple=dynamic)
- `makeVarHoverPlugin()` — ViewPlugin showing tooltip on hover over `{{var}}` spans
- `makeVarCompletionSource()` — Autocomplete triggered by `{{` that suggests all available variables

`ResponseView.tsx` uses a separate read-only CodeMirror for JSON responses with:
- Syntax highlighting (custom `responseSyntax` HighlightStyle)
- Native CM search panel (`Ctrl+F`, styled to match app theme)
- Fold gutter

---

## 7. Build & Development

### Commands

```bash
# Frontend dependencies
npm install

# Full dev (hot-reload frontend + Rust backend)
cargo tauri dev

# Frontend only
npx vite build

# Rust type-check only
cd src-tauri && cargo check

# TypeScript type-check only
npx tsc --noEmit
```

### Key Dependencies

**Rust**: `tauri 2`, `reqwest 0.12` (HTTP), `rusqlite 0.31` (SQLite, bundled), `keyring 3` (OS keychain), `serde/serde_json`, `chrono`, `uuid`, `sha2`

**Frontend**: `react 19`, `@uiw/react-codemirror`, `@codemirror/*`, `@dnd-kit/*`, `lucide-react`, `tailwindcss 4`, `vite 8`, `typescript 6`

### Hot Reloading

- `cargo tauri dev` runs Vite dev server on `:5173` and launches the Tauri window pointing at it
- Frontend changes hot-reload instantly (Vite HMR)
- Rust changes trigger a full rebuild (~15-30s)

---

## 8. Adding a New Feature — Checklist

Here's the typical path for a full-stack feature:

1. **Rust models** (`src-tauri/src/models/`) — Add/modify structs with `#[derive(Serialize, Deserialize)]`
2. **DB schema** (`schema.rs`) — Add migration (idempotent, appended to `initialize()`)
3. **DB operations** (`operations.rs`) — Add query methods on `Database`
4. **Commands** (`commands.rs`) — Add `#[tauri::command]` handlers
5. **Register** (`lib.rs`) — Add command to `generate_handler![...]`
6. **TypeScript types** (`types.ts`) — Mirror the Rust structs
7. **API bindings** (`api.ts`) — Add `invoke()` wrapper
8. **Components** — Build/modify React components
9. **Verify** — `cd src-tauri && cargo check` + `npx tsc --noEmit`

### Migration Pattern

```rust
// In schema.rs — add a new function:
fn migrate_new_feature(conn: &Connection) {
    // Check if already done
    let cols: Vec<String> = conn
        .prepare("PRAGMA table_info(some_table)")...;
    if cols.contains(&"new_column".to_string()) { return; }
    // Apply
    conn.execute_batch("ALTER TABLE some_table ADD COLUMN new_column TEXT DEFAULT ''");
}

// Then call it from initialize():
migrate_new_feature(conn);
```

---

## 9. Gotchas & Notes

- **No global state management** — Everything flows through `App.tsx` props. This works because the app is essentially single-page with one "active request" at a time.

- **`editorData` vs saved version** — `editorData` is the in-memory working copy. It's only saved to the DB when the user sends the request or switches away (auto-save on navigation). The `dirty` flag tracks unsaved changes.

- **`data_json` in `request_versions`** — Stores `RequestData` as a JSON string. This means the full request configuration (method, URL, headers, params, body) is frozen per version.

- **`response_json` in `request_executions`** — Stores `ResponseData` metadata (status, headers, size) but the body is stored separately in `response_bodies` (referenced by `body_hash`).

- **Env variables vs Collection variables** — These are separate systems. Env variables are global (shared across all collections). Collection variables are scoped to a single collection. Both support per-environment values.

- **`auth_config` and `headers_config`** on collections are JSON blobs — they're opaque to the DB and parsed by the frontend/commands layer.

- **Secrets** — When `is_secret` is true, the actual value lives in the OS keychain (key = `lr-secret-{value_id}`). The DB column stores an empty string. Read/write happens transparently in the operations layer.

- **`deleted_at` filtering** — Every list query has `WHERE deleted_at IS NULL`. If you add a new list query, don't forget this filter.

- **Sort order** — Folders, requests, and environments have `sort_order` columns managed by drag-and-drop on the frontend. New items get `sort_order = (max + 1)`.
