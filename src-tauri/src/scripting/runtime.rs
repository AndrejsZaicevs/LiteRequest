use rquickjs::{Context, Runtime};
use std::time::{Duration, Instant};

/// Wall-clock budget for one script run. The interrupt handler makes QuickJS
/// abandon the script once it is exceeded, so a `while (true) {}` cannot hang
/// the blocking thread it runs on.
pub const SCRIPT_TIMEOUT: Duration = Duration::from_secs(10);

/// Manages the QuickJS runtime. One per script execution.
pub struct ScriptEngine {
    runtime: Runtime,
}

impl ScriptEngine {
    pub fn new() -> Result<Self, String> {
        Self::with_timeout(SCRIPT_TIMEOUT)
    }

    pub fn with_timeout(timeout: Duration) -> Result<Self, String> {
        let runtime = Runtime::new().map_err(|e| format!("Failed to create JS runtime: {e}"))?;
        runtime.set_memory_limit(64 * 1024 * 1024);
        runtime.set_max_stack_size(1024 * 1024);
        let deadline = Instant::now() + timeout;
        runtime.set_interrupt_handler(Some(Box::new(move || Instant::now() >= deadline)));
        Ok(Self { runtime })
    }

    /// Create a new execution context for a single script run.
    pub fn create_context(&self) -> Result<Context, String> {
        Context::full(&self.runtime)
            .map_err(|e| format!("Failed to create JS context: {e}"))
    }
}

/// QuickJS surfaces an interrupted run as an `InternalError: interrupted`.
pub fn is_timeout_error(message: &str) -> bool {
    message.contains("interrupted")
}
