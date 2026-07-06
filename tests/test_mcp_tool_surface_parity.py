from __future__ import annotations

import json
from pathlib import Path
import unittest

from bridges.mcp_cli import LITERATURE_TOOLS
from bridges.mcp_tool_handlers import MCP_TOOL_DEFINITIONS


REPO_ROOT = Path(__file__).resolve().parents[1]

COMMON_LITERATURE_PROVIDER_TOOLS = {
    "qiongli_literature_status",
    "qiongli_search_plan",
    "qiongli_config_status",
    "qiongli_configure_provider",
    "qiongli_save_provider_config",
    "qiongli_open_config_wizard",
    "qiongli_literature_search",
    "qiongli_literature_export_evidence",
}


class MCPToolSurfaceParityTests(unittest.TestCase):
    def test_python_full_mcp_exposes_common_literature_provider_tools(self) -> None:
        names = {tool["name"] for tool in MCP_TOOL_DEFINITIONS}

        self.assertTrue(COMMON_LITERATURE_PROVIDER_TOOLS.issubset(names))

    def test_python_cli_literature_tool_inventory_matches_router_surface(self) -> None:
        self.assertEqual(
            LITERATURE_TOOLS,
            [
                "qiongli_literature_status",
                "qiongli_search_plan",
                "qiongli_literature_search",
                "qiongli_literature_export_evidence",
            ],
        )

    def test_node_mcpb_manifest_exposes_common_literature_provider_tools(self) -> None:
        manifest_path = REPO_ROOT / "packages" / "qiongli-literature-mcpb" / "manifest.json"
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        names = {tool["name"] for tool in manifest["tools"]}

        self.assertTrue(COMMON_LITERATURE_PROVIDER_TOOLS.issubset(names))

    def test_collect_evidence_is_python_full_external_adapter_not_mcpb_provider_status(self) -> None:
        python_names = {tool["name"] for tool in MCP_TOOL_DEFINITIONS}
        manifest_path = REPO_ROOT / "packages" / "qiongli-literature-mcpb" / "manifest.json"
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        node_names = {tool["name"] for tool in manifest["tools"]}

        self.assertIn("qiongli_collect_evidence", python_names)
        self.assertNotIn("qiongli_collect_evidence", node_names)


if __name__ == "__main__":
    unittest.main()
