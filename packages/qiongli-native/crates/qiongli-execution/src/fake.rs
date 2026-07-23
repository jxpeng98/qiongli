use std::collections::{BTreeSet, VecDeque};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use crate::{
    AGENT_BACKEND_PROTOCOL_VERSION, AgentAuthState, AgentBackend, AgentBackendCapabilitiesV1,
    AgentBackendDescriptorV1, AgentBackendError, AgentBackendErrorCode, AgentBackendFuture,
    AgentCancellationSemantics, AgentEventStream, AgentEventV1, AgentRequestV1, AgentRetryClass,
    BackendId, CancellationToken, ExecutionError,
};

#[derive(Clone)]
pub struct DeterministicFakeBackend {
    descriptor: AgentBackendDescriptorV1,
    scripts: Arc<Vec<Vec<Result<AgentEventV1, AgentBackendError>>>>,
    starts: Arc<AtomicUsize>,
    last_request: Arc<Mutex<Option<AgentRequestV1>>>,
}

impl DeterministicFakeBackend {
    pub fn new(
        script: Vec<Result<AgentEventV1, AgentBackendError>>,
    ) -> Result<Self, ExecutionError> {
        Self::from_turns(vec![script])
    }

    pub fn from_turns(
        scripts: Vec<Vec<Result<AgentEventV1, AgentBackendError>>>,
    ) -> Result<Self, ExecutionError> {
        if scripts.is_empty()
            || scripts
                .iter()
                .flatten()
                .filter_map(|event| event.as_ref().ok())
                .any(|event| event.validate().is_err())
        {
            return Err(ExecutionError::InvalidAgentRequest);
        }
        Ok(Self {
            descriptor: AgentBackendDescriptorV1 {
                schema_version: 1,
                backend_id: BackendId::parse("deterministic-fake")
                    .expect("static fake backend identity is valid"),
                protocol_version: AGENT_BACKEND_PROTOCOL_VERSION.to_string(),
                authentication: AgentAuthState::NotRequired,
                models: vec!["deterministic-v1".to_string()],
                capabilities: AgentBackendCapabilitiesV1 {
                    maximum_context_tokens: 32_000,
                    maximum_output_tokens: 4_096,
                    streaming: true,
                    structured_output: true,
                    tool_calls: true,
                    multimodal: true,
                    cancellation: AgentCancellationSemantics::Cooperative,
                    retry_classes: BTreeSet::from([
                        AgentRetryClass::NetworkTransient,
                        AgentRetryClass::ProviderBusy,
                    ]),
                },
                host_constraint_codes: vec!["test-only".to_string()],
            },
            scripts: Arc::new(scripts),
            starts: Arc::new(AtomicUsize::new(0)),
            last_request: Arc::new(Mutex::new(None)),
        })
    }

    #[must_use]
    pub fn start_count(&self) -> usize {
        self.starts.load(Ordering::Acquire)
    }

    #[must_use]
    pub fn last_request(&self) -> Option<AgentRequestV1> {
        self.last_request
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
}

impl AgentBackend for DeterministicFakeBackend {
    fn descriptor(&self) -> AgentBackendDescriptorV1 {
        self.descriptor.clone()
    }

    fn start<'a>(
        &'a self,
        request: AgentRequestV1,
        cancellation: CancellationToken,
    ) -> AgentBackendFuture<'a, Result<Box<dyn AgentEventStream>, AgentBackendError>> {
        Box::pin(async move {
            if cancellation.is_cancelled() {
                return Err(AgentBackendError::new(
                    AgentBackendErrorCode::Cancelled,
                    None,
                ));
            }
            if request.validate().is_err() {
                return Err(AgentBackendError::new(
                    AgentBackendErrorCode::InvalidRequest,
                    None,
                ));
            }
            let turn = self.starts.fetch_add(1, Ordering::AcqRel);
            *self
                .last_request
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(request);
            let events = self
                .scripts
                .get(turn)
                .ok_or_else(|| {
                    AgentBackendError::new(AgentBackendErrorCode::ResponseInvalid, None)
                })?
                .iter()
                .cloned()
                .collect::<VecDeque<_>>();
            Ok(Box::new(FakeEventStream { events }) as Box<dyn AgentEventStream>)
        })
    }
}

struct FakeEventStream {
    events: VecDeque<Result<AgentEventV1, AgentBackendError>>,
}

impl AgentEventStream for FakeEventStream {
    fn next_event<'a>(
        &'a mut self,
        cancellation: &'a CancellationToken,
    ) -> AgentBackendFuture<'a, Option<Result<AgentEventV1, AgentBackendError>>> {
        Box::pin(async move {
            if cancellation.is_cancelled() {
                Some(Err(AgentBackendError::new(
                    AgentBackendErrorCode::Cancelled,
                    None,
                )))
            } else {
                self.events.pop_front()
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use std::future::Future;
    use std::task::{Context, Poll, Waker};

    use crate::{AgentFinishReason, AgentMessageV1, AgentResponseConstraintsV1, AgentRole, RunId};

    use super::*;

    fn ready<T>(mut future: impl Future<Output = T> + Unpin) -> T {
        let waker = Waker::noop();
        let mut context = Context::from_waker(waker);
        match std::pin::Pin::new(&mut future).poll(&mut context) {
            Poll::Ready(value) => value,
            Poll::Pending => panic!("deterministic fake future must be immediately ready"),
        }
    }

    fn request() -> AgentRequestV1 {
        AgentRequestV1 {
            schema_version: 1,
            run_id: RunId::parse(format!("run_{}", "2".repeat(32))).unwrap(),
            model: "deterministic-v1".to_string(),
            messages: vec![AgentMessageV1 {
                role: AgentRole::User,
                content: "Return one deterministic event.".to_string(),
                tool_call_id: None,
            }],
            attachments: Vec::new(),
            response: AgentResponseConstraintsV1 {
                maximum_output_tokens: 128,
                structured_output_schema: None,
            },
            tools: Vec::new(),
        }
    }

    #[test]
    fn fake_backend_replays_events_and_observes_cancellation() {
        let backend = DeterministicFakeBackend::new(vec![Ok(AgentEventV1::Completed {
            finish_reason: AgentFinishReason::Stop,
        })])
        .unwrap();
        let cancellation = CancellationToken::new();
        let mut stream = ready(backend.start(request(), cancellation.clone())).unwrap();
        assert!(matches!(
            ready(stream.next_event(&cancellation)),
            Some(Ok(AgentEventV1::Completed { .. }))
        ));
        assert!(ready(stream.next_event(&cancellation)).is_none());
        assert_eq!(backend.start_count(), 1);
        assert!(backend.last_request().is_some());

        let cancellation = CancellationToken::new();
        cancellation.cancel();
        let error = match ready(backend.start(request(), cancellation)) {
            Ok(_) => panic!("cancelled fake start must fail"),
            Err(error) => error,
        };
        assert_eq!(error.code, AgentBackendErrorCode::Cancelled);
        assert_eq!(backend.start_count(), 1);
    }
}
