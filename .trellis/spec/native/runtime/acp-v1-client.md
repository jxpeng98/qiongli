# ACP v1 client boundary

## 1. Scope / Trigger

Use this contract when native code connects Qiongli to Codex or Claude through
ACP. It keeps the first integration on stable ACP v1, prevents arbitrary process
launch, and keeps SDK transport objects out of Qiongli-owned state and App APIs.

## 2. Signatures

The native owner is `qiongli-execution`:

```rust
AcpV1Client::for_development_npx(
    preset: AcpDevelopmentPresetV1,
) -> AcpV1Client

AcpV1Client::run_turn(
    self,
    cwd: impl AsRef<Path>,
    prompt: impl Into<String>,
    cancellation: CancellationToken,
) -> Result<AcpV1TurnOutcome, AgentBackendError>

AcpV1Client::with_session<R>(
    self,
    cwd: impl AsRef<Path>,
    cancellation: CancellationToken,
    use_session: impl AsyncFnOnce(&mut AcpV1Session) -> Result<R, AgentBackendError>,
) -> Result<R, AgentBackendError>

AcpV1Client::with_timeouts(self, timeouts: AcpV1Timeouts) -> Result<Self, AgentBackendError>
AcpV1Client::with_control(self, control: AcpV1Control) -> Self
AcpV1Client::with_read_view(self, files: Vec<(String, String)>) -> Result<Self, AgentBackendError>
AcpV1Control::new(run_id: RunId, role: OrchestrationRole) -> Result<Self, AgentBackendError>
AcpV1Control::apply(&self, request: AcpV1ControlRequest) -> Result<(), AgentBackendError>
AcpV1Control::try_next_update(&self) -> Option<AcpV1Update>
AcpV1Control::next_update(&self) -> Option<AcpV1Update> // async

AcpV1Session::info(&self) -> &AcpV1SessionInfo
AcpV1Session::session_id(&self) -> &str
AcpV1Session::run_turn(
    &mut self,
    prompt: impl Into<String>,
    cancellation: CancellationToken,
) -> Result<AcpV1TurnOutcome, AgentBackendError>

AcpV1TurnOutcome::protocol_version(&self) -> u16
AcpV1TurnOutcome::session_id(&self) -> &str
AcpV1TurnOutcome::turn_id(&self) -> u64
AcpV1TurnOutcome::events(&self) -> &[AgentEventV1]

AcpV1TurnOutcome::project_first_coordinator_turn(
    &self,
    state: &mut AllChatStateV1,
    expected_generation: u64,
) -> Result<(), AllChatStateError>
```

Both client operations and session `run_turn` are asynchronous. Client operations
consume the factory; `with_session` lends one reusable session to its callback.
Client `run_turn` remains the single-turn convenience wrapper.
`AcpDevelopmentPresetV1` is closed to `Codex` and `Claude`.

## 3. Contracts

- Always send `InitializeRequest` with `ProtocolVersion::V1` and reject any
  negotiated version other than v1.
- The development constructor is explicit and is not packaged support. It uses
  direct argv only: `npx -y @agentclientprotocol/codex-acp@1.9.0` or
  `npx -y @agentclientprotocol/claude-agent-acp@0.74.0`.
- Callers cannot supply a command, argv, shell fragment, environment override,
  registry package, or ACP SDK transport/schema object.
- One client initializes once and creates one session for the callback lifetime.
  Mutable borrowing serializes prompts and prevents the session from escaping
  the scope. No project/App lock spans a protocol wait. Session load, capability
  controls and packaged sidecars remain later contracts.
- Each submitted prompt receives a session-local monotonic turn ID, starting at
  1 and bounded to 9,007,199,254,740,991. Each outcome owns its events; content and
  update limits reset per turn. Input rejection or pre-cancellation consumes no
  ID. A failed or dropped in-flight prompt retires the session; queued messages
  cannot be consumed by another prompt. A validated terminal outcome permits reuse.
- The `with_session` cancellation token covers startup; prompts supply their
  own tokens. Ending/dropping the scope releases the SDK process guard. On Unix,
  SDK 2.1.0 kills its owned process group, including wrapper descendants. Windows
  development `npx` launch fails before spawning (`CapabilityUnavailable`) until
  an owned Windows Job guarantees descendant cleanup; the SDK only kills the
  immediate Windows child. In-process transport remains usable for offline tests.
- Only bounded text chunks become `AgentEventV1::ContentDelta`.
  `EndTurn` becomes completed/stop, `MaxTokens` and `MaxTurnRequests` become
  completed/length, and `Cancelled` becomes a cancelled event only after
  Qiongli sent `session/cancel`.
