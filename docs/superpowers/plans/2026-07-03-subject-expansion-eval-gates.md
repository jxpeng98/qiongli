# Subject Expansion Evaluation Gates Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the infrastructure that prevents new adaptive runtime subjects from activating until they have a validated contract and passing evaluation gate.

**Architecture:** Introduce a focused runtime-subject contract loader, keep subject resource validation outside the router, extend the existing router evaluation runner with subject-scoped gate reports, and make subject refinement respect activation status when loading subject-level resources. Existing economics and finance behavior remains compatible; candidate subjects can exist as packaged content without being suggested by adaptive runtime.

**Tech Stack:** Python 3.12 standard library, PyYAML already used by the project, unittest, JSON/YAML content files, existing Qiongli bridge modules.

---

## Desired Final Commit Shape

After all tasks pass and reviews are complete, squash implementation commits into these planned groups:

1. `feat(subjects): add runtime subject contracts`
   - contract loader, schema, subject manifest files, loader tests
2. `feat(eval): gate subject expansion activation`
   - subject-scoped eval runner, fixture metadata, gate tests
3. `feat(subjects): enforce subject activation gates`
   - subject refinement/resource activation guard, runtime tests
4. `docs(subjects): document subject expansion gates`
   - CLI/install/release docs and final verification notes

Do not squash the whole feature into one commit.

## File Structure

Create:

- `packages/python-qiongli/src/qiongli/bridges/subject_contracts.py`
  - One responsibility: load and validate runtime-subject contracts.
- `tests/test_subject_contracts.py`
  - Unit tests for schema-like validation, path safety, and default repository contracts.
- `content/schemas/runtime-subject.schema.json`
  - Documentation and release validation schema for runtime subject manifests.
- `content/subjects/<subject>/runtime-subject.yaml`
  - Runtime activation metadata for `economics`, `finance`, `accounting`, `business`,
    `political-economy`, `geoeconomics`, and `economics-accounting`.

Modify:

- `tooling/scripts/evaluate_subject_router.py`
  - Add recursive fixture loading, `--subject`, `--gate runtime-enabled`, and gate reports.
- `tests/test_subject_router_eval.py`
  - Add gate and recursive fixture tests.
- `tests/fixtures/subject_router_eval/*.json`
  - Add `subject_under_test` and `tags` metadata without changing current expectations.
- `packages/python-qiongli/src/qiongli/bridges/subject_refinement.py`
  - Read runtime-subject contracts and withhold subject-level resources when not enabled.
- `tests/test_subject_refinement.py`
  - Add activation-gate regression tests.
- `docs/reference/cli.md`
  - Document subject gate evaluation command and status meaning.
- `docs/guide/install.md`
  - Explain content package vs runtime activation.
- `docs/advanced/publish-pypi.md`
  - Add subject gate checks to optional/release-readiness guidance.

---

## Task 1: Runtime Subject Contract Loader

**Files:**

- Create: `packages/python-qiongli/src/qiongli/bridges/subject_contracts.py`
- Create: `tests/test_subject_contracts.py`

- [ ] **Step 1: Write failing tests for contract loading and validation**

Add `tests/test_subject_contracts.py`:

