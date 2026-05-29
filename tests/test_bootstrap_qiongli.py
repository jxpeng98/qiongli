from __future__ import annotations

import os
import subprocess
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
BOOTSTRAP_SCRIPT = REPO_ROOT / "scripts" / "bootstrap_qiongli.sh"
POWERSHELL_BOOTSTRAP = REPO_ROOT / "scripts" / "bootstrap_qiongli.ps1"
SYSTEM_BASH = Path("/bin/bash")


class BootstrapQiongliTests(unittest.TestCase):
    def test_partial_profile_dry_run_skips_cli_and_doctor(self) -> None:
        if not SYSTEM_BASH.exists():
            self.skipTest("/bin/bash is not available")

        with tempfile.TemporaryDirectory() as tmp_dir:
            result = subprocess.run(
                [
                    str(SYSTEM_BASH),
                    str(BOOTSTRAP_SCRIPT),
                    "--profile",
                    "partial",
                    "--project-dir",
                    tmp_dir,
                    "--dry-run",
                ],
                cwd=str(REPO_ROOT),
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
                check=False,
            )

        self.assertEqual(result.returncode, 0, msg=result.stdout + "\n" + result.stderr)
        self.assertIn("profile: partial", result.stdout)
        self.assertIn("cli:     skip", result.stdout)
        self.assertNotIn("--install-cli", result.stdout)
        self.assertNotIn("--doctor", result.stdout)

    def test_full_profile_dry_run_enables_cli_and_doctor(self) -> None:
        if not SYSTEM_BASH.exists():
            self.skipTest("/bin/bash is not available")

        with tempfile.TemporaryDirectory() as tmp_dir:
            result = subprocess.run(
                [
                    str(SYSTEM_BASH),
                    str(BOOTSTRAP_SCRIPT),
                    "--profile",
                    "full",
                    "--project-dir",
                    tmp_dir,
                    "--dry-run",
                ],
                cwd=str(REPO_ROOT),
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
                check=False,
            )

        self.assertEqual(result.returncode, 0, msg=result.stdout + "\n" + result.stderr)
        self.assertIn("profile: full", result.stdout)
        self.assertIn("cli:     install ->", result.stdout)
        self.assertIn("--install-cli", result.stdout)
        self.assertIn("--doctor", result.stdout)
        self.assertNotIn("install via mise", result.stdout)

    def test_dry_run_passes_parts_to_installer(self) -> None:
        if not SYSTEM_BASH.exists():
            self.skipTest("/bin/bash is not available")

        with tempfile.TemporaryDirectory() as tmp_dir:
            result = subprocess.run(
                [
                    str(SYSTEM_BASH),
                    str(BOOTSTRAP_SCRIPT),
                    "--profile",
                    "partial",
                    "--project-dir",
                    tmp_dir,
                    "--parts",
                    "project,cli",
                    "--dry-run",
                ],
                cwd=str(REPO_ROOT),
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
                check=False,
            )

        self.assertEqual(result.returncode, 0, msg=result.stdout + "\n" + result.stderr)
        self.assertIn("--parts project\\,cli", result.stdout)

    def test_partial_profile_from_source_repo_exits_cleanly(self) -> None:
        if not SYSTEM_BASH.exists():
            self.skipTest("/bin/bash is not available")

        with tempfile.TemporaryDirectory() as temp_root:
            env = dict(os.environ)
            env["HOME"] = str(Path(temp_root) / "home")
            env["CODEX_HOME"] = str(Path(temp_root) / "codex-home")
            env["CLAUDE_CODE_HOME"] = str(Path(temp_root) / "claude-home")
            env["GEMINI_HOME"] = str(Path(temp_root) / "gemini-home")
            env["ANTIGRAVITY_HOME"] = str(Path(temp_root) / "antigravity-home")
            project_dir = Path(temp_root) / "project"
            cli_dir = Path(temp_root) / "bin"

            result = subprocess.run(
                [
                    str(SYSTEM_BASH),
                    str(BOOTSTRAP_SCRIPT),
                    "--profile",
                    "partial",
                    "--source-repo",
                    str(REPO_ROOT),
                    "--project-dir",
                    str(project_dir),
                    "--cli-dir",
                    str(cli_dir),
                    "--overwrite",
                ],
                cwd=str(REPO_ROOT),
                env=env,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
                check=False,
            )

            self.assertEqual(result.returncode, 0, msg=result.stdout + "\n" + result.stderr)
            self.assertIn("source:   local checkout", result.stdout)
            self.assertFalse((project_dir / ".env").exists())
            self.assertFalse((cli_dir / "qiongli").exists())

    def test_missing_profile_in_noninteractive_mode_fails_fast(self) -> None:
        if not SYSTEM_BASH.exists():
            self.skipTest("/bin/bash is not available")

        with tempfile.TemporaryDirectory() as tmp_dir:
            env = dict(os.environ)
            env["RESEARCH_SKILLS_NONINTERACTIVE"] = "1"

            result = subprocess.run(
                [
                    str(SYSTEM_BASH),
                    str(BOOTSTRAP_SCRIPT),
                    "--project-dir",
                    tmp_dir,
                ],
                cwd=str(REPO_ROOT),
                env=env,
                stdin=subprocess.DEVNULL,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
                check=False,
            )

        self.assertEqual(result.returncode, 2, msg=result.stdout + "\n" + result.stderr)
        self.assertIn("Missing --profile and no interactive terminal is available", result.stderr)

    def test_shell_bootstrap_supports_explicit_noninteractive_mode(self) -> None:
        content = BOOTSTRAP_SCRIPT.read_text(encoding="utf-8")

        self.assertIn("QIONGLI_NONINTERACTIVE", content)
        self.assertIn("RESEARCH_SKILLS_NONINTERACTIVE", content)
        self.assertIn('[[ "${QIONGLI_NONINTERACTIVE:-${RESEARCH_SKILLS_NONINTERACTIVE:-}}" == "1" ]]', content)

    def test_shell_bootstrap_does_not_install_python_runtime(self) -> None:
        content = BOOTSTRAP_SCRIPT.read_text(encoding="utf-8")

        self.assertNotIn("install_mise()", content)
        self.assertNotIn('"$MISE_BIN" install python@3.12', content)
        self.assertNotIn('"$MISE_BIN" use -g python@3.12', content)
        self.assertNotIn("PYTHON_RUNTIME_MODE", content)
        self.assertNotIn("curl https://mise.run", content)
        self.assertIn("Python 3.12+ is required for full profile", content)
        self.assertIn("python.org/downloads", content)
        self.assertIn("brew install python", content)
        self.assertIn("winget install -e --id Python.Python.3.12", content)

    def test_powershell_bootstrap_is_manifest_driven(self) -> None:
        content = POWERSHELL_BOOTSTRAP.read_text(encoding="utf-8")

        self.assertIn("install\\install_manifest.tsv", content)
        self.assertIn("Expand-Archive", content)
        self.assertIn("Install-FromRepo", content)
        self.assertIn("[switch]$Beta", content)
        self.assertIn('[ValidateSet("tag", "branch", "local")]', content)
        self.assertIn("Invoke-NativeChecked", content)
        self.assertIn("Ensure-PathEntry", content)
        self.assertIn("Refresh-SessionPath", content)
        self.assertIn("Get-AddPathCommand", content)
        self.assertIn('SetEnvironmentVariable("Path"', content)
        self.assertIn("Out-Host", content)
        self.assertIn("$PSVersionTable.PSVersion.Major -lt 7", content)
        self.assertIn("Microsoft.PowerShell", content)
        self.assertIn('[string]$SourceRepo = ""', content)
        self.assertIn("$sourceRepoRoot", content)
        self.assertIn("/releases?per_page=20", content)
        self.assertNotIn("Ensure-Mise", content)
        self.assertNotIn("Find-Mise", content)
        self.assertNotIn("Ensure-NativePython312", content)
        self.assertNotIn("winget install jdx.mise", content)
        self.assertNotIn("mise install python@3.12", content)
        self.assertIn("Python 3.12+ is required for full profile", content)
        self.assertIn("python.org/downloads/windows", content)
        self.assertIn("winget install -e --id Python.Python.3.12", content)
        self.assertIn("Find-UsablePython", content)
        self.assertIn("Copy/paste this command to add it now:", content)
        self.assertIn("pwsh -NoProfile -Command", content)
        self.assertNotIn('Install-FromRepo "C:\\dry-run\\qiongli"', content)
        self.assertIn("[dry-run] Install workflow assets into client directories", content)
        self.assertNotIn('bootstrapUrl = "https://raw.githubusercontent.com', content)
        self.assertNotIn('$content = @"', content)
        self.assertIn('$env:PYTHONPATH = $RepoRoot', content)
        self.assertIn("Remove-LegacyResidues", content)
        self.assertIn("Legacy Install Cleanup", content)
        self.assertIn("research-paper-workflow", content)

    def test_shell_bootstrap_documents_beta_channel(self) -> None:
        content = BOOTSTRAP_SCRIPT.read_text(encoding="utf-8")

        self.assertIn("--beta", content)
        self.assertIn("--source-repo", content)
        self.assertIn("source:   local checkout", content)
        self.assertIn("latest beta/prerelease tag", content)
        self.assertIn("/releases?per_page=20", content)
        self.assertIn("persist_shell_path_entries", content)
        self.assertIn(".zshrc", content)
        self.assertIn(".bashrc", content)
        self.assertIn(".profile", content)


if __name__ == "__main__":
    unittest.main()
