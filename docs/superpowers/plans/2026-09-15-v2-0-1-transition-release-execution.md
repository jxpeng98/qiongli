# 2.0.1 transition release execution

Date: 2026-09-15. Authority: the maintainer retains a v1 transition in 2.0.1
and places v1 support removal in 2.1. Program tasks remain CLI-403 / CLI-410;
the program ledger owns their states and accepted evidence.

## Outcome and scope

- 2.0.1 is a stable compatibility patch. Its public `plugin-source-status`
  continues to produce v1, and local/public Plugin installation IDs retain
  their existing Next identities. A stable npm package still uses `latest`;
  the Plugin's temporary Next identity does not make the package a Beta.
- 2.1 is the planned boundary for retiring that v1 output and shipping the
  channel-specific identity migration: stable Qiongli, Alpha/Beta Qiongli Next.
  Retiring an output interface must not remove verified legacy receipt readers,
  upgrade/retry/rollback support or historical schema/fixture evidence.
- Here v1/v2 name the Plugin source-status interface, not Qiongli 1.x/2.x,
  the frozen Python release line, all MCP contracts or all receipt schemas.
- Preserve ordinary CLI behavior, canonical research content, model settings,
  approval/CAS checks, source ownership and one selected active channel per Host.
  Do not manufacture a v1 response that labels an observed Qiongli ID as Next.

## Sources and ownership

| Item | Exact baseline / owner |
|---|---|
| Released reference | v2.0.0, `4f2107f7fda9d8f6dd8874b7dcffb36f5d7751ba` |
| Existing patch candidate | local main `fe0d4531c8d5b4d70af5f11b1829f7db56a1ecbe` |
| Development source | `a90c060c` on `2.x`; includes the deferred migration |
| Existing local evidence | `/private/tmp/qiongli-2.0.1-fe0d4531-qualified-retry/`; macOS ARM64 only |
| Executor | User-confirmed `gpt-6-astra`, reasoning `low` (requested astra-light) |
| Execution checkout | `fix/release-201-transition`, isolated worktree from local main |
| Coordinator | This task: plan, architecture/roadmap records, final review and local integration |

The executor owns release tooling, focused tests, release notes and runbook in
its worktree. The coordinator owns this plan, architecture decisions, current
roadmap/ledger/index in the development checkout. No concurrent canonical edits.
Return exact commits, commands/results and gaps; a subagent's summary is not
acceptance. The coordinator inspects the actual diff and evidence.

## Execution steps and acceptance

### 1. Record the transition contract — coordinator

- Supersede ADR 0228's immediate stable-identity timing with a new accepted ADR;
  preserve the old decision. Record 2.0.1 retention and 2.1 removal explicitly.
- Update the current roadmap and ledger references without changing any of the
  249 task states/dependencies or 46 accepted records.
- Keep the patch candidate isolated from the unqualified development migration.

Acceptance: release notes, runbook, roadmap and architecture describe the same
version boundary; no document claims that 2.0.1 installs the new stable ID.

### 2. Enforce the patch contract through existing release owners — executor

- Trace `native_cli_release.py`, `native_registry_install_check.py`,
  `native_release_assets.py`, `native_marketplace_plugins.py` and their callers.
- Add the smallest reusable release check that verifies actual 2.0.1 behavior:
  v1 status and the legacy Plugin identities, including extracted/installed
  artifacts. Keep checks isolated from real user configuration. Preserve secure
  destination rules; do not skip a failing probe or weaken them for temporary
  paths. Scope new requirements so immutable historical releases remain verifiable.
- Bind the observed transition result to the existing release receipt and make
  final packet verification reject missing/wrong transition evidence for this
  candidate. Reject v2 output, mixed identities, source/version mismatches and
  substituted assets. Reuse existing source/digest/channel checks.
- Keep stable publishing on exact main/tag, GitHub stable/latest and npm latest;
  keep prereleases on their existing prerelease/next lane. Do not add a parallel
  publisher or a mutable npm tag as a runtime identity setting.
- Update 2.0.1 notes and the current runbook: temporary Next ID, v1 retention,
  planned 2.1 cutoff, supported upgrade path, recovery steps and qualification
  limits. The 2.1 removal gate must still require target/consumer evidence.

Acceptance: a valid synthetic/real 2.0.1 candidate passes; a version-labelled
2.0.1 candidate with v2/new IDs or absent evidence fails before publication.
Historical supported packets still verify. No publication/Host trust occurs.

### 3. Focused checks and review — executor, then coordinator

- Reproduce each demonstrated gap and retain its smallest meaningful negative
  check. Run affected release/registry/Marketplace/version/authorization tests.
- Preserve mismatch, unknown/stale source, tampering, unsafe path, cancelled
  approval and partial target-evidence negatives. Existing tests count only when
  their inputs match; do not rerun unrelated suites just because of a commit.
- Commit only scoped changes. The coordinator reviews request fidelity and
  behavior independently of the executor, including every affected caller.
  Send findings back for repair before local integration.

Acceptance: no unresolved blocking review findings in the selected patch scope;
exact tests/results recorded. Full-suite historical failures remain disclosed,
and any failure affecting this release must be resolved rather than waived away.

### 4. Integrate and qualify the final local candidate — coordinator

- Fast-forward the reviewed compatible branch into local main; do not merge
  all of `2.x` into it. Port shared release guards/docs back into development,
  resolving the different legacy/current Plugin projection owners explicitly.
- Versioned canonical content stays at its existing exact commit unless its
  bytes change. Regenerate embedded content only through its existing owner.
- Run the existing release-readiness owner on the exact clean final main into
  a new external staging directory, using pinned Rust 1.97.0. Use the normal
  permission mechanism for the localhost Zotero test listener if required.
- Verify extracted CLI, npm/wheel installs, both Plugin archives, transition
  evidence and all asset hashes. Use isolated checks for missing/current source,
  2.0.0 update, repeated update, refusal/drift and recovery where affected.

Acceptance: named macOS ARM64 receipt is `qualified-unpublished`, bound to the
final candidate and its artifacts. Earlier fe0d4531 evidence remains historical;
it does not qualify a changed release owner/candidate. The worktree is clean.

### 5. External release gates — pending separate execution authority

1. Synchronize the reviewed source/tag through existing rules without force
   rewriting protected refs or replacing immutable artifacts.
2. Qualify macOS ARM64, Windows x64 and Linux x64 from that same commit using
   `native-cli-distribution.yml`; verify the complete combined packet and installs.
3. Bind the publication decision to the exact candidate/assets/channels, then
   use existing release automation and npm/PyPI/Cargo publishers.
4. Verify public downloads/registry installs. Announcements and external
   Marketplace catalog changes retain their separate authorization.

No push, tag, remote dispatch, publication, live user Plugin registration or
announcement is authorized merely by this implementation task. Do all local
preparation and review first. Missing external checks remain pending, never pass.

## 2.1 follow-up gate

Test old Next to stable Qiongli, repeated migration, cancellation after source
export, registration failure, retry and restoration. Preserve caches and model
settings. Codex is primary; document and verify Claude's manual conflict step.
Test a stable candidate as well as Beta: Beta-to-Beta alone never exercises the
new stable identity. Update named v1 consumers and state unsupported external
consumer behavior before removing v1 output. Keep old receipt readers separately
versioned and tested. Do not auto-promote this follow-up to accepted.

## Completion record

Plan prepared. Executor dispatch, implementation, review, local integration and
fresh artifact qualification are pending. External release gates remain pending.
The coordinator will replace this paragraph with actual commits, checks,
findings/resolutions and remaining gaps after execution.