```python
from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

import yaml

from qiongli.bridges.subject_contracts import (
    RuntimeSubjectContract,
    SubjectContractValidationError,
    load_runtime_subject_contracts,
    subject_activation_status,
    validate_runtime_subject_contract,
)


class RuntimeSubjectContractTests(unittest.TestCase):
    def test_validate_runtime_enabled_contract(self) -> None:
        payload = {
            "schema_version": 1.0,
            "subject": "finance",
            "display_name": "Finance",
            "activation_status": "runtime_enabled",
            "extends": "core",
            "domain_profile": "content/skills/domain-profiles/finance.yaml",
            "overlay": "overlays/finance.yaml",
            "subject_skill": "skills/finance/SKILL.md",
            "signal_groups": {
                "method": [],
                "data_or_outcome": [],
                "venue": [],
            },
            "method_lenses": {
                "event-study": {
                    "resource": "method-packs/finance/event-study.yaml",
                    "activation": "method_only",
                }
            },
            "evaluation_pack": "tests/fixtures/subject_router_eval",
            "near_miss_policy": {"forbidden_subjects": ["economics"]},
            "activation_gate": {
                "required_metrics": {
                    "primary_subject_accuracy": 0.90,
                    "suggest_subject_precision": 0.85,
                    "near_miss_false_positives": 0,
                }
            },
        }

        contract = validate_runtime_subject_contract(payload, source="inline.yaml")

        self.assertIsInstance(contract, RuntimeSubjectContract)
        self.assertEqual(contract.subject, "finance")
        self.assertEqual(contract.activation_status, "runtime_enabled")
        self.assertEqual(contract.method_lenses["event-study"]["resource"], "method-packs/finance/event-study.yaml")

    def test_rejects_unknown_activation_status(self) -> None:
        payload = {
            "schema_version": 1.0,
            "subject": "finance",
            "display_name": "Finance",
            "activation_status": "almost-ready",
            "extends": "core",
            "signal_groups": {},
            "method_lenses": {},
            "evaluation_pack": "tests/fixtures/subject_router_eval",
            "activation_gate": {"required_metrics": {}},
        }

        with self.assertRaises(SubjectContractValidationError) as raised:
            validate_runtime_subject_contract(payload, source="bad.yaml")

        self.assertIn("activation_status", str(raised.exception))

    def test_rejects_path_escape(self) -> None:
        payload = {
            "schema_version": 1.0,
            "subject": "finance",
            "display_name": "Finance",
            "activation_status": "runtime_enabled",
            "extends": "core",
            "domain_profile": "../outside.yaml",
            "signal_groups": {},
            "method_lenses": {},
            "evaluation_pack": "tests/fixtures/subject_router_eval",
            "activation_gate": {"required_metrics": {}},
        }

        with self.assertRaises(SubjectContractValidationError) as raised:
            validate_runtime_subject_contract(payload, source="escape.yaml")

        self.assertIn("path escape", str(raised.exception))

    def test_load_runtime_subject_contracts_reads_nested_subject_files(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            subjects_root = Path(tmp_dir)
            finance_dir = subjects_root / "finance"
            finance_dir.mkdir()
            (finance_dir / "runtime-subject.yaml").write_text(
                yaml.safe_dump(
                    {
                        "schema_version": 1.0,
                        "subject": "finance",
                        "display_name": "Finance",
                        "activation_status": "runtime_enabled",
                        "extends": "core",
                        "signal_groups": {},
                        "method_lenses": {},
                        "evaluation_pack": "tests/fixtures/subject_router_eval",
                        "activation_gate": {"required_metrics": {}},
                    }
                ),
                encoding="utf-8",
            )

            contracts = load_runtime_subject_contracts(subjects_root)

        self.assertEqual(set(contracts), {"finance"})
        self.assertEqual(subject_activation_status("finance", contracts), "runtime_enabled")
        self.assertEqual(subject_activation_status("accounting", contracts), "candidate")


if __name__ == "__main__":
    unittest.main()
```

- [ ] **Step 2: Run tests to verify they fail**

Run:

```bash
uv run python -m unittest tests.test_subject_contracts
```

Expected: fail with `ModuleNotFoundError` or missing `subject_contracts`.

- [ ] **Step 3: Implement the contract loader**

Create `packages/python-qiongli/src/qiongli/bridges/subject_contracts.py`:

```python
from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Any, Mapping

import yaml


RUNTIME_SUBJECT_FILE = "runtime-subject.yaml"
ALLOWED_ACTIVATION_STATUSES = {
    "candidate",
    "eval_ready",
    "runtime_enabled",
    "disabled",
}
PATH_FIELDS = {
    "domain_profile",
    "overlay",
    "subject_skill",
    "evaluation_pack",
}


class SubjectContractValidationError(ValueError):
    pass


@dataclass(frozen=True)
class RuntimeSubjectContract:
    subject: str
    display_name: str
    activation_status: str
    extends: str
    source: str
    domain_profile: str
    overlay: str
    subject_skill: str
    signal_groups: dict[str, list[dict[str, Any]]]
    method_lenses: dict[str, dict[str, Any]]
    evaluation_pack: str
    near_miss_policy: dict[str, Any]
    activation_gate: dict[str, Any]


def load_runtime_subject_contracts(subjects_root: Path | str | None = None) -> dict[str, RuntimeSubjectContract]:
    root = Path(subjects_root) if subjects_root is not None else _default_subjects_root()
    contracts: dict[str, RuntimeSubjectContract] = {}
    if not root.exists():
        return contracts
    for path in sorted(root.glob(f"*/{RUNTIME_SUBJECT_FILE}")):
        payload = yaml.safe_load(path.read_text(encoding="utf-8")) or {}
        contract = validate_runtime_subject_contract(payload, source=_repo_relative(path))
        contracts[contract.subject] = contract
    return contracts


def subject_activation_status(
    subject: str,
    contracts: Mapping[str, RuntimeSubjectContract] | None = None,
) -> str:
    if subject in {"auto", "core"}:
        return "runtime_enabled"
    catalog = dict(contracts) if contracts is not None else load_runtime_subject_contracts()
    contract = catalog.get(subject)
    if contract is None:
        return "candidate"
    return contract.activation_status


def validate_runtime_subject_contract(
    payload: Mapping[str, Any],
    *,
    source: str,
) -> RuntimeSubjectContract:
    if not isinstance(payload, Mapping):
        raise SubjectContractValidationError(f"{source}: expected YAML object")
    subject = _required_string(payload, "subject", source=source)
    display_name = _required_string(payload, "display_name", source=source)
    activation_status = _required_string(payload, "activation_status", source=source)
    if activation_status not in ALLOWED_ACTIVATION_STATUSES:
        raise SubjectContractValidationError(
            f"{source}: activation_status must be one of {sorted(ALLOWED_ACTIVATION_STATUSES)}, got {activation_status!r}"
        )
    for field in PATH_FIELDS:
        value = payload.get(field, "")
        if isinstance(value, str) and value:
            _validate_relative_path(value, source=source, field=field)
    signal_groups = payload.get("signal_groups", {})
    method_lenses = payload.get("method_lenses", {})
    activation_gate = payload.get("activation_gate", {})
    if not isinstance(signal_groups, Mapping):
        raise SubjectContractValidationError(f"{source}: signal_groups must be an object")
    if not isinstance(method_lenses, Mapping):
        raise SubjectContractValidationError(f"{source}: method_lenses must be an object")
    if not isinstance(activation_gate, Mapping):
        raise SubjectContractValidationError(f"{source}: activation_gate must be an object")
    return RuntimeSubjectContract(
        subject=subject,
        display_name=display_name,
        activation_status=activation_status,
        extends=str(payload.get("extends", "core") or "core"),
        source=source,
        domain_profile=str(payload.get("domain_profile", "") or ""),
        overlay=str(payload.get("overlay", "") or ""),
        subject_skill=str(payload.get("subject_skill", "") or ""),
        signal_groups={str(key): list(value or []) for key, value in signal_groups.items() if isinstance(value, list)},
        method_lenses={str(key): dict(value) for key, value in method_lenses.items() if isinstance(value, Mapping)},
        evaluation_pack=str(payload.get("evaluation_pack", "") or ""),
        near_miss_policy=dict(payload.get("near_miss_policy", {}) or {}),
        activation_gate=dict(activation_gate),
    )


def _required_string(payload: Mapping[str, Any], key: str, *, source: str) -> str:
    value = payload.get(key)
    if not isinstance(value, str) or not value.strip():
        raise SubjectContractValidationError(f"{source}: missing required string {key!r}")
    return value.strip()


def _validate_relative_path(value: str, *, source: str, field: str) -> None:
    path = Path(value)
    if path.is_absolute() or ".." in path.parts:
        raise SubjectContractValidationError(f"{source}: {field} path escape is not allowed: {value!r}")


def _default_subjects_root() -> Path:
    runtime_file = Path(__file__).resolve()
    for parent in runtime_file.parents:
        candidate = parent / "content" / "subjects"
        if candidate.is_dir():
            return candidate
    return Path("content") / "subjects"


def _repo_relative(path: Path) -> str:
    resolved = path.resolve()
    for parent in resolved.parents:
        if (parent / ".git").exists():
            try:
                return str(resolved.relative_to(parent))
            except ValueError:
                break
    return str(path)
```

- [ ] **Step 4: Run focused tests**

Run:

```bash
uv run python -m unittest tests.test_subject_contracts
```

Expected: all tests pass.

- [ ] **Step 5: Commit**

```bash
git add packages/python-qiongli/src/qiongli/bridges/subject_contracts.py tests/test_subject_contracts.py
git commit -m "feat(subjects): load runtime subject contracts"
```

---

## Task 2: Repository Runtime Subject Manifests

**Files:**

