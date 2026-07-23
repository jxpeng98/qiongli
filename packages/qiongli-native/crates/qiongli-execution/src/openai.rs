use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::future::poll_fn;
use std::str;
use std::sync::{Arc, Mutex};
use std::task::{Poll, Waker};
use std::time::Duration;

use qiongli_config::{SecretRef, SecretStore, SecretStoreError, SecretStoreStatus};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::{
    AGENT_BACKEND_PROTOCOL_VERSION, AgentAuthState, AgentBackend, AgentBackendCapabilitiesV1,
    AgentBackendDescriptorV1, AgentBackendError, AgentBackendErrorCode, AgentBackendFuture,
    AgentCancellationSemantics, AgentEventStream, AgentEventV1, AgentFinishReason, AgentRequestV1,
    AgentRetryClass, AgentRole, AgentToolRequestV1, AgentUsageV1, BackendId, CancellationToken,
    RunId, ToolCallId,
};

const OPENAI_RESPONSES_URL: &str = "https://api.openai.com/v1/responses";
const OPENAI_MODEL: &str = "gpt-5.6-sol";
const MAX_RESPONSE_BYTES: usize = 8 * 1024 * 1024;
const MAX_PROVIDER_ID_BYTES: usize = 256;
const MAX_PROVIDER_CALLS: usize = 1024;
const MAX_EVENT_CONTENT_BYTES: usize = 64 * 1024;

/// Fixed configuration for the first direct OpenAI Responses API adapter.
///
/// The endpoint and model are deliberately not configurable in R4D so an
/// untrusted project cannot redirect a credential to another host.
#[derive(Clone)]
pub struct OpenAiBackendConfigV1 {
    secret_ref: SecretRef,
}

impl OpenAiBackendConfigV1 {
    #[must_use]
    pub fn gpt_5_6_sol(secret_ref: SecretRef) -> Self {
        Self { secret_ref }
    }

    #[must_use]
    pub const fn model(&self) -> &'static str {
        OPENAI_MODEL
    }
}

#[derive(Clone)]
pub struct OpenAiResponsesBackend {
    config: OpenAiBackendConfigV1,
    secrets: Arc<dyn SecretStore>,
    transport: Arc<dyn OpenAiTransport>,
    provider_calls: Arc<Mutex<BTreeMap<(String, String), ProviderCall>>>,
}

impl OpenAiResponsesBackend {
    pub fn new(
        config: OpenAiBackendConfigV1,
        secrets: Arc<dyn SecretStore>,
    ) -> Result<Self, AgentBackendError> {
        let transport = ReqwestOpenAiTransport::new()?;
        Ok(Self::with_transport(config, secrets, Arc::new(transport)))
    }

    fn with_transport(
        config: OpenAiBackendConfigV1,
        secrets: Arc<dyn SecretStore>,
        transport: Arc<dyn OpenAiTransport>,
    ) -> Self {
        Self {
            config,
            secrets,
            transport,
            provider_calls: Arc::new(Mutex::new(BTreeMap::new())),
        }
    }

    fn authentication(&self) -> AgentAuthState {
        if self.secrets.status() != SecretStoreStatus::Available {
            return AgentAuthState::Unavailable;
        }
        match self.secrets.resolve(&self.config.secret_ref) {
            Ok(secret) if str::from_utf8(secret.as_bytes()).is_ok() => AgentAuthState::Ready,
            Ok(_) => AgentAuthState::Unavailable,
            Err(SecretStoreError::NotFound) => AgentAuthState::MissingCredential,
            Err(SecretStoreError::Unavailable | SecretStoreError::PersistenceFailed) => {
                AgentAuthState::Unavailable
            }
        }
    }

    /// Removes provider call metadata after an orchestrator abandons a run.
    /// Completed non-tool responses clear this state automatically.
    pub fn forget_run(&self, run_id: &RunId) {
        clear_run_calls(&self.provider_calls, run_id);
    }
}

