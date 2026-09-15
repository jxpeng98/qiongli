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
2. Qualify macOS ARM64 manually and Windows/Linux x64 through
   `native-cli-distribution.yml` at the same commit; assemble locally, retain
   the Mac receipt and verify the exact combined packet on Linux/Windows.
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

### Execution and review

The user-confirmed **gpt-6-astra / low** executor worked in isolated checkouts.
Compatible-main implementation commits are `dc4e83f3`, `651bfdb9` and `0d0a6aba`.
The coordinator inspected the code and callers independently of the executor,
then returned two repair rounds before integration:

- Require an integer schema version; Python boolean equality must not accept
  `true` as v1. Pin the six Next archive names and three platform IDs independently
  of helpers that change on the development branch.
- Upload only verified manifest-listed artifacts plus the manifest/checksum
  files. Unlisted staging files must not become public attachments. Retain the
  missing-index refusal and existing immutable-tag/publication checks.

All selected-scope findings are resolved in the actual final diff. The
coordinator's policy/docs and main integration are self-reviewed; no independent
program acceptance is claimed. Main's two publishing guides also needed their
current-version examples corrected from 2.0.0 to 2.0.1.

Before the CI policy increment below, local `main` was **`789b0bbe`** (`release/2.0.1-transition-reviewed`), a fast-forward
of the compatible candidate. Native runtime and canonical content have no byte
diff from `fe0d4531`. Canonical versioned content remains at
`dc73165567cd0df670f71e4dd920e3fa02bf8058`.

Development forward-port commits are `d430a922`, `80e177ff`, `31e5e9d8` and
**`60f2cf37`**, based on coordinator plan/policy source `bcd21b22`. The coordinator
reviewed its resolved diff: production channel-aware projectors, v2 runtime and
legacy receipt readers remain unchanged. Its positive 2.0.1 packet fixture
reuses the existing legacy projection owner; it does not weaken the new gate or
pretend the development runtime is a compatible patch.

### Focused validation

| Source / scope | Observed result |
|---|---|
| Compatible main release tooling | 40 tests pass across `test_native_cli_release`, `test_native_marketplace_plugins`, `test_native_release_assets`, `test_native_registry_packages`, `test_release_version_contract` and `test_native_release_publish`. |
| Main architecture/schema | 35 tests pass across `test_arc_201_adrs`, `test_frozen_2x_architecture_baseline` and `test_public_schema_policy`. |
| Main distribution/install docs | 24 tests covered. The first combined run failed the publishing-guide version examples; after the two guide fixes, all 12 distribution-document tests pass. The other 12 CLI-guide results remain applicable. |
| Development forward-port | 41 tests pass across the same six release modules; legacy stable packets and 2.0.1 transition negatives remain covered. |
| Separate retained diagnostic lane | 23/24 pass; `test_schema_rejects_cross_field_channel_mismatch` still fails because its negative replaces stable with stable. Its test/validator sources are unchanged and outside the CLI release/assets/publisher call chain. Full Python discovery remains unqualified. |

### Final-main artifact qualification

The existing owner completed successfully on clean main
`789b0bbe8b3f9fdfcfae3af00ee7b4b658621aed`:

```sh
bash scripts/release_ready.sh --version 2.0.1 --cli-github \
  --staging-dir /private/tmp/qiongli-2.0.1-789b0bbe-qualified
```

Receipt: `/private/tmp/qiongli-2.0.1-789b0bbe-qualified/assets/release-manifest.json`.
Observed target: `aarch64-apple-darwin`; Rust **1.97.0**; status
**qualified-unpublished**; managed product authority **false**. Format and
Clippy pass, as do **40 CLI + 7 MCP tests**, extracted empty-PATH CLI smoke,
npm/wheel installs and both bundled native Plugin checks. Localhost permission
was granted through the normal execution review for the existing Zotero test;
no check was bypassed. Source remained clean and unchanged throughout.

The coordinator checked all five artifact sizes/hashes and `SHA256SUMS` against
actual files. The extracted CLI, npm install, wheel install, Codex Plugin and
Claude Plugin each retain actual both-Host v1/Next observations in the receipt.
Both Plugin archives verify against the final source and exact executable.

