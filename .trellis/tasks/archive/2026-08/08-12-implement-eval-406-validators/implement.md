# Implementation Plan

## 1. Lock The Seven Contracts In The Existing Test Owner

- [x] Extend `tests/test_eval_cases.py` with one temporary valid case that
      executes all seven EVAL-406 validators alongside the existing contains
      validators.
- [x] Add a table-driven mismatch check: schema violation, disallowed field,
      broken equation, cross-file mismatch, invalid locator, missing citekey,
      and wrong digest must each return false.
- [x] Add a compact malformed-input table covering type-specific key errors,
      empty applicable data, parse failure, missing reference, and path escape;
      each must block without an uncaught exception.
- [x] Generalize the shared-case contract/materializer only enough to render the
      new PRISMA count assertion.

Focused red check:

```bash
python -m unittest tests.test_eval_cases -v
```

## 2. Extend The Shared Runner Once

- [x] Replace the shared `values` assumption with strict per-type payload
      validation while preserving `contains_all` and `contains_any`.
- [x] Add one contained relative-path resolver and small CSV/count helpers.
- [x] Reuse `validate_instance` for JSON/YAML schema assertions and
      `audit_citation_integrity` for ledger/BibTeX identity.
- [x] Implement field, count, cross-artifact, locator, and digest comparisons
      with the Python standard library.
- [x] Count only completed comparisons as executed; convert configuration,
      reference, decoding, and parse problems into blocked reasons.
- [x] Preserve the existing five-clause return predicate and CLI behavior.

## 3. Put One Scientific Validator On A Real Shared Case

- [x] Add `Records screened = Records excluded + Reports sought for retrieval`
      as a `count_conservation` assertion on
      `evals/cases/sr-social-media-mental-health.yaml`.
- [x] Keep the other shared cases unchanged because their current artifacts do
      not expose stable structured fields, citations, or digests.
- [x] Run the focused eval test again and inspect the human-readable blocked/
      failed reasons for the negative subtests.

## 4. Verify At The Necessary Levels

- [x] Run the focused owner while editing:

```bash
python -m unittest tests.test_eval_cases -v
```

- [x] After the diff is frozen, run the repository's CI-owned unit-test command
      once, then formatting hygiene:

```bash
python -m unittest discover -s tests -v
git diff --check
```

- [x] Do not add an umbrella command, dependency install, package build, Host
      acceptance, or release run; this slice changes none of those boundaries.

## 5. Record Only Proven Completion

- [x] Update `.trellis/spec/product/control/eval-truth-v1.md` with the exact
      implemented payloads, format limits, and blocked/failure semantics.
- [x] Mark only `EVAL-406` complete in the master roadmap after focused and full
      checks pass.
- [x] Run the Trellis quality/check and finish workflows; request the required
      one-shot commit confirmation before committing.

## Review Focus

- No validator can pass on an empty/missing applicable dataset or parse error.
- New paths cannot escape their case/output roots.
- Schema and citation logic reuse their existing owners instead of drifting.
- Count extraction matches the canonical PRISMA line form without becoming a
  Markdown parser or expression language.
- EVAL-407 and later work does not enter the diff.

## Rollback Point

Revert the implementation commit. There is no data, package, Host, or release
rollback.
