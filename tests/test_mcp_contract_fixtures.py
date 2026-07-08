from __future__ import annotations

import json
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
CONTRACT_ROOT = REPO_ROOT / "content" / "mcp-contracts"


class MCPContractFixtureTests(unittest.TestCase):
    def test_lite_tools_contract_declares_required_tools(self) -> None:
        tools = json.loads((CONTRACT_ROOT / "lite-tools.json").read_text(encoding="utf-8"))
        names = {tool["name"] for tool in tools["tools"]}

        self.assertTrue(
            {
                "qiongli_config_status",
                "qiongli_configure_provider",
                "qiongli_save_provider_config",
                "qiongli_open_config_wizard",
                "qiongli_literature_status",
                "qiongli_search_plan",
                "qiongli_literature_search",
                "qiongli_literature_export_evidence",
                "qiongli_zotero_status",
                "qiongli_zotero_export_import_files",
                "qiongli_orchestrator_route",
                "qiongli_task_plan",
            }.issubset(names)
        )

    def test_lite_tools_contract_has_unique_described_tools(self) -> None:
        tools = json.loads((CONTRACT_ROOT / "lite-tools.json").read_text(encoding="utf-8"))
        names = [tool["name"] for tool in tools["tools"]]

        self.assertEqual(len(names), len(set(names)))
        for tool in tools["tools"]:
            self.assertTrue(tool["description"].strip())
            self.assertEqual(tool["inputSchema"]["type"], "object")

    def test_expected_normalized_results_fixture_has_stable_shape(self) -> None:
        payload = json.loads(
            (CONTRACT_ROOT / "fixtures" / "expected-normalized-results.json").read_text(
                encoding="utf-8"
            )
        )
        first = payload["results"][0]

        self.assertEqual(first["title"], "A Test Paper")
        self.assertEqual(first["doi"], "10.1234/example")
        self.assertEqual(first["year"], 2025)
        self.assertEqual(first["providers"], ["openalex"])