- Create: `content/schemas/runtime-subject.schema.json`
- Create: `content/subjects/economics/runtime-subject.yaml`
- Create: `content/subjects/finance/runtime-subject.yaml`
- Create: `content/subjects/accounting/runtime-subject.yaml`
- Create: `content/subjects/business/runtime-subject.yaml`
- Create: `content/subjects/political-economy/runtime-subject.yaml`
- Create: `content/subjects/geoeconomics/runtime-subject.yaml`
- Create: `content/subjects/economics-accounting/runtime-subject.yaml`
- Modify: `tests/test_subject_contracts.py`

- [ ] **Step 1: Add failing tests for repository default contracts**

Append to `RuntimeSubjectContractTests`:

```python
    def test_default_repository_contracts_classify_enabled_and_candidates(self) -> None:
        contracts = load_runtime_subject_contracts()

        self.assertEqual(subject_activation_status("economics", contracts), "runtime_enabled")
        self.assertEqual(subject_activation_status("finance", contracts), "runtime_enabled")
        for subject in {
            "accounting",
            "business",
            "political-economy",
            "geoeconomics",
            "economics-accounting",
        }:
            self.assertEqual(subject_activation_status(subject, contracts), "candidate")
            self.assertIn(subject, contracts)

    def test_runtime_enabled_subjects_declare_gate_metrics(self) -> None:
        contracts = load_runtime_subject_contracts()

        for subject in ("economics", "finance"):
            metrics = contracts[subject].activation_gate["required_metrics"]
            self.assertIn("primary_subject_accuracy", metrics)
            self.assertIn("suggest_subject_precision", metrics)
            self.assertIn("near_miss_false_positives", metrics)
```

- [ ] **Step 2: Run tests to verify they fail**

Run:

```bash
uv run python -m unittest tests.test_subject_contracts
```

Expected: fail because default repository manifests do not exist yet.

- [ ] **Step 3: Add JSON schema**

Create `content/schemas/runtime-subject.schema.json`:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://qiongli.dev/schemas/runtime-subject.schema.json",
  "title": "Qiongli Runtime Subject Contract",
  "type": "object",
  "required": [
    "schema_version",
    "subject",
    "display_name",
    "activation_status",
    "extends",
    "signal_groups",
    "method_lenses",
    "evaluation_pack",
    "activation_gate"
  ],
  "properties": {
    "schema_version": {"type": ["number", "string"]},
    "subject": {"type": "string", "minLength": 1},
    "display_name": {"type": "string", "minLength": 1},
    "activation_status": {
      "type": "string",
      "enum": ["candidate", "eval_ready", "runtime_enabled", "disabled"]
    },
    "extends": {"type": "string", "minLength": 1},
    "domain_profile": {"type": "string"},
    "overlay": {"type": "string"},
    "subject_skill": {"type": "string"},
    "signal_groups": {"type": "object"},
    "method_lenses": {"type": "object"},
    "evaluation_pack": {"type": "string"},
    "near_miss_policy": {"type": "object"},
    "activation_gate": {
      "type": "object",
      "required": ["required_metrics"],
      "properties": {
        "required_metrics": {"type": "object"}
      },
      "additionalProperties": true
    }
  },
  "additionalProperties": true
}
```

- [ ] **Step 4: Add economics and finance runtime-enabled manifests**

Create `content/subjects/economics/runtime-subject.yaml`:

```yaml
schema_version: 1.0
subject: economics
display_name: Economics
activation_status: runtime_enabled
extends: core
domain_profile: content/skills/domain-profiles/economics.yaml
overlay: overlays/economics.yaml
subject_skill: skills/economics/SKILL.md
signal_groups:
  method: []
  venue: []
method_lenses:
  did:
    resource: method-packs/economics/did.yaml
    activation: method_only
  causal-identification:
    resource: method-packs/economics/causal-identification.yaml
    activation: subject
evaluation_pack: tests/fixtures/subject_router_eval
near_miss_policy:
  forbidden_subjects:
    - finance
activation_gate:
  required_metrics:
    primary_subject_accuracy: 0.90
    suggest_subject_precision: 0.85
    near_miss_false_positives: 0
```

Create `content/subjects/finance/runtime-subject.yaml`:

```yaml
schema_version: 1.0
subject: finance
display_name: Finance
activation_status: runtime_enabled
extends: core
domain_profile: content/skills/domain-profiles/finance.yaml
overlay: overlays/finance.yaml
subject_skill: skills/finance/SKILL.md
signal_groups:
  method: []
  data_or_outcome: []
  venue: []
