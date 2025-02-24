use std::collections::HashMap;
use std::sync::Arc;

use crate::model::types::LLMFinishEvent;
use crate::model::types::ModelEvent;
use crate::types::gateway::CompletionModelUsage;
use futures::future::join;
use futures::TryStreamExt;
use futures::{Stream, StreamExt};

use crate::{
    model::{
        types::{ModelEventType, ModelFinishReason},
        ModelInstance,
    },
    types::{
        engine::ParentCompletionOptions,
        gateway::{ChatCompletionDelta, FunctionCall, ToolCall},
        threads::Message,
    },
};
use tokio_stream::wrappers::ReceiverStream;
use tracing::Span;
use tracing_futures::Instrument;

use crate::handler::{CallbackHandlerFn, ModelEventWithDetails};
use crate::types::engine::CompletionModelDefinition;
use crate::types::engine::ParentDefinition;
use crate::GatewayApiError;

pub async fn stream_chunks(
    completion_model_definition: CompletionModelDefinition,
    model: Box<dyn ModelInstance>,
    messages: Vec<Message>,
    callback_handler: Arc<CallbackHandlerFn>,
    tags: HashMap<String, String>,
) -> Result<
    impl Stream<
        Item = Result<(Option<ChatCompletionDelta>, Option<CompletionModelUsage>), GatewayApiError>,
    >,
    GatewayApiError,
> {
    let parent_definition =
        ParentDefinition::CompletionModel(Box::new(completion_model_definition.clone()));
    let model_options = ParentCompletionOptions {
        definition: Box::new(parent_definition),
        named_args: Default::default(),
        verbose: true,
    };

    let db_model = model_options.definition.get_db_model();
    let (outer_tx, rx) = tokio::sync::mpsc::channel(100);

    tokio::spawn(
        async move {
            let (tx, mut rx) = tokio::sync::mpsc::channel(100);
            let result_fut = model
                .stream(HashMap::new(), tx, messages, tags)
                .instrument(Span::current());

            let forward_fut = async {
                let mut assistant_msg = String::new();
                while let Some(Some(mut msg)) = rx.recv().await {
                    if let ModelEventType::LlmContent(event) = &mut msg.event {
                        assistant_msg.push_str(event.content.as_str());
                    }

                    callback_handler
                        .on_message(ModelEventWithDetails::new(msg.clone(), db_model.clone()));
                    let e = outer_tx.send(Ok(msg)).await;
                    match e {
                        Ok(_) => {}
                        Err(e) => {
                            tracing::error!("Error in sending message: {e}");
                        }
                    }
                }

                let span = Span::current();
                span.record("response", assistant_msg.clone());
            };
            let (result, _) = join(result_fut, forward_fut).await;
            if let Err(e) = result {
                outer_tx
                    .send(Err(GatewayApiError::GatewayError(e)))
                    .await
                    .unwrap();
            }
        }
        .in_current_span(),
    );
    let event_stream = ReceiverStream::new(rx)
        .into_stream()
        .filter_map(|e: Result<ModelEvent, GatewayApiError>| async move {
            e.map_or_else(
                |e| Some(Err(e)),
                |model_event| match model_event.event {
                    ModelEventType::LlmContent(_)
                    | ModelEventType::ToolStart(_)
                    | ModelEventType::LlmStop(_) => Some(Ok(model_event)),
                    _ => None,
                },
            )
        })
        .then(move |e: Result<ModelEvent, GatewayApiError>| async move {
            match e {
                Ok(e) => match e.event {
                    ModelEventType::LlmContent(content) => Ok((
                        Some(ChatCompletionDelta {
                            role: Some("assistant".to_string()),
                            content: Some(content.content),
                            tool_calls: None,
                        }),
                        None,
                    )),
                    ModelEventType::ToolStart(tool_call) => Ok((
                        Some(ChatCompletionDelta {
                            role: Some("assistant".to_string()),
                            content: None,
                            tool_calls: Some(vec![ToolCall {
                                id: tool_call.tool_id.clone(),
                                r#type: "function".into(),
                                function: FunctionCall {
                                    name: tool_call.tool_name.clone(),
                                    arguments: tool_call.input.clone(),
                                },
                            }]),
                        }),
                        None,
                    )),
                    ModelEventType::LlmStop(LLMFinishEvent {
                        usage,
                        finish_reason,
                        tool_calls,
                        ..
                    }) => {
                        let ev = match finish_reason {
                            ModelFinishReason::ToolCalls => Some(ChatCompletionDelta {
                                role: Some("assistant".to_string()),
                                content: None,
                                tool_calls: Some(
                                    tool_calls
                                        .into_iter()
                                        .map(|tc| ToolCall {
                                            id: tc.tool_id.clone(),
                                            r#type: "function".into(),
                                            function: FunctionCall {
                                                name: tc.tool_name.clone(),
                                                arguments: tc.input.clone(),
                                            },
                                        })
                                        .collect(),
                                ),
                            }),
                            _ => None,
                        };

                        Ok((ev, usage))
                    }
                    _ => Err(GatewayApiError::CustomError(
                        "Unsupported event".to_string(),
                    )),
                },
                Err(e) => {
                    tracing::error!("Error in event: {e}");
                    Err(e)
                }
            }
        });

    Ok(event_stream)
}
