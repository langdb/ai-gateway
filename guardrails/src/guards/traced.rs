use langdb_core::events::JsonValue;
use langdb_core::events::SPAN_GUARD_EVAULATION;
use langdb_core::types::gateway::ChatCompletionMessage;
use langdb_core::types::guardrails::evaluator::Evaluator;
use langdb_core::types::guardrails::Guard;
use langdb_core::types::guardrails::GuardResult;
use tracing::field;
use tracing::info_span;
use tracing_futures::Instrument;
use valuable::Valuable;

pub struct TracedGuard {
    inner: Box<dyn Evaluator>,
}

// Implement Send + Sync since inner is already Send + Sync
unsafe impl Send for TracedGuard {}
unsafe impl Sync for TracedGuard {}

impl TracedGuard {
    pub fn new(inner: Box<dyn Evaluator>) -> Self {
        Self { inner }
    }
}

#[async_trait::async_trait]
impl Evaluator for TracedGuard {
    async fn evaluate(
        &self,
        messages: &[ChatCompletionMessage],
        guard: &Guard,
    ) -> Result<GuardResult, String> {
        let span = info_span!(
            target: "langdb::user_tracing::guard",
            SPAN_GUARD_EVAULATION,
            id = guard.id(),
            label = guard.name(),
            user_input = JsonValue(&serde_json::to_value(guard.parameters()).map_err(|e| e.to_string())?).as_value(),
            result = field::Empty
        );

        let result = self
            .inner
            .evaluate(messages, guard)
            .instrument(span.clone())
            .await;
        let result_value = serde_json::to_value(result.clone()).map_err(|e| e.to_string())?;
        span.record("result", JsonValue(&result_value).as_value());

        result
    }
}
