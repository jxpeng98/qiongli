from __future__ import annotations

import unittest

from bridges.subject_resources import build_resource_activation_plan


class SubjectResourceActivationTests(unittest.TestCase):
    def test_suggest_subject_plan_proposes_subject_resources_and_method_pack(self) -> None:
        plan = build_resource_activation_plan(
            decision="suggest_subject",
            active_subject="auto",
            primary_subject="finance",
            loaded_resources={
                "levels": ["subject_overlay", "subject_skill", "method_pack"],
                "overlays": ["overlays/finance.yaml"],
                "subject_skills": ["skills/finance/SKILL.md"],
                "method_packs": ["method-packs/finance/event-study.yaml"],
                "contract_warnings": [],
            },
            method_lenses=["event-study"],
            borrowed_lenses=[],
            persistence={"status": "proposed"},
        )

        self.assertEqual(plan["decision"], "suggest_subject")
        self.assertEqual(plan["active_subject"], "auto")
        self.assertEqual(plan["primary_subject"], "finance")
        self.assertEqual(plan["levels"], ["core", "subject_overlay", "subject_skill", "method_pack"])
        self.assertIn(
            {
                "kind": "subject_overlay",
                "subject": "finance",
                "path": "overlays/finance.yaml",
                "activation": "proposed",
            },
            plan["resources"],
        )
        self.assertIn(
            {
                "kind": "subject_skill",
                "subject": "finance",
                "path": "skills/finance/SKILL.md",
                "activation": "proposed",
            },
            plan["resources"],
        )
        self.assertIn(
            {
                "kind": "method_pack",
                "subject": "finance",
                "lens": "event-study",
                "path": "method-packs/finance/event-study.yaml",
                "activation": "proposed",
            },
            plan["resources"],
        )
        self.assertEqual(
            plan["persistence_recommendation"],
            {
                "status": "proposed",
                "write_manifest": False,
                "recommended_subject_mode": "suggested",
            },
        )
        self.assertEqual(plan["contract_warnings"], [])

    def test_borrow_lens_plan_keeps_active_subject_and_marks_method_pack_temporary(self) -> None:
        plan = build_resource_activation_plan(
            decision="borrow_lens",
            active_subject="auto",
            primary_subject="auto",
            loaded_resources={
                "levels": ["method_pack_only"],
                "method_packs": ["method-packs/finance/event-study.yaml"],
                "contract_warnings": [],
            },
            method_lenses=[],
            borrowed_lenses=[
                {
                    "source_subject": "finance",
                    "lens": "event-study",
                    "resource_level": "method_pack_only",
                }
            ],
            persistence={"status": "temporary"},
        )

        self.assertEqual(plan["decision"], "borrow_lens")
        self.assertEqual(plan["active_subject"], "auto")
        self.assertEqual(plan["primary_subject"], "auto")
        self.assertEqual(plan["levels"], ["core", "method_pack_only"])
        self.assertIn(
            {
                "kind": "method_pack_only",
                "subject": "finance",
                "lens": "event-study",
                "path": "method-packs/finance/event-study.yaml",
                "activation": "temporary",
            },
            plan["resources"],
        )
        self.assertFalse(plan["persistence_recommendation"]["write_manifest"])


if __name__ == "__main__":
    unittest.main()
