# Qiongli Skill Set Optimization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Improve the current 71-skill Qiongli corpus by making quality gates executable, domain method packs auditable, and skill-set quality measurable across offline evals.

**Architecture:** Keep `skills/` as the canonical skill source, `skills/registry.yaml` as the canonical metadata/version source, and `qiongli-workflow/` plus `plugins/qiongli/skills/qiongli-workflow/` as synchronized distribution copies. Add contracts and validators before changing skill behavior so every optimization is measurable and can be enforced by CI.

**Tech Stack:** Python stdlib, `unittest`, YAML/Markdown contracts, existing validator scripts, existing `scripts/sync_skill_package.sh`, existing offline academic quality eval runner.

---

## Current Baseline

- Canonical registered skills: 71.
- Stage distribution:
  - `A_framing`: 6
  - `B_literature`: 9
  - `C_design`: 9
  - `D_ethics`: 3
  - `E_synthesis`: 5
  - `F_writing`: 7
  - `G_compliance`: 3
  - `H_submission`: 7
  - `I_code`: 10
  - `J_proofread`: 4
  - `K_presentation`: 4
  - `Z_cross_cutting`: 4
- Existing quality gates `Q1`-`Q4` are present in `standards/research-workflow-contract.yaml` and task routing, but they are still mostly labels plus prompt instructions.
- Domain profiles already exist for economics, finance, psychology, biomedical, education, CS/AI, political science, epidemiology, ecology/environmental, and business/management.
- Offline academic quality evals currently score 6 cases across 7 dimensions.

## Optimization Priorities

1. Convert `Q1`-`Q4` into executable semantic gate contracts.
2. Add a repeatable skill-set scorecard so future skill changes can be compared against a baseline.
3. Deepen economics and finance method packs first, because they have high demand and clear validator rules.
4. Expand offline evals to cover gate failures, domain method mistakes, and skill routing choices.
5. Keep runtime UX changes minimal until gate and method-pack behavior is measurable.

## File Structure

### New Files

- `docs/maintainer/skill-set-optimization-scorecard.md`
  - Human-readable baseline and target scorecard for this optimization round.
- `standards/quality-gate-contract.yaml`
  - Machine-readable definitions for Q1-Q4 checks, required evidence, severity levels, and artifact expectations.
- `scripts/audit_quality_gates.py`
  - Offline audit for gate report files and bundled gate contract.
- `scripts/audit_domain_method_packs.py`
  - Offline audit for domain profile method-pack completeness.
- `tests/test_quality_gate_contract.py`
  - Tests for gate contract shape and audit behavior.
- `tests/test_domain_method_packs.py`
  - Tests for economics and finance method-pack fields and invalid fixtures.
- `tests/test_skill_set_scorecard.py`
  - Tests for generated scorecard metrics.
- `evals/academic_quality/cases/q1-rq-method-mismatch.yaml`
- `evals/academic_quality/cases/q2-unsupported-claim.yaml`
- `evals/academic_quality/cases/q3-reporting-gap.yaml`
- `evals/academic_quality/cases/q4-reproducibility-gap.yaml`
- `evals/academic_quality/cases/economics-did-invalid-parallel-trends.yaml`
- `evals/academic_quality/cases/finance-event-study-leakage.yaml`

### Modified Files

- `scripts/validate_research_standard.py`
  - Add strict-mode checks for the gate contract and domain method-pack audit.
- `scripts/run_academic_quality_evals.py`
  - Add optional expected gate and domain-method dimensions.
- `tests/test_academic_quality_evals.py`
  - Expect the expanded fixture set and dimensions.
- `standards/research-workflow-contract.yaml`
  - Link each Q gate to the detailed gate contract without duplicating all rules.
- `qiongli-workflow/references/workflow-contract.md`
  - Regenerate after contract changes.
- `templates/quality-gate-report.md`
  - Add a stable report block consumed by `audit_quality_gates.py`.
- `qiongli-workflow/templates/quality-gate-report.md`
  - Updated through package sync.
- `skills/domain-profiles/economics.yaml`
  - Add executable method-pack fields for DID, RD, IV/2SLS, synthetic control, and panel/event-study variants.
- `skills/domain-profiles/finance.yaml`
  - Add executable method-pack fields for event study, factor models, portfolio optimization, GARCH, and option-pricing variants.
- `schemas/domain-profile.schema.json`
  - Extend `method_templates` shape with validator-facing fields.
