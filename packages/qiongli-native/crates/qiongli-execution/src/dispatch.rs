use std::collections::BTreeMap;
use std::panic::AssertUnwindSafe;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use qiongli_project::ProjectId;
use qiongli_runtime::{FullProjectService, FullProjectServiceErrorKind, FullProjectToolId};
use serde_json::{Map, Value, json};

use crate::{
    CancellationToken, ExecutionError, ToolClass, ToolExecutionKind, ToolHostInvocationV1,
    ToolHostRegistry, ToolHostResultInput, ToolHostResultV1, ToolId, ToolRegistrationV1,
    ToolResultStatus,
};

const MAX_RESULT_DEPTH: usize = 64;
const MAX_RESULT_NODES: usize = 100_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ToolServiceError {
    reason_code: &'static str,
}

impl ToolServiceError {
    pub fn new(reason_code: &'static str) -> Result<Self, ExecutionError> {
        if !valid_reason_code(reason_code) {
            return Err(ExecutionError::ToolHostContractInvalid);
        }
        Ok(Self { reason_code })
    }

    #[must_use]
    pub const fn reason_code(self) -> &'static str {
        self.reason_code
    }
}

pub trait ReadOnlyToolService: Send + Sync {
    fn invoke(
        &self,
        request: ReadOnlyToolRequest<'_>,
        cancellation: &CancellationToken,
    ) -> Result<Value, ToolServiceError>;
}

#[derive(Clone, Copy)]
pub struct ReadOnlyToolRequest<'a> {
    pub arguments: &'a Value,
    pub project_id: Option<&'a ProjectId>,
    pub expected_project_revision: Option<u64>,
    pub project_root: Option<&'a std::path::Path>,
}

#[derive(Clone, Default)]
pub struct InProcessToolHost {
    registry: ToolHostRegistry,
    handlers: BTreeMap<ToolId, Arc<dyn ReadOnlyToolService>>,
}

