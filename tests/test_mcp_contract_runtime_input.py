from __future__ import annotations

import unittest

from bridges.mcp_tool_handlers import MCP_TOOL_DEFINITIONS, call_qiongli_tool


class MCPContractRuntimeInputTests(unittest.TestCase):
    def test_unknown_tool_error_does_not_echo_tool_name(self) -> None:
        canary = "QIONGLI_UNKNOWN_TOOL_CANARY_DO_NOT_ECHO"

        result = call_qiongli_tool(canary, {})

        self.assertTrue(result["isError"])
        self.assertEqual(result["structuredContent"]["message"], "tool is unavailable")
        self.assertNotIn(canary, str(result))

    def test_full_runtime_rejects_schema_invalid_arguments_before_dispatch(self) -> None:
        for definition in MCP_TOOL_DEFINITIONS:
            name = definition["name"]
            arguments = (
                {"query": []}
                if name == "qiongli_literature_search"
                else {"__contract_probe__": True}
            )

            with self.subTest(tool=name):
                result = call_qiongli_tool(name, arguments)

                self.assertTrue(result["isError"], result)
                self.assertEqual(
                    result["structuredContent"].get("error_kind"),
                    "invalid_arguments",
                    result,
                )
                self.assertEqual(result["structuredContent"].get("tool"), name)

    def test_full_runtime_rejects_non_object_argument_carriers(self) -> None:
        for arguments in ([], ["unexpected"], "unexpected", True):
            with self.subTest(arguments=arguments):
                result = call_qiongli_tool(  # type: ignore[arg-type]
                    "qiongli_list_provider_env",
                    arguments,
                )

                self.assertTrue(result["isError"], result)
                self.assertEqual(
                    result["structuredContent"].get("error_kind"),
                    "invalid_arguments",
                    result,
                )


if __name__ == "__main__":
    unittest.main()
