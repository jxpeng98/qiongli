# Implementation Plan

## 1. Lock The Failure Contract In The Existing Test Owner

- [x] Extend `tests/test_eval_cases.py`; do not create a new harness.
- [x] Keep the existing all-cases success test and migrate its fixture builder
      to `contains_all` / `contains_any` values.
- [x] Add one table-driven truth-predicate regression covering empty/missing
      required output, zero assertions, malformed assertion data, unknown type,
      unsupported version, and malformed YAML.
- [x] Add one positive case proving an absent optional artifact is allowed only
      beside an executed passing assertion.

Focused red check:

```bash
python -m unittest tests.test_eval_cases -v
```

## 2. Migrate The Four Owned Cases Atomically

- [x] Add `schema_version: "1.0"` to each case.
- [x] Mark every expected artifact with explicit boolean `required`.
- [x] Convert simple lists to `contains_all` and textual `or` alternatives to
      `contains_any` values.
- [x] Remove every `must_contain` and scalar `validation` field.

## 3. Fix The Shared Runner Once

- [x] Validate YAML root, case version, expected-output shape, artifact path,
      requiredness, assertion list, assertion type, and values at the shared
      `run_case` boundary.
- [x] Reuse one minimal assertion dispatcher for `contains_all` and
      `contains_any`; add no plugin registry or dependency.
- [x] Convert required missing/empty/read failure and validator unavailability
      into non-success counters/reasons.
- [x] Apply the five-clause mandatory success predicate and retain the boolean
      API plus CLI exit behavior.

## 4. Verify Once At Each Necessary Level

- [x] Run the focused eval test while editing:

```bash
python -m unittest tests.test_eval_cases -v
```

- [x] After the diff is frozen, run the repository CI-owned unit-test command
      once, then formatting hygiene:

```bash
python -m unittest discover -s tests -v
git diff --check
```

Final evidence on 2026-08-12: the repository suite passed all `1737` tests with
`18` environment skips. `git diff --check`, the focused eval owner, and the
Capability Contract v2 validator also passed. At the user's direction, stale
distribution/native-release failures that blocked this gate were repaired as
separately scoped prerequisite maintenance.

- [x] Do not add an umbrella test or rerun package/Host/release acceptance;
      this slice changes no product package input.

## 5. Record Only Proven Completion

- [x] Add the V1 assertion/predicate contract to the narrowest Trellis product
      spec after implementation proves it.
- [x] Mark only `EVAL-401` through `EVAL-405` complete in the master roadmap,
      with the focused test as evidence.
- [x] Run Trellis check/update-spec and leave exact-head CI/publication for a
      separately authorized GitHub step.
- [ ] Commit the reviewed batches, then run Trellis finish-work to archive the
      task and record the session.

## Review Focus

- Empty output and all-skipped cases cannot pass.
- Unknown or malformed assertions cannot disappear from the result.
- Optional means only "absence allowed", not "present failures ignored".
- No legacy assertion path, new dependency, or EVAL-406+ behavior entered the
  diff.

## Rollback Point

Revert the single implementation commit if downstream internal fixtures have
not migrated. There is no data or release rollback.