impl InProcessToolHost {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            registry: ToolHostRegistry::new(),
            handlers: BTreeMap::new(),
        }
    }

    pub fn with_full_project_service(service: FullProjectService) -> Result<Self, ExecutionError> {
        let mut host = Self::new();
        for tool in full_project_tools() {
            let registration = full_project_registration(tool)?;
            if tool.is_read_only() {
                host.register_read_only(
                    registration,
                    Arc::new(FullProjectReadService {
                        service: service.clone(),
                        tool,
                    }),
                )?;
            } else {
                host.registry.register(registration)?;
            }
        }
        Ok(host)
    }

    pub fn register_read_only(
        &mut self,
        registration: ToolRegistrationV1,
        service: Arc<dyn ReadOnlyToolService>,
    ) -> Result<(), ExecutionError> {
        if registration.execution != ToolExecutionKind::InProcessReadOnly || !registration.read_only
        {
            return Err(ExecutionError::InvalidToolRegistration);
        }
        let tool_id = registration.tool_id.clone();
        self.registry.register(registration)?;
        if self.handlers.insert(tool_id, service).is_some() {
            return Err(ExecutionError::InvalidToolRegistration);
        }
        Ok(())
    }

    #[must_use]
    pub const fn registry(&self) -> &ToolHostRegistry {
        &self.registry
    }

    pub fn dispatch(
        &self,
        invocation: &ToolHostInvocationV1,
        cancellation: &CancellationToken,
    ) -> Result<ToolHostResultV1, ExecutionError> {
        invocation.validate()?;
        let registered = self
            .registry
            .registration(&invocation.request.tool_id)
            .ok_or(ExecutionError::ToolNotRegistered)?;
        if registered != &invocation.registration
            || registered.execution != ToolExecutionKind::InProcessReadOnly
            || !registered.read_only
        {
            return Err(ExecutionError::ToolHostContractInvalid);
        }
        let handler = self
            .handlers
            .get(&invocation.request.tool_id)
            .ok_or(ExecutionError::ToolNotRegistered)?;
        let started_at_unix_ms = now_unix_ms();
        let input_bytes = serialized_size(&invocation.request.arguments);

        let outcome = if cancellation.is_cancelled() {
            DispatchOutcome::cancelled()
        } else if input_bytes > invocation.limits.input_bytes {
            DispatchOutcome::limit_exceeded()
        } else {
            match std::panic::catch_unwind(AssertUnwindSafe(|| {
                handler.invoke(
                    ReadOnlyToolRequest {
                        arguments: &invocation.request.arguments,
                        project_id: invocation.request.project_id.as_ref(),
                        expected_project_revision: invocation.request.expected_project_revision,
                        project_root: invocation.project_root.as_deref(),
                    },
                    cancellation,
                )
            })) {
                Ok(Ok(_)) if cancellation.is_cancelled() => DispatchOutcome::cancelled(),
                Ok(Ok(mut content)) => {
                    let mut redaction_count = 0_u32;
                    let mut node_count = 0_usize;
                    if !sanitize_value(
                        &mut content,
                        &invocation.redaction,
                        0,
                        &mut node_count,
                        &mut redaction_count,
                    ) {
                        DispatchOutcome::limit_exceeded()
                    } else {
                        let output_bytes = serialized_size(&content);
                        if output_bytes > invocation.redaction.maximum_result_bytes
                            || output_bytes > invocation.limits.output_bytes
                        {
                            DispatchOutcome::limit_exceeded()
                        } else {
                            DispatchOutcome {
                                status: ToolResultStatus::Completed,
                                content,
                                reason_code: "tool-completed",
                                redaction_count,
                                truncated: false,
                            }
                        }
                    }
                }
                Ok(Err(error)) => DispatchOutcome::failed(error.reason_code()),
                Err(_) => DispatchOutcome::failed("tool-service-panicked"),
            }
        };
        let output_bytes = serialized_size(&outcome.content);
        ToolHostResultV1::from_redacted(
            invocation,
            ToolHostResultInput {
                status: outcome.status,
                content: outcome.content,
                truncated: outcome.truncated,
                redaction_count: outcome.redaction_count,
                reason_code: outcome.reason_code.to_string(),
                input_bytes,
                output_bytes,
                started_at_unix_ms,
                finished_at_unix_ms: now_unix_ms().max(started_at_unix_ms),
            },
        )
    }
}

struct DispatchOutcome {
    status: ToolResultStatus,
    content: Value,
    reason_code: &'static str,
    redaction_count: u32,
    truncated: bool,
}

impl DispatchOutcome {
    fn cancelled() -> Self {
        Self {
            status: ToolResultStatus::Cancelled,
            content: json!({"reasonCode": "tool-cancelled"}),
            reason_code: "tool-cancelled",
            redaction_count: 0,
            truncated: false,
        }
    }

    fn limit_exceeded() -> Self {
        Self {
            status: ToolResultStatus::LimitExceeded,
            content: json!({"reasonCode": "tool-result-limit-exceeded"}),
            reason_code: "tool-result-limit-exceeded",
            redaction_count: 0,
            truncated: true,
        }
    }

    fn failed(reason_code: &'static str) -> Self {
        Self {
            status: ToolResultStatus::Failed,
            content: json!({"reasonCode": reason_code}),
            reason_code,
            redaction_count: 0,
            truncated: false,
        }
    }
}

#[derive(Clone)]
struct FullProjectReadService {
    service: FullProjectService,
    tool: FullProjectToolId,
}

