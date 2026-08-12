# Technical Design

## Boundary

Keep the existing owner and public surface:

```text
case YAML -> strict V1 assertion config -> contained artifact paths
          -> format-bound validator -> five truth counters -> bool / CLI exit
```

`evals/runner/run_eval.py` remains the only eval runner. Add small functions for
path resolution, CSV reading, labeled-count extraction, and explicit
type-specific checks; do not introduce a validator framework or result model.

## Case Contract V1 Additions

```yaml
assertions:
  - type: schema
    schema: schemas/result.schema.json

  - type: field_constraint
    field: status
    allowed_values: [included, excluded]

  - type: count_conservation
    total: Records screened
    parts: [Records excluded, Reports sought for retrieval]

  - type: cross_artifact_consistency
    field: record_id
    other_artifact: search_results.csv
    other_field: record_id
    relation: subset

  - type: locator_syntax
    field: source_location

  - type: citation_identity
    bibliography: references.bib

  - type: file_digest
    sha256: 0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef
```

Each type has an exact allowed-key set. Common `type` plus the fields shown
above are required; no optional future fields are accepted. `equal` and
`subset` are the only cross-artifact relations.

## Resolution And Parsing

- Resolve the primary `artifact` and all artifact references under
  `output_dir`; resolve `schema` under the case file's directory. Reject
  absolute paths and resolved paths outside the owning root.
- Read bytes only for `file_digest`. Decode UTF-8 only for validators that need
  text, so binary digest inputs are supported without weakening decode errors.
- Parse schema targets as JSON and primary schema artifacts as JSON or YAML by
  suffix. The supported schema vocabulary is exactly the existing
  `validate_instance` subset, including local fragment `$ref`.
- Parse CSV with `DictReader`, reject missing headers/fields and zero applicable
  rows, and strip values before comparison.
- Extract a configured count only from a full line shaped as
  `Label: n = <non-negative integer>` (Markdown bullet/bold decoration allowed).
  Missing or duplicate label matches block the assertion.

## Validator Semantics

| Type | Pass condition |
| --- | --- |
| `schema` | Parsed artifact has no failures from `validate_instance`. |
| `field_constraint` | Every row's configured field is non-empty and belongs to `allowed_values`. |
| `count_conservation` | The one total equals the sum of all configured parts. |
| `cross_artifact_consistency` | The primary CSV column multiset equals/is a subset of the other column multiset. |
| `locator_syntax` | At least one locator is present and every present value is `p. N`, `pp. N-N`, or `citekey:anchor`. |
| `citation_identity` | At least one citable row exists and the reused audit finds no source ID absent from the bibliography. |
| `file_digest` | SHA-256 of exact artifact bytes equals the configured lowercase-normalized digest. |

Blank locator cells are ignored because unsupported/gap rows may legitimately
lack a source location; a file with no present locators is blocked. Citation
identity remains separate from locator shape, source availability, relevance,
and claim support, matching the roadmap's later evidence-verification boundary.

## Truth And Error Semantics

Configuration is validated before artifact execution. Unknown types increment
both unknown and blocked counters as today. For known types:

- missing/extra configuration, missing referenced files, path escape, decoding
  or parsing errors, absent columns/counts, duplicate count labels, and empty
  applicable datasets increment `blocked_assertions`;
- a completed comparison increments `executed_assertions`; a false result also
  increments `failed_assertions`;
- one bad assertion does not prevent independent assertions from reporting, but
  the unchanged final five-clause predicate makes the case non-success.

Keep human-readable reasons and counters only. Per-assertion status objects,
reason codes, JSON, and JUnit belong to EVAL-407.

## Reuse And Compatibility

- Import `validate_instance` from its existing tooling owner and
  `audit_citation_integrity` from the existing citation owner. Add the repository
  root to `sys.path` for the direct-script invocation, following existing
  tooling scripts.
- Preserve `run_case(case_path, output_dir) -> bool`, the CLI, schema version
  `1.0`, contains semantics, directory handling, and final truth predicate.
- Keep tests in `tests/test_eval_cases.py`; use temporary files and table-driven
  mutations instead of a new fixture hierarchy.

## Rollback

The product change is confined to the runner, one existing case, its test
owner, the Evaluation Truth spec, and the roadmap checkbox. Reverting the
implementation commit restores the prior V1 validator set; no project data,
Host state, package, or release state is mutated.
