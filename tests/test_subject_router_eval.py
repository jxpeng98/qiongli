from __future__ import annotations

import contextlib
import io
import json
import tempfile
import unittest
from dataclasses import replace
from pathlib import Path
from typing import Any, Mapping
from unittest.mock import patch

from tooling.scripts.evaluate_subject_router import (
    DEFAULT_THRESHOLDS,
    EvalCase,
    _actual_eval_result,
    _metrics,
    evaluate_cases,
    load_eval_cases,
    main,
    subject_gate_report,
    threshold_failures,
)
from qiongli.bridges.subject_contracts import RuntimeSubjectContract
from qiongli.bridges.project_manifest import ProjectManifest


FIXTURE_DIR = Path("tests/fixtures/subject_router_eval")


def _finance_contract(
    *,
    activation_status: str = "runtime_enabled",
    source: str | Path | None = None,
    domain_profile: str = "content/skills/domain-profiles/finance.yaml",
    overlay: str = "overlays/finance.yaml",
    subject_skill: str = "skills/finance/SKILL.md",
    evaluation_pack: str = "tests/fixtures/subject_router_eval",
    method_lenses: Mapping[str, Mapping[str, Any]] | None = None,
    required_metrics: Mapping[str, float] | None = None,
) -> RuntimeSubjectContract:
    return RuntimeSubjectContract(
        subject="finance",
        display_name="Finance",
        activation_status=activation_status,
        extends="core",
        source=str(
            source or Path("content/subjects/finance/runtime-subject.yaml").resolve()
        ),
        domain_profile=domain_profile,
        overlay=overlay,
        subject_skill=subject_skill,
        signal_groups={"method": [], "data_or_outcome": [], "venue": []},
        method_lenses={
            key: dict(value)
            for key, value in (
                method_lenses
                or {
                    "event-study": {
                        "resource": "method-packs/finance/event-study.yaml",
                        "activation": "method_only",
                    }
                }
            ).items()
        },
        evaluation_pack=evaluation_pack,
        near_miss_policy={"forbidden_subjects": ["economics"]},
        activation_gate={
            "required_metrics": dict(
                required_metrics
                or {
                    "primary_subject_accuracy": 0.90,
                    "suggest_subject_precision": 0.85,
                    "near_miss_false_positives": 0,
                }
            )
        },
    )


