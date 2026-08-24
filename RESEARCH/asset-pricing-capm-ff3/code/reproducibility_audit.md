---
task_id: I4
template_type: reproducibility_audit
topic: asset-pricing-capm-ff3
primary_artifact: code/reproducibility_audit.md
---

# Reproducibility Audit

## Audit Contract Block

```json
{
  "task_id": "I4",
  "topic": "asset-pricing-capm-ff3",
  "audit_artifact": "code/reproducibility_audit.md",
  "reviewed_artifacts": ["code/plan.md", "code/performance_profile.md", "code/documentation/README.md"],
  "environment_files": ["analysis/run_analysis.py", "analysis/run_analysis.py.lock"],
  "seed_policy_status": "PASS",
  "rerun_entrypoints": [
    {"command": "uv run --script RESEARCH/asset-pricing-capm-ff3/analysis/run_analysis.py"},
    {"command": "uv run --script RESEARCH/asset-pricing-capm-ff3/analysis/run_analysis.py --check"}
  ],
  "verdict": "PASS",
  "blocking_gaps": []
}
```

## Audit Scope

- Inputs, dependency lock, transformations, estimation, diagnostics, output inventory, deterministic check, and recovery behavior.

## Environment Evidence

- PEP 723 pins NumPy 2.3.2, pandas 2.3.2, and statsmodels 0.14.5.
- `analysis/run_analysis.py.lock` resolves the complete environment without modifying repository-wide dependencies.
- The verified host was Darwin 27.0.0 arm64 with Python 3.12 selected by `uv`.

## Data Provenance / Immutability

- Factors archive SHA-256: `cd6d8e0d175b6f423862a6ad15a3073a6e4264b52b2ac9262396c79f707c6bcb`.
- Portfolio archive SHA-256: `43cfc360fca14e7d50766e8432fb8b6151c47078512efe74bd0f5d3804789a2a`.
- The script rejects digest drift, unexpected or unsafe ZIP members, malformed months, duplicates, sentinels, nonfinite values, and incomplete portfolios.
- Raw publisher archives are ignored and never rewritten by the analysis.

## Determinism / Seed Control

- No random operation is used, so a seed is not applicable.
- Columns, rows, float formatting, JSON key order, line endings, and output paths are deterministic.
- `--check` recomputed the complete output set and confirmed byte equality.

## Rerun Recipe

1. Run `uv sync --script RESEARCH/asset-pricing-capm-ff3/analysis/run_analysis.py --locked`.
2. Run `uv run --script RESEARCH/asset-pricing-capm-ff3/analysis/run_analysis.py`.
3. Run `uv run --script RESEARCH/asset-pricing-capm-ff3/analysis/run_analysis.py --check`.
4. Compare the reported sample and inventory with `analysis/results/analysis_summary.json` and `output_digests.json`.

## Failure Points / Recovery

- Missing inputs are downloaded only from the two fixed HTTPS URLs.
- Digest or archive-layout drift fails closed and requires explicit vintage review.
- Output replacement is atomic; a failed run does not partially replace a result file.
- Dependency resolution failure is recovered by restoring the exact lock, not by loosening versions.

## Audit Verdict

PASS. A reviewer can locate exact inputs, environment, code, commands, decisions, outputs, and known limits; the tested rerun is byte-identical on the verified host.

## Required Remediations

- None for the approved scope. Test cross-platform byte equality before claiming platform-independent identity.

## Confidence

- 0.94
