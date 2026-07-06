# Q1-Q4 Semantic Gates Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Upgrade Qiongli Q1-Q4 quality gates from structural status fields into auditable semantic gate reports with evidence anchors, check results, and blocking issue guidance.

**Architecture:** Keep `content/standards/quality-gate-contract.yaml` as the source of truth, strengthen `content/templates/quality-gate-report.md` as the default report shape, and enforce the shape in `tooling/scripts/audit_quality_gates.py`. Update only the canonical skill sources that own Q1-Q4 gate production or consumption; generated plugin and package payloads stay untouched.

**Tech Stack:** Python 3.12, `yaml`, `unittest`, existing Qiongli `RepoLayout`, Markdown skill cards, YAML standards contracts.

---

## File Structure

- Modify: `content/standards/quality-gate-contract.yaml`
  - Owns canonical Q1-Q4 definitions.
  - Add a shared semantic report schema and gate-specific semantic check IDs.
- Modify: `content/templates/quality-gate-report.md`
  - Owns the user-facing default gate report template.
  - Add populated semantic check scaffolds for Q1-Q4.
- Modify: `tooling/scripts/audit_quality_gates.py`
  - Owns strict validation of quality gate report files.
  - Validate `semantic_checks`, evidence anchors, and structured blocking issues.
- Modify: `tests/test_quality_gate_contract.py`
  - Owns regression tests for gate contract loading, report validation, and CLI failures.
  - Add failing cases for missing semantic checks and malformed evidence anchors.
- Modify: `tests/test_skill_structure_lint.py`
  - Owns static checks that key skill consumers reference required gate and method-pack contracts.
  - Add static coverage for semantic gate report requirements in gate-producing skills.
- Modify: `content/skills/C_design/study-designer.md`
  - Q1 owner for question-to-method alignment.
  - Require a Q1 semantic check before finalizing `study_design.md`.
- Modify: `content/skills/F_writing/manuscript-architect.md`
  - Q2 owner for claim-evidence traceability during writing.
  - Require Q2 evidence anchors and blocking issue behavior for unsupported claims.
- Modify: `content/skills/G_compliance/reporting-checker.md`
  - Q3 owner for reporting completeness.
  - Require Q3 checklist artifacts and waiver evidence.
- Modify: `content/skills/I_code/reproducibility-auditor.md`
  - Q4 owner for reproducibility baseline.
  - Require Q4 environment, data, code, and rerun evidence anchors.
- Modify: `content/workflow/references/stage-C-design.md`
  - Surface Q1 semantic gate behavior in the stage playbook.
- Modify: `content/workflow/references/stage-F-writing.md`
  - Surface Q2 semantic gate behavior in the stage playbook.
- Modify: `content/workflow/references/stage-G-compliance.md`
  - Surface Q3 semantic gate behavior in the stage playbook.
- Modify: `content/workflow/references/stage-I-code.md`
  - Surface Q4 semantic gate behavior in the stage playbook.
- Modify: `docs/maintainer/skill-set-optimization-scorecard.md`
  - Record that executable Q1-Q4 semantic gates are no longer just labels and static templates.

Generated payloads such as plugin directories, distribution archives, and package mirrors must not be edited in this plan.

## Semantic Gate Schema

Use this YAML shape inside every quality gate report:

```yaml
gates:
  Q1:
    status: PASS | WARN | FAIL | BLOCKED
    evidence:
      - artifact: RESEARCH/[topic]/study_design.md
        anchor: "RQ-method-outcome matrix"
        supports: "Each RQ maps to data, method, and analysis strategy."
    semantic_checks:
      - check_id: q1_rq_method_alignment
        status: PASS | WARN | FAIL | BLOCKED
        finding: "Every RQ has a traceable method, data source, measurement plan, and analysis strategy."
        evidence_refs:
          - RESEARCH/[topic]/study_design.md#rq-method-outcome-matrix
    blocking_issues:
      - issue: "RQ2 has no outcome or analysis strategy."
        required_action: "Add an RQ2 row linking data source, measurement, method, and estimand."
```

Validation rules:

- `status` must be one of the contract `status_values`.
- `semantic_checks` must be a non-empty list for every gate.
- Each semantic check must contain `check_id`, `status`, `finding`, and `evidence_refs`.
- `PASS` and `WARN` gates require at least one structured evidence item.
- `FAIL` and `BLOCKED` gates require at least one structured blocking issue.
- Evidence items may remain backward-compatible strings during migration, but the default template must use structured evidence mappings.
- Blocking issues may remain backward-compatible strings during migration, but the default template must use mappings with `issue` and `required_action`.

