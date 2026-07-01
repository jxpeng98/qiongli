from __future__ import annotations

import contextlib
import io
import json
import tempfile
import unittest
from pathlib import Path

from tooling.scripts.evaluate_subject_router import (
    DEFAULT_THRESHOLDS,
    EvalCase,
    _actual_eval_result,
    _metrics,
    evaluate_cases,
    load_eval_cases,
    main,
    threshold_failures,
)
from qiongli.bridges.project_manifest import ProjectManifest


FIXTURE_DIR = Path("tests/fixtures/subject_router_eval")


class SubjectRouterEvalTests(unittest.TestCase):
    def test_load_eval_cases_reads_all_fixtures(self) -> None:
        cases = load_eval_cases(FIXTURE_DIR)

        ids = [case.id for case in cases]
        self.assertEqual(
            ids,
            [
                "clear_economics",
                "clear_finance",
                "economics_method_only_borrow",
                "finance_method_only_borrow",
                "locked_subject_neighbor_lens",
                "mixed_econ_finance",
                "near_miss_finance",
                "weak_core_only",
            ],
        )
        self.assertEqual(len(cases), 8)
        self.assertTrue(all(isinstance(case, EvalCase) for case in cases))
        self.assertTrue(all(case.source.endswith(".json") for case in cases))

    def test_evaluate_cases_reports_required_metrics_and_cases(self) -> None:
        report = evaluate_cases(load_eval_cases(FIXTURE_DIR))

        self.assertEqual(report["case_count"], 8)
        self.assertEqual(len(report["cases"]), 8)
        self.assertEqual(
            set(report["metrics"]),
            {
                "decision_accuracy",
                "forbidden_subject_accuracy",
                "method_lens_accuracy",
                "primary_subject_accuracy",
                "suggest_subject_precision",
                "near_miss_false_positives",
                "all_case_checks_passed",
            },
        )
        for case in report["cases"]:
            self.assertEqual(
                set(case),
                {"id", "description", "source", "expected", "actual", "passed"},
            )
            self.assertEqual(
                set(case["actual"]),
                {"decision", "primary_subject", "suggest_subjects", "method_lenses"},
            )
            self.assertEqual(
                set(case["passed"]),
                {
                    "decision",
                    "primary_subject",
                    "suggest_subjects",
                    "forbidden_subjects",
                    "method_lenses",
                },
            )

    def test_threshold_failures_returns_named_failures(self) -> None:
        metrics = {
            "decision_accuracy": 0.89,
            "primary_subject_accuracy": 0.89,
            "suggest_subject_precision": 0.84,
            "near_miss_false_positives": 1,
            "forbidden_subject_accuracy": 1.0,
            "method_lens_accuracy": 1.0,
            "all_case_checks_passed": 1.0,
        }

        failures = threshold_failures(metrics, DEFAULT_THRESHOLDS)

        self.assertEqual(
            [failure["metric"] for failure in failures],
            [
                "decision_accuracy",
                "primary_subject_accuracy",
                "suggest_subject_precision",
                "near_miss_false_positives",
            ],
        )

    def test_near_miss_false_positives_allows_values_below_threshold(self) -> None:
        failures = threshold_failures(
            {
                "near_miss_false_positives": 1,
            },
            {
                "near_miss_false_positives": 2,
            },
        )

        self.assertEqual(failures, [])

    def test_method_lens_failure_makes_main_return_one(self) -> None:
        payload = {
            "id": "method_lens_failure",
            "description": "method lens expectation intentionally fails",
            "request": (
                "Design a study of abnormal returns using asset pricing controls "
                "and event-study windows."
            ),
            "manifest": {
                "active_subject": "auto",
                "subject_mode": "auto",
                "secondary_subjects": [],
                "venue_profiles": [],
                "method_lenses": [],
                "strictness": "standard",
            },
            "expected": {
                "decision": "recommend",
                "primary_subject": "finance",
                "suggest_subjects": ["finance"],
                "forbidden_subjects": [],
                "method_lenses": ["did"],
            },
        }
        with tempfile.TemporaryDirectory() as tmp_dir:
            fixture_dir = Path(tmp_dir)
            (fixture_dir / "method_lens_failure.json").write_text(
                json.dumps(payload),
                encoding="utf-8",
            )
            stdout = io.StringIO()

            with contextlib.redirect_stdout(stdout):
                exit_code = main(["--fixtures", str(fixture_dir), "--json"])

        report = json.loads(stdout.getvalue())
        self.assertEqual(exit_code, 1)
        self.assertIn(
            "method_lens_accuracy",
            {failure["metric"] for failure in report["threshold_failures"]},
        )

    def test_suggest_subject_precision_penalizes_extra_unaccepted_suggestions(self) -> None:
        metrics = _metrics(
            [
                {
                    "id": "extra_subject",
                    "expected": {
                        "suggest_subjects": ["finance"],
                        "allowed_neighbor_subjects": [],
                    },
                    "actual": {
                        "suggest_subjects": ["finance", "history"],
                    },
                    "passed": {
                        "decision": True,
                        "primary_subject": True,
                        "forbidden_subjects": True,
                        "method_lenses": True,
                        "suggest_subjects": True,
                    },
                }
            ]
        )

        self.assertEqual(metrics["suggest_subject_precision"], 0.5)

    def test_locked_primary_subject_is_not_counted_as_suggestion(self) -> None:
        actual = _actual_eval_result(
            ProjectManifest(
                active_subject="economics",
                subject_mode="locked",
            ).normalized(),
            {
                "decision": "lock_subject",
                "primary_subject": "economics",
                "candidate_subjects": [{"subject": "finance"}],
                "method_lenses": [],
                "borrowed_lenses": [],
            },
        )

        self.assertEqual(actual["suggest_subjects"], ["finance"])

    def test_borrow_lens_decision_ignores_expected_suggestions(self) -> None:
        actual = _actual_eval_result(
            ProjectManifest().normalized(),
            {
                "decision": "borrow_lens",
                "primary_subject": "auto",
                "candidate_subjects": [],
                "method_lenses": [],
                "borrowed_lenses": [],
            },
        )

        self.assertEqual(actual["decision"], "core_only")

    def test_load_eval_cases_rejects_duplicate_ids(self) -> None:
        payload = {
            "id": "duplicate",
            "description": "duplicate case",
            "request": "revise introduction",
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
                "forbidden_subjects": [],
                "method_lenses": [],
            },
        }
        with tempfile.TemporaryDirectory() as tmp_dir:
            fixture_dir = Path(tmp_dir)
            (fixture_dir / "a.json").write_text(json.dumps(payload), encoding="utf-8")
            (fixture_dir / "b.json").write_text(json.dumps(payload), encoding="utf-8")

            with self.assertRaises(ValueError) as raised:
                load_eval_cases(fixture_dir)

        self.assertIn("duplicate fixture id", str(raised.exception))
        self.assertIn("duplicate", str(raised.exception))

    def test_main_json_returns_zero_for_current_fixture_corpus(self) -> None:
        stdout = io.StringIO()

        with contextlib.redirect_stdout(stdout):
            exit_code = main(["--json"])

        report = json.loads(stdout.getvalue())
        self.assertEqual(exit_code, 0)
        self.assertEqual(report["case_count"], 8)
        self.assertEqual(report["threshold_failures"], [])

    def test_main_returns_one_for_failing_fixture_directory(self) -> None:
        payload = {
            "id": "intentionally_failing",
            "description": "fixture expectation intentionally fails",
            "request": "Help me organize a research workflow and folder structure.",
            "manifest": {
                "active_subject": "auto",
                "subject_mode": "auto",
                "secondary_subjects": [],
                "venue_profiles": [],
                "method_lenses": [],
                "strictness": "standard",
            },
            "expected": {
                "decision": "recommend",
                "primary_subject": "finance",
                "suggest_subjects": ["finance"],
                "forbidden_subjects": [],
                "method_lenses": [],
            },
        }
        with tempfile.TemporaryDirectory() as tmp_dir:
            fixture_dir = Path(tmp_dir)
            (fixture_dir / "intentionally_failing.json").write_text(
                json.dumps(payload),
                encoding="utf-8",
            )
            stdout = io.StringIO()

            with contextlib.redirect_stdout(stdout):
                exit_code = main(["--fixture-dir", str(fixture_dir), "--json"])

        report = json.loads(stdout.getvalue())
        self.assertEqual(exit_code, 1)
        self.assertTrue(report["threshold_failures"])


if __name__ == "__main__":
    unittest.main()
