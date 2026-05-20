from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from scripts.audit_agent_handoffs import audit_agent_handoffs


def write_json(path: Path, payload: dict[str, object]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2), encoding="utf-8")


class AgentHandoffAuditTests(unittest.TestCase):
    def test_duo_run_missing_handoff_fails(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            write_json(
                root / "runs" / "duo-run.json",
                {
                    "run_id": "duo-run",
                    "execution_mode": "duo",
                    "codex_position": "accept",
                    "claude_position": "revise",
                    "artifacts_written": ["reviews/disagreement-matrix.md"],
                },
            )
            (root / "reviews").mkdir()
            (root / "reviews" / "disagreement-matrix.md").write_text(
                "| issue_id | codex_position | claude_position | evidence_refs | risk_level | final_decision |\n"
                "| --- | --- | --- | --- | --- | --- |\n"
                "| D1 | accept | revise | E1 | high | revise |\n",
                encoding="utf-8",
            )

            errors = audit_agent_handoffs(root)

        self.assertIn("duo-run", "\n".join(errors))
        self.assertIn("handoff", "\n".join(errors).lower())

    def test_duo_conflict_missing_disagreement_artifact_fails(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            write_json(
                root / "runs" / "duo-run.json",
                {
                    "run_id": "duo-run",
                    "execution_mode": "duo",
                    "codex_position": "accept",
                    "claude_position": "reject",
                    "artifacts_written": ["handoffs/agent-handoff.md"],
                },
            )
            (root / "handoffs").mkdir()
            (root / "handoffs" / "agent-handoff.md").write_text(
                "# Agent Handoff\n\n## Handoff Metadata\n- from_agent: codex\n- to_agent: claude\n",
                encoding="utf-8",
            )

            errors = audit_agent_handoffs(root)

        self.assertIn("duo-run", "\n".join(errors))
        self.assertIn("disagreement", "\n".join(errors).lower())

    def test_duo_nested_conflict_without_artifacts_fails(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            write_json(
                root / "runs" / "duo-run.json",
                {
                    "run_id": "duo-run",
                    "execution_mode": "duo",
                    "disagreements": [
                        {
                            "issue_id": "D1",
                            "codex_position": "accept",
                            "claude_position": "revise",
                        }
                    ],
                    "artifacts_written": [],
                },
            )

            errors = audit_agent_handoffs(root)

        joined = "\n".join(errors).lower()
        self.assertIn("duo-run", joined)
        self.assertIn("handoff", joined)
        self.assertIn("disagreement", joined)

    def test_duo_conflict_with_handoff_and_disagreement_passes(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            write_json(
                root / "runs" / "duo-run.json",
                {
                    "run_id": "duo-run",
                    "execution_mode": "duo",
                    "codex_position": "accept",
                    "claude_position": "revise",
                    "artifacts_written": [
                        "handoffs/agent-handoff.md",
                        "reviews/disagreement-matrix.md",
                    ],
                },
            )
            (root / "handoffs").mkdir()
            (root / "handoffs" / "agent-handoff.md").write_text(
                "# Agent Handoff\n\n## Handoff Metadata\n- from_agent: codex\n- to_agent: claude\n",
                encoding="utf-8",
            )
            (root / "reviews").mkdir()
            (root / "reviews" / "disagreement-matrix.md").write_text(
                "| issue_id | codex_position | claude_position | evidence_refs | risk_level | final_decision |\n"
                "| --- | --- | --- | --- | --- | --- |\n"
                "| D1 | accept | revise | E1 | high | revise |\n",
                encoding="utf-8",
            )

            errors = audit_agent_handoffs(root)

        self.assertEqual([], errors)


if __name__ == "__main__":
    unittest.main()
