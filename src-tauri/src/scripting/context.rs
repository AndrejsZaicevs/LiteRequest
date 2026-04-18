use crate::models::{RequestData, ResponseData};
use rquickjs::{Function, Object, Ctx, IntoJs};
use rquickjs::prelude::Rest;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Collected side-effects from a script execution.
#[derive(Debug, Default)]
pub struct ScriptSideEffects {
    pub logs: Vec<String>,
    pub variables_set: HashMap<String, String>,
    pub transformed_response: Option<String>,
}

/// Shared mutable state that JS bridge functions write into.
pub type SharedEffects = Arc<Mutex<ScriptSideEffects>>;

/// Context data injected as the `lr` global for post-execution scripts.
pub struct PostExecContext {
    pub request: RequestData,
    pub response: ResponseData,
    pub latency_ms: u64,
    pub variables: HashMap<String, String>,
    pub environment: String,
}

/// Context data injected as the `lr` global for standalone scripts.
pub struct StandaloneContext {
    pub variables: HashMap<String, String>,
    pub environment: String,
}

fn make_log_fn(effects: SharedEffects) -> impl Fn(Rest<String>) + Clone {
    move |args: Rest<String>| {
        let line = args.0.join(" ");
        if let Ok(mut eff) = effects.lock() {
            eff.logs.push(line);
        }
    }
}

fn make_set_var_fn(effects: SharedEffects) -> impl Fn(String, String) + Clone {
    move |name: String, value: String| {
        if let Ok(mut eff) = effects.lock() {
            eff.variables_set.insert(name, value);
        }
    }
}

/// Inject the `lr` global for a post-execution script.
pub fn inject_post_exec_globals<'js>(
    ctx: &Ctx<'js>,
    post_ctx: &PostExecContext,
    effects: SharedEffects,
) -> Result<(), String> {
    let globals = ctx.globals();

    let lr = Object::new(ctx.clone()).map_err(|e| format!("{e}"))?;

    // lr.request
    let req_obj = build_request_object(ctx, &post_ctx.request)?;
    lr.set("request", req_obj).map_err(|e| format!("{e}"))?;

    // lr.response
    let resp_obj = build_response_object(ctx, &post_ctx.response, post_ctx.latency_ms)?;
    lr.set("response", resp_obj).map_err(|e| format!("{e}"))?;

    // lr.variables
    let vars = post_ctx.variables.clone().into_js(ctx).map_err(|e| format!("{e}"))?;
    lr.set("variables", vars).map_err(|e| format!("{e}"))?;

    // lr.environment
    lr.set("environment", post_ctx.environment.as_str()).map_err(|e| format!("{e}"))?;

    // lr.setVariable(name, value)
    let set_var = Function::new(ctx.clone(), make_set_var_fn(effects.clone()))
        .map_err(|e| format!("{e}"))?;
    set_var.set_name("setVariable").map_err(|e| format!("{e}"))?;
    lr.set("setVariable", set_var).map_err(|e| format!("{e}"))?;

    // lr.log(...args)
    let log_fn = Function::new(ctx.clone(), make_log_fn(effects.clone()))
        .map_err(|e| format!("{e}"))?;
    log_fn.set_name("log").map_err(|e| format!("{e}"))?;
    lr.set("log", log_fn).map_err(|e| format!("{e}"))?;

    globals.set("lr", lr).map_err(|e| format!("{e}"))?;

    // console.log
    install_console(ctx, effects)?;

    Ok(())
}

/// Inject the `lr` global for a standalone script.
pub fn inject_standalone_globals<'js>(
    ctx: &Ctx<'js>,
    standalone_ctx: &StandaloneContext,
    effects: SharedEffects,
) -> Result<(), String> {
    let globals = ctx.globals();

    let lr = Object::new(ctx.clone()).map_err(|e| format!("{e}"))?;

    // lr.variables
    let vars = standalone_ctx.variables.clone().into_js(ctx).map_err(|e| format!("{e}"))?;
    lr.set("variables", vars).map_err(|e| format!("{e}"))?;

    // lr.environment
    lr.set("environment", standalone_ctx.environment.as_str()).map_err(|e| format!("{e}"))?;

    // lr.setVariable
    let set_var = Function::new(ctx.clone(), make_set_var_fn(effects.clone()))
        .map_err(|e| format!("{e}"))?;
    set_var.set_name("setVariable").map_err(|e| format!("{e}"))?;
    lr.set("setVariable", set_var).map_err(|e| format!("{e}"))?;

    // lr.log
    let log_fn = Function::new(ctx.clone(), make_log_fn(effects.clone()))
        .map_err(|e| format!("{e}"))?;
    log_fn.set_name("log").map_err(|e| format!("{e}"))?;
    lr.set("log", log_fn).map_err(|e| format!("{e}"))?;

    globals.set("lr", lr).map_err(|e| format!("{e}"))?;

    // console.log
    install_console(ctx, effects)?;

    Ok(())
}

