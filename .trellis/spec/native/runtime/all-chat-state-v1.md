# All Chat State v1

## 1. Scope / Trigger

Use this contract when an ACP adapter, orchestration flow, or future App timeline
creates or appends Qiongli-visible collaboration events. It prevents the shared
timeline from becoming a second scheduler, project store, or provider transcript.

## 2. Signatures

The native owner is `qiongli-execution`:

```rust
AllChatStateV1::try_new(
    run_id: RunId,
    project_id: ProjectId,
    expected_project_revision: u64,
    profile: &OrchestrationProfileV1,
) -> Result<AllChatStateV1, AllChatStateError>

state.append_event(
    expected_generation: u64,
    sequence: u64,
    kind: AllChatEventKindV1,
) -> Result<(), AllChatStateError>
```

`ALL_CHAT_STATE_SCHEMA_VERSION` is `1`. Initial event kinds are run start,
user message, Agent session ready, task delegation, task result, coordinator
message, Agent-turn completed/cancelled, and run completed/failed/cancelled.

## 3. Contracts

- Participants come from the validated `OrchestrationProfileV1`: `Primary` is
  the only coordinator and `Reviewer`/`Verifier` provide at most two collaborators.
- The state references existing run, project, backend, role, and task identities.
  It does not schedule work or grant project mutation authority.
- Generation starts at `0`; the first event sequence is `1`. Every successful
  append advances both exactly once. A rejected append changes nothing.
- Only a ready coordinator may delegate, only to a ready collaborator. A result
  must match that collaborator's unique assignment.
- Completion requires a ready coordinator and no outstanding assignments.
  Completed, failed, and cancelled runs reject every later event.
- Task inputs and results are digest references. Candidate text and evidence stay
  with their existing owners rather than being copied into this projection.
- Event JSON denies unknown fields. Session IDs and visible text are non-empty,
  bounded, and reject control characters.
- Session identity is scoped to the existing run and participant role. Different
  participants may use the same opaque provider session string; each participant
  may initialize only once. No global provider-session registry is implied.
- The first ACP coordinator projection applies session-ready, one aggregated
  completed message, and its stop/length turn event atomically. An ACP turn end
  never implies run completion. A confirmed cancelled turn commits no partial
  message and leaves the run active; duplicate or non-session-adjacent first-turn
  cancellation is invalid.
- The current contract is an in-memory projection. Do not persist or accept a
  serialized full state until a canonical decoder and recovery store are added.

## 4. Validation & Error Matrix

- invalid run/project/revision/profile binding -> `all-chat-state-invalid`;
- malformed text, digest, task, role, or session -> `all-chat-event-invalid`;
- repeated participant initialization with the same or another valid session
  string -> `all-chat-transition-invalid`, with the entire state unchanged;
- mismatched expected generation -> `all-chat-generation-stale`;
- skipped or duplicate sequence -> `all-chat-sequence-unexpected`;
- non-coordinator delegation, duplicate assignment, pending completion, or
  post-terminal append -> `all-chat-transition-invalid`;
- Agent-turn completion without an immediately preceding Primary coordinator
  message, or first-turn cancellation outside an immediately preceding Primary
  session-ready event -> `all-chat-transition-invalid`;
- Agent-turn completion with any finish reason other than stop/length ->
  `all-chat-event-invalid`;
- missing, mismatched, or duplicate task result -> `all-chat-task-unassigned`;
- event, sequence, or generation bound exhausted -> `all-chat-limit-exhausted`.

Unknown JSON fields fail deserialization before state mutation.

## 5. Good / Base / Bad Cases

- Good: one ready coordinator delegates one task to each of two ready
  collaborators, receives both digest-bound results, synthesizes, and completes.
- Base: a solo profile records a run and coordinator messages without delegation.
- Bad: a worker delegates to another worker, returns an unassigned task, appends
  with stale generation, or writes after cancellation; each fails without change.

## 6. Tests Required

Run:

```bash
cargo fmt --manifest-path packages/qiongli-native/Cargo.toml --all -- --check
cargo clippy --manifest-path packages/qiongli-native/Cargo.toml \
  -p qiongli-execution --all-targets --locked -- -D warnings
cargo test --manifest-path packages/qiongli-native/Cargo.toml \
  -p qiongli-execution --locked
```

Keep one valid coordinator/two-collaborator flow and one compact invalid-case
table covering stale generation, sequence gaps, nested delegation, unassigned
results, pending completion, post-terminal events, and unknown JSON fields.
Also prove equal session strings across all three roles, repeated participant
initialization, malformed IDs and absent roles, comparing the complete state
before and after each rejected append.
The ACP projection test must additionally prove clone/apply/replace rollback,
turn-level stop/length semantics, and cancellation without committed partial
text.

## 7. Wrong vs Correct

Wrong: add worker execution, project writes, provider credentials, hidden
reasoning, or full candidate text to `AllChatStateV1`.

Correct: append a bounded provider-neutral event that references the existing
orchestration, candidate, evidence, and project authorities.

Wrong: treat an Agent prompt's stop/length reason as completion of the whole
All Chat run, or commit streamed fragments before the turn reaches a valid
terminal state.

Correct: keep the run active, commit one bounded message only after a completed
turn, and discard partial text when that turn is cancelled.

## App observation persistence (Stage 3a)

The App's public ACP stream can be saved for restart display without committing
partial text into `AllChatStateV1`. `apps/qiongli/src/all_chat_history.rs` owns that
private versioned observation log and rebuilds the App view without execution.
Its [App contract](../../app-api/contract/all-chat-app-v1.md) defines storage,
bounds and recovery. Task/evidence/checkpoint authority remains with existing
owners; their receipt projection is integrated with the Stage 3c tool boundary.
