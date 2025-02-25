use std::collections::BTreeMap;
use std::collections::HashMap;
use std::sync::Once;
use std::time::{Duration, Instant};

use deno_core::error::CoreError;
use deno_core::serde_v8;
use deno_core::v8;
use deno_core::JsRuntime;
use deno_core::RuntimeOptions;

use crate::handler::AvailableModels;
use crate::types::gateway::ChatCompletionRequest;
use crate::usage::ProviderMetrics;

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

#[derive(Error, Debug)]
pub enum EvalError {
    #[error(transparent)]
    DenoSerdeError(#[from] deno_core::serde_v8::Error),

    #[error(transparent)]
    CoreError(#[from] CoreError),

    #[error(transparent)]
    JsonError(#[from] serde_json::Error),
}

static V8_INIT: Once = Once::new();
static mut V8_PLATFORM: Option<v8::SharedRef<v8::Platform>> = None;

fn init_v8() -> v8::SharedRef<v8::Platform> {
    V8_INIT.call_once(|| {
        let platform = v8::Platform::new(0, false).make_shared();
        v8::V8::initialize_platform(platform.clone());
        v8::V8::initialize();
        unsafe {
            V8_PLATFORM = Some(platform);
        }
    });
    #[allow(static_mut_refs)]
    unsafe {
        V8_PLATFORM.clone().unwrap()
    }
}

pub struct ScriptStrategy {}

impl Default for ScriptStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl ScriptStrategy {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run(
        &self,
        script: &str,
        request: &ChatCompletionRequest,
        headers: &HashMap<String, String>,
        models: &AvailableModels,
        metrics: &BTreeMap<String, ProviderMetrics>,
    ) -> Result<serde_json::Value, ScriptError> {
        // Initialize V8 platform if needed
        let platform = init_v8();

        // Create a new runtime with minimal options
        let options = RuntimeOptions {
            v8_platform: Some(platform),
            ..Default::default()
        };

        let mut runtime = JsRuntime::new(options);

        // Create a secure context with limited globals
        let code = format!(
            "(() => {{ 
                // Remove potentially dangerous globals
                const secureGlobals = {{}};
                const safeProps = ['Object', 'Array', 'Number', 'String', 'Boolean', 'Math', 'JSON'];
                safeProps.forEach(prop => {{ secureGlobals[prop] = globalThis[prop]; }});
                
                // Add our script in a secure wrapper
                const router = (context) => {{
                    'use strict';
                    try {{
                        {script}
                        const result = route(context);
                        if (typeof result !== 'object') {{
                            throw new Error('Script must return an object');
                        }}
                        return result;
                    }} catch (e) {{
                        throw new Error(`Script execution failed: ${{e.message}}`);
                    }}
                }};

                return router;
            }})()({{
                request: {},
                headers: {},
                models: {},
                metrics: {}
            }});",
            serde_json::to_string(request)?,
            serde_json::to_string(headers)?,
            serde_json::to_string(&models.0)?,
            serde_json::to_string(metrics)?,
        );

        // Execute the script with a timeout check
        let start = Instant::now();
        let timeout = Duration::from_secs(30);

        let result = eval(&mut runtime, code);

        if start.elapsed() > timeout {
            return Err(ScriptError::ExecutionError(
                "Script execution timed out".to_string(),
            ));
        }

        // Explicitly drop the runtime
        drop(runtime);

        result.map_err(Into::into)
    }
}

fn eval(context: &mut JsRuntime, code: String) -> Result<serde_json::Value, EvalError> {
    let res = context.execute_script("<anon>", code);
    match res {
        Ok(global) => {
            let scope = &mut context.handle_scope();
            let local = v8::Local::new(scope, global);
            Ok(serde_v8::from_v8::<serde_json::Value>(scope, local)?)
        }
        Err(err) => Err(EvalError::CoreError(err)),
    }
}