fn install_console<'js>(ctx: &Ctx<'js>, effects: SharedEffects) -> Result<(), String> {
    let console = Object::new(ctx.clone()).map_err(|e| format!("{e}"))?;
    let console_log = Function::new(ctx.clone(), make_log_fn(effects))
        .map_err(|e| format!("{e}"))?;
    console_log.set_name("log").map_err(|e| format!("{e}"))?;
    console.set("log", console_log).map_err(|e| format!("{e}"))?;
    ctx.globals().set("console", console).map_err(|e| format!("{e}"))?;
    Ok(())
}

fn build_request_object<'js>(ctx: &Ctx<'js>, data: &RequestData) -> Result<Object<'js>, String> {
    let obj = Object::new(ctx.clone()).map_err(|e| format!("{e}"))?;
    obj.set("method", data.method.as_str()).map_err(|e| format!("{e}"))?;
    obj.set("url", data.url.as_str()).map_err(|e| format!("{e}"))?;

    let headers: HashMap<String, String> = data.headers.iter()
        .filter(|h| h.enabled && !h.key.is_empty())
        .map(|h| (h.key.clone(), h.value.clone()))
        .collect();
    let h = headers.into_js(ctx).map_err(|e| format!("{e}"))?;
    obj.set("headers", h).map_err(|e| format!("{e}"))?;

    let qp: HashMap<String, String> = data.query_params.iter()
        .filter(|p| p.enabled && !p.key.is_empty())
        .map(|p| (p.key.clone(), p.value.clone()))
        .collect();
    let q = qp.into_js(ctx).map_err(|e| format!("{e}"))?;
    obj.set("queryParams", q).map_err(|e| format!("{e}"))?;

    let pp: HashMap<String, String> = data.path_params.iter()
        .filter(|p| p.enabled && !p.key.is_empty())
        .map(|p| (p.key.clone(), p.value.clone()))
        .collect();
    let p = pp.into_js(ctx).map_err(|e| format!("{e}"))?;
    obj.set("pathParams", p).map_err(|e| format!("{e}"))?;

    obj.set("body", data.body.as_str()).map_err(|e| format!("{e}"))?;
    obj.set("bodyType", data.body_type.as_str()).map_err(|e| format!("{e}"))?;

    Ok(obj)
}

fn build_response_object<'js>(
    ctx: &Ctx<'js>,
    resp: &ResponseData,
    latency_ms: u64,
) -> Result<Object<'js>, String> {
    let obj = Object::new(ctx.clone()).map_err(|e| format!("{e}"))?;
    obj.set("status", resp.status).map_err(|e| format!("{e}"))?;
    obj.set("statusText", resp.status_text.as_str()).map_err(|e| format!("{e}"))?;

    let h = resp.headers.clone().into_js(ctx).map_err(|e| format!("{e}"))?;
    obj.set("headers", h).map_err(|e| format!("{e}"))?;

    obj.set("body", resp.body.as_str()).map_err(|e| format!("{e}"))?;
    obj.set("sizeBytes", resp.size_bytes).map_err(|e| format!("{e}"))?;
    obj.set("latencyMs", latency_ms).map_err(|e| format!("{e}"))?;

    // Store the body as a string property — json() will parse it in JS.
    // We inject a helper that does JSON.parse(lr.response.body) instead of
    // using a Rust closure (avoids lifetime issues with Ctx).
    let json_src = "JSON.parse(this.body)";
    let json_fn: Function<'js> = ctx.eval(format!(
        "(function() {{ return {}; }})",
        json_src
    )).map_err(|e| format!("{e}"))?;
    obj.set("json", json_fn).map_err(|e| format!("{e}"))?;

    Ok(obj)
}
