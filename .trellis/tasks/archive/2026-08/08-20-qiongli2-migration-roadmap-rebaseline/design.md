# Design: replacement-first roadmap and tiered verification

## Problem statement

Qiongli 2 needs a shorter path to becoming the dependable replacement for
Qiongli 1.19. The current program mixes that replacement path with a much larger
research-harness program, while routine task closeout and Native CI repeatedly
run evidence that belongs to package or release acceptance. The result is slow
business feedback, high agent-token use, and technical acceptance claims that
do not always match the user's end-to-end experience.

The minimum correction is to narrow the 2.0 critical path and make verification
scope explicit. It is not to add another roadmap, test runner, graph system, or
compatibility framework.

## Authority boundaries

| Decision | Existing owner | Change |
| --- | --- | --- |
| 1.x/2.x branch roles and 1.x support window | `docs/maintainer/release-branch-policy.md` | Reuse; clarify verification tiers in English and Chinese copies. |
| Long-term ordering and task descriptions | Master roadmap | Rebase around replacement-first 2.0 delivery. |
| Live task state and exact evidence | Program Ledger v1 | Preserve accepted rows; update ordered unaccepted rows and add only missing recovery owners. |
| Generated current view | `qiongli-current-program-index.md` | Regenerate; never edit directly. |
| Local evidence selection | Product-control spec | Name the three verification tiers and risk exceptions. |
| Trellis phase/check behavior | `.trellis/workflow.md` and `trellis-check` | Make the selected tier explicit and keep output compact. |
| Automatic 2.x source checks | `Native CI` | Preserve the four required branch-protection contexts. |
| Candidate/package/promotion evidence | Existing acceptance and promotion workflows | Run only through an explicit candidate action. |

Historical acceptance ledgers remain evidence for their exact source and scope.
They are not rewritten when a wider user journey is still unverified.

## Roadmap shape

### 2.0 critical path

The revised roadmap expresses three outcome gates before the existing advanced
research program:

1. **Replacement truth**
   - Surface the existing 1.19 critical-fix-only policy and 90-day post-Stable
     support window.
   - Keep the 16-outcome 1.x parity ledger truthful as a bounded classification
     contract; do not call it complete product parity.
   - Add one concise replacement matrix for CLI, Plugin, Skills, Lite/Full MCP,
     Zotero, App, and Graph v1. Each row names the exact 1.19 oracle, current 2.x
     evidence or gap, cutover requirement, and owning roadmap task.

2. **Replacement implementation and dogfood**
   - Close the shared native CLI -> Plugin/Skills -> Lite/Full MCP -> Zotero
     behavior before treating App presentation as proof.
   - Stabilize App as a client of the same native owners.
   - Prove Graph v1 with one representative migrated project: source-bound
     scholarly nodes, non-containment relations, useful query/visualization,
     deterministic rebuild, and truthful empty/sparse states.

3. **Cutover**
   - Freeze one exact 2.x candidate only after the replacement matrix is green.
   - Prove packaged installation, current Codex/Claude integration, migration,
     rollback, and supported-target behavior for that candidate.
   - Release 2.0, begin the existing 90-day 1.x support countdown, then end 1.x
     maintenance unless an explicit later support decision supersedes it.

### Post-2.0 program

Graph v2, the Typed Research Kernel, Kernel-dependent Evidence/Reproducibility
objects, institutional research modes, and broad extension/collaboration work do
not block the 1.19 replacement. Existing unaccepted task IDs remain traceable
but move behind the 2.0 cutover or become explicitly deferred/superseded.

Accepted M0/M1 evidence remains in its historical location. Unaccepted rows may
be reordered. Add only `GOV-320` for replacement truth and tiered verification,
`PLT-320` for the shared CLI/Plugin/Skills/MCP/Zotero vertical, `PLT-321` for App
stability on native contracts, and `PLT-322` for Graph v1 migrated-project
acceptance. Reuse the existing `REL-*` tasks for cutover. The authoritative
inventory therefore grows from 233 to 237 IDs without duplicating the backlog
in Trellis.

The replacement matrix is a planning/review surface, not another live task
state machine. Program Ledger v1 remains the only owner of task state.

## Verification model

