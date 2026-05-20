use crate::common::ResponseEvent;
use crate::common::ResponseStream;
use crate::error::ApiError;
use crate::telemetry::SseTelemetry;
use codex_client::ByteStream;
use codex_client::StreamResponse;
use codex_protocol::models::ContentItem;
use codex_protocol::models::MessagePhase;
use codex_protocol::models::ResponseItem;
use codex_protocol::protocol::TokenUsage;
use eventsource_stream::Eventsource;
use futures::StreamExt;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::OnceLock;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::Instant;
use tokio::time::timeout;
use tracing::debug;
use tracing::trace;

const REQUEST_ID_HEADER: &str = "x-request-id";
const OPENAI_MODEL_HEADER: &str = "openai-model";

pub fn spawn_chat_completions_stream(
    stream_response: StreamResponse,
    idle_timeout: Duration,
    telemetry: Option<Arc<dyn SseTelemetry>>,
    turn_state: Option<Arc<OnceLock<String>>>,
) -> ResponseStream {
    let upstream_request_id = stream_response
        .headers
        .get(REQUEST_ID_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);

    let server_model = stream_response
        .headers
        .get(OPENAI_MODEL_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(ToString::to_string);

    if let Some(turn_state) = turn_state.as_ref()
        && let Some(header_value) = stream_response
            .headers
            .get("x-codex-turn-state")
            .and_then(|v| v.to_str().ok())
    {
        let _ = turn_state.set(header_value.to_string());
    }

    let (tx_event, rx_event) = mpsc::channel::<Result<ResponseEvent, ApiError>>(1600);

    tokio::spawn(async move {
        if let Some(model) = server_model {
            let _ = tx_event.send(Ok(ResponseEvent::ServerModel(model))).await;
        }
        // Emit Created to satisfy consumers that expect it
        if tx_event.send(Ok(ResponseEvent::Created)).await.is_err() {
            return;
        }
        process_chat_completions_sse(
            stream_response.bytes,
            tx_event,
            idle_timeout,
            telemetry,
        )
        .await;
    });

    ResponseStream {
        rx_event,
        upstream_request_id,
    }
}

// ── SSE deserialization types ────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct ChatChunk {
    id: Option<String>,
    model: Option<String>,
    choices: Option<Vec<ChatChunkChoice>>,
    usage: Option<ChatChunkUsage>,
}