impl ReadOnlyToolService for FullProjectReadService {
    fn invoke(
        &self,
        request: ReadOnlyToolRequest<'_>,
        cancellation: &CancellationToken,
    ) -> Result<Value, ToolServiceError> {
        if cancellation.is_cancelled() {
            return Err(static_service_error("tool-cancelled"));
        }
        if full_project_registration(self.tool)
            .expect("static Full project registration is valid")
            .requires_project
        {
            self.validate_scope(request)?;
        }
        self.service
            .dispatch(self.tool, request.arguments)
            .map_err(|error| {
                let reason_code = if error.kind() == FullProjectServiceErrorKind::InvalidArguments {
                    "tool-arguments-invalid"
                } else {
                    error.reason_code()
                };
                ToolServiceError::new(reason_code).expect("shared service reason codes are valid")
            })
    }
}

impl FullProjectReadService {
    fn validate_scope(&self, request: ReadOnlyToolRequest<'_>) -> Result<(), ToolServiceError> {
        let (Some(project_id), Some(revision), Some(root)) = (
            request.project_id,
            request.expected_project_revision,
            request.project_root,
        ) else {
            return Err(static_service_error("tool-project-scope-required"));
        };
        if argument_project_id(self.tool, request.arguments)
            .is_some_and(|argument_id| argument_id != project_id.as_str())
        {
            return Err(static_service_error("tool-project-scope-mismatch"));
        }
        self.service
            .verify_project_scope(project_id, revision, root)
            .map_err(|error| static_service_error(error.reason_code()))
    }
}

fn argument_project_id(tool: FullProjectToolId, arguments: &Value) -> Option<&str> {
    match tool {
        FullProjectToolId::CapturePreview => arguments
            .pointer("/capture/binding/project_id")
            .and_then(Value::as_str),
        FullProjectToolId::Read
        | FullProjectToolId::GraphSnapshot
        | FullProjectToolId::GraphQuery
        | FullProjectToolId::ArtifactChanges
        | FullProjectToolId::CaptureCoverage => arguments.get("project_id").and_then(Value::as_str),
        FullProjectToolId::List
        | FullProjectToolId::GraphPortfolio
        | FullProjectToolId::CaptureApply => None,
    }
}

fn static_service_error(reason_code: &'static str) -> ToolServiceError {
    ToolServiceError::new(reason_code).expect("shared service reason codes are valid")
}

fn full_project_tools() -> [FullProjectToolId; 9] {
    [
        FullProjectToolId::List,
        FullProjectToolId::Read,
        FullProjectToolId::GraphSnapshot,
        FullProjectToolId::GraphPortfolio,
        FullProjectToolId::GraphQuery,
        FullProjectToolId::ArtifactChanges,
        FullProjectToolId::CaptureCoverage,
        FullProjectToolId::CapturePreview,
        FullProjectToolId::CaptureApply,
    ]
}

fn full_project_registration(
    tool: FullProjectToolId,
) -> Result<ToolRegistrationV1, ExecutionError> {
    let read_only = tool.is_read_only();
    Ok(ToolRegistrationV1 {
        schema_version: 1,
        tool_id: ToolId::parse(tool.public_name())?,
        class: if read_only {
            ToolClass::Read
        } else {
            ToolClass::ProjectWrite
        },
        execution: if read_only {
            ToolExecutionKind::InProcessReadOnly
        } else {
            ToolExecutionKind::ReservedChild
        },
        read_only,
        requires_project: !matches!(
            tool,
            FullProjectToolId::List | FullProjectToolId::GraphPortfolio
        ),
        requires_approval: !read_only,
        allows_network: false,
    })
}

fn sanitize_value(
    value: &mut Value,
    redaction: &crate::RedactionPolicyV1,
    depth: usize,
    node_count: &mut usize,
    redaction_count: &mut u32,
) -> bool {
    *node_count = node_count.saturating_add(1);
    if depth > MAX_RESULT_DEPTH || *node_count > MAX_RESULT_NODES {
        return false;
    }
    match value {
        Value::Array(values) => values
            .iter_mut()
            .all(|value| sanitize_value(value, redaction, depth + 1, node_count, redaction_count)),
        Value::Object(entries) => {
            sanitize_object(entries, redaction, depth, node_count, redaction_count)
        }
        Value::String(text) => {
            if (redaction.redact_absolute_paths && looks_like_absolute_path(text))
                || (redaction.redact_authorization_headers && looks_like_authorization(text))
            {
                *text = "<redacted>".to_string();
                *redaction_count = redaction_count.saturating_add(1);
            }
            true
        }
        Value::Null | Value::Bool(_) | Value::Number(_) => true,
    }
}

