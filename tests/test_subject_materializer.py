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
                    coverage="focused",
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

    def test_default_economics_complete_package_keeps_core_coverage_and_overlays(self) -> None:
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

            manifest = json.loads((out / "SUBJECT_MANIFEST.json").read_text(encoding="utf-8"))
            self.assertEqual(manifest["coverage"], "complete")

            registry = yaml.safe_load((out / "skills" / "registry.yaml").read_text(encoding="utf-8"))
            registry_ids = {entry["id"] for entry in registry["skills"]}
            self.assertIn("prisma-checker", registry_ids)
            self.assertIn("citation-formatter", registry_ids)
            self.assertIn("econ-identification-auditor", registry_ids)

            self.assertTrue((out / "skills" / "domain-profiles" / "cs-ai.yaml").exists())
            self.assertTrue((out / "skills" / "domain-profiles" / "finance.yaml").exists())
            self.assertTrue((out / "skills" / "domain-profiles" / "economics.yaml").exists())
            self.assertTrue((out / "venue-profiles" / "neurips.yaml").exists())
            self.assertTrue((out / "venue-profiles" / "aer.yaml").exists())

            manuscript = (out / "skills" / "F_writing" / "manuscript-architect.md").read_text(encoding="utf-8")
            self.assertIn("## Economics Overlay", manuscript)
            stats = (out / "skills" / "I_code" / "stats-engine.md").read_text(encoding="utf-8")
            self.assertIn("clustered standard errors", stats)

    def test_economics_complete_contains_v2_method_depth(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            out = Path(tmp_dir) / "qiongli-workflow"

            materialize_subject_package(
                MaterializeOptions(
                    source=REPO_ROOT,
                    out=out,
                    subject="economics",
                    flavor="full",
                    coverage="complete",
                )
            )

            registry = yaml.safe_load((out / "skills" / "registry.yaml").read_text(encoding="utf-8"))
            registry_ids = {entry["id"] for entry in registry["skills"]}
            self.assertIn("econ-replication-package-auditor", registry_ids)

            self.assertTrue((out / "venue-profiles" / "econometrica.yaml").exists())
            self.assertTrue((out / "venue-profiles" / "jpe.yaml").exists())

            study_designer = (out / "skills" / "C_design" / "study-designer.md").read_text(encoding="utf-8")
            self.assertIn("## Economics Overlay", study_designer)
            robustness_planner = (out / "skills" / "C_design" / "robustness-planner.md").read_text(encoding="utf-8")
            self.assertIn("identification threat", robustness_planner)
            analysis_interpreter = (out / "skills" / "F_writing" / "analysis-interpreter.md").read_text(
                encoding="utf-8"
            )
            self.assertIn("economic magnitude", analysis_interpreter)
            replication_auditor = (out / "skills" / "I_code" / "econ-replication-package-auditor.md").read_text(
                encoding="utf-8"
            )
            self.assertTrue(replication_auditor.startswith("---\n"))
            self.assertIn("id: econ-replication-package-auditor", replication_auditor)
            self.assertIn("## Quality Bar", replication_auditor)

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

    def test_composite_manifest_lists_component_layers(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            out = Path(tmp_dir) / "qiongli-workflow"
            materialize_subject_package(
                MaterializeOptions(
                    source=REPO_ROOT,
                    out=out,
                    subject="economics-accounting",
                    flavor="full",
                    coverage="complete",
                )
            )
            manifest = json.loads((out / "SUBJECT_MANIFEST.json").read_text(encoding="utf-8"))
            self.assertEqual(
                manifest["layers"],
                ["core", "economics", "accounting", "economics-accounting"],
            )

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

    def test_custom_dir_appends_overlay_skill_and_profiles(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            custom_dir = root / "custom"
            (custom_dir / "overlays" / "skills").mkdir(parents=True)
            (custom_dir / "skills").mkdir(parents=True)
            (custom_dir / "domain-profiles").mkdir(parents=True)
            (custom_dir / "venue-profiles").mkdir(parents=True)
            (custom_dir / "subject.yaml").write_text(
                "\n".join(
                    [
                        "skill_refs: [custom-validity-auditor]",
                        "domain_profiles: [custom-ledger]",
                        "venue_profiles: [custom-journal]",
                        "skill_overrides:",
                        "  - skill: manuscript-architect",
                        "    overlay: overlays/skills/manuscript-architect.md",
                        "    mode: append",
                    ]
                ),
                encoding="utf-8",
            )
            (custom_dir / "skills" / "registry.yaml").write_text(
                "\n".join(
                    [
                        "skills:",
                        "  - id: custom-validity-auditor",
                        "    stage: C_design",
                        "    version: \"0.1.0\"",
                        "    file: skills/C_design/custom-validity-auditor.md",
                        "    canonical: false",
                        "    summary: Custom validity audit",
                        "    display_name: Custom Validity Auditor",
                        "    when_to_use: Custom validity checks",
                        "    inputs: [DesignSpec]",
                        "    outputs: [CustomAudit]",
                    ]
                ),
                encoding="utf-8",
            )
            (custom_dir / "skills" / "custom-validity-auditor.md").write_text(
                "# Custom Validity Auditor\n\nCustom audit body.\n",
                encoding="utf-8",
            )
            (custom_dir / "overlays" / "skills" / "manuscript-architect.md").write_text(
                "## Custom Overlay\n\nRequire local lab disclosure language.\n",
                encoding="utf-8",
            )
            (custom_dir / "domain-profiles" / "custom-ledger.yaml").write_text(
                "domain: custom-ledger\n",
                encoding="utf-8",
            )
            (custom_dir / "venue-profiles" / "custom-journal.yaml").write_text(
                "venue: custom-journal\n",
                encoding="utf-8",
            )
            out = root / "qiongli-workflow"

            materialize_subject_package(
                MaterializeOptions(
                    source=REPO_ROOT,
                    out=out,
                    subject="economics",
                    flavor="full",
                    coverage="focused",
                    custom_dir=custom_dir,
                )
            )

            registry = yaml.safe_load((out / "skills" / "registry.yaml").read_text(encoding="utf-8"))
            registry_ids = {entry["id"] for entry in registry["skills"]}
            self.assertIn("custom-validity-auditor", registry_ids)
            self.assertTrue((out / "skills" / "C_design" / "custom-validity-auditor.md").exists())
            self.assertTrue((out / "skills" / "domain-profiles" / "custom-ledger.yaml").exists())
            self.assertTrue((out / "venue-profiles" / "custom-journal.yaml").exists())
            manuscript = (out / "skills" / "F_writing" / "manuscript-architect.md").read_text(encoding="utf-8")
            self.assertIn("## Custom Overlay", manuscript)

    def test_custom_registry_duplicate_id_fails(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            custom_dir = root / "custom"
            (custom_dir / "skills").mkdir(parents=True)
            (custom_dir / "skills" / "registry.yaml").write_text(
                "\n".join(
                    [
                        "skills:",
                        "  - id: stats-engine",
                        "    stage: I_code",
                        "    version: \"0.1.0\"",
                        "    file: skills/I_code/stats-engine.md",
                        "    canonical: false",
                    ]
                ),
                encoding="utf-8",
            )

            with self.assertRaisesRegex(SubjectMaterializationError, "duplicate custom registry id"):
                materialize_subject_package(
                    MaterializeOptions(
                        source=REPO_ROOT,
                        out=root / "out",
                        subject="economics",
                        custom_dir=custom_dir,
                    )
                )

    def test_custom_unknown_skill_ref_fails(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            custom_dir = root / "custom"
            custom_dir.mkdir(parents=True)
            (custom_dir / "subject.yaml").write_text(
                "skill_refs: [missing-custom-skill]\n",
                encoding="utf-8",
            )

            with self.assertRaisesRegex(SubjectMaterializationError, "custom subject references unknown skills"):
                materialize_subject_package(
                    MaterializeOptions(
                        source=REPO_ROOT,
                        out=root / "out",
                        subject="economics",
                        custom_dir=custom_dir,
                    )
                )

    def test_custom_replace_sections_missing_section_fails(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            custom_dir = root / "custom"
            (custom_dir / "overlays" / "skills").mkdir(parents=True)
            (custom_dir / "subject.yaml").write_text(
                "\n".join(
                    [
                        "skill_overrides:",
                        "  - skill: manuscript-architect",
                        "    overlay: overlays/skills/manuscript-architect.md",
                        "    mode: replace_sections",
                        "    sections: [Does Not Exist]",
                    ]
                ),
                encoding="utf-8",
            )
            (custom_dir / "overlays" / "skills" / "manuscript-architect.md").write_text(
                "## Does Not Exist\n\nReplacement.\n",
                encoding="utf-8",
            )

            with self.assertRaisesRegex(SubjectMaterializationError, "base skill missing replace_sections section"):
                materialize_subject_package(
                    MaterializeOptions(
                        source=REPO_ROOT,
                        out=root / "out",
                        subject="economics",
                        custom_dir=custom_dir,
                    )
                )

    def test_materialized_economics_filters_profiles(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            out = Path(tmp_dir) / "qiongli-workflow"

            materialize_subject_package(
                MaterializeOptions(source=REPO_ROOT, out=out, subject="economics", flavor="full", coverage="focused")
            )

            self.assertTrue((out / "skills" / "domain-profiles" / "economics.yaml").exists())
            self.assertFalse((out / "skills" / "domain-profiles" / "cs-ai.yaml").exists())
            self.assertFalse((out / "skills" / "domain-profiles" / "biomedical.yaml").exists())
            self.assertFalse((out / "skills" / "domain-profiles" / "psychology.yaml").exists())
            self.assertFalse((out / "skills" / "domain-profiles" / "finance.yaml").exists())
            self.assertTrue((out / "venue-profiles" / "aer.yaml").exists())
            self.assertFalse((out / "venue-profiles" / "neurips.yaml").exists())

    def test_materializes_economics_accounting_focused_composite(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            out = Path(tmp_dir) / "qiongli-workflow"

            materialize_subject_package(
                MaterializeOptions(
                    source=REPO_ROOT,
                    out=out,
                    subject="economics-accounting",
                    flavor="full",
                    coverage="focused",
                )
            )

            manifest = json.loads((out / "SUBJECT_MANIFEST.json").read_text(encoding="utf-8"))
            self.assertEqual(manifest["subject"], "economics-accounting")
            self.assertEqual(manifest["coverage"], "focused")

            registry = yaml.safe_load((out / "skills" / "registry.yaml").read_text(encoding="utf-8"))
            registry_ids = {entry["id"] for entry in registry["skills"]}
            self.assertIn("econ-identification-auditor", registry_ids)
            self.assertIn("accounting-measurement-auditor", registry_ids)
            self.assertNotIn("biomedical", registry_ids)

            self.assertTrue((out / "skills" / "C_design" / "econ-identification-auditor.md").exists())
            self.assertTrue((out / "skills" / "C_design" / "accounting-measurement-auditor.md").exists())
            self.assertTrue((out / "skills" / "domain-profiles" / "economics.yaml").exists())
            self.assertTrue((out / "skills" / "domain-profiles" / "accounting.yaml").exists())
            self.assertFalse((out / "skills" / "domain-profiles" / "cs-ai.yaml").exists())
            self.assertTrue((out / "venue-profiles" / "qje.yaml").exists())
            self.assertTrue((out / "venue-profiles" / "accounting-review.yaml").exists())
            self.assertFalse((out / "venue-profiles" / "neurips.yaml").exists())

            manuscript = (out / "skills" / "F_writing" / "manuscript-architect.md").read_text(encoding="utf-8")
            self.assertIn("## Economics and Accounting Overlay", manuscript)
            stats = (out / "skills" / "I_code" / "stats-engine.md").read_text(encoding="utf-8")
            self.assertIn("archival accounting", stats)

    def test_materializes_accounting_focused_package(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            out = Path(tmp_dir) / "qiongli-workflow"
            materialize_subject_package(
                MaterializeOptions(source=REPO_ROOT, out=out, subject="accounting", flavor="full", coverage="focused")
            )
            registry = yaml.safe_load((out / "skills" / "registry.yaml").read_text(encoding="utf-8"))
            registry_ids = {entry["id"] for entry in registry["skills"]}
            self.assertIn("accounting-measurement-auditor", registry_ids)
            self.assertNotIn("econ-identification-auditor", registry_ids)
            self.assertTrue((out / "skills" / "domain-profiles" / "accounting.yaml").exists())
            self.assertFalse((out / "skills" / "domain-profiles" / "economics.yaml").exists())
            manuscript = (out / "skills" / "F_writing" / "manuscript-architect.md").read_text(encoding="utf-8")
            self.assertIn("## Accounting Overlay", manuscript)

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
