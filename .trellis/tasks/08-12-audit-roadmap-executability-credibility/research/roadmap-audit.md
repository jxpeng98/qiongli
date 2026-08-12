# Qiongli 2 Roadmap Executability and Credibility Audit

- Audit date: 2026-08-12
- Local audit snapshot: `255baa3ec430efdc837748a6676b547163ba4416`
- Working branch: `fix/alpha3-codex-claude-host-qualification`
- Declared target branch: `2.x`
- Current local/remote `2.x`: `6494b004ac141bcef0e6a799552d4b63720b7b7f`

## Executive verdict

The roadmap is still useful as a **strategic dependency and release-safety
document**, but it is not currently trustworthy as the program's single live
status authority.

- **Executability: 3/5 overall.** M1 is actionable and has one ready-now item,
  but later milestones are progressively dependency-bound, and most tasks do
  not yet name an implementation owner, exact write set, or runnable command.
- **Live-state credibility: 2/5.** The release non-claim is well supported, but
  current task state is split across a local feature branch, stale roadmap
  prose, stale GitHub Epics, and a machine-readable program ledger that the
  roadmap says exists but the repository does not contain.
- **Strategic/evidence-model credibility: 4/5.** The dependency sequence,
  exact-source invalidation rules, authority separation, and M0 release
  non-claims are coherent and independently supported.
- **Confidence: high for repository, branch, Issue, run, tag, and Release
  findings; medium overall.** GitHub Project V2 custom fields could not be read
  because the authenticated token lacks `read:project`, so Project field values
  remain unverified rather than failed.

No P0 release-safety defect was found. The roadmap and acceptance ledger both
continue to forbid Alpha 3 publication, and GitHub confirms that no Alpha 3 tag
or Release exists. The P1 defects are governance defects: they can select the
wrong next task or overstate acceptance if the roadmap is treated as live state.

## Rating rubric

| Score | Meaning |
|---:|---|
| 5 | Current owner, dependency, executable validation/evidence, and exit gate are exact. |
| 4 | Actionable now; only bounded Trellis decomposition is needed. |
| 3 | Coherent, but one material contract, baseline, owner, or evidence source is missing. |
| 2 | Multiple dependencies or authority decisions remain unresolved. |
| 1 | Directional/aspirational; no credible current execution boundary. |
| 0 | Internally contradictory or disproven. |

Severity in this report means:

- **P1:** can misstate accepted/current work or send execution down the wrong lane;
- **P2:** materially stale or unverifiable, but bounded by a stronger safety gate;
- **P3:** structural/documentation hygiene with low execution risk.

## Material findings

