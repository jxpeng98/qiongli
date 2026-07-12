from __future__ import annotations

import copy
import json
import os
import socket
import subprocess
import sys
import tempfile
import threading
import unittest
from collections.abc import Iterator, Mapping
from contextlib import contextmanager
from pathlib import Path
from typing import Any
from unittest import mock

from tooling.scripts.build_lite_mcp import build_current_platform
from tooling.scripts.validate_capability_contract import validate_instance


REPO_ROOT = Path(__file__).resolve().parents[1]
CONTRACT_ROOT = REPO_ROOT / "content" / "mcp-contracts" / "v2"
SMOKE_CALLS_PATH = (
    REPO_ROOT
    / "content"
    / "mcp-contracts"
    / "fixtures"
    / "capability-contract-v2-smoke-calls.json"
)
PYTHON_QIONGLI = REPO_ROOT / "packages" / "python-qiongli" / "src" / "qiongli"
if str(PYTHON_QIONGLI) not in sys.path:
    sys.path.insert(0, str(PYTHON_QIONGLI))

from bridges import literature_mcp_tools
from bridges.mcp_tool_handlers import call_qiongli_tool


EXPOSED_PROFILES = ("marketplace-lite", "full")
NON_ERROR_RESPONSE_CLASSES = {"success", "bounded_local_result"}
INPUT_ERROR_ONLY_PAIRS = {
    ("marketplace-lite", "qiongli_configure_provider"),
    ("marketplace-lite", "qiongli_open_config_wizard"),
    ("full", "qiongli_configure_provider"),
    ("full", "qiongli_open_config_wizard"),
    ("marketplace-lite", "qiongli_zotero_status"),
}
PROVIDER_ENV_KEYS = (
    "QIONGLI_OPENALEX_API_KEY",
    "OPENALEX_API_KEY",
    "QIONGLI_MCPB_OPENALEX_API_KEY",
    "QIONGLI_OPENALEX_EMAIL",
    "OPENALEX_EMAIL",
    "QIONGLI_MCPB_OPENALEX_EMAIL",
    "QIONGLI_SEMANTIC_SCHOLAR_API_KEY",
    "SEMANTIC_SCHOLAR_API_KEY",
    "S2_API_KEY",
    "QIONGLI_MCPB_SEMANTIC_SCHOLAR_API_KEY",
    "QIONGLI_CROSSREF_EMAIL",
    "CROSSREF_EMAIL",
    "QIONGLI_MCPB_CROSSREF_EMAIL",
    "QIONGLI_NCBI_API_KEY",
    "NCBI_API_KEY",
    "PUBMED_API_KEY",
    "QIONGLI_MCPB_PUBMED_API_KEY",
)