- `skills/C_design/study-designer.md`
- `skills/C_design/robustness-planner.md`
- `skills/I_code/stats-engine.md`
- `skills/I_code/code-builder.md`
- `skills/I_code/code-review.md`
  - Teach these skills to consume method-pack fields without embedding domain-specific checklist copies.

---

## Task 0: Baseline And Scorecard

**Files:**
- Create: `docs/maintainer/skill-set-optimization-scorecard.md`
- Create: `tests/test_skill_set_scorecard.py`
- Modify: `scripts/validate_research_standard.py`

- [ ] **Step 1: Write the scorecard test**

Add `tests/test_skill_set_scorecard.py`:

```python
from __future__ import annotations

import unittest
from pathlib import Path

import yaml


REPO_ROOT = Path(__file__).resolve().parents[1]


class SkillSetScorecardTests(unittest.TestCase):
    def test_scorecard_records_registered_skill_count_and_priorities(self) -> None:
        scorecard = REPO_ROOT / "docs" / "maintainer" / "skill-set-optimization-scorecard.md"
        content = scorecard.read_text(encoding="utf-8")
        registry = yaml.safe_load((REPO_ROOT / "skills" / "registry.yaml").read_text(encoding="utf-8"))
        skill_count = len(registry["skills"])

        self.assertIn(f"Canonical registered skills: {skill_count}", content)
        self.assertIn("Executable Q1-Q4 semantic gates", content)
        self.assertIn("Economics and finance method packs", content)
        self.assertIn("Offline eval expansion", content)


if __name__ == "__main__":
    unittest.main()
```

- [ ] **Step 2: Run the failing test**

Run: `python3 -m unittest tests.test_skill_set_scorecard -v`

Expected: FAIL because `docs/maintainer/skill-set-optimization-scorecard.md` does not exist.

- [ ] **Step 3: Add the scorecard**

Create `docs/maintainer/skill-set-optimization-scorecard.md`:

````markdown
# Skill Set Optimization Scorecard

## Baseline

- Canonical registered skills: 71
- Current gate model: Q1-Q4 labels exist in workflow contracts and task routing.
- Current domain model: domain profiles are injected at runtime.
- Current eval model: offline academic quality fixtures score broad quality dimensions.

## Next Optimization Targets

| Target | Baseline | Next Bar |
|--------|----------|----------|
| Executable Q1-Q4 semantic gates | Gate labels and prompt instructions | Contract-backed gate reports audited offline |
| Economics and finance method packs | Method names and general diagnostics | Method templates with assumptions, required diagnostics, artifacts, and failure modes |
| Offline eval expansion | 6 broad cases | Gate-failure and domain-method cases included |
| Skill routing precision | Registry inputs/outputs plus task routing | Gate and domain-pack requirements visible in task prompts |
| Release confidence | Structural validation | Structural + semantic contract validation |

## Measurement Commands

```bash
python3 scripts/validate_research_standard.py --strict
python3 scripts/audit_skill_sections.py --strict
python3 scripts/audit_quality_gates.py --strict
python3 scripts/audit_domain_method_packs.py --strict
python3 scripts/run_academic_quality_evals.py evals/academic_quality/cases
```
````

- [ ] **Step 4: Add strict validator awareness**

In `scripts/validate_research_standard.py`, add constants:

```python
QUALITY_GATE_CONTRACT_REQUIRED_FILES = (
    "standards/quality-gate-contract.yaml",
    "scripts/audit_quality_gates.py",
    "templates/quality-gate-report.md",
    "tests/test_quality_gate_contract.py",
)

DOMAIN_METHOD_PACK_REQUIRED_FILES = (
    "scripts/audit_domain_method_packs.py",
    "tests/test_domain_method_packs.py",
)
```

Add validation functions that mirror existing `validate_literature_first_contracts()` behavior:

```python
def validate_quality_gate_contracts(root: Path, report: ValidationReport, strict: bool) -> None:
    if not strict:
        return
    for relative_path in QUALITY_GATE_CONTRACT_REQUIRED_FILES:
        report.check(
            (root / relative_path).exists(),
            f"Quality gate contract file exists: {relative_path}",
            f"Missing quality gate contract file: {relative_path}",
        )


def validate_domain_method_pack_contracts(root: Path, report: ValidationReport, strict: bool) -> None:
    if not strict:
        return
    for relative_path in DOMAIN_METHOD_PACK_REQUIRED_FILES:
        report.check(
            (root / relative_path).exists(),
            f"Domain method pack file exists: {relative_path}",
            f"Missing domain method pack file: {relative_path}",
        )
```

