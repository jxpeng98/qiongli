# Subject Specialization Roadmap Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the next subject-specialization layer: measurable subject-depth audits, richer economics guidance, a standalone accounting subject, explicit composite metadata, and an easier local customization workflow.

**Architecture:** Keep `subjects/catalog.yaml` plus the Python materializer as the source of truth. Add small audit and eval scripts that inspect catalog definitions and materialized packages instead of invoking an LLM. Deepen official subjects through overlays, subject-specific skills, selected profiles, and manifest metadata while preserving `coverage=complete` as the default installation mode.

**Tech Stack:** Python 3 stdlib `unittest`, PyYAML, existing `qiongli.subject_materializer`, Node/npm package tests, existing marketplace and distribution validators.

---

## Current State

- `dev` already contains subject v1 plus coverage-aware installs.
- Existing official subjects are `core`, `economics`, and `economics-accounting`.
- `coverage=complete` means full core framework plus selected subject overlays and subject-specific skills.
- `coverage=focused` means a slim selected subject package, mainly for Desktop/Web ZIPs and deliberate narrow installs.
- `custom_dir` exists in the Python materializer, but users do not yet have a first-class scaffold command.
- The main checkout has a dirty `docs/public/mark.svg`; perform all work in a `dev` worktree and do not touch the main checkout.

## File Structure

- Create `scripts/audit_subject_specialization.py`
  - Validates that each official non-core subject has enough domain-specific depth in both focused and complete outputs.
  - Produces deterministic text or JSON findings for CI and release checks.
- Create `scripts/audit_subject_eval_cases.py`
  - Loads subject eval fixture YAML files and checks materialized package contents against expected skills, profiles, overlays, and forbidden profiles.
- Create `tests/test_subject_specialization_audit.py`
  - Unit tests for subject-depth audits and failure messages.
- Create `tests/test_subject_eval_cases.py`
  - Unit tests for eval fixture schema and materialized package checks.
- Create `evals/subject-specialization/cases/*.yaml`
  - Static subject-quality cases for economics, accounting, and economics-accounting.
- Modify `subjects/catalog.yaml`
  - Add `accounting`, deepen `economics`, and add metadata for composite subjects.
- Modify `qiongli/subject_materializer.py`
  - Add optional composite metadata to subject definitions and manifest layer rendering.
- Modify `subjects/economics/**`
  - Add economics v2 overlays, venue profiles, and one additional subject-specific skill.
- Create `subjects/accounting/**`
  - Add official accounting subject overlays, registry, skill, and venue profiles.
- Modify `subjects/economics-accounting/**`
  - Reuse accounting and economics subject assets through explicit catalog references, while keeping composite-specific groups and overlays.
- Modify `qiongli/cli.py` or the existing CLI entrypoint
  - Add a local customization scaffold command.
- Modify `scripts/validate_marketplace_install.py`
  - Include subject audit and eval fixture checks in release validation.
- Modify docs and READMEs
  - Document subject depth, accounting, composite selection, eval expectations, and customization scaffolds.

## Commit Plan

1. `test(subjects): add specialization quality audit`
2. `test(subjects): add subject specialization eval cases`
3. `feat(subjects): deepen economics method coverage`
4. `feat(subjects): add accounting subject`
5. `feat(subjects): declare composite subject layers`
6. `feat(subjects): scaffold local custom subject overlays`
7. `build(release): validate subject specialization quality`
8. `docs(subjects): document subject depth roadmap`

### Task 1: Specialization Quality Audit

**Files:**
- Create: `scripts/audit_subject_specialization.py`
- Create: `tests/test_subject_specialization_audit.py`
- Modify: `scripts/validate_marketplace_install.py`

- [ ] **Step 1: Write failing audit tests**

Add this test file:

```python
from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from scripts.audit_subject_specialization import (
    SubjectSpecializationFinding,
    audit_subject_specialization,
)


REPO_ROOT = Path(__file__).resolve().parents[1]


class SubjectSpecializationAuditTests(unittest.TestCase):
    def test_current_subjects_pass_depth_audit(self) -> None:
        findings = audit_subject_specialization(REPO_ROOT)
        self.assertEqual(findings, [])

    def test_focused_output_excludes_unselected_profiles(self) -> None:
        findings = audit_subject_specialization(REPO_ROOT, subjects=["economics"])
        self.assertEqual([finding.code for finding in findings], [])

    def test_missing_overlay_term_is_reported(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            target = root / "repo"
            self._copy_minimal_repo(REPO_ROOT, target)
            overlay = target / "subjects" / "economics" / "overlays" / "skills" / "manuscript-architect.md"
            overlay.write_text("## Economics Overlay\n\nGeneric manuscript guidance.\n", encoding="utf-8")

            findings = audit_subject_specialization(target, subjects=["economics"])

        self.assertTrue(any(finding.code == "missing-subject-term" for finding in findings))

    def _copy_minimal_repo(self, source: Path, target: Path) -> None:
        import shutil

        ignore = shutil.ignore_patterns(".git", ".worktrees", ".venv", "node_modules", "dist", "build")
        shutil.copytree(source, target, ignore=ignore)
```

- [ ] **Step 2: Run tests to verify failure**

Run:

```bash
.venv/bin/python -m unittest tests/test_subject_specialization_audit.py -v
```

Expected: fail with `ModuleNotFoundError: No module named 'scripts.audit_subject_specialization'`.

- [ ] **Step 3: Implement the audit script**

Create `scripts/audit_subject_specialization.py` with:

```python
from __future__ import annotations

import argparse
import json
import tempfile
from dataclasses import asdict, dataclass
from pathlib import Path

import yaml

from qiongli.subject_materializer import MaterializeOptions, materialize_subject_package, validate_subject_catalog


SUBJECT_TERMS = {
    "economics": ("identification", "estimand", "robustness", "causal"),
    "accounting": ("accrual", "disclosure", "measurement", "audit"),
    "economics-accounting": ("identification", "disclosure", "measurement", "causal"),
}

UNRELATED_FOCUSED_PROFILES = {
    "economics": {"cs-ai.yaml", "biomedical.yaml", "psychology.yaml"},
    "accounting": {"cs-ai.yaml", "biomedical.yaml", "psychology.yaml", "economics.yaml"},
    "economics-accounting": {"cs-ai.yaml", "biomedical.yaml", "psychology.yaml"},
}


@dataclass(frozen=True)
class SubjectSpecializationFinding:
    subject: str
    code: str
    message: str


def audit_subject_specialization(root: Path, subjects: list[str] | None = None) -> list[SubjectSpecializationFinding]:
    root = Path(root)
    catalog = validate_subject_catalog(root)
    subject_ids = subjects or sorted(subject_id for subject_id in catalog.subjects if subject_id != "core")
    findings: list[SubjectSpecializationFinding] = []
    for subject_id in subject_ids:
        subject = catalog.subjects[subject_id]
        if subject_id == "core":
            continue
        if not subject.domain_profiles:
            findings.append(_finding(subject_id, "missing-domain-profiles", "non-core subject has no domain profiles"))
        if not subject.venue_profiles:
            findings.append(_finding(subject_id, "missing-venue-profiles", "non-core subject has no venue profiles"))
        if len(subject.skill_overrides) + len(subject.subject_specific_skill_refs) < 2:
            findings.append(_finding(subject_id, "thin-subject-layer", "subject needs at least two overlays or subject skills"))
        findings.extend(_audit_materialized_outputs(root, subject_id))
    return findings


def _audit_materialized_outputs(root: Path, subject_id: str) -> list[SubjectSpecializationFinding]:
    findings: list[SubjectSpecializationFinding] = []
    with tempfile.TemporaryDirectory() as tmp_dir:
        base = Path(tmp_dir)
        focused = base / "focused" / "qiongli-workflow"
        complete = base / "complete" / "qiongli-workflow"
        materialize_subject_package(
            MaterializeOptions(source=root, out=focused, subject=subject_id, flavor="full", coverage="focused")
        )
        materialize_subject_package(
            MaterializeOptions(source=root, out=complete, subject=subject_id, flavor="full", coverage="complete")
        )
        findings.extend(_audit_focused_profiles(focused, subject_id))
        findings.extend(_audit_subject_terms(complete, subject_id))
    return findings


def _audit_focused_profiles(package_root: Path, subject_id: str) -> list[SubjectSpecializationFinding]:
    profile_root = package_root / "skills" / "domain-profiles"
    present = {path.name for path in profile_root.glob("*.yaml")}
    forbidden = UNRELATED_FOCUSED_PROFILES.get(subject_id, set()) & present
    if forbidden:
        joined = ", ".join(sorted(forbidden))
        return [_finding(subject_id, "unrelated-focused-profile", f"focused output includes unrelated profiles: {joined}")]
    return []


def _audit_subject_terms(package_root: Path, subject_id: str) -> list[SubjectSpecializationFinding]:
    wanted = SUBJECT_TERMS.get(subject_id, ())
    if not wanted:
        return []
    searchable = []
    for path in sorted((package_root / "skills").glob("**/*.md")):
        searchable.append(path.read_text(encoding="utf-8").lower())
    text = "\n".join(searchable)
    missing = [term for term in wanted if term not in text]
    if missing:
        joined = ", ".join(missing)
        return [_finding(subject_id, "missing-subject-term", f"materialized skills are missing terms: {joined}")]
    return []


def _finding(subject: str, code: str, message: str) -> SubjectSpecializationFinding:
    return SubjectSpecializationFinding(subject=subject, code=code, message=message)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", default=".")
    parser.add_argument("--subject", action="append", dest="subjects")
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()
    findings = audit_subject_specialization(Path(args.root), args.subjects)
    if args.json:
        print(json.dumps([asdict(finding) for finding in findings], indent=2, sort_keys=True))
    else:
        for finding in findings:
            print(f"{finding.subject}: {finding.code}: {finding.message}")
    return 1 if findings else 0


if __name__ == "__main__":
    raise SystemExit(main())
```

- [ ] **Step 4: Run audit tests**

Run:

```bash
.venv/bin/python -m unittest tests/test_subject_specialization_audit.py -v
```

Expected: pass.

- [ ] **Step 5: Wire audit into release validation**

In `scripts/validate_marketplace_install.py`, import `audit_subject_specialization` and fail if findings are returned:

```python
from scripts.audit_subject_specialization import audit_subject_specialization


def _validate_subject_specialization(root: Path) -> None:
    findings = audit_subject_specialization(root)
    if findings:
        details = "\n".join(f"{f.subject}: {f.code}: {f.message}" for f in findings)
        raise SystemExit(f"Subject specialization audit failed:\n{details}")
```

Call `_validate_subject_specialization(REPO_ROOT)` from the existing main validation flow after subject ZIP validation.

- [ ] **Step 6: Run validation command**

Run:

```bash
.venv/bin/python scripts/audit_subject_specialization.py --root .
```

Expected: exit 0 with no output.

- [ ] **Step 7: Commit**

```bash
git add scripts/audit_subject_specialization.py tests/test_subject_specialization_audit.py scripts/validate_marketplace_install.py
git commit -m "test(subjects): add specialization quality audit"
```

### Task 2: Subject Eval Fixtures

**Files:**
- Create: `evals/subject-specialization/cases/economics-did-identification.yaml`
- Create: `evals/subject-specialization/cases/economics-accounting-disclosure-study.yaml`
- Create: `scripts/audit_subject_eval_cases.py`
- Create: `tests/test_subject_eval_cases.py`
- Modify: `scripts/validate_marketplace_install.py`

- [ ] **Step 1: Add eval fixture YAML files**

Create `evals/subject-specialization/cases/economics-did-identification.yaml`:

```yaml
id: economics-did-identification
subject: economics
coverage: complete
prompt: "Design a difference-in-differences economics study with staggered adoption and firm-level outcomes."
expected_skill_refs:
  - stats-engine
  - econ-identification-auditor
expected_terms:
  - parallel trends
  - clustered standard errors
  - identification strategy
expected_domain_profiles:
  - economics.yaml
forbidden_domain_profiles: []
```

Create `evals/subject-specialization/cases/economics-accounting-disclosure-study.yaml`:

```yaml
id: economics-accounting-disclosure-study
subject: economics-accounting
coverage: complete
prompt: "Plan a causal archival study of mandatory disclosure rules and capital-market outcomes."
expected_skill_refs:
  - econ-identification-auditor
  - accounting-measurement-auditor
expected_terms:
  - disclosure
  - identification
  - measurement
expected_domain_profiles:
  - economics.yaml
  - accounting.yaml
forbidden_domain_profiles: []
```

- [ ] **Step 2: Write failing eval audit tests**

Create `tests/test_subject_eval_cases.py`:

```python
from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from scripts.audit_subject_eval_cases import SubjectEvalFinding, audit_subject_eval_cases, load_subject_eval_cases


REPO_ROOT = Path(__file__).resolve().parents[1]


class SubjectEvalCaseTests(unittest.TestCase):
    def test_eval_cases_load(self) -> None:
        cases = load_subject_eval_cases(REPO_ROOT / "evals" / "subject-specialization" / "cases")
        case_ids = {case.id for case in cases}
        self.assertIn("economics-did-identification", case_ids)
        self.assertIn("economics-accounting-disclosure-study", case_ids)

    def test_eval_cases_pass_against_materialized_outputs(self) -> None:
        findings = audit_subject_eval_cases(REPO_ROOT)
        self.assertEqual(findings, [])

    def test_missing_expected_term_is_reported(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            cases = root / "cases"
            cases.mkdir(parents=True)
            (cases / "broken.yaml").write_text(
                "\n".join(
                    [
                        "id: broken",
                        "subject: economics",
                        "coverage: focused",
                        "prompt: broken",
                        "expected_skill_refs: [stats-engine]",
                        "expected_terms: [term-that-does-not-exist]",
                        "expected_domain_profiles: [economics.yaml]",
                        "forbidden_domain_profiles: []",
                    ]
                ),
                encoding="utf-8",
            )
            findings = audit_subject_eval_cases(REPO_ROOT, case_dir=cases)
        self.assertTrue(any(finding.code == "missing-expected-term" for finding in findings))
```