- Without an explicitly attached `AcpV1Control`, permissions retain the original
  fail-closed behavior: answer ACP `Cancelled`, suppress the accumulated outcome
  and return `CapabilityUnavailable`. A control never grants Qiongli project-write
  authority; those writes still require the existing preview/approval/CAS owners.
- `CancellationToken::cancelled()` wakes all observers without Agent output;
  its existing synchronous API remains compatible. Race startup requests and
  every update against cancellation/deadlines. Send `session/cancel` once and
  suppress later text/activity. Only Agent-confirmed `StopReason::Cancelled`
  produces a cancelled outcome. Missing acknowledgement ends the scoped runner
  as `TransportUnavailable`; it is never reported as confirmed cancellation.
- Provider session IDs are opaque and bounded. They do not grant project or tool
  authority. No credentials, absolute paths, SDK types, or streamed content are
  written to project state by this boundary.
- Bind the owned session during `session/new` response dispatch, before any
  immediately following updates. Validate `session/update` at connection ingress
  and pass valid messages through to SDK routing. Malformed or unowned updates
  abort the runner and latch `ResponseInvalid`, discarding all accumulated output
  even if a successful stop races the abort. Do not wait for a terminal update.
  An SDK notification-handler error alone is insufficient: the SDK only logs it.
- Prompt-response dispatch closes the update window until the next prompt.
  Updates observed while idle between prompts fail as `ResponseInvalid`. ACP v1
  updates have no turn ID: local IDs distinguish serialized prompt windows, but
  cannot prove causal origin for a late update arriving after the next prompt
  begins. Preserve the existing initial-owned-update behavior after session/new.
- The first coordinator projection is atomic. Completed text chunks become one
  bounded coordinator message followed by an Agent-turn completion event;
  confirmed cancellation discards every partial chunk and records only the
  ready session plus turn cancellation. Neither outcome completes the run. The
  caller supplies the existing generation CAS value; event sequence is derived
  from the staged All Chat state. Only turn 1 may use this projection; later
  turns require the subsequent App/state integration contract.

### Phase budgets, controls and transient updates (2b.2 / 2b.3)

- Default budgets: initialize 30 s, session creation 30 s, prompt 300 s,
  permission 120 s, cancellation grace 2 s. Explicit test/development overrides
  must be greater than zero and at most 24 h per phase. Prompt and permission
  limits are absolute deadlines; incoming activity does not reset them.
- Startup cancellation/timeout drops the connection. Prompt timeout sends cancel
  and gives the Agent only the grace budget; a late successful answer cannot turn
  timeout into success. Forced abort also ends a callback that catches the error.
  Permission timeout cancels its response and the turn; teardown follows the
  same grace budget. These are cooperative async limits, not a hard OS kill SLO.
- Each control is claimable by one connection. An OS-random 128-bit connection ID
  prevents stale responses matching a recreated connection even when the provider
  reuses its opaque session ID. Every turn control matches connection/run/role/
  session/turn; permission additionally matches a still-pending monotonic request
  ID and an offered option. Late, duplicate, unknown, cancelled and expired choices
  fail without consuming a valid pending request.
- One permission may be pending per participant; at most 64 requests per turn,
  16 unique options per request, IDs up to 256 bytes and labels up to 4 KiB.
  User-selected allow-once/reject-once and explicit cancellation are supported.
  Remembered allow/reject options remain visible but disabled. No automatic choice.
  Concurrent permission requests and terminal responses before an unresolved,
  non-cancelled decision fail closed. The SDK dispatch callback never awaits UI;
  its bounded background task owns the responder. Turn/scope drop retires pending
  controls and wakes waiting tasks; closed transports receive no new approvals.
- Session info records the pinned development package, advertised authentication
  method IDs, session-established/authentication-required state, load/resume
  advertisement and mode/model IDs/current selections. Auth-required errors are
  observed at response dispatch because the SDK may abort before an awaiting
  request resumes. Readiness is not a credential inspection. Unsupported controls
  are explicit false values even when the Agent advertises support.
- `AcpV1Update` is a private, transient native stream for the future 3b adapter:
  version 1, connection/run/role, monotonic sequence and a closed tagged kind.
  Kinds cover session info, turn status, text, plan, tool status, pending permission
  and resolution. No stream event commits research data or completes the All Chat
  run. Tool arguments/results/locations, metadata and hidden reasoning are omitted.
  Unsupported update kinds return `CapabilityUnavailable`.
