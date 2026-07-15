# Qiongli R3M Lite Alpha.1 Release Candidate Execution Plan

Status: active; Tasks 1-5 complete, exact-head release gates pending

Date: July 15, 2026

**Goal:** Assemble, install, and accept one current-target dependency-free Lite
Alpha.1 release candidate without placing signing private keys in the product,
repository, logs, or candidate.

**Architecture:** `qiongli-platform` verifies one release-signed candidate and
returns target-specific portable and PluginBundle capabilities; the canonical
application composes the accepted payload, plugin, activation, and desktop
services; release automation records exact artifacts and evidence but cannot
publish without explicit maintainer authority.

## Files

| Path | Change |
|---|---|
| `packages/qiongli-native/crates/qiongli-platform/src/release_candidate.rs` | Strict candidate construction, signing preimage, and verification |
| `packages/qiongli-native/crates/qiongli-platform/src/native_release.rs` | Internal reuse of the accepted release-key verifier |
| `packages/qiongli-native/apps/qiongli/src/` | Candidate-backed install, diagnose, remove, and UI composition |
| native integration tests | Candidate tamper, target, lifecycle, and isolated-runtime evidence |
| `.github/workflows/` and release tooling | Current-target candidate build and non-publishing acceptance |
| native README, roadmap, and release notes | Exact claims, limitations, and observed receipts |

## Task 1 — Freeze R3M

- [x] Audit R3H-R3L, current release dry-run automation, client bundle
  composers, activation coordinator, and ignored real-client gates.
- [x] Freeze a three-file candidate set and one signed canonical candidate
  descriptor.
- [x] Keep private keys behind an external signing boundary and require an
  explicit final publication decision.
- [x] Commit the design checkpoint on the rolling branch.

## Task 2 — Add The Signed Candidate

- [x] Add bounded canonical candidate, client target, status, notes descriptor,
  detached signature, and fixed errors.
- [x] Require the nested portable release plus exact Codex and Claude Code
  PluginBundle grants.
- [x] Enforce generation, time, channel, target, source, notes, binary, pack,
  mode, and target-scope closure.
- [x] Return a private verified token containing only the requested target's
  accepted capabilities.
- [x] Add canonicalization, tamper, role, replay, swap, omission, redaction, and
  bound tests.

## Task 3 — Materialize Local Integrations

- [x] Compose or replay the R3I native payload from the verified candidate.
- [x] Compose and verify the fixed R3D or R3E plugin source from the installed
  target-native binary.
- [x] Preview and apply the R3L registration using the exact three approvals.
- [x] Verify after every fresh mutation and compensate only fresh committed
  steps in reverse order.
- [x] Preserve client-owned cache, enablement, registry, settings, and trust as
  explicit outstanding host actions.

## Task 4 — Expose The Product Journey

- [x] Add closed candidate preview/apply/verify/remove CLI grammar and versioned
  redacted output.
- [x] Create only fixed owner-private Qiongli roots derived from the current
  user boundary; accept no model/MCP-selected path.
- [x] Connect the same prepared target sessions to the native desktop manager.
- [x] Keep wrong digest, partial approval, stale candidate, drift, and recovery
  states fail closed.
- [x] Make remove receipt-backed and independent of expired release inputs.

## Task 5 — Prove The Candidate

- [x] Build an authority-injected current-target test candidate with distinct
  ephemeral release and launch-grant keys.
- [x] Run CLI, embedded skills, UI startup preflight, and Lite MCP outside the
  checkout with an empty `PATH`.
- [x] Run isolated Codex and Claude Code apply/diagnose/remove and rollback
  journeys without reading normal user state.
- [x] Label the real-client, displayed-window, and production-signing gates as
  `not-run` with explicit external-boundary reasons; execute them only when the
  required host and maintainer authority are provided.
- [x] Assert no runtime dependency on Rust, Python, Node, Cargo, npm, or pip.

Observed local evidence uses the current-target macOS aarch64 portable archive
with product source commit `54f958c388cf54f060a0d322f058176d34935b40`.
The extracted product completed the entire acceptance journey outside the
checkout with an empty runtime `PATH`. The evidence records successful
candidate-file closure, embedded skills, UI preflight, Lite MCP, both isolated
client lifecycles, wrong/partial approval rejection, fresh-step compensation,
and unrelated-state preservation. It also records `publication_allowed: false`;
this is development evidence, not production-signing or public-release
authority. Exact-head Linux evidence is delegated to Native CI.

## Task 6 — Accept Or Block Publication

- [x] Run format, locked workspace check, strict Clippy, complete native tests,
  focused Lite compatibility, Windows check/Clippy, and frozen boundary.
- [ ] Commit and push cohesive R3M checkpoints to rolling Draft PR #63.
- [ ] Require exact-head Native CI and Cloudflare Pages to pass.
- [ ] Bind and review the exact release notes and current-target artifacts.
- [ ] Record any external signing, real-client, UI, or clean-machine blocker;
  do not create a tag or public release without explicit maintainer authority.

Local Task 6 development gates pass: native and Lite formatting, the frozen
2.x boundary, locked all-target/all-feature native check, strict host Clippy,
the complete native workspace test suite, all 69 focused Lite compatibility
tests, and Windows MSVC locked check plus strict Clippy. The external gates and
exact-head remote checks remain pending as listed above.

## Required Development Commands

```bash
cargo fmt --manifest-path packages/qiongli-native/Cargo.toml --all -- --check
cargo check --manifest-path packages/qiongli-native/Cargo.toml --workspace --all-targets --all-features --locked
cargo clippy --manifest-path packages/qiongli-native/Cargo.toml --workspace --all-targets --all-features --locked -- -D warnings
cargo test --manifest-path packages/qiongli-native/Cargo.toml --workspace --all-targets --all-features --locked
cargo test --manifest-path packages/qiongli-lite-mcp/Cargo.toml --locked \
  --test mcp_protocol --test mcp_server --test provider_http \
  --test providers --test search_orchestration \
  --test literature_planning_mcp --test searchplan --test zotero_export \
  --test zotero_companion --test orchestrator_preview
cargo check --manifest-path packages/qiongli-native/Cargo.toml --workspace --all-targets --all-features --target x86_64-pc-windows-msvc --locked
cargo clippy --manifest-path packages/qiongli-native/Cargo.toml --workspace --all-targets --all-features --target x86_64-pc-windows-msvc --locked -- -D warnings
./scripts/check_2x_native_change_boundary.sh --base-ref origin/2.x
```

Full legacy Python and Node suites remain outside this native batch. Release
automation may use build tools, but every advertised installed-product path is
tested with no user language-runtime dependency.

## Completion Definition

R3M is complete only when the exact signed current-target candidate supports
the advertised CLI, UI, Lite MCP, Codex local, and Claude Code local journeys
from isolated extracted bytes; managed apply, diagnose, remove, rollback, and
unrelated-state preservation pass; exact release limitations are bound and
reviewed; and every required external gate is recorded. Completion permits an
explicit Alpha.1 publication decision but does not itself publish.
