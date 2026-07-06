from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import yaml

from bridges import subject_refinement as subject_refinement_module
from bridges.project_manifest import ProjectManifest
from bridges.subject_contracts import RuntimeSubjectContract
from bridges.subject_refinement import infer_subject_refinement


def _runtime_subject_contract(
    *,
    subject: str = "accounting",
    subject_skill: str = "",
    signal_groups: dict[str, list[dict[str, object]]] | None = None,
    method_lenses: dict[str, dict[str, object]] | None = None,
) -> RuntimeSubjectContract:
    return RuntimeSubjectContract(
        subject=subject,
        display_name=subject.title(),
        activation_status="eval_ready",
        extends="core",
        source="test-runtime-subject.yaml",
        domain_profile="",
        overlay="",
        subject_skill=subject_skill,
        signal_groups=signal_groups or {},
        method_lenses=method_lenses or {},
        evaluation_pack="",
        near_miss_policy={},
        activation_gate={},
    )


def _business_runtime_subject_contract(
    *,
    activation_status: str = "eval_ready",
) -> RuntimeSubjectContract:
    return RuntimeSubjectContract(
        subject="business",
        display_name="Business",
        activation_status=activation_status,
        extends="core",
        source="content/subjects/business/runtime-subject.yaml",
        domain_profile="content/skills/domain-profiles/business-management.yaml",
        overlay="",
        subject_skill="",
        signal_groups={
            "method": [
                {
                    "id": "business.method.case-study",
                    "value": "case-study",
                    "weight": 0.30,
                    "activation": "method_only",
                    "patterns": [r"\bmultiple case study\b"],
                    "method_lenses": [
                        "business-positioning",
                        "qualitative-transparency",
                    ],
                },
                {
                    "id": "business.method.gioia",
                    "value": "gioia-method",
                    "weight": 0.30,
                    "activation": "method_only",
                    "patterns": [r"\bGioia\b", r"\bfirst-order concepts\b"],
                    "method_lenses": ["qualitative-transparency"],
                },
            ],
            "data_or_outcome": [
                {
                    "id": "business.data.qualitative-fieldwork",
                    "value": "qualitative-fieldwork",
                    "weight": 0.25,
                    "activation": "subject",
                    "patterns": [r"\binterviews with managers\b"],
                }
            ],
            "venue": [
                {
                    "id": "business.venue.amj",
                    "value": "academy-of-management-journal",
                    "weight": 0.20,
                    "activation": "context_only",
                    "patterns": [r"\bAcademy of Management Journal\b", r"\bAMJ\b"],
                    "method_lenses": ["business-positioning"],
                }
            ],
            "theory_or_construct": [
                {
                    "id": "business.construct.theory-contribution",
                    "value": "theory-contribution",
                    "weight": 0.25,
                    "activation": "subject",
                    "patterns": [r"\bmanagement theory\b", r"\btheory contribution\b"],
                }
            ],
        },
        method_lenses={
            "business-positioning": {
                "resource": (
                    "content/subjects/business/skills/"
                    "business-journal-positioning-auditor.md"
                ),
                "activation": "method_only",
            },
            "qualitative-transparency": {
                "resource": "content/subjects/business/overlays/skills/study-designer.md",
                "activation": "method_only",
            },
        },
        evaluation_pack="tests/fixtures/subject_router_eval/business",
        near_miss_policy={"forbidden_subjects": ["finance", "economics"]},
        activation_gate={
            "required_metrics": {
                "primary_subject_accuracy": 0.95,
                "suggest_subject_precision": 0.95,
                "near_miss_false_positives": 0,
            }
        },
    )


