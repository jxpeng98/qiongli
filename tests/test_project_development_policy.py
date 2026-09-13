from __future__ import annotations

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]


class ProjectDevelopmentPolicyTests(unittest.TestCase):
    def test_retired_trellis_entrypoints_stay_removed(self) -> None:
        for pattern in (".agents/skills/trellis-*", ".codex/agents/trellis-*"):
            with self.subTest(pattern=pattern):
                self.assertEqual(list(REPO_ROOT.glob(pattern)), [])
        for name in (
            "session-start.py",
            "inject-workflow-state.py",
            "inject-subagent-context.py",
        ):
            with self.subTest(hook=name):
                self.assertFalse((REPO_ROOT / ".codex/hooks" / name).exists())
        for relative in ("AGENTS.md", ".trellis/workflow.md"):
            with self.subTest(document=relative):
                text = (REPO_ROOT / relative).read_text(encoding="utf-8")
                self.assertNotIn("[workflow-state:", text)
                self.assertNotIn("<!-- TRELLIS:START -->", text)


if __name__ == "__main__":
    unittest.main()
