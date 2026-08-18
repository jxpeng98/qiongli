# Correct architecture, ADR, and parity truth

## Goal

Make the current Qiongli architecture, accepted ADR set, and 1.x parity state
unambiguous and machine-checkable so governance records cannot overstate what
the 2.x product implements.

## Background

- `docs/architecture.md` already names the current Tauri 2 / Svelte 5,
  Rust-native, Host-driven architecture, but no focused regression protects
  that overview from returning to the superseded egui/direct-provider model.
- Two accepted files currently claim ADR number `0208`. The earlier
  target-specific launcher ADR is already registered as `ARC-208`; the later
  Community Alpha distribution decision is unregistered.
- `tooling/architecture/arc-201-decisions.json` is a frozen 0201-0207 baseline.
  The architecture guard intentionally forbids rewriting it, so it cannot also
  serve as the current decision inventory.
- The 1.x parity ledger uses root `status: complete` for classification
  completeness while six capability outcomes remain deferred. Its per-outcome
  dispositions are truthful, but the root name can be read as implementation
  completeness.

## Requirements

- Preserve the existing architecture choice and add a focused check that the
  English overview identifies Tauri/Svelte, Rust native ownership, ADR 0210,
  ADR 0211, and Host-driven model execution.
- Keep the earlier launcher decision as ADR 0208 and renumber the later
  Community Alpha distribution decision to the next unused number, ADR 0215.
- Update every repository reference to the renamed Community Alpha ADR.
- Add one additive machine-readable current decision registry covering every
  accepted ADR file while leaving the frozen ARC-201 baseline byte-unchanged.
- Extend the existing ADR validator and tests to reject missing, extra,
  duplicate, reordered, path-mismatched, or metadata-mismatched current
  decisions.
- Rename the 1.x parity root field to `classification_status`, bump its schema
  version, and preserve each capability's current disposition and evidence.
- Make the parity test state explicitly that classification completeness is
  independent of implementation; deferred capabilities must remain deferred.
- Track `GOV-405` through `GOV-407` as active during implementation and accept
  them only after exact-head CI evidence exists.

## Constraints

- Do not modify the frozen ARC-201 inventory or ADR 0201-0207 files.
- Do not change runtime behavior, capability disposition, or release authority.
- Use the existing Python validator, Rust parity test, and standard library;
  add no dependency or second validation framework.
- Historical roadmap checkboxes remain presentation only; live state stays in
  Program Ledger v1.

## Acceptance Criteria

- [ ] `GOV-405`: a focused automated check proves the architecture overview
  names the current presentation, native owner, and Host execution boundary.
- [ ] `GOV-406`: accepted ADR numbers and paths are unique, the Community Alpha
  decision is ADR 0215, the current registry exactly covers all ADR Markdown
  files, and all renamed references resolve.
- [ ] `GOV-407`: the parity record exposes `classification_status`, does not
  expose ambiguous root `status`, and keeps implemented, deferred, and retired
  outcomes distinguishable through validated dispositions and evidence.
- [ ] Existing ARC-201 frozen-baseline checks still pass without changing their
  protected files.
- [ ] Program Ledger v1 validation and generated-index freshness pass.
- [ ] Focused Python and Rust tests plus repository formatting checks pass.
- [ ] Exact-head required CI passes before `GOV-405` through `GOV-407` become
  `accepted`.

## Out of Scope

- `GOV-408` and later schema/authorization governance.
- New runtime architecture, Graph behavior, package contents, or UI work.
- Rewriting accepted ADR prose beyond the Community Alpha number/path metadata.
- Reclassifying any 1.x capability as implemented, deferred, or retired.
