# Implementation Plan

## 1. Lock The Receipt Contract In The Existing Test Owner

- [x] Add one mixed-outcome temporary case to `tests/test_eval_cases.py` with a
      passing assertion, a completed mismatch, an unavailable-input blocker,
      and an optional missing artifact.
- [x] Invoke the CLI with both receipt flags; parse JSON and JUnit with the
      standard library and assert case identity, assertion ordering, status,
      reason codes, evidence paths/digests, and suite counts.
- [x] Run the same case twice to different targets and require byte-identical
      JSON and XML.
- [x] Add case-contract and all-skipped scenarios proving both receipts remain
      non-green and contain a synthetic blocked outcome.
- [x] Add redaction canaries for artifact content and absolute temporary roots.
- [x] Cover each flag independently, the no-flag default, same-target rejection,
      and an unwritable destination with no partial temporary file.

Focused red check:

```bash
python -m unittest tests.test_eval_cases -v
```

## 2. Capture Outcomes Once In The Existing Runner

- [x] Move the current evaluation body into `_evaluate_case` and capture stable
      assertion/synthetic records plus private legacy display lines.
- [x] Keep every current validation branch, five counter meanings, directory
      selection rule, detailed human diagnostic, and success predicate intact.
- [x] Add small helpers for portable messages, reason-code selection, relative
      evidence identity, and exact-byte digesting; do not add classes.
- [x] Make `run_case` print captured legacy lines and return the unchanged
      predicate result without writing files.

## 3. Add The Two Deterministic Projections

- [x] Build the public JSON object by selecting portable fields from the internal
      result; serialize UTF-8 with sorted keys, `allow_nan=False`, indentation,
      and one trailing newline.
- [x] Build one JUnit testsuite with `xml.etree.ElementTree`, deterministic suite
      and testcase properties, exact status mapping, stable indentation, and one
      trailing newline.
- [x] Guarantee at least one synthetic blocked testcase for case-contract or
      zero-execution failures.
- [x] Keep detailed artifact values and exception text out of both projections.

## 4. Add Explicit CLI Writes

- [x] Replace the manual argv block with `argparse` while preserving `CASE` and
      optional `OUTPUT_DIR` positionals.
- [x] Add independent `--json-receipt` and `--junit-receipt` paths, reject a
      shared resolved destination, and evaluate only once.
- [x] Add one stdlib atomic-byte writer used by both formats; clean its temporary
      file on every failure and make a write error return non-zero.
- [x] Confirm a no-flag CLI invocation and all Python callers remain read-only.

## 5. Verify And Record Only Proven Completion

- [x] Run focused tests while editing, then freeze the product/test diff.
- [x] Run the repository's CI-owned unit-test command once and formatting checks:

```bash
python -m unittest discover -s tests
python -m py_compile evals/runner/run_eval.py tests/test_eval_cases.py
git diff --check
```

- [x] Update `.trellis/spec/product/control/eval-truth-v1.md` with the exact JSON,
      JUnit, evidence, status/reason, determinism, and CLI contracts.
- [x] Expand the M1 authorization wording in the control index only as needed to
      avoid the stale EVAL-401–406 boundary; do not grant M2 or release authority.
- [x] Mark only `EVAL-407` complete in the master roadmap after every acceptance
      check passes.
- [ ] Run Trellis quality/spec/finish workflows and obtain one-shot commit
      confirmation before committing; do not push.

## Review Focus

- Renderers consume one result and cannot disagree with the boolean predicate.
- Blocked and all-skipped evaluations cannot look green in JUnit.
- Receipts contain relative identities and digests, not raw or absolute data.
- Reason codes remain stable lower-case hyphenated tokens.
- The default Python/CLI path remains read-only and backward compatible.
- EVAL-408 and EVAL-410 do not leak into this diff.

## Rollback Point

Revert the work commit. No case schema, research artifact, CI workflow, or
package payload migration is required.