Call both from `main()` with the same `strict=args.strict` pattern used by existing contract validators.

- [ ] **Step 5: Run tests**

Run:

```bash
python3 -m unittest tests.test_skill_set_scorecard tests.test_research_standard_validator -v
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add docs/maintainer/skill-set-optimization-scorecard.md tests/test_skill_set_scorecard.py scripts/validate_research_standard.py
git commit -m "Add skill set optimization scorecard"
```

---

## Task 1: Executable Q1-Q4 Gate Contract

**Files:**
- Create: `standards/quality-gate-contract.yaml`
- Create: `scripts/audit_quality_gates.py`
- Create: `tests/test_quality_gate_contract.py`
- Modify: `templates/quality-gate-report.md`
- Modify: `standards/research-workflow-contract.yaml`
- Modify: `qiongli-workflow/references/workflow-contract.md` after regeneration

- [ ] **Step 1: Write failing gate contract tests**

Add `tests/test_quality_gate_contract.py`:

```python
from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

import yaml

from scripts.audit_quality_gates import audit_gate_report, load_gate_contract


REPO_ROOT = Path(__file__).resolve().parents[1]


class QualityGateContractTests(unittest.TestCase):
    def test_contract_defines_q1_to_q4_with_required_fields(self) -> None:
        contract = load_gate_contract(REPO_ROOT / "standards" / "quality-gate-contract.yaml")
        self.assertEqual({"Q1", "Q2", "Q3", "Q4"}, set(contract["gates"]))
        for gate_id, gate in contract["gates"].items():
            self.assertIn("name", gate)
            self.assertIn("required_evidence", gate)
            self.assertIn("pass_criteria", gate)
            self.assertIn("fail_conditions", gate)
            self.assertIn("report_fields", gate)

    def test_gate_report_fails_for_missing_gate_status(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            report = Path(tmp_dir) / "quality-gate-report.md"
            report.write_text(
                "# Quality Gate Report\n\n```yaml\ngates:\n  Q1:\n    status: PASS\n```\n",
                encoding="utf-8",
            )
            result = audit_gate_report(
                report,
                load_gate_contract(REPO_ROOT / "standards" / "quality-gate-contract.yaml"),
            )

        self.assertFalse(result.passed)
        self.assertTrue(any("Q2" in issue for issue in result.errors))

    def test_gate_report_accepts_complete_report(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            report = Path(tmp_dir) / "quality-gate-report.md"
            report.write_text(
                "# Quality Gate Report\n\n"
                "```yaml\n"
                "gates:\n"
                "  Q1:\n"
                "    status: PASS\n"
                "    evidence: [framing/research_question.md, study_design.md]\n"
                "    blocking_issues: []\n"
                "  Q2:\n"
                "    status: PASS\n"
                "    evidence: [evidence/claim-evidence-ledger.csv]\n"
                "    blocking_issues: []\n"
                "  Q3:\n"
                "    status: WARN\n"
                "    evidence: [reporting_checklist.md]\n"
                "    blocking_issues: []\n"
                "  Q4:\n"
                "    status: PASS\n"
                "    evidence: [analysis_plan.md, code/reproducibility_audit.md]\n"
                "    blocking_issues: []\n"
                "```\n",
                encoding="utf-8",
            )
            result = audit_gate_report(
                report,
                load_gate_contract(REPO_ROOT / "standards" / "quality-gate-contract.yaml"),
            )

        self.assertTrue(result.passed)
        self.assertEqual([], result.errors)


if __name__ == "__main__":
    unittest.main()
