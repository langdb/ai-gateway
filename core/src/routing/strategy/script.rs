use std::cell::RefCell;
use std::collections::BTreeMap;
use std::collections::HashMap;

use deno_core::error::CoreError;
use deno_core::serde_v8;
use deno_core::v8;
use deno_core::Extension;
use deno_core::JsRuntime;
use deno_core::RuntimeOptions;

use crate::handler::AvailableModels;
use crate::types::gateway::ChatCompletionRequest;
use crate::usage::ProviderMetrics;
use std::cell::RefMut;

use thiserror::Error;

#[derive(Debug, thiserror::Error)]
pub enum ScriptError {
    #[error("Failed to serialize JSON: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Script execution failed: {0}")]
    ExecutionError(String),

    #[error("Memory limit exceeded")]
    MemoryLimitExceeded,

    #[error("Invalid return value: {0}")]
    InvalidReturnValue(String),
}

impl From<EvalError> for ScriptError {
    fn from(err: EvalError) -> Self {
        match err {
            EvalError::CoreError(e) => {
                if e.to_string().contains("memory") {
                    ScriptError::MemoryLimitExceeded
                } else {
                    ScriptError::ExecutionError(e.to_string())
                }
            }
            EvalError::DenoSerdeError(e) => ScriptError::InvalidReturnValue(e.to_string()),
            EvalError::JsonError(e) => ScriptError::SerializationError(e),
        }
    }
}

thread_local! {
    static JS_RUNTIME: RefCell<Option<JsRuntime>> = RefCell::new(None);
}

#[derive(Error, Debug)]
pub enum EvalError {
    #[error(transparent)]
    DenoSerdeError(#[from] deno_core::serde_v8::Error),

    #[error(transparent)]
    CoreError(#[from] CoreError),

    #[error(transparent)]
    JsonError(#[from] serde_json::Error),
}

pub struct ScriptStrategy {}

impl ScriptStrategy {
    fn with_runtime<F, R>(f: F) -> Result<R, ScriptError>
    where
        F: FnOnce(&mut JsRuntime) -> Result<R, ScriptError>,
    {
        JS_RUNTIME.with(|cell| {
            let mut runtime_ref = cell.borrow_mut();
            if runtime_ref.is_none() {
                // Configure runtime options with security constraints and memory limits
                let create_params = v8::CreateParams::default().heap_limits(0, 64 * 1024 * 1024); // Set max heap to 64MB

                let options = RuntimeOptions {
                    extensions: vec![Extension {
                        name: "routing",
                        ops: vec![].into(),
                        js_files: vec![].into(),
                        esm_files: vec![].into(),
                        esm_entry_point: None,
                        lazy_loaded_esm_files: vec![].into(),
                        enabled: true,
                        ..Default::default()
                    }],
                    module_loader: None,    // Disable module loading
                    startup_snapshot: None, // No startup snapshot
                    shared_array_buffer_store: None,
                    create_params: Some(create_params),
                    v8_platform: None,
                    inspector: false, // Disable inspector
                    skip_op_registration: false,
                    ..Default::default()
                };

                *runtime_ref = Some(JsRuntime::new(options));
            }
            
            f(runtime_ref.as_mut().unwrap())
        })
    }

    pub fn run(
        script: &str,
        request: &ChatCompletionRequest,
        headers: &HashMap<String, String>,
        models: &AvailableModels,
        metrics: &BTreeMap<String, ProviderMetrics>,
    ) -> Result<serde_json::Value, ScriptError> {
        // Create a secure context with limited globals
        let code = format!(
            r#"
            (function() {{
                const request = {};
                const headers = {};
                const models = {};
                const metrics = {};
                
                {}

                return route(request, headers, models, metrics);
            }})()
            "#,
            serde_json::to_string(request)?,
            serde_json::to_string(headers)?,
            serde_json::to_string(&models.0)?,
            serde_json::to_string(metrics)?,
            script
        );

        Self::with_runtime(|runtime| {
            eval(runtime, code).map_err(Into::into)
        })
    }
}

fn eval(runtime: &mut JsRuntime, code: String) -> Result<serde_json::Value, EvalError> {
    let res = runtime.execute_script("<anon>", code)?;
    let scope = &mut runtime.handle_scope();
    let local = v8::Local::new(scope, res);
    Ok(serde_v8::from_v8(scope, local)?)
}
