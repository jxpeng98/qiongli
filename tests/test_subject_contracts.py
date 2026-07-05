from __future__ import annotations

import json
import os
import re
import sys
import tempfile
import unittest
from pathlib import Path

import yaml

from qiongli.bridges.subject_contracts import (
    RuntimeSubjectContract,
    SubjectContractValidationError,
    _default_subjects_root,
    load_runtime_subject_contracts,
    subject_activation_status,
    validate_runtime_subject_contract,
)


def _valid_payload(**overrides: object) -> dict[str, object]:
    payload: dict[str, object] = {
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
    payload.update(overrides)
    return payload


def _write_runtime_subject(path: Path, **overrides: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        yaml.safe_dump(_valid_payload(**overrides)),
        encoding="utf-8",
    )


class RuntimeSubjectContractTests(unittest.TestCase):
    def test_validate_runtime_enabled_contract(self) -> None:
        contract = validate_runtime_subject_contract(
            _valid_payload(),
            source="inline.yaml",
        )

        self.assertIsInstance(contract, RuntimeSubjectContract)
        self.assertEqual(contract.subject, "finance")
        self.assertEqual(contract.display_name, "Finance")
        self.assertEqual(contract.activation_status, "runtime_enabled")
        self.assertEqual(contract.extends, "core")
        self.assertEqual(contract.source, "inline.yaml")
        self.assertEqual(
            contract.method_lenses["event-study"]["resource"],
            "method-packs/finance/event-study.yaml",
        )

    def test_rejects_unknown_activation_status(self) -> None:
        with self.assertRaises(SubjectContractValidationError) as raised:
            validate_runtime_subject_contract(
                _valid_payload(activation_status="almost-ready"),
                source="bad.yaml",
            )

        self.assertIn("activation_status", str(raised.exception))

    def test_rejects_path_escape(self) -> None:
        with self.assertRaises(SubjectContractValidationError) as raised:
            validate_runtime_subject_contract(
                _valid_payload(domain_profile="../outside.yaml"),
                source="escape.yaml",
            )

        self.assertIn("path escape", str(raised.exception))

    def test_rejects_nested_method_resource_path_escape(self) -> None:
        with self.assertRaises(SubjectContractValidationError) as raised:
            validate_runtime_subject_contract(
                _valid_payload(
                    method_lenses={
                        "evil": {
                            "resource": "../outside.yaml",
                            "activation": "method_only",
                        }
                    }
                ),
                source="method-lens-escape.yaml",
            )

        self.assertIn("path escape", str(raised.exception))

    def test_rejects_absolute_paths(self) -> None:
        with self.assertRaises(SubjectContractValidationError) as raised:
            validate_runtime_subject_contract(
                _valid_payload(evaluation_pack="/tmp/fixtures"),
                source="absolute.yaml",
            )

        self.assertIn("absolute path", str(raised.exception))

    def test_rejects_non_mapping_payload(self) -> None:
        with self.assertRaises(SubjectContractValidationError) as raised:
            validate_runtime_subject_contract([], source="list.yaml")  # type: ignore[arg-type]

        self.assertIn("mapping", str(raised.exception))

    def test_requires_string_fields(self) -> None:
        with self.assertRaises(SubjectContractValidationError) as raised:
            validate_runtime_subject_contract(
                _valid_payload(display_name=""),
                source="missing.yaml",
            )

        self.assertIn("display_name", str(raised.exception))

    def test_rejects_non_mapping_optional_objects(self) -> None:
        with self.assertRaises(SubjectContractValidationError) as raised:
            validate_runtime_subject_contract(
                _valid_payload(signal_groups=[]),
                source="signal-groups.yaml",
            )

        self.assertIn("signal_groups", str(raised.exception))

    def test_rejects_signal_groups_with_non_list_values(self) -> None:
        with self.assertRaises(SubjectContractValidationError) as raised:
            validate_runtime_subject_contract(
                _valid_payload(signal_groups={"method": "not-a-list"}),
                source="signal-group-value.yaml",
            )

        self.assertIn("signal_groups", str(raised.exception))

    def test_rejects_signal_groups_with_non_mapping_entries(self) -> None:
        with self.assertRaises(SubjectContractValidationError) as raised:
            validate_runtime_subject_contract(
                _valid_payload(signal_groups={"method": ["not-a-mapping"]}),
                source="signal-group-entry.yaml",
            )

        self.assertIn("signal_groups", str(raised.exception))

    def test_rejects_method_lenses_with_non_mapping_entries(self) -> None:
        with self.assertRaises(SubjectContractValidationError) as raised:
            validate_runtime_subject_contract(
                _valid_payload(method_lenses={"event-study": "not-a-mapping"}),
                source="method-lens-value.yaml",
            )

        self.assertIn("method_lenses", str(raised.exception))

    def test_rejects_optional_scalar_fields_with_wrong_type(self) -> None:
        with self.assertRaises(SubjectContractValidationError) as raised:
            validate_runtime_subject_contract(
                _valid_payload(extends=["core"]),
                source="scalar-type.yaml",
            )

        self.assertIn("extends", str(raised.exception))

    def test_load_runtime_subject_contracts_reads_nested_subject_files(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            subjects_root = Path(tmp_dir)
            _write_runtime_subject(
                subjects_root / "finance" / "nested" / "runtime-subject.yaml",
            )

            contracts = load_runtime_subject_contracts(subjects_root)

        self.assertEqual(set(contracts), {"finance"})
        self.assertEqual(
            subject_activation_status("finance", contracts),
            "runtime_enabled",
        )
        self.assertEqual(
            subject_activation_status("accounting", contracts),
            "candidate",
        )

    def test_explicit_custom_root_detects_nested_duplicate_subjects(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            subjects_root = Path(tmp_dir)
            _write_runtime_subject(subjects_root / "finance" / "runtime-subject.yaml")
            _write_runtime_subject(
                subjects_root
                / "generated"
                / "qiongli-workflow"
                / "subjects"
                / "finance"
                / "runtime-subject.yaml",
            )

            with self.assertRaises(SubjectContractValidationError) as raised:
                load_runtime_subject_contracts(subjects_root)

        self.assertIn("duplicate subject 'finance'", str(raised.exception))

    def test_default_subjects_root_finds_python_payload_subjects(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            install_root = Path(tmp_dir)
            runtime_file = (
                install_root
                / "site-packages"
                / "qiongli"
                / "bridges"
                / "subject_contracts.py"
            )
            subjects_root = (
                install_root / "site-packages" / "qiongli" / "payload" / "subjects"
            )
            _write_runtime_subject(subjects_root / "finance" / "runtime-subject.yaml")

            discovered = _default_subjects_root(runtime_file)
            contracts = load_runtime_subject_contracts(discovered)

        self.assertEqual(discovered, subjects_root.resolve())
        self.assertEqual(subject_activation_status("finance", contracts), "runtime_enabled")

    def test_default_loading_ignores_generated_payload_duplicate_copies(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            install_root = Path(tmp_dir)
            runtime_file = (
                install_root
                / "site-packages"
                / "qiongli"
                / "bridges"
                / "subject_contracts.py"
            )
            subjects_root = (
                install_root / "site-packages" / "qiongli" / "payload" / "subjects"
            )
            _write_runtime_subject(subjects_root / "finance" / "runtime-subject.yaml")
            _write_runtime_subject(
                subjects_root / "economics" / "runtime-subject.yaml",
                subject="economics",
                display_name="Economics",
                domain_profile="content/skills/domain-profiles/economics.yaml",
                overlay="overlays/economics.yaml",
                subject_skill="skills/economics/SKILL.md",
            )
            for coverage in ("complete", "focused"):
                _write_runtime_subject(
                    subjects_root
                    / "finance"
                    / coverage
                    / "qiongli-workflow"
                    / "subjects"
                    / "finance"
                    / "runtime-subject.yaml",
                    activation_status="candidate",
                )
                _write_runtime_subject(
                    subjects_root
                    / "economics"
                    / coverage
                    / "qiongli-workflow"
                    / "subjects"
                    / "economics"
                    / "runtime-subject.yaml",
                    subject="economics",
                    display_name="Economics",
                    activation_status="candidate",
                    domain_profile="content/skills/domain-profiles/economics.yaml",
                    overlay="overlays/economics.yaml",
                    subject_skill="skills/economics/SKILL.md",
                )

            contracts = load_runtime_subject_contracts(runtime_file=runtime_file)

        self.assertEqual(set(contracts), {"economics", "finance"})
        self.assertEqual(subject_activation_status("finance", contracts), "runtime_enabled")
        self.assertEqual(subject_activation_status("economics", contracts), "runtime_enabled")

    def test_default_subjects_root_finds_ancestor_payload_subjects(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            package_root = Path(tmp_dir) / "package"
            runtime_file = (
                package_root / "lib" / "qiongli" / "bridges" / "subject_contracts.py"
            )
            subjects_root = package_root / "payload" / "subjects"
            _write_runtime_subject(
                subjects_root / "economics" / "runtime-subject.yaml",
                subject="economics",
                display_name="Economics",
                domain_profile="content/skills/domain-profiles/economics.yaml",
                overlay="overlays/economics.yaml",
                subject_skill="skills/economics/SKILL.md",
            )

            discovered = _default_subjects_root(runtime_file)
            contracts = load_runtime_subject_contracts(discovered)

        self.assertEqual(discovered, subjects_root.resolve())
        self.assertEqual(subject_activation_status("economics", contracts), "runtime_enabled")

    def test_default_subjects_root_finds_ancestor_subjects_directory(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            package_root = Path(tmp_dir) / "python-runtime"
            runtime_file = (
                package_root / "lib" / "qiongli" / "bridges" / "subject_contracts.py"
            )
            subjects_root = package_root / "subjects"
            _write_runtime_subject(subjects_root / "finance" / "runtime-subject.yaml")

            discovered = _default_subjects_root(runtime_file)
            contracts = load_runtime_subject_contracts(runtime_file=runtime_file)

        self.assertEqual(discovered, subjects_root.resolve())
        self.assertEqual(subject_activation_status("finance", contracts), "runtime_enabled")

    def test_default_subjects_root_prefers_runtime_subjects_over_package_payload(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            package_root = Path(tmp_dir) / "package"
            runtime_root = package_root / "python-runtime"
            runtime_file = (
                runtime_root / "qiongli" / "bridges" / "subject_contracts.py"
            )
            runtime_subjects = runtime_root / "subjects"
            payload_subjects = package_root / "payload" / "subjects"
            _write_runtime_subject(
                runtime_subjects / "finance" / "runtime-subject.yaml",
                activation_status="runtime_enabled",
            )
            _write_runtime_subject(
                payload_subjects / "finance" / "runtime-subject.yaml",
                activation_status="candidate",
            )

            discovered = _default_subjects_root(runtime_file)
            contracts = load_runtime_subject_contracts(runtime_file=runtime_file)

        self.assertEqual(discovered, runtime_subjects.resolve())
        self.assertEqual(subject_activation_status("finance", contracts), "runtime_enabled")

    def test_default_subjects_root_prefers_source_content_over_payload(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            repo_root = Path(tmp_dir) / "repo"
            runtime_file = (
                repo_root
                / "packages"
                / "python-qiongli"
                / "src"
                / "qiongli"
                / "bridges"
                / "subject_contracts.py"
            )
            source_root = repo_root / "content" / "subjects"
            payload_root = (
                repo_root
                / "packages"
                / "python-qiongli"
                / "src"
                / "qiongli"
                / "payload"
                / "subjects"
            )
            source_root.mkdir(parents=True)
            payload_root.mkdir(parents=True)

            discovered = _default_subjects_root(runtime_file)

        self.assertEqual(discovered, source_root.resolve())

    def test_default_subjects_root_uses_source_path_when_installed_module_is_loaded(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            repo_root = root / "repo"
            source_path = repo_root / "packages" / "python-qiongli" / "src"
            source_path.mkdir(parents=True)
            source_root = repo_root / "content" / "subjects"
            _write_runtime_subject(source_root / "finance" / "runtime-subject.yaml")
            runtime_file = (
                root
                / "site-packages"
                / "qiongli"
                / "bridges"
                / "subject_contracts.py"
            )
            runtime_file.parent.mkdir(parents=True)
            project_root = root / "isolated-project"
            project_root.mkdir()

            old_cwd = Path.cwd()
            sys.path.insert(0, str(source_path))
            try:
                os.chdir(project_root)
                discovered = _default_subjects_root(runtime_file)
                contracts = load_runtime_subject_contracts(runtime_file=runtime_file)
            finally:
                os.chdir(old_cwd)
                sys.path.remove(str(source_path))

        self.assertEqual(discovered, source_root.resolve())
        self.assertEqual(subject_activation_status("finance", contracts), "runtime_enabled")

    def test_default_repository_contracts_classify_runtime_enabled_and_candidates(self) -> None:
        contracts = load_runtime_subject_contracts()

        self.assertEqual(
            subject_activation_status("economics", contracts),
            "runtime_enabled",
        )
        self.assertEqual(
            subject_activation_status("finance", contracts),
            "runtime_enabled",
        )
        self.assertEqual(
            subject_activation_status("accounting", contracts),
            "runtime_enabled",
        )
        self.assertIn("accounting", contracts)
        self.assertEqual(subject_activation_status("business", contracts), "eval_ready")
        self.assertIn("business", contracts)
        for subject in {
            "political-economy",
            "geoeconomics",
            "economics-accounting",
        }:
            self.assertEqual(subject_activation_status(subject, contracts), "candidate")
            self.assertIn(subject, contracts)

    def test_default_deferred_candidate_subjects_are_manifest_shells(self) -> None:
        contracts = load_runtime_subject_contracts()
        deferred_subjects = {
            "political-economy",
            "geoeconomics",
            "economics-accounting",
        }

        for subject in sorted(deferred_subjects):
            with self.subTest(subject=subject):
                contract = contracts[subject]
                self.assertEqual(contract.activation_status, "candidate")
                self.assertEqual(contract.evaluation_pack, "")
                self.assertNotEqual(
                    contract.evaluation_pack,
                    "tests/fixtures/subject_router_eval/accounting",
                )
                self.assertEqual(contract.method_lenses, {})
                self.assertTrue(
                    all(
                        isinstance(entries, list) and not entries
                        for entries in contract.signal_groups.values()
                    )
                )

        self.assertEqual(
            subject_activation_status("accounting", contracts),
            "runtime_enabled",
        )
        self.assertEqual(
            subject_activation_status("finance", contracts),
            "runtime_enabled",
        )
        self.assertEqual(
            subject_activation_status("economics", contracts),
            "runtime_enabled",
        )

    def test_accounting_runtime_enabled_manifest_declares_signals_and_method_lenses(self) -> None:
        contracts = load_runtime_subject_contracts()
        contract = contracts["accounting"]

        self.assertEqual(contract.activation_status, "runtime_enabled")
        self.assertEqual(
            set(contract.signal_groups),
            {"method", "data_or_outcome", "venue", "theory_or_construct"},
        )
        valid_activations = {"subject", "method_only", "context_only"}
        for dimension in ("method", "data_or_outcome", "venue", "theory_or_construct"):
            self.assertTrue(contract.signal_groups[dimension], dimension)
            for entry in contract.signal_groups[dimension]:
                with self.subTest(dimension=dimension, signal_id=entry.get("id")):
                    self.assertIsInstance(entry["id"], str)
                    self.assertTrue(entry["id"].strip())
                    self.assertIsInstance(entry["value"], str)
                    self.assertTrue(entry["value"].strip())
                    self.assertIsInstance(entry["weight"], (int, float))
                    self.assertGreater(entry["weight"], 0)
                    self.assertIn(entry["activation"], valid_activations)
                    for field in ("patterns", "examples", "near_misses"):
                        self.assertIsInstance(entry[field], list)
                        self.assertTrue(entry[field], field)
                        for value in entry[field]:
                            self.assertIsInstance(value, str)
                            self.assertTrue(value.strip(), field)
                    for pattern in entry["patterns"]:
                        re.compile(pattern, re.I)
        self.assertIn("accrual-quality", contract.method_lenses)
        self.assertIn("construct-proxy-audit", contract.method_lenses)
        self.assertEqual(
            contract.method_lenses["accrual-quality"]["activation"],
            "method_only",
        )
        self.assertEqual(
            contract.activation_gate["required_metrics"],
            {
                "primary_subject_accuracy": 0.95,
                "suggest_subject_precision": 0.95,
                "near_miss_false_positives": 0,
            },
        )
        repo_root = Path(__file__).resolve().parents[1]
        declared_paths = [
            contract.domain_profile,
            contract.subject_skill,
            contract.evaluation_pack,
        ]
        if contract.overlay:
            declared_paths.append(contract.overlay)
        declared_paths.extend(
            method_lens["resource"] for method_lens in contract.method_lenses.values()
        )
        for declared_path in declared_paths:
            path = Path(declared_path)
            self.assertFalse(path.is_absolute(), declared_path)
            resolved_path = (repo_root / path).resolve()
            try:
                resolved_path.relative_to(repo_root)
            except ValueError:
                self.fail(f"{declared_path} escapes the repository")
            self.assertTrue(resolved_path.exists(), declared_path)

    def test_business_eval_ready_manifest_declares_signals_and_method_lenses(self) -> None:
        contracts = load_runtime_subject_contracts()
        contract = contracts["business"]

        self.assertEqual(contract.activation_status, "eval_ready")
        self.assertEqual(
            contract.evaluation_pack,
            "tests/fixtures/subject_router_eval/business",
        )
        self.assertEqual(
            set(contract.signal_groups),
            {"method", "data_or_outcome", "venue", "theory_or_construct"},
        )
        valid_activations = {"subject", "method_only", "context_only"}
        for dimension in ("method", "data_or_outcome", "venue", "theory_or_construct"):
            self.assertTrue(contract.signal_groups[dimension], dimension)
            for entry in contract.signal_groups[dimension]:
                with self.subTest(dimension=dimension, signal_id=entry.get("id")):
                    self.assertIsInstance(entry["id"], str)
                    self.assertTrue(entry["id"].strip())
                    self.assertIsInstance(entry["value"], str)
                    self.assertTrue(entry["value"].strip())
                    self.assertIsInstance(entry["weight"], (int, float))
                    self.assertGreater(entry["weight"], 0)
                    self.assertIn(entry["activation"], valid_activations)
                    for field in ("patterns", "examples", "near_misses"):
                        self.assertIsInstance(entry[field], list)
                        self.assertTrue(entry[field], field)
                        for value in entry[field]:
                            self.assertIsInstance(value, str)
                            self.assertTrue(value.strip(), field)
                    for pattern in entry["patterns"]:
                        re.compile(pattern, re.I)
        self.assertIn("business-positioning", contract.method_lenses)
        self.assertIn("qualitative-transparency", contract.method_lenses)
        self.assertIn("construct-level-fit", contract.method_lenses)
        for lens in contract.method_lenses.values():
            self.assertEqual(lens["activation"], "method_only")
        self.assertEqual(
            contract.activation_gate["required_metrics"],
            {
                "primary_subject_accuracy": 0.95,
                "suggest_subject_precision": 0.95,
                "near_miss_false_positives": 0,
            },
        )

    def test_runtime_enabled_subjects_declare_gate_metrics(self) -> None:
        contracts = load_runtime_subject_contracts()
        expected_metrics = {
            "primary_subject_accuracy": 0.90,
            "suggest_subject_precision": 0.85,
            "near_miss_false_positives": 0,
        }

        for subject in ("economics", "finance"):
            metrics = contracts[subject].activation_gate["required_metrics"]
            self.assertIsInstance(metrics, dict)
            self.assertEqual(metrics, expected_metrics)

    def test_runtime_subject_schema_requires_numeric_gate_metrics(self) -> None:
        schema_path = Path("content/schemas/runtime-subject.schema.json")
        schema = json.loads(schema_path.read_text(encoding="utf-8"))

        metric_schema = schema["properties"]["activation_gate"]["properties"][
            "required_metrics"
        ]

        self.assertEqual(
            metric_schema["required"],
            [
                "primary_subject_accuracy",
                "suggest_subject_precision",
                "near_miss_false_positives",
            ],
        )
        for metric_name in metric_schema["required"]:
            self.assertEqual(metric_schema["properties"][metric_name]["type"], "number")


if __name__ == "__main__":
    unittest.main()
