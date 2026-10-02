from __future__ import annotations

import json
import unittest
from pathlib import Path

from tooling.scripts.validate_capability_contract import (
    validate_capability_contract,
    validate_instance,
)

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
                "qiongli_literature_read_fulltext",
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

    def test_every_lite_tool_has_one_safe_call_fixture(self) -> None:
        tools = json.loads((CONTRACT_ROOT / "lite-tools.json").read_text(encoding="utf-8"))
        smoke = json.loads(
            (CONTRACT_ROOT / "fixtures" / "lite-tool-smoke-calls.json").read_text(
                encoding="utf-8"
            )
        )
        tool_names = [tool["name"] for tool in tools["tools"]]
        call_names = [call["name"] for call in smoke["calls"]]

        self.assertEqual(smoke["schema_version"], "1.0")
        self.assertEqual(len(call_names), len(set(call_names)))
        self.assertEqual(set(call_names), set(tool_names))
        self.assertEqual(
            {call["expected_response_class"] for call in smoke["calls"]},
            {"success", "input_error", "bounded_local_result"},
        )
        for call in smoke["calls"]:
            self.assertEqual(
                set(call["side_effects"]),
                {"config", "network", "loopback_listener"},
            )
            self.assertIsInstance(call["forbidden_output"], list)
            for equality in call.get("required_output_equalities", []):
                self.assertEqual(set(equality), {"left", "right"})
                self.assertTrue(equality["left"].startswith("/"))
                self.assertTrue(equality["right"].startswith("/"))

        search_plan = next(
            call for call in smoke["calls"] if call["name"] == "qiongli_search_plan"
        )
        self.assertEqual(search_plan["arguments"]["from_year"], 2020)
        self.assertEqual(search_plan["arguments"]["toYear"], "2026")
        self.assertEqual(len(search_plan["required_output_equalities"]), 2)

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

    def test_expected_search_response_uses_gate_zero_semantic_projection(self) -> None:
        payload = json.loads(
            (CONTRACT_ROOT / "fixtures" / "expected-search-response.json").read_text(
                encoding="utf-8"
            )
        )["semantic_projection"]

        self.assertEqual(payload["status"], "warning")
        self.assertEqual(payload["diagnostics"]["status"], "partial")
        self.assertEqual(payload["diagnostics"]["providers"]["openalex"]["count"], 1)
        self.assertEqual(
            payload["diagnostics"]["providers"]["semantic_scholar"]["error_kind"],
            "timeout",
        )
        record = payload["results"][0]
        self.assertEqual(
            set(record),
            {"title", "doi", "year", "venue", "provider", "providers"},
        )
        self.assertNotIn(None, record.values())

    def test_lite_search_contract_rejects_unknown_arguments_and_bounds_limits(self) -> None:
        contract = json.loads((CONTRACT_ROOT / "lite-tools.json").read_text(encoding="utf-8"))
        search = next(
            tool for tool in contract["tools"] if tool["name"] == "qiongli_literature_search"
        )["inputSchema"]

        self.assertIs(search["additionalProperties"], False)
        self.assertEqual(search["properties"]["per_provider_limit"]["minimum"], 1)
        self.assertEqual(search["properties"]["per_provider_limit"]["maximum"], 200)
        self.assertEqual(search["properties"]["total_limit"]["maximum"], 1000)

    def test_native_fulltext_contract_bounds_read_inputs_and_success_evidence(self) -> None:
        name = "qiongli_literature_read_fulltext"
        schema = json.loads((CONTRACT_ROOT / f"{name}.input.schema.json").read_text())
        # Host tool projection must retain URL instead of exposing only an
        # offset branch. Cross-field continuation checks stay in the runtime.
        self.assertNotIn("oneOf", schema)
        self.assertEqual(schema["required"], ["url"])
        valid = {"url": "https://example.org/paper.xml", "expected_sha256": "a" * 64}
        self.assertEqual(validate_instance(valid, schema), [])
        self.assertEqual(validate_instance({**valid, "offset": 2}, schema), [])
        for invalid in (
            {}, {"url": "file:///tmp/paper.pdf"}, {**valid, "offset": -1},
            {**valid, "limit": 0}, {**valid, "limit": 51}, {**valid, "refresh": "true"},
            {**valid, "expected_sha256": "A" * 64}, {**valid, "cwd": "/tmp"},
        ):
            with self.subTest(arguments=invalid):
                self.assertTrue(validate_instance(invalid, schema))
        output = json.loads((CONTRACT_ROOT / f"{name}.output.schema.json").read_text())
        result = {
            "status": "readable_text", "source_url": valid["url"],
            "resolved_url": valid["url"], "source_sha256": "a" * 64,
            "format": "jats_xml", "retrieved_at_unix_seconds": 1, "cached": False,
            "total_segments": 1, "segments": [{"index": 0, "anchor": "section:1", "text": "Source text"}],
            "identity_status": "unverified", "warnings": [],
        }
        self.assertEqual(validate_instance(result, output), [])
        for invalid in (
            {**result, "source_sha256": ""}, {**result, "status": "downloaded"},
            {**result, "segments": [{"index": 0, "text": "Unanchored"}]},
            {**result, "document_doi": None},
        ):
            with self.subTest(result=invalid):
                self.assertTrue(validate_instance(invalid, output))

    def test_native_extension_does_not_relax_frozen_tool_inventory(self) -> None:
        contract = json.loads((CONTRACT_ROOT / "lite-tools.json").read_text())
        tools = {item["name"]: item for item in contract["tools"]}
        self.assertEqual(validate_capability_contract(REPO_ROOT, lite_tool_definitions=tools), [])
        missing = dict(tools)
        del missing["qiongli_literature_read_fulltext"]
        self.assertTrue(validate_capability_contract(REPO_ROOT, lite_tool_definitions=missing))
        extra = {**tools, "qiongli_unregistered_native_tool": {
            "name": "qiongli_unregistered_native_tool", "description": "Unexpected",
            "inputSchema": {"type": "object"},
        }}
        self.assertTrue(validate_capability_contract(REPO_ROOT, lite_tool_definitions=extra))
        missing_legacy = dict(tools)
        del missing_legacy["qiongli_literature_search"]
        self.assertTrue(validate_capability_contract(REPO_ROOT, lite_tool_definitions=missing_legacy))
        drift = dict(tools)
        name = "qiongli_literature_read_fulltext"
        drift[name] = {**tools[name], "inputSchema": {"type": "object"}}
        self.assertTrue(validate_capability_contract(REPO_ROOT, lite_tool_definitions=drift))
