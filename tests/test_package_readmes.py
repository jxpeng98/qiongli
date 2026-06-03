from __future__ import annotations

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
PACKAGE_READMES = (
    REPO_ROOT / "README_PYPI.md",
    REPO_ROOT / "packages" / "npm-qiongli" / "README.md",
)
OFFICIAL_SUBJECTS = (
    "core",
    "economics",
    "accounting",
    "business",
    "finance",
    "political-economy",
    "geoeconomics",
    "economics-accounting",
)
DESKTOP_SUBJECTS = (
    "core",
    "economics",
    "business",
    "finance",
    "political-economy",
    "geoeconomics",
    "economics-accounting",
)


class PackageReadmeTests(unittest.TestCase):
    def test_registry_readmes_document_current_subjects(self) -> None:
        for readme_path in PACKAGE_READMES:
            with self.subTest(readme=readme_path.relative_to(REPO_ROOT).as_posix()):
                text = readme_path.read_text(encoding="utf-8")
                for subject in OFFICIAL_SUBJECTS:
                    self.assertIn(subject, text)
                for subject in DESKTOP_SUBJECTS:
                    self.assertIn(subject, text)
                self.assertIn("no standalone accounting Desktop ZIP", text)


if __name__ == "__main__":
    unittest.main()
