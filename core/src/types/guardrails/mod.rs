use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

pub mod evaluator;
pub mod service;

#[derive(Debug, Error)]
pub enum GuardError {
    #[error("Guard not found: {0}")]
    GuardNotFound(String),

    #[error("Guard evaluation error: {0}")]
    GuardEvaluationError(String),

    #[error("Output guardrails not supported in streaming")]
    OutputGuardrailsNotSupportedInStreaming,

    #[error("Request stopped after guard evaluation: {0}")]
    RequestStoppedAfterGuardEvaluation(String),
}

/// Enum representing when a guard should be applied
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum GuardStage {
    /// Applied to user messages before being sent to the LLM
    Input,
    /// Applied to LLM responses before being returned to the user
    Output,
}

/// Enum representing what action a guard should take
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum GuardAction {
    /// Only observes and logs results without blocking
    Observe,
    /// Validates and can block/fail a request
    Validate,
}

/// The result of a guard evaluation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum GuardResult {
    /// Pass/fail result
    Boolean {
        passed: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        confidence: Option<f64>,
    },
    /// Text result for observation
    Text {
        text: String,
        passed: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        confidence: Option<f64>,
    },
    /// Structured JSON result
    Json { schema: Value, passed: bool },
}

/// Base guard configuration shared by all guard types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct GuardConfig {
    pub definition_id: String,
    pub definition_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub stage: GuardStage,
    pub action: GuardAction,
}

/// A guard that has been configured with input variables
/// This is used to evaluate the guard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Guard {
    pub id: String,
    pub name: String,
    #[serde(flatten)]
    pub definition: GuardDefinition,
    /// User defined metadata for the guard
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_input: Option<Value>,
}

/// The main Guard type that encompasses all guard types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GuardDefinition {
    /// Schema-based guard using JSON schema for validation
    Schema {
        #[serde(flatten)]
        config: GuardConfig,
        schema: Value,
    },
    /// LLM-based guard that uses another LLM as a judge
    LlmJudge {
        #[serde(flatten)]
        config: GuardConfig,
        model: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        system_prompt: Option<String>,
        user_prompt_template: String,
        parameters: Value,
    },
    /// Dataset-based guard that uses vector similarity to examples
    Dataset {
        #[serde(flatten)]
        config: GuardConfig,
        embedding_model: String,
        threshold: f64,
        dataset: DatasetSource,
        schema: Value,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase", tag = "type")]
pub enum DatasetSource {
    /// A dataset of examples without labels
    Examples(Vec<GuardExample>),
    /// A dataset name that will be loaded from a source
    Source(String),
}

/// Example entry for dataset-based guard
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub struct GuardExample {
    pub text: String,
    pub label: bool,
    pub embedding: Option<Vec<f32>>,
}

/// Trait for loading datasets
#[async_trait::async_trait]
pub trait DatasetLoader: Send + Sync {
    async fn load(&self, source: &str) -> Result<Vec<GuardExample>, String>;
}

impl GuardDefinition {
    /// Returns the stage at which this guard should be applied
    pub fn stage(&self) -> &GuardStage {
        match self {
            GuardDefinition::Schema { config, .. } => &config.stage,
            GuardDefinition::LlmJudge { config, .. } => &config.stage,
            GuardDefinition::Dataset { config, .. } => &config.stage,
        }
    }

    /// Returns the action this guard should take
    pub fn action(&self) -> &GuardAction {
        match self {
            GuardDefinition::Schema { config, .. } => &config.action,
            GuardDefinition::LlmJudge { config, .. } => &config.action,
            GuardDefinition::Dataset { config, .. } => &config.action,
        }
    }

    /// Returns the ID of this guard
    pub fn id(&self) -> &String {
        match self {
            GuardDefinition::Schema { config, .. } => &config.definition_id,
            GuardDefinition::LlmJudge { config, .. } => &config.definition_id,
            GuardDefinition::Dataset { config, .. } => &config.definition_id,
        }
    }

    /// Returns the name of this guard
    pub fn name(&self) -> &String {
        match self {
            GuardDefinition::Schema { config, .. } => &config.definition_name,
            GuardDefinition::LlmJudge { config, .. } => &config.definition_id,
            GuardDefinition::Dataset { config, .. } => &config.definition_name,
        }
    }

    pub fn schema(&self) -> &Value {
        match self {
            GuardDefinition::Schema { schema, .. } => schema,
            GuardDefinition::LlmJudge { parameters, .. } => parameters,
            GuardDefinition::Dataset { schema, .. } => schema,
        }
    }
}
