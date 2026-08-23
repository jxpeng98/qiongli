# Implementation plan: PLT-321 App Full MCP journey

## 1. Activate the Slice

- [x] Create a stacked task branch from the accepted PLT-320 source while PR
      #140 remains open.
- [x] Set `PLT-321` to `active` and regenerate the current program index.
- [x] Run the roadmap generator check and unit tests.

## 2. Make the existing self-test prove Full MCP

- [x] Add a failing focused test that expects the combined authoritative Full
      registry and a successful Full-only orchestration route.
- [x] Reuse one `FullMcpServer` constructor from both stdio and desktop paths.
- [x] Replace the Lite-only self-test server/check with the Full server, derive
      the tool count from existing constants, and retain cancel, timeout, and
      credential-free behavior.
- [x] Rename misleading Lite-only internal intent/test labels to Full.

Focused Rust checks:

```bash
cargo test --manifest-path packages/qiongli-native/Cargo.toml -p qiongli \
  desktop::tests::full_mcp_self_test_uses_exact_registry_and_full_only_dispatch -- --exact
cargo test --manifest-path packages/qiongli-native/Cargo.toml -p qiongli \
  desktop::tests::full_mcp_self_test_does_not_resolve_provider_credentials -- --exact
cargo test --manifest-path packages/qiongli-native/Cargo.toml -p qiongli \
  desktop::tests::full_mcp_self_test_supports_cancel_and_fixed_timeout -- --exact
cargo test --manifest-path packages/qiongli-native/Cargo.toml -p qiongli \
  --test mcp_stdio copied_full_binary_routes_to_host_orchestration_without_lite_upgrade -- --exact
```

## 3. Expose the typed App workflow

- [x] Add strict run/poll/cancel Full self-test intents and one typed event/view
      to the Rust App API and TypeScript schema.
- [x] Update the deterministic App API fixture and cross-language contract
      tests; make any required schema-version bump once.
- [x] Store the latest view in App state and poll only while state is `running`.
- [x] Add one compact Full MCP panel to Client Integrations showing profile,
      combined tool count, check statuses, and explicit retry/cancel behavior.
- [x] Keep integration status authoritative and visibly separate from embedded
      Full MCP health.

Focused App checks:

```bash
pnpm --dir packages/qiongli-app-api test
pnpm --dir packages/qiongli-desktop test
pnpm --dir packages/qiongli-desktop check
cargo test --manifest-path packages/qiongli-native/Cargo.toml -p qiongli \
  desktop_api::tests --lib
```

## 4. Regress setup, status, and recovery

- [x] Run the existing setup selection, unsupported-client, fresh Host probe,
      drift/reconciliation, fixed Host plan, and partial-failure tests.
- [x] Add only the smallest missing cross-layer assertion proving a stale/error
      integration remains non-Ready while the Full self-test result is reported.
- [x] Fix production code only at a shared native or projection owner if these
      checks expose a defect.

## 5. Run the Slice gate

- [x] Run formatting, affected Rust check/Clippy/tests, App API/Desktop
      check/tests/build, roadmap checks, and `git diff --check`.
- [x] Commit the product Slice, push the task branch, and open a PR targeting
      `2.x` (stacked on #140 until its dependency merges).
- [x] Require exact-head Evaluation Truth and Native CI; do not dispatch package
      or promotion jobs.

## 6. Record evidence and close PLT-321

- [x] Add one path-redacted acceptance note with exact product commit, run,
      combined registry result, Full-only result, and explicit nonclaims.
- [x] Set `PLT-321` to `accepted`, regenerate the current index, and rerun the
      roadmap checks.
- [x] Commit and push evidence closeout, run the final Trellis Slice check, and
      leave merge/release decisions to the user.

## Rollback points

- Before product change: restore only the PLT-321 ledger row and generated
  index.
- After product change: revert the Full self-test App bridge/UI; existing setup,
  repair, Lite/Full stdio, and user state remain untouched.
