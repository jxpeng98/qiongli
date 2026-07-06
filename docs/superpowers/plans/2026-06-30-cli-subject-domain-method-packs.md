# CLI Subject Domain Method Packs Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make full CLI-installed Qiongli automatically carry and apply deeper economics and finance domain method contracts on top of the canonical workflow.

**Architecture:** Preserve Qiongli as one complete workflow package. Subject installation and project `active_subject` routing select or infer the domain layer, while canonical skills continue to load `skills/domain-profiles/[domain].yaml` as the executable method contract. Strengthen economics and finance profiles, validate them with the existing audit script, and test that installed subject payloads preserve the enhanced fields.

**Tech Stack:** Python 3.12, `unittest`, PyYAML, JSON Schema, Qiongli subject materializer, Markdown/YAML skill assets.

---

## File Structure

- Modify: `tests/test_domain_method_packs.py`
  - Add failing tests for enhanced method-pack fields and validation errors.
- Modify: `tooling/scripts/audit_domain_method_packs.py`
  - Validate `canonical_references`, `gate_relevance`, `diagnostic_artifacts`, and `failure_triggers`.
- Modify: `content/schemas/domain-profile.schema.json`
  - Document the enhanced method-template field shapes.
- Modify: `content/skills/domain-profiles/economics.yaml`
  - Add literature-anchored diagnostics and blockers for economics methods.
- Modify: `content/skills/domain-profiles/finance.yaml`
  - Add literature-anchored diagnostics and blockers for finance methods.
- Modify: `tests/test_universal_installer.py`
  - Verify installed economics and finance subject payloads preserve enhanced domain profile fields.
- Modify: `content/workflow/SKILL.md`
  - Clarify that subject-installed packages and `active_subject: auto` must load domain packs as a refinement layer, not a replacement for canonical contracts.
- Modify: `docs/maintainer/skill-set-optimization-scorecard.md`
  - Record the economics/finance method-pack hardening status.

Generated package payloads, plugin mirrors, and installed artifacts must not be edited directly.

## Runtime Scheme

The full CLI install remains one complete Qiongli workflow. Domain specialization is a layer:

1. Install-time subject packaging writes `SUBJECT`, `SUBJECT_MANIFEST.json`, subject overlays, and `skills/domain-profiles/*.yaml`.
2. Runtime routing starts from the canonical workflow and checks project-local guidance.
3. If `active_subject` is explicit, use that subject's domain profile.
4. If `active_subject: auto`, infer a temporary subject from project context and topic evidence.
5. Core skills such as `study-designer`, `robustness-planner`, `stats-engine`, `code-builder`, and `code-review` load the matched profile and treat `method_templates[*]` as mandatory constraints.
6. Domain packs refine Q1-Q4 evidence, diagnostics, and blockers. They do not override canonical workflow outputs, evidence gates, or safety constraints.

## Task 1: Add Enhanced Method-Pack Audit Tests

**Files:**
- Modify: `tests/test_domain_method_packs.py`
- Test: `tests/test_domain_method_packs.py`

- [ ] **Step 1: Add failing tests**

Add these tests to `DomainMethodPackAuditTests`:

```python
    def test_economics_and_finance_method_packs_have_enhanced_contract_fields(self) -> None:
        for name in ("economics", "finance"):
            with self.subTest(name=name):
                path = RepoLayout(REPO_ROOT).skills / "domain-profiles" / f"{name}.yaml"
                result = audit_domain_profile(path)

                self.assertEqual([], result.errors)

    def test_invalid_method_pack_reports_missing_enhanced_contract_fields(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            profile = Path(tmp_dir) / "minimal.yaml"
            profile.write_text(
                "\n".join(
                    [
                        "domain: test",
                        "display_name: Test",
                        "libraries: {}",
                        "method_templates:",
                        "  - name: Bare Method",
                        "    tier: standard",
                        "    assumptions: [A]",
                        "    required_diagnostics: [D]",
                        "    required_artifacts: [R]",
                        "    failure_modes: [F]",
                        "    minimum_report_fields: [M]",
                    ]
                ),
                encoding="utf-8",
            )

            result = audit_domain_profile(profile)

        joined = "\n".join(result.errors)
        self.assertIn("canonical_references", joined)
        self.assertIn("gate_relevance", joined)
        self.assertIn("diagnostic_artifacts", joined)
        self.assertIn("failure_triggers", joined)

    def test_invalid_gate_relevance_reports_allowed_quality_gates(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            profile = Path(tmp_dir) / "bad-gate.yaml"
            profile.write_text(
                "\n".join(
                    [
                        "domain: test",
                        "display_name: Test",
                        "libraries: {}",
                        "method_templates:",
                        "  - name: Bad Gate Method",
                        "    tier: standard",
                        "    assumptions: [A]",
                        "    required_diagnostics: [D]",
                        "    required_artifacts: [R]",
                        "    failure_modes: [F]",
                        "    minimum_report_fields: [M]",
                        "    canonical_references:",
                        "      - citation_key: example_2024_method",
                        "        role: baseline method anchor",
                        "    gate_relevance: [Q1, Q9]",
                        "    diagnostic_artifacts:",
                        "      - artifact: RESEARCH/[topic]/analysis/example.md",
                        "        required_for: example claims",
                        "    failure_triggers:",
                        "      - missing comparison group blocks causal claims",
                    ]
                ),
                encoding="utf-8",
            )

            result = audit_domain_profile(profile)

        self.assertIn("gate_relevance contains unsupported gate: Q9", "\n".join(result.errors))
```

