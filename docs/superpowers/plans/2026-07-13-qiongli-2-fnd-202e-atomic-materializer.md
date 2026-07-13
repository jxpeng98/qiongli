# Qiongli 2 FND-202E Atomic Materializer Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Materialize an already verified Qiongli resource-pack profile into a temporary or explicitly approved filesystem target without exposing arbitrary write paths to untrusted MCP input.

**Architecture:** `qiongli-content` introduces a target capability that can only be created by the private temporary-target factory or an explicitly named trusted-caller approval boundary. The materializer preflights the selected profile and existing target, rejects traversal, links, reparse points, Unix hard-link drift, insecure Unix ancestor chains, and unmanaged contents, writes a complete private sibling staging tree plus a canonical managed receipt, then commits by renaming the old managed tree to a backup and the staging tree into place. Promotion failure restores the backup; post-promotion cleanup failure is a distinct committed state.

**Tech Stack:** Rust 1.97, Rust standard-library filesystem APIs, `serde`, RFC 8785 canonical JSON, SHA-256, Cargo integration and unit tests.

---

## File Map

| File | Responsibility |
|---|---|
| `packages/qiongli-native/crates/qiongli-content/src/materializer.rs` | Trusted target capability, receipt contract, preflight, staging writes, atomic replacement, rollback, and managed-tree verification |
| `packages/qiongli-native/crates/qiongli-content/src/lib.rs` | Public FND-202E API exports |
| `packages/qiongli-native/crates/qiongli-content/tests/atomic_materializer.rs` | Public behavior, target authorization, profile projection, permissions, link rejection, unmanaged/drift refusal, and replacement tests |
| `packages/qiongli-native/README.md` | Native content lifecycle and FND-202E nonclaims |
| `docs/superpowers/roadmaps/2026-07-13-qiongli-2-accelerated-rust-migration-roadmap.md` | FND-202E completion receipt and next FND-202F slice |

## Task 1: Freeze the Public Safety Contract with Failing Tests

- [x] Add fixtures that build and load a deterministic in-memory pack.
- [x] Require temporary targets to use a private container under the canonical OS temporary directory.
- [x] Require explicit approval for caller-selected absolute targets and reject relative/traversal or insecure Unix-ancestor targets.
- [x] Verify profile materialization, canonical receipt fields, regular/executable logical modes, and no unselected files.
- [x] Reject target and ancestor links/reparse points without writing outside the target.
- [x] Reject unmanaged and drifted targets without overwriting prior bytes; lock ownership is covered by the internal transaction tests.
- [x] Verify a second managed materialization replaces the old tree and leaves no staging or backup residue.
- [x] Run the focused test and confirm RED because the materializer API does not exist.

## Task 2: Implement Target Authorization and Managed Receipt

- [x] Add `MaterializationTarget` with separate temporary and trusted-caller factories.
- [x] Validate absolute normalized paths, portable target leaves, existing directory ancestors, and no links/reparse points.
- [x] Define the versioned receipt and materialized-entry schema using canonical JSON.
- [x] Export only the service API required by later CLI/UI composition; do not add an MCP path-taking command.

## Task 3: Implement Transactional Materialization

- [x] Resolve and freeze the canonical profile before filesystem mutation.
- [x] Acquire a target-specific create-new lock, pin its identity, and fail closed when another writer owns it.
- [x] Validate an existing tree against its managed receipt and refuse unmanaged or drifted contents.
- [x] Keep staging private, create Unix files as `0600`, close and sync them, then apply final logical modes before promotion.
- [x] Commit with sibling renames; restore the prior managed tree if staging promotion fails.
- [x] Clean staging and lock artifacts on handled failures and remove the prior backup after a normal successful promotion.
- [x] Distinguish post-promotion cleanup failure from a transaction that never committed.
- [x] Add an internal fault-injection unit test proving the rollback branch restores prior bytes.

## Task 4: Verify and Record FND-202E

- [x] Run `cargo fmt --all -- --check`.
- [x] Run the focused `qiongli-content` materializer tests.
- [x] Run `cargo check --workspace --all-targets --all-features --locked`.
- [x] Run `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`.
- [x] Run `cargo test --workspace --all-targets --all-features --locked`.
- [x] Update the native README and accelerated roadmap with exact evidence and explicit nonclaims.
- [x] Commit and push the cohesive FND-202E checkpoint to the existing rolling Draft PR #63.

Final cross-platform evidence: GitHub Actions run `29291560721` passed the
native change boundary plus Linux, macOS, and Windows Rust jobs at code head
`870d85b8f0ac5f57311292d06b7278441eb9d3f7`.
