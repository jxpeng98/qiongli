from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

import yaml

from qiongli.subject_materializer import (
    MaterializeOptions,
    SubjectMaterializationError,
    materialize_subject_package,
)


REPO_ROOT = Path(__file__).resolve().parents[1]


class SubjectMaterializerTests(unittest.TestCase):
    def test_materializes_economics_full_package_with_overlays(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            out = Path(tmp_dir) / "qiongli-workflow"

            materialize_subject_package(
                MaterializeOptions(
                    source=REPO_ROOT,
                    out=out,
                    subject="economics",
                    flavor="full",
                )
            )

            self.assertEqual((out / "SUBJECT").read_text(encoding="utf-8").strip(), "economics")
            skill_text = (out / "SKILL.md").read_text(encoding="utf-8")
            self.assertIn("## Economics Workflow Map", skill_text)
            self.assertIn("### 1. Research Framing", skill_text)
            self.assertIn("### 3. Identification and Study Design", skill_text)

            registry = yaml.safe_load((out / "skills" / "registry.yaml").read_text(encoding="utf-8"))
            registry_ids = {entry["id"] for entry in registry["skills"]}
            self.assertIn("manuscript-architect", registry_ids)
            self.assertIn("stats-engine", registry_ids)
            self.assertIn("econ-identification-auditor", registry_ids)
            self.assertNotIn("prisma-checker", registry_ids)

            manuscript = (out / "skills" / "F_writing" / "manuscript-architect.md").read_text(encoding="utf-8")
            self.assertIn("## Economics Overlay", manuscript)
            self.assertIn("identification strategy", manuscript)

            stats = (out / "skills" / "I_code" / "stats-engine.md").read_text(encoding="utf-8")
            self.assertIn("## Quality Bar", stats)
            self.assertIn("clustered standard errors", stats)
            self.assertNotIn("模型选择有统计学依据", stats)
            self.assertIn("## Common Pitfalls", stats)
            self.assertIn("naive TWFE", stats)

    def test_materialization_writes_subject_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            out = Path(tmp_dir) / "qiongli-workflow"

            materialize_subject_package(
                MaterializeOptions(
                    source=REPO_ROOT,
                    out=out,
                    subject="economics",
                    flavor="full",
                    coverage="focused",
                )
            )

            manifest = json.loads((out / "SUBJECT_MANIFEST.json").read_text(encoding="utf-8"))
            self.assertEqual(manifest["subject"], "economics")
            self.assertEqual(manifest["coverage"], "focused")
            self.assertEqual(manifest["flavor"], "full")
            self.assertEqual(manifest["layers"], ["core", "economics"])

    def test_unknown_coverage_reports_clear_error(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            out = Path(tmp_dir) / "qiongli-workflow"

            with self.assertRaisesRegex(SubjectMaterializationError, "unsupported coverage"):
                materialize_subject_package(
                    MaterializeOptions(
                        source=REPO_ROOT,
                        out=out,
                        subject="economics",
                        flavor="full",
                        coverage="wide",
                    )
                )

    def test_materialized_economics_filters_profiles(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            out = Path(tmp_dir) / "qiongli-workflow"

            materialize_subject_package(
                MaterializeOptions(source=REPO_ROOT, out=out, subject="economics", flavor="full")
            )

            self.assertTrue((out / "skills" / "domain-profiles" / "economics.yaml").exists())
            self.assertFalse((out / "skills" / "domain-profiles" / "cs-ai.yaml").exists())
            self.assertFalse((out / "skills" / "domain-profiles" / "biomedical.yaml").exists())
            self.assertFalse((out / "skills" / "domain-profiles" / "psychology.yaml").exists())
            self.assertFalse((out / "skills" / "domain-profiles" / "finance.yaml").exists())
            self.assertTrue((out / "venue-profiles" / "aer.yaml").exists())
            self.assertFalse((out / "venue-profiles" / "neurips.yaml").exists())

    def test_materializes_core_desktop_package_under_file_budget(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            out = Path(tmp_dir) / "qiongli"

            materialize_subject_package(
                MaterializeOptions(source=REPO_ROOT, out=out, subject="core", flavor="desktop")
            )

            files = [path for path in out.rglob("*") if path.is_file()]
            self.assertLessEqual(len(files), 180)
            self.assertEqual((out / "SUBJECT").read_text(encoding="utf-8").strip(), "core")
            self.assertTrue((out / "skills" / "registry.yaml").exists())
            self.assertFalse((out / "skills" / "F_writing" / "manuscript-architect.md").exists())

    def test_replace_sections_requires_declared_base_section(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            (root / "qiongli-workflow" / "skills" / "I_code").mkdir(parents=True)
            (root / "qiongli-workflow" / "skills" / "I_code" / "stats-engine.md").write_text(
                "# Stats Engine\n\n## Present\nbase\n",
                encoding="utf-8",
            )
            (root / "qiongli-workflow" / "VERSION").write_text("v9.9.9\n", encoding="utf-8")
            for rel in ("workflows", "references", "templates", "standards", "roles", "venue-profiles"):
                (root / "qiongli-workflow" / rel).mkdir(parents=True)
            (root / "qiongli-workflow" / "SKILL.md").write_text("---\nname: qiongli\n---\n", encoding="utf-8")
            (root / "qiongli-workflow" / "skills-core.md").write_text("# Core\n", encoding="utf-8")
            (root / "qiongli-workflow" / "skills-summary.md").write_text("# Summary\n", encoding="utf-8")
            (root / "skills").mkdir()
            (root / "skills" / "registry.yaml").write_text(
                "\n".join(
                    [
                        "skills:",
                        "  - id: stats-engine",
                        "    stage: I_code",
                        "    version: \"9.9.9\"",
                        "    file: skills/I_code/stats-engine.md",
                        "    canonical: true",
                        "    summary: Stats",
                        "    display_name: Stats",
                        "    when_to_use: Stats",
                        "    summary_zh: Stats",
                        "    display_name_zh: Stats",
                        "    when_to_use_zh: Stats",
                        "    inputs: [AnalysisPlan]",
                        "    outputs: [StatsReport]",
                    ]
                ),
                encoding="utf-8",
            )
            (root / "subjects" / "demo" / "overlays" / "skills").mkdir(parents=True)
            (root / "subjects" / "demo" / "overlays" / "skills" / "stats-engine.md").write_text(
                "## Missing\nreplacement\n",
                encoding="utf-8",
            )
            (root / "subjects" / "catalog.yaml").write_text(
                "\n".join(
                    [
                        "subjects:",
                        "  demo:",
                        "    display_name: Demo",
                        "    package_goal: Demo",
                        "    skill_overrides:",
                        "      - skill: stats-engine",
                        "        overlay: overlays/skills/stats-engine.md",
                        "        mode: replace_sections",
                        "        sections: [Missing]",
                        "    skill_groups:",
                        "      - order: 1",
                        "        heading: Demo",
                        "        subheading: Demo",
                        "        skill_refs: [stats-engine]",
                    ]
                ),
                encoding="utf-8",
            )

            with self.assertRaisesRegex(SubjectMaterializationError, "Missing"):
                materialize_subject_package(
                    MaterializeOptions(source=root, out=root / "out", subject="demo", flavor="full")
                )


if __name__ == "__main__":
    unittest.main()
