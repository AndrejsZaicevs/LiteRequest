Project: "LiteRequest" - Functional & Lightweight Offline API Client

1. Core Architecture (The "Speed-First" Stack)

Language: Rust

UI Framework: egui (Immediate Mode GUI). GPU-accelerated, no WebView, sub-100MB RAM footprint.

HTTP Engine: reqwest (Asynchronous, supports HTTP/2, Brotli, and proxy configuration).

Primary Storage: SQLite (WAL mode) for all metadata, versioning, and logs.

2. Feature Implementation Detail

A. Hierarchical Collections & BasePaths

Collection Identity: Acts as the root workspace with a base_path.

Inherited BasePath: * Supports environment variables: {{protocol}}://{{host}}/v1.

Automatic path resolution (Collection Base + Folder Path + Request Path).

Folders: Recursive organization with optional Auth/Path overrides.

B. Dual-Layer History System (The "Version Control" Model)

To eliminate "Save" buttons and manual tabs, the app uses two linked history tables:

Request Version History (The "Blueprint"):

Auto-Save: Every change made in the UI (URL, Method, Body, Headers) is automatically saved as a new "Version" in SQLite.

State Recovery: The sidebar allows you to scrub through the evolution of the definition of the request.

Execution History (The "Audit"):

Snapshot on Send: Every time "Send" is clicked, an entry is created that links the specific Version ID used to the Response received.

Correlation: You can see exactly which version of the request body produced a specific 400 or 500 error, even if you have since modified the request.

C. Advanced JSON Editor & IntelliSense

Implementation: egui_code_editor with a custom JSON lexer.

Basic IntelliSense:

Variable Suggestions: {{ triggers a popup for Environment and Collection variables.

Key Completion: Suggests keys based on the schema of previous successful responses in the history.

Live Linting: Background serde_json validation with inline highlighting.

D. Environments: Variables vs. Secrets

Variables (Public): Stored in SQLite; included in .lreq exports.

Secrets (Sensitive): Flagged in the UI and stored in the OS-native Keyring (Keychain/CredMgr).

Privacy: Secrets are never exported. Imports prompt for missing sensitive values.

E. Authentication & Token Rotation

Inherited Auth: Defined at the collection level.

Auth0 + PKCE: Native flow with system browser.

Auto-Rotation: Background refresh via refresh_token stored in the Keyring, executed just-in-time before requests.

F. Visual Response Inspector

Tree View: Interactive expand/collapse for large JSON responses using egui_json_tree.

Data Visualizer: Basic table view for lists of objects.

Search: High-speed filtering within the response body.

3. The "Lightweight" Checklist

Binary Size: ~12MB.

Memory Usage: 40MB - 80MB.

Performance: 60fps UI; non-blocking I/O using Tokio.

Export/Import: .lreq (JSON) format for collections/folders (Latest Version only).

4. Technical Strategy

Database Schema: * requests: id, current_version_id

request_versions: id, request_id, timestamp, data_json (Method, URL, Body, Headers)

request_executions: id, version_id, timestamp, response_json, status, latency

The "Auto-Save" Debouncer: A small delay (e.g., 500ms) after the last keystroke before committing a new entry to request_versions to prevent DB bloat.

Database Maintenance (Cold Storage): To prevent the SQLite file from ballooning indefinitely, implement an optional "Vacuum" or "Pruning" strategy where execution logs older than 30 days are archived to a secondary .db file or deleted.

UI Layout:

Left: Collection Tree.

Center: Code Editor + Response View.

Right: * Top Panel: Version History (How the request was built).

Bottom Panel: Execution History (What the server returned for those versions).