# Implementation Plan

## 1. Close the current App slice

- [x] Review the existing dirty diff against R1 and keep provider/scroll changes
      separate from project-guidance changes in commit history.
- [x] Run App API tests, Desktop tests/check, and the affected native/project
      tests; fix only failures caused by this slice.
- [x] Commit and review the App slice before starting Zotero work.

Focused commands:

```bash
pnpm --dir packages/qiongli-app-api test
pnpm --dir packages/qiongli-desktop test
pnpm --dir packages/qiongli-desktop check
cargo test --manifest-path packages/qiongli-native/Cargo.toml -p qiongli-project local_guidance
cargo test --manifest-path packages/qiongli-native/Cargo.toml -p qiongli --lib app_api
```

## 2. Close the native Zotero slice

- [x] Start from the merged App-slice `2.x` head and update `task.json.branch`.
- [x] Add bounded Companion search/upsert calls to the native client.
- [x] Add registry, schema, dispatch, Skill, and docs parity in the same slice.
- [x] Test status/version rejection, search, dry-run/apply, replay/changed-plan
      rejection, timeout, oversized/malformed response, and fallback.

Focused commands:

```bash
cargo test --manifest-path packages/qiongli-native/Cargo.toml -p qiongli-runtime zotero
cargo test --manifest-path packages/qiongli-native/Cargo.toml -p qiongli --test mcp_stdio zotero
python3 scripts/validate_capability_contract.py
```

## 3. Freeze truthful source contracts

- [x] Search generated/current Alpha 3 notes for nonexistent commands and stale
      Full MCP/Zotero claims.
- [x] Run only the release/contract checks affected by the corrected sources.
- [x] Freeze one clean commit and stop feature changes.

## 4. Qualify one exact candidate

- [x] Require Native CI success for the frozen exact head.
- [x] Run the existing packaged vertical only because App API, embedded content,
      integration, and Zotero inputs changed.
- [x] Record the source SHA, run IDs, package digests, and
      `publication_allowed=false` in the Alpha ledger.
- [x] Hold A8 until A6-A7 public-claim evidence is separately accepted.

Accepted internal receipt: source `cced60826ac4d7dad596669103a7e15b61868e81`,
Native CI `31438158969`, promotion `31439930097`, and candidate-set SHA-256
`47ef8d95449472bb6f01ed91d90364f729270bec29dd8961a481d37f757cc182`.
Publication approval was rejected. The native CLI size overage and all manual
A6-A9 release claims remain outside this completed internal-use task.

Packaged command:

```bash
pnpm desktop:macos:acceptance -- --diagnostics
```
