<p align="center">
  <img src="LiteRequestIcon.svg" width="80" alt="LiteRequest" />
</p>

<h1 align="center">LiteRequest</h1>

<p align="center">
  A fast, offline-first API client. No accounts, no cloud sync, no telemetry.
</p>

---

## Features

### Collections & Organisation

Organise requests into **collections**, each with its own base URL, default headers, authentication and variables. Inside a collection, use **folders** (with unlimited nesting) to group related endpoints. Everything is reorderable via **drag-and-drop** — requests and folders can be moved across collections and folders at will. A full **tree view** with coloured icons (violet collections, amber folders, coloured method badges) and depth guide lines makes navigation instant.

Right-click any item in the tree for a **context menu** with rename, clone, delete and move options. Inline renaming auto-selects the text so you can start typing immediately.

### Requests

- **All HTTP methods** — GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS
- **URL bar** with live base-path preview when a collection base URL is set
- **Query params**, **path variables**, and **headers** — all managed as key/value tables with per-row enable/disable toggles
- **Request body** — None, JSON (with syntax highlighting), Form URL-encoded, Multipart/form-data (with file upload), or Raw. The JSON editor is a full CodeMirror instance with folding, bracket matching, search/replace and a one-click Format button
- **Variable interpolation** — `{{variable}}` tokens are resolved in URLs, headers and body before sending. Hover over any `{{variable}}` in the URL bar or body editor to see a **tooltip** with its resolved value and source
- **Variable autocomplete** — type `{{` in the body editor to get completions for all available variables (environment, collection, global)
- **Copy as cURL** — generates a complete `curl` command with all headers, auth and body applied, resolved with the active environment variables
- **Import from cURL** — paste any `curl` command and it is parsed into a full request (method, URL, headers, body)

### Variables & Environments

Create multiple **environments** per collection (e.g. `dev`, `staging`, `prod`) and switch between them without touching the request. Variables are defined once and can hold plain-text or **secret** values — secrets are stored in the platform keychain (via the OS secret service) rather than in the database. Global variables sit outside collections and are merged at send time.

**Operative variables** can be flagged at the collection level — these appear in the inspector panel of any request that references them, giving you one-click access to edit a shared value (like a `storeId` returned from one request) without digging into collection settings. A built-in `requestName` variable is also injected per request.

### Authentication

Authentication is configured at the **collection level** and automatically injected into every request without having to repeat yourself:

| Type | Mechanism |
|---|---|
| Bearer Token | `Authorization: Bearer <token>` header |
| Basic Auth | `Authorization: Basic <base64>` header |
| API Key | Custom header name + value |

Per-request headers can still override any injected header.

### Client Certificates

Attach **PEM** or **PKCS12** client certificates to specific host patterns (including wildcards like `*.example.com`). A custom CA certificate can be bundled alongside each entry.

### Response

- **Syntax-highlighted JSON** — full CodeMirror viewer with folding, line numbers and search (`Ctrl+F`)
- **Response headers** with inline search and filtering
- **Status dot**, latency, body size — all visible at a glance
- **Copy** response body to clipboard
- **Download** response to file (auto-detects extension from content type)
- **Maximise** response panel to full width

### Execution History

Every request send is recorded with the full response body, status, latency and size. History is grouped by date in a collapsible timeline (today, yesterday, older). Filters let you narrow by version or environment. Old executions are automatically pruned after a configurable number of days.

### Versions

Each request automatically tracks **versions** based on structural changes (method, URL, parameter keys, header keys, body type). Switching between versions restores the full request configuration. Execution history is linked to the version that produced it.

### Inspector

The right-hand inspector panel is a live view of the active request:

- **Operative Variables** — edit shared variables inline (only shown when referenced by the current request)
- **Query Params** and **Headers** — edit in place, section badge shows how many are active
- **Path Variables** — auto-detected from `:param` segments in the URL
- **Versions** — browse and switch between saved versions
- **Executions** — full execution history with response status, latency, size and body

The request/response split is **drag-resizable**.

### Search

Press `Ctrl+K` to open global search. It searches across **everything** — request names, collection names, every version's URL / headers / query params / body, and all execution response bodies. Results are grouped by type, with a colour-coded match-context snippet. Clicking a result navigates directly to the request or opens the matching execution in the inspector.

### Soft Delete & Trash

Deleting a collection, folder or request is a **soft delete** — items are moved to the trash and can be **restored** or **permanently purged** from the Trash section in settings. An "Empty Trash" action removes everything at once.

### Data Cleanup

A built-in cleanup tool lets you delete old execution history and versions older than a configurable number of days.

### Resizable Layout

All three panels — sidebar, editor and inspector — are independently resizable by dragging the dividers.

---

## Implementation

LiteRequest is built with **Tauri v2** (Rust backend, WebView frontend).

| Layer | Technology |
|---|---|
| Desktop shell | [Tauri v2](https://tauri.app) |
| Backend language | Rust (stable) |
| HTTP client | [reqwest](https://github.com/seanmonstar/reqwest) — brotli, HTTP/2, rustls, SOCKS proxy |
| Database | SQLite via [rusqlite](https://github.com/rusqlite/rusqlite) (bundled, file stored in app data dir) |
| Secret storage | OS keychain via [keyring](https://github.com/hwchen/keyring-rs) |
| Frontend | React 19 + TypeScript |
| Styling | Tailwind CSS v4 |
| Code editor | [CodeMirror 6](https://codemirror.net/) via `@uiw/react-codemirror` |
| Drag-and-drop | [@dnd-kit](https://dnd-kit.com/) |
| Icons | [lucide-react](https://lucide.dev/) |

All application data (collections, requests, versions, execution history, environments and variables) lives in a single SQLite file on disk — no network required after installation.

---

## Building

**Prerequisites:** Rust (stable), Node.js ≥ 18, the [Tauri CLI prerequisites](https://tauri.app/start/prerequisites/) for your platform.

```bash
# Install frontend dependencies
npm install

# Development (hot-reload frontend + Rust backend)
cargo tauri dev

# Production build
cargo tauri build
```

Frontend only (no Rust):

```bash
npx vite build
```

Rust type-check only:

```bash
cd src-tauri && cargo check
```

---

## Roadmap

- [ ] WebSocket support
- [ ] Collection import/export (Postman, OpenAPI)
- [ ] GraphQL body type
- [ ] Proxy configuration per collection
