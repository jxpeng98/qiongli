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
    signal_groups: Mapping[str, list[Mapping[str, Any]]] | None = None,
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
        signal_groups={
            key: [dict(item) for item in value]
            for key, value in (
                signal_groups or {"method": [], "data_or_outcome": [], "venue": []}
            ).items()
        },
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


def _accounting_contract(
    *,
    activation_status: str = "eval_ready",
    source: str | Path | None = None,
    evaluation_pack: str = "tests/fixtures/subject_router_eval/accounting",
    overlay: str = "",
    subject_skill: str = (
        "content/subjects/accounting/skills/accounting-measurement-auditor.md"
    ),
    signal_groups: Mapping[str, list[Mapping[str, Any]]] | None = None,
    method_lenses: Mapping[str, Mapping[str, Any]] | None = None,
    required_metrics: Mapping[str, float] | None = None,
) -> RuntimeSubjectContract:
    return RuntimeSubjectContract(
        subject="accounting",
        display_name="Accounting",
        activation_status=activation_status,
        extends="core",
        source=str(
            source or Path("content/subjects/accounting/runtime-subject.yaml").resolve()
        ),
        domain_profile="content/skills/domain-profiles/accounting.yaml",
        overlay=overlay,
        subject_skill=subject_skill,
        signal_groups={
            key: [dict(item) for item in value]
            for key, value in (
                signal_groups
                or {
                    "method": [{"id": "accounting.method.accrual-quality"}],
                    "data_or_outcome": [{"id": "accounting.data.audit-analytics"}],
                    "venue": [{"id": "accounting.venue.accounting-review"}],
                    "theory_or_construct": [
                        {"id": "accounting.construct.reporting-quality"}
                    ],
                }
            ).items()
        },
        method_lenses={
            key: dict(value)
            for key, value in (
                method_lenses
                or {
                    "accrual-quality": {
                        "resource": "content/subjects/accounting/skills/accounting-measurement-auditor.md",
                        "activation": "method_only",
                    }
                }
            ).items()
        },
        evaluation_pack=evaluation_pack,
        near_miss_policy={"forbidden_subjects": ["finance", "economics"]},
        activation_gate={
            "required_metrics": dict(
                required_metrics
                or {
                    "primary_subject_accuracy": 0.95,
                    "suggest_subject_precision": 0.95,
                    "near_miss_false_positives": 0,
                }
            )
        },
    )


def _business_contract(
    *,
    activation_status: str = "candidate",
    source: str | Path | None = None,
    evaluation_pack: str = "",
    signal_groups: Mapping[str, list[Mapping[str, Any]]] | None = None,
) -> RuntimeSubjectContract:
    return RuntimeSubjectContract(
        subject="business",
        display_name="Business",
        activation_status=activation_status,
        extends="core",
        source=str(
            source or Path("content/subjects/business/runtime-subject.yaml").resolve()
        ),
        domain_profile="content/skills/domain-profiles/business-management.yaml",
        overlay="",
        subject_skill="",
        signal_groups={
            key: [dict(item) for item in value]
            for key, value in (
                signal_groups
                or {
                    "method": [],
                    "data_or_outcome": [],
                    "venue": [],
                    "theory_or_construct": [],
                }
            ).items()
        },
        method_lenses={},
        evaluation_pack=evaluation_pack,
        near_miss_policy={"forbidden_subjects": ["finance", "economics"]},
        activation_gate={
            "required_metrics": {
                "primary_subject_accuracy": 0.95,
                "suggest_subject_precision": 0.95,
                "near_miss_false_positives": 0,
            }
        },
    )


def _successful_eval_report() -> dict[str, Any]:
    return {
        "case_count": 3,
        "metrics": {
            "decision_accuracy": 1.0,
            "primary_subject_accuracy": 1.0,
            "suggest_subject_precision": 1.0,
            "near_miss_false_positives": 0,
            "forbidden_subject_accuracy": 1.0,
            "method_lens_accuracy": 1.0,
            "all_case_checks_passed": 1.0,
        },
        "cases": [],
        "threshold_failures": [],
    }