impl AgentBackend for OpenAiResponsesBackend {
    fn descriptor(&self) -> AgentBackendDescriptorV1 {
        AgentBackendDescriptorV1 {
            schema_version: 1,
            backend_id: BackendId::parse("openai-responses")
                .expect("static OpenAI backend identity is valid"),
            protocol_version: AGENT_BACKEND_PROTOCOL_VERSION.to_string(),
            authentication: self.authentication(),
            models: vec![OPENAI_MODEL.to_string()],
            capabilities: AgentBackendCapabilitiesV1 {
                maximum_context_tokens: 1_050_000,
                maximum_output_tokens: 128_000,
                // R4D first ships a bounded non-streaming HTTP adapter. The
                // AgentEventStream boundary still permits streaming later.
                streaming: false,
                structured_output: false,
                tool_calls: true,
                multimodal: false,
                cancellation: AgentCancellationSemantics::Cooperative,
                retry_classes: BTreeSet::from([
                    AgentRetryClass::RateLimited,
                    AgentRetryClass::NetworkTransient,
                    AgentRetryClass::ServerError,
                ]),
            },
            host_constraint_codes: vec![
                "direct-api-opt-in".to_string(),
                "no-hosted-tools".to_string(),
                "store-disabled".to_string(),
            ],
        }
    }

    fn start<'a>(
        &'a self,
        request: AgentRequestV1,
        cancellation: CancellationToken,
    ) -> AgentBackendFuture<'a, Result<Box<dyn AgentEventStream>, AgentBackendError>> {
        Box::pin(async move {
            validate_openai_request(&request)?;
            if cancellation.is_cancelled() {
                return Err(backend_error(AgentBackendErrorCode::Cancelled));
            }
            let prepared = build_request_body(&request, &self.provider_calls)?;
            let state = Arc::new(Mutex::new(OpenAiStreamState::Pending { waker: None }));
            let worker_state = Arc::clone(&state);
            let worker_cancellation = cancellation.clone();
            let stream_cancellation = cancellation;
            let transport = Arc::clone(&self.transport);
            let secrets = Arc::clone(&self.secrets);
            let secret_ref = self.config.secret_ref.clone();
            let provider_calls = Arc::clone(&self.provider_calls);
            let run_id = request.run_id.clone();
            let body = prepared.body;
            let offered_tools = prepared.tools_by_provider_name;
            std::thread::Builder::new()
                .name("qiongli-openai-responses".to_string())
                .spawn(move || {
                    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        let normalization = NormalizationContext {
                            run_id: &run_id,
                            offered_tools: &offered_tools,
                            provider_calls: &provider_calls,
                        };
                        run_worker(
                            transport.as_ref(),
                            secrets.as_ref(),
                            &secret_ref,
                            &body,
                            &worker_cancellation,
                            &normalization,
                        )
                    }))
                    .unwrap_or_else(|_| Err(backend_error(AgentBackendErrorCode::ResponseInvalid)));
                    finish_stream(&worker_state, result);
                })
                .map_err(|_| {
                    AgentBackendError::new(
                        AgentBackendErrorCode::TransportUnavailable,
                        Some(AgentRetryClass::NetworkTransient),
                    )
                })?;
            Ok(Box::new(OpenAiEventStream {
                state,
                worker_cancellation: stream_cancellation,
                terminal: false,
            }) as Box<dyn AgentEventStream>)
        })
    }
}

fn validate_openai_request(request: &AgentRequestV1) -> Result<(), AgentBackendError> {
    if request.validate().is_err()
        || request.model != OPENAI_MODEL
        || !request.attachments.is_empty()
        || request.response.structured_output_schema.is_some()
        || request.response.maximum_output_tokens > 128_000
    {
        return Err(backend_error(AgentBackendErrorCode::InvalidRequest));
    }
    Ok(())
}

#[derive(Clone)]
struct ProviderCall {
    provider_call_id: String,
    provider_tool_name: String,
    arguments: Value,
}

struct NormalizationContext<'a> {
    run_id: &'a RunId,
    offered_tools: &'a BTreeMap<String, String>,
    provider_calls: &'a Mutex<BTreeMap<(String, String), ProviderCall>>,
}

struct PreparedOpenAiRequest {
    body: Value,
    tools_by_provider_name: BTreeMap<String, String>,
}

