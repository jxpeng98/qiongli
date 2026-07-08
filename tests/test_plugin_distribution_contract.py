from __future__ import annotations

from dataclasses import replace
import importlib.util
import json
import os
import shutil
import subprocess
import sys
import tempfile
import unittest
import zipfile
from pathlib import Path

from qiongli.source_layout import RepoLayout


REPO_ROOT = Path(__file__).resolve().parents[1]
LAYOUT = RepoLayout(REPO_ROOT)
PLUGIN_ROOT = LAYOUT.plugin_package
NEXT_PLUGIN_ROOT = LAYOUT.next_plugin_package
WORKFLOW_ROOT = LAYOUT.workflow
WORKFLOW_VERSION = (WORKFLOW_ROOT / "VERSION").read_text(encoding="utf-8").strip().lstrip("v")
VALIDATOR_SCRIPT_PATH = LAYOUT.scripts / "validate_marketplace_install.py"
if str(LAYOUT.scripts) not in sys.path:
    sys.path.insert(0, str(LAYOUT.scripts))
VALIDATOR_SPEC = importlib.util.spec_from_file_location("validate_marketplace_install", VALIDATOR_SCRIPT_PATH)
assert VALIDATOR_SPEC is not None and VALIDATOR_SPEC.loader is not None
validator = importlib.util.module_from_spec(VALIDATOR_SPEC)
sys.modules["validate_marketplace_install"] = validator
VALIDATOR_SPEC.loader.exec_module(validator)


def find_usable_bash() -> str | None:
    candidates: list[str] = []
    for value in (os.environ.get("BASH"), shutil.which("bash")):
        if value:
            candidates.append(value)

    candidates.extend(
        [
            "/bin/bash",
            r"C:\Program Files\Git\bin\bash.exe",
            r"C:\Program Files\Git\usr\bin\bash.exe",
        ]
    )

    seen: set[str] = set()
    for candidate in candidates:
        normalized = str(Path(candidate))
        if normalized in seen:
            continue
        seen.add(normalized)
        if not Path(candidate).exists():
            continue

        result = subprocess.run(
            [candidate, "--version"],
            text=True,
            capture_output=True,
            check=False,
        )
        if result.returncode == 0:
            return candidate

    return None


