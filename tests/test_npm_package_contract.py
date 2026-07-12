from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from qiongli.source_layout import RepoLayout


REPO_ROOT = Path(__file__).resolve().parents[1]
LAYOUT = RepoLayout(REPO_ROOT)
NPM_PACKAGE_ROOT = REPO_ROOT / "packages" / "npm-qiongli"
MATERIALIZER = LAYOUT.scripts / "materialize_distribution_payloads.py"


class NpmPackageContractTests(unittest.TestCase):
    def test_npm_package_manifest_is_public_qiongli_launcher(self) -> None:
        package_json = json.loads((NPM_PACKAGE_ROOT / "package.json").read_text(encoding="utf-8"))

        self.assertEqual(package_json["name"], "qiongli")
        self.assertEqual(
            package_json["description"],
            "Qiongli Python-free academic workflow asset manager.",
        )
        self.assertEqual(
            package_json["author"],
            {"name": "Jiaxin Peng", "url": "https://github.com/jxpeng98"},
        )
        self.assertEqual(package_json["bin"], {"qiongli": "bin/qiongli.mjs"})
        self.assertEqual(package_json["engines"]["node"], ">=18")
        self.assertNotIn("postinstall", package_json.get("scripts", {}))
        self.assertEqual(
            sorted(package_json["files"]),
            sorted(["bin/", "lib/", "payload/", "python-runtime/", "README.md", "LICENSE"]),
        )

    def test_root_package_declares_npm_workspace_without_changing_docs_identity(self) -> None:
        root_package_json = json.loads((REPO_ROOT / "package.json").read_text(encoding="utf-8"))

        self.assertEqual(root_package_json["name"], "qiongli-docs")
        self.assertTrue(root_package_json["private"])
        self.assertIn("packages/npm-qiongli", root_package_json["workspaces"])

    def test_root_package_exposes_validate_for_marketplace_installability(self) -> None:
        root_package_json = json.loads((REPO_ROOT / "package.json").read_text(encoding="utf-8"))

        validate_script = root_package_json["scripts"]["validate"]
        self.assertIn("scripts/validate_marketplace_install.py", validate_script)
        self.assertIn("npm --prefix packages/npm-qiongli test", validate_script)

    def test_package_lock_tracks_workspace_version(self) -> None:
        package_json = json.loads((NPM_PACKAGE_ROOT / "package.json").read_text(encoding="utf-8"))
        package_lock = json.loads((REPO_ROOT / "package-lock.json").read_text(encoding="utf-8"))

        self.assertEqual(
            package_lock["packages"]["packages/npm-qiongli"]["version"],
            package_json["version"],
        )

    def test_sync_versions_exposes_npm_semver_prerelease(self) -> None:
        result = subprocess.run(
            [
                "python3",
                "scripts/sync_versions.py",
                "0.8.0b1",
                "--print-field",
                "npm_version",
            ],
            cwd=REPO_ROOT,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout.strip(), "0.8.0-beta.1")

    def test_npm_payload_bundles_plugin_lite_assets_and_transitional_python_runtime(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            materialized_root = Path(tmp) / "qiongli-dist"
            subprocess.run(
                [
                    sys.executable,
                    str(MATERIALIZER),
                    "--target",
                    "npm",
                    "--out",
                    str(materialized_root),
                    "--force",
                ],
                cwd=REPO_ROOT,
                check=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
            )
            package_root = materialized_root / "packages" / "npm-qiongli"
            workflow_root = package_root / "payload" / "qiongli-workflow"
            plugin_payload_root = package_root / "payload" / "plugins"
            runtime_root = package_root / "python-runtime"

            self._assert_npm_payload_and_runtime(package_root, workflow_root, plugin_payload_root, runtime_root)

    def _assert_npm_payload_and_runtime(
        self,
        package_root: Path,
        workflow_root: Path,
        plugin_payload_root: Path,
        runtime_root: Path,
    ) -> None:
        package_json = json.loads((package_root / "package.json").read_text(encoding="utf-8"))

        self.assertTrue((workflow_root / "SKILL.md").is_file())
        self.assertTrue((workflow_root / "workflows" / "paper.md").is_file())
        self.assertTrue((workflow_root / "templates" / "search-diagnostics.md").is_file())
        self.assertEqual(
            package_json["version"],
            (workflow_root / "VERSION").read_text(encoding="utf-8").strip().removeprefix("v"),
        )
        self.assertEqual(
            (LAYOUT.skills / "registry.yaml").read_text(encoding="utf-8"),
            (workflow_root / "skills" / "registry.yaml").read_text(encoding="utf-8"),
        )
        cli_source = (package_root / "payload" / "scripts" / "qiongli_cli.sh").read_text(encoding="utf-8")
        bootstrap_source = (package_root / "payload" / "scripts" / "bootstrap_qiongli.sh").read_text(encoding="utf-8")
        self.assertIn('CLI_FLAVOR="shell-bootstrap"', cli_source)
        self.assertIn("qiongli <command>", cli_source)
        self.assertIn('DEFAULT_REPO="jxpeng98/qiongli"', bootstrap_source)
        self.assertIn("--profile <partial|full>", bootstrap_source)
        self._assert_plugin_lite_payload(plugin_payload_root)
        self._assert_platform_target_registry(package_root)

        self.assertTrue((runtime_root / "bridges" / "__init__.py").is_file())
        self.assertTrue((runtime_root / "qiongli" / "bridges" / "orchestrator.py").is_file())
        self.assertTrue((runtime_root / "qiongli" / "bridges" / "mcp_cli.py").is_file())
        self.assertTrue((runtime_root / "qiongli" / "bridges" / "mcp_server_stdio.py").is_file())
        self.assertTrue((runtime_root / "qiongli" / "bridges" / "providers" / "literature_search.py").is_file())
        self.assertTrue((runtime_root / "qiongli" / "self_update.py").is_file())
        self.assertTrue((runtime_root / "scripts" / "validate_project_artifacts.py").is_file())
        self.assertTrue((runtime_root / "qiongli" / "workflow_contract_doc.py").is_file())
        self.assertTrue((runtime_root / "standards" / "research-workflow-contract.yaml").is_file())
        self.assertTrue((runtime_root / "skills" / "registry.yaml").is_file())
        self.assertEqual(
            (LAYOUT.python_package / "__init__.py").read_text(encoding="utf-8"),
            (runtime_root / "qiongli" / "__init__.py").read_text(encoding="utf-8"),
        )
        self.assertEqual(
            (LAYOUT.skills / "registry.yaml").read_text(encoding="utf-8"),
            (runtime_root / "skills" / "registry.yaml").read_text(encoding="utf-8"),
        )

    def _assert_plugin_lite_payload(self, plugin_payload_root: Path) -> None:
        fallback_plugin_root = plugin_payload_root / "qiongli"
        codex_plugin_root = plugin_payload_root / "codex" / "qiongli"
        claude_plugin_root = plugin_payload_root / "claude" / "qiongli"

        self.assertTrue(
            fallback_plugin_root.is_dir(),
            msg=f"expected shared plugin-lite fallback at {fallback_plugin_root}",
        )
        self.assertTrue(
            (fallback_plugin_root / "skills" / "qiongli-workflow" / "SKILL.md").is_file(),
            msg="expected bundled plugin-lite skill entrypoint in npm payload",
        )
        self.assertTrue(
            (fallback_plugin_root / "bin" / "qiongli-literature-provider").is_file(),
            msg="expected bundled Rust Lite MCP provider binary in npm payload",
        )
        self.assertTrue(
            (fallback_plugin_root / ".codex-plugin" / "plugin.json").is_file()
            or (fallback_plugin_root / ".claude-plugin" / "plugin.json").is_file()
            or (fallback_plugin_root / "plugin.json").is_file(),
            msg="expected at least one recognized qiongli plugin manifest in npm plugin-lite payload",
        )
        for target_root in (codex_plugin_root, claude_plugin_root):
            if target_root.exists():
                self.assertTrue(
                    (target_root / "skills" / "qiongli-workflow" / "SKILL.md").is_file(),
                    msg=f"expected plugin-lite skill entrypoint in target payload {target_root}",
                )
                self.assertTrue(
                    (target_root / "bin" / "qiongli-literature-provider").is_file(),
                    msg=f"expected bundled Rust Lite MCP provider binary in target payload {target_root}",
                )

    def _assert_platform_target_registry(self, package_root: Path) -> None:
        target_registry = json.loads(
            (
                package_root
                / "payload"
                / "content"
                / "distribution"
                / "platform-targets.json"
            ).read_text(encoding="utf-8")
        )
        npm_target = target_registry["targets"]["npm-plugin-lite"]

        self.assertEqual(npm_target["target_id"], "npm-plugin-lite")
        self.assertEqual(npm_target["artifact_kind"], "npm-package")
        self.assertEqual(npm_target["archive_format"], "npm-tarball")
        self.assertEqual(npm_target["command_surface"], "npx-cli")
        self.assertEqual(npm_target["validator"], "npm-plugin-lite")

    def test_transitional_python_runtime_packaging_resolves_payload_root(self) -> None:
        env = os.environ.copy()
        with tempfile.TemporaryDirectory() as temp_dir:
            materialized_root = Path(temp_dir) / "qiongli-dist"
            subprocess.run(
                [
                    sys.executable,
                    str(MATERIALIZER),
                    "--target",
                    "npm",
                    "--out",
                    str(materialized_root),
                    "--force",
                ],
                cwd=REPO_ROOT,
                check=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
            )
            package_root = materialized_root / "packages" / "npm-qiongli"
            expected_payload_root = (package_root / "payload").resolve()
            runtime_root = package_root / "python-runtime"
            self.assertTrue(
                runtime_root.is_dir(),
                msg="expected transitional python-runtime packaging to remain available in the npm payload",
            )
            env["PYTHONPATH"] = str(runtime_root)
            result = subprocess.run(
                [
                    sys.executable,
                    "-c",
                    "from qiongli.setup_wizard import _packaged_payload_root; print(_packaged_payload_root())",
                ],
                cwd=Path(temp_dir),
                env=env,
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                check=False,
            )

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(str(Path(result.stdout.strip()).resolve()), str(expected_payload_root))

    def test_release_workflows_cover_pypi_and_npm_names(self) -> None:
        pypi_workflow = (REPO_ROOT / ".github" / "workflows" / "publish-pypi.yml").read_text(encoding="utf-8")
        npm_workflow = (REPO_ROOT / ".github" / "workflows" / "publish-npm.yml").read_text(encoding="utf-8")

        self.assertIn("https://pypi.org/p/qiongli", pypi_workflow)
        self.assertNotIn("qiongli-installer", pypi_workflow)
        self.assertIn("id-token: write", npm_workflow)
        self.assertIn("node-version: '24'", npm_workflow)
        self.assertNotIn("package-manager-cache", npm_workflow)
        self.assertIn("npm publish --tag next", npm_workflow)
        self.assertIn("npm publish --tag latest", npm_workflow)
        self.assertIn("npm dist-tag rm qiongli latest", npm_workflow)
        self.assertIn("[npm-publish] warning: unable to remove beta latest dist-tag", npm_workflow)
        self.assertNotIn("npm publish --tag beta", npm_workflow)
        self.assertIn("scripts/npm_preflight.sh", npm_workflow)
        for workflow in (pypi_workflow, npm_workflow):
            self.assertIn("if: ${{ !startsWith(github.ref_name, 'v2.') }}", workflow)
            self.assertIn("scripts/release_version.py", workflow)
            self.assertIn('if [[ "$release_line" == "native-2x" ]]; then', workflow)
            self.assertIn("RLS-201/PKG gate", workflow)
        self.assertIn('--print-field channel', npm_workflow)
        self.assertIn('if [[ "$channel" != "stable" ]]; then', npm_workflow)
        self.assertNotIn('if [[ "${RELEASE_TAG}" == *beta* ]]; then', npm_workflow)

    def test_docs_use_npm_next_dist_tag_for_prereleases(self) -> None:
        docs = "\n".join(
            [
                (REPO_ROOT / "README.md").read_text(encoding="utf-8"),
                (REPO_ROOT / "README_CN.md").read_text(encoding="utf-8"),
                (REPO_ROOT / "docs" / "guide" / "install.md").read_text(encoding="utf-8"),
                (NPM_PACKAGE_ROOT / "README.md").read_text(encoding="utf-8"),
            ]
        )

        self.assertIn("qiongli@next", docs)
        self.assertNotIn("qiongli@beta", docs)

    def test_npm_preflight_packs_from_package_directory_with_temp_cache(self) -> None:
        preflight = (LAYOUT.scripts / "npm_preflight.sh").read_text(encoding="utf-8")

        self.assertIn('NPM_CACHE="${NPM_CONFIG_CACHE:-${TMPDIR:-/tmp}/qiongli-npm-cache}"', preflight)
        self.assertIn('NPM_CONFIG_CACHE="$NPM_CACHE" npm --prefix "$PKG_DIR" test', preflight)
        self.assertIn('cd "$PKG_DIR"\n  NPM_CONFIG_CACHE="$NPM_CACHE" npm pack --dry-run', preflight)
        self.assertNotIn('npm --prefix "$PKG_DIR" pack --dry-run', preflight)

    def test_sync_npm_payload_bootstraps_repo_imports_before_package_install(self) -> None:
        sync_script = (LAYOUT.scripts / "sync_npm_package_payload.py").read_text(encoding="utf-8")

        self.assertIn("import sys", sync_script)
        self.assertIn("REPO_ROOT = Path(__file__).resolve().parents[2]", sync_script)
        self.assertIn('PYTHON_SOURCE_ROOT = REPO_ROOT / "packages" / "python-qiongli" / "src"', sync_script)
        self.assertIn("sys.path.insert(0, str(import_root))", sync_script)
        self.assertLess(sync_script.index("sys.path.insert(0, str(import_root))"), sync_script.index("from qiongli.subject_materializer"))


if __name__ == "__main__":
    unittest.main()