### Task 1: Extend The Gate Contract And Template

**Files:**
- Modify: `content/standards/quality-gate-contract.yaml`
- Modify: `content/templates/quality-gate-report.md`
- Test: `tests/test_quality_gate_contract.py`

- [ ] **Step 1: Write the failing contract test**

Add this test method to `QualityGateContractTests` in `tests/test_quality_gate_contract.py`:

```python
    def test_contract_defines_semantic_check_ids_for_each_gate(self) -> None:
        contract = self.audit_module.load_gate_contract(CONTRACT_PATH)

        gates = contract.get("gates", {})
        expected_checks = {
            "Q1": {"q1_rq_method_alignment"},
            "Q2": {"q2_claim_evidence_traceability"},
            "Q3": {"q3_reporting_completeness"},
            "Q4": {"q4_reproducibility_baseline"},
        }
        for gate_id, check_ids in expected_checks.items():
            gate = gates[gate_id]
            self.assertIn("semantic_checks", gate)
            found = {item["check_id"] for item in gate["semantic_checks"]}
            self.assertTrue(check_ids.issubset(found), gate_id)
            self.assertIn("semantic_checks", gate["report_fields"])
```

- [ ] **Step 2: Run the targeted failing test**

Run:

```bash
uv run python -m unittest tests.test_quality_gate_contract.QualityGateContractTests.test_contract_defines_semantic_check_ids_for_each_gate -v
```

Expected: FAIL because `semantic_checks` does not exist in `quality-gate-contract.yaml`.

- [ ] **Step 3: Update the contract**

In `content/standards/quality-gate-contract.yaml`, add `semantic_checks` to each gate's `report_fields` and add the gate-specific semantic check list:

```yaml
  Q1:
    name: "question-to-method alignment"
    required_evidence:
      - "Research question, method choice, outcomes, and analysis plan are cross-referenced."
    semantic_checks:
      - check_id: "q1_rq_method_alignment"
        description: "Every central research question or hypothesis maps to data, measurement, method, outcome, and analysis strategy."
        blocking_when:
          - "A research question has no method, data source, outcome, or analysis strategy."
          - "The method can only support a weaker claim than the stated research question requires."
    pass_criteria:
      - "Methods and outcomes directly answer the stated research question without unsupported scope drift."
    fail_conditions:
      - "Research question, design, method, or outcome claims are inconsistent or untraceable."
    report_fields:
      - status
      - evidence
      - semantic_checks
      - blocking_issues
```

Add equivalent semantic checks for Q2-Q4:

```yaml
  Q2:
    semantic_checks:
      - check_id: "q2_claim_evidence_traceability"
        description: "Every central claim maps to source evidence, analysis output, citation anchor, or an explicit gap note."
        blocking_when:
          - "A central claim has no evidence anchor."
          - "A strong causal, predictive, or normative claim is supported only by descriptive evidence."
  Q3:
    semantic_checks:
      - check_id: "q3_reporting_completeness"
        description: "Required sections, checklists, disclosures, and waivers are present for the paper type and venue."
        blocking_when:
          - "A required reporting item is missing without an explicit waiver."
          - "Submission-facing statements contradict methods, data, ethics, or availability artifacts."
  Q4:
    semantic_checks:
      - check_id: "q4_reproducibility_baseline"
        description: "Inputs, code, environment, analysis decisions, and rerun limits are documented with artifact anchors."
        blocking_when:
          - "Core data, code, environment, or rerun instructions are missing."
          - "A reported result cannot be traced to an input, script, command, or output artifact."
```

- [ ] **Step 4: Update the report template**

Replace the fenced YAML block in `content/templates/quality-gate-report.md` with this default blocked but semantically structured report:

```yaml
gates:
  Q1:
    status: BLOCKED
    evidence: []
    semantic_checks:
      - check_id: q1_rq_method_alignment
        status: BLOCKED
        finding: "No RQ-method-outcome evidence has been supplied yet."
        evidence_refs: []
    blocking_issues:
      - issue: "Research question, method, data source, outcome, and analysis strategy are not yet cross-referenced."
        required_action: "Add or update RESEARCH/[topic]/study_design.md with an RQ-method-outcome matrix."
  Q2:
    status: BLOCKED
    evidence: []
    semantic_checks:
      - check_id: q2_claim_evidence_traceability
        status: BLOCKED
        finding: "No claim-evidence ledger evidence has been supplied yet."
        evidence_refs: []
    blocking_issues:
      - issue: "Central manuscript claims are not yet mapped to evidence anchors."
        required_action: "Add or update RESEARCH/[topic]/evidence/claim-evidence-ledger.csv and manuscript claim map."
  Q3:
    status: BLOCKED
    evidence: []
    semantic_checks:
      - check_id: q3_reporting_completeness
        status: BLOCKED
        finding: "No reporting checklist evidence has been supplied yet."
        evidence_refs: []
    blocking_issues:
      - issue: "Required reporting checklist, disclosures, or waiver evidence is missing."
        required_action: "Add or update RESEARCH/[topic]/reporting_checklist.md and submission statements."
  Q4:
    status: BLOCKED
    evidence: []
    semantic_checks:
      - check_id: q4_reproducibility_baseline
        status: BLOCKED
        finding: "No reproducibility baseline evidence has been supplied yet."
        evidence_refs: []
    blocking_issues:
      - issue: "Data, code, environment, command, or rerun evidence is missing."
        required_action: "Add or update RESEARCH/[topic]/code/reproducibility_audit.md and relevant environment records."
```

- [ ] **Step 5: Run the targeted test again**

Run:

```bash
uv run python -m unittest tests.test_quality_gate_contract.QualityGateContractTests.test_contract_defines_semantic_check_ids_for_each_gate -v
```

Expected: PASS.

- [ ] **Step 6: Commit the contract/template task**

```bash
git add content/standards/quality-gate-contract.yaml content/templates/quality-gate-report.md tests/test_quality_gate_contract.py
git commit -m "feat(gates): define semantic quality checks"
```

### Task 2: Enforce Semantic Checks In The Audit Script

**Files:**
- Modify: `tooling/scripts/audit_quality_gates.py`
- Modify: `tests/test_quality_gate_contract.py`

- [ ] **Step 1: Write failing tests for missing semantic checks**

Add these test methods to `QualityGateContractTests`:

```python
    def test_gate_report_rejects_missing_semantic_checks(self) -> None:
        contract = self.audit_module.load_gate_contract(CONTRACT_PATH)
        with tempfile.TemporaryDirectory() as tmp_dir:
            report_path = Path(tmp_dir) / "quality-gate-report.md"
            write_report(
                report_path,
                """
                gates:
                  Q1:
                    status: PASS
                    evidence:
                      - artifact: RESEARCH/topic/study_design.md
                        anchor: RQ-method matrix
                        supports: RQ1 maps to DID and event-study diagnostics.
                    blocking_issues: []
                  Q2:
                    status: BLOCKED
                    evidence: []
                    semantic_checks:
                      - check_id: q2_claim_evidence_traceability
                        status: BLOCKED
                        finding: Missing claim ledger.
                        evidence_refs: []
                    blocking_issues:
                      - issue: Claim ledger missing.
                        required_action: Add claim-evidence-ledger.csv.
                  Q3:
                    status: BLOCKED
                    evidence: []
                    semantic_checks:
                      - check_id: q3_reporting_completeness
                        status: BLOCKED
                        finding: Checklist missing.
                        evidence_refs: []
                    blocking_issues:
                      - issue: Checklist missing.
                        required_action: Add reporting_checklist.md.
                  Q4:
                    status: BLOCKED
                    evidence: []
                    semantic_checks:
                      - check_id: q4_reproducibility_baseline
                        status: BLOCKED
                        finding: Reproducibility audit missing.
                        evidence_refs: []
                    blocking_issues:
                      - issue: Reproducibility audit missing.
                        required_action: Add code/reproducibility_audit.md.
                """,
            )

            result = self.audit_module.audit_gate_report(report_path, contract)

        self.assertFalse(result.passed)
        self.assertIn("Q1 missing report field: semantic_checks", result.errors)

    def test_gate_report_rejects_malformed_semantic_check(self) -> None:
        contract = self.audit_module.load_gate_contract(CONTRACT_PATH)
        with tempfile.TemporaryDirectory() as tmp_dir:
            report_path = Path(tmp_dir) / "quality-gate-report.md"
            write_report(
                report_path,
                """
                gates:
                  Q1:
                    status: BLOCKED
                    evidence: []
                    semantic_checks:
                      - check_id: q1_rq_method_alignment
                        status: BLOCKED
                        evidence_refs: []
                    blocking_issues:
                      - issue: Alignment matrix missing.
                        required_action: Add study_design.md matrix.
                  Q2:
                    status: BLOCKED
                    evidence: []
                    semantic_checks:
                      - check_id: q2_claim_evidence_traceability
                        status: BLOCKED
                        finding: Missing claim ledger.
                        evidence_refs: []
                    blocking_issues:
                      - issue: Claim ledger missing.
                        required_action: Add claim-evidence-ledger.csv.
                  Q3:
                    status: BLOCKED
                    evidence: []
                    semantic_checks:
                      - check_id: q3_reporting_completeness
                        status: BLOCKED
                        finding: Checklist missing.
                        evidence_refs: []
                    blocking_issues:
                      - issue: Checklist missing.
                        required_action: Add reporting_checklist.md.
                  Q4:
                    status: BLOCKED
                    evidence: []
                    semantic_checks:
                      - check_id: q4_reproducibility_baseline
                        status: BLOCKED
                        finding: Reproducibility audit missing.
                        evidence_refs: []
                    blocking_issues:
                      - issue: Reproducibility audit missing.
                        required_action: Add code/reproducibility_audit.md.
                """,
            )

            result = self.audit_module.audit_gate_report(report_path, contract)

        self.assertFalse(result.passed)
        self.assertIn("Q1 semantic_checks[1] missing required field: finding", result.errors)
```

