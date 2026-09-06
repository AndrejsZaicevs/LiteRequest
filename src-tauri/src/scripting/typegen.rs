/// `.d.ts` describing the `lr` global available to post-execution scripts.
/// Served to the script editor (CodeMirror) for completions.
pub fn generate_all_types() -> String {
    BASE_TYPES.to_string()
}

const BASE_TYPES: &str = r#"
interface LrPostExec {
  /** The request as sent, after variable interpolation. */
  request: {
    method: string;
    url: string;
    headers: Record<string, string>;
    queryParams: Record<string, string>;
    pathParams: Record<string, string>;
    body: string;
    bodyType: string;
  };
  /** The response that came back. */
  response: {
    status: number;
    statusText: string;
    headers: Record<string, string>;
    body: string;
    sizeBytes: number;
    latencyMs: number;
    /** Parse the body as JSON. */
    json(): any;
  };
  /** Variables resolved for this run (environment + collection). */
  variables: Record<string, string>;
  /** Name of the active environment. */
  environment: string;
  /** Persist a variable to the active environment; non-string values are stringified. */
  setVariable(name: string, value: unknown): void;
  /** Read a variable, including ones set earlier in this script. */
  getVariable(name: string): string | undefined;
  /** Write to the script output panel. */
  log(...args: any[]): void;
}

declare const lr: LrPostExec;
declare const console: { log(...args: any[]): void };
"#;
