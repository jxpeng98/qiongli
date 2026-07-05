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
        subject_skill="",
        signal_groups=signal_groups or {},
        method_lenses=method_lenses or {},
        evaluation_pack="",
        near_miss_policy={},
        activation_gate={},
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

    def test_eval_ready_confirmed_subject_withholds_subject_level_resources(self) -> None:
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
        self.assertNotIn("subject_overlay", packet["resource_activation_plan"]["levels"])
        self.assertNotIn("subject_skill", packet["resource_activation_plan"]["levels"])
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

    def test_eval_ready_gate_mode_can_measure_accounting_primary_subject(self) -> None:
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
        self.assertEqual(packet["loaded_resources"]["subject_skills"], [])
        self.assertTrue(packet["loaded_resources"]["contract_warnings"])

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
