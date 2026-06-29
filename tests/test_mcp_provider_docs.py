from __future__ import annotations

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]


class MCPProviderDocsTests(unittest.TestCase):
    def test_english_provider_setup_includes_capability_matrix_and_modes(self) -> None:
        content = (REPO_ROOT / "docs" / "advanced" / "mcp-providers-setup.md").read_text(
            encoding="utf-8"
        )

        for token in (
            "## MCP Capability Matrix",
            "## Quick Decision Rules",
            "Full external override",
            "Builtin baseline + external overlay",
            "`RESEARCH_MCP_METADATA_REGISTRY_ENRICH_CMD`",
            "`RESEARCH_MCP_FULLTEXT_RETRIEVAL_RESOLVE_CMD`",
            "| `screening-tracker` | Yes, checkpoint stub only. Reads local screening artifacts and resume state. |",
            "| `reporting-guidelines` | No builtin MCP, but strong skill-level fallback via `reporting-checker`. |",
            "| `submission-kit` | No builtin MCP, but strong skill-level fallback via `submission-packager`. |",
        ):
            self.assertIn(token, content)

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
            "native:codex_web_search",
            "native:claude_web_search",
            "mcp:openalex",
            "mcp:semantic_scholar",
            "user_corpus",
        ):
            self.assertIn(token, content)

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
                    "Hybrid Search Router",
                    "qiongli_search_plan",
                    "hybrid_search",
                    "provider_connected",
                    "native_only",
                    "strategy_only",
                    "provider_capability_mode",
                    "MCP servers",
                    "native search",
                    "active agent",
                    "native_search_queries",
                ):
                    self.assertIn(token, content)

    def test_install_docs_document_desktop_provider_boundary(self) -> None:
        docs = {
            "README.md": (REPO_ROOT / "README.md").read_text(encoding="utf-8"),
            "README_CN.md": (REPO_ROOT / "README_CN.md").read_text(encoding="utf-8"),
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

    def test_chinese_provider_setup_includes_capability_matrix_and_modes(self) -> None:
        content = (
            REPO_ROOT / "docs" / "zh" / "advanced" / "mcp-providers-setup.md"
        ).read_text(encoding="utf-8")

        for token in (
            "## MCP 能力矩阵",
            "## 快速决策规则",
            "完整外部替换",
            "builtin baseline + 外部 overlay",
            "`RESEARCH_MCP_METADATA_REGISTRY_ENRICH_CMD`",
            "`RESEARCH_MCP_FULLTEXT_RETRIEVAL_RESOLVE_CMD`",
            "| `screening-tracker` | 有，但只提供 checkpoint stub。读取本地 screening artifacts 和 resume state。 |",
            "| `reporting-guidelines` | 没有 builtin MCP，但 `reporting-checker` skill 已经提供强 fallback。 |",
            "| `submission-kit` | 没有 builtin MCP，但 `submission-packager` skill 已经提供强 fallback。 |",
        ):
            self.assertIn(token, content)

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
