# 2.0.1 transition release execution

Date: 2026-09-15. Authority: the maintainer retains a v1 transition in 2.0.1
and places v1 support removal in 2.1. Program tasks remain CLI-403 / CLI-410;
the program ledger owns their states and accepted evidence.

Current direction (September 30): the maintainer selected stable **2.1.0**,
including the npm-installable DSH Plugin and post-2.0.0 changes. The final
September 30 section owns this release increment; the 2.0.1 candidate remains a
historical compatibility observation, not a publication prerequisite.
The maintainer requests stopping local tracking once the release command is
accepted, without waiting for or claiming public publication completion.

September 22: ADR 0231 restores hosted three-platform CI.
The September 15 local-only policy below is historical. The next bounded
research increment is search → verified bibliography → Zotero linkage, as
diagnosed and planned in the final section.

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


## Local macOS CI policy increment — September 15 (superseded September 22)

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


## September 22: restore hosted CI and select literature delivery work

### CI restoration — implemented and reviewed locally

The maintainer restored the previous build approach after checking public-repo
billing. [GitHub documents](https://docs.github.com/en/billing/concepts/product-billing/github-actions)
free standard hosted runner time on public repositories, including macOS;
larger runners and storage have separate terms. ADR 0231 supersedes ADR 0230
without editing the historical decision.

Executor **gpt-6-astra / low** restored development in **`9e1f575d`** and
compatible main in **`370e250f`**. The coordinator checked exact diffs: operational
CI/release files match pre-policy development `5fccfe89` and main `789b0bbe`.
The v1/Next transition guards and each branch's existing projection owners
remain intact. Development has **158** relevant checks passing; main has **140**.
All seven program-roadmap checks, the ADR validator (7 frozen / 31 current),
frozen-source/native-boundary and diff checks also pass. All 249 task states
and dependencies plus 46 accepted records remain unchanged.
This restores hosted Mac/Linux/Windows build, assembly, combined installs and
publication gates. The local Mac receipt/draft-upload requirement is retired.

The September 15 legacy CI-only branches and proposed ruleset-removal payload
are withdrawn from rollout and must not be applied. No remote rules, refs,
Actions runs or publications were changed. These source checks do not freshly
qualify a changed release candidate.

### Literature diagnosis — actual observations and source bounds

The user's report concerns weak literature-search assistance and reliable BibTeX
plus Zotero linkage. A specific failing user query was requested but not supplied
at diagnosis time. A bounded public known-title probe was used instead; it is
not a topical-search benchmark or proof of the user's exact failure.

Observed through the currently installed Plugin on September 22; the visible
CLI reports **2.0.0**, while the underlying MCP executable's full source identity
was not independently verified. Inspected repository source was `6471ec1d`;
CI restoration leaves the relevant native/content sources byte-identical.
Raw public probe: `/private/tmp/qiongli-literature-probe-2026-09-22.json`.

| Observation | Existing owner / consequence |
|---|---|
| Literature status reports OpenAlex, Semantic Scholar and arXiv configured; Crossref email and PubMed key missing under current runtime policy. | `providers/access.rs`; configuration readiness alone does not establish search quality. No settings or credentials were changed. |
| All three selected providers returned records for `Attention Is All You Need` (3 per provider, total limit 5). First result is a 2025 same-title record; the 2017 NeurIPS record is fourth. No returned record contains authors or a source URL. | `providers/search.rs`: provider-order concatenation then truncation, no relevance reranking; `LiteratureResult` stores only title/DOI/year/venue/provider(s). This observed ordering does not establish that the same-title 2025 record is fabricated. |
| The official [NeurIPS record](https://papers.nips.cc/paper/2017/hash/3f5ee243547dee91fbd053c1c4a845aa-Abstract.html) identifies the 2017 work and provides original BibTeX. | Known-item authority for this probe; a matching title or any resolvable DOI is insufficient to identify the intended edition. |
| Planner accepts `title`/DOI modes and filters; search executor accepts only auto/topic/review/systematic_review and no year/venue arguments. | `searchplan.rs` versus `providers/search.rs` and their schemas: a recorded plan is not evidence that its filters were executed. |
| Host exposes export `records` as `Array<string>`; passing a serialized synthetic record gives `-32602: records must contain literature results`. Canonical schema has an array without an item schema; Rust requires object records. | `qiongli_zotero_export_import_files` input schema and `zotero/export.rs`; explicit object-item contracts need a fresh Host verification. The missing item schema is a supported diagnosis, not independently proven converter internals. |
| Rust export drops author/citekey/type/detail fields, emits every record as article/journal and generates qiongli1, qiongli2 by list order. | `zotero/export.rs`, shared `LiteratureResult`; rewriting the Skill alone cannot make this a complete accurate bibliography. |
| Zotero connector and Companion 0.3.1 / endpoint 2 both respond successfully. | Connection check only. Existing Companion already returns item_key/select_uri and supports receipt-bound dry-run/apply; no library search or write was performed. |

### Literature increment — authorized implementation order and acceptance

Use the existing provider runtime, metadata/export owner, Companion, B-stage
skills and preview/approval/CAS. Do not create a new search service, bibliography
store or mandatory third-party Zotero plugin. The maintainer authorized execution on September 22. The ordered scope below
is implemented in the source increment recorded after its acceptance cases.

1. **Preserve bibliographic records end to end.** Add explicit object schemas and
   compatible optional metadata fields to the existing result/export contract:
   stable IDs and source URLs, ordered structured authors, publication type,
   dates, venue, volume/issue/pages or article number, publisher and existing
   citekey. Preserve fields from provider response through export. Existing
   incomplete records stay valid but visibly incomplete. Parse/validate exports;
   never invent authors, DOI, year or pages. Existing citekeys survive reordering
   and repeated exports; choose the correct entry type.
2. **Verify identity and improve targeted retrieval.** Make planned modes/filters
   executable or explicitly mark unsupported portions for Host-side checking.
   Reuse configured providers plus available Host search with separate provenance.
   Compare title, author, year and version, then enrich selected records through
   DOI/registry or publisher metadata. Official [DOI content negotiation](https://www.crossref.org/documentation/retrieve-metadata/content-negotiation/)
   provides BibTeX/CSL-JSON across registration agencies; it still needs identity
   and field validation. Handle no-DOI preprints/books by their real IDs and
   official source. Rank/deduplicate before the final result limit, retaining
   contrary/near-miss evidence and rate-limit/coverage failures.
3. **Give Skills one scoped delivery path.** Reconcile the provider-first prose
   with the existing hybrid/native-only fallback. A request for a few relevant
   papers should return verified candidates, brief relevance, persistent source
   links, requested BibTeX and explicit missing fields. Read the metadata and
   reference-manager cards when those outputs are requested. Formal B1 retains
   its protocol/diagnostic gates; bounded lookup needs no systematic-review
   scaffold. Remove stale hardcoded API quotas and keep configuration guidance
   tied to observed capability.
4. **Link selected verified records to Zotero.** Reuse Companion status, exact
   dry-run, reviewed approval and receipt-bound apply. DOI/stable-ID dedup and
   fill-blank policy preserve curated records; verify actual results and retain
   record_id/citekey ↔ item_key/select_uri in the existing import report. Reuse
   that mapping on retry and verify library scope before constructing group
   links. Import files remain available when Companion is unavailable; generating
   a file or succeeding at dry-run never means Zotero import succeeded.

Acceptance starts with a small fixed corpus: the known 2017 paper and a same-title
wrong-year candidate; a paper with an official DOI/BibTeX; a no-DOI preprint;
non-ASCII/corporate authors; duplicate/reordered records; and metadata conflicts.
Check author/order/type/DOI/year/citekey preservation and BibTeX parseability,
exact filter application, Host object-schema calls, zero fabricated fields,
Zotero repeat-import identity and refusal after stale/changed/cancelled approval.
Then run a real topical query supplied by the maintainer and independently review
relevance and source matching. Fixture success alone does not establish search
coverage. Live writes require their exact reviewed preview; this planning request
has not written to the user's library or updated the installed Plugin.

This increment is separate from the already bounded 2.0.1 transition patch;
choose its release version after the behavior and compatibility checks. CLI-405
and CLI-410 retain their current states, dependencies and accepted-evidence
heads. The new selected priority is literature delivery; unrelated baseline
failure groups remain recorded for later bounded work.


### September 22 execution result — literature delivery

Source: `15a8b901a141df6079fb907a06340dcecba03f4b`, developed on
`fix/literature-bibliography-delivery` from `3cb7a063`. This remains a development
increment on `2.x`, separate from the 2.0.1 transition patch on `main`.
`10fb4199` adds only the reviewed Crossref enum boxing required by Clippy,
without changing serialization or result semantics; the independent reviewer
confirmed that final two-line adjustment has no additional finding.

- **Records/export:** the shared Rust owner retains ordered structured or literal
  authors, publication type/date, source ID/URL, volume/issue/pages, publisher and
  explicit citekeys. Typed object-array MCP schemas reach native Lite/Full and
  the Lite compatibility executable. Stable generated keys replace list indices;
  missing/conflicting metadata is visible and unknown types export generically.
  DOI normalization, bounded inputs/output, repeat/reordered keys and distinct
  preprint versions retain regression checks. Export does not verify metadata.
- **Search:** exact normalized title/DOI selection and year/venue filters precede
  final truncation; relevance ranking is bounded and never certifies identity.
  Crossref DOI uses its item endpoint, OpenAlex DOI uses its identifier filter,
  and arXiv title mode uses a quoted title query. Other filtering is explicitly
  over returned candidates; unsupported planner-only fields remain Host checks.
- **Skills:** scoped lookup links discovery → authoritative identity/field check
  → requested bibliography → optional approved Zotero import. Native-only/hybrid
  routes preserve actual provenance; stale quota claims are removed. No new
  service, bibliography store or mandatory Zotero plugin was introduced.
- **Zotero:** both live bootstrap and testable bridge now guard versions/types
  before DOI/stable-URL/title-year matching. Within-batch duplicates fail before
  preview. Existing receipts, fill-blank writes and returned ordered item keys/
  select URIs own the mapping in the import report. Current scope is personal
  library; no group support or real user-library write is claimed.

Execution: the coordinator implemented runtime/Companion fixes. The retained
`astra_light_release_201` executor implemented canonical Skills and schemas.
Independent `literature_runtime_review` ran as **gpt-6-astra / low**, read source,
ran targeted Node identity assertions and simulated three Skill routing cases.
Its findings caught preprint/formal-version merging and `[A,B,A]` citekey order;
fixes and regressions are included. The same reviewer confirmed no remaining
blocking findings, including the final `15a8b901` arXiv/schema increment; this is
not live Host acceptance. The shared Full/Lite export schema stays with its
existing owner; no fictitious legacy Full export implementation or validator
exception was added.

The existing resource-lock generator binds 434 embedded entries to `15a8b901`,
content-root SHA-256 `8eb54a551b9d6a67e190ab5fc7d0daa8ac9f7055b2f849b3e8d86f9c5ef21848`.

Validation:

| Check | Result and limit |
| --- | --- |
| Native runtime | 63 tests passed: 48 unit, 6 bibliography, 9 MCP; actual `tools/list` object schema and `tools/call` covered |
| Native CLI consumers | 9 tests passed (7 copied-binary stdio, 2 provider runtime); rebuilt embedded Lite and Full both accepted object records and exported the eight-author public record |
| Lite compatibility | 93 tests passed; subsequent arXiv title change passed the 16-provider-HTTP-test suite (adds one case) and rebuilt the binary |
| Public schema/capability | 55 tests, 54 passed and 1 environment-dependent skip; complete contract validator passed |
| Companion | 31 tests passed, including both bootstrap/module repeat-import identity, links, changed/expired/replayed receipts and preview-without-write |
| Literature content | 17 tests run; two pre-existing exact-text assertions fail on unchanged `paper-read.md` (provider ownership phrasing); not new product regressions |
| Packaging | Existing Companion builder emitted a local XPI; no install/update or release publication performed |
| Static checks | Runtime Clippy with `-D warnings` passed after boxing the single-work Crossref enum variant; six bibliography regressions rechecked successfully; formatting and diff checks passed |
| Architecture | Native change-boundary and frozen-baseline guards passed; accepted evidence unchanged |

Public read-only smoke used the newly built Lite candidate with isolated config:
`Attention Is All You Need`, `search_mode=title`, `from_year=to_year=2017`,
`providers=[arxiv]`, `per_provider_limit=10`, `total_limit=5`. The initial
all-field query returned ten candidates but none survived exact filters; the
quoted title request then returned the intended record with all eight authors,
2017 date, preprint type and `http://arxiv.org/abs/1706.03762v7`. The candidate
exported it successfully; system BibTeX 0.99e parsed the result without errors.
This identifies the repository version, not a substituted NeurIPS proceedings
citation, and does not prove broad topical recall.

A separate authorized public read from the [Crossref work endpoint](https://api.crossref.org/works/10.1109/CVPR.2016.90)
returned *Deep Residual Learning for Image Recognition*, DOI
`10.1109/cvpr.2016.90`, four ordered structured authors and proceedings-article
type. The candidate exported `@inproceedings` with preserved explicit citekey;
BibTeX also parsed that file plus a clearly synthetic non-ASCII/corporate-author
case. This was Host-side public registry retrieval (`native:crossref_registry`),
not proof that a missing local Crossref configuration was connected.
Provider request semantics follow the [Crossref API](https://github.com/CrossRef/rest-api-doc),
[OpenAlex attributes](https://help.openalex.org/data/works/attributes/) and
[arXiv query contract](https://info.arxiv.org/help/api/user-manual.html).
Temporary diagnostic files are under `/private/tmp/qiongli-lit-public-probe`;
search JSON SHA-256 is
`f3020d06ed5101a975a494378cacfd9b0e6ec2b7a277651e8592fdb406e75d75`.

Remaining qualification: the maintainer-specific topical query has not yet been
supplied. Installed Plugin/Host refresh and a real library import require their
concrete previews and approval; neither has occurred. Candidate versions were
not bumped, so the local XPI is development-only and must receive an immutable
new Companion version before publication. Provider coverage/quota, Linux/Windows
and release-candidate acceptance remain their own gates. CLI-405/CLI-410 states,
all 249 task dependencies and 46 accepted rows remain unchanged. Next increment:
qualify a fresh installed candidate on the maintainer's actual query and approve
one bounded Zotero import after reviewing its exact proposed changes.

### September 22 next increment — Host delegation results (2.1 development)

Requested outcome: GPT/Codex coordinates actual Host subagents and reconciles
their source-bound proposals before one coordinator submits. Extend the existing
Host candidate with optional, bounded delegation observations: actual dispatch
tool/execution identity, scope, originating handoff, terminal state and exact
returned text/digest. Reuse the existing checkpoint CAS, authenticated reads and
candidate digest; no second task store or execution backend.

1. Add shared validation and the live Full MCP schema, preserving old candidate
   bytes when the optional field is absent. Reject stale, duplicate, unfinished,
   cancelled and tampered reports before advancing the checkpoint.
2. Update the existing collaborator Skill and portable templates to collect
   actual Host results and distinguish reported execution from authenticated
   research evidence. Configured external tools may return the same proposal
   format; shipping a new external runtime adapter is the following increment.
3. Run focused Rust/stdio compatibility and negative cases, independently review
   the diff and exercise the Skill with isolated synthetic inputs and actual
   Host tools. Regenerate the embedded pack through its existing owner.
4. Record results/gaps here and in the existing ledger, then locally fast-forward
   into `2.x`. Keep the 2.0.1 patch, accepted evidence and publication unchanged.

This implements ADR 0218's Host-owned boundary. A Host-reported execution ID is
not server authentication or proof of independence. Cancellation remains owned
by the dispatching Host; only observed completed, reconciled results are eligible
for submission. Native task claims across separate Hosts, automatic external
launch, crash recovery and real library writes remain unqualified.


### September 22 execution result — Host delegation results

Canonical Skill source: `051a35dd8dc8cdbac6fbbab991951d416f8a98cd`, developed
on `feat/host-delegation-results` from `423c1cbb`. Native implementation and pack
lock: `2e4a5c1e8a09c2f002ed0de9e7c9ecc3211426c8`. Runtime changes stay in the
existing `HostCandidateEnvelopeV1` and live Full MCP schema. Optional
`delegationResults` preserves old v2 canonical bytes when absent. Up to eight
observations bind adapter/execution ID, dispatch tool, scope, handoff and exact
result bytes/digest. Only completed results pass, with candidate and delegated
text sharing the handoff byte limit. Existing authenticated evidence, checkpoint
CAS and artifact-approval owners retain authority; submission stores only the
candidate digest. No package version or 2.0.1 patch change is included.

The retained executor `astra_light_release_201` updated the three canonical
Skill/template files. The coordinator implemented Rust validation and stdio
regressions. Independent `literature_runtime_review` (**gpt-6-astra / low**)
reviewed source against `423c1cbb` and reported no blocking findings. Its reviewed
runtime/Skill/template/test diff SHA-256 was
`4f6bfdce908d77b2e61a58ed216220cb011303a7df13a9076d54d1faa0993786`;
resource generation and validation records were completed by the coordinator.
After review, the coordinator added one exact combined-byte-budget assertion
and its over-budget UTF-8 negative case; production behavior remained unchanged.

The independent Skill forward test actually dispatched
`/root/literature_runtime_review/fixture_bibliography_audit` through
`collaboration.spawn_agent`, with an isolated source packet and no prior verdict.
Dispatch returned that identity; the same child's `FINAL_ANSWER` was collected.
The coordinator checked the returned corrections against synthetic registry
records, preserving `C-001`/`D-001`/`Alpha2024`, `C-002`/`D-002`/`Beta2025v2`,
source anchors, R2 and method limits. It restored the missing second author and
corrected a preprint's type while retaining its version and unknown venue.
Requested model/effort is recorded, not independently attested by the runtime.

Raw local artifacts remain under `/private/tmp/qiongli-delegation-forward`:
`dispatch.json`, `source.json`, `child-result.txt`, `candidate.json` and
`model_collab_trace.md`. Source SHA-256:
`807b3e7a91b553c5ca19c016c1aaebce162c664de7e3c95e639fb39434913c70`;
exact returned text SHA-256:
`a591a8c00789d5929e30021a5682a0a5bc98b4e44b7e5b7113421a1e330cbd68`.
Both were independently recomputed before recording. This test used the portable
trace; no registered Full MCP run, source-read receipt or project write was
fabricated. The copied-binary MCP tests separately use synthetic Host observations.
Together they provide development evidence, not installed-Host acceptance.

Validation on pinned Rust/Cargo 1.97.0:

- Execution library: **117 passed**, including old candidate round trips,
  native/external observation validation, duplicates, unfinished/cancelled states,
  digest and source-binding substitution, limits and debug redaction. An initial
  run had six resource-root mismatches after canonical content changed; these
  passed after regenerating the pack through its existing owner.
- Copied-binary MCP: **7 passed**, including live schema, completed result digest,
  cancelled/tampered/stale rejection without consuming evidence, second-process
  evidence rejection, duplicate submission and result arrival after run cancel.
- Existing Skill/handoff contracts: **11 passed**.
- Clippy for `qiongli-execution` and `qiongli`, all targets with `-D warnings`,
  passed; formatting and diff checks passed. After the final test-only boundary
  assertion, all six handoff tests and execution all-target Clippy passed again.
- Native boundary/frozen-baseline guards passed. All **249** task states and
  dependencies and **46** accepted rows remain unchanged.
- Embedded pack: **434** entries, bound to `051a35dd`; content root
  `755de18e25a313eb2db127e040c15efac51bf194d43b3f71398ad6d70a09c073`.

Remaining increment: add one explicitly selected, configured external Agent
transport through the same coordinator/result contract, then exercise timeout,
late cancellation and connection recovery with that actual runtime. This change
provides its result format, not automatic external launching or cross-Host atomic
claims. Live installed Plugin refresh, real research/library mutations, multi-Host
qualification, Linux/Windows execution and publication remain separate evidence
and authorization scopes. CLI-406 remains proposed; local integration is not
collaboration release acceptance.

### September 22 next increment — configured Codex external transport

The maintainer requests continuing to the external adapter. Select the existing
configured Codex CLI and its documented `exec --json --ephemeral` interface.
Preserve ADR 0218: the current Host's execution tools own launch, wait, cancel
and cleanup. Qiongli prepares the bounded invocation and validates the observed
event stream through the existing delegation result owner. This reuses native
Host process supervision rather than adding an App launcher or Agent daemon.

1. Add `agent codex prepare` and `collect`: bind the handoff and authorized source
   packet, preserve model/effort/auth, collect only completed, successful and
   matching results; return precise failure codes without private error text.
2. Update the existing collaborator Skill, help and native contract. Require the
   original process status and event log, current source checks and explicit
   recovery after interruption. Never infer success from a final-looking message.
3. Exercise protocol/CLI negative cases, an actual configured Codex synthetic
   task, cancellation and timeout. Independently review, regenerate the existing
   resource pack, record evidence/limitations here, and locally integrate `2.x`.

This is an opt-in Host-executed transport adapter. Multi-Host atomic claims,
automatic persistent-session reconnection, real research/library writes and
installed-package/release qualification remain separate. No 2.0.1 version,
accepted evidence or program task dependencies are changed.

### September 22 execution result — configured Codex external transport

Canonical Skill source: `e99f33876999f9b5796fb4c5ea85efff4f226675`.
Native implementation and embedded lock:
`05113e41b58760e3698e382b9b17f5fe0505ab8b`, developed on
`feat/codex-external-agent` from `cb0898d8` for local integration into `2.x`.
The native CLI adds `agent codex prepare` and `collect`; the calling Host executes
the returned fixed argv/stdin and owns process status, deadlines, cancellation
and cleanup. The adapter preserves configured accounts/model/effort, uses
`codex exec --json --ephemeral --sandbox read-only`, and returns the existing
`HostDelegationResultV1`. It adds no launcher, daemon, task store or MCP tool.
Result text retains the complete exact final message and its source-binding
wrapper; a proposal still needs coordinator review and authenticated evidence.

The retained executor `astra_light_release_201` updated canonical Skill/template
instructions. The coordinator implemented the native adapter and tests.
Independent `literature_runtime_review` (**gpt-6-astra / low**) reviewed the code
and actual run artifacts, checked result/source/thread bindings and reported no
blocking findings. Its final review covered the thread-ID guard, CLI help scope
and added file-boundary cases; the subsequent Clippy fix only replaced the parity
expression with `is_multiple_of`. This is scoped development review, not package
or research-quality acceptance.

Validation on pinned Rust/Cargo 1.97.0:

| Check | Result and limit |
| --- | --- |
| Execution library | **118 passed**, including successful source-bound collection, old candidate compatibility, malformed/truncated/duplicate events, cancelled/failed/nonzero outcomes, changed sources and byte limits |
| Native CLI | **41 passed**; prepare/collect works with empty PATH and creates no config; invalid flags, missing status, UTF-8, directories and stale packets fail |
| Copied-binary MCP | **7 passed**; existing candidate/evidence/CAS validation remains intact |
| Existing Skill/handoff contracts | **11 passed** |
| Final focused CLI rerun | Passed after adding oversized-file and Unix-symlink rejection cases and the Clippy correction; included in the 41 cases, not an additional unique test |
| Static and architecture | All-target execution/CLI Clippy with `-D warnings`, formatting, diff check, native boundary and frozen-baseline guards passed |
| Embedded content | Existing generator produced **434** entries bound to `e99f3387`; initial source-root mismatch was resolved by regeneration, not a validator exception |

Embedded content-root SHA-256:
`aed725b4a81a378a96e3487aab8de14d941e72b0a9c8176e48c2aa31578b26dd`;
pack SHA-256:
`4171b0b3f982a6f5d26358cc5edd67a858e86ac52cfc4f6d115e1b092b723f73`.

The live probe used configured **Codex CLI 0.155.1**, an isolated working directory
and synthetic bibliography/source/handoff fixtures. A temporary Host-side Python
harness executed the prepared command, supervised its process group and passed
the observed status/exit into collect; Python is not a shipped dependency.
No model override, installed Plugin refresh, registered Full MCP run or canonical
research/library write occurred. The original processes were reaped before the
explicit fresh run.

| Actual run | Observed execution identity | Outcome |
| --- | --- | --- |
| Success | `01a0c8dc-7cb7-7351-b150-a18524a81db8` | Exit 0; collected source-bound final reply, 34.09 seconds |
| Cancellation | `01a0c8de-0c66-72f2-9894-086e064bbef3` | Thread and turn started; Host terminated/reaped process; collection rejected, 2.42 seconds |
| Startup timeout | No thread ID emitted | 0.2-second startup deadline; empty JSONL; Host terminated/reaped process and collection rejected; 0.45 seconds total |
| Explicit fresh run | `01a0c8df-921e-76f2-93dd-a57a7d4034cd` | New execution identity, exit 0 and valid collection, 35.5 seconds; not session resume |

Both successful replies restored the missing author and journal metadata for the
synthetic Alpha record and corrected Beta to a versioned preprint with unknown
venue. They retained `C-001`/`D-001`/`Alpha2024`, `C-002`/`D-002`/`Beta2025v2`,
source anchors, R2 and the supplied method limits. These findings were checked
against the fixture, not promoted to real bibliographic evidence. One successful
stream contained a nonterminal Skill-context-budget warning; the actual completed
turn and process exit determined success.

Raw local packets, JSONL, results, receipts and the test harness remain under
`/private/tmp/qiongli-codex-external`. The independently checked JSONL hashes are:

- Success: `0bdcc26b4c9d11705688f1da776295d8667a94560254dd55ffc33e82fe3393fb`.
- Cancellation: `d4ea05e0606e4c7aa7d6a737f50d71b3aa11092dad5264b3dbe089670b3c0814`.
- Startup timeout: `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.
- Fresh run: `a4a1aa0adce9e49ae91cfc92093c1266ec5fd7ef0831e71187944c7eb66be4c8`.

Handoff SHA-256:
`f176847b4f855fd398b986abf261fbda1fcbbd995d7b6712a1d3459acf903a4a`;
packet SHA-256:
`5a9c343c62e9d4ddff09df3a76a48c2efa1c6096fa075308ad6e1603eef1b438`.

Remaining increment: exercise this same adapter inside a registered synthetic
Full MCP journey, including source revision change before candidate submission
and a deadline after the external turn starts. This probe proves startup timeout
and explicit restart only; in-turn timeout, network reconnection, persistent
resume and two-Host atomic claims remain unqualified. Installed-Plugin refresh,
real research/Zotero writes, Linux/Windows execution and publication retain their
separate scopes. The read-only sandbox is not full MCP/hooks isolation. All
**249** task states/dependencies and **46** accepted rows remain unchanged;
CLI-406 stays proposed. `main` and the 2.0.1 release candidate remain unchanged.

### September 23 test result — registered synthetic Full MCP journey

Tested local `2.x` at `130bebf3` with configured Codex CLI 0.155.1. Two isolated
synthetic projects under `/private/tmp/qiongli-codex-full-mcp-5cdv80_i` were
created and refreshed through the existing preview/approved-apply owner. A real
Full MCP stdio process handled `initialize`, doctor, start and an authenticated
`qiongli_project_read`. The coordinator saved its actual handoff, source snapshot
and evidence reference, called `agent codex prepare`, executed the returned
`codex exec` command using existing account/model settings, and passed exact
JSONL and observed process status through `collect` before submission. The
Full MCP server exited cleanly with empty stderr. Test-only Python supervised
the Host processes; it is not a product dependency.

| Case | Actual Codex execution and outcome |
| --- | --- |
| Changed source | Thread `01a0cb62-12e9-7ef3-8315-3e68274c74cd` completed in 70.69 seconds and collected successfully. The synthetic project's semantic revision changed from 2 to 3 before `qiongli_orchestration_submit`; the old candidate was rejected as `project-revision-conflict`. JSONL SHA-256: `6f841d940418fcf66764f3a464fdd5ba8f5b1d7f0e3feb9ac4ce6bac46886a1f`. |
| In-turn timeout | Thread `01a0cb63-2cd5-7a91-9bce-9da9b06be0ae` emitted `thread.started` and `turn.started`; after the Host deadline, SIGTERM was reaped at 0.95 seconds with no `turn.completed`. `collect` rejected it as `codex-exec-not-completed`. JSONL SHA-256: `c94153b20d37eeff0035e943e2fb454c965d41353f3c658c2a20f8fdc1df337a`. |
| Explicit fresh execution | Thread `01a0cb63-305e-72a0-a700-e0173dce6a3f` completed in 73.14 seconds. The coordinator collected its source-bound reply and submitted a synthetic candidate with the authenticated read evidence to the same Full MCP process. The server returned `candidate-accepted`, digest `909bd0cd8cbb64f69448c3f18292fd72cdb18f501a6f5228d207ab0f925783e1`. JSONL SHA-256: `573aad70239273f7290ea40a52835874ba2ab470b7ce22556299c66eb303c9e9`. |

The coordinator recomputed all three JSONL hashes, both collected result-text
hashes and their handoff/packet bindings. The fresh task used a new execution
identity; there was no session resume. The candidate's primary content was a
fixed synthetic canary, with the exact Codex response attached as a delegation
result. This validates transport, evidence binding, revision rejection and the
Full MCP submission path, not the scholarly quality of the model's proposal.
The source packet contained the authenticated project overview and evidence
reference plus an explicit synthetic method limit; it did not include a complete
bibliographic record or private library data.

No product source changed during this test. The existing 177 scoped tests and
static checks remain applicable to `130bebf3`; no redundant suite was run.
Installed Plugin behavior, a real research/Zotero journey, network loss and
persistent reconnection remain unqualified. Host-reported process status is
observational, not independent execution authentication. CLI-406 remains
proposed; all 249 task states/dependencies and 46 accepted rows stay unchanged.
Next qualify the adapter in a named installed-Host candidate and separately
exercise a user-authorized real research source without promoting these synthetic
observations into release acceptance.

### September 24 increment — DeepSeek Plugin and external Host adapters

Maintainer request: make Qiongli usable as a Plugin in the official DeepSeek
Harness and let the current Agents coordinate external DeepSeek, Antigravity and
Claude Code. The maintainer confirmed that “Cloud Code” meant Claude Code.
Continue 2.1 development under ADR 0218; do not change the 2.0.1 patch or add a
model backend, daemon, task store or project-write owner.

Implementation reuses `agent codex prepare/collect`, the shared handoff/source
packet and `HostDelegationResultV1`. Explicit `claude`, `deepseek` and
`antigravity` selections prepare fixed CLI invocations and validate each terminal
protocol/session identity. The Host owns dispatch, deadlines, cancellation and
cleanup; original process status and exit 0 are necessary for collection. Saved
accounts/model/effort stay with the selected Host. Codex serialization remains
unchanged; only DeepSeek adds a child-only read-only environment override.
Failed, cancelled, incomplete, stale, duplicate and identity-mismatched streams
cannot advance through collection. The coordinator retains authenticated reads,
candidate reconciliation, checkpoint CAS and project preview/approval.

The existing native Marketplace generator gains opt-in `--platform deepseek`.
It projects the same canonical Skills, target-specific native binary, Full MCP
and verifiable receipt into a Cordis bundle; no separate academic-content mirror
or runtime dependency is added to Qiongli. Default release artifacts remain
Codex/Claude. DeepSeek uses the existing `unknown` route / `other-local` Host
contract. Canonical Skill commit: `839cc17f5e43e4a97ae70fcea0fd0a845eb499d7`.
The embedded pack was regenerated by its existing owner: 434 entries, content
root `c794899fc1d6a6e41421e0353aae5746ff482849d90ac4dc9960e22887cad43e`,
pack `5c6c02cc6964d8d2011323f1dc597250edd150987b9a193261b62372dd167d31`.

Official DeepSeek protocol source:
`477b4f420553e8a52c2fbccc464d7561b239c443`. A temporary install of published
`@deepseek-ai/dsh@0.1.5-rc.3` revealed a real compatibility gap: its headless
runner has no `--json`, although the pinned upstream source does. The adapter
requires that machine-readable protocol; plain stdout cannot establish a session
identity. This lane remains unqualified for real model execution until an
appropriate configured build is available. Do not silently fall back to text.

Observed live checks used isolated synthetic packets, never private research:

| Probe | Observed result |
| --- | --- |
| DeepSeek bundle in temporary 0.1.5-rc.3 profile | Official plugin manager accepted the generated bundle. Real Cordis boot listed and loaded `qiongli` and `no-qiongli`, exposed 32 native Full MCP tools and successfully called `qiongli_config_status`; exit 0, empty stderr. |
| Claude Code | Actual print JSON run completed, exit 0, collected after 15.26 seconds; session `2876e661-b78d-4a0e-8fe1-2fcf93373a09`, dispatch `claude.print`. |
| Antigravity | Actual stream JSON run completed, exit 0, collected after 13.42 seconds; conversation `1d41930a-999c-4e5d-b41d-d6b5fbb91519`, dispatch `antigravity.print`. |

Both actual model replies preserved C-001, D-001, Alpha2024 and mock:alpha and
retained the full-text evidence gap. Exact result hashes and handoff/packet
bindings were recomputed. These are transport observations, not authenticated
independence or evidence that a scholarly claim is supported. Claude stdout
SHA-256: `20d7dc4d6b88699e967a59b112b7037f3e536e219cf85d9ee11c9e1297a3abed`;
Antigravity stdout SHA-256:
`144bc5f97071191736c64a175d3624faab119f41346829b3c2cbd437962f23f6`.
Raw packets/logs are under `/private/tmp/qiongli-external-host-probe`;
the temporary DSH runtime/profile and boot probe are under
`/private/tmp/qiongli-dsh-compat`. Initial DSH boot hit sandbox filesystem-watcher
EMFILE; the identical isolated boot passed outside the sandbox without disabling
watchers or changing product policy. The initial bundle smoke was a dirty-tree
development projection; its receipt is not an exact-source release qualification.

Remaining qualification: supported DeepSeek headless execution, installed-user
Plugin refresh, full registered research journeys with each new Host, real-source
permissions, cancellation/network recovery with those live runtimes, Linux/Windows
execution and release acceptance. New adapters do not implement atomic cross-Host
claims or persistent reconnection. No user Host configuration, saved model,
canonical research data, remote ref or published artifact was changed. CLI-403,
CLI-406 and CLI-407 retain their states/dependencies and historical acceptance.
See `docs/advanced/external-host-coordination.md` for usage and protocol limits.

Focused validation and review for this increment:

- Pinned Rust/Cargo **1.97.0**: **119 execution-library**, **41 CLI**, and **7 MCP**
  tests pass. Two existing tests initially hit sandbox permissions (owned child
  process inspection and the synthetic Zotero loopback listener); each passed
  unchanged outside that sandbox. No assertion or permission check was removed.
- **20 native Plugin packaging tests** pass, including real execution of the
  generated JavaScript Skill provider, three-target projection, duplicate platform,
  source collision and coordinated loader/receipt tampering negatives. Existing
  Codex/Claude default outputs and historical archive checks remain covered.
- **90 workflow/Skill-document tests** and **7 program-ledger tests** pass.
  The final compatibility note changes no executable workflow condition.
- Coordinator review traced prepare/collect callers, common result validation,
  CLI input bounds, native content projection and archive verification. It added
  the missing DeepSeek projection-collision refusal and its regression assertion.
  All-target Clippy identified one collapsible conditional; its equivalent
  let-chain retains the session mismatch check, and the external protocol test
  passed again after that change. No independent review or release acceptance is
  claimed by this coordinator review.
- Native change-boundary and frozen-architecture guards pass. All 249 task states,
  dependencies and evidence and 46 accepted rows remain unchanged. CI routing
  reports broader Desktop/Lite scope for shared/tooling paths; those complete
  release matrices were not run for this local increment.
- Final all-target `qiongli-execution`/`qiongli` Clippy with `-D warnings`, Cargo
  formatting and diff checks pass. The final CLI help wrapping is checked through
  the existing nested-help integration test; no behavior suite is rerun merely
  for the upcoming scoped commit/local fast-forward.

### September 30 observation — installed DeepSeek Harness 0.2.0-rc.2

Maintainer request: check whether the newly installed Harness supports Qiongli
Plugin installation. The actual `/usr/local/bin/dsh` resolves into the installed
DeepSeek Harness App and reports `0.2.0-rc.2`. A fresh macOS ARM64 development
bundle was built with pinned Rust/Cargo 1.97.0 from clean source
`c2e10fb18fe58d9edc76a04588fefb12831cd859`, using the existing content exporter,
projector and archive verifier. No runtime or canonical Skill change was needed.

- Official `dsh plugin --profile qiongli-probe add <bundle-directory>` passed,
  exit 0, using the App's pnpm 11.7.0. Repeat installation left `package.json`
  byte-identical, with one bundle registration and one MCP row.
- The same bundle installed into the isolated Web-based `qiongli-test` profile.
  Real Cordis activation loaded `qiongli` and `no-qiongli`, exposed 32 Full MCP
  tools, called `qiongli_config_status` successfully (`status: ok`), and exited 0.
  Its only stderr diagnostic was Node's `DEP0180` deprecation warning.
- Official removal from `qiongli-probe` passed and retained its original base
  and headless bundles. The isolated Web test profile is retained for inspection.
- `dsh headless --help` now exposes `--json`. This removes the observed missing
  flag in 0.1.5-rc.3; no actual headless model run or collector result is claimed.

The initial headless bootstrap had no task and correctly rejected it; this was
a probe-profile mismatch. The initial sandbox Web bootstrap timed out; the same
isolated bootstrap passed outside the sandbox. These failed attempts do not
establish a Plugin incompatibility or a diagnosed Harness runtime defect.
Installer/help commands emitted an Electron codesign diagnostic while exiting 0.

The temporary root is `/private/tmp/qiongli-dsh-020-vtekjgcb`, with separate
`DSH_HOME` and `QIONGLI_CONFIG_HOME`; `result.json` summarizes results
and limits alongside the retained probe and stdout/stderr. Archive SHA-256:
`242d569ce41b449bc6c6123345056b77c9a412e42d821454135c6d6ed9f138e8`;
binary SHA-256:
`d7365ad81678067950e1edec32cbd94522a09349bbdc879c881c76f7dfe4a10c`.
The embedded pack remains `5c6c02cc6964d8d2011323f1dc597250edd150987b9a193261b62372dd167d31`.

This is installed-runtime Plugin compatibility on one target, with a development
binary. The user's profiles/models, research data and published artifacts were
unchanged. Actual headless execution, installed-user activation, research
journeys, other targets and release acceptance remain open. CLI-403/CLI-406
states, dependencies and accepted evidence remain unchanged.

Recording checks: all seven program-ledger tests pass, the generated index is
current, and the diff check passes. A direct before/after comparison preserves
all 249 task states/dependencies/evidence and every historical accepted row;
only CLI-403/CLI-406 observation notes and their update dates changed.

### September 30 increment — matching Codex and DSH workflow entries

Maintainer follow-up: preserve Codex's multiple workflow entrances in DSH. The
two-entry DSH catalog was reproduced by the packaging test (`2 != 22`). Source
`92f1f7d4e48f13cca4c0ba7e810bdac60b44d0a3` reuses the existing workflow wrapper
generator and registers all 20 workflow wrappers alongside `qiongli` and
`no-qiongli`. Names, descriptions and entry bytes match Codex; each directory
is its resource base, and both model/user invocation flags are enabled. Academic
content and the embedded pack are unchanged. New DSH receipt schema 4 verifies
the complete wrappers/catalog; schema 3 retains the old two-entry projection.
Codex/Claude defaults and receipt schemas remain unchanged.

A fresh clean-source macOS ARM64 bundle, built with Rust/Cargo 1.97.0, was
installed through the actual DSH 0.2.0-rc.2 manager into a temporary Web profile.
The real registry's complete snapshot exposed all 22 expected entries with the
winning provider `qiongli`. Every entry loaded with exact packaged bytes,
canonical descriptions, both invocation flags and valid shared resource paths.
The actual `sessionSkillCatalog.list` service used by the browser returned all
22 entries for an isolated cold coding Session, without activating an Agent.
Full MCP still exposed 32 tools and config status returned `ok`; the probe
exited 0, with only Node's DEP0180 warning. `--no-open` kept the browser closed.
This checks the directory API and loader, not visual browser interaction or a
model's task execution.

Temporary root: `/private/tmp/qiongli-dsh-multi-77efi1o_`, with isolated
`DSH_HOME` and `QIONGLI_CONFIG_HOME`. Archive SHA-256:
`65f537a47a7d4ab555bac6a1c0dcb69d14d897166a630d9cc788ce2f5e366952`;
binary SHA-256:
`119ab9a27aa778e2310fdfebd2bc37c5d5d2f42aef79dce1936ab0937240463d`.
All 22 entry files also match the fresh Codex projection byte-for-byte.

All 21 packaging tests pass, covering the real generated provider, parity,
three-target projection, invalid workflow metadata, missing/changed wrappers
and catalogs with rewritten receipts, and historical schema-3 projection.
The actual previous two-entry archive retains its recorded SHA-256 and passes
the current verifier. Seven program-ledger tests and the generated-index check
pass; frozen/native boundary and diff checks pass. Broader Desktop/Lite/platform
matrices are not rerun for this opt-in projector change. Existing user profiles,
models and research data are unchanged; installing the new bundle into a user
profile and research/model execution remain separate checks. No program state,
dependency or historical accepted evidence is promoted.

### September 30 release increment — stable 2.1.0 and npm DSH installation

The maintainer selected 2.1.0 stable and authorized publication after consolidating
post-2.0.0 changes, with no follow-up tracking after the release command is
accepted. `tooling/release/v2.1.0.md` records user-facing changes, the v2 source
status cutoff, installation, practical limits and rollback. No announcement or
external Marketplace promotion is part of this release request.

Canonical version source `17c5df01` sets native/content versions to 2.1.0. The
supported content-lock generator retains 434 resources with content root
`e61a1075b347db90a8fc8803b14198998947241d1979708fcdea03f105d8f94c` and pack
`4c976864755fae55d1b4c4aeea0572194ba91256338a68cf620270919311fc56`.
Implementation `c10c4b7086ef2220a6dd1ba347fd0270ff1f5f28` was reviewed by the
coordinator and integrated locally into 2.x. This is self-review, not independent
review or accepted program evidence.

The existing `qiongli` npm package now exports `dsh/index.mjs` and
`dsh/cordis.patch.yml`. It reuses the shared 22-entry generator and selects the
already bundled executable for each of the three platforms; Full MCP remains
32 tools. There is no separate npm name, runtime download or publisher. The
combined packet verifier reconstructs the canonical pack and projection and
binds them to all three executables and target-native install observations.
Historical pre-2.1 packets and earlier DSH receipt versions remain verifiable.
`qiongli install plugin --target deepseek`, its aliases and terminal choice 4
show the official DSH profile-specific manager commands. They do not write a
Host configuration or silently run a package manager.

Fresh checks at the implementation source:

- 60 focused distribution/version/ledger checks pass; the final additional
  missing-DSH-evidence negative also passes. The 11 focused native-boundary and
  release-asset checks pass, as do release-note versions and the diff check.
- `./scripts/release_ready.sh --cli-github --version 2.1.0 --skip-bump
  --staging-dir /private/tmp/qiongli-v210-dsh-release` passes on macOS ARM64 with
  Rust 1.97.0: format, all-target workspace CLI Clippy, 42 release CLI tests,
  seven release MCP tests, empty-PATH archive smoke, clean npm/wheel installs,
  installed DSH provider dispatch and both standalone Plugin checks.
- The debug suite passes 42 CLI and six MCP tests; the remaining Zotero loopback
  fixture was denied by the sandbox at `TcpListener::bind`. Its focused rerun
  outside the sandbox passes. No assertion or permission negative was removed.
- All five candidate artifact digests/sizes match the release manifest. npm
  SHA-256 is `eba6ef541b016c0e886af9f0495710e167a09d2f812291029c9a05c9b36d9e57`;
  native binary SHA-256 is
  `33a5f715dc7f70447d4b7c56e6324ca93e937af9b8eaf3b51e48966f0f27f764`.

Actual installed DSH 0.2.0-rc.2 / embedded pnpm 11.7.0 observations are retained
in `/private/tmp/qiongli-v210-dsh-install`. The official manager installs the
local npm archive and `qiongli@2.1.0` from a loopback npm registry serving those
exact bytes. The registry probe supplies a fresh publication timestamp and an
empty temporary store; metadata and tarball are both fetched. pnpm's default
explicit-add behavior records a package-age exclusion in the temporary profile.
This is registry-protocol compatibility, not public npm publication evidence.

Fresh profiles use the official `--from-default-profile web` template. Both
local/archive and registry/name installations boot successfully, with all 22
winning `qiongli` entries, exact source bytes/resource paths, both invocation
flags, 22 browser-facing Session catalog entries, 32 tools and config status
`ok`. No Agent/model is activated; `--no-open` keeps the browser closed. Only
Node DEP0180 is logged. Earlier base-only probe profiles timed out because their
browser catalog service was absent; adding the Web app only as a patch still
lacked its launcher-owned webServer. Initializing the official Web template
resolved that test setup issue. One registry probe expected a tarball fetch
although pnpm reused a cached copy; the empty-store retry resolves it.

These observations do not attest installed-user profiles, visual browser
interaction, DeepSeek model execution, other native target runtimes or two-Host
research acceptance. User research and saved model settings remain unchanged.
The existing tag-bound release automation owns fresh three-platform builds,
combined installs, verified GitHub assets and npm/PyPI/Cargo publication. Stable
source must equal remote main; no release checks, credentials or protection are
bypassed. Protected remote 2.x is left unchanged, and the unpublished local
2.0.1 main draft is retained. The selected integrated 2.x source can advance
remote main normally from the released 2.0.0 ancestor.

After final record checks, freeze the source and immutable v2.1.0 tag, then submit
`gh workflow run release-automation.yml --ref v2.1.0 -f mode=post -f tag=v2.1.0`.
Stop local tracking upon accepted dispatch; no completed publication is inferred.
Program task states, dependencies and historical accepted evidence remain
unchanged. Real Host/library/collaboration qualification remains the next
independent development scope.


### September 30 installer fix — multiple Hosts and confirmed DSH npm execution

The maintainer reported that the installed 2.1.0 guide accepted only one menu
choice and that DeepSeek choice 4 merely displayed another command to run. The
local `fix/multi-host-plugin-install` increment fixes both owners, based on
integrated 2.x source `09f669d5`. The published 2.1.0 tag remains immutable; this
increment is local development, with no push, new release or CI tracking.

One Host table and parser serves the terminal menu and named `--target` option.
Comma/space lists retain input order and deduplicate; `all` expands the supported
Hosts, while `3`/`both` retain Codex+Claude. A shared options validator refuses a
single destination for multiple Hosts and DSH source/hook options before writes.
Each selected Host retains its own installer and confirmation. Cancellation or
failure stops remaining Hosts and preserves completed steps.

The terminal-only DSH adapter discovers the official CLI, requires 0.2+, selects
an existing Desktop profile by default or a Web CLI profile otherwise, and shows
exact commands before trust confirmation. New CLI profiles use the official Web
template within that plan. Executable and profile file digests are revalidated
before execution. The existing bounded Host launcher passes captured DSH_HOME
only to the child. The official manager installs the matching exact npm version
with the official registry and disabled install scripts; profile bundle, package
version and content receipt checks precede the completion message. Conflicting
old dsh-qiongli-* bundles refuse without automatic deletion. Public managed App
target enums, schemas and research write owners are unchanged.

Focused validation and coordinator self-review:

- Format and all-target CLI Clippy pass. The 14 focused release unit checks cover
  multi-selection, CLI aliases/options, empty/partial/negative confirmation,
  stale profile state, unsafe paths/symlinks, old-bundle conflict, command failure
  and missing/mismatched registration evidence; existing Host tests remain green.
- The 42-case release CLI run passed 41 checks; the updated no-terminal assertion
  initially retained --text and encountered the existing output-option refusal.
  Removing that fixture flag checks terminal refusal directly, and its rerun
  passes. The final help and no-terminal checks pass after the last shared parser/
  help changes. Seven release stdio MCP tests pass outside the sandbox, including
  the existing loopback fixtures. No permission or compatibility negative is removed.
- The installed macOS ARM64 DSH 0.2.0-rc.2 / embedded pnpm 11.7.0 receives the
  actual new terminal flow in an isolated home/profile. Choice `4,1` followed by
  refusal runs no installation command, creates no DSH profile and stops before
  Codex. Confirmed installation initializes a Web profile and installs
  `qiongli@2.1.0` from https://registry.npmjs.org, verifies registration and exits 0.
  Logs/result are under
  `packages/qiongli-native/target/multi-host-live-probe/tmp4opoy4rj` (ignored local
  test artifacts). The first PTY collector failed while collecting process exit
  after installation had finished; waiting for exit after PTY hangup fixes that
  test collector. The corrected run exits 0.
- Booting only that isolated profile with --no-open loads all 22 winning Skill
  entries with exact bytes/resource paths, both invocation flags, 22 browser
  Session catalog entries, 32 Full MCP tools and config status ok; it exits 0.
  No Agent/model is activated. Only the existing Node DEP0180 warning is logged.

This is coordinator self-review and local Host observation, not independent
review, installed-user-profile qualification or program acceptance. Real user
profiles/model settings and canonical research data remain untouched. All 249
program task states/dependencies and 46 historical accepted rows remain unchanged;
only the CLI-403 observation note is extended. Future supported Hosts add one
menu-table entry and their installation branch; selection and all need no new
parser. Fresh installed-user/model/two-Host research qualification remains the
next independent scope. A separately authorized patch release is needed to ship
this fix to existing registry users.


### September 30 — install-time Skill language (local development)

The maintainer selected install-time language choice and explicitly required
humanizer-quality descriptions. Canonical copy at `1db95c63` adds one shared
22-entry English/Chinese catalog and normalizes default descriptions to clear
English. Coordinator self-review applies the humanizer English/Chinese guidance:
faithful scope, concrete verbs, natural phrasing and no inflated promises.
Descriptions and the main Codex display name/short description/default prompt
are localized; bodies, resource paths, aliases and invocation names remain stable.

The terminal guide asks Auto/中文/English once for the selected Hosts or standalone
Skills. `--language auto|zh|en` uses the same validation and skips that prompt.
Auto captures locale environment settings, then macOS/Windows OS language through
the existing bounded launcher, with English fallback. It does not claim a universal
client-language API. Resolved language participates in plan/digest/receipt checks;
source and managed Skills updates preserve saved language when omitted, including
compensation. No workflow-variant, model, research-write or Host trust owner changes.
DSH still installs through its official manager, then atomically saves the chosen
Qiongli profile preference under digest revalidation and an exclusive preference
lock. The provider reads public profileContext and applies the shared translation
to both summaries and loaded frontmatter. Historical catalog-free packets retain
their exact provider/projection; optional-field-free receipt readers remain.

Validation and limits:

- 22 native Marketplace/npm projection checks and six Skill-document checks pass.
  Generated DSH modules exercise zh/en on all three npm dispatch targets, profile
  paths containing spaces/#, malformed preferences and invalid language refusal.
- Twelve content materialization/variant checks plus the locale parser check pass,
  preserving body bytes, exact receipts and path/drift/permission negatives. Five
  platform bundle checks pass, including legacy identities and bounded wrappers.
- Initial source schema check correctly failed until regenerated with the existing
  contract example. The broad 236-case CLI library run observed 232 passes, one
  ignored capacity check and three failures: a language-plan digest was not
  recomputed, a test retained an invalid preference, and the old all-Hosts fixture
  expected two. The digest/result behavior and fixtures were corrected; the 40
  affected CLI checks and 14 Plugin checks pass. The final DSH test passes after
  adding the preference lock. Clippy and format pass; no negative checks removed.
- Actual local exports from the built CLI verify Chinese then omitted-language
  preservation then English for Codex (22) and Claude (2). Standalone Skills switch
  from zh to en and report updated. Official macOS ARM64 DSH 0.2.0-rc.2 installs a
  locally staged npm archive in isolated HOME/profile and boots both languages:
  all 22 winning entries, loaded descriptions, unchanged bodies/resource paths,
  22 Session catalog entries, 32 Full MCP tools and config status ok; exit 0 and no
  model run. Logs/result are in ignored
  `packages/qiongli-native/target/skill-language-live-probe/tmpbchetojk`.
  Binary SHA-256: `60d704d83b776560c7fed10828233b2236cee3735438ab0346c9b850afd77ccb`;
  pack SHA-256: `dce758ef109900b793abf9397970be7028da5273aa5a89570373284dbc821e92`.
  The staging source is the content commit plus this working-tree implementation;
  this is a development observation, not an immutable release qualification.
- A real PTY with `LANG=zh_CN.UTF-8` shows Auto(zh), and language cancellation exits
  0 without source/Skills/DSH writes. Its log is in the same ignored probe parent,
  `tmp_5_fvehn`. A mistyped ledger test module produced an import error; the actual
  program-roadmap check initially found its generated index stale; regeneration
  through the existing owner makes all seven checks pass at integration.

This is local coordinator self-review, not independent or installed-user-session
acceptance. No remote tracking, push, version bump or publication is performed.
Existing 249 task states/dependencies and 46 accepted rows remain unchanged. OS
language reading on Windows/Linux and installed-user client refresh remain external
qualification gaps. A new authorized release is required to distribute this work;
client UI language changes alone do not rewrite installed Skills. DSH reloads its
saved language when the profile/app restarts; creating a chat alone is insufficient.


### September 30 stable patch — 2.1.1 release preparation

The maintainer authorizes version consolidation, reviewed local integration,
normal main push, a new immutable stable tag and submission to the existing
release automation. Stop local tracking after accepted submission; do not infer
completed publication or create follow-up automation. No announcement,
Marketplace promotion, user-profile/model change or research-data access is scoped.

Remote main and v2.1.0 identify `09f669d55b2aa5474cd5460d75f745d2014795e1`.
The prior release run 36709911779 completed successfully and GitHub exposes
v2.1.0. The unpublished local main transition draft diverges; retain it rather
than rewriting it. The release source advances remote main from its existing
ancestor through reviewed 2.x changes. Tag/npm/PyPI/Cargo lookups found 2.1.1
unoccupied. Never overwrite published tags or assets.

Source changes `81cf5839`, `1db95c63`, `bc947b5e` supply multi-Host installation,
concise bilingual Skill metadata and shared auto/zh/en selection. Canonical
version synchronization at `6fe46a77` uses the existing owner. Its generated
content lock has 435 resources, content root
`f69e6eb618642c5df6cb7aa0d57ebd562f137759f0b32b9532f8c6c94e9b075e`
and pack `30065976c8dd5077ea94bf0d345586243f3c70261ab953667d9568ff13df304d`.
`tooling/release/v2.1.1.md` records installation, language persistence, DSH restart,
channels, limits and rollback. No generated mirror was hand-edited.

Preparation checks: 30 version/registry/assets/CLI-release/publisher/program
checks pass; seven program checks pass after regenerating the existing index.
Tag/version verification passes. The existing native registry generator creates
all nine Cargo source archives in `/private/tmp/qiongli-v211-cargo-source`;
archive creation is not Cargo install verification or publication. Coordinator
self-review preserves version projection and release authorization negatives.
The 249 task states/dependencies and 46 accepted rows remain unchanged.

The clean integrated source must next pass
`./scripts/release_ready.sh --cli-github --version 2.1.1 --staging-dir
/private/tmp/qiongli-v211-release-qualified` on macOS ARM64 before remote publication
submission. This packet owner builds and checks native CLI, npm/wheel installs,
DSH projection and standalone Plugins. Only its successful result authorizes a
readiness claim for that target. Existing tag-bound automation must freshly
qualify all three targets and combined npm/assets before publishing GitHub,
npm latest, stable PyPI and Cargo. Stable tag must equal frozen remote main HEAD.
No check or protection is bypassed; no protected 2.x push is needed.

Earlier focused implementation/isolated DSH observations remain scoped to their
source. Do not claim a final full 236-case library rerun. Windows/Linux OS locale,
real client refresh, installed-user/model collaboration and broader research
acceptance remain open. The next independent development increment is real
Host/library qualification, not duplicate release monitoring.


Release-gate correction: the first clean-source macOS attempt at `723f1770`
passes fmt, Clippy, 42 release CLI checks, seven MCP checks and archive smoke,
but fails the installed DSH probe: it compared localized loaded descriptions
against unmodified frontmatter. Reproduction isolates the qiongli description's
version prefix; bodies remain unchanged. The existing shared registry checker
now validates auto/zh/en descriptions against the packaged canonical catalog
and permits exactly that frontmatter substitution while comparing every other
byte, entry order, invocation flags and path-refusal behavior. Catalog-free
historical packages retain exact-byte comparison. Its existing npm projection
test now runs the same installed-provider probe. All 34 affected checks pass,
and the same installed 2.1.1 packet passes the corrected probe with 22 Skills
and 32 Full MCP tools. Final qualification must run on the newly frozen source
at the fresh staging directory above; failed attempt evidence is not reused as
qualification. Public schema policy and frozen 1.x migration guards pass.

An optional retained Python experience-record check against the checkout finds
old ignored trace records missing required inputs; it is not a native CLI release
gate and no records are changed. This failed diagnostic is not a passing schema
claim. Frozen public schema validation is the applicable compatibility check.


### October 1 — discipline guidance and lifecycle reinforcement (local development)

The maintainer requests more precise discipline/common-method content and another
pass over the full research lifecycle on the 2.1.1 baseline. Canonical source
`cf42c26fff29de3b640ef36f8868e99a75038ba8` adds one conditional index and seven
field guides: economics/finance/accounting, business/society/policy,
education/psychology, health/biomedical, computing/engineering,
environment/spatial, and humanities/language/law. Reuse existing profiles and
stage artifacts; the guides add no Host entries, subject IDs, model choices,
permission owners or research-write paths. Focused packages disclose missing
profiles rather than pretending to load them. This is local development, not a
release or installation request.

All A–M references now carry more concrete source, unit, measurement, method,
interpretation and handoff decisions. Selected existing design, statistics,
reporting and profile cards replace universal numerical heuristics and method
rankings with design-specific reasoning. Stage G owns applicable standard/version
selection; the general reporting card stops copying stale checklist tables.
PRISMA distinguishes records, reports, studies and synthesis subsets, reports
actual registration status, and fixes the working template's expanded-item total
from 40 to 42. Reporting completeness does not establish study quality. Existing
formal gates, method contracts, approval/CAS and user-configured models remain.
English/Chinese public Skills guides are regenerated through their existing owner.

Validation and review:

- Across affected batches, 59 distinct Python checks pass in
  `test_skill_resource_links`, `test_skill_structure_lint`,
  `test_skill_contract_alignment`, `test_domain_method_packs`,
  `test_skill_doc_generation`, `test_subject_catalog` and
  `test_native_marketplace_plugins`. The first batch found an obsolete proposal
  heading assertion; its assertions now track the existing unchanged
  proposal-writer route and retain the opening-report/artifact checks. Final
  source link/structure/contract/document checks pass after the PRISMA and routing
  edits; the last generated-copy edit passes all six document checks.
- Capability-contract validation and the skill-creator root validator pass.
  All domain YAML profiles parse. A direct count reconciles the PRISMA template's
  27 numbered items, 42 expanded rows and section totals. Isolated materialization
  for core, economics, business, finance and accounting retains all seven guides
  byte-for-byte with no missing internal links.
- The 48 `qiongli-content` tests pass, retaining path/symlink, authorization,
  drift and data-preservation negatives. The final product `embedded_pack` test
  passes after regenerating the native lock through `update_qiongli_core_lock`.
  The 443-resource pack has content root
  `5420d53526eb2246abea97fe11f63b45d4ba066d2af0f470df9db92e7b087ab0`
  and pack SHA-256
  `9cad4dac8aa825524bc509f48cf631ee8a689264893b886c1cfe38091ff2961a`.
- The built macOS ARM64 debug CLI and existing `export_marketplace_content`
  example export the verified final content into
  `/private/tmp/qiongli-skill-depth-cf42c26f-content`.
  `native_marketplace_plugins.read_content` reconstructs its native pack digest;
  `project` produces Codex/Claude/DeepSeek content with 22/2/22 Skill entries.
  All three projections preserve the index and seven guides exactly and pass
  the resource-link audit. Binary SHA-256:
  `8510dac1e5519e50eff3af0991685f92f582f4cbdcce87b49580224b89f0859d`.
  This is local packaging evidence, not installation or release qualification.
- All seven program-roadmap checks and the generated-index freshness check pass.
  A before/after ledger comparison preserves every state, dependency and accepted
  row. Coordinator diff review and `git diff --check` pass; integration uses the
  required frozen-source boundary guard and local fast-forward merge.

The actual configured Host sub-agent `/root/skill_forward_trial`, with no model
override, performed read-only synthetic forward trials from the candidate sources
while editing. It returned answers and the paths read to the coordinator; it did
not write canonical research files or use private studies. The coordinator found
all nine cases consistent with the intended boundaries:

| Synthetic input | Observed decision |
|---|---|
| Accounting DID with announcement/effective-date ambiguity and a financing control | Preserve timing ambiguity; assess post-treatment control risk and identification |
| Eight organizational excerpts for reflexive thematic analysis | Do not invent participant count, coding agreement or saturation |
| Two schools, 400 pupils, one intervention school | Retain two assignment units; no causal/significance claim from pupil count |
| Repeated patient records split across training/test, AUC 0.91 | Identify leakage and missing external validation; qualify prediction reporting |
| Five correlated CV folds and a closed LLM | Reject independent-fold inference and unsupported training-contamination assurances |
| Neighboring pixels used to claim next-year/new-region accuracy | Require validation for the spatial/temporal generalization target |
| Two incomplete 1910 legal transcriptions | Preserve provenance, historical scope and transcription uncertainty |
| Mixed-methods thesis with r = 0.32 and contradictory interviews | Integrate divergence; do not invent causality, mediation or supervisor approval |
| PRISMA: 100 records → 80 screened → 30 sought → 25 assessed → 18 reports → 14 studies → 10 pooled | Reconcile each unit; no forced equal counts or invented registration requirement |

The ninth case followed the PRISMA correction. The final root review-route label
and generated documentation were clarified after the earlier trials; those small
edits received coordinator review and the affected checks, not a fresh behavioral
trial. These observations are bounded sub-agent trials assessed by the coordinator,
not expert, cross-model, installed-Host or program acceptance. No private research
access, push, version bump, publication or installed-plugin update occurred.
The next increment is qualification against maintainer-selected real study cases
and relevant domain review, then installed-Host observations when authorized.
All 249 task states/dependencies and 46 historical accepted rows remain unchanged;
CLI-405 records only this source progress and its remaining qualification limits.

### October 1 — evidence-bound C/E/F decisions (local development)

The maintainer selects the next increment around three questions: what must be
decided now, what evidence settles it, and what can proceed under which conditions.
Canonical source `8289e3f0effdc4fbe3a2cb20a29c0b4892ff52f3` places that contract in
the shared academic output rubric and specializes it for study design, synthesis
and results/writing. Models retain choice of method, tools, sequence and form.
Existing stage artifacts, decision/claim IDs and handoffs carry the source basis,
limits and resumption conditions; changed sources reopen only dependent work.
No new decision service, state machine, interview or formal gate is introduced.

Synthesis and interpretation cards now guide supported execution and partial
completion instead of imposing a fixed interview, default random-effects model,
universal qualitative procedure, interpretation ladder or citation quota. Missing
variance, appraisal or practical benchmarks remain specific unmet requirements.
The effect interpretation card also uses the existing F3 `EffectInterpretation`
type and `manuscript/effect_interpretation.md` path; a regression check covers both
F3 cards against registry types and task outputs. Legacy files remain readable
prior material, never silently renamed. The handoff template gains three headings
already required by its owner. Preview/approval/CAS, model settings and the native
runtime are unchanged. The source change removes 218 net lines.

Focused checks and delivery:

- 52 distinct Python tests pass across resource links, structure lint, skill
  contract alignment, stage handoff, cross-platform routing/grill and native
  Marketplace projection modules; overlapping reruns are not additional tests.
  Existing projection permission, path and drift negatives remain covered.
  Direct canonical structure validation passes 649 checks with no errors and
  eight advisory warnings. Capability-contract and skill-creator root validation
  pass. The final diff was reviewed against its existing owners.
- Regenerated the native lock with `update_qiongli_core_lock`; the actual product
  `embedded_pack` test passes. The 443-resource pack has content root
  `b59fbb053182b20dcffc75c0d7f8fecd208d8966b603cdcf44ec55673e5b8eab`
  and pack SHA-256
  `317e9f315c6ca1e0fd4df8ad49ce955652003401c7d21827c650ccbf90edb2d1`.
- The macOS ARM64 debug CLI and `export_marketplace_content` export that pack to
  `/private/tmp/qiongli-decision-content-8289e3f0`. The existing projection owner
  verifies its digest and produces Codex/Claude/DeepSeek content with 22/2/22 Skill
  entries. Each preserves all 15 changed content resources exactly and has no
  missing resource links. Binary SHA-256:
  `61724d214d0b4f9059677ff99032aac2e13377120b109cc9fd3c41971b2ba103`.
- Initial system-Python test imports failed because PyYAML was unavailable;
  rerunning with the existing `.venv/bin/python` passed. Initial native commands
  could not resolve the registry and offline cache lacked locked dependencies;
  an authorized locked fetch/build recovered, followed by successful offline
  export/build. An existing platform atomic deprecation warning remains outside
  this content change. No dependency or lockfile version was changed.

Two actual configured Host subagents, `/root/decision_quant_trial` and
`/root/decision_qual_trial`, independently executed bounded fictional tasks with
no model override. They read frozen candidate guidance and their own authorized
inputs, then wrote only isolated candidate files. Each reports 19 guidance paths
actually read. The coordinator inspects the substantive outputs and calculations;
this is independent task execution with coordinator assessment, not an independent
expert review, cross-model benchmark or installed-Host acceptance.

The fixed denominator is two complete C/E/F candidate cases, with a further
changed-source checkpoint for the quantitative case. Inputs, criteria and guidance
remain under `/private/tmp/qiongli-decision-trial-53k4h9oi`. Criteria and the R2
correction were fixed before execution; the correction was withheld until R1
completed. Input/criteria manifest SHA-256:
`10b303ca8141705977e0e8308b74b5ae95ef26dfa0127a31b9e72e997021dd76`;
282-file guidance manifest SHA-256:
`b380a50f138411ee8e5b8604376cab1c610a44f2c4ae87967ed977225fa96329`.
The coordinator retained all 19 quantitative R1 files before releasing R2;
history manifest SHA-256:
`fe69d02ab54cfa4d00d019609b5e92db5e9154a0b61c4c32d56eda823a6893b2`.
These temporary artifacts support local observations, not durable accepted evidence.

| Checkpoint | Coordinator-observed result |
|---|---|
| Quantitative R1 | Actual design, analysis, synthesis, ledger, Chinese results and handoff; R01/R01b counted as one study. Executed common-effect O1 MD 2.615385 (95% CI 0.984596–4.246173), with assumptions and small-study sensitivity limits. O2 stays a 3.0-point estimate without invented error or denominator; attrition and practical importance stay unresolved. |
| Qualitative Q1 | Actual design, analysis, source-linked matrix/codebook, ledger, interpretive Chinese paragraph and handoff. Four quotations are preserved; Q01's two excerpts share P01, Q02 identities remain unknown, and Q03 is abstract-only. Supported help/expression tension is interpreted without inventing efficacy, prevalence, saturation, coding agreement or formal confidence. |
| Quantitative R2 continuation | The released correction changes only S02/O1 from +2.0 to -1.0. Re-executed inverse-variance MD is 0.538462 (95% CI -1.092327–2.169250); DL sensitivity is 1.375 (-3.518781–6.268781). DEC-003 retains the estimator but narrows interpretation to these two fixed studies; current synthesis, CLM-004, Chinese prose and handoff all reflect the opposing estimates without claiming no effect/equivalence. DEC-001/002, unaffected claim rows and O2/attrition gaps remain; the old decision log and all 19 R1 files are intact. |

All three checkpoints meet the fixed bounded criteria on coordinator inspection.
Separate numeric assertions reconcile both quantitative outputs, unchanged claim
rows and current source anchors; quotation coverage and input/guidance/history
hashes also pass. The candidates contain 19 R1 files, 20 R2 files and 13 qualitative
files. The extra R2 file records the read-only history check, not a new workflow
contract. No trial-triggered source repair or discarded case is hidden. The
quantitative agent recovered one nonexistent guidance-path read; during R2 a
login-shell startup attempted a mise cache write, which the sandbox denied, and
subsequent commands disabled login startup. These do not establish research or
external-write acceptance. Exact execution model IDs were not exposed; none is
invented or configured by this increment.

All seven program-roadmap tests and index freshness pass. A before/after ledger
comparison retains all 249 task states/dependencies, all 46 accepted rows, and
every other row unchanged; only CLI-405 progress text changes. `git diff --check`
passes. Integration follows the required frozen-source guard and local fast-forward
merge without rerunning unchanged checks solely for the commit/merge.

The next increment remains maintainer-selected real-study and domain review, then
installed-Host qualification when authorized. These two small fictional tasks do
not establish broad discipline coverage, formal stage acceptance or production
research quality. No private research access, push, release/version bump,
publication or installed-plugin update occurred.


## October 2 — selected public fulltext and Host search increment

The maintainer authorized the proposed discovery → document retrieval → anchored
reading path, including additional search channels and the active Agent's native
search. Implementation `fb3d6d76` preserves the five existing providers rather
than introducing another configurable search service: OpenAlex/Semantic metadata,
Crossref links, PubMed-reported PMC identifiers and arXiv PDF links retain optional
abstracts, external identifiers, candidate format/version/license and deduplication
provenance. Numeric reported IDs, including Semantic CorpusId and OpenAlex MAG,
are normalized to strings; they are not invented identifiers or access proof.

`qiongli_literature_read_fulltext` is the shared native Lite/Full and standalone
Rust Lite reader. It fetches public HTTPS PDF/TEI/JATS, reports source digest,
retrieval time, page/section/segment anchors, pagination, identity status and
parser warnings. Nonzero offsets require the prior digest; explicit refresh,
changed bytes, wrong DOI, abstract-only XML and multi-paper XML have distinct
outcomes. Public reads need no configured provider. The exact OpenAlex content
endpoint may use the existing key; this increment did not access that key.

Network boundaries reject credentials, private destinations and unsafe redirects,
pin public DNS results per hop, disable proxies and bound download/decoded gzip.
The session cache holds at most eight documents, with emitted-text, segment and
PDF-page limits. The pinned pure-Rust `pdf-extract` dependency avoids a separately
installed reader; existing quick-xml and flate2 handle XML and gzip. PDF internal
stream/font decompression remains library-owned: this is not a hard process
memory/time sandbox. HTML/OCR, private attachments and authenticated publisher
browsing remain available-Host responsibilities. No canonical research write,
Graph ingestion, new research store, model selection or registration was added.

The fulltext Skill and B2 route now request actual document reading and preserve
identity/version, evidence limits and existing manifest/approval/CAS owners.
Existing hybrid/native-only plans keep native search execution in the Host;
provider search, Host-discovered URLs, parsed text and inspected passages remain
separate. Fulltext availability does not decide review eligibility. The explicit
native extension brings Lite/Full to 15/33 tools while preserving the frozen
CTR-201/Python v2 inventories. CLI help reflects the current native inventory.

Focused verification and repaired failures:

- Runtime checks cover 62 unit, six bibliography and ten Lite MCP cases. Changed
  parser/provider cases were rerun after repair; unchanged cases were reused.
  Runtime all-target Clippy passes. Tests include SSRF URL/IP rejection, credential
  redaction, continuation/digest/refresh, wrong DOI, metadata-only/HTML/multi-paper
  responses, bounded extraction, PDF page anchors and TEI/JATS section text.
- Standalone Rust Lite passes 30 focused checks and offline locked all-target
  checking. One cache/reader owner is reused; wrappers do not duplicate parsing.
- Canonical contract, resource links, literature routing and native Marketplace
  projection tests pass (49 tests). The capability validator passes while retaining
  missing/unknown-tool and schema-drift negatives. Two previous paper-read literal
  assertions now follow its explicit shared routing reference instead of requiring
  duplicated instructions; no ownership requirement was removed.
- Product embedded-pack, seven copied-binary MCP and one guided-install/local
  MCP check pass. An initial hardcoded 14-tool assertion was corrected to the
  registry length. The existing platform atomic deprecation warning remains.
- The initial sandboxed Cargo fetch lacked network access; authorized fetch
  recovered. Some existing loopback fixtures required authorized escalation.
  An initial pytest invocation failed because the existing environment uses
  unittest; tests then ran with `.venv/bin/python`. An outdated MAG omission
  assertion failed after numeric-ID preservation and was corrected and rerun.

The actual macOS ARM64 native export contains 445 resources, content source
`fb3d6d7641461d35aac168713018ab0987440172`, content root
`98602d3d7ebcefa494ba1ed746a1292eab24c0b79d0428faa426110db4af3917`, and pack
`dea7da3fa371dd30eeb330b40ba76f69a3fe1e3ec88834c8146f2a5232dbbe0f`.
Existing Codex/Claude/DeepSeek projectors produce 22/2/22 Skill entries and preserve
all seven changed exported content resources byte-for-byte. Development binary
SHA-256 is `f67c4a4a28566248818e75632eb107a0a0f16504e3000369633a37d3b3757e65`.
Export directory: `/private/tmp/qiongli-fulltext-content-fb3d6d76`.

The existing distribution checks now recognize only coherent legacy 14/32 or
named-fulltext 15/33 inventories, require agreement between local and stdio
profiles, and bind DSH/Marketplace counts to the same archive smoke receipt.
Unknown additions, duplicate names, mixed profiles/targets and mismatched hashes
remain failures; old immutable packages retain their original inventory.
All 33 focused distribution/release-script tests pass, including legacy alpha and
2.0.1 packet checks, the named reader extension and mixed-count/hash negatives.
Actual temporary Codex/Claude macOS ARM64 native archives pass extracted,
empty-PATH `check_plugins` with 15 Lite tools and the same embedded pack; their
binary checks also exercise 33 Full tools. Archives are development artifacts
under `/private/tmp/qiongli-fulltext-plugins-fb3d6d76`, not published v2.1.1 assets.

Public-source probes used no provider credentials or private library:

| Source | Actual observation |
|---|---|
| `https://arxiv.org/pdf/1706.03762` | PDF parsed to 27 segments; the first two returned excerpts have page-1 anchors. The second request used the digest and session cache. DOI identity was not checked; no complete-reading claim. Source SHA-256 `bdfaa68d8984f0dc02beaca527b76f207d99b666d31d1da728ee0728182df697`. |
| PMC EFetch `db=pmc&id=3433999&retmode=xml` | JATS parsed to 25 segments, with title and Introduction anchors; structured DOI `10.1002/ece3.315`. Cached continuation passed. Source SHA-256 `fe7a0b0098f8d9d6e70022bd47af74a2dd3168d17b8db259e06348df7472f75d`. Repeated after document-scope repair. |
| Actual Full MCP binary, isolated temporary configuration | 33 tools discovered; the same public PMC read returned `readable_text`, `identity_status: matched`, 25 segments and the same digest. Exit 0, empty stderr. This is a CLI protocol observation, not an installed-Host session. |

Temporary smoke responses live under `/private/tmp/qiongli-fulltext-*`; no paper
was added to a canonical research project. These probes demonstrate public-source
transport/parse/continuation, not scientific interpretation or database coverage.

Actual Host subagents `/root/literature_metadata` and `/root/fulltext_contracts`
implemented bounded disjoint provider/contract work. The latter independently
reviewed the coordinator's reader and identified multi-paper and wrapper-header
identity leakage, unbounded emitted PDF output, XML heading/segment amplification,
HTML misclassification and concatenated table cells. All reported issues received
source fixes and regression cases; the final read-only recheck reports no remaining
blocker from that review scope. The coordinator ran checks and retains integration
responsibility; parser-internal decompression remains the disclosed limitation.

One independent synthetic forward observation, `/root/fulltext_forward_trial`,
read the two actual guides and addressed three records: a DOI mismatch, an
unverified preprint with only introductory excerpts, and an abstract-only 403.
It proposed actual Host search including education-specific discovery, retained
all citekeys/evidence gaps, required digest-bound continuation and did not infer
unseen methods or exclude a study for missing fulltext. No simulated research
calls or writes were reported. A final source re-read found the routing
clarification did not change its response. Final guidance SHA-256 values:
`fulltext-fetcher.md` = `05391737262a054b71f892f487bd5ad032a737a317b3d7758435adf719953b89`;
`literature-provider-routing.md` = `16a93c3a80ee6f1f9e74c585b4cacf9a4bdcf0dce15424b0912cec6d6338aede`.
This single supplied-material case is coordinator-assessed behavior, not a model
benchmark, source evidence, domain-expert review or installed-Plugin acceptance.
No model override was selected and unavailable model identities are not inferred.

The next increment is maintainer-selected real-study/installed-Host qualification,
with parser isolation and additional document formats assessed from actual needs.
Authenticated OpenAlex content, paid/subscription/private sources, OCR and
Windows/Linux target-native behavior were not qualified. No release/version bump,
installed-plugin update, push or publication is included. The existing plan and
CLI-405 progress own this work; all 249 task states/dependencies and all 46 accepted
rows remain unchanged. Seven program-roadmap tests and index freshness pass.


## October 2 — selected public-paper and evidence-review increment

The maintainer selected real-paper tasks, an on-demand evidence reviewer and a
same-task comparison on the configured Codex and Claude Code Hosts. Content
`d44c45d2` adds one shared source-bound verification guide, routes existing
self-critique/collaboration to it and extends the existing review packet with
claim coverage. It checks substantive claims, source access, units, methods,
uncertainty and changed-evidence dependencies. There is no permanent agent per
discipline, new runtime service, Host adapter, model override or research-write
owner. Revision `2a368167` incorporates defects observed in the actual reviews.

Corpus `b4ce372f` adds three attributed public-paper packets under
`evals/research_journey/public-papers/`, with source/task hashes, XML paragraph
selectors, licenses and a stdlib integrity check. These are supplied-excerpt
appraisal and writing tasks, not new searches, full reviews or data reanalyses:

| Paper | Bounded task |
|---|---|
| Ritchie et al. (2013), DOI `10.1371/journal.pone.0078976` | Quantitative school experiment; Q1 appraisal and Q2 continuation with actual previously withheld sensitivity paragraphs from the same article. |
| Severe et al. (2024), DOI `10.1371/journal.pone.0297771` | Qualitative interview appraisal, source-author interpretation and a bounded Chinese writing task. |
| Trumble et al. (2023 online / 2024 issue), DOI `10.1007/s10459-023-10274-3` | Audit of a published health-professions systematic review, not execution of another systematic review. |

Manifest SHA-256 is
`fb9f1279115f22d41fca225642a16ad2cfcf66c882115b4fd8c8f814c28b5b0a`;
withheld criteria SHA-256 is
`c05a5ad7f2fd4bb1c8e8d0baa14f10b0946a7895dfd9d193be5db4cf5796c611`.
The initial guidance snapshot binds content `d44c45d2`, with manifest SHA-256
`8d05a63c4ef86825ebdab807b9b8d4a9b015f9e3948affbd3f48ae7488e24b17`.
Raw publisher/PMC XML was retrieved to `/private/tmp/qiongli-public-papers` and
matched against the attributed excerpts, DOI, license and paragraph selectors.
The XML, figures, tables, supplements and underlying data were not supplied to
task runners. The quantitative XML names CC BY without a version; none is added.

Actual native tasks `/root/public_quantitative_trial`,
`/root/public_qualitative_trial` and `/root/public_review_trial` produced all four
preselected checkpoints. Q2 received the actual Q1 bytes; originals remain
unchanged. Runners did not receive the rubric, sibling answers or parent verdict.
No configured model was replaced; exact native model/token/cost data was unavailable.

| Checkpoint | Observed result and coordinator assessment |
|---|---|
| Q1, 260.709 s | Distinguishes 109 pupils from 108 learning sheets, within-class retrieval from between-class mind-map allocation, adjusted analysis from causal mechanism, and a pilot proposal from implementation. Initial assessment passed; final reconciliation reopens the post-hoc computation/units check below. |
| Q2, 207.528 s | Actual added evidence narrows Q-C2/Q-C3/DEC-Q1: without the covariate, retrieval p=.14 and interaction p=.41. Retains prior history and IDs, does not infer no effect, and attributes the GLMM report without claiming the supplement was read. Passes the fixed bounded checks. |
| L1, 238.706 s | Retains 19 interviews, 68 invitations, recruitment/context limits, labeled quotation translation and the difference between author-reported saturation and independent verification. Passes the fixed bounded checks. |
| S1, 271.250 s | Separates seven databases from the EBSCO platform; 1,818 records, 56 studies, 63 experiments and 43 positive experiments. Flags abstract/body terminology differences and missing screening/appraisal details; invents no pooled or clinical effect. Passes the fixed bounded checks. |

Artifacts are under `/private/tmp/qiongli-evidence-trials-tzjdi7x9`. Initial
whole-answer span assessment `coordinator-review.json` has SHA-256
`a41d1c436e9ea1025f3a8c0648a5a19e4caad7a5a6e5a806fb777a27312fdadf`.
Final addendum `final-reconciliation.json` has SHA-256
`2bc019a9239a2f90ecbd8718c19534f7fc4810264bb5615e77ad36c57d70fa12`.
It preserves the initial assessment but records the final denominator as four
completed checkpoints, three passing coordinator review and Q1 with one reopened
`method/denominator` check marked unreviewed. These are model judgments, not
automated entailment, human expert review or program acceptance. Native task
recovery included one unavailable `TextEncoder` helper and one nonexistent
guidance-path read; both recovered without changing sources or fabricating work.

The configured Codex CLI 0.159.3 and Claude Code 2.1.287 independently reviewed
the same Q1 source/candidate through existing `qiongli agent ... prepare/collect`.
Both received identical 33,338-byte stdin, SHA-256
`534962d026c2aa9d98a12ee0cafecc1dec482118b41e50f240575cec2e23c8b7`,
without the rubric, Q2 or the other's report. Transport run/project IDs were
explicit synthetic fixtures with unknown component readiness; these were not
registered Full MCP runs or installed-Plugin qualification. Actual process exit
and collection both succeeded, with zero retries and process groups reaped.

| Host / actual execution ID | Observed time and usage |
|---|---|
| Codex / `01a0fbec-0df8-7823-823d-1d1a80055f0a` | 231.637 s; input 29,555, cached input 13,184, output 9,278, reasoning output 5,696 tokens as separately reported fields. Exact model and cost unavailable. |
| Claude Code / `ff9bc908-9735-4bca-b8ae-7e162a391ff2` | 207.124 s; input 12,334, output 22,103, cached input 0. Configured model reported as `deepseek-v4-pro[1m]`; Host name does not imply an Anthropic model. Host-reported USD 0.614245 has unknown cost basis, not verified billing. |

Both reports initially found no mandatory candidate correction. Their successful
transport is separate from quality: Codex inferred self-review because the prompt
could not supply its future execution ID; Claude described model comparison as
human comparison and stated a possible covariate pathway too categorically.
The guide now leaves unavailable identity to coordinator reconciliation and
preserves uncertainty in the review itself. Original reports remain unchanged.
Codex also emitted configured MCP startup diagnostics; no tool-call items appear
in its completed JSONL. Transport flags do not prove complete config isolation.
Allowlisted comparison observations are in
`/private/tmp/qiongli-two-host-zrn6vurm/comparison-observation.json`, SHA-256
`30c714b8d7f33b6e5b04d8f5c62bee60a2cce2607d935fdc5db8a08a9e397df2`.

A fresh `/root/review_reconciliation_trial` received the updated guide, actual
sources/candidate/reviews and allowlisted execution receipts, without parent
findings. Its unchanged 5,053-byte answer, SHA-256
`df2cbb3df9488d353d745707dc356a9a504d53731d210b6426e43ccf08ef9c93`,
corrects execution/human-review labels, retains the possible-path qualifier and
distinguishes old review guidance from the new guide. It also questions Q-C2's
percentage-point/adjusted-mean wording. The source reports descriptive percentages,
z-score analyses and post-hoc mean differences labeled 14.93% and −6.16%; Codex
accepts percentage points, while the fresh reviewer requests narrower wording.
This is not a demonstrated numeric error, but the supplied excerpts do not resolve
the exact post-hoc computation. The coordinator preserves that disagreement and
reopens the check; proposed wording retains the author's labels and the missing
calculation scope. Captured answers are not repaired or silently rescored as passes.
This follow-up tests the changed reconciliation guidance only; the earlier author
tasks and external reviews were not rerun under it.

Focused verification passes: 60 distinct Python tests covering content contracts,
links, handoffs, structure, Marketplace projection and existing journey evaluators;
the capability validator; root Skill validation; and corpus integrity against raw
XML with changed-count, swapped-paper and forged-anchor negative cases. Resource
links were rechecked after the small guide correction. Final embedded-pack test
and fresh native export/projections pass. An initial lock-generation invocation
used a short commit hash and correctly failed before writing; it was rerun with
the full source commit. The existing atomic API deprecation warning is unchanged.

Final native pack: content source `2a36816760437a9b09ce307837ce03728ebc8c78`,
446 resources, content root
`21ca4da75c492d9b420b14a3868f05f76b5673be896e23c65e30080c51da3688`,
pack `cdd51be04eaf129ac08305f70f4585be2cd5623e269a9016c32baf72e5566355`.
The development binary SHA-256 is
`d0f8c13ee511e9cc1f78a930740a6c5b8169bfb43d3cf70b942e960824bca200`.
Codex/Claude/DeepSeek projections preserve all 444 non-manifest content resources
byte-for-byte, expose 22/2/22 Skill entries and have no missing resource links.
Final exports and `projection-check-final.json` remain beside the trial artifacts.
The original external trials used the earlier pinned binary/guidance; these final
package checks do not relabel those runs as final-package qualification.

All seven program-roadmap tests and index freshness pass. The ledger comparison
preserves all 249 task states/dependencies and all 46 accepted rows; every row
except CLI-405 progress is unchanged. Final diff review and whitespace checks
pass; integration uses the frozen-source guard and local fast-forward merge,
without rerunning unchanged checks solely for commit/merge.

This increment records progress under CLI-405 without changing any task state,
dependency or accepted row. The next increment is maintainer-selected review of
actual tables/supplements and installed-Host/project end-to-end qualification.
One candidate pair does not estimate accuracy gain, Host superiority or an optimal
agent count; retain on-demand bounded collaboration. No release/version bump,
installed-user Plugin update, private research access, push or publication occurred.


## October 2 — selected workflow reliability execution plan

The maintainer now authorizes planning and executing the six improvements found
in the preceding review. This is one connected research-workflow outcome, with
the following order and completion evidence; a successful subtask does not close
the other rows. Reuse canonical content, native readers, project approval/CAS,
existing search and evaluation owners, and the user's configured models.

| Order | Work and owner | Required completion evidence |
|---|---|---|
| 1 | Resolve the quantitative example against actual tables/supplement; preserve source and claim history under the existing public-paper corpus. | Retrieved source identity, inspected table headings/notes and supplement passages; explicit resolution or evidence-bounded wording for Q-C2; a new answer-bound continuation without altering prior outputs. |
| 2 | Bound native fulltext parsing through the shared runtime and its CLI/Lite consumers. | Normal PDF/XML reading plus malformed, oversized, timed-out and resource-limited parsing cases; the supervising MCP remains responsive and child work is reaped; compatibility and public-network restrictions remain intact. |
| 3 | Exercise the installed development candidate in an isolated actual Host/project. | Search → source read → extracted evidence → draft → review → approved project write → process restart/resume, with current revisions and stale-review rejection. Use public data and isolated profiles; distinguish actual model/Host behavior from direct protocol tests. |
| 4 | Measure and improve existing provider/Host search routing on fixed public queries. | Predeclared known-item and topic queries, observed source contributions, duplicates, body readability, identity mismatches and elapsed time; source-bound comparison and narrowly justified routing changes. Unavailable providers remain in the report. |
| 5 | Preserve actual failure modes as reproducible regression material using existing journey evaluators. | Durable minimal public-source/answer observations and checks for unit/denominator errors, abstract/body conflicts, changed versions and review attribution; frozen answers and negative cases cannot be replaced with passing fixtures. |
| 6 | Consolidate Skills and current roadmap state. | Shared guidance remains discoverable without duplicated constraints, field-specific judgments survive, outdated next-step wording is removed from the current horizon, affected checks and regenerated content projections pass. |

Independent review is bounded to consequential source or implementation questions;
one coordinator owns integration. Compare accepted corrections and observed cost
before expanding collaboration. No permanent discipline-Agent registry, new
research store, publication or installed-user-profile update is selected.
Track implementation, observations, unresolved gates and integration below, then
update CLI-405 progress once at integration without promoting accepted evidence.

### Implementation and source-bound observations

The scoped source increments are `2ae0c8d6` (shared Skill guidance), `9f1bd51e`
(parser isolation, credential routing and embedded pack), and `96d20848`
(public-table continuation and frozen regressions). No fixed Agent team or new
research store is introduced. Existing source/review and project write owners
remain authoritative.

**Actual tables and supplement.** The frozen public-paper corpus now includes
`quantitative-education/tables-source.md`, `tables-task.md` and
`tables-manifest.json`. The source packet SHA-256 is
`3eca31d16e9ddf2803697236fe9f904c3a86f4743756f8b0bb930c92d4b7ac34`;
the manifest SHA-256 is
`f865b00eac01a6903974317f19a9f535e2819da276730b2fdb1323adfaca8530`.
The packet preserves Table 1, Tables S2/S4/S5 and the Text S1 analysis excerpt
from the actual publisher material, with raw-source identities in the manifest.
The coordinator inspected the table and rendered supplement, then checked a
fresh source-bound `/root/table_continuation_trial` answer. That task saw the
transcribed packet, not the publisher originals or earlier reviewer conclusions.
Its original answer is preserved as `observations/q3-answer.md`; execution/session
and model fields unavailable from the dispatch are not invented.

Table 1's descriptive differences must not be substituted for the author's
adjusted post-hoc values. The precise scale conversion behind the reported
14.93% and −6.16% remains unresolved; retain the labels and computation limit.
Table 1 totals 108 while S5 reports 109 observations. S4's P5 total is printed
as 38 although its cells sum to 58; S2's condition totals and other denominators
also disagree. Do not repair the source or explain all discrepancies as one
missing participant. Text S1 supports the named covariate and participant random
intercept, not an invented classroom adjustment or corrected software call.
The continuation retains Q-C1/C2/C3, DEC-Q1 and stage history with bounded wording.
Original retrieval/review material is retained locally at
`/private/tmp/qiongli-table-source-dgojnamy` and the forward task at
`/private/tmp/qiongli-table-continuation-leyjy8q0`. Signed publisher redirect URLs
are excluded from the durable manifest. This is model/coordinator evidence, not
domain-expert or human research acceptance.

**Bounded parsing.** The existing native CLI, Full/Lite MCP and standalone Lite
reader share a same-executable parser child. It receives document bytes only,
with a cleared environment, 12 MiB input and 16 MiB response ceilings, a 30-second
supervisor deadline and independent child watchdog, and a 512 MiB Rust requested
heap ceiling. The small stdlib allocator boundary preserves unrestricted parent
allocation and System realloc/zeroed behavior. The parent reaps the child on
failure and validates the returned source digest and segment count. These are
not OS RSS, stack or native-mapping limits, an OCR service or a security sandbox.
Existing public URL/DNS/redirect restrictions and write approvals are unchanged.
Ownership, review policy, affected-check routing and the runtime contract include
the allocator explicitly.

Actual normal and compressed-PDF probes succeeded on CLI and standalone Lite;
a 612,083-byte PDF whose stream expands to 600 MiB hit allocation failure, and a
subsequent ordinary PDF was readable. Raw observations and exact earlier binary
hashes are in `/private/tmp/qiongli-worker-boundary-d2f14x1d`. After the allocator's
final zeroed/realloc changes, the initial installed CLI candidate was retested with the same
inputs: success, allocation abort, success (0.025/0.804/0.020 seconds). Its record
is `current-worker-smoke.json` in the installed-journey directory below. These
direct worker measurements do not by themselves establish MCP responsiveness;
the persistent supervisor/protocol tests cover failure handling and recovery.

**Fixed public search observation.** The predeclared three queries are the Ritchie
paper's title, “Attention Is All You Need”, and “retrieval practice classroom
delayed retention”, with limit 3 per provider. Artifacts are retained at
`/private/tmp/qiongli-search-observation-hyrf6ucc`. With isolated empty credentials,
12 of 15 calls were not run and three arXiv calls had network errors/timeouts.
With normal configured read-only provider access, all nine selected active-provider
calls were blocked by the shared credential loader, including explicit arXiv.
There were no successful provider queries from which to estimate recall or rank
providers. The first observation used a mutable development path rebuilt during
the trial; its two observed hashes are retained, not asserted identical.

Actual Host web search took 2.250/3.236/2.103 seconds. Its predefined nine top-result
URLs include six paper mentions representing three works, three duplicate paper
mentions and three non-paper pages. Body reads matched public paper identities;
Host output supplies no byte digest and the arXiv version was unknown. A separate
fourth topic result used for body reading is excluded from top-three contribution
counts. Search success and readable body are separate observations.

The measured credential fault is fixed at the existing `for_search` owner:
explicit public-only or inactive selections use provider preview state without
loading unrelated secrets. Queries that require configured credentials retain
the existing bounded loader; this is not per-provider credential isolation.
The post-fix arXiv-only repeat used the immutable initial installed binary, with matching
before/after hashes. All three queries reached network diagnostics (1.199/15.003/
1.398 seconds), with zero credential-unavailable failures. Hit counts remain
unknown because network calls failed. No provider quality advantage is claimed.

**Durable failures and shared guidance.** The existing public-paper evaluator now
replays five selected, exact answer spans: units, denominators, abstract/body
conflict, source revision and review attribution. Frozen observations remain
three passes and two failures; original answers and failed human-review labels
are not rewritten into passing fixtures. Six additional negative mutations
protect answer/source bytes, denominators, anchors, reviewer identity and result
promotion. Parent model judgments are labeled as such, with unavailable event
capture explicit. This is a selected-span regression corpus, not an overall
research-quality score.

Self-critique links the existing A–M owners instead of repeating their constraints;
field-specific references remain available. Fulltext guidance now explains parser
failure limits and bounded fallback without repeated identical requests or weaker
network guards. The current roadmap horizon links this single six-part increment;
accepted ADRs and historical evidence remain unchanged.

### Focused verification and candidate identity

The 35 affected Python ownership, boundary and registry checks pass. The 39
Skill/resource/handoff/structure/roadmap/evaluator checks pass after correcting a
stale Stage B audit expectation; the original failure and focused two-test rerun
are retained as a tooling correction, not a research answer repair. Capability
and root Skill validators pass. The public corpus integrity check, five frozen
observations and its negative mutations pass their expected results.

Native checks include the allocator's two tests, runtime supervisor/reader tests,
eight CLI stdio tests, 19 standalone Lite MCP/planning tests and embedded-content
verification. Six runtime loopback cases initially hit the execution sandbox;
the nine-case Zotero companion fixture set passed with loopback permission, without
accessing a real Zotero library. One CLI socket fixture likewise passed when
permitted. Runtime/allocator all-target Clippy passes; after the final allocator
change its two tests and Clippy were repeated. The pre-existing atomic API
deprecation warning remains. Existing checks are reused when inputs are unchanged.

The initial immutable CLI candidate has source
`9f1bd51e5cbb389ef13eb7751cc5b90cc042539e`, content source
`2ae0c8d64a1bfc5c9ff9545b869d5edd300a532b`, and binary SHA-256
`a799e2496d0e3d9c9bf8904ac35bf76cfa21c365279320943abc8aefe912db30`.
Its 446-resource native pack has content root
`632b6a4cd7ea19e2ef34041f8ef0f01417010303222d6f15c5ab9c3838841e6a`
and pack SHA-256
`77d56766c582666ef08730c9981fd66f5a0830b213a6b0f79260411af464a7f4`.
Fresh native export and Codex/Claude/DeepSeek projections preserve all 444
non-manifest resources byte-for-byte, with 22/2/22 Skill entries. Native Plugin
internal resource links have zero missing targets. The projection observations
are scoped package checks, separate from actual Host qualification.


### Installed Host journey and compatibility corrections

The actual isolated development profile/project is retained at
`packages/qiongli-native/target/installed-journey-qku0zg37` (ignored local artifacts,
not durable accepted evidence). It uses Codex CLI 0.159.3 and the actual enabled
`qiongli@qiongli-cli-local` 2.1.1 Plugin, with 22 Skills and Full MCP's 33-tool
inventory. Native source preview/apply and official Codex registration verified
source receipts, enabled state and cached bytes. Only the isolated profile was
updated. The original binary and raw author output remain unchanged.

Author session `01a0fc2e-f432-7b33-8110-393bc29f6fe6` actually discovered and called
seven installed public/read/preview tools, performed native search and Host web
search, read publisher HTML body, extracted J-C1–J-C6, drafted a bounded teaching
paragraph and performed explicitly labeled self-review. Its recorded turn context
confirms the configured `gpt-6-astra` / `xhigh`, without model override. The process
exited 0 after 738.766 seconds; this is an elapsed observation, not a benchmark.
`author-answer.md` SHA-256 is
`ba298b03ae22590955249117778a294d9ee0669b99ba6a293721f71aac9b1129`.
The coordinator's prompt accidentally supplied the wrong title for the correct
DOI. The author corrected the DOI-linked identity and retained the mismatch;
this was not an intentionally seeded test. Four providers were unconfigured,
arXiv timed out, and native XML access was blocked by a nonpublic DNS result.
Host HTML reading supplied body passages but no byte hash or native segment IDs.
These limitations are preserved, not relabeled successful native discovery/read.

Two actual Host compatibility failures justified small existing-owner fixes:

- `2ae52aa4` replaces the fulltext input's root `oneOf` with the existing plain
  object fields. The original Host exposed only `offset`, losing required URL.
  Runtime continuation digest, URL/access and unknown-field checks remain intact.
- `d50a1ef0`, corrected by `f3f1379a`, lets capture preview compute a new capture ID
  through `ResearchCaptureDraftV1::into_capture`. Only `capture_id` may be omitted;
  version/kind/binding remain required. Preview returns the normalized capture
  alongside its existing plan. Supplied IDs are never repaired; apply still
  requires that complete object, digest, explicit approval and current revisions.
  An independent static review of the initial patch found partial-envelope
  schema/runtime disagreement. The final smaller correction was reviewed by the
  coordinator and exercised by positive and negative protocol tests; the earlier
  review is not represented as an independent review of the final correction.

The fixed immutable candidate's source is
`d1b3af4ff985bc37bc306755ccba7cb43800da0e`, content source
`f3f1379a9d99119e51d3a7e9a324019c33730a0c`, binary SHA-256
`e0a8aace09729fac8f3a152b9390ccfe3f66b973a0f40347c218051bdec0ba79`.
Its 446-resource content root is
`27823ab6da29beece98c5a974776d09319fb4cc8be4fd218a71ed6d05ec36fcc`,
pack SHA-256
`a3cf0c13da6e93efd2c659809badd2d51832f7fc9fcee18c006ac8f10a7a25b8`.
Native guided update applied source receipt
`ac37f3cdd05e3f358c0a449c729ce9514b3b71be6ac1fb076a994d2ddcccd532`,
then reviewed official `plugin remove` / `plugin add` actions refreshed the old
verified cache. Cached binary hash and corrected contracts match the candidate.
Fresh native export and Codex/Claude/DeepSeek projections again preserve all 444
non-manifest resources, expose 22/2/22 entries and have zero missing resource links.

A new read-only process resumed the same author session on the fixed installation
(exit 0; 135.706 seconds). Actual fulltext invocation now included URL, DOI, offset
and limit; the existing network guard still blocked the source. The installed
preview succeeded after omitting only the invalid ID. Structured comparison
confirms no research/binding field changed. Its capture is
`cap_fdd144424da14f8b543c4635a72e1b5bcd91a8caaa6c35601eef36bf09ff1dce`,
plan `6ef057824904624b4385da585481a0d6951471c35abc72698fa521d73e5222f1`.
This is continuation by the same author, not another reviewer.

**Review and approved writes.** The coordinator inspected the unchanged complete
answer and separately retrieved publisher XML (SHA-256
`18acae39e5cfbb8832a6ab616f49de795af8130ef9b1be52e03028febb202cb2`).
XML anchors s2a2/p1, s2b/p1–4, s3a1/p1, s3a4/p1, s3b/p2–7, s4/p5 and s4a/p1
support the transcribed values and bounded assessment. E1's 109 is recruitment,
not a universal analysis denominator; E2's p=.002 refers to the joint follow-up
model. Table/supplement inconsistencies, method questions and the author's HTML
hash remain unresolved. This later source inspection is not attributed to the
author. `coordinator-review.md` records model review, not human/expert acceptance.

The coordinator used the exact installed preview to approve native CLI intake of
the original capture as pending history. Its consolidation preview correctly
reported scope-boundary and contradiction conflicts. The original was neither
rewritten nor consolidated. A separate reviewed continuity capture,
`cap_291f0110d55973ce9d0d1ad611d62088759f8dc38a30e00f9bedc79f5470b6bd`,
preserves Ritchie2013, J-C1–J-C6, the exact paragraph wording, tentative J-DEC1,
source scopes and unresolved gaps. One coordinator owns these native writes;
they are not reported as model-initiated Host apply calls. An initial coordinator
preview rejected a trailing space introduced while splitting the paragraph into
bounded fields; the corrected fields join to the unchanged original paragraph.
The rejected observation is retained; the validation rule was not weakened.

The inspected native consolidation plan created only `context/research_state.md`
and `context/decision_log.md`, advancing project/library revision 1 to 2. The
actual file hashes match the preview's proposed hashes. Generated decision row
`dec_291f0110d55973ce_1` retains J-DEC1 in its tentative statement. `stage_handoff`,
paper-note and standalone manuscript files remain candidates; this existing owner
does not write them. No project stage, scope, locked decision or acceptance changed.

A diagnostic replay of the same reviewed continuity was separately previewed as
ready at revision 1 under capture
`cap_f187c91318b6bb2618f5301bea186e6bd309654cb101f9c22936fc2c0c6608f6`.
After the approved revision-2 write, applying that exact old consolidation plan
returned `capture-consolidation-conflict`. All project-file and library-file
hashes were identical before/after rejection. This is an actual stale revision
negative, not a fabricated digest failure or an extra research review.

The execution profile used read-only shell and exact Plugin-scoped tool allowlists.
Early sandbox/model-routing and default-approval attempts failed without research
writes. Automatic approval review rejected a broad `--approve-for-me` attempt;
that mode was not used. The subsequent approved narrower configuration retained
all non-allowlisted tool approval boundaries, no model override and no private
library access. No credential contents were inspected by the coordinator or included in the
trial records. The Plugin-scoped
configuration follows the official [Plugin documentation](https://developers.openai.com/plugins/build/plugins).

Final compatibility checks: nine MCP schema tests, capability validation,
runtime all-target Clippy and native formatting pass. CLI stdio has eight passing
cases, including normalized/full capture equivalence, missing envelope, forged
identity, direct-draft apply and portable-delivery rejection. Its loopback fixture
initially hit sandbox denial and passed when granted local fixture permission;
no real library was accessed. Embedded pack and runtime registry checks both pass.
The pre-existing atomic deprecation warning is unchanged. Corpus integrity again
reproduces the frozen three supported/two failed spans and its negative checks.


**Restart/resume.** A third process resumed the same installed session (exit 0,
120.295 seconds). Actual project/artifact/coverage tools and read-only local reads
confirmed revision 2, both persisted artifacts and their receipt-bound hashes,
the J-DEC1/generated-row mapping, and missing stage-handoff/note/manuscript files.
It preserved the author HTML/coordinator XML distinction and all denominator,
method and repeated-assessment limits. Coverage reports one current/two stale
captures and zero pending-review count; the original proposal remains semantically
unconsolidated in history, not accepted merely because a counter is zero. Its
report checked current project hashes against the stale-rejection observation.
All three recorded turn contexts retain `gpt-6-astra` / `xhigh` and read-only shell.
`resume-answer.md`, `resume-events.jsonl`, `all-turn-model-observations.json` and
`stale-consolidation-rejection.json` preserve the observations locally.

### Integration audit and remaining scope

| Selected outcome | Completion and limits |
|---|---|
| Actual tables/supplement | Retrieved identities and inspected source packet retained; Q-C2 uses evidence-bounded wording where computation/denominators remain unresolved; original answers unchanged. |
| Bounded parser | Shared CLI/Lite supervisor and allocator implemented; normal/malformed/oversized/timeout/resource/crash/recovery checks and actual compressed-PDF probe recorded; no OS RSS or security-sandbox claim. |
| Installed Host/project journey | Actual installed author search/body/extraction/draft/self-review; coordinator source review and approved native writes; restarted installed Host verifies revision/continuity; genuine stale-plan rejection changes no files. Failed native network paths and missing standalone handoff artifacts stay explicit. |
| Search observation/routing | Fixed queries and unavailable providers recorded; measured unrelated-secret fault corrected at existing routing owner; Host contributions and duplicate/body checks retained without provider-quality ranking. |
| Durable regressions | Frozen actual failure spans, source identities and negative mutations replay unchanged; observed failures remain failures. |
| Skills/roadmap consolidation | Duplicate checks replaced by existing owners; discipline guidance retained; current horizon aligned; final pack/projections and affected checks pass. |

These six bounded development outcomes are implemented. They do not establish
program/release, domain-expert or general installed-Host acceptance. The next
increment is configured-provider/transport qualification on real quotas and
representative discipline tasks, then an explicitly selected formal handoff
persistence flow if needed. Reuse current owners and measure concrete failures
before adding agents or another store. Source denominators/computation, human
academic review, private-library access and target-native distribution qualification
remain separate gates; no research uncertainty is closed by a software check.

At integration, only CLI-405's progress text changes. All 249 task states and
dependencies and all 46 accepted rows remain unchanged. Final diff review keeps
the single trailing-space exception in the frozen, unedited Text S1 quotation
(`tables-source.md:87`); trimming it would invalidate the observed source packet.
All other changed lines pass whitespace checks. The frozen-source guard, generated
program index freshness and seven program-roadmap tests are run before the local
fast-forward merge. No additional checks are scheduled solely because of a commit
or merge; no push, publication, version bump or user-profile installation occurs.


## October 2 — public channel qualification follow-up

The maintainer selects the next public-channel increment after the six-part
reliability work. Smallest useful outcome: execute bounded public queries with
existing configured providers, distinguish credential/DNS/HTTP failures, and
retain usable provider results when another configured channel cannot load its
credentials. Preserve network guards, configured models and existing project
write owners; no private corpus or user-profile installation changes.

Predeclared hypotheses: shared credential loading blocks selected channels before
network; special-use DNS answers explain the native fulltext guard; provider
transport failures differ from credentials and zero results. Start with the same
known-item title and at most three records per active provider, without retries;
retain original observations. Validate the earliest owner with deterministic
negative cases and repeat only the changed combined search. Record current quota
access as unavailable when no authenticated request executes. Formal handoff and
representative discipline assessment follow this bounded channel increment.

### Observations, correction and integration

Base `4ca98350079cb7b515465c5034cef7e4fa236d0b`; implementation
`ef7885ded17dc4f53f9a89eee0fe1876168202c9`; regenerated-pack candidate
`a432ea356c8f979e9a16ac639ba614707ae47ce7`. The before process uses the unchanged
prior candidate `d1b3af4ff985bc37bc306755ccba7cb43800da0e`, binary SHA-256
`e0a8aace09729fac8f3a152b9390ccfe3f66b973a0f40347c218051bdec0ba79`.
The new macOS arm64 development binary has SHA-256
`f318a4ef2052186164d6d3f3fddb19803abcae974742036090b7d976e0915749`, content source
`ef7885ded17dc4f53f9a89eee0fe1876168202c9`, content root
`f4e87b2e1c57ce5f9d40a988451efd16082dd3e3954907d19053884a0a46635a` and pack
`2014a0a12df531522820d5e0a815d5275caf6d37d8617f86e0f97005e8ec568a`.

Actual JSON-lines MCP probes use the native Lite CLI and existing configuration
owner, without Host/model overrides or credential inspection. Redacted status
reports OpenAlex, Semantic Scholar and arXiv configured; Crossref lacks email
and PubMed lacks a key. Only the known-item title **Attention Is All You Need**
was executed, with limits of three and no retries. The other two queries retained
in the observation plan were not executed; there is no three-query comparison or
provider-quality ranking. A title-filtered result is distinct from the raw count.

| Actual call | Outcome |
|---|---|
| Before: OpenAlex alone | Credential-load tool error after 3.007 s; hits unknown; no provider HTTP response. |
| Before: Semantic Scholar alone | Immediate same error while the shared loader remains busy; not proof that its own key was attempted. |
| Before: arXiv alone | Complete, raw count 3, one retained known-title record in 1.308 s. |
| Before: all three selected | Credential-load tool error after 3.003 s; arXiv never executes. |
| After: identical combined request | Partial in 4.108 s; arXiv raw count 3 and one retained `1706.03762v7` record; two explicit credential-unavailable warnings. |

The shared deferred credential loader was the earliest failing owner: a timeout
returned before usable selected channels could execute. It now derives an
uncached fallback from the metadata preview, disables entries with unresolved
values, and executes usable selected channels. A completed credential load still
supersedes the fallback. Sole blocked selections retain the bounded error;
unselected channels receive no warnings and unavailable channels receive no
fabricated zero-result counts. This does not repair secret-store access itself.

An authorized DNS-only probe returned `198.18.0.0/15` addresses for the four
selected public domains. This is non-globally-reachable benchmarking space in
the [IANA registry](https://www.iana.org/assignments/iana-ipv4-special-registry).
System HTTP/HTTPS proxy flags were enabled; proxy environment variables were
absent. The specific mapping/secret-store cause is unproven. The earlier sandbox
DNS attempt could not resolve and is not counted as a provider failure. The
unchanged native fulltext guard correctly refuses these addresses before HTTP;
the prior fulltext failure is reused, not presented as a fresh body retrieval.
No DNS/proxy/permission change or network-guard bypass was attempted. No
authenticated provider request completed, so API authentication and quota
qualification remain unavailable, not failed or accepted. arXiv transport
succeeded; that does not qualify other endpoints or fulltext access.

Canonical Skills now distinguish metadata configuration from credential/query
success, explain mixed-search partial results and correct native setup guidance:
native configuration writes/wizard remain unavailable. Standalone Lite's wizard
is unchanged. Public Host reading remains a separately attributed option under
the existing access rules; no Host body read was executed in this follow-up.

Validation: one fallback/cache unit case, all 11 runtime Lite-MCP cases, all 95
standalone Lite cases, 21 focused Python contract/content checks, capability
validation, runtime all-target Clippy, native formatting and both CLI embedded
pack/registry cases pass. The pre-existing platform atomic deprecation warning
is unchanged. Native export verifies 446 resources; all 444 non-manifest bytes
match Codex/Claude/DeepSeek projections, with 22/2/22 entries and no broken
resource links. These are package projections, not new installed-Host trials.

Local observations are under the private temporary directory
`qiongli-channel-qualification-dukx8djh`: `plan.json`, redacted status and
`known-attention-*.json`, `mixed-before.json`, `dns-network-permitted.json`,
`candidate.json`, `after/mixed-after.json` and `projection-checks.json`.
The original binary hash and before responses remain unchanged. Temporary files
are local observations, not durable accepted evidence. Main-agent review is
self-review. At integration, only CLI-405 progress changes; all 249 task states
and dependencies and all 46 accepted rows remain unchanged. The generated index,
seven roadmap tests, whitespace check and frozen-source guard pass before local
fast-forward integration. No push, publication, version bump or normal user
Plugin/profile update occurs. Next select representative discipline assessment
and, if needed, the formal handoff persistence flow using existing write owners;
unresolved credential/transport qualification stays explicit.

## October 2 — representative discipline transition assessment

Continue from `61de2204bda88e29dd8e19836e1268b9ee7223fe`. The smallest outcome is
two source-bound C-to-F exercises: the existing education experiment with its
actual Q3 predecessor, and a computing benchmark with newly inspected public
source passages. Assess whether design decisions and final prose preserve the
same evidence limits, source/claim IDs and unresolved prerequisites. The main
agent executes and reviews both; this is transparent self-review, not a blind
forward test or evidence of model accuracy improvement. Existing frozen failures
remain unchanged. No private data, experiment execution, installation or canonical
research-project writes are selected. Formal handoff persistence follows separately.

Before drafting, retain each task, source/predecessor hashes and review criteria:
education must preserve measurement timing, adjusted versus unadjusted evidence,
unit/denominator uncertainty and DEC-Q1; computing must preserve source version,
benchmark/evaluation split, measured versus estimated quantities and actual
execution status. A changed source or unsupported claim must reopen only the
affected decision. Update existing guidance only where the source-backed exercise
identifies a useful missing decision rule; do not add another agent registry,
workflow schema or store.

### Bounded results and candidate

Implementation `14e33e19384665dc0d96b5366a466e9d041ed7e9` adds the missing
adjustment-timing decision at the shared Stage C owner and benchmark quantity/split
boundaries in the computing guide. The two exercises and frozen self-review
annotations live in `evals/research_journey/public-papers/discipline-transition/`.
Education retains Q-C1/Q-C2/Q-C3, DEC-Q1 and the actual Q3 predecessor while carrying
the adjustment and unresolved table limits into proposed prose. Computing keeps
M-C1/M-C2 and DEC-M1 conditional on the evidence appropriate to each claim.
Neither exercise executes a study, benchmark or canonical project handoff.

The computing source notes come from actual Host reads of the
[versioned paper HTML](https://arxiv.org/html/1706.03762v7) and
[PDF text](https://arxiv.org/pdf/1706.03762v7). The read exposed a table/body
discrepancy; its cause stays unresolved. PDF screenshot requests returned references
without viewable pixels in this tool surface, so no visual-table inspection is
claimed. Raw download hashes are unavailable; the note digest identifies only the
coordinator's short paraphrase. The adjustment reference's PMC body presented a
browser check; its [PubMed abstract](https://pubmed.ncbi.nlm.nih.gov/19525685/) was
read, without claiming full-article review or working around that check.

The tasks were declared before drafting; the same agent then refined guidance,
wrote candidates and annotated two selected manuscript spans. Their source-bound
self-review supports these bounded decisions, not a claim that a model's failure
rate improved. The original 20 corpus files are byte-identical to the baseline.
Existing five-span replay remains three supported/two failed; the two additional
spans are separately labeled self-review. Four new negative mutations reject
changed source notes, changed answers, forged anchors and independent-review
misattribution. The same existing projection owner evaluates all spans; no new
semantic grader or schema is introduced.

Tasks SHA-256 `a1e5bfe363b85f25250d603e4aa81b1527ca25e41f558a23011046837506a9b1`;
notes `904356512006f0ffc76fd148d98986eab2924623089b0d048241709006872322`;
answers `e78915208a65133b503a4da5f0de4627cb4a1d5a186bd953bea29383473c31a4`;
observations `47bc9346ea418d260d702ac83742b253f56f05b13e542aef2f81a835e20499b3`.

Regenerated-pack candidate `f9a1fc071bf9142f770e8f3a199990f22bb8ff73` builds on
macOS arm64. Binary SHA-256
`df12856a78d7e59083128bb80c785efbf68fa0466142d29ddbd0e810592365d5`;
content source `14e33e19384665dc0d96b5366a466e9d041ed7e9`, root
`53786224f3d80ad7392a9a99d993d31ed2096b0bcce2715a5592b98d88d52008`, pack
`4c355799192f0036eb58c95820af20ca42ccc3fe69b4765be8c99d51a1079be6`.
Its binary/export/projection observations are retained in the private temporary
directory `qiongli-discipline-transition-ex6h89u7`; these are not installation or
release receipts. All 446 resources export; 444 non-manifest resources match each
of Codex/Claude/DeepSeek, with 22/2/22 Skill entries and no broken resource links.

Verification: 32 focused research-journey, continuity and Skill tests, corpus
integrity/replay with all negative mutations, capability validation and root Skill
validation pass. Both CLI embedded-pack/registry checks pass; the existing platform
atomic deprecation warning is unchanged. Runtime logic did not change, so prior
search/transport tests are reused without another live query.

The next bounded increment is a formal stage-handoff persistence/resume flow
through the existing preview/approval/CAS owner. This exercise does not resolve
that gap. Broader disciplines, blind forward evaluation, domain-expert review and
authenticated-provider/transport qualification remain open. Integration records
only CLI-405 progress; task states, dependencies and accepted evidence stay fixed.

## October 2 — formal stage-handoff persistence and resume

The maintainer continues the selected next increment and requests lightweight
`gpt-6.1-sol` reasoning for checks. Baseline `d03dfbed` already reads registered
handoffs but capture consolidation can write only research state and decisions.
Implementation `fafd68014b7c2bf4007c7ebbdc9b9e9a295d2db6` adds the optional CLI
`--stage-handoff-file` to that existing owner. It appends explicitly supplied
Markdown, previews the complete resulting handoff, preserves all earlier bytes,
and binds the new content to the existing digest, dual approval, transaction,
receipt and revision checks. No automatic handoff is inferred from a capture's
summary. The native boundary validates bounded UTF-8 data; scholarly completeness
and the required handoff sections remain review obligations.

Both preview and apply now recheck the registered semantic digest, including
inputs that are not write targets. Unrefreshed drift is rejected rather than
silently adopted into the next manifest. Non-registered sources and versioned
stage summaries remain outside this digest and require separate source-byte
checks. Default API/output behavior and old receipts remain readable; an older
binary cannot read receipts containing the new closed `stage-handoff` variant.
No MCP save endpoint, storage subsystem, stage advance, automatic Graph rebuild,
summary creation or model/profile change is introduced.

Two isolated copied-CLI journeys pass on macOS arm64: the existing default
capture flow and the optional handoff flow. Separate processes perform preview,
approved apply, explicit Graph snapshot and source read; revision 2 returns the
exact previewed handoff, while revision 1 fails. Changed draft bytes, changed
registered inputs, incomplete approval, held library lock, a killed waiting
writer, altered review timestamp and replay leave the expected project/config
bytes intact. The existing fixture restores its deliberately changed test input
before the positive save; this is not production recovery guidance. Service tests
also append a second capture without erasing the first entry. These are storage
fixtures with synthetic claims/limits, not completed research-stage examples or
installed-Host observations.

The requested native sub-agent `/root/handoff_persistence_check` ran as
`gpt-6.1-sol / low`. Its read-only review identified a draft-path Debug disclosure
and wording that could imply native revision coverage for arbitrary summaries;
both were corrected. It directly ran 20 existing compatibility tests: capture
12, artifact drift 2, semantic timeline 5, revision-bound reader 1. Test-binary
SHA-256 `6e37b7450a1063516466070d8bb2f1b6b9e7de1f5722299848b966ecd8d9469b`.
The coordinator ran 11 consolidation tests, 3 CLI parser tests, the 2 copied-CLI
journeys, 13 handoff/continuity/resource-link checks and 7 roadmap checks. All
pass. An initial negative fixture named a non-registered evidence path and was
corrected to the actual registered boundary-review artifact; no protection of
arbitrary files is claimed.

`qiongli-project --all-targets` Clippy, touched-file rustfmt and diff checks pass.
The wider CLI Clippy check is not green under Rust 1.99.0: the new
`chunks_exact_to_as_chunks` lint stops at three unchanged platform locations
(`community_alpha_integrity.rs`, `grant.rs`, `release_authority.rs`); a
`--no-deps` check also exposes four unchanged CLI locations (`command.rs` three,
`external_agent_cli.rs` one). This increment neither changes those files nor
weakens their lints. The existing atomic deprecation warning remains.

Regenerated-pack candidate `4b82685bf977f97f34d843d76b5f4fd21b5588cf` contains 446
resources, with content source `fafd68014b7c2bf4007c7ebbdc9b9e9a295d2db6`, root
`8b7c2014d01fbf496cfaa1f59ce4a786fc9d9d5f514bc232ce07dedde06d7b1c` and pack
`8d2ef5f713f369bf8d6ac04c3cb538dd8ee5deb2c07a7694c58eeabbd624b9e8`.
Both embedded-pack/registry checks pass. The rebuilt local CLI SHA-256 is
`9bde87915e5ce1e6be783e37163c308b1ef5fb261804cceacea7e7255717e3fc`;
the earlier two runtime journeys used SHA-256
`6c4d661b94b0bf31fa637b6ee8c31064227af883046ed53d04e53b190e339887` with identical
runtime source and content bytes, before the pack source-commit metadata was
rebound. Their results are reused without claiming an additional installed run.

Next: exercise this save/resume from a configured Host in an isolated project
using public evidence, including continuation after a source changes. Native
versioned stage-summary/history writes remain separately scoped. Installed-Host,
domain-expert, authenticated-provider and fulltext-transport qualification remain
open. Integration updates only CLI-405 progress: all 249 task states/dependencies
and 46 accepted rows remain unchanged. No private research access, installed
Plugin/profile update, publication or program acceptance is claimed.

## October 2 — actual Host handoff save, restart and changed-source continuation

The maintainer selects the preceding next increment and requests lightweight
`gpt-6.1-sol` reasoning for verification. Baseline
`058a047b5301a12688b7dbb5a4f4cfc3259eca9f` supplies the copied CLI with SHA-256
`9bde87915e5ce1e6be783e37163c308b1ef5fb261804cceacea7e7255717e3fc` and the
previously recorded 446-resource pack. No native runtime or content changes are
needed. The isolated local trial is retained under
`packages/qiongli-native/target/handoff-host-fncseb5l`; its raw observations are
development evidence, not durable accepted evidence. Project
`prj_3a086024bffea4d334bc246eae9a8667` contains only the public quantitative
education excerpt packet and coordinator-owned continuity files.

The actual Codex session `01a0fcc5-3fe8-7213-8aac-edf613d3a228` runs with observed
`gpt-6.1-sol / low`, read-only sandbox and approval `never`. Process-local Full MCP
uses the copied candidate and isolated project state with a read/preview tool
allowlist. User configuration, apps and Plugins are disabled for these processes;
this is an actual Host/native-MCP journey, not an installed-Plugin qualification.
The requested sub-agent `/root/handoff_persistence_check` checks execution
identity, tool calls and raw outputs. Resumed answers share the author's
conversation; source appraisal is coordinator review, not independent domain
review or blind evaluation. Predeclared criteria SHA-256:
`3bfa57feb42990f6a9d0300b3082acb8fc1239edfb2995bb390c134f6a9e8039`.

The first Host answer preserves Q-C1–Q-C3, tentative DEC-Q1 and
Ritchie2013Retrieval, with all ten handoff sections and excerpt-only limits.
Answer SHA-256:
`b9f70fc1a2eb9a9384c858ebae20ee351d87c7b51cb84a26ec4113bacb93f95b`.
Its original allocation-conflict capture remains unmerged: native consolidation
reports `contradiction-requires-resolution`, and an attempted approved fixture
apply refuses with all project/state bytes unchanged. After checking the actual
source, the coordinator prepares a separate refinement recording supported
findings and the still-open allocation limitation; no concealed-randomization
claim, original-capture resolution or locked-decision transition is adopted.
The preserved original answer is distinguished from the reviewed continuity
entry by a historical-context preface and corrected relative Markdown links.

The existing intake/consolidation preview, dual approvals and CAS save the
reviewed continuity at revision 2. All three artifact digests match preview and
receipt, including handoff SHA-256
`5ee474282ab84bee9818667a9b41f069782fdd429d6f969cfd3b961558ef9308`.
A fresh Codex process resumes the same session, reads live project metadata,
canonical state/decisions/handoff, receipt and source, and independently computes
matching file hashes. It correctly distinguishes historical absence statements
from current saved files; no stage summary exists. Resume answer SHA-256:
`6f3d2d58db3157f5682572116f379fe3570b01457d93dedeebff904b92e4f226`.

The coordinator then preserves exact R1 bytes as `sources/r1-original.md` and
expands `sources/current.md` with previously withheld paragraphs from the same
public article XML. This is expanded reading scope, not a publisher correction.
Packet SHA-256 changes from
`8cdb89377b9991d0dda5192e10e4e4cd93d55f3181c2e6ac3ae9bcb462af3917` to
`8d3e35193922b5da1a33d98845971397f0020abe8b52e44d8c3a68a721e6852e`;
all 11 other project/state files remain byte-identical and revision stays 2.
This deliberately exercises the separate source-byte check for unregistered
attachments rather than attributing arbitrary-source coverage to native CAS.

Another fresh process resumes that session at revision 2 and detects both packet
hashes. It preserves Q-C1, reopens Q-C2/Q-C3/DEC-Q1, and names affected entries in
all three continuity files. Adjusted retrieval p=.01 remains alongside unadjusted
p=.14; adjusted interaction p=.001 is qualified by unadjusted p=.41. The response
does not infer equivalence from nonsignificance or validate a learning-phase
covariate as pretreatment ability. Supplementary GLMM agreement stays an author
report, with Text S1/Table S5 explicitly unread. No hidden expected values or
answer rubric were provided to this turn; it did receive a clarification
distinguishing open method limits from formal locked-decision conflicts.
The unchanged actual preview draft is `refinement`, based on revision 2. Answer
SHA-256: `bbcf4a17325bc67f7436b47e5fc23434cf7434d37a0b4ce463b608ed372d6fef`.
All 13 project/state files remain byte-identical during this Host process.

After checking sources, IDs, ten sections and resolving Markdown paths, the
coordinator retains the exact Host handoff and teaching-prose candidate with a
review/pre-save attribution preface. Existing native owners save this second
entry at revision 3. Preview, receipt and disk digests agree; all prior bytes in
research state, decisions and handoff remain exact prefixes. Final handoff:
14,484 bytes, SHA-256
`d10d50afdad57e173ce5f21f26ef66a93fab2e62ffb9ccb8e5d7f82b9da52a35`.
A fresh native MCP read observes revision 3; no fourth model/Host restart is
claimed. The stage remains literature, DEC-Q1 remains tentative, and the original
unmerged conflict is retained. No stage summary, Graph rebuild or accepted
research stage is produced by saving continuity.

All three successful Host processes are observed as `gpt-6.1-sol / low`; each
first sandbox-network attempt failed before tool use and is retained separately.
Normal approved network escalation allowed the subsequent processes to finish;
no evidence or failed attempt was substituted. This journey adds no provider
search/fulltext transport, table inspection, data reanalysis or expert review.

One demonstrated tooling defect is repaired in the existing handoff auditor:
valid H3 sections nested within a captured handoff previously failed its H2-only
heading check. It now accepts H2–H6 with horizontal whitespace, preserving the
seven legacy required headings and missing-section failure. This remains a
structural heading check, not a ten-section scholarly-completeness validator or
per-entry history audit. Four focused unit tests and both saved-handoff audits
pass. The requested lightweight sub-agent reviews the two-file fix without
actionable findings and reuses unchanged test evidence. Seven roadmap checks,
generated-index consistency, the frozen-source guard and diff checks pass.
Native/content bytes did not change; prior checks and pack identity are reused.

Next bounded follow-up: versioned stage-summary persistence/history through
existing write owners, with source-change and restart coverage. Installed-Plugin,
domain-expert, authenticated-provider and fulltext-transport qualification remain
separate gaps; previous wider CLI Clippy toolchain debt remains unchanged.
Integration updates only CLI-405 progress: all 249 task states/dependencies and
46 accepted rows stay fixed. No user-profile update, private-library access,
publication or program acceptance is claimed.

## October 2 — versioned stage-summary persistence and history

The maintainer selects the preceding next increment. Implementation
`683169cf5b0af2839ebb5ca796d514c321ccad0d` extends the existing capture
consolidation with optional `--stage-summary-file` on preview/apply. Its strict
version-1 JSON draft names a new summary ID, document status, actual source
hashes, optional immediate predecessor/hash and reviewed Markdown. The canonical
stage-consolidation reference documents the format and its limits; the native
runtime contract records the write and compatibility boundary.

One transaction creates `context/stage_summaries/<ID>.md`, appends the exact
history-table row in research state and adds a handoff link, alongside any
existing decision update, receipt and manifest. Preview exposes all three exact
continuity contents. Existing summary paths are refused even for identical bytes;
old documents and history rows remain intact. Listed source and predecessor
bytes are checked at preview and apply, separately from the registered semantic
revision. Strict paths, bounded reads, duplicate-key rejection, dual approvals,
plan binding and library/project CAS retain their existing owners. Sources are
limited to 64 project-local files, 4 MiB per file/draft and 16 MiB per source set
including the predecessor. External/omitted sources remain review obligations.

The service journey saves two versions across a fresh service instance, expands
an unregistered source without changing the project revision, rejects changed
predecessor bytes, and preserves the first document and prior history after the
second save. Negative cases cover changed sources/drafts, existing destination,
wrong plan digest and either missing approval. A six-file transaction exercise
checks explicit rollback. Create-only publication and rollback preserve a
competing file even when its bytes match; unattempted targets are excluded from
rollback. The collision check models the write interleaving directly; it is not
a concurrent-process stress test or a crash-recovery qualification.

All 194 project tests pass, with one existing capacity test ignored. Three
copied-CLI journeys pass on macOS arm64 with empty PATH and isolated state:
default consolidation, optional handoff and optional summary. Separate processes
perform preview/apply and revision-bound handoff reads; saved summary/state/
handoff bytes match preview and the receipt records the actual summary hash.
Draft/source drift, incomplete approval, held lock, a killed waiting writer,
altered review timestamp and replay retain the expected bytes. These synthetic
storage fixtures do not establish research-stage completeness or an actual
model's summary quality. Fixture-only source restoration is not recovery advice.

The requested native sub-agent `/root/handoff_persistence_check` ran as
`gpt-6.1-sol / low`, added and ran four strict-input/history tests, then reviewed
the final runtime, CLI integration coverage and documentation without actionable
findings. It did not launch another Host or perform independent domain review.
The coordinator ran the full project tests, three CLI parser tests, the three
copied-CLI journeys, two embedded-pack/registry checks and 14 focused
handoff/continuity/resource-link checks. Project all-targets Clippy, touched-file
rustfmt and diff checks pass. The prior wider CLI Clippy toolchain debt remains
separate and was not rerun or claimed green.

Regenerated-pack candidate `df0b42f2fe000edf331904d7de1a7ce329f7ea95` binds the
implementation commit above and contains 446 resources. Content root:
`1fca45bb4038a0f25cc8a9e9c620e19a92f886f153958665ccb8efeb29aafd01`;
pack: `f6ca963a247708ed793ae7d0133af741b5f8ff1d5941e59a3029579e2403fc08`.
The tested local CLI SHA-256 is
`0fef9a01064dca078caad66bf4d15c7568dab5dd30bb490a1c3ca874a2b41f36`.
No installed user profile or model setting changes. The option extends receipt
artifacts; older readers cannot consume `stage-summary` receipts. Existing
no-option output and plan semantics are retained. Summaries remain continuity
documents, not new Graph authority, accepted evidence or an automatic stage
advance; no new MCP endpoint or storage subsystem is added.

Next: an isolated configured-Host public research journey that creates successive
summaries, resumes from actual saved files and rechecks changed sources.
Installed-Plugin, expert review, authenticated-provider and fulltext-transport
qualification remain separate gaps. Integration updates CLI-405 progress only;
seven roadmap checks, generated-index consistency and the frozen-source guard
pass, with all 249 task states/dependencies and 46 accepted rows unchanged.
No private-library access, publication or program acceptance is claimed.

## October 2 — actual Host versioned-summary save and source continuation

The maintainer selects the preceding next increment. Baseline
`cd18e16d0f2827b319bdf9ac15f1636869ea348c` supplies the unchanged copied CLI,
SHA-256 `0fef9a01064dca078caad66bf4d15c7568dab5dd30bb490a1c3ca874a2b41f36`,
and the previously recorded 446-resource pack. The existing isolated public
project `prj_3a086024bffea4d334bc246eae9a8667` resumes from actual revision-3
state, decisions and handoff, with no summary present. New raw observations are
retained under `packages/qiongli-native/target/handoff-host-fncseb5l/summary-continuation`;
they are local development evidence, not durable accepted evidence. The earlier
trial, source packets and outputs remain separately identified. Predeclared
criteria SHA-256:
`407e4bfccbdc33de9e3460b47317df64d8a4c48542955a33b5146796974cdcf8`.

Actual Codex session `01a0fd9f-5453-71c2-8b36-2d4bee06d0d5` uses
`gpt-6.1-sol / low`, read-only sandbox and approval `never`. Process-local Full
MCP exposes only config/project/artifact/coverage reads with the copied candidate
and isolated state. Apps, Plugins and user configuration are disabled; this is
not an installed-Plugin observation. The first sandbox-network attempt fails
before tool use and leaves project/state unchanged. Its original logs remain;
normal network escalation permits the subsequent actual processes. No user
profile, model configuration or credential-store changes are made.

The first author reads and hashes all three continuity files and the R2 excerpt
packet. Its partial `STG-B-001` preserves Q-C1–Q-C3, tentative DEC-Q1,
Ritchie2013Retrieval, adjusted/unadjusted results, source anchors and open
allocation, denominator, covariate, clustering and mechanism limits. It
explicitly attributes original-unmerged status to the continuity records rather
than claiming direct inspection. Answer SHA-256:
`b673fe6c058ba959ea9eae17b2f62f6e9ebf8d0063dc1570caf426a4ff19d812`.
The coordinator checks those claims, input hashes and relative links, then adds
only a preface identifying the retained author text as revision-3 pre-save
observations. Native preview, dual approval and CAS persist revision 4; exact
preview, receipt and disk bytes agree. Summary SHA-256:
`15f37b016b1b9136d56c196c98ac233a3edd504ae885668f0ef6db45c2fe1383`.

A fresh process resumes the same author session and reads actual revision 4,
canonical history/handoff, summary, receipt and sources. It verifies all three
receipt artifact hashes. State/handoff input hashes differ because of the
expected save, while decision/source bytes match the pre-save basis; it correctly
distinguishes that from research-source drift. Answer SHA-256:
`7e32d42f80dbdaa5c6ecf3a89418392400977452b9396fe208841fa05f53c5ce`.
Both author and resumed processes leave their starting project/state bytes
unchanged. Saved continuity grants no subsequent approval or stage acceptance.

The coordinator prepares a clearly labeled synthetic second-summary negative
fixture through actual capture intake and consolidation preview. The R2 source
is then preserved as `sources/r2-before-tables.md`, and the current packet gains
links to the already retrieved same-paper tables/supplement packet and manifest.
Current-source SHA-256 changes from
`8d3e35193922b5da1a33d98845971397f0020abe8b52e44d8c3a68a721e6852e` to
`84b915d5c736009802548084a94818ded2b9d6373442601a7e8c91b991f1371b`.
The table packet and manifest remain exact copies of the prior public corpus,
SHA-256 `3eca31d16e9ddf2803697236fe9f904c3a86f4743756f8b0bb930c92d4b7ac34`
and `f865b00eac01a6903974317f19a9f535e2819da276730b2fdb1323adfaca8530`.
Only the current packet changes among existing files; 18 other files remain
identical and native revision stays 4. This expands local reading scope, not the
paper, and adds no new network retrieval or original image/DOC inspection.
Approved application of the stale preview returns `project-revision-conflict`
without changing any of the 22 project/state files. The rejected synthetic
capture remains unmerged; no second summary is written and no source is restored
to stale bytes to force acceptance.

Another fresh process resumes that author session at revision 4 and detects the
changed packet despite current native artifacts. It reads nine actual project
files, including the first summary/receipt, preserved R2, table packet and
manifest; all declared input and predecessor hashes match. The selected tables
supply reported recall cells and model details, narrowing the earlier absence
statements without erasing them. It retains S4's printed Primary 5 total 38
alongside the calculated cell sum 58, differing S2 distributions, and Table 1's
N=108 versus S5's 109 observations. Participant-ID random effects are not
misrepresented as class adjustment; supplementary-model reporting is not data
reanalysis, and the same learning-phase covariate keeps sensitivity unresolved.
Original answer SHA-256:
`e2902c500082ca9521732147b31d13de23c03d814313743eccafb46c52772399`.

The requested actual sub-agent `/root/handoff_persistence_check`, running as
`gpt-6.1-sol / low`, independently checks execution identity, tools, file hashes,
receipts and selected source-bound conclusions. This verifier has seen the
criteria and candidate; it is a dependent validation pass, not blind evaluation
or domain-expert review. It flags one missing explicit limit: reported S5
coefficients must not be read directly as marginal percentages/percentage-point
effects without verified link, coding, scaling and estimand. The coordinator adds
that qualification without inventing a link function, clarifies one S2/S4
sentence so an equal cell count is not grouped ambiguously with a differing
count, and adds a revision-4 pre-save attribution preface. The exact original
Host answer remains unchanged. No model-quality improvement is estimated.

After coordinator source review, a separate native capture/preview and dual
approval/CAS save `STG-B-002` at revision 5, including a new tentative DEC-Q1
coverage qualification. All four artifact digests match receipt, preview and
disk. The second summary is 21,395 bytes, SHA-256
`17c2f06933ceafd1044125587e7ea9d3780560021952f04ab31d9f01fe5e5b13`;
its receipt SHA-256 is
`109c900d5ddc0bfcfd67fc7428834110f6573a85f69f6287df078a61aa2cc04c`.
The first summary is byte-identical, both history rows remain, and the new
predecessor link/hash identifies the actual first document. Existing decisions
and handoff remain exact prefixes; prior research-state bytes remain a prefix
after excluding the inserted history row. Stage stays literature, DEC-Q1 stays
tentative, and the original unmerged proposal plus the rejected synthetic stale
capture remain unmerged. Saving creates no Graph authority or stage acceptance.

A fourth process resumes the same session, observes actual revision 5 and reads
both summaries, canonical continuity, the second receipt and current sources.
It independently hashes all ten inspected files, confirms all four receipt
artifact digests, both history rows and the second summary's predecessor hash.
It retains the new coverage and coefficient limitations without treating
historical pre-save statements as current absence. Final answer SHA-256:
`f9a7006dea6ec422d9d2090ddaf8bdf249d1c1f888940b4844ab5cd869ad4292`.
The verifier confirms actual `gpt-6.1-sol / low` turn contexts, four successful
read-only MCP calls per process, and unchanged starting project/state snapshots
of 15, 18, 22 and 25 files respectively. The four successful process durations
are approximately 222, 124, 292 and 93 seconds; these are observations, not a
performance comparison. An initial read-only shell heredoc attempt could not
create its temporary file; subsequent direct reads succeeded without weakening
the sandbox. The original failed attempt is retained in the Host events.

Final coordinator assertions and the requested lightweight verification pass
confirm both saves and restarts, source-drift refusal, retained original files,
actual receipt/source hashes and unresolved scholarly limits. No runtime/content
repair is demonstrated or introduced; the previous affected native/content test
results and pack identity are reused. Seven roadmap checks, generated-index
consistency, unchanged-state/accepted-row checks and the frozen-source guard
pass. Integration changes CLI-405 progress only: all 249 task states/dependencies
and 46 accepted rows remain unchanged. The earlier wider CLI Clippy toolchain
debt remains separate.

Next bounded increment: diagnose the remaining public fulltext transport failure
and its user-facing error path through the existing reader. Preserve DNS,
redirect and network guards; do not infer a need for more permanent agents.
Installed-Plugin, authenticated-provider, original table-file/data reanalysis and
domain-expert qualification remain separate gaps. No private-library access,
user-profile installation, publication or program acceptance is claimed.

### October 2 — public fulltext transport diagnostics

The bounded follow-up starts from clean local `2.x` at
`c82b5d5bf540a995003f573962b1829b435408fe`. Implementation
`194b4eaa0a0a1406f4f3b4a3d5c71621fdf180e3` corrects the shared reader's failure
reporting; regenerated-pack candidate
`a9e462c5013874a14d8d821eba242f544297af10` contains 446 resources, content root
`5f6d8598647297d0e646b7bcd5a1d7d041b9a18bf73d1a76b6d017719841a9e4` and pack
`0434fcd3cff6f90912953d3cda764c9a2c2ab57c92753c8f4377a96b3295ff9b`.
The copied before/after CLI hashes are respectively
`0fef9a01064dca078caad66bf4d15c7568dab5dd30bb490a1c3ca874a2b41f36` and
`a3463a9dc69a3f1ff35f16af81b3818998ad62993f068e8588417bedeb7b7479`.

Fresh authorized public observations differ from the earlier special-address
mapping. `journals.plos.org` now resolves to `35.190.43.188`; the prior CLI's Lite
and Full calls for PLOS DOI `10.1371/journal.pone.0078976` still return generic
`fulltext-url-blocked`. A separate first-hop GET, without following its redirect,
returns HTTP 302 to `storage.googleapis.com` with X-Goog signing parameters.
Only the status, hostname and query-key names are retained; signing values are
not logged. This identifies a signed-redirect policy limitation for this observed
request, not a DNS failure, bad original locator, paywall or missing article body.
The cause of the earlier system mapping and its change remains unproven.

The existing public URL policy now names signed/credential-bearing query
parameters, and the redirect owner identifies a blocked destination before
contacting it. Both actual rebuilt CLI profiles return that precise message with
the original reason code. The shared DNS result boundary distinguishes timeout,
worker disconnection, resolver error and empty answers as `fulltext-network-error`;
private/special-use and mixed public/nonpublic answers retain
`fulltext-url-blocked`. Static messages do not expose addresses, locators or
underlying errors. Native Lite, Full and standalone Lite reuse the same owner.
Canonical fulltext guidance explains the failure stage and safe evidence limit.
No URL allowlist, DNS/proxy change, signed-URL exception, redirect relaxation,
new endpoint, dependency or provider credential access is introduced. Request
send/body error classification remains unchanged and is not newly qualified.

One additional public native Lite read of `https://arxiv.org/pdf/1706.03762`
succeeds in approximately 2.18 seconds, parsing 27 PDF segments and returning
segment 0. A fresh process continues at the returned offset 1 with limit 3 and
`expected_sha256`, returning segments 1–3 in approximately 4.24 seconds with the
same decoded source digest:
`bdfaa68d8984f0dc02beaca527b76f207d99b666d31d1da728ee0728182df697`.
Page-2 anchors expose Introduction/body passages, beyond the initial abstract.
Both reads report `cached: false`; this is digest-checked retrieval across
processes, not a cache demonstration. The title is visible in returned text,
but structured identity remains `not_checked`. Only four segments are exposed;
no complete reading, verified paper version, table/formula extraction, scholarly
claim, research-quality gain or permission to redistribute follows. This current
public-source success is not caused by the diagnostic-only patch and does not
resolve PLOS signed redirects or authenticated providers.

Raw local observations and temporary runners remain in
`packages/qiongli-native/target/fulltext-transport-uz6sctkf/`:
`before.json` SHA-256
`1384c45224a8ef9e695c185274e1d438bb03e47ed1da94d3987faa03e467c894`,
`first-hop.json` SHA-256
`a346b115a810f1356b1ffa760c7a1945ee5c6d73e203f486705b54bf4b1ac75b`,
`after.json` SHA-256
`17a38f19eb85af8812eea471bfbf38eb1c4eec2865eeacf855c1843abf6ff5f8`,
`after-arxiv.json` SHA-256
`215123679369bc5f9f6672260546c872779aab1faf5b7c9fb6849c10316685bb`,
and `after-arxiv-continuation.json` SHA-256
`db9897f8a6c13ea5eba3745c19607e93c340b6f94d22027e17eac8a6205c1083`.
These copied-CLI observations use isolated Qiongli configuration and no canonical
research writes; they are not installed-Plugin or release acceptance.

Validation: seven shared fulltext tests pass (one worker probe remains ignored),
including DNS result classification, mixed-address refusal, redirect policy,
redaction, parser isolation and existing source/cache boundaries. The requested
actual `gpt-6.1-sol / low` sub-agent `/root/handoff_persistence_check` reviews the
source diff without actionable findings and independently runs the native and
standalone Lite fulltext compatibility tests (one pass each). Thirty-seven
focused literature/content/MCP checks, runtime all-target Clippy with warnings
denied, formatting and the rebuilt embedded-pack/registry test pass. The initial
system-Python run lacked PyYAML; the existing `.venv` supplies it without an
installation. The broader research-standard validator reports 6,100 passes,
24 failures and 17 warnings in unmodified generated-doc/workflow/release and
legacy-contract checks; it is not reported as green or repaired in this scope.
The CLI build also retains the unrelated platform `fetch_update` deprecation.
The verifier also checks all five observation JSON files without a mismatch;
process launch and continuation binary identity additionally rely on the
coordinator's retained runner/execution record, not the continuation JSON alone.
Seven roadmap tests, generated-index consistency, ledger invariance and the
frozen-source guard at the pack candidate pass. All 249 task states/dependencies
and 46 accepted rows remain unchanged; only CLI-405 progress is extended.

Next exercise the working native public-PDF route in an isolated Host journey,
carrying source digests and actual body anchors into reviewed paper notes through
existing write owners. Signed publisher redirects, authenticated-provider and
installed-Plugin qualification remain separate. No user profile/model, private
library, publication, task-state/dependency or accepted-evidence change is made.
