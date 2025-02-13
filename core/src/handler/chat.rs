use std::collections::HashMap;

use crate::executor::chat_completion::execute;
use crate::routing::RouteStrategy;
use crate::types::gateway::ChatCompletionRequestWithTools;
use crate::types::gateway::CompletionModelUsage;
use crate::GatewayError;
use actix_web::{web, HttpRequest, HttpResponse};
use bytes::Bytes;
use either::Either::{Left, Right};
use futures::StreamExt;
use futures::TryStreamExt;

use crate::types::gateway::{
    ChatCompletionChunk, ChatCompletionChunkChoice, ChatCompletionDelta, ChatCompletionUsage,
    CostCalculator,
};
use opentelemetry::trace::TraceContextExt as _;
use tokio::sync::broadcast;
use tracing::Span;
use tracing_futures::Instrument;
use tracing_opentelemetry::OpenTelemetrySpanExt as _;

use crate::handler::find_model_by_full_name;
use crate::handler::AvailableModels;
use crate::handler::CallbackHandlerFn;
use crate::otel::{trace_id_uuid, TraceMap};
use crate::routing::LlmRouter;
use crate::GatewayApiError;

use super::can_execute_llm_for_request;

#[allow(clippy::too_many_arguments)]
pub async fn create_chat_completion(
    request: web::Json<ChatCompletionRequestWithTools>,
    callback_handler: web::Data<CallbackHandlerFn>,
    traces: web::Data<TraceMap>,
    req: HttpRequest,
    provided_models: web::Data<AvailableModels>,
    cost_calculator: web::Data<Box<dyn CostCalculator>>,
) -> Result<HttpResponse, GatewayApiError> {
    can_execute_llm_for_request(&req).await?;

    let span = Span::or_current(tracing::info_span!(
        target: "langdb::user_tracing::api_invoke",
        "api_invoke",
        request = tracing::field::Empty,
        response = tracing::field::Empty,
        error = tracing::field::Empty,
        thread_id = tracing::field::Empty,
        message_id = tracing::field::Empty,
    ));

    let mut request = request.into_inner();
    if request.request.model.starts_with("router/") {
        let router_name = request.request.model.split('/').last().unwrap().to_string();
        span.record("router_name", &router_name);
        if router_name == *"dynamic" {
            if let Some(router) = &request.routing {
                let llm_router = LlmRouter {
                    name: router.name.clone().unwrap_or("dynamic".to_string()),
                    strategy: router.strategy.clone(),
                    models: router.models.clone(),
                    max_cost: router.max_cost,
                    fallbacks: None,
                };
                let available_models = provided_models.get_ref();
                request.request = llm_router
                    .route(
                        request.request.clone(),
                        available_models.clone(),
                        req.headers()
                            .into_iter()
                            .map(|(k, v)| (k.to_string(), v.to_str().unwrap().to_string()))
                            .collect(),
                    )
                    .await?;
            }
        }
    }

    span.record("request", &serde_json::to_string(&request)?);
    let trace_id = span.context().span().span_context().trace_id();
    traces
        .entry(trace_id)
        .or_insert_with(|| broadcast::channel(8));

    let callback_handler = callback_handler.get_ref().clone();

    let model_name = request.request.model.clone();

    let available_models = provided_models.get_ref();
    let llm_model = find_model_by_full_name(&request.request.model, available_models)?;

    let response = execute(
        request,
        &callback_handler,
        req.clone(),
        cost_calculator.into_inner(),
        &llm_model,
    )
    .instrument(span.clone())
    .await?;

    let mut response_builder = HttpResponse::Ok();
    let builder = response_builder
        .insert_header(("X-Trace-Id", trace_id_uuid(trace_id).to_string()))
        .insert_header(("X-Model-Name", model_name.clone()))
        .insert_header((
            "X-Provider-Name",
            llm_model.inference_provider.provider.to_string(),
        ));

    match response {
        Left(result_stream) => {
            let result = result_stream?
                .map_err(|e| {
                    GatewayApiError::GatewayError(GatewayError::CustomError(e.to_string()))
                })
                .then(move |delta| {
                    let model_name = model_name.clone();
                    async move { map_sso_event(delta, model_name) }
                })
                .chain(futures::stream::once(async {
                    Ok::<_, GatewayApiError>(Bytes::from("data: [DONE]\n\n"))
                }));

            Ok(builder.content_type("text/event-stream").streaming(result))
        }
        Right(completions_response) => Ok(builder.json(completions_response?)),
    }
}

pub fn map_sso_event(
    delta: Result<(Option<ChatCompletionDelta>, Option<CompletionModelUsage>), GatewayApiError>,
    model_name: String,
) -> Result<Bytes, GatewayApiError> {
    let model_name = model_name.clone();
    let chunk = match delta {
        Ok((None, None)) => Ok(None),
        Ok((delta, usage)) => {
            let chunk = ChatCompletionChunk {
                id: uuid::Uuid::new_v4().to_string(),
                object: "chat.completion.chunk".to_string(),
                created: chrono::Utc::now().timestamp(),
                model: model_name.clone(),
                choices: delta.as_ref().map_or(vec![], |d| {
                    vec![ChatCompletionChunkChoice {
                        index: 0,
                        delta: d.clone(),
                        finish_reason: None,
                        logprobs: None,
                    }]
                }),
                usage: usage.as_ref().map(|u| ChatCompletionUsage {
                    prompt_tokens: u.input_tokens as i32,
                    completion_tokens: u.output_tokens as i32,
                    total_tokens: u.total_tokens as i32,
                    cost: 0.0,
                }),
            };

            Ok(Some(chunk))
        }
        Err(e) => Err(e),
    };

    let json_str = match chunk {
        Ok(r) => r.map(|c| {
            serde_json::to_string(&c)
                .unwrap_or_else(|e| format!("{{\"error\": \"Failed to serialize chunk: {}\"}}", e))
        }),
        Err(e) => Some(
            serde_json::to_string(&HashMap::from([("error", e.to_string())]))
                .unwrap_or_else(|e| format!("{{\"error\": \"Failed to serialize chunk: {}\"}}", e)),
        ),
    };

    let result = json_str
        .as_ref()
        .map_or(String::new(), |json_str| format!("data: {json_str}\n\n"));

    Ok(Bytes::from(result))
}
