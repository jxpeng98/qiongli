# Implementation Plan

## 1. Record The Approved Architecture Change

- [x] Add ADR 0213 (`ARC-213`) and decision-log index entry; do not edit frozen
      ADR 0206.
- [x] Retain explicit approval, official CLI allowlisting, Host policy/trust,
      direct-cache prohibition, ownership/conflict, separate state, failure,
      remote/cloud, security, and rollback contracts.
- [x] Run the existing ADR/frozen-baseline checks before product edits:

```bash
python3 scripts/validate_arc_201_adrs.py
python3 scripts/check_frozen_2x_architecture_baseline.py
```

## 2. Lock The Fixed-Plan Contract In Native Tests

- [x] Add the smallest table-driven tests for Codex/Claude install and
      receipt-owned repair argv, scope, path resolution, and deterministic order.
- [x] Prove the preview digest binds the Host plan and stale state cannot execute.
- [x] Extend isolated fake-client tests for spawn, timeout, non-zero exit,
      oversized output, decoding/JSON failure, partial batch failure, and no
      shell/cache mutation.

Focused red/green command:

```bash
cargo test --manifest-path packages/qiongli-native/Cargo.toml \
  -p qiongli --lib host_ -- --nocapture
```

## 3. Reuse The Existing Preview And Bounded Runner

- [x] Add one native fixed target/action plan shared by preview display and
      execution; do not add a generic executor.
- [x] Bind the plan into the existing packaged-product pending preview/digest.
- [x] Refine the existing bounded Host helper only enough to return stable
      failure classes and run concrete managed paths without a shell.
- [x] On confirmation, apply the existing receipt-owned packaged transaction,
      run only the bound Host plan, stop on first failure, and retain an explicit
      retry/verify/repair state.

## 4. Make Fresh Probes Own Ready

- [x] Clear previous observations after every attempted Host plan.
- [x] Parse Codex Plugin/MCP JSON and verify exact source plus cache receipt.
- [x] Parse Claude Plugin JSON and component details, including the exact Skill
      and MCP inventory, plus cache receipt.
- [x] Keep command failure and every evidence mismatch non-Ready until an
      explicit later verify passes.
- [x] Update UI copy so command lines are approval detail, not a manual step.
      Avoid a wire-schema change unless the current reason-code surface is
      insufficient.

## 5. Preserve The Existing CLI Vertical

- [x] Run the current CLI install/PATH/fresh-shell tests for exact version,
      missing, shadowed, and mismatched commands.
- [x] Change CLI code only if those checks reproduce a defect.

## 6. Verify At The Smallest Necessary Levels

Focused and contract checks:

```bash
cargo fmt --manifest-path packages/qiongli-native/Cargo.toml --all -- --check
cargo test --manifest-path packages/qiongli-native/Cargo.toml \
  -p qiongli --lib --locked
pnpm --dir packages/qiongli-app-api check
pnpm --dir packages/qiongli-app-api test
pnpm --dir packages/qiongli-desktop check
pnpm --dir packages/qiongli-desktop test
pnpm --dir packages/qiongli-desktop build
```

- [x] Run the existing ignored Codex and Claude clean-client Plugin tests with
      absolute installed client binaries and isolated temporary homes. Extend
      their App-confirmation path instead of creating another harness.
- [ ] After the product diff is frozen, run the native workspace test command
      once and build one packaged macOS acceptance App because product/package
      input changed:

```bash
cargo test --manifest-path packages/qiongli-native/Cargo.toml \
  --workspace --all-targets --all-features --locked
bash scripts/build_macos_acceptance_app.sh
git diff --check
```

- [x] Do not run authenticated model prompts or mutate the normal Host profile.

## 7. Close Only The Proven Outcome

- [ ] Record focused and packaged evidence without claiming public or
      target-native release qualification.
- [x] Update the narrow product-control spec if the implemented Ready contract
      adds a reusable invariant.
- [ ] Run Trellis check/update-spec, commit, archive P0, then start P1.

## Evidence

- `python3 scripts/validate_arc_201_adrs.py` and the frozen 2.x architecture
  guard pass with accepted `ARC-213`.
- Focused Host regression: 12 passed; native workspace all-target/all-feature
  tests pass with the explicit real-client cases excluded from that aggregate.
- Isolated clean-client evidence passes with Codex `0.147.0-alpha.6.5` and
  Claude Code `2.1.222`; neither test uses the normal Host profile.
- App API: check plus 32 tests pass. Desktop: check, 244 tests, production
  build, and bundle contract pass. Capability Contract v2 is valid.
- Packaged macOS vertical acceptance remains the only open product-input gate.

## Review Focus

- One approval is bound to exact fixed commands, not rendered strings.
- Command success alone cannot become Ready.
- Partial failure cannot touch unrelated Plugins or later targets.
- Existing CLI/install/control owners are reused; no generic framework appears.

## Rollback Point

Revert the single product change to restore manual Host actions. Inspect and
remove only receipt-owned Qiongli state through existing explicit flows; never
delete Host caches directly.
