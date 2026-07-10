from __future__ import annotations

import copy
import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from tooling.scripts.build_lite_mcp import build_current_platform
from tooling.scripts.validate_capability_contract import (
    runtime_schema_projection,
    validate_capability_contract,
    validate_instance,
)


REPO_ROOT = Path(__file__).resolve().parents[1]
CONTRACT_ROOT = REPO_ROOT / "content" / "mcp-contracts" / "v2"
PYTHON_QIONGLI = REPO_ROOT / "packages" / "python-qiongli" / "src" / "qiongli"
if str(PYTHON_QIONGLI) not in sys.path:
    sys.path.insert(0, str(PYTHON_QIONGLI))

from bridges.mcp_tool_handlers import MCP_TOOL_DEFINITIONS, call_qiongli_tool


TOOL_NAME = "qiongli_literature_export_evidence"


class CapabilityContractV2Tests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls._temporary_directory = tempfile.TemporaryDirectory()
        cls._root = Path(cls._temporary_directory.name)
        cls._binary = build_current_platform(REPO_ROOT, cls._root / "build")
        cls._registry = json.loads(
            (CONTRACT_ROOT / "registry.json").read_text(encoding="utf-8")
        )
        cls._registry_schema = json.loads(
            (CONTRACT_ROOT / "registry.schema.json").read_text(encoding="utf-8")
        )
        cls._input_schema = json.loads(
            (
                CONTRACT_ROOT
                / "schemas/qiongli_literature_export_evidence.input.schema.json"
            ).read_text(encoding="utf-8")
        )
        cls._output_schema = json.loads(
            (
                CONTRACT_ROOT
                / "schemas/qiongli_literature_export_evidence.output.schema.json"
            ).read_text(encoding="utf-8")
        )

    @classmethod
    def tearDownClass(cls) -> None:
        cls._temporary_directory.cleanup()

    def test_registry_pilot_is_structurally_and_semantically_valid(self) -> None:
        self.assertEqual(validate_capability_contract(REPO_ROOT), [])
        self.assertEqual(
            validate_instance(self._registry, self._registry_schema),
            [],
        )
        self.assertEqual(self._registry["coverage"]["mode"], "pilot")
        self.assertEqual(self._registry["coverage"]["canonical_tool_count"], 1)
        self.assertEqual(self._registry["coverage"]["target_canonical_tool_count"], 23)
        self.assertEqual(self._registry["coverage"]["target_public_name_count"], 24)

    def test_lite_and_full_declarations_match_the_canonical_input_schema(self) -> None:
        expected = runtime_schema_projection(self._input_schema)
        lite_contract = json.loads(
            (REPO_ROOT / "content/mcp-contracts/lite-tools.json").read_text(
                encoding="utf-8"
            )
        )
        lite = next(tool for tool in lite_contract["tools"] if tool["name"] == TOOL_NAME)
        full = next(tool for tool in MCP_TOOL_DEFINITIONS if tool["name"] == TOOL_NAME)

        self.assertEqual(lite["inputSchema"], expected)
        self.assertEqual(full["inputSchema"], expected)
        for alias in ("query_plan", "search_results", "search_diagnostics"):
            self.assertTrue(expected["properties"][alias]["deprecated"])

    def test_golden_alias_call_has_schema_valid_equivalent_core_output(self) -> None:
        arguments = {
            "cwd": str(self._root),
            "query": "capability governance",
            "provider_status": {"arxiv": "configured"},
            "query_plan": {"search_execution_mode": "provider_connected"},
            "search_results": [{"title": "A Contract Paper"}],
            "search_diagnostics": {"status": "complete"},
        }
        lite_response = self._call_lite(arguments)
        full_response = call_qiongli_tool(TOOL_NAME, arguments)
        lite_output = lite_response["result"]["structuredContent"]
        full_output = full_response["structuredContent"]

        self.assertEqual(validate_instance(lite_output, self._output_schema), [])
        self.assertEqual(validate_instance(full_output, self._output_schema), [])
        self.assertEqual(
            self._common_output(lite_output),
            self._common_output(full_output),
        )
        self.assertEqual(lite_output["status"], "ok")
        self.assertIn("exported_at", full_output)

    def test_invalid_arguments_share_semantic_error_class(self) -> None:
        for arguments in ({"unexpected": True}, {"results": ["not-an-object"]}):
            with self.subTest(arguments=arguments):
                lite_response = self._call_lite(arguments)
                full_response = call_qiongli_tool(TOOL_NAME, arguments)

                self.assertEqual(lite_response["error"]["code"], -32602)
                self.assertTrue(full_response["isError"])
                self.assertEqual(
                    full_response["structuredContent"]["error_kind"],
                    "invalid_arguments",
                )

    def test_registry_schema_rejects_missing_security_contract(self) -> None:
        invalid = copy.deepcopy(self._registry)
        del invalid["tools"][0]["security"]

        failures = validate_instance(invalid, self._registry_schema)

        self.assertTrue(any("security" in failure for failure in failures), failures)

    def test_output_schema_rejects_untraceable_snapshot(self) -> None:
        failures = validate_instance(
            {"artifact_type": "qiongli_literature_evidence_snapshot", "results": []},
            self._output_schema,
        )

        self.assertTrue(any("missing required property" in failure for failure in failures))

    def _call_lite(self, arguments: dict[str, object]) -> dict[str, object]:
        request = json.dumps(
            {
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": {"name": TOOL_NAME, "arguments": arguments},
            }
        )
        env = os.environ.copy()
        env["PATH"] = ""
        env["QIONGLI_CONFIG_HOME"] = str(self._root / "config")
        process = subprocess.run(
            [str(self._binary), "--transport", "stdio"],
            input=request + "\n",
            text=True,
            capture_output=True,
            check=False,
            timeout=10,
            env=env,
        )
        self.assertEqual(process.returncode, 0, msg=process.stderr)
        return json.loads(process.stdout.splitlines()[0])

    @staticmethod
    def _common_output(payload: dict[str, object]) -> dict[str, object]:
        return {
            key: payload[key]
            for key in (
                "artifact_type",
                "query",
                "provider_status",
                "search_plan",
                "diagnostics",
                "result_count",
                "results",
            )
        }


if __name__ == "__main__":
    unittest.main()
