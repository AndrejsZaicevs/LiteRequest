use rquickjs::{Context, Runtime};

/// Manages the QuickJS runtime. One per script execution.
pub struct ScriptEngine {
    runtime: Runtime,
}

impl ScriptEngine {
    pub fn new() -> Result<Self, String> {
        let runtime = Runtime::new().map_err(|e| format!("Failed to create JS runtime: {e}"))?;
        runtime.set_memory_limit(64 * 1024 * 1024);
        runtime.set_max_stack_size(1024 * 1024);
        Ok(Self { runtime })
    }

    /// Create a new execution context for a single script run.
    pub fn create_context(&self) -> Result<Context, String> {
        Context::full(&self.runtime)
            .map_err(|e| format!("Failed to create JS context: {e}"))
    }
}
