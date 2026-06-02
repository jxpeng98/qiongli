# Academic Skills Next Enhancement Roadmap

> **For future agents:** This roadmap is the source-of-truth queue after the
> generated-output cleanup work in PR #16 and PR #17. Start each feature from
> latest `dev`, keep canonical source clean, and open one small PR per milestone.

## Goal

Make the `qiongli` plugin and `qiongli-workflow` academic skill package more
evidence-grounded, stateful, venue-aware, methodologically rigorous, and
measurable without reintroducing duplicated generated package outputs.

## Current Architecture

- Canonical source lives in root-level source directories:
  - `skills/`
  - `templates/`
  - `standards/`
  - `roles/`
  - `venue-profiles/`
  - `subjects/`
  - `qiongli-workflow/`
- Generated outputs are intentionally untracked and ignored:
  - `qiongli/payload/`
  - `packages/npm-qiongli/payload/`
  - `packages/npm-qiongli/python-runtime/`
  - `plugins/qiongli/skills/qiongli-workflow/`
  - mirrored package directories under `qiongli-workflow/`
- Use `scripts/materialize_distribution_payloads.py --out ...` for packaging
  validation. Use `--in-place` only for explicit release or maintenance work.
- If in-place materialization leaves local outputs, clean them with:

```bash
python scripts/clean_generated_outputs.py --dry-run
python scripts/clean_generated_outputs.py --apply
```

## Operating Principles

- Keep `skills/` as the source of truth for canonical internal skill specs.
- Do not edit generated distribution copies directly.
- Do not add new skills unless the capability cannot be cleanly added to an
  existing skill, workflow, template, or contract.
- Every new scholarly claim feature must encode evidence handling,
  insufficient-input behavior, and non-fabrication rules.
- Every implementation phase must add or update tests before release.
- Prefer machine-readable contracts plus focused audits over prompt-only rules.
- For feature PRs, materialize to `/tmp` or `/private/tmp`, not into the source
  checkout.

## Current Capability Baseline

This baseline reflects the latest `dev` after PR #17.

| Area | Status | Evidence |
|---|---|---|
| Source/output separation | Complete | `docs/development/distribution-materialization.md`, `scripts/clean_generated_outputs.py` |
| Skill section completeness | Complete | `docs/maintainer/skill-quality-gap-report.md` reports 71/71 complete skills |
| Evidence ledger | Complete enough for current release | `scripts/audit_evidence_contract.py`, `tests/test_evidence_ledger_contract.py`, `templates/evidence-ledger.md` |
| Citation risk | Complete enough for current release | `scripts/audit_citation_risk.py`, `tests/test_citation_risk_audit.py`, `qiongli-workflow/references/citation-risk-policy.md` |
| Stage handoff | Complete enough for current release | `scripts/audit_stage_handoffs.py`, `tests/test_stage_handoff_contract.py`, `templates/stage-handoff.md` |
| Venue profiles | Partial | root `venue-profiles/`, `scripts/audit_venue_profiles.py`, and `tests/test_venue_profiles.py` exist; `venue-profile-contract.md` is still missing |
| Research state | Partial | `templates/research-state.md` and `templates/decision-log.md` exist; contract/audit/tests are missing |
| Reviewer model | Not started | no objection-map template, revision-plan template, contract, or tests |
| Reproducibility pack | Not started | no reproducibility-pack templates, audit, or tests |
| Systematic review / qualitative depth | Not started | no screening conflict log, coding book, negative-case log, contract, or tests |
| Paragraph-level writing quality | Not started | no paragraph diagnostic template, contract, audit, or tests |
| Contribution/theory calibration | Not started | no contribution calibration template, theory fit matrix, taxonomy, or tests |

## Next PR Queue

### PR 18: Roadmap Calibration

**Status:** this document.

**Purpose:** Replace the stale pre-cleanup TODO with a current roadmap that
matches the clean-source repository structure.

**Acceptance:**

- The roadmap no longer instructs agents to commit generated outputs.
- Completed and partial phases are distinguished clearly.
- The next feature PRs are ordered by dependency.

### PR 19: Research State Contract

**Purpose:** Make long-running academic work resumable across sessions,
clients, and workflow stages.

**Primary files to create:**

- `qiongli-workflow/references/research-state-contract.md`
- `scripts/audit_research_state.py`
- `tests/test_research_state_contract.py`

**Primary files to modify:**

