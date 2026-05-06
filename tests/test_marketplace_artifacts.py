from __future__ import annotations

import importlib.util
import json
import tarfile
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
SCRIPT_PATH = REPO_ROOT / "scripts" / "build_marketplace_artifacts.py"
SPEC = importlib.util.spec_from_file_location("build_marketplace_artifacts", SCRIPT_PATH)
assert SPEC is not None and SPEC.loader is not None
module = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(module)


class MarketplaceArtifactsTests(unittest.TestCase):
    def test_builds_three_self_contained_marketplace_artifacts(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            dist_dir = Path(tmp_dir) / "dist"
            current_tag = (REPO_ROOT / "research-paper-workflow" / "VERSION").read_text(
                encoding="utf-8"
            ).strip()

            artifacts = module.build_artifacts(REPO_ROOT, current_tag, dist_dir)

            self.assertEqual(
                sorted(path.name for path in artifacts),
                [
                    f"research-skills-claude-plugin-{current_tag}.tar.gz",
                    f"research-skills-codex-plugin-{current_tag}.tar.gz",
                    f"research-skills-gemini-extension-{current_tag}.tar.gz",
                ],
            )
            for artifact in artifacts:
                self.assertTrue(artifact.is_file(), msg=f"missing artifact: {artifact}")

            self._assert_contains(
                dist_dir / f"research-skills-codex-plugin-{current_tag}.tar.gz",
                [
                    f"research-skills-codex-plugin-{current_tag}/.agents/plugins/marketplace.json",
                    f"research-skills-codex-plugin-{current_tag}/plugins/research-skills/.codex-plugin/plugin.json",
                    f"research-skills-codex-plugin-{current_tag}/plugins/research-skills/skills/research-paper-workflow/SKILL.md",
                ],
            )
            self._assert_contains(
                dist_dir / f"research-skills-claude-plugin-{current_tag}.tar.gz",
                [
                    f"research-skills-claude-plugin-{current_tag}/.claude-plugin/marketplace.json",
                    f"research-skills-claude-plugin-{current_tag}/plugins/research-skills/.claude-plugin/plugin.json",
                    f"research-skills-claude-plugin-{current_tag}/plugins/research-skills/skills/research-paper-workflow/SKILL.md",
                ],
            )
            self._assert_contains(
                dist_dir / f"research-skills-gemini-extension-{current_tag}.tar.gz",
                [
                    f"research-skills-gemini-extension-{current_tag}/gemini-extension.json",
                    f"research-skills-gemini-extension-{current_tag}/skills/research-paper-workflow/SKILL.md",
                ],
            )

    def test_fails_when_artifact_versions_do_not_match_tag(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            (root / "research-paper-workflow").mkdir(parents=True)
            (root / "research-paper-workflow" / "VERSION").write_text("v0.5.0-beta.3\n", encoding="utf-8")
            (root / ".agents" / "plugins").mkdir(parents=True)
            (root / ".agents" / "plugins" / "marketplace.json").write_text("{}", encoding="utf-8")
            (root / "plugins" / "research-skills" / ".codex-plugin").mkdir(parents=True)
            (root / "plugins" / "research-skills" / ".codex-plugin" / "plugin.json").write_text(
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


if __name__ == "__main__":
    unittest.main()
