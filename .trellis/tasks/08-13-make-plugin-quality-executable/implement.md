# Implementation Plan

## 1. Replace The Fake Score Contract In Focused Tests

- [ ] Rewrite `tests/test_academic_quality_evals.py` to require all 12 V1 cases,
      contained fixture roots, executed assertions, and no declared dimensions.
- [ ] Add one temporary-fixture mutation that removes a required finding and
      proves the case/batch fails.
- [ ] Cover empty, missing, malformed, and zero-assertion batch failure without
      duplicating V1 runner tests.

Focused red/green command:

```bash
python3 -m unittest \
  tests.test_academic_quality_evals \
  tests.test_eval_cases -v
```

## 2. Convert The 12 Cases Through Evaluation Truth V1

- [ ] Add V1 identity/input/expected-output fields to every academic case.
- [ ] Add one bounded captured artifact fixture per case with concrete positive
      evidence or exact expected gate/severity/finding evidence.
- [ ] Remove all `expected_dimensions` data.
- [ ] Reduce `run_academic_quality_evals.py` to a contained batch loop over the
      existing `run_case`; emit counts and a non-zero exit when any case fails.
- [ ] Update the README to state exactly what the captured fixtures do and do
      not prove.

## 3. Repair The Eight Canonical Skills

- [ ] Update only the eight PRD-listed canonical files at their existing
      semantic sections.
- [ ] Add task-specific insufficient-input, finding/interpretation/implication,
      and non-fabrication behavior without changing purpose or artifacts.
- [ ] Review the diff manually for semantic usefulness, then generate the report
      and require 82/82:

```bash
python3 scripts/audit_skill_sections.py \
  --output docs/maintainer/skill-quality-gap-report.md
python3 scripts/audit_skill_sections.py --strict
```

## 4. Verify The Staged Plugin Payload

- [ ] Materialize all targets into a fresh temporary directory, audit the
      payloads, and discard the directory afterward.
- [ ] Run capability and Coursework/Dissertation contract tests.
- [ ] Run the existing Codex/Claude Plugin bundle tests so the repaired
      canonical Skill content is present in both staged Host forms.

Commands:

```bash
python3 scripts/validate_capability_contract.py
python3 -m unittest \
  tests.test_skill_contract_alignment \
  tests.test_coursework_dissertation_contract -v
python3 scripts/materialize_distribution_payloads.py \
  --target all --out <fresh-temp-directory> --force
python3 scripts/audit_distribution_payloads.py \
  --root <fresh-temp-directory>
cargo test --manifest-path packages/qiongli-native/Cargo.toml \
  -p qiongli --test codex_plugin_bundle --test claude_plugin_bundle --locked
```

## 5. Freeze And Check Once

- [ ] Run the academic batch directly and require 12/12 passing executable
      cases.
- [ ] Run the repository Python suite once after the content/eval diff is frozen.
- [ ] Run formatting/diff hygiene and confirm no generated mirror or temporary
      receipt entered Git.

```bash
python3 scripts/run_academic_quality_evals.py evals/academic_quality/cases
python3 -m unittest discover -s tests -v
git diff --check
git status --short
```

- [ ] Do not run `claude plugin eval` or any paid/authenticated model call in
      this deterministic task.

## 6. Record Only EVAL-409

- [ ] Update the roadmap/checklist only after the deterministic gate, canonical
      audit, and staged Plugin checks pass.
- [ ] State that captured outputs test finding detection and Skill contracts,
      not live model quality.
- [ ] Run Trellis check/update-spec, commit, and archive P1. Leave EVAL-410,
      EVAL-411, M1 exit, and release qualification open.

## Review Focus

- No self-declared number can make a case green.
- The V1 runner remains the only truth owner.
- Skill edits are useful instructions, not keyword padding.
- Generated Plugin trees and model outputs remain outside the commit.

## Rollback Point

Revert the atomic eval/content commit. There is no runtime, Host, or user-data
rollback. Do not restore old dimension averages as accepted evidence.