- [ ] **Step 3: Run tests to verify failure**

Run:

```bash
.venv/bin/python -m unittest tests/test_subject_eval_cases.py -v
```

Expected: fail with `ModuleNotFoundError: No module named 'scripts.audit_subject_eval_cases'`.

- [ ] **Step 4: Implement eval audit script**

Create `scripts/audit_subject_eval_cases.py` with:

```python
from __future__ import annotations

import argparse
import json
import tempfile
from dataclasses import asdict, dataclass
from pathlib import Path

import yaml

from qiongli.subject_materializer import MaterializeOptions, materialize_subject_package


@dataclass(frozen=True)
class SubjectEvalCase:
    id: str
    subject: str
    coverage: str
    prompt: str
    expected_skill_refs: tuple[str, ...]
    expected_terms: tuple[str, ...]
    expected_domain_profiles: tuple[str, ...]
    forbidden_domain_profiles: tuple[str, ...]


@dataclass(frozen=True)
class SubjectEvalFinding:
    case_id: str
    code: str
    message: str


def load_subject_eval_cases(case_dir: Path) -> list[SubjectEvalCase]:
    cases: list[SubjectEvalCase] = []
    for path in sorted(Path(case_dir).glob("*.yaml")):
        raw = yaml.safe_load(path.read_text(encoding="utf-8")) or {}
        cases.append(
            SubjectEvalCase(
                id=_required_str(raw, "id", path),
                subject=_required_str(raw, "subject", path),
                coverage=_required_str(raw, "coverage", path),
                prompt=_required_str(raw, "prompt", path),
                expected_skill_refs=tuple(_required_list(raw, "expected_skill_refs", path)),
                expected_terms=tuple(_required_list(raw, "expected_terms", path)),
                expected_domain_profiles=tuple(_required_list(raw, "expected_domain_profiles", path)),
                forbidden_domain_profiles=tuple(_required_list(raw, "forbidden_domain_profiles", path)),
            )
        )
    return cases


def audit_subject_eval_cases(root: Path, case_dir: Path | None = None) -> list[SubjectEvalFinding]:
    root = Path(root)
    case_dir = case_dir or root / "evals" / "subject-specialization" / "cases"
    findings: list[SubjectEvalFinding] = []
    for case in load_subject_eval_cases(case_dir):
        with tempfile.TemporaryDirectory() as tmp_dir:
            out = Path(tmp_dir) / "qiongli-workflow"
            materialize_subject_package(
                MaterializeOptions(source=root, out=out, subject=case.subject, flavor="full", coverage=case.coverage)
            )
            findings.extend(_check_case(case, out))
    return findings


def _check_case(case: SubjectEvalCase, package_root: Path) -> list[SubjectEvalFinding]:
    findings: list[SubjectEvalFinding] = []
    registry = yaml.safe_load((package_root / "skills" / "registry.yaml").read_text(encoding="utf-8"))
    registry_ids = {entry["id"] for entry in registry["skills"]}
    missing_skills = sorted(set(case.expected_skill_refs) - registry_ids)
    if missing_skills:
        findings.append(_finding(case.id, "missing-expected-skill", ", ".join(missing_skills)))
    profile_names = {path.name for path in (package_root / "skills" / "domain-profiles").glob("*.yaml")}
    missing_profiles = sorted(set(case.expected_domain_profiles) - profile_names)
    forbidden_profiles = sorted(set(case.forbidden_domain_profiles) & profile_names)
    if missing_profiles:
        findings.append(_finding(case.id, "missing-expected-profile", ", ".join(missing_profiles)))
    if forbidden_profiles:
        findings.append(_finding(case.id, "forbidden-profile-present", ", ".join(forbidden_profiles)))
    text = "\n".join(path.read_text(encoding="utf-8").lower() for path in sorted((package_root / "skills").glob("**/*.md")))
    missing_terms = [term for term in case.expected_terms if term.lower() not in text]
    if missing_terms:
        findings.append(_finding(case.id, "missing-expected-term", ", ".join(missing_terms)))
    return findings


def _required_str(raw: dict, key: str, path: Path) -> str:
    value = raw.get(key)
    if not isinstance(value, str) or not value.strip():
        raise ValueError(f"{path}: {key} must be a non-empty string")
    return value


def _required_list(raw: dict, key: str, path: Path) -> list[str]:
    value = raw.get(key)
    if not isinstance(value, list) or not all(isinstance(item, str) for item in value):
        raise ValueError(f"{path}: {key} must be a string list")
    return value


def _finding(case_id: str, code: str, message: str) -> SubjectEvalFinding:
    return SubjectEvalFinding(case_id=case_id, code=code, message=message)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", default=".")
    parser.add_argument("--case-dir")
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()
    findings = audit_subject_eval_cases(Path(args.root), Path(args.case_dir) if args.case_dir else None)
    if args.json:
        print(json.dumps([asdict(finding) for finding in findings], indent=2, sort_keys=True))
    else:
        for finding in findings:
            print(f"{finding.case_id}: {finding.code}: {finding.message}")
    return 1 if findings else 0


if __name__ == "__main__":
    raise SystemExit(main())
```

- [ ] **Step 5: Run eval audit tests**

Run:

```bash
.venv/bin/python -m unittest tests/test_subject_eval_cases.py -v
```

Expected: pass.

- [ ] **Step 6: Wire eval audit into release validation**

Add this to `scripts/validate_marketplace_install.py`:

```python
from scripts.audit_subject_eval_cases import audit_subject_eval_cases


def _validate_subject_eval_cases(root: Path) -> None:
    findings = audit_subject_eval_cases(root)
    if findings:
        details = "\n".join(f"{f.case_id}: {f.code}: {f.message}" for f in findings)
        raise SystemExit(f"Subject eval case audit failed:\n{details}")
```

Call `_validate_subject_eval_cases(REPO_ROOT)` after `_validate_subject_specialization(REPO_ROOT)`.

- [ ] **Step 7: Commit**

```bash
git add evals/subject-specialization/cases scripts/audit_subject_eval_cases.py tests/test_subject_eval_cases.py scripts/validate_marketplace_install.py
git commit -m "test(subjects): add subject specialization eval cases"
```

### Task 3: Economics v2 Depth

**Files:**
- Modify: `subjects/catalog.yaml`
- Create: `subjects/economics/overlays/skills/study-designer.md`
- Create: `subjects/economics/overlays/skills/robustness-planner.md`
- Create: `subjects/economics/overlays/skills/analysis-interpreter.md`
- Create: `subjects/economics/skills/econ-replication-package-auditor.md`
- Modify: `subjects/economics/skills/registry.yaml`
- Create: `subjects/economics/venue-profiles/econometrica.yaml`
- Create: `subjects/economics/venue-profiles/jpe.yaml`
- Modify: `tests/test_subject_materializer.py`
- Modify: `evals/subject-specialization/cases/economics-did-identification.yaml`

- [ ] **Step 1: Add failing materializer assertions**

Append to `tests/test_subject_materializer.py`:

