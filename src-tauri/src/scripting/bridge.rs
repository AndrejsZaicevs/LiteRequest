use crate::models::ClientCertEntry;
use rquickjs::{Ctx, Function};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Bridge data needed to execute HTTP requests from within scripts.
pub struct BridgeConfig {
    pub variables: HashMap<String, String>,
    pub base_paths: HashMap<String, String>,
    pub client_certs: Vec<ClientCertEntry>,
}

/// Install the `__lr_execute` global function used by generated collection modules.
/// This function is the bridge between JS and the Rust HTTP client.
pub fn install_execute_bridge<'js>(
    ctx: &Ctx<'js>,
    _config: Arc<Mutex<BridgeConfig>>,
) -> Result<(), String> {
    let globals = ctx.globals();

    // __lr_execute(requestSpec) → response object
    // The actual async HTTP execution will be wired up in a follow-up phase.
    // For now, returns a placeholder that indicates the bridge is not yet active.
    let exec_fn = Function::new(ctx.clone(), |_spec: String| -> String {
        r#"{"status":0,"statusText":"Bridge not yet wired","body":""}"#.to_string()
    }).map_err(|e| format!("{e}"))?;
    exec_fn.set_name("__lr_execute").map_err(|e| format!("{e}"))?;
    globals.set("__lr_execute", exec_fn).map_err(|e| format!("{e}"))?;

    Ok(())
}

/// Install a `__lr_sleep(ms)` global for `lr.sleep()` support.
pub fn install_sleep_bridge<'js>(ctx: &Ctx<'js>) -> Result<(), String> {
    let globals = ctx.globals();

    let sleep_fn = Function::new(ctx.clone(), |ms: u64| {
        std::thread::sleep(std::time::Duration::from_millis(ms));
    }).map_err(|e| format!("{e}"))?;
    sleep_fn.set_name("__lr_sleep").map_err(|e| format!("{e}"))?;
    globals.set("__lr_sleep", sleep_fn).map_err(|e| format!("{e}"))?;

    Ok(())
}
