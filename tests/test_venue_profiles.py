from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from qiongli.source_layout import RepoLayout

import yaml

from scripts.audit_venue_profiles import audit_venue_profile


REPO_ROOT = Path(__file__).resolve().parents[1]


class VenueProfileTests(unittest.TestCase):
    def test_initial_venue_profiles_exist_and_are_valid(self) -> None:
        profile_dir = RepoLayout(REPO_ROOT).venue_profiles
        for venue_id in ("chi", "acl", "neurips", "nature", "jama", "aom"):
            with self.subTest(venue_id=venue_id):
                path = profile_dir / f"{venue_id}.yaml"
                self.assertTrue(path.exists(), f"Missing {path}")
                result = audit_venue_profile(path)
                self.assertEqual([], result.errors)

    def test_invalid_profile_missing_evidence_standards_fails(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            profile = Path(tmp_dir) / "bad.yaml"
            profile.write_text(
                yaml.safe_dump(
                    {
                        "venue_id": "bad",
                        "community": "Example",
                        "article_types": ["article"],
                        "contribution_expectations": ["novelty"],
                        "methods_expectations": ["soundness"],
                        "writing_style": ["clear"],
                        "common_reviewer_objections": ["unclear contribution"],
                        "formatting_constraints": ["none"],
                        "required_reporting_standards": ["none"],
                    },
                    sort_keys=False,
                ),
                encoding="utf-8",
            )

            result = audit_venue_profile(profile)

        self.assertIn("missing required field: evidence_standards", result.errors)


if __name__ == "__main__":
    unittest.main()
