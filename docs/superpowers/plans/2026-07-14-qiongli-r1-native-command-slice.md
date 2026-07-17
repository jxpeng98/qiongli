# Qiongli R1 Native Command Slice Implementation Plan

Status: complete

Date: July 14, 2026

**Goal:** Compose the accepted embedded-content and cross-platform config
services into useful, versioned, redacted native CLI commands and close R1.

**Architecture:** Keep `main.rs` as a process adapter. Put grammar, typed
command intent, environment resolution, service composition, JSON output, and
allowlisted errors in the app library. Reuse `qiongli-content` and
`qiongli-config` directly without creating another domain crate.

## Files

| Path | Change |
|---|---|
| `packages/qiongli-native/apps/qiongli/Cargo.toml` | Add config and serialization dependencies |
| `packages/qiongli-native/apps/qiongli/src/lib.rs` | Export the command adapter while retaining embedded content |
| `packages/qiongli-native/apps/qiongli/src/command.rs` | Add grammar, composition, output, home resolution, and redacted errors |
| `packages/qiongli-native/apps/qiongli/src/main.rs` | Reduce to process I/O and exit-code adaptation |
| `packages/qiongli-native/apps/qiongli/tests/cli.rs` | Add binary contracts and no-runtime/path-redaction proof |
| `packages/qiongli-native/crates/qiongli-content/src/materializer.rs` | Add stable path-free materialization reason codes |
| accelerated roadmap and Draft PR #63 | Record only observed current-head evidence |

## Task 1 — Freeze The Public Command Boundary

- [x] Review the R1 roadmap, current parser, embedded-content API, config API,
  and materialization capability boundary.
- [x] Limit config mutation to default profile plus expected revision.
- [x] Freeze JSON, exit-code, environment, privacy, and explicit nonclaim
  contracts.
- [x] Commit the design and this implementation plan.

## Task 2 — Build The Typed Command Adapter

- [x] Add exact nested grammar for help, version, content, config, status, and
  doctor.
- [x] Keep target paths as `OsString`/`PathBuf` and reject non-UTF-8 control
  tokens without lossy conversion.
- [x] Resolve config roots without printing concrete environment values.
- [x] Return complete buffered stdout/stderr plus exit code so failures cannot
  leave partial output.
- [x] Keep process launch, network, provider, MCP, UI, and installer behavior
  out of the adapter.

## Task 3 — Compose Content And Config Services

- [x] Add versioned JSON for content list and approved-target materialization.
- [x] Add a path-free `MaterializationError::reason_code()` surface.
- [x] Add redacted config show and revision-safe default-profile set.
- [x] Preserve all provider settings when changing the default profile.
- [x] Add combined status and read-only doctor checks with explicit blocking
  semantics.
- [x] Keep every operational error on an allowlisted reason code.

## Task 4 — Prove The Public CLI Contract

- [x] Test every accepted and rejected grammar family.
- [x] Test embedded profile listing and explicit materialization from the real
  binary.
- [x] Test missing, ready, stale, and invalid config without exposing the test
  root or private canaries.
- [x] Test doctor success and blocking exit behavior.
- [x] Test copied-binary and empty-`PATH` execution for supported read commands.
- [x] Confirm failed target/config commands preserve prior state.
- [x] Run the same binary contracts on Linux, macOS, and Windows CI.

## Task 5 — Gate And Close R1

- [x] Run the native change boundary.
- [x] Run workspace format, locked check, Clippy with warnings denied, and all
  Rust tests locally.
- [x] Run Windows cross-target workspace check and Clippy.
- [x] Audit production code for process launch, Python/Node, raw secrets, and
  path/argument/environment canary rendering.
- [x] Commit cohesive implementation and test checkpoints.
- [x] Push the same rolling branch and keep Draft PR #63 as the only PR.
- [x] Require exact-head boundary, Linux, macOS, and Windows jobs to pass.
- [x] Update the roadmap and PR only from observed evidence.
- [x] Leave the branch clean, pushed, synchronized, and Draft.

## Required Commands

```bash
./scripts/check_2x_native_change_boundary.sh --base-ref origin/2.x
cargo fmt --manifest-path packages/qiongli-native/Cargo.toml --all -- --check
cargo check --manifest-path packages/qiongli-native/Cargo.toml --workspace --all-targets --all-features --locked
cargo clippy --manifest-path packages/qiongli-native/Cargo.toml --workspace --all-targets --all-features --locked -- -D warnings
cargo test --manifest-path packages/qiongli-native/Cargo.toml --workspace --all-targets --all-features --locked
cargo check --manifest-path packages/qiongli-native/Cargo.toml --workspace --all-targets --all-features --target x86_64-pc-windows-msvc --locked
cargo clippy --manifest-path packages/qiongli-native/Cargo.toml --workspace --all-targets --all-features --target x86_64-pc-windows-msvc --locked -- -D warnings
```

Legacy Python and Node suites remain diagnostic-only and are not part of this
batch.

## Completion Definition

R1 command composition is complete when the copied canonical binary can list
and materialize embedded content, show and revision-safely update redacted
global config, and report status/doctor results with an empty `PATH`; every
public failure is allowlisted and redacted; all exact-head native jobs are
green; and no R2/R3 capability is claimed.

## Execution Receipt

- Design and implementation plan: `45a1cc87`.
- Native command implementation and binary contracts: `f2a6fbe6`.
- Local evidence: native boundary, format, locked workspace check, strict
  Clippy, and all 92 Rust tests passed.
- Cross-target evidence: Windows MSVC workspace check and strict Clippy passed.
- GitHub evidence: Native CI run `29323018180` passed boundary, Linux, macOS,
  and real Windows jobs on exact implementation head
  `f2a6fbe608c234219526c8947f0cb50470ac1482` in 4s, 37s, 40s, and 1m19s.
- Runtime evidence: a copied executable listed and materialized the verified
  embedded pack outside the checkout with an empty `PATH`.
- Scope evidence: no Python or Node suite ran; credentials, providers, MCP,
  project state, migration, agents, orchestration, UI, installation, and
  packaging remain outside R1.
