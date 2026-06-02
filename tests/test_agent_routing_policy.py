from __future__ import annotations

import unittest
from pathlib import Path

from qiongli.source_layout import RepoLayout

import yaml


REPO_ROOT = Path(__file__).resolve().parents[1]

POLICY_PATH = RepoLayout(REPO_ROOT).standards / "agent-routing-policy.yaml"

STAGE_DEFAULTS = {
    "B_literature": {"primary": "claude", "reviewer": "codex", "verifier": "codex"},
    "F_writing": {"primary": "claude", "reviewer": "codex", "verifier": "codex"},
    "I_code": {"primary": "codex", "reviewer": "claude", "verifier": "codex"},
}

SOLO_GATE_MAPPINGS = {
    "solo_codex": {
        "writing_gates": [
            "evidence_ledger_check",
            "citation_risk_check",
            "claim_calibration_check",
            "scholarly_voice_check",
        ],
        "code_gates": ["tests", "strict_validator", "diff_review"],
    },
    "solo_claude": {
        "writing_gates": [
            "evidence_ledger_check",
            "citation_risk_check",
            "reviewer_self_critique",
        ],
        "code_gates": [
            "implementation_intent",
            "declared_write_set",
            "failing_test_first",
            "command_evidence",
            "rollback_notes",
        ],
    },
}


class AgentRoutingPolicyTests(unittest.TestCase):
    def test_agent_routing_policy_defines_exact_contract_shape(self) -> None:
        self.assertTrue(POLICY_PATH.exists(), f"Missing required artifact: {POLICY_PATH}")

        policy = yaml.safe_load(POLICY_PATH.read_text(encoding="utf-8")) or {}
        self.assertIsInstance(policy, dict)
        self.assertEqual(
            {"policy_version", "stage_defaults", "solo_gate_mappings"},
            set(policy),
        )
        self.assertEqual("1.0.0", policy["policy_version"])

        stage_defaults = policy.get("stage_defaults")
        self.assertIsInstance(stage_defaults, dict)
        self.assertEqual(set(STAGE_DEFAULTS), set(stage_defaults))
        for stage, expected_routing in STAGE_DEFAULTS.items():
            with self.subTest(stage=stage):
                routing = stage_defaults.get(stage)
                self.assertIsInstance(routing, dict)
                self.assertEqual({"primary", "reviewer", "verifier"}, set(routing))
                self.assertEqual(expected_routing, routing)

        solo_gate_mappings = policy.get("solo_gate_mappings")
        self.assertIsInstance(solo_gate_mappings, dict)
        self.assertEqual(set(SOLO_GATE_MAPPINGS), set(solo_gate_mappings))
        for solo_mode, expected_mapping in SOLO_GATE_MAPPINGS.items():
            with self.subTest(solo_mode=solo_mode):
                mapping = solo_gate_mappings.get(solo_mode)
                self.assertIsInstance(mapping, dict)
                self.assertEqual({"writing_gates", "code_gates"}, set(mapping))
                self.assertEqual(expected_mapping, mapping)


if __name__ == "__main__":
    unittest.main()
