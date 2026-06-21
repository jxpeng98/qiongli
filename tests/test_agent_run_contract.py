from __future__ import annotations

import json
import unittest
from pathlib import Path

from qiongli.source_layout import RepoLayout

import yaml


REPO_ROOT = Path(__file__).resolve().parents[1]

CONTRACT_PATH = RepoLayout(REPO_ROOT).standards / "agent-run-contract.yaml"
RUN_PACKET_PATH = RepoLayout(REPO_ROOT).templates / "agent-run-packet.json"
REVIEW_PACKET_PATH = RepoLayout(REPO_ROOT).templates / "agent-review-packet.md"
HANDOFF_PATH = RepoLayout(REPO_ROOT).templates / "agent-handoff.md"

REQUIRED_RUN_FIELDS = {
    "run_id",
    "execution_mode",
    "controller",
    "primary_agent",
    "task_id",
    "paper_type",
    "topic",
    "input_context_hash",
    "session_id",
    "artifacts_read",
    "artifacts_written",
    "warnings",
    "blocking_issues",
    "confidence",
    "verification_status",
}

REQUIRED_REVIEW_FIELDS = {
    "reviewer_agent",
    "reviewed_run_id",
    "review_status",
    "findings",
    "blocking_issues",
    "required_revisions",
    "verification_evidence",
}

REQUIRED_HANDOFF_FIELDS = {
    "from_agent",
    "to_agent",
    "task_id",
    "completed_artifacts",
    "decision_summary",
    "unresolved_questions",
    "evidence_dependencies",
    "assumptions",
    "risks",
    "next_actions",
}

EXECUTION_MODES = {"solo_codex", "solo_claude", "solo_antigravity", "duo", "triad"}
RUNTIME_AGENTS = {"codex", "claude", "antigravity"}
VERIFICATION_STATUSES = {"passed", "failed", "blocked"}

REVIEW_FIELD_MARKERS = {
    "reviewer_agent": "- reviewer_agent:",
    "reviewed_run_id": "- reviewed_run_id:",
    "review_status": "- review_status:",
    "findings": "## Findings",
    "blocking_issues": "## Blocking Issues",
    "required_revisions": "## Required Revisions",
    "verification_evidence": "## Verification Evidence",
}

HANDOFF_FIELD_MARKERS = {
    "from_agent": "- from_agent:",
    "to_agent": "- to_agent:",
    "task_id": "- task_id:",
    "completed_artifacts": "## Completed Artifacts",
    "decision_summary": "## Decision Summary",
    "unresolved_questions": "## Unresolved Questions",
    "evidence_dependencies": "## Evidence Dependencies",
    "assumptions": "## Assumptions",
    "risks": "## Risks",
    "next_actions": "## Next Actions",
}


class AgentRunContractTests(unittest.TestCase):
    def test_required_artifacts_exist(self) -> None:
        for path in (
            CONTRACT_PATH,
            RUN_PACKET_PATH,
            REVIEW_PACKET_PATH,
            HANDOFF_PATH,
        ):
            with self.subTest(path=path):
                self.assertTrue(path.exists(), f"Missing required artifact: {path}")

    def test_contract_defines_runtime_enums_and_required_fields(self) -> None:
        self.assertTrue(CONTRACT_PATH.exists(), f"Missing required artifact: {CONTRACT_PATH}")

        contract = yaml.safe_load(CONTRACT_PATH.read_text(encoding="utf-8")) or {}
        self.assertIsInstance(contract, dict)

        for key in (
            "execution_modes",
            "runtime_agents",
            "verification_statuses",
            "required_run_fields",
            "required_review_fields",
            "required_handoff_fields",
        ):
            with self.subTest(key=key):
                self.assertIsInstance(contract.get(key), list)

        self.assertEqual("1.0.0", contract.get("contract_version"))
        self.assertEqual(EXECUTION_MODES, set(contract.get("execution_modes", [])))
        self.assertEqual(RUNTIME_AGENTS, set(contract.get("runtime_agents", [])))
        self.assertEqual(VERIFICATION_STATUSES, set(contract.get("verification_statuses", [])))
        self.assertEqual(REQUIRED_RUN_FIELDS, set(contract.get("required_run_fields", [])))
        self.assertEqual(REQUIRED_REVIEW_FIELDS, set(contract.get("required_review_fields", [])))
        self.assertEqual(REQUIRED_HANDOFF_FIELDS, set(contract.get("required_handoff_fields", [])))

    def test_run_packet_includes_safe_defaults_for_required_run_fields(self) -> None:
        self.assertTrue(RUN_PACKET_PATH.exists(), f"Missing required artifact: {RUN_PACKET_PATH}")

        packet = json.loads(RUN_PACKET_PATH.read_text(encoding="utf-8"))

        self.assertEqual(REQUIRED_RUN_FIELDS, set(packet))
        self.assertIn(packet["execution_mode"], EXECUTION_MODES)
        self.assertIn(packet["controller"], RUNTIME_AGENTS)
        self.assertIn(packet["primary_agent"], RUNTIME_AGENTS)
        self.assertIn(packet["verification_status"], VERIFICATION_STATUSES)

        for field in ("run_id", "task_id", "paper_type", "topic", "input_context_hash", "session_id"):
            with self.subTest(field=field):
                self.assertEqual("", packet[field])

        for field in ("artifacts_read", "artifacts_written", "warnings", "blocking_issues"):
            with self.subTest(field=field):
                self.assertEqual([], packet[field])

        self.assertEqual(0.0, packet["confidence"])

    def test_review_packet_includes_required_headings(self) -> None:
        self.assertTrue(REVIEW_PACKET_PATH.exists(), f"Missing required artifact: {REVIEW_PACKET_PATH}")

        text = REVIEW_PACKET_PATH.read_text(encoding="utf-8")

        for heading in (
            "# Agent Review Packet",
            "## Review Metadata",
            "## Findings",
            "## Blocking Issues",
            "## Required Revisions",
            "## Verification Evidence",
        ):
            with self.subTest(heading=heading):
                self.assertIn(heading, text)

    def test_review_packet_represents_required_contract_fields(self) -> None:
        text = REVIEW_PACKET_PATH.read_text(encoding="utf-8")

        for field in REQUIRED_REVIEW_FIELDS:
            with self.subTest(field=field):
                self.assertIn(REVIEW_FIELD_MARKERS[field], text)

    def test_handoff_template_includes_required_headings(self) -> None:
        self.assertTrue(HANDOFF_PATH.exists(), f"Missing required artifact: {HANDOFF_PATH}")

        text = HANDOFF_PATH.read_text(encoding="utf-8")

        for heading in (
            "# Agent Handoff",
            "## Handoff Metadata",
            "## Completed Artifacts",
            "## Decision Summary",
            "## Unresolved Questions",
            "## Evidence Dependencies",
            "## Assumptions",
            "## Risks",
            "## Next Actions",
        ):
            with self.subTest(heading=heading):
                self.assertIn(heading, text)

    def test_handoff_template_represents_required_contract_fields(self) -> None:
        text = HANDOFF_PATH.read_text(encoding="utf-8")

        for field in REQUIRED_HANDOFF_FIELDS:
            with self.subTest(field=field):
                self.assertIn(HANDOFF_FIELD_MARKERS[field], text)


if __name__ == "__main__":
    unittest.main()