```python
    def test_economics_complete_contains_v2_method_depth(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            out = Path(tmp_dir) / "qiongli-workflow"

            materialize_subject_package(
                MaterializeOptions(source=REPO_ROOT, out=out, subject="economics", flavor="full", coverage="complete")
            )

            registry = yaml.safe_load((out / "skills" / "registry.yaml").read_text(encoding="utf-8"))
            registry_ids = {entry["id"] for entry in registry["skills"]}
            self.assertIn("econ-replication-package-auditor", registry_ids)
            self.assertTrue((out / "venue-profiles" / "econometrica.yaml").exists())
            self.assertTrue((out / "venue-profiles" / "jpe.yaml").exists())
            study_designer = (out / "skills" / "C_design" / "study-designer.md").read_text(encoding="utf-8")
            robustness = (out / "skills" / "C_design" / "robustness-planner.md").read_text(encoding="utf-8")
            interpreter = (out / "skills" / "F_writing" / "analysis-interpreter.md").read_text(encoding="utf-8")
            self.assertIn("## Economics Overlay", study_designer)
            self.assertIn("identification threat", robustness)
            self.assertIn("economic magnitude", interpreter)
```

- [ ] **Step 2: Run tests to verify failure**

Run:

```bash
.venv/bin/python -m unittest tests/test_subject_materializer.py -v
```

Expected: fail because the new economics overlays, skill, and venue profiles do not exist.

- [ ] **Step 3: Update economics catalog metadata**

In `subjects/catalog.yaml`, add these overrides under `subjects.economics.skill_overrides`:

```yaml
      - skill: study-designer
        overlay: overlays/skills/study-designer.md
        mode: append
      - skill: robustness-planner
        overlay: overlays/skills/robustness-planner.md
        mode: append
      - skill: analysis-interpreter
        overlay: overlays/skills/analysis-interpreter.md
        mode: append
```

Add `econ-replication-package-auditor` under `subjects.economics.subject_specific_skill_refs`.

Add `econometrica` and `jpe` under `subjects.economics.venue_profiles`.

Add `econ-replication-package-auditor` to the "Manuscript and Reproducibility" skill group.

- [ ] **Step 4: Add economics overlays**

Create `subjects/economics/overlays/skills/study-designer.md`:

```markdown
## Economics Overlay

- State the estimand before the model and distinguish it from the estimating equation.
- Identify the source of variation, the comparison group, and the identifying assumption.
- For panel or policy designs, describe timing, treatment definition, anticipation, and spillover risks.
- Record the main identification threat and the design feature that addresses it.
```

Create `subjects/economics/overlays/skills/robustness-planner.md`:

```markdown
## Economics Overlay

- Link every robustness check to an identification threat instead of listing generic alternatives.
- Include placebo outcomes, placebo timing, alternative control groups, and sensitivity to sample construction when relevant.
- For difference-in-differences, include pre-trend diagnostics and heterogeneous treatment timing checks.
- For IV designs, include first-stage strength, exclusion restriction discussion, and over-identification diagnostics when applicable.
```

Create `subjects/economics/overlays/skills/analysis-interpreter.md`:

```markdown
## Economics Overlay

- Interpret economic magnitude in natural units, baseline shares, elasticities, or welfare-relevant quantities.
- Separate statistical precision from economic importance.
- State whether the estimate identifies an average treatment effect, local average treatment effect, intent-to-treat effect, or descriptive association.
- Avoid policy claims that exceed the identifying variation.
```

- [ ] **Step 5: Add economics replication skill and registry entry**

Create `subjects/economics/skills/econ-replication-package-auditor.md`:

```markdown
# Economics Replication Package Auditor

Use this skill when an economics manuscript needs a reproducibility, data, code, and disclosure audit before submission.

## Inputs

- Manuscript draft
- Analysis scripts
- Data dictionary
- Results tables and figures
- Data availability statement

## Audit Steps

1. Match every reported estimate to a script, table source, and sample definition.
2. Confirm that treatment, outcome, controls, fixed effects, clusters, and weights match the manuscript.
3. Check that restricted data, licensed data, and generated data are separated in the package.
4. Verify that random seeds, environment files, and execution order are documented.
5. Flag undisclosed researcher degrees of freedom and robustness checks that cannot be reproduced.

## Output

Return a replication audit with:

- Reproducibility status
- Missing files
- Script-to-table map
- Data access constraints
- High-risk discrepancies
- Required fixes before submission
```

Add this entry to `subjects/economics/skills/registry.yaml`:

```yaml
  - id: econ-replication-package-auditor
    stage: I_code
    version: "0.1.0"
    file: skills/I_code/econ-replication-package-auditor.md
    canonical: false
    summary: Economics replication package and reproducibility audit.
    display_name: Economics Replication Package Auditor
    when_to_use: Use before submitting an economics paper with empirical results and code.
    inputs: [ManuscriptDraft, AnalysisScripts, DataDictionary]
    outputs: [ReplicationAudit]
```

- [ ] **Step 6: Add economics venue profiles**

Create `subjects/economics/venue-profiles/econometrica.yaml`:

```yaml
id: econometrica
display_name: Econometrica
discipline: economics
quality_bar:
  - rigorous identification or formal econometric/theoretical contribution
  - precise estimand and assumptions
  - transparent robustness and reproducibility materials
fit_notes:
  - strongest fit for econometric methods, economic theory, and highly rigorous empirical designs
```

Create `subjects/economics/venue-profiles/jpe.yaml`:

```yaml
id: jpe
display_name: Journal of Political Economy
discipline: economics
quality_bar:
  - economically important question
  - credible identification or theory
  - clear contribution to economic mechanisms
fit_notes:
  - strongest fit when the paper changes interpretation of an important economic phenomenon
```

- [ ] **Step 7: Run focused economics tests and audit**

Run:

```bash
.venv/bin/python -m unittest tests/test_subject_catalog.py tests/test_subject_materializer.py tests/test_subject_specialization_audit.py -v
.venv/bin/python scripts/audit_subject_specialization.py --root . --subject economics
```

Expected: both commands pass.

- [ ] **Step 8: Commit**

```bash
git add subjects/catalog.yaml subjects/economics tests/test_subject_materializer.py evals/subject-specialization/cases/economics-did-identification.yaml
git commit -m "feat(subjects): deepen economics method coverage"
```

### Task 4: Standalone Accounting Subject

**Files:**
- Modify: `subjects/catalog.yaml`
- Create: `subjects/accounting/overlays/skills/manuscript-architect.md`
- Create: `subjects/accounting/overlays/skills/stats-engine.md`
- Create: `subjects/accounting/overlays/skills/variable-constructor.md`
- Create: `subjects/accounting/skills/accounting-measurement-auditor.md`
- Create: `subjects/accounting/skills/registry.yaml`
- Create: `subjects/accounting/venue-profiles/accounting-review.yaml`
- Create: `subjects/accounting/venue-profiles/journal-of-accounting-research.yaml`
- Create: `subjects/accounting/venue-profiles/review-of-accounting-studies.yaml`
- Create: `evals/subject-specialization/cases/accounting-accruals-measurement.yaml`
- Modify: `tests/test_subject_catalog.py`
- Modify: `tests/test_subject_materializer.py`
- Modify: `tests/test_subject_eval_cases.py`
- Modify: `packages/npm-qiongli/test/installer.test.mjs`

