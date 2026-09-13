# EVAL-406 Existing Validator Research

## Current execution path

`tests/test_eval_cases.py` imports `evals/runner/run_eval.py` directly. The
runner loads one YAML case, validates each expected artifact, chooses a readable
file, runs typed assertions, updates five truth counters, and returns one
boolean. No other command owns the four shared cases.

## Reusable owners

| Need | Existing owner | Decision |
| --- | --- | --- |
| JSON Schema checks | `tooling/scripts/validate_capability_contract.py::validate_instance` | Import and reuse the tested subset; do not copy it or add `jsonschema`. |
| Ledger/BibTeX identity | `tooling/scripts/audit_citation_risk.py::audit_citation_integrity` | Reuse it after a non-vacuous citable-row check. |
| CSV parsing | Python `csv.DictReader` | Add one small row/column helper in the runner. |
| File digest | Python `hashlib.sha256` | Hash exact bytes; no manifest abstraction. |
| PRISMA counts | `content/templates/prisma-flowchart.md` and paper-screener contract | Parse only configured `Label: n = N` lines and compare one equation. |
| Locator forms | Evidence-ledger and paper-reading contracts | Accept `p. N`, `pp. N-N`, and `citekey:anchor`; semantic verification is later work. |

## Repository constraints

- `pyproject.toml` declares only PyYAML for this runner's parsing needs;
  `jsonschema` is absent.
- `validate_project_artifacts.py::check_prisma_flow` only proves that some
  number exists and cannot establish conservation.
- The current academic-quality evaluator consumes declared scores; roadmap
  `EVAL-409` owns converting those cases to executable findings.
- Current full-cycle ledger fixtures use values that conflict with the canonical
  evidence-ledger enums, so they are not suitable as positive validator fixtures.
- Existing shared cases are mostly Markdown/code keyword checks. PRISMA count
  conservation is the only EVAL-406 validator that maps directly to a current
  shared case without inventing a new artifact.

## Minimal contract choice

Use explicit format-bound assertions instead of a general data/query language:

- JSON/YAML artifact + case-relative JSON schema;
- CSV allowed-value, cross-file column, locator, and citation checks;
- configured Markdown count labels;
- byte-level SHA-256.

Focused tests can materialize all required files in temporary directories.
Broad checked-in positive/negative fixture corpora remain EVAL-408 work.
