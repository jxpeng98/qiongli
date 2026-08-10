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

- [ ] Start from the merged App-slice `2.x` head and update `task.json.branch`.
- [ ] Add bounded Companion search/upsert calls to the native client.
- [ ] Add registry, schema, dispatch, Skill, and docs parity in the same slice.
- [ ] Test status/version rejection, search, dry-run/apply, replay/changed-plan
      rejection, timeout, oversized/malformed response, and fallback.

Focused commands:

```bash
cargo test --manifest-path packages/qiongli-native/Cargo.toml -p qiongli-runtime zotero
cargo test --manifest-path packages/qiongli-native/Cargo.toml -p qiongli --test mcp_stdio zotero
python3 scripts/validate_capability_contract.py
```

## 3. Freeze truthful source contracts

- [ ] Search generated/current Alpha 3 notes for nonexistent commands and stale
      Full MCP/Zotero claims.
- [ ] Run only the release/contract checks affected by the corrected sources.
- [ ] Freeze one clean commit and stop feature changes.

## 4. Qualify one exact candidate

- [ ] Require Native CI success for the frozen exact head.
- [ ] Run the existing packaged vertical only because App API, embedded content,
      integration, and Zotero inputs changed.
- [ ] Record the source SHA, run IDs, package digests, and
      `publication_allowed=false` in the Alpha ledger.
- [ ] Hold A8 until A6-A7 public-claim evidence is separately accepted.

Packaged command:

```bash
pnpm desktop:macos:acceptance -- --diagnostics
```
