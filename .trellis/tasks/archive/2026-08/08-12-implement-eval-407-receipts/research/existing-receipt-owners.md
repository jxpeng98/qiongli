# Existing receipt owners and constraints

## Shared eval owner

- `evals/runner/run_eval.py` owns shared-case parsing, assertion dispatch, the
  five truth counters, human output, boolean API, and CLI exit status.
- `tests/test_eval_cases.py` is the focused contract owner. It already covers
  pass/fail/block/skip behavior and all nine V1 assertion types.

## Reusable repository conventions

- Python tools render deterministic JSON with `sort_keys=True`, explicit UTF-8,
  and a trailing newline for checked artifacts.
- `tooling/migration/ctr-201-cli-runtime.schema.json` constrains reason codes to
  lower-case hyphenated tokens: `^[a-z0-9][a-z0-9-]*$`.
- `tooling/scripts/run_full_cycle_workflow_harness.py` demonstrates an explicit
  report-path CLI boundary and parent-directory creation, but its JSON lacks
  stable key ordering and atomic replacement; reuse the boundary, not those
  omissions.
- The repository contains no JUnit renderer or dependency. Use
  `xml.etree.ElementTree` and test the result with the same standard library.

## Privacy and evidence boundary

- The master roadmap requires zero receipt leakage of credentials, absolute
  paths, Host content, or restricted research data.
- The minimum useful eval evidence identity is role + root-relative path +
  exact-byte SHA-256. Do not copy artifact contents or exception paths into a
  receipt.
- Case and output roots are execution context, not portable evidence identity.

## Deferred owners

- EVAL-410 owns the canonical multi-case CI command and compatibility shims.
- EVAL-408 owns a broad checked-in adversarial fixture corpus.
- Governance authorization receipts are a different roadmap contract and must
  not be merged with evaluation-result receipts.
