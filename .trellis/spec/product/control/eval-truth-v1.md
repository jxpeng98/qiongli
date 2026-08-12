# Evaluation Truth V1

## 1. Scope / Trigger

Use this contract when changing `evals/runner/run_eval.py` or the shared cases
under `evals/cases/`. It prevents an eval case from passing when evidence is
missing, malformed, skipped, unreadable, or never checked.

## 2. Signatures

```python
run_case(case_path: Path, output_dir: Path) -> bool
```

```text
python evals/runner/run_eval.py <case.yaml> <output-dir>
```

The Python API remains boolean. The CLI exits `0` only for `True`, otherwise
it exits non-zero.

## 3. Contracts

Each case has `schema_version: "1.0"` and a non-empty `expected_outputs` object.
Every expected output contains:

- a non-empty relative `artifact` path;
- `required` as an explicit boolean;
- a non-empty `assertions` list;
- assertions whose fields exactly match one supported type:

| Type | Required fields | Executable condition |
|---|---|---|
| `contains_all` | non-empty string `values` | every value occurs case-insensitively |
| `contains_any` | non-empty string `values` | at least one value occurs case-insensitively |
| `schema` | case-relative JSON `schema` path | JSON/YAML artifact satisfies the existing supported Schema subset |
| `field_constraint` | CSV `field`, non-empty `allowed_values` | every non-empty row value belongs to the allowlist |
| `count_conservation` | `total` label, non-empty `parts` labels | unique `Label: n = N` counts satisfy total = sum(parts) |
| `cross_artifact_consistency` | CSV `field`, output-relative `other_artifact`, `other_field`, `relation` | value multisets are `equal` or primary is a `subset` |
| `locator_syntax` | CSV `field` | every present locator is `p. N`, `pp. N-N`, or `citekey:anchor` |
| `citation_identity` | output-relative `bibliography` | every paper/theory source ID exists as a BibTeX citekey |
| `file_digest` | 64-hex `sha256` | SHA-256 matches the artifact's exact bytes |

Assertion types reject missing and extra fields. String lists are non-empty and
duplicate-free. `schema` uses the tested subset owned by
`tooling/scripts/validate_capability_contract.py`; it is not a claim of full
JSON Schema Draft 2020-12 support. Citation identity reuses
`tooling/scripts/audit_citation_risk.py` and remains separate from locator,
availability, relevance, and claim-support semantics.

The primary artifact, bibliography, and cross-artifact references remain under
the output root. Schema references remain under the case directory. Absolute
or escaping paths are blocked. A digest may read binary bytes; text, CSV, JSON,
and YAML validators decode only their owned formats.

A case succeeds only when all five conditions hold:

```text
required_missing == 0
executed_assertions > 0
failed_assertions == 0
blocked_assertions == 0
unknown_validation_types == 0
```

`must_contain` and scalar `validation` are not V1 compatibility inputs; reject
them rather than maintaining a second parser.

## 4. Validation & Error Matrix

| Condition | Result |
|---|---|
| Missing optional artifact | SKIP; allowed only if another assertion executes |
| Missing or empty required artifact | `required_missing += 1`; false |
| Completed scientific/content comparison is false | `failed_assertions += 1`; false |
| Malformed case/assertion, path escape, unreadable or unparseable input | blocked; false |
| Missing referenced schema, bibliography, or cross-artifact | blocked; false |
| Empty CSV/applicable set, missing field/count, or duplicate count label | blocked; false |
| Unknown assertion type | unknown and blocked; false |
| Unsupported schema version or malformed YAML | false |
| All artifacts skipped | `executed_assertions == 0`; false |

## 5. Good / Base / Bad Cases

- Good: required output exists and every configured typed assertion executes
  against non-empty applicable evidence and passes.
- Base: an optional output is absent while at least one required assertion
  executes and passes.
- Bad: an empty directory, all-optional absence, unknown assertion, parse
  error, or failed assertion returns false.

## 6. Tests Required

Run:

```bash
python -m unittest tests.test_eval_cases -v
```

The focused owner must assert all four cases pass minimal valid fixtures and
must cover required absence, zero execution, malformed/unknown assertions,
unsupported versions, read errors, malformed YAML, and optional presence and
absence. It also executes every scientific validator against temporary
JSON/YAML, CSV, Markdown, BibTeX, and binary artifacts; each semantic mismatch
must fail, while malformed configuration/data and path escapes must block. Run
the repository unit-test command once after freezing the diff; the roadmap
slice closes only after that command passes.

## 7. Wrong vs Correct

Wrong: free-form text can be ignored and does not declare requiredness.

```yaml
- artifact: result.md
  validation: "contains alpha or beta"
```

Correct: requiredness and alternatives are machine-executable.

```yaml
- artifact: result.md
  required: true
  assertions:
    - type: contains_any
      values: [alpha, beta]
```

Correct scientific example: configured PRISMA labels are parsed once and their
equation is executable.

```yaml
- artifact: screening/prisma_flow.md
  required: true
  assertions:
    - type: count_conservation
      total: Records screened
      parts: [Records excluded, Reports sought for retrieval]
```
