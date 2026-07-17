# Qiongli R3K Native CLI And Release Authority Implementation Plan

Status: complete — accepted on exact implementation head `d90d4846`

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
- [x] Commit the design checkpoint on the rolling branch.

## Task 2 — Add Embedded Public Authority

- [x] Add bounded strict canonical authority parsing with separate release and
  launch-grant key roles.
- [x] Enforce sorted unique IDs, Ed25519 key validity, generation floors,
  release-key windows, key-count limits, and fixed errors.
- [x] Add optional build-time authority-file validation and byte embedding.
- [x] Keep ordinary builds empty and reject runtime/caller trust overrides.
- [x] Test canonical success, malformed/oversized input, role confusion,
  duplicate/unsorted keys, invalid windows, and redacted debug/errors.

## Task 3 — Add Native CLI Lifecycle

- [x] Add exact order-independent preview/apply/verify/remove grammar.
- [x] Derive the current Lite portable artifact and authority policy rather
  than accepting version/channel/profile/key/generation arguments.
- [x] Compose bounded release read, R3H target approval, R3J verification,
  approved managed root, R3I plan, and plan re-verification.
- [x] Require expected semantic digest plus explicit filesystem-write approval
  before apply; require explicit approval before remove.
- [x] Emit versioned redacted summaries and preserve fixed reason-code errors.
- [x] Keep receipt-backed verify/remove independent of release source expiry.

## Task 4 — Verify The Vertical

- [x] Prove source builds report no authority and cannot preview/apply.
- [x] Use distinct deterministic test keys to exercise authority-backed
  preview/no-mutation, digest rejection, apply/replay, verify, and remove.
- [x] Reject tampered release/archive, wrong role, stale generation, unsafe
  root, invalid install ID, duplicate/unknown options, and missing approval.
- [x] Assert output never contains release/archive/root/environment canaries.
- [x] Preserve the accepted installed-binary empty-`PATH` Lite runtime proof.

## Task 5 — Accept R3K

- [x] Run format, locked workspace check, strict Clippy, normal native tests,
  platform tests, focused Lite compatibility, Windows MSVC check/Clippy, and
  the frozen 2.x boundary.
- [x] Commit and push implementation to the single rolling Draft PR #63.
- [x] Require exact-head Native CI and Cloudflare Pages to pass.
- [x] Update this receipt, native README, accelerated roadmap, and PR body with
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

## Acceptance Record

R3K is accepted at design checkpoint `e6619560` and implementation head
`d90d4846`.

Local acceptance passed:

- native and Lite Rust formatting checks;
- locked all-target, all-feature native workspace check and strict Clippy;
- 232 passing native Rust tests, with the two real external-client tests
  remaining explicitly ignored;
- focused post-hardening platform, app-library, and CLI tests (60 + 8 + 16);
- all 69 focused Lite compatibility tests;
- Windows MSVC all-target, all-feature check and strict Clippy;
- the committed native 2.x frozen-boundary check; and
- an isolated authority-injected build whose `install status` ran with an
  empty environment and `PATH` and reported `release_authority: embedded`.

The isolated build gate first rejected a newline-terminated noncanonical
authority fixture, then accepted the byte-canonical fixture containing only
RFC test public keys. No private key material entered the source tree or build.

Exact-head Native CI run `29369405002` passed `d90d4846`: frozen boundary in
5s, focused Lite in 38s, macOS in 7m55s, Linux in 8m16s, and Windows in 8m26s.
Cloudflare Pages passed on the same head.

R3K supplies the reviewed production-grade public-policy injection mechanism;
it does not select a production public key or handle a production private key.
Managed-root creation/discovery, release signing and download, repair, client
activation, desktop apply, packaged-window startup, Marketplace publication,
updater behavior, OS signing/notarization, checksum/SBOM/provenance output,
cross-target artifacts, clean-machine release acceptance, and Alpha.1
publication remain outside this accepted batch.
