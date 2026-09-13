# REL-913 installation lifecycle acceptance

## Goal

Prove that the exact REL-910 native candidate can complete the supported clean
install, upgrade, repair, rollback, and uninstall lifecycle without changing
user project bytes, global Qiongli 2 state outside the operation's receipt-owned
paths, or unmanaged Host/home state.

## Background

- Program Ledger `REL-913` depends only on accepted `REL-910` and directly
  unblocks the remaining `REL-906` replacement-retirement gate.
- `native_candidate_acceptance` already performs candidate-backed clean install,
  verification, conflict compensation, and uninstall for Codex and Claude Code,
  but its Acceptance job runs only on Linux and preserves only one generic file.
- The packaged-product acceptance already exercises a previous-content update,
  receipt-backed client repair, restart verification, and removal.
- `ManagedNativePayloadExecutor` already owns cross-platform apply, repair,
  verify, remove, rollback, receipt, quarantine, and failure recovery.
- The packaged macOS update journey already executes the real packaged update
  helper through atomic replacement, health commit, failed-health rollback, and
  cleanup, but its predecessor is not version-distinct and it does not preserve
  explicit project/global/unmanaged canaries.

The missing work is therefore current Acceptance evidence and stronger
fixtures, not another installer, lifecycle state machine, or public command.

## Requirements

1. Reuse the existing candidate, packaged-product, native-payload,
   reconciliation, and macOS update-helper owners. Do not add a parallel
   installer, transaction journal, lifecycle CLI, or public schema.
2. Run the existing candidate-backed Codex and Claude Code lifecycle on Linux,
   macOS, and Windows during explicit Native CI Acceptance. Preserve the current
   preview, approval-digest rejection, unmanaged-conflict compensation, verify,
   and uninstall behavior.
3. Before candidate mutation, seed each isolated home with bounded byte canaries
   representing a user project, global v2 state outside installer-owned paths,
   and unrelated Host/home state. Verify exact bytes after failed previews,
   failed apply/compensation, successful install, verification, and uninstall.
4. Make the macOS update journey use a version-distinct, ad-hoc-signed
   predecessor fixture and the exact current packaged application as the target.
   Prove both successful version advancement and failed-health restoration of
   the predecessor.
5. Preserve project, global-state, and unmanaged-state canaries across both the
   successful update and rollback journeys. Keep update state mutation confined
   to its existing private transaction paths.
6. Give the existing native-payload clean install/uninstall, payload repair,
   managed-content upgrade/rollback, successful update, and failed-health
   rollback tests a shared `rel_913` focused filter. Add assertions only where
   preservation or version identity is currently missing.
7. Keep Acceptance receipts path-redacted and non-publishing. Add explicit
   lifecycle and preservation checks without storing runner-local paths or
   project content.
8. Run exact-head Slice CI before merge, then one explicit exact-head Native CI
   Acceptance. Inspect the three target candidate receipts, packaged-product
   receipt, macOS update-journey receipt, and downstream exact candidate before
   accepting `REL-913`.

## Acceptance Criteria

- [x] `cargo test --manifest-path packages/qiongli-native/Cargo.toml -p
      qiongli-platform rel_913 --locked` passes the shared cross-platform
      payload clean-install/uninstall and repair contract.
- [x] `cargo test --manifest-path packages/qiongli-native/Cargo.toml -p qiongli
      --lib rel_913 --locked` passes the managed-content upgrade/rollback and
      macOS replacement commit/rollback contract on supported targets.
- [x] Explicit Native CI runs candidate acceptance for Linux, macOS, and Windows;
      each target completes candidate-backed Codex and Claude Code clean install,
      verification, and uninstall with all canaries unchanged.
- [x] Packaged-product acceptance proves previous-content update, receipt-backed
      repair, restart verification, and removal through the existing owner.
- [x] The packaged macOS helper advances from a distinct predecessor version to
      the exact current version after healthy activation.
- [x] The same helper restores the exact predecessor application and
      last-known-good version after failed health, with transaction cleanup
      complete.
- [x] User-project, global-state, and unmanaged-state canaries remain byte-exact
      after clean install, upgrade, repair, rollback, and uninstall.
- [x] Rust formatting, affected Clippy/tests, workflow policy, roadmap validation,
      Trellis validation, exact-head Slice CI, and explicit Acceptance pass.
- [x] A source-bound acceptance receipt records the exact product commit, Native
      CI run, downstream candidate run, target identities, and relevant receipt
      digests before Program Ledger `REL-913` becomes `accepted`.

## Out of Scope

- A new installer, updater, public lifecycle command, transaction abstraction,
  receipt format, App screen, or MCP/Plugin/Skill capability.
- Production Developer ID/notarization or Authenticode (`REL-911`), package
  manager publication or manager-specific UX (`REL-912`), independent public
  download verification (`REL-914`), or revocation policy (`REL-915`).
- Claiming that an ephemeral predecessor fixture is a previously published
  release, or that ad-hoc signing satisfies production trust.
- Live provider calls, developer-normal homes, Host caches, public publication,
  release authorization, or deletion of any user data.

## Notes

- The candidate matrix proves portable candidate lifecycle on every REL-910
  target. The packaged macOS journey proves the only current in-product binary
  replacement path. Package-manager-specific upgrade behavior remains owned by
  `REL-912` and is not inferred here.
- No blocking product, security, compatibility, or UX decision remains.