```

- [ ] **Step 2: Run the failing test**

Run: `python3 -m unittest tests.test_quality_gate_contract -v`

Expected: FAIL because `scripts.audit_quality_gates` and `standards/quality-gate-contract.yaml` do not exist.

- [ ] **Step 3: Add the gate contract**

Create `standards/quality-gate-contract.yaml`:

```yaml
contract_version: "1.0.0"
status_values: [PASS, WARN, FAIL, BLOCKED]
severity_values: [info, minor, major, critical]
gates:
  Q1:
    name: "question-to-method alignment"
    required_evidence:
      - "framing/research_question.md"
      - "study_design.md"
      - "analysis_plan.md"
    pass_criteria:
      - "Every active RQ or hypothesis maps to a design, data source, measurement plan, and analysis strategy."
      - "Method choice is justified against at least one rejected alternative."
    fail_conditions:
      - "A central RQ has no method or outcome mapping."
      - "The method cannot answer the stated question."
    report_fields: [status, evidence, blocking_issues]
  Q2:
    name: "claim-evidence traceability"
    required_evidence:
      - "evidence/evidence-ledger.md"
      - "evidence/claim-evidence-ledger.csv"
      - "bibliography.bib"
    pass_criteria:
      - "Every central claim maps to at least one source, analysis result, artifact, or explicit gap note."
      - "Unsupported claims are downgraded or converted into gap notes."
    fail_conditions:
      - "Central manuscript claim has no evidence row."
      - "Citation metadata is invented or cannot be traced to the bibliography."
    report_fields: [status, evidence, blocking_issues]
  Q3:
    name: "reporting completeness"
    required_evidence:
      - "reporting_checklist.md"
      - "submission/submission_checklist.md"
    pass_criteria:
      - "Required reporting checklist exists for the paper type and target venue."
      - "Known missing checklist items are explicitly marked with rationale."
    fail_conditions:
      - "No reporting checklist exists for a submission-ready output."
      - "Required checklist items are omitted without explanation."
    report_fields: [status, evidence, blocking_issues]
  Q4:
    name: "reproducibility baseline"
    required_evidence:
      - "analysis_plan.md"
      - "data_management_plan.md"
      - "code/reproducibility_audit.md"
    pass_criteria:
      - "Analysis decisions, artifact paths, random seeds, and environment assumptions are documented."
      - "Known non-reproducible dependencies are named."
    fail_conditions:
      - "Analysis outputs cannot be traced to inputs and scripts."
      - "No reproducibility or environment baseline is documented for computational work."
    report_fields: [status, evidence, blocking_issues]
```

- [ ] **Step 4: Add the audit script**

Create `scripts/audit_quality_gates.py`:

```python
#!/usr/bin/env python3
from __future__ import annotations

import argparse
import re
from dataclasses import dataclass, field
from pathlib import Path

import yaml


@dataclass
class GateAuditResult:
    passed: bool
    errors: list[str] = field(default_factory=list)


def load_gate_contract(path: Path) -> dict[str, object]:
    return yaml.safe_load(path.read_text(encoding="utf-8")) or {}


def _extract_yaml_block(content: str) -> dict[str, object]:
    match = re.search(r"```yaml\s*\n(?P<body>.*?)\n```", content, flags=re.DOTALL)
    if not match:
        return {}
    return yaml.safe_load(match.group("body")) or {}


def audit_gate_report(path: Path, contract: dict[str, object]) -> GateAuditResult:
    errors: list[str] = []
    payload = _extract_yaml_block(path.read_text(encoding="utf-8"))
    report_gates = payload.get("gates", {})
    contract_gates = contract.get("gates", {})
    status_values = set(contract.get("status_values", []))
    for gate_id, gate_contract in contract_gates.items():
        gate_report = report_gates.get(gate_id)
        if not isinstance(gate_report, dict):
            errors.append(f"{path}: missing report for {gate_id}")
            continue
        for field_name in gate_contract.get("report_fields", []):
            if field_name not in gate_report:
                errors.append(f"{path}: {gate_id} missing field {field_name}")
        status = gate_report.get("status")
        if status not in status_values:
            errors.append(f"{path}: {gate_id} invalid status {status!r}")
        if status in {"FAIL", "BLOCKED"} and not gate_report.get("blocking_issues"):
            errors.append(f"{path}: {gate_id} {status} requires blocking_issues")
    return GateAuditResult(passed=not errors, errors=errors)


def main() -> int:
    parser = argparse.ArgumentParser(description="Audit Qiongli quality gate reports.")
    parser.add_argument("--contract", type=Path, default=Path("standards/quality-gate-contract.yaml"))
    parser.add_argument("--report", type=Path, default=Path("templates/quality-gate-report.md"))
    parser.add_argument("--strict", action="store_true")
    args = parser.parse_args()

    contract = load_gate_contract(args.contract)
    result = audit_gate_report(args.report, contract)
    for error in result.errors:
        print(f"[FAIL] {error}")
    if result.errors and args.strict:
        return 1
    print("[PASS] Quality gate report audit complete")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
```

- [ ] **Step 5: Update the quality gate report template**

Update `templates/quality-gate-report.md` to include:

````markdown
# Quality Gate Report

```yaml
gates:
  Q1:
    status: WARN
    evidence: []
    blocking_issues: []
  Q2:
    status: WARN
    evidence: []
    blocking_issues: []
  Q3:
    status: WARN
    evidence: []
    blocking_issues: []
  Q4:
    status: WARN
    evidence: []
    blocking_issues: []
