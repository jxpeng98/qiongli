from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


class MCPCLITests(unittest.TestCase):
    def test_mcp_cli_doctor_json_reports_shared_provider_config(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            env = self._env(root)
            subprocess.run(
                [
                    sys.executable,
                    "-m",
                    "bridges.mcp_cli",
                    "configure",
                    "--provider",
                    "openalex",
                    "--field",
                    "email",
                    "--value",
                    "user@example.com",
                ],
                capture_output=True,
                text=True,
                check=True,
                env=env,
            )

            result = subprocess.run(
                [sys.executable, "-m", "bridges.mcp_cli", "doctor", "--json", "--cwd", str(root)],
                capture_output=True,
                text=True,
                check=False,
                env=env,
            )

        self.assertEqual(result.returncode, 0, result.stderr)
        payload = json.loads(result.stdout)
        rendered = json.dumps(payload, sort_keys=True)
        self.assertEqual(payload["providers"]["openalex"], "configured")
        self.assertEqual(payload["capability_mode"], "provider_connected")
        self.assertNotIn("user@example.com", rendered)

    def test_mcp_cli_config_example_for_codex_json(self) -> None:
        result = subprocess.run(
            [sys.executable, "-m", "bridges.mcp_cli", "config", "example", "--target", "codex", "--json"],
            capture_output=True,
            text=True,
            check=False,
        )

        self.assertEqual(result.returncode, 0, result.stderr)
        payload = json.loads(result.stdout)
        self.assertEqual(payload["target"], "codex")
        self.assertEqual(payload["server"]["command"], "qiongli")
        self.assertEqual(payload["server"]["args"], ["mcp", "serve", "--transport", "stdio"])

    def test_qiongli_cli_delegates_mcp_subcommand(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            result = subprocess.run(
                [sys.executable, "-m", "qiongli.cli", "mcp", "doctor", "--json", "--cwd", str(root)],
                capture_output=True,
                text=True,
                check=False,
                env=self._env(root),
            )

        self.assertEqual(result.returncode, 0, result.stderr)
        payload = json.loads(result.stdout)
        self.assertEqual(payload["server"]["name"], "qiongli-mcp")
        self.assertEqual(payload["providers"]["semantic_scholar"], "missing")

    def _env(self, root: Path) -> dict[str, str]:
        env = dict(os.environ)
        env["QIONGLI_CONFIG_HOME"] = str(root / "config")
        return env


if __name__ == "__main__":
    unittest.main()