- Executable SHA-256: `7af0316aa141fee061643d0455b5bb5bf71ed68f7e015b2e9cd44fa8a26573b7`.
- Content pack SHA-256: `2bafecb0a9c217d90151b92b94736233bb0b25402fab24cab1c566e776bfab29`.
- Content root and versioned content commit are unchanged. The final executable
  is source-bound to this candidate; the earlier fe0d4531 receipt is historical.

| Artifact | SHA-256 |
|---|---|
| `qiongli-2.0.1-aarch64-apple-darwin.tar.gz` | `0cabec055e3895efc9b1bab05451a15ae72f6444c6d67d7c1b4dcd580e6650de` |
| `qiongli-2.0.1-py3-none-macosx_11_0_arm64.whl` | `0ba06d5bfce271ea3205fb78e181d580011c05cf1d69c521dd8c12e94628dc24` |
| `qiongli-2.0.1.tgz` | `a22360936e360971432b4c87cbf7d30cba36b8b2dacc5eb56631b3d59fff1dc5` |
| `qiongli-next-claude-plugin-v2.0.1-aarch64-apple-darwin.tar.gz` | `b31901fc1da27a34fa1a73fb405c78cdda27d9b7bbef8395f5026bfcac9c37fb` |
| `qiongli-next-codex-plugin-v2.0.1-aarch64-apple-darwin.tar.gz` | `d9ff1f55ae6941490e3eeac27a2944588243e7e73a193f0ccbdbeec5ca83b5a3` |

Windows ACL setup has mocked checks only; neither those checks nor synthetic
combined packets constitute Windows/Linux runtime qualification. Cargo's real
final-candidate source-package and public-install checks remain in the external
release lane; macOS npm/wheel checks do not stand in for them.

### Actual old-to-new upgrade and recovery

The coordinator executed the actual installed 2.0.0 binary and final extracted
2.0.1 binary in a private, disposable checkout fixture. Report and runnable
harness are `/private/tmp/qiongli-2.0.1-789b0bbe-qualified/upgrade-check.json`
and `upgrade-check.py` in the same directory. Old executable SHA-256:
`bcd93378265d30c5a7b8f09b5b9278afedabddae08b305c6a02967ac37edac5d`; the new
hash equals the qualified executable above.

Both **Codex and Claude** pass: old source install, approved update to 2.0.1,
repeat update with identical source hashes, refusal without approval, stale-plan
refusal preserving the test's added file, retry after removing only that test
file, restoration with the old 2.0.0 CLI, and a second upgrade to exactly the
same 2.0.1 source bytes. Final status remains v1/Next and source-current; it
explicitly reports Host state as not verified. Final source inventories contain
457 Codex files and 437 Claude files including receipts.

Existing synthetic home/model-setting files remain byte-identical. The only
new home files are the two expected empty coordination locks. No real Host
registration, Plugin cache, user model configuration or existing installation
is changed. This is local Plugin **source** upgrade/recovery evidence, not
live Host activation or end-to-end Python 1.x product migration evidence.

Harness preparation preserved two earlier fixture failures: reserializing a
canonical plan caused a refusal, resolved by retaining raw CLI plan output;
asserting no new home files overlooked the expected empty locks, now checked
explicitly. Neither failure was bypassed in the successful final run.


### Integration checks

All seven program-roadmap tests pass; the generated index is current. The ADR
validator passes for seven frozen and 29 current decisions. Frozen-architecture,
native change-boundary and diff checks pass. The native boundary classification
is conservative for tooling inputs; it does not claim completed Desktop/Lite
or external platform matrices. Reviewed local branches integrate by fast-forward;
no changed release source or post-merge-only test rerun is needed.

### Remaining release gates

The local implementation/review scope and external publication scope stay
separate. No push, tag, remote dispatch, publication, live user Host registration
or announcement has occurred. Windows x64 and Linux x64 qualification, the real
three-target packet/combined installs, public registry/download checks and
explicit publication authority remain required. 2.1 consumer retirement and
identity migration retain their own upgrade/cancellation/recovery gate.