```
````

- [ ] **Step 6: Link the gate contract from the workflow contract**

In `standards/research-workflow-contract.yaml`, add a `contract_ref` under each Q gate:

```yaml
quality_gates:
  - id: "Q1"
    name: "question-to-method alignment"
    rule: "RQs in framing match methods and outcomes in manuscript"
    contract_ref: "standards/quality-gate-contract.yaml#gates.Q1"
```

Repeat for Q2-Q4.

- [ ] **Step 7: Run tests**

Run:

```bash
python3 -m unittest tests.test_quality_gate_contract tests.test_research_standard_validator -v
python3 scripts/audit_quality_gates.py --strict
```

Expected: PASS.

- [ ] **Step 8: Commit**

```bash
git add standards/quality-gate-contract.yaml standards/research-workflow-contract.yaml templates/quality-gate-report.md scripts/audit_quality_gates.py tests/test_quality_gate_contract.py scripts/validate_research_standard.py
git commit -m "Add executable quality gate contract"
```

---

## Task 2: Economics And Finance Method Packs

**Files:**
- Create: `scripts/audit_domain_method_packs.py`
- Create: `tests/test_domain_method_packs.py`
- Modify: `schemas/domain-profile.schema.json`
- Modify: `skills/domain-profiles/economics.yaml`
- Modify: `skills/domain-profiles/finance.yaml`

- [ ] **Step 1: Write failing domain method-pack tests**

Add `tests/test_domain_method_packs.py`:

```python
from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from scripts.audit_domain_method_packs import audit_domain_profile


REPO_ROOT = Path(__file__).resolve().parents[1]