fn build_request_body(
    request: &AgentRequestV1,
    provider_calls: &Mutex<BTreeMap<(String, String), ProviderCall>>,
) -> Result<PreparedOpenAiRequest, AgentBackendError> {
    let provider_calls = provider_calls
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut input = Vec::with_capacity(request.messages.len());
    for message in &request.messages {
        if message.role == AgentRole::Tool {
            let call_id = message
                .tool_call_id
                .as_ref()
                .ok_or_else(|| backend_error(AgentBackendErrorCode::InvalidRequest))?;
            let call = provider_calls
                .get(&(
                    request.run_id.as_str().to_string(),
                    call_id.as_str().to_string(),
                ))
                .ok_or_else(|| backend_error(AgentBackendErrorCode::InvalidRequest))?;
            input.push(json!({
                "type": "function_call",
                "call_id": call.provider_call_id,
                "name": call.provider_tool_name,
                "arguments": serde_json::to_string(&call.arguments)
                    .map_err(|_| backend_error(AgentBackendErrorCode::InvalidRequest))?,
            }));
            input.push(json!({
                "type": "function_call_output",
                "call_id": call.provider_call_id,
                "output": message.content,
            }));
        } else {
            input.push(json!({
                "role": role_name(message.role),
                "content": message.content,
            }));
        }
    }
    let mut tools_by_provider_name = BTreeMap::new();
    let tools = request
        .tools
        .iter()
        .map(|tool| {
            let provider_name = provider_tool_name(&tool.name);
            if tools_by_provider_name
                .insert(provider_name.clone(), tool.name.clone())
                .is_some()
            {
                return Err(backend_error(AgentBackendErrorCode::InvalidRequest));
            }
            Ok(json!({
                "type": "function",
                "name": provider_name,
                "description": tool.description,
                "parameters": tool.input_schema,
                "strict": false,
            }))
        })
        .collect::<Result<Vec<_>, AgentBackendError>>()?;
    let mut body = json!({
        "model": OPENAI_MODEL,
        "store": false,
        "input": input,
        "max_output_tokens": request.response.maximum_output_tokens,
    });
    if !tools.is_empty() {
        body["tools"] = Value::Array(tools);
        body["tool_choice"] = Value::String("auto".to_string());
    }
    Ok(PreparedOpenAiRequest {
        body,
        tools_by_provider_name,
    })
}

fn provider_tool_name(tool_name: &str) -> String {
    if tool_name.len() <= 64
        && tool_name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
    {
        return tool_name.to_string();
    }
    let mut stem = tool_name
        .bytes()
        .take(47)
        .map(|byte| {
            if byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-') {
                char::from(byte)
            } else {
                '_'
            }
        })
        .collect::<String>();
    while stem.ends_with('_') {
        stem.pop();
    }
    if stem.is_empty() {
        stem.push_str("tool");
    }
    let digest = format!("{:x}", Sha256::digest(tool_name.as_bytes()));
    format!("{stem}_{}", &digest[..12])
}

const fn role_name(role: AgentRole) -> &'static str {
    match role {
        AgentRole::System => "system",
        AgentRole::User => "user",
        AgentRole::Assistant => "assistant",
        AgentRole::Tool => "tool",
    }
}

fn run_worker(
    transport: &dyn OpenAiTransport,
    secrets: &dyn SecretStore,
    secret_ref: &SecretRef,
    body: &Value,
    cancellation: &CancellationToken,
    normalization: &NormalizationContext<'_>,
) -> Result<VecDeque<Result<AgentEventV1, AgentBackendError>>, AgentBackendError> {
    if cancellation.is_cancelled() {
        return Err(backend_error(AgentBackendErrorCode::Cancelled));
    }
    let secret = secrets.resolve(secret_ref).map_err(map_secret_error)?;
    let credential = str::from_utf8(secret.as_bytes())
        .map_err(|_| backend_error(AgentBackendErrorCode::AuthenticationUnavailable))?;
    let response = transport.send(credential, body, cancellation)?;
    if cancellation.is_cancelled() {
        return Err(backend_error(AgentBackendErrorCode::Cancelled));
    }
    normalize_response(
        response,
        normalization.run_id,
        normalization.offered_tools,
        normalization.provider_calls,
    )
}

