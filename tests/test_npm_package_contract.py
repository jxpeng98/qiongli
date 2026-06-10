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
            "Qiongli academic research workflow installer and optional Python bridge runtime.",
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

    def test_npm_payload_and_python_runtime_are_bundled(self) -> None:
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
            runtime_root = package_root / "python-runtime"

            self._assert_npm_payload_and_runtime(package_root, workflow_root, runtime_root)

    def _assert_npm_payload_and_runtime(self, package_root: Path, workflow_root: Path, runtime_root: Path) -> None:
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

        self.assertTrue((runtime_root / "bridges" / "__init__.py").is_file())
        self.assertTrue((runtime_root / "qiongli" / "bridges" / "orchestrator.py").is_file())
        self.assertTrue((runtime_root / "qiongli" / "bridges" / "mcp_cli.py").is_file())
        self.assertTrue((runtime_root / "qiongli" / "bridges" / "mcp_server_stdio.py").is_file())
        self.assertTrue((runtime_root / "qiongli" / "bridges" / "providers" / "literature_search.py").is_file())
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

    def test_npm_runtime_setup_wizard_resolves_npm_payload_root(self) -> None:
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
            env["PYTHONPATH"] = str(package_root / "python-runtime")
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