def _gate_case(case_id: str, tags: list[str]) -> EvalCase:
    return EvalCase(
        id=case_id,
        description=case_id,
        request="accounting fixture",
        manifest={
            "active_subject": "auto",
            "subject_mode": "auto",
            "secondary_subjects": [],
            "venue_profiles": [],
            "method_lenses": [],
            "strictness": "standard",
        },
        expected={
            "decision": "recommend",
            "primary_subject": "auto",
            "suggest_subjects": [],
            "forbidden_subjects": [],
            "method_lenses": ["accrual-quality"],
        },
        source=f"tests/fixtures/subject_router_eval/accounting/{case_id}.json",
        subject_under_test="accounting",
        tags=["accounting", *tags],
    )


def _subject_gate_case(subject: str, case_id: str, tags: list[str]) -> EvalCase:
    base = _gate_case(case_id, tags)
    return replace(
        base,
        id=case_id,
        description=case_id,
        source=f"tests/fixtures/subject_router_eval/{subject}/{case_id}.json",
        subject_under_test=subject,
        tags=[subject, *tags],
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
        cases_by_id = {case.id: case for case in cases}
        self.assertTrue(
            {
                "clear_economics",
                "clear_finance",
                "economics_method_only_borrow",
                "finance_method_only_borrow",
                "locked_subject_neighbor_lens",
                "mixed_econ_finance",
                "near_miss_economics_workflow",
                "near_miss_finance",
                "weak_core_only",
            }.issubset(set(ids))
        )
        required_accounting_ids = {
            "accounting_clear_discretionary_accruals",
            "accounting_method_only_auto_accrual_controls",
            "accounting_method_only_borrow_accrual_quality",
            "accounting_mixed_reporting_returns",
            "accounting_near_miss_account_for_heterogeneity",
            "accounting_near_miss_bookkeeping_budget",
            "accounting_locked_finance_borrow_measurement",
            "accounting_confirmed_construct_audit",
            "accounting_near_miss_financial_reporting_operations",
            "accounting_near_miss_management_forecast_staffing",
        }
        self.assertTrue(required_accounting_ids.issubset(set(ids)))
        accounting_tags = {
            tag
            for case_id in required_accounting_ids
            for tag in list(cases_by_id[case_id].tags or [])
        }
        self.assertTrue(
            {
                "clear_positive",
                "method_only_borrow",
                "mixed_subject",
                "near_miss",
                "locked_subject",
                "confirmed_subject",
            }.issubset(accounting_tags)
        )
        for case_id in {
            "accounting_clear_discretionary_accruals",
            "accounting_mixed_reporting_returns",
        }:
            self.assertEqual(
                cases_by_id[case_id].expected["primary_subject"],
                "accounting",
            )
            self.assertEqual(
                cases_by_id[case_id].expected["suggest_subjects"],
                ["accounting"],
            )
            self.assertEqual(cases_by_id[case_id].expected["forbidden_subjects"], [])
            self.assertEqual(
                cases_by_id[case_id].expected_for_gate("eval-ready")[
                    "forbidden_subjects"
                ],
                [],
            )
            self.assertEqual(
                cases_by_id[case_id].expected_for_gate("runtime-enabled")[
                    "forbidden_subjects"
                ],
                [],
            )
        self.assertTrue(
            {"accounting", "finance", "economics"}.issubset(
                set(
                    cases_by_id[
                        "accounting_near_miss_financial_reporting_operations"
                    ].expected.get("forbidden_subjects", [])
                )
            )
        )
        self.assertGreaterEqual(len(cases), 15)
        self.assertTrue(all(isinstance(case, EvalCase) for case in cases))
        self.assertTrue(all(case.source.endswith(".json") for case in cases))
        self.assertTrue(all(isinstance(case.tags, list) for case in cases))
        self.assertTrue(any(case.subject_under_test == "finance" for case in cases))

    def test_evaluate_cases_reports_required_metrics_and_cases(self) -> None:
        cases = load_eval_cases(FIXTURE_DIR)

        report = evaluate_cases(cases)

        self.assertEqual(report["case_count"], len(cases))
        self.assertEqual(len(report["cases"]), len(cases))
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

    def test_load_eval_cases_rejects_malformed_gate_expected_entries(self) -> None:
        payload = {
            "id": "malformed_gate_expected",
            "description": "malformed gate-specific expectation",
            "request": "Design an archival accounting study.",
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
                "primary_subject": "auto",
                "suggest_subjects": [],
                "forbidden_subjects": [],
                "method_lenses": [],
            },
            "gate_expected": {
                "eval-ready": ["not", "a", "mapping"],
            },
        }
        with tempfile.TemporaryDirectory() as tmp_dir:
            fixture_dir = Path(tmp_dir)
            (fixture_dir / "malformed_gate_expected.json").write_text(
                json.dumps(payload),
                encoding="utf-8",
            )

            with self.assertRaises(ValueError) as raised:
                load_eval_cases(fixture_dir)

        message = str(raised.exception)
        self.assertIn("gate_expected", message)
        self.assertIn("malformed_gate_expected", message)
        self.assertIn("eval-ready", message)
        self.assertIn("malformed_gate_expected.json", message)

    def test_load_eval_cases_rejects_unknown_gate_expected_names(self) -> None:
        payload = {
            "id": "unknown_gate_expected",
            "description": "unknown gate-specific expectation",
            "request": "Design an archival accounting study.",
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
                "primary_subject": "auto",
                "suggest_subjects": [],
                "forbidden_subjects": [],
                "method_lenses": [],
            },
            "gate_expected": {
                "eval_ready": {
                    "decision": "recommend",
                    "primary_subject": "accounting",
                    "suggest_subjects": ["accounting"],
                    "forbidden_subjects": [],
                    "method_lenses": [],
                },
            },
        }
        with tempfile.TemporaryDirectory() as tmp_dir:
            fixture_dir = Path(tmp_dir)
            (fixture_dir / "unknown_gate_expected.json").write_text(
                json.dumps(payload),
                encoding="utf-8",
            )

            with self.assertRaises(ValueError) as raised:
                load_eval_cases(fixture_dir)

        message = str(raised.exception)
        self.assertIn("gate_expected", message)
        self.assertIn("eval_ready", message)
        self.assertIn("unknown_gate_expected", message)
        self.assertIn("unknown_gate_expected.json", message)

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

    def test_accounting_runtime_enabled_gate_passes_real_fixture_pack(self) -> None:
        cases = load_eval_cases(FIXTURE_DIR)

        report = subject_gate_report("accounting", cases, gate="runtime-enabled")

        self.assertEqual(report["subject"], "accounting")
        self.assertEqual(report["activation_status"], "runtime_enabled")
        self.assertFalse(report["eligible_for_eval_ready"])
        self.assertTrue(report["eligible_for_runtime_enabled"])
        self.assertEqual(report["blocking_failures"], [])
        self.assertEqual(report["metrics"]["near_miss_false_positives"], 0)

    def test_economics_runtime_enabled_gate_passes_real_fixture_pack(self) -> None:
        cases = load_eval_cases(FIXTURE_DIR)

        report = subject_gate_report("economics", cases, gate="runtime-enabled")

        self.assertEqual(report["activation_status"], "runtime_enabled")
        self.assertFalse(report["eligible_for_eval_ready"])
        self.assertTrue(report["eligible_for_runtime_enabled"])
        self.assertEqual(report["blocking_failures"], [])
        self.assertEqual(report["metrics"]["decision_accuracy"], 1.0)
        self.assertEqual(report["metrics"]["primary_subject_accuracy"], 1.0)
        self.assertEqual(report["metrics"]["suggest_subject_precision"], 1.0)
        self.assertEqual(report["metrics"]["forbidden_subject_accuracy"], 1.0)
        self.assertEqual(report["metrics"]["method_lens_accuracy"], 1.0)
        self.assertEqual(report["metrics"]["all_case_checks_passed"], 1.0)
        self.assertEqual(report["metrics"]["near_miss_false_positives"], 0)

    def test_candidate_subject_eval_ready_gate_reports_deferred_shell_reasons(self) -> None:
        cases = load_eval_cases(FIXTURE_DIR)
        deferred_subjects = (
            "business",
            "political-economy",
            "geoeconomics",
            "economics-accounting",
        )

        for subject in deferred_subjects:
            with self.subTest(subject=subject):
                report = subject_gate_report(subject, cases, gate="eval-ready")

                self.assertEqual(report["subject"], subject)
                self.assertEqual(report["activation_status"], "candidate")
                self.assertFalse(report["eligible_for_eval_ready"])
                self.assertFalse(report["eligible_for_runtime_enabled"])
                self.assertEqual(report["case_count"], 0)
                self.assertIn(
                    "activation_status is candidate",
                    report["blocking_failures"],
                )
                self.assertIn(
                    "missing evaluation_pack for deferred subject",
                    report["blocking_failures"],
                )
                for dimension in (
                    "method",
                    "data_or_outcome",
                    "venue",
                    "theory_or_construct",
                ):
                    self.assertIn(
                        f"missing signal dimension: {dimension}",
                        report["blocking_failures"],
                    )
                for tag in (
                    "clear_positive",
                    "method_only_borrow",
                    "near_miss",
                ):
                    self.assertIn(
                        f"missing {tag} fixtures",
                        report["blocking_failures"],
                    )

    def test_eval_ready_gate_reports_subject_specific_pack_mismatch(self) -> None:
        cases = [
            _subject_gate_case("business", "business_clear", ["clear_positive"]),
            _subject_gate_case(
                "business",
                "business_method",
                ["method_only_borrow"],
            ),
            _subject_gate_case("business", "business_near_miss", ["near_miss"]),
        ]

        with patch(
            "tooling.scripts.evaluate_subject_router.load_runtime_subject_contracts",
            return_value={
                "business": _business_contract(
                    activation_status="eval_ready",
                    evaluation_pack="tests/fixtures/subject_router_eval/accounting",
                    signal_groups={
                        "method": [{"id": "business.method.case-study"}],
                        "data_or_outcome": [
                            {"id": "business.data.organization-panel"}
                        ],
                        "venue": [{"id": "business.venue.amj"}],
                        "theory_or_construct": [
                            {"id": "business.construct.capability"}
                        ],
                    },
                )
            },
        ), patch(
            "tooling.scripts.evaluate_subject_router.evaluate_cases",
            return_value=_successful_eval_report(),
        ):
            report = subject_gate_report("business", cases, gate="eval-ready")

        self.assertFalse(report["eligible_for_eval_ready"])
        self.assertIn(
            "evaluation_pack subject mismatch: expected business, found accounting",
            report["blocking_failures"],
        )

    def test_eval_ready_gate_accepts_eval_ready_subject_without_runtime_activation(self) -> None:
        cases = [
            _gate_case("accounting_clear", ["clear_positive"]),
            _gate_case("accounting_method", ["method_only_borrow"]),
            _gate_case("accounting_near_miss", ["near_miss"]),
        ]

        with patch(
            "tooling.scripts.evaluate_subject_router.load_runtime_subject_contracts",
            return_value={
                "accounting": _accounting_contract(
                    evaluation_pack="tests/fixtures/subject_router_eval"
                )
            },
        ), patch(
            "tooling.scripts.evaluate_subject_router.evaluate_cases",
            return_value=_successful_eval_report(),
        ):
            report = subject_gate_report("accounting", cases, gate="eval-ready")

        self.assertEqual(report["subject"], "accounting")
        self.assertEqual(report["activation_status"], "eval_ready")
        self.assertTrue(report["eligible_for_eval_ready"])
        self.assertFalse(report["eligible_for_runtime_enabled"])
        self.assertEqual(report["blocking_failures"], [])

    def test_eval_ready_gate_allows_blank_optional_subject_resources(self) -> None:
        cases = [
            _gate_case("accounting_clear", ["clear_positive"]),
            _gate_case("accounting_method", ["method_only_borrow"]),
            _gate_case("accounting_near_miss", ["near_miss"]),
        ]

        with patch(
            "tooling.scripts.evaluate_subject_router.load_runtime_subject_contracts",
            return_value={
                "accounting": _accounting_contract(
                    evaluation_pack="tests/fixtures/subject_router_eval",
                    overlay="",
                    subject_skill="",
                )
            },
        ), patch(
            "tooling.scripts.evaluate_subject_router.evaluate_cases",
            return_value=_successful_eval_report(),
        ):
            report = subject_gate_report("accounting", cases, gate="eval-ready")

        self.assertTrue(report["eligible_for_eval_ready"])
        self.assertEqual(report["blocking_failures"], [])

    def test_eval_ready_gate_rejects_runtime_enabled_subject(self) -> None:
        cases = [
            _gate_case("accounting_clear", ["clear_positive"]),
            _gate_case("accounting_method", ["method_only_borrow"]),
            _gate_case("accounting_near_miss", ["near_miss"]),
        ]

        with patch(
            "tooling.scripts.evaluate_subject_router.load_runtime_subject_contracts",
            return_value={
                "accounting": _accounting_contract(
                    activation_status="runtime_enabled"
                )
            },
        ), patch(
            "tooling.scripts.evaluate_subject_router.evaluate_cases",
            return_value=_successful_eval_report(),
        ):
            report = subject_gate_report("accounting", cases, gate="eval-ready")

        self.assertFalse(report["eligible_for_eval_ready"])
        self.assertIn(
            "activation_status is runtime_enabled",
            report["blocking_failures"],
        )

    def test_eval_ready_gate_reports_missing_evaluation_pack(self) -> None:
        cases = [
            _gate_case("accounting_clear", ["clear_positive"]),
            _gate_case("accounting_method", ["method_only_borrow"]),
            _gate_case("accounting_near_miss", ["near_miss"]),
        ]

        with patch(
            "tooling.scripts.evaluate_subject_router.load_runtime_subject_contracts",
            return_value={
                "accounting": _accounting_contract(
                    evaluation_pack="missing/accounting-eval-fixtures"
                )
            },
        ), patch(
            "tooling.scripts.evaluate_subject_router.evaluate_cases",
            return_value=_successful_eval_report(),
        ):
            report = subject_gate_report("accounting", cases, gate="eval-ready")

        self.assertFalse(report["eligible_for_eval_ready"])
        self.assertIn(
            "missing resource: evaluation_pack missing/accounting-eval-fixtures",
            report["blocking_failures"],
        )

    def test_eval_ready_signal_dimension_requirements_are_subject_scoped(self) -> None:
        cases = [
            replace(
                _gate_case("finance_clear", ["clear_positive"]),
                subject_under_test="finance",
                tags=["finance", "clear_positive"],
            ),
            replace(
                _gate_case("finance_method", ["method_only_borrow"]),
                subject_under_test="finance",
                tags=["finance", "method_only_borrow"],
            ),
            replace(
                _gate_case("finance_near_miss", ["near_miss"]),
                subject_under_test="finance",
                tags=["finance", "near_miss"],
            ),
        ]

        with patch(
            "tooling.scripts.evaluate_subject_router.load_runtime_subject_contracts",
            return_value={
                "finance": _finance_contract(
                    activation_status="eval_ready",
                    signal_groups={
                        "method": [{"id": "finance.method.event-study"}],
                        "data_or_outcome": [],
                        "venue": [],
                    },
                )
            },
        ), patch(
            "tooling.scripts.evaluate_subject_router.evaluate_cases",
            return_value=_successful_eval_report(),
        ):
            report = subject_gate_report("finance", cases, gate="eval-ready")

        self.assertTrue(report["eligible_for_eval_ready"])
        self.assertEqual(
            [
                failure
                for failure in report["blocking_failures"]
                if failure.startswith("missing signal dimension:")
            ],
            [],
        )

    def test_runtime_enabled_gate_still_blocks_eval_ready_subject_contract(self) -> None:
        cases = [
            _gate_case("accounting_clear", ["clear_positive"]),
            _gate_case("accounting_method", ["method_only_borrow"]),
            _gate_case("accounting_near_miss", ["near_miss"]),
        ]

        with patch(
            "tooling.scripts.evaluate_subject_router.load_runtime_subject_contracts",
            return_value={
                "accounting": _accounting_contract(
                    activation_status="eval_ready"
                )
            },
        ), patch(
            "tooling.scripts.evaluate_subject_router.evaluate_cases",
            return_value=_successful_eval_report(),
        ):
            report = subject_gate_report("accounting", cases, gate="runtime-enabled")

        self.assertFalse(report["eligible_for_eval_ready"])
        self.assertFalse(report["eligible_for_runtime_enabled"])
        self.assertIn("activation_status is eval_ready", report["blocking_failures"])

    def test_gate_specific_expected_overrides_default_expected(self) -> None:
        case = EvalCase(
            id="accounting_gate_specific",
            description="gate-specific accounting expectation",
            request="Design an archival accounting study of discretionary accruals.",
            manifest={
                "active_subject": "auto",
                "subject_mode": "auto",
                "secondary_subjects": [],
                "venue_profiles": [],
                "method_lenses": [],
                "strictness": "standard",
            },
            expected={
                "decision": "recommend",
                "primary_subject": "auto",
                "suggest_subjects": [],
                "forbidden_subjects": [],
                "method_lenses": ["accrual-quality"],
            },
            source="inline.json",
            subject_under_test="accounting",
            tags=["accounting", "clear_positive"],
            gate_expected={
                "eval-ready": {
                    "decision": "recommend",
                    "primary_subject": "accounting",
                    "suggest_subjects": ["accounting"],
                    "forbidden_subjects": [],
                    "method_lenses": ["accrual-quality"],
                }
            },
        )

        self.assertEqual(
            case.expected_for_gate("")["primary_subject"],
            "auto",
        )
        self.assertEqual(
            case.expected_for_gate("eval-ready")["primary_subject"],
            "accounting",
        )
        malformed_case = replace(
            case,
            gate_expected={"eval-ready": ["not", "a", "mapping"]},
        )
        self.assertEqual(
            malformed_case.expected_for_gate("eval-ready")["primary_subject"],
            "auto",
        )

    def test_evaluate_cases_passes_eval_subjects_and_uses_gate_expected(self) -> None:
        case = EvalCase(
            id="accounting_eval_plumbing",
            description="eval plumbing case",
            request="Design an accounting accrual-quality study.",
            manifest={
                "active_subject": "auto",
                "subject_mode": "auto",
                "secondary_subjects": [],
                "venue_profiles": [],
                "method_lenses": [],
                "strictness": "standard",
            },
            expected={
                "decision": "recommend",
                "primary_subject": "auto",
                "suggest_subjects": [],
                "forbidden_subjects": [],
                "method_lenses": [],
            },
            source="inline.json",
            subject_under_test="accounting",
            tags=["accounting", "clear_positive"],
            gate_expected={
                "eval-ready": {
                    "decision": "recommend",
                    "primary_subject": "accounting",
                    "suggest_subjects": ["accounting"],
                    "forbidden_subjects": [],
                    "method_lenses": ["accrual-quality"],
                }
            },
        )
        captured: dict[str, Any] = {}

        class FakePacket:
            def to_packet(self) -> dict[str, Any]:
                return {
                    "decision": "suggest_subject",
                    "primary_subject": "accounting",
                    "candidate_subjects": [{"subject": "accounting"}],
                    "method_lenses": ["accrual-quality"],
                    "borrowed_lenses": [],
                }

        def fake_infer(
            task_packet: Mapping[str, Any],
            *,
            manifest_state: ProjectManifest,
            evaluation_subjects: set[str] | None = None,
        ) -> FakePacket:
            captured["task_packet"] = task_packet
            captured["manifest_state"] = manifest_state
            captured["evaluation_subjects"] = evaluation_subjects
            return FakePacket()

        with patch(
            "tooling.scripts.evaluate_subject_router._infer_subject_refinement",
            side_effect=fake_infer,
        ):
            report = evaluate_cases(
                [case],
                gate="eval-ready",
                evaluation_subjects={"accounting"},
            )

        self.assertEqual(captured["evaluation_subjects"], {"accounting"})
        self.assertEqual(
            report["cases"][0]["expected"]["primary_subject"],
            "accounting",
        )
        self.assertEqual(report["threshold_failures"], [])

    def test_main_subject_eval_ready_gate_uses_eval_ready_eligibility(self) -> None:
        stdout = io.StringIO()

        with patch(
            "tooling.scripts.evaluate_subject_router.load_runtime_subject_contracts",
            return_value={
                "accounting": _accounting_contract(
                    evaluation_pack="tests/fixtures/subject_router_eval"
                )
            },
        ), patch(
            "tooling.scripts.evaluate_subject_router.evaluate_cases",
            return_value=_successful_eval_report(),
        ), patch(
            "tooling.scripts.evaluate_subject_router.load_eval_cases",
            return_value=[
                _gate_case("accounting_clear", ["clear_positive"]),
                _gate_case("accounting_method", ["method_only_borrow"]),
                _gate_case("accounting_near_miss", ["near_miss"]),
            ],
        ), contextlib.redirect_stdout(stdout):
            exit_code = main(
                ["--subject", "accounting", "--gate", "eval-ready", "--json"]
            )

        report = json.loads(stdout.getvalue())
        self.assertEqual(exit_code, 0)
        self.assertTrue(report["subject_gate"]["eligible_for_eval_ready"])

    def test_main_subject_gate_exit_ignores_global_threshold_failures(self) -> None:
        stdout = io.StringIO()
        cases = [
            _gate_case("accounting_clear", ["clear_positive"]),
            _gate_case("accounting_method", ["method_only_borrow"]),
            _gate_case("accounting_near_miss", ["near_miss"]),
        ]
        calls: list[dict[str, Any]] = []

        def fake_evaluate(
            selected_cases: list[EvalCase],
            thresholds: Mapping[str, float] = DEFAULT_THRESHOLDS,
            *,
            gate: str = "",
            evaluation_subjects: list[str] | None = None,
        ) -> dict[str, Any]:
            calls.append(
                {
                    "case_count": len(selected_cases),
                    "thresholds": thresholds,
                    "gate": gate,
                    "evaluation_subjects": evaluation_subjects,
                }
            )
            if len(calls) == 1:
                report = _successful_eval_report()
                report["threshold_failures"] = [
                    {"metric": "global_fixture_failure"},
                ]
                return report
            return _successful_eval_report()

        with patch(
            "tooling.scripts.evaluate_subject_router.load_runtime_subject_contracts",
            return_value={
                "accounting": _accounting_contract(
                    evaluation_pack="tests/fixtures/subject_router_eval"
                )
            },
        ), patch(
            "tooling.scripts.evaluate_subject_router.load_eval_cases",
            return_value=cases,
        ), patch(
            "tooling.scripts.evaluate_subject_router.evaluate_cases",
            side_effect=fake_evaluate,
        ), contextlib.redirect_stdout(stdout):
            exit_code = main(
                ["--subject", "accounting", "--gate", "eval-ready", "--json"]
            )

        report = json.loads(stdout.getvalue())
        self.assertEqual(exit_code, 0)
        self.assertEqual(
            report["threshold_failures"],
            [{"metric": "global_fixture_failure"}],
        )
        self.assertIsNone(calls[0]["evaluation_subjects"])
        self.assertEqual(calls[1]["evaluation_subjects"], ["accounting"])

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

    def test_main_subject_runtime_gate_json_returns_zero_for_accounting(self) -> None:
        stdout = io.StringIO()

        with contextlib.redirect_stdout(stdout):
            exit_code = main(
                ["--subject", "accounting", "--gate", "runtime-enabled", "--json"],
            )

        report = json.loads(stdout.getvalue())
        self.assertEqual(exit_code, 0)
        self.assertEqual(report["subject_gate"]["subject"], "accounting")
        self.assertEqual(report["subject_gate"]["activation_status"], "runtime_enabled")
        self.assertFalse(report["subject_gate"]["eligible_for_eval_ready"])
        self.assertTrue(report["subject_gate"]["eligible_for_runtime_enabled"])
        self.assertEqual(report["subject_gate"]["blocking_failures"], [])

    def test_main_json_returns_zero_for_current_fixture_corpus(self) -> None:
        stdout = io.StringIO()

        with contextlib.redirect_stdout(stdout):
            exit_code = main(["--json"])

        report = json.loads(stdout.getvalue())
        self.assertEqual(exit_code, 0)
        self.assertGreaterEqual(report["case_count"], 15)
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