- [ ] **Step 2: Run tests to verify RED**

Run:

```bash
uv run python -m unittest tests.test_domain_method_packs -v
```

Expected: FAIL because enhanced fields are not yet required by the audit or present in the profiles.

## Task 2: Implement Enhanced Audit Rules

**Files:**
- Modify: `tooling/scripts/audit_domain_method_packs.py`
- Test: `tests/test_domain_method_packs.py`

- [ ] **Step 1: Extend constants**

Add:

```python
ENHANCED_METHOD_FIELDS = {
    "canonical_references",
    "gate_relevance",
    "diagnostic_artifacts",
    "failure_triggers",
}
QUALITY_GATES = {"Q1", "Q2", "Q3", "Q4"}
```

- [ ] **Step 2: Validate enhanced fields**

Inside the per-method loop, call:

```python
        for field_name in sorted(ENHANCED_METHOD_FIELDS):
            if field_name in {"canonical_references", "diagnostic_artifacts"}:
                if not _is_non_empty_object_list(method.get(field_name)):
                    errors.append(f"{path}: {method_name} missing or empty required method field: {field_name}")
            elif not _is_non_empty_string_list(method.get(field_name)):
                errors.append(f"{path}: {method_name} missing or empty required method field: {field_name}")
        _validate_gate_relevance(path, method_name, method.get("gate_relevance"), errors)
        _validate_canonical_references(path, method_name, method.get("canonical_references"), errors)
        _validate_diagnostic_artifacts(path, method_name, method.get("diagnostic_artifacts"), errors)
```

Add helper functions:

```python
def _is_non_empty_object_list(value: Any) -> bool:
    return isinstance(value, list) and any(isinstance(item, dict) and item for item in value)


def _validate_gate_relevance(path: Path, method_name: str, value: Any, errors: list[str]) -> None:
    if not isinstance(value, list):
        return
    for gate in value:
        if gate not in QUALITY_GATES:
            errors.append(f"{path}: {method_name} gate_relevance contains unsupported gate: {gate}")


def _validate_canonical_references(path: Path, method_name: str, value: Any, errors: list[str]) -> None:
    if not isinstance(value, list):
        return
    for index, ref in enumerate(value, start=1):
        if not isinstance(ref, dict):
            errors.append(f"{path}: {method_name} canonical_references[{index}] must be an object")
            continue
        if not isinstance(ref.get("citation_key"), str) or not ref["citation_key"].strip():
            errors.append(f"{path}: {method_name} canonical_references[{index}] missing citation_key")
        if not isinstance(ref.get("role"), str) or not ref["role"].strip():
            errors.append(f"{path}: {method_name} canonical_references[{index}] missing role")


def _validate_diagnostic_artifacts(path: Path, method_name: str, value: Any, errors: list[str]) -> None:
    if not isinstance(value, list):
        return
    for index, artifact in enumerate(value, start=1):
        if not isinstance(artifact, dict):
            errors.append(f"{path}: {method_name} diagnostic_artifacts[{index}] must be an object")
            continue
        artifact_path = artifact.get("artifact")
        if not isinstance(artifact_path, str) or "RESEARCH/[topic]/" not in artifact_path:
            errors.append(f"{path}: {method_name} diagnostic_artifacts[{index}] must name a RESEARCH/[topic]/ artifact")
        if not isinstance(artifact.get("required_for"), str) or not artifact["required_for"].strip():
            errors.append(f"{path}: {method_name} diagnostic_artifacts[{index}] missing required_for")
```

