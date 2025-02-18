use std::collections::BTreeMap;

use crate::{
    routing::RouterError,
    usage::{Metrics, ProviderMetrics},
};

#[derive(Debug, serde::Serialize, serde::Deserialize, Default, Clone)]
#[serde(rename_all = "snake_case")]
pub enum MetricSelector {
    Requests,
    #[default]
    Latency,
    Ttft,
    Tps,
    ErrorRate,
}

impl MetricSelector {
    fn get_value(&self, metrics: &Metrics) -> Option<f64> {
        match self {
            MetricSelector::Requests => metrics.requests,
            MetricSelector::Latency => metrics.latency,
            MetricSelector::Ttft => metrics.ttft,
            MetricSelector::Tps => metrics.tps,
            MetricSelector::ErrorRate => metrics.error_rate,
        }
    }
}

pub async fn route(
    models: &[String],
    metrics: &BTreeMap<String, ProviderMetrics>,
    metric: &MetricSelector,
    minimize: bool,
) -> Result<String, RouterError> {
    // Find the model with the best metric value
    let best_model = models
        .iter()
        .filter_map(|model| {
            // Get model metrics based on whether provider is specified
            if let Some((provider, model_name)) = model.split_once('/') {
                // Provider specified, look only in that provider's metrics
                metrics
                    .get(provider)
                    .and_then(|provider_metrics| provider_metrics.models.get(model_name))
                    .and_then(|metrics| {
                        metric
                            .get_value(&metrics.metrics.total)
                            .map(|value| (model.clone(), value))
                    })
            } else {
                // No provider specified, look in all providers for this model
                let mut all_matches: Vec<_> = metrics
                    .iter()
                    .filter_map(|(provider, provider_metrics)| {
                        provider_metrics.models.get(model).and_then(|metrics| {
                            metric
                                .get_value(&metrics.metrics.total)
                                .map(|value| (format!("{}/{}", provider, model), value))
                        })
                    })
                    .collect();

                // Sort by metric value and take the best one
                if minimize {
                    all_matches.sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap());
                } else {
                    all_matches.sort_by(|(_, a), (_, b)| b.partial_cmp(a).unwrap());
                }
                all_matches.into_iter().next()
            }
        })
        .min_by(|(_, value_a), (_, value_b)| {
            if minimize {
                value_a.partial_cmp(value_b).unwrap()
            } else {
                value_b.partial_cmp(value_a).unwrap()
            }
        });

    match best_model {
        Some((model, _)) => Ok(model),
        None => Err(RouterError::MetricRouterError(
            "No valid model found".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::usage::{ModelMetrics, TimeMetrics};

    fn create_model_metrics(requests_duration: f64, ttft: f64) -> ModelMetrics {
        ModelMetrics {
            metrics: TimeMetrics {
                total: Metrics {
                    requests: Some(100.0),
                    input_tokens: Some(5000.0),
                    output_tokens: Some(2000.0),
                    total_tokens: Some(7000.0),
                    latency: Some(requests_duration),
                    ttft: Some(ttft),
                    llm_usage: Some(0.05),
                    tps: Some(0.1),
                    error_rate: Some(0.01),
                },
                monthly: BTreeMap::new(),
                daily: BTreeMap::new(),
                hourly: BTreeMap::new(),
            },
        }
    }

    #[tokio::test]
    async fn test_metric_router() {
        let openai_models = BTreeMap::from([
            (
                "gpt-4o-mini".to_string(),
                create_model_metrics(1550.0, 1800.0),
            ),
            ("gpt-4o".to_string(), create_model_metrics(2550.0, 1900.0)),
        ]);
        let openai_metrics = ProviderMetrics {
            models: openai_models,
        };

        let gemini_models = BTreeMap::from([
            (
                "gemini-1.5-flash-latest".to_string(),
                create_model_metrics(500.0, 1000.0),
            ),
            (
                "gemini-1.5-pro-latest".to_string(),
                create_model_metrics(4500.0, 1100.0),
            ),
        ]);
        let gemini_metrics = ProviderMetrics {
            models: gemini_models,
        };

        let metrics = BTreeMap::from([
            ("openai".to_string(), openai_metrics),
            ("gemini".to_string(), gemini_metrics),
        ]);

        let models = vec![
            "openai/gpt-4o-mini".to_string(),
            "gemini/gemini-1.5-flash-latest".to_string(),
            "openai/gpt-4o".to_string(),
            "gemini/gemini-1.5-pro-latest".to_string(),
        ];

        // Test with TTFT metric (minimize)
        let new_model = super::route(&models, &metrics, &MetricSelector::Ttft, true)
            .await
            .unwrap();

        assert_eq!(new_model, "gemini/gemini-1.5-flash-latest".to_string());

        // Test with requests metric (maximize)
        let new_model = super::route(&models, &metrics, &MetricSelector::Requests, false)
            .await
            .unwrap();

        // All models have same request count, so first one should be selected
        assert_eq!(new_model, "openai/gpt-4o-mini".to_string());
    }

    #[tokio::test]
    async fn test_metric_router_for_all_providers() {
        let provider_a_models = BTreeMap::from([
            ("model_a".to_string(), create_model_metrics(4550.0, 3800.0)),
            ("model_b".to_string(), create_model_metrics(3550.0, 2900.0)),
        ]);
        let provider_a_metrics = ProviderMetrics {
            models: provider_a_models,
        };
        let provider_b_models = BTreeMap::from([
            ("model_a".to_string(), create_model_metrics(1550.0, 1800.0)),
            ("model_c".to_string(), create_model_metrics(2550.0, 1900.0)),
        ]);
        let provider_b_metrics = ProviderMetrics {
            models: provider_b_models,
        };
        let provider_c_models = BTreeMap::from([
            ("model_a".to_string(), create_model_metrics(1950.0, 1200.0)),
            ("model_d".to_string(), create_model_metrics(2950.0, 1700.0)),
        ]);
        let provider_c_metrics = ProviderMetrics {
            models: provider_c_models,
        };

        let metrics = BTreeMap::from([
            ("provider_a".to_string(), provider_a_metrics),
            ("provider_b".to_string(), provider_b_metrics),
            ("provider_c".to_string(), provider_c_metrics),
        ]);

        let models = vec!["model_a".to_string(), "provider_c/model_d".to_string()];

        // Test with TTFT metric (minimize)
        let new_model = super::route(&models, &metrics, &MetricSelector::Ttft, true)
            .await
            .unwrap();

        assert_eq!(new_model, "provider_c/model_a".to_string());

        // Test with request duration (minimize)
        let new_model = super::route(&models, &metrics, &MetricSelector::Latency, true)
            .await
            .unwrap();

        assert_eq!(new_model, "provider_b/model_a".to_string());
    }
}
