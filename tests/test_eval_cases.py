from __future__ import annotations

import hashlib
import io
import importlib.util
import json
import subprocess
import sys
import tempfile
import unittest
import xml.etree.ElementTree as ET
from contextlib import redirect_stdout
from pathlib import Path

import yaml

from qiongli.source_layout import RepoLayout


REPO_ROOT = Path(__file__).resolve().parents[1]
LAYOUT = RepoLayout(REPO_ROOT)
EVAL_CASES_DIR = REPO_ROOT / "evals" / "cases"
PIPELINES_DIR = LAYOUT.pipelines
RUN_EVAL_PATH = REPO_ROOT / "evals" / "runner" / "run_eval.py"
TARGET_PIPELINES = {"systematic-review-prisma", "empirical-study", "theory-paper"}
SUPPORTED_ASSERTION_TYPES = {
    "contains_all",
    "contains_any",
    "schema",
    "field_constraint",
    "count_conservation",
    "cross_artifact_consistency",
    "locator_syntax",
    "citation_identity",
    "file_digest",
}


def load_yaml(path: Path) -> dict:
    return yaml.safe_load(path.read_text(encoding="utf-8"))


def load_eval_runner():
    spec = importlib.util.spec_from_file_location("test_eval_runner_module", RUN_EVAL_PATH)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"Unable to load eval runner from {RUN_EVAL_PATH}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


RUN_EVAL = load_eval_runner()