- The queue is bounded to 1,024 updates and 8 MiB of serialized pending data;
  overflow fails the session instead of silently dropping permission/history.
  `next_update` wakes on new data or scope closure, including startup validation
  rejection and pre-cancellation. All mutex sections are short;
  none spans an await. Plan entries are bounded to 64, capability ID lists to 64,
  and normalized inbound update JSON to 68 KiB in addition to the text limits.
- Rust-generated `crates/qiongli-execution/tests/fixtures/acp-control-stream-v1.json`
  contains representative controls and updates for 3b. It is not a Tauri command,
  public App schema, TypeScript decoder or persistent chat format. Schema 19 and
  the existing committed All Chat snapshot remain unchanged.

## 4. Validation & Error Matrix

### Q04 stable ACP read view (runtime opt-in)

- `with_read_view` accepts one to three immutable approved text snapshots, each
  nonempty and at most 64 KiB. Keys are exact UTF-8 virtual absolute paths below
  `/qiongli-context/`, at most 4 KiB; empty/dot/parent components, backslashes,
  control characters and duplicate keys fail before connection and close an
  attached unclaimed control. No disk reads,
  path canonicalization, directory listing or fallback to project files exists.
- The caller must attach the existing `AcpV1Control`. Only this opt-in advertises
  stable `fs.readTextFile`; file writes and terminal capabilities remain false.
  Every other inbound request, including permission requests and unknown methods,
  retires this mode with `ResponseInvalid`. Existing clients without a read view
  retain their permission behavior. This adds no MCP extension or dependency.
- Raw read requests are capped at 8 KiB of serialized params and decoded strictly
  before the SDK's permissive `DefaultOnError` fields. Unknown fields or invalid
  range types fail; absent/null range fields mean unspecified. A supplied line
  is one-based, and limit is positive; a start beyond actual content is rejected.
  Limit is a maximum, so a valid read truncates at EOF as ACP specifies.
  Returned slices preserve their original newline bytes, including CRLF.
- Ingress requires the owned session and open prompt window. The existing turn
  owner additionally rejects idle, cancelled, stopped, expired and dropped turns.
  Each turn permits at most 16 reads and 256 KiB returned text; counters reset
  with that owner's next turn. A rejection latches failure and aborts the runner,
  so an Agent that ignores an RPC error cannot publish a successful outcome.
- ACP v1 read requests have no turn ID, so next-window causal attribution has
  the same limitation as session updates. The caller owns source authorization
  and revalidation before each turn. This immutable view does not authorize
  stale candidate submission or replace existing source/version checks.
- The App now opts in only through research manifest v2 and explicit
  selected-excerpt consent. Historical v1 manifests never acquire read authority.
  Its debug fixture uses `for_development_read_responses` to read and compare the
  three approved snapshots through the same ACP boundary on every turn, emitting
  bounded tool activity before a deterministic candidate. Regular development
  responses without a read view keep their prior behavior.
- Client capabilities are not provider OS isolation. SDK 2.1.0's subprocess
  launcher inherits environment and startup cwd; `session/new.cwd` is protocol
  context. Native file/Shell/network tools, hooks, MCP/Skills configuration,
  authentication and packaged execution still require independent qualification.

### Error mapping

- relative, empty, NUL-containing, or over-4-KiB encoded cwd; empty or
  over-256-KiB prompt; exhausted local turn ID; invalid timeout/control binding,
  unknown/stale/duplicate/expired permission choice -> `InvalidRequest`;
- authentication required -> `AuthenticationUnavailable`;
- non-v1 negotiation, missing ACP method, unsupported lifecycle update, unknown
  stop reason, permission without controls, disabled remembered choice, or Windows
  development launcher -> `CapabilityUnavailable`;
- malformed, mismatched, empty, control-containing, or over-256-byte session ID;
  malformed/non-notification or idle-between-turn update; empty, non-text, or over-64-KiB text
  chunk; more than 1,024 updates including the terminal update; more than
  8 MiB accepted content; unsolicited cancellation; or any non-cancelled stop
  after cancellation -> `ResponseInvalid`;
- ACP parse, invalid-request, or invalid-params error -> `ResponseInvalid`;
- Agent refusal or missing provider resource -> `ProviderRejected`;
- missing `npx`, spawn/transport failure, peer shutdown, ACP request cancellation,
  internal ACP error, unknown ACP error code, or future unmapped ACP error ->
  `TransportUnavailable`; phase timeout or missing cancel acknowledgement uses the
  same code (transient turn status distinguishes `timed_out` where applicable);
- reuse after a failed or abandoned prompt -> `TransportUnavailable`, unless a
  latched notification or permission fault supplies its more specific error;
