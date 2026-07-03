from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import yaml

from bridges.project_manifest import ProjectManifest
from bridges.subject_refinement import infer_subject_refinement


class SubjectRefinementTests(unittest.TestCase):
    def test_contract_declares_locked_persistence_status(self) -> None:
        contract_path = Path("content/standards/subject-refinement-contract.yaml")
        contract = yaml.safe_load(contract_path.read_text(encoding="utf-8")) or {}

        statuses = contract["persistence_statuses"]
        self.assertIn("locked", statuses)
        self.assertIn("confirmed", statuses["applied"]["description"].lower())
        self.assertNotIn("locked", statuses["applied"]["description"].lower())

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
        self.assertNotIn("subject_overlay", packet["loaded_resources"]["levels"])
        self.assertNotIn("subject_skill", packet["loaded_resources"]["levels"])
        self.assertNotIn("subject_overlay", packet["resource_activation_plan"]["levels"])
        self.assertNotIn("subject_skill", packet["resource_activation_plan"]["levels"])
        self.assertTrue(packet["loaded_resources"]["contract_warnings"])
        self.assertIn(
            "activation_status=candidate",
            packet["loaded_resources"]["contract_warnings"][0],
        )

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
