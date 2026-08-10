from __future__ import annotations

import copy
import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

import tooling.scripts.validate_capability_contract as capability_contract_validator
from tooling.scripts.build_lite_mcp import build_current_platform
from tooling.scripts.validate_capability_contract import (
    _load_json,
    runtime_schema_projection,
    validate_capability_contract,
    validate_instance,
)


REPO_ROOT = Path(__file__).resolve().parents[1]
CONTRACT_ROOT = REPO_ROOT / "content" / "mcp-contracts" / "v2"
PYTHON_QIONGLI = REPO_ROOT / "packages" / "python-qiongli" / "src" / "qiongli"
if str(PYTHON_QIONGLI) not in sys.path:
    sys.path.insert(0, str(PYTHON_QIONGLI))

from bridges import mcp_tool_handlers as tool_handlers
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
        cls._configuration_schemas = {
            name: {
                kind: json.loads(
                    (CONTRACT_ROOT / f"schemas/{name}.{kind}.schema.json").read_text(
                        encoding="utf-8"
                    )
                )
                for kind in ("input", "output")
            }
            for name in (
                "qiongli_config_status",
                "qiongli_save_provider_config",
                "qiongli_configure_provider",
            )
        }
        cls._literature_planning_schemas = {
            name: {
                kind: json.loads(
                    (CONTRACT_ROOT / f"schemas/{name}.{kind}.schema.json").read_text(
                        encoding="utf-8"
                    )
                )
                for kind in ("input", "output")
            }
            for name in (
                "qiongli_literature_status",
                "qiongli_search_plan",
            )
        }

    @classmethod
    def tearDownClass(cls) -> None:
        cls._temporary_directory.cleanup()

    def _validate_registry_mutation(
        self,
        registry: dict[str, object],
        **kwargs: object,
    ) -> list[str]:
        original_load_json = capability_contract_validator._load_json
        registry_path = (REPO_ROOT / "content/mcp-contracts/v2/registry.json").resolve()

        def load_json(path: Path) -> object:
            if path.resolve() == registry_path:
                return copy.deepcopy(registry)
            return original_load_json(path)

        with mock.patch.object(
            capability_contract_validator,
            "_load_json",
            side_effect=load_json,
        ):
            return capability_contract_validator.validate_capability_contract(
                REPO_ROOT,
                **kwargs,
            )

    def test_registry_complete_is_structurally_and_semantically_valid(self) -> None:
        self.assertEqual(validate_capability_contract(REPO_ROOT, require_complete=True), [])
        self.assertEqual(
            validate_instance(self._registry, self._registry_schema),
            [],
        )
        self.assertEqual(self._registry["status"], "preview")
        self.assertEqual(self._registry["coverage"]["mode"], "complete")
        self.assertEqual(self._registry["coverage"]["canonical_tool_count"], 25)
        self.assertEqual(self._registry["coverage"]["public_name_count"], 26)
        self.assertEqual(self._registry["coverage"]["target_canonical_tool_count"], 25)
        self.assertEqual(self._registry["coverage"]["target_public_name_count"], 26)

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

    def test_configuration_declarations_and_wizard_alias_match_canonical_schemas(self) -> None:
        lite_contract = json.loads(
            (REPO_ROOT / "content/mcp-contracts/lite-tools.json").read_text(
                encoding="utf-8"
            )
        )
        lite_tools = {tool["name"]: tool for tool in lite_contract["tools"]}
        full_tools = {tool["name"]: tool for tool in MCP_TOOL_DEFINITIONS}

        for name, schemas in self._configuration_schemas.items():
            expected = runtime_schema_projection(schemas["input"])
            with self.subTest(tool=name, runtime="lite"):
                self.assertEqual(lite_tools[name]["inputSchema"], expected)
            with self.subTest(tool=name, runtime="full"):
                self.assertEqual(full_tools[name]["inputSchema"], expected)

        canonical = "qiongli_configure_provider"
        alias = "qiongli_open_config_wizard"
        for runtime_tools in (lite_tools, full_tools):
            self.assertEqual(
                runtime_tools[alias]["inputSchema"],
                runtime_tools[canonical]["inputSchema"],
            )
            self.assertEqual(
                runtime_tools[alias]["description"],
                f"Compatibility alias for {canonical}.",
            )

        record = next(tool for tool in self._registry["tools"] if tool["name"] == canonical)
        self.assertEqual([item["name"] for item in record["aliases"]], [alias])
        self.assertEqual(
            record["security"]["sensitive_output_paths"],
            ["/url", "/config_path"],
        )
        for name in ("qiongli_config_status", "qiongli_save_provider_config"):
            record = next(tool for tool in self._registry["tools"] if tool["name"] == name)
            self.assertEqual(
                record["security"]["sensitive_output_paths"],
                ["/config_path"],
            )

    def test_literature_planning_declarations_match_canonical_schemas(self) -> None:
        lite_contract = json.loads(
            (REPO_ROOT / "content/mcp-contracts/lite-tools.json").read_text(
                encoding="utf-8"
            )
        )
        lite_tools = {tool["name"]: tool for tool in lite_contract["tools"]}
        full_tools = {tool["name"]: tool for tool in MCP_TOOL_DEFINITIONS}

        for name, schemas in self._literature_planning_schemas.items():
            expected = runtime_schema_projection(schemas["input"])
            with self.subTest(tool=name, runtime="lite"):
                self.assertEqual(lite_tools[name]["inputSchema"], expected)
            with self.subTest(tool=name, runtime="full"):
                self.assertEqual(full_tools[name]["inputSchema"], expected)

        search_record = next(
            tool for tool in self._registry["tools"] if tool["name"] == "qiongli_search_plan"
        )
        self.assertEqual(
            search_record["security"]["sensitive_output_paths"],
            [
                "/query",
                "/provider_queries",
                "/native_search_queries",
                "/native_fulltext_queries",
            ],
        )

    def test_experience_records_are_declared_as_sensitive_outputs(self) -> None:
        records = {tool["name"]: tool for tool in self._registry["tools"]}

        self.assertEqual(
            records["qiongli_experience_query"]["security"]["sensitive_output_paths"],
            ["/project_dir", "/records"],
        )
        self.assertEqual(
            records["qiongli_experience_show"]["security"]["sensitive_output_paths"],
            ["/project_dir", "/record_path", "/record"],
        )
        self.assertEqual(
            records["qiongli_experience_lessons"]["security"]["sensitive_output_paths"],
            ["/project_dir", "/records"],
        )

    def test_divergent_outputs_declare_profile_scoped_sensitive_paths(self) -> None:
        records = {tool["name"]: tool for tool in self._registry["tools"]}

        for name in (
            "qiongli_literature_search",
            "qiongli_orchestrator_route",
            "qiongli_task_plan",
        ):
            with self.subTest(tool=name):
                paths = records[name]["security"]["profile_sensitive_output_paths"]
                self.assertEqual(set(paths), {"marketplace-lite", "full"})
                self.assertNotEqual(paths["marketplace-lite"], paths["full"])

    def test_validator_rejects_missing_profile_scoped_security_metadata(self) -> None:
        invalid = copy.deepcopy(self._registry)
        record = next(
            tool
            for tool in invalid["tools"]
            if tool["name"] == "qiongli_literature_search"
        )
        del record["security"]["profile_sensitive_output_paths"]["full"]

        failures = self._validate_registry_mutation(invalid)

        self.assertTrue(
            any(
                "divergent output schemas require profile-sensitive paths" in failure
                for failure in failures
            ),
            failures,
        )

    def test_literature_status_golden_call_is_schema_valid_and_secret_free(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            config_home = root / "config"
            canary = "QIONGLI_LITERATURE_STATUS_CANARY_DO_NOT_ECHO"
            runtime_env = self._provider_env(config_home, openalex=canary)
            arguments = {"cwd": str(root)}
            lite_response = self._call_lite(
                arguments,
                tool_name="qiongli_literature_status",
                config_home=config_home,
                env_overrides=runtime_env,
            )
            with mock.patch.dict(os.environ, runtime_env, clear=True):
                full_response = call_qiongli_tool("qiongli_literature_status", arguments)

        lite_output = lite_response["result"]["structuredContent"]
        full_output = full_response["structuredContent"]
        schema = self._literature_planning_schemas["qiongli_literature_status"]["output"]
        self.assertEqual(validate_instance(lite_output, schema), [])
        self.assertEqual(validate_instance(full_output, schema), [])
        for field in (
            "status",
            "capability_mode",
            "providers",
            "active_providers",
            "missing",
            "next_action",
        ):
            self.assertEqual(lite_output.get(field), full_output.get(field))
        self.assertEqual(
            set(lite_output["provider_capabilities"]),
            set(full_output["provider_capabilities"]),
        )
        for provider, lite_capability in lite_output["provider_capabilities"].items():
            full_capability = full_output["provider_capabilities"][provider]
            with self.subTest(provider=provider):
                self.assertEqual(lite_capability["status"], full_capability["status"])
                self.assertEqual(
                    lite_capability.get("max_per_provider_limit"),
                    full_capability.get("max_per_provider_limit"),
                )
                self.assertTrue(
                    set(lite_capability["capabilities"]).issubset(
                        full_capability["capabilities"]
                    )
                )
        self.assertNotIn(canary, json.dumps(lite_response, sort_keys=True))
        self.assertNotIn(canary, json.dumps(full_response, sort_keys=True))

    def test_rich_search_plan_golden_call_matches_across_runtimes(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            config_home = root / "config"
            runtime_env = self._provider_env(config_home)
            arguments = {
                "cwd": str(root),
                "query": "platform governance",
                "platform": "codex",
                "native_search_available": True,
                "native_search_tools": ["codex_web_search"],
                "query_variants": ["governance platform"],
                "include_working_papers": True,
                "from_year": 2020,
                "to_year": "2026",
                "search_mode": "review",
                "venue_filter": "Research Policy",
                "document_types": ["article", "preprint"],
            }
            lite_response = self._call_lite(
                arguments,
                tool_name="qiongli_search_plan",
                config_home=config_home,
                env_overrides=runtime_env,
            )
            with mock.patch.dict(os.environ, runtime_env, clear=True):
                full_response = call_qiongli_tool("qiongli_search_plan", arguments)

        lite_output = lite_response["result"]["structuredContent"]
        full_output = full_response["structuredContent"]
        schema = self._literature_planning_schemas["qiongli_search_plan"]["output"]
        self.assertEqual(validate_instance(lite_output, schema), [])
        self.assertEqual(validate_instance(full_output, schema), [])
        self.assertEqual(lite_output, full_output)
        self.assertEqual(lite_output["search_execution_mode"], "hybrid_search")
        self.assertTrue(
            all(
                query["filters"]["from_year"] == query["filters"]["fromYear"]
                and query["filters"]["to_year"] == query["filters"]["toYear"]
                for query in lite_output["provider_queries"]
            )
        )
        self.assertEqual(
            {query["source"] for query in lite_output["provider_queries"]},
            {"primary", "variant"},
        )
        self.assertTrue(
            all(
                query["candidate_status"] == "candidate_only"
                for query in lite_output["native_fulltext_queries"]
            )
        )

    def test_search_plan_legacy_aliases_normalize_to_canonical_output(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            config_home = root / "config"
            runtime_env = self._provider_env(config_home)
            arguments = {
                "cwd": str(root),
                "query": "reproducible scholarship",
                "platform": "claude-code",
                "nativeSearchAvailable": True,
                "nativeSearchTools": ["claude_web_search"],
                "queryVariants": ["reproducible research"],
                "includeWorkingPapers": False,
                "fromYear": "2021",
                "toYear": 2026,
                "searchMode": "topic",
                "venueFilter": "Scientometrics",
                "documentTypes": ["article"],
            }
            lite_response = self._call_lite(
                arguments,
                tool_name="qiongli_search_plan",
                config_home=config_home,
                env_overrides=runtime_env,
            )
            with mock.patch.dict(os.environ, runtime_env, clear=True):
                full_response = call_qiongli_tool("qiongli_search_plan", arguments)

        lite_output = lite_response["result"]["structuredContent"]
        full_output = full_response["structuredContent"]
        self.assertEqual(lite_output, full_output)
        self.assertEqual(lite_output["search_mode"], "topic")
        self.assertEqual(lite_output["native_search_tools"], ["claude_web_search"])
        self.assertEqual(
            lite_output["provider_queries"][0]["filters"]["fromYear"],
            2021,
        )
        self.assertEqual(
            lite_output["provider_queries"][0]["filters"]["toYear"],
            2026,
        )

    def test_search_plan_identifier_edges_normalize_equally_across_runtimes(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            config_home = root / "config"
            runtime_env = self._provider_env(config_home)
            arguments = {
                "cwd": str(root),
                "query": "normalization edge",
                "platform": "Codex-",
                "native_search_available": True,
                "native_search_tools": ["CODEX  web--search_"],
                "query_variants": ["Straße", "STRASSE"],
                "document_types": ["Straße", "STRASSE"],
            }
            lite_response = self._call_lite(
                arguments,
                tool_name="qiongli_search_plan",
                config_home=config_home,
                env_overrides=runtime_env,
            )
            with mock.patch.dict(os.environ, runtime_env, clear=True):
                full_response = call_qiongli_tool("qiongli_search_plan", arguments)

        lite_output = lite_response["result"]["structuredContent"]
        full_output = full_response["structuredContent"]
        self.assertEqual(lite_output, full_output)
        self.assertEqual(lite_output["platform"], "codex")
        self.assertEqual(lite_output["native_search_tools"], ["codex_web_search"])
        self.assertEqual(
            [query["query"] for query in lite_output["provider_queries"][:3]],
            ["normalization edge", "Straße", "STRASSE"],
        )

    def test_literature_planning_argument_errors_share_semantic_class(self) -> None:
        cases = (
            ("qiongli_literature_status", {"cwd": 7}),
            ("qiongli_literature_status", {"unexpected": True}),
            ("qiongli_search_plan", {}),
            ("qiongli_search_plan", {"query": "   "}),
            ("qiongli_search_plan", {"query": "x", "unexpected": True}),
            (
                "qiongli_search_plan",
                {
                    "query": "x",
                    "native_search_available": True,
                    "nativeSearchAvailable": True,
                },
            ),
            ("qiongli_search_plan", {"query": "x", "from_year": 2026, "to_year": 2020}),
            ("qiongli_search_plan", {"query": "x", "query_variants": ["x"] * 17}),
            ("qiongli_search_plan", {"query": "x", "platform": "İ"}),
            (
                "qiongli_search_plan",
                {"query": "x", "query_variants": [" y ", "y"]},
            ),
            (
                "qiongli_search_plan",
                {"query": "x", "document_types": [" article ", "Article"]},
            ),
            (
                "qiongli_search_plan",
                {
                    "query": "x",
                    "native_search_tools": ["codex web search", "codex_web-search"],
                },
            ),
        )
        for name, arguments in cases:
            with self.subTest(tool=name, arguments=arguments):
                lite_response = self._call_lite(arguments, tool_name=name)
                full_response = call_qiongli_tool(name, arguments)
                self.assertEqual(lite_response["error"]["code"], -32602)
                self.assertTrue(full_response["isError"])
                self.assertEqual(
                    full_response["structuredContent"]["error_kind"],
                    "invalid_arguments",
                )

    def test_literature_planning_config_failure_is_redacted_tool_error(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            config_home = root / "private-config-canary"
            config_home.mkdir()
            config_path = config_home / "providers.json"
            malformed_canary = "QIONGLI_MALFORMED_LITERATURE_CANARY"
            config_path.write_text(f'{{"providers": "{malformed_canary}"', encoding="utf-8")
            runtime_env = self._provider_env(config_home)
            calls = (
                ("qiongli_literature_status", {"cwd": str(root)}),
                ("qiongli_search_plan", {"cwd": str(root), "query": "governance"}),
            )
            for name, arguments in calls:
                with self.subTest(tool=name):
                    lite_response = self._call_lite(
                        arguments,
                        tool_name=name,
                        config_home=config_home,
                        env_overrides=runtime_env,
                    )
                    with mock.patch.dict(os.environ, runtime_env, clear=True):
                        full_response = call_qiongli_tool(name, arguments)
                    self.assertTrue(lite_response["result"]["isError"])
                    self.assertEqual(
                        lite_response["result"]["structuredContent"]["error_kind"],
                        "tool_error",
                    )
                    self.assertTrue(full_response["isError"])
                    self.assertEqual(
                        full_response["structuredContent"]["error_kind"],
                        "tool_error",
                    )
                    rendered = json.dumps(
                        {"lite": lite_response, "full": full_response}, sort_keys=True
                    )
                    self.assertNotIn(malformed_canary, rendered)
                    self.assertNotIn(str(config_home), rendered)

    def test_config_status_golden_call_is_schema_valid_and_secret_free(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            config_home = root / "config"
            canary = "QIONGLI_CONFIG_STATUS_CANARY_DO_NOT_ECHO"
            runtime_env = {
                "QIONGLI_CONFIG_HOME": str(config_home),
                "QIONGLI_OPENALEX_API_KEY": canary,
                "OPENALEX_API_KEY": "",
                "QIONGLI_MCPB_OPENALEX_API_KEY": "",
                "QIONGLI_SEMANTIC_SCHOLAR_API_KEY": canary,
                "SEMANTIC_SCHOLAR_API_KEY": "",
                "S2_API_KEY": "",
                "QIONGLI_MCPB_SEMANTIC_SCHOLAR_API_KEY": "",
                "QIONGLI_CROSSREF_EMAIL": "",
                "CROSSREF_EMAIL": "",
                "QIONGLI_MCPB_CROSSREF_EMAIL": "",
                "QIONGLI_NCBI_API_KEY": "",
                "NCBI_API_KEY": "",
                "PUBMED_API_KEY": "",
                "QIONGLI_MCPB_PUBMED_API_KEY": "",
            }
            arguments = {"cwd": str(root)}
            lite_response = self._call_lite(
                arguments,
                tool_name="qiongli_config_status",
                config_home=config_home,
                env_overrides=runtime_env,
            )
            with mock.patch.dict(os.environ, runtime_env, clear=True):
                full_response = call_qiongli_tool("qiongli_config_status", arguments)

        lite_output = lite_response["result"]["structuredContent"]
        full_output = full_response["structuredContent"]
        schema = self._configuration_schemas["qiongli_config_status"]["output"]
        self.assertEqual(validate_instance(lite_output, schema), [])
        self.assertEqual(validate_instance(full_output, schema), [])
        for field in (
            "status",
            "config_path",
            "providers",
            "capability_mode",
            "missing",
            "next_action",
        ):
            self.assertEqual(lite_output[field], full_output[field])
        self.assertEqual(
            lite_output["redacted_config"]["providers"],
            full_output["redacted_config"]["providers"],
        )
        self.assertNotIn(canary, json.dumps(lite_response, sort_keys=True))
        self.assertNotIn(canary, json.dumps(full_response, sort_keys=True))

    def test_config_status_disabled_arxiv_is_strategy_only_across_runtimes(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            config_home = root / "config"
            config_home.mkdir()
            (config_home / "providers.json").write_text(
                json.dumps(
                    {
                        "version": 1,
                        "providers": {"arxiv": {"enabled": False}},
                    }
                ),
                encoding="utf-8",
            )
            runtime_env = self._provider_env(config_home)
            arguments = {"cwd": str(root)}
            lite_response = self._call_lite(
                arguments,
                tool_name="qiongli_config_status",
                config_home=config_home,
                env_overrides=runtime_env,
            )
            with mock.patch.dict(os.environ, runtime_env, clear=True):
                full_response = call_qiongli_tool(
                    "qiongli_config_status",
                    arguments,
                )

        lite_output = lite_response["result"]["structuredContent"]
        full_output = full_response["structuredContent"]
        schema = self._configuration_schemas["qiongli_config_status"]["output"]
        self.assertEqual(validate_instance(lite_output, schema), [])
        self.assertEqual(validate_instance(full_output, schema), [])
        self.assertEqual(lite_output["capability_mode"], "strategy_only")
        self.assertEqual(full_output["capability_mode"], "strategy_only")
        self.assertEqual(lite_output["providers"], full_output["providers"])
        self.assertEqual(lite_output["providers"]["arxiv"], "configured")
        self.assertTrue(
            all(
                status == "missing"
                for provider, status in lite_output["providers"].items()
                if provider != "arxiv"
            )
        )
        self.assertIs(
            lite_output["redacted_config"]["providers"]["arxiv"]["enabled"],
            False,
        )
        self.assertIs(
            full_output["redacted_config"]["providers"]["arxiv"]["enabled"],
            False,
        )

    def test_save_provider_config_golden_call_normalizes_and_never_echoes_secret(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            config_home = root / "config"
            canary = "QIONGLI_SAVE_CONFIG_CANARY_DO_NOT_ECHO"
            arguments = {
                "provider": "s2",
                "field": "api-key",
                "value": f"  {canary}  ",
            }
            schema = self._configuration_schemas["qiongli_save_provider_config"]
            self.assertEqual(validate_instance(arguments, schema["input"]), [])
            self.assertTrue(
                any(
                    "oneOf" in failure
                    for failure in validate_instance(
                        {"provider": "crossref", "field": "api_key", "value": "x"},
                        schema["input"],
                    )
                )
            )
            lite_response = self._call_lite(
                arguments,
                tool_name="qiongli_save_provider_config",
                config_home=config_home,
            )
            with mock.patch.dict(
                os.environ,
                {"QIONGLI_CONFIG_HOME": str(config_home)},
                clear=True,
            ):
                full_response = call_qiongli_tool("qiongli_save_provider_config", arguments)
            persisted = json.loads(
                (config_home / "providers.json").read_text(encoding="utf-8")
            )

        lite_output = lite_response["result"]["structuredContent"]
        full_output = full_response["structuredContent"]
        self.assertEqual(validate_instance(lite_output, schema["output"]), [])
        self.assertEqual(validate_instance(full_output, schema["output"]), [])
        self.assertEqual(lite_output, full_output)
        self.assertEqual(lite_output["provider"], "semantic_scholar")
        self.assertEqual(lite_output["field"], "api_key")
        self.assertIs(lite_output["saved"], True)
        self.assertEqual(
            persisted["providers"]["semantic_scholar"]["api_key"],
            canary,
        )
        self.assertNotIn(canary, json.dumps(lite_response, sort_keys=True))
        self.assertNotIn(canary, json.dumps(full_response, sort_keys=True))

    def test_configuration_argument_errors_share_semantic_class(self) -> None:
        cases = (
            ("qiongli_config_status", {"cwd": 7}),
            ("qiongli_config_status", {"unexpected": True}),
            (
                "qiongli_save_provider_config",
                {"provider": "crossref", "field": "api_key", "value": "secret"},
            ),
            (
                "qiongli_save_provider_config",
                {"provider": "openalex", "field": "api_key", "value": 7},
            ),
            ("qiongli_configure_provider", {"host": "example.invalid"}),
            ("qiongli_configure_provider", {"port": 1.5}),
            ("qiongli_open_config_wizard", {"provider": "arxiv"}),
        )
        for name, arguments in cases:
            with self.subTest(tool=name, arguments=arguments):
                lite_response = self._call_lite(arguments, tool_name=name)
                full_response = call_qiongli_tool(name, arguments)
                self.assertEqual(lite_response["error"]["code"], -32602)
                self.assertTrue(full_response["isError"])
                self.assertEqual(
                    full_response["structuredContent"]["error_kind"],
                    "invalid_arguments",
                )

    def test_configure_provider_outputs_validate_and_alias_reuses_active_session(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            config_home = root / "config"
            arguments = {"provider": "s2", "host": "localhost", "port": 0}
            lite_response = self._call_lite(
                arguments,
                tool_name="qiongli_configure_provider",
                config_home=config_home,
            )
            with mock.patch.dict(
                os.environ,
                {"QIONGLI_CONFIG_HOME": str(config_home)},
                clear=True,
            ):
                full_response = call_qiongli_tool("qiongli_configure_provider", arguments)
                if full_response.get("isError"):
                    self.skipTest(full_response["structuredContent"].get("error", "bind failed"))
                alias_response = call_qiongli_tool(
                    "qiongli_open_config_wizard",
                    {"provider": "openalex", "host": "127.0.0.1", "port": 0},
                )
            active_wizard = tool_handlers._ACTIVE_CONFIG_WIZARD
            try:
                lite_output = lite_response["result"]["structuredContent"]
                full_output = full_response["structuredContent"]
                alias_output = alias_response["structuredContent"]
                schema = self._configuration_schemas["qiongli_configure_provider"]["output"]
                self.assertEqual(validate_instance(lite_output, schema), [])
                self.assertEqual(validate_instance(full_output, schema), [])
                self.assertEqual(validate_instance(alias_output, schema), [])
                for output in (lite_output, full_output, alias_output):
                    self.assertEqual(output["host"], "127.0.0.1")
                    self.assertEqual(output["provider"], "semantic_scholar")
                self.assertEqual(full_output["status"], "ready")
                self.assertEqual(alias_output["status"], "already_running")
                self.assertEqual(alias_output["url"], full_output["url"])
            finally:
                if active_wizard is not None:
                    active_wizard.stop()
                tool_handlers._ACTIVE_CONFIG_WIZARD = None

    def test_malformed_config_fails_closed_as_tool_error_without_secret_echo(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            config_home = root / "config"
            config_home.mkdir()
            config_path = config_home / "providers.json"
            malformed = '{"version":1,"providers":'
            config_path.write_text(malformed, encoding="utf-8")
            canary = "QIONGLI_MALFORMED_CONFIG_CANARY_DO_NOT_ECHO"
            calls = (
                ("qiongli_config_status", {"cwd": str(root)}),
                (
                    "qiongli_save_provider_config",
                    {"provider": "openalex", "field": "api_key", "value": canary},
                ),
            )
            for name, arguments in calls:
                with self.subTest(tool=name):
                    lite_response = self._call_lite(
                        arguments,
                        tool_name=name,
                        config_home=config_home,
                    )
                    with mock.patch.dict(
                        os.environ,
                        {"QIONGLI_CONFIG_HOME": str(config_home)},
                        clear=True,
                    ):
                        full_response = call_qiongli_tool(name, arguments)
                    self.assertTrue(lite_response["result"]["isError"])
                    self.assertEqual(
                        lite_response["result"]["structuredContent"]["error_kind"],
                        "tool_error",
                    )
                    self.assertTrue(full_response["isError"])
                    self.assertEqual(
                        full_response["structuredContent"]["error_kind"],
                        "tool_error",
                    )
                    self.assertNotIn(canary, json.dumps(lite_response, sort_keys=True))
                    self.assertNotIn(canary, json.dumps(full_response, sort_keys=True))
            self.assertEqual(config_path.read_text(encoding="utf-8"), malformed)

    def test_v2_generic_tool_errors_do_not_echo_sensitive_exception_text(self) -> None:
        path_canary = "/private/users/canary/.config/qiongli/providers.json"
        cases = (
            (
                "qiongli_save_provider_config",
                {"provider": "openalex", "field": "api_key", "value": "secret"},
                mock.patch.object(
                    tool_handlers,
                    "set_provider_value",
                    side_effect=OSError(f"permission denied: {path_canary}"),
                ),
            ),
            (
                "qiongli_configure_provider",
                {"host": "127.0.0.1", "port": 0},
                mock.patch.object(
                    tool_handlers,
                    "start_config_wizard",
                    side_effect=OSError(f"bind failed near {path_canary}"),
                ),
            ),
        )

        for name, arguments, patcher in cases:
            with self.subTest(tool=name), patcher:
                response = call_qiongli_tool(name, arguments)
                rendered = json.dumps(response, sort_keys=True)
                self.assertTrue(response["isError"])
                self.assertEqual(
                    response["structuredContent"]["error_kind"],
                    "tool_error",
                )
                self.assertNotIn(path_canary, rendered)
                self.assertNotIn("permission denied", rendered)
                self.assertNotIn("bind failed", rendered)

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

    def test_validator_rejects_missing_runtime_alias(self) -> None:
        full_definitions = {tool["name"]: tool for tool in MCP_TOOL_DEFINITIONS}
        del full_definitions["qiongli_open_config_wizard"]

        failures = validate_capability_contract(
            REPO_ROOT,
            full_tool_definitions=full_definitions,
        )

        self.assertTrue(
            any("alias qiongli_open_config_wizard is missing" in failure for failure in failures),
            failures,
        )

    def test_validator_rejects_runtime_deletion_against_frozen_inventory(self) -> None:
        full_definitions = {tool["name"]: tool for tool in MCP_TOOL_DEFINITIONS}
        del full_definitions["qiongli_task_run"]

        failures = validate_capability_contract(
            REPO_ROOT,
            full_tool_definitions=full_definitions,
        )

        self.assertTrue(
            any(
                "full runtime public names differs from frozen CTR-201 inventory" in failure
                and "qiongli_task_run" in failure
                for failure in failures
            ),
            failures,
        )
        self.assertTrue(
            any(
                "live Full/Lite runtime union" in failure
                for failure in failures
            ),
            failures,
        )

    def test_validator_rejects_stale_registry_target_coverage_fields(self) -> None:
        original_load_json = capability_contract_validator._load_json
        registry_path = (REPO_ROOT / "content/mcp-contracts/v2/registry.json").resolve()

        for field, stale_value, derived_value in (
            ("target_canonical_tool_count", 24, 25),
            ("target_public_name_count", 25, 26),
        ):
            invalid = copy.deepcopy(self._registry)
            invalid["coverage"][field] = stale_value

            def load_json(path: Path, *, registry: dict[str, object] = invalid) -> object:
                if path.resolve() == registry_path:
                    return copy.deepcopy(registry)
                return original_load_json(path)

            with self.subTest(field=field), mock.patch.object(
                capability_contract_validator,
                "_load_json",
                side_effect=load_json,
            ):
                failures = capability_contract_validator.validate_capability_contract(
                    REPO_ROOT
                )

            self.assertTrue(
                any(
                    field in failure and f"({derived_value})" in failure
                    for failure in failures
                ),
                failures,
            )

    def test_validator_rejects_complete_registry_reordering(self) -> None:
        invalid = copy.deepcopy(self._registry)
        invalid["tools"][0], invalid["tools"][1] = (
            invalid["tools"][1],
            invalid["tools"][0],
        )

        failures = self._validate_registry_mutation(invalid)

        self.assertIn(
            "registry canonical tool order differs from frozen CTR-201 inventory",
            failures,
        )

    def test_require_complete_rejects_pilot_mode(self) -> None:
        invalid = copy.deepcopy(self._registry)
        invalid["status"] = "pilot"
        invalid["coverage"]["mode"] = "pilot"

        failures = self._validate_registry_mutation(
            invalid,
            require_complete=True,
        )

        self.assertIn("Capability Contract v2 complete coverage is required", failures)

    def test_validator_rejects_profile_overclaim_and_non_tool_schema_refs(self) -> None:
        invalid = copy.deepcopy(self._registry)
        task_run = next(
            tool for tool in invalid["tools"] if tool["name"] == "qiongli_task_run"
        )
        task_run["profiles"]["marketplace-lite"] = {
            "exposure": "tool",
            "input_schema_ref": task_run["profiles"]["full"]["input_schema_ref"],
            "output_schema_ref": task_run["profiles"]["full"]["output_schema_ref"],
            "error_transport": "json-rpc-and-tool-result",
        }

        failures = self._validate_registry_mutation(invalid)

        self.assertTrue(
            any(
                "qiongli_task_run.marketplace-lite: exposure must be 'unavailable'"
                in failure
                for failure in failures
            ),
            failures,
        )

        invalid = copy.deepcopy(self._registry)
        task_run = next(
            tool for tool in invalid["tools"] if tool["name"] == "qiongli_task_run"
        )
        task_run["profiles"]["marketplace-lite"]["input_schema_ref"] = (
            task_run["profiles"]["full"]["input_schema_ref"]
        )
        failures = self._validate_registry_mutation(invalid)
        self.assertTrue(
            any(
                "qiongli_task_run.marketplace-lite: non-tool exposure must not declare schema refs"
                in failure
                for failure in failures
            ),
            failures,
        )

    def test_validator_indexes_divergent_input_schemas_by_profile(self) -> None:
        lite_contract = json.loads(
            (REPO_ROOT / "content/mcp-contracts/lite-tools.json").read_text(
                encoding="utf-8"
            )
        )
        lite_definitions = {tool["name"]: tool for tool in lite_contract["tools"]}
        full_definitions = {tool["name"]: tool for tool in MCP_TOOL_DEFINITIONS}
        lite_definitions["qiongli_literature_search"] = {
            **lite_definitions["qiongli_literature_search"],
            "inputSchema": full_definitions["qiongli_literature_search"]["inputSchema"],
        }

        failures = validate_capability_contract(
            REPO_ROOT,
            lite_tool_definitions=lite_definitions,
        )

        self.assertIn(
            "qiongli_literature_search: marketplace-lite input schema drifts from v2 registry",
            failures,
        )

    def test_validator_rejects_wrong_profile_specific_schema_id(self) -> None:
        schema_path = (
            CONTRACT_ROOT
            / "schemas/qiongli_task_run.full.input.schema.json"
        ).resolve()
        invalid_schema = json.loads(schema_path.read_text(encoding="utf-8"))
        invalid_schema["$id"] = (
            "https://qiongli.dev/schemas/tools/qiongli_task_run.input.v2.json"
        )
        original_load_json = capability_contract_validator._load_json

        def load_json(path: Path) -> object:
            if path.resolve() == schema_path:
                return copy.deepcopy(invalid_schema)
            return original_load_json(path)

        with mock.patch.object(
            capability_contract_validator,
            "_load_json",
            side_effect=load_json,
        ):
            failures = validate_capability_contract(REPO_ROOT)

        self.assertTrue(
            any(
                "qiongli_task_run.full: input_schema_ref must declare $id" in failure
                and "qiongli_task_run.full.input.v2.json" in failure
                for failure in failures
            ),
            failures,
        )

    def test_validator_rejects_open_or_empty_top_level_output_envelopes(self) -> None:
        schema_path = (
            CONTRACT_ROOT
            / "schemas/qiongli_task_run.full.output.schema.json"
        ).resolve()
        invalid_schema = json.loads(schema_path.read_text(encoding="utf-8"))
        invalid_schema["properties"] = {}
        invalid_schema["required"] = []
        invalid_schema["additionalProperties"] = True
        original_load_json = capability_contract_validator._load_json

        def load_json(path: Path) -> object:
            if path.resolve() == schema_path:
                return copy.deepcopy(invalid_schema)
            return original_load_json(path)

        with mock.patch.object(
            capability_contract_validator,
            "_load_json",
            side_effect=load_json,
        ):
            failures = validate_capability_contract(REPO_ROOT)

        self.assertTrue(
            any(
                "must close its top-level additional properties" in failure
                for failure in failures
            ),
            failures,
        )
        self.assertTrue(
            any("must declare non-empty top-level properties" in failure for failure in failures),
            failures,
        )
        self.assertTrue(
            any(
                "must declare non-empty top-level required properties" in failure
                for failure in failures
            ),
            failures,
        )

    def test_validator_rejects_smoke_orphans_and_missing_profile_pairs(self) -> None:
        invalid = copy.deepcopy(self._registry)
        shared = next(
            tool for tool in invalid["tools"] if len(tool["smoke_call_ids"]) >= 2
        )
        shared["smoke_call_ids"].pop()

        failures = self._validate_registry_mutation(invalid)

        self.assertTrue(
            any("registry/fixture smoke IDs" in failure for failure in failures),
            failures,
        )
        self.assertTrue(
            any("no referenced smoke fixture covers" in failure for failure in failures),
            failures,
        )

    def test_validator_rejects_downgrading_positive_smoke_to_input_error(self) -> None:
        fixture_path = (
            REPO_ROOT
            / "content/mcp-contracts/fixtures/capability-contract-v2-smoke-calls.json"
        ).resolve()
        invalid_fixture = json.loads(fixture_path.read_text(encoding="utf-8"))
        task_plan = next(
            call
            for call in invalid_fixture["calls"]
            if call["profile"] == "full" and call["name"] == "qiongli_task_plan"
        )
        task_plan["arguments"] = {}
        task_plan["expected_response_class"] = "input_error"
        original_load_json = capability_contract_validator._load_json

        def load_json(path: Path) -> object:
            if path.resolve() == fixture_path:
                return copy.deepcopy(invalid_fixture)
            return original_load_json(path)

        with mock.patch.object(
            capability_contract_validator,
            "_load_json",
            side_effect=load_json,
        ):
            failures = validate_capability_contract(REPO_ROOT)

        self.assertTrue(
            any("input-error smoke boundary" in failure for failure in failures),
            failures,
        )

    def test_strict_json_loader_rejects_duplicate_and_nonfinite_values(self) -> None:
        cases = (
            '{"value":1,"value":2}',
            '{"value":NaN}',
            '{"value":1e999}',
        )
        with tempfile.TemporaryDirectory() as directory:
            for index, source in enumerate(cases):
                path = Path(directory) / f"invalid-{index}.json"
                path.write_text(source, encoding="utf-8")
                with self.subTest(source=source), self.assertRaises(ValueError):
                    _load_json(path)

    def test_validator_rejects_runtime_alias_outside_frozen_public_surface(
        self,
    ) -> None:
        lite_contract = json.loads(
            (REPO_ROOT / "content/mcp-contracts/lite-tools.json").read_text(
                encoding="utf-8"
            )
        )
        lite_definitions = {tool["name"]: tool for tool in lite_contract["tools"]}
        lite_definitions["qiongli_task_plan_legacy"] = {
            "name": "qiongli_task_plan_legacy",
            "description": "Compatibility alias for qiongli_task_plan.",
            "inputSchema": lite_definitions["qiongli_task_plan"]["inputSchema"],
        }

        failures = validate_capability_contract(
            REPO_ROOT,
            lite_tool_definitions=lite_definitions,
        )

        self.assertTrue(
            any(
                "marketplace-lite runtime public names differs from frozen CTR-201 inventory"
                in failure
                for failure in failures
            ),
            failures,
        )

    def test_validator_rejects_mcpb_manifest_description_drift(self) -> None:
        manifest = json.loads(
            (REPO_ROOT / "packages/qiongli-literature-mcpb/manifest.json").read_text(
                encoding="utf-8"
            )
        )
        definitions = {tool["name"]: tool for tool in manifest["tools"]}
        definitions["qiongli_search_plan"] = {
            **definitions["qiongli_search_plan"],
            "description": "drifted description",
        }

        failures = validate_capability_contract(
            REPO_ROOT,
            mcpb_tool_definitions=definitions,
        )

        self.assertTrue(
            any("MCPB manifest description drifts" in failure for failure in failures),
            failures,
        )

    def test_validator_requires_the_exact_frozen_mcpb_surface_without_full_only_tools(
        self,
    ) -> None:
        manifest = json.loads(
            (REPO_ROOT / "packages/qiongli-literature-mcpb/manifest.json").read_text(
                encoding="utf-8"
            )
        )
        definitions = {tool["name"]: tool for tool in manifest["tools"]}
        self.assertNotIn("qiongli_task_run", definitions)
        del definitions["qiongli_task_plan"]

        failures = validate_capability_contract(
            REPO_ROOT,
            mcpb_tool_definitions=definitions,
        )

        self.assertTrue(
            any("frozen MCPB surface" in failure for failure in failures),
            failures,
        )
        self.assertFalse(
            any(
                "qiongli_task_run is missing from the MCPB manifest" in failure
                for failure in failures
            ),
            failures,
        )

    def test_schema_validator_enforces_declared_string_and_array_bounds(self) -> None:
        schema = self._literature_planning_schemas["qiongli_search_plan"]["input"]

        string_failures = validate_instance({"query": "x" * 4097}, schema)
        array_failures = validate_instance(
            {"query": "x", "native_search_tools": [f"tool-{index}" for index in range(9)]},
            schema,
        )

        self.assertTrue(any("longer than 4096" in failure for failure in string_failures))
        self.assertTrue(any("at most 8 items" in failure for failure in array_failures))

    def test_schema_validator_enforces_declared_formats(self) -> None:
        self.assertEqual(
            validate_instance(
                "2026-07-12T20:30:50Z",
                {"type": "string", "format": "date-time"},
            ),
            [],
        )
        self.assertNotEqual(
            validate_instance(
                "2026-07-12 20:30:50",
                {"type": "string", "format": "date-time"},
            ),
            [],
        )
        self.assertEqual(
            validate_instance(
                "http://127.0.0.1:8765/setup?token=redacted",
                {"type": "string", "format": "uri"},
            ),
            [],
        )
        self.assertNotEqual(
            validate_instance(
                "/relative/setup",
                {"type": "string", "format": "uri"},
            ),
            [],
        )

    def test_output_schema_rejects_untraceable_snapshot(self) -> None:
        failures = validate_instance(
            {"artifact_type": "qiongli_literature_evidence_snapshot", "results": []},
            self._output_schema,
        )

        self.assertTrue(any("missing required property" in failure for failure in failures))

    def _call_lite(
        self,
        arguments: dict[str, object],
        *,
        tool_name: str = TOOL_NAME,
        config_home: Path | None = None,
        env_overrides: dict[str, str] | None = None,
    ) -> dict[str, object]:
        request = json.dumps(
            {
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": {"name": tool_name, "arguments": arguments},
            }
        )
        env = os.environ.copy()
        env["PATH"] = ""
        env["QIONGLI_CONFIG_HOME"] = str(config_home or self._root / "config")
        env.update(env_overrides or {})
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

    @staticmethod
    def _provider_env(config_home: Path, *, openalex: str = "") -> dict[str, str]:
        return {
            "QIONGLI_CONFIG_HOME": str(config_home),
            "QIONGLI_OPENALEX_API_KEY": openalex,
            "OPENALEX_API_KEY": "",
            "QIONGLI_MCPB_OPENALEX_API_KEY": "",
            "QIONGLI_OPENALEX_EMAIL": "",
            "OPENALEX_EMAIL": "",
            "QIONGLI_MCPB_OPENALEX_EMAIL": "",
            "QIONGLI_SEMANTIC_SCHOLAR_API_KEY": "",
            "SEMANTIC_SCHOLAR_API_KEY": "",
            "S2_API_KEY": "",
            "QIONGLI_MCPB_SEMANTIC_SCHOLAR_API_KEY": "",
            "QIONGLI_CROSSREF_EMAIL": "",
            "CROSSREF_EMAIL": "",
            "QIONGLI_MCPB_CROSSREF_EMAIL": "",
            "QIONGLI_NCBI_API_KEY": "",
            "NCBI_API_KEY": "",
            "PUBMED_API_KEY": "",
            "QIONGLI_MCPB_PUBMED_API_KEY": "",
        }


if __name__ == "__main__":
    unittest.main()
