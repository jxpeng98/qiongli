from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from tooling.scripts.build_lite_mcp import build_current_platform


REPO_ROOT = Path(__file__).resolve().parents[1]
PYTHON_QIONGLI = REPO_ROOT / "packages" / "python-qiongli" / "src" / "qiongli"
if str(PYTHON_QIONGLI) not in sys.path:
    sys.path.insert(0, str(PYTHON_QIONGLI))

from bridges.mcp_tool_handlers import MCP_TOOL_DEFINITIONS


FULL_CLI_OVERLAP_EXCEPTIONS = {
    "qiongli_zotero_status",
    "qiongli_zotero_export_import_files",
}


class LiteFullMCPParityTests(unittest.TestCase):
    def test_lite_binary_tools_match_shared_contract(self) -> None:
        contract = json.loads(
            (REPO_ROOT / "content" / "mcp-contracts" / "lite-tools.json").read_text(
                encoding="utf-8"
            )
        )
        expected_lite_names = [tool["name"] for tool in contract["tools"]]

        with tempfile.TemporaryDirectory() as tmp_dir:
            binary = build_current_platform(REPO_ROOT, Path(tmp_dir))
            lite = subprocess.run(
                [str(binary), "--transport", "stdio"],
                input='{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}\n',
                text=True,
                capture_output=True,
                check=False,
                timeout=10,
                env=self._runtime_env(Path(tmp_dir) / "config"),
            )

        self.assertEqual(lite.returncode, 0, msg=lite.stderr)
        lite_names = [
            tool["name"]
            for tool in json.loads(lite.stdout.splitlines()[0])["result"]["tools"]
        ]
        self.assertEqual(lite_names, expected_lite_names)

    def test_python_full_cli_exposes_overlapping_lite_contract_tools(self) -> None:
        contract = json.loads(
            (REPO_ROOT / "content" / "mcp-contracts" / "lite-tools.json").read_text(
                encoding="utf-8"
            )
        )
        expected_overlap = {
            tool["name"]
            for tool in contract["tools"]
            if tool["name"] not in FULL_CLI_OVERLAP_EXCEPTIONS
        }
        full_names = {tool["name"] for tool in MCP_TOOL_DEFINITIONS}

        self.assertTrue(expected_overlap.issubset(full_names))

    def _runtime_env(self, config_home: Path) -> dict[str, str]:
        env = os.environ.copy()
        env["PATH"] = ""
        env["QIONGLI_CONFIG_HOME"] = str(config_home)
        return env


if __name__ == "__main__":
    unittest.main()
