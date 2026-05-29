from __future__ import annotations

import json
import os
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
PLUGIN_ROOT = REPO_ROOT / "plugins" / "qiongli"
PLUGIN_SKILL_ROOT = PLUGIN_ROOT / "skills" / "qiongli-workflow"
WORKFLOW_ROOT = REPO_ROOT / "qiongli-workflow"
WORKFLOW_VERSION = (WORKFLOW_ROOT / "VERSION").read_text(encoding="utf-8").strip().lstrip("v")


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
    def test_platform_manifests_share_workflow_version(self) -> None:
        codex = json.loads((PLUGIN_ROOT / ".codex-plugin" / "plugin.json").read_text(encoding="utf-8"))
        claude = json.loads((PLUGIN_ROOT / ".claude-plugin" / "plugin.json").read_text(encoding="utf-8"))
        gemini = json.loads((PLUGIN_ROOT / "gemini-extension.json").read_text(encoding="utf-8"))

        self.assertEqual(codex["version"], WORKFLOW_VERSION)
        self.assertEqual(claude["version"], WORKFLOW_VERSION)
        self.assertEqual(gemini["version"], WORKFLOW_VERSION)

    def test_codex_plugin_exposes_skill_directory(self) -> None:
        manifest = json.loads((PLUGIN_ROOT / ".codex-plugin" / "plugin.json").read_text(encoding="utf-8"))

        self.assertEqual(manifest["skills"], "./skills/")
        self.assertTrue((PLUGIN_ROOT / "skills").is_dir())

    def test_plugin_package_contains_real_portable_skill_copy(self) -> None:
        self.assertTrue((PLUGIN_SKILL_ROOT / "SKILL.md").is_file())
        self.assertTrue((PLUGIN_SKILL_ROOT / "skills" / "registry.yaml").is_file())
        self.assertFalse(PLUGIN_SKILL_ROOT.is_symlink(), "plugin package must be a real copy, not a symlink")
        self.assertEqual(
            (WORKFLOW_ROOT / "VERSION").read_text(encoding="utf-8"),
            (PLUGIN_SKILL_ROOT / "VERSION").read_text(encoding="utf-8"),
        )
        self.assertEqual(
            (WORKFLOW_ROOT / "skills" / "registry.yaml").read_text(encoding="utf-8"),
            (PLUGIN_SKILL_ROOT / "skills" / "registry.yaml").read_text(encoding="utf-8"),
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

    def test_marketplace_validator_builds_platform_artifacts_and_checks_invocation(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            result = subprocess.run(
                [
                    "python3",
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
        self.assertIn("[OK] codex marketplace artifact", result.stdout)
        self.assertIn("[OK] codex marketplace artifact (economics)", result.stdout)
        self.assertIn("[OK] codex marketplace artifact (business)", result.stdout)
        self.assertIn("[OK] codex marketplace artifact (finance)", result.stdout)
        self.assertIn("[OK] claude marketplace artifact", result.stdout)
        self.assertIn("[OK] claude marketplace artifact (economics)", result.stdout)
        self.assertIn("[OK] claude marketplace artifact (business)", result.stdout)
        self.assertIn("[OK] claude marketplace artifact (finance)", result.stdout)
        self.assertIn("[OK] claude-desktop skill artifact", result.stdout)
        self.assertIn("under desktop file budget", result.stdout)
        self.assertIn("[OK] gemini marketplace artifact", result.stdout)
        self.assertIn("qiongli invocation", result.stdout)


if __name__ == "__main__":
    unittest.main()