- [ ] **Step 2: Run the failing tests**

Run:

```bash
uv run python -m unittest tests.test_quality_gate_contract.QualityGateContractTests.test_gate_report_rejects_missing_semantic_checks tests.test_quality_gate_contract.QualityGateContractTests.test_gate_report_rejects_malformed_semantic_check -v
```

Expected: FAIL because `audit_gate_report` currently only checks `report_fields`, status, evidence, and blocking issue presence.

- [ ] **Step 3: Add semantic check validators**

In `tooling/scripts/audit_quality_gates.py`, add these helpers after `_has_non_empty_items`:

```python
def _validate_semantic_checks(
    gate_id: str,
    gate_report: dict[str, Any],
    gate_contract: dict[str, Any],
    status_values: set[str],
    errors: list[str],
) -> None:
    semantic_checks = gate_report.get("semantic_checks")
    if not isinstance(semantic_checks, list) or not semantic_checks:
        errors.append(f"{gate_id} missing report field: semantic_checks")
        return

    expected_ids = {
        str(item.get("check_id"))
        for item in _as_list(gate_contract.get("semantic_checks"))
        if isinstance(item, dict) and str(item.get("check_id", "")).strip()
    }
    found_ids: set[str] = set()
    required_fields = ("check_id", "status", "finding", "evidence_refs")

    for index, check in enumerate(semantic_checks, start=1):
        if not isinstance(check, dict):
            errors.append(f"{gate_id} semantic_checks[{index}] must be an object")
            continue
        for field_name in required_fields:
            if field_name not in check:
                errors.append(
                    f"{gate_id} semantic_checks[{index}] missing required field: {field_name}"
                )
        check_id = str(check.get("check_id", "")).strip()
        if check_id:
            found_ids.add(check_id)
        status = str(check.get("status", "")).strip()
        if status not in status_values:
            errors.append(
                f"{gate_id} semantic_checks[{index}] status {status or '<missing>'} "
                "not in contract status_values"
            )
        finding = str(check.get("finding", "")).strip()
        if not finding:
            errors.append(f"{gate_id} semantic_checks[{index}] finding is empty")
        evidence_refs = check.get("evidence_refs")
        if not isinstance(evidence_refs, list):
            errors.append(f"{gate_id} semantic_checks[{index}] evidence_refs must be a list")

    missing_ids = sorted(expected_ids - found_ids)
    for check_id in missing_ids:
        errors.append(f"{gate_id} missing semantic check id: {check_id}")


def _validate_structured_evidence(gate_id: str, evidence: Any, errors: list[str]) -> None:
    if not isinstance(evidence, list):
        errors.append(f"{gate_id} evidence must be a list")
        return
    for index, item in enumerate(evidence, start=1):
        if isinstance(item, str):
            continue
        if not isinstance(item, dict):
            errors.append(f"{gate_id} evidence[{index}] must be a string or object")
            continue
        for field_name in ("artifact", "anchor", "supports"):
            if not str(item.get(field_name, "")).strip():
                errors.append(f"{gate_id} evidence[{index}] missing field: {field_name}")


def _validate_structured_blocking_issues(
    gate_id: str,
    blocking_issues: Any,
    errors: list[str],
) -> None:
    if not isinstance(blocking_issues, list):
        errors.append(f"{gate_id} blocking_issues must be a list")
        return
    for index, item in enumerate(blocking_issues, start=1):
        if isinstance(item, str):
            continue
        if not isinstance(item, dict):
            errors.append(f"{gate_id} blocking_issues[{index}] must be a string or object")
            continue
        for field_name in ("issue", "required_action"):
            if not str(item.get(field_name, "")).strip():
                errors.append(
                    f"{gate_id} blocking_issues[{index}] missing field: {field_name}"
                )
```

