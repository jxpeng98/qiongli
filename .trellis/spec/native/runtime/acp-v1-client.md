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

AcpV1TurnOutcome::protocol_version(&self) -> u16
AcpV1TurnOutcome::session_id(&self) -> &str
AcpV1TurnOutcome::events(&self) -> &[AgentEventV1]
```

`run_turn` is asynchronous and consumes the single-use client.
`AcpDevelopmentPresetV1` is closed to `Codex` and `Claude`.

## 3. Contracts

- Always send `InitializeRequest` with `ProtocolVersion::V1` and reject any
  negotiated version other than v1.
- The development constructor is explicit and is not packaged support. It uses
  direct argv only: `npx -y @agentclientprotocol/codex-acp@1.9.0` or
  `npx -y @agentclientprotocol/claude-agent-acp@0.74.0`.
- Callers cannot supply a command, argv, shell fragment, environment override,
  registry package, or ACP SDK transport/schema object.
- One client creates one new session and consumes one prompt turn. Session load,
  retained sessions, capability controls, and packaged sidecars remain later
  contracts.
- Only bounded text chunks become `AgentEventV1::ContentDelta`.
  `EndTurn` becomes completed/stop, `MaxTokens` and `MaxTurnRequests` become
  completed/length, and `Cancelled` becomes a cancelled event only after
  Qiongli sent `session/cancel`.
- Permission requests are answered with ACP `Cancelled`. If any permission was
  requested, discard the whole accumulated outcome and return
  `CapabilityUnavailable`; never auto-approve.
- Cancellation uses the existing polling-only token. Check it before protocol
  phases and before and immediately after each awaited update, send
  `session/cancel`, discard later valid text chunks, and accept cancellation
  only after the Agent returns
  `StopReason::Cancelled`.
- Provider session IDs are opaque and bounded. They do not grant project or tool
  authority. No credentials, absolute paths, SDK types, or streamed content are
  written to project state by this boundary.

## 4. Validation & Error Matrix

- relative, empty, NUL-containing, or over-4-KiB encoded cwd; empty or
  over-256-KiB prompt -> `InvalidRequest`;
- authentication required -> `AuthenticationUnavailable`;
- non-v1 negotiation, missing ACP method, unsupported lifecycle update, unknown
  stop reason, or any permission request -> `CapabilityUnavailable`;
- malformed, mismatched, empty, control-containing, or over-256-byte session ID;
  malformed/non-notification update; empty, non-text, or over-64-KiB text
  chunk; more than 1,024 updates including the terminal update; more than
  8 MiB accepted content; unsolicited cancellation; or any non-cancelled stop
  after cancellation -> `ResponseInvalid`;
- ACP parse, invalid-request, or invalid-params error -> `ResponseInvalid`;
- Agent refusal or missing provider resource -> `ProviderRejected`;
- missing `npx`, spawn/transport failure, peer shutdown, ACP request cancellation,
  internal ACP error, unknown ACP error code, or future unmapped ACP error ->
  `TransportUnavailable`;
- cancellation observed before a request phase -> `Cancelled`; after prompting,
  a cancelled event requires Qiongli to have sent `session/cancel` and the Agent
  to confirm it.

## 5. Good / Base / Bad Cases

- Good: an explicitly selected fixed development preset negotiates v1, creates
  a session, emits bounded text events, and ends with a completed event.
- Base: a pre-cancelled token stops before Agent work and returns `Cancelled`.
- Bad: an Agent asks for permission and then emits text, reports unsolicited
  cancellation, sends another session's update, or emits an unsupported update;
  the turn fails without returning accumulated output.

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

## 7. Wrong vs Correct

Wrong: expose `AcpAgentConfig`, accept an arbitrary command string, auto-select a
permission option, or treat every ACP `RequestCancelled` error as user intent.

Correct: expose only the two fixed development presets, fail permission closed,
and derive user cancellation only from Qiongli's request plus the Agent's
matching terminal acknowledgement.
