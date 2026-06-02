from __future__ import annotations

import unittest
from pathlib import Path

from qiongli.source_layout import RepoLayout


REPO_ROOT = Path(__file__).resolve().parents[1]
WORKFLOW_DIR = RepoLayout(REPO_ROOT).workflow / "workflows"
COMMAND_DIR = REPO_ROOT / "plugins" / "qiongli" / "commands"


class CommandWorkflowAlignmentTests(unittest.TestCase):
    def test_every_workflow_has_thin_plugin_command(self) -> None:
        workflow_names = sorted(path.name for path in WORKFLOW_DIR.glob("*.md"))
        command_names = sorted(path.name for path in COMMAND_DIR.glob("*.md"))

        self.assertEqual(command_names, workflow_names)

    def test_commands_reference_workflow_skill_and_matching_workflow(self) -> None:
        for workflow in sorted(WORKFLOW_DIR.glob("*.md")):
            with self.subTest(command=workflow.name):
                command = COMMAND_DIR / workflow.name
                text = command.read_text(encoding="utf-8")
                nonempty_lines = [line for line in text.splitlines() if line.strip()]

                self.assertLessEqual(len(nonempty_lines), 15)
                self.assertIn("qiongli-workflow", text)
                self.assertIn(f"workflows/{workflow.name}", text)


if __name__ == "__main__":
    unittest.main()