Call the helpers inside the per-gate loop in `audit_gate_report` after status validation:

```python
        _validate_semantic_checks(
            gate_id,
            gate_report,
            _as_mapping(gate_contract),
            status_values,
            errors,
        )
        _validate_structured_evidence(gate_id, gate_report.get("evidence"), errors)
        _validate_structured_blocking_issues(
            gate_id,
            gate_report.get("blocking_issues"),
            errors,
        )
```

- [ ] **Step 4: Update the complete-report test fixture**

In `test_gate_report_accepts_complete_report`, add one valid `semantic_checks` block to each gate:

```yaml
semantic_checks:
  - check_id: q1_rq_method_alignment
    status: PASS
    finding: RQ1 maps to method, data source, outcome, and analysis strategy.
    evidence_refs:
      - reports/q1-verification.md#rq-method-matrix
```

Use the matching check IDs for Q2-Q4:

```text
q2_claim_evidence_traceability
q3_reporting_completeness
q4_reproducibility_baseline
```

- [ ] **Step 5: Run all quality gate contract tests**

Run:

```bash
uv run python -m unittest tests.test_quality_gate_contract -v
```

Expected: PASS.

- [ ] **Step 6: Run the strict audit**

Run:

```bash
uv run python scripts/audit_quality_gates.py --strict
```

Expected: PASS with `[PASS] Quality gate report satisfies contract`.

- [ ] **Step 7: Commit the audit task**

```bash
git add tooling/scripts/audit_quality_gates.py tests/test_quality_gate_contract.py
git commit -m "test(gates): enforce semantic gate report shape"
```

### Task 3: Wire Q1-Q4 Requirements Into Core Gate-Owner Skills

**Files:**
- Modify: `content/skills/C_design/study-designer.md`
- Modify: `content/skills/F_writing/manuscript-architect.md`
- Modify: `content/skills/G_compliance/reporting-checker.md`
- Modify: `content/skills/I_code/reproducibility-auditor.md`
- Modify: `tests/test_skill_structure_lint.py`

- [ ] **Step 1: Write the failing static coverage test**

Add this test to `SkillStructureLintTests` in `tests/test_skill_structure_lint.py`:

```python
    def test_core_gate_owner_skills_reference_semantic_gate_report_requirements(self) -> None:
        root = Path(__file__).resolve().parents[1]
        required_tokens = {
            "skills/C_design/study-designer.md": [
                "semantic_checks",
                "q1_rq_method_alignment",
                "RQ-method-outcome matrix",
            ],
            "skills/F_writing/manuscript-architect.md": [
                "semantic_checks",
                "q2_claim_evidence_traceability",
                "claim-evidence ledger",
            ],
            "skills/G_compliance/reporting-checker.md": [
                "semantic_checks",
                "q3_reporting_completeness",
                "reporting checklist",
            ],
            "skills/I_code/reproducibility-auditor.md": [
                "semantic_checks",
                "q4_reproducibility_baseline",
                "reproducibility baseline",
            ],
        }

        missing_tokens: list[str] = []
        layout = RepoLayout(root)
        for relative_path, tokens in required_tokens.items():
            text = layout.resolve_source_path(relative_path).read_text(encoding="utf-8")
            missing_tokens.extend(
                f"{relative_path}: {token}" for token in tokens if token not in text
            )

        self.assertEqual([], missing_tokens)
```

- [ ] **Step 2: Run the failing static coverage test**

Run:

```bash
uv run python -m unittest tests.test_skill_structure_lint.SkillStructureLintTests.test_core_gate_owner_skills_reference_semantic_gate_report_requirements -v
```

Expected: FAIL because the four gate-owner skills do not all reference the semantic report requirements.