fn sanitize_object(
    entries: &mut Map<String, Value>,
    redaction: &crate::RedactionPolicyV1,
    depth: usize,
    node_count: &mut usize,
    redaction_count: &mut u32,
) -> bool {
    entries.iter_mut().all(|(key, value)| {
        if redaction.redact_secret_values && sensitive_key(key) {
            *value = Value::String("<redacted>".to_string());
            *redaction_count = redaction_count.saturating_add(1);
            true
        } else {
            sanitize_value(value, redaction, depth + 1, node_count, redaction_count)
        }
    })
}

fn sensitive_key(key: &str) -> bool {
    let key = key.to_ascii_lowercase();
    key == "authorization"
        || key == "credential"
        || key.ends_with("credential")
        || key == "secret"
        || key.ends_with("_secret")
        || key == "token"
        || key.ends_with("_token")
        || (key.ends_with("token") && !key.ends_with("tokens"))
        || key == "api_key"
        || key.ends_with("_api_key")
        || key == "apikey"
        || key.ends_with("apikey")
        || key == "password"
        || key.ends_with("_password")
        || key == "private_key"
        || key.ends_with("_private_key")
        || key == "privatekey"
        || key.ends_with("privatekey")
        || key == "cookie"
        || key == "set-cookie"
}

fn looks_like_authorization(value: &str) -> bool {
    let value = value.to_ascii_lowercase();
    value.contains("authorization:") || value.starts_with("bearer ") || value.starts_with("basic ")
}

fn looks_like_absolute_path(value: &str) -> bool {
    value.split_ascii_whitespace().any(|part| {
        let part = part.trim_matches(|character: char| {
            matches!(
                character,
                '"' | '\'' | '(' | ')' | '[' | ']' | '{' | '}' | ',' | ';'
            )
        });
        part.starts_with('/')
            || part.to_ascii_lowercase().starts_with("file://")
            || part.starts_with("\\\\")
            || (part.len() >= 3
                && part.as_bytes()[0].is_ascii_alphabetic()
                && part.as_bytes()[1] == b':'
                && matches!(part.as_bytes()[2], b'/' | b'\\'))
    })
}

fn serialized_size(value: &Value) -> u64 {
    serde_json::to_vec(value).map_or(u64::MAX, |bytes| bytes.len() as u64)
}

fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| {
            duration.as_millis().try_into().unwrap_or(u64::MAX)
        })
}