- [ ] **Step 3: Run tests to verify GREEN for invalid snippets**

Run:

```bash
uv run python -m unittest tests.test_domain_method_packs.DomainMethodPackAuditTests.test_invalid_method_pack_reports_missing_enhanced_contract_fields tests.test_domain_method_packs.DomainMethodPackAuditTests.test_invalid_gate_relevance_reports_allowed_quality_gates -v
```

Expected: PASS for invalid snippet tests, while real profile tests still fail until profiles are updated.

## Task 3: Update Domain Profile Schema

**Files:**
- Modify: `content/schemas/domain-profile.schema.json`

- [ ] **Step 1: Add schema properties under `method_templates.items.properties`**

Add:

```json
"canonical_references": {
  "type": "array",
  "items": {
    "type": "object",
    "required": ["citation_key", "role"],
    "properties": {
      "citation_key": {"type": "string"},
      "role": {"type": "string"},
      "source_url": {"type": "string"}
    },
    "additionalProperties": true
  }
},
"gate_relevance": {
  "type": "array",
  "items": {"type": "string", "enum": ["Q1", "Q2", "Q3", "Q4"]}
},
"diagnostic_artifacts": {
  "type": "array",
  "items": {
    "type": "object",
    "required": ["artifact", "required_for"],
    "properties": {
      "artifact": {"type": "string"},
      "required_for": {"type": "string"}
    },
    "additionalProperties": true
  }
},
"failure_triggers": {
  "type": "array",
  "items": {"type": "string"}
}
```

## Task 4: Strengthen Economics And Finance Profiles

**Files:**
- Modify: `content/skills/domain-profiles/economics.yaml`
- Modify: `content/skills/domain-profiles/finance.yaml`
- Test: `tests/test_domain_method_packs.py`

- [ ] **Step 1: Add enhanced fields to each economics method**

For every economics `method_templates` entry, add:

```yaml
    gate_relevance: [Q1, Q2, Q4]
    canonical_references:
      - citation_key: "..."
        role: "..."
        source_url: "..."
    diagnostic_artifacts:
      - artifact: "RESEARCH/[topic]/analysis/..."
        required_for: "..."
    failure_triggers:
      - "..."
```

Use classic and recent anchors for DID, RD, IV, synthetic control, and BLP/demand estimation.

- [ ] **Step 2: Add enhanced fields to each finance method**

For every finance `method_templates` entry, add the same fields. Use classic and recent anchors for volatility models, event studies, portfolio optimization, factor models, and derivatives.

- [ ] **Step 3: Run profile audit**

Run:

```bash
uv run python -m unittest tests.test_domain_method_packs -v
```

Expected: PASS.

## Task 5: Verify Installed Subject Payload Visibility

**Files:**
- Modify: `tests/test_universal_installer.py`

- [ ] **Step 1: Add economics install visibility assertion**

In `test_install_materializes_requested_subject`, after loading `economics.yaml`, assert:

```python
            economics_profile = yaml.safe_load(
                (skill_dir / "skills" / "domain-profiles" / "economics.yaml").read_text(encoding="utf-8")
            )
            did = economics_profile["method_templates"][0]
            self.assertIn("canonical_references", did)
            self.assertIn("gate_relevance", did)
            self.assertIn("diagnostic_artifacts", did)
            self.assertIn("failure_triggers", did)
```

- [ ] **Step 2: Add finance install visibility test**

Add a focused finance install test:

```python
    def test_install_materializes_finance_subject_with_enhanced_method_pack(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            temp_root = Path(tmp_dir)
            project_dir = temp_root / "project"
            project_dir.mkdir(parents=True)
            codex_home = temp_root / "codex-home"
            env = _isolated_qiongli_env(temp_root)
            env["CODEX_HOME"] = str(codex_home)
            env["PATH"] = ""

            with mock.patch.dict(os.environ, env, clear=True):
                result = install(
                    InstallOptions(
                        repo_root=REPO_ROOT,
                        project_dir=project_dir,
                        target="codex",
                        profile="partial",
                        subject="finance",
                        coverage="focused",
                    )
                )

            self.assertEqual(result, 0)
            skill_dir = codex_home / "skills" / "qiongli-workflow"
            manifest = json.loads((skill_dir / "SUBJECT_MANIFEST.json").read_text(encoding="utf-8"))
            self.assertEqual(manifest["subject"], "finance")
            self.assertEqual(manifest["coverage"], "focused")
            finance_profile = yaml.safe_load(
                (skill_dir / "skills" / "domain-profiles" / "finance.yaml").read_text(encoding="utf-8")
            )
            event_study = next(item for item in finance_profile["method_templates"] if item["name"] == "Event Study")
            self.assertIn("canonical_references", event_study)
            self.assertIn("gate_relevance", event_study)
            self.assertIn("diagnostic_artifacts", event_study)
            self.assertIn("failure_triggers", event_study)
```

