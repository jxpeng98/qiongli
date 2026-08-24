---
task_id: I7
template_type: performance_profile
topic: asset-pricing-capm-ff3
primary_artifact: code/performance_profile.md
---

# Performance Profile

## Execution Contract Block

```json
{
  "task_id": "I7",
  "topic": "asset-pricing-capm-ff3",
  "plan_source": "code/plan.md",
  "performance_artifact": "code/performance_profile.md",
  "analysis_outputs": ["analysis/provenance.json", "analysis/results/"],
  "documentation_outputs": ["code/documentation/README.md"],
  "container_outputs": ["code/container_config/README.md"],
  "validation_runs": [
    {"step_id": "S1", "evidence": "script lock resolved"},
    {"step_id": "S2", "evidence": "100 model rows generated"},
    {"step_id": "S3", "evidence": "byte-identical check passed"},
    {"step_id": "S4", "evidence": "profiled check passed"}
  ],
  "profiling_targets": [
    {"component": "complete cached validation", "command": "/usr/bin/time -l uv run --script RESEARCH/asset-pricing-capm-ff3/analysis/run_analysis.py --check --profile"}
  ]
}
```

## Scope Executed

- Digest validation, ZIP member validation, parsing, common-sample construction, 100 model fits, diagnostics, deterministic serialization, and byte comparison.

## Implementation Ledger

| Step ID | Planned Output | Observed Output | Status | Notes |
|---|---|---|---|---|
| S1 | Exact dependency graph | `analysis/run_analysis.py.lock` | PASS | Compatible pandas is explicitly pinned. |
| S2 | Provenance and results | 100 models over 756 months | PASS | All trust-boundary assertions passed. |
| S3 | Byte-identical recomputation | No differing output | PASS | Check mode wrote nothing. |
| S4 | Cached runtime below 30 seconds | 0.233-second profiled execute region | PASS | Whole timed command was 1.77 seconds. |

## Validation Evidence

| Check | Evidence | Result | Artifact |
|---|---|---|---|
| Input identity | Exact SHA-256 and expected ZIP member | PASS | `analysis/provenance.json` |
| Model inventory | 100 model rows and 100 diagnostic rows | PASS | `analysis/results/analysis_summary.json` |
| Determinism | Recomputed bytes match stored outputs | PASS | `analysis/results/output_digests.json` |
| Performance | cProfile plus `/usr/bin/time -l` | PASS | This profile |

## Artifact Inventory

- `analysis/provenance.json`
- `analysis/results/analysis_summary.json`
- `analysis/results/model_results.csv`
- `analysis/results/model_comparison.csv`
- `analysis/results/model_summary.csv`
- `analysis/results/factor_diagnostics.json`
- `analysis/results/residual_diagnostics.csv`
- `analysis/results/output_digests.json`
- `code/documentation/README.md`
- `code/container_config/README.md`

## Environment / Containerization

- OS: Darwin 27.0.0 arm64.
- Python version: 3.12 selected by `uv`.
- Key deps: NumPy 2.3.2, pandas 2.3.2, statsmodels 0.14.5.
- Container: none; the PEP 723 script lock is the minimal environment contract.

## Profiling Results

- Dataset size: 756 months, 25 portfolios, two weighting schemes, and two models; 100 fitted regressions.
- Command: `/usr/bin/time -l uv run --script RESEARCH/asset-pricing-capm-ff3/analysis/run_analysis.py --check --profile`.

| Component | Time | Notes |
|---|---:|---|
| Complete Python execute region | 0.233 s | 374,305 calls; cached inputs and environment. |
| Model fitting | 0.133 s | Largest cumulative component. |
| Portfolio parsing | 0.079 s | Both monthly sections. |
| Whole timed command | 1.77 s | Includes `uv` startup. |

Maximum resident set size for the whole timed command was 169,476,096 bytes (about 161.6 MiB).

## Optimization Actions Taken

1. None. The complete cached validation is already well below the 30-second budget.

## Reproduction Commands

1. `uv sync --script RESEARCH/asset-pricing-capm-ff3/analysis/run_analysis.py --locked`
2. `uv run --script RESEARCH/asset-pricing-capm-ff3/analysis/run_analysis.py`
3. `uv run --script RESEARCH/asset-pricing-capm-ff3/analysis/run_analysis.py --check --profile`

## Remaining Gaps / Blockers

- First-run download and environment resolution depend on upstream availability.
- Memory is measured for the whole `uv` command rather than isolated Python allocations.
- No performance blocker remains.
