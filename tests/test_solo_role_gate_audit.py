from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from scripts.audit_solo_role_gates import audit_solo_role_gates


def write_json(path: Path, payload: dict[str, object]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2), encoding="utf-8")


class SoloRoleGateAuditTests(unittest.TestCase):
    def test_solo_codex_writing_without_claim_map_fails(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            write_json(
                root / "runs" / "codex-writing.json",
                {
                    "run_id": "codex-writing",
                    "execution_mode": "solo_codex",
                    "controller": "codex",
                    "primary_agent": "codex",
                    "task_type": "writing",
                    "verification_status": "passed",
                    "artifacts_written": ["manuscript/draft.md"],
                },
            )

            errors = audit_solo_role_gates(root)

        joined = "\n".join(errors).lower()
        self.assertIn("codex-writing", joined)
        self.assertIn("claim map", joined)

    def test_solo_claude_code_without_implementation_intent_fails(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            write_json(
                root / "runs" / "claude-code.json",
                {
                    "run_id": "claude-code",
                    "execution_mode": "solo_claude",
                    "controller": "claude",
                    "primary_agent": "claude",
                    "task_type": "code",
                    "verification_status": "passed",
                    "artifacts_written": ["code/change.patch"],
                },
            )

            errors = audit_solo_role_gates(root)

        joined = "\n".join(errors).lower()
        self.assertIn("claude-code", joined)
        self.assertIn("implementation intent", joined)

    def test_task_run_solo_codex_writing_without_claim_map_fails(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            write_json(
                root / "runs" / "task-run-codex-writing.json",
                {
                    "run_id": "task-run-codex-writing",
                    "execution_mode": "solo",
                    "controller": "codex",
                    "primary_agent": "codex",
                    "task_type": "writing",
                    "verification_status": "passed",
                    "artifacts_written": ["manuscript/draft.md"],
                },
            )

            errors = audit_solo_role_gates(root)

        joined = "\n".join(errors).lower()
        self.assertIn("task-run-codex-writing", joined)
        self.assertIn("claim map", joined)

    def test_task_run_solo_claude_code_without_implementation_intent_fails(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            write_json(
                root / "runs" / "task-run-claude-code.json",
                {
                    "run_id": "task-run-claude-code",
                    "execution_mode": "solo",
                    "controller": "claude",
                    "primary_agent": "claude",
                    "task_type": "code",
                    "verification_status": "passed",
                    "artifacts_written": ["code/change.patch"],
                },
            )

            errors = audit_solo_role_gates(root)

        joined = "\n".join(errors).lower()
        self.assertIn("task-run-claude-code", joined)
        self.assertIn("implementation intent", joined)

    def test_solo_role_gates_off_skips_role_specific_artifact_gates(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            write_json(
                root / "runs" / "codex-writing-experiment.json",
                {
                    "run_id": "codex-writing-experiment",
                    "execution_mode": "solo",
                    "controller": "codex",
                    "primary_agent": "codex",
                    "solo_role_gates": "off",
                    "task_type": "writing",
                    "verification_status": "passed",
                    "artifacts_written": ["manuscript/draft.md"],
                },
            )

            errors = audit_solo_role_gates(root)

        self.assertEqual([], errors)

    def test_duo_run_without_handoff_fails(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            write_json(
                root / "runs" / "duo-run.json",
                {
                    "run_id": "duo-run",
                    "execution_mode": "duo",
                    "controller": "codex",
                    "primary_agent": "codex",
                    "verification_status": "passed",
                    "artifacts_written": ["reviews/duo-review.json"],
                },
            )

            errors = audit_solo_role_gates(root)

        joined = "\n".join(errors).lower()
        self.assertIn("duo-run", joined)
        self.assertIn("handoff", joined)

    def test_reviewer_blocker_with_final_status_passed_fails(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            write_json(
                root / "runs" / "run-with-blocker.json",
                {
                    "run_id": "run-with-blocker",
                    "execution_mode": "solo_codex",
                    "controller": "codex",
                    "primary_agent": "codex",
                    "verification_status": "passed",
                    "artifacts_written": ["reviews/review.json"],
                },
            )
            write_json(
                root / "reviews" / "review.json",
                {
                    "reviewer_agent": "claude",
                    "reviewed_run_id": "run-with-blocker",
                    "review_status": "blocked",
                    "blocking_issues": ["missing evidence"],
                    "required_revisions": [],
                    "verification_evidence": [],
                },
            )

            errors = audit_solo_role_gates(root)

        joined = "\n".join(errors).lower()
        self.assertIn("run-with-blocker", joined)
        self.assertIn("reviewer blocker", joined)
        self.assertIn("passed", joined)

    def test_missing_verification_status_fails(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            write_json(
                root / "runs" / "missing-status.json",
                {
                    "run_id": "missing-status",
                    "execution_mode": "solo_claude",
                    "controller": "claude",
                    "primary_agent": "claude",
                    "artifacts_written": [],
                },
            )

            errors = audit_solo_role_gates(root)

        joined = "\n".join(errors).lower()
        self.assertIn("missing-status", joined)
        self.assertIn("verification_status", joined)

    def test_run_packet_template_is_not_audited_as_real_run(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            write_json(
                root / "templates" / "agent-run-packet.json",
                {
                    "run_id": "",
                    "execution_mode": "solo_codex",
                    "controller": "codex",
                    "primary_agent": "codex",
                    "task_id": "F3",
                    "artifacts_written": [],
                },
            )

            errors = audit_solo_role_gates(root)

        self.assertEqual([], errors)

    def test_outside_root_artifact_path_does_not_satisfy_gate(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            tmp_root = Path(tmp_dir)
            root = tmp_root / "project"
            root.mkdir()
            outside = tmp_root / "outside"
            outside.mkdir()
            (outside / "writing-claim-map.md").write_text(
                "# Outside Claim Map\n",
                encoding="utf-8",
            )
            write_json(
                root / "runs" / "codex-writing.json",
                {
                    "run_id": "codex-writing",
                    "execution_mode": "solo_codex",
                    "controller": "codex",
                    "primary_agent": "codex",
                    "task_type": "writing",
                    "verification_status": "passed",
                    "artifacts_written": ["../outside/writing-claim-map.md"],
                },
            )

            errors = audit_solo_role_gates(root)

        joined = "\n".join(errors).lower()
        self.assertIn("codex-writing", joined)
        self.assertIn("outside audit root", joined)
        self.assertIn("claim map", joined)

    def test_complete_controller_mode_artifacts_pass(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            write_json(
                root / "runs" / "codex-writing.json",
                {
                    "run_id": "codex-writing",
                    "execution_mode": "solo_codex",
                    "controller": "codex",
                    "primary_agent": "codex",
                    "task_type": "writing",
                    "verification_status": "passed",
                    "artifacts_written": ["writing/writing-claim-map.md"],
                },
            )
            (root / "writing").mkdir()
            (root / "writing" / "writing-claim-map.md").write_text(
                "# Writing Claim Map\n",
                encoding="utf-8",
            )
            write_json(
                root / "runs" / "claude-code.json",
                {
                    "run_id": "claude-code",
                    "execution_mode": "solo_claude",
                    "controller": "claude",
                    "primary_agent": "claude",
                    "task_type": "code",
                    "verification_status": "passed",
                    "artifacts_written": ["code/implementation-intent.md"],
                },
            )
            (root / "code").mkdir()
            (root / "code" / "implementation-intent.md").write_text(
                "# Implementation Intent\n",
                encoding="utf-8",
            )
            write_json(
                root / "runs" / "duo-run.json",
                {
                    "run_id": "duo-run",
                    "execution_mode": "duo",
                    "controller": "codex",
                    "primary_agent": "codex",
                    "verification_status": "passed",
                    "artifacts_written": ["handoffs/agent-handoff.md"],
                },
            )
            (root / "handoffs").mkdir()
            (root / "handoffs" / "agent-handoff.md").write_text(
                "# Agent Handoff\n",
                encoding="utf-8",
            )

            errors = audit_solo_role_gates(root)

        self.assertEqual([], errors)


if __name__ == "__main__":
    unittest.main()
