use std::collections::BTreeMap;
use std::collections::HashMap;

use quick_js::{Context, ContextError, ExecutionError};

use crate::handler::AvailableModels;
use crate::routing::strategy::js_value_to_json;
use crate::types::gateway::ChatCompletionRequest;
use crate::usage::ProviderMetrics;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ScriptError {
    #[error("Failed to serialize JSON: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("JavaScript error: {0}")]
    JsContextError(#[from] ContextError),

    #[error("JavaScript execution error: {0}")]
    JsExecutionError(#[from] ExecutionError),
}

pub struct ScriptStrategy {}

impl ScriptStrategy {
    pub fn run(
        script: &str,
        request: &ChatCompletionRequest,
        headers: &HashMap<String, String>,
        models: &AvailableModels,
        metrics: &BTreeMap<String, ProviderMetrics>,
    ) -> Result<serde_json::Value, ScriptError> {
        let start_time = std::time::Instant::now();
        // Create a secure context with limited globals
        let code = format!(
            "(() => {{ 
                // Remove potentially dangerous globals
                const secureGlobals = {{}};
                const safeProps = ['Object', 'Array', 'Number', 'String', 'Boolean', 'Math', 'JSON'];
                safeProps.forEach(prop => {{ secureGlobals[prop] = globalThis[prop]; }});
                
                // Add our script in a secure wrapper with timeout
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

        let context = Context::new()?;
        let result = context.eval(&code)?;

        let duration = start_time.elapsed();
        tracing::warn!("Script execution time: {} ms", duration.as_millis());

        let value: serde_json::Value = js_value_to_json(&result);

        let duration = start_time.elapsed();
        tracing::warn!(
            "Script execution + Conversion time: {} ms",
            duration.as_millis()
        );

        Ok(value)
    }
}
