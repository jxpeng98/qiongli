from __future__ import annotations

import json
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]


class MCPProviderDocsTests(unittest.TestCase):
    def test_native_provider_docs_cover_the_shared_provider_schema(self) -> None:
        schema = json.loads((REPO_ROOT / "content/mcp-contracts/provider-config.schema.json").read_text())
        providers = schema["properties"]["providers"]["properties"]
        for language in ("", "zh/"):
            content = (REPO_ROOT / f"docs/{language}advanced/mcp-providers-setup.md").read_text()
            with self.subTest(language=language):
                for provider in providers:
                    self.assertIn(f"`{provider}`", content)
                for tool in ("qiongli_configure_provider", "qiongli_config_status", "qiongli_literature_status"):
                    self.assertIn(tool, content)
                self.assertNotIn("RESEARCH_MCP_METADATA_REGISTRY_ENRICH_CMD", content)

    def test_rigorous_search_docs_define_non_exhaustive_coverage_metrics(self) -> None:
        content = (REPO_ROOT / "docs" / "advanced" / "rigorous-literature-search.md").read_text(
            encoding="utf-8"
        )

        for phrase in (
            "No provider can prove absolute completeness",
            "known-item recall",
            "duplicate saturation",
            "full-text access coverage",
            "native_fulltext_queries",
            "Zotero attachment verification",
            "evidence_limit",
        ):
            self.assertIn(phrase, content)

    def test_cross_platform_mcp_docs_include_literature_search_plan(self) -> None:
        content = (REPO_ROOT / "docs" / "advanced" / "cross-platform-mcp.md").read_text(
            encoding="utf-8"
        )

        for token in (
            "qiongli_search_plan",
            "hybrid_search",
            "provider_connected",
            "native_only",
            "strategy_only",
            "provider_capability_mode",
            "MCP servers must not call Codex or Claude native search directly",
            "user_corpus",
        ):
            self.assertIn(token, " ".join(content.split()))

    def test_cli_reference_lists_literature_status_search_plan_and_search(self) -> None:
        content = (REPO_ROOT / "docs" / "reference" / "cli.md").read_text(encoding="utf-8")

        for tool in (
            "qiongli_literature_status",
            "qiongli_search_plan",
            "qiongli_literature_search",
            "qiongli_literature_export_evidence",
        ):
            self.assertIn(tool, content)

    def test_cross_platform_docs_use_native_stdio_and_separate_connection_checks(self) -> None:
        content = (REPO_ROOT / "docs/advanced/cross-platform-mcp.md").read_text()
        for token in ("mcp serve --profile full --transport stdio", "qiongli mcp check --profile full",
                      "qiongli_config_status", "qiongli_literature_status"):
            self.assertIn(token, content)
        self.assertNotIn("--transport http", content)
        self.assertNotIn("Python-backed", content)

    def test_provider_setup_docs_explain_hybrid_search_router(self) -> None:
        docs = {
            "docs/advanced/mcp-providers-setup.md": (
                REPO_ROOT / "docs" / "advanced" / "mcp-providers-setup.md"
            ).read_text(encoding="utf-8"),
            "docs/zh/advanced/mcp-providers-setup.md": (
                REPO_ROOT / "docs" / "zh" / "advanced" / "mcp-providers-setup.md"
            ).read_text(encoding="utf-8"),
        }

        for label, content in docs.items():
            with self.subTest(label=label):
                for token in (
                    "qiongli_search_plan",
                    "hybrid_search",
                    "provider_connected",
                    "native_only",
                    "strategy_only",
                    "provider_capability_mode",
                    "search_execution_mode",
                    "native_search_queries",
                ):
                    self.assertIn(token, content)

    def test_install_docs_document_desktop_provider_boundary(self) -> None:
        docs = {
            "docs/guide/install.md": (REPO_ROOT / "docs" / "guide" / "install.md").read_text(
                encoding="utf-8"
            ),
            "docs/zh/guide/install.md": (
                REPO_ROOT / "docs" / "zh" / "guide" / "install.md"
            ).read_text(encoding="utf-8"),
        }

        for label, content in docs.items():
            with self.subTest(label=label):
                for token in (
                    "`qiongli provider setup`",
                    ".mcpb",
                    "qiongli-literature-provider",
                    "qiongli_config_status",
                    "qiongli_configure_provider",
                    "qiongli_save_provider_config",
                    "Codex",
                    "OpenAlex",
                    "Semantic Scholar",
                    "provider_connected",
                    "strategy_only",
                    "180",
                    "skill-only",
                ):
                    self.assertIn(token, content)

                for forbidden in (
                    "qiongli " + "companion",
                    "companion " + "setup",
                    "companion " + "doctor",
                    "export" + "-status",
                ):
                    self.assertNotIn(forbidden, content)

    def test_native_zotero_docs_use_current_write_arguments(self) -> None:
        for language in ("", "zh/"):
            content = (REPO_ROOT / f"docs/{language}advanced/mcp-zotero-integration.md").read_text()
            with self.subTest(language=language):
                for token in ("`items`", "`records`", "dry_run_receipt", 'write_intent: "apply"',
                              "qiongli_zotero_search", "QIONGLI_ZOTERO_CONNECTOR_URL"):
                    self.assertIn(token, content)
                self.assertNotIn("QIONGLI_ZOTERO_LOCAL_ENABLED", content)
                self.assertNotIn('"include_zotero": true', content)

    def test_qiongli_workflow_requires_codex_literature_status_preflight(self) -> None:
        content = (REPO_ROOT / "content" / "workflow" / "SKILL.md").read_text(
            encoding="utf-8"
        )
        self.assertIn("qiongli_literature_status", content)
        self.assertIn("Codex", content)
        self.assertIn("before declaring `strategy_only`", content)
        self.assertIn("provider_connected", content)


if __name__ == "__main__":
    unittest.main()
