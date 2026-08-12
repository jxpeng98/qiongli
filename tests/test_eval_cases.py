from __future__ import annotations

import io
import importlib.util
import sys
import tempfile
import unittest
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
                        self.assertIn(assertion.get("type"), {"contains_all", "contains_any"})
                        values = assertion.get("values")
                        self.assertIsInstance(values, list)
                        self.assertTrue(values)
                        self.assertTrue(all(isinstance(value, str) and value for value in values))

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
        scenarios = [
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
            values = assertion["values"]
            if assertion["type"] == "contains_any":
                lines.append(values[0])
            else:
                lines.extend(values)
        return "\n".join(lines) + "\n"


if __name__ == "__main__":
    unittest.main()