fn normalize_response(
    response: Value,
    run_id: &RunId,
    offered_tools: &BTreeMap<String, String>,
    provider_calls: &Mutex<BTreeMap<(String, String), ProviderCall>>,
) -> Result<VecDeque<Result<AgentEventV1, AgentBackendError>>, AgentBackendError> {
    if serde_json::to_vec(&response).map_or(true, |bytes| bytes.len() > MAX_RESPONSE_BYTES) {
        return Err(backend_error(AgentBackendErrorCode::ResponseInvalid));
    }
    let output = response
        .get("output")
        .and_then(Value::as_array)
        .ok_or_else(|| backend_error(AgentBackendErrorCode::ResponseInvalid))?;
    let status = required_string(&response, "status")?;
    let reached_output_limit = status == "incomplete"
        && response
            .pointer("/incomplete_details/reason")
            .and_then(Value::as_str)
            == Some("max_output_tokens");
    if status != "completed" && !reached_output_limit {
        return Err(backend_error(AgentBackendErrorCode::ResponseInvalid));
    }
    let mut events = VecDeque::new();
    let mut calls = Vec::new();
    for item in output {
        match required_string(item, "type")? {
            "message" => normalize_message(item, &mut events)?,
            "function_call" => calls.push(normalize_function_call(item, run_id, offered_tools)?),
            "reasoning" => {}
            _ => return Err(backend_error(AgentBackendErrorCode::ResponseInvalid)),
        }
    }
    if let Some(usage) = response.get("usage") {
        events.push_back(Ok(AgentEventV1::Usage {
            usage: normalize_usage(usage)?,
        }));
    }
    let finish_reason = if calls.is_empty() {
        clear_run_calls(provider_calls, run_id);
        if reached_output_limit {
            AgentFinishReason::Length
        } else {
            AgentFinishReason::Stop
        }
    } else {
        if status != "completed" {
            return Err(backend_error(AgentBackendErrorCode::ResponseInvalid));
        }
        insert_provider_calls(provider_calls, run_id, &calls)?;
        for (request, _) in calls {
            events.push_back(Ok(AgentEventV1::ToolRequest { request }));
        }
        AgentFinishReason::ToolRequest
    };
    events.push_back(Ok(AgentEventV1::Completed { finish_reason }));
    Ok(events)
}

fn normalize_message(
    item: &Value,
    events: &mut VecDeque<Result<AgentEventV1, AgentBackendError>>,
) -> Result<(), AgentBackendError> {
    let content = item
        .get("content")
        .and_then(Value::as_array)
        .ok_or_else(|| backend_error(AgentBackendErrorCode::ResponseInvalid))?;
    for part in content {
        match required_string(part, "type")? {
            "output_text" => push_content(events, required_string(part, "text")?)?,
            "refusal" => {
                let _ = required_string(part, "refusal")?;
                return Err(backend_error(AgentBackendErrorCode::ProviderRejected));
            }
            _ => return Err(backend_error(AgentBackendErrorCode::ResponseInvalid)),
        }
    }
    Ok(())
}

fn push_content(
    events: &mut VecDeque<Result<AgentEventV1, AgentBackendError>>,
    content: &str,
) -> Result<(), AgentBackendError> {
    if content.is_empty() {
        return Err(backend_error(AgentBackendErrorCode::ResponseInvalid));
    }
    let mut start = 0;
    while start < content.len() {
        let mut end = (start + MAX_EVENT_CONTENT_BYTES).min(content.len());
        while !content.is_char_boundary(end) {
            end -= 1;
        }
        let event = AgentEventV1::ContentDelta {
            content: content[start..end].to_string(),
        };
        event
            .validate()
            .map_err(|_| backend_error(AgentBackendErrorCode::ResponseInvalid))?;
        events.push_back(Ok(event));
        start = end;
    }
    Ok(())
}

fn normalize_function_call(
    item: &Value,
    run_id: &RunId,
    offered_tools: &BTreeMap<String, String>,
) -> Result<(AgentToolRequestV1, ProviderCall), AgentBackendError> {
    let provider_call_id = required_string(item, "call_id")?;
    if provider_call_id.is_empty() || provider_call_id.len() > MAX_PROVIDER_ID_BYTES {
        return Err(backend_error(AgentBackendErrorCode::ResponseInvalid));
    }
    let provider_tool_name = required_string(item, "name")?;
    let tool_name = offered_tools
        .get(provider_tool_name)
        .ok_or_else(|| backend_error(AgentBackendErrorCode::ResponseInvalid))?;
    let arguments: Value = serde_json::from_str(required_string(item, "arguments")?)
        .map_err(|_| backend_error(AgentBackendErrorCode::ResponseInvalid))?;
    if !arguments.is_object() {
        return Err(backend_error(AgentBackendErrorCode::ResponseInvalid));
    }
    let call_id = normalized_call_id(run_id, provider_call_id);
    let request = AgentToolRequestV1 {
        call_id,
        tool_name: tool_name.to_string(),
        arguments: arguments.clone(),
    };
    request_event_valid(&request)?;
    Ok((
        request,
        ProviderCall {
            provider_call_id: provider_call_id.to_string(),
            provider_tool_name: provider_tool_name.to_string(),
            arguments,
        },
    ))
}

