# MetricsRepository Trait for Advanced Routing

This document explains how to use the new `MetricsRepository` trait that has been introduced to improve the routing functionality by abstracting metrics access.

## Overview

The `MetricsRepository` trait provides a clean abstraction for accessing metrics data needed for routing decisions. Instead of passing metrics directly to the route function, you now implement this trait to provide metrics from any data source (database, cache, API, etc.).

## Trait Definition

```rust
#[async_trait::async_trait]
pub trait MetricsRepository {
    /// Fetch metrics for all providers and models
    async fn get_metrics(&self) -> Result<BTreeMap<String, ProviderMetrics>, RouterError>;
    
    /// Fetch metrics for a specific provider
    async fn get_provider_metrics(&self, provider: &str) -> Result<Option<ProviderMetrics>, RouterError>;
    
    /// Fetch metrics for a specific model from a specific provider
    async fn get_model_metrics(&self, provider: &str, model: &str) -> Result<Option<crate::usage::ModelMetrics>, RouterError>;
}
```

## Updated Route Function

The `RouteStrategy::route` function signature has been updated to accept a `MetricsRepository`:

```rust
async fn route<M: MetricsRepository + Send + Sync>(
    &self,
    request: ChatCompletionRequest,
    available_models: &AvailableModels,
    headers: HashMap<String, String>,
    metrics_repository: &M,
) -> Result<Targets, RouterError>;
```

## Built-in Implementation: InMemoryMetricsRepository

A simple in-memory implementation is provided for testing and reference:

```rust
let metrics_repo = InMemoryMetricsRepository::new(metrics_map);
```

## Usage Example

Here's how to use the new routing system with a metrics repository:

```rust
use crate::routing::{LlmRouter, RoutingStrategy, MetricsDuration, InMemoryMetricsRepository};
use crate::routing::strategy::metric::MetricSelector;
use crate::usage::{ProviderMetrics, ModelMetrics, TimeMetrics, Metrics};
use std::collections::{BTreeMap, HashMap};

// 1. Create sample metrics data
let mut provider_metrics = ProviderMetrics {
    models: BTreeMap::new(),
};

provider_metrics.models.insert(
    "gpt-4".to_string(),
    ModelMetrics {
        metrics: TimeMetrics {
            total: Metrics {
                requests: Some(100.0),
                latency: Some(150.0),
                ttft: Some(50.0),
                tps: Some(20.0),
                error_rate: Some(0.01),
                input_tokens: Some(1000.0),
                output_tokens: Some(500.0),
                total_tokens: Some(1500.0),
                llm_usage: Some(0.5),
            },
            last_15_minutes: Metrics::default(),
            last_hour: Metrics::default(),
        },
    },
);

// 2. Create metrics repository
let mut metrics_map = BTreeMap::new();
metrics_map.insert("openai".to_string(), provider_metrics);
let metrics_repo = InMemoryMetricsRepository::new(metrics_map);

// 3. Create router with optimized strategy
let router = LlmRouter {
    name: "production_router".to_string(),
    strategy: RoutingStrategy::Optimized {
        metric: MetricSelector::Latency,
    },
    targets: vec![
        HashMap::from([
            ("model".to_string(), serde_json::Value::String("openai/gpt-4".to_string())),
        ]),
    ],
    metrics_duration: Some(MetricsDuration::Total),
};

// 4. Use the router
let request = ChatCompletionRequest::default();
let available_models = AvailableModels(vec![]);
let headers = HashMap::new();

let result = router.route(request, &available_models, headers, &metrics_repo).await?;
```

## Custom Implementation Examples

### Database-backed MetricsRepository

```rust
pub struct DatabaseMetricsRepository {
    pool: Arc<sqlx::Pool<sqlx::Postgres>>,
}

#[async_trait::async_trait]
impl MetricsRepository for DatabaseMetricsRepository {
    async fn get_metrics(&self) -> Result<BTreeMap<String, ProviderMetrics>, RouterError> {
        // Query your database for all metrics
        // Transform the data into the expected format
        // Return the metrics
        todo!("Implement database query")
    }
    
    async fn get_provider_metrics(&self, provider: &str) -> Result<Option<ProviderMetrics>, RouterError> {
        // Query for specific provider metrics
        todo!("Implement provider-specific query")
    }
    
    async fn get_model_metrics(&self, provider: &str, model: &str) -> Result<Option<crate::usage::ModelMetrics>, RouterError> {
        // Query for specific model metrics
        todo!("Implement model-specific query")
    }
}
```

### Cache-backed MetricsRepository

```rust
pub struct CacheMetricsRepository {
    cache: Arc<dyn Cache + Send + Sync>,
}

#[async_trait::async_trait]
impl MetricsRepository for CacheMetricsRepository {
    async fn get_metrics(&self) -> Result<BTreeMap<String, ProviderMetrics>, RouterError> {
        match self.cache.get("all_metrics").await {
            Ok(Some(data)) => {
                let metrics = serde_json::from_str(&data)
                    .map_err(|e| RouterError::MetricsRepositoryError(e.to_string()))?;
                Ok(metrics)
            },
            Ok(None) => Ok(BTreeMap::new()),
            Err(e) => Err(RouterError::MetricsRepositoryError(e.to_string())),
        }
    }
    
    // ... implement other methods
}
```

## Migration Guide

If you were previously calling the route function directly with metrics:

### Before:
```rust
let result = router.route(request, &available_models, headers, metrics).await?;
```

### After:
```rust
let metrics_repo = InMemoryMetricsRepository::new(metrics);
let result = router.route(request, &available_models, headers, &metrics_repo).await?;
```

## Benefits

1. **Separation of Concerns**: Metrics retrieval logic is now separated from routing logic
2. **Flexibility**: You can implement metrics access from any data source
3. **Testability**: Easy to mock the metrics repository for testing
4. **Performance**: Can implement caching, connection pooling, etc. in the repository
5. **Async Support**: Full async support for metrics retrieval operations

## Error Handling

The trait methods return `Result<_, RouterError>`, allowing for proper error propagation when metrics retrieval fails. You can handle specific error cases in your implementation and convert them to `RouterError::MetricsRepositoryError`.

## Thread Safety

All trait methods are async and the trait requires `Send + Sync`, making it safe to use across multiple threads and tasks.