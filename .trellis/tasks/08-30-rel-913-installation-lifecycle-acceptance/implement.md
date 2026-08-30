# REL-913 implementation plan

## 1. Name the focused existing contract

- [ ] Prefix the existing native payload apply/remove and repair tests with
      `rel_913`.
- [ ] Prefix the existing managed-content activation/rollback and macOS update
      commit/failed-health rollback tests with `rel_913`.
- [ ] Add only missing version and project/global/unmanaged preservation
      assertions.

## 2. Harden candidate lifecycle acceptance

- [ ] Seed bounded project, global-state, and unmanaged-state canaries before
      candidate preview in each Codex and Claude Code isolated home.
- [ ] Verify exact canary bytes after rejection/compensation, install/verify,
      and uninstall; expose path-redacted lifecycle check names in the existing
      receipt.
- [ ] Run the existing candidate example as a Linux/macOS/Windows matrix and
      include the target label in uploaded artifact names.
- [ ] Update only the closest branch-policy assertions required by the matrix.

## 3. Make packaged update evidence versioned

- [ ] Derive and ad-hoc sign a version-distinct predecessor App fixture inside
      `macos_native_update_journey.sh`; retain the exact package as the target.
- [ ] Assert current-version advancement on healthy activation and predecessor
      restoration on failed health.
- [ ] Add project, global-state, and unmanaged-state canaries to both journeys
      and record additive path-redacted receipt checks.
- [ ] Preserve the existing nonclaims for production trust, network selection,
      publication, and clean-machine status.

## 4. Freeze and merge the implementation

- [ ] Add the seven-section REL-913 executable scenario to the product-control
      Trellis spec.
- [ ] Run formatting, the two focused `rel_913` filters, affected Clippy/tests,
      shell syntax, workflow policy, roadmap check, and Trellis validation.
- [ ] Commit, open a PR to `2.x`, wait for exact-head required Slice CI, and
      merge only when all required checks pass.

## 5. Run Acceptance and close REL-913

- [ ] Dispatch Native CI on the exact merged `2.x` source and wait for all
      three candidate targets, packaged product, macOS update journey, and the
      downstream non-publishing candidate to succeed.
- [ ] Download and inspect the three candidate receipts, packaged-product
      receipt, macOS update receipt, and aggregate candidate identities/digests.
- [ ] Commit the source-bound acceptance receipt, set Program Ledger `REL-913`
      to `accepted`, regenerate the index, and merge the evidence-only PR.
- [ ] Archive the Trellis task and record the session only after accepted
      evidence is merged.

## Focused validation

```bash
cargo fmt --manifest-path packages/qiongli-native/Cargo.toml --all -- --check
cargo test --manifest-path packages/qiongli-native/Cargo.toml \
  -p qiongli-platform rel_913 --locked
cargo test --manifest-path packages/qiongli-native/Cargo.toml \
  -p qiongli --lib rel_913 --locked
bash -n tooling/scripts/macos_native_update_journey.sh
python -m unittest tests.test_branch_policy -v
python3 tooling/scripts/update_program_roadmap.py --check
python3 ./.trellis/scripts/task.py validate \
  .trellis/tasks/08-30-rel-913-installation-lifecycle-acceptance
```

## Risk and rollback points

- Do not convert the ephemeral predecessor into a published-release or
  production-signing claim.
- Do not weaken candidate approvals, receipt closure, conflict refusal, update
  health, or path ownership to make Acceptance pass.
- Do not accept REL-913 from ordinary Slice CI or historical lifecycle receipts;
  the exact changed candidate needs a fresh explicit Acceptance run.
