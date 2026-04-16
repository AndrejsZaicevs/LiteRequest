// ── HTTP Method ──────────────────────────────────────────────
export type HttpMethod = "GET" | "POST" | "PUT" | "PATCH" | "DELETE" | "HEAD" | "OPTIONS";

export const HTTP_METHODS: HttpMethod[] = ["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"];

export function methodColor(m: HttpMethod): string {
  switch (m) {
    case "GET": return "#60a5fa";
    case "POST": return "#4ade80";
    case "PUT": return "#facc15";
    case "PATCH": return "#c084fc";
    case "DELETE": return "#f87171";
    case "HEAD": return "#9ca3af";
    case "OPTIONS": return "#9ca3af";
  }
}

export const METHOD_STYLES: Record<string, { text: string; bg: string; border: string }> = {
  GET:     { text: "text-blue-400",   bg: "bg-blue-500/10",   border: "border-blue-500/20" },
  POST:    { text: "text-green-400",  bg: "bg-green-500/10",  border: "border-green-500/20" },
  PUT:     { text: "text-yellow-400", bg: "bg-yellow-500/10", border: "border-yellow-500/20" },
  PATCH:   { text: "text-purple-400", bg: "bg-purple-500/10", border: "border-purple-500/20" },
  DELETE:  { text: "text-red-400",    bg: "bg-red-500/10",    border: "border-red-500/20" },
  HEAD:    { text: "text-gray-400",   bg: "bg-gray-500/10",   border: "border-gray-500/20" },
  OPTIONS: { text: "text-gray-400",   bg: "bg-gray-500/10",   border: "border-gray-500/20" },
};

// ── Key-Value Pair ───────────────────────────────────────────
export interface KeyValuePair {
  key: string;
  value: string;
  enabled: boolean;
}

// ── Body Type ────────────────────────────────────────────────
// Matches Rust enum variant names exactly
export type BodyType = "None" | "Json" | "FormUrlEncoded" | "Raw" | "Multipart";

// Form data (for FormUrlEncoded body)
export type FormData = KeyValuePair[];

// ── Multipart Field ──────────────────────────────────────────
export interface MultipartField {
  key: string;
  value: string;
  is_file: boolean;
  file_path: string;
  enabled: boolean;
}

// ── Client Certificate ───────────────────────────────────────
export type CertType = "Pem" | "Pkcs12";

export interface ClientCertEntry {
  enabled: boolean;
  host: string;
  cert_type: CertType;
  cert_path: string;
  key_path: string;
  ca_path: string;
  passphrase: string;
}

// ── Request Data (version payload) ───────────────────────────
export interface RequestData {
  method: HttpMethod;
  url: string;
  headers: KeyValuePair[];
  query_params: KeyValuePair[];
  path_params: KeyValuePair[];
  body: string;
  body_type: BodyType;
  multipart_fields: MultipartField[];
}

export function defaultRequestData(): RequestData {
  return {
    method: "GET",
    url: "",
    headers: [],
    query_params: [],
    path_params: [],
    body: "",
    body_type: "None",
    multipart_fields: [],
  };
}

// ── Request (metadata — no method/url; those are in RequestVersion.data) ──
export interface Request {
  id: string;
  collection_id: string;
  folder_id: string | null;
  name: string;
  current_version_id: string | null;
  sort_order: number;
}

// ── Request Version ──────────────────────────────────────────
export interface RequestVersion {
  id: string;
  request_id: string;
  data: RequestData;
  fingerprint: string;
  created_at: string;
}

// ── Response Data ────────────────────────────────────────────
// headers is a flat object (HashMap<String,String> in Rust)
export interface ResponseData {
  status: number;
  status_text: string;
  headers: Record<string, string>;
  body: string;
  size_bytes: number;
  /** True when body is base64-encoded binary content */
  is_binary?: boolean;
}

// ── Request Execution ────────────────────────────────────────
export interface RequestExecution {
  id: string;
  version_id: string;
  request_id: string;
  environment_id: string;
  response: ResponseData;
  latency_ms: number;
  executed_at: string;
  /** Snapshot of the request data at send time (absent for legacy executions) */
  request_data?: RequestData;
}

// ── Collection ───────────────────────────────────────────────
export interface Collection {
  id: string;
  name: string;
  base_path: string;
  auth_config: string | null;
  headers_config: string | null;
  created_at: string;
  updated_at: string;
}

// ── Folder ───────────────────────────────────────────────────
export interface Folder {
  id: string;
  collection_id: string;
  parent_folder_id: string | null;
  name: string;
  path_prefix: string;
  auth_override: string | null;
  sort_order: number;
}

// ── Environment ──────────────────────────────────────────────
export interface Environment {
  id: string;
  name: string;
  is_active: boolean;
  sort_order: number;
  created_at: string;
}