| ID | Severity | Location | Verdict | Evidence and impact | Smallest correction |
|---|---:|---|---|---|---|
| F1 | P1 | Roadmap lines 283-300 and 421-424 | contradictory | The Project is declared to be derived from a machine-readable program ledger, and detailed task state is said to remain in that ledger. Repository-wide inspection found no such program ledger, while `GOV-401` explicitly remains open. There is therefore no authority that can enforce the six states or exact evidence requirements. | Implement `GOV-401` through `GOV-403` as one validated ledger foundation before making more checkbox-only status updates. Leave generated roadmap output to `GOV-404`. |
| F2 | P1 | Roadmap line 9 and lines 397-407 | partially-supported | EVAL-401 through EVAL-407 are implemented, committed, archived, and green on the local audit branch. Their commits (`be7422be`, `ae4dfaef`, `33f9d6a0`) are not ancestors of `origin/2.x`, whose current SHA is `6494b004`. Thus the checked state is true for this branch but not yet integrated into the roadmap's declared target branch. | Record these items as local/active evidence until the protected `2.x` delivery decision is completed; accept them only from the resulting integration identity. |
| F3 | P1 | Roadmap lines 35, 297-300, and 333-334 | contradictory | The roadmap contains **233** unique checklist task IDs, not 232. All IDs are unique. The 32 GitHub Epics contain 232 unique task IDs exactly once, with no duplicates or extras, but omit `REL-300`. | Make the ledger's ID inventory authoritative, then either add `REL-300` to the owning M0 Epic or explicitly classify it as a non-program historical marker and remove it from the counted checklist. Update both `232` claims from generated data. |
| F4 | P1 | Roadmap lines 89-91, 327-338; acceptance ledger lines 117-145 | partially-supported | `REL-301` through `REL-304` have valid historical evidence for exact source `cced6082`: Native CI `31438158969` succeeded and promotion run `31439930097` produced three target inputs plus an aggregate candidate. The later Full/Lite route fix changed product input, so that candidate is not current or reusable for release. Bare checked boxes cannot express “historically completed, currently invalidated.” | Preserve the historical receipts, but represent current release qualification as reopened/blocked in the program ledger. Do not erase or reuse the old evidence. |
| F5 | P2 | Roadmap lines 116-120 versus 397-407 | stale | The confirmed-gap table still says an empty eval can pass and YAML validation is non-executable. Current code and 13 focused tests prove the EVAL-401 through EVAL-407 false-green and typed-validation path is implemented on this branch. The separate metadata-only academic-quality gap remains open under EVAL-409. | Split repaired eval gaps from the still-open EVAL-409 gap, with branch/integration status explicit. |
| F6 | P2 | Roadmap lines 265-266, 292, 307-310, 375-391, and 1144-1150 | stale | The document still names the archived Alpha 3 qualification task as active and directs the next Trellis task to EVAL-401 through EVAL-405. Those tasks, EVAL-406, and EVAL-407 are archived. The first-90-days section repeats the old starting point. | Regenerate or reconcile the “current/next” projection after the ledger exists; the next technical EVAL item is EVAL-408. |
| F7 | P2 | GitHub Issues #83-#87 | stale | All 32 Epics remain open and were last updated on 2026-08-03. M1 Epics still say `Proposed`; Issue #83 says it is blocked by “M0 accepted,” while the roadmap explicitly permits M1 code work with external M0 evidence open and seven EVAL items are locally complete. | Reconcile Epic state/dependency text from the ledger after integration; do not infer state from open/closed Issue status. |
| F8 | P2 | GitHub Project V2 #1 | unverified | Issues, PRs, Actions, branch, tag, and Release were readable. Project V2 fields were not: GraphQL reported that `read:project` is required. | Re-run only the Project field audit with a read-only `read:project` token; no write scope is needed. |

## Structural integrity

### Checklist inventory

| Milestone | Total | Checked | Open |
|---|---:|---:|---:|
| M0 | 16 | 6 | 10 |
| M1 | 42 | 7 | 35 |
| M2 | 29 | 0 | 29 |
| M3 | 41 | 0 | 41 |
| M4 | 37 | 0 | 37 |
| M5 | 35 | 0 | 35 |
| M6 | 24 | 0 | 24 |
| M7 | 9 | 0 | 9 |
| **Total** | **233** | **13** | **220** |

- Checklist IDs: 233 occurrences, 233 unique, zero duplicate IDs.
- GitHub Epic IDs: 232 occurrences, 232 unique, zero duplicate IDs.
- Roadmap IDs absent from the Epics: `REL-300` only.
- Epic-only IDs absent from the roadmap: none.
- Local Markdown links: both relative targets exist.
- Headings: `Exit gate` repeats six times, once per applicable milestone. This is
  intentional contextual reuse, not a conflicting duplicate section.
- Milestone ordering and the M0 -> M1 -> M2/M3 -> M4 -> M5 -> M6 -> M7
  dependency spine are internally coherent.

### Live GitHub evidence