- [ ] **Step 1: Add failing catalog and materializer tests**

Add to `tests/test_subject_catalog.py`:

```python
    def test_accounting_subject_is_ordered_and_explicit(self) -> None:
        catalog = validate_subject_catalog(REPO_ROOT)
        subject = catalog.subjects["accounting"]
        self.assertEqual(subject.extends, "core")
        self.assertEqual([group.order for group in subject.skill_groups], [1, 2, 3, 4, 5])
        self.assertIn("accounting-measurement-auditor", subject.skill_refs)
        self.assertEqual(subject.domain_profiles, ("accounting",))
```

Add to `tests/test_subject_materializer.py`:

```python
    def test_materializes_accounting_focused_package(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            out = Path(tmp_dir) / "qiongli-workflow"
            materialize_subject_package(
                MaterializeOptions(source=REPO_ROOT, out=out, subject="accounting", flavor="full", coverage="focused")
            )
            registry = yaml.safe_load((out / "skills" / "registry.yaml").read_text(encoding="utf-8"))
            registry_ids = {entry["id"] for entry in registry["skills"]}
            self.assertIn("accounting-measurement-auditor", registry_ids)
            self.assertNotIn("econ-identification-auditor", registry_ids)
            self.assertTrue((out / "skills" / "domain-profiles" / "accounting.yaml").exists())
            self.assertFalse((out / "skills" / "domain-profiles" / "economics.yaml").exists())
            manuscript = (out / "skills" / "F_writing" / "manuscript-architect.md").read_text(encoding="utf-8")
            self.assertIn("## Accounting Overlay", manuscript)
```

- [ ] **Step 2: Run tests to verify failure**

Run:

```bash
.venv/bin/python -m unittest tests/test_subject_catalog.py tests/test_subject_materializer.py -v
```

Expected: fail because `accounting` is not in `subjects/catalog.yaml`.

- [ ] **Step 3: Add accounting catalog definition**

Add this `subjects.accounting` entry to `subjects/catalog.yaml`:

```yaml
  accounting:
    display_name: Qiongli Accounting
    extends: core
    package_goal: "Accounting-focused archival, disclosure, audit, and measurement workflow."
    domain_profiles:
      - accounting
    venue_profiles:
      - accounting-review
      - journal-of-accounting-research
      - review-of-accounting-studies
    template_refs:
      - analysis-plan.md
      - claim-evidence-ledger.csv
      - data-availability.md
      - data-management-plan.md
      - figures-tables-plan.md
      - manuscript-outline.md
      - method-diagnostic-report.md
      - quality-gate-report.md
      - research-state.md
      - search-log.md
      - stage-handoff.md
      - study-design.md
      - validity-threat-matrix.md
      - writing-claim-map.md
    skill_overrides:
      - skill: manuscript-architect
        overlay: overlays/skills/manuscript-architect.md
        mode: append
      - skill: stats-engine
        overlay: overlays/skills/stats-engine.md
        mode: replace_sections
        sections:
          - Quality Bar
          - Common Pitfalls
      - skill: variable-constructor
        overlay: overlays/skills/variable-constructor.md
        mode: append
    subject_specific_skill_refs:
      - accounting-measurement-auditor
    skill_groups:
      - order: 1
        heading: "Accounting Research Framing"
        subheading: "Construct, setting, contribution, theory, and venue fit"
        stages: ["A"]
        skill_refs:
          - question-refiner
          - contribution-crafter
          - gap-analyzer
          - theory-mapper
          - venue-analyzer
      - order: 2
        heading: "Accounting Literature Positioning"
        subheading: "Disclosure, audit, financial reporting, and archival literature grounding"
        stages: ["B"]
        skill_refs:
          - academic-searcher
          - citation-snowballer
          - literature-mapper
          - paper-extractor
          - paper-screener
          - reference-manager-bridge
      - order: 3
        heading: "Measurement and Research Design"
        subheading: "Accounting constructs, samples, variables, identification, and robustness"
        stages: ["C", "I"]
        skill_refs:
          - study-designer
          - variable-constructor
          - data-dictionary-builder
          - robustness-planner
          - stats-engine
          - accounting-measurement-auditor
      - order: 4
        heading: "Results and Interpretation"
        subheading: "Measurement diagnostics, economic interpretation, tables, and figures"
        stages: ["E", "F"]
        skill_refs:
          - analysis-interpreter
          - effect-size-interpreter
          - table-generator
          - figure-specifier
      - order: 5
        heading: "Manuscript and Submission"
        subheading: "Architecture, disclosure transparency, reproducibility, and review readiness"
        stages: ["F", "G", "H", "I", "J"]
        skill_refs:
          - manuscript-architect
          - discussion-writer
          - reporting-checker
          - submission-packager
          - fatal-flaw-detector
          - code-review
          - reproducibility-auditor
          - final-proofreader
```

- [ ] **Step 4: Add accounting overlays**

Create `subjects/accounting/overlays/skills/manuscript-architect.md`:

```markdown
## Accounting Overlay

- Explain the accounting construct, institutional setting, and reporting or contracting mechanism before presenting tests.
- Tie hypotheses to disclosure, audit, financial reporting, governance, or capital-market channels.
- Separate measurement-validity claims from causal-identification claims.
- Report sample filters and variable construction choices in a way that supports replication.
```

Create `subjects/accounting/overlays/skills/stats-engine.md`:

```markdown
## Quality Bar

- Model choices must match the accounting construct, sample structure, and identifying variation.
- Standard errors must reflect firm, time, industry, auditor, or market clustering where the design requires it.
- Accrual, disclosure, and market-reaction measures must include distribution checks and influential-observation diagnostics.
- The analysis must document fixed effects, winsorization, scaling, and sample screens.

## Common Pitfalls

- Treating a noisy accounting proxy as if it were the latent construct.
- Changing winsorization, scaling, or sample filters without tracking effect on inference.
- Reporting market reactions without clarifying event windows and confounding disclosures.
- Using firm-year panels without explaining serial correlation and clustering choices.
```

Create `subjects/accounting/overlays/skills/variable-constructor.md`:

```markdown
## Accounting Overlay

- Define each accounting construct separately from its empirical proxy.
- Record Compustat, CRSP, Audit Analytics, IBES, or filing-source variable names when used.
- Document scaling, winsorization, lag structure, industry adjustment, and missing-value treatment.
- Flag constructs where measurement error could reverse interpretation.
```

- [ ] **Step 5: Add accounting skill and registry**

Create `subjects/accounting/skills/accounting-measurement-auditor.md`:

```markdown
# Accounting Measurement Auditor

Use this skill when an accounting paper depends on constructed variables, archival proxies, disclosure measures, audit measures, or financial-reporting constructs.

## Inputs

- Research question
- Construct definitions
- Variable construction notes
- Data source list
- Tables or planned tests

## Audit Steps

1. Distinguish the theoretical construct from every empirical proxy.
2. Check whether the data source, sample screen, scaling choice, and winsorization rule are recorded.
3. Identify sources of measurement error, construct drift, and mechanical correlation.
4. Verify that lag timing and fiscal-year alignment match the hypothesis.
5. Flag variables that need alternative proxies or disclosure about limitations.

## Output

Return an accounting measurement audit with:

- Construct-to-proxy map
- Data-source map
- Timing and scaling checks
- Measurement-error risks
- Required robustness checks
- Manuscript disclosure language
```

Create `subjects/accounting/skills/registry.yaml`:

```yaml
skills:
  - id: accounting-measurement-auditor
    stage: C_design
    version: "0.1.0"
    file: skills/C_design/accounting-measurement-auditor.md
    canonical: false
    summary: Accounting construct and proxy measurement audit.
    display_name: Accounting Measurement Auditor
    when_to_use: Use when archival accounting variables, disclosure proxies, or audit measures drive the research design.
    inputs: [ConstructDefinitions, VariablePlan, DataSources]
    outputs: [AccountingMeasurementAudit]
```

- [ ] **Step 6: Add accounting venue profiles**

Create accounting venue profiles under `subjects/accounting/venue-profiles/` using the existing filenames:

```yaml
id: accounting-review
display_name: The Accounting Review
discipline: accounting
quality_bar:
  - clear accounting contribution
  - credible construct measurement
  - strong research design and transparent sample construction
fit_notes:
  - strongest fit for financial reporting, audit, disclosure, governance, and capital-market accounting research
```

Use the same schema for `journal-of-accounting-research.yaml` and `review-of-accounting-studies.yaml`, with display names `Journal of Accounting Research` and `Review of Accounting Studies`.

- [ ] **Step 7: Remove duplicate composite-owned accounting skill files**

If `subjects/economics-accounting/skills/accounting-measurement-auditor.md` and its registry duplicate the new accounting skill, remove the duplicate source and make `economics-accounting` reference the accounting skill through the global subject registry loaded by the materializer:

```bash
git rm subjects/economics-accounting/skills/accounting-measurement-auditor.md subjects/economics-accounting/skills/registry.yaml
```

Keep composite-specific overlays and venue profiles only if they contain bridge guidance that differs from standalone accounting.

- [ ] **Step 8: Add accounting eval fixture**

Create `evals/subject-specialization/cases/accounting-accruals-measurement.yaml`:

```yaml
id: accounting-accruals-measurement
subject: accounting
coverage: focused
prompt: "Plan an archival accounting paper about discretionary accruals and earnings management."
expected_skill_refs:
  - accounting-measurement-auditor
expected_terms:
  - accrual
  - measurement
  - disclosure
expected_domain_profiles:
  - accounting.yaml
forbidden_domain_profiles:
  - economics.yaml
  - cs-ai.yaml
  - biomedical.yaml
```

Update `tests/test_subject_eval_cases.py`:

```python
        self.assertIn("accounting-accruals-measurement", case_ids)
```

- [ ] **Step 9: Run Python and npm tests**

Run:

```bash
.venv/bin/python -m unittest tests/test_subject_catalog.py tests/test_subject_materializer.py tests/test_subject_eval_cases.py -v
npm --prefix packages/npm-qiongli test
```

Expected: pass.

- [ ] **Step 10: Commit**

```bash
git add subjects/catalog.yaml subjects/accounting subjects/economics-accounting evals/subject-specialization/cases tests/test_subject_catalog.py tests/test_subject_materializer.py tests/test_subject_eval_cases.py packages/npm-qiongli
git commit -m "feat(subjects): add accounting subject"
```

### Task 5: Composite Subject Layers

**Files:**
- Modify: `qiongli/subject_materializer.py`
- Modify: `subjects/catalog.yaml`
- Modify: `tests/test_subject_catalog.py`
- Modify: `tests/test_subject_materializer.py`
- Modify: `docs/advanced/subject-packaging-model.md`
- Modify: `docs/zh/advanced/subject-packaging-model.md`

- [ ] **Step 1: Add failing tests for composite metadata**

Add to `tests/test_subject_catalog.py`:

```python
    def test_composite_subject_declares_component_subjects(self) -> None:
        catalog = validate_subject_catalog(REPO_ROOT)
        subject = catalog.subjects["economics-accounting"]
        self.assertEqual(subject.composes, ("economics", "accounting"))
```

Add to `tests/test_subject_materializer.py`:

```python
    def test_composite_manifest_lists_component_layers(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            out = Path(tmp_dir) / "qiongli-workflow"
            materialize_subject_package(
                MaterializeOptions(
                    source=REPO_ROOT,
                    out=out,
                    subject="economics-accounting",
                    flavor="full",
                    coverage="complete",
                )
            )
            manifest = json.loads((out / "SUBJECT_MANIFEST.json").read_text(encoding="utf-8"))
            self.assertEqual(
                manifest["layers"],
                ["core", "economics", "accounting", "economics-accounting"],
            )
```

- [ ] **Step 2: Run tests to verify failure**

Run:

```bash
.venv/bin/python -m unittest tests/test_subject_catalog.py tests/test_subject_materializer.py -v
```

Expected: fail because `SubjectDefinition` has no `composes` field.

- [ ] **Step 3: Add `composes` to subject definitions**

In `qiongli/subject_materializer.py`, update `SubjectDefinition`:

```python
@dataclass(frozen=True)
class SubjectDefinition:
    id: str
    display_name: str
    package_goal: str
    extends: str | None
    skill_groups: tuple[SubjectGroup, ...]
    domain_profiles: tuple[str, ...] = ()
    venue_profiles: tuple[str, ...] = ()
    template_refs: tuple[str, ...] = ()
    skill_overrides: tuple[dict[str, Any], ...] = ()
    subject_specific_skill_refs: tuple[str, ...] = ()
    composes: tuple[str, ...] = ()
    skill_refs: tuple[str, ...] = field(init=False)
```

When constructing `SubjectDefinition`, pass:

```python
composes=tuple(_string_list(raw_subject.get("composes"), "composes", subject_id)),
```

After validating `extends`, validate `composes`:

```python
    for component in subject.composes:
        if component not in subjects:
            raise SubjectCatalogError(f"subject {subject_id} composes unknown subject: {component}")
        if component == subject_id:
            raise SubjectCatalogError(f"subject {subject_id} cannot compose itself")
```

- [ ] **Step 4: Update manifest layer rendering**

Update `_subject_layers`:

```python
def _subject_layers(subject: SubjectDefinition, custom_layer: CustomSubjectLayer) -> list[str]:
    layers: list[str] = []
    if subject.extends:
        layers.append(subject.extends)
    for component in subject.composes:
        if component not in layers:
            layers.append(component)
    if subject.id not in layers:
        layers.append(subject.id)
    if custom_layer.root is not None:
        layers.append("custom")
    return layers
```

- [ ] **Step 5: Add composite metadata to catalog**

In `subjects/catalog.yaml`, add:

```yaml
    composes:
      - economics
      - accounting
```

under `subjects.economics-accounting`.