// ── Environment Variable ─────────────────────────────────────
export interface EnvVariable {
  id: string;
  environment_id: string;
  key: string;
  value: string;
  is_secret: boolean;
}

// ── Env Variable Def (global key, shared across environments) ─
export interface EnvVarDef {
  id: string;
  key: string;
  sort_order: number;
}

// ── Collection Variable Def ──────────────────────────────────
export interface VarDef {
  id: string;
  collection_id: string;
  key: string;
  sort_order: number;
}

// ── Collection Variable Row (def + value for an env) ─────────
export interface VarRow {
  def_id: string;
  key: string;
  value: string;
  is_secret: boolean;
  value_id: string | null;
}

// ── Auth Config ──────────────────────────────────────────────
export interface AuthConfig {
  auth_type: "none" | "bearer" | "basic" | "api_key";
  bearer_token?: string;
  basic_username?: string;
  basic_password?: string;
  api_key_header?: string;
  api_key_value?: string;
}

// ── Status color helper ──────────────────────────────────────
export function statusColor(code: number): string {
  if (code >= 200 && code < 300) return "#49cc90";
  if (code >= 300 && code < 400) return "#fca130";
  if (code >= 400 && code < 500) return "#f93e3e";
  if (code >= 500) return "#ff5757";
  return "#8c8c96";
}

// ── Version fingerprint ──────────────────────────────────────
// Must match the Rust implementation in models/request.rs RequestData::fingerprint()
export function computeVersionFingerprint(data: RequestData): string {
  const qpKeys = data.query_params
    .filter(p => p.enabled && p.key)
    .map(p => p.key)
    .sort()
    .join(",");
  const hKeys = data.headers
    .filter(h => h.enabled && h.key)
    .map(h => h.key.toLowerCase())
    .sort()
    .join(",");
  const mpKeys = (data.multipart_fields ?? [])
    .filter(f => f.enabled && f.key)
    .map(f => f.key)
    .sort()
    .join(",");
  return `${data.method}|${data.url}|${qpKeys}|${hKeys}|${data.body_type}|${mpKeys}`;
}

// ── Variable resolution ───────────────────────────────────────

/**
 * Resolves {{variable}} references within variable values themselves.
 * Runs multiple passes until stable, allowing chained definitions like:
 *   base_path = "https://{{service_name}}.example.com"
 *   service_name = "users"  →  base_path becomes "https://users.example.com"
 *
 * Circular references are silently left unresolved (the {{ref}} stays as-is).
 */
export function resolveVariableRefs(
  vars: Record<string, string>,
  maxPasses = 10,
): Record<string, string> {
  const resolved = { ...vars };
  for (let pass = 0; pass < maxPasses; pass++) {
    let changed = false;
    for (const key of Object.keys(resolved)) {
      const newVal = resolved[key].replace(/\{\{([^}]+)\}\}/g, (match, name) => {
        const trimmed = name.trim();
        if (trimmed in resolved && resolved[trimmed] !== resolved[key]) {
          changed = true;
          return resolved[trimmed];
        }
        return match;
      });
      resolved[key] = newVal;
    }
    if (!changed) break;
  }
  return resolved;
}

/**
 * Extracts all {{variable}} names referenced in a string.
 */
export function collectVarRefs(text: string): string[] {
  const refs: string[] = [];
  const re = /\{\{([^}]+)\}\}/g;
  let m: RegExpExecArray | null;
  while ((m = re.exec(text)) !== null) refs.push(m[1].trim());
  return refs;
}

/**
 * Returns the set of {{variable}} names that appear in the request data
 * (url, params, headers, body, base path, multipart fields) but are NOT
 * present in the resolved variables map.
 */
export function findUnresolvedVars(
  data: RequestData,
  basePath: string,
  resolvedVars: Record<string, string>,
): string[] {
  const refs = new Set<string>();

  const scan = (s: string) => collectVarRefs(s).forEach(r => refs.add(r));

  scan(basePath);
  scan(data.url);
  data.query_params.forEach(p => { scan(p.key); scan(p.value); });
  data.headers.forEach(h => { scan(h.key); scan(h.value); });
  if (data.body) scan(data.body);
  data.multipart_fields?.forEach(f => { scan(f.key); if (!f.is_file) scan(f.value); });

  return [...refs].filter(name => !(name in resolvedVars) && !name.startsWith("$"));
}

// ── Search ────────────────────────────────────────────────────
export interface SearchHit {
  result_type: "request" | "collection" | "version" | "version_old" | "execution";
  request_id: string;
  request_name: string;
  collection_id: string;
  collection_name: string;
  version_id: string | null;
  execution_id: string | null;
  match_field: string;
  match_context: string;
  method: string | null;
  url: string | null;
  executed_at: string | null;
  status: number | null;
}