fn valid_reason_code(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 96
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use qiongli_config::resolve_config_root;
    use qiongli_project::ProjectStateService;
    use qiongli_runtime::FullProjectToolId;
    use serde_json::json;

    use crate::{
        AgentExecutionPolicy, ExecutionLimitsV1, ExecutionProfile, ExecutionUsageV1,
        PolicyToolRequestV1, RedactionPolicyV1, RunId, ToolCallId,
    };

    use super::*;

    #[derive(Clone)]
    struct FakeReadService {
        calls: Arc<AtomicUsize>,
        output: Value,
    }

    impl ReadOnlyToolService for FakeReadService {
        fn invoke(
            &self,
            _request: ReadOnlyToolRequest<'_>,
            _cancellation: &CancellationToken,
        ) -> Result<Value, ToolServiceError> {
            self.calls.fetch_add(1, Ordering::AcqRel);
            Ok(self.output.clone())
        }
    }

    fn registration() -> ToolRegistrationV1 {
        ToolRegistrationV1 {
            schema_version: 1,
            tool_id: ToolId::parse("project.read-metadata").unwrap(),
            class: ToolClass::Read,
            execution: ToolExecutionKind::InProcessReadOnly,
            read_only: true,
            requires_project: false,
            requires_approval: false,
            allows_network: false,
        }
    }

    fn host(output: Value, calls: Arc<AtomicUsize>) -> (InProcessToolHost, Arc<AtomicUsize>) {
        let mut host = InProcessToolHost::new();
        host.register_read_only(
            registration(),
            Arc::new(FakeReadService {
                calls: Arc::clone(&calls),
                output,
            }),
        )
        .unwrap();
        (host, calls)
    }

    fn invocation(host: &InProcessToolHost, limits: ExecutionLimitsV1) -> ToolHostInvocationV1 {
        let request = PolicyToolRequestV1 {
            schema_version: 1,
            run_id: RunId::parse(format!("run_{}", "3".repeat(32))).unwrap(),
            call_id: ToolCallId::parse(format!("call_{}", "4".repeat(32))).unwrap(),
            purpose: "Read bounded project metadata.".to_string(),
            tool_id: registration().tool_id,
            arguments: json!({"view": "summary"}),
            project_id: None,
            expected_project_revision: None,
            declared_artifacts: Vec::new(),
        };
        let policy = AgentExecutionPolicy::locked(
            1,
            ExecutionProfile::Full,
            [request.tool_id.clone()],
            None,
            limits,
            RedactionPolicyV1::strict_default(),
        )
        .unwrap();
        let decision = policy.evaluate(
            &registration(),
            &request,
            &ExecutionUsageV1::default(),
            None,
            100,
        );
        host.registry()
            .prepare(&policy, request, &decision)
            .unwrap()
    }

    #[test]
    fn dispatch_redacts_private_values_and_keeps_non_secret_token_counts() {
        let calls = Arc::new(AtomicUsize::new(0));
        let (host, calls) = host(
            json!({
                "path": "/private/project-canary",
                "authorization": "Bearer private-header-canary",
                "api_token": "private-token-canary",
                "password": "private-password-canary",
                "input_tokens": 17,
                "summary": "bounded metadata",
                "note": "See /private/embedded-path-canary for details"
            }),
            calls,
        );
        let result = host
            .dispatch(
                &invocation(&host, ExecutionLimitsV1::bounded_default()),
                &CancellationToken::new(),
            )
            .unwrap();
        assert_eq!(result.status, ToolResultStatus::Completed);
        assert_eq!(result.redaction_count, 5);
        assert_eq!(result.content["path"], "<redacted>");
        assert_eq!(result.content["authorization"], "<redacted>");
        assert_eq!(result.content["api_token"], "<redacted>");
        assert_eq!(result.content["password"], "<redacted>");
        assert_eq!(result.content["note"], "<redacted>");
        assert_eq!(result.content["input_tokens"], 17);
        assert_eq!(calls.load(Ordering::Acquire), 1);
        let audit = serde_json::to_string(&result.audit).unwrap();
        assert!(!audit.contains("private-project-canary"));
        assert!(!audit.contains("private-header-canary"));
        assert!(!audit.contains("private-token-canary"));
    }

    #[test]
    fn cancellation_and_input_limits_stop_before_the_service() {
        let calls = Arc::new(AtomicUsize::new(0));
        let (host, calls) = host(json!({"status": "unused"}), calls);
        let cancellation = CancellationToken::new();
        cancellation.cancel();
        let cancelled = host
            .dispatch(
                &invocation(&host, ExecutionLimitsV1::bounded_default()),
                &cancellation,
            )
            .unwrap();
        assert_eq!(cancelled.status, ToolResultStatus::Cancelled);

        let mut limits = ExecutionLimitsV1::bounded_default();
        limits.input_bytes = 1;
        let limited = host
            .dispatch(&invocation(&host, limits), &CancellationToken::new())
            .unwrap();
        assert_eq!(limited.status, ToolResultStatus::LimitExceeded);
        assert!(limited.truncated);
        assert_eq!(calls.load(Ordering::Acquire), 0);
    }

    #[test]
    fn full_project_writes_remain_reserved_child_only() {
        for tool in full_project_tools() {
            let registration = full_project_registration(tool).unwrap();
            assert_eq!(registration.read_only, tool.is_read_only());
            if tool == FullProjectToolId::CaptureApply {
                assert_eq!(registration.class, ToolClass::ProjectWrite);
                assert_eq!(registration.execution, ToolExecutionKind::ReservedChild);
                assert!(registration.requires_approval);
            } else {
                assert_eq!(registration.class, ToolClass::Read);
                assert_eq!(registration.execution, ToolExecutionKind::InProcessReadOnly);
                assert!(!registration.requires_approval);
            }
        }
    }

    #[test]
    fn full_project_handler_rejects_argument_scope_substitution() {
        let fixture_root =
            std::env::temp_dir().join(format!("qiongli-tool-host-scope-{}", std::process::id()));
        let config =
            resolve_config_root(Some(fixture_root.as_os_str()), &fixture_root.join("home"))
                .unwrap();
        let handler = FullProjectReadService {
            service: FullProjectService::new(ProjectStateService::new(config)),
            tool: FullProjectToolId::Read,
        };
        let allowed = ProjectId::parse(format!("prj_{}", "1".repeat(32))).unwrap();
        let substituted = ProjectId::parse(format!("prj_{}", "2".repeat(32))).unwrap();
        let arguments = json!({"project_id": substituted.as_str()});
        let error = handler
            .invoke(
                ReadOnlyToolRequest {
                    arguments: &arguments,
                    project_id: Some(&allowed),
                    expected_project_revision: Some(1),
                    project_root: Some(&fixture_root),
                },
                &CancellationToken::new(),
            )
            .unwrap_err();
        assert_eq!(error.reason_code(), "tool-project-scope-mismatch");
    }

    #[test]
    fn shared_full_project_list_dispatches_through_the_same_service() {
        let fixture_root = std::env::temp_dir().join(format!(
            "qiongli-tool-host-read-only-{}",
            std::process::id()
        ));
        let config =
            resolve_config_root(Some(fixture_root.as_os_str()), &fixture_root.join("home"))
                .unwrap();
        let host = InProcessToolHost::with_full_project_service(FullProjectService::new(
            ProjectStateService::new(config),
        ))
        .unwrap();
        assert_eq!(host.registry().len(), 9);

        let registration = host
            .registry()
            .registration(&ToolId::parse(FullProjectToolId::List.public_name()).unwrap())
            .unwrap()
            .clone();
        let request = PolicyToolRequestV1 {
            schema_version: 1,
            run_id: RunId::parse(format!("run_{}", "5".repeat(32))).unwrap(),
            call_id: ToolCallId::parse(format!("call_{}", "6".repeat(32))).unwrap(),
            purpose: "List registered projects.".to_string(),
            tool_id: registration.tool_id.clone(),
            arguments: json!({}),
            project_id: None,
            expected_project_revision: None,
            declared_artifacts: Vec::new(),
        };
        let policy = AgentExecutionPolicy::locked(
            1,
            ExecutionProfile::Full,
            [request.tool_id.clone()],
            None,
            ExecutionLimitsV1::bounded_default(),
            RedactionPolicyV1::strict_default(),
        )
        .unwrap();
        let decision = policy.evaluate(
            &registration,
            &request,
            &ExecutionUsageV1::default(),
            None,
            100,
        );
        let invocation = host
            .registry()
            .prepare(&policy, request, &decision)
            .unwrap();
        let result = host
            .dispatch(&invocation, &CancellationToken::new())
            .unwrap();
        assert_eq!(result.status, ToolResultStatus::Completed);
        assert_eq!(result.content["schemaVersion"], 1);
        assert_eq!(result.content["projects"], json!([]));
    }
}