The next independently actionable implementation is the bounded baseline
validation diagnosis already in the master roadmap. All 249 task
states/dependencies and all 46 accepted rows remain unchanged. The plan, ledger
and generated index record local completion without promoting task acceptance.


## Local macOS CI policy increment — September 15

The maintainer explicitly selected manual local macOS checks with retained
receipts. ADR 0230 supersedes only hosted execution; Linux/Windows remain on
Actions and all three targets remain required for release qualification.

### Implementation and review

- Development commits **`21e3f43b` / `86714195`** remove all macOS runner rows
  from the six affected workflows. Retained Community Alpha aggregation and
  authorization use the existing local owners after all three target results.
- Existing release assets/publisher owners now require a complete locally
  assembled packet, manual Mac combined npm/wheel/Plugin/Cargo checks and
  Linux/Windows install receipts bound to the exact manifest digest. Publication
  verifies the reviewed draft, source refs and unchanged assets before publishing.
- Executor **gpt-6-astra / low** implemented four workflow changes and their
  checks. The coordinator implemented the release handoff and documentation;
  the executor independently reviewed those six release/tooling/test files.
  Its one actionable finding (retain Linux tooling tests and all-wheel metadata
  validation after removing hosted assembly) was fixed. Final source review
  found no remaining actionable findings; documentation is coordinator-reviewed.
- The executor ported only these changes onto compatible main. The coordinator
  reviewed the resolved import and exact diff, then fast-forwarded local main to
  **`4ebd1825c72e473168b4692a5f9edad99f205ca0`**, after `45330d07`.
  Main retains its legacy Next/v1 owners, 2.0.1 examples and unchanged native
  runtime/content. Development's deferred identity migration was not merged.
- Minimal workflow/test-only maintenance patches are prepared as
  `ci/local-macos-legacy-dev` **`401b253c`** from `70c5bd9e` and
  `ci/local-macos-legacy-release` **`7c71ea59`** from `8d2e9986`.
  The frozen 1.x branch/tag and product code are untouched. These patches apply
  the explicitly requested CI policy without introducing product changes.

### Observed checks and limits

Development: **165 focused tests pass** across branch policy, release automation,
CLI assets/publisher, Marketplace/registry packaging, distribution/install docs,
ADRs and public schema policy. All workflow YAML parses; **37 Bash step bodies**
pass `bash -n`. The ADR validator passes **7 frozen / 30 current** records; native
change-boundary, frozen-architecture and diff checks pass. Main port: **144
focused tests pass**, including its different legacy Marketplace projection.
Each maintenance patch passes the affected checkout-matrix regression check.
All seven program-roadmap tests pass; the regenerated index is current. An exact
ledger comparison confirms all 249 states/dependencies and 46 accepted rows
remain unchanged.

These are tooling/workflow checks performed locally on macOS. No new complete
Mac release qualification, Cargo archive installation, three-target draft
handoff, Linux/Windows Actions run or public installation ran for this source.
The earlier `789b0bbe` qualification remains historical and cannot qualify
changed main `4ebd1825`. Fresh exact-source release evidence is still required.

### Remote rollout remains pending

No push, tag, draft upload, remote rule update or publication occurred. A fresh
read of GitHub ruleset **18800504** still requires `Rust native foundation
(macOS)`. The prepared update removes only that context and retains Linux,
Windows, Native 2.x change boundary, Evaluation Truth, PR, deletion and
non-fast-forward protections. Snapshot and proposed request are
`/private/tmp/qiongli-macos-ruleset-before.json` and
`/private/tmp/qiongli-macos-ruleset-update.json`; re-read before applying.

Synchronize reviewed patches to the relevant remote branches under separate
push/rule authority and existing PR protections. Do not bulk-push unrelated local
`2.x` work or rewrite frozen tags. Remote branches still execute their old
workflows until synchronization. The local Mac guide supplies the release
handoff commands and receipt paths. All 249 task states/dependencies and 46
accepted records remain unchanged; this increment adds no program acceptance.
