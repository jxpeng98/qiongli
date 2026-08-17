from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tooling.scripts.build_lite_mcp import build_current_platform


REPO_ROOT = Path(__file__).resolve().parents[1]
PYTHON_QIONGLI = REPO_ROOT / "packages" / "python-qiongli" / "src" / "qiongli"
if str(PYTHON_QIONGLI) not in sys.path:
    sys.path.insert(0, str(PYTHON_QIONGLI))

from bridges.mcp_tool_handlers import MCP_TOOL_DEFINITIONS, call_qiongli_tool


FULL_CLI_OVERLAP_EXCEPTIONS = {
    "qiongli_zotero_status",
    "qiongli_zotero_search",
    "qiongli_zotero_upsert_references",
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

    def test_shared_config_plan_and_evidence_semantics_match(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            env = self._runtime_env(root / "config")
            calls = [
                ("qiongli_config_status", {}),
                (
                    "qiongli_search_plan",
                    {
                        "query": "platform governance",
                        "search_mode": "topic",
                        "native_search_available": False,
                    },
                ),
                (
                    "qiongli_literature_export_evidence",
                    {
                        "query": "platform governance",
                        "provider_status": {"arxiv": "configured"},
                        "search_plan": {"search_execution_mode": "provider_connected"},
                        "results": [
                            {
                                "title": "A Test Paper",
                                "doi": "10.1234/example",
                                "year": 2025,
                                "provider": "openalex",
                                "providers": ["openalex"],
                            }
                        ],
                        "diagnostics": {"status": "complete", "providers": {}},
                    },
                ),
            ]
            lite = self._run_lite_calls(root / "build", env, calls)
            with mock.patch.dict(os.environ, env, clear=True):
                full = [
                    call_qiongli_tool(name, {**arguments, "cwd": str(root)})
                    if name != "qiongli_config_status"
                    else call_qiongli_tool(name, {"cwd": str(root)})
                    for name, arguments in calls
                ]

        lite_config = lite[0]["result"]["structuredContent"]
        full_config = full[0]["structuredContent"]
        self.assertEqual(
            self._config_projection(lite_config),
            self._config_projection(full_config),
        )

        lite_plan = lite[1]["result"]["structuredContent"]
        full_plan = full[1]["structuredContent"]
        self.assertEqual(self._plan_projection(lite_plan), self._plan_projection(full_plan))

        lite_evidence = lite[2]["result"]["structuredContent"]
        full_evidence = full[2]["structuredContent"]
        self.assertEqual(
            self._evidence_projection(lite_evidence),
            self._evidence_projection(full_evidence),
        )

    def test_shared_redaction_input_errors_and_task_identity_match(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            env = self._runtime_env(root / "config")
            secrets = {
                "QIONGLI_MCPB_OPENALEX_API_KEY": "oa-parity-secret",
                "QIONGLI_MCPB_SEMANTIC_SCHOLAR_API_KEY": "s2-parity-secret",
            }
            env.update(secrets)
            calls = [
                ("qiongli_config_status", {}),
                ("qiongli_task_plan", {}),
                (
                    "qiongli_task_plan",
                    {
                        "task_id": "B1",
                        "paper_type": "systematic-review",
                        "topic": "ai feedback",
                    },
                ),
            ]
            lite = self._run_lite_calls(root / "build", env, calls)
            with mock.patch.dict(os.environ, env, clear=True):
                full = [
                    call_qiongli_tool("qiongli_config_status", {"cwd": str(root)}),
                    call_qiongli_tool("qiongli_task_plan", {"cwd": str(root)}),
                    call_qiongli_tool(
                        "qiongli_task_plan",
                        {
                            "task_id": "B1",
                            "paper_type": "systematic-review",
                            "topic": "ai feedback",
                            "cwd": str(root),
                        },
                    ),
                ]

        serialized = json.dumps({"lite": lite, "full": full}, sort_keys=True)
        for secret in secrets.values():
            self.assertNotIn(secret, serialized)
        self.assertEqual(
            self._redacted_provider_projection(
                lite[0]["result"]["structuredContent"]
            ),
            self._redacted_provider_projection(full[0]["structuredContent"]),
        )
        self.assertEqual(self._input_error_class(lite[1]), "input_error")
        self.assertEqual(self._input_error_class(full[1]), "input_error")

        lite_task = lite[2]["result"]["structuredContent"]
        full_task = full[2]["structuredContent"]
        self.assertEqual(
            self._task_identity(lite_task, lite=True),
            self._task_identity(full_task, lite=False),
        )
        self.assertTrue(lite_task["preview_only"])
        self.assertFalse(lite_task["run_agents_allowed"])
        self.assertFalse(lite_task["shell_execution_allowed"])
        self.assertFalse(lite_task["project_writes_allowed"])
        self.assertTrue(lite_task["upgrade"]["required_for_execution"])

    def _run_lite_calls(
        self,
        build_root: Path,
        env: dict[str, str],
        calls: list[tuple[str, dict[str, object]]],
    ) -> list[dict[str, object]]:
        binary = build_current_platform(REPO_ROOT, build_root)
        stdin = "\n".join(
            json.dumps(
                {
                    "jsonrpc": "2.0",
                    "id": index,
                    "method": "tools/call",
                    "params": {"name": name, "arguments": arguments},
                }
            )
            for index, (name, arguments) in enumerate(calls, start=1)
        )
        process = subprocess.run(
            [str(binary), "--transport", "stdio"],
            input=stdin + "\n",
            text=True,
            capture_output=True,
            check=False,
            timeout=10,
            env=env,
        )
        self.assertEqual(process.returncode, 0, msg=process.stderr)
        return [json.loads(line) for line in process.stdout.splitlines() if line.strip()]

    @staticmethod
    def _config_projection(payload: dict[str, object]) -> dict[str, object]:
        return {
            "capability_mode": payload["capability_mode"],
            "providers": payload["providers"],
        }

    @staticmethod
    def _plan_projection(payload: dict[str, object]) -> dict[str, object]:
        return {
            "query": payload["query"],
            "search_execution_mode": payload["search_execution_mode"],
            "provider_capability_mode": payload["provider_capability_mode"],
            "native_search_available": payload["native_search_available"],
        }

    @staticmethod
    def _evidence_projection(payload: dict[str, object]) -> dict[str, object]:
        return {
            "artifact_type": payload["artifact_type"],
            "query": payload["query"],
            "provider_status": payload["provider_status"],
            "search_plan": payload["search_plan"],
            "diagnostics": payload["diagnostics"],
            "result_count": payload["result_count"],
            "results": payload["results"],
        }

    @staticmethod
    def _redacted_provider_projection(payload: dict[str, object]) -> dict[str, object]:
        redacted = payload["redacted_config"]
        assert isinstance(redacted, dict)
        providers = redacted["providers"]
        assert isinstance(providers, dict)
        return providers

    @staticmethod
    def _input_error_class(response: dict[str, object]) -> str:
        error = response.get("error")
        if isinstance(error, dict) and error.get("code") == -32602:
            return "input_error"
        result = response.get("result", response)
        if isinstance(result, dict) and result.get("isError") is True:
            return "input_error"
        return "other"

    @staticmethod
    def _task_identity(payload: dict[str, object], *, lite: bool) -> dict[str, object]:
        if lite:
            return {
                "task_id": payload["task_id"],
                "paper_type": payload["paper_type"],
                "topic": payload["topic"],
            }
        data = payload["data"]
        assert isinstance(data, dict)
        return {
            "task_id": data["task_id"],
            "paper_type": data["paper_type"],
            "topic": str(data["topic"]).replace("-", " "),
        }

    def _runtime_env(self, config_home: Path) -> dict[str, str]:
        env = os.environ.copy()
        env["PATH"] = ""
        env["QIONGLI_CONFIG_HOME"] = str(config_home)
        contract = json.loads(
            (REPO_ROOT / "content" / "mcp-contracts" / "provider-config.schema.json").read_text(
                encoding="utf-8"
            )
        )
        for aliases in contract["x-qiongli-env-aliases"].values():
            for alias in aliases:
                env.pop(alias, None)
        return env


if __name__ == "__main__":
    unittest.main()