fn request_event_valid(request: &AgentToolRequestV1) -> Result<(), AgentBackendError> {
    AgentEventV1::ToolRequest {
        request: request.clone(),
    }
    .validate()
    .map_err(|_| backend_error(AgentBackendErrorCode::ResponseInvalid))
}

fn normalized_call_id(run_id: &RunId, provider_call_id: &str) -> ToolCallId {
    let digest = Sha256::digest(format!("{}\0{provider_call_id}", run_id.as_str()).as_bytes());
    let digest = format!("{digest:x}");
    ToolCallId::parse(format!("call_{}", &digest[..32]))
        .expect("SHA-256 creates a valid normalized call identity")
}

fn insert_provider_calls(
    provider_calls: &Mutex<BTreeMap<(String, String), ProviderCall>>,
    run_id: &RunId,
    new_calls: &[(AgentToolRequestV1, ProviderCall)],
) -> Result<(), AgentBackendError> {
    let mut calls = provider_calls
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut staged = BTreeMap::new();
    for (_, call) in new_calls {
        let call_id = normalized_call_id(run_id, &call.provider_call_id);
        let key = (run_id.as_str().to_string(), call_id.as_str().to_string());
        if staged.insert(key.clone(), call.clone()).is_some()
            || calls
                .get(&key)
                .is_some_and(|existing| existing.provider_call_id != call.provider_call_id)
        {
            return Err(backend_error(AgentBackendErrorCode::ResponseInvalid));
        }
    }
    let additional = staged
        .keys()
        .filter(|key| !calls.contains_key(*key))
        .count();
    if calls.len().saturating_add(additional) > MAX_PROVIDER_CALLS {
        return Err(backend_error(AgentBackendErrorCode::ResponseInvalid));
    }
    calls.extend(staged);
    Ok(())
}

fn clear_run_calls(
    provider_calls: &Mutex<BTreeMap<(String, String), ProviderCall>>,
    run_id: &RunId,
) {
    provider_calls
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .retain(|(stored_run_id, _), _| stored_run_id != run_id.as_str());
}

fn normalize_usage(value: &Value) -> Result<AgentUsageV1, AgentBackendError> {
    let input_tokens = required_u64(value, "input_tokens")?;
    let output_tokens = required_u64(value, "output_tokens")?;
    let cached_input_tokens = value
        .pointer("/input_tokens_details/cached_tokens")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let usage = AgentUsageV1 {
        input_tokens,
        output_tokens,
        cached_input_tokens,
    };
    AgentEventV1::Usage {
        usage: usage.clone(),
    }
    .validate()
    .map_err(|_| backend_error(AgentBackendErrorCode::ResponseInvalid))?;
    Ok(usage)
}

fn required_string<'a>(value: &'a Value, key: &str) -> Result<&'a str, AgentBackendError> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| backend_error(AgentBackendErrorCode::ResponseInvalid))
}

fn required_u64(value: &Value, key: &str) -> Result<u64, AgentBackendError> {
    value
        .get(key)
        .and_then(Value::as_u64)
        .ok_or_else(|| backend_error(AgentBackendErrorCode::ResponseInvalid))
}

fn map_secret_error(error: SecretStoreError) -> AgentBackendError {
    match error {
        SecretStoreError::NotFound
        | SecretStoreError::Unavailable
        | SecretStoreError::PersistenceFailed => {
            backend_error(AgentBackendErrorCode::AuthenticationUnavailable)
        }
    }
}

const fn backend_error(code: AgentBackendErrorCode) -> AgentBackendError {
    AgentBackendError::new(code, None)
}

trait OpenAiTransport: Send + Sync {
    fn send(
        &self,
        credential: &str,
        body: &Value,
        cancellation: &CancellationToken,
    ) -> Result<Value, AgentBackendError>;
}

struct ReqwestOpenAiTransport {
    client: reqwest::blocking::Client,
}

impl ReqwestOpenAiTransport {
    fn new() -> Result<Self, AgentBackendError> {
        let client = reqwest::blocking::Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(120))
            .redirect(reqwest::redirect::Policy::none())
            .no_proxy()
            .build()
            .map_err(|_| {
                AgentBackendError::new(
                    AgentBackendErrorCode::TransportUnavailable,
                    Some(AgentRetryClass::NetworkTransient),
                )
            })?;
        Ok(Self { client })
    }
}