def _finance_precision_cases() -> list[EvalCase]:
    fixtures = {case.id: case for case in load_eval_cases(FIXTURE_DIR)}
    base = fixtures["clear_finance"]
    cases: list[EvalCase] = []
    for index in range(10):
        expected = dict(base.expected)
        if index == 9:
            expected["suggest_subjects"] = []
        tags = ["finance"]
        if index == 0:
            tags.append("clear_positive")
        if index == 1:
            tags.append("method_only_borrow")
        if index == 2:
            tags.append("near_miss")
        cases.append(
            replace(
                base,
                id=f"finance_precision_{index}",
                expected=expected,
                subject_under_test="finance",
                tags=tags,
            )
        )
    return cases


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
        self.assertTrue(all(isinstance(case.tags, list) for case in cases))
        self.assertTrue(any(case.subject_under_test == "finance" for case in cases))

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

    def test_near_miss_false_positives_counts_tagged_cases(self) -> None:
        metrics = _metrics(
            [
                {
                    "id": "accounting_near_miss_budget",
                    "tags": ["accounting", "near_miss"],
                    "expected": {
                        "suggest_subjects": [],
                        "allowed_neighbor_subjects": [],
                    },
                    "actual": {
                        "suggest_subjects": ["accounting"],
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

        self.assertEqual(metrics["near_miss_false_positives"], 1)

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
            (nested / "near_miss_budget.json").write_text(
                json.dumps(payload),
                encoding="utf-8",
            )

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

    def test_subject_gate_report_uses_subject_scoped_threshold_failures(self) -> None:
        fixtures = {case.id: case for case in load_eval_cases(FIXTURE_DIR)}
        target = fixtures["finance_method_only_borrow"]
        target_expected = dict(target.expected)
        target_expected["allowed_neighbor_subjects"] = []
        target_case = replace(
            target,
            id="finance_extra_neighbor_without_gate_allowance",
            expected=target_expected,
            subject_under_test="finance",
            tags=["finance", "clear_positive", "method_only_borrow"],
        )
        near_miss_case = replace(
            fixtures["near_miss_finance"],
            id="finance_gate_near_miss",
            subject_under_test="finance",
            tags=["finance", "near_miss"],
        )
        unrelated_cases = [
            replace(
                fixtures["clear_economics"],
                id=f"unrelated_clear_economics_{index}",
                subject_under_test="economics",
                tags=["economics", "clear_positive"],
            )
            for index in range(6)
        ]
        cases = [target_case, near_miss_case, *unrelated_cases]

        self.assertEqual(evaluate_cases(cases)["threshold_failures"], [])

        report = subject_gate_report("finance", cases)

        self.assertEqual(report["case_count"], 2)
        self.assertEqual(report["metrics"]["suggest_subject_precision"], 0.5)
        self.assertIn(
            "threshold failure: suggest_subject_precision",
            report["blocking_failures"],
        )
        self.assertFalse(report["eligible_for_runtime_enabled"])

    def test_subject_gate_report_uses_contract_required_metric_thresholds(self) -> None:
        cases = _finance_precision_cases()

        with patch(
            "tooling.scripts.evaluate_subject_router.load_runtime_subject_contracts",
            return_value={
                "finance": _finance_contract(
                    required_metrics={"suggest_subject_precision": 0.95},
                )
            },
        ):
            report = subject_gate_report("finance", cases)

        self.assertEqual(report["metrics"]["suggest_subject_precision"], 0.9)
        self.assertIn(
            "threshold failure: suggest_subject_precision",
            report["blocking_failures"],
        )
        self.assertNotIn(
            "threshold failure: near_miss_false_positives",
            report["blocking_failures"],
        )
        self.assertFalse(report["eligible_for_runtime_enabled"])

    def test_subject_gate_report_blocks_runtime_enabled_missing_resources(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            source = Path(tmp_dir) / "subjects" / "finance" / "runtime-subject.yaml"
            source.parent.mkdir(parents=True)
            contract = _finance_contract(
                source=source,
                domain_profile="missing/domain-profile.yaml",
                overlay="missing/overlay.yaml",
                subject_skill="missing/SKILL.md",
                evaluation_pack="missing/eval-fixtures",
                method_lenses={
                    "event-study": {
                        "resource": "missing/event-study.yaml",
                        "activation": "method_only",
                    }
                },
                required_metrics={"suggest_subject_precision": 0.0},
            )

            with patch(
                "tooling.scripts.evaluate_subject_router.load_runtime_subject_contracts",
                return_value={"finance": contract},
            ):
                report = subject_gate_report("finance", _finance_precision_cases())

        self.assertIn(
            "missing resource: domain_profile missing/domain-profile.yaml",
            report["blocking_failures"],
        )
        self.assertIn(
            "missing resource: overlay missing/overlay.yaml",
            report["blocking_failures"],
        )
        self.assertIn(
            "missing resource: subject_skill missing/SKILL.md",
            report["blocking_failures"],
        )
        self.assertIn(
            "missing resource: evaluation_pack missing/eval-fixtures",
            report["blocking_failures"],
        )
        self.assertIn(
            "missing resource: method_lenses[event-study].resource missing/event-study.yaml",
            report["blocking_failures"],
        )
        self.assertFalse(report["eligible_for_runtime_enabled"])

    def test_main_subject_gate_json_returns_one_for_candidate_subject(self) -> None:
        stdout = io.StringIO()

        with contextlib.redirect_stdout(stdout):
            exit_code = main(
                ["--subject", "accounting", "--gate", "runtime-enabled", "--json"],
            )

        report = json.loads(stdout.getvalue())
        self.assertEqual(exit_code, 1)
        self.assertEqual(report["subject_gate"]["subject"], "accounting")
        self.assertFalse(report["subject_gate"]["eligible_for_runtime_enabled"])

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
