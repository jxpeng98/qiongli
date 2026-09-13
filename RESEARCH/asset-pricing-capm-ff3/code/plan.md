---
task_id: I6
template_type: code_plan
topic: asset-pricing-capm-ff3
primary_artifact: code/plan.md
---

# Execution Plan

## Plan Contract Block

```json
{
  "task_id": "I6",
  "topic": "asset-pricing-capm-ff3",
  "spec_source": "code/code_specification.md",
  "plan_artifact": "code/plan.md",
  "steps": [
    {"step_id": "S1", "depends_on": [], "owner": "single-agent", "command": "uv lock --script RESEARCH/asset-pricing-capm-ff3/analysis/run_analysis.py", "outputs": ["analysis/run_analysis.py.lock"], "checkpoint": "lock resolves exact declared dependencies", "rollback": "fix only the declared dependency versions"},
    {"step_id": "S2", "depends_on": ["S1"], "owner": "single-agent", "command": "uv run --script RESEARCH/asset-pricing-capm-ff3/analysis/run_analysis.py", "outputs": ["analysis/provenance.json", "analysis/results/"], "checkpoint": "all input and model-grid assertions pass", "rollback": "retain prior outputs and fix the first failing invariant"},
    {"step_id": "S3", "depends_on": ["S2"], "owner": "single-agent", "command": "uv run --script RESEARCH/asset-pricing-capm-ff3/analysis/run_analysis.py --check", "outputs": [], "checkpoint": "recomputed outputs match byte-for-byte", "rollback": "identify the first changed output and remove nondeterminism"},
    {"step_id": "S4", "depends_on": ["S3"], "owner": "single-agent", "command": "uv run --script RESEARCH/asset-pricing-capm-ff3/analysis/run_analysis.py --check --profile", "outputs": [], "checkpoint": "profiled validation completes under the performance budget", "rollback": "record a justified bottleneck; optimize only if budget fails"}
  ],
  "dataset_lineage_checks": ["official URL and SHA-256", "one safe expected ZIP member", "monthly section/header", "unique dates", "sentinel rejection", "common sample"],
  "diagnostics": ["factor correlations and condition numbers", "Durbin-Watson", "Ljung-Box(6)", "Breusch-Pagan"],
  "robustness_checks": ["equal-weighted complete model grid", "portfolio-level alpha comparisons"],
  "manuscript_outputs": [],
  "rerun_evidence": ["analysis/results/output_digests.json", "--check byte comparison", "code/performance_profile.md", "code/reproducibility_audit.md"]
}
```

## Scope Lock

One Python script and one lock implement the full analysis; all other files are research artifacts or generated bounded outputs.

## Assumptions From Spec

- Official archives retain the reviewed bytes and member names.
- Python 3.11+ can install the two project-local dependencies.
- No random operation or OS-specific path enters results.

## Step Ledger

1. [x] `S1` — resolved the exact script lock after pinning compatible pandas.
2. [x] `S2` — validated both inputs and generated the complete 100-model grid.
3. [x] `S3` — recomputed in check mode with byte-identical outputs.
4. [x] `S4` — profiled cached check mode at 0.233 seconds, below the 30-second budget.

## Checkpoint Matrix

| Step | Inputs | Output | Pass Condition | Failure Trigger |
|---|---|---|---|---|
| S1 | PEP 723 metadata | script lock | exact packages resolved | resolver error or version conflict |
| S2 | two pinned ZIPs | provenance and results | fixed row counts and all assertions | any trust-boundary/model invariant fails |
| S3 | cached inputs and results | none | byte equality | missing or differing output |
| S4 | cached environment | stdout profile | total cached run under 30 seconds | budget exceeded |

## Academic Analysis Code Evidence Plan

- Dataset lineage: `design/dataset_plan.md` plus runtime provenance.
- Model diagnostics: two deterministic diagnostic outputs.
- Robustness: equal-weighted rerun and paired portfolio table.
- Manuscript-facing outputs: none; the project stops at analysis.
- Rerun evidence: script lock, command documentation, digest manifest, check-mode pass.
- Anti-pattern guard: no service layer, controller, framework scaffold, or unnecessary class.

## Exact Run Commands

```bash
uv lock --script RESEARCH/asset-pricing-capm-ff3/analysis/run_analysis.py
uv run --script RESEARCH/asset-pricing-capm-ff3/analysis/run_analysis.py
uv run --script RESEARCH/asset-pricing-capm-ff3/analysis/run_analysis.py --check
uv run --script RESEARCH/asset-pricing-capm-ff3/analysis/run_analysis.py --check --profile
```

## Parallelization / Dependency Map

`S1 -> S2 -> S3 -> S4`. The pipeline is intentionally sequential because every step consumes the prior exact artifacts; parallel scaffolding would add no value.

## Rollback / Recovery

The script does not modify raw inputs. Generated outputs use atomic replacement. On failure, fix the first invariant and rerun; digest drift requires explicit review rather than recovery logic.

## Risks / Blockers

- Network/dependency availability affects first-run setup only.
- Publisher revision blocks execution until the expected digest is reviewed.
