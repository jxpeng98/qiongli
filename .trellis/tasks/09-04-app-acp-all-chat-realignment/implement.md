# ACP and All Chat State implementation plan

## Stage 0 -- Rebaseline direction and close stale planning state

- [x] Add ADR 0217 and register it without editing accepted ADR 0211.
- [x] Update the master roadmap horizon, product boundary, platform slice, and
      `PILOT-702` wording.
- [x] Keep `PLT-401`--`PLT-403` accepted, set `PLT-404` active, and regenerate
      the current program index.
- [x] Update Product Control priority text to match the master roadmap.
- [x] Validate ADR and program-ledger truth.

## Stage 1 -- Implement the All Chat State core

- [x] Add the smallest provider-neutral `AllChatStateV1` module to
      `qiongli-execution` using existing run, backend, project, role, task, and
      digest types where they already fit.
- [x] Enforce one coordinator, at most two collaborators, coordinator-only
      delegation, exact sequence/generation, assigned-result, and terminal-state
      invariants.
- [x] Add focused unit tests for a valid coordinator/two-worker flow and one
      table of invalid transitions.
- [x] Export the contract from the crate without wiring it to the retired direct
      OpenAI path.

## Stage 2 -- Prove ACP v1 transport

- [ ] Pin the official Rust ACP SDK and implement a narrow client adapter around
      initialize, session/new or load, prompt/update, permission, and cancel.
      Partial: the stable v1 single-turn boundary now covers initialize,
      session/new, text updates, fail-closed permission, and update-boundary
      cancellation; broader session lifecycle support remains deferred.
- [x] Add a deterministic in-process or test-process ACP Agent fixture so CI
      needs no credential, network, Node.js, Codex, or Claude installation.
- [ ] Add an explicit development smoke path for the pinned Codex and Claude
      adapters; fail clearly when its opt-in runtime prerequisite is absent.
      Partial: callers can explicitly construct either fixed, exact-version
      adapter through direct `npx` argv without an explicitly configured shell
      or caller-supplied command, argv, or environment overrides; live readiness
      evidence and packaged sidecars remain open.
- [ ] Map ACP lifecycle data into All Chat events without exposing SDK types to
      project state.

## Stage 3 -- Add the App vertical

- [ ] Define one Rust-owned versioned All Chat Tauri contract, generated schema,
      golden fixtures, and compatibility record.
- [ ] Add the TypeScript/Zod consumer and dev transport fixture.
- [ ] Extend the Orchestrator page with Agent selection, prompt, unified labelled
      timeline, cancellation, permission, failure, and resume states.
- [ ] Keep the existing App schema 19 and host-driven run path intact.

## Stage 4 -- Exercise bounded multi-Agent coordination

- [ ] Let the coordinator produce at most two validated assignments.
- [ ] Run worker/reviewer sessions independently and return bounded task results.
- [ ] Feed the structured result projection to the coordinator for synthesis.
- [ ] Prove cancellation, adapter loss, restart recovery, duplicate/out-of-order
      update rejection, and stale project-write rejection.
- [ ] Record separate macOS and Windows packaged acceptance only after sidecars
      are self-contained; do not make this a daily-development gate.

## Validation

Run the smallest applicable checks after each stage:

```bash
python3 tooling/scripts/update_program_roadmap.py --check
python3 scripts/validate_arc_201_adrs.py
python3 -m unittest tests.test_program_roadmap tests.test_arc_201_adrs -v
cargo fmt --manifest-path packages/qiongli-native/Cargo.toml --all -- --check
cargo test --manifest-path packages/qiongli-native/Cargo.toml \
  -p qiongli-execution --locked
```

Stages 2 and 3 add their closest ACP and App contract tests. Cross-platform
Build and packaged Acceptance remain separately triggered lanes.

## Risk and rollback points

- Stop after Stage 1 if the All Chat model duplicates the existing orchestration
  state instead of projecting it.
- Stop after the deterministic Stage 2 fixture if the SDK cannot support ACP v1
  on the workspace Rust toolchain without a broad runtime migration.
- Do not call development `npx` evidence a packaged-product result.
- Revert the narrow stage at fault; none of Stages 0--3 may mutate existing
  project data or historical host-driven checkpoints.

## First implementation increment

This authorized run completes Stage 0 and Stage 1 first. It may begin Stage 2
only if dependency resolution is small and the focused native checks remain
green. App UI, live credentials, adapter downloads, packaged sidecars, release,
and publication are not part of the first increment.
