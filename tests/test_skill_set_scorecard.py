from __future__ import annotations

import unittest
from pathlib import Path

import yaml

from qiongli.source_layout import RepoLayout


ROOT = Path(__file__).resolve().parents[1]
SCORECARD_PATH = ROOT / "docs" / "maintainer" / "skill-set-optimization-scorecard.md"
REGISTRY_PATH = RepoLayout(ROOT).skills / "registry.yaml"


class SkillSetScorecardTests(unittest.TestCase):
    def test_scorecard_records_registry_baseline_and_targets(self) -> None:
        scorecard = SCORECARD_PATH.read_text(encoding="utf-8")
        registry = yaml.safe_load(REGISTRY_PATH.read_text(encoding="utf-8"))
        skill_count = len(registry["skills"])

        self.assertIn(f"Canonical registered skills: {skill_count}", scorecard)
        self.assertIn("Executable Q1-Q4 semantic gates", scorecard)
        self.assertIn("Q1-Q4 semantic gate report", scorecard)
        self.assertIn("semantic_checks", scorecard)
        self.assertIn("quality-gate-report.md", scorecard)
        self.assertIn("Real-Agent Smoke Roadmap", scorecard)
        self.assertIn("maintainer-only opt-in", scorecard)
        self.assertIn("isolated HOME", scorecard)
        self.assertIn("CODEX_HOME", scorecard)
        self.assertIn("CLAUDE_CODE_HOME", scorecard)
        self.assertIn("ANTIGRAVITY_HOME", scorecard)
        self.assertIn("Stage C/F/G/I", scorecard)
        self.assertIn("not a default CI gate", scorecard)
        self.assertIn("Recommended Next Optimization", scorecard)
        self.assertIn("Economics and finance method packs", scorecard)
        self.assertIn("Offline eval expansion", scorecard)


if __name__ == "__main__":
    unittest.main()
