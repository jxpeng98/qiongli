from __future__ import annotations

import importlib.util
import json
import shutil
import tarfile
import tempfile
import unittest
import zipfile
from pathlib import Path

from qiongli.source_layout import RepoLayout


REPO_ROOT = Path(__file__).resolve().parents[1]
SCRIPT_PATH = RepoLayout(REPO_ROOT).scripts / "build_plugin_artifacts.py"
EXPECTED_AUTHOR = {"name": "Jiaxin Peng", "url": "https://github.com/jxpeng98"}
EXPECTED_CATEGORY = "Education"
EXPECTED_REPOSITORY = "https://github.com/jxpeng98/qiongli"
EXPECTED_LICENSE = "MIT"
SPEC = importlib.util.spec_from_file_location("build_plugin_artifacts", SCRIPT_PATH)
assert SPEC is not None and SPEC.loader is not None
module = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(module)


class PluginArtifactsTests(unittest.TestCase):
    def test_release_builds_expected_channel_artifacts(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            dist_dir = Path(tmp_dir) / "dist"
            current_tag = (RepoLayout(REPO_ROOT).workflow / "VERSION").read_text(
                encoding="utf-8"
            ).strip()

            artifacts = module.build_artifacts(REPO_ROOT, current_tag, dist_dir)
            for artifact in artifacts:
                self.assertTrue(artifact.is_file(), msg=f"missing artifact: {artifact}")

            if module._is_prerelease_tag(current_tag):
                self._assert_next_release_artifacts(dist_dir, current_tag, artifacts)
            else:
                self._assert_stable_release_artifacts(dist_dir, current_tag, artifacts)

    def _assert_next_release_artifacts(self, dist_dir: Path, current_tag: str, artifacts: list[Path]) -> None:
        desktop_agent_support = [
            "qiongli-next/agents/openai.yaml",
            "qiongli-next/roles/pi.yaml",
            "qiongli-next/templates/agent-run-packet.json",
            "qiongli-next/templates/agent-review-packet.md",
            "qiongli-next/templates/agent-handoff.md",
        ]
        self.assertEqual(
            sorted(path.name for path in artifacts),
            [
                f"qiongli-next-claude-desktop-plugin-{current_tag}.zip",
                f"qiongli-next-claude-desktop-skill-core-{current_tag}.zip",
                f"qiongli-next-claude-plugin-{current_tag}.tar.gz",
                f"qiongli-next-claude-plugin-{current_tag}.zip",
                f"qiongli-next-codex-plugin-{current_tag}.tar.gz",
            ],
        )
        self._assert_contains(
            dist_dir / f"qiongli-next-codex-plugin-{current_tag}.tar.gz",
            [
                f"qiongli-next-codex-plugin-{current_tag}/plugins/qiongli-next/.codex-plugin/plugin.json",
                f"qiongli-next-codex-plugin-{current_tag}/plugins/qiongli-next/.mcp.json",
                f"qiongli-next-codex-plugin-{current_tag}/plugins/qiongli-next/mcp/qiongli-literature-provider/index.mjs",
                f"qiongli-next-codex-plugin-{current_tag}/plugins/qiongli-next/mcp/qiongli-literature-provider/query.mjs",
                f"qiongli-next-codex-plugin-{current_tag}/plugins/qiongli-next/commands/paper.md",
                f"qiongli-next-codex-plugin-{current_tag}/plugins/qiongli-next/skills/qiongli-workflow/SKILL.md",
                f"qiongli-next-codex-plugin-{current_tag}/plugins/qiongli-next/skills/qiongli-next-lit-review/SKILL.md",
                f"qiongli-next-codex-plugin-{current_tag}/plugins/qiongli-next/skills/qiongli-workflow/agents/openai.yaml",
                f"qiongli-next-codex-plugin-{current_tag}/plugins/qiongli-next/skills/qiongli-workflow/roles/pi.yaml",
            ],
        )
        self._assert_contains(
            dist_dir / f"qiongli-next-claude-plugin-{current_tag}.tar.gz",
            [
                f"qiongli-next-claude-plugin-{current_tag}/plugins/qiongli-next/.claude-plugin/plugin.json",
                f"qiongli-next-claude-plugin-{current_tag}/plugins/qiongli-next/mcp/qiongli-literature-provider/index.mjs",
                f"qiongli-next-claude-plugin-{current_tag}/plugins/qiongli-next/mcp/qiongli-literature-provider/query.mjs",
                f"qiongli-next-claude-plugin-{current_tag}/plugins/qiongli-next/commands/paper.md",
                f"qiongli-next-claude-plugin-{current_tag}/plugins/qiongli-next/skills/qiongli-workflow/SKILL.md",
                f"qiongli-next-claude-plugin-{current_tag}/plugins/qiongli-next/skills/qiongli-workflow/agents/openai.yaml",
                f"qiongli-next-claude-plugin-{current_tag}/plugins/qiongli-next/skills/qiongli-workflow/roles/pi.yaml",
            ],
        )
        self._assert_zip_contains(
            dist_dir / f"qiongli-next-claude-plugin-{current_tag}.zip",
            [
                f"qiongli-next-claude-plugin-{current_tag}/plugins/qiongli-next/.claude-plugin/plugin.json",
                f"qiongli-next-claude-plugin-{current_tag}/plugins/qiongli-next/mcp/qiongli-literature-provider/index.mjs",
                f"qiongli-next-claude-plugin-{current_tag}/plugins/qiongli-next/commands/paper.md",
                f"qiongli-next-claude-plugin-{current_tag}/plugins/qiongli-next/skills/qiongli-workflow/SKILL.md",
                f"qiongli-next-claude-plugin-{current_tag}/plugins/qiongli-next/skills/qiongli-workflow/agents/openai.yaml",
                f"qiongli-next-claude-plugin-{current_tag}/plugins/qiongli-next/skills/qiongli-workflow/roles/pi.yaml",
            ],
        )
        self._assert_claude_manifest_mcp_server(
            dist_dir / f"qiongli-next-claude-plugin-{current_tag}.tar.gz",
            f"qiongli-next-claude-plugin-{current_tag}/plugins/qiongli-next/.claude-plugin/plugin.json",
            server_name="qiongli-next",
        )
        self._assert_claude_zip_manifest_mcp_server(
            dist_dir / f"qiongli-next-claude-plugin-{current_tag}.zip",
            f"qiongli-next-claude-plugin-{current_tag}/plugins/qiongli-next/.claude-plugin/plugin.json",
            server_name="qiongli-next",
        )
        self._assert_codex_mcp_server(
            dist_dir / f"qiongli-next-codex-plugin-{current_tag}.tar.gz",
            f"qiongli-next-codex-plugin-{current_tag}/plugins/qiongli-next/.mcp.json",
            server_name="qiongli-next",
        )
        codex_manifest = self._read_tar_json(
            dist_dir / f"qiongli-next-codex-plugin-{current_tag}.tar.gz",
            f"qiongli-next-codex-plugin-{current_tag}/plugins/qiongli-next/.codex-plugin/plugin.json",
        )
        claude_manifest = self._read_tar_json(
            dist_dir / f"qiongli-next-claude-plugin-{current_tag}.tar.gz",
            f"qiongli-next-claude-plugin-{current_tag}/plugins/qiongli-next/.claude-plugin/plugin.json",
        )
        self.assertEqual(codex_manifest["name"], "qiongli-next")
        self.assertEqual(codex_manifest["author"], EXPECTED_AUTHOR)
        self.assertNotIn("category", codex_manifest)
        self.assertEqual(codex_manifest["repository"], EXPECTED_REPOSITORY)
        self.assertEqual(codex_manifest["license"], EXPECTED_LICENSE)
        self.assertEqual(codex_manifest["interface"]["displayName"], "Qiongli Next")
        self.assertEqual(codex_manifest["interface"]["developerName"], EXPECTED_AUTHOR["name"])
        self.assertEqual(codex_manifest["interface"]["category"], EXPECTED_CATEGORY)
        self.assertIn("prerelease", codex_manifest["interface"]["longDescription"].lower())
        self.assertEqual(claude_manifest["name"], "qiongli-next")
        self.assertEqual(claude_manifest["author"], EXPECTED_AUTHOR)
        self.assertEqual(claude_manifest["category"], EXPECTED_CATEGORY)
        self.assertEqual(claude_manifest["repository"], EXPECTED_REPOSITORY)
        self.assertEqual(claude_manifest["license"], EXPECTED_LICENSE)
        skill_text = self._read_tar_text(
            dist_dir / f"qiongli-next-codex-plugin-{current_tag}.tar.gz",
            f"qiongli-next-codex-plugin-{current_tag}/plugins/qiongli-next/skills/qiongli-workflow/SKILL.md",
        )
        command_text = self._read_tar_text(
            dist_dir / f"qiongli-next-codex-plugin-{current_tag}.tar.gz",
            f"qiongli-next-codex-plugin-{current_tag}/plugins/qiongli-next/commands/paper.md",
        )
        self.assertIn("name: qiongli-next\n", skill_text)
        self.assertIn("$qiongli-next", skill_text)
        self.assertIn("Load the `qiongli-next` skill", command_text)
        self.assertNotIn("Load the `qiongli` skill", command_text)
        wrapper_text = self._read_tar_text(
            dist_dir / f"qiongli-next-codex-plugin-{current_tag}.tar.gz",
            f"qiongli-next-codex-plugin-{current_tag}/plugins/qiongli-next/skills/qiongli-next-lit-review/SKILL.md",
        )
        self.assertIn("name: qiongli-next-lit-review\n", wrapper_text)
        self.assertIn("$qiongli-next", wrapper_text)
        self.assertIn("../qiongli-workflow/workflows/lit-review.md", wrapper_text)
        self.assertIn("Claude Code `/lit-review`", wrapper_text)
        self._assert_zip_contains(
            dist_dir / f"qiongli-next-claude-desktop-skill-core-{current_tag}.zip",
            desktop_agent_support
            + [
                "qiongli-next/SKILL.md",
                "qiongli-next/SUBJECT",
                "qiongli-next/skills/registry.yaml",
            ],
        )
        desktop_skill_text = self._read_zip_text(
            dist_dir / f"qiongli-next-claude-desktop-skill-core-{current_tag}.zip",
            "qiongli-next/SKILL.md",
        )
        self.assertIn("name: qiongli-next\n", desktop_skill_text)
        self.assertIn("$qiongli-next", desktop_skill_text)

    def _assert_stable_release_artifacts(self, dist_dir: Path, current_tag: str, artifacts: list[Path]) -> None:
        expected_names = [
            f"qiongli-codex-plugin-{current_tag}.tar.gz",
            f"qiongli-claude-plugin-{current_tag}.tar.gz",
            f"qiongli-claude-plugin-{current_tag}.zip",
            f"qiongli-claude-desktop-plugin-{current_tag}.zip",
        ]
        for subject in module._marketplace_subjects(REPO_ROOT):
            expected_names.extend(
                [
                    f"qiongli-{subject}-codex-plugin-{current_tag}.tar.gz",
                    f"qiongli-{subject}-claude-plugin-{current_tag}.tar.gz",
                    f"qiongli-{subject}-claude-plugin-{current_tag}.zip",
                ]
            )
        expected_names.extend(
            f"qiongli-claude-desktop-skill-{subject}-{current_tag}.zip"
            for subject in module._desktop_subjects(REPO_ROOT)
        )
        expected_names.append(f"qiongli-claude-desktop-skill-{current_tag}.zip")
        self.assertEqual(sorted(expected_names), sorted(path.name for path in artifacts))

        self._assert_contains(
            dist_dir / f"qiongli-codex-plugin-{current_tag}.tar.gz",
            [
                f"qiongli-codex-plugin-{current_tag}/plugins/qiongli/.codex-plugin/plugin.json",
                f"qiongli-codex-plugin-{current_tag}/plugins/qiongli/.mcp.json",
                f"qiongli-codex-plugin-{current_tag}/plugins/qiongli/mcp/qiongli-literature-provider/index.mjs",
                f"qiongli-codex-plugin-{current_tag}/plugins/qiongli/mcp/qiongli-literature-provider/query.mjs",
                f"qiongli-codex-plugin-{current_tag}/plugins/qiongli/commands/paper.md",
                f"qiongli-codex-plugin-{current_tag}/plugins/qiongli/skills/qiongli-workflow/SKILL.md",
                f"qiongli-codex-plugin-{current_tag}/plugins/qiongli/skills/qiongli-lit-review/SKILL.md",
            ],
        )
        self._assert_zip_contains(
            dist_dir / f"qiongli-claude-plugin-{current_tag}.zip",
            [
                f"qiongli-claude-plugin-{current_tag}/plugins/qiongli/.claude-plugin/plugin.json",
                f"qiongli-claude-plugin-{current_tag}/plugins/qiongli/mcp/qiongli-literature-provider/index.mjs",
                f"qiongli-claude-plugin-{current_tag}/plugins/qiongli/mcp/qiongli-literature-provider/query.mjs",
                f"qiongli-claude-plugin-{current_tag}/plugins/qiongli/commands/paper.md",
                f"qiongli-claude-plugin-{current_tag}/plugins/qiongli/skills/qiongli-workflow/SKILL.md",
            ],
        )
        self._assert_claude_manifest_mcp_server(
            dist_dir / f"qiongli-claude-plugin-{current_tag}.tar.gz",
            f"qiongli-claude-plugin-{current_tag}/plugins/qiongli/.claude-plugin/plugin.json",
        )
        self._assert_claude_zip_manifest_mcp_server(
            dist_dir / f"qiongli-claude-plugin-{current_tag}.zip",
            f"qiongli-claude-plugin-{current_tag}/plugins/qiongli/.claude-plugin/plugin.json",
        )
        self._assert_codex_mcp_server(
            dist_dir / f"qiongli-codex-plugin-{current_tag}.tar.gz",
            f"qiongli-codex-plugin-{current_tag}/plugins/qiongli/.mcp.json",
        )
        codex_manifest = self._read_tar_json(
            dist_dir / f"qiongli-codex-plugin-{current_tag}.tar.gz",
            f"qiongli-codex-plugin-{current_tag}/plugins/qiongli/.codex-plugin/plugin.json",
        )
        claude_manifest = self._read_tar_json(
            dist_dir / f"qiongli-claude-plugin-{current_tag}.tar.gz",
            f"qiongli-claude-plugin-{current_tag}/plugins/qiongli/.claude-plugin/plugin.json",
        )
        self.assertEqual(codex_manifest["name"], "qiongli")
        self.assertEqual(codex_manifest["author"], EXPECTED_AUTHOR)
        self.assertNotIn("category", codex_manifest)
        self.assertEqual(codex_manifest["repository"], EXPECTED_REPOSITORY)
        self.assertEqual(codex_manifest["license"], EXPECTED_LICENSE)
        self.assertEqual(codex_manifest["interface"]["displayName"], "Qiongli")
        self.assertEqual(codex_manifest["interface"]["developerName"], EXPECTED_AUTHOR["name"])
        self.assertEqual(codex_manifest["interface"]["category"], EXPECTED_CATEGORY)
        self.assertEqual(claude_manifest["name"], "qiongli")
        self.assertEqual(claude_manifest["author"], EXPECTED_AUTHOR)
        self.assertEqual(claude_manifest["category"], EXPECTED_CATEGORY)
        self.assertEqual(claude_manifest["repository"], EXPECTED_REPOSITORY)
        self.assertEqual(claude_manifest["license"], EXPECTED_LICENSE)
        skill_text = self._read_tar_text(
            dist_dir / f"qiongli-codex-plugin-{current_tag}.tar.gz",
            f"qiongli-codex-plugin-{current_tag}/plugins/qiongli/skills/qiongli-workflow/SKILL.md",
        )
        command_text = self._read_tar_text(
            dist_dir / f"qiongli-codex-plugin-{current_tag}.tar.gz",
            f"qiongli-codex-plugin-{current_tag}/plugins/qiongli/commands/paper.md",
        )
        self.assertIn("name: qiongli\n", skill_text)
        self.assertIn("Load the `qiongli` skill", command_text)
        wrapper_text = self._read_tar_text(
            dist_dir / f"qiongli-codex-plugin-{current_tag}.tar.gz",
            f"qiongli-codex-plugin-{current_tag}/plugins/qiongli/skills/qiongli-lit-review/SKILL.md",
        )
        self.assertIn("name: qiongli-lit-review\n", wrapper_text)
        self.assertIn("$qiongli", wrapper_text)
        self.assertIn("../qiongli-workflow/workflows/lit-review.md", wrapper_text)
        self.assertIn("Claude Code `/lit-review`", wrapper_text)

        self._assert_zip_contains(
            dist_dir / f"qiongli-claude-desktop-skill-core-{current_tag}.zip",
            [
                "qiongli/agents/openai.yaml",
                "qiongli/roles/pi.yaml",
                "qiongli/templates/agent-run-packet.json",
                "qiongli/templates/agent-review-packet.md",
                "qiongli/templates/agent-handoff.md",
                "qiongli/SKILL.md",
                "qiongli/SUBJECT",
                "qiongli/skills/registry.yaml",
            ],
        )
        desktop_skill_text = self._read_zip_text(
            dist_dir / f"qiongli-claude-desktop-skill-core-{current_tag}.zip",
            "qiongli/SKILL.md",
        )
        self.assertIn("name: qiongli\n", desktop_skill_text)

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
            skill_text = (dest / "SKILL.md").read_text(encoding="utf-8")
            self.assertIn(
                "Do not use `qiongli_collect_evidence` to judge built-in literature provider configuration",
                skill_text,
            )
            self.assertIn("Platform-native search alone is `native_only`, not `provider_connected`", skill_text)
            self.assertIn("if no provider MCP/MCPB and no platform-native search is available", skill_text)
            self.assertIn("do not claim review-grade external provider or native-search coverage", skill_text)
            self.assertNotIn("use platform search or user-supplied corpus", skill_text)
            self.assertNotIn(
                "or platform-native search capability before claiming `provider_connected`",
                skill_text,
            )
            registry = (dest / "skills" / "registry.yaml").read_text(encoding="utf-8")
            self.assertIn("id: accounting-measurement-auditor", registry)
            manifest = json.loads((dest / "SUBJECT_MANIFEST.json").read_text(encoding="utf-8"))
            self.assertEqual(
                ["core", "economics", "accounting", "economics-accounting"],
                manifest["layers"],
            )

    def test_fails_when_artifact_versions_do_not_match_tag(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            (root / "qiongli-workflow").mkdir(parents=True)
            (root / "qiongli-workflow" / "VERSION").write_text("v0.5.0-beta.2\n", encoding="utf-8")

            with self.assertRaisesRegex(ValueError, "version mismatch"):
                module.build_artifacts(root, "v0.5.0-beta.3", root / "dist")

    def _assert_contains(self, artifact: Path, expected: list[str]) -> None:
        with tarfile.open(artifact, "r:gz") as tar:
            names = set(tar.getnames())
        for name in expected:
            self.assertIn(name, names)

    def _read_tar_json(self, artifact: Path, member: str) -> dict[str, object]:
        with tarfile.open(artifact, "r:gz") as tar:
            extracted = tar.extractfile(member)
            self.assertIsNotNone(extracted, msg=f"missing tar member: {member}")
            assert extracted is not None
            return json.loads(extracted.read().decode("utf-8"))

    def _read_tar_text(self, artifact: Path, member: str) -> str:
        with tarfile.open(artifact, "r:gz") as tar:
            extracted = tar.extractfile(member)
            self.assertIsNotNone(extracted, msg=f"missing tar member: {member}")
            assert extracted is not None
            return extracted.read().decode("utf-8")

    def _assert_claude_manifest_mcp_server(
        self,
        artifact: Path,
        member: str,
        *,
        server_name: str = "qiongli",
    ) -> None:
        manifest = self._read_tar_json(artifact, member)
        self.assertIn("mcpServers", manifest)
        self.assertIn(server_name, manifest["mcpServers"])
        if server_name != "qiongli":
            self.assertNotIn("qiongli", manifest["mcpServers"])
        server = manifest["mcpServers"][server_name]
        self.assertEqual(server["command"], "node")
        self.assertEqual(
            server["args"],
            ["${CLAUDE_PLUGIN_ROOT}/mcp/qiongli-literature-provider/index.mjs"],
        )
        self.assertEqual(server["cwd"], "${CLAUDE_PLUGIN_ROOT}")

    def _assert_codex_mcp_server(
        self,
        artifact: Path,
        member: str,
        *,
        server_name: str = "qiongli",
    ) -> None:
        manifest = self._read_tar_json(artifact, member)
        self.assertIn("mcpServers", manifest)
        self.assertIn(server_name, manifest["mcpServers"])
        if server_name != "qiongli":
            self.assertNotIn("qiongli", manifest["mcpServers"])
        server = manifest["mcpServers"][server_name]
        self.assertEqual(server["command"], "node")
        self.assertEqual(server["args"], ["./mcp/qiongli-literature-provider/index.mjs"])

    def _assert_zip_contains(self, artifact: Path, expected: list[str]) -> None:
        with zipfile.ZipFile(artifact) as archive:
            names = set(archive.namelist())
        for name in expected:
            self.assertIn(name, names)

    def _read_zip_text(self, artifact: Path, member: str) -> str:
        with zipfile.ZipFile(artifact) as archive:
            return archive.read(member).decode("utf-8")

    def _read_zip_json(self, artifact: Path, member: str) -> dict[str, object]:
        with zipfile.ZipFile(artifact) as archive:
            return json.loads(archive.read(member).decode("utf-8"))

    def _assert_claude_zip_manifest_mcp_server(
        self,
        artifact: Path,
        member: str,
        *,
        server_name: str = "qiongli",
    ) -> None:
        manifest = self._read_zip_json(artifact, member)
        self.assertIn("mcpServers", manifest)
        self.assertIn(server_name, manifest["mcpServers"])
        if server_name != "qiongli":
            self.assertNotIn("qiongli", manifest["mcpServers"])
        server = manifest["mcpServers"][server_name]
        self.assertEqual(server["command"], "node")
        self.assertEqual(
            server["args"],
            ["${CLAUDE_PLUGIN_ROOT}/mcp/qiongli-literature-provider/index.mjs"],
        )
        self.assertEqual(server["cwd"], "${CLAUDE_PLUGIN_ROOT}")

    def _make_fallback_root(self, root: Path) -> Path:
        shutil.copytree(RepoLayout(REPO_ROOT).workflow, root / "qiongli-workflow")
        shutil.copytree(RepoLayout(REPO_ROOT).templates, root / "qiongli-workflow" / "templates", dirs_exist_ok=True)
        shutil.copytree(RepoLayout(REPO_ROOT).skills, root / "qiongli-workflow" / "skills", dirs_exist_ok=True)
        shutil.copytree(RepoLayout(REPO_ROOT).skills, root / "skills")
        shutil.copytree(RepoLayout(REPO_ROOT).subjects, root / "subjects")
        shutil.copy2(RepoLayout(REPO_ROOT).skills_core, root / "qiongli-workflow" / "skills-core.md")
        shutil.copy2(RepoLayout(REPO_ROOT).skills_summary, root / "qiongli-workflow" / "skills-summary.md")
        return root


if __name__ == "__main__":
    unittest.main()