- `templates/research-state.md`
- `templates/decision-log.md`
- `qiongli-workflow/SKILL.md`
- `skills/Z_cross_cutting/academic-context-maintainer.md`
- `skills/Z_cross_cutting/model-collaborator.md`
- `bridges/orchestrator.py`
- `tests/test_academic_context_continuity.py`
- `tests/test_orchestrator_workflows.py`

**Tasks:**

- Define required `research-state.md` sections:
  - current paper type
  - active venue target
  - research question
  - claim set
  - evidence ledger path
  - open decisions
  - unresolved gaps
  - latest artifact map
  - next recommended task
- Define required `decision-log.md` fields:
  - decision ID
  - date
  - decision
  - alternatives considered
  - evidence used
  - owner role
  - downstream impact
- Add an audit that validates complete, stale, and missing state fixtures.
- Update state-consuming skills to make missing state an explicit gap note
  instead of guessed context.

**Acceptance:**

- New sessions can reconstruct paper context from state artifacts.
- Task-run prompts include current state and unresolved gaps when present.
- Missing state produces a structured gap note.
- `python scripts/audit_research_state.py --strict` passes.

### PR 20: Venue Profile Contract Alignment

**Purpose:** Complete venue-aware workflow support by documenting the contract
that current profile data and tests already enforce.

**Primary files to create:**

- `qiongli-workflow/references/venue-profile-contract.md`

**Primary files to modify:**

- `skills/A_framing/venue-analyzer.md`
- `skills/A_framing/contribution-crafter.md`
- `skills/F_writing/manuscript-architect.md`
- `skills/H_submission/submission-packager.md`
- `skills/H_submission/peer-review-simulation.md`
- `skills/H_submission/rebuttal-assistant.md`
- `qiongli-workflow/SKILL.md`
- `tests/test_venue_profiles.py`

**Tasks:**

- Define the venue profile schema in a reference contract.
- Ensure profiles remain root-level source files under `venue-profiles/`.
- Update venue-facing skills to consume profile fields explicitly.
- Add unsupported-venue fallback guidance.

**Acceptance:**

- A workflow can declare `venue_profile: chi` or another profile and receive
  profile-specific guidance.
- Unsupported venues produce a venue-gap note instead of generic assumptions.
- Venue profile behavior is data-driven, not hard-coded in one skill.

### PR 21: Reviewer Model And Rebuttal Intelligence

**Purpose:** Make review simulation and rebuttal drafting specific,
evidence-backed, and tied to actual manuscript changes.

**Primary files to create:**

- `templates/reviewer-objection-map.md`
- `templates/revision-plan.md`
- `qiongli-workflow/references/reviewer-model-contract.md`
- `tests/test_reviewer_model_contract.py`

**Primary files to modify:**

- `skills/H_submission/peer-review-simulation.md`
- `skills/H_submission/reviewer-empathy-checker.md`
- `skills/H_submission/rebuttal-assistant.md`
- `skills/H_submission/fatal-flaw-detector.md`
- `skills/H_submission/limitation-auditor.md`
- `qiongli-workflow/workflows/rebuttal.md`
- `qiongli-workflow/workflows/submission-prep.md`

**Acceptance:**

- Rebuttal outputs separate reviewer response text from actual manuscript
  changes.
- Each simulated objection names evidence used or missing evidence.
- Objection maps connect objection, evidence, response strategy, and required
  manuscript change.

### PR 22: Reproducibility Pack

**Purpose:** Make Stage I code and analysis work executable, inspectable, and
auditable.

**Primary files to create:**

- `templates/reproducible-analysis-pack.md`
- `templates/analysis-script-order.md`
- `templates/computational-environment.md`
- `scripts/audit_reproducibility_pack.py`
- `tests/test_reproducibility_pack.py`

**Primary files to modify:**

- `skills/I_code/code-specification.md`
- `skills/I_code/code-planning.md`
- `skills/I_code/code-builder.md`
- `skills/I_code/code-execution.md`
- `skills/I_code/reproducibility-auditor.md`
- `skills/I_code/release-packager.md`
- `qiongli-workflow/workflows/code-build.md`

**Acceptance:**

- Code-build flow asks for a reproducibility pack before execution guidance.
- The audit flags missing environment, script order, expected outputs, and
  untracked inputs.

### PR 23: Systematic Review And Qualitative Depth

**Purpose:** Improve systematic-review and qualitative workflows beyond generic
synthesis.

**Primary files to create:**

