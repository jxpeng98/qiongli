from __future__ import annotations

import json
import unittest
from pathlib import Path

import yaml

from qiongli.source_layout import RepoLayout


REPO_ROOT = Path(__file__).resolve().parents[1]
LAYOUT = RepoLayout(REPO_ROOT)

CONTRACT_PATH = LAYOUT.standards / "worker-orchestration-contract.yaml"
RUN_PACKET_PATH = LAYOUT.templates / "worker-run-packet.json"
REVIEW_PACKET_PATH = LAYOUT.templates / "worker-review-packet.md"
MERGE_REPORT_PATH = LAYOUT.templates / "worker-merge-report.md"

ORCHESTRATION_MODES = {"none", "delegated_workers", "review_swarm"}
PLATFORM_ADAPTERS = {"generic_prompt", "codex_subagent", "claude_cowork"}
WORKER_STATUSES = {"planned", "running", "passed", "failed", "blocked", "skipped"}
MERGE_POLICIES = {
    "synthesize_with_conflict_matrix",
    "consensus_then_gaps",
    "controller_adjudication",
}

REQUIRED_WORKER_PLAN_FIELDS = {
    "orchestration_mode",
    "controller_runtime",
    "platform_adapter",
    "task_id",
    "paper_type",
    "topic",
    "workers",
    "merge",
    "final_review",
}

REQUIRED_WORKER_FIELDS = {
    "id",
    "goal",
    "functional_role",
    "required_skills",
    "allowed_artifacts",
    "forbidden_artifacts",
    "review_required",
    "stop_conditions",
}

REQUIRED_MERGE_FIELDS = {
    "agent",
    "policy",
    "output_artifacts",
}

REQUIRED_FINAL_REVIEW_FIELDS = {
    "reviewer",
    "gate",
}

REQUIRED_WORKER_RUN_PACKET_FIELDS = {
    "run_id",
    "worker_id",
    "controller_runtime",
    "platform_adapter",
    "task_id",
    "paper_type",
    "topic",
    "goal",
    "functional_role",
    "required_skills",
    "required_mcp",
    "allowed_artifacts",
    "forbidden_artifacts",
    "artifacts_read",
    "artifacts_written",
    "warnings",
    "blocking_issues",
    "status",
    "confidence",
}

REQUIRED_WORKER_REVIEW_FIELDS = {
    "reviewer",
    "worker_id",
    "run_id",
    "review_status",
    "findings",
    "blocking_issues",
    "required_revisions",
    "verification_evidence",
    "verdict",
}

REVIEW_FIELD_MARKERS = {
    "reviewer": "- reviewer:",
    "worker_id": "- worker_id:",
    "run_id": "- run_id:",
    "review_status": "- review_status:",
    "findings": "## Findings",
    "blocking_issues": "## Blocking Issues",
    "required_revisions": "## Required Revisions",
    "verification_evidence": "- verification_evidence:",
    "verdict": "- verdict:",
}

MERGE_FIELD_MARKERS = {
    "worker_plan_run_id": "- worker_plan_run_id:",
    "agent": "- agent:",
    "policy": "- policy:",
    "output_artifacts": "- output_artifacts:",
    "worker_status_table": "## Worker Status Table",
    "accepted_worker_outputs": "## Accepted Worker Outputs",
    "rejected_or_blocked_worker_outputs": "## Rejected Or Blocked Worker Outputs",
    "conflict_summary": "## Conflict Summary",
    "gap_summary": "## Gap Summary",
    "controller_adjudication": "## Controller Adjudication",
    "canonical_output_update_plan": "## Canonical Output Update Plan",
    "final_reviewer": "- final_reviewer:",
    "final_review_gate": "- final_review_gate:",
}


