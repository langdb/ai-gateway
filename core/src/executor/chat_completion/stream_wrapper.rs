use crate::handler::chat::SSOChatEvent;
use crate::GatewayApiError;
use futures::Stream;
use std::pin::Pin;

/// Type alias for the concrete stream type returned by chat completion functions
pub type ChatCompletionStream =
    Pin<Box<dyn Stream<Item = Result<SSOChatEvent, GatewayApiError>> + Send>>;

/// Wraps a stream into a boxed and pinned ChatCompletionStream
pub fn wrap_stream<S>(stream: S) -> ChatCompletionStream
where
    S: Stream<Item = Result<SSOChatEvent, GatewayApiError>> + Send + 'static,
{
    Box::pin(stream)
}
