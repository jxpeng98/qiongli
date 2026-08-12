# Technical Design

## Boundary

Keep the existing path and boolean API:

```text
case YAML -> load/validate v1 -> resolve artifact -> run typed assertions
          -> apply truth predicate -> bool / CLI exit code
```

`evals/runner/run_eval.py` remains the single owner. No schema library,
registry class, result framework, or second runner is introduced.

## Case Contract V1

The smallest complete shape is:

```yaml
schema_version: "1.0"
expected_outputs:
  skill-id:
    artifact: path/to/file.md
    required: true
    assertions:
      - type: contains_all
        values: [first phrase, second phrase]
      - type: contains_any
        values: [alternative one, alternative two]
```

- `schema_version` must equal the string `"1.0"`.
- `artifact` must be a non-empty relative path and `required` must be a boolean.
- `assertions` must be a non-empty list for every declared artifact.
- `contains_all` succeeds only when every value occurs case-insensitively.
- `contains_any` succeeds when at least one value occurs case-insensitively.
- Values must be non-empty strings. The old textual `"a or b"` parser is
  removed; alternatives are represented structurally.

## Evaluation Semantics

Maintain five local counters matching the roadmap predicate:

- `required_missing`: required artifact absent or required directory has no
  supported readable file;
- `executed_assertions`: known, well-formed assertions actually evaluated;
- `failed_assertions`: executed assertions whose content condition is false;
- `blocked_assertions`: assertions that cannot run because the version,
  structure, artifact, or read operation is invalid;
- `unknown_validation_types`: assertion types with no V1 implementation.

Malformed YAML/case roots return false immediately with a failure reason.
Unknown types increment both the unknown and blocked counters. Missing optional
artifacts print SKIP and contribute no failure, but the case-level
`executed_assertions > 0` guard prevents an all-skipped pass. Present optional
artifacts follow the same assertion rules as required artifacts.

The final boolean is the exact five-clause predicate from the roadmap. The
existing human-readable output gains these counter names; structured receipts
remain EVAL-407 work.

## Migration And Compatibility

All four cases and their fixture materializer move to V1 in the same change.
The runner rejects missing/unsupported schema versions and malformed/legacy
assertion shapes instead of providing a false-green compatibility shim.
Callers retain the same function and CLI contract, so CI wiring does not change.

## Error And Security Boundary

Use `pathlib`/existing file handling and PyYAML. YAML parse errors, invalid
UTF-8, and ordinary file I/O failures become deterministic non-success. Path
containment hardening is not added in this slice because cases already own
repository-authored relative fixture paths; trust-boundary changes belong to a
separate task.

## Rollback

The change is atomic across runner, four cases, tests, and roadmap. Reverting
that commit restores the old internal eval schema. No project data, package,
Host registration, or release state is mutated.