class WorkerOrchestrationContractTests(unittest.TestCase):
    def test_contract_defines_enums_and_required_fields(self) -> None:
        self.assertTrue(CONTRACT_PATH.exists(), f"Missing {CONTRACT_PATH}")
        contract = yaml.safe_load(CONTRACT_PATH.read_text(encoding="utf-8")) or {}
        self.assertIsInstance(contract, dict)

        self.assertEqual("1.0.0", contract.get("contract_version"))
        for key in (
            "orchestration_modes",
            "platform_adapters",
            "worker_statuses",
            "merge_policies",
            "required_worker_plan_fields",
            "required_worker_fields",
            "required_merge_fields",
            "required_final_review_fields",
            "required_worker_run_packet_fields",
            "required_worker_review_fields",
        ):
            with self.subTest(key=key):
                self.assertIsInstance(contract.get(key), list)

        self.assertEqual(ORCHESTRATION_MODES, set(contract.get("orchestration_modes", [])))
        self.assertEqual(PLATFORM_ADAPTERS, set(contract.get("platform_adapters", [])))
        self.assertEqual(WORKER_STATUSES, set(contract.get("worker_statuses", [])))
        self.assertEqual(MERGE_POLICIES, set(contract.get("merge_policies", [])))
        self.assertEqual(REQUIRED_WORKER_PLAN_FIELDS, set(contract.get("required_worker_plan_fields", [])))
        self.assertEqual(REQUIRED_WORKER_FIELDS, set(contract.get("required_worker_fields", [])))
        self.assertEqual(REQUIRED_MERGE_FIELDS, set(contract.get("required_merge_fields", [])))
        self.assertEqual(REQUIRED_FINAL_REVIEW_FIELDS, set(contract.get("required_final_review_fields", [])))
        self.assertEqual(
            REQUIRED_WORKER_RUN_PACKET_FIELDS,
            set(contract.get("required_worker_run_packet_fields", [])),
        )
        self.assertEqual(
            REQUIRED_WORKER_REVIEW_FIELDS,
            set(contract.get("required_worker_review_fields", [])),
        )

    def test_worker_run_packet_has_safe_defaults(self) -> None:
        self.assertTrue(RUN_PACKET_PATH.exists(), f"Missing {RUN_PACKET_PATH}")
        packet = json.loads(RUN_PACKET_PATH.read_text(encoding="utf-8"))

        self.assertEqual(REQUIRED_WORKER_RUN_PACKET_FIELDS, set(packet))
        self.assertEqual("planned", packet["status"])
        self.assertEqual("generic_prompt", packet["platform_adapter"])
        self.assertEqual(0.0, packet["confidence"])
        for key in ("required_skills", "required_mcp", "allowed_artifacts", "forbidden_artifacts", "artifacts_read", "artifacts_written", "warnings", "blocking_issues"):
            self.assertEqual([], packet[key], key)

    def test_worker_review_template_has_required_headings(self) -> None:
        self.assertTrue(REVIEW_PACKET_PATH.exists(), f"Missing {REVIEW_PACKET_PATH}")
        text = REVIEW_PACKET_PATH.read_text(encoding="utf-8")
        for heading in (
            "# Worker Review Packet",
            "## Review Metadata",
            "## Findings",
            "## Blocking Issues",
            "## Required Revisions",
            "## Verification Evidence",
            "## Verdict",
        ):
            with self.subTest(heading=heading):
                self.assertIn(heading, text)

    def test_worker_review_template_represents_required_fields(self) -> None:
        text = REVIEW_PACKET_PATH.read_text(encoding="utf-8")

        for field in REQUIRED_WORKER_REVIEW_FIELDS:
            with self.subTest(field=field):
                self.assertIn(REVIEW_FIELD_MARKERS[field], text)

    def test_worker_merge_template_has_required_headings(self) -> None:
        self.assertTrue(MERGE_REPORT_PATH.exists(), f"Missing {MERGE_REPORT_PATH}")
        text = MERGE_REPORT_PATH.read_text(encoding="utf-8")
        for heading in (
            "# Worker Merge Report",
            "## Worker Status Table",
            "## Accepted Worker Outputs",
            "## Rejected Or Blocked Worker Outputs",
            "## Conflict Summary",
            "## Gap Summary",
            "## Controller Adjudication",
            "## Canonical Output Update Plan",
            "## Final Review Request",
        ):
            with self.subTest(heading=heading):
                self.assertIn(heading, text)

    def test_worker_merge_template_represents_required_fields(self) -> None:
        text = MERGE_REPORT_PATH.read_text(encoding="utf-8")

        for field, marker in MERGE_FIELD_MARKERS.items():
            with self.subTest(field=field):
                self.assertIn(marker, text)


if __name__ == "__main__":
    unittest.main()
