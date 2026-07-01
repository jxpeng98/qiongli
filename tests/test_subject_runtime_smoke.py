from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tooling.scripts.run_subject_runtime_smoke import (
    FIXTURE_DIR,
    SmokeCase,
    _assert_case,
    load_smoke_cases,
    run_smoke_suite,
)


class SubjectRuntimeSmokeTests(unittest.TestCase):
    def test_load_smoke_cases_reads_all_fixtures(self) -> None:
        cases = load_smoke_cases(FIXTURE_DIR)

        names = {case.name for case in cases}
        self.assertEqual(
            names,
            {
                "no_subject_core_only",
                "borrow_finance_lens",
                "suggest_finance_subject",
                "locked_economics_borrow_finance",
            },
        )
        self.assertEqual([case.name for case in cases], sorted(names))
        self.assertTrue(all(isinstance(case, SmokeCase) for case in cases))

    def test_preview_suite_passes_and_writes_inside_isolated_project(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            workspace_root = Path(tmp_dir).resolve()
            report = run_smoke_suite(
                fixture_dir=FIXTURE_DIR,
                workspace_root=workspace_root,
                mode="preview",
                selected_cases=[],
            )

            self.assertEqual(report["summary"]["failed"], 0)
            self.assertEqual(report["summary"]["passed"], 4)
            self.assertEqual(report["mode"], "preview")
            for case in report["cases"]:
                project_root = Path(case["project_root"]).resolve()
                self.assertTrue(project_root.is_relative_to(workspace_root))
                self.assertFalse(case["result"]["run_agents"])
                self.assertEqual(case["status"], "passed")
                for path in case["environment"].values():
                    self.assertTrue(Path(path).resolve().is_relative_to(project_root))

            created_names = {path.name for path in workspace_root.iterdir()}
            self.assertEqual(
                created_names,
                {
                    "no_subject_core_only",
                    "borrow_finance_lens",
                    "suggest_finance_subject",
                    "locked_economics_borrow_finance",
                },
            )

    def test_report_is_json_serializable_and_selected_cases_work(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            report = run_smoke_suite(
                fixture_dir=FIXTURE_DIR,
                workspace_root=Path(tmp_dir),
                mode="preview",
                selected_cases=["suggest_finance_subject"],
            )

        encoded = json.dumps(report, sort_keys=True)
        decoded = json.loads(encoded)
        self.assertEqual(decoded["summary"]["total"], 1)
        self.assertEqual(decoded["summary"]["failed"], 0)
        self.assertEqual(decoded["cases"][0]["name"], "suggest_finance_subject")

    def test_empty_fixture_dir_raises_instead_of_green_zero_case_report(self) -> None:
        with tempfile.TemporaryDirectory() as fixture_dir:
            with tempfile.TemporaryDirectory() as workspace_dir:
                with self.assertRaises(ValueError) as raised:
                    run_smoke_suite(
                        fixture_dir=Path(fixture_dir),
                        workspace_root=Path(workspace_dir),
                        mode="preview",
                    )

        self.assertIn("no subject runtime smoke cases", str(raised.exception))

    def test_unknown_selected_case_still_raises_clear_error(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            with self.assertRaises(ValueError) as raised:
                run_smoke_suite(
                    fixture_dir=FIXTURE_DIR,
                    workspace_root=Path(tmp_dir),
                    mode="preview",
                    selected_cases=["missing_case"],
                )

        self.assertIn("unknown smoke case(s): missing_case", str(raised.exception))

    def test_local_agent_mode_requires_environment_opt_in(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            with mock.patch.dict("os.environ", {}, clear=True):
                with self.assertRaises(RuntimeError) as raised:
                    run_smoke_suite(
                        fixture_dir=FIXTURE_DIR,
                        workspace_root=Path(tmp_dir),
                        mode="local-agent",
                        selected_cases=["suggest_finance_subject"],
                    )

        self.assertIn("QIONGLI_SMOKE_RUN_AGENTS=1", str(raised.exception))

    def test_non_no_subject_refinement_requires_packet_v2_fields(self) -> None:
        case = SmokeCase(
            name="suggest_finance_subject",
            manifest=None,
            args={},
            expected={
                "run_agents": False,
                "decision": "suggest_subject",
                "primary_subject": "finance",
                "effective_domain": "finance",
                "resource_levels": [],
            },
            source=Path("suggest_finance_subject.json"),
        )
        result = {
            "structuredContent": {
                "run_agents": False,
                "data": {
                    "task_run_preview": {
                        "effective_domain": "finance",
                        "subject_refinement": {
                            "decision": "suggest_subject",
                            "primary_subject": "finance",
                            "loaded_resources": {"levels": []},
                        },
                    }
                },
            }
        }

        failures = _assert_case(case, result)

        self.assertIn("missing signals ledger", failures)
        self.assertIn("missing resource_activation_plan", failures)

    def test_suggest_finance_case_exposes_packet_v2_fields(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            report = run_smoke_suite(
                fixture_dir=FIXTURE_DIR,
                workspace_root=Path(tmp_dir),
                mode="preview",
                selected_cases=["suggest_finance_subject"],
            )

        refinement = report["cases"][0]["result"]["data"]["task_run_preview"][
            "subject_refinement"
        ]
        self.assertTrue(refinement["signals"])
        self.assertEqual(refinement["resource_activation_plan"]["primary_subject"], "finance")


if __name__ == "__main__":
    unittest.main()
