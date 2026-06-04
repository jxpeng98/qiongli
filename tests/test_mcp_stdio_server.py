from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


class MCPStdioServerTests(unittest.TestCase):
    def test_stdio_server_handles_initialize_and_tools_list(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            messages = [
                {
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "initialize",
                    "params": {
                        "protocolVersion": "2024-11-05",
                        "capabilities": {},
                        "clientInfo": {"name": "unittest", "version": "0"},
                    },
                },
                {"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}},
            ]
            process = self._run_server(messages, root)

        self.assertEqual(process.returncode, 0, process.stderr)
        self.assertEqual(process.stderr, "")
        responses = [json.loads(line) for line in process.stdout.splitlines() if line.strip()]
        self.assertEqual(responses[0]["id"], 1)
        self.assertEqual(responses[0]["result"]["serverInfo"]["name"], "qiongli-mcp")
        self.assertEqual(responses[1]["id"], 2)
        tool_names = {tool["name"] for tool in responses[1]["result"]["tools"]}
        self.assertIn("qiongli_config_status", tool_names)

    def test_stdio_server_calls_tool_without_leaking_saved_secret(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            messages = [
                {
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "tools/call",
                    "params": {
                        "name": "qiongli_save_provider_config",
                        "arguments": {
                            "provider": "semantic-scholar",
                            "field": "api-key",
                            "value": "secret-demo-key",
                        },
                    },
                },
                {
                    "jsonrpc": "2.0",
                    "id": 2,
                    "method": "tools/call",
                    "params": {
                        "name": "qiongli_config_status",
                        "arguments": {"cwd": str(root)},
                    },
                },
            ]
            process = self._run_server(messages, root)

        self.assertEqual(process.returncode, 0, process.stderr)
        self.assertNotIn("secret-demo-key", process.stdout)
        responses = [json.loads(line) for line in process.stdout.splitlines() if line.strip()]
        self.assertEqual(
            responses[1]["result"]["structuredContent"]["providers"]["semantic_scholar"],
            "configured",
        )

    def _run_server(
        self,
        messages: list[dict[str, object]],
        root: Path,
    ) -> subprocess.CompletedProcess[str]:
        stdin = "\n".join(json.dumps(message) for message in messages) + "\n"
        env = dict(os.environ)
        env["QIONGLI_CONFIG_HOME"] = str(root / "config")
        return subprocess.run(
            [sys.executable, "-m", "bridges.mcp_server_stdio"],
            input=stdin,
            capture_output=True,
            text=True,
            timeout=10,
            check=False,
            env=env,
        )


if __name__ == "__main__":
    unittest.main()