impl OpenAiTransport for ReqwestOpenAiTransport {
    fn send(
        &self,
        credential: &str,
        body: &Value,
        cancellation: &CancellationToken,
    ) -> Result<Value, AgentBackendError> {
        if cancellation.is_cancelled() {
            return Err(backend_error(AgentBackendErrorCode::Cancelled));
        }
        let response = self
            .client
            .post(OPENAI_RESPONSES_URL)
            .bearer_auth(credential)
            .json(body)
            .send()
            .map_err(map_reqwest_error)?;
        let status = response.status();
        if !status.is_success() {
            return Err(map_http_status(status.as_u16()));
        }
        if response
            .content_length()
            .is_some_and(|length| length > MAX_RESPONSE_BYTES as u64)
        {
            return Err(backend_error(AgentBackendErrorCode::ResponseInvalid));
        }
        let bytes = response.bytes().map_err(map_reqwest_error)?;
        if bytes.len() > MAX_RESPONSE_BYTES {
            return Err(backend_error(AgentBackendErrorCode::ResponseInvalid));
        }
        serde_json::from_slice(&bytes)
            .map_err(|_| backend_error(AgentBackendErrorCode::ResponseInvalid))
    }
}

fn map_reqwest_error(error: reqwest::Error) -> AgentBackendError {
    let retry = if error.is_timeout() || error.is_connect() {
        Some(AgentRetryClass::NetworkTransient)
    } else {
        None
    };
    AgentBackendError::new(AgentBackendErrorCode::TransportUnavailable, retry)
}

const fn map_http_status(status: u16) -> AgentBackendError {
    match status {
        401 | 403 => backend_error(AgentBackendErrorCode::AuthenticationUnavailable),
        408 => AgentBackendError::new(
            AgentBackendErrorCode::TransportUnavailable,
            Some(AgentRetryClass::NetworkTransient),
        ),
        429 => AgentBackendError::new(
            AgentBackendErrorCode::ProviderRejected,
            Some(AgentRetryClass::RateLimited),
        ),
        500..=599 => AgentBackendError::new(
            AgentBackendErrorCode::TransportUnavailable,
            Some(AgentRetryClass::ServerError),
        ),
        _ => backend_error(AgentBackendErrorCode::ProviderRejected),
    }
}

enum OpenAiStreamState {
    Pending { waker: Option<Waker> },
    Ready(VecDeque<Result<AgentEventV1, AgentBackendError>>),
}

struct OpenAiEventStream {
    state: Arc<Mutex<OpenAiStreamState>>,
    worker_cancellation: CancellationToken,
    terminal: bool,
}

impl AgentEventStream for OpenAiEventStream {
    fn next_event<'a>(
        &'a mut self,
        cancellation: &'a CancellationToken,
    ) -> AgentBackendFuture<'a, Option<Result<AgentEventV1, AgentBackendError>>> {
        Box::pin(poll_fn(move |context| {
            if self.terminal {
                return Poll::Ready(None);
            }
            if cancellation.is_cancelled() {
                self.worker_cancellation.cancel();
                self.terminal = true;
                return Poll::Ready(Some(Err(backend_error(AgentBackendErrorCode::Cancelled))));
            }
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            match &mut *state {
                OpenAiStreamState::Pending { waker } => {
                    *waker = Some(context.waker().clone());
                    Poll::Pending
                }
                OpenAiStreamState::Ready(events) => match events.pop_front() {
                    Some(event) => Poll::Ready(Some(event)),
                    None => {
                        self.terminal = true;
                        Poll::Ready(None)
                    }
                },
            }
        }))
    }
}

fn finish_stream(
    state: &Mutex<OpenAiStreamState>,
    result: Result<VecDeque<Result<AgentEventV1, AgentBackendError>>, AgentBackendError>,
) {
    let mut state = state
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let waker = match &mut *state {
        OpenAiStreamState::Pending { waker } => waker.take(),
        OpenAiStreamState::Ready(_) => None,
    };
    *state = OpenAiStreamState::Ready(match result {
        Ok(events) => events,
        Err(error) => VecDeque::from([Err(error)]),
    });
    drop(state);
    if let Some(waker) = waker {
        waker.wake();
    }
}

#[cfg(test)]
mod tests {
    use std::future::Future;
    use std::pin::Pin;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::task::{Context, Poll, Waker};
    use std::time::{Duration, Instant};

    use qiongli_config::{SecretValue, SecretValueError};