class CapabilityContractV2SmokeTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls._build_directory = tempfile.TemporaryDirectory()
        cls._binary = build_current_platform(
            REPO_ROOT,
            Path(cls._build_directory.name) / "build",
        )
        cls._registry = _load_json(CONTRACT_ROOT / "registry.json")
        cls._fixture = _load_json(SMOKE_CALLS_PATH)
        cls._public_records = _public_record_index(cls._registry)
        cls._schemas: dict[tuple[str, str, str], dict[str, Any]] = {}

    @classmethod
    def tearDownClass(cls) -> None:
        cls._build_directory.cleanup()

    def test_fixture_exactly_closes_the_exposed_profile_surface(self) -> None:
        expected = _exposed_public_surface(self._registry)
        calls = self._fixture["calls"]
        actual = [(call["profile"], call["name"]) for call in calls]
        call_ids = [call["id"] for call in calls]
        referenced_ids = [
            call_id
            for record in self._registry["tools"]
            for call_id in record["smoke_call_ids"]
        ]

        self.assertEqual(len(expected), 34)
        self.assertEqual(len(actual), 34)
        self.assertEqual(len(actual), len(set(actual)), "duplicate profile/name case")
        self.assertEqual(set(actual), expected, "missing or orphan profile/name case")
        self.assertEqual(len(call_ids), len(set(call_ids)), "duplicate smoke call id")
        self.assertTrue(all(isinstance(call_id, str) and call_id for call_id in call_ids))
        self.assertEqual(
            len(referenced_ids),
            len(set(referenced_ids)),
            "a smoke call id must belong to exactly one canonical record",
        )
        self.assertEqual(
            set(referenced_ids),
            set(call_ids),
            "registry smoke_call_ids and executable fixture cases must close exactly",
        )
        actual_input_error_pairs = {
            (call["profile"], call["name"])
            for call in calls
            if call["expected_response_class"] == "input_error"
        }
        self.assertEqual(actual_input_error_pairs, INPUT_ERROR_ONLY_PAIRS)
        self.assertEqual(len(calls) - len(actual_input_error_pairs), 29)

    def test_profile_bound_calls_are_safe_classified_and_schema_valid(self) -> None:
        canary = self._fixture["canary_value"]
        for call in self._fixture["calls"]:
            case_label = f"{call['profile']}:{call['name']}:{call['id']}"
            with self.subTest(case=case_label), tempfile.TemporaryDirectory() as tmp_dir:
                isolated_root = Path(tmp_dir)
                project_root, config_home, runtime_env = _prepare_isolated_runtime(
                    isolated_root,
                    canary=canary,
                )
                record = self._public_records[call["name"]]
                profile = call["profile"]
                profile_contract = record["profiles"][profile]
                input_schema = self._schema(record, profile, "input")
                output_schema = self._schema(record, profile, "output")
                arguments = copy.deepcopy(call["arguments"])
                fixture_input_failures = validate_instance(arguments, input_schema)
                expected_class = call["expected_response_class"]

                if expected_class == "input_error":
                    self.assertTrue(
                        fixture_input_failures,
                        "input_error fixture must remain invalid before execution",
                    )
                else:
                    self.assertIn(expected_class, NON_ERROR_RESPONSE_CLASSES)
                    self.assertEqual(fixture_input_failures, [])
                    if _schema_accepts_cwd(input_schema) and "cwd" not in arguments:
                        arguments["cwd"] = str(project_root)
                    self.assertEqual(validate_instance(arguments, input_schema), [])

                side_effects = call["side_effects"]
                for prohibited_effect in (
                    "network",
                    "loopback_listener",
                    "process_launch",
                    "agent_launch",
                ):
                    self.assertFalse(
                        side_effects[prohibited_effect],
                        f"executable smoke case cannot allow {prohibited_effect}",
                    )
                before_root = _tree_snapshot(isolated_root)
                before_project = _tree_snapshot(project_root)
                before_config = _tree_snapshot(config_home)
                if profile == "marketplace-lite":
                    response = self._call_lite(
                        call["name"],
                        arguments,
                        cwd=project_root,
                        env=runtime_env,
                    )
                else:
                    response = self._call_full(
                        call["name"],
                        arguments,
                        cwd=project_root,
                        env=runtime_env,
                    )

                result = _classify_response(
                    self,
                    response,
                    profile=profile,
                    expected_class=expected_class,
                )
                if expected_class in NON_ERROR_RESPONSE_CLASSES:
                    structured = result["structuredContent"]
                    self.assertEqual(
                        validate_instance(structured, output_schema),
                        [],
                        f"output violates {profile_contract['output_schema_ref']}",
                    )
                    for equality in call.get("required_output_equalities", []):
                        self.assertEqual(
                            _resolve_json_pointer(structured, equality["left"]),
                            _resolve_json_pointer(structured, equality["right"]),
                        )

                rendered_response = json.dumps(response, ensure_ascii=False, sort_keys=True)
                self.assertNotIn(canary, rendered_response)
                for forbidden in call["forbidden_output"]:
                    self.assertNotIn(forbidden, rendered_response)

                config_side_effect = bool(side_effects.get("config", False))
                project_side_effect = bool(side_effects.get("project_write", False))
                if not config_side_effect:
                    self.assertEqual(_tree_snapshot(config_home), before_config)
                if not project_side_effect:
                    self.assertEqual(_tree_snapshot(project_root), before_project)
                if not config_side_effect and not project_side_effect:
                    self.assertEqual(_tree_snapshot(isolated_root), before_root)

    def _schema(
        self,
        record: Mapping[str, Any],
        profile: str,
        kind: str,
    ) -> dict[str, Any]:
        canonical_name = record["name"]
        cache_key = (canonical_name, profile, kind)
        schema = self._schemas.get(cache_key)
        if schema is None:
            reference = record["profiles"][profile][f"{kind}_schema_ref"]
            schema = _load_json(CONTRACT_ROOT / reference)
            self._schemas[cache_key] = schema
        return schema

    def _call_lite(
        self,
        name: str,
        arguments: dict[str, Any],
        *,
        cwd: Path,
        env: dict[str, str],
    ) -> dict[str, Any]:
        request = json.dumps(
            {
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": {"name": name, "arguments": arguments},
            }
        )
        process = subprocess.run(
            [str(self._binary), "--transport", "stdio"],
            input=request + "\n",
            text=True,
            capture_output=True,
            check=False,
            timeout=10,
            cwd=cwd,
            env=env,
        )
        self.assertEqual(process.returncode, 0, msg=process.stderr)
        responses = [
            json.loads(line) for line in process.stdout.splitlines() if line.strip()
        ]
        self.assertEqual(len(responses), 1, msg=process.stdout)
        return responses[0]

    @staticmethod
    def _call_full(
        name: str,
        arguments: dict[str, Any],
        *,
        cwd: Path,
        env: dict[str, str],
    ) -> dict[str, Any]:
        with (
            mock.patch.dict(os.environ, env, clear=True),
            _working_directory(cwd),
            mock.patch.object(
                literature_mcp_tools,
                "_configured_provider_fns",
                return_value={},
            ),
            mock.patch.object(
                socket,
                "socket",
                side_effect=AssertionError("smoke fixture attempted socket access"),
            ) as socket_mock,
            mock.patch.object(
                subprocess,
                "Popen",
                side_effect=AssertionError("smoke fixture attempted process launch"),
            ) as process_mock,
            mock.patch.object(
                threading.Thread,
                "start",
                side_effect=AssertionError("smoke fixture attempted thread launch"),
            ) as thread_mock,
        ):
            result = call_qiongli_tool(name, arguments)
        socket_mock.assert_not_called()
        process_mock.assert_not_called()
        thread_mock.assert_not_called()
        return result


