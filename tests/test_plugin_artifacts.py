from __future__ import annotations

import importlib.util
import json
import shutil
import tarfile
import tempfile
import unittest
import zipfile
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
SCRIPT_PATH = REPO_ROOT / "scripts" / "build_plugin_artifacts.py"
SPEC = importlib.util.spec_from_file_location("build_plugin_artifacts", SCRIPT_PATH)
assert SPEC is not None and SPEC.loader is not None
module = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(module)


class PluginArtifactsTests(unittest.TestCase):
    def test_builds_release_distribution_artifacts(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            dist_dir = Path(tmp_dir) / "dist"
            current_tag = (REPO_ROOT / "qiongli-workflow" / "VERSION").read_text(
                encoding="utf-8"
            ).strip()

            artifacts = module.build_artifacts(REPO_ROOT, current_tag, dist_dir)

            self.assertEqual(
                sorted(path.name for path in artifacts),
                [
                    f"qiongli-claude-desktop-skill-core-{current_tag}.zip",
                    f"qiongli-claude-desktop-skill-economics-accounting-{current_tag}.zip",
                    f"qiongli-claude-desktop-skill-economics-{current_tag}.zip",
                    f"qiongli-claude-desktop-skill-{current_tag}.zip",
                    f"qiongli-claude-plugin-{current_tag}.tar.gz",
                    f"qiongli-codex-plugin-{current_tag}.tar.gz",
                    f"qiongli-gemini-extension-{current_tag}.tar.gz",
                ],
            )
            for artifact in artifacts:
                self.assertTrue(artifact.is_file(), msg=f"missing artifact: {artifact}")

            self._assert_contains(
                dist_dir / f"qiongli-codex-plugin-{current_tag}.tar.gz",
                [
                    f"qiongli-codex-plugin-{current_tag}/plugins/qiongli/.codex-plugin/plugin.json",
                    f"qiongli-codex-plugin-{current_tag}/plugins/qiongli/commands/paper.md",
                    f"qiongli-codex-plugin-{current_tag}/plugins/qiongli/skills/qiongli-workflow/SKILL.md",
                ],
            )
            self._assert_contains(
                dist_dir / f"qiongli-claude-plugin-{current_tag}.tar.gz",
                [
                    f"qiongli-claude-plugin-{current_tag}/plugins/qiongli/.claude-plugin/plugin.json",
                    f"qiongli-claude-plugin-{current_tag}/plugins/qiongli/commands/paper.md",
                    f"qiongli-claude-plugin-{current_tag}/plugins/qiongli/skills/qiongli-workflow/SKILL.md",
                ],
            )
            self._assert_contains(
                dist_dir / f"qiongli-gemini-extension-{current_tag}.tar.gz",
                [
                    f"qiongli-gemini-extension-{current_tag}/gemini-extension.json",
                    f"qiongli-gemini-extension-{current_tag}/skills/qiongli-workflow/SKILL.md",
                ],
            )
            self._assert_zip_contains(
                dist_dir / f"qiongli-claude-desktop-skill-core-{current_tag}.zip",
                [
                    "qiongli/SKILL.md",
                    "qiongli/SUBJECT",
                    "qiongli/skills/registry.yaml",
                ],
            )
            self._assert_zip_contains(
                dist_dir / f"qiongli-claude-desktop-skill-economics-{current_tag}.zip",
                [
                    "qiongli/SKILL.md",
                    "qiongli/SUBJECT",
                    "qiongli/skills/C_design/econ-identification-auditor.md",
                    "qiongli/skills/F_writing/manuscript-architect.md",
                    "qiongli/venue-profiles/aer.yaml",
                ],
            )
            self._assert_zip_contains(
                dist_dir / f"qiongli-claude-desktop-skill-economics-accounting-{current_tag}.zip",
                [
                    "qiongli/SKILL.md",
                    "qiongli/SUBJECT",
                    "qiongli/skills/C_design/econ-identification-auditor.md",
                    "qiongli/skills/C_design/accounting-measurement-auditor.md",
                    "qiongli/skills/F_writing/manuscript-architect.md",
                    "qiongli/venue-profiles/accounting-review.yaml",
                ],
            )

    def test_fallback_economics_accounting_desktop_skill_includes_accounting_auditor(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = self._make_fallback_root(Path(tmp_dir) / "repo")
            dest = Path(tmp_dir) / "qiongli"
            original_materializer = module.materialize_subject_package
            original_options = module.MaterializeOptions
            try:
                module.materialize_subject_package = None
                module.MaterializeOptions = None

                module._copy_claude_desktop_skill(root, dest, "economics-accounting")
            finally:
                module.materialize_subject_package = original_materializer
                module.MaterializeOptions = original_options

            self.assertTrue((dest / "skills" / "C_design" / "accounting-measurement-auditor.md").exists())
            registry = (dest / "skills" / "registry.yaml").read_text(encoding="utf-8")
            self.assertIn("id: accounting-measurement-auditor", registry)

    def test_fails_when_artifact_versions_do_not_match_tag(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            (root / "qiongli-workflow").mkdir(parents=True)
            (root / "qiongli-workflow" / "VERSION").write_text("v0.5.0-beta.3\n", encoding="utf-8")
            (root / "plugins" / "qiongli" / ".codex-plugin").mkdir(parents=True)
            (root / "plugins" / "qiongli" / ".codex-plugin" / "plugin.json").write_text(
                json.dumps({"version": "0.5.0-beta.2"}),
                encoding="utf-8",
            )

            with self.assertRaisesRegex(ValueError, "version mismatch"):
                module.build_artifacts(root, "v0.5.0-beta.3", root / "dist")

    def _assert_contains(self, artifact: Path, expected: list[str]) -> None:
        with tarfile.open(artifact, "r:gz") as tar:
            names = set(tar.getnames())
        for name in expected:
            self.assertIn(name, names)

    def _assert_zip_contains(self, artifact: Path, expected: list[str]) -> None:
        with zipfile.ZipFile(artifact) as archive:
            names = set(archive.namelist())
        for name in expected:
            self.assertIn(name, names)

    def _make_fallback_root(self, root: Path) -> Path:
        shutil.copytree(REPO_ROOT / "qiongli-workflow", root / "qiongli-workflow")
        shutil.copytree(REPO_ROOT / "templates", root / "qiongli-workflow" / "templates", dirs_exist_ok=True)
        shutil.copytree(REPO_ROOT / "skills", root / "qiongli-workflow" / "skills", dirs_exist_ok=True)
        shutil.copytree(REPO_ROOT / "skills", root / "skills")
        shutil.copytree(REPO_ROOT / "subjects", root / "subjects")
        shutil.copy2(REPO_ROOT / "skills-core.md", root / "qiongli-workflow" / "skills-core.md")
        shutil.copy2(REPO_ROOT / "skills-summary.md", root / "qiongli-workflow" / "skills-summary.md")
        return root


if __name__ == "__main__":
    unittest.main()