| Tier | Trigger | Required scope | Excluded by default |
| --- | --- | --- | --- |
| **Focused** | Every implementation loop | Smallest test that can falsify the changed behavior; targeted negative checks for security, data loss, schema, path, ownership, or trust changes | Unrelated packages, full workspace, package assembly, live Hosts |
| **Slice** | One complete user-visible business vertical is frozen for PR/integration | All affected package checks, cross-layer contract check, full task-scope regressions, and the four required exact-head Native CI contexts | Three-target product packages, packaged acceptance, promotion, public-release receipts |
| **Acceptance** | Explicit 2.x cutover or release candidate | Exact source, workspace/source matrix, target packages, packaged product, real Hosts, migration/rollback, trust/supply-chain and claimed manual journeys | Nothing required by the candidate's declared claims |

“Full-scope” at the end of a Trellis task means the whole task and every affected
package at Slice tier. It does not mean unrelated repository or release tests.

If a higher-tier run fails, return to one focused reproduction, fix it, and then
rerun the invalidated higher-tier job. Do not repeatedly stream or rerun every
successful job.

## Token and output contract

- Prefer non-verbose/default reporters for successful local checks.
- Record command, tier, result, and concise counts in the final check report.
- Include detailed output only for the first actionable failure; give the
  minimum focused reproduction.
- Do not fetch or repeat successful CI logs. Use job/check summaries and open
  only the failing step when diagnosis is required.
- Full logs remain CI artifacts; they are not copied into task documents.

This changes agent behavior and check selection without adding a log parser or
wrapper script.

## Native CI boundary

Keep these required Slice contexts and their names unchanged:

- `Native 2.x change boundary`;
- `Rust native foundation (Linux)`;
- `Rust native foundation (macOS)`;
- `Rust native foundation (Windows)`.

Within the Rust matrix, run portable App API/Desktop/npm checks once on Linux;
macOS and Windows retain target-native Rust checks and the static Desktop build.

Gate these existing jobs to explicit `workflow_dispatch` candidate runs:

- `desktop-package-assembly`;
- `packaged-product-acceptance`;
- `lite-alpha-candidate-acceptance`;
- the dependent Community Alpha promotion dispatch.

`lite-runtime-compatibility` remains a bounded Slice check. Evaluation Truth
remains automatic because it is small and owns roadmap/governance fail-closed
checks. The explicit promotion workflow continues to verify the named Native CI
run and exact current `2.x` source before rebuilding targets.

## Compatibility and migration

- No user project, Plugin, Skill, Graph, or App data migration occurs in this
  governance task.
- The four protected-branch required context identities do not change.
- The 1.x frozen branch and accepted parity artifacts remain read-only.
- A changed product candidate still invalidates exact package/release evidence;
  the tier policy changes *when* that evidence is requested, not its rigor.
- `.trellis/workflow.md` and `trellis-check` are locally editable Trellis
  templates. The project intentionally owns this customization; a later
  `trellis update` may flag it as locally modified and must not overwrite it
  silently.

## Task and rollout structure

Keep this as one implementation task. Roadmap gates, local check semantics, and
Native CI promotion behavior must change together or the documentation will
describe a workflow the repository does not enforce. Missing product features
discovered by the replacement matrix become separate, bounded follow-up Trellis
tasks, one active implementation task at a time.

Rollout order:

1. freeze the revised authority and replacement sequence;
2. update local Trellis verification semantics;
3. gate expensive CI acceptance jobs;
4. update bilingual branch-policy guidance and executable policy tests;
5. regenerate and validate the current program index.

## Risks and rollback

- **Advanced trust work deferred too far:** security, authorization, data-loss,
  schema-compatibility, and release-trust blockers remain 2.0 gates even when a
  larger Kernel-dependent feature moves post-2.0.
- **Candidate acceptance forgotten:** the roadmap and release policy make an
  explicit Acceptance run mandatory before cutover; ordinary green Slice CI
  cannot authorize a release.
- **Branch protection drift:** required check names remain unchanged and policy
  tests assert them before merge.
- **Skipped-job dependency error:** focused workflow tests assert that expensive
  jobs are skipped for PR/push and included for explicit dispatch.
- **Roadmap/ledger mismatch:** update all three authority files together and run
  the existing deterministic generator check.

Rollback is a normal revert of the roadmap/ledger/Trellis/CI policy change,
followed by regenerating the current index. No user data rollback is required.