- [ ] **Step 3: Update Q1 owner skill**

In `content/skills/C_design/study-designer.md`, extend the existing `### Gate And Method-Pack Alignment` section with this paragraph:

```markdown
Before finalizing the design package, create or update `RESEARCH/[topic]/quality-gate-report.md` with a Q1 `semantic_checks` entry using `q1_rq_method_alignment`. The evidence must include an `RQ-method-outcome matrix` anchor from `RESEARCH/[topic]/study_design.md` or `RESEARCH/[topic]/analysis_plan.md`. If any RQ lacks a method, data source, outcome, measurement plan, estimand, or analysis strategy, set Q1 to `BLOCKED` and add a blocking issue with a concrete required action instead of drafting around the gap.
```

- [ ] **Step 4: Update Q2 owner skill**

In `content/skills/F_writing/manuscript-architect.md`, add this paragraph after `## Writing Harness Contract`:

```markdown
Before accepting a manuscript section as ready, create or update `RESEARCH/[topic]/quality-gate-report.md` with a Q2 `semantic_checks` entry using `q2_claim_evidence_traceability`. The evidence must cite the claim-evidence ledger, manuscript claim map, source note, analysis output, citation anchor, or an explicit gap note. Unsupported central claims must be narrowed, moved to limitations, or recorded as `BLOCKED`; do not turn an unsupported claim into polished prose.
```

- [ ] **Step 5: Update Q3 owner skill**

In `content/skills/G_compliance/reporting-checker.md`, add this paragraph near the beginning of `## Process`:

```markdown
Create or update `RESEARCH/[topic]/quality-gate-report.md` with a Q3 `semantic_checks` entry using `q3_reporting_completeness`. Evidence must point to `RESEARCH/[topic]/reporting_checklist.md`, relevant submission statements, and any explicit waiver for non-applicable items. If a required reporting item is absent or contradicts methods, ethics, data availability, or analysis artifacts, set Q3 to `BLOCKED` or `FAIL` and record the required action.
```

- [ ] **Step 6: Update Q4 owner skill**

In `content/skills/I_code/reproducibility-auditor.md`, add this paragraph near the beginning of `## Process`:

```markdown
Create or update `RESEARCH/[topic]/quality-gate-report.md` with a Q4 `semantic_checks` entry using `q4_reproducibility_baseline`. Evidence must point to data lineage, code entrypoints, environment files, command logs, seeds, outputs, and rerun limits. If a reported result cannot be traced to an input, script, command, or output artifact, set Q4 to `BLOCKED` and record the missing reproducibility baseline as a blocking issue.
```

- [ ] **Step 7: Run static coverage tests**

Run:

```bash
uv run python -m unittest tests.test_skill_structure_lint -v
```

Expected: PASS.

- [ ] **Step 8: Commit skill wiring**

```bash
git add content/skills/C_design/study-designer.md content/skills/F_writing/manuscript-architect.md content/skills/G_compliance/reporting-checker.md content/skills/I_code/reproducibility-auditor.md tests/test_skill_structure_lint.py
git commit -m "feat(skills): wire semantic quality gates into core skills"
```

### Task 4: Surface Semantic Gates In Stage Playbooks

**Files:**
- Modify: `content/workflow/references/stage-C-design.md`
- Modify: `content/workflow/references/stage-F-writing.md`
- Modify: `content/workflow/references/stage-G-compliance.md`
- Modify: `content/workflow/references/stage-I-code.md`
- Test: `tests/test_workflow_contract_doc.py`

- [ ] **Step 1: Write the failing playbook coverage test**

Add this test to the workflow contract documentation test module that already checks stage reference content:

```python
    def test_stage_playbooks_surface_semantic_quality_gate_ids(self) -> None:
        root = Path(__file__).resolve().parents[1]
        required_tokens = {
            "content/workflow/references/stage-C-design.md": [
                "q1_rq_method_alignment",
                "quality-gate-report.md",
            ],
            "content/workflow/references/stage-F-writing.md": [
                "q2_claim_evidence_traceability",
                "quality-gate-report.md",
            ],
            "content/workflow/references/stage-G-compliance.md": [
                "q3_reporting_completeness",
                "quality-gate-report.md",
            ],
            "content/workflow/references/stage-I-code.md": [
                "q4_reproducibility_baseline",
                "quality-gate-report.md",
            ],
        }

        missing: list[str] = []
        for relative_path, tokens in required_tokens.items():
            text = (root / relative_path).read_text(encoding="utf-8")
            missing.extend(f"{relative_path}: {token}" for token in tokens if token not in text)

        self.assertEqual([], missing)
```

