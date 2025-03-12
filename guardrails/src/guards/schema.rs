use jsonschema::{Draft, JSONSchema};
use langdb_core::types::gateway::ChatCompletionMessage;
use langdb_core::types::guardrails::{evaluator::Evaluator, Guard, GuardResult};
use serde_json::Value;

pub struct SchemaEvaluator;

#[async_trait::async_trait]
impl Evaluator for SchemaEvaluator {
    async fn evaluate(
        &self,
        messages: &[ChatCompletionMessage],
        guard: &Guard,
    ) -> Result<GuardResult, String> {
        let text = self.messages_to_text(messages)?;
        if let Guard::Schema {
            user_defined_schema,
            ..
        } = &guard
        {
            // Try to parse the text as JSON
            let json_result = serde_json::from_str::<Value>(&text);

            match json_result {
                Ok(json_value) => {
                    // Compile the schema
                    let compiled_schema = match JSONSchema::options()
                        .with_draft(Draft::Draft7)
                        .compile(user_defined_schema)
                    {
                        Ok(schema) => schema,
                        Err(e) => {
                            return Err(format!("Invalid schema definition: {}", e));
                        }
                    };

                    let json_value_clone = json_value.clone();
                    // Validate against the schema
                    let validation_result = compiled_schema.validate(&json_value_clone);
                    match validation_result {
                        Ok(_) => Ok(GuardResult::Json {
                            schema: json_value,
                            passed: true,
                        }),
                        Err(errors) => {
                            let error_messages: Vec<String> =
                                errors.map(|err| format!("{}", err)).collect();

                            Ok(GuardResult::Text {
                                text: error_messages.join("; "),
                                passed: false,
                                confidence: Some(1.0),
                            })
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Invalid response JSON: {}", e);
                    Ok(GuardResult::Text {
                        text: e.to_string(),
                        passed: false,
                        confidence: Some(1.0),
                    })
                }
            }
        } else {
            Err("Invalid guard definition".to_string())
        }
    }
}
