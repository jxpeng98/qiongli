from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from typing import Any
from unittest import mock

from tooling.scripts.build_lite_mcp import build_current_platform


REPO_ROOT = Path(__file__).resolve().parents[1]
CONTRACT_ROOT = REPO_ROOT / "content" / "mcp-contracts"
SMOKE_CALLS_PATH = CONTRACT_ROOT / "fixtures" / "lite-tool-smoke-calls.json"
PYTHON_QIONGLI = REPO_ROOT / "packages" / "python-qiongli" / "src" / "qiongli"
if str(PYTHON_QIONGLI) not in sys.path:
    sys.path.insert(0, str(PYTHON_QIONGLI))

from bridges.provider_config import resolve_provider_config


class LiteMCPBehaviorContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls._temp_dir = tempfile.TemporaryDirectory()
        cls._root = Path(cls._temp_dir.name)
        cls._binary = build_current_platform(REPO_ROOT, cls._root / "build")
        cls._fixture = json.loads(SMOKE_CALLS_PATH.read_text(encoding="utf-8"))

    @classmethod
    def tearDownClass(cls) -> None:
        cls._temp_dir.cleanup()

    def test_every_declared_tool_has_exactly_one_safe_call(self) -> None:
        contract = json.loads(
            (CONTRACT_ROOT / "lite-tools.json").read_text(encoding="utf-8")
        )
        declared = [tool["name"] for tool in contract["tools"]]
        covered = [call["name"] for call in self._fixture["calls"]]

        self.assertEqual(len(covered), len(set(covered)))
        self.assertEqual(set(covered), set(declared))
        self.assertEqual(len(covered), 12)

    def test_safe_calls_are_dispatchable_classified_and_secret_free(self) -> None:
        responses, process = self._call_fixture()

        self.assertEqual(process.returncode, 0, msg=process.stderr)
        self.assertEqual(len(responses), len(self._fixture["calls"]))
        serialized = json.dumps(responses, sort_keys=True)
        self.assertNotIn(self._fixture["canary_value"], serialized)

        for call, response in zip(self._fixture["calls"], responses, strict=True):
            with self.subTest(tool=call["name"]):
                error = response.get("error")
                self.assertNotEqual((error or {}).get("code"), -32601)

                if call["expected_response_class"] == "input_error":
                    is_handler_error = (error or {}).get("code") == -32602 or bool(
                        response.get("result", {}).get("isError")
                    )
                    self.assertTrue(is_handler_error, msg=response)
                else:
                    self.assertIn("result", response, msg=response)
                    self.assertFalse(response["result"].get("isError", False), msg=response)
                    self.assertIn("content", response["result"])
                    self.assertIn("structuredContent", response["result"])

                for forbidden in call["forbidden_output"]:
                    self.assertNotIn(forbidden, json.dumps(response, sort_keys=True))

    def test_full_runtime_resolves_all_mcpb_provider_aliases(self) -> None:
        aliases = {
            "QIONGLI_MCPB_OPENALEX_API_KEY": "oa-canary",
            "QIONGLI_MCPB_OPENALEX_EMAIL": "oa@example.invalid",
            "QIONGLI_MCPB_SEMANTIC_SCHOLAR_API_KEY": "s2-canary",
            "QIONGLI_MCPB_CROSSREF_EMAIL": "crossref@example.invalid",
            "QIONGLI_MCPB_PUBMED_API_KEY": "pubmed-canary",
        }
        with mock.patch.dict(
            os.environ,
            {"QIONGLI_CONFIG_HOME": str(self._root / "full-config")},
            clear=False,
        ):
            resolved = resolve_provider_config(cwd=self._root, env=aliases)

        providers = resolved["providers"]
        self.assertEqual(providers["openalex"]["api_key"], "oa-canary")
        self.assertEqual(providers["openalex"]["email"], "oa@example.invalid")
        self.assertEqual(providers["semantic_scholar"]["api_key"], "s2-canary")
        self.assertEqual(providers["crossref"]["email"], "crossref@example.invalid")
        self.assertEqual(providers["pubmed"]["api_key"], "pubmed-canary")

    def test_full_runtime_does_not_activate_openalex_with_email_alone(self) -> None:
        with mock.patch.dict(
            os.environ,
            {"QIONGLI_CONFIG_HOME": str(self._root / "full-email-only")},
            clear=False,
        ):
            resolved = resolve_provider_config(
                cwd=self._root,
                env={"QIONGLI_MCPB_OPENALEX_EMAIL": "oa@example.invalid"},
            )

        self.assertFalse(resolved["providers"]["openalex"]["configured"])

    def _call_fixture(self) -> tuple[list[dict[str, Any]], subprocess.CompletedProcess[str]]:
        requests = []
        for request_id, call in enumerate(self._fixture["calls"], start=1):
            requests.append(
                json.dumps(
                    {
                        "jsonrpc": "2.0",
                        "id": request_id,
                        "method": "tools/call",
                        "params": {
                            "name": call["name"],
                            "arguments": call["arguments"],
                        },
                    }
                )
            )

        env = os.environ.copy()
        env["PATH"] = ""
        env["QIONGLI_CONFIG_HOME"] = str(self._root / "config")
        env["QIONGLI_ZOTERO_LOCAL_ENABLED"] = "0"
        process = subprocess.run(
            [str(self._binary), "--transport", "stdio"],
            input="\n".join(requests) + "\n",
            text=True,
            capture_output=True,
            check=False,
            timeout=15,
            env=env,
        )
        responses = [
            json.loads(line) for line in process.stdout.splitlines() if line.strip()
        ]
        return responses, process


if __name__ == "__main__":
    unittest.main()
