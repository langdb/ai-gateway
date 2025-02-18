use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    models::ModelDefinition,
    types::{
        credentials::{ApiKeyCredentials, Credentials},
        provider::InferenceModelProvider,
        LANGDB_API_URL,
    },
};

pub mod chat_completion;
pub mod embeddings;
pub mod image_generation;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ProvidersConfig(pub HashMap<String, ApiKeyCredentials>);

pub fn get_key_credentials(
    key_credentials: Option<&Credentials>,
    providers_config: Option<&ProvidersConfig>,
    provider_name: &str,
) -> Option<Credentials> {
    match key_credentials {
        Some(credentials) => Some(credentials.clone()),
        None => match providers_config {
            Some(providers_config) => providers_config
                .0
                .get(provider_name)
                .map(|credentials| Credentials::ApiKey(credentials.clone())),
            None => None,
        },
    }
}

pub fn get_langdb_proxy_key(
    key_credentials: Option<Credentials>,
    providers_config: Option<&ProvidersConfig>,
) -> Option<Credentials> {
    match (
        key_credentials,
        providers_config
            .as_ref()
            .and_then(|p| p.0.get("langdb_proxy")),
    ) {
        (None, Some(key)) => Some(Credentials::ApiKey(key.clone())),
        (credentials, _) => credentials,
    }
}

pub fn use_langdb_proxy(
    key_credentials: Option<Credentials>,
    mut llm_model: ModelDefinition,
    providers_config: Option<&ProvidersConfig>,
) -> (Option<Credentials>, ModelDefinition) {
    let (key_credentials, endpoint) = match (
        key_credentials,
        providers_config
            .as_ref()
            .and_then(|p| p.0.get("langdb_proxy")),
    ) {
        (None, Some(key)) => (
            Some(Credentials::ApiKey(key.clone())),
            Some(format!(
                "{}/v1",
                std::env::var("LANGDB_API_URL")
                    .ok()
                    .unwrap_or(LANGDB_API_URL.to_string())
            )),
        ),
        (credentials, _) => (credentials, None),
    };

    if let Some(ref endpoint) = endpoint {
        llm_model.inference_provider.provider =
            InferenceModelProvider::Proxy(llm_model.inference_provider.provider.to_string());
        llm_model.inference_provider.endpoint = Some(endpoint.clone());
    }

    (key_credentials, llm_model)
}
