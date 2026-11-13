# Same-Host three-arm comparison, protocol 1

This freezes a comparison design and **two synthetic calibration tasks**, not
results. No model has run these tasks at preparation time. No six-task main cohort
has yet been selected. The historical three-case `plugin-baseline.json`, its
required tools, scores and accepted evidence retain their original meanings.

## Question and conditions

Compare the same configured Host/model across fresh isolated sessions:

| Arm | Available support | Contrast |
| --- | --- | --- |
| A | Normal task, identical source bytes, ordinary Host reading, writing, searching and persistent notes | Actual direct-use baseline |
| B | A plus frozen Qiongli research guidance | B−A estimates guidance integration |
| C | B plus installed native Qiongli tools and project persistence | C−B estimates tooling/state integration; C−A estimates the complete treatment |

The comparison is of workflows, not a pure code effect: C also needs tool schemas,
routing, project entry and approval interactions. B and C must have identical
available research-guidance bytes. C's Plugin metadata/Host adapter additions and
which guidance each model actually receives must be recorded separately. Guidance
may be read selectively; do not force equal token consumption or give A equivalent
research advice under another name. A retains its ordinary Host's general guidance.

Preserve normal Host abilities in **all** arms, including local file search, notes,
saved outputs and resumption from persistent work. Limit each arm to its own
public/synthetic workspace; apply the same task network policy, model settings,
turn timeout and retry rule. Do not disable A/B persistence or require Qiongli
files, claim IDs, CLI/MCP calls, preview receipts or exact source digests from A/B
as a condition of common outcome success. A clear ordinary locator is sufficient.
For C, inspect native revision/approval/CAS behavior as a separate process measure.

## Calibration and main-cohort separation

The [machine-readable design](protocol.json) selects:

- `cal-reading`: three invented reports, two independent studies; tables,
  missing figure, confidence intervals and differing outcomes must be reconciled.
- `cal-continuity`: an already faithful bilingual draft from invented interviews,
  saved decisions, then a new session with a source correction. Original meaning,
  version history, local style scope and unresolved issues must survive.

Run each calibration once in A/B/C: **6 experimental units and 9 model sessions**,
because continuity has two sessions in each arm. Execution order is shuffled and
retained before running. A continuity unit includes both fresh sessions and the
intervening save; do not count those sessions as independent samples. Calibration
checks access, policy, timing, recording, reviewer questions and budget. Exclude
it from effect estimates even if its outputs look favorable. Revisions after
calibration get a new protocol/input binding; preserve earlier attempts.

The main cohort is a later decision: six new task packages × three arms × three
repeats = 54 units. Some units span multiple sessions. This is not a power guarantee
or 54 independent research scenarios. Before generating main outputs, freeze six
actual packages satisfying [selection rules](selection.md), their provenance and
hashes, order, settings, reviewer assignments, cost ceiling and worthwhile-benefit
thresholds. No invented numeric benefit threshold is supplied on the maintainer's
behalf. Calibration is allowed before those decisions; main-cohort launch is not.
Used observations, this calibration and rule-development material stay development
or regression data, regardless of new filenames. Freeze one second batch later
to confirm the most promising result, rather than adapting the first main cohort.

## Shared task and continuation handling

Copy the identical initial source/task/original bytes to each arm. Substitute only
the assigned workspace entry, permitted installed guidance entry and tool entry.
Retain the exact prompts and a comparison of their arm-specific additions before
execution. Do not insert expected answers, hidden source checks or prior results.
When C ingests sources into native packets, verify that their source bodies match
the A/B files. Record wrappers, extraction/ingestion steps and costs separately;
do not add scientific summaries or evidence that A/B were not given.
Expose only the current checkpoint directory; the later correction, coordinator
instructions, review criteria, other arms and identity key must be inaccessible.

For writing, retain the complete returned original, even when incorrect or empty.
After the first continuity turn, all arms receive the same authored decision text.
A/B may preserve it with their normal files/Host tools. C uses existing native
preview/review/approval/CAS, with explicit approval for exact synthetic writes.
Keep original and candidate distinct. Do not rewrite poor answers before recovery.
If a coordinator must save or organize anything, log exact actions and active time
for **every** arm. Common authorization permits only the assigned synthetic trial;
it never implies blanket approval of all MCP tools or access to private research.

The next session gets the common followup task, corrected source and its own entry:
an ordinary project directory/entry note in A/B, registered project ID in C. Include
no previous prose or preference inline, and do not grant access to prior chat logs.
If a Host normally has automatic persistent memory, use the same frozen setting
in all arms and record what survives; never silently remove a baseline capability.
Native CLI context fallback is allowed in C and must be identified. A recovered
file alone is not success: the returned continuation must use current sources and
decisions and preserve unresolved scope. Missing predecessors remain failed units;
record the unrun dependent session, not a fabricated replacement.

## Execution, records and review

Use [records.md](records.md) for required identity, attempt, intervention, outcome
and review records. Freeze installed package/content/guidance, Host/model settings,
effective permissions, inputs and schedule **before** model calls. A package
version string alone is insufficient. Check actual tool policy and material
access with existing preflight owners; a material preparation receipt below is
not Host readiness or permission to launch a model. Keep the user's configured
model; never silently change it to complete a comparison.

Execution/cleanup stays with `antigravity_observation.run_process`, trace/delivery
inspection with `plugin_baseline.inspect_trace` / `inspect_tool_delivery`, explicit
input binding with `writing_observation`, and source/span review with `observe`'s
review principles. Do not feed A/B through `plugin_baseline.score`, and do not
reuse its structural pass count as common outcome success. This protocol adds no
model runner, scorer, native project schema or automated scientific verdict.

No in-turn coaching. After a documented transport/environment failure, allow at
most one restart from the same frozen initial state and retained predecessor.
Record both attempts and total cost; the original remains in the denominator.
Content defects, denied writes and incomplete output are not reasons to reroll.
An affected unit can receive a separately labeled repair after its original has
been recorded. A repaired answer never overwrites the original observation.

Common metrics and reviewer responsibilities are specified in [review/rubric.md](review/rubric.md).
Report facts, coverage, citation support, synthesis, meaning, expression, continuity,
human effort and execution cost separately. Never trade a fabricated source or
unapproved write against a fluency score. Produce random opaque review IDs with a
**different unexported randomization key** from the run-order seed. Remove product
labels and logs from reviewer views without rewriting scientific prose, citations
or limitations. Keep the identity key outside generation and review workspaces.
Record guessed arm, confidence and suspected cues; anonymous IDs do not prove
blindness. Common outcome review precedes any process/arm attribution.

## Offline preparation

From the repository root:

```sh
python3 -m evals.research_journey.comparison check
python3 -m evals.research_journey.comparison prepare /absolute/new-calibration --seed 20261113
```

The small helper verifies the frozen selection, makes a new external copy and
records a shuffled six-unit schedule with all nine planned sessions. It reuses
`writing_observation.freeze_packet`; the returned packet hash verifies only those
prepared files. It does **not** configure a Host, bind real installed guidance,
hide files through permissions, create projects, invoke models, approve writes,
score outputs or claim execution. Only grant a producer its current `initial/`
or `followup/` material plus its own persistent work; granting the preparation
root would expose the hidden criteria and later checkpoint. `check` is read-only;
`prepare` refuses an existing destination and changed/missing/escaping inputs.

Run calibration next after binding the real runtime and policy. Report setup and
quality observations separately, choose the main-study cost/benefit rules before
seeing main outputs, and keep missing expert review or billing explicitly unknown.