method_lenses:
  event-study:
    resource: method-packs/finance/event-study.yaml
    activation: method_only
  asset-pricing:
    resource: method-packs/finance/asset-pricing.yaml
    activation: method_only
evaluation_pack: tests/fixtures/subject_router_eval
near_miss_policy:
  forbidden_subjects:
    - economics
activation_gate:
  required_metrics:
    primary_subject_accuracy: 0.90
    suggest_subject_precision: 0.85
    near_miss_false_positives: 0
```

- [ ] **Step 5: Add candidate manifests**

Create the five candidate files. Use the exact shape below and change only
`subject`, `display_name`, and `domain_profile`.

`content/subjects/accounting/runtime-subject.yaml`:

```yaml
schema_version: 1.0
subject: accounting
display_name: Accounting
activation_status: candidate
extends: core
domain_profile: content/skills/domain-profiles/accounting.yaml
overlay: ""
subject_skill: ""
signal_groups:
  method: []
  data_or_outcome: []
  venue: []
  theory_or_construct: []
method_lenses: {}
evaluation_pack: tests/fixtures/subject_router_eval/accounting
near_miss_policy:
  forbidden_subjects:
    - finance
    - economics
activation_gate:
  required_metrics:
    primary_subject_accuracy: 0.95
    suggest_subject_precision: 0.95
    near_miss_false_positives: 0