- [ ] **Step 3: Run installer tests**

Run:

```bash
uv run python -m unittest tests.test_universal_installer.UniversalInstallerTests.test_install_materializes_requested_subject tests.test_universal_installer.UniversalInstallerTests.test_install_materializes_finance_subject_with_enhanced_method_pack -v
```

Expected: PASS.

## Task 6: Clarify Workflow Runtime Contract

**Files:**
- Modify: `content/workflow/SKILL.md`
- Test: `tests/test_skill_structure_lint.py`

- [ ] **Step 1: Add a static lint test**

Add a test requiring `content/workflow/SKILL.md` to mention subject-installed domain packs and enhanced fields:

```python
    def test_workflow_skill_documents_subject_installed_domain_pack_contract(self) -> None:
        text = (RepoLayout(Path(__file__).resolve().parents[1]).workflow / "SKILL.md").read_text(encoding="utf-8")

        self.assertIn("subject-installed domain profile", text)
        self.assertIn("canonical_references", text)
        self.assertIn("diagnostic_artifacts", text)
        self.assertIn("failure_triggers", text)
```

- [ ] **Step 2: Run test to verify RED**

Run:

```bash
uv run python -m unittest tests.test_skill_structure_lint.SkillStructureLintTests.test_workflow_skill_documents_subject_installed_domain_pack_contract -v
```

Expected: FAIL until the workflow contract is updated.

- [ ] **Step 3: Update `content/workflow/SKILL.md`**

Add a concise `Subject Domain Packs` subsection under `Project-Local Guidance`:

```markdown
### Subject Domain Packs

When Qiongli is installed through the CLI with a subject package, treat the
subject-installed domain profile as the specialization layer for the canonical
workflow. If `SUBJECT_MANIFEST.json` names `economics`, load
`skills/domain-profiles/economics.yaml`; if it names `finance`, load
`skills/domain-profiles/finance.yaml`. If project guidance is
`active_subject: auto`, infer a temporary economics or finance domain from the
task context, then apply the same profile rules.

Domain profiles refine canonical contracts; they do not replace them. For each
matched method, apply `canonical_references` as method anchors,
`gate_relevance` as Q1-Q4 routing hints, `diagnostic_artifacts` as required
local evidence, and `failure_triggers` as blocker language for unsupported
claims.
```

- [ ] **Step 4: Run test to verify GREEN**

Run:

```bash
uv run python -m unittest tests.test_skill_structure_lint.SkillStructureLintTests.test_workflow_skill_documents_subject_installed_domain_pack_contract -v
```

Expected: PASS.

## Task 7: Update Scorecard And Run Verification

**Files:**
- Modify: `docs/maintainer/skill-set-optimization-scorecard.md`

- [ ] **Step 1: Update scorecard**

Record that economics and finance method packs now have audited references, gate routing, diagnostics, blockers, and install visibility tests.

- [ ] **Step 2: Run targeted verification**

Run:

```bash
uv run python -m unittest tests.test_domain_method_packs -v
uv run python -m unittest tests.test_universal_installer.UniversalInstallerTests.test_install_materializes_requested_subject tests.test_universal_installer.UniversalInstallerTests.test_install_materializes_finance_subject_with_enhanced_method_pack -v
uv run python -m unittest tests.test_skill_structure_lint.SkillStructureLintTests.test_workflow_skill_documents_subject_installed_domain_pack_contract -v
```

Expected: PASS.

- [ ] **Step 3: Run audit command**

Run:

```bash
uv run python tooling/scripts/audit_domain_method_packs.py --strict
```

Expected: PASS with no `[FAIL]` lines.

## Acceptance Criteria

- Full CLI subject installs preserve enhanced economics and finance method-pack fields.
- `active_subject: auto` and explicit subject packages are documented as specialization layers on top of canonical workflow contracts.
- Economics and finance profiles include canonical method references, Q1-Q4 relevance, concrete diagnostic artifacts, and blocking triggers.
- The audit rejects incomplete enhanced method-pack entries.
- Targeted tests pass without editing generated payloads.
