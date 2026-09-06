use crate::models::{RequestData, ResponseData};
use rquickjs::{Function, Object, Ctx, IntoJs, Coerced};
use rquickjs::prelude::Rest;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Collected side-effects from a script execution.
#[derive(Debug, Default)]
pub struct ScriptSideEffects {
    pub logs: Vec<String>,
    pub variables_set: HashMap<String, String>,
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

fn make_log_fn(effects: SharedEffects) -> impl Fn(Rest<Coerced<String>>) + Clone {
    move |args: Rest<Coerced<String>>| {
        let line = args.0.iter().map(|s| s.0.as_str()).collect::<Vec<_>>().join(" ");
        if let Ok(mut eff) = effects.lock() {
            eff.logs.push(line);
        }
    }
}

fn make_set_var_fn(effects: SharedEffects) -> impl Fn(String, Coerced<String>) + Clone {
    move |name: String, value: Coerced<String>| {
        if let Ok(mut eff) = effects.lock() {
            eff.variables_set.insert(name, value.0);
        }
    }
}

/// `lr.getVariable(name)`: values set earlier in the same script win over the
/// resolved variables the script started with. Returns `undefined` when unset.
fn make_get_var_fn(
    effects: SharedEffects,
    base: HashMap<String, String>,
) -> impl Fn(String) -> Option<String> + Clone {
    move |name: String| {
        if let Ok(eff) = effects.lock() {
            if let Some(v) = eff.variables_set.get(&name) {
                return Some(v.clone());
            }
        }
        base.get(&name).cloned()
    }
}

fn install_variable_fns<'js>(
    ctx: &Ctx<'js>,
    lr: &Object<'js>,
    effects: &SharedEffects,
    base: &HashMap<String, String>,
) -> Result<(), String> {
    let set_var = Function::new(ctx.clone(), make_set_var_fn(effects.clone()))
        .map_err(|e| format!("{e}"))?;
    set_var.set_name("setVariable").map_err(|e| format!("{e}"))?;
    lr.set("setVariable", set_var).map_err(|e| format!("{e}"))?;

    let get_var = Function::new(ctx.clone(), make_get_var_fn(effects.clone(), base.clone()))
        .map_err(|e| format!("{e}"))?;
    get_var.set_name("getVariable").map_err(|e| format!("{e}"))?;
    lr.set("getVariable", get_var).map_err(|e| format!("{e}"))?;
    Ok(())
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

    // lr.setVariable(name, value) / lr.getVariable(name)
    install_variable_fns(ctx, &lr, &effects, &post_ctx.variables)?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scripting::runtime::{is_timeout_error, ScriptEngine};
    use rquickjs::CatchResultExt;
    use std::time::Duration;

    fn post_exec_ctx(base: &[(&str, &str)]) -> PostExecContext {
        PostExecContext {
            request: RequestData::default(),
            response: ResponseData {
                status: 200,
                status_text: "OK".into(),
                headers: HashMap::new(),
                body: r#"{"id": 7, "user": {"name": "ada"}}"#.into(),
                size_bytes: 0,
                is_binary: false,
                truncated: false,
            },
            latency_ms: 1,
            variables: base.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
            environment: "dev".into(),
        }
    }

    fn run_post_exec(script: &str, base: &[(&str, &str)]) -> Result<ScriptSideEffects, String> {
        let engine = ScriptEngine::new().unwrap();
        let ctx = engine.create_context().unwrap();
        let effects: SharedEffects = Arc::new(Mutex::new(ScriptSideEffects::default()));
        let post = post_exec_ctx(base);
        let result = ctx.with(|js| {
            inject_post_exec_globals(&js, &post, effects.clone()).unwrap();
            js.eval::<(), _>(script).catch(&js).map_err(|e| format!("{e}"))
        });
        let eff = effects.lock().unwrap();
        result.map(|_| ScriptSideEffects {
            logs: eff.logs.clone(),
            variables_set: eff.variables_set.clone(),
        })
    }

    #[test]
    fn extracts_from_response_and_sets_variables() {
        let eff = run_post_exec(
            r#"
                const data = lr.response.json();
                lr.setVariable("userId", data.id);
                lr.setVariable("userName", data.user.name);
                lr.log("status", lr.response.status, lr.environment);
            "#,
            &[],
        ).unwrap();
        assert_eq!(eff.variables_set.get("userId").map(String::as_str), Some("7"));
        assert_eq!(eff.variables_set.get("userName").map(String::as_str), Some("ada"));
        assert_eq!(eff.logs, vec!["status 200 dev"]);
    }

    #[test]
    fn get_variable_reads_base_then_script_set_values() {
        let eff = run_post_exec(
            r#"
                lr.log(lr.getVariable("token"));
                lr.log(String(lr.getVariable("missing")));
                lr.setVariable("token", "fresh");
                lr.log(lr.getVariable("token"));
            "#,
            &[("token", "abc")],
        ).unwrap();
        assert_eq!(eff.logs, vec!["abc", "undefined", "fresh"]);
        assert_eq!(eff.variables_set.get("token").map(String::as_str), Some("fresh"));
    }

    #[test]
    fn set_variable_stringifies_non_string_values() {
        let eff = run_post_exec(
            r#"
                lr.setVariable("count", 42);
                lr.setVariable("flag", true);
                lr.setVariable("obj", { toString() { return "custom"; } });
            "#,
            &[],
        ).unwrap();
        assert_eq!(eff.variables_set.get("count").map(String::as_str), Some("42"));
        assert_eq!(eff.variables_set.get("flag").map(String::as_str), Some("true"));
        assert_eq!(eff.variables_set.get("obj").map(String::as_str), Some("custom"));
    }

    #[test]
    fn script_errors_are_reported_not_panicked() {
        let err = run_post_exec("lr.response.json().nope.deeper;", &[]).unwrap_err();
        assert!(err.contains("nope") || err.contains("undefined"), "{err}");
    }

    #[test]
    fn runaway_script_is_interrupted() {
        let engine = ScriptEngine::with_timeout(Duration::from_millis(200)).unwrap();
        let ctx = engine.create_context().unwrap();
        let err = ctx.with(|js| {
            js.eval::<(), _>("while (true) {}").catch(&js).map_err(|e| format!("{e}")).unwrap_err()
        });
        assert!(is_timeout_error(&err), "unexpected error text: {err}");
    }
}
