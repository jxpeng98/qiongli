from __future__ import annotations

import json
import os
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from bridges.mcp_tool_handlers import call_qiongli_tool
from bridges.provider_config import set_provider_value


class MCPLiteratureToolTests(unittest.TestCase):
    def test_literature_status_reports_capabilities_without_secrets(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            with mock.patch.dict(
                "os.environ",
                {"QIONGLI_CONFIG_HOME": str(root / "config")},
                clear=True,
            ):
                set_provider_value("openalex", "api-key", "openalex-secret-key")
                result = call_qiongli_tool(
                    "qiongli_literature_status",
                    {"cwd": str(root)},
                )

        payload = result["structuredContent"]
        rendered = json.dumps(payload, sort_keys=True)
        self.assertFalse(result["isError"])
        self.assertEqual(payload["status"], "ok")
        self.assertEqual(payload["providers"]["openalex"], "configured")
        self.assertEqual(payload["active_providers"], ["openalex", "arxiv"])
        self.assertEqual(
            payload["missing"],
            ["semantic_scholar.api_key", "crossref.email", "pubmed.api_key"],
        )
        self.assertEqual(payload["provider_capabilities"], payload["capabilities"])
        self.assertIn("openalex", payload["capabilities"])
        self.assertIn("semantic_scholar", payload["capabilities"])
        self.assertIn("crossref", payload["capabilities"])
        self.assertIn("pubmed", payload["capabilities"])
        self.assertIn("arxiv", payload["capabilities"])
        self.assertEqual(payload["providers"]["arxiv"], "configured")
        self.assertEqual(
            payload["next_action"]["args"],
            {"provider": "semantic_scholar"},
        )
        self.assertNotIn("openalex-secret-key", rendered)

    def test_literature_status_and_search_plan_reject_invalid_contract_inputs(self) -> None:
        canary = "QIONGLI_LITERATURE_INPUT_CANARY_DO_NOT_ECHO"
        cases = (
            ("qiongli_literature_status", {"unknown": canary}),
            ("qiongli_literature_status", {"cwd": 7}),
            ("qiongli_literature_status", {"cwd": "   "}),
            ("qiongli_search_plan", {}),
            ("qiongli_search_plan", {"query": "   "}),
            ("qiongli_search_plan", {"query": ["not", "a", "string"]}),
            ("qiongli_search_plan", {"query": "valid", "unexpected": canary}),
            (
                "qiongli_search_plan",
                {
                    "query": "valid",
                    "native_search_available": False,
                    "nativeSearchAvailable": False,
                },
            ),
            (
                "qiongli_search_plan",
                {"query": "valid", "native_search_available": "false"},
            ),
            (
                "qiongli_search_plan",
                {"query": "valid", "from_year": "20x4"},
            ),
            (
                "qiongli_search_plan",
                {"query": "valid", "from_year": 2025, "toYear": 2024},
            ),
            (
                "qiongli_search_plan",
                {"query": "valid", "native_search_tools": [canary] * 9},
            ),
        )

        with tempfile.TemporaryDirectory() as tmp_dir:
            config_home = Path(tmp_dir) / "config"
            with mock.patch.dict(
                "os.environ",
                {"QIONGLI_CONFIG_HOME": str(config_home)},
                clear=True,
            ):
                for tool, arguments in cases:
                    with self.subTest(tool=tool, arguments=list(arguments)):
                        result = call_qiongli_tool(tool, arguments)
                        self.assertTrue(result["isError"])
                        self.assertEqual(
                            result["structuredContent"]["error_kind"],
                            "invalid_arguments",
                        )
                        self.assertNotIn(canary, json.dumps(result, sort_keys=True))

    def test_search_plan_normalizes_legacy_aliases_and_years(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            with mock.patch.dict(
                "os.environ",
                {"QIONGLI_CONFIG_HOME": str(root / "config")},
                clear=True,
            ):
                result = call_qiongli_tool(
                    "qiongli_search_plan",
                    {
                        "cwd": str(root),
                        "query": "  AI feedback  ",
                        "platform": "Claude Code",
                        "nativeSearchAvailable": True,
                        "nativeSearchTools": ["claude_web_search"],
                        "queryVariants": ["algorithmic feedback"],
                        "includeWorkingPapers": True,
                        "fromYear": "2020",
                        "toYear": 2025,
                        "searchMode": "review",
                        "venueFilter": "AER",
                        "documentTypes": ["journal-article"],
                    },
                )

        payload = result["structuredContent"]
        self.assertFalse(result["isError"])
        self.assertEqual(payload["query"], "AI feedback")
        self.assertEqual(payload["search_mode"], "review")
        self.assertEqual(payload["platform"], "claude_code")
        self.assertEqual(
            [query["query"] for query in payload["native_search_queries"]],
            ["AI feedback", "algorithmic feedback"],
        )
        self.assertEqual(
            payload["native_search_queries"][0]["filters"],
            {
                "include_working_papers": True,
                "from_year": 2020,
                "fromYear": 2020,
                "to_year": 2025,
                "toYear": 2025,
                "search_mode": "review",
                "venue_filter": "AER",
                "document_types": ["journal-article"],
            },
        )

    def test_search_plan_omits_empty_venue_and_normalizes_platform(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            with mock.patch.dict(
                "os.environ",
                {"QIONGLI_CONFIG_HOME": str(root / "config")},
                clear=True,
            ):
                result = call_qiongli_tool(
                    "qiongli_search_plan",
                    {
                        "cwd": str(root),
                        "query": "governance",
                        "platform": "Codex  Desktop",
                        "venue_filter": "   ",
                    },
                )

        payload = result["structuredContent"]
        self.assertFalse(result["isError"])
        self.assertEqual(payload["platform"], "codex_desktop")
        self.assertNotIn("venue_filter", payload["provider_queries"][0]["filters"])

    def test_search_plan_excludes_configured_but_disabled_providers(self) -> None:
        secret = "disabled-openalex-secret"
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            config_home = root / "config"
            config_home.mkdir()
            (config_home / "providers.json").write_text(
                json.dumps(
                    {
                        "providers": {
                            "openalex": {
                                "enabled": False,
                                "api_key": secret,
                            }
                        }
                    }
                ),
                encoding="utf-8",
            )
            with mock.patch.dict(
                "os.environ",
                {"QIONGLI_CONFIG_HOME": str(config_home)},
                clear=True,
            ):
                status = call_qiongli_tool(
                    "qiongli_literature_status",
                    {"cwd": str(root)},
                )
                plan = call_qiongli_tool(
                    "qiongli_search_plan",
                    {"cwd": str(root), "query": "governance"},
                )

        status_payload = status["structuredContent"]
        plan_payload = plan["structuredContent"]
        self.assertEqual(status_payload["providers"]["openalex"], "configured")
        self.assertEqual(status_payload["active_providers"], ["arxiv"])
        self.assertEqual(
            [query["provider"] for query in plan_payload["provider_queries"]],
            ["arxiv"],
        )
        rendered = json.dumps({"status": status, "plan": plan}, sort_keys=True)
        self.assertNotIn(secret, rendered)

    def test_provider_config_errors_are_fixed_and_do_not_expose_path_or_content(self) -> None:
        canary = "QIONGLI_MALFORMED_LITERATURE_CANARY_DO_NOT_ECHO"
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            config_home = root / "private-config-location"
            config_home.mkdir()
            config_path = config_home / "providers.json"
            config_path.write_text(
                '{"providers":{"openalex":{"api_key":"' + canary + '"}',
                encoding="utf-8",
            )
            with mock.patch.dict(
                "os.environ",
                {"QIONGLI_CONFIG_HOME": str(config_home)},
                clear=True,
            ):
                responses = (
                    call_qiongli_tool(
                        "qiongli_literature_status",
                        {"cwd": str(root)},
                    ),
                    call_qiongli_tool(
                        "qiongli_search_plan",
                        {"cwd": str(root), "query": "governance"},
                    ),
                )

        for response in responses:
            with self.subTest(tool=response["structuredContent"]["tool"]):
                rendered = json.dumps(response, sort_keys=True)
                self.assertTrue(response["isError"])
                self.assertEqual(
                    response["structuredContent"]["error_kind"],
                    "tool_error",
                )
                self.assertEqual(
                    response["structuredContent"]["message"],
                    "provider configuration could not be read safely",
                )
                self.assertNotIn(canary, rendered)
                self.assertNotIn(str(config_home), rendered)
                self.assertNotIn("providers.json", rendered)

    def test_invalid_provider_structure_is_redacted_across_config_and_planning_tools(self) -> None:
        persisted_canary = "QIONGLI_INVALID_PROVIDER_STRUCTURE_CANARY"
        save_canary = "QIONGLI_SAVE_VALUE_CANARY"
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            config_home = root / "private-config-location"
            config_home.mkdir()
            config_path = config_home / "providers.json"
            original = json.dumps(
                {
                    "version": 1,
                    "providers": {
                        "openalex": {
                            "enabled": "true",
                            "api_key": persisted_canary,
                        }
                    },
                },
                sort_keys=True,
            ).encode("utf-8")
            config_path.write_bytes(original)
            calls = (
                ("qiongli_config_status", {"cwd": str(root)}),
                (
                    "qiongli_save_provider_config",
                    {
                        "provider": "openalex",
                        "field": "api_key",
                        "value": save_canary,
                    },
                ),
                ("qiongli_literature_status", {"cwd": str(root)}),
                (
                    "qiongli_search_plan",
                    {"cwd": str(root), "query": "governance"},
                ),
            )
            with mock.patch.dict(
                os.environ,
                {"QIONGLI_CONFIG_HOME": str(config_home)},
                clear=True,
            ):
                responses = [
                    call_qiongli_tool(tool, arguments)
                    for tool, arguments in calls
                ]
            persisted_after = config_path.read_bytes()

        self.assertEqual(persisted_after, original)
        for response in responses:
            with self.subTest(tool=response["structuredContent"]["tool"]):
                rendered = json.dumps(response, sort_keys=True)
                self.assertTrue(response["isError"])
                self.assertEqual(
                    response["structuredContent"]["error_kind"],
                    "tool_error",
                )
                self.assertEqual(
                    response["structuredContent"]["message"],
                    "provider configuration could not be read safely",
                )
                self.assertNotIn(persisted_canary, rendered)
                self.assertNotIn(save_canary, rendered)
                self.assertNotIn(str(config_home), rendered)
                self.assertNotIn("providers.json", rendered)

    def test_normalized_config_collisions_fail_closed_across_tools(self) -> None:
        persisted_canary = "QIONGLI_PROVIDER_COLLISION_CANARY"
        save_canary = "QIONGLI_COLLISION_SAVE_CANARY"
        collision_payloads = {
            "provider": {
                "version": 1,
                "providers": {
                    "semantic-scholar": {"api-key": persisted_canary},
                    "semantic_scholar": {"api_key": "second-key"},
                },
            },
            "field": {
                "version": 1,
                "providers": {
                    "semantic_scholar": {
                        "api-key": persisted_canary,
                        "api_key": "second-key",
                    }
                },
            },
        }
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            config_home = root / "private-config-location"
            config_home.mkdir()
            config_path = config_home / "providers.json"
            calls = (
                ("qiongli_config_status", {"cwd": str(root)}),
                (
                    "qiongli_save_provider_config",
                    {
                        "provider": "semantic_scholar",
                        "field": "api_key",
                        "value": save_canary,
                    },
                ),
                ("qiongli_literature_status", {"cwd": str(root)}),
                (
                    "qiongli_search_plan",
                    {"cwd": str(root), "query": "governance"},
                ),
            )
            with mock.patch.dict(
                os.environ,
                {"QIONGLI_CONFIG_HOME": str(config_home)},
                clear=True,
            ):
                for collision, payload in collision_payloads.items():
                    original = json.dumps(payload, sort_keys=True).encode("utf-8")
                    config_path.write_bytes(original)
                    responses = [
                        call_qiongli_tool(tool, arguments)
                        for tool, arguments in calls
                    ]
                    self.assertEqual(config_path.read_bytes(), original)
                    for response in responses:
                        with self.subTest(
                            collision=collision,
                            tool=response["structuredContent"]["tool"],
                        ):
                            rendered = json.dumps(response, sort_keys=True)
                            self.assertTrue(response["isError"])
                            self.assertEqual(
                                response["structuredContent"]["error_kind"],
                                "tool_error",
                            )
                            self.assertEqual(
                                response["structuredContent"]["message"],
                                "provider configuration could not be read safely",
                            )
                            self.assertNotIn(persisted_canary, rendered)
                            self.assertNotIn(save_canary, rendered)
                            self.assertNotIn(str(config_home), rendered)
                            self.assertNotIn("providers.json", rendered)

    def test_literature_search_returns_search_plan_diagnostics_and_results(self) -> None:
        fake_result = {
            "status": "ok",
            "summary": "Found 1 unique papers across 1 query attempts (1 raw hits, 0 deduplicated).",
            "provenance": ["mock-provider"],
            "data": {
                "provider_mode": "provider_translations",
                "query_plan": {"search_mode": "targeted_search", "legacy_query_variants": []},
                "provider_summaries": {"semantic_scholar": {"status": "ok", "normalized_hits": 1}},
                "search_diagnostics": {"gate_status": "pass", "blocking_reasons": []},
                "search_results": [{"title": "A Test Paper", "year": 2025, "providers": ["semantic_scholar"]}],
                "dedup_log": [],
                "search_log": [],
            },
        }

        with mock.patch(
            "bridges.literature_mcp_tools.run_literature_search",
            return_value=fake_result,
        ) as search:
            result = call_qiongli_tool(
                "qiongli_literature_search",
                {"query": "AI feedback in education", "limit": 5, "search_mode": "topic"},
            )

        payload = result["structuredContent"]
        self.assertFalse(result["isError"])
        self.assertEqual(payload["status"], "ok")
        self.assertEqual(payload["data"]["search_results"][0]["title"], "A Test Paper")
        search.assert_called_once()

    def test_literature_search_uses_configured_provider_clients(self) -> None:
        provider_calls: list[tuple[str, dict[str, object], int]] = []

        def openalex_search(
            translation: dict[str, object],
            limit: int,
            *,
            api_key: str,
            email: str,
        ) -> dict[str, object]:
            self.assertEqual(api_key, "openalex-secret-key")
            self.assertEqual(email, "maintainer@example.com")
            provider_calls.append(("openalex", translation, limit))
            return {
                "data": [
                    {
                        "paperId": "openalex-1",
                        "title": "OpenAlex Provider Paper",
                        "year": 2024,
                    }
                ]
            }

        def crossref_search(
            translation: dict[str, object],
            limit: int,
            *,
            email: str,
        ) -> dict[str, object]:
            self.assertEqual(email, "maintainer@example.com")
            provider_calls.append(("crossref", translation, limit))
            return {"data": []}

        def pubmed_search(
            translation: dict[str, object],
            limit: int,
            *,
            api_key: str,
        ) -> dict[str, object]:
            self.assertEqual(api_key, "pubmed-secret-key")
            provider_calls.append(("pubmed", translation, limit))
            return {"data": []}

        def arxiv_search(translation: dict[str, object], limit: int) -> dict[str, object]:
            provider_calls.append(("arxiv", translation, limit))
            return {"data": []}

        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            with mock.patch.dict("os.environ", {"QIONGLI_CONFIG_HOME": str(root / "config")}, clear=True):
                set_provider_value("openalex", "api-key", "openalex-secret-key")
                set_provider_value("openalex", "email", "maintainer@example.com")
                set_provider_value("crossref", "email", "maintainer@example.com")
                set_provider_value("pubmed", "api-key", "pubmed-secret-key")
                with (
                    mock.patch("bridges.literature_mcp_tools.openalex_client.search", openalex_search),
                    mock.patch("bridges.literature_mcp_tools.crossref_client.search", crossref_search),
                    mock.patch("bridges.literature_mcp_tools.pubmed_client.search", pubmed_search),
                    mock.patch("bridges.literature_mcp_tools.arxiv_client.search", arxiv_search),
                ):
                    result = call_qiongli_tool(
                        "qiongli_literature_search",
                        {
                            "query": "AI feedback in education",
                            "fromYear": 2020,
                            "per_provider_limit": 4,
                        },
                    )

        payload = result["structuredContent"]
        self.assertFalse(result["isError"])
        self.assertEqual(payload["data"]["provider_mode"], "provider_translations")
        self.assertEqual(
            [call[0] for call in provider_calls],
            ["openalex", "crossref", "pubmed", "arxiv"],
        )
        self.assertTrue(all(call[2] == 4 for call in provider_calls))
        self.assertEqual(
            payload["data"]["search_diagnostics"]["attempted_providers"],
            ["openalex", "crossref", "pubmed", "arxiv"],
        )
        self.assertEqual(payload["data"]["search_results"][0]["source"], "openalex")
        self.assertEqual(provider_calls[0][1]["filters"]["year_start"], "2020")

    def test_literature_search_with_all_providers_disabled_never_uses_legacy_network_fallback(
        self,
    ) -> None:
        disabled_config = {
            "providers": {
                provider: {"enabled": False, "configured": True}
                for provider in ("openalex", "semantic_scholar", "crossref", "pubmed", "arxiv")
            }
        }

        with (
            mock.patch(
                "bridges.literature_mcp_tools.resolve_provider_config",
                return_value=disabled_config,
            ),
            mock.patch("bridges.literature_mcp_tools.search_paper") as legacy_search,
        ):
            result = call_qiongli_tool(
                "qiongli_literature_search",
                {"query": "provider opt-out must remain offline"},
            )

        payload = result["structuredContent"]
        self.assertFalse(result["isError"])
        self.assertEqual(payload["status"], "warning")
        self.assertEqual(payload["data"]["provider_mode"], "strategy_only")
        self.assertEqual(
            payload["data"]["search_diagnostics"]["attempted_providers"],
            [],
        )
        self.assertEqual(
            payload["data"]["search_diagnostics"]["status_reason"],
            "no_active_providers",
        )
        self.assertIn("no active literature providers", payload["summary"].lower())
        legacy_search.assert_not_called()

    def test_literature_search_binds_credentials_from_the_requested_cwd(self) -> None:
        target_key = "target-project-s2-key"
        wrong_key = "process-cwd-s2-key"
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            target_project = root / "target-project"
            process_project = root / "process-project"
            config_home = root / "config"
            target_project.mkdir()
            process_project.mkdir()
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
            (target_project / ".env").write_text(
                f"QIONGLI_SEMANTIC_SCHOLAR_API_KEY={target_key}\n",
                encoding="utf-8",
            )
            (process_project / ".env").write_text(
                f"QIONGLI_SEMANTIC_SCHOLAR_API_KEY={wrong_key}\n",
                encoding="utf-8",
            )
            original_cwd = Path.cwd()
            try:
                os.chdir(process_project)
                with (
                    mock.patch.dict(
                        os.environ,
                        {"QIONGLI_CONFIG_HOME": str(config_home)},
                        clear=True,
                    ),
                    mock.patch(
                        "bridges.literature_mcp_tools.search_paper",
                        return_value={"data": []},
                    ) as semantic_search,
                ):
                    result = call_qiongli_tool(
                        "qiongli_literature_search",
                        {"cwd": str(target_project), "query": "cwd-isolated credentials"},
                    )
            finally:
                os.chdir(original_cwd)

        self.assertFalse(result["isError"])
        semantic_search.assert_called_once()
        self.assertEqual(semantic_search.call_args.kwargs["api_key"], target_key)
        self.assertNotEqual(semantic_search.call_args.kwargs["api_key"], wrong_key)

    def test_literature_search_review_mode_defaults_to_fifty_per_provider(self) -> None:
        provider_calls: list[int] = []

        def openalex_search(
            translation: dict[str, object],
            limit: int,
            *,
            api_key: str,
            email: str,
        ) -> dict[str, object]:
            self.assertEqual(api_key, "")
            self.assertEqual(email, "")
            provider_calls.append(limit)
            return {
                "data": [
                    {
                        "paperId": "openalex-review-1",
                        "title": "OpenAlex Review Paper",
                        "year": 2024,
                    }
                ]
            }

        config = {
            "providers": {
                "openalex": {"enabled": True, "configured": True},
            },
        }

        with (
            mock.patch("bridges.literature_mcp_tools.resolve_provider_config", return_value=config),
            mock.patch("bridges.literature_mcp_tools.openalex_client.search", openalex_search),
        ):
            result = call_qiongli_tool(
                "qiongli_literature_search",
                {"query": "AI feedback systematic review", "search_mode": "review"},
            )

        payload = result["structuredContent"]
        self.assertFalse(result["isError"])
        self.assertEqual(provider_calls, [50])
        self.assertEqual(payload["data"]["per_query_limit"], 50)

    def test_literature_export_evidence_wraps_supplied_snapshot(self) -> None:
        result = call_qiongli_tool(
            "qiongli_literature_export_evidence",
            {
                "query": "AI feedback",
                "provider_status": {"openalex": "configured"},
                "results": [{"title": "A Test Paper"}],
            },
        )

        payload = result["structuredContent"]
        self.assertFalse(result["isError"])
        self.assertEqual(payload["artifact_type"], "qiongli_literature_evidence_snapshot")
        self.assertEqual(payload["query"], "AI feedback")
        self.assertEqual(payload["result_count"], 1)


if __name__ == "__main__":
    unittest.main()