- cancellation observed before a request phase -> `Cancelled`; after prompting,
  a cancelled event requires Qiongli to have sent `session/cancel` and the Agent
  to confirm it.
- stale All Chat generation -> `AllChatStateError::StaleGeneration`; malformed
  outcome protocol/event shape, turn ID other than 1, or empty/oversized aggregate message ->
  `AllChatStateError::InvalidEvent`; staged event exhaustion ->
  `AllChatStateError::LimitExceeded`. Every projection error leaves the original
  All Chat state byte-for-byte unchanged.

## 5. Good / Base / Bad Cases

- Good: an explicitly selected fixed development preset negotiates v1, creates
  a session, emits bounded text events, and ends with a completed event.
- Base: a pre-cancelled token stops before Agent work and returns `Cancelled`.
- Base projection: a confirmed first-turn cancellation records session-ready and
  turn-cancelled, discards every partial content delta, and keeps the run active.
- Bad: an Agent asks for permission and then emits text, reports unsolicited
  cancellation, sends another session's update, or emits an unsupported update;
  the turn fails without returning accumulated output.
- Bad projection: a stale writer, invalid message, or staged event-limit failure
  mutates none of the original All Chat state.

## 6. Tests Required

Run:

```bash
cargo fmt --manifest-path packages/qiongli-native/Cargo.toml --all -- --check
cargo clippy --manifest-path packages/qiongli-native/Cargo.toml \
  -p qiongli-execution --all-targets --locked -- -D warnings
cargo test --manifest-path packages/qiongli-native/Cargo.toml \
  -p qiongli-execution --locked
```

Keep one credential-free in-process fixture on the production `run_turn` path.
It must assert exact v1 negotiation, exact fixed preset argv with an empty
configured environment-override map, normalized text/completion,
permission-output suppression, unsolicited-cancel rejection, and
cancellation/terminal consistency. It must not invoke `npx`, the network,
Codex, or Claude.
The connection regression must preserve an owned update immediately after
session creation, reject an unknown update at that same boundary, and reject
unknown updates after valid content both with and without a later `EndTurn`.
Bound the no-terminal test with a watchdog so loss of error wakeup fails CI
instead of hanging it; the watchdog is a test bound, not a product latency SLO.
Reuse that fixture through `with_session` to assert one initialization/session,
two distinct prompts, turn IDs 1/2 and isolated results. Cover preflight rejection
without consuming IDs, ID exhaustion, failed/abandoned-turn retirement, unchanged
earlier outcomes and idle-update rejection without waiting for another prompt.
Reject second-turn use of the first-coordinator projection atomically.

Keep deterministic silent-initialize/session/prompt tests, cancelled-without-output
and permission-pending cancel/timeout/exit cases. Assert once-only allow/deny,
wrong connection/run/role/session/turn/request rejection, bounded queues and no
hidden/raw-content exposure. Generate and compare the control/stream fixture via
the Rust test (set `QIONGLI_UPDATE_ACP_FIXTURE=1` only to regenerate deliberately).
The Unix process regression starts its own local shell/sleep fixture and checks
that launcher and descendant exit within 3 s after the 300 ms startup timeout.
It requires a read-only `ps` probe; it never launches a provider or uses the network.
Other cancellation/phase fixtures have a 2 s observation bound, not a product SLO.

Keep one focused projection test proving atomic success and rollback, bounded
delta aggregation, cancellation without partial text, and that ACP stop/length
events leave the All Chat run active rather than appending `RunCompleted`.

The three `read_view` tests exercise actual ACP request/response dispatch across
two turns, byte-preserving ranges, malformed parameters, exact-path aliases,
unknown sessions/methods, write/terminal/permission denial, count/byte limits,
startup/idle/cancel windows, scope drop and deadline expiry. They require no
provider, credentials, network or source files. A 3 s watchdog bounds the idle
and cancellation dispatch fixtures; it is not a provider teardown claim.

## 7. Wrong vs Correct

Wrong: expose `AcpAgentConfig`, accept an arbitrary command string, auto-select a
permission option, or treat every ACP `RequestCancelled` error as user intent.

Correct: expose only the two fixed development presets, fail permission closed,
and derive user cancellation only from Qiongli's request plus the Agent's
matching terminal acknowledgement.

Wrong: translate ACP `EndTurn` or `MaxTokens` directly into All Chat
`RunCompleted`, or mutate the live state one projected event at a time.

Correct: stage the complete first-coordinator projection on a clone, derive
sequence from that state, and replace the live state only after every append
succeeds.