class PluginDistributionContractTests(unittest.TestCase):
    def materialize_payload_root(self, tmp_dir: str, target: str = "plugin") -> Path:
        out = Path(tmp_dir) / "dist-source"
        result = subprocess.run(
            [
                sys.executable,
                "scripts/materialize_distribution_payloads.py",
                "--target",
                target,
                "--out",
                str(out),
                "--force",
            ],
            cwd=REPO_ROOT,
            text=True,
            capture_output=True,
            check=False,
        )
        self.assertEqual(result.returncode, 0, msg=result.stderr + result.stdout)
        return out

    def materialize_plugin_payload(self, tmp_dir: str) -> Path:
        return self.materialize_payload_root(tmp_dir) / "plugins" / "qiongli"

    def materialize_next_plugin_payload(self, tmp_dir: str) -> Path:
        return self.materialize_payload_root(tmp_dir, target="next-plugin") / "plugins" / "qiongli-next"

    def test_platform_target_registry_declares_boundary_rules(self) -> None:
        from qiongli.platform_targets import load_platform_targets

        targets = load_platform_targets(REPO_ROOT)

        expected_targets = {
            "codex-marketplace-plugin",
            "claude-code-marketplace-plugin",
            "claude-desktop-direct-plugin",
            "claude-desktop-skill-zip",
            "antigravity-local-plugin",
            "npm-plugin-lite",
            "pypi-full-runtime",
        }
        self.assertTrue(expected_targets.issubset(targets.keys()))
        self.assertIn(
            ".codex-plugin/plugin.json",
            targets["codex-marketplace-plugin"].required_paths,
        )
        self.assertIn(
            ".codex-plugin/",
            targets["claude-desktop-direct-plugin"].forbidden_paths,
        )
        self.assertIn(
            ".mcp.json",
            targets["claude-desktop-direct-plugin"].forbidden_paths,
        )
        self.assertEqual(targets["claude-desktop-direct-plugin"].archive_format, "zip")
        self.assertEqual(
            targets["codex-marketplace-plugin"].adapter["plugin_manifest_platform"],
            "codex",
        )
        self.assertEqual(targets["codex-marketplace-plugin"].adapter["kind"], "plugin")
        self.assertEqual(
            targets["codex-marketplace-plugin"].adapter["materializer"],
            "plugin_artifacts",
        )
        self.assertEqual(
            targets["claude-desktop-direct-plugin"].adapter["plugin_manifest_platform"],
            "claude",
        )
        self.assertEqual(
            targets["claude-desktop-skill-zip"].adapter["plugin_manifest_platform"],
            "none",
        )
        self.assertEqual(targets["claude-desktop-skill-zip"].adapter["kind"], "skill-zip")
        self.assertEqual(
            targets["claude-desktop-skill-zip"].adapter["materializer"],
            "desktop_skill_artifacts",
        )
        self.assertEqual(targets["antigravity-local-plugin"].adapter["kind"], "local-plugin")
        self.assertEqual(targets["pypi-full-runtime"].adapter["kind"], "package")
        self.assertEqual(
            targets["npm-plugin-lite"].adapter["materializer"],
            "npm_package",
        )
        self.assertEqual(
            targets["codex-marketplace-plugin"].smoke["structural_archive_check"],
            "marketplace_validation",
        )
        self.assertEqual(
            targets["codex-marketplace-plugin"].smoke["client_activation_check"],
            "local_install_acceptance",
        )
        self.assertEqual(
            targets["claude-desktop-skill-zip"].smoke["client_activation_check"],
            "not_applicable",
        )

    def test_platform_target_registry_rejects_targets_without_positive_or_negative_checks(self) -> None:
        from qiongli.platform_targets import validate_platform_target_registry

        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            self._write_platform_target_registry(
                root,
                required_paths=[],
                forbidden_paths=[".codex-plugin/"],
            )

            failures = validate_platform_target_registry(root)

        self.assertTrue(
            any("missing positive required_path checks" in failure for failure in failures),
            failures,
        )

        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            self._write_platform_target_registry(
                root,
                required_paths=["plugin.json"],
                forbidden_paths=[],
            )

            failures = validate_platform_target_registry(root)

        self.assertTrue(
            any("missing negative forbidden_path checks" in failure for failure in failures),
            failures,
        )

    def test_platform_target_registry_rejects_missing_release_download_metadata(self) -> None:
        from qiongli.platform_targets import validate_platform_target_registry

        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            self._write_platform_target_registry(
                root,
                required_paths=["plugin.json"],
                forbidden_paths=[".codex-plugin/"],
                include_release_download=False,
            )

            failures = validate_platform_target_registry(root)

        self.assertTrue(
            any("release_download" in failure for failure in failures),
            failures,
        )

    def test_platform_target_registry_rejects_missing_adapter_metadata(self) -> None:
        from qiongli.platform_targets import validate_platform_target_registry

        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            self._write_platform_target_registry(
                root,
                required_paths=["plugin.json"],
                forbidden_paths=[".codex-plugin/"],
                include_adapter=False,
            )

            failures = validate_platform_target_registry(root)

        self.assertTrue(
            any("adapter" in failure for failure in failures),
            failures,
        )

    def test_platform_target_registry_rejects_missing_adapter_materializer(self) -> None:
        from qiongli.platform_targets import validate_platform_target_registry

        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            self._write_platform_target_registry(
                root,
                required_paths=["plugin.json"],
                forbidden_paths=[".codex-plugin/"],
                adapter={
                    "kind": "plugin",
                    "plugin_manifest_platform": "none",
                },
            )

            failures = validate_platform_target_registry(root)

        self.assertTrue(
            any("adapter.materializer" in failure for failure in failures),
            failures,
        )

    def test_platform_target_registry_rejects_unknown_adapter_materializer(self) -> None:
        from qiongli.platform_targets import validate_platform_target_registry

        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            self._write_platform_target_registry(
                root,
                required_paths=["plugin.json"],
                forbidden_paths=[".codex-plugin/"],
                adapter={
                    "kind": "plugin",
                    "plugin_manifest_platform": "none",
                    "materializer": "ad_hoc_script",
                },
            )

            failures = validate_platform_target_registry(root)

        self.assertTrue(
            any("adapter.materializer must be one of" in failure for failure in failures),
            failures,
        )

    def test_platform_target_registry_rejects_unknown_adapter_kind(self) -> None:
        from qiongli.platform_targets import validate_platform_target_registry

        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            self._write_platform_target_registry(
                root,
                required_paths=["plugin.json"],
                forbidden_paths=[".codex-plugin/"],
                adapter={
                    "kind": "handcrafted-plugin",
                    "plugin_manifest_platform": "none",
                    "materializer": "plugin_artifacts",
                },
            )

            failures = validate_platform_target_registry(root)

        self.assertTrue(
            any("adapter.kind must be one of" in failure for failure in failures),
            failures,
        )

    def test_platform_target_registry_rejects_unknown_manifest_platform(self) -> None:
        from qiongli.platform_targets import validate_platform_target_registry

        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            self._write_platform_target_registry(
                root,
                required_paths=["plugin.json"],
                forbidden_paths=[".codex-plugin/"],
                adapter={
                    "kind": "plugin",
                    "plugin_manifest_platform": "gemini",
                    "materializer": "plugin_artifacts",
                },
            )

            failures = validate_platform_target_registry(root)

        self.assertTrue(
            any("adapter.plugin_manifest_platform must be one of" in failure for failure in failures),
            failures,
        )

    def test_platform_target_registry_rejects_adapter_manifest_platform_mismatch(self) -> None:
        from qiongli.platform_targets import validate_platform_target_registry

        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            self._write_platform_target_registry(
                root,
                required_paths=["package.json"],
                forbidden_paths=[".codex-plugin/"],
                adapter={
                    "kind": "package",
                    "plugin_manifest_platform": "codex",
                    "materializer": "npm_package",
                },
            )

            failures = validate_platform_target_registry(root)

        self.assertTrue(
            any("adapter.plugin_manifest_platform=codex is not valid" in failure for failure in failures),
            failures,
        )

    def test_platform_target_registry_rejects_adapter_materializer_mismatch(self) -> None:
        from qiongli.platform_targets import validate_platform_target_registry

        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            self._write_platform_target_registry(
                root,
                required_paths=["SKILL.md"],
                forbidden_paths=[".codex-plugin/"],
                adapter={
                    "kind": "skill-zip",
                    "plugin_manifest_platform": "none",
                    "materializer": "plugin_artifacts",
                },
            )

            failures = validate_platform_target_registry(root)

        self.assertTrue(
            any("adapter.materializer=plugin_artifacts is not valid" in failure for failure in failures),
            failures,
        )

    def test_platform_target_registry_rejects_missing_smoke_metadata(self) -> None:
        from qiongli.platform_targets import validate_platform_target_registry

        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            self._write_platform_target_registry(
                root,
                required_paths=["plugin.json"],
                forbidden_paths=[".codex-plugin/"],
                include_smoke=False,
            )

            failures = validate_platform_target_registry(root)

        self.assertTrue(
            any("target fixture-target.smoke must be an object" in failure for failure in failures),
            failures,
        )

    def test_platform_target_registry_rejects_unknown_smoke_policy(self) -> None:
        from qiongli.platform_targets import validate_platform_target_registry

        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            self._write_platform_target_registry(
                root,
                required_paths=["plugin.json"],
                forbidden_paths=[".codex-plugin/"],
                smoke={
                    "structural_archive_check": "ad_hoc_manual_check",
                    "client_activation_check": "local_install_acceptance",
                },
            )

            failures = validate_platform_target_registry(root)

        self.assertTrue(
            any(
                "target fixture-target.smoke.structural_archive_check "
                "must be one of" in failure
                for failure in failures
            ),
            failures,
        )

    def test_marketplace_validator_uses_target_adapter_for_manifest_platform(self) -> None:
        from qiongli.platform_targets import load_platform_targets

        codex_target = load_platform_targets(REPO_ROOT)["codex-marketplace-plugin"]
        fake_target = replace(codex_target, target_id="fixture-codex-like-plugin")

        self.assertEqual(validator._platform_for_target(fake_target), "codex")

    def test_marketplace_validator_selects_target_by_recommended_key(self) -> None:
        from qiongli.platform_targets import load_platform_targets

        codex_target = load_platform_targets(REPO_ROOT)["codex-marketplace-plugin"]
        fake_target = replace(codex_target, target_id="fixture-codex-target")

        selected = validator._target_by_recommended_key(
            {"fixture-codex-target": fake_target},
            "codex",
        )

        self.assertEqual(selected.target_id, "fixture-codex-target")

    def test_materialized_plugin_has_root_plugin_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            materialized_plugin = self.materialize_plugin_payload(tmp_dir)
            manifest = json.loads((materialized_plugin / "plugin.json").read_text(encoding="utf-8"))

        self.assertEqual(manifest, {"name": "qiongli"})

    def test_materialized_next_plugin_has_root_plugin_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            materialized_plugin = self.materialize_next_plugin_payload(tmp_dir)
            manifest = json.loads((materialized_plugin / "plugin.json").read_text(encoding="utf-8"))

        self.assertEqual(manifest, {"name": "qiongli-next"})

    def _write_platform_target_registry(
        self,
        root: Path,
        *,
        required_paths: list[str],
        forbidden_paths: list[str],
        include_release_download: bool = True,
        release_download: dict[str, object] | None = None,
        include_adapter: bool = True,
        adapter: dict[str, object] | None = None,
        include_smoke: bool = True,
        smoke: dict[str, object] | None = None,
    ) -> None:
        registry = root / "content" / "distribution" / "platform-targets.yaml"
        registry.parent.mkdir(parents=True)
        target: dict[str, object] = {
            "display_name": "Fixture Platform",
            "artifact_kind": "fixture",
            "archive_format": "zip",
            "source_inputs": ["content/workflow/**"],
            "required_paths": required_paths,
            "allowed_wrapper_dirs": [],
            "forbidden_paths": forbidden_paths,
            "bundled_mcp_mode": "none",
            "command_surface": "fixture-cli",
            "validator": "fixture-validator",
        }
        if include_adapter:
            target["adapter"] = adapter or {
                "kind": "local-plugin",
                "plugin_manifest_platform": "none",
                "materializer": "local_plugin_installer",
            }
        if include_smoke:
            target["smoke"] = smoke or {
                "structural_archive_check": "marketplace_validation",
                "client_activation_check": "local_install_acceptance",
            }
        if include_release_download:
            target["release_download"] = release_download or {
                "guide_label": "Fixture Platform",
                "recommended_key": "fixture",
                "asset_groups": [],
            }
        registry.write_text(
            json.dumps(
                {
                    "schema_version": "1.0",
                    "targets": {"fixture-target": target},
                },
                indent=2,
            ),
            encoding="utf-8",
        )

    def test_platform_manifests_share_workflow_version(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            materialized_plugin = self.materialize_plugin_payload(tmp_dir)
            codex = json.loads((materialized_plugin / ".codex-plugin" / "plugin.json").read_text(encoding="utf-8"))
            claude = json.loads((materialized_plugin / ".claude-plugin" / "plugin.json").read_text(encoding="utf-8"))

        self.assertEqual(codex["version"], WORKFLOW_VERSION)
        self.assertEqual(claude["version"], WORKFLOW_VERSION)
        self.assertFalse((materialized_plugin / "gemini-extension.json").exists())

    def test_codex_plugin_exposes_skill_directory(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            materialized_plugin = self.materialize_plugin_payload(tmp_dir)
            manifest = json.loads(
                (materialized_plugin / ".codex-plugin" / "plugin.json").read_text(encoding="utf-8")
            )

            self.assertEqual(manifest["skills"], "./skills/")
            self.assertTrue((materialized_plugin / "skills").is_dir())
            self.assertTrue((materialized_plugin / ".mcp.json").is_file())
            self.assertTrue(
                (materialized_plugin / "bin" / "qiongli-literature-provider").is_file()
            )

    def test_git_backed_next_codex_plugin_source_is_installable(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            materialized_next = self.materialize_next_plugin_payload(tmp_dir)
            manifest_path = materialized_next / ".codex-plugin" / "plugin.json"
            mcp_manifest_path = materialized_next / ".mcp.json"
            skill_root = materialized_next / "skills" / "qiongli-workflow"

            validator._assert_manifest(
                "codex",
                manifest_path,
                WORKFLOW_VERSION,
                expected_plugin_name="qiongli-next",
                expected_skill_name="qiongli-next",
            )

            manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
            self.assertEqual(manifest["name"], "qiongli-next")
            self.assertEqual(manifest["skills"], "./skills/")
            self.assertEqual(manifest["mcpServers"], "./.mcp.json")

            mcp_manifest = json.loads(mcp_manifest_path.read_text(encoding="utf-8"))
            self.assertEqual(set(mcp_manifest["mcpServers"]), {"qiongli-next"})
            validator._assert_bundled_literature_mcp(
                materialized_next,
                "codex",
                mcp_server_name="qiongli-next",
            )

            workflow_names = validator._assert_skill_invocation(
                skill_root,
                f"v{WORKFLOW_VERSION}",
                skill_name="qiongli-next",
            )
            skill_text = (skill_root / "SKILL.md").read_text(encoding="utf-8")
            self.assertIn(f"Qiongli Next version: v{WORKFLOW_VERSION}", skill_text)
            self.assertIn(f"Installed Qiongli workflow version: `v{WORKFLOW_VERSION}`", skill_text)
            validator._assert_subject_marker(skill_root, "core")
            validator._assert_subject_manifest(skill_root, "core", "complete")
            validator._assert_command_invocation(materialized_next, workflow_names, skill_name="qiongli-next")

            self.assertFalse((materialized_next / ".claude-plugin").exists())
            self.assertFalse((materialized_next / "gemini-extension.json").exists())

    def test_codex_plugin_materializes_bundled_mcp_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            materialized_plugin = self.materialize_plugin_payload(tmp_dir)
            manifest = json.loads(
                (materialized_plugin / ".codex-plugin" / "plugin.json").read_text(encoding="utf-8")
            )
            mcp_manifest = json.loads((materialized_plugin / ".mcp.json").read_text(encoding="utf-8"))

        self.assertEqual(manifest["mcpServers"], "./.mcp.json")
        self.assertEqual(
            mcp_manifest["mcpServers"]["qiongli"]["command"],
            "./bin/qiongli-literature-provider",
        )
        self.assertEqual(
            mcp_manifest["mcpServers"]["qiongli"]["args"],
            ["--transport", "stdio"],
        )

    def test_claude_plugin_materializes_bundled_mcp_server(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            materialized_plugin = self.materialize_plugin_payload(tmp_dir)
            manifest = json.loads(
                (materialized_plugin / ".claude-plugin" / "plugin.json").read_text(encoding="utf-8")
            )

            self.assertIn("mcpServers", manifest)
            self.assertIn("qiongli", manifest["mcpServers"])
            server = manifest["mcpServers"]["qiongli"]
            self.assertEqual(server["command"], "${CLAUDE_PLUGIN_ROOT}/bin/qiongli-literature-provider")
            self.assertEqual(
                server["args"],
                ["--transport", "stdio"],
            )
            self.assertEqual(server["cwd"], "${CLAUDE_PLUGIN_ROOT}")
            self.assertTrue(
                (materialized_plugin / "bin" / "qiongli-literature-provider").is_file()
            )

    def test_codex_bundled_mcp_validation_requires_plugin_manifest_reference(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            materialized_plugin = self.materialize_plugin_payload(tmp_dir)
            manifest_path = materialized_plugin / ".codex-plugin" / "plugin.json"
            manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
            del manifest["mcpServers"]
            manifest_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")

            with self.assertRaisesRegex(ValueError, "mcpServers"):
                validator._assert_bundled_literature_mcp(materialized_plugin, "codex")

    def test_plugin_package_contains_real_portable_skill_copy(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            materialized_plugin = self.materialize_plugin_payload(tmp_dir)
            plugin_skill_root = materialized_plugin / "skills" / "qiongli-workflow"

            self.assertTrue((plugin_skill_root / "SKILL.md").is_file())
            self.assertTrue((plugin_skill_root / "skills" / "registry.yaml").is_file())
            self.assertFalse(plugin_skill_root.is_symlink(), "plugin package must be a real copy, not a symlink")
            self.assertEqual(
                (WORKFLOW_ROOT / "VERSION").read_text(encoding="utf-8"),
                (plugin_skill_root / "VERSION").read_text(encoding="utf-8"),
            )
            self.assertEqual(
                (LAYOUT.skills / "registry.yaml").read_text(encoding="utf-8"),
                (plugin_skill_root / "skills" / "registry.yaml").read_text(encoding="utf-8"),
            )

    def test_sync_script_accepts_all_target_in_dry_run(self) -> None:
        bash = find_usable_bash()
        if bash is None:
            self.skipTest("usable bash is not available")

        result = subprocess.run(
            [bash, "scripts/sync_skill_package.sh", "--target", "all", "--dry-run"],
            cwd=REPO_ROOT,
            text=True,
            capture_output=True,
            check=False,
        )

        self.assertEqual(result.returncode, 0, msg=result.stderr + result.stdout)
        self.assertIn("qiongli-workflow", result.stdout)
        self.assertIn("plugins/qiongli/skills/qiongli-workflow", result.stdout)

    def test_build_artifacts_includes_direct_desktop_plugin(self) -> None:
        from scripts.build_plugin_artifacts import build_artifacts

        current_tag = (RepoLayout(REPO_ROOT).workflow / "VERSION").read_text(encoding="utf-8").strip()
        is_next = "-" in current_tag.removeprefix("v")
        plugin_name = "qiongli-next" if is_next else "qiongli"
        skill_name = "qiongli-next" if is_next else "qiongli"
        expected_name = f"{plugin_name}-claude-desktop-plugin-{current_tag}.zip"

        with tempfile.TemporaryDirectory() as tmp_dir:
            artifacts = build_artifacts(REPO_ROOT, current_tag, Path(tmp_dir))
            artifact_by_name = {artifact.name: artifact for artifact in artifacts}
            self.assertIn(expected_name, artifact_by_name)

            with zipfile.ZipFile(artifact_by_name[expected_name]) as archive:
                names = set(archive.namelist())
                manifest = json.loads(archive.read(f"{plugin_name}/plugin.json").decode("utf-8"))
                skill_text = archive.read(f"{plugin_name}/skills/qiongli-workflow/SKILL.md").decode("utf-8")

        self.assertEqual(manifest, {"name": plugin_name})
        self.assertIn(f"{plugin_name}/.claude-plugin/plugin.json", names)
        self.assertIn(f"{plugin_name}/commands/lit-review.md", names)
        self.assertIn(f"{plugin_name}/bin/qiongli-literature-provider", names)
        self.assertIn(f"{plugin_name}/skills/qiongli-workflow/SKILL.md", names)
        self.assertIn(f"name: {skill_name}", skill_text)
        self.assertNotIn(f"{plugin_name}/.codex-plugin/plugin.json", names)
        self.assertNotIn(f"{plugin_name}/.mcp.json", names)
        self.assertNotIn(f"{plugin_name}/skills/{skill_name}-lit-review/SKILL.md", names)
        self.assertFalse(
            any(
                name.startswith(f"{plugin_name}/skills/{skill_name}-")
                and not name.startswith(f"{plugin_name}/skills/qiongli-workflow/")
                for name in names
            ),
            "Claude Desktop direct plugin must not include Codex workflow wrapper skills",
        )

    def test_marketplace_validator_builds_platform_artifacts_and_checks_invocation(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            result = subprocess.run(
                [
                    sys.executable,
                    "scripts/validate_marketplace_install.py",
                    "--dist-dir",
                    tmp_dir,
                ],
                cwd=REPO_ROOT,
                text=True,
                capture_output=True,
                check=False,
            )

        self.assertEqual(result.returncode, 0, msg=result.stderr + result.stdout)
        current_tag = (RepoLayout(REPO_ROOT).workflow / "VERSION").read_text(encoding="utf-8").strip()
        if "-" in current_tag.removeprefix("v"):
            self.assertIn(
                "[OK] codex marketplace artifact (core-next): "
                "qiongli-next invocation checked; bundled literature MCP checked",
                result.stdout,
            )
            self.assertIn(
                "[OK] claude marketplace artifact (core-next): "
                "qiongli-next invocation checked; bundled literature MCP checked",
                result.stdout,
            )
            self.assertIn(
                "[OK] claude marketplace ZIP artifact (core-next): "
                "qiongli-next invocation checked; bundled literature MCP checked",
                result.stdout,
            )
            self.assertIn(
                "[OK] claude-desktop direct plugin artifact (core-next): "
                "qiongli-next invocation checked; bundled literature MCP checked",
                result.stdout,
            )
            self.assertIn("[OK] claude-desktop skill artifact (core-next)", result.stdout)
            self.assertNotIn("[OK] gemini marketplace artifact", result.stdout)
            self.assertNotIn("artifact (economics)", result.stdout)
            self.assertIn("qiongli-next invocation", result.stdout)
        else:
            self.assertIn(
                "[OK] codex marketplace artifact: qiongli invocation checked; bundled literature MCP checked",
                result.stdout,
            )
            self.assertIn(
                "[OK] claude marketplace artifact: qiongli invocation checked; bundled literature MCP checked",
                result.stdout,
            )
            self.assertIn(
                "[OK] claude marketplace ZIP artifact: qiongli invocation checked; bundled literature MCP checked",
                result.stdout,
            )
            self.assertIn(
                "[OK] claude-desktop direct plugin artifact: "
                "qiongli invocation checked; bundled literature MCP checked",
                result.stdout,
            )
            self.assertNotIn("[OK] gemini marketplace artifact", result.stdout)
            self.assertIn("[OK] codex marketplace artifact (economics):", result.stdout)
            self.assertIn("[OK] claude-desktop skill artifact (core)", result.stdout)
            self.assertNotIn("core-next", result.stdout)
        self.assertIn("under desktop file budget", result.stdout)
        self.assertIn("invocation checked", result.stdout)
        self.assertIn("bundled literature MCP checked", result.stdout)
        self.assertIn("[OK] structural archive checks completed", result.stdout)
        self.assertIn(
            "[SKIP] client CLI activation checks skipped for targets: "
            "antigravity-local-plugin, claude-code-marketplace-plugin, codex-marketplace-plugin; "
            "run scripts/release_local_install_check.py",
            result.stdout,
        )


if __name__ == "__main__":
    unittest.main()
