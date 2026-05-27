from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from qiongli.subject_materializer import SubjectCatalogError, load_subject_catalog, validate_subject_catalog


REPO_ROOT = Path(__file__).resolve().parents[1]


class SubjectCatalogTests(unittest.TestCase):
    def test_core_and_economics_catalog_groups_are_ordered(self) -> None:
        catalog = load_subject_catalog(REPO_ROOT)

        self.assertEqual(["core", "economics", "economics-accounting"], sorted(catalog["subjects"]))
        economics = catalog["subjects"]["economics"]
        self.assertEqual("core", economics["extends"])
        self.assertIn("skill_groups", economics)
        self.assertNotIn("ids", economics)

        orders = [group["order"] for group in economics["skill_groups"]]
        self.assertEqual(list(range(1, len(orders) + 1)), orders)
        for group in economics["skill_groups"]:
            self.assertIsInstance(group["heading"], str)
            self.assertTrue(group["heading"].strip())
            self.assertIsInstance(group["subheading"], str)
            self.assertTrue(group["subheading"].strip())
            self.assertTrue(group["skill_refs"])

    def test_catalog_skill_refs_exist_in_registry_or_subject_registry(self) -> None:
        catalog = validate_subject_catalog(REPO_ROOT)

        economics = catalog.subjects["economics"]
        self.assertIn("manuscript-architect", economics.skill_refs)
        self.assertIn("stats-engine", economics.skill_refs)
        self.assertIn("econ-identification-auditor", economics.skill_refs)

        composite = catalog.subjects["economics-accounting"]
        self.assertIn("econ-identification-auditor", composite.skill_refs)
        self.assertIn("accounting-measurement-auditor", composite.skill_refs)

    def test_invalid_group_order_reports_clear_error(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            (root / "subjects").mkdir()
            (root / "skills").mkdir()
            (root / "skills" / "registry.yaml").write_text("skills: []\n", encoding="utf-8")
            (root / "subjects" / "catalog.yaml").write_text(
                "\n".join(
                    [
                        "subjects:",
                        "  core:",
                        "    display_name: Core",
                        "    package_goal: Core",
                        "    skill_groups:",
                        "      - order: 2",
                        "        heading: Late",
                        "        subheading: Broken",
                        "        skill_refs: []",
                    ]
                ),
                encoding="utf-8",
            )

            with self.assertRaisesRegex(SubjectCatalogError, "orders must be consecutive"):
                validate_subject_catalog(root)


if __name__ == "__main__":
    unittest.main()
