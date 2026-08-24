# Implementation plan: Codex and Claude compatibility matrix

## 1. Start and baseline the task

- [x] After explicit approval of this plan, run `task.py start` and create a
      stacked task branch from the accepted PLT-321 work without modifying PR
      #141's already-green product identity.
- [x] Load Phase 2 context with `trellis-before-dev` and read the affected native
      runtime/content/product specs before editing.
- [x] Run the current exact MCP profile regressions and the two ignored real
      client tests to preserve a before-change baseline.

Focused baseline commands:

```bash
cargo test --manifest-path packages/qiongli-native/Cargo.toml -p qiongli \
  --test mcp_stdio copied_binary_serves_initialize_list_and_bounded_calls_without_path_runtime -- --exact
cargo test --manifest-path packages/qiongli-native/Cargo.toml -p qiongli \
  --test mcp_stdio copied_full_binary_routes_to_host_orchestration_without_lite_upgrade -- --exact
cargo test --manifest-path packages/qiongli-native/Cargo.toml -p qiongli \
  --test codex_plugin_bundle real_codex_clean_client_installs_enables_caches_and_launches_bundle \
  -- --ignored --exact --nocapture
cargo test --manifest-path packages/qiongli-native/Cargo.toml -p qiongli \
  --test claude_plugin_bundle real_claude_clean_client_discovers_and_installs_both_local_forms \
  -- --ignored --exact --nocapture
```

## 2. Strengthen the Codex compatibility test

- [x] In `apps/qiongli/tests/codex_plugin_bundle.rs`, reuse the existing
      fixture, Plugin lifecycle, cached bundle verifier, and stdio helper.
- [x] Register a Lite MCP entry through the real isolated Codex CLI and assert
      `mcp get/list --json` returns the exact packaged command and `--profile
      lite` arguments.
- [x] Exercise Lite and Full `initialize`, `tools/list`, and the Full route call;
      assert the exact profile-specific registries and response boundary.
- [x] Remove the isolated Lite entry and Plugin through Codex, then verify both
      are absent.
- [x] Extend the emitted redacted JSON evidence with client version, Lite/Full
      tool counts, Plugin/Skill/cache checks, and removal checks.

## 3. Strengthen the Claude Code compatibility test

- [x] In `apps/qiongli/tests/claude_plugin_bundle.rs`, reuse the existing direct
      Skills, marketplace, Plugin details, cache, and removal flow.
- [x] Register a Lite MCP entry through the real isolated Claude CLI and require
      the approved `mcp get/list` health observation.
- [x] Exercise Lite and Full `initialize`, `tools/list`, and the Full route call;
      assert the same exact profile-specific registries used by Codex.
- [x] Remove the isolated Lite entry, uninstall the Plugin, and remove its
      marketplace; verify final absence.
- [x] Extend the emitted redacted JSON evidence without including fixture paths,
      auth state, prompts, or responses.

## 4. Lock the path contract and documentation

- [x] Add or strengthen one focused `client_inventory` regression only if the
      exact user/project path matrix is not already asserted sufficiently.
- [x] Update `docs/alpha/install-2x.md` with the Codex/Claude matrix, `.agents`
      plural rule, Host-cache ownership, Full readiness boundary, Lite
      compatibility scope, later-agent nonclaim, and matching Chinese summary.
- [x] Do not edit legacy 1.x install docs or generated Plugin/Skill trees.
- [x] If tests expose a real shared-owner mismatch, patch the highest shared
      native owner and add one focused regression; otherwise make no production
      runtime change.

## 5. Validate the complete Slice

- [x] Rerun both ignored real-client tests with current supported Codex and
      Claude Code binaries under disposable homes.
- [x] Run the full `mcp_stdio`, `codex_plugin_bundle`, `claude_plugin_bundle`,
      and `client_activation` integration suites.
- [x] Run native formatting, affected Clippy/check/tests, documentation checks,
      path-redaction inspection, and `git diff --check`.

Slice commands:

```bash
cargo fmt --manifest-path packages/qiongli-native/Cargo.toml --all -- --check
cargo clippy --manifest-path packages/qiongli-native/Cargo.toml \
  --workspace --all-targets --all-features --locked -- -D warnings
cargo check --manifest-path packages/qiongli-native/Cargo.toml \
  --workspace --all-targets --all-features --locked
cargo test --manifest-path packages/qiongli-native/Cargo.toml -p qiongli --test mcp_stdio
cargo test --manifest-path packages/qiongli-native/Cargo.toml -p qiongli --test codex_plugin_bundle
cargo test --manifest-path packages/qiongli-native/Cargo.toml -p qiongli --test claude_plugin_bundle
cargo test --manifest-path packages/qiongli-native/Cargo.toml -p qiongli --test client_activation
git diff --check
```

- [x] Commit and push the product Slice, open a stacked PR to the PLT-321 branch
      while #141 remains unmerged (or retarget to `2.x` if #141 has merged), and
      require exact-head Evaluation Truth and Native CI.

## 6. Record bounded evidence and close the task

- [x] Add one path-redacted acceptance note under
      `docs/superpowers/acceptance/` bound to the product commit, Codex/Claude
      versions, focused results, and exact CI runs.
- [x] State that the result does not authorize a package/release and does not
      cover authenticated model sessions or additional agents.
- [x] Run the Trellis full-task Slice check, archive the task, and record the
      session. Leave merge and release decisions to the user.

## Risky files and rollback points

- Host-specific CLI parsing is isolated to the two existing ignored integration
  tests; keep assertions version-aware only at the declared supported versions.
- Do not factor a new test framework unless both existing files cannot express
  the matrix with their current helpers.
- Any production-code edit is conditional on a reproduced mismatch and must be
  reverted independently if the compatibility proof can pass through existing
  owners.