    use crate::{AgentMessageV1, AgentResponseConstraintsV1, AgentToolSchemaV1, ExecutionError};

    use super::*;

    #[derive(Clone)]
    struct TestSecretStore {
        value: Vec<u8>,
    }

    impl SecretStore for TestSecretStore {
        fn status(&self) -> SecretStoreStatus {
            SecretStoreStatus::Available
        }

        fn resolve(&self, _secret_ref: &SecretRef) -> Result<SecretValue, SecretStoreError> {
            SecretValue::new(self.value.clone())
                .map_err(|SecretValueError| SecretStoreError::PersistenceFailed)
        }
    }

    #[derive(Clone)]
    struct FakeTransport {
        responses: Arc<Mutex<VecDeque<Value>>>,
        bodies: Arc<Mutex<Vec<Value>>>,
        credentials_seen: Arc<AtomicUsize>,
    }

    impl FakeTransport {
        fn new(responses: Vec<Value>) -> Self {
            Self {
                responses: Arc::new(Mutex::new(responses.into())),
                bodies: Arc::new(Mutex::new(Vec::new())),
                credentials_seen: Arc::new(AtomicUsize::new(0)),
            }
        }
    }

    impl OpenAiTransport for FakeTransport {
        fn send(
            &self,
            credential: &str,
            body: &Value,
            _cancellation: &CancellationToken,
        ) -> Result<Value, AgentBackendError> {
            assert_eq!(credential, "test-secret");
            self.credentials_seen.fetch_add(1, Ordering::AcqRel);
            self.bodies
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(body.clone());
            self.responses
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .pop_front()
                .ok_or_else(|| backend_error(AgentBackendErrorCode::ResponseInvalid))
        }
    }

    fn secret_ref() -> SecretRef {
        SecretRef::parse(&format!("qsr1_{}", "a".repeat(32))).unwrap()
    }

    fn backend(transport: FakeTransport) -> OpenAiResponsesBackend {
        OpenAiResponsesBackend::with_transport(
            OpenAiBackendConfigV1::gpt_5_6_sol(secret_ref()),
            Arc::new(TestSecretStore {
                value: b"test-secret".to_vec(),
            }),
            Arc::new(transport),
        )
    }

    fn request() -> AgentRequestV1 {
        AgentRequestV1 {
            schema_version: 1,
            run_id: RunId::parse(format!("run_{}", "1".repeat(32))).unwrap(),
            model: OPENAI_MODEL.to_string(),
            messages: vec![AgentMessageV1 {
                role: AgentRole::User,
                content: "Summarize the evidence.".to_string(),
                tool_call_id: None,
            }],
            attachments: Vec::new(),
            response: AgentResponseConstraintsV1 {
                maximum_output_tokens: 512,
                structured_output_schema: None,
            },
            tools: Vec::new(),
        }
    }

    fn ready<T>(future: impl Future<Output = T>) -> T {
        let mut future = Box::pin(future);
        let waker = Waker::noop();
        let mut context = Context::from_waker(waker);
        let started = Instant::now();
        loop {
            match Pin::new(&mut future).poll(&mut context) {
                Poll::Ready(value) => return value,
                Poll::Pending if started.elapsed() < Duration::from_secs(2) => {
                    std::thread::sleep(Duration::from_millis(1));
                }
                Poll::Pending => panic!("bounded backend test timed out"),
            }
        }
    }

    fn collect_events(
        backend: &OpenAiResponsesBackend,
        request: AgentRequestV1,
    ) -> Result<Vec<AgentEventV1>, AgentBackendError> {
        let cancellation = CancellationToken::new();
        let mut stream = ready(backend.start(request, cancellation.clone()))?;
        let mut events = Vec::new();
        while let Some(event) = ready(stream.next_event(&cancellation)) {
            events.push(event?);
        }
        Ok(events)
    }

    #[test]
    fn descriptor_is_truthful_and_fixed_to_the_direct_api() {
        let backend = backend(FakeTransport::new(Vec::new()));
        let descriptor = backend.descriptor();
        assert!(descriptor.validate().is_ok());
        assert_eq!(descriptor.authentication, AgentAuthState::Ready);
        assert_eq!(descriptor.models, [OPENAI_MODEL]);
        assert!(!descriptor.capabilities.streaming);
        assert!(descriptor.capabilities.tool_calls);
        assert!(!descriptor.capabilities.multimodal);
        assert!(
            descriptor
                .host_constraint_codes
                .contains(&"store-disabled".to_string())
        );
    }