- `2.x` is protected and currently points to `6494b004ac141bcef0e6a799552d4b63720b7b7f`.
- Native CI run
  [31438158969](https://github.com/jxpeng98/qiongli/actions/runs/31438158969)
  completed successfully for exact SHA `cced60826ac4d7dad596669103a7e15b61868e81`.
- Promotion run
  [31439930097](https://github.com/jxpeng98/qiongli/actions/runs/31439930097)
  bound the same SHA; exact-head verification, macOS, Windows, Linux, and
  aggregation jobs succeeded, while the authorization job failed. This matches
  the ledger's non-publication explanation.
- The public release list contains `v2.0.0-alpha.1`, but no
  `v2.0.0-alpha.3`; the Alpha 3 tag lookup also returns not found.
- The 32 roadmap Epics are Issues
  [#81](https://github.com/jxpeng98/qiongli/issues/81) through
  [#112](https://github.com/jxpeng98/qiongli/issues/112); all are open.

## Checked M0/M1 evidence trace

| Checked IDs | Verdict | Narrowest evidence owner | Limit |
|---|---|---|---|
| `REL-300` | supported | Archived `close-alpha3-first-usable-spine` task, exact source `cced6082`, PR/CI/promotion evidence | Internal first-use only; no target/live-Host/update/trust/publication claim. |
| `REL-301`-`REL-304` | partially-supported | Alpha 3 acceptance ledger plus runs `31438158969` and `31439930097` | Valid historical exact-candidate work; invalidated for release reuse by changed product input. |
| `GOV-301` | supported | Acceptance ledger and `CHANGELOG.md` both state `Unpublished candidate`, `publication_allowed=false`, and public Alpha 1 only | Does not create a general program-state ledger. |
| `EVAL-401`-`EVAL-405` | supported on audit branch | Archived task, commit `be7422be`, Evaluation Truth V1 spec, focused tests | Not integrated into `origin/2.x`. |
| `EVAL-406` | supported on audit branch | Archived task, commit `ae4dfaef`, seven executable validator paths and negative tests | Not integrated into `origin/2.x`. |
| `EVAL-407` | supported on audit branch | Archived task, commit `33f9d6a0`, JSON/JUnit contract and deterministic/redaction tests | Not integrated into `origin/2.x`. |

The audit re-ran `python -m unittest tests.test_eval_cases -v`: all 13 focused
tests passed. Archived tasks record broader repository passes, but this audit
did not re-run the full suite and does not promote those historical results to
new exact-head release evidence.

## Milestone executability

| Milestone | Classification | Score | Basis and current blocker |
|---|---|---:|---|
| M0 / Alpha 3 | dependency-blocked | 3/5 | Exact gates, evidence owners, and publication order are unusually strong. Resumption requires a new exact candidate, a CLI-size decision, target/manual receipts, two live-Host receipts, update/rollback proof, and independent release authority. Old candidate evidence cannot satisfy the new source. |
| M1 / Alpha 4 | ready-now plus planning-needed | 4/5 | EVAL-408 has an existing runner/test owner and a bounded outcome. Governance, baseline, and threat-model work is concrete but needs Trellis decomposition. The missing ledger and unintegrated EVAL commits prevent a clean live-state view, not local implementation. |
| M2 / Alpha 5 | dependency-blocked | 3/5 | Kernel/evidence objects and exit invariants are detailed. Work must wait for M1 schema authority, program-state truth, and evaluation gates; exact storage/serialization/migration owners are intentionally not frozen yet. |
| M3 / Alpha 6 | dependency-blocked | 3/5 | Reproducibility, Gate, orchestration, and adversarial deliverables are test-shaped and measurable, but they depend on accepted Kernel/evidence identities and reporting profiles. |
| M4 / Beta 1 | dependency-blocked | 3/5 | Platform tasks are concrete, but cache/delta/job/UX changes require M1 measurements and M3 contracts. Absolute SLOs are correctly deferred rather than invented. |
| M5 / Beta 2 | dependency-blocked | 2/5 | Outcomes and safety non-claims are credible, but data classification, ethics authority, key management, external validators, and institutional ownership require several policy/ADR decisions before implementation. |
| M6 / RC/Stable | dependency-blocked | 2/5 | The Stable gate is strong, but target signing, package-manager channels, long-duration pilots, frozen budgets, and independent reviewers do not yet have accepted inputs or named owners. |
| M7 / 2.1 | aspirational | 1/5 | It is correctly outside the 2.0 critical path, but has no milestone entry/exit gate, accepted collaboration threat model, ownership model, or bounded execution slices. It should remain a horizon, not a backlog. |

The roadmap is therefore executable as a **sequencing graph**, especially for
M1, but not as 233 ready implementation tickets. That is appropriate for M2-M7
as long as the document stops claiming to be the live task-state ledger.

## Remaining M1 readiness — 35/35 IDs

`ready-now` means implementation can enter Trellis planning immediately without
an upstream roadmap decision. `planning-needed` means the outcome is coherent
but its owner/write set/test contract must be bounded first.

| IDs | Count | Classification | Dependency, owner, and minimum executable boundary |
|---|---:|---|---|
| `EVAL-408` | 1 | ready-now | Reuse `evals/runner/run_eval.py` and `tests/test_eval_cases.py`; add the six named fixture families and prove each expected failure/reason. No new engine is needed. |
| `EVAL-409` | 1 | planning-needed | First define executable inputs and expected findings for the existing academic-quality path; keep declared-score removal and compatibility behavior in one bounded task. |
| `EVAL-410`-`EVAL-411` | 2 | dependency-blocked | Canonical-command ownership and mutation coverage should follow the EVAL-408/409 fixture corpus, otherwise they would standardize or mutate an incomplete suite. |
| `GOV-401`-`GOV-403` | 3 | planning-needed | One minimum ledger task can define the file/schema, six states, 233-ID inventory, exact-evidence rule, and validation test. It must distinguish local work from target-branch acceptance. |
| `GOV-404` | 1 | dependency-blocked | Generated roadmap/index output requires an accepted GOV-401-403 ledger first. |
| `GOV-405`-`GOV-407` | 3 | planning-needed | Architecture, ADR registry, and parity truth each have a bounded repository owner, but need exact target files and a “classification versus implementation” evidence rule. |
| `GOV-408` | 1 | planning-needed | Requires a schema-authority ADR with Rust generation and consumer/golden-fixture boundaries before code generation starts. |
| `GOV-409` | 1 | dependency-blocked | Compatibility classification depends on the accepted schema authority from GOV-408. |
| `GOV-410` | 1 | planning-needed | The roadmap already contains a candidate authorization matrix; a task must identify its canonical policy owner and reconcile existing release/research/repository rules. |
| `GOV-411`-`GOV-418` | 8 | dependency-blocked | Non-transitivity, receipts, GitHub policy, checklists, invalidation, revocation, and negative tests depend on the canonical matrix and receipt schema; remote policy changes also require maintainer authority. |
| `PLT-401` | 1 | planning-needed | Define deterministic small/medium/product-limit fixture generation from existing native limits and leave one reproducible benchmark entry point. |
| `PLT-402`-`PLT-403` | 2 | dependency-blocked | Measurements and one-over-limit rejection require the PLT-401 corpus first. |
| `PLT-404`-`PLT-406` | 3 | planning-needed | Real IPC, stale-preview, crash/lock/restart journeys can be planned now, but each needs a target package/harness owner and deterministic cleanup/receipt boundary. |
| `PLT-407`-`PLT-408` | 2 | dependency-blocked | Budgets, lock order, and cancellation contracts must be derived from PLT-402/404-406 evidence, not guessed. |
| `SEC-401`-`SEC-402` | 2 | planning-needed | One threat-model task can inventory untrusted sources and freeze the content/control boundary across Host and ToolHost owners. |
| `SEC-403`-`SEC-405` | 3 | dependency-blocked | Permission-negative tests, adversarial fixtures, quarantine, and safe inspection require the SEC-401/402 boundary and capability inventory first. |
| **Total** | **35** | 1 ready-now, 15 planning-needed, 19 dependency-blocked | No M1 item is merely aspirational, but only EVAL-408 is ready without a prior contract/dependency task. |

## Credibility assessment by claim class

| Claim class | Rating | Assessment |
|---|---:|---|
| Release state and non-publication | 4/5 | Exact source, CI, promotion, acceptance ledger, changelog, tag, and Release state agree. Historical candidate invalidation is stated correctly. |
| M0 checkbox state | 3/5 | Evidence exists, but checkboxes cannot represent invalidated historical completion versus current candidate readiness. |
| M1 implementation state | 3/5 locally; 1/5 on target branch | Commits, tasks, specs, and tests support EVAL-401-407 locally. `origin/2.x` and GitHub Epics do not yet contain or reflect that state. |
| Program task-state authority | 1/5 | The claimed machine-readable ledger does not exist; current state is manually duplicated. |
| Dependency and gate design | 4/5 | Ordering, non-claims, exact-head invalidation, and phase exits are coherent and conservative. |
| GitHub collaboration mapping | 2/5 | Issue coverage is deterministic but one ID is missing and all Epic states are stale; Project custom fields are unverified. |
| Long-horizon feasibility | 2/5 | Technically plausible, but M5-M7 require future policy, institutional, signing, pilot, and threat-model decisions. |

The roadmap can be trusted for **why the phases are ordered this way** and for
**what must not be claimed**. It should not be trusted for **which task is
currently active/accepted** until the state authority is repaired.

## Prioritized repair and execution sequence

1. **Resolve delivery identity for the current branch.** Decide whether the
   local EVAL-401-407 commits will enter protected `2.x`; until then, do not call
   them target-branch accepted.
2. **Create one validated program ledger (`GOV-401`-`GOV-403`).** Seed all 233
   IDs, restrict states, require exact evidence for accepted work, and record
   blockers/invalidation without creating another parallel status system.
3. **Derive the current roadmap/Issue view (`GOV-404` plus reconciliation).** Fix
   232/233, map or reclassify `REL-300`, replace stale task/gap/90-day language,
   and update Epic states/dependencies. Re-check Project fields with
   `read:project` access.
4. **Continue the technical eval lane with `EVAL-408`.** It is the smallest
   ready product task once state ownership is no longer being compounded by
   manual checkbox updates.

## Recommended next Trellis task

Create **“Implement GOV-401-403 program ledger v1”** as the next roadmap task,
after the current branch delivery decision is explicit.

Minimum scope:

- one machine-readable ledger containing exactly the 233 roadmap IDs;
- only `proposed`, `active`, `accepted`, `blocked`, `deferred`, and
  `superseded` states;
- required `id`, `state`, `owner`, `dependencies`, `evidence`, `commit`, `run`,
  `updated_at`, and `blocker` fields;
- validation that IDs are unique/complete and `accepted` has exact evidence;
- explicit representation of local/unintegrated EVAL work and invalidated M0
  candidate evidence;
- no generated roadmap, GitHub mutation, ADR repair, or EVAL-408 implementation
  in this first slice.

Why this task precedes EVAL-408: EVAL-408 is technically ready, but another
checkbox-only completion would deepen the root problem this audit found. A
minimal ledger foundation fixes the shared state owner once; EVAL-408 can then
be recorded without another manual-source contradiction.

## Methods and verification limits

Read-only checks used:

- deterministic `rg`, `awk`, `sort`, Git, and filesystem inspection for IDs,
  checkboxes, headings, links, branches, ancestry, commits, and task archives;
- `gh issue`, `gh api`, `gh run`, `gh release`, and `gh pr` for current remote
  Issues, branch identity, CI/promotion, tag/Release absence, and PR identity;
- focused `tests.test_eval_cases` execution: 13/13 passed;
- acceptance-ledger and changelog comparison for publication truth.

Limits:

- GitHub Project V2 fields are unverified because the token lacks
  `read:project`; this does not invalidate the independently readable Issues.
- No target-native, live-Host, update, trust, signing, or publication journey
  was re-run.
- Long-horizon ratings assess dependency and task shape, not code-line
  feasibility before entry contracts exist.
- No roadmap, checkbox, acceptance record, product file, remote Issue, Project,
  tag, Release, or public claim was changed by this audit.
