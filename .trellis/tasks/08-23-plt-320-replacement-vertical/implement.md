# Implementation plan: PLT-320 native replacement vertical

## 1. Close the dependency and activate the slice

- [x] Confirm Native CI run `32651745050` succeeded for exact merge head
      `41accaf0407510c971c596fa174f6f3527e03b30`.
- [x] Set `GOV-320` to `accepted` with archived evidence and exact run identity.
- [x] Set `PLT-320` to `active` and regenerate the program index.
- [x] Run `python3 tooling/scripts/update_program_roadmap.py --check` and
      `python3 -m unittest tests.test_program_roadmap`.

## 2. Establish the focused baseline

- [x] Run the existing Full-profile routing regression.
- [x] Run the existing copied-binary Zotero stdio regression.
- [x] Run fixed official Host plan/order/failure tests.
- [x] Run the ignored isolated real Codex and Claude Code bundle tests against
      disposable homes; stop and record an external prerequisite if a current
      supported client is unavailable.

Focused commands:

```bash
cargo test --manifest-path packages/qiongli-native/Cargo.toml -p qiongli \
  --test mcp_stdio copied_full_binary_routes_to_host_orchestration_without_lite_upgrade -- --exact
cargo test --manifest-path packages/qiongli-native/Cargo.toml -p qiongli \
  --test mcp_stdio copied_binary_routes_zotero_search_preview_and_approved_write -- --exact
cargo test --manifest-path packages/qiongli-native/Cargo.toml -p qiongli \
  desktop::tests::host_plugin_plans_bind_fixed_argv_state_and_target_order -- --exact
cargo test --manifest-path packages/qiongli-native/Cargo.toml -p qiongli \
  --test codex_plugin_bundle real_codex_clean_client_installs_enables_caches_and_launches_bundle \
  -- --ignored --exact
cargo test --manifest-path packages/qiongli-native/Cargo.toml -p qiongli \
  --test claude_plugin_bundle real_claude_clean_client_discovers_and_installs_both_local_forms \
  -- --ignored --exact
```

## 3. Close the Zotero write proof

- [x] Extend the existing copied-binary stdio test to return a dry-run approval
      receipt, call the same public tool with `dry_run=false`,
      `write_intent=apply`, and that receipt, then assert the applied result.
- [x] Assert the exact Connector/Companion request sequence for search, preview,
      and apply.
- [x] Keep existing malformed/changed approval rejection coverage green.
- [x] No production defect was exposed, so no production patch was required.

## 4. Run the affected Slice gate

- [x] Run the focused commands again.
- [x] Run the affected Slice gate below. The target Clippy check passes with the
      Rust 1.98 `chunks_exact_to_as_chunks` baseline lint excluded; the unmodified
      2.x workspace has six pre-existing instances of that lint plus one unused
      test import, while all other listed checks pass.

```bash
cargo fmt --manifest-path packages/qiongli-native/Cargo.toml --all -- --check
cargo clippy --manifest-path packages/qiongli-native/Cargo.toml \
  --workspace --all-targets --all-features --locked -- -D warnings
cargo check --manifest-path packages/qiongli-native/Cargo.toml \
  --workspace --all-targets --all-features --locked
cargo test --manifest-path packages/qiongli-native/Cargo.toml -p qiongli --test mcp_stdio
python3 tooling/scripts/update_program_roadmap.py --check
python3 -m unittest tests.test_program_roadmap
git diff --check
```

- [x] Commit the product Slice, push a task branch, open PR `#140` to `2.x`, and wait
      for exact-head Evaluation Truth and Native CI.

## 5. Record evidence and close program state

- [x] Add one path-redacted acceptance note with exact product commit, run IDs,
      focused results, supported Host versions, and explicit nonclaims.
- [x] Set `PLT-320` to `accepted`, regenerate the current index, and rerun the
      roadmap checks.
- [x] Commit and push the evidence-only closeout; require the final source CI
      checks but do not run package/promotion jobs.
- [x] Run the Trellis full-task Slice check and leave merge/release decisions to
      the user.

## Rollback points

- Before product change: restore only the two ledger rows and generated index.
- After product change: revert the focused stdio change and any shared-owner fix.
- Never clean or modify normal Codex, Claude, or Zotero profiles; all test roots
  are disposable.
