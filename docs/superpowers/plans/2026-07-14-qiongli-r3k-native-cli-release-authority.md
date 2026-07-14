# Qiongli R3K Native CLI And Release Authority Implementation Plan

Status: in progress

Date: July 14, 2026

**Goal:** Embed validated publisher public-key policy at build time and expose
the accepted signed native-payload lifecycle through a guarded current-target
CLI.

**Architecture:** `qiongli-platform` parses the strict public authority policy;
the app build embeds validated bytes or an empty source-build sentinel; the CLI
composes R3J verification, R3I plans, explicit approval, and receipt-backed
lifecycle operations without duplicating transaction logic.

## Files

| Path | Change |
|---|---|
| `packages/qiongli-native/crates/qiongli-platform/src/release_authority.rs` | Strict canonical embedded public trust policy |
| `packages/qiongli-native/apps/qiongli/build.rs` | Optional bounded build-time public policy injection |
| `packages/qiongli-native/apps/qiongli/src/lib.rs` | Embedded authority loader |
| `packages/qiongli-native/apps/qiongli/src/command.rs` | Closed native install grammar and status truth |
| `packages/qiongli-native/apps/qiongli/src/native_cli.rs` | R3J/R3I CLI composition and redacted outputs |
| native Rust tests and `packages/qiongli-native/README.md` | Lifecycle, security, runtime, and boundary evidence |

## Task 1 — Freeze R3K

- [x] Audit R3A, R3I, R3J, the canonical CLI, build script, managed-root
  approval, and current-target archive tests.
- [x] Freeze the authority schema, build-only injection, source-build sentinel,
  command grammar, preview/apply approval binding, outputs, and nonclaims.
- [x] Keep private signing, managed-root discovery/creation, activation,
  desktop apply, publication, and updater behavior outside R3K.
- [ ] Commit the design checkpoint on the rolling branch.

## Task 2 — Add Embedded Public Authority

- [ ] Add bounded strict canonical authority parsing with separate release and
  launch-grant key roles.
- [ ] Enforce sorted unique IDs, Ed25519 key validity, generation floors,
  release-key windows, key-count limits, and fixed errors.
- [ ] Add optional build-time authority-file validation and byte embedding.
- [ ] Keep ordinary builds empty and reject runtime/caller trust overrides.
- [ ] Test canonical success, malformed/oversized input, role confusion,
  duplicate/unsorted keys, invalid windows, and redacted debug/errors.

## Task 3 — Add Native CLI Lifecycle

- [ ] Add exact order-independent preview/apply/verify/remove grammar.
- [ ] Derive the current Lite portable artifact and authority policy rather
  than accepting version/channel/profile/key/generation arguments.
- [ ] Compose bounded release read, R3H target approval, R3J verification,
  approved managed root, R3I plan, and plan re-verification.
- [ ] Require expected semantic digest plus explicit filesystem-write approval
  before apply; require explicit approval before remove.
- [ ] Emit versioned redacted summaries and preserve fixed reason-code errors.
- [ ] Keep receipt-backed verify/remove independent of release source expiry.

## Task 4 — Verify The Vertical

- [ ] Prove source builds report no authority and cannot preview/apply.
- [ ] Use distinct deterministic test keys to exercise authority-backed
  preview/no-mutation, digest rejection, apply/replay, verify, and remove.
- [ ] Reject tampered release/archive, wrong role, stale generation, unsafe
  root, invalid install ID, duplicate/unknown options, and missing approval.
- [ ] Assert output never contains release/archive/root/environment canaries.
- [ ] Preserve the accepted installed-binary empty-`PATH` Lite runtime proof.

## Task 5 — Accept R3K

- [ ] Run format, locked workspace check, strict Clippy, normal native tests,
  platform tests, focused Lite compatibility, Windows MSVC check/Clippy, and
  the frozen 2.x boundary.
- [ ] Commit and push implementation to the single rolling Draft PR #63.
- [ ] Require exact-head Native CI and Cloudflare Pages to pass.
- [ ] Update this receipt, native README, accelerated roadmap, and PR body with
  observed evidence only.

## Required Commands

```bash
cargo fmt --manifest-path packages/qiongli-native/Cargo.toml --all -- --check
cargo check --manifest-path packages/qiongli-native/Cargo.toml --workspace --all-targets --all-features --locked
cargo clippy --manifest-path packages/qiongli-native/Cargo.toml --workspace --all-targets --all-features --locked -- -D warnings
cargo test --manifest-path packages/qiongli-native/Cargo.toml --workspace --all-targets --all-features --locked
cargo test --manifest-path packages/qiongli-native/Cargo.toml -p qiongli-platform --locked
python -m pytest <focused Lite compatibility selection>
cargo check --manifest-path packages/qiongli-native/Cargo.toml --workspace --all-targets --all-features --target x86_64-pc-windows-msvc --locked
cargo clippy --manifest-path packages/qiongli-native/Cargo.toml --workspace --all-targets --all-features --target x86_64-pc-windows-msvc --locked -- -D warnings
./scripts/check_2x_native_change_boundary.sh --base-ref origin/2.x
```

Full legacy Python and Node suites remain outside this accelerated native
batch. The focused Lite selection is taken from the accepted Native CI gate.

## Completion Definition

R3K is complete when an authority-injected current-target product can safely
preview, explicitly approve and apply, verify, and remove one exact signed Lite
payload using shared Rust services; an ordinary source build remains
non-installable; no runtime trust override or private material exists; and all
required exact-head gates pass without claiming activation or Alpha.1 release.
