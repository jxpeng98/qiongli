from __future__ import annotations

import importlib.util
import json
import subprocess
import sys
import tempfile
import tomllib
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
with (REPO_ROOT / "pyproject.toml").open("rb") as handle:
    EXPECTED_PRODUCT_VERSION = tomllib.load(handle)["project"]["version"]


def _literature_mcpb_asset_name() -> str:
    manifest = json.loads(
        (REPO_ROOT / "packages" / "qiongli-literature-mcpb" / "manifest.json").read_text(
            encoding="utf-8"
        )
    )
    return f"{manifest['name']}-{manifest['version']}.mcpb"


def _load_release_download_module():
    spec = importlib.util.spec_from_file_location(
        "generate_release_downloads",
        REPO_ROOT / "tooling" / "scripts" / "generate_release_downloads.py",
    )
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def _load_validate_platform_targets_module():
    spec = importlib.util.spec_from_file_location(
        "validate_platform_targets",
        REPO_ROOT / "tooling" / "scripts" / "validate_platform_targets.py",
    )
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class ReleaseDownloadsTests(unittest.TestCase):
    def _write_valid_platform_registry(self, root: Path) -> None:
        registry = root / "content" / "distribution" / "platform-targets.yaml"
        registry.parent.mkdir(parents=True)
        registry.write_text(
            json.dumps(
                {
                    "schema_version": "1.0",
                    "targets": {
                        "fixture-target": {
                            "display_name": "Fixture Target",
                            "artifact_kind": "local-plugin",
                            "archive_format": "directory",
                            "adapter": {
                                "kind": "local-plugin",
                                "plugin_manifest_platform": "none",
                                "materializer": "local_plugin_installer",
                            },
                            "smoke": {
                                "structural_archive_check": "marketplace_validation",
                                "client_activation_check": "local_install_acceptance",
                            },
                            "source_inputs": ["content/workflow/**"],
                            "required_paths": ["plugin.json"],
                            "allowed_wrapper_dirs": [],
                            "forbidden_paths": [".codex-plugin/"],
                            "bundled_mcp_mode": "none",
                            "command_surface": "fixture-cli",
                            "validator": "fixture-validator",
                            "release_download": {
                                "guide_label": "Fixture",
                                "recommended_key": "fixture",
                                "asset_groups": [],
                            },
                        }
                    },
                },
                indent=2,
            ),
            encoding="utf-8",
        )

    def _write_valid_companion_registry(self, root: Path) -> None:
        (root / "pyproject.toml").write_text(
            '[project]\nname = "fixture"\nversion = "1.1.0b2"\n',
            encoding="utf-8",
        )
        lite_manifest = root / "packages" / "qiongli-lite-mcp" / "Cargo.toml"
        lite_manifest.parent.mkdir(parents=True, exist_ok=True)
        lite_manifest.write_text(
            '[package]\nname = "fixture-lite"\nversion = "0.2.0-beta.1"\n',
            encoding="utf-8",
        )
        mcpb_manifest = root / "packages" / "qiongli-literature-mcpb" / "manifest.json"
        mcpb_manifest.parent.mkdir(parents=True, exist_ok=True)
        mcpb_manifest.write_text(
            json.dumps({"version": "0.2.0-beta.1"}),
            encoding="utf-8",
        )
        contract = root / "content" / "mcp-contracts" / "lite-tools.json"
        contract.parent.mkdir(parents=True, exist_ok=True)
        contract.write_text(
            json.dumps({"schema_version": "1.0"}),
            encoding="utf-8",
        )
        registry = root / "content" / "distribution" / "release-companion-targets.yaml"
        registry.parent.mkdir(parents=True, exist_ok=True)
        registry.write_text(
            json.dumps(
                {
                    "schema_version": "1.0",
                    "targets": {
                        "claude_desktop_literature_mcpb": {
                            "target_id": "claude-desktop-literature-mcpb",
                            "subject": "literature",
                            "artifact_kind": "mcpb",
                            "expected_install_method": "download_mcpb",
                        },
                        "zotero_desktop_companion": {
                            "target_id": "zotero-desktop-companion-xpi",
                            "subject": "zotero",
                            "artifact_kind": "xpi",
                            "expected_install_method": "download_xpi",
                        },
                        "zotero_desktop_companion_updates": {
                            "target_id": "zotero-desktop-companion-update-manifest",
                            "subject": "zotero",
                            "artifact_kind": "release-metadata",
                            "expected_install_method": "automatic_update_manifest",
                        },
                        "download_guide": {
                            "target_id": "release-download-guide",
                            "subject": "not-applicable",
                            "artifact_kind": "release-metadata",
                            "expected_install_method": "download_markdown",
                        },
                        "download_index": {
                            "target_id": "release-download-index",
                            "subject": "not-applicable",
                            "artifact_kind": "release-metadata",
                            "expected_install_method": "download_json",
                        },
                        "artifact_manifest": {
                            "target_id": "release-artifact-manifest",
                            "subject": "not-applicable",
                            "artifact_kind": "release-metadata",
                            "expected_install_method": "download_json",
                        },
                    },
                },
                indent=2,
            ),
            encoding="utf-8",
        )

    def _platform_target_fixture(
        self,
        *,
        recommended_key: str,
        kind: str = "plugin",
    ) -> dict[str, object]:
        adapters = {
            "plugin": {
                "kind": "plugin",
                "plugin_manifest_platform": "codex",
                "materializer": "plugin_artifacts",
            },
            "claude-plugin": {
                "kind": "plugin",
                "plugin_manifest_platform": "claude",
                "materializer": "plugin_artifacts",
            },
            "skill-zip": {
                "kind": "skill-zip",
                "plugin_manifest_platform": "none",
                "materializer": "desktop_skill_artifacts",
            },
            "package": {
                "kind": "package",
                "plugin_manifest_platform": "none",
                "materializer": "npm_package",
            },
        }
        return {
            "display_name": f"Fixture {recommended_key}",
            "artifact_kind": kind,
            "archive_format": "zip" if kind == "skill-zip" else "tar.gz",
            "adapter": adapters[kind],
            "smoke": {
                "structural_archive_check": "marketplace_validation",
                "client_activation_check": "not_applicable",
            },
            "source_inputs": ["content/workflow/**"],
            "required_paths": ["plugin.json"],
            "allowed_wrapper_dirs": [],
            "forbidden_paths": [".codex-plugin/"],
            "bundled_mcp_mode": "none",
            "command_surface": "fixture-cli",
            "validator": f"fixture-{recommended_key}",
            "release_download": {
                "guide_label": f"Fixture {recommended_key}",
                "recommended_key": recommended_key,
                "asset_groups": [],
            },
        }

    def test_stable_download_section_updater_rewrites_docs(self) -> None:
        literature_mcpb_asset = _literature_mcpb_asset_name()
        targets = {
            "README.md": "## Latest Stable Downloads",
            "README_CN.md": "## 最新稳定版下载",
            "docs/index.md": "## Latest Stable Downloads",
            "docs/zh/index.md": "## 最新稳定版下载",
            "docs/guide/install.md": "## Latest Stable Downloads",
            "docs/zh/guide/install.md": "## 最新稳定版下载",
        }

        with tempfile.TemporaryDirectory() as tmp_dir:
            docs_root = Path(tmp_dir)
            for relative_path, heading in targets.items():
                path = docs_root / relative_path
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(
                    "\n".join(
                        [
                            f"# Fixture for {relative_path}",
                            "",
                            heading,
                            "",
                            "Current stable release: [v1.5.0](https://github.com/jxpeng98/qiongli/releases/tag/v1.5.0).",
                            "",
                            "| Need | Link or command |",
                            "|---|---|",
                            "| npm CLI | stale v1.5.0 link |",
                            "",
                            "## After",
                            "",
                            "Keep this trailing section.",
                            "",
                        ]
                    ),
                    encoding="utf-8",
                )

            result = subprocess.run(
                [
                    sys.executable,
                    "scripts/update_stable_download_sections.py",
                    "--tag",
                    "v1.6.0",
                    "--root",
                    str(docs_root),
                    "--asset-root",
                    str(REPO_ROOT),
                ],
                cwd=REPO_ROOT,
                text=True,
                capture_output=True,
                check=False,
            )

            self.assertEqual(result.returncode, 0, msg=result.stderr)
            self.assertIn("updated stable download sections: 6 files", result.stdout)

            for relative_path, heading in targets.items():
                content = (docs_root / relative_path).read_text(encoding="utf-8")
                self.assertIn(f"# Fixture for {relative_path}", content)
                self.assertIn(heading, content)
                self.assertIn("## After", content)
                self.assertIn("Keep this trailing section.", content)
                self.assertNotIn("v1.5.0", content)
                self.assertIn("[v1.6.0](https://github.com/jxpeng98/qiongli/releases/tag/v1.6.0)", content)
                self.assertIn("qiongli-claude-desktop-skill-core-v1.6.0.zip", content)
                self.assertIn("qiongli-claude-desktop-plugin-v1.6.0.zip", content)
                self.assertIn(literature_mcpb_asset, content)
                self.assertIn("qiongli-zotero-companion-0.3.0.xpi", content)
                self.assertIn("qiongli-downloads-v1.6.0.md", content)

            english = (docs_root / "README.md").read_text(encoding="utf-8")
            chinese = (docs_root / "README_CN.md").read_text(encoding="utf-8")
            self.assertIn("Current stable release:", english)
            self.assertIn("| Claude Desktop recommended plugin |", english)
            self.assertIn("| Claude Desktop/Web fallback skill ZIP |", english)
            self.assertIn("| All release assets |", english)
            self.assertIn("当前稳定版是", chinese)
            self.assertIn("| Claude Desktop 推荐插件 |", chinese)
            self.assertIn("| Claude Desktop/Web fallback skill ZIP |", chinese)
            self.assertIn("| 全部 release assets |", chinese)

    def test_generates_human_and_machine_download_guides(self) -> None:
        literature_mcpb_asset = _literature_mcpb_asset_name()
        with tempfile.TemporaryDirectory() as tmp_dir:
            out_dir = Path(tmp_dir)
            result = subprocess.run(
                [
                    sys.executable,
                    "scripts/generate_release_downloads.py",
                    "--tag",
                    "v1.1.0-beta.2",
                    "--out-dir",
                    str(out_dir),
                ],
                cwd=REPO_ROOT,
                text=True,
                capture_output=True,
                check=False,
            )

            self.assertEqual(result.returncode, 0, msg=result.stderr)
            guide_path = out_dir / "qiongli-downloads-v1.1.0-beta.2.md"
            index_path = out_dir / "qiongli-downloads-v1.1.0-beta.2.json"
            manifest_path = out_dir / "qiongli-artifacts-v1.1.0-beta.2.json"
            self.assertIn(str(guide_path), result.stdout)
            self.assertIn(str(index_path), result.stdout)
            self.assertIn(str(manifest_path), result.stdout)

            guide = guide_path.read_text(encoding="utf-8")
            index = json.loads(index_path.read_text(encoding="utf-8"))
            manifest = json.loads(manifest_path.read_text(encoding="utf-8"))

        self.assertIn("# Qiongli v1.1.0-beta.2 Download Guide", guide)
        self.assertIn("## Direct downloads", guide)
        self.assertIn("Start here", guide)
        self.assertIn("they are not generic multi-platform binaries", guide)
        self.assertIn(
            index["component_versions"]["lite_mcp"]["native_target"],
            guide,
        )
        self.assertIn("https://github.com/jxpeng98/qiongli/releases/download/v1.1.0-beta.2/qiongli-next-claude-desktop-skill-core-v1.1.0-beta.2.zip", guide)
        self.assertIn("qiongli-next-claude-desktop-plugin-v1.1.0-beta.2.zip", guide)
        self.assertIn("recommended direct plugin", guide)
        self.assertIn("fallback skill ZIP", guide)
        self.assertIn("npx qiongli@next install --target all", guide)
        self.assertIn("marketplace dist refs are not advanced", guide)
        self.assertIn("only when the bundled target identity matches", guide)
        self.assertIn("qiongli-next-claude-desktop-skill-core-v1.1.0-beta.2.zip", guide)
        self.assertIn(literature_mcpb_asset, guide)
        self.assertIn("qiongli-zotero-companion-0.3.0.xpi", guide)
        self.assertIn("qiongli-zotero-companion-updates.json", guide)
        self.assertIn("qiongli-next-claude-plugin-v1.1.0-beta.2.zip", guide)
        self.assertIn("qiongli-downloads-v1.1.0-beta.2.json", guide)

        self.assertEqual(index["tag"], "v1.1.0-beta.2")
        self.assertEqual(index["channel"], "next")
        self.assertEqual(
            index["component_versions"]["product"]["version"],
            EXPECTED_PRODUCT_VERSION,
        )
        self.assertEqual(
            index["component_versions"]["lite_mcp"]["version"],
            index["component_versions"]["literature_mcpb"]["version"],
        )
        self.assertEqual(
            index["component_versions"]["lite_mcp"]["target_policy"],
            "current-host-only",
        )
        self.assertIn("native_target", index["component_versions"]["lite_mcp"])
        self.assertEqual(index["component_versions"]["lite_contract"]["version"], "1.0")
        self.assertEqual(index["release_url"], "https://github.com/jxpeng98/qiongli/releases/tag/v1.1.0-beta.2")
        self.assertEqual(
            index["companion_target_registry"]["path"],
            "content/distribution/release-companion-targets.yaml",
        )
        self.assertEqual(index["companion_target_registry"]["schema_version"], "1.0")
        self.assertEqual(index["recommended"]["qiongli_cli"]["install"], "npm_next")
        self.assertEqual(
            index["recommended"]["codex"]["install"],
            "download_matching_native_asset",
        )
        self.assertEqual(index["recommended"]["codex"]["plugin"], "qiongli-next")
        self.assertEqual(
            index["recommended"]["codex"]["marketplace_dist_ref"],
            "paused_current_host_only",
        )

        self.assertEqual(
            index["recommended"]["codex"]["manual_asset"],
            "qiongli-next-codex-plugin-v1.1.0-beta.2.tar.gz",
        )
        self.assertEqual(
            index["recommended"]["claude_code"]["install"],
            "download_matching_native_asset",
        )
        self.assertEqual(index["recommended"]["claude_code"]["plugin"], "qiongli-next")
        self.assertEqual(
            index["recommended"]["claude_code"]["manual_asset"],
            "qiongli-next-claude-plugin-v1.1.0-beta.2.zip",
        )
        self.assertEqual(
            index["recommended"]["claude_desktop_plugin"]["asset"],
            "qiongli-next-claude-desktop-plugin-v1.1.0-beta.2.zip",
        )
        self.assertEqual(
            index["recommended"]["claude_desktop_literature_mcpb"]["asset"],
            literature_mcpb_asset,
        )
        self.assertEqual(
            index["recommended"]["zotero_desktop_companion"]["asset"],
            "qiongli-zotero-companion-0.3.0.xpi",
        )
        self.assertEqual(
            index["recommended"]["zotero_desktop_companion"][
                "automatic_update_manifest"
            ],
            "qiongli-zotero-companion-updates.json",
        )
        self.assertEqual(
            index["recommended"]["zotero_desktop_companion"][
                "automatic_update_channel"
            ],
            "latest-stable",
        )
        self.assertEqual(
            index["assets"]["zotero_desktop_companion"],
            "qiongli-zotero-companion-0.3.0.xpi",
        )
        self.assertEqual(
            index["assets"]["zotero_desktop_companion_updates"],
            "qiongli-zotero-companion-updates.json",
        )
        self.assertEqual(
            index["assets"]["claude_desktop_plugin"],
            "qiongli-next-claude-desktop-plugin-v1.1.0-beta.2.zip",
        )
        self.assertEqual(
            index["assets"]["artifact_manifest"],
            "qiongli-artifacts-v1.1.0-beta.2.json",
        )
        self.assertEqual(
            index["asset_urls"]["claude_desktop_plugin"],
            "https://github.com/jxpeng98/qiongli/releases/download/v1.1.0-beta.2/qiongli-next-claude-desktop-plugin-v1.1.0-beta.2.zip",
        )
        self.assertIn(
            "qiongli-next-claude-desktop-skill-core-v1.1.0-beta.2.zip",
            index["assets"]["claude_desktop_skills"],
        )
        self.assertIn(
            "qiongli-next-claude-plugin-v1.1.0-beta.2.tar.gz",
            index["assets"]["maintainer_plugin_tarballs"],
        )
        self.assertIn(
            "qiongli-next-claude-plugin-v1.1.0-beta.2.zip",
            index["assets"]["maintainer_plugin_zips"],
        )
        self.assertIn("## Platform target registry", guide)
        self.assertIn("codex-marketplace-plugin", guide)
        self.assertIn("claude-desktop-direct-plugin", guide)
        self.assertIn("npm-plugin-lite", guide)
        self.assertIn("pypi-full-runtime", guide)
        self.assertEqual(index["recommended"]["codex"]["target_id"], "codex-marketplace-plugin")
        self.assertEqual(index["recommended"]["claude_code"]["target_id"], "claude-code-marketplace-plugin")
        self.assertEqual(index["recommended"]["claude_desktop_plugin"]["target_id"], "claude-desktop-direct-plugin")
        self.assertEqual(index["recommended"]["claude_desktop_skill"]["target_id"], "claude-desktop-skill-zip")
        self.assertEqual(index["recommended"]["qiongli_cli"]["target_id"], "npm-plugin-lite")
        self.assertEqual(
            index["platform_targets"]["claude-desktop-direct-plugin"]["display_name"],
            "Claude Desktop Direct Plugin",
        )
        self.assertEqual(
            index["platform_targets"]["claude-desktop-direct-plugin"]["archive_format"],
            "zip",
        )
        self.assertEqual(
            index["platform_targets"]["codex-marketplace-plugin"]["adapter"]["plugin_manifest_platform"],
            "codex",
        )
        self.assertEqual(
            index["platform_targets"]["codex-marketplace-plugin"]["adapter"]["materializer"],
            "plugin_artifacts",
        )
        self.assertEqual(
            index["platform_targets"]["codex-marketplace-plugin"]["smoke"][
                "client_activation_check"
            ],
            "local_install_acceptance",
        )
        self.assertEqual(
            index["assets_by_target"]["claude-desktop-direct-plugin"]["claude_desktop_plugin"],
            "qiongli-next-claude-desktop-plugin-v1.1.0-beta.2.zip",
        )
        self.assertIn(
            "qiongli-next-codex-plugin-v1.1.0-beta.2.tar.gz",
            index["assets_by_target"]["codex-marketplace-plugin"]["maintainer_plugin_tarballs"],
        )
        self.assertNotIn(
            "qiongli-next-claude-plugin-v1.1.0-beta.2.tar.gz",
            index["assets_by_target"]["codex-marketplace-plugin"]["maintainer_plugin_tarballs"],
        )
        companion_assets = index["assets_by_target"]
        self.assertEqual(
            companion_assets["claude-desktop-literature-mcpb"][
                "claude_desktop_literature_mcpb"
            ],
            literature_mcpb_asset,
        )
        self.assertEqual(
            companion_assets["zotero-desktop-companion-xpi"]["zotero_desktop_companion"],
            "qiongli-zotero-companion-0.3.0.xpi",
        )
        self.assertEqual(
            companion_assets["zotero-desktop-companion-update-manifest"][
                "zotero_desktop_companion_updates"
            ],
            "qiongli-zotero-companion-updates.json",
        )
        self.assertEqual(
            companion_assets["release-download-guide"]["download_guide"],
            "qiongli-downloads-v1.1.0-beta.2.md",
        )
        self.assertEqual(
            companion_assets["release-download-index"]["download_index"],
            "qiongli-downloads-v1.1.0-beta.2.json",
        )
        self.assertEqual(
            companion_assets["release-artifact-manifest"]["artifact_manifest"],
            "qiongli-artifacts-v1.1.0-beta.2.json",
        )
        self.assertNotIn(
            "qiongli-economics-claude-plugin-v1.1.0-beta.2.tar.gz",
            index["assets"]["maintainer_plugin_tarballs"],
        )
        self.assertNotIn(
            "qiongli-economics-claude-plugin-v1.1.0-beta.2.zip",
            index["assets"]["maintainer_plugin_zips"],
        )
        self.assertEqual(manifest["schema_version"], "1.0")
        self.assertEqual(manifest["tag"], "v1.1.0-beta.2")
        codex_record = next(
            item
            for item in manifest["artifacts"]
            if item["asset"] == "qiongli-next-codex-plugin-v1.1.0-beta.2.tar.gz"
        )
        self.assertEqual(codex_record["target_id"], "codex-marketplace-plugin")
        self.assertEqual(codex_record["archive_format"], "tar.gz")
        self.assertEqual(
            codex_record["expected_install_method"],
            "download_matching_native_asset",
        )
        self.assertIn(".claude-plugin/", codex_record["forbidden_paths"])
        self.assertEqual(
            codex_record["smoke"]["structural_archive_check"],
            "marketplace_validation",
        )
        self.assertEqual(
            codex_record["smoke"]["client_activation_check"],
            "local_install_acceptance",
        )
        self.assertEqual(
            codex_record["adapter"]["materializer"],
            "plugin_artifacts",
        )
        desktop_record = next(
            item
            for item in manifest["artifacts"]
            if item["asset"] == "qiongli-next-claude-desktop-plugin-v1.1.0-beta.2.zip"
        )
        self.assertEqual(desktop_record["target_id"], "claude-desktop-direct-plugin")
        self.assertEqual(desktop_record["subject"], "core")
        self.assertEqual(
            desktop_record["adapter"]["materializer"],
            "plugin_artifacts",
        )
        claude_zip_record = next(
            item
            for item in manifest["artifacts"]
            if item["asset"] == "qiongli-next-claude-plugin-v1.1.0-beta.2.zip"
        )
        self.assertEqual(claude_zip_record["archive_format"], "zip")
        mcpb_record = next(
            item
            for item in manifest["artifacts"]
            if item["asset"] == literature_mcpb_asset
        )
        self.assertEqual(mcpb_record["target_id"], "claude-desktop-literature-mcpb")
        self.assertEqual(mcpb_record["expected_install_method"], "download_mcpb")
        self.assertEqual(mcpb_record["artifact_kind"], "mcpb")
        self.assertFalse(mcpb_record["registry_target"])
        self.assertEqual(mcpb_record["native_variant"]["policy"], "current-host-only")
        self.assertEqual(
            mcpb_record["native_variant"]["target_triple"],
            index["component_versions"]["lite_mcp"]["native_target"],
        )
        zotero_record = next(
            item
            for item in manifest["artifacts"]
            if item["asset"] == "qiongli-zotero-companion-0.3.0.xpi"
        )
        self.assertEqual(zotero_record["target_id"], "zotero-desktop-companion-xpi")
        self.assertEqual(zotero_record["expected_install_method"], "download_xpi")
        zotero_update_record = next(
            item
            for item in manifest["artifacts"]
            if item["asset"] == "qiongli-zotero-companion-updates.json"
        )
        self.assertEqual(
            zotero_update_record["target_id"],
            "zotero-desktop-companion-update-manifest",
        )
        self.assertEqual(
            zotero_update_record["expected_install_method"],
            "automatic_update_manifest",
        )
        manifest_record = next(
            item
            for item in manifest["artifacts"]
            if item["asset"] == "qiongli-artifacts-v1.1.0-beta.2.json"
        )
        self.assertEqual(manifest_record["target_id"], "release-artifact-manifest")
        self.assertNotIn(
            "release-companion",
            {item["target_id"] for item in manifest["artifacts"]},
        )
        self.assertEqual(
            index["companion_targets"]["artifact_manifest"]["target_id"],
            "release-artifact-manifest",
        )
        self.assertEqual(manifest["component_versions"], index["component_versions"])

    def test_native_alpha_is_rejected_by_legacy_download_generator(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            result = subprocess.run(
                [
                    sys.executable,
                    "scripts/generate_release_downloads.py",
                    "--tag",
                    "v2.0.0-alpha.1",
                    "--out-dir",
                    tmp_dir,
                ],
                cwd=REPO_ROOT,
                text=True,
                capture_output=True,
                check=False,
            )

            self.assertNotEqual(result.returncode, 0)
            self.assertIn("native 2.x release metadata is isolated", result.stderr)
            self.assertEqual(list(Path(tmp_dir).iterdir()), [])

    def test_recommended_target_ids_follow_registry_recommended_keys(self) -> None:
        module = _load_release_download_module()

        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            registry = root / "content" / "distribution" / "platform-targets.yaml"
            registry.parent.mkdir(parents=True, exist_ok=True)
            registry.write_text(
                json.dumps(
                    {
                        "schema_version": "1.0",
                        "targets": {
                            "fixture-npm-target": self._platform_target_fixture(
                                recommended_key="qiongli_cli",
                                kind="package",
                            ),
                            "fixture-codex-target": self._platform_target_fixture(
                                recommended_key="codex",
                            ),
                            "fixture-claude-target": self._platform_target_fixture(
                                recommended_key="claude_code",
                                kind="claude-plugin",
                            ),
                            "fixture-desktop-skill-target": self._platform_target_fixture(
                                recommended_key="claude_desktop_skill",
                                kind="skill-zip",
                            ),
                            "fixture-desktop-plugin-target": self._platform_target_fixture(
                                recommended_key="claude_desktop_plugin",
                                kind="claude-plugin",
                            ),
                        },
                    },
                    indent=2,
                ),
                encoding="utf-8",
            )
            self._write_valid_companion_registry(root)

            index = module.build_index("v1.1.0-beta.2", root=root)

        self.assertEqual(index["recommended"]["qiongli_cli"]["target_id"], "fixture-npm-target")
        self.assertEqual(index["recommended"]["codex"]["target_id"], "fixture-codex-target")
        self.assertEqual(index["recommended"]["claude_code"]["target_id"], "fixture-claude-target")
        self.assertEqual(
            index["recommended"]["claude_desktop_skill"]["target_id"],
            "fixture-desktop-skill-target",
        )
        self.assertEqual(
            index["recommended"]["claude_desktop_plugin"]["target_id"],
            "fixture-desktop-plugin-target",
        )

    def test_release_guides_label_recommended_targets_from_index(self) -> None:
        module = _load_release_download_module()

        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            registry = root / "content" / "distribution" / "platform-targets.yaml"
            registry.parent.mkdir(parents=True, exist_ok=True)
            registry.write_text(
                json.dumps(
                    {
                        "schema_version": "1.0",
                        "targets": {
                            "fixture-npm-target": self._platform_target_fixture(
                                recommended_key="qiongli_cli",
                                kind="package",
                            ),
                            "fixture-codex-target": self._platform_target_fixture(
                                recommended_key="codex",
                            ),
                            "fixture-claude-target": self._platform_target_fixture(
                                recommended_key="claude_code",
                                kind="claude-plugin",
                            ),
                            "fixture-desktop-skill-target": self._platform_target_fixture(
                                recommended_key="claude_desktop_skill",
                                kind="skill-zip",
                            ),
                            "fixture-desktop-plugin-target": self._platform_target_fixture(
                                recommended_key="claude_desktop_plugin",
                                kind="claude-plugin",
                            ),
                        },
                    },
                    indent=2,
                ),
                encoding="utf-8",
            )
            self._write_valid_companion_registry(root)
            index = module.build_index("v1.1.0-beta.2", root=root)

        guide = module.render_markdown(index)
        notes = module.render_release_notes_download_summary(index)

        self.assertIn("Fixture qiongli_cli (`fixture-npm-target`)", guide)
        self.assertIn("Fixture codex (`fixture-codex-target`)", guide)
        self.assertIn("Fixture claude_code (`fixture-claude-target`)", guide)
        self.assertIn("Fixture claude_desktop_plugin (`fixture-desktop-plugin-target`)", guide)
        self.assertIn("Fixture claude_desktop_skill (`fixture-desktop-skill-target`)", guide)
        self.assertIn("Fixture qiongli_cli (`fixture-npm-target`)", notes)
        self.assertIn("Fixture codex (`fixture-codex-target`)", notes)
        self.assertIn("not generic multi-platform binaries", notes)
        self.assertIn("generic marketplace dist ref is not advanced", notes)

    def test_release_companion_target_registry_rejects_missing_metadata(self) -> None:
        module = _load_release_download_module()

        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            registry = root / "content" / "distribution" / "release-companion-targets.yaml"
            registry.parent.mkdir(parents=True)
            registry.write_text(
                json.dumps(
                    {
                        "schema_version": "1.0",
                        "targets": {
                            "fixture_asset": {
                                "target_id": "fixture-target",
                                "subject": "fixture",
                                "artifact_kind": "fixture",
                            }
                        },
                    }
                ),
                encoding="utf-8",
            )

            with self.assertRaisesRegex(ValueError, "expected_install_method"):
                module.load_companion_targets(root)

    def test_release_companion_target_registry_rejects_missing_required_key(self) -> None:
        module = _load_release_download_module()

        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            registry = root / "content" / "distribution" / "release-companion-targets.yaml"
            registry.parent.mkdir(parents=True)
            registry.write_text(
                json.dumps(
                    {
                        "schema_version": "1.0",
                        "targets": {
                            "claude_desktop_literature_mcpb": {
                                "target_id": "claude-desktop-literature-mcpb",
                                "subject": "literature",
                                "artifact_kind": "mcpb",
                                "expected_install_method": "download_mcpb",
                            },
                            "zotero_desktop_companion": {
                                "target_id": "zotero-desktop-companion-xpi",
                                "subject": "zotero",
                                "artifact_kind": "xpi",
                                "expected_install_method": "download_xpi",
                            },
                            "zotero_desktop_companion_updates": {
                                "target_id": "zotero-desktop-companion-update-manifest",
                                "subject": "zotero",
                                "artifact_kind": "release-metadata",
                                "expected_install_method": "automatic_update_manifest",
                            },
                            "download_guide": {
                                "target_id": "release-download-guide",
                                "subject": "not-applicable",
                                "artifact_kind": "release-metadata",
                                "expected_install_method": "download_markdown",
                            },
                            "download_index": {
                                "target_id": "release-download-index",
                                "subject": "not-applicable",
                                "artifact_kind": "release-metadata",
                                "expected_install_method": "download_json",
                            },
                        },
                    }
                ),
                encoding="utf-8",
            )

            with self.assertRaisesRegex(
                ValueError,
                "missing required companion target keys: artifact_manifest",
            ):
                module.load_companion_targets(root)

    def test_release_target_registry_validator_reports_companion_failures(self) -> None:
        module = _load_validate_platform_targets_module()

        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            self._write_valid_platform_registry(root)
            companion = root / "content" / "distribution" / "release-companion-targets.yaml"
            companion.write_text(
                json.dumps(
                    {
                        "schema_version": "1.0",
                        "targets": {
                            "fixture_asset": {
                                "target_id": "fixture-target",
                                "subject": "fixture",
                                "artifact_kind": "fixture",
                            }
                        },
                    }
                ),
                encoding="utf-8",
            )

            failures = module.validate_release_target_registries(root)

        self.assertTrue(
            any("release companion target registry" in failure for failure in failures),
            failures,
        )
        self.assertTrue(any("expected_install_method" in failure for failure in failures), failures)

    def test_release_notes_include_download_guide_section(self) -> None:
        literature_mcpb_asset = _literature_mcpb_asset_name()
        with tempfile.TemporaryDirectory() as tmp_dir:
            note_path = Path(tmp_dir) / "notes.md"
            result = subprocess.run(
                [
                    "bash",
                    "scripts/generate_release_notes.sh",
                    "--tag",
                    "v1.1.0-beta.2",
                    "--from-tag",
                    "v1.1.0-beta.1",
                    "--output",
                    str(note_path),
                    "--overwrite",
                ],
                cwd=REPO_ROOT,
                text=True,
                capture_output=True,
                check=False,
            )

            self.assertEqual(result.returncode, 0, msg=result.stderr)
            notes = note_path.read_text(encoding="utf-8")

        self.assertIn("## Download Guide", notes)
        self.assertIn("Most users should not download plugin tarballs manually", notes)
        self.assertIn("Qiongli npm/npx CLI (`npm-plugin-lite`)", notes)
        self.assertIn("Claude Desktop direct plugin (`claude-desktop-direct-plugin`)", notes)
        self.assertIn("qiongli-downloads-v1.1.0-beta.2.md", notes)
        self.assertIn("qiongli-artifacts-v1.1.0-beta.2.json", notes)
        self.assertIn("not generic multi-platform binaries", notes)
        self.assertIn("qiongli-next-claude-desktop-plugin-v1.1.0-beta.2.zip", notes)
        self.assertIn("recommended direct plugin", notes)
        self.assertIn("fallback skill ZIP", notes)
        self.assertIn("qiongli-next-claude-desktop-skill-core-v1.1.0-beta.2.zip", notes)
        self.assertIn(literature_mcpb_asset, notes)
        self.assertIn("qiongli-zotero-companion-0.3.0.xpi", notes)
        self.assertIn("qiongli-zotero-companion-updates.json", notes)
        self.assertIn("Claude plugin ZIPs", notes)

    def test_stable_release_notes_include_category_downloads_and_changelog(self) -> None:
        literature_mcpb_asset = _literature_mcpb_asset_name()
        with tempfile.TemporaryDirectory() as tmp_dir:
            note_path = Path(tmp_dir) / "stable-notes.md"
            result = subprocess.run(
                [
                    sys.executable,
                    "scripts/generate_stable_release_notes.py",
                    "--tag",
                    "v1.5.0",
                    "--output",
                    str(note_path),
                ],
                cwd=REPO_ROOT,
                text=True,
                capture_output=True,
                check=False,
            )

            self.assertEqual(result.returncode, 0, msg=result.stderr)
            notes = note_path.read_text(encoding="utf-8")

        self.assertIn("## Release Category", notes)
        self.assertIn("## Download Guide", notes)
        self.assertIn("npm install -g qiongli@latest", notes)
        self.assertIn("pipx install qiongli", notes)
        self.assertIn("qiongli-claude-desktop-plugin-v1.5.0.zip", notes)
        self.assertIn("recommended direct plugin", notes)
        self.assertIn("fallback skill ZIP", notes)
        self.assertIn("qiongli-claude-desktop-skill-core-v1.5.0.zip", notes)
        self.assertIn(literature_mcpb_asset, notes)
        self.assertIn("qiongli-zotero-companion-0.3.0.xpi", notes)
        self.assertIn("qiongli-zotero-companion-updates.json", notes)
        self.assertIn("qiongli-downloads-v1.5.0.md", notes)
        self.assertIn("qiongli-artifacts-v1.5.0.json", notes)
        self.assertIn("## Changelog", notes)
        self.assertIn("### [1.5.0] - 2026-06-23", notes)


if __name__ == "__main__":
    unittest.main()
