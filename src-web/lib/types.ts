// ── HTTP Method ──────────────────────────────────────────────
export type HttpMethod = "GET" | "POST" | "PUT" | "PATCH" | "DELETE" | "HEAD" | "OPTIONS";

export const HTTP_METHODS: HttpMethod[] = ["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"];

export function methodColor(m: HttpMethod): string {
  switch (m) {
    case "GET": return "#61affe";
    case "POST": return "#49cc90";
    case "PUT": return "#fca130";
    case "PATCH": return "#50e3c2";
    case "DELETE": return "#f93e3e";
    case "HEAD": return "#9012fe";
    case "OPTIONS": return "#0d5aa7";
  }
}

// ── Key-Value Pair ───────────────────────────────────────────
export interface KeyValuePair {
  key: string;
  value: string;
  enabled: boolean;
}

// ── Body Type ────────────────────────────────────────────────
// Matches Rust enum variant names exactly
export type BodyType = "None" | "Json" | "FormUrlEncoded" | "Raw";

// Form data (for FormUrlEncoded body)
export type FormData = KeyValuePair[];

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
