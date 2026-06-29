from __future__ import annotations

import unittest

from bridges.hybrid_search_router import build_hybrid_search_plan


class HybridSearchRouterTests(unittest.TestCase):
    def test_hybrid_plan_uses_provider_and_native_queries(self) -> None:
        plan = build_hybrid_search_plan(
            {
                "query": "AI feedback in education",
                "platform": "codex",
                "native_search_available": True,
                "native_search_tools": ["codex_web_search"],
                "include_working_papers": True,
                "fromYear": 2020,
                "toYear": 2025,
                "search_mode": "systematic_review",
                "venue_filter": "learning analytics",
                "document_types": ["journal-article", "conference-paper"],
            },
            provider_capability_mode="provider_connected",
        )

        self.assertEqual(plan["artifact_type"], "qiongli_hybrid_search_plan")
        self.assertEqual(plan["query"], "AI feedback in education")
        self.assertEqual(plan["platform"], "codex")
        self.assertEqual(plan["search_execution_mode"], "hybrid_search")
        self.assertEqual(plan["provider_capability_mode"], "provider_connected")
        self.assertTrue(plan["native_search_available"])
        self.assertEqual(plan["native_search_tools"], ["codex_web_search"])
        self.assertEqual(
            plan["provenance_labels"]["provider"],
            [
                "mcp:semantic_scholar",
                "mcp:openalex",
                "mcp:crossref",
                "mcp:pubmed",
                "mcp:arxiv",
            ],
        )
        self.assertEqual(plan["provenance_labels"]["native"], ["native:codex_web_search"])
        self.assertEqual(plan["provenance_labels"]["user_corpus"], ["user_corpus"])
        self.assertEqual({query["provider"] for query in plan["provider_queries"]}, {"semantic_scholar", "openalex", "crossref", "pubmed", "arxiv"})
        self.assertEqual(plan["native_search_queries"][0]["tool"], "codex_web_search")
        self.assertEqual(plan["provider_queries"][0]["filters"]["fromYear"], 2020)
        self.assertTrue(plan["provider_queries"][0]["filters"]["include_working_papers"])
        self.assertEqual(
            [step["action"] for step in plan["execution_sequence"]],
            [
                "call qiongli_literature_status",
                "call qiongli_search_plan",
                "call qiongli_literature_search",
                "execute platform-native search",
                "merge/dedupe/search_log",
            ],
        )
        self.assertTrue(all(step["actor"] == "agent" for step in plan["execution_sequence"]))
        self.assertIn(
            "MCP servers must not call Codex or Claude native search directly.",
            plan["agent_instructions"],
        )
        self.assertIn(
            "The active agent executes native_search_queries only when the platform exposes native search.",
            plan["agent_instructions"],
        )
        self.assertIn(
            "Do not treat native-search results as provider-reproducible records.",
            plan["agent_instructions"],
        )
        self.assertIn(
            "Write provider, native, and user-corpus records with distinct provenance labels.",
            plan["agent_instructions"],
        )

    def test_provider_connected_without_native_search_uses_provider_only_mode(self) -> None:
        plan = build_hybrid_search_plan(
            {"query": "AI feedback", "platform": "codex", "native_search_available": False},
            provider_capability_mode="provider_connected",
        )

        self.assertEqual(plan["search_execution_mode"], "provider_connected")
        self.assertEqual(plan["native_search_queries"], [])
        self.assertIn(
            "Platform-native search was not declared available.",
            plan["limitations"],
        )
        self.assertIn("call qiongli_literature_search", [step["action"] for step in plan["execution_sequence"]])
        self.assertNotIn("execute platform-native search", [step["action"] for step in plan["execution_sequence"]])

    def test_strategy_only_with_native_search_uses_native_only_mode(self) -> None:
        plan = build_hybrid_search_plan(
            {"query": "AI feedback", "platform": "claude_code", "native_search_available": True},
            provider_capability_mode="strategy_only",
        )

        self.assertEqual(plan["search_execution_mode"], "native_only")
        self.assertEqual(plan["native_search_tools"], ["claude_web_search"])
        self.assertEqual(plan["provider_queries"], [])
        self.assertEqual(plan["provenance_labels"]["native"], ["native:claude_web_search"])
        self.assertIn(
            "Provider MCP search is unavailable; native results require explicit provenance labels.",
            plan["limitations"],
        )
        self.assertIn("execute platform-native search", [step["action"] for step in plan["execution_sequence"]])
        self.assertNotIn("call qiongli_literature_search", [step["action"] for step in plan["execution_sequence"]])

    def test_unknown_provider_capability_mode_normalizes_to_strategy_only(self) -> None:
        plan = build_hybrid_search_plan(
            {"query": "AI feedback", "platform": "codex", "native_search_available": True},
            provider_capability_mode="provider_missing",
        )

        self.assertEqual(plan["provider_capability_mode"], "strategy_only")
        self.assertEqual(plan["search_execution_mode"], "native_only")
        self.assertEqual(plan["provider_queries"], [])
        self.assertEqual(plan["native_search_tools"], ["codex_web_search"])

    def test_strategy_only_without_native_search_returns_strategy_only_mode(self) -> None:
        plan = build_hybrid_search_plan(
            {"query": "AI feedback", "platform": "unknown", "native_search_available": False},
            provider_capability_mode="strategy_only",
        )

        self.assertEqual(plan["search_execution_mode"], "strategy_only")
        self.assertEqual(plan["provider_queries"], [])
        self.assertEqual(plan["native_search_queries"], [])
        self.assertIn(
            "No provider MCP search or platform-native search is available.",
            plan["limitations"],
        )
        self.assertEqual(
            [step["action"] for step in plan["execution_sequence"]],
            [
                "call qiongli_literature_status",
                "call qiongli_search_plan",
                "merge/dedupe/search_log",
            ],
        )

    def test_empty_query_returns_strategy_only_mode(self) -> None:
        plan = build_hybrid_search_plan(
            {"query": "   ", "platform": "antigravity", "native_search_available": True},
            provider_capability_mode="provider_connected",
        )

        self.assertEqual(plan["query"], "")
        self.assertEqual(plan["search_execution_mode"], "strategy_only")
        self.assertEqual(plan["provider_queries"], [])
        self.assertEqual(plan["native_search_queries"], [])
        self.assertIn("Search query is empty.", plan["limitations"])

    def test_native_search_defaults_to_platform_tool_when_available(self) -> None:
        cases = (
            ("codex", "codex_web_search"),
            ("claude", "claude_web_search"),
            ("claude_code", "claude_web_search"),
            ("antigravity", "antigravity_search"),
            ("other", "platform_native_search"),
        )

        for platform, expected_tool in cases:
            with self.subTest(platform=platform):
                plan = build_hybrid_search_plan(
                    {"query": "AI feedback", "platform": platform, "native_search_available": True},
                    provider_capability_mode="strategy_only",
                )

                self.assertEqual(plan["native_search_tools"], [expected_tool])
                self.assertEqual(plan["provenance_labels"]["native"], [f"native:{expected_tool}"])

    def test_provider_status_limits_provider_queries_to_configured_providers(self) -> None:
        plan = build_hybrid_search_plan(
            {
                "query": "AI feedback in education",
                "platform": "codex",
                "native_search_available": True,
            },
            provider_capability_mode="provider_connected",
            provider_status={
                "openalex": "configured",
                "semantic_scholar": "missing",
                "crossref": "missing",
                "pubmed": "missing",
                "arxiv": "configured",
            },
        )

        self.assertEqual(plan["search_execution_mode"], "hybrid_search")
        self.assertEqual(
            [query["provider"] for query in plan["provider_queries"]],
            ["openalex", "arxiv"],
        )
        self.assertEqual(plan["provenance_labels"]["provider"], ["mcp:openalex", "mcp:arxiv"])

    def test_camel_case_aliases_and_query_variants_match_cross_platform_payloads(self) -> None:
        plan = build_hybrid_search_plan(
            {
                "query": "AI feedback in education",
                "queryVariants": ["algorithmic feedback in classrooms"],
                "platform": "claude-code",
                "nativeSearchAvailable": True,
                "includeWorkingPapers": True,
                "searchMode": "review",
                "venueFilter": "learning analytics",
                "documentTypes": ["journal-article"],
            },
            provider_capability_mode="strategy_only",
        )

        self.assertEqual(plan["search_execution_mode"], "native_only")
        self.assertEqual(plan["native_search_tools"], ["claude_web_search"])
        self.assertEqual([query["query_id"] for query in plan["native_search_queries"]], ["Q1", "Q2"])
        self.assertEqual(
            [query["query"] for query in plan["native_search_queries"]],
            ["AI feedback in education", "algorithmic feedback in classrooms"],
        )
        filters = plan["native_search_queries"][0]["filters"]
        self.assertTrue(filters["include_working_papers"])
        self.assertEqual(filters["search_mode"], "review")
        self.assertEqual(filters["venue_filter"], "learning analytics")
        self.assertEqual(filters["document_types"], ["journal-article"])


if __name__ == "__main__":
    unittest.main()