class DomainMethodPackTests(unittest.TestCase):
    def test_economics_and_finance_profiles_have_executable_method_pack_fields(self) -> None:
        for name in ("economics", "finance"):
            result = audit_domain_profile(REPO_ROOT / "skills" / "domain-profiles" / f"{name}.yaml")
            self.assertEqual([], result.errors, name)

    def test_invalid_method_pack_reports_missing_required_diagnostics(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            path = Path(tmp_dir) / "bad.yaml"
            path.write_text(
                "domain: bad\n"
                "display_name: Bad\n"
                "libraries: {}\n"
                "method_templates:\n"
                "  - name: Difference-in-Differences\n"
                "    tier: standard\n",
                encoding="utf-8",
            )
            result = audit_domain_profile(path)

        self.assertTrue(any("required_diagnostics" in error for error in result.errors))


if __name__ == "__main__":
    unittest.main()
```

- [ ] **Step 2: Run the failing test**

Run: `python3 -m unittest tests.test_domain_method_packs -v`

Expected: FAIL because `scripts.audit_domain_method_packs` does not exist and profile fields are not yet enforced.

- [ ] **Step 3: Extend domain profile schema**

In `schemas/domain-profile.schema.json`, extend `method_templates.items.properties`:

```json
"assumptions": {
  "type": "array",
  "items": {"type": "string"}
},
"required_diagnostics": {
  "type": "array",
  "items": {"type": "string"}
},
"required_artifacts": {
  "type": "array",
  "items": {"type": "string"}
},
"failure_modes": {
  "type": "array",
  "items": {"type": "string"}
},
"minimum_report_fields": {
  "type": "array",
  "items": {"type": "string"}
}
```

- [ ] **Step 4: Add the audit script**

Create `scripts/audit_domain_method_packs.py`:

```python
#!/usr/bin/env python3
from __future__ import annotations

import argparse
from dataclasses import dataclass, field
from pathlib import Path

import yaml


REQUIRED_METHOD_FIELDS = {
    "assumptions",
    "required_diagnostics",
    "required_artifacts",
    "failure_modes",
    "minimum_report_fields",
}


@dataclass
class DomainMethodPackAuditResult:
    errors: list[str] = field(default_factory=list)


def audit_domain_profile(path: Path) -> DomainMethodPackAuditResult:
    payload = yaml.safe_load(path.read_text(encoding="utf-8")) or {}
    errors: list[str] = []
    methods = payload.get("method_templates", [])
    if not isinstance(methods, list) or not methods:
        errors.append(f"{path}: method_templates must be a non-empty list")
        return DomainMethodPackAuditResult(errors=errors)
    for index, method in enumerate(methods, start=1):
        name = method.get("name", f"method[{index}]") if isinstance(method, dict) else f"method[{index}]"
        if not isinstance(method, dict):
            errors.append(f"{path}: {name} must be an object")
            continue
        for field_name in sorted(REQUIRED_METHOD_FIELDS):
            value = method.get(field_name)
            if not isinstance(value, list) or not value:
                errors.append(f"{path}: {name} missing non-empty {field_name}")
    return DomainMethodPackAuditResult(errors=errors)


def main() -> int:
    parser = argparse.ArgumentParser(description="Audit domain profile method packs.")
    parser.add_argument("--strict", action="store_true")
    parser.add_argument("profiles", nargs="*", type=Path)
    args = parser.parse_args()
    profiles = args.profiles or [
        Path("skills/domain-profiles/economics.yaml"),
        Path("skills/domain-profiles/finance.yaml"),
    ]
    errors: list[str] = []
    for profile in profiles:
        errors.extend(audit_domain_profile(profile).errors)
    for error in errors:
        print(f"[FAIL] {error}")
    if errors and args.strict:
        return 1
    print("[PASS] Domain method pack audit complete")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
```

- [ ] **Step 5: Upgrade economics method templates**

For each economics method in `skills/domain-profiles/economics.yaml`, add concrete fields. Example for DID:

```yaml
  - name: Difference-in-Differences
    tier: standard
    description: Estimate treatment effects from treated/control groups before and after a shock.
    languages: [python, r, stata]
    assumptions:
      - parallel trends before treatment
      - no anticipatory treatment response
      - stable treatment definition
      - no simultaneous confounding shock
    required_diagnostics:
      - pre-trend event-study coefficients
      - treatment timing table
      - treated/control balance summary
      - placebo treatment timing check
    required_artifacts:
      - analysis_plan.md
      - design/validity-threat-matrix.md
      - manuscript/tables/
      - manuscript/figures/
    failure_modes:
      - significant pre-treatment trend divergence
      - treatment timing overlaps with another policy shock
      - outcome redefinition after treatment
    minimum_report_fields:
      - treatment definition
      - control group definition
      - time window
      - fixed effects
      - clustering level
      - effect estimate with confidence interval
```

Apply the same shape to RD, IV/2SLS, synthetic control, and panel/event-study methods.

- [ ] **Step 6: Upgrade finance method templates**

For each finance method in `skills/domain-profiles/finance.yaml`, add concrete fields. Example for event study:

```yaml
  - name: Event Study
    tier: standard
    description: Estimate abnormal returns around a clearly dated market event.
    languages: [python, r]
    assumptions:
      - event date is known before outcome measurement
      - estimation window excludes event leakage
      - benchmark model is specified before inspecting results
      - confounding events are identified
    required_diagnostics:
      - event window sensitivity
      - estimation window sensitivity
      - abnormal return model comparison
      - confounding event screen
    required_artifacts:
      - analysis_plan.md
      - design/validity-threat-matrix.md
      - manuscript/tables/
      - manuscript/figures/
    failure_modes:
      - event leakage before the declared event date
      - overlapping events contaminate returns
      - benchmark model chosen after seeing results
    minimum_report_fields:
      - event definition
      - estimation window
      - event window
      - benchmark model
      - cumulative abnormal return
      - statistical test
```

Apply the same shape to factor models, portfolio optimization, GARCH, and option-pricing methods.

- [ ] **Step 7: Run tests**

Run:

```bash
python3 -m unittest tests.test_domain_method_packs -v
python3 scripts/audit_domain_method_packs.py --strict
```

Expected: PASS.

- [ ] **Step 8: Commit**

```bash
git add schemas/domain-profile.schema.json skills/domain-profiles/economics.yaml skills/domain-profiles/finance.yaml scripts/audit_domain_method_packs.py tests/test_domain_method_packs.py
git commit -m "Add executable economics and finance method packs"
```

---

## Task 3: Skill Consumption Of Gate And Method Contracts

**Files:**
- Modify: `skills/C_design/study-designer.md`
- Modify: `skills/C_design/robustness-planner.md`
- Modify: `skills/I_code/stats-engine.md`
- Modify: `skills/I_code/code-builder.md`
- Modify: `skills/I_code/code-review.md`
- Modify: `skills/Z_cross_cutting/self-critique.md`
- Modify: `scripts/audit_skill_sections.py` if new wording needs audit support

- [ ] **Step 1: Add skill-level tests using existing lint**

Extend `tests/test_skill_structure_lint.py` with a test that checks these files mention:

```python
required_tokens = {
    "skills/C_design/study-designer.md": ["quality-gate-contract.yaml", "Q1", "design/validity-threat-matrix.md"],
    "skills/C_design/robustness-planner.md": ["quality-gate-contract.yaml", "method_templates", "required_diagnostics"],
    "skills/I_code/stats-engine.md": ["method_templates", "required_diagnostics", "minimum_report_fields"],
    "skills/I_code/code-builder.md": ["method_templates", "required_artifacts", "failure_modes"],
    "skills/I_code/code-review.md": ["quality-gate-contract.yaml", "failure_modes", "minimum_report_fields"],
}
```

- [ ] **Step 2: Run the failing lint test**

Run: `python3 -m unittest tests.test_skill_structure_lint -v`

Expected: FAIL until the skill files reference the new contracts.

- [ ] **Step 3: Update design skills**

In `skills/C_design/study-designer.md`, add a short section under `## Process`:

```markdown
### Gate And Method-Pack Alignment

When a domain profile is available, load `skills/domain-profiles/[domain].yaml` and use each selected method template's `assumptions`, `required_diagnostics`, `required_artifacts`, `failure_modes`, and `minimum_report_fields` as constraints. Before finalizing `RESEARCH/[topic]/study_design.md`, check Q1 from `standards/quality-gate-contract.yaml`: every RQ or hypothesis must map to a method, data source, measurement plan, and analysis strategy.
```

In `skills/C_design/robustness-planner.md`, add:

```markdown
### Domain Method-Pack Robustness

Use the active domain profile's `method_templates[*].required_diagnostics` as the minimum robustness checklist. If the selected method has no matching template, write an insufficient-input gap note in `RESEARCH/[topic]/design/validity-threat-matrix.md` instead of inventing diagnostics.
```

- [ ] **Step 4: Update Stage I skills**

In `skills/I_code/stats-engine.md`, add:

```markdown
### Method-Pack Execution Constraints

When `--domain` is specified, treat `method_templates[*].required_diagnostics` and `minimum_report_fields` as mandatory output checks. The model or script recommendation must name which diagnostics can be executed with the available inputs and which are blocked.
```

In `skills/I_code/code-builder.md`, add:

```markdown
### Method-Pack Artifact Contract

Generated analysis code must produce or update the active method template's `required_artifacts`. If a method template lists `failure_modes`, include checks or comments that make those risks visible in `RESEARCH/[topic]/code/reproducibility_audit.md`.
```

In `skills/I_code/code-review.md`, add:

```markdown
### Gate-Aware Review

Review code and analysis outputs against `standards/quality-gate-contract.yaml`. For domain methods, compare the implementation to `failure_modes` and `minimum_report_fields`; block when reported estimates omit required fields or diagnostics.
```

- [ ] **Step 5: Run tests**

Run:

```bash
python3 -m unittest tests.test_skill_structure_lint -v
python3 scripts/audit_skill_sections.py --strict
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add skills/C_design/study-designer.md skills/C_design/robustness-planner.md skills/I_code/stats-engine.md skills/I_code/code-builder.md skills/I_code/code-review.md tests/test_skill_structure_lint.py
git commit -m "Wire skills to quality gates and method packs"
```

---

## Task 4: Offline Eval Expansion

**Files:**
- Create the six eval cases listed in the File Structure section.
- Modify: `scripts/run_academic_quality_evals.py`
- Modify: `tests/test_academic_quality_evals.py`

- [ ] **Step 1: Extend eval dimensions**

In `scripts/run_academic_quality_evals.py`, extend `REQUIRED_DIMENSIONS`:

```python
REQUIRED_DIMENSIONS = [
    "artifact_completeness",
    "evidence_traceability",
    "no_fabricated_sources",
    "claim_calibration",
    "venue_fit",
    "method_validity",
    "scholarly_voice",
    "quality_gate_compliance",
    "domain_method_fit",
]
```

- [ ] **Step 2: Update existing eval fixtures**

For every existing file under `evals/academic_quality/cases/*.yaml`, add:

```yaml
expected_dimensions:
  quality_gate_compliance: 0.7
  domain_method_fit: 0.7
```

Use higher values only when the case explicitly exercises gate or domain-method behavior.

- [ ] **Step 3: Add gate-failure eval cases**

Create `evals/academic_quality/cases/q2-unsupported-claim.yaml`:

```yaml
id: q2-unsupported-claim
paper_type: empirical
task_id: F4
description: Manuscript draft includes a central claim that is not present in the evidence ledger.
expected_dimensions:
  artifact_completeness: 0.6
  evidence_traceability: 0.2
  no_fabricated_sources: 0.5
  claim_calibration: 0.2
  venue_fit: 0.5
  method_validity: 0.5
  scholarly_voice: 0.6
  quality_gate_compliance: 0.1
  domain_method_fit: 0.5
expected_failures:
  - Q2 claim-evidence traceability should fail or block.
```

Create analogous cases for Q1, Q3, and Q4 with the corresponding failure mode in `expected_failures`.

- [ ] **Step 4: Add domain method eval cases**

Create `evals/academic_quality/cases/economics-did-invalid-parallel-trends.yaml`:

```yaml
id: economics-did-invalid-parallel-trends
paper_type: empirical
task_id: C3
domain: economics
method: Difference-in-Differences
description: DID design lacks pre-trend evidence and treats post hoc model choice as robustness.
expected_dimensions:
  artifact_completeness: 0.5
  evidence_traceability: 0.6
  no_fabricated_sources: 0.8
  claim_calibration: 0.4
  venue_fit: 0.5
  method_validity: 0.1
  scholarly_voice: 0.6
  quality_gate_compliance: 0.2
  domain_method_fit: 0.1
expected_failures:
  - DID required_diagnostics must flag missing pre-trend event-study coefficients.
```

Create `finance-event-study-leakage.yaml` with `domain: finance`, `method: Event Study`, and an expected failure for event leakage.

- [ ] **Step 5: Update eval tests**

In `tests/test_academic_quality_evals.py`, update expected count and dimensions:

```python
self.assertEqual(12, result.case_count)
self.assertIn("quality_gate_compliance", result.dimension_scores)
self.assertIn("domain_method_fit", result.dimension_scores)
```

- [ ] **Step 6: Run tests**

Run:

```bash
python3 -m unittest tests.test_academic_quality_evals -v
python3 scripts/run_academic_quality_evals.py evals/academic_quality/cases
```

Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add scripts/run_academic_quality_evals.py tests/test_academic_quality_evals.py evals/academic_quality/cases
git commit -m "Expand offline academic quality evals"
```

---

## Task 5: Sync Distribution Copies And Final Validation

**Files:**
- Modify through sync: `qiongli-workflow/`
- Modify through sync: `plugins/qiongli/skills/qiongli-workflow/`
- Modify through sync if package payload is generated: `packages/npm-qiongli/payload/qiongli-workflow/`

- [ ] **Step 1: Sync source changes into portable and plugin packages**

Run:

```bash
bash scripts/sync_skill_package.sh --target all
```

Expected: source skills, templates, standards, and references are copied into distribution surfaces.

- [ ] **Step 2: Regenerate docs if required**

Run:

```bash
python3 scripts/generate_skill_docs.py
python3 scripts/generate_workflow_contract_doc.py
```

Expected: English and Chinese reference docs remain aligned with registry and contracts.

- [ ] **Step 3: Run focused verification**

Run:

```bash
python3 -m unittest tests.test_quality_gate_contract tests.test_domain_method_packs tests.test_skill_set_scorecard tests.test_academic_quality_evals -v
python3 scripts/audit_quality_gates.py --strict
python3 scripts/audit_domain_method_packs.py --strict
python3 scripts/validate_research_standard.py --strict
```

Expected: PASS.

- [ ] **Step 4: Run full verification**

Run:

```bash
python3 -m unittest discover -s tests -v
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add .
git commit -m "Complete skill set optimization contracts"
```

---

## Out Of Scope For This Round

- Adding new top-level skills unless audits prove an existing skill cannot own the behavior.
- Reworking `task-run` execution semantics beyond strict validator awareness.
- Implementing generic `--only` / `--skip` stage toggles.
- Building more domain packs beyond economics and finance.
- Adding network-backed live evals.

## Execution Order

1. Task 0 establishes the baseline and strict-mode placeholders.
2. Task 1 makes Q1-Q4 executable.
3. Task 2 makes the first two domain method packs auditable.
4. Task 3 wires existing skills to the new contracts.
5. Task 4 expands evals to prevent semantic regression.
6. Task 5 syncs and validates the release surfaces.

## Review Checklist

- [ ] No new canonical skill was added without a clear capability gap.
- [ ] Q1-Q4 gate status is auditable from a machine-readable report block.
- [ ] Economics and finance method packs define assumptions, diagnostics, artifacts, failure modes, and minimum report fields.
- [ ] Skills consume domain profiles instead of duplicating domain checklist prose.
- [ ] Offline evals include both gate failures and domain method failures.
- [ ] `scripts/validate_research_standard.py --strict` remains the top-level release guard.
