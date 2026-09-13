# Implement EVAL-406 scientific validators

## Goal

Extend the existing version-1 golden-eval contract with the seven scientific
validators required by roadmap item `EVAL-406`: schema, field constraint,
count conservation, cross-artifact consistency, locator syntax, citation
identity, and file digest. Keep the current runner, boolean/CLI API, and exact
five-clause success predicate; do not start the later receipt, fixture-corpus,
CI-owner, or academic-quality work.

## Confirmed Facts

- `evals/runner/run_eval.py::run_case` is the sole owner of the four shared
  golden cases and already rejects unknown or malformed V1 assertions.
- The runner currently supports only `contains_all` and `contains_any`, and
  assumes every assertion has a non-empty string `values` list.
- The repository already contains a tested JSON Schema subset validator in
  `tooling/scripts/validate_capability_contract.py` and a tested evidence-ledger
  to BibTeX identity audit in `tooling/scripts/audit_citation_risk.py`.
- Python 3.12, PyYAML, `csv`, `json`, `hashlib`, `re`, and `pathlib` cover the
  remaining work. The project does not declare `jsonschema`, and this task does
  not need it.
- Canonical research artifacts use JSON/YAML, CSV, Markdown, and BibTeX. The
  PRISMA template expresses counts as `Label: n = N`; evidence locators use
  `p. N`, `pp. N-N`, or `citekey:anchor` forms.

## Requirements

### R1 — Preserve and extend Case Contract V1

- Keep `schema_version: "1.0"`, `contains_all`, and `contains_any` compatible.
- Add these recognized types with strict, type-specific payloads:
  - `schema`: `schema`, a case-relative JSON Schema path;
  - `field_constraint`: `field` plus non-empty `allowed_values` for a CSV
    column;
  - `count_conservation`: `total` label plus at least one `parts` label in a
    UTF-8 `Label: n = N` artifact;
  - `cross_artifact_consistency`: `field`, output-relative `other_artifact`,
    `other_field`, and `relation` (`equal` or `subset`) for CSV values;
  - `locator_syntax`: `field` for present CSV locators;
  - `citation_identity`: output-relative `bibliography` for an evidence-ledger
    CSV and BibTeX pair;
  - `file_digest`: a 64-hex-character `sha256` value.
- Reject missing, malformed, unknown, or extra type-specific fields instead of
  guessing intent.

### R2 — Give every validator executable, non-vacuous semantics

- `schema` parses a JSON/YAML artifact and validates it with the repository's
  existing JSON Schema subset implementation.
- `field_constraint` requires a CSV header, at least one row, a present column,
  and a non-empty allowed value in every row.
- `count_conservation` requires exactly one non-negative integer for each
  configured label and proves `total == sum(parts)`.
- `cross_artifact_consistency` requires non-empty columns and compares trimmed
  CSV values with multiplicity using the configured equality/subset relation.
- `locator_syntax` checks every present locator, requires at least one present
  locator, and accepts only page/page-range or non-empty `citekey:anchor` forms.
- `citation_identity` requires at least one citable `paper`/`theory` row and
  proves every declared source ID exists as a BibTeX citekey by reusing the
  existing citation audit.
- `file_digest` computes SHA-256 over the artifact's exact bytes.

### R3 — Fail closed without changing the truth predicate

- Artifact, schema, bibliography, and cross-artifact paths must be relative and
  remain inside their case/output roots.
- Invalid assertion configuration, missing referenced artifacts, decode/parse
  errors, missing fields, duplicate count labels, and validators with no
  applicable data are `BLOCKED`, not passes.
- A known, well-formed validator increments `executed_assertions` only after its
  input was parsed and its scientific comparison ran. A false comparison
  increments `failed_assertions`.
- Preserve the mandatory success predicate exactly:

```text
required_missing == 0
executed_assertions > 0
failed_assertions == 0
blocked_assertions == 0
unknown_validation_types == 0
```

### R4 — Integrate at the narrowest useful boundary

- Keep all dispatch and validator glue in `evals/runner/run_eval.py`; do not add
  a validator package, registry class, expression language, or result model.
- Add one count-conservation assertion to the existing systematic-review case,
  where the current PRISMA artifact already owns the relevant equation.
- Extend `tests/test_eval_cases.py` with temporary, table-driven positive,
  mismatch, and malformed-input checks for all seven validators. Do not create
  the broad adversarial fixture corpus reserved for `EVAL-408`.
- After verification, update the narrow Evaluation Truth V1 spec and mark only
  `EVAL-406` complete in the master roadmap.

### R5 — Preserve compatibility

- Preserve `run_case(case_path, output_dir) -> bool`, its CLI exit behavior, and
  existing human-readable output.
- Do not change the academic-quality, controller-mode, or subject-specialization
  evaluators.
- Use only the current standard library and PyYAML dependency.

## Acceptance Criteria

- [x] All seven EVAL-406 types have strict payload validation and executable,
      non-vacuous behavior; the two existing contains validators remain valid.
- [x] A valid focused case exercising all seven scientific validators passes.
- [x] A schema violation, disallowed field value, broken count equation,
      cross-artifact mismatch, malformed locator, missing citekey, or wrong
      digest makes its case fail.
- [x] Malformed type-specific configuration, unreadable or malformed referenced
      data, empty applicable data, and path escape attempts block rather than
      pass or raise an uncaught exception.
- [x] The systematic-review shared case executes a real PRISMA count equation;
      all four shared cases still pass with valid minimal fixtures.
- [x] The five-clause truth predicate, boolean API, CLI exit behavior, and text
      counters remain intact.
- [x] Focused eval tests, the final repository unit-test pass, and
      `git diff --check` succeed.
- [x] Evaluation Truth V1 documents the implemented payloads and semantics, and
      the roadmap marks only `EVAL-406` complete after those checks pass.

## Out Of Scope

- EVAL-407 JSON/JUnit receipts, EVAL-408 broad adversarial fixture files,
  EVAL-409 academic-quality conversion, EVAL-410 CI command ownership, and
  EVAL-411 mutation testing.
- Full JSON Schema Draft 2020-12 support, JSONPath, a general constraint DSL,
  arbitrary Markdown-table parsing, semantic locator verification, DOI lookup,
  source relevance/support judgment, or file-tree digests.
- Kernel schema migration, Desktop/Host/provider work, release qualification,
  tags, publication, or a new dependency.

## Open Questions

None. The roadmap, current artifact contracts, and existing reusable validators
determine a bounded initial implementation.

## Notes

- This task targets Alpha 4 development and grants no Alpha 3 release or
  publication claim.
- The intentionally narrow CSV/text contracts can be extended under a later
  schema version when an executable case proves another operation is needed.
