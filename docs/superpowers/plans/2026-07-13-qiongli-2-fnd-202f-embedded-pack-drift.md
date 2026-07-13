# Qiongli 2 FND-202F Embedded Pack And Drift Closure Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use
> superpowers:executing-plans to implement this plan task-by-task. Steps use
> checkbox (`- [ ]`) syntax for tracking.

**Goal:** Compile the frozen Qiongli 1.19 canonical content into the native
Qiongli application, verify it against a committed expected identity during
every build, and expose profile list/read/materialize services without a source
checkout or external runtime at execution time.

**Architecture:** `qiongli-content` owns a versioned resource-pack lock
contract and an `EmbeddedContent` service over already verified static bytes.
The `qiongli` application build script uses `qiongli-content` as a Rust build
dependency, collects the canonical repository `content/` tree, deterministically
builds the pack from frozen metadata, rejects any mismatch with the committed
lock, and writes only the verified pack plus digest into Cargo `OUT_DIR`. The
application library embeds those outputs with `include_bytes!`/`include_str!`.
A native maintenance example regenerates the lock explicitly when a future
accepted content baseline changes; normal builds never rewrite source files.

**Tech Stack:** Rust 1.97, Cargo build scripts, existing `qiongli-content`
collector/writer/loader/materializer, `serde`, RFC 8785 canonical JSON, SHA-256.

---

## File Map

| File | Responsibility |
|---|---|
| `.gitattributes` | Cross-platform LF checkout identity for canonical content bytes |
| `packages/qiongli-native/crates/qiongli-content/src/pack_lock.rs` | Versioned expected pack identity, canonical serialization, and drift verification |
| `packages/qiongli-native/crates/qiongli-content/src/embedded.rs` | Verified static pack service with profile list/read/materialize APIs |
| `packages/qiongli-native/crates/qiongli-content/src/lib.rs` | Public FND-202F API exports and committed lock bytes |
| `packages/qiongli-native/crates/qiongli-content/resources/qiongli-core.lock.json` | Frozen 1.19 pack metadata and expected digests |
| `packages/qiongli-native/crates/qiongli-content/examples/update_qiongli_core_lock.rs` | Explicit Rust-only lock regeneration tool |
| `packages/qiongli-native/crates/qiongli-content/tests/deterministic_writer.rs` | Lock round-trip and drift-mutation tests |
| `packages/qiongli-native/crates/qiongli-content/tests/embedded_content.rs` | Public embedded profile service contract |
| `packages/qiongli-native/apps/qiongli/build.rs` | Reproducible build-time compilation and fail-closed lock check |
| `packages/qiongli-native/apps/qiongli/src/lib.rs` | Product-owned embedded bytes and service composition |
| `packages/qiongli-native/apps/qiongli/tests/embedded_pack.rs` | Real application embedding and frozen identity tests |
| `packages/qiongli-native/README.md` | Build/runtime lifecycle, update command, and nonclaims |
| `docs/superpowers/roadmaps/2026-07-13-qiongli-2-accelerated-rust-migration-roadmap.md` | FND-202F receipt and next R1 slice |

## Task 1: Freeze Lock And Embedded Service Contracts

- [x] Add a lock round-trip test that binds metadata, entry count, content root,
      and whole-pack SHA-256.
- [x] Add a same-size content mutation test that the lock rejects as source
      drift.
- [x] Add service tests for canonical profile listing, profile-scoped resource
      reads, unknown-profile rejection, and materialization through an approved
      target capability.
- [x] Add an application integration test that requires real embedded bytes and
      the generated expected digest.
- [x] Run the focused tests and record RED because the FND-202F API and build
      integration do not yet exist.

## Task 2: Implement The Native Pack Lock And Service

- [x] Add strict versioned lock parsing, validation, canonical serialization,
      construction from a built pack, and typed mismatch errors.
- [x] Keep the frozen build metadata inside the committed lock; do not derive
      identity from wall-clock time, filesystem metadata, current build path,
      or an ambient Git command.
- [x] Add `EmbeddedContent` over verified `'static` bytes with pack/profile
      inspection, profile-scoped reads, and delegation to the atomic
      materializer.
- [x] Export the lock contract, embedded service, and committed lock source from
      `qiongli-content`.

## Task 3: Embed The Verified Pack In The Product

- [x] Add the Rust-only lock regeneration example and generate the initial
      canonical lock for the 418-entry tree whose academic content version is
      `v1.19.0-beta.1` and whose exact canonical source commit is
      `ff2c4f35cd1ee5df78a04ff90a0325273917eed8`.
- [x] Add `qiongli` normal and build dependencies on `qiongli-content`.
- [x] Build the deterministic pack in `build.rs`, compare it to the lock, fail
      closed on drift, and write only verified outputs under `OUT_DIR`.
- [x] Embed the verified pack and digest in the application library and expose a
      thin `embedded_content()` constructor.
- [x] Confirm the real embedded application test passes without reading the
      repository source tree at runtime.

## Task 4: Verify And Record FND-202F

- [x] Run `cargo fmt --all -- --check`.
- [x] Run the focused lock, embedded-service, and application embedding tests.
- [x] Run `cargo check --workspace --all-targets --all-features --locked`.
- [x] Run `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`.
- [x] Run `cargo test --workspace --all-targets --all-features --locked`.
- [x] Review build-script path handling, drift failure semantics, public API
      scope, and the claim that runtime needs no source checkout.
- [ ] Update the native README, accelerated roadmap, and rolling Draft PR #63
      capability ledger with exact evidence and explicit nonclaims.
- [ ] Commit and push the cohesive FND-202F checkpoint to the existing rolling
      branch and obtain exact-head Linux, macOS, and Windows CI evidence.
