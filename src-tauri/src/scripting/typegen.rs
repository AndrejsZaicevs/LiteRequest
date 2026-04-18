use crate::db::Database;
use crate::models::*;

/// Generate `.d.ts` type definitions for all collections a script can import.
/// This is served to the Monaco editor for IntelliSense.
pub fn generate_all_types(db: &Database) -> Result<String, String> {
    let mut output = String::new();

    // Base types (always available)
    output.push_str(BASE_TYPES);
    output.push('\n');

    // Per-collection module declarations
    let collections = db.list_collections().map_err(|e| format!("{e}"))?;
    for collection in &collections {
        let module_dts = generate_collection_module(db, collection)?;
        output.push_str(&module_dts);
        output.push('\n');
    }

    Ok(output)
}

/// Generate `.d.ts` for a single collection's module.
fn generate_collection_module(db: &Database, collection: &Collection) -> Result<String, String> {
    let requests = db.list_requests_by_collection(&collection.id)
        .map_err(|e| format!("{e}"))?;

    let mut functions = Vec::new();
    for req in &requests {
        let fn_name = crate::scripting::modules::sanitize_to_camel_case(&req.name);
        if fn_name.is_empty() {
            continue;
        }

        let doc = format!("  /** {} */", req.name);
        functions.push(format!(
            "{}\n  export function {}(options?: RequestOptions): Promise<LrResponse>;",
            doc, fn_name
        ));
    }

    Ok(format!(
        "declare module \"lr:collections/{}\" {{\n{}\n}}",
        collection.name,
        functions.join("\n")
    ))
}

/// Base type definitions available in all scripts.
const BASE_TYPES: &str = r#"
interface LrResponse {
  status: number;
  statusText: string;
  headers: Record<string, string>;
  body: string;
  sizeBytes: number;
  latencyMs: number;
  json(): any;
}

interface RequestOptions {
  variables?: Record<string, string>;
  headers?: Record<string, string>;
  queryParams?: Record<string, string>;
  pathParams?: Record<string, string>;
  body?: string;
}

interface LrPostExec {
  request: {
    method: string;
    url: string;
    headers: Record<string, string>;
    queryParams: Record<string, string>;
    pathParams: Record<string, string>;
    body: string;
    bodyType: string;
  };
  response: {
    status: number;
    statusText: string;
    headers: Record<string, string>;
    body: string;
    sizeBytes: number;
    latencyMs: number;
    json(): any;
  };
  variables: Record<string, string>;
  environment: string;
  setVariable(name: string, value: string): void;
  log(...args: any[]): void;
}

interface LrStandalone {
  variables: Record<string, string>;
  environment: string;
  setVariable(name: string, value: string): void;
  log(...args: any[]): void;
  sleep(ms: number): Promise<void>;
}

declare const lr: LrPostExec & LrStandalone;
declare const console: { log(...args: any[]): void };
"#;
