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

            artifacts = module.build_artifacts(REPO_ROOT, "v0.5.0-beta.3", dist_dir)

            self.assertEqual(
                sorted(path.name for path in artifacts),
                [
                    "research-skills-claude-plugin-v0.5.0-beta.3.tar.gz",
                    "research-skills-codex-plugin-v0.5.0-beta.3.tar.gz",
                    "research-skills-gemini-extension-v0.5.0-beta.3.tar.gz",
                ],
            )
            for artifact in artifacts:
                self.assertTrue(artifact.is_file(), msg=f"missing artifact: {artifact}")

            self._assert_contains(
                dist_dir / "research-skills-codex-plugin-v0.5.0-beta.3.tar.gz",
                [
                    "research-skills-codex-plugin-v0.5.0-beta.3/.agents/plugins/marketplace.json",
                    "research-skills-codex-plugin-v0.5.0-beta.3/plugins/research-skills/.codex-plugin/plugin.json",
                    "research-skills-codex-plugin-v0.5.0-beta.3/plugins/research-skills/skills/research-paper-workflow/SKILL.md",
                ],
            )
            self._assert_contains(
                dist_dir / "research-skills-claude-plugin-v0.5.0-beta.3.tar.gz",
                [
                    "research-skills-claude-plugin-v0.5.0-beta.3/.claude-plugin/marketplace.json",
                    "research-skills-claude-plugin-v0.5.0-beta.3/plugins/research-skills/.claude-plugin/plugin.json",
                    "research-skills-claude-plugin-v0.5.0-beta.3/plugins/research-skills/skills/research-paper-workflow/SKILL.md",
                ],
            )
            self._assert_contains(
                dist_dir / "research-skills-gemini-extension-v0.5.0-beta.3.tar.gz",
                [
                    "research-skills-gemini-extension-v0.5.0-beta.3/gemini-extension.json",
                    "research-skills-gemini-extension-v0.5.0-beta.3/skills/research-paper-workflow/SKILL.md",
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