- [ ] **Step 2: Run the failing test**

Run:

```bash
uv run python -m unittest tests.test_workflow_contract_doc -v
```

Expected: FAIL because stage playbooks do not yet all mention the semantic check IDs.

- [ ] **Step 3: Update Stage C playbook**

Add this under the Stage C quality gate focus:

```markdown
- Semantic gate report: update `quality-gate-report.md` with `q1_rq_method_alignment`; evidence must anchor each RQ/hypothesis to method, data, outcome, measurement, estimand, and analysis strategy.
```

- [ ] **Step 4: Update Stage F playbook**

Add this under the Stage F quality gate focus:

```markdown
- Semantic gate report: update `quality-gate-report.md` with `q2_claim_evidence_traceability`; evidence must anchor central claims to the claim-evidence ledger, source notes, analysis outputs, citations, or explicit gap notes.
```

- [ ] **Step 5: Update Stage G playbook**

Add this under the Stage G quality gate focus:

```markdown
- Semantic gate report: update `quality-gate-report.md` with `q3_reporting_completeness`; evidence must anchor required checklist items, disclosures, submission statements, and explicit waivers.
```

- [ ] **Step 6: Update Stage I playbook**

Add this under the Stage I quality gate focus:

```markdown
- Semantic gate report: update `quality-gate-report.md` with `q4_reproducibility_baseline`; evidence must anchor data lineage, code entrypoints, commands, environment, outputs, seeds, and rerun limits.
```

- [ ] **Step 7: Run workflow documentation tests**

Run:

```bash
uv run python -m unittest tests.test_workflow_contract_doc -v
```

Expected: PASS.

- [ ] **Step 8: Commit playbook updates**

```bash
git add content/workflow/references/stage-C-design.md content/workflow/references/stage-F-writing.md content/workflow/references/stage-G-compliance.md content/workflow/references/stage-I-code.md tests/test_workflow_contract_doc.py
git commit -m "docs(workflow): surface semantic quality gates"
```

### Task 5: Update Maintainer Scorecard And Validation Expectations

**Files:**
- Modify: `docs/maintainer/skill-set-optimization-scorecard.md`
- Test: `tests/test_skill_set_scorecard.py`

- [ ] **Step 1: Write the failing scorecard test**

Extend `test_scorecard_records_registry_baseline_and_targets` in `tests/test_skill_set_scorecard.py` with:

```python
        self.assertIn("Q1-Q4 semantic gate report", scorecard)
        self.assertIn("semantic_checks", scorecard)
        self.assertIn("quality-gate-report.md", scorecard)
```

- [ ] **Step 2: Run the failing scorecard test**

Run:

```bash
uv run python -m unittest tests.test_skill_set_scorecard -v
```

Expected: FAIL because the scorecard does not yet record the new semantic gate report milestone.

- [ ] **Step 3: Update the scorecard baseline and target table**

In `docs/maintainer/skill-set-optimization-scorecard.md`, update the Q1-Q4 target row to:

```markdown
| Executable Q1-Q4 semantic gates | Q1-Q4 semantic gate report schema, `semantic_checks`, and `quality-gate-report.md` audit exist | Gate reports generated by Q1-Q4 owner skills include evidence anchors and blocking actions in real task-run traces |
```

Add a short section:

```markdown
## Semantic Gate Milestone

The Q1-Q4 gate layer now requires `semantic_checks` in `quality-gate-report.md`.
This moves the baseline from status-only reporting to evidence-anchored checks:

- `q1_rq_method_alignment`
- `q2_claim_evidence_traceability`
- `q3_reporting_completeness`
- `q4_reproducibility_baseline`

The next bar is task-run evidence: real Stage C/F/G/I runs should write gate
reports whose evidence anchors resolve to project artifacts.
```

- [ ] **Step 4: Run scorecard tests**

Run:

```bash
uv run python -m unittest tests.test_skill_set_scorecard -v
```

Expected: PASS.

- [ ] **Step 5: Commit maintainer docs**

```bash
git add docs/maintainer/skill-set-optimization-scorecard.md tests/test_skill_set_scorecard.py
git commit -m "docs(maintainer): record semantic gate milestone"
```

### Task 6: Run Full Validation And Boundary Review

**Files:**
- No source edits in this task unless validation exposes a defect.