class EvalCaseCoverageTests(unittest.TestCase):
    def test_every_eval_case_references_an_existing_pipeline(self) -> None:
        pipeline_ids = {load_yaml(path)["id"] for path in sorted(PIPELINES_DIR.glob("*.yaml"))}

        for case_path in sorted(EVAL_CASES_DIR.glob("*.yaml")):
            case = load_yaml(case_path)
            self.assertIn(case["pipeline"], pipeline_ids, case_path.name)

    def test_every_eval_case_skill_matches_its_pipeline(self) -> None:
        pipelines = {
            load_yaml(path)["id"]: load_yaml(path)
            for path in sorted(PIPELINES_DIR.glob("*.yaml"))
        }

        for case_path in sorted(EVAL_CASES_DIR.glob("*.yaml")):
            case = load_yaml(case_path)
            pipeline_skills = {step["skill"] for step in pipelines[case["pipeline"]]["steps"]}
            expected_skills = set(case.get("expected_outputs", {}))
            self.assertTrue(expected_skills.issubset(pipeline_skills), case_path.name)

    def test_target_refactored_pipelines_have_eval_cases(self) -> None:
        covered_pipelines = {
            load_yaml(path)["pipeline"] for path in sorted(EVAL_CASES_DIR.glob("*.yaml"))
        }
        self.assertTrue(TARGET_PIPELINES.issubset(covered_pipelines))

    def test_every_eval_case_uses_v1_typed_assertions(self) -> None:
        for case_path in sorted(EVAL_CASES_DIR.glob("*.yaml")):
            case = load_yaml(case_path)
            with self.subTest(case=case_path.name):
                self.assertEqual(case.get("schema_version"), "1.0")
                expected_outputs = case.get("expected_outputs")
                self.assertIsInstance(expected_outputs, dict)
                self.assertTrue(expected_outputs)
                for expected in expected_outputs.values():
                    self.assertIs(type(expected.get("required")), bool)
                    self.assertNotIn("must_contain", expected)
                    self.assertNotIn("validation", expected)
                    assertions = expected.get("assertions")
                    self.assertIsInstance(assertions, list)
                    self.assertTrue(assertions)
                    for assertion in assertions:
                        self.assertIsInstance(assertion, dict)
                        assertion_type = assertion.get("type")
                        self.assertIn(assertion_type, SUPPORTED_ASSERTION_TYPES)
                        if assertion_type in {"contains_all", "contains_any"}:
                            values = assertion.get("values")
                            self.assertIsInstance(values, list)
                            self.assertTrue(values)
                            self.assertTrue(
                                all(isinstance(value, str) and value for value in values)
                            )
                        elif assertion_type == "count_conservation":
                            self.assertIsInstance(assertion.get("total"), str)
                            self.assertTrue(assertion["total"])
                            self.assertIsInstance(assertion.get("parts"), list)
                            self.assertTrue(assertion["parts"])

    def test_eval_runner_passes_for_all_cases_with_minimal_fixtures(self) -> None:
        for case_path in sorted(EVAL_CASES_DIR.glob("*.yaml")):
            case = load_yaml(case_path)
            with self.subTest(case=case["case_id"]):
                with tempfile.TemporaryDirectory() as temp_dir:
                    output_dir = Path(temp_dir)
                    self._materialize_case_outputs(case, output_dir)
                    self.assertTrue(RUN_EVAL.run_case(str(case_path), str(output_dir)))

    def test_eval_runner_fails_closed_for_invalid_or_missing_evidence(self) -> None:
        def make_case(
            *,
            schema_version: str = "1.0",
            artifact: str = "result.md",
            required: object = True,
            assertions: object = None,
        ) -> dict:
            if assertions is None:
                assertions = [{"type": "contains_all", "values": ["evidence"]}]
            return {
                "schema_version": schema_version,
                "case_id": "truth-contract",
                "pipeline": "empirical-study",
                "input": {"topic": "truth contract"},
                "expected_outputs": {
                    "skill": {
                        "artifact": artifact,
                        "required": required,
                        "assertions": assertions,
                    }
                },
            }

        legacy_case = make_case()
        legacy_case["expected_outputs"]["skill"]["must_contain"] = ["evidence"]
        missing_input_case = make_case()
        del missing_input_case["input"]
        scenarios = [
            ("missing input topic", missing_input_case, None, "valid-file"),
            ("missing required artifact", make_case(), None, None),
            ("all optional artifacts skipped", make_case(required=False), None, None),
            ("empty required directory", make_case(artifact="analysis/"), None, "empty-dir"),
            ("zero assertions", make_case(assertions=[]), None, "valid-file"),
            (
                "failed contains_any assertion",
                make_case(assertions=[{"type": "contains_any", "values": ["other", "absent"]}]),
                None,
                "valid-file",
            ),
            (
                "malformed assertion",
                make_case(assertions=[{"type": "contains_all", "values": ["evidence", 7]}]),
                None,
                "valid-file",
            ),
            (
                "empty required digest artifact",
                make_case(
                    assertions=[
                        {
                            "type": "file_digest",
                            "sha256": hashlib.sha256(b"").hexdigest(),
                        }
                    ]
                ),
                None,
                "empty-file",
            ),
            (
                "unknown assertion",
                make_case(assertions=[{"type": "not_a_validator", "values": ["evidence"]}]),
                None,
                "valid-file",
            ),
            ("unsupported schema", make_case(schema_version="2.0"), None, "valid-file"),
            ("malformed requiredness", make_case(required="yes"), None, "valid-file"),
            ("legacy assertion shape", legacy_case, None, "valid-file"),
            ("artifact read failure", make_case(), None, "invalid-utf8"),
            ("malformed YAML", None, "schema_version: [", None),
        ]

        for name, payload, case_text, setup in scenarios:
            with self.subTest(name=name), tempfile.TemporaryDirectory() as temp_dir:
                root = Path(temp_dir)
                case_path = root / "case.yaml"
                case_path.write_text(
                    case_text if case_text is not None else yaml.safe_dump(payload),
                    encoding="utf-8",
                )
                output_dir = root / "output"
                output_dir.mkdir()
                if setup == "valid-file":
                    (output_dir / "result.md").write_text("evidence\n", encoding="utf-8")
                elif setup == "empty-file":
                    (output_dir / "result.md").touch()
                elif setup == "invalid-utf8":
                    (output_dir / "result.md").write_bytes(b"\xff")
                elif setup == "empty-dir":
                    (output_dir / "analysis").mkdir()

                stdout = io.StringIO()
                with redirect_stdout(stdout):
                    self.assertFalse(RUN_EVAL.run_case(str(case_path), str(output_dir)))
                if name == "missing required artifact":
                    output = stdout.getvalue()
                    for counter in (
                        "required_missing",
                        "executed_assertions",
                        "failed_assertions",
                        "blocked_assertions",
                        "unknown_validation_types",
                    ):
                        self.assertIn(counter, output)

    def test_eval_runner_allows_missing_optional_beside_executed_assertion(self) -> None:
        case = {
            "schema_version": "1.0",
            "case_id": "optional-artifact",
            "pipeline": "empirical-study",
            "input": {"topic": "optional artifact"},
            "expected_outputs": {
                "required-skill": {
                    "artifact": "result.md",
                    "required": True,
                    "assertions": [{"type": "contains_all", "values": ["evidence"]}],
                },
                "optional-skill": {
                    "artifact": "optional.md",
                    "required": False,
                    "assertions": [{"type": "contains_all", "values": ["extra"]}],
                },
            },
        }
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            case_path = root / "case.yaml"
            case_path.write_text(yaml.safe_dump(case), encoding="utf-8")
            output_dir = root / "output"
            output_dir.mkdir()
            (output_dir / "result.md").write_text("evidence\n", encoding="utf-8")
            self.assertTrue(RUN_EVAL.run_case(str(case_path), str(output_dir)))
            (output_dir / "optional.md").write_text("wrong\n", encoding="utf-8")
            self.assertFalse(RUN_EVAL.run_case(str(case_path), str(output_dir)))

    def test_eval_runner_executes_all_scientific_validators(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            case, output_dir = self._materialize_scientific_case(root)
            case_path = self._write_case(root, case)

            self.assertTrue(RUN_EVAL.run_case(str(case_path), str(output_dir)))
            receipt_path = root / "scientific.json"
            result = self._run_eval_cli(
                case_path,
                output_dir,
                "--json-receipt",
                str(receipt_path),
            )
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            outcomes = json.loads(receipt_path.read_text(encoding="utf-8"))["assertions"]
            self.assertTrue(all(outcome["status"] == "pass" for outcome in outcomes))
            roles = {
                (outcome["output_id"], outcome["index"]): [
                    evidence["role"] for evidence in outcome["evidence"]
                ]
                for outcome in outcomes
            }
            self.assertEqual(roles[("schema", 0)], ["artifact", "schema"])
            self.assertEqual(
                roles[("cross-artifact", 0)], ["artifact", "other-artifact"]
            )
            self.assertEqual(roles[("ledger", 2)], ["artifact", "bibliography"])

    def test_scientific_validator_mismatches_fail(self) -> None:
        scenarios = (
            "schema violation",
            "disallowed field",
            "broken equation",
            "cross-artifact mismatch",
            "invalid locator",
            "missing citekey",
            "wrong digest",
        )

        for scenario in scenarios:
            with self.subTest(scenario=scenario), tempfile.TemporaryDirectory() as temp_dir:
                root = Path(temp_dir)
                case, output_dir = self._materialize_scientific_case(root)

                if scenario == "schema violation":
                    (output_dir / "record.json").write_text(
                        json.dumps({"count": 2}), encoding="utf-8"
                    )
                elif scenario == "disallowed field":
                    case["expected_outputs"]["ledger"]["assertions"][0][
                        "allowed_values"
                    ] = ["theory"]
                elif scenario == "broken equation":
                    (output_dir / "prisma.md").write_text(
                        "Records screened: n = 6\n"
                        "Records excluded: n = 2\n"
                        "Reports sought for retrieval: n = 3\n",
                        encoding="utf-8",
                    )
                elif scenario == "cross-artifact mismatch":
                    (output_dir / "screened.csv").write_text(
                        "record_id\nR1\nR4\n", encoding="utf-8"
                    )
                elif scenario == "invalid locator":
                    (output_dir / "ledger.csv").write_text(
                        "evidence_type,source_id,source_location\n"
                        "paper,Smith2024,near the conclusion\n",
                        encoding="utf-8",
                    )
                elif scenario == "missing citekey":
                    (output_dir / "references.bib").write_text(
                        "@article{Other2024, title={Other}}\n", encoding="utf-8"
                    )
                else:
                    case["expected_outputs"]["digest"]["assertions"][0]["sha256"] = (
                        "0" * 64
                    )

                case_path = self._write_case(root, case)
                self.assertFalse(RUN_EVAL.run_case(str(case_path), str(output_dir)))

    def test_scientific_validator_configuration_and_inputs_block(self) -> None:
        malformed_assertions = (
            ("schema", "schema", {"type": "schema"}),
            (
                "field constraint",
                "ledger",
                {"type": "field_constraint", "field": "evidence_type", "allowed_values": []},
            ),
            (
                "count conservation",
                "counts",
                {"type": "count_conservation", "total": "Records screened", "parts": []},
            ),
            (
                "cross artifact",
                "cross-artifact",
                {
                    "type": "cross_artifact_consistency",
                    "field": "record_id",
                    "other_artifact": "search.csv",
                    "other_field": "record_id",
                    "relation": "overlap",
                },
            ),
            (
                "locator",
                "ledger",
                {"type": "locator_syntax", "field": "source_location", "pattern": ".*"},
            ),
            (
                "citation",
                "ledger",
                {"type": "citation_identity", "bibliography": "../outside.bib"},
            ),
            ("digest", "digest", {"type": "file_digest", "sha256": "not-a-digest"}),
        )

        for name, output_key, malformed in malformed_assertions:
            with self.subTest(config=name), tempfile.TemporaryDirectory() as temp_dir:
                root = Path(temp_dir)
                case, output_dir = self._materialize_scientific_case(root)
                case["expected_outputs"][output_key]["assertions"] = [malformed]
                case_path = self._write_case(root, case)
                stdout = io.StringIO()
                with redirect_stdout(stdout):
                    self.assertFalse(RUN_EVAL.run_case(str(case_path), str(output_dir)))
                self.assertIn("BLOCKED", stdout.getvalue())

        input_scenarios = (
            "malformed JSON",
            "empty applicable CSV",
            "missing referenced artifact",
            "missing count label",
            "duplicate count label",
            "primary path escape",
        )
        for scenario in input_scenarios:
            with self.subTest(input=scenario), tempfile.TemporaryDirectory() as temp_dir:
                root = Path(temp_dir)
                case, output_dir = self._materialize_scientific_case(root)
                if scenario == "malformed JSON":
                    (output_dir / "record.json").write_text("{", encoding="utf-8")
                elif scenario == "empty applicable CSV":
                    (output_dir / "ledger.csv").write_text(
                        "evidence_type,source_id,source_location\n", encoding="utf-8"
                    )
                elif scenario == "missing referenced artifact":
                    (output_dir / "search.csv").unlink()
                elif scenario == "missing count label":
                    (output_dir / "prisma.md").write_text(
                        "Records screened: n = 5\nRecords excluded: n = 2\n",
                        encoding="utf-8",
                    )
                elif scenario == "duplicate count label":
                    (output_dir / "prisma.md").write_text(
                        "Records screened: n = 5\n"
                        "Records screened: n = 5\n"
                        "Records excluded: n = 2\n"
                        "Reports sought for retrieval: n = 3\n",
                        encoding="utf-8",
                    )
                else:
                    (root / "outside.bin").write_bytes(b"outside")
                    case["expected_outputs"]["digest"]["artifact"] = "../outside.bin"

                case_path = self._write_case(root, case)
                stdout = io.StringIO()
                with redirect_stdout(stdout):
                    self.assertFalse(RUN_EVAL.run_case(str(case_path), str(output_dir)))
                self.assertIn("BLOCKED", stdout.getvalue())

    def test_receipts_are_deterministic_redacted_and_status_equivalent(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            case, output_dir, canary = self._materialize_receipt_case(root)
            case_path = self._write_case(root, case)
            destinations = [
                (root / "first.json", root / "first.xml"),
                (root / "second.json", root / "second.xml"),
            ]

            for json_path, junit_path in destinations:
                result = self._run_eval_cli(
                    case_path,
                    output_dir,
                    "--json-receipt",
                    str(json_path),
                    "--junit-receipt",
                    str(junit_path),
                )
                self.assertEqual(result.returncode, 1, result.stdout + result.stderr)

            self.assertEqual(destinations[0][0].read_bytes(), destinations[1][0].read_bytes())
            self.assertEqual(destinations[0][1].read_bytes(), destinations[1][1].read_bytes())

            receipt = json.loads(destinations[0][0].read_text(encoding="utf-8"))
            self.assertEqual(receipt["receipt_version"], "1.0")
            self.assertEqual(
                receipt["case"],
                {
                    "id": "receipt-contract",
                    "pipeline": "empirical-study",
                    "schema_version": "1.0",
                    "status": "blocked",
                    "reason_code": "assertion-blocked",
                },
            )
            self.assertEqual(
                receipt["summary"],
                {
                    "required_missing": 0,
                    "executed_assertions": 2,
                    "failed_assertions": 1,
                    "blocked_assertions": 1,
                    "unknown_validation_types": 0,
                },
            )
            self.assertEqual(
                [record["status"] for record in receipt["assertions"]],
                ["pass", "fail", "blocked", "skip"],
            )
            self.assertEqual(
                [record["reason_code"] for record in receipt["assertions"]],
                [
                    "assertion-passed",
                    "contains-all-failed",
                    "assertion-evidence-unavailable",
                    "optional-artifact-missing",
                ],
            )
            artifacts = {
                "pass": output_dir / "pass.md",
                "fail": output_dir / "fail.md",
                "blocked": output_dir / "blocked.md",
            }
            for record in receipt["assertions"]:
                evidence = record["evidence"][0]
                evidence_path = Path(evidence["path"])
                self.assertFalse(evidence_path.is_absolute())
                self.assertNotIn("..", evidence_path.parts)
                if record["output_id"] in artifacts:
                    self.assertEqual(
                        evidence["sha256"],
                        hashlib.sha256(artifacts[record["output_id"]].read_bytes()).hexdigest(),
                    )
                else:
                    self.assertNotIn("sha256", evidence)

            suite = ET.fromstring(destinations[0][1].read_bytes())
            self.assertEqual(
                {name: suite.attrib[name] for name in ("tests", "failures", "errors", "skipped")},
                {"tests": "4", "failures": "1", "errors": "1", "skipped": "1"},
            )
            testcases = suite.findall("testcase")
            self.assertEqual(len(testcases), len(receipt["assertions"]))
            for record, testcase in zip(receipt["assertions"], testcases, strict=True):
                properties = self._junit_properties(testcase)
                self.assertEqual(properties["status"], record["status"])
                self.assertEqual(properties["reason_code"], record["reason_code"])
            self.assertIsNone(testcases[0].find("failure"))
            self.assertIsNotNone(testcases[1].find("failure"))
            self.assertIsNotNone(testcases[2].find("error"))
            self.assertIsNotNone(testcases[3].find("skipped"))

            rendered = destinations[0][0].read_bytes() + destinations[0][1].read_bytes()
            self.assertNotIn(str(root).encode(), rendered)
            self.assertNotIn(canary.encode(), rendered)

    def test_receipts_keep_contract_and_zero_execution_failures_non_green(self) -> None:
        scenarios = (
            ("malformed", "schema_version: [", "case-load-failed", ["blocked"]),
            (
                "all-skipped",
                yaml.safe_dump(
                    {
                        "schema_version": "1.0",
                        "case_id": "all-skipped",
                        "pipeline": "empirical-study",
                        "input": {"topic": "all skipped"},
                        "expected_outputs": {
                            "optional": {
                                "artifact": "optional.md",
                                "required": False,
                                "assertions": [
                                    {"type": "contains_all", "values": ["evidence"]}
                                ],
                            }
                        },
                    },
                    sort_keys=False,
                ),
                "no-assertions-executed",
                ["skip", "blocked"],
            ),
        )

        for name, case_text, reason_code, statuses in scenarios:
            with self.subTest(name=name), tempfile.TemporaryDirectory() as temp_dir:
                root = Path(temp_dir)
                case_path = root / "case.yaml"
                case_path.write_text(case_text, encoding="utf-8")
                output_dir = root / "output"
                output_dir.mkdir()
                json_path = root / "receipt.json"
                junit_path = root / "receipt.xml"
                result = self._run_eval_cli(
                    case_path,
                    output_dir,
                    "--json-receipt",
                    str(json_path),
                    "--junit-receipt",
                    str(junit_path),
                )

                self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
                receipt = json.loads(json_path.read_text(encoding="utf-8"))
                self.assertEqual(receipt["case"]["status"], "blocked")
                self.assertEqual(receipt["case"]["reason_code"], reason_code)
                self.assertEqual(
                    [record["status"] for record in receipt["assertions"]], statuses
                )
                suite = ET.fromstring(junit_path.read_bytes())
                self.assertGreater(int(suite.attrib["errors"]), 0)

    def test_receipt_cli_is_opt_in_independent_and_atomic(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            output_dir = root / "output"
            output_dir.mkdir()
            (output_dir / "result.md").write_text("evidence\n", encoding="utf-8")
            case_path = self._write_case(
                root,
                {
                    "schema_version": "1.0",
                    "case_id": "receipt-flags",
                    "pipeline": "empirical-study",
                    "input": {"topic": "receipt flags"},
                    "expected_outputs": {
                        "result": {
                            "artifact": "result.md",
                            "required": True,
                            "assertions": [
                                {"type": "contains_all", "values": ["evidence"]}
                            ],
                        }
                    },
                },
            )

            before = sorted(str(path.relative_to(root)) for path in root.rglob("*"))
            result = self._run_eval_cli(case_path, output_dir)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertEqual(
                before,
                sorted(str(path.relative_to(root)) for path in root.rglob("*")),
            )

            json_path = root / "nested" / "only.json"
            result = self._run_eval_cli(
                case_path, output_dir, "--json-receipt", str(json_path)
            )
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertTrue(json_path.is_file())

            junit_path = root / "only.xml"
            result = self._run_eval_cli(
                case_path, output_dir, "--junit-receipt", str(junit_path)
            )
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertTrue(junit_path.is_file())

            shared_path = root / "shared.receipt"
            result = self._run_eval_cli(
                case_path,
                output_dir,
                "--json-receipt",
                str(shared_path),
                "--junit-receipt",
                str(shared_path),
            )
            self.assertEqual(result.returncode, 2)
            self.assertFalse(shared_path.exists())

            directory_target = root / "receipt-directory"
            directory_target.mkdir()
            result = self._run_eval_cli(
                case_path,
                output_dir,
                "--json-receipt",
                str(directory_target),
            )
            self.assertEqual(result.returncode, 1)
            self.assertTrue(directory_target.is_dir())
            self.assertEqual(list(root.glob(".receipt-directory.*.tmp")), [])

    def _materialize_case_outputs(self, case: dict, output_dir: Path) -> None:
        for expected in case.get("expected_outputs", {}).values():
            artifact = expected["artifact"]
            content = self._build_content(expected.get("assertions", []))
            artifact_path = output_dir / artifact

            if artifact.endswith("/"):
                artifact_path.mkdir(parents=True, exist_ok=True)
                (artifact_path / "fixture.md").write_text(content, encoding="utf-8")
                continue

            artifact_path.parent.mkdir(parents=True, exist_ok=True)
            artifact_path.write_text(content, encoding="utf-8")

    def _build_content(self, assertions: list[dict]) -> str:
        lines = []
        for assertion in assertions:
            assertion_type = assertion["type"]
            if assertion_type in {"contains_all", "contains_any"}:
                values = assertion["values"]
                if assertion_type == "contains_any":
                    lines.append(values[0])
                else:
                    lines.extend(values)
            elif assertion_type == "count_conservation":
                parts = assertion["parts"]
                lines.append(f"{assertion['total']}: n = {len(parts)}")
                lines.extend(f"{part}: n = 1" for part in parts)
            else:
                self.fail(f"Shared case fixture cannot render {assertion_type}")
        return "\n".join(lines) + "\n"

    def _materialize_scientific_case(self, root: Path) -> tuple[dict, Path]:
        output_dir = root / "output"
        output_dir.mkdir()
        payload = b"\x00qiongli-eval\xff"

        (root / "schema.json").write_text(
            json.dumps(
                {
                    "type": "object",
                    "required": ["case_id", "count"],
                    "properties": {
                        "case_id": {"type": "string", "minLength": 1},
                        "count": {"type": "integer", "minimum": 1},
                    },
                    "additionalProperties": False,
                }
            ),
            encoding="utf-8",
        )
        (output_dir / "record.json").write_text(
            json.dumps({"case_id": "C1", "count": 2}), encoding="utf-8"
        )
        (output_dir / "record.yaml").write_text(
            "case_id: C2\ncount: 3\n", encoding="utf-8"
        )
        (output_dir / "ledger.csv").write_text(
            "evidence_type,source_id,source_location\n"
            "paper,Smith2024,p. 4\n"
            "theory,Jones2025,Jones2025:methods section\n"
            "paper,Brown2023,pp. 12-14\n",
            encoding="utf-8",
        )
        (output_dir / "references.bib").write_text(
            "@article{Smith2024, title={Evidence}}\n"
            "@book{Jones2025, title={Theory}}\n"
            "@article{Brown2023, title={Range}}\n",
            encoding="utf-8",
        )
        (output_dir / "prisma.md").write_text(
            "Records screened: n = 5\n"
            "Records excluded: n = 2\n"
            "Reports sought for retrieval: n = 3\n",
            encoding="utf-8",
        )
        (output_dir / "screened.csv").write_text(
            "record_id\nR1\nR2\n", encoding="utf-8"
        )
        (output_dir / "search.csv").write_text(
            "record_id\nR1\nR2\nR3\n", encoding="utf-8"
        )
        (output_dir / "screened-copy.csv").write_text(
            "record_id\nR1\nR2\n", encoding="utf-8"
        )
        (output_dir / "payload.bin").write_bytes(payload)

        case = {
            "schema_version": "1.0",
            "case_id": "scientific-validators",
            "pipeline": "empirical-study",
            "input": {"topic": "scientific validators"},
            "expected_outputs": {
                "schema": {
                    "artifact": "record.json",
                    "required": True,
                    "assertions": [{"type": "schema", "schema": "schema.json"}],
                },
                "schema-yaml": {
                    "artifact": "record.yaml",
                    "required": True,
                    "assertions": [{"type": "schema", "schema": "schema.json"}],
                },
                "ledger": {
                    "artifact": "ledger.csv",
                    "required": True,
                    "assertions": [
                        {
                            "type": "field_constraint",
                            "field": "evidence_type",
                            "allowed_values": ["paper", "theory"],
                        },
                        {"type": "locator_syntax", "field": "source_location"},
                        {
                            "type": "citation_identity",
                            "bibliography": "references.bib",
                        },
                    ],
                },
                "counts": {
                    "artifact": "prisma.md",
                    "required": True,
                    "assertions": [
                        {
                            "type": "count_conservation",
                            "total": "Records screened",
                            "parts": ["Records excluded", "Reports sought for retrieval"],
                        }
                    ],
                },
                "cross-artifact": {
                    "artifact": "screened.csv",
                    "required": True,
                    "assertions": [
                        {
                            "type": "cross_artifact_consistency",
                            "field": "record_id",
                            "other_artifact": "search.csv",
                            "other_field": "record_id",
                            "relation": "subset",
                        },
                        {
                            "type": "cross_artifact_consistency",
                            "field": "record_id",
                            "other_artifact": "screened-copy.csv",
                            "other_field": "record_id",
                            "relation": "equal",
                        }
                    ],
                },
                "digest": {
                    "artifact": "payload.bin",
                    "required": True,
                    "assertions": [
                        {
                            "type": "file_digest",
                            "sha256": hashlib.sha256(payload).hexdigest(),
                        }
                    ],
                },
            },
        }
        return case, output_dir

    def _materialize_receipt_case(self, root: Path) -> tuple[dict, Path, str]:
        output_dir = root / "output"
        output_dir.mkdir()
        canary = "restricted-artifact-content-canary"
        (output_dir / "pass.md").write_text(
            f"evidence\n{canary}\n", encoding="utf-8"
        )
        (output_dir / "fail.md").write_text("present\n", encoding="utf-8")
        (output_dir / "blocked.md").write_bytes(b"\xff")
        expected_outputs = {}
        for output_id, artifact, required, value in (
            ("pass", "pass.md", True, "evidence"),
            ("fail", "fail.md", True, "missing"),
            ("blocked", "blocked.md", True, "evidence"),
            ("optional", "optional.md", False, "optional"),
        ):
            expected_outputs[output_id] = {
                "artifact": artifact,
                "required": required,
                "assertions": [{"type": "contains_all", "values": [value]}],
            }
        return (
            {
                "schema_version": "1.0",
                "case_id": "receipt-contract",
                "pipeline": "empirical-study",
                "input": {"topic": "receipt contract"},
                "expected_outputs": expected_outputs,
            },
            output_dir,
            canary,
        )

    def _run_eval_cli(
        self, case_path: Path, output_dir: Path, *arguments: str
    ) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [
                sys.executable,
                str(RUN_EVAL_PATH),
                str(case_path),
                str(output_dir),
                *arguments,
            ],
            cwd=REPO_ROOT,
            capture_output=True,
            text=True,
            check=False,
        )

    def _junit_properties(self, testcase: ET.Element) -> dict[str, str]:
        properties = testcase.find("properties")
        self.assertIsNotNone(properties)
        return {
            item.attrib["name"]: item.attrib["value"]
            for item in properties.findall("property")
        }

    def _write_case(self, root: Path, case: dict) -> Path:
        case_path = root / "case.yaml"
        case_path.write_text(yaml.safe_dump(case, sort_keys=False), encoding="utf-8")
        return case_path


if __name__ == "__main__":
    unittest.main()
