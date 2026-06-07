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
        self.assertIn("Economics and finance method packs", scorecard)
        self.assertIn("Offline eval expansion", scorecard)


if __name__ == "__main__":
    unittest.main()