class SubjectRefinementTests(unittest.TestCase):
    def test_contract_declares_locked_persistence_status(self) -> None:
        contract_path = Path("content/standards/subject-refinement-contract.yaml")
        contract = yaml.safe_load(contract_path.read_text(encoding="utf-8")) or {}

        statuses = contract["persistence_statuses"]
        self.assertIn("locked", statuses)
        self.assertIn("confirmed", statuses["applied"]["description"].lower())
        self.assertNotIn("locked", statuses["applied"]["description"].lower())

    def test_default_contract_lookup_uses_repo_content_from_current_working_dir(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            fake_runtime_file = (
                Path(tmp_dir)
                / "site-packages"
                / "qiongli"
                / "bridges"
                / "subject_refinement.py"
            )
            fake_runtime_file.parent.mkdir(parents=True)
            fake_runtime_file.write_text("# installed package placeholder\n", encoding="utf-8")

            with patch.object(subject_refinement_module, "__file__", str(fake_runtime_file)):
                result = subject_refinement_module._load_contract(None)

        self.assertEqual(result.warnings, [])
        self.assertEqual(result.contract["name"], "subject-refinement-contract")

    def test_finance_event_study_method_signal_borrows_lens_without_subject_switch(self) -> None:
        packet = infer_subject_refinement(
            {"topic": "policy announcement effects", "context": "Use an event study design."},
            manifest_state=ProjectManifest(active_subject="economics"),
        ).to_packet()

        self.assertEqual(packet["decision"], "borrow_lens")
        self.assertEqual(packet["mode"], "auto")
        self.assertEqual(packet["active_subject"], "economics")
        self.assertEqual(packet["primary_subject"], "economics")
        self.assertIsInstance(packet["borrowed_lenses"][0], dict)
        self.assertEqual(packet["borrowed_lenses"][0]["source_subject"], "finance")
        self.assertEqual(packet["borrowed_lenses"][0]["lens"], "event-study")
        self.assertEqual(packet["borrowed_lenses"][0]["resource_level"], "method_pack_only")
        self.assertIn("method-only", packet["borrowed_lenses"][0]["reason"])
        self.assertEqual(packet["loaded_resources"]["levels"], ["method_pack_only"])
        self.assertEqual(packet["persistence"]["status"], "temporary")

    def test_finance_method_data_outcome_and_venue_suggests_finance(self) -> None:
        packet = infer_subject_refinement(
            {
                "topic": "earnings announcement returns",
                "context": "Estimate abnormal returns using CRSP data for Journal of Finance.",
            },
            manifest_state=ProjectManifest(),
            draft_content="Use an event study with event windows around earnings announcements.",
        ).to_packet()

        self.assertEqual(packet["decision"], "suggest_subject")
        self.assertEqual(packet["mode"], "suggested")
        self.assertEqual(packet["active_subject"], "auto")
        self.assertEqual(packet["primary_subject"], "finance")
        self.assertGreaterEqual(packet["confidence"], 0.6)
        self.assertTrue(packet["evidence"])
        self.assertIsInstance(packet["candidate_subjects"][0], dict)
        self.assertEqual(packet["candidate_subjects"][0]["subject"], "finance")
        self.assertIn("confidence", packet["candidate_subjects"][0])
        self.assertTrue(packet["candidate_subjects"][0]["evidence"])
        self.assertEqual(
            packet["candidate_subjects"][0]["matched_dimensions"],
            ["method", "data_or_outcome", "venue"],
        )
        self.assertIn("event-study", packet["candidate_subjects"][0]["method_lenses"])
        self.assertIn("event-study", packet["method_lenses"])
        self.assertEqual(packet["persistence"]["status"], "proposed")
        self.assertIn("overlays/finance.yaml", packet["loaded_resources"]["overlays"])
        self.assertIn("skills/finance/SKILL.md", packet["loaded_resources"]["subject_skills"])
        self.assertEqual(packet["resource_activation_plan"]["primary_subject"], "finance")
        self.assertEqual(
            packet["resource_activation_plan"]["levels"],
            ["core", "subject_overlay", "subject_skill", "method_pack"],
        )
        self.assertIn(
            {
                "kind": "method_pack",
                "subject": "finance",
                "lens": "event-study",
                "path": "method-packs/finance/event-study.yaml",
                "activation": "proposed",
            },
            packet["resource_activation_plan"]["resources"],
        )

    def test_subject_refinement_packet_separates_task_text_and_manifest_sources(self) -> None:
        packet = infer_subject_refinement(
            {
                "topic": "earnings announcement returns",
                "context": "Estimate abnormal returns using CRSP data for Journal of Finance.",
            },
            manifest_state=ProjectManifest(),
            draft_content="Use an event study with event windows around earnings announcements.",
        ).to_packet()

        sources = packet["evidence_sources"]

        self.assertEqual(sources["manifest_state"]["active_subject"], "auto")
        self.assertEqual(sources["manifest_state"]["subject_mode"], "auto")
        self.assertEqual(sources["task_text"]["status"], "present")
        self.assertIn("finance.method.event-study", sources["task_text"]["signal_ids"])
        self.assertIn("trace_memory", sources)
        self.assertIn("user_action", sources)
        self.assertEqual(sources["trace_memory"]["status"], "not_loaded")
        self.assertEqual(sources["user_action"]["status"], "not_loaded")

    def test_candidate_finance_signals_borrow_lens_without_subject_suggestion(self) -> None:
        with patch(
            "bridges.subject_refinement.subject_activation_status",
            return_value="candidate",
        ):
            packet = infer_subject_refinement(
                {
                    "topic": "earnings announcement returns",
                    "context": "Estimate abnormal returns using CRSP data for Journal of Finance.",
                },
                manifest_state=ProjectManifest(),
                draft_content="Use an event study with event windows around earnings announcements.",
            ).to_packet()

        self.assertEqual(packet["decision"], "borrow_lens")
        self.assertEqual(packet["mode"], "auto")
        self.assertEqual(packet["active_subject"], "auto")
        self.assertEqual(packet["primary_subject"], "auto")
        self.assertNotIn(
            "finance",
            [candidate["subject"] for candidate in packet["candidate_subjects"]],
        )
        self.assertEqual(packet["borrowed_lenses"][0]["source_subject"], "finance")
        self.assertEqual(packet["borrowed_lenses"][0]["lens"], "event-study")

    def test_candidate_economics_signals_borrow_lens_without_subject_suggestion(self) -> None:
        with patch(
            "bridges.subject_refinement.subject_activation_status",
            side_effect=lambda subject: (
                "candidate" if subject == "economics" else "runtime_enabled"
            ),
        ):
            packet = infer_subject_refinement(
                {
                    "topic": "parallel trends identification",
                    "context": "Use DID and parallel trends diagnostics for the policy shock.",
                },
                manifest_state=ProjectManifest(),
            ).to_packet()

        self.assertEqual(packet["decision"], "borrow_lens")
        self.assertEqual(packet["mode"], "auto")
        self.assertEqual(packet["active_subject"], "auto")
        self.assertEqual(packet["primary_subject"], "auto")
        self.assertNotIn(
            "economics",
            [candidate["subject"] for candidate in packet["candidate_subjects"]],
        )
        self.assertEqual(packet["borrowed_lenses"][0]["source_subject"], "economics")
        self.assertEqual(packet["borrowed_lenses"][0]["lens"], "did")

    def test_finance_suggestion_includes_structured_signal_records(self) -> None:
        packet = infer_subject_refinement(
            {
                "topic": "earnings announcement returns",
                "context": "Estimate abnormal returns using CRSP data for Journal of Finance.",
            },
            manifest_state=ProjectManifest(),
            draft_content="Use an event study with event windows around earnings announcements.",
        ).to_packet()

        signals = packet["signals"]
        signal_ids = {signal["id"] for signal in signals}
        signal_dimensions = {signal["dimension"] for signal in signals}
        self.assertIn("finance.method.event-study", signal_ids)
        self.assertIn("data_or_outcome", signal_dimensions)
        self.assertIn("venue", signal_dimensions)
        event_study = next(
            signal for signal in signals if signal["id"] == "finance.method.event-study"
        )
        self.assertEqual(event_study["subject"], "finance")
        self.assertEqual(event_study["weight"], 0.35)
        self.assertEqual(event_study["source"], "task_text")
        self.assertIn("event study", event_study["snippet"])

    def test_asset_pricing_method_phrase_without_outcome_does_not_suggest_finance(self) -> None:
        packet = infer_subject_refinement(
            {
                "topic": "portfolio sorts",
                "context": "Plan portfolio sorts and factor regressions for the appendix.",
            },
            manifest_state=ProjectManifest(),
        ).to_packet()

        self.assertNotEqual(packet["decision"], "suggest_subject")
        self.assertNotEqual(packet["primary_subject"], "finance")
        self.assertEqual(packet["decision"], "borrow_lens")
        self.assertEqual(packet["borrowed_lenses"][0]["lens"], "asset-pricing")

    def test_locked_economics_manifest_prevents_switch_but_can_borrow_finance_lens(self) -> None:
        packet = infer_subject_refinement(
            {
                "topic": "treatment announcement",
                "context": "Use CRSP abnormal returns and an event study for Journal of Finance.",
            },
            manifest_state=ProjectManifest(active_subject="economics", subject_mode="locked"),
        ).to_packet()

        self.assertEqual(packet["decision"], "lock_subject")
        self.assertEqual(packet["mode"], "locked")
        self.assertEqual(packet["active_subject"], "economics")
        self.assertEqual(packet["primary_subject"], "economics")
        self.assertEqual(packet["borrowed_lenses"][0]["source_subject"], "finance")
        self.assertEqual(packet["borrowed_lenses"][0]["lens"], "event-study")
        self.assertEqual(packet["persistence"]["status"], "locked")

    def test_locked_eval_ready_accounting_signal_borrows_without_candidate(self) -> None:
        packet = infer_subject_refinement(
            {
                "topic": "archival accounting accrual quality",
                "context": "Use discretionary accruals for the accounting measurement check.",
            },
            manifest_state=ProjectManifest(active_subject="finance", subject_mode="locked"),
            evaluation_subjects={"accounting"},
        ).to_packet()

        self.assertEqual(packet["decision"], "lock_subject")
        self.assertEqual(packet["primary_subject"], "finance")
        self.assertNotIn(
            "accounting",
            [candidate["subject"] for candidate in packet["candidate_subjects"]],
        )
        self.assertIn(
            ("accounting", "accrual-quality"),
            {
                (lens["source_subject"], lens["lens"])
                for lens in packet["borrowed_lenses"]
            },
        )
        self.assertIn(
            "content/subjects/accounting/skills/accounting-measurement-auditor.md",
            packet["loaded_resources"]["method_packs"],
        )

    def test_locked_finance_with_strong_accounting_evidence_keeps_lock_and_exposes_candidate(self) -> None:
        packet = infer_subject_refinement(
            {
                "topic": "finance paper with accounting measurement",
                "context": (
                    "Keep finance as the locked subject, but add discretionary "
                    "accruals, construct-proxy checks, and earnings quality."
                ),
            },
            manifest_state=ProjectManifest(
                active_subject="finance",
                subject_mode="locked",
                method_lenses=["event-study"],
            ),
        ).to_packet()

        self.assertEqual(packet["decision"], "lock_subject")
        self.assertEqual(packet["primary_subject"], "finance")
        self.assertIn(
            "accounting",
            [candidate["subject"] for candidate in packet["candidate_subjects"]],
        )
        self.assertIn(
            ("accounting", "accrual-quality"),
            {
                (lens["source_subject"], lens["lens"])
                for lens in packet["borrowed_lenses"]
            },
        )

    def test_confirmed_finance_manifest_controls_when_context_is_weak(self) -> None:
        packet = infer_subject_refinement(
            {"topic": "revise introduction", "context": "Tighten the framing."},
            manifest_state=ProjectManifest(active_subject="finance", subject_mode="confirmed"),
        ).to_packet()

        self.assertEqual(packet["decision"], "confirm_subject")
        self.assertEqual(packet["mode"], "confirmed")
        self.assertEqual(packet["active_subject"], "finance")
        self.assertEqual(packet["primary_subject"], "finance")
        self.assertEqual(packet["persistence"]["status"], "applied")

    def test_confirmed_subject_does_not_borrow_hard_coded_neighbor_lenses(self) -> None:
        cases = [
            (
                ProjectManifest(active_subject="economics", subject_mode="confirmed"),
                {
                    "topic": "treatment announcement",
                    "context": "Use CRSP abnormal returns and an event study.",
                },
            ),
            (
                ProjectManifest(active_subject="finance", subject_mode="confirmed"),
                {
                    "topic": "policy shock identification",
                    "context": "Use DID and parallel trends diagnostics.",
                },
            ),
        ]

        for manifest, task_packet in cases:
            with self.subTest(active_subject=manifest.active_subject):
                packet = infer_subject_refinement(
                    task_packet,
                    manifest_state=manifest,
                ).to_packet()

                self.assertEqual(packet["decision"], "confirm_subject")
                self.assertEqual(packet["primary_subject"], manifest.active_subject)
                self.assertEqual(packet["borrowed_lenses"], [])
                self.assertNotIn("method_pack_only", packet["loaded_resources"]["levels"])

    def test_runtime_enabled_confirmed_accounting_loads_subject_resources(self) -> None:
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
        self.assertIn(
            "content/subjects/accounting/skills/accounting-measurement-auditor.md",
            packet["loaded_resources"]["subject_skills"],
        )
        self.assertIn("subject_overlay", packet["loaded_resources"]["levels"])
        self.assertIn("subject_skill", packet["loaded_resources"]["levels"])
        self.assertIn("subject_overlay", packet["resource_activation_plan"]["levels"])
        self.assertIn("subject_skill", packet["resource_activation_plan"]["levels"])
        self.assertEqual(packet["loaded_resources"]["contract_warnings"], [])

    def test_eval_ready_accounting_confirmed_subject_withholds_subject_resources(self) -> None:
        with patch(
            "bridges.subject_refinement.subject_activation_status",
            return_value="eval_ready",
        ):
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
        self.assertNotIn("subject_overlay", packet["loaded_resources"]["levels"])
        self.assertNotIn("subject_skill", packet["loaded_resources"]["levels"])
        self.assertTrue(packet["loaded_resources"]["contract_warnings"])
        self.assertIn(
            "activation_status=eval_ready",
            packet["loaded_resources"]["contract_warnings"][0],
        )

    def test_runtime_enabled_accounting_signals_suggest_accounting(self) -> None:
        packet = infer_subject_refinement(
            {
                "topic": "archival accounting accrual quality",
                "context": (
                    "Use discretionary accruals, Audit Analytics restatements, "
                    "internal-control weaknesses, financial reporting quality, "
                    "and Journal of Accounting Research positioning."
                ),
            },
            manifest_state=ProjectManifest(),
        ).to_packet()

        self.assertEqual(packet["decision"], "suggest_subject")
        self.assertEqual(packet["primary_subject"], "accounting")
        self.assertIn(
            "accounting",
            [candidate["subject"] for candidate in packet["candidate_subjects"]],
        )
        self.assertIn("accrual-quality", packet["method_lenses"])
        self.assertIn("construct-proxy-audit", packet["method_lenses"])

    def test_runtime_enabled_accounting_method_only_auto_borrows_lens(self) -> None:
        packet = infer_subject_refinement(
            {
                "topic": "robustness controls",
                "context": (
                    "Add accrual quality and discretionary accrual controls "
                    "to the empirical appendix."
                ),
            },
            manifest_state=ProjectManifest(),
        ).to_packet()

        self.assertEqual(packet["decision"], "borrow_lens")
        self.assertEqual(packet["primary_subject"], "auto")
        self.assertNotIn(
            "accounting",
            [candidate["subject"] for candidate in packet["candidate_subjects"]],
        )
        self.assertIn(
            ("accounting", "accrual-quality"),
            {
                (lens["source_subject"], lens["lens"])
                for lens in packet["borrowed_lenses"]
            },
        )

    def test_accounting_method_and_context_only_stays_borrowed_lens_without_candidate(self) -> None:
        packet = infer_subject_refinement(
            {
                "topic": "archival measurement appendix",
                "context": (
                    "Use discretionary accruals and position the measurement "
                    "discussion for Journal of Accounting Research."
                ),
            },
            manifest_state=ProjectManifest(),
        ).to_packet()

        self.assertEqual(packet["decision"], "borrow_lens")
        self.assertEqual(packet["primary_subject"], "auto")
        self.assertNotIn(
            "accounting",
            [candidate["subject"] for candidate in packet["candidate_subjects"]],
        )

    def test_accounting_context_and_subject_level_construct_can_suggest_accounting(self) -> None:
        packet = infer_subject_refinement(
            {
                "topic": "financial reporting mechanisms",
                "context": (
                    "Develop the financial reporting mechanism and disclosure "
                    "quality argument for Journal of Accounting Research."
                ),
            },
            manifest_state=ProjectManifest(),
        ).to_packet()

        self.assertEqual(packet["decision"], "suggest_subject")
        self.assertEqual(packet["primary_subject"], "accounting")
        self.assertIn(
            "accounting",
            [candidate["subject"] for candidate in packet["candidate_subjects"]],
        )

    def test_accounting_method_and_subject_level_data_can_suggest_accounting(self) -> None:
        packet = infer_subject_refinement(
            {
                "topic": "accrual quality restatements",
                "context": (
                    "Use discretionary accruals with Audit Analytics "
                    "restatements as the accounting data source."
                ),
            },
            manifest_state=ProjectManifest(),
        ).to_packet()

        self.assertEqual(packet["decision"], "suggest_subject")
        self.assertEqual(packet["primary_subject"], "accounting")
        self.assertIn(
            "accounting",
            [candidate["subject"] for candidate in packet["candidate_subjects"]],
        )

    def test_mixed_finance_and_accounting_method_only_borrows_both_lenses(self) -> None:
        packet = infer_subject_refinement(
            {
                "topic": "announcement and accrual design",
                "context": "Use an event study and discretionary accruals.",
            },
            manifest_state=ProjectManifest(),
        ).to_packet()

        borrowed = {
            (lens["source_subject"], lens["lens"])
            for lens in packet["borrowed_lenses"]
        }
        self.assertIn(("finance", "event-study"), borrowed)
        self.assertIn(("accounting", "accrual-quality"), borrowed)
        self.assertIn(
            "method-packs/finance/event-study.yaml",
            packet["loaded_resources"]["method_packs"],
        )
        self.assertIn(
            "content/subjects/accounting/skills/accounting-measurement-auditor.md",
            packet["loaded_resources"]["method_packs"],
        )

    def test_runtime_enabled_gate_mode_can_measure_accounting_primary_subject(self) -> None:
        packet = infer_subject_refinement(
            {
                "topic": "archival accounting accrual quality",
                "context": (
                    "Design a study of discretionary accruals, Audit Analytics "
                    "restatements, financial reporting quality, and Journal of "
                    "Accounting Research positioning."
                ),
            },
            manifest_state=ProjectManifest(),
            evaluation_subjects={"accounting"},
        ).to_packet()

        self.assertEqual(packet["decision"], "suggest_subject")
        self.assertEqual(packet["primary_subject"], "accounting")
        self.assertIn(
            "accounting",
            [candidate["subject"] for candidate in packet["candidate_subjects"]],
        )
        self.assertIn("accrual-quality", packet["method_lenses"])
        self.assertEqual(packet["loaded_resources"]["overlays"], [])
        self.assertIn(
            "content/subjects/accounting/skills/accounting-measurement-auditor.md",
            packet["loaded_resources"]["subject_skills"],
        )
        self.assertEqual(packet["loaded_resources"]["contract_warnings"], [])

    def test_eval_ready_business_signals_can_be_measured_under_evaluation_subjects(self) -> None:
        with patch(
            "bridges.subject_refinement.load_runtime_subject_contracts",
            return_value={"business": _business_runtime_subject_contract()},
        ):
            packet = infer_subject_refinement(
                {
                    "topic": "management theory case study",
                    "context": (
                        "Use a multiple case study with interviews with managers "
                        "to develop a management theory contribution for AMJ."
                    ),
                },
                manifest_state=ProjectManifest(),
                evaluation_subjects={"business"},
            ).to_packet()

        self.assertEqual(packet["decision"], "suggest_subject")
        self.assertEqual(packet["primary_subject"], "business")
        self.assertEqual(packet["domain"], "business-management")
        self.assertIn(
            "business",
            [candidate["subject"] for candidate in packet["candidate_subjects"]],
        )
        self.assertIn("business-positioning", packet["method_lenses"])
        self.assertIn("qualitative-transparency", packet["method_lenses"])
        self.assertEqual(packet["loaded_resources"]["overlays"], [])
        self.assertEqual(packet["loaded_resources"]["subject_skills"], [])
        self.assertIn(
            "content/subjects/business/skills/business-journal-positioning-auditor.md",
            packet["loaded_resources"]["method_packs"],
        )
        self.assertIn(
            "content/subjects/business/overlays/skills/study-designer.md",
            packet["loaded_resources"]["method_packs"],
        )
        self.assertTrue(packet["loaded_resources"]["contract_warnings"])
        self.assertIn(
            "activation_status=eval_ready",
            packet["loaded_resources"]["contract_warnings"][0],
        )

    def test_business_activation_override_measures_default_runtime_suggestion(self) -> None:
        with patch(
            "bridges.subject_refinement.load_runtime_subject_contracts",
            return_value={"business": _business_runtime_subject_contract()},
        ):
            packet = infer_subject_refinement(
                {
                    "topic": "management theory case study",
                    "context": (
                        "Use a multiple case study with interviews with managers "
                        "to develop a management theory contribution for AMJ."
                    ),
                },
                manifest_state=ProjectManifest(),
                activation_status_overrides={"business": "runtime_enabled"},
            ).to_packet()

        self.assertEqual(packet["decision"], "suggest_subject")
        self.assertEqual(packet["primary_subject"], "business")
        self.assertEqual(packet["domain"], "business-management")
        self.assertIn(
            "business",
            [candidate["subject"] for candidate in packet["candidate_subjects"]],
        )
        self.assertIn("business-positioning", packet["method_lenses"])
        self.assertIn("qualitative-transparency", packet["method_lenses"])
        self.assertEqual(packet["loaded_resources"]["contract_warnings"], [])

    def test_business_without_activation_override_remains_suppressed_by_default(self) -> None:
        with patch(
            "bridges.subject_refinement.load_runtime_subject_contracts",
            return_value={"business": _business_runtime_subject_contract()},
        ), patch(
            "bridges.subject_refinement.subject_activation_status",
            return_value="eval_ready",
        ):
            packet = infer_subject_refinement(
                {
                    "topic": "management theory case study",
                    "context": (
                        "Use a multiple case study with interviews with managers "
                        "to develop a management theory contribution for AMJ."
                    ),
                },
                manifest_state=ProjectManifest(),
            ).to_packet()

        self.assertNotEqual(packet["decision"], "suggest_subject")
        self.assertNotEqual(packet["primary_subject"], "business")
        self.assertNotIn(
            "business",
            [candidate["subject"] for candidate in packet["candidate_subjects"]],
        )

    def test_eval_ready_business_method_only_borrows_lens_without_subject_suggestion(self) -> None:
        with patch(
            "bridges.subject_refinement.load_runtime_subject_contracts",
            return_value={"business": _business_runtime_subject_contract()},
        ):
            packet = infer_subject_refinement(
                {
                    "topic": "qualitative coding appendix",
                    "context": (
                        "Use the Gioia method with first-order concepts and "
                        "second-order themes for a qualitative coding appendix."
                    ),
                },
                manifest_state=ProjectManifest(),
            ).to_packet()

        self.assertEqual(packet["decision"], "borrow_lens")
        self.assertEqual(packet["primary_subject"], "auto")
        self.assertNotIn(
            "business",
            [candidate["subject"] for candidate in packet["candidate_subjects"]],
        )
        self.assertIn(
            ("business", "qualitative-transparency"),
            {
                (lens["source_subject"], lens["lens"])
                for lens in packet["borrowed_lenses"]
            },
        )
        self.assertIn(
            "content/subjects/business/overlays/skills/study-designer.md",
            packet["loaded_resources"]["method_packs"],
        )

    def test_eval_ready_business_default_runtime_does_not_suggest_business(self) -> None:
        with patch(
            "bridges.subject_refinement.load_runtime_subject_contracts",
            return_value={"business": _business_runtime_subject_contract()},
        ), patch(
            "bridges.subject_refinement.subject_activation_status",
            return_value="eval_ready",
        ):
            packet = infer_subject_refinement(
                {
                    "topic": "management theory case study",
                    "context": (
                        "Use a multiple case study with interviews with managers "
                        "to develop a management theory contribution for AMJ."
                    ),
                },
                manifest_state=ProjectManifest(),
            ).to_packet()

        self.assertNotEqual(packet["decision"], "suggest_subject")
        self.assertNotEqual(packet["primary_subject"], "business")
        self.assertNotIn(
            "business",
            [candidate["subject"] for candidate in packet["candidate_subjects"]],
        )

    def test_runtime_enabled_business_real_manifest_can_be_measured_under_evaluation_subjects(self) -> None:
        packet = infer_subject_refinement(
            {
                "topic": "management theory case study",
                "context": (
                    "Design a multiple case study using interviews with managers "
                    "to develop a management theory contribution about organizational "
                    "routines for Academy of Management Journal business journal positioning."
                ),
            },
            manifest_state=ProjectManifest(),
            evaluation_subjects={"business"},
        ).to_packet()

        self.assertEqual(packet["decision"], "suggest_subject")
        self.assertEqual(packet["primary_subject"], "business")
        self.assertEqual(packet["domain"], "business-management")
        self.assertIn(
            "business",
            [candidate["subject"] for candidate in packet["candidate_subjects"]],
        )
        self.assertIn("business-positioning", packet["method_lenses"])
        self.assertIn("qualitative-transparency", packet["method_lenses"])

    def test_runtime_enabled_business_real_manifest_suggests_business(self) -> None:
        packet = infer_subject_refinement(
            {
                "topic": "management theory case study",
                "context": (
                    "Design a multiple case study using interviews with managers "
                    "to develop a management theory contribution for AMJ."
                ),
            },
            manifest_state=ProjectManifest(),
        ).to_packet()

        self.assertEqual(packet["decision"], "suggest_subject")
        self.assertEqual(packet["primary_subject"], "business")
        self.assertIn(
            "business",
            [candidate["subject"] for candidate in packet["candidate_subjects"]],
        )
        self.assertIn("business-positioning", packet["method_lenses"])
        self.assertIn(
            "content/subjects/business/skills/business-journal-positioning-auditor.md",
            packet["loaded_resources"]["subject_skills"],
        )
        self.assertEqual(packet["loaded_resources"]["contract_warnings"], [])

    def test_runtime_enabled_business_confirmed_manifest_loads_subject_resources(self) -> None:
        packet = infer_subject_refinement(
            {
                "topic": "journal positioning",
                "context": (
                    "Tighten the Academy of Management Journal positioning, "
                    "management theory contribution, construct clarity, and "
                    "boundary conditions for this business manuscript."
                ),
            },
            manifest_state=ProjectManifest(
                active_subject="business",
                subject_mode="confirmed",
                method_lenses=["business-positioning"],
            ),
        ).to_packet()

        self.assertEqual(packet["decision"], "confirm_subject")
        self.assertEqual(packet["primary_subject"], "business")
        self.assertIn(
            "content/subjects/business/skills/business-journal-positioning-auditor.md",
            packet["loaded_resources"]["subject_skills"],
        )
        self.assertIn("subject_skill", packet["loaded_resources"]["levels"])
        self.assertEqual(packet["loaded_resources"]["contract_warnings"], [])

    def test_runtime_enabled_business_method_only_real_manifest_borrows_lens(self) -> None:
        packet = infer_subject_refinement(
            {
                "topic": "qualitative coding",
                "context": (
                    "Use the Gioia method with first-order concepts, second-order "
                    "themes, and aggregate dimensions to organize qualitative coding."
                ),
            },
            manifest_state=ProjectManifest(),
        ).to_packet()

        self.assertEqual(packet["decision"], "borrow_lens")
        self.assertEqual(packet["primary_subject"], "auto")
        self.assertNotIn(
            "business",
            [candidate["subject"] for candidate in packet["candidate_subjects"]],
        )
        self.assertIn(
            ("business", "qualitative-transparency"),
            {
                (lens["source_subject"], lens["lens"])
                for lens in packet["borrowed_lenses"]
            },
        )

    def test_accounting_near_miss_account_for_heterogeneity_keeps_core(self) -> None:
        packet = infer_subject_refinement(
            {
                "topic": "regression diagnostics",
                "context": "Explain how to account for heterogeneity in a generic regression model.",
            },
            manifest_state=ProjectManifest(),
        ).to_packet()

        self.assertNotIn(
            "accounting",
            [signal["subject"] for signal in packet["signals"]],
        )
        self.assertEqual(packet["decision"], "no_subject")
        self.assertNotEqual(packet["primary_subject"], "accounting")

    def test_runtime_contract_load_failure_preserves_hard_coded_routing_warning(self) -> None:
        with patch(
            "bridges.subject_refinement.load_runtime_subject_contracts",
            side_effect=RuntimeError("runtime manifests unavailable"),
        ):
            packet = infer_subject_refinement(
                {
                    "topic": "policy announcement effects",
                    "context": "Use an event study design.",
                },
                manifest_state=ProjectManifest(active_subject="economics"),
            ).to_packet()

        self.assertEqual(packet["decision"], "borrow_lens")
        self.assertEqual(packet["borrowed_lenses"][0]["source_subject"], "finance")
        self.assertEqual(packet["borrowed_lenses"][0]["lens"], "event-study")
        self.assertTrue(packet["loaded_resources"]["contract_warnings"])
        self.assertIn(
            "runtime manifests unavailable",
            packet["loaded_resources"]["contract_warnings"][0],
        )

    def test_blank_runtime_subject_skill_does_not_erase_subject_skill_fallback(self) -> None:
        contract = {"subject_skills": {"finance": "skills/finance/SKILL.md"}}
        runtime_contract = _runtime_subject_contract(
            subject="finance",
            subject_skill="",
        )

        with patch(
            "bridges.subject_refinement._safe_load_runtime_subject_contracts",
            return_value=({"finance": runtime_contract}, []),
        ):
            resources = subject_refinement_module._subject_resource_map(
                contract,
                "subject_skills",
                {},
            )

        self.assertEqual(resources["finance"], "skills/finance/SKILL.md")

    def test_non_numeric_manifest_signal_weight_does_not_break_routing(self) -> None:
        contract = _runtime_subject_contract(
            signal_groups={
                "method": [
                    {
                        "id": "accounting.method.accrual-quality",
                        "value": "accrual-quality",
                        "weight": "not-a-number",
                        "patterns": [r"\bdiscretionary accruals?\b"],
                    },
                    {
                        "id": "accounting.method.construct-proxy-audit",
                        "value": "construct-proxy-audit",
                        "weight": float("nan"),
                        "patterns": [r"\bmeasurement validity\b"],
                    }
                ]
            },
            method_lenses={
                "accrual-quality": {
                    "resource": (
                        "content/subjects/accounting/skills/"
                        "accounting-measurement-auditor.md"
                    )
                },
                "construct-proxy-audit": {
                    "resource": (
                        "content/subjects/accounting/overlays/skills/"
                        "variable-constructor.md"
                    )
                }
            },
        )

        with patch(
            "bridges.subject_refinement.load_runtime_subject_contracts",
            return_value={"accounting": contract},
        ):
            packet = infer_subject_refinement(
                {
                    "topic": "archival accounting",
                    "context": "Estimate discretionary accruals and assess measurement validity.",
                },
                manifest_state=ProjectManifest(),
            ).to_packet()

        self.assertEqual(packet["decision"], "borrow_lens")
        signal_weights = {
            signal["id"]: signal["weight"]
            for signal in packet["signals"]
            if signal["subject"] == "accounting"
        }
        self.assertEqual(signal_weights["accounting.method.accrual-quality"], 0.0)
        self.assertEqual(signal_weights["accounting.method.construct-proxy-audit"], 0.0)

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

    def test_confirmed_manifest_method_lenses_include_method_pack_level(self) -> None:
        packet = infer_subject_refinement(
            {"topic": "revise introduction", "context": "Tighten the framing."},
            manifest_state=ProjectManifest(
                active_subject="finance",
                subject_mode="confirmed",
                method_lenses=["event-study"],
            ),
        ).to_packet()

        self.assertTrue(packet["loaded_resources"]["method_packs"])
        self.assertIn("method_pack", packet["loaded_resources"]["levels"])

    def test_locked_manifest_method_lenses_include_method_pack_level(self) -> None:
        packet = infer_subject_refinement(
            {"topic": "revise introduction", "context": "Tighten the framing."},
            manifest_state=ProjectManifest(
                active_subject="finance",
                subject_mode="locked",
                method_lenses=["event-study"],
            ),
        ).to_packet()

        self.assertTrue(packet["loaded_resources"]["method_packs"])
        self.assertIn("method_pack", packet["loaded_resources"]["levels"])

    def test_raw_mapping_legacy_non_auto_manifest_is_confirmed(self) -> None:
        packet = infer_subject_refinement(
            {"topic": "revise introduction", "context": "Tighten the framing."},
            manifest_state={"active_subject": "finance"},
        ).to_packet()

        self.assertEqual(packet["decision"], "confirm_subject")
        self.assertEqual(packet["mode"], "confirmed")
        self.assertEqual(packet["active_subject"], "finance")

    def test_missing_explicit_standards_dir_surfaces_contract_warning(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            packet = infer_subject_refinement(
                {"topic": "revise introduction", "context": "Tighten the framing."},
                manifest_state=ProjectManifest(),
                standards_dir=Path(tmp_dir) / "missing-standards",
            ).to_packet()

        warnings = packet["loaded_resources"]["contract_warnings"]
        self.assertTrue(warnings)
        self.assertIn("Missing subject refinement contract", warnings[0])

    def test_no_subject_signal_keeps_core_only_resources(self) -> None:
        packet = infer_subject_refinement(
            {"topic": "revise introduction", "context": "Tighten the framing."},
            manifest_state=ProjectManifest(),
        ).to_packet()

        self.assertEqual(packet["decision"], "no_subject")
        self.assertEqual(packet["mode"], "auto")
        self.assertEqual(packet["active_subject"], "auto")
        self.assertEqual(packet["primary_subject"], "auto")
        self.assertEqual(packet["loaded_resources"]["levels"], ["core_only"])
        self.assertEqual(packet["persistence"]["status"], "none")
        self.assertEqual(packet["signals"], [])
        self.assertEqual(packet["evidence"], [])


if __name__ == "__main__":
    unittest.main()