```

For `business`, use `display_name: Business` and
`domain_profile: content/skills/domain-profiles/business-management.yaml`.

For `political-economy`, use `display_name: Political Economy` and
`domain_profile: content/skills/domain-profiles/political-economy.yaml`.

For `geoeconomics`, use `display_name: Geoeconomics` and
`domain_profile: content/skills/domain-profiles/geoeconomics.yaml`.

For `economics-accounting`, use `display_name: Economics-Accounting` and
`domain_profile: content/skills/domain-profiles/economics.yaml`.

- [ ] **Step 6: Run focused tests**

Run:

```bash
uv run python -m unittest tests.test_subject_contracts
```

Expected: all tests pass.

- [ ] **Step 7: Commit**

```bash
git add content/schemas/runtime-subject.schema.json content/subjects/*/runtime-subject.yaml tests/test_subject_contracts.py
git commit -m "feat(subjects): add runtime subject manifests"
```

---

## Task 3: Subject-Scoped Evaluation Gate

**Files:**

- Modify: `tooling/scripts/evaluate_subject_router.py`
- Modify: `tests/test_subject_router_eval.py`
- Modify: `tests/fixtures/subject_router_eval/*.json`

- [ ] **Step 1: Add failing tests for recursive loading and gate reports**

Add imports in `tests/test_subject_router_eval.py`:

```python
from tooling.scripts.evaluate_subject_router import subject_gate_report
```

Append tests:

```python
    def test_load_eval_cases_reads_nested_subject_fixture_packs(self) -> None:
        payload = {
            "id": "accounting_near_miss_budget",
            "subject_under_test": "accounting",
            "description": "Budget wording should not activate accounting.",
            "request": "Help me plan a project budget and milestone tracker.",
            "manifest": {
                "active_subject": "auto",
                "subject_mode": "auto",
                "secondary_subjects": [],
                "venue_profiles": [],
                "method_lenses": [],
                "strictness": "standard",
            },
            "expected": {
                "decision": "core_only",
                "primary_subject": "core",
                "suggest_subjects": [],
                "forbidden_subjects": ["accounting"],
                "method_lenses": [],
            },
            "tags": ["accounting", "near_miss"],
        }
        with tempfile.TemporaryDirectory() as tmp_dir:
            nested = Path(tmp_dir) / "accounting"
            nested.mkdir()
            (nested / "near_miss_budget.json").write_text(json.dumps(payload), encoding="utf-8")

            cases = load_eval_cases(Path(tmp_dir))

        self.assertEqual([case.id for case in cases], ["accounting_near_miss_budget"])
        self.assertEqual(cases[0].subject_under_test, "accounting")
        self.assertIn("near_miss", cases[0].tags)

    def test_candidate_subject_gate_reports_blocking_failures(self) -> None:
        cases = load_eval_cases(FIXTURE_DIR)

        report = subject_gate_report("accounting", cases)

        self.assertEqual(report["subject"], "accounting")
        self.assertEqual(report["activation_status"], "candidate")
        self.assertIs(report["eligible_for_runtime_enabled"], False)
        self.assertIn(
            "activation_status is candidate",
            report["blocking_failures"],
        )

    def test_main_subject_gate_json_returns_one_for_candidate_subject(self) -> None:
        stdout = io.StringIO()

        with contextlib.redirect_stdout(stdout):
            exit_code = main(["--subject", "accounting", "--gate", "runtime-enabled", "--json"])

        report = json.loads(stdout.getvalue())
        self.assertEqual(exit_code, 1)
        self.assertEqual(report["subject_gate"]["subject"], "accounting")
        self.assertFalse(report["subject_gate"]["eligible_for_runtime_enabled"])
```

Also extend `EvalCase` assertions in `test_load_eval_cases_reads_all_fixtures`:

```python
self.assertTrue(all(isinstance(case.tags, list) for case in cases))
self.assertTrue(any(case.subject_under_test == "finance" for case in cases))
```

- [ ] **Step 2: Run tests to verify they fail**

Run:

```bash
uv run python -m unittest tests.test_subject_router_eval
```

Expected: fail because `subject_gate_report`, `subject_under_test`, and `tags`
are not implemented.

- [ ] **Step 3: Extend fixture metadata**

Add these fields to existing fixture JSON files:

- `clear_economics.json`: `"subject_under_test": "economics"`, tags
  `["economics", "clear_positive"]`
- `clear_finance.json`: `"subject_under_test": "finance"`, tags
  `["finance", "clear_positive"]`
- `economics_method_only_borrow.json`: `"subject_under_test": "economics"`,
  tags `["economics", "method_only_borrow"]`
- `finance_method_only_borrow.json`: `"subject_under_test": "finance"`,
  tags `["finance", "method_only_borrow"]`
- `locked_subject_neighbor_lens.json`: `"subject_under_test": "economics"`,
  tags `["economics", "locked_subject"]`
- `mixed_econ_finance.json`: `"subject_under_test": "economics"`, tags
  `["economics", "mixed_subject"]`
- `near_miss_finance.json`: `"subject_under_test": "finance"`, tags
  `["finance", "near_miss"]`
- `weak_core_only.json`: `"subject_under_test": "core"`, tags
  `["core", "legacy_regression"]`

- [ ] **Step 4: Implement recursive loading and gate report**

Modify `tooling/scripts/evaluate_subject_router.py`:

```python
from qiongli.bridges.subject_contracts import load_runtime_subject_contracts
```

Extend `EvalCase`:

```python
@dataclass(frozen=True)
class EvalCase:
    id: str
    description: str
    request: str
    manifest: dict[str, Any]
    expected: dict[str, Any]
    source: str
    subject_under_test: str = ""
    tags: list[str] | None = None
```

Change fixture discovery:

```python
for path in sorted(Path(fixture_dir).rglob("*.json")):
```

When constructing cases:

```python
subject_under_test = str(payload.get("subject_under_test", "") or path.parent.name)
tags = [tag for tag in list(payload.get("tags", []) or []) if isinstance(tag, str)]
```

Add constants and gate function:

```python
REQUIRED_GATE_TAGS = {
    "clear_positive",
    "method_only_borrow",
    "near_miss",
}


def subject_gate_report(subject: str, cases: list[EvalCase]) -> dict[str, Any]:
    contracts = load_runtime_subject_contracts()
    contract = contracts.get(subject)
    activation_status = contract.activation_status if contract else "candidate"
    subject_cases = [
        case for case in cases
        if case.subject_under_test == subject or subject in list(case.tags or [])
    ]
    report = evaluate_cases(cases)
    subject_tags = {tag for case in subject_cases for tag in list(case.tags or [])}
    blocking_failures: list[str] = []
    if contract is None:
        blocking_failures.append("missing runtime subject contract")
    if activation_status != "runtime_enabled":
        blocking_failures.append(f"activation_status is {activation_status}")
    missing_tags = sorted(REQUIRED_GATE_TAGS - subject_tags)
    for tag in missing_tags:
        blocking_failures.append(f"missing {tag} fixtures")
    for failure in report["threshold_failures"]:
        metric = failure.get("metric", "unknown")
        blocking_failures.append(f"threshold failure: {metric}")
    return {
        "subject": subject,
        "activation_status": activation_status,
        "eligible_for_runtime_enabled": not blocking_failures,
        "case_count": len(subject_cases),
        "required_tags": sorted(REQUIRED_GATE_TAGS),
        "present_tags": sorted(subject_tags),
        "metrics": report["metrics"],
        "blocking_failures": blocking_failures,
    }
```

Add CLI args:

```python
parser.add_argument("--subject", default="")
parser.add_argument("--gate", choices=["runtime-enabled"], default="")
```

After `report = evaluate_cases(...)`:

```python
if args.subject and args.gate:
    report["subject_gate"] = subject_gate_report(args.subject, load_eval_cases(args.fixture_dir))
```

Return non-zero if `subject_gate.eligible_for_runtime_enabled` is false:

```python
gate = report.get("subject_gate")
if isinstance(gate, Mapping) and not gate.get("eligible_for_runtime_enabled"):
    return 1
```

- [ ] **Step 5: Run focused tests**

Run:

```bash
uv run python -m unittest tests.test_subject_router_eval
```

Expected: all tests pass.

- [ ] **Step 6: Run eval CLI checks**

Run:

```bash
uv run python tooling/scripts/evaluate_subject_router.py --json
```

Expected: exit 0 and `threshold_failures: []`.

Run:

```bash
uv run python tooling/scripts/evaluate_subject_router.py --subject accounting --gate runtime-enabled --json
```

Expected: exit 1, with `subject_gate.eligible_for_runtime_enabled: false`.

- [ ] **Step 7: Commit**

```bash
git add tooling/scripts/evaluate_subject_router.py tests/test_subject_router_eval.py tests/fixtures/subject_router_eval
git commit -m "feat(eval): add subject expansion gate reports"
```

---

## Task 4: Runtime Activation Guard In Subject Refinement

**Files:**

- Modify: `packages/python-qiongli/src/qiongli/bridges/subject_refinement.py`
- Modify: `tests/test_subject_refinement.py`

- [ ] **Step 1: Add failing tests for candidate resource withholding**

Append to `tests/test_subject_refinement.py`:

```python
    def test_candidate_confirmed_subject_withholds_subject_level_resources(self) -> None:
        packet = infer_subject_refinement(
            {"topic": "earnings management", "context": "Tighten the framing."},
            manifest_state=ProjectManifest(
                active_subject="accounting",
                subject_mode="confirmed",
            ),
        ).to_packet()

        self.assertEqual(packet["decision"], "confirm_subject")
        self.assertEqual(packet["primary_subject"], "accounting")
        self.assertEqual(packet["loaded_resources"]["overlays"], [])
        self.assertEqual(packet["loaded_resources"]["subject_skills"], [])
        self.assertTrue(packet["loaded_resources"]["contract_warnings"])
        self.assertIn("activation_status=candidate", packet["loaded_resources"]["contract_warnings"][0])

    def test_runtime_enabled_finance_still_loads_subject_resources(self) -> None:
        packet = infer_subject_refinement(
            {"topic": "revise introduction", "context": "Tighten the framing."},
            manifest_state=ProjectManifest(
                active_subject="finance",
                subject_mode="confirmed",
            ),
        ).to_packet()

        self.assertIn("overlays/finance.yaml", packet["loaded_resources"]["overlays"])
        self.assertIn("skills/finance/SKILL.md", packet["loaded_resources"]["subject_skills"])
        self.assertEqual(packet["loaded_resources"]["contract_warnings"], [])
```

- [ ] **Step 2: Run tests to verify they fail**

Run:

```bash
uv run python -m unittest tests.test_subject_refinement
```

Expected: accounting candidate warning/resource withholding test fails until
the guard is implemented.

- [ ] **Step 3: Implement activation guard**

Modify imports in `subject_refinement.py`:

```python
from .subject_contracts import subject_activation_status
```

Add helper near `_loaded_resources`:

```python
def _subject_level_resources_enabled(subject: str) -> tuple[bool, list[str]]:
    status = subject_activation_status(subject)
    if subject in {"auto", "core"} or status == "runtime_enabled":
        return True, []
    return False, [f"Subject {subject} activation_status={status}; subject resources withheld"]
```

Inside `_loaded_resources`, before appending overlays/skills:

```python
activation_enabled, activation_warnings = _subject_level_resources_enabled(primary_subject)
warnings = list(contract_warnings) + activation_warnings
...
if (
    activation_enabled
    and primary_subject not in {"auto", "core"}
    and ("subject_overlay" in levels or "subject_skill" in levels)
):
    ...
...
"contract_warnings": warnings,
```

Keep method pack lookup unchanged. This allows method-only borrowing to remain
possible when a method pack is configured, while blocking subject overlay and
subject skill activation for candidate subjects.

- [ ] **Step 4: Run focused tests**

Run:

```bash
uv run python -m unittest tests.test_subject_refinement tests.test_subject_resources
```

Expected: all tests pass.

- [ ] **Step 5: Run smoke/eval checks**

Run:

```bash
uv run python tooling/scripts/evaluate_subject_router.py --json
```

Expected: exit 0, no threshold failures.

Run:

```bash
uv run python tooling/scripts/run_subject_runtime_smoke.py --json
```

Expected: exit 0, summary `failed: 0`.

- [ ] **Step 6: Commit**

```bash
git add packages/python-qiongli/src/qiongli/bridges/subject_refinement.py tests/test_subject_refinement.py
git commit -m "feat(subjects): enforce subject activation gates"
```

---

## Task 5: Documentation And Final Verification

**Files:**

- Modify: `docs/reference/cli.md`
- Modify: `docs/guide/install.md`
- Modify: `docs/advanced/publish-pypi.md`

- [ ] **Step 1: Add subject gate documentation**

In `docs/reference/cli.md`, near the subject install/runtime sections, add:

````markdown
### Subject Expansion Gate

Adaptive runtime subjects are not activated only because their content exists
in the installed package. New subjects must pass the runtime subject gate before
the router can suggest them automatically.

```bash
uv run python tooling/scripts/evaluate_subject_router.py \
  --subject accounting \
  --gate runtime-enabled \
  --json
```

`eligible_for_runtime_enabled: false` means the subject can remain packaged as
candidate content, but adaptive runtime must not suggest it as a primary
subject.
````

In `docs/guide/install.md`, add a short paragraph near the subject package
discussion:

```markdown
Subject packages and runtime activation are separate. A marketplace, npm-lite,
or Desktop ZIP package may include subject content for compatibility, but the
adaptive runtime only suggests subjects whose runtime-subject contract is
`runtime_enabled` and whose evaluation gate passes. Candidate subjects can
still be installed or inspected; they are not automatically activated.
```

In `docs/advanced/publish-pypi.md`, add:

````markdown
Optional subject expansion gate:

```bash
uv run python tooling/scripts/evaluate_subject_router.py --json
uv run python tooling/scripts/evaluate_subject_router.py \
  --subject accounting \
  --gate runtime-enabled \
  --json
```

The first command must pass for release. The subject-scoped command should fail
closed for candidate subjects and pass only when a subject is intentionally
promoted to runtime-enabled.
````

- [ ] **Step 2: Run docs checks**

Run:

```bash
rg -n "Subject Expansion Gate|runtime-enabled|eligible_for_runtime_enabled" docs/reference/cli.md docs/guide/install.md docs/advanced/publish-pypi.md
```

Expected: each docs file has at least one relevant match.

Run:

```bash
git diff --check
```

Expected: no output.

- [ ] **Step 3: Run final verification**

Run:

```bash
uv run python -m unittest tests.test_subject_contracts tests.test_subject_router_eval tests.test_subject_refinement tests.test_subject_resources
```

Expected: all tests pass.

Run:

```bash
uv run python -m unittest tests.test_subject_guidance tests.test_subject_lifecycle tests.test_guidance_runtime tests.test_subject_runtime_smoke tests.test_mcp_tool_handlers
```

Expected: all tests pass.

Run:

```bash
uv run python tooling/scripts/evaluate_subject_router.py --json
```

Expected: exit 0 with `threshold_failures: []`.

Run:

```bash
uv run python tooling/scripts/evaluate_subject_router.py --subject accounting --gate runtime-enabled --json
```

Expected: exit 1 with `subject_gate.eligible_for_runtime_enabled: false` and a blocking failure mentioning `activation_status is candidate`.

Run:

```bash
uv run python tooling/scripts/run_subject_runtime_smoke.py --json
```

Expected: exit 0 with `summary.failed: 0`.

- [ ] **Step 4: Commit**

```bash
git add docs/reference/cli.md docs/guide/install.md docs/advanced/publish-pypi.md
git commit -m "docs(subjects): document subject expansion gates"
```

---

## Final Review And Squash Instructions

After all tasks pass:

1. Dispatch a final code reviewer over the full implementation range.
2. Fix any critical or important review findings before proceeding.
3. Squash commits into the planned final shape listed at the top of this plan.
4. Re-run final verification after squashing.
5. Merge back to `dev` only after verification passes.

Do not push unless explicitly requested.
