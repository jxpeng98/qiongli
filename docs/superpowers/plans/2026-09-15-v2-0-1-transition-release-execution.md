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
