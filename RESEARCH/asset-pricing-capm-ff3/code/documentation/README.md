# Reproducing The Asset-Pricing Analysis

Requirements: `uv`, network access for the first dependency/data fetch, and Python 3.11+ as selected by `uv`.

From the repository root:

```bash
uv sync --script RESEARCH/asset-pricing-capm-ff3/analysis/run_analysis.py --locked
uv run --script RESEARCH/asset-pricing-capm-ff3/analysis/run_analysis.py
uv run --script RESEARCH/asset-pricing-capm-ff3/analysis/run_analysis.py --check
```

The normal run downloads missing official archives into the ignored `analysis/data/raw/` directory, verifies fixed SHA-256 digests, and atomically replaces the bounded result files. `--check` writes nothing and fails if recomputed bytes differ. The script pins pandas because statsmodels imports it at runtime and the selected statsmodels release is incompatible with the newer pandas API.

Use `--check --profile` for a bounded cProfile summary. The script uses no randomness. A source digest mismatch is intentional failure: inspect the new publisher vintage and update design, pin, results, and evidence together only after explicit review.
