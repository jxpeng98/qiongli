from __future__ import annotations

import unittest
from pathlib import Path

from qiongli.source_layout import RepoLayout

import yaml


REPO_ROOT = Path(__file__).resolve().parents[1]

POLICY_PATH = RepoLayout(REPO_ROOT).standards / "solo-role-policy.yaml"
SOLO_TASK_PACKET_PATH = RepoLayout(REPO_ROOT).templates / "solo-task-packet.md"
SOLO_SELF_REVIEW_PATH = RepoLayout(REPO_ROOT).templates / "solo-self-review.md"
IMPLEMENTATION_INTENT_PATH = RepoLayout(REPO_ROOT).templates / "implementation-intent.md"
WRITING_CLAIM_MAP_PATH = RepoLayout(REPO_ROOT).templates / "writing-claim-map.md"
QUALITY_GATE_REPORT_PATH = RepoLayout(REPO_ROOT).templates / "quality-gate-report.md"

SOLO_ROLES = {"solo_codex", "solo_claude"}

CODEX_WRITING_GATES = {
    "evidence_ledger_check",
    "citation_risk_check",
    "claim_calibration_check",
    "scholarly_voice_check",
}

CLAUDE_CODE_GATES = {
    "implementation_intent",
    "declared_write_set",
    "failing_test_first",
    "command_evidence",
    "rollback_notes",
}

TEMPLATE_HEADINGS = {
    SOLO_TASK_PACKET_PATH: (
        "## Task Metadata",
        "## Required Artifacts",
        "## Role Gates",
        "## Verification Commands",
    ),
    SOLO_SELF_REVIEW_PATH: (
        "## Draft Pass",
        "## Self-Critique Pass",
        "## Revision Pass",
        "## Final Checklist",
    ),
    IMPLEMENTATION_INTENT_PATH: (
        "## Declared Write Set",
        "## Rationale",
        "## Failing Test Plan",
        "## Rollback Notes",
    ),
    WRITING_CLAIM_MAP_PATH: (
        "## Claim ID",
        "## Claim Text",
        "## Evidence IDs",
        "## Calibration",
        "## Unsupported Claims",
    ),
    QUALITY_GATE_REPORT_PATH: (
        "## Gate Metadata",
        "## Passed Gates",
        "## Failed Gates",
        "## Blocked Verification",
        "## Next Actions",
    ),
}


class SoloRolePolicyTests(unittest.TestCase):
    def test_solo_role_policy_defines_required_roles_and_gates(self) -> None:
        self.assertTrue(POLICY_PATH.exists(), f"Missing required artifact: {POLICY_PATH}")

        policy = yaml.safe_load(POLICY_PATH.read_text(encoding="utf-8")) or {}
        self.assertIsInstance(policy, dict)
        self.assertEqual({"policy_version", "solo_modes"}, set(policy))
        self.assertEqual("1.0.0", policy["policy_version"])

        modes = policy.get("solo_modes")
        self.assertIsInstance(modes, dict)
        self.assertEqual(SOLO_ROLES, set(modes))

        solo_codex = modes.get("solo_codex", {})
        self.assertEqual(
            CODEX_WRITING_GATES,
            set(solo_codex.get("writing_required_gates", [])),
        )
        self.assertEqual(
            {"tests", "strict_validator", "diff_review"},
            set(solo_codex.get("code_required_gates", [])),
        )

        solo_claude = modes.get("solo_claude", {})
        self.assertEqual(
            {"evidence_ledger_check", "citation_risk_check", "reviewer_self_critique"},
            set(solo_claude.get("writing_required_gates", [])),
        )
        self.assertEqual(
            CLAUDE_CODE_GATES,
            set(solo_claude.get("code_required_gates", [])),
        )

    def test_required_solo_templates_exist_with_contract_headings(self) -> None:
        for path, headings in TEMPLATE_HEADINGS.items():
            with self.subTest(path=path):
                self.assertTrue(path.exists(), f"Missing required artifact: {path}")
                text = path.read_text(encoding="utf-8")
                for heading in headings:
                    self.assertIn(heading, text)


if __name__ == "__main__":
    unittest.main()
