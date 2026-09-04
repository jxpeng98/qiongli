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
message, and run completed/failed/cancelled.

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
- The current contract is an in-memory projection. Do not persist or accept a
  serialized full state until a canonical decoder and recovery store are added.

## 4. Validation & Error Matrix

- invalid run/project/revision/profile binding -> `all-chat-state-invalid`;
- malformed text, digest, task, role, or session -> `all-chat-event-invalid`;
- mismatched expected generation -> `all-chat-generation-stale`;
- skipped or duplicate sequence -> `all-chat-sequence-unexpected`;
- non-coordinator delegation, duplicate assignment, pending completion, or
  post-terminal append -> `all-chat-transition-invalid`;
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

## 7. Wrong vs Correct

Wrong: add worker execution, project writes, provider credentials, hidden
reasoning, or full candidate text to `AllChatStateV1`.

Correct: append a bounded provider-neutral event that references the existing
orchestration, candidate, evidence, and project authorities.