- [ ] **Step 6: Document composite metadata semantics**

In both subject packaging model docs, add this rule:

```markdown
`composes` is metadata, not automatic union. It records that an official composite subject was
designed from component subject expectations, but the composite still declares its own ordered
groups, profile selection, overlays, and subject-specific skills.
```

- [ ] **Step 7: Run tests**

Run:

```bash
.venv/bin/python -m unittest tests/test_subject_catalog.py tests/test_subject_materializer.py -v
```

Expected: pass.

- [ ] **Step 8: Commit**

```bash
git add qiongli/subject_materializer.py subjects/catalog.yaml tests/test_subject_catalog.py tests/test_subject_materializer.py docs/advanced/subject-packaging-model.md docs/zh/advanced/subject-packaging-model.md
git commit -m "feat(subjects): declare composite subject layers"
```

### Task 6: Local Customization Scaffold

**Files:**
- Modify: `qiongli/cli.py`
- Create: `qiongli/custom_subject.py`
- Create: `tests/test_custom_subject_cli.py`
- Modify: `README.md`
- Modify: `README_CN.md`

- [ ] **Step 1: Locate CLI entrypoint**

Run:

```bash
rg "argparse|def main|subparsers|install" qiongli tests -n
```

Expected: identify the file that defines `qiongli install`, `qiongli upgrade`, and `qiongli check`.

- [ ] **Step 2: Write failing CLI scaffold tests**

Create `tests/test_custom_subject_cli.py`:

```python
from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from qiongli.custom_subject import scaffold_custom_subject


class CustomSubjectScaffoldTests(unittest.TestCase):
    def test_scaffold_custom_subject_dir(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            out = Path(tmp_dir) / "econ-lab"
            scaffold_custom_subject(out, base_subject="economics", name="econ-lab")

            self.assertTrue((out / "subject.yaml").exists())
            self.assertTrue((out / "overlays" / "skills" / "README.md").exists())
            self.assertTrue((out / "skills" / "registry.yaml").exists())
            subject_yaml = (out / "subject.yaml").read_text(encoding="utf-8")
            self.assertIn("base_subject: economics", subject_yaml)
            self.assertIn("skill_overrides:", subject_yaml)

    def test_scaffold_refuses_non_empty_directory_without_force(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            out = Path(tmp_dir) / "custom"
            out.mkdir()
            (out / "existing.txt").write_text("keep", encoding="utf-8")

            with self.assertRaisesRegex(FileExistsError, "not empty"):
                scaffold_custom_subject(out, base_subject="economics", name="custom")
```

- [ ] **Step 3: Run tests to verify failure**

Run:

```bash
.venv/bin/python -m unittest tests/test_custom_subject_cli.py -v
```

Expected: fail with `ModuleNotFoundError: No module named 'qiongli.custom_subject'`.

- [ ] **Step 4: Implement scaffold helper**

Create `qiongli/custom_subject.py`:

```python
from __future__ import annotations

from pathlib import Path


def scaffold_custom_subject(out: Path, *, base_subject: str, name: str, force: bool = False) -> None:
    out = Path(out)
    if out.exists() and any(out.iterdir()) and not force:
        raise FileExistsError(f"custom subject directory is not empty: {out}")
    (out / "overlays" / "skills").mkdir(parents=True, exist_ok=True)
    (out / "skills").mkdir(parents=True, exist_ok=True)
    (out / "domain-profiles").mkdir(parents=True, exist_ok=True)
    (out / "venue-profiles").mkdir(parents=True, exist_ok=True)
    (out / "subject.yaml").write_text(_subject_yaml(base_subject, name), encoding="utf-8")
    (out / "skills" / "registry.yaml").write_text("skills: []\n", encoding="utf-8")
    (out / "overlays" / "skills" / "README.md").write_text(_overlay_readme(base_subject), encoding="utf-8")
    (out / "README.md").write_text(_readme(base_subject, name), encoding="utf-8")


def _subject_yaml(base_subject: str, name: str) -> str:
    return "\n".join(
        [
            f"name: {name}",
            f"base_subject: {base_subject}",
            "skill_refs: []",
            "domain_profiles: []",
            "venue_profiles: []",
            "skill_overrides:",
            "  - skill: manuscript-architect",
            "    overlay: overlays/skills/manuscript-architect.md",
            "    mode: append",
            "",
        ]
    )


def _overlay_readme(base_subject: str) -> str:
    return "\n".join(
        [
            "# Skill Overlays",
            "",
            f"This custom directory can be applied on top of `{base_subject}` with:",
            "",
            "```bash",
            "python3 scripts/materialize_subject_package.py --subject "
            + base_subject
            + " --custom-dir path/to/custom --source . --out /tmp/qiongli-workflow",
            "```",
            "",
        ]
    )


def _readme(base_subject: str, name: str) -> str:
    return "\n".join(
        [
            f"# {name}",
            "",
            f"Custom Qiongli subject layer for `{base_subject}`.",
            "",
            "- Add append overlays under `overlays/skills/`.",
            "- Add local skill markdown under `skills/` and registry entries in `skills/registry.yaml`.",
            "- Add local profiles under `domain-profiles/` and `venue-profiles/`.",
            "- Materialization applies this directory only to the generated output.",
            "",
        ]
    )
```

- [ ] **Step 5: Add CLI command**

In the CLI entrypoint, add:

```python
custom_parser = subparsers.add_parser("customize", help="Create a local custom subject overlay directory")
custom_parser.add_argument("--subject", default="core")
custom_parser.add_argument("--name", required=True)
custom_parser.add_argument("--out", required=True)
custom_parser.add_argument("--force", action="store_true")
```

Dispatch:

```python
if args.command == "customize":
    from qiongli.custom_subject import scaffold_custom_subject

    scaffold_custom_subject(Path(args.out), base_subject=args.subject, name=args.name, force=args.force)
    print(f"Created custom subject overlay at {args.out}")
    return 0
```

- [ ] **Step 6: Run helper and existing CLI tests**

Run:

```bash
.venv/bin/python -m unittest tests/test_custom_subject_cli.py tests/test_cli.py -v
```

Expected: pass.

- [ ] **Step 7: Document quick customization command**

Add to `README.md`:

````markdown
Create a local customization layer:

```bash
qiongli customize --subject economics --name my-econ-lab --out ./qiongli-custom/econ-lab
python3 scripts/materialize_subject_package.py --subject economics --custom-dir ./qiongli-custom/econ-lab --source . --out /tmp/qiongli-workflow
```
````

Add the equivalent Chinese text to `README_CN.md`.

- [ ] **Step 8: Commit**

```bash
git add qiongli/custom_subject.py qiongli tests/test_custom_subject_cli.py README.md README_CN.md
git commit -m "feat(subjects): scaffold local custom subject overlays"
```

### Task 7: Release and Distribution Validation

**Files:**
- Modify: `scripts/build_plugin_artifacts.py`
- Modify: `scripts/validate_marketplace_install.py`
- Modify: `scripts/audit_distribution_payloads.py`
- Modify: `tests/test_distribution_payloads.py`
- Modify: `scripts/sync_npm_package_payload.py`
- Modify: `packages/npm-qiongli/lib/installer.mjs`
- Modify: `packages/npm-qiongli/test/installer.test.mjs`

- [ ] **Step 1: Add failing distribution tests for accounting payloads**

In `tests/test_distribution_payloads.py`, add:

```python
    def test_distribution_includes_accounting_subject_payloads(self) -> None:
        payload_root = REPO_ROOT / "qiongli" / "payload" / "subjects"
        self.assertTrue((payload_root / "accounting" / "complete" / "qiongli-workflow" / "SUBJECT_MANIFEST.json").exists())
        self.assertTrue((payload_root / "accounting" / "focused" / "qiongli-workflow" / "SUBJECT_MANIFEST.json").exists())
