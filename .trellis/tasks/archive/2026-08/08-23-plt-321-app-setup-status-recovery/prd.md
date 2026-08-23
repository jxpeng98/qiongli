# Close PLT-321 App setup, status, and recovery

## Goal

Stabilize the packaged App as a client of the existing native product owners so
a user can set up a supported Codex or Claude integration, see fresh and
truthful status, recover a receipt-owned stale/error state explicitly, and
complete one bounded critical workflow without App-only product logic.

## Background

- `PLT-320` is accepted on product commit `f3c2c0edea04` with Native CI run
  `32653636419`; PR #140 contains its evidence closeout and remains unmerged.
- The current branch contains that dependency and will be the base for this
  stacked Slice unless PR #140 merges first.
- Program Ledger v1 records `PLT-321` as `proposed`, depending only on
  `PLT-320`, and defines the outcome as App setup, status, recovery, and one
  critical workflow with explicit stale/error states.
- The App already consumes versioned App API snapshots and intents from the
  native desktop service. The native owner already implements receipt-bound
  Codex/Claude setup, fixed official Host CLI plans, fresh Host/cache/MCP
  probes, explicit install/repair actions, and a bounded Lite MCP self-test.
- Existing focused tests cover individual readiness matrices, stale/cache
  drift, unsupported clients, command failure, source-build read-only behavior,
  and selected-target install/repair preconditions. There is not yet one
  exact-source App Slice receipt that binds setup, status, recovery, and the
  selected critical workflow together.

## Requirements

### R1 — Keep native ownership authoritative

- Reuse the existing native desktop service, App API schema, packaged product
  control, Host command allowlists, and receipt/digest chain.
- The Svelte App may project state and send versioned intents; it must not write
  Plugin trees or Host caches, run arbitrary commands, or implement a second
  readiness/recovery state machine.
- Do not add a new dependency, installer, route, registry, or acceptance
  framework.

### R2 — Prove setup through the App contract

- Starting from a supported detected Host with no managed Qiongli integration,
  the App journey identifies setup as available, previews the exact managed
  targets and required approvals, and applies only after explicit confirmation.
- Setup continues through the existing receipt-owned product installation and
  fixed official Host CLI plan for the selected Codex/Claude target.
- Unsupported, absent, or unselected clients remain blocked or inspect-only and
  do not gain mutation authority.

### R3 — Make status fresh and fail closed

- Ready requires fresh managed source, registration, Host activation/cache, and
  MCP attachment evidence from the native owner.
- Missing, unsupported, stale, drifted, command-failed, probe-unavailable, and
  recovery-required states remain distinguishable through the App API and UI.
- A refresh clears earlier observations before probing; old success cannot keep
  an integration Ready after the underlying state changes.

### R4 — Prove explicit recovery

- A receipt-owned stale or drifted integration becomes repair-ready rather than
  silently repaired or flattened into a generic failure.
- Repair uses the existing preview, digest-bound confirmation, fixed Host plan,
  post-apply verification, and fresh observation path.
- Failed or partial repair stays non-Ready, exposes a bounded remediation code,
  and leaves unrelated Host/user state untouched.

### R5 — Close the App journey on Full MCP

- Add a user-visible Full MCP self-test to the App integration surface with
  typed run, poll, cancel, success, failure, and timeout states.
- “Full MCP” means the exact combined Lite, Full project, and Full orchestration
  registries from the embedded native contracts, plus successful dispatch of a
  Full-only route that cannot fall back to a Lite upgrade response.
- Bind the Full MCP result to the same native version and integration status
  shown by the App. A stale/error integration remains visibly non-Ready even if
  the local embedded Full contract itself is healthy.
- Preserve the existing bounded, offline, credential-free self-test behavior.
  This Slice does not exhaustively execute every Full tool or duplicate their
  existing domain tests.

### R6 — Record Slice evidence truthfully

- Add the smallest cross-layer regression that proves the complete App journey;
  fix production code only if that regression exposes a shared-root defect.
- Run focused App/native checks and exact-head Slice CI, then add one
  path-redacted acceptance note and set `PLT-321` to `accepted` with exact
  product commit and run identity.
- Preserve explicit nonclaims for Graph v1 (`PLT-322`), candidate packaging,
  real user profiles/data, signing, promotion, publication, and release.

## Acceptance Criteria

- [ ] A supported missing Codex or Claude integration reaches a confirmable
      setup plan through the App contract without direct App-owned mutation.
- [ ] Fresh native evidence is required before Ready; stale, drifted, failed,
      unavailable, recovery-required, and unsupported states remain explicit.
- [ ] A receipt-owned stale/drifted target follows explicit repair and cannot
      return to Ready until post-repair verification and fresh Host/MCP probes
      succeed.
- [ ] From a repaired Ready integration, the App runs a typed Full MCP self-test
      that verifies protocol initialization, the exact combined registry, and
      one Full-only dispatch without a Lite upgrade/profile response.
- [ ] The Full self-test is cancellable and bounded, reads no provider secrets,
      and cannot turn stale/error integration evidence into Ready.
- [ ] Focused frontend/App API/native tests and exact-head Native CI pass at
      Slice tier.
- [ ] A redacted acceptance record binds the result to one exact product commit
      and run; the ledger and generated current index are current.
- [ ] No App-only product authority, new public schema unless strictly required,
      new dependency, Graph/release claim, or normal Host-profile mutation is
      introduced.

## Out of Scope

- Graph v1 migrated-project acceptance (`PLT-322`) or Graph v2/Kernel work.
- A second Plugin/Skills/MCP/Zotero implementation in the App.
- New clients, arbitrary Host commands, automatic background repair, filesystem
  watching, performance refactors, or UI redesign.
- Candidate/package acceptance, authenticated model sessions, real user Zotero
  libraries, signing/notarization, promotion, publication, or release approval.

## Key Decisions

- The critical workflow is `Client Integration Ready -> Full MCP self-test`.
- Full readiness requires the exact Lite + Full project + Full orchestration
  registry and a representative Full-only call; process startup alone is not
  sufficient.
- Exhaustive execution of every Full tool stays with existing domain suites and
  future candidate acceptance. It is not required to prove the App can operate
  the Full profile.