def _load_json(path: Path) -> dict[str, Any]:
    value = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise AssertionError(f"expected a JSON object: {path}")
    return value


def _public_record_index(registry: Mapping[str, Any]) -> dict[str, Mapping[str, Any]]:
    records: dict[str, Mapping[str, Any]] = {}
    for record in registry["tools"]:
        records[record["name"]] = record
        for alias in record["aliases"]:
            records[alias["name"]] = record
    return records


def _exposed_public_surface(registry: Mapping[str, Any]) -> set[tuple[str, str]]:
    surface: set[tuple[str, str]] = set()
    for record in registry["tools"]:
        public_names = [record["name"], *(alias["name"] for alias in record["aliases"])]
        for profile in EXPOSED_PROFILES:
            if record["profiles"][profile]["exposure"] == "tool":
                surface.update((profile, name) for name in public_names)
    return surface


def _schema_accepts_cwd(schema: Mapping[str, Any]) -> bool:
    properties = schema.get("properties", {})
    return isinstance(properties, Mapping) and isinstance(properties.get("cwd"), Mapping)


def _prepare_isolated_runtime(
    root: Path,
    *,
    canary: str,
) -> tuple[Path, Path, dict[str, str]]:
    project_root = root / "project"
    home = root / "home"
    config_home = root / "config"
    temp_home = root / "tmp"
    for path in (project_root, home, config_home, temp_home):
        path.mkdir(parents=True)

    experience_root = (
        project_root
        / ".qiongli"
        / "trace"
        / "runs"
        / "contract-smoke"
    )
    experience_root.mkdir(parents=True)
    (experience_root / "experience_record.json").write_text(
        json.dumps(
            {
                "schema_version": "1.0",
                "run_id": "contract-smoke",
                "privacy": {"redaction_status": "not_needed"},
            },
            sort_keys=True,
        )
        + "\n",
        encoding="utf-8",
    )

    env = {
        "HOME": str(home),
        "USERPROFILE": str(home),
        "QIONGLI_CONFIG_HOME": str(config_home),
        "QIONGLI_ZOTERO_LOCAL_ENABLED": "0",
        "PATH": "",
        "TMPDIR": str(temp_home),
        "TMP": str(temp_home),
        "TEMP": str(temp_home),
        "QIONGLI_CONTRACT_SMOKE_CANARY": canary,
        **{key: "" for key in PROVIDER_ENV_KEYS},
    }
    return project_root, config_home, env


def _classify_response(
    test: unittest.TestCase,
    response: Mapping[str, Any],
    *,
    profile: str,
    expected_class: str,
) -> Mapping[str, Any]:
    if profile == "marketplace-lite":
        rpc_error = response.get("error")
        if isinstance(rpc_error, Mapping):
            if expected_class == "input_error":
                test.assertEqual(rpc_error.get("code"), -32602, msg=response)
                return {}
            test.fail(f"unexpected JSON-RPC error: {response}")
        result = response.get("result")
    else:
        result = response

    test.assertIsInstance(result, Mapping, msg=response)
    assert isinstance(result, Mapping)
    is_error = result.get("isError") is True
    if expected_class == "input_error":
        test.assertTrue(is_error, msg=response)
        structured = result.get("structuredContent", {})
        test.assertIsInstance(structured, Mapping, msg=response)
        assert isinstance(structured, Mapping)
        test.assertEqual(structured.get("error_kind"), "invalid_arguments", msg=response)
    else:
        test.assertIn(expected_class, NON_ERROR_RESPONSE_CLASSES)
        test.assertFalse(is_error, msg=response)
        test.assertIn("content", result)
        test.assertIsInstance(result.get("structuredContent"), Mapping, msg=response)
    return result


def _tree_snapshot(root: Path) -> tuple[tuple[str, str, bytes | str], ...]:
    entries: list[tuple[str, str, bytes | str]] = []
    for path in sorted(root.rglob("*"), key=lambda item: item.as_posix()):
        relative = path.relative_to(root).as_posix()
        if path.is_symlink():
            entries.append(("symlink", relative, os.readlink(path)))
        elif path.is_dir():
            entries.append(("directory", relative, ""))
        elif path.is_file():
            entries.append(("file", relative, path.read_bytes()))
        else:
            entries.append(("other", relative, ""))
    return tuple(entries)


def _resolve_json_pointer(value: Any, pointer: str) -> Any:
    current = value
    for raw_token in pointer.removeprefix("/").split("/"):
        token = raw_token.replace("~1", "/").replace("~0", "~")
        current = current[int(token)] if isinstance(current, list) else current[token]
    return current


@contextmanager
def _working_directory(path: Path) -> Iterator[None]:
    previous = Path.cwd()
    os.chdir(path)
    try:
        yield
    finally:
        os.chdir(previous)


if __name__ == "__main__":
    unittest.main()