    #[test]
    fn normalizes_text_and_usage_without_leaking_the_secret() {
        let transport = FakeTransport::new(vec![json!({
            "status": "completed",
            "output": [{
                "type": "message",
                "content": [{"type": "output_text", "text": "Bounded answer."}]
            }],
            "usage": {
                "input_tokens": 12,
                "output_tokens": 3,
                "input_tokens_details": {"cached_tokens": 2}
            }
        })]);
        let backend = backend(transport.clone());
        let events = collect_events(&backend, request()).unwrap();
        assert!(matches!(
            &events[0],
            AgentEventV1::ContentDelta { content } if content == "Bounded answer."
        ));
        assert!(matches!(&events[1], AgentEventV1::Usage { usage } if usage.input_tokens == 12));
        assert!(matches!(
            events[2],
            AgentEventV1::Completed {
                finish_reason: AgentFinishReason::Stop
            }
        ));
        let bodies = transport
            .bodies
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        assert_eq!(bodies[0]["store"], false);
        assert!(!bodies[0].to_string().contains("test-secret"));
        assert_eq!(transport.credentials_seen.load(Ordering::Acquire), 1);
    }

    #[test]
    fn function_call_identity_round_trips_without_provider_storage() {
        let provider_name = provider_tool_name("project.graph-query");
        let transport = FakeTransport::new(vec![
            json!({
                "status": "completed",
                "output": [{
                    "type": "function_call",
                    "call_id": "provider-call-17",
                    "name": provider_name.clone(),
                    "arguments": "{\"query\":\"citation graph\"}"
                }],
                "usage": {"input_tokens": 8, "output_tokens": 4}
            }),
            json!({
                "status": "completed",
                "output": [{
                    "type": "message",
                    "content": [{"type": "output_text", "text": "Graph inspected."}]
                }],
                "usage": {"input_tokens": 18, "output_tokens": 3}
            }),
        ]);
        let backend = backend(transport.clone());
        let mut first = request();
        first.tools.push(AgentToolSchemaV1 {
            name: "project.graph-query".to_string(),
            description: "Query the local project graph.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {"query": {"type": "string"}},
                "required": ["query"],
                "additionalProperties": false
            }),
        });
        let events = collect_events(&backend, first.clone()).unwrap();
        let call_id = match &events[1] {
            AgentEventV1::ToolRequest { request } => {
                assert_eq!(request.tool_name, "project.graph-query");
                request.call_id.clone()
            }
            _ => panic!("expected normalized tool request"),
        };
        first.messages.push(AgentMessageV1 {
            role: AgentRole::Tool,
            content: "{\"nodes\":3}".to_string(),
            tool_call_id: Some(call_id),
        });
        let events = collect_events(&backend, first).unwrap();
        assert!(matches!(
            &events[0],
            AgentEventV1::ContentDelta { content } if content == "Graph inspected."
        ));
        let bodies = transport
            .bodies
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let input = bodies[1]["input"].as_array().unwrap();
        assert_eq!(input[input.len() - 2]["type"], "function_call");
        assert_eq!(input[input.len() - 2]["call_id"], "provider-call-17");
        assert_eq!(input[input.len() - 2]["name"], provider_name);
        assert_eq!(input[input.len() - 1]["type"], "function_call_output");
        assert_eq!(bodies[1]["store"], false);
        assert_eq!(bodies[0]["tools"][0]["strict"], false);
        assert_eq!(bodies[0]["tools"][0]["name"], provider_name);
        assert!(!provider_name.contains('.'));
    }

    #[test]
    fn rejects_provider_tools_that_were_not_offered() {
        let backend = backend(FakeTransport::new(vec![json!({
            "status": "completed",
            "output": [{
                "type": "function_call",
                "call_id": "provider-call-18",
                "name": "shell.exec",
                "arguments": "{}"
            }]
        })]));
        let error = match collect_events(&backend, request()) {
            Ok(_) => panic!("an unoffered provider tool must fail closed"),
            Err(error) => error,
        };
        assert_eq!(error.code, AgentBackendErrorCode::ResponseInvalid);
    }

    #[test]
    fn tool_messages_require_a_call_identity() {
        let mut request = request();
        request.messages[0].role = AgentRole::Tool;
        assert_eq!(request.validate(), Err(ExecutionError::InvalidAgentRequest));
    }
}
