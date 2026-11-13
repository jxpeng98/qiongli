# Capture and decision records

These are required fields for externally captured observations, not a new database
or a replacement for the existing native project service. Use JSON/CSV plus the
existing raw event/answer files. An empty template is never an observed result.

## Freeze before execution

- Protocol/fixture hash and source revision; source/task/original bytes, provenance,
  exposure classification, required coverage and review criteria (review-only).
- Host version/executable identity, effective configured model/effort/settings,
  session/memory policy, timestamps and any known service-version limits. Unknown
  provider deployment/seed stays unknown. Each repeat is a separate fresh unit.
- Native source/CLI hash, content pack and installed cache/source receipts for C;
  B/C research-guidance file hashes and explicit C-only adapter/schema additions.
  A's ordinary baseline instructions/capabilities are recorded too.
- Arm-specific workspace/config, permitted reads/writes/network/tool approvals,
  real preflight and policy readback, exact generation prompts and differences,
  shuffled schedule/seed, task budget, retry policy and identity-key custody.
- Main only: six chosen tasks, three repeats, actual execution/cost ceiling,
  worthwhile-benefit thresholds and reviewer assignments. Null is **not ready**.

The preparer captures only public inputs; never dump private auth/configuration.
Record exact nonsecret selected settings, not secret hashes or credential values.
If settings drift mid-block, retain outputs and mark the comparison ineligible
until a separate consistently frozen block exists. Never silently replace results.

## One row per planned unit, then all its attempts and sessions

Use `unit_id`, `task_id`, `arm`, `repeat`, `order` and `checkpoint`. Record
`status` as `not-started`, `completed`, `partial`, `timeout`, `refused`,
`environment-blocked` or `cancelled`; preserve explanations and original files.
`not-started` is not a pass and cannot disappear from planned totals. Capture:

- Raw prompt/events/stderr/rollout hashes, complete answer bytes, process exit,
  timeout, cleanup and actual Host/model/thread identity. Retain missing outputs.
- Every tool attempt/result and actual delivery limits. Inner success is not
  proof the model received all required text; audit delivered ranges/figures.
- Session lineage, original/candidate/predecessor/source identities and snapshots
  before/after; exact saved version and approval steps. A/B can use normal files.
  C retains native preview, approval and revision/digest receipts without granting
  automatic semantic correctness. Never transplant another arm's predecessor.
- Attempts, restart/repair parent IDs and causes. Keep the first attempt and all
  extra work. Content-quality failure does not authorize a fresh best-of-N run.
- Active human/coordinator time by activity: setup, task specification, evidence
  lookup, approval/save, restoration and repair. Record automated coordinator
  actions separately; model activity is not measured human time. Use a real timer
  or mark unknown. Distinguish first-install cost from per-task recurring cost.
- Wall/model/tool wait, tool calls, denied/failed/retried calls, input/cache/output
  tokens when observed, currency/billing evidence and unknown values. Do not turn
  token counts or text edit distance into billed cost or human effort.

Keep all units in completion/failure summaries. For a content metric without an
answer, record missing/not-assessable, report coverage/completion separately and
do not treat it as zero factual errors. For an interrupted continuation, retain
the first stage and the uncompleted second stage. Do not invent semantic scores.

## Outcome review and reconciliation

Each atomic claim review binds `answer_sha256`, exact character span/quote,
source/version/location, verdict, error type, whether a citation is required,
whether the adjacent citation supports it, reviewer identity/role and uncertainty.
Claims added without a citation still belong in the source-dependent denominator.
Necessary-fact coverage binds predeclared check IDs to actual answer spans; no
mention means missing, not supported. Original errors and editing-added errors
must be distinguished. Style changes alone are not meaning errors.

Record blinded presentation hash, opaque ID, reviewer knowledge of conditions,
condition guess/confidence/reason, accidental exposure and the adjudication of
disagreements. Separate first judgment from corrected judgment. Ordinary readers
judge expression, domain reviewers judge source support and methods; model review
can locate candidate issues but cannot stand in for independent expertise.

## Reporting and investment decision

Report per task and repeat, then B−A, C−B and C−A in each **common** dimension.
Match task/repeat identities; do not pool units across tasks as independent studies.
Separate completion, structural checks, semantic judgments and process fidelity.
Keep failed units, first attempts and recovery costs visible; report the number of
assessable answers beside any error proportion. Report model/reader/expert judgments
under their actual roles. Do not compute one weighted product-quality score.

Before main results, choose the smallest worthwhile reduction in active human
minutes or necessary repairs, acceptable extra execution cost and coverage floor
using calibration cost data and maintainer priorities. Record the decision and
its time. No defaults have been invented for those preferences here.

Hard failures are fabricated sources/numbers, unsupported causal/range strengthening,
claims of reading unavailable evidence, silent stale-source acceptance or an
unapproved project write. They cannot be offset by expression preferences. An
introduced hard failure pauses the affected candidate's default enablement or
quality-improvement claim; retain failures, diagnose and separately verify repairs.
Existing baseline errors remain reported and are never normalized away.

Possible decisions: C−B recovery benefit with unclear B−A writing benefit favors
continuity; B−A benefit with little C−B benefit favors lighter integration; benefits
confined to one task/Host narrow support claims; higher cost without clear benefit
favors fixing specific faults and reducing default workflow. Missing independent
review, human minutes or billing limits the corresponding conclusion, without
preventing further bounded engineering work.