- `templates/screening-conflict-log.md`
- `templates/coding-book.md`
- `templates/negative-case-log.md`
- `qiongli-workflow/references/systematic-review-advanced-contract.md`
- `qiongli-workflow/references/qualitative-research-contract.md`
- `tests/test_review_and_qualitative_contracts.py`

**Primary files to modify:**

- `skills/B_literature/paper-screener.md`
- `skills/E_synthesis/quality-assessor.md`
- `skills/E_synthesis/evidence-synthesizer.md`
- `skills/E_synthesis/qualitative-coding.md`
- `skills/G_compliance/prisma-checker.md`
- `templates/prisma-flowchart.md`
- `templates/grade-summary-of-findings.md`

**Acceptance:**

- Systematic review workflow distinguishes search, screening, extraction,
  quality, and synthesis risks.
- Qualitative workflow produces coding, memoing, and negative-case artifacts.

### PR 24: Paragraph-Level Scholarly Writing Quality

**Purpose:** Move writing quality checks from complete-but-generic outputs to
paragraph-level scholarly argument repair.

**Primary files to create:**

- `templates/paragraph-diagnostic-report.md`
- `qiongli-workflow/references/paragraph-quality-contract.md`
- `scripts/audit_paragraph_quality.py`
- `tests/test_paragraph_quality_contract.py`

**Primary files to modify:**

- `skills/F_writing/analysis-interpreter.md`
- `skills/F_writing/discussion-writer.md`
- `skills/F_writing/manuscript-architect.md`
- `skills/F_writing/meta-optimizer.md`
- `skills/J_proofread/human-voice-rewriter.md`
- `skills/J_proofread/final-proofreader.md`
- `qiongli-workflow/references/academic-output-rubric.md`

**Acceptance:**

- Writing skills can produce paragraph-level revision notes.
- Proofreading distinguishes grammar cleanup from scholarly argument repair.

### PR 25: Contribution And Theory Calibration

**Purpose:** Make contribution statements precise enough for venue reviewers and
domain scholars.

**Primary files to create:**

- `templates/contribution-calibration.md`
- `templates/theory-fit-matrix.md`
- `qiongli-workflow/references/contribution-taxonomy.md`
- `tests/test_contribution_calibration.py`

**Primary files to modify:**

- `skills/A_framing/contribution-crafter.md`
- `skills/A_framing/gap-analyzer.md`
- `skills/A_framing/theory-mapper.md`
- `skills/A_framing/hypothesis-generator.md`
- `skills/F_writing/manuscript-architect.md`

**Acceptance:**

- Contribution outputs state what changes in knowledge, for whom, and under
  what boundary conditions.
- Theory outputs distinguish borrowed framing from genuine theory contribution.

### PR 26: Evaluation Corpus And Regression Scoring

**Purpose:** Make quality improvements measurable and prevent regression toward
vague AI prose.

**Primary files to create or extend:**

- `evals/academic_quality/`
- `scripts/run_academic_quality_evals.py`
- `scripts/score_academic_output.py`
- `tests/test_academic_quality_evals.py`

**Primary files to modify:**

- `docs/maintainer/skill-quality-contract.md`
- `.github/workflows/ci.yml`
- `scripts/release_preflight.sh`

**Acceptance:**

- Eval cases cover evidence traceability, no fabricated sources, claim strength
  calibration, venue fit, method validity awareness, and scholarly voice.
- Release preflight can run semantic eval checks without requiring network.

## Deprecated Local Branch Handling

Older local branches such as `codex/qiongli-subject-packages`,
`codex/qiongli-coverage-composite`, and
`codex/qiongli-skill-set-optimization` were created before generated outputs
were removed from Git. Do not rebase them directly into new PRs. Instead:

1. Start a fresh branch from latest `origin/dev`.
2. Inspect old commits only for intent.
3. Re-implement the relevant source-only subset.
4. Validate with source tests and materialization into a temporary output
   directory.

This avoids reintroducing ignored package payloads and keeps review diffs
small.

## Standard Verification For Feature PRs

Use the narrowest relevant tests first, then add packaging validation when the
change touches installable package shape.

Common checks:

```bash
python scripts/clean_generated_outputs.py --dry-run
python scripts/check_generated_payload_edits.py --base-ref origin/dev
python -m unittest tests.<targeted_test_module> -v
python scripts/materialize_distribution_payloads.py --target all --out /tmp/qiongli-dist --force
```

Use full CI as the final authority before merge.
