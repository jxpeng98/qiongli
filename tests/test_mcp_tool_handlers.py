from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path
from unittest import mock

import bridges.mcp_tool_handlers as tool_handlers
from bridges.mcp_tool_handlers import MCP_TOOL_DEFINITIONS, call_qiongli_tool
from bridges.provider_config import set_provider_value


class MCPToolHandlerTests(unittest.TestCase):
    def test_tool_definitions_include_config_and_evidence_tools(self) -> None:
        names = {tool["name"] for tool in MCP_TOOL_DEFINITIONS}

        self.assertTrue(
            {
                "qiongli_config_status",
                "qiongli_save_provider_config",
                "qiongli_collect_evidence",
                "qiongli_list_provider_env",
                "qiongli_test_provider",
                "qiongli_open_config_wizard",
            }.issubset(names)
        )

    def test_config_status_redacts_saved_provider_values(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            with mock.patch.dict(
                "os.environ",
                {"QIONGLI_CONFIG_HOME": str(root / "config")},
                clear=False,
            ):
                set_provider_value("semantic-scholar", "api-key", "secret-demo-key")
                result = call_qiongli_tool("qiongli_config_status", {"cwd": str(root)})

        rendered = json.dumps(result, sort_keys=True)
        self.assertEqual(
            result["structuredContent"]["providers"]["semantic_scholar"],
            "configured",
        )
        self.assertEqual(result["structuredContent"]["capability_mode"], "provider_connected")
        self.assertNotIn("secret-demo-key", rendered)

    def test_save_provider_config_persists_to_shared_config(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            with mock.patch.dict(
                "os.environ",
                {"QIONGLI_CONFIG_HOME": str(root / "config")},
                clear=False,
            ):
                result = call_qiongli_tool(
                    "qiongli_save_provider_config",
                    {"provider": "openalex", "field": "email", "value": "user@example.com"},
                )
                status = call_qiongli_tool("qiongli_config_status", {"cwd": str(root)})

        rendered = json.dumps(result, sort_keys=True)
        self.assertEqual(result["structuredContent"]["provider"], "openalex")
        self.assertEqual(result["structuredContent"]["field"], "email")
        self.assertEqual(status["structuredContent"]["providers"]["openalex"], "configured")
        self.assertNotIn("user@example.com", rendered)

    def test_collect_evidence_tool_uses_existing_connector(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            project_root = root / "RESEARCH" / "demo-topic"
            project_root.mkdir(parents=True)
            (project_root / "analysis.md").write_text("# analysis\n", encoding="utf-8")

            result = call_qiongli_tool(
                "qiongli_collect_evidence",
                {
                    "cwd": str(root),
                    "provider": "filesystem",
                    "task_packet": {
                        "topic": "demo-topic",
                        "artifact_root": "RESEARCH/[topic]/",
                        "required_outputs": ["analysis.md"],
                    },
                },
            )

        self.assertEqual(result["structuredContent"]["evidence"]["status"], "ok")
        self.assertEqual(
            result["structuredContent"]["evidence"]["data"]["existing_output_count"],
            1,
        )

    def test_list_provider_env_returns_aliases_not_values(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            with mock.patch.dict(
                "os.environ",
                {"QIONGLI_CONFIG_HOME": str(root / "config")},
                clear=False,
            ):
                set_provider_value("semantic-scholar", "api-key", "secret-demo-key")
                result = call_qiongli_tool("qiongli_list_provider_env", {})

        rendered = json.dumps(result, sort_keys=True)
        aliases = result["structuredContent"]["providers"]["semantic_scholar"]["api_key"]
        self.assertIn("QIONGLI_SEMANTIC_SCHOLAR_API_KEY", aliases)
        self.assertIn("S2_API_KEY", aliases)
        self.assertNotIn("secret-demo-key", rendered)

    def test_open_config_wizard_returns_local_url(self) -> None:
        class StubWizard:
            url = "http://127.0.0.1:8765/?token=abc"
            host = "127.0.0.1"
            port = 8765
            token = "abc"
            config_path = "/tmp/qiongli/providers.json"

        with mock.patch.object(
            tool_handlers,
            "start_config_wizard",
            lambda **_: StubWizard(),
        ):
            result = call_qiongli_tool(
                "qiongli_open_config_wizard",
                {"host": "127.0.0.1", "port": 0},
            )

        self.assertEqual(
            result["structuredContent"]["url"],
            "http://127.0.0.1:8765/?token=abc",
        )
        self.assertEqual(result["structuredContent"]["config_path"], "/tmp/qiongli/providers.json")


if __name__ == "__main__":
    unittest.main()
