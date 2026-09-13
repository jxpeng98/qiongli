# Implement EVAL-407 deterministic receipts

## Goal

Turn every shared golden-eval outcome into portable CI evidence by emitting
deterministic JSON and JUnit receipts that identify the case, each assertion,
its redacted evidence, status, and stable reason code. Preserve the existing
five-clause truth predicate, boolean API, human-readable output, and CLI exit
meaning.

## Confirmed Facts

- `evals/runner/run_eval.py::run_case` is the sole owner of the four shared
  golden cases. It currently returns only a boolean and prints free-form output
  plus five counters; assertion outcomes are not retained structurally.
- Roadmap item `EVAL-407` explicitly requires deterministic JSON and JUnit
  receipts with case, assertion, evidence, status, and reason code.
- `EVAL-410`, not this task, will later choose one canonical eval command and
  make it own CI. This task must expose a usable receipt boundary without
  prematurely changing CI ownership.
- The repository has no JUnit emitter or extra XML dependency. Python's
  `json` and `xml.etree.ElementTree` are sufficient.
- Existing machine-readable reason codes use lower-case hyphenated tokens,
  enforced elsewhere by `^[a-z0-9][a-z0-9-]*$`.
- The roadmap requires receipt leakage of credentials, absolute paths, Host
  content, and restricted research data to remain zero. Eval receipts therefore
  must identify evidence by contained relative path and SHA-256, never by raw
  artifact content or resolved absolute path.

## Requirements

### R1 — One evaluation truth, two receipt formats

- JSON and JUnit must be rendered from the same assertion-outcome records that
  determine the existing boolean result; neither renderer may reinterpret
  pass/fail truth.
- Preserve all five truth counters and the exact mandatory success predicate.
- Represent successful, failed, blocked, and skipped assertion outcomes. When
  case/config validation prevents a real assertion record from existing, emit
  one explicit case-contract outcome instead of an empty green receipt.

### R2 — Stable, complete JSON contract

- Use a versioned JSON object containing case identity, pipeline, case status,
  case reason code, the five counters, and an ordered assertion list.
- Each assertion outcome includes output ID, zero-based assertion index,
  assertion type, status, stable reason code, human-readable message, and a
  deterministic evidence list.
- Evidence entries contain only role, contained relative path, and exact-byte
  SHA-256 when the evidence exists; missing evidence uses no fabricated digest.

### R3 — CI-compatible JUnit projection

- Emit one `<testsuite>` per evaluated case and one `<testcase>` per recorded
  assertion/case-contract outcome.
- Map `fail` to `<failure>`, `blocked` to `<error>`, `skip` to `<skipped>`, and
  `pass` to a childless testcase. Expose status, reason code, and evidence as
  deterministic testcase properties.
- Omit timestamps, host paths, elapsed times, random IDs, and other volatile
  fields. Counts must agree with the emitted testcase children.

### R4 — Determinism, privacy, and write safety

- Identical case and artifact bytes must produce byte-identical JSON and JUnit
  receipts across repeated runs.
- Serialization order, newline policy, XML element/attribute order, record
  ordering, and reason-code vocabulary must be stable.
- Receipt-write errors must make the CLI non-success and must not leave a
  partially written target file.

### R5 — Explicit emission boundary

- Add independent optional CLI arguments `--json-receipt PATH` and
  `--junit-receipt PATH`. Either, both, or neither may be supplied.
- With neither option, preserve today's read-only behavior and create no
  receipt. Relative receipt destinations are interpreted from the caller's
  current working directory.
- Evaluate the case exactly once, then render every requested format from that
  result. Reject destinations that resolve to the same path; create parent
  directories and replace each requested file atomically.

### R6 — Compatibility and narrow scope

- Preserve `run_case(case_path, output_dir) -> bool`, existing direct callers,
  the current positional CLI arguments, human-readable lines, and exit status.
- Keep capture/render/write helpers in the existing runner; do not add a receipt
  framework, new package, dependency, generated schema system, or CI command.
- Extend `tests/test_eval_cases.py` with focused receipt contract, semantic
  mapping, determinism, redaction, and CLI-write-failure checks.

## Acceptance Criteria

- [x] One passing, one failing, one blocked, and one skipped assertion are
      represented consistently in JSON and JUnit with stable reason codes.
- [x] Case-contract failures and zero-execution cases cannot yield an empty or
      green receipt.
- [x] Every evidence path is relative and every available evidence digest
      matches exact bytes; receipts contain no artifact content or absolute
      case/output paths.
- [x] Running the same evaluation twice yields byte-identical JSON and JUnit.
- [x] JUnit parses with the standard library and its tests/failures/errors/
      skipped counts match the assertion records.
- [x] Existing callers, human output, truth counters, success predicate, and CLI
      exit behavior remain compatible.
- [x] The two receipt flags work independently and together; omitting both
      creates no file, while resolving both to the same target is rejected.
- [x] Receipt destinations fail closed on write error without a partial target.
- [x] Focused eval tests, the final repository unit-test pass, `py_compile`, and
      `git diff --check` succeed.
- [x] Evaluation Truth V1 documents the receipt contract and only `EVAL-407` is
      marked complete after verification.

## Out Of Scope

- EVAL-408 adversarial fixture corpus, EVAL-409 academic-quality conversion,
  EVAL-410 canonical CI ownership, and EVAL-411 mutation testing.
- Receipt aggregation across multiple case files, CI upload/publishing,
  historical comparison, timing/performance data, signatures, timestamps,
  authorization receipts, or full JUnit dialect coverage.
- Raw evidence excerpts, Host/model transcripts, credentials, user directory
  paths, or restricted research content.

## Notes

- This is Alpha 4 evaluation-truth work and grants no Alpha 3 publication claim.
