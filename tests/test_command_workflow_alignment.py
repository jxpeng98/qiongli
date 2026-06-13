from __future__ import annotations

import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from qiongli.source_layout import RepoLayout


REPO_ROOT = Path(__file__).resolve().parents[1]
LAYOUT = RepoLayout(REPO_ROOT)
WORKFLOW_DIR = LAYOUT.workflow / "workflows"


class CommandWorkflowAlignmentTests(unittest.TestCase):
    def materialize_payload_root(self, tmp_dir: str) -> Path:
        out = Path(tmp_dir) / "dist-source"
        result = subprocess.run(
            [
                sys.executable,
                "scripts/materialize_distribution_payloads.py",
                "--target",
                "plugin",
                "--out",
                str(out),
                "--force",
            ],
            cwd=REPO_ROOT,
            text=True,
            capture_output=True,
            check=False,
        )
        self.assertEqual(result.returncode, 0, msg=result.stderr + result.stdout)
        return out

    def test_every_workflow_has_thin_plugin_command(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            payload_root = self.materialize_payload_root(tmp_dir)
            command_dir = payload_root / "plugins" / "qiongli" / "commands"
            workflow_names = sorted(path.name for path in WORKFLOW_DIR.glob("*.md"))
            command_names = sorted(path.name for path in command_dir.glob("*.md"))

        self.assertEqual(command_names, workflow_names)

    def test_commands_reference_workflow_skill_and_matching_workflow(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            payload_root = self.materialize_payload_root(tmp_dir)
            command_dir = payload_root / "plugins" / "qiongli" / "commands"
            for workflow in sorted(WORKFLOW_DIR.glob("*.md")):
                with self.subTest(command=workflow.name):
                    command = command_dir / workflow.name
                    text = command.read_text(encoding="utf-8")
                    nonempty_lines = [line for line in text.splitlines() if line.strip()]

                    self.assertLessEqual(len(nonempty_lines), 15)
                    self.assertIn("qiongli-workflow", text)
                    self.assertIn(f"workflows/{workflow.name}", text)

    def test_platform_agent_workflows_match_canonical_workflows(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            payload_root = self.materialize_payload_root(tmp_dir)
            platform_agent_workflow_dir = payload_root / ".agent" / "workflows"
            for workflow in sorted(WORKFLOW_DIR.glob("*.md")):
                platform_workflow = platform_agent_workflow_dir / workflow.name
                with self.subTest(workflow=workflow.name):
                    self.assertTrue(
                        platform_workflow.exists(),
                        msg=f"missing platform agent workflow copy: {platform_workflow}",
                    )
                    self.assertEqual(
                        workflow.read_text(encoding="utf-8"),
                        platform_workflow.read_text(encoding="utf-8"),
                    )


if __name__ == "__main__":
    unittest.main()
