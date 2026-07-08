from __future__ import annotations

import json
import os
import subprocess
import tempfile
import unittest
from pathlib import Path

from tooling.scripts.build_lite_mcp import build_current_platform


REPO_ROOT = Path(__file__).resolve().parents[1]


class LiteMCPContractTests(unittest.TestCase):
    def test_binary_initializes_and_lists_tools_without_node_or_python(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            binary = build_current_platform(REPO_ROOT, Path(tmp_dir))
            process = subprocess.run(
                [str(binary), "--transport", "stdio"],
                input=(
                    '{"jsonrpc":"2.0","id":1,"method":"initialize",'
                    '"params":{"protocolVersion":"2025-11-25"}}\n'
                    '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}\n'
                ),
                text=True,
                capture_output=True,
                check=False,
                timeout=10,
                env=self._runtime_env(Path(tmp_dir)),
            )

        self.assertEqual(process.returncode, 0, msg=process.stderr)
        self.assertEqual(process.stderr, "")
        lines = [json.loads(line) for line in process.stdout.splitlines() if line.strip()]
        self.assertEqual(lines[0]["result"]["serverInfo"]["name"], "qiongli-literature-provider")
        names = {tool["name"] for tool in lines[1]["result"]["tools"]}
        self.assertIn("qiongli_literature_status", names)
        self.assertIn("qiongli_literature_search", names)
        self.assertIn("qiongli_zotero_export_import_files", names)

    def test_binary_config_status_does_not_leak_saved_secret(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            binary = build_current_platform(REPO_ROOT, root / "build")
            process = subprocess.run(
                [str(binary), "--transport", "stdio"],
                input=(
                    '{"jsonrpc":"2.0","id":1,"method":"tools/call",'
                    '"params":{"name":"qiongli_save_provider_config",'
                    '"arguments":{"provider":"semantic-scholar","field":"api-key",'
                    '"value":"secret-demo-key"}}}\n'
                    '{"jsonrpc":"2.0","id":2,"method":"tools/call",'
                    '"params":{"name":"qiongli_config_status","arguments":{}}}\n'
                ),
                text=True,
                capture_output=True,
                check=False,
                timeout=10,
                env=self._runtime_env(root / "config"),
            )

        self.assertEqual(process.returncode, 0, msg=process.stderr)
        self.assertNotIn("secret-demo-key", process.stdout)
        lines = [json.loads(line) for line in process.stdout.splitlines() if line.strip()]
        status = lines[1]["result"]["structuredContent"]
        self.assertEqual(status["providers"]["semantic_scholar"], "configured")

    def _runtime_env(self, config_home: Path) -> dict[str, str]:
        env = os.environ.copy()
        env["PATH"] = ""
        env["QIONGLI_CONFIG_HOME"] = str(config_home)
        return env


if __name__ == "__main__":
    unittest.main()