```

- [ ] **Step 2: Run distribution tests to verify failure**

Run:

```bash
.venv/bin/python -m unittest tests/test_distribution_payloads.py -v
```

Expected: fail until the payload sync generates accounting outputs.

- [ ] **Step 3: Update Python payload sync**

Find the payload sync script:

```bash
rg "materialize_subject_package|payload/subjects|subjects/" scripts qiongli packages/npm-qiongli -n
```

Add `accounting` to the list of generated subject payloads for both `complete` and `focused`.

- [ ] **Step 4: Update Desktop artifact validation**

Keep Desktop ZIPs focused-only. In this phase, validate `accounting` as an installable CLI/npm payload but do not emit an `accounting` Desktop ZIP; public Desktop ZIPs remain `core`, `economics`, and `economics-accounting`.

Use this rule in `scripts/build_plugin_artifacts.py`:

```python
DESKTOP_SUBJECTS = ("core", "economics", "economics-accounting")
PAYLOAD_SUBJECTS = ("core", "economics", "accounting", "economics-accounting")
```

- [ ] **Step 5: Update npm payload sync and tests**

Generate these npm payload paths:

```text
packages/npm-qiongli/payload/subjects/accounting/complete/qiongli-workflow
packages/npm-qiongli/payload/subjects/accounting/focused/qiongli-workflow
```

Add npm tests that call the existing installer helper with:

```text
install --subject accounting --coverage complete
install --subject accounting --coverage focused
```

Expected installed `SUBJECT_MANIFEST.json` subject is `accounting` and coverage matches the requested coverage.

- [ ] **Step 6: Run release validation commands**

Run:

```bash
.venv/bin/python scripts/audit_subject_specialization.py --root .
.venv/bin/python scripts/audit_subject_eval_cases.py --root .
.venv/bin/python scripts/validate_marketplace_install.py
.venv/bin/python scripts/audit_distribution_payloads.py
npm --prefix packages/npm-qiongli test
```

Expected: pass.

- [ ] **Step 7: Commit**

```bash
git add scripts tests qiongli/payload packages/npm-qiongli
git commit -m "build(release): validate subject specialization quality"
```

### Task 8: Documentation and Final Regression

**Files:**
- Modify: `README.md`
- Modify: `README_CN.md`
- Modify: `README_PYPI.md`
- Modify: `packages/npm-qiongli/README.md`
- Modify: `docs/quickstart.md`
- Modify: `docs/zh/quickstart.md`
- Modify: `docs/guide/install.md`
- Modify: `docs/zh/guide/install.md`
- Modify: `docs/reference/cli.md`
- Modify: `docs/zh/reference/cli.md`
- Modify: `CHANGELOG.md`

- [ ] **Step 1: Update user-facing subject selection docs**

Document these exact user paths:

```bash
qiongli install --target all
qiongli install --subject economics --target all
qiongli install --subject accounting --target all
qiongli install --subject economics-accounting --target all
qiongli install --subject economics --coverage focused --target all
qiongli upgrade --subject accounting --target all
qiongli customize --subject economics --name my-econ-lab --out ./qiongli-custom/econ-lab
```

State:

- Default install is `core/complete`.
- `--subject economics` means `economics/complete`, not a reduced package.
- `--coverage focused` is the deliberate slim path.
- Official composite subjects are named subjects, not arbitrary comma-separated stacking.
- Custom overlays affect generated output only and do not rewrite canonical source files.

- [ ] **Step 2: Update developer docs**

Add a developer section that explains:

```markdown
When adding or deepening a subject, update all of these together:

1. `subjects/catalog.yaml`
2. subject overlays
3. subject-specific registry and markdown
4. selected domain and venue profiles
5. subject eval fixtures
6. specialization audit expected terms
7. materializer tests
8. npm payload tests when the subject is installable through npm
9. release validation if the subject has a Desktop/Web artifact
```

- [ ] **Step 3: Run docs and regression tests**

Run:

```bash
npm run docs:build
.venv/bin/python -m unittest discover -s tests -v
npm --prefix packages/npm-qiongli test
```

Expected: pass.

- [ ] **Step 4: Run release/package validation**

Run:

```bash
.venv/bin/python scripts/materialize_subject_package.py --subject economics --coverage complete --source . --out /private/tmp/qiongli-econ-complete --flavor full
.venv/bin/python scripts/materialize_subject_package.py --subject economics --coverage focused --source . --out /private/tmp/qiongli-econ-focused --flavor full
.venv/bin/python scripts/materialize_subject_package.py --subject accounting --coverage complete --source . --out /private/tmp/qiongli-accounting-complete --flavor full
.venv/bin/python scripts/materialize_subject_package.py --subject economics-accounting --coverage complete --source . --out /private/tmp/qiongli-econ-accounting-complete --flavor full
.venv/bin/python scripts/validate_marketplace_install.py
.venv/bin/python scripts/audit_distribution_payloads.py
```

Expected: pass.

- [ ] **Step 5: Commit**

```bash
git add README.md README_CN.md README_PYPI.md packages/npm-qiongli/README.md docs CHANGELOG.md
git commit -m "docs(subjects): document subject depth roadmap"
```

## Final Merge Procedure

- [ ] **Step 1: Confirm feature branch is clean**

Run:

```bash
git status --short --branch
```

Expected: branch is `codex/qiongli-subject-specialization-roadmap` and has no uncommitted changes.

- [ ] **Step 2: Fast-forward `dev` from a temporary sync worktree**

From the main checkout:

```bash
git worktree add /Users/pengjiaxin/Work/utility/cli-tools/research-skills/.worktrees/dev-subject-specialization-sync dev
git -C /Users/pengjiaxin/Work/utility/cli-tools/research-skills/.worktrees/dev-subject-specialization-sync merge --ff-only codex/qiongli-subject-specialization-roadmap
git worktree remove /Users/pengjiaxin/Work/utility/cli-tools/research-skills/.worktrees/dev-subject-specialization-sync
```

Expected: `dev` fast-forwards without touching the dirty main checkout.

## Execution Options

Plan complete and saved to `docs/superpowers/plans/2026-05-27-subject-specialization-roadmap.md`.

1. **Subagent-Driven (recommended)** - Dispatch a fresh subagent per task, review between tasks, and keep each commit focused.
2. **Inline Execution** - Execute tasks in this session using executing-plans, with checkpoints after each commit.