#[derive(Debug, Deserialize)]
struct ChatChunkChoice {
    delta: Option<ChatChunkDelta>,
    #[serde(default)]
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ChatChunkDelta {
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    reasoning_content: Option<String>,
    #[serde(default)]
    tool_calls: Option<Vec<ChatChunkToolCallDelta>>,
}

#[derive(Debug, Deserialize)]
struct ChatChunkToolCallDelta {
    index: i32,
    id: Option<String>,
    function: Option<ChatChunkFunctionDelta>,
}

#[derive(Debug, Deserialize)]
struct ChatChunkFunctionDelta {
    name: Option<String>,
    arguments: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ChatChunkUsage {
    prompt_tokens: Option<i64>,
    completion_tokens: Option<i64>,
    total_tokens: i64,
}

#[derive(Debug, Deserialize)]
struct ChatErrorPayload {
    error: Option<ChatErrorDetail>,
}

#[derive(Debug, Deserialize)]
struct ChatErrorDetail {
    code: Option<String>,
    message: Option<String>,
}

// ── Delta accumulation ───────────────────────────────────────────────

#[derive(Default)]
struct AccumulatedToolCall {
    id: String,
    name: String,
    arguments: String,
}

#[derive(Default)]
struct DeltaAccumulator {
    content: String,
    tool_calls: BTreeMap<i32, AccumulatedToolCall>,
}

#[derive(Default)]
struct ChatChunkAccumulator {
    choices: BTreeMap<i32, DeltaAccumulator>,
    last_server_model_sent: Option<String>,
}

// ── Main SSE loop ────────────────────────────────────────────────────

async fn process_chat_completions_sse(
    stream: ByteStream,
    tx_event: mpsc::Sender<Result<ResponseEvent, ApiError>>,
    idle_timeout: Duration,
    telemetry: Option<Arc<dyn SseTelemetry>>,
) {
    let mut stream = stream.eventsource();
    let mut acc = ChatChunkAccumulator::default();
    let mut response_id: Option<String> = None;

    loop {
        let start = Instant::now();
        let response = timeout(idle_timeout, stream.next()).await;
        if let Some(t) = telemetry.as_ref() {
            t.on_sse_poll(&response, start.elapsed());
        }
        let sse = match response {
            Ok(Some(Ok(sse))) => sse,
            Ok(Some(Err(e))) => {
                debug!("SSE Error: {e:#}");
                let _ = tx_event.send(Err(ApiError::Stream(e.to_string()))).await;
                return;
            }
            Ok(None) => {
                // Stream ended — emit completed
                emit_completed(&acc, &tx_event, response_id.as_deref()).await;
                return;
            }
            Err(_) => {
                let _ = tx_event
                    .send(Err(ApiError::Stream("idle timeout waiting for SSE".into())))
                    .await;
                return;
            }
        };

        trace!("Chat SSE: {}", &sse.data);

        // [DONE] marker
        if sse.data.trim() == "[DONE]" {
            emit_completed(&acc, &tx_event, response_id.as_deref()).await;
            return;
        }

        // Check for error payload first
        if let Ok(error_payload) = serde_json::from_str::<ChatErrorPayload>(&sse.data) {
            if let Some(detail) = error_payload.error {
                let msg = detail.message.unwrap_or_else(|| "chat completions error".to_string());
                let api_error = categorize_chat_error(detail.code.as_deref(), &msg);
                let _ = tx_event.send(Err(api_error)).await;
                return;
            }
        }

        // Parse standard chunk
        let chunk: ChatChunk = match serde_json::from_str(&sse.data) {
            Ok(chunk) => chunk,
            Err(e) => {
                debug!("Failed to parse chat completions chunk: {e}, data: {}", &sse.data);
                continue;
            }
        };

        // Capture response id from first chunk
        if response_id.is_none() {
            response_id = chunk.id.clone();
        }

        // Track model changes
        if let Some(ref model) = chunk.model
            && acc.last_server_model_sent.as_deref() != Some(model.as_str())
        {
            acc.last_server_model_sent = Some(model.clone());
            if tx_event
                .send(Ok(ResponseEvent::ServerModel(model.clone())))
                .await
                .is_err()
            {
                return;
            }
        }

        // Process choices
        if let Some(choices) = &chunk.choices {
            for (_ci, choice) in choices.iter().enumerate() {
                let idx = 0i32;
                let choice_acc = acc.choices.entry(idx).or_default();

                if let Some(delta) = &choice.delta {
                    // 1. Reasoning content (DeepSeek / o1-style)
                    if let Some(ref reasoning) = delta.reasoning_content {
                        if !choice_acc.content.is_empty() {
                            // flush accumulated text before reasoning
                            emit_accumulated_text(choice_acc, &tx_event).await;
                        }
                        let content_index = 0i64;
                        if tx_event
                            .send(Ok(ResponseEvent::ReasoningContentDelta {
                                delta: reasoning.clone(),
                                content_index,
                            }))
                            .await
                            .is_err()
                        {
                            return;
                        }
                    }

                    // 2. Tool calls delta
                    if let Some(tool_calls) = &delta.tool_calls {
                        for tc in tool_calls {
                            let tc_acc = choice_acc.tool_calls.entry(tc.index).or_default();
                            if let Some(ref id) = tc.id {
                                tc_acc.id = id.clone();
                            }
                            if let Some(ref func) = tc.function {
                                if let Some(ref name) = func.name {
                                    tc_acc.name.push_str(name);
                                }
                                if let Some(ref args) = func.arguments {
                                    tc_acc.arguments.push_str(args);
                                    if tx_event
                                        .send(Ok(ResponseEvent::ToolCallInputDelta {
                                            item_id: tc_acc.id.clone(),
                                            call_id: Some(tc_acc.id.clone()),
                                            delta: args.clone(),
                                        }))
                                        .await
                                        .is_err()
                                    {
                                        return;
                                    }
                                }
                            }
                        }
                    }

                    // 3. Text content delta
                    if let Some(ref text) = delta.content {
                        choice_acc.content.push_str(text);
                        if tx_event
                            .send(Ok(ResponseEvent::OutputTextDelta(text.clone())))
                            .await
                            .is_err()
                        {
                            return;
                        }
                    }
                }

                // 4. Finish reason handling
                if let Some(ref reason) = choice.finish_reason
                    && !reason.is_empty()
                    && reason != "null"
                {
                    match reason.as_str() {
                        "stop" | "length" => {
                            emit_accumulated_text(choice_acc, &tx_event).await;
                        }
                        "tool_calls" => {
                            emit_tool_call_output_items(choice_acc, &tx_event).await;
                        }
                        "content_filter" => {
                            emit_accumulated_text(choice_acc, &tx_event).await;
                        }
                        _ => {}
                    }
                }
            }
        }

        // 5. Usage in chunk indicates terminal
        if let Some(usage) = chunk.usage {
            let c_usage = TokenUsage {
                input_tokens: usage.prompt_tokens.unwrap_or(0),
                cached_input_tokens: 0,
                output_tokens: usage.completion_tokens.unwrap_or(0),
                reasoning_output_tokens: 0,
                total_tokens: usage.total_tokens,
            };
            let resp_id = response_id.clone().unwrap_or_default();
            if tx_event
                .send(Ok(ResponseEvent::Completed {
                    response_id: resp_id,
                    token_usage: Some(c_usage),
                    end_turn: Some(true),
                }))
                .await
                .is_err()
            {
                return;
            }
            return;
        }
    }
}

// ── Helpers ──────────────────────────────────────────────────────────

async fn emit_accumulated_text(
    choice_acc: &DeltaAccumulator,
    tx: &mpsc::Sender<Result<ResponseEvent, ApiError>>,
) {
    if !choice_acc.content.is_empty() {
        let item = ResponseItem::Message {
            id: None,
            role: "assistant".to_string(),
            content: vec![ContentItem::OutputText {
                text: choice_acc.content.clone(),
            }],
            phase: Some(MessagePhase::FinalAnswer),
        };
        let _ = tx.send(Ok(ResponseEvent::OutputItemDone(item))).await;
    }
}

async fn emit_tool_call_output_items(
    choice_acc: &DeltaAccumulator,
    tx: &mpsc::Sender<Result<ResponseEvent, ApiError>>,
) {
    for tc_acc in choice_acc.tool_calls.values() {
        if !tc_acc.id.is_empty() && !tc_acc.name.is_empty() {
            let item = ResponseItem::FunctionCall {
                id: None,
                name: tc_acc.name.clone(),
                namespace: None,
                arguments: tc_acc.arguments.clone(),
                call_id: tc_acc.id.clone(),
            };
            let _ = tx.send(Ok(ResponseEvent::OutputItemDone(item))).await;
        }
    }
}

async fn emit_completed(
    _acc: &ChatChunkAccumulator,
    tx: &mpsc::Sender<Result<ResponseEvent, ApiError>>,
    response_id: Option<&str>,
) {
    let _ = tx
        .send(Ok(ResponseEvent::Completed {
            response_id: response_id.unwrap_or("").to_string(),
            token_usage: None,
            end_turn: Some(true),
        }))
        .await;
}

fn categorize_chat_error(code: Option<&str>, message: &str) -> ApiError {
    match code {
        Some("context_length_exceeded") => ApiError::ContextWindowExceeded,
        Some("insufficient_quota") => ApiError::QuotaExceeded,
        Some("rate_limit_exceeded") => ApiError::Retryable {
            message: message.to_string(),
            delay: None,
        },
        Some("invalid_request_error") => ApiError::InvalidRequest {
            message: message.to_string(),
        },
        Some("server_error") => ApiError::Retryable {
            message: message.to_string(),
            delay: None,
        },
        _ => ApiError::Stream(message.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use codex_client::TransportError;
    use codex_protocol::models::ResponseItem;
    use futures::TryStreamExt;
    use tokio::sync::mpsc;
    use tokio_test::io::Builder as IoBuilder;
    use tokio_util::io::ReaderStream;

    fn idle_timeout() -> Duration {
        Duration::from_millis(1000)
    }

    async fn collect_events(chunks: &[&[u8]]) -> Vec<Result<ResponseEvent, ApiError>> {
        let mut builder = IoBuilder::new();
        for chunk in chunks {
            builder.read(chunk);
        }
        let reader = builder.build();
        let stream =
            ReaderStream::new(reader).map_err(|err| TransportError::Network(err.to_string()));
        let (tx, mut rx) = mpsc::channel::<Result<ResponseEvent, ApiError>>(16);

        // Emit Created first
        let _ = tx.send(Ok(ResponseEvent::Created));
        tokio::spawn(process_chat_completions_sse(
            Box::pin(stream),
            tx,
            idle_timeout(),
            None,
        ));

        let mut events = Vec::new();
        while let Some(ev) = rx.recv().await {
            events.push(ev);
        }
        events
    }

    #[tokio::test]
    async fn parses_simple_text_chunks() {
        let chunk1 = r#"data: {"id":"chatcmpl-1","object":"chat.completion.chunk","choices":[{"delta":{"content":"Hello"},"finish_reason":null}]}

"#;
        let chunk2 = r#"data: {"id":"chatcmpl-1","object":"chat.completion.chunk","choices":[{"delta":{"content":" World"},"finish_reason":null}]}

"#;
        let chunk3 = r#"data: {"id":"chatcmpl-1","object":"chat.completion.chunk","choices":[{"delta":{},"finish_reason":"stop"}],"usage":{"prompt_tokens":10,"completion_tokens":2,"total_tokens":12}}

"#;

        let events = collect_events(&[chunk1.as_bytes(), chunk2.as_bytes(), chunk3.as_bytes()]).await;

        // Created + 2 OutputTextDelta + OutputItemDone(Message) + Completed
        assert_eq!(events.len(), 5);
    }

    #[tokio::test]
    async fn parses_reasoning_content_chunks() {
        let chunk = r#"data: {"id":"chatcmpl-1","choices":[{"delta":{"reasoning_content":"Let me think..."},"finish_reason":null}]}

"#;
        let final_chunk = r#"data: {"id":"chatcmpl-1","choices":[{"delta":{"content":"Answer"},"finish_reason":"stop"}],"usage":{"prompt_tokens":5,"completion_tokens":1,"total_tokens":6}}

"#;

        let events = collect_events(&[chunk.as_bytes(), final_chunk.as_bytes()]).await;

        let has_reasoning = events.iter().any(|e| {
            matches!(e, Ok(ResponseEvent::ReasoningContentDelta { .. }))
        });
        assert!(has_reasoning, "expected ReasoningContentDelta in {events:?}");
    }

    #[tokio::test]
    async fn handles_done_marker() {
        let done = "data: [DONE]\n\n";

        let events = collect_events(&[done.as_bytes()]).await;

        assert_eq!(events.len(), 2); // Created + Completed
        assert!(matches!(&events[1], Ok(ResponseEvent::Completed { .. })));
    }

    #[tokio::test]
    async fn handles_error_chunk() {
        let error_chunk = r#"data: {"error":{"code":"context_length_exceeded","message":"Too long"}}

"#;

        let events = collect_events(&[error_chunk.as_bytes()]).await;

        assert!(matches!(&events[1], Err(ApiError::ContextWindowExceeded)));
    }

    #[tokio::test]
    async fn handles_stream_without_completed() {
        // Stream ends without usage or [DONE]
        let mut builder = IoBuilder::new();
        builder.read(b"data: {\"id\":\"c1\",\"choices\":[{\"delta\":{\"content\":\"Hi\"},\"finish_reason\":null}]}\n\n");
        let reader = builder.build();
        let stream = ReaderStream::new(reader).map_err(|err| TransportError::Network(err.to_string()));
        let (tx, mut rx) = mpsc::channel::<Result<ResponseEvent, ApiError>>(8);

        let _ = tx.send(Ok(ResponseEvent::Created));
        tokio::spawn(process_chat_completions_sse(
            Box::pin(stream),
            tx,
            idle_timeout(),
            None,
        ));

        let mut events = Vec::new();
        while let Some(ev) = rx.recv().await {
            events.push(ev);
        }
        // Should end with a Completed event (synthesized on stream close)
        assert!(events.last().is_some_and(|e| matches!(e, Ok(ResponseEvent::Completed { .. }))));
    }

    #[tokio::test]
    async fn parses_tool_call_delta_chunks() {
        let chunks = [
            br#"data: {"id":"c1","choices":[{"delta":{"tool_calls":[{"index":0,"id":"call_1","type":"function","function":{"name":"get_weather","arguments":""}}]},"finish_reason":null}]}

"# as &[u8],
            br#"data: {"id":"c1","choices":[{"delta":{"tool_calls":[{"index":0,"function":{"arguments":"{\"city\":\"NY\"}"}}]},"finish_reason":null}]}

"#,
            br#"data: {"id":"c1","choices":[{"delta":{},"finish_reason":"tool_calls"}],"usage":{"prompt_tokens":10,"completion_tokens":3,"total_tokens":13}}

"#,
        ];
        let events = collect_events(&chunks).await;

        let has_tool_delta = events.iter().any(|e| {
            matches!(e, Ok(ResponseEvent::ToolCallInputDelta { .. }))
        });
        assert!(has_tool_delta, "expected ToolCallInputDelta in {events:?}");
    }
}
