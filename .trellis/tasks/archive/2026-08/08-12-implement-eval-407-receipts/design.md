# Technical Design

## Boundary

Keep one evaluation owner and add two projections:

```text
case + output artifacts
        |
        v
_evaluate_case() -> one structured result + legacy display lines
        |                         |
        |                         +-> run_case() prints and returns bool
        +-> JSON renderer -> optional atomic file
        +-> JUnit renderer -> optional atomic file
```

`evals/runner/run_eval.py` remains the sole owner. Use ordinary dictionaries,
`json`, `xml.etree.ElementTree`, `hashlib`, and `tempfile`; do not add a receipt
package, result class hierarchy, XML library, or schema generator.

## Public Compatibility

Preserve the Python entrypoint exactly:

```python
run_case(case_path: str | Path, output_dir: str | Path | None = None) -> bool
```

It writes no receipt and retains the current human output. Replace the manual
`sys.argv` parsing with `main(argv: list[str] | None = None) -> int` using
`argparse`, while preserving both positional arguments:

```text
python evals/runner/run_eval.py CASE [OUTPUT_DIR]
    [--json-receipt PATH] [--junit-receipt PATH]
```

The flags are independent and opt-in. A relative destination is relative to the
process working directory. The CLI evaluates once, prints the legacy report,
writes requested receipts, and returns non-zero if evaluation fails or any
requested receipt cannot be written. Two destinations resolving to the same
path are rejected before evaluation.

## Internal Result Contract

`_evaluate_case` returns one internal dictionary. Private display lines are not
serialized. Its portable JSON projection is:

```json
{
  "receipt_version": "1.0",
  "case": {
    "id": "scientific-validators",
    "pipeline": "empirical-study",
    "schema_version": "1.0",
    "status": "blocked",
    "reason_code": "assertion-blocked"
  },
  "summary": {
    "required_missing": 0,
    "executed_assertions": 3,
    "failed_assertions": 1,
    "blocked_assertions": 1,
    "unknown_validation_types": 0
  },
  "assertions": [
    {
      "output_id": "ledger",
      "index": 0,
      "type": "field_constraint",
      "status": "pass",
      "reason_code": "assertion-passed",
      "message": "Assertion passed.",
      "evidence": [
        {
          "role": "artifact",
          "path": "ledger.csv",
          "sha256": "0123456789abcdef..."
        }
      ]
    }
  ]
}
```

Configured assertions retain their zero-based index. Synthetic outcomes use
`index: null` and type `case-contract`, `output-contract`, or
`artifact-contract`. A synthetic case-contract outcome is mandatory whenever
case parsing/validation fails or the five-clause predicate would fail solely
because no assertion executed. This prevents an empty or all-skipped JUnit file
from appearing green.

The status vocabulary is `pass`, `fail`, `blocked`, and `skip` for assertion
records. Case status is `pass`, `fail`, or `blocked`; blockers and zero execution
take precedence over ordinary failures because the comparison was incomplete.

## Reason Codes

Reason codes use the repository's lower-case hyphenated convention. Keep the
vocabulary bounded and owned by the runner:

| Outcome | Code |
|---|---|
| case success | `case-passed` |
| case parse/read failure | `case-load-failed` |
| malformed case fields | `case-contract-invalid` |
| unsupported case schema | `schema-version-unsupported` |
| malformed expected outputs | `expected-outputs-invalid` |
| no executed assertion | `no-assertions-executed` |
| assertion success | `assertion-passed` |
| completed false comparison | `<assertion-type>-failed` with `_` converted to `-` |
| malformed assertion payload | `assertion-config-invalid` |
| unknown assertion type | `assertion-type-unknown` |
| assertion input unavailable | `assertion-evidence-unavailable` |
| malformed output payload | `output-contract-invalid` |
| invalid/escaping artifact path | `artifact-path-invalid` |
| required artifact missing/empty | `required-artifact-missing` / `required-artifact-empty` |
| optional artifact absent/empty | `optional-artifact-missing` / `optional-artifact-empty` |
| unreadable/wrong-kind artifact | `artifact-unreadable` / `artifact-kind-invalid` |

Case reason codes are derived from the same counters/records as the mandatory
predicate, never from a renderer. Receipt messages are fixed portable text for
their reason code. Existing detailed human diagnostics remain display-only so
exceptions, source values, or machine paths cannot leak into receipts.

## Evidence Contract

Each assertion outcome has an ordered evidence list. Evidence entries contain:

- `role`: `case`, `artifact`, `schema`, `other-artifact`, or `bibliography`;
- `path`: POSIX path relative to the owning case/output root;
- `sha256`: exact-byte digest, present only when that file was available.

Primary evidence comes first, followed by a type-specific reference. Directory
artifacts identify the actual selected file rather than only the directory.
Missing or unsafe paths never receive a fabricated digest. Receipts exclude raw
file content, topics, exception strings, timestamps, elapsed times, random IDs,
current directories, and resolved absolute paths.

## JUnit Projection

Emit one `<testsuite>` and one `<testcase>` per assertion/synthetic outcome.
Suite properties carry receipt version, case ID, pipeline, case status/reason,
and all five truth counters. Testcase properties carry output ID, assertion
index/type, status, reason code, and flattened evidence fields.

Mapping:

| Receipt status | JUnit child |
|---|---|
| `pass` | none |
| `fail` | `<failure type="reason-code" message="portable message">` |
| `blocked` | `<error type="reason-code" message="portable message">` |
| `skip` | `<skipped message="portable message">` |

`tests`, `failures`, `errors`, and `skipped` are computed from the emitted
testcases. Omit `time`, `timestamp`, hostname, stdout/stderr dumps, and arbitrary
system properties. Render UTF-8 XML with a declaration, stable insertion order,
fixed two-space indentation, and one trailing newline.

## Atomic Writes And Errors

Render requested formats fully in memory. For each destination, create its
parent, write a named temporary file in that parent, flush/close it, then replace
the destination. Remove the temporary file on error. Atomicity is per receipt;
cross-file transactionality is not claimed.

Receipt rendering/writing is outside the evaluation predicate. A write error
does not rewrite evaluation counters or fabricate a receipt outcome, but the CLI
returns non-zero and prints one human diagnostic. `run_case` remains unaffected
because it does not write receipts.

## Rollback

Revert the implementation commit. Existing cases and artifacts require no
migration because receipt emission is opt-in and versioned independently from
case schema `1.0`.
