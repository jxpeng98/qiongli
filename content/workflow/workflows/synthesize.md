---
description: "Synthesize screened, extracted, and appraised evidence using narrative, qualitative, or meta-analytic methods."
---

# Evidence Synthesis / Meta-analysis

Use the **Synthesis decisions** in `references/stage-E-synthesis.md` to turn
screened, extracted and appraised evidence into the requested synthesis.

Canonical Task IDs:
- `E1` synthesis strategy
- `E2` effect size table
- `E3` meta-analysis results
- `E3_5` missing-results bias
- `E4` certainty assessment
- `E5` integrated synthesis

## Target

$ARGUMENTS

## Establish the decision and available evidence

Reuse the authorized project, current research state, handoff, review question,
selected outcomes and protocol. A supplied-evidence question can be answered in
chat. Ask for a project destination only if a required write target is unknown;
do not enumerate other research folders or create a new project automatically.

Inspect the relevant extraction, quality assessment and source notes. Missing
files are missing evidence, not an instruction to create completed upstream work.
Identify which decision the gap affects and what source/check could resolve it.
Continue supported inventory, planning or bounded synthesis while that branch is
unresolved. Use `boundary-interviewer` only for a consequential unresolved or
changed boundary; reuse settled decisions.

## Select and execute the supported synthesis

Use `skills/E_synthesis/evidence-synthesizer.md` and the Stage E decision table.
Choose the method per outcome/theme from the question, compatibility, dependence
and usable evidence. Reuse known time points, direction conventions and subgroup
requirements instead of asking a fixed list of questions.

For selected formal tasks, retain Stage E's outputs and Q2/Q4:
- E1 uses `templates/meta-analysis-plan.md`, including narrative/qualitative plans.
- E2 uses `templates/effect-size-extraction-table.md` with source-bound inputs.
- E3 uses `templates/meta-analysis-report.md` only for an actually executed analysis.
- E3_5/E4 record the applicable missing-results/ certainty assessment and its limits.
- E5 writes `synthesis.md` and `synthesis_matrix.md`; qualitative evidence also uses
  the existing qualitative dictionary/codebook outputs.

Persist through the existing preview/approval/CAS owner. Preserve user material,
claim/decision IDs, citekeys and source locators; a candidate is not a saved result.

## Check the result and hand it forward

Reconcile each conclusion with the studies/sources that actually contribute,
transformations, executed results, appraisal and counterevidence. State what can
be concluded now and what remains conditional. A narrative result does not remove
missing evidence or establish certainty. Do not claim a pooled result from a plan.

For a formal transition, use `references/stage-handoff-contract.md`: carry the
method decision, source/output revision, claim limits, outstanding check and
next supported action into F. If an input changed, revisit affected decisions
before reusing old results or prose. Finish at the requested deliverable with its
specific evidence gaps; do not restart completed upstream stages automatically.
