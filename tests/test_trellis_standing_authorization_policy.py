from __future__ import annotations

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
POLICY_SURFACES = (
    REPO_ROOT / ".trellis/workflow.md",
    REPO_ROOT / ".agents/skills/trellis-brainstorm/SKILL.md",
    REPO_ROOT / ".agents/skills/trellis-continue/SKILL.md",
    REPO_ROOT / ".codex/hooks/session-start.py",
)


class TrellisStandingAuthorizationPolicyTests(unittest.TestCase):
    def test_policy_surfaces_keep_both_authorization_paths(self) -> None:
        for path in POLICY_SURFACES:
            with self.subTest(path=path.relative_to(REPO_ROOT)):
                text = path.read_text(encoding="utf-8").lower()
                self.assertIn("scoped standing implementation authorization", text)
                self.assertIn("latest plan", text)
                self.assertIn("fresh user approval", text)

        brainstorm = POLICY_SURFACES[1].read_text(encoding="utf-8")
        self.assertNotIn(
            "Only a subsequent user message that explicitly approves the latest "
            "planning summary authorizes",
            brainstorm,
        )
        self.assertIn("user-owned product, scope, UX, compatibility, risk", brainstorm)


if __name__ == "__main__":
    unittest.main()
