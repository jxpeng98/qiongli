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
- assertions with type `contains_all` or `contains_any` and non-empty string
  `values`.

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
| Present artifact fails a content assertion | `failed_assertions += 1`; false |
| Malformed case, assertion, or unreadable artifact | blocked; false |
| Unknown assertion type | unknown and blocked; false |
| Unsupported schema version or malformed YAML | false |
| All artifacts skipped | `executed_assertions == 0`; false |

## 5. Good / Base / Bad Cases

- Good: required output exists and every typed assertion passes.
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
absence. Run the repository unit-test command once after freezing the diff;
the roadmap slice closes only after that command passes.

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