- [ ] **Step 1: Run the targeted semantic gate suite**

Run:

```bash
uv run python -m unittest tests.test_quality_gate_contract tests.test_skill_structure_lint tests.test_workflow_contract_doc tests.test_skill_set_scorecard -v
```

Expected: PASS.

- [ ] **Step 2: Run strict gate and method audits**

Run:

```bash
uv run python scripts/audit_quality_gates.py --strict
uv run python scripts/audit_domain_method_packs.py --strict
uv run python scripts/audit_skill_sections.py --strict
```

Expected:

```text
[PASS] Quality gate report satisfies contract
[PASS] Economics and finance method packs satisfy required fields
```

The skill section audit should report 73/73 complete skills.

- [ ] **Step 3: Run the strict research standard validator**

Run:

```bash
uv run python scripts/validate_research_standard.py --strict
```

Expected: PASS with zero failures.

- [ ] **Step 4: Check working tree and generated payload boundaries**

Run:

```bash
git status --short
```

Expected modified paths are limited to:

```text
content/standards/quality-gate-contract.yaml
content/templates/quality-gate-report.md
tooling/scripts/audit_quality_gates.py
tests/test_quality_gate_contract.py
tests/test_skill_structure_lint.py
tests/test_workflow_contract_doc.py
tests/test_skill_set_scorecard.py
content/skills/C_design/study-designer.md
content/skills/F_writing/manuscript-architect.md
content/skills/G_compliance/reporting-checker.md
content/skills/I_code/reproducibility-auditor.md
content/workflow/references/stage-C-design.md
content/workflow/references/stage-F-writing.md
content/workflow/references/stage-G-compliance.md
content/workflow/references/stage-I-code.md
docs/maintainer/skill-set-optimization-scorecard.md
```

If generated payloads, plugin manifests, release archives, local install directories, or marketplace catalog files appear, inspect the diff and remove those unintended edits before finalizing.

- [ ] **Step 5: Commit validation cleanup when needed**

If validation fixes are needed, commit them with a narrow message:

```bash
git add <validated-source-files>
git commit -m "fix(gates): align semantic gate validation"
```

If no validation fixes are needed, do not create an empty commit.

## Implementation Notes

- Keep backward compatibility for existing report evidence strings in `audit_quality_gates.py`; require structured mappings in the default template and new examples.
- Do not add model-based NLP scoring to this phase. This phase makes semantic gate evidence explicit and auditable; runtime generation quality can be evaluated in a later phase with real task-run traces.
- Do not duplicate domain-specific method rules in gate-owner skills. Domain methods remain in `skills/domain-profiles/[domain].yaml`; skills should reference method-pack fields instead of copying DID, event-study, or factor-model rules inline.
- Keep source changes in `content/`, `tooling/scripts/`, `scripts/` wrappers only if needed, `tests/`, and `docs/maintainer/`.
- Do not edit generated plugin/package payloads in this plan.

## Final Verification Commands

```bash
uv run python -m unittest tests.test_quality_gate_contract tests.test_skill_structure_lint tests.test_workflow_contract_doc tests.test_skill_set_scorecard -v
uv run python scripts/audit_quality_gates.py --strict
uv run python scripts/audit_domain_method_packs.py --strict
uv run python scripts/audit_skill_sections.py --strict
uv run python scripts/validate_research_standard.py --strict
git diff --check
```

## Self-Review

Spec coverage:
- Q1-Q4 semantic gate schema is covered by Tasks 1 and 2.
- Gate-owner skill behavior is covered by Task 3.
- Stage playbook routing is covered by Task 4.
- Maintainer scorecard and validation expectations are covered by Task 5.
- Repository boundary and release-safety checks are covered by Task 6.

Placeholder scan:
- This plan contains no deferred implementation markers, unresolved field names, or unspecified test commands.

Type consistency:
- The schema uses `semantic_checks`, `check_id`, `status`, `finding`, `evidence_refs`, `artifact`, `anchor`, `supports`, `issue`, and `required_action` consistently across contract, template, audit tests, and skill text.

## Execution Handoff

Plan complete and saved to `docs/superpowers/plans/2026-06-29-q1-q4-semantic-gates.md`. Two execution options:

1. **Subagent-Driven (recommended)** - Dispatch a fresh subagent per task, review between tasks, fast iteration.
2. **Inline Execution** - Execute tasks in this session using executing-plans, batch execution with checkpoints.
