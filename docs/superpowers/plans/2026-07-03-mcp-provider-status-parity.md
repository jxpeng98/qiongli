# MCP Provider Status Parity Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make Qiongli MCP provider status, literature search, and external evidence adapter semantics unambiguous across Python full MCP, Node literature MCPB, generated workflow guidance, docs, and tests.

**Architecture:** Keep `qiongli_literature_status`, `qiongli_search_plan`, and `qiongli_literature_search` as the canonical literature-provider path backed by shared provider config. Keep `qiongli_collect_evidence` as a backward-compatible filesystem/builtin/external-command evidence adapter, but make direct academic-provider calls return a precise external-command diagnostic that cannot be mistaken for provider-config status. Treat provider discovery, platform-native full-text candidate search, Zotero attachment verification, and retrieval-manifest evidence limits as separate layers. Add parity tests so Python full MCP, CLI doctor/config output, Node MCPB manifests, Zotero companion output, and docs cannot drift again.

**Tech Stack:** Python 3.12 MCP bridge modules, Node ESM MCPB manifest/server files, unittest, node:test, Markdown docs, Qiongli generated workflow materialization.

---

## Current Audit Findings

Severity labels use release-impact priority.

- **P1: `qiongli_collect_evidence` reports `openalex not_configured` for the external command adapter, not the OpenAlex API provider config.**
  - Evidence: `packages/python-qiongli/src/qiongli/bridges/mcp_tool_handlers.py` delegates `qiongli_collect_evidence` to `MCPConnector().collect(...)`; `packages/python-qiongli/src/qiongli/bridges/mcp_connectors.py` returns `External MCP not configured. Set RESEARCH_MCP_OPENALEX_CMD.` when no external command is configured.
  - Impact: A user can have OpenAlex configured in `~/.config/qiongli/providers.json` and successful `qiongli_literature_search`, while `qiongli_collect_evidence` still says `not_configured`.
  - Fix: Preserve adapter behavior, but report `not_configured_scope: external_command_adapter`, include the redacted provider-config status, and point callers to `qiongli_literature_status` / `qiongli_literature_search`.

- **P1: Python full MCP `literature_tools` output omits `qiongli_search_plan`.**
  - Evidence: `packages/python-qiongli/src/qiongli/bridges/mcp_tool_handlers.py` exposes `qiongli_search_plan`, but `packages/python-qiongli/src/qiongli/bridges/mcp_cli.py` lists only status/search/export in `LITERATURE_TOOLS`.
  - Impact: `qiongli mcp doctor --json` and `qiongli mcp config example --json` under-report the hybrid routing tool.
  - Fix: Add `qiongli_search_plan` to `LITERATURE_TOOLS` immediately after `qiongli_literature_status`.

- **P1: Install docs incorrectly imply `qiongli_literature_search` requires the full Python runtime in some paths.**
  - Evidence: `docs/guide/install.md` and `docs/zh/guide/install.md` state the bundled Claude Code plugin covers provider/search/status, then include `qiongli_literature_search` in the full-runtime-only list.
  - Impact: Users may install full runtime unnecessarily or misdiagnose bundled MCPB/plugin-lite as unable to search.
  - Fix: State that bundled Node literature MCP and full CLI MCP both expose `qiongli_literature_search`; full runtime is needed for Python-backed orchestrator/task tools.

- **P2: Full MCP docs mix provider-config readiness and external command evidence adapters.**
  - Evidence: `docs/advanced/cross-platform-mcp.md` describes `qiongli_collect_evidence` alongside provider config/status tools for "MCP/provider readiness".
  - Impact: Encourages exactly the false inference that caused the OpenAlex misdiagnosis.
  - Fix: Split the docs into "provider config/status/search tools" and "external evidence adapter tools".

- **P2: Source workflow guidance is stale relative to generated installed guidance.**
  - Evidence: `content/workflow/SKILL.md` says Desktop/Web MCPB is for OpenAlex and Semantic Scholar, while `packages/python-qiongli/src/qiongli/subject_materializer.py` and the installed skill mention OpenAlex, Semantic Scholar, Crossref, PubMed, and arXiv.
  - Impact: Re-materializing from source can regenerate older provider guidance.
  - Fix: Update source workflow guidance and materializer text in the same change.

- **P3: No release-boundary issue found.**
  - Evidence: The inconsistency is inside source, generated text, docs, and tests. No marketplace catalog files, secrets, absolute local paths, or third-party bundles were introduced by this fix scope.
  - Fix: Keep changes scoped to Qiongli MCP source/docs/tests and generated guidance text.

- **P1: Search-result defaults are split across Python full MCP and Node MCPB.**
  - Evidence: Node MCPB uses normal-search default 25 and review default 50, but Python `run_scholarly_search` uses `DEFAULT_PER_QUERY_LIMIT = 20`, and individual provider fallback defaults in Node provider modules are still 10.
  - Impact: Users can see different result counts depending on whether they call Python full MCP, Node MCPB, direct provider helpers, or old calls that omit `per_provider_limit`.
  - Fix: Standardize normal topic search default to 25 per provider/query, review/systematic-review default to 50, and keep explicit `per_provider_limit` / `search_depth` as the way to go higher.

- **P1: Literature discovery does not prove full-text availability.**
  - Evidence: OpenAlex, Semantic Scholar, Crossref, PubMed, and arXiv search mostly return metadata, abstracts, identifiers, links, and occasional OA/PDF hints. Qiongli already has `retrieval_manifest.csv`, `fulltext-fetcher`, and `fulltext-retrieval`, but Node MCPB normalization currently does not preserve full-text/OA candidate fields consistently.
  - Impact: A search can be broad in metadata coverage but weak for claims that require full-text reading. Without retrieval diagnostics, "coverage" and "readable evidence" are conflated.
  - Fix: Preserve OA/full-text candidate URLs in search results, require retrieval status tracking for review-grade claims, and report coverage as separate discovery, dedup, retrieval, and evidence-limit metrics.

- **P1: Platform-native LLM search and Zotero attachment evidence are not modeled as separate full-text candidate/verification layers.**
  - Evidence: `qiongli_search_plan` already tells the active agent to execute `native_search_queries`, but it does not emit dedicated full-text candidate queries or an expected candidate payload. `packages/qiongli-zotero-companion/chrome/content/qiongli-bridge.js` and `packages/qiongli-zotero-companion/bootstrap.js` return compact item metadata without attachment summaries, so MCPB cannot tell whether Zotero has a local PDF or only a citation record.
  - Impact: Native LLM search can find PDF/PMC/arXiv/author-page full-text candidates, but those results are not auditable. Zotero can be the strongest local truth source, but Qiongli currently cannot reliably distinguish "reference exists in Zotero" from "verified full-text attachment exists in Zotero".
  - Fix: Add `native_fulltext_queries` and candidate schema guidance to search plans, keep active agents responsible for executing platform search, preserve candidate provenance as `candidate_only`, and upgrade the Zotero companion/MCPB normalization path to expose attachment-level verification with match basis and confidence.

## File Structure

- Modify: `packages/python-qiongli/src/qiongli/bridges/mcp_connectors.py`
  - Responsibility: external command adapter resolution and evidence diagnostics.

- Modify: `packages/python-qiongli/src/qiongli/bridges/mcp_tool_handlers.py`
  - Responsibility: full Python MCP tool schemas/descriptions and tool dispatch.

- Modify: `packages/python-qiongli/src/qiongli/bridges/mcp_cli.py`
  - Responsibility: `qiongli mcp doctor` and `qiongli mcp config example` tool inventory output.

- Modify: `tests/test_mcp_connectors.py`
  - Responsibility: connector behavior, including external-command status scope and provider-config diagnostics.

- Modify: `tests/test_mcp_tool_handlers.py`
  - Responsibility: Python full MCP schema/tool behavior regression tests.

- Modify: `tests/test_mcp_cli.py`
  - Responsibility: CLI doctor/config example output parity. Preserve the existing wizard payload-reader edits in this file.

- Create: `tests/test_mcp_tool_surface_parity.py`
  - Responsibility: cross-surface parity tests for common literature-provider tools across Python full MCP and Node MCPB manifest.

- Modify: `packages/python-qiongli/src/qiongli/bridges/providers/literature_search.py`
  - Responsibility: Python full MCP search defaults, review-depth limits, and search diagnostics.

- Modify: `tests/test_literature_search.py`
  - Responsibility: Python search default and limit regression tests.

- Modify: `packages/qiongli-literature-mcpb/server/providers/openalex.mjs`
- Modify: `packages/qiongli-literature-mcpb/server/providers/semantic-scholar.mjs`
- Modify: `packages/qiongli-literature-mcpb/server/providers/crossref.mjs`
- Modify: `packages/qiongli-literature-mcpb/server/providers/pubmed.mjs`
- Modify: `packages/qiongli-literature-mcpb/server/providers/arxiv.mjs`
  - Responsibility: Node provider fallback defaults when called directly without resolved search options.

- Modify: `packages/qiongli-literature-mcpb/server/normalize.mjs`
  - Responsibility: preserve OA/full-text candidate fields in normalized search results.

- Modify: `packages/qiongli-literature-mcpb/server/search-plan.mjs`
- Modify: `packages/python-qiongli/src/qiongli/bridges/hybrid_search_router.py`
  - Responsibility: emit platform-native full-text candidate search queries while keeping active agents, not MCP servers, responsible for executing native search.

- Modify: `packages/qiongli-literature-mcpb/server/providers/openalex.mjs`
- Modify: `packages/qiongli-literature-mcpb/server/providers/semantic-scholar.mjs`
- Modify: `packages/qiongli-literature-mcpb/server/providers/arxiv.mjs`
  - Responsibility: map provider OA/PDF hints into normalized fields.

- Modify: `packages/qiongli-literature-mcpb/server/zotero/search-source.mjs`
  - Responsibility: normalize Zotero attachment summaries into local full-text evidence hints and local match diagnostics.

- Modify: `packages/qiongli-zotero-companion/chrome/content/qiongli-bridge.js`
- Modify: `packages/qiongli-zotero-companion/bootstrap.js`
- Modify: `packages/qiongli-zotero-companion/README.md`
  - Responsibility: expose attachment-level metadata from Zotero safely, with raw local paths gated behind explicit request flags.

- Modify: `packages/qiongli-literature-mcpb/test/tools.test.mjs`
- Modify: `packages/qiongli-literature-mcpb/test/providers.test.mjs`
  - Responsibility: Node default limit and OA/full-text candidate preservation tests.

- Modify: `packages/qiongli-literature-mcpb/test/zotero.test.mjs`
- Modify: `packages/qiongli-zotero-companion/test/bridge.test.mjs`
- Modify: `tests/test_hybrid_search_router.py`
  - Responsibility: native full-text query planning, Zotero attachment exposure, and Zotero result normalization tests.

- Modify: `tests/test_literature_contract.py`
  - Responsibility: workflow guidance contract tests for status/search-plan routing and `qiongli_collect_evidence` non-authority.

- Modify: `tests/test_mcp_provider_docs.py` and `tests/test_cli_setup_docs.py`
  - Responsibility: docs contract tests that guard install and MCP-provider wording.

- Modify: `content/workflow/SKILL.md`
  - Responsibility: canonical workflow source guidance.

- Modify: `packages/python-qiongli/src/qiongli/subject_materializer.py`
  - Responsibility: generated workflow guidance text inserted into materialized subject packages.

- Modify: `README.md`, `README_CN.md`
  - Responsibility: top-level install/provider guidance.

- Modify: `docs/reference/cli.md`, `docs/zh/reference/cli.md`
  - Responsibility: CLI/MCP tool inventory.

- Modify: `docs/advanced/cross-platform-mcp.md`
  - Responsibility: full MCP vs bundled MCPB semantics and evidence-adapter boundary.

- Modify: `docs/guide/install.md`, `docs/zh/guide/install.md`
  - Responsibility: install-surface guidance.

- Modify: `docs/advanced/qiongli-cli-plugin-structure.html`
  - Responsibility: static advanced reference tool list if it remains manually maintained.

---

### Task 1: Clarify `qiongli_collect_evidence` Diagnostics

**Files:**
- Modify: `packages/python-qiongli/src/qiongli/bridges/mcp_connectors.py`
- Modify: `tests/test_mcp_connectors.py`

- [ ] **Step 1: Write the failing connector test**

Append this test to `tests/test_mcp_connectors.py` after `test_builtin_scholarly_search_reports_provider_config`:

```python
    def test_direct_literature_provider_without_external_command_reports_adapter_scope(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            config_home = root / "config"
            with mock.patch.dict(
                os.environ,
                {"QIONGLI_CONFIG_HOME": str(config_home)},
                clear=False,
            ):
                set_provider_value("openalex", "api-key", "openalex-key")
                evidence = self.connector.collect("openalex", {"topic": "demo"}, root)

        self.assertEqual(evidence.provider, "openalex")
        self.assertEqual(evidence.status, "not_configured")
        self.assertIn("external command adapter", evidence.summary)
        self.assertIn("RESEARCH_MCP_OPENALEX_CMD", evidence.provenance)
        self.assertEqual(evidence.data["not_configured_scope"], "external_command_adapter")
        self.assertEqual(evidence.data["provider_config_status"], "configured")
        self.assertEqual(evidence.data["provider_config"]["openalex"], "configured")
        self.assertEqual(evidence.data["recommended_status_tool"], "qiongli_literature_status")
        self.assertEqual(evidence.data["recommended_search_tool"], "qiongli_literature_search")
```

- [ ] **Step 2: Run the failing test**

Run:

```bash
uv run python -m unittest tests.test_mcp_connectors.MCPConnectorTests.test_direct_literature_provider_without_external_command_reports_adapter_scope -v
```

Expected: FAIL because the current `not_configured` response has no `data` scope or recommendation fields.

- [ ] **Step 3: Implement scoped diagnostics**

In `packages/python-qiongli/src/qiongli/bridges/mcp_connectors.py`, add this constant near `MCPProviderResolution`:

```python
DIRECT_LITERATURE_PROVIDER_KEYS = {
    "openalex",
    "semantic_scholar",
    "crossref",
    "pubmed",
    "arxiv",
}
```

In the `if not command:` block inside `_collect_external_command`, replace the returned `MCPEvidence(...)` with:

```python
            config_summary = provider_config_summary(resolve_provider_config(cwd=cwd))
            provider_key = self._provider_config_key(provider)
            data: dict[str, Any] = {
                "not_configured_scope": "external_command_adapter",
                "external_command_env": env_name,
                "provider_config": config_summary,
            }
            if provider_key in DIRECT_LITERATURE_PROVIDER_KEYS:
                data.update(
                    {
                        "provider_config_status": config_summary.get(provider_key, "missing"),
                        "recommended_status_tool": "qiongli_literature_status",
                        "recommended_search_tool": "qiongli_literature_search",
                    }
                )
                summary = (
                    f"External command adapter for {provider} is not configured. "
                    f"Set {env_name} only if you intend to use a separate external MCP command. "
                    "Use qiongli_literature_status or qiongli_literature_search to check "
                    "the built-in literature provider config."
                )
            else:
                summary = f"External command adapter not configured. Set {env_name}."
            return MCPEvidence(
                provider=provider,
                status="not_configured",
                summary=summary,
                provenance=[env_name],
                data=data,
            )
```

Add this helper method near `_provider_env_var`:

```python
    def _provider_config_key(self, provider: str) -> str:
        key = provider.strip().lower().replace("-", "_")
        if key == "semantic_scholar":
            return "semantic_scholar"
        return key
```

Update the import at the top of `mcp_connectors.py`:

```python
from bridges.provider_config import (
    provider_config_env,
    provider_config_summary,
    resolve_provider_config,
)
```

- [ ] **Step 4: Run the connector tests**

Run:

```bash
uv run python -m unittest tests.test_mcp_connectors.MCPConnectorTests.test_direct_literature_provider_without_external_command_reports_adapter_scope tests.test_mcp_connectors.MCPConnectorTests.test_builtin_scholarly_search_reports_provider_config -v
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add packages/python-qiongli/src/qiongli/bridges/mcp_connectors.py tests/test_mcp_connectors.py
git commit -m "fix(mcp): clarify external evidence adapter diagnostics"
```

### Task 2: Make `qiongli_collect_evidence` Tool Schema Unambiguous

**Files:**
- Modify: `packages/python-qiongli/src/qiongli/bridges/mcp_tool_handlers.py`
- Modify: `tests/test_mcp_tool_handlers.py`

- [ ] **Step 1: Write the failing schema and behavior tests**

Append these tests to `tests/test_mcp_tool_handlers.py` after `test_collect_evidence_tool_uses_existing_connector`:

```python
    def test_collect_evidence_description_names_external_command_boundary(self) -> None:
        definitions = {tool["name"]: tool for tool in MCP_TOOL_DEFINITIONS}
        description = definitions["qiongli_collect_evidence"]["description"]

        self.assertIn("external command adapters", description)
        self.assertIn("qiongli_literature_status", description)
        self.assertIn("qiongli_literature_search", description)

    def test_collect_evidence_openalex_not_configured_is_not_provider_config_status(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            with mock.patch.dict(
                "os.environ",
                {"QIONGLI_CONFIG_HOME": str(root / "config")},
                clear=False,
            ):
                set_provider_value("openalex", "api-key", "openalex-secret-key")
                result = call_qiongli_tool(
                    "qiongli_collect_evidence",
                    {
                        "cwd": str(root),
                        "provider": "openalex",
                        "task_packet": {"topic": "demo-topic"},
                    },
                )

        payload = result["structuredContent"]["evidence"]
        rendered = json.dumps(result, sort_keys=True)
        self.assertFalse(result["isError"])
        self.assertEqual(payload["status"], "not_configured")
        self.assertEqual(payload["data"]["not_configured_scope"], "external_command_adapter")
        self.assertEqual(payload["data"]["provider_config_status"], "configured")
        self.assertEqual(payload["data"]["recommended_status_tool"], "qiongli_literature_status")
        self.assertEqual(payload["data"]["recommended_search_tool"], "qiongli_literature_search")
        self.assertNotIn("openalex-secret-key", rendered)
```

- [ ] **Step 2: Run the failing tests**

Run:

```bash
uv run python -m unittest tests.test_mcp_tool_handlers.MCPToolHandlerTests.test_collect_evidence_description_names_external_command_boundary tests.test_mcp_tool_handlers.MCPToolHandlerTests.test_collect_evidence_openalex_not_configured_is_not_provider_config_status -v
```

Expected: first test FAILS until the description is updated; second test passes only after Task 1.

- [ ] **Step 3: Update the tool description**

In `packages/python-qiongli/src/qiongli/bridges/mcp_tool_handlers.py`, replace the `qiongli_collect_evidence` description with:

```python
        "description": (
            "Collect evidence from filesystem, built-in workflow adapters, or externally "
            "configured command adapters. Do not use this to judge built-in literature "
            "provider config; use qiongli_literature_status or qiongli_literature_search "
            "for OpenAlex, Semantic Scholar, Crossref, PubMed, and arXiv provider status/search."
        ),
```

- [ ] **Step 4: Run the MCP tool handler tests**

Run:

```bash
uv run python -m unittest tests.test_mcp_tool_handlers.MCPToolHandlerTests.test_tool_definitions_include_config_and_evidence_tools tests.test_mcp_tool_handlers.MCPToolHandlerTests.test_collect_evidence_description_names_external_command_boundary tests.test_mcp_tool_handlers.MCPToolHandlerTests.test_collect_evidence_openalex_not_configured_is_not_provider_config_status -v
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add packages/python-qiongli/src/qiongli/bridges/mcp_tool_handlers.py tests/test_mcp_tool_handlers.py
git commit -m "fix(mcp): document collect evidence adapter boundary"
```

### Task 3: Restore Python Full MCP Tool Inventory Parity

**Files:**
- Modify: `packages/python-qiongli/src/qiongli/bridges/mcp_cli.py`
- Modify: `tests/test_mcp_cli.py`

- [ ] **Step 1: Write failing CLI inventory assertions**

In `tests/test_mcp_cli.py`, add this assertion to `test_mcp_cli_doctor_json_reports_shared_provider_config` after the existing `qiongli_literature_search` assertion:

```python
        self.assertIn("qiongli_search_plan", payload["literature_tools"])
```

Add this assertion to `test_mcp_cli_config_example_for_codex_json` after the existing `qiongli_literature_search` assertion:

```python
        self.assertIn("qiongli_search_plan", payload["literature_tools"])
```

Add this assertion to `test_mcp_cli_config_example_for_antigravity_json` after the existing `qiongli_literature_search` assertion:

```python
        self.assertIn("qiongli_search_plan", payload["literature_tools"])
```

- [ ] **Step 2: Run the failing CLI tests**

Run:

```bash
uv run python -m unittest tests.test_mcp_cli.MCPCLITests.test_mcp_cli_doctor_json_reports_shared_provider_config tests.test_mcp_cli.MCPCLITests.test_mcp_cli_config_example_for_codex_json tests.test_mcp_cli.MCPCLITests.test_mcp_cli_config_example_for_antigravity_json -v
```

Expected: FAIL because `qiongli_search_plan` is missing from `LITERATURE_TOOLS`.

- [ ] **Step 3: Add `qiongli_search_plan` to `LITERATURE_TOOLS`**

In `packages/python-qiongli/src/qiongli/bridges/mcp_cli.py`, change `LITERATURE_TOOLS` to:

```python
LITERATURE_TOOLS = [
    "qiongli_literature_status",
    "qiongli_search_plan",
    "qiongli_literature_search",
    "qiongli_literature_export_evidence",
]
```

- [ ] **Step 4: Run the CLI tests**

Run:

```bash
uv run python -m unittest tests.test_mcp_cli -v
```

Expected: PASS. If the wizard server cannot bind in the local sandbox, the existing test helper may skip that environment-specific case.

- [ ] **Step 5: Commit**

```bash
git add packages/python-qiongli/src/qiongli/bridges/mcp_cli.py tests/test_mcp_cli.py
git commit -m "fix(mcp): include search plan in literature tool inventory"
```

### Task 4: Add Cross-Surface MCP Tool Parity Tests

**Files:**
- Create: `tests/test_mcp_tool_surface_parity.py`

- [ ] **Step 1: Create the failing parity test file**

Create `tests/test_mcp_tool_surface_parity.py`:

```python
from __future__ import annotations

import json
from pathlib import Path
import unittest

from bridges.mcp_cli import LITERATURE_TOOLS
from bridges.mcp_tool_handlers import MCP_TOOL_DEFINITIONS


REPO_ROOT = Path(__file__).resolve().parents[1]

COMMON_LITERATURE_PROVIDER_TOOLS = {
    "qiongli_literature_status",
    "qiongli_search_plan",
    "qiongli_config_status",
    "qiongli_configure_provider",
    "qiongli_save_provider_config",
    "qiongli_open_config_wizard",
    "qiongli_literature_search",
    "qiongli_literature_export_evidence",
}


class MCPToolSurfaceParityTests(unittest.TestCase):
    def test_python_full_mcp_exposes_common_literature_provider_tools(self) -> None:
        names = {tool["name"] for tool in MCP_TOOL_DEFINITIONS}

        self.assertTrue(COMMON_LITERATURE_PROVIDER_TOOLS.issubset(names))

    def test_python_cli_literature_tool_inventory_matches_router_surface(self) -> None:
        self.assertEqual(
            LITERATURE_TOOLS,
            [
                "qiongli_literature_status",
                "qiongli_search_plan",
                "qiongli_literature_search",
                "qiongli_literature_export_evidence",
            ],
        )

    def test_node_mcpb_manifest_exposes_common_literature_provider_tools(self) -> None:
        manifest_path = REPO_ROOT / "packages" / "qiongli-literature-mcpb" / "manifest.json"
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        names = {tool["name"] for tool in manifest["tools"]}

        self.assertTrue(COMMON_LITERATURE_PROVIDER_TOOLS.issubset(names))

    def test_collect_evidence_is_python_full_external_adapter_not_mcpb_provider_status(self) -> None:
        python_names = {tool["name"] for tool in MCP_TOOL_DEFINITIONS}
        manifest_path = REPO_ROOT / "packages" / "qiongli-literature-mcpb" / "manifest.json"
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        node_names = {tool["name"] for tool in manifest["tools"]}

        self.assertIn("qiongli_collect_evidence", python_names)
        self.assertNotIn("qiongli_collect_evidence", node_names)


if __name__ == "__main__":
    unittest.main()
```

- [ ] **Step 2: Run the failing parity tests**

Run:

```bash
uv run python -m unittest tests.test_mcp_tool_surface_parity -v
```

Expected: FAIL until Task 3 adds `qiongli_search_plan` to `LITERATURE_TOOLS`.

- [ ] **Step 3: Re-run after Task 3**

Run:

```bash
uv run python -m unittest tests.test_mcp_tool_surface_parity -v
```

Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add tests/test_mcp_tool_surface_parity.py
git commit -m "test(mcp): add literature tool surface parity checks"
```

### Task 5: Update Canonical Workflow Guidance and Materializer Text

**Files:**
- Modify: `content/workflow/SKILL.md`
- Modify: `packages/python-qiongli/src/qiongli/subject_materializer.py`
- Modify: `tests/test_literature_contract.py`

- [ ] **Step 1: Write failing workflow contract tests**

Append these tests to `tests/test_literature_contract.py`:

```python
    def test_workflow_guidance_rejects_collect_evidence_as_provider_status_source(self) -> None:
        content = (REPO_ROOT / "content" / "workflow" / "SKILL.md").read_text(encoding="utf-8")

        self.assertIn("Do not use `qiongli_collect_evidence` to judge", content)
        self.assertIn("`qiongli_literature_status`", content)
        self.assertIn("`qiongli_literature_search`", content)

    def test_workflow_guidance_lists_all_bundled_literature_providers(self) -> None:
        content = (REPO_ROOT / "content" / "workflow" / "SKILL.md").read_text(encoding="utf-8")

        for provider in ("OpenAlex", "Semantic Scholar", "Crossref", "PubMed", "arXiv"):
            self.assertIn(provider, content)
```

- [ ] **Step 2: Run the failing contract tests**

Run:

```bash
uv run python -m unittest tests.test_literature_contract.LiteratureContractTests.test_workflow_guidance_rejects_collect_evidence_as_provider_status_source tests.test_literature_contract.LiteratureContractTests.test_workflow_guidance_lists_all_bundled_literature_providers -v
```

Expected: FAIL because the guidance does not yet explicitly reject `qiongli_collect_evidence` as provider status authority, and source text may still list only OpenAlex/Semantic Scholar in one Desktop/Web sentence.

- [ ] **Step 3: Update `content/workflow/SKILL.md`**

Under `## Literature Provider Configuration`, add this bullet after the `qiongli_literature_status` / `qiongli_search_plan` bullets:

```markdown
- Do not use `qiongli_collect_evidence` to judge built-in literature provider configuration. That tool is a filesystem/builtin/external-command evidence adapter; direct provider names such as `openalex` require a separate `RESEARCH_MCP_OPENALEX_CMD`. Use `qiongli_literature_status`, `qiongli_config_status`, `qiongli_test_provider`, and `qiongli_literature_search` to judge OpenAlex, Semantic Scholar, Crossref, PubMed, and arXiv provider availability.
```

Replace the Desktop/Web MCPB sentence with:

```markdown
- Desktop/Web users need the Qiongli Literature Provider `.mcpb` (`qiongli-literature-provider.mcpb`) or another configured provider MCP before claiming `provider_connected` literature search. The MCPB is the separate local Claude Desktop provider for OpenAlex, Semantic Scholar, Crossref, PubMed, and arXiv configuration/search. arXiv is enabled without credentials. Platform-native search alone is `native_only`, not `provider_connected`; if no provider MCP/MCPB and no platform-native search is available, record the run as `strategy_only`.
```

- [ ] **Step 4: Update `subject_materializer.py`**

In `packages/python-qiongli/src/qiongli/subject_materializer.py`, find the provider guidance list near the existing `Desktop/Web users need...` string and add the same `qiongli_collect_evidence` boundary bullet. Confirm the Desktop/Web MCPB string matches the text from Step 3.

- [ ] **Step 5: Run contract tests**

Run:

```bash
uv run python -m unittest tests.test_literature_contract -v
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add content/workflow/SKILL.md packages/python-qiongli/src/qiongli/subject_materializer.py tests/test_literature_contract.py
git commit -m "docs(workflow): clarify literature provider status authority"
```

### Task 6: Fix CLI and Cross-Platform MCP Documentation

**Files:**
- Modify: `docs/reference/cli.md`
- Modify: `docs/zh/reference/cli.md`
- Modify: `docs/advanced/cross-platform-mcp.md`
- Modify: `README.md`
- Modify: `README_CN.md`
- Modify: `tests/test_mcp_provider_docs.py`

- [ ] **Step 1: Write failing docs contract tests**

Append these tests to `tests/test_mcp_provider_docs.py`:

```python
    def test_cli_reference_lists_literature_status_search_plan_and_search(self) -> None:
        content = (REPO_ROOT / "docs" / "reference" / "cli.md").read_text(encoding="utf-8")

        for tool in (
            "qiongli_literature_status",
            "qiongli_search_plan",
            "qiongli_literature_search",
            "qiongli_literature_export_evidence",
        ):
            self.assertIn(tool, content)

    def test_cross_platform_docs_separate_collect_evidence_from_provider_status(self) -> None:
        content = (REPO_ROOT / "docs" / "advanced" / "cross-platform-mcp.md").read_text(
            encoding="utf-8"
        )

        self.assertIn("External evidence adapter", content)
        self.assertIn("Do not use `qiongli_collect_evidence` to judge", content)
        self.assertIn("RESEARCH_MCP_<PROVIDER>_CMD", content)
```

- [ ] **Step 2: Run the failing docs tests**

Run:

```bash
uv run python -m unittest tests.test_mcp_provider_docs -v
```

Expected: FAIL until docs are updated.

- [ ] **Step 3: Update `docs/reference/cli.md`**

Replace the MCP tools list under "MCP tools exposed by the server" with:

```markdown
MCP tools exposed by the full Python server:
- `qiongli_literature_status`
- `qiongli_search_plan`
- `qiongli_literature_search`
- `qiongli_literature_export_evidence`
- `qiongli_config_status`
- `qiongli_save_provider_config`
- `qiongli_configure_provider`
- `qiongli_open_config_wizard`
- `qiongli_list_provider_env`
- `qiongli_test_provider`
- `qiongli_collect_evidence` — filesystem/builtin/external-command evidence adapter. Do not use it to judge OpenAlex/Semantic Scholar/Crossref/PubMed/arXiv provider config; direct provider names require `RESEARCH_MCP_<PROVIDER>_CMD`.
- `qiongli_subject_status`
- `qiongli_subject_update`
- `qiongli_orchestrator_route`
- `qiongli_orchestrator_doctor`
- `qiongli_task_plan`
- `qiongli_task_run`
```

- [ ] **Step 4: Update `docs/zh/reference/cli.md`**

Replace the matching Chinese tools sentence with:

```markdown
full Python server 暴露的 MCP tools 包括 `qiongli_literature_status`、`qiongli_search_plan`、`qiongli_literature_search`、`qiongli_literature_export_evidence`、`qiongli_config_status`、`qiongli_save_provider_config`、`qiongli_configure_provider`、`qiongli_open_config_wizard`、`qiongli_list_provider_env`、`qiongli_test_provider`、`qiongli_collect_evidence`、`qiongli_subject_status`、`qiongli_subject_update`、`qiongli_orchestrator_route`、`qiongli_orchestrator_doctor`、`qiongli_task_plan` 和 `qiongli_task_run`。其中 `qiongli_collect_evidence` 是 filesystem / builtin / external-command evidence adapter；不要用它判断 OpenAlex、Semantic Scholar、Crossref、PubMed 或 arXiv 的 provider config。直接传入 `openalex` 这类 provider name 时，它检查的是 `RESEARCH_MCP_<PROVIDER>_CMD` 外部命令。
```

- [ ] **Step 5: Update `docs/advanced/cross-platform-mcp.md`**

Replace this bullet:

```markdown
- `qiongli_config_status`, `qiongli_configure_provider`, `qiongli_save_provider_config`, and `qiongli_collect_evidence` for MCP/provider readiness.
```

with:

```markdown
- `qiongli_config_status`, `qiongli_configure_provider`, `qiongli_save_provider_config`, `qiongli_list_provider_env`, and `qiongli_test_provider` for provider configuration and redacted readiness.
- `qiongli_collect_evidence` for filesystem, builtin workflow adapters, and external evidence adapter commands. Do not use `qiongli_collect_evidence` to judge built-in literature provider config. Direct academic provider names such as `openalex` require `RESEARCH_MCP_<PROVIDER>_CMD`; OpenAlex/Semantic Scholar/Crossref/PubMed/arXiv provider availability is checked through `qiongli_literature_status` and `qiongli_literature_search`.
```

Also replace the Claude Code bundled plugin sentence that says full runtime includes `qiongli_literature_search` with:

```markdown
This bundled runtime covers literature-provider tools such as provider configuration, status, search, search planning, and evidence export without requiring the `qiongli` CLI. Use `qiongli install --profile full --target claude --surface plugin` when Claude Code needs Python-backed orchestration tools, including `qiongli_orchestrator_route`, `qiongli_orchestrator_doctor`, `qiongli_task_plan`, or `qiongli_task_run`.
```

- [ ] **Step 6: Update top-level README files**

In `README.md`, replace the provider paragraph around the `.mcpb` sentence with:

```markdown
Provider credentials stay in provider config, not generated skill bundles. Use `qiongli provider setup` for OpenAlex, Semantic Scholar, Crossref, PubMed, and arXiv-supported literature workflows, then `qiongli provider doctor` to verify. The `qiongli-literature-provider` `.mcpb` exposes `qiongli_literature_status`, `qiongli_search_plan`, `qiongli_literature_search`, `qiongli_literature_export_evidence`, `qiongli_config_status`, `qiongli_configure_provider`, and `qiongli_save_provider_config` for Codex/Desktop flows; statuses include `provider_connected`, `native_only`, and `strategy_only` depending on provider and platform search availability. `qiongli_collect_evidence` is an external evidence adapter path and must not be used as the OpenAlex provider-config check.
```

In `README_CN.md`, use:

```markdown
Provider 凭据保存在 provider config，不写进生成的 skill bundle。使用 `qiongli provider setup` 配置 OpenAlex、Semantic Scholar、Crossref、PubMed 和 arXiv 支持的文献 workflow，再用 `qiongli provider doctor` 验证。`qiongli-literature-provider` `.mcpb` 为 Codex/Desktop 流程暴露 `qiongli_literature_status`、`qiongli_search_plan`、`qiongli_literature_search`、`qiongli_literature_export_evidence`、`qiongli_config_status`、`qiongli_configure_provider` 和 `qiongli_save_provider_config`；状态会根据 provider 和平台原生搜索可用性区分 `provider_connected`、`native_only` 和 `strategy_only`。`qiongli_collect_evidence` 是 external evidence adapter 路径，不能作为 OpenAlex provider config 检查。
```

- [ ] **Step 7: Run docs tests**

Run:

```bash
uv run python -m unittest tests.test_mcp_provider_docs -v
```

Expected: PASS.

- [ ] **Step 8: Commit**

```bash
git add docs/reference/cli.md docs/zh/reference/cli.md docs/advanced/cross-platform-mcp.md README.md README_CN.md tests/test_mcp_provider_docs.py
git commit -m "docs(mcp): separate provider status from evidence adapters"
```

### Task 7: Fix Install Guide Runtime Boundary Wording

**Files:**
- Modify: `docs/guide/install.md`
- Modify: `docs/zh/guide/install.md`
- Modify: `tests/test_cli_setup_docs.py`

- [ ] **Step 1: Write failing install docs tests**

Append these tests to `tests/test_cli_setup_docs.py`:

```python
    def test_install_docs_do_not_mark_literature_search_full_runtime_only(self) -> None:
        content = (REPO_ROOT / "docs" / "guide" / "install.md").read_text(encoding="utf-8")

        self.assertIn("bundled runtime covers literature-provider tools", content)
        self.assertNotIn(
            "Full runtime commands, including `qiongli_literature_search`",
            content,
        )
        self.assertIn("Python-backed orchestration tools", content)

    def test_zh_install_docs_do_not_mark_literature_search_full_runtime_only(self) -> None:
        content = (REPO_ROOT / "docs" / "zh" / "guide" / "install.md").read_text(encoding="utf-8")

        self.assertIn("内置 literature-provider tools", content)
        self.assertNotIn(
            "需要 `qiongli_literature_search`、`qiongli_task_plan`",
            content,
        )
        self.assertIn("Python-backed orchestration tools", content)
```

- [ ] **Step 2: Run the failing install docs tests**

Run:

```bash
uv run python -m unittest tests.test_cli_setup_docs -v
```

Expected: FAIL until wording is updated.

- [ ] **Step 3: Update English install guide**

In `docs/guide/install.md`, replace the Claude Code plugin paragraph with:

```markdown
The Claude Code plugin also bundles the zero-dependency Node literature-provider MCP runtime under `mcp/qiongli-literature-provider/`, using the same provider, search, search-plan, evidence-export, and status tools as the Codex plugin. It covers literature-provider MCP without installing the `qiongli` CLI. Python-backed orchestration tools, including `qiongli_orchestrator_route`, `qiongli_task_plan`, `qiongli_task_run`, and `qiongli_orchestrator_doctor`, require the full runtime: `pipx install qiongli`. Then run `qiongli install --profile full --target claude --surface plugin` to generate a local Claude Code plugin that launches the unified `qiongli mcp serve --transport stdio` server. Use `--target antigravity` to generate the Antigravity plugin with its root `mcp_config.json`, `--target hermes` for Hermes MCP config, or `--target all --surface plugin` when Codex/Claude Code/Antigravity should use local plugins and Hermes should receive managed full MCP config.
```

- [ ] **Step 4: Update Chinese install guide**

In `docs/zh/guide/install.md`, replace the matching Claude Code plugin paragraph with:

```markdown
Claude Code marketplace plugin 也内置 `mcp/qiongli-literature-provider/` 下的零依赖 Node literature-provider MCP runtime，提供与 Codex plugin 相同的 provider、search、search-plan、evidence-export 和 status tools。只使用这些内置 literature-provider tools 时，不需要安装 `qiongli` CLI。`qiongli_orchestrator_route`、`qiongli_task_plan`、`qiongli_task_run` 和 `qiongli_orchestrator_doctor` 这类 Python-backed orchestration tools 需要完整运行时：`pipx install qiongli`。然后运行 `qiongli install --profile full --target claude --surface plugin` 生成本地 Claude Code plugin，并由这个 plugin 启动统一的 `qiongli mcp serve --transport stdio` server。`--target antigravity` 会生成带 root `mcp_config.json` 的 Antigravity plugin，`--target hermes` 写入 Hermes MCP config；`--target all --surface plugin` 会让 Codex / Claude Code / Antigravity 使用本地 plugin，同时给 Hermes 写入受管理的 full MCP client 配置。
```

- [ ] **Step 5: Run install docs tests**

Run:

```bash
uv run python -m unittest tests.test_cli_setup_docs -v
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add docs/guide/install.md docs/zh/guide/install.md tests/test_cli_setup_docs.py
git commit -m "docs(install): clarify bundled literature MCP search support"
```

### Task 8: Update Static Advanced Reference If Still Maintained

**Files:**
- Modify: `docs/advanced/qiongli-cli-plugin-structure.html`

- [ ] **Step 1: Inspect static source ownership**

Run:

```bash
rg -n "qiongli-cli-plugin-structure|qiongli_collect_evidence|qiongli_literature_search" docs tooling scripts packages -S
```

Expected: Identify whether `docs/advanced/qiongli-cli-plugin-structure.html` is generated from a source file. If a generator exists, update the source and regenerate. If no generator exists, edit the HTML directly.

- [ ] **Step 2: Update the tool list**

Ensure the HTML tool list includes:

```html
<li><code>qiongli_literature_status</code></li>
<li><code>qiongli_search_plan</code></li>
<li><code>qiongli_literature_search</code></li>
<li><code>qiongli_literature_export_evidence</code></li>
<li><code>qiongli_config_status</code></li>
<li><code>qiongli_configure_provider</code></li>
<li><code>qiongli_save_provider_config</code></li>
<li><code>qiongli_collect_evidence</code> <span>external command evidence adapter, not provider-config status</span></li>
```

- [ ] **Step 3: Verify no stale full-runtime-only search claim remains**

Run:

```bash
rg -n "Full runtime commands, including `qiongli_literature_search`|需要 `qiongli_literature_search`|OpenAlex and Semantic Scholar configuration" docs content packages README.md README_CN.md -S
```

Expected: no matches for stale full-runtime-only search claims or two-provider-only MCPB descriptions.

- [ ] **Step 4: Commit**

```bash
git add docs/advanced/qiongli-cli-plugin-structure.html
git commit -m "docs(mcp): refresh static plugin structure reference"
```

### Task 9: Standardize Literature Search Defaults

**Files:**
- Modify: `packages/python-qiongli/src/qiongli/bridges/providers/literature_search.py`
- Modify: `packages/python-qiongli/src/qiongli/bridges/literature_mcp_tools.py`
- Modify: `tests/test_literature_search.py`
- Modify: `tests/test_mcp_literature_tools.py`
- Modify: `packages/qiongli-literature-mcpb/server/providers/openalex.mjs`
- Modify: `packages/qiongli-literature-mcpb/server/providers/semantic-scholar.mjs`
- Modify: `packages/qiongli-literature-mcpb/server/providers/crossref.mjs`
- Modify: `packages/qiongli-literature-mcpb/server/providers/pubmed.mjs`
- Modify: `packages/qiongli-literature-mcpb/server/providers/arxiv.mjs`
- Modify: `packages/qiongli-literature-mcpb/test/config.test.mjs`
- Modify: `packages/qiongli-literature-mcpb/test/providers.test.mjs`

- [ ] **Step 1: Write failing Python default-limit tests**

In `tests/test_literature_search.py`, change `test_run_scholarly_search_uses_wider_default_per_query_limit` so it expects 25:

```python
        self.assertEqual(result["data"]["per_query_limit"], 25)
        self.assertTrue(seen_limits)
        self.assertTrue(all(limit == 25 for limit in seen_limits))
```

Append this review-mode test after it:

```python
    def test_run_scholarly_search_uses_review_default_per_query_limit(self) -> None:
        seen_limits: list[int] = []

        def fake_search(query: str, limit: int) -> dict[str, object]:
            seen_limits.append(limit)
            return {"data": [{"paperId": query, "title": query, "year": 2024}]}

        result = run_scholarly_search(
            {
                "topic": "qualitative governance systematic review",
                "search_mode": "review",
                "keywords": ["governance", "firms"],
            },
            fake_search,
            retrieved_at="2026-03-25T12:00:00+00:00",
        )

        self.assertEqual(result["data"]["per_query_limit"], 50)
        self.assertTrue(seen_limits)
        self.assertTrue(all(limit == 50 for limit in seen_limits))
```

- [ ] **Step 2: Run the failing Python search tests**

Run:

```bash
uv run python -m unittest tests.test_literature_search.LiteratureSearchTests.test_run_scholarly_search_uses_wider_default_per_query_limit tests.test_literature_search.LiteratureSearchTests.test_run_scholarly_search_uses_review_default_per_query_limit -v
```

Expected: FAIL because Python default is still 20 and has no review-mode default.

- [ ] **Step 3: Implement Python default tiers**

In `packages/python-qiongli/src/qiongli/bridges/providers/literature_search.py`, replace the current default constants with:

```python
STANDARD_PER_QUERY_LIMIT = 25
REVIEW_PER_QUERY_LIMIT = 50
DEEP_PER_QUERY_LIMIT = 100
MAX_PER_QUERY_LIMIT = 200
```

Replace `_resolve_per_query_limit(...)` with:

```python
def _resolve_per_query_limit(task_packet: dict[str, Any]) -> int:
    default_limit = _default_per_query_limit(task_packet)
    for key in ("per_query_limit", "per_provider_limit", "limit", "search_limit"):
        value = task_packet.get(key)
        try:
            parsed = int(str(value).strip())
        except (TypeError, ValueError):
            continue
        return max(1, min(parsed, MAX_PER_QUERY_LIMIT))
    return default_limit


def _default_per_query_limit(task_packet: dict[str, Any]) -> int:
    raw_depth = str(task_packet.get("search_depth", "") or "").strip().lower().replace("-", "_")
    raw_mode = str(task_packet.get("search_mode", "") or "").strip().lower().replace("-", "_")
    raw_paper_type = str(task_packet.get("paper_type", "") or "").strip().lower().replace("-", "_")
    if raw_depth == "deep":
        return DEEP_PER_QUERY_LIMIT
    if raw_depth in {"review", "systematic_review"}:
        return REVIEW_PER_QUERY_LIMIT
    if raw_mode in {"review", "systematic_review", "literature_review", "lit_review"}:
        return REVIEW_PER_QUERY_LIMIT
    if raw_paper_type in {"review", "systematic_review", "literature_review", "lit_review"}:
        return REVIEW_PER_QUERY_LIMIT
    return STANDARD_PER_QUERY_LIMIT
```

- [ ] **Step 4: Preserve `search_depth` in Python MCP task packets**

In `packages/python-qiongli/src/qiongli/bridges/literature_mcp_tools.py`, add this field to the dict returned by `_task_packet_from_search_args(...)`:

```python
        "search_depth": args.get("search_depth", args.get("searchDepth")),
```

- [ ] **Step 5: Add MCP task-packet default test**

Append this test to `tests/test_mcp_literature_tools.py`:

```python
    def test_literature_search_review_mode_defaults_to_fifty_per_provider(self) -> None:
        provider_calls: list[int] = []

        def openalex_search(translation: dict[str, object], limit: int) -> dict[str, object]:
            provider_calls.append(limit)
            return {"data": []}

        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            with mock.patch.dict("os.environ", {"QIONGLI_CONFIG_HOME": str(root / "config")}, clear=True):
                set_provider_value("openalex", "api-key", "openalex-secret-key")
                with mock.patch("bridges.literature_mcp_tools.openalex_client.search", openalex_search):
                    result = call_qiongli_tool(
                        "qiongli_literature_search",
                        {"query": "AI feedback systematic review", "search_mode": "review"},
                    )

        self.assertFalse(result["isError"])
        self.assertEqual(provider_calls, [50])
        self.assertEqual(result["structuredContent"]["data"]["per_query_limit"], 50)
```

- [ ] **Step 6: Run focused Python tests**

Run:

```bash
uv run python -m unittest tests.test_literature_search tests.test_mcp_literature_tools -v
```

Expected: PASS.

- [ ] **Step 7: Update Node provider fallback constants**

In each of these files, change `const DEFAULT_LIMIT = 10;` to `const DEFAULT_LIMIT = 25;`:

```text
packages/qiongli-literature-mcpb/server/providers/openalex.mjs
packages/qiongli-literature-mcpb/server/providers/semantic-scholar.mjs
packages/qiongli-literature-mcpb/server/providers/crossref.mjs
packages/qiongli-literature-mcpb/server/providers/pubmed.mjs
packages/qiongli-literature-mcpb/server/providers/arxiv.mjs
```

Keep `packages/qiongli-literature-mcpb/server/index.mjs` unchanged: it already has `REVIEW_DEFAULT_LIMIT = 50`, `STANDARD_MAX_LIMIT = 50`, and `REVIEW_MAX_LIMIT = 200`.

- [ ] **Step 8: Add Node fallback default tests**

In `packages/qiongli-literature-mcpb/test/providers.test.mjs`, add provider-specific assertions where provider URL assertions already exist. For example, in the OpenAlex no-limit request test, assert:

```javascript
assert.equal(requestedUrl.searchParams.get("per-page"), "25");
```

For Semantic Scholar, Crossref, PubMed, and arXiv no-limit tests, assert the provider-specific request parameter is 25:

```javascript
assert.equal(requestedUrl.searchParams.get("limit"), "25");
assert.equal(requestedUrl.searchParams.get("rows"), "25");
assert.equal(requestedUrl.searchParams.get("retmax"), "25");
assert.equal(requestedUrl.searchParams.get("max_results"), "25");
```

- [ ] **Step 9: Run Node provider tests**

Run:

```bash
node --test packages/qiongli-literature-mcpb/test/providers.test.mjs packages/qiongli-literature-mcpb/test/config.test.mjs packages/qiongli-literature-mcpb/test/tools.test.mjs
```

Expected: PASS.

- [ ] **Step 10: Commit**

```bash
git add \
  packages/python-qiongli/src/qiongli/bridges/providers/literature_search.py \
  packages/python-qiongli/src/qiongli/bridges/literature_mcp_tools.py \
  tests/test_literature_search.py \
  tests/test_mcp_literature_tools.py \
  packages/qiongli-literature-mcpb/server/providers/openalex.mjs \
  packages/qiongli-literature-mcpb/server/providers/semantic-scholar.mjs \
  packages/qiongli-literature-mcpb/server/providers/crossref.mjs \
  packages/qiongli-literature-mcpb/server/providers/pubmed.mjs \
  packages/qiongli-literature-mcpb/server/providers/arxiv.mjs \
  packages/qiongli-literature-mcpb/test/providers.test.mjs
git commit -m "fix(search): standardize literature result defaults"
```

### Task 10: Preserve Full-Text Access Hints in Search Results

**Files:**
- Modify: `packages/qiongli-literature-mcpb/server/normalize.mjs`
- Modify: `packages/qiongli-literature-mcpb/server/providers/openalex.mjs`
- Modify: `packages/qiongli-literature-mcpb/server/providers/semantic-scholar.mjs`
- Modify: `packages/qiongli-literature-mcpb/server/providers/arxiv.mjs`
- Modify: `packages/qiongli-literature-mcpb/test/providers.test.mjs`
- Modify: `packages/qiongli-literature-mcpb/test/tools.test.mjs`
- Modify: `content/skills/B_literature/fulltext-fetcher.md`
- Modify: `content/templates/paper-note.md`

- [ ] **Step 1: Write failing Node normalization test**

Append this test to `packages/qiongli-literature-mcpb/test/tools.test.mjs` near the search schema tests:

```javascript
test("normalizeResult preserves full-text access candidate fields", () => {
  const result = normalizeResult({
    title: "OA Paper",
    open_access_pdf_url: "https://example.org/paper.pdf",
    access_url: "https://example.org/paper",
    fulltext_status: "not_retrieved:oa_candidate",
    evidence_limit: "abstract_only"
  });

  assert.equal(result.open_access_pdf_url, "https://example.org/paper.pdf");
  assert.equal(result.access_url, "https://example.org/paper");
  assert.equal(result.fulltext_status, "not_retrieved:oa_candidate");
  assert.equal(result.evidence_limit, "abstract_only");
});
```

If `normalizeResult` is not imported in that file, add:

```javascript
import { normalizeResult } from "../server/normalize.mjs";
```

- [ ] **Step 2: Run the failing normalization test**

Run:

```bash
node --test packages/qiongli-literature-mcpb/test/tools.test.mjs
```

Expected: FAIL because `normalizeResult` currently drops these fields.

- [ ] **Step 3: Preserve fields in `normalizeResult`**

In `packages/qiongli-literature-mcpb/server/normalize.mjs`, add these fields to the returned object in `normalizeResult(record)`:

```javascript
    open_access_pdf_url: cleanString(record?.open_access_pdf_url ?? record?.openAccessPdf?.url),
    access_url: cleanString(record?.access_url),
    fulltext_status: cleanString(record?.fulltext_status),
    evidence_limit: cleanString(record?.evidence_limit),
    license: cleanString(record?.license),
```

- [ ] **Step 4: Map OpenAlex OA candidate URLs**

In `packages/qiongli-literature-mcpb/server/providers/openalex.mjs`, add this helper near `openAlexUrl(...)`:

```javascript
function openAlexPdfUrl(work) {
  return (
    work?.best_oa_location?.pdf_url ??
    work?.primary_location?.pdf_url ??
    work?.open_access?.oa_url ??
    null
  );
}
```

In `mapWork(work)`, add:

```javascript
    open_access_pdf_url: openAlexPdfUrl(work),
    access_url: openAlexPdfUrl(work) ?? openAlexUrl(work),
    fulltext_status: openAlexPdfUrl(work) ? "not_retrieved:oa_candidate" : "metadata_only",
    evidence_limit: work?.abstract_inverted_index ? "abstract_only" : "metadata_only",
    license: work?.best_oa_location?.license ?? work?.primary_location?.license,
```

- [ ] **Step 5: Map Semantic Scholar OA candidate URLs**

In `packages/qiongli-literature-mcpb/server/providers/semantic-scholar.mjs`, add this helper near `doiFor(...)`:

```javascript
function openAccessPdfUrl(paper) {
  return typeof paper?.openAccessPdf?.url === "string" ? paper.openAccessPdf.url : null;
}
```

In `BASE_FIELDS`, ensure `openAccessPdf` is included:

```javascript
  "openAccessPdf",
```

In `mapPaper(paper)`, add:

```javascript
    open_access_pdf_url: openAccessPdfUrl(paper),
    access_url: openAccessPdfUrl(paper) ?? paper?.url,
    fulltext_status: openAccessPdfUrl(paper) ? "not_retrieved:oa_candidate" : "metadata_only",
    evidence_limit: paper?.abstract ? "abstract_only" : "metadata_only",
```

- [ ] **Step 6: Map arXiv PDF URLs**

In `packages/qiongli-literature-mcpb/server/providers/arxiv.mjs`, where the record is built from an entry, add:

```javascript
    open_access_pdf_url: pdfUrl,
    access_url: pdfUrl ?? absUrl,
    fulltext_status: pdfUrl ? "not_retrieved:oa_candidate" : "metadata_only",
    evidence_limit: abstract ? "abstract_only" : "metadata_only",
```

Use the local variable names already present in the arXiv mapper. If they differ, keep the same meaning: PDF URL first, abstract/entry landing URL second.

- [ ] **Step 7: Add provider mapping tests**

In `packages/qiongli-literature-mcpb/test/providers.test.mjs`, add assertions to existing OpenAlex, Semantic Scholar, and arXiv normalization tests:

```javascript
assert.equal(result.open_access_pdf_url, "https://example.org/paper.pdf");
assert.equal(result.fulltext_status, "not_retrieved:oa_candidate");
assert.equal(result.evidence_limit, "abstract_only");
```

Use the fixture field names for each provider:

```javascript
best_oa_location: { pdf_url: "https://example.org/paper.pdf", license: "cc-by" }
openAccessPdf: { url: "https://example.org/paper.pdf" }
<link title="pdf" href="https://arxiv.org/pdf/1234.5678" />
```

- [ ] **Step 8: Update evidence-boundary docs**

In `content/skills/B_literature/fulltext-fetcher.md`, under "Provider Ownership Boundary", add:

```markdown
Search providers can identify full-text candidates, but they do not prove that the full text was retrieved or read. Treat `open_access_pdf_url` and `access_url` as retrieval candidates until `retrieval_manifest.csv` records `retrieved_oa`, `retrieved_preprint`, or a controlled `not_retrieved:*` status.
```

In `content/templates/paper-note.md`, under "Evidence Boundary", add:

```markdown
Metadata search coverage and full-text access are separate. A paper found through OpenAlex, Semantic Scholar, Crossref, PubMed, or arXiv remains `abstract_only` or `metadata_only` until the retrieval manifest records a readable full-text version.
```

- [ ] **Step 9: Run Node and contract tests**

Run:

```bash
node --test packages/qiongli-literature-mcpb/test/tools.test.mjs packages/qiongli-literature-mcpb/test/providers.test.mjs
uv run python -m unittest tests.test_literature_contract -v
```

Expected: PASS.

- [ ] **Step 10: Commit**

```bash
git add \
  packages/qiongli-literature-mcpb/server/normalize.mjs \
  packages/qiongli-literature-mcpb/server/providers/openalex.mjs \
  packages/qiongli-literature-mcpb/server/providers/semantic-scholar.mjs \
  packages/qiongli-literature-mcpb/server/providers/arxiv.mjs \
  packages/qiongli-literature-mcpb/test/providers.test.mjs \
  packages/qiongli-literature-mcpb/test/tools.test.mjs \
  content/skills/B_literature/fulltext-fetcher.md \
  content/templates/paper-note.md
git commit -m "fix(search): preserve full text access candidates"
```

### Task 11: Add Native Full-Text Candidate and Zotero Attachment Verification Layer

**Files:**
- Modify: `packages/qiongli-literature-mcpb/server/search-plan.mjs`
- Modify: `packages/python-qiongli/src/qiongli/bridges/hybrid_search_router.py`
- Modify: `packages/qiongli-literature-mcpb/test/tools.test.mjs`
- Modify: `tests/test_hybrid_search_router.py`
- Modify: `packages/qiongli-zotero-companion/chrome/content/qiongli-bridge.js`
- Modify: `packages/qiongli-zotero-companion/bootstrap.js`
- Modify: `packages/qiongli-zotero-companion/test/bridge.test.mjs`
- Modify: `packages/qiongli-zotero-companion/README.md`
- Modify: `packages/qiongli-literature-mcpb/server/zotero/search-source.mjs`
- Modify: `packages/qiongli-literature-mcpb/test/zotero.test.mjs`

- [ ] **Step 1: Write failing Node search-plan tests for native full-text candidate queries**

Append this test to `packages/qiongli-literature-mcpb/test/tools.test.mjs` near the existing `buildHybridSearchPlan` tests:

```javascript
test("buildHybridSearchPlan emits native full-text candidate queries separately", () => {
  const plan = buildHybridSearchPlan(
    {
      query: "AI feedback in education",
      platform: "codex",
      native_search_available: true,
      native_search_tools: ["codex_web_search"],
      search_mode: "review"
    },
    "provider_connected",
    { openalex: "configured", arxiv: "configured" }
  );

  assert.equal(plan.search_execution_mode, "hybrid_search");
  assert.equal(plan.native_search_queries.length, 1);
  assert.equal(plan.native_fulltext_queries.length, 1);
  assert.equal(plan.native_fulltext_queries[0].tool, "codex_web_search");
  assert.equal(plan.native_fulltext_queries[0].purpose, "fulltext_candidate_discovery");
  assert.equal(plan.native_fulltext_queries[0].candidate_status, "candidate_only");
  assert.match(plan.native_fulltext_queries[0].query, /PDF/);
  assert.match(plan.native_fulltext_queries[0].query, /full text/);
  assert.equal(plan.native_fulltext_queries[0].provenance_label, "native:codex_web_search");
  assert.deepEqual(plan.native_fulltext_candidate_schema.required, [
    "query_id",
    "source_agent",
    "url",
    "title",
    "candidate_status",
    "retrieved_at"
  ]);
  assert.ok(
    plan.agent_instructions.includes(
      "Use native_fulltext_queries only to discover candidate URLs; do not mark full text as retrieved from search snippets."
    )
  );
  assert.ok(
    plan.merge_policy.fulltext_candidate_records.includes("candidate_only")
  );
});
```

- [ ] **Step 2: Write failing Python hybrid-router tests**

Append this test to `tests/test_hybrid_search_router.py`:

```python
    def test_native_fulltext_queries_are_separate_candidate_discovery(self) -> None:
        plan = build_hybrid_search_plan(
            {
                "query": "AI feedback in education",
                "platform": "codex",
                "native_search_available": True,
                "native_search_tools": ["codex_web_search"],
                "search_mode": "review",
            },
            provider_capability_mode="provider_connected",
            provider_status={"openalex": "configured", "arxiv": "configured"},
        )

        self.assertEqual(plan["search_execution_mode"], "hybrid_search")
        self.assertEqual(len(plan["native_search_queries"]), 1)
        self.assertEqual(len(plan["native_fulltext_queries"]), 1)
        fulltext_query = plan["native_fulltext_queries"][0]
        self.assertEqual(fulltext_query["purpose"], "fulltext_candidate_discovery")
        self.assertEqual(fulltext_query["candidate_status"], "candidate_only")
        self.assertEqual(fulltext_query["provenance_label"], "native:codex_web_search")
        self.assertIn("PDF", fulltext_query["query"])
        self.assertIn("full text", fulltext_query["query"])
        self.assertEqual(
            plan["native_fulltext_candidate_schema"]["required"],
            ["query_id", "source_agent", "url", "title", "candidate_status", "retrieved_at"],
        )
        self.assertIn("candidate_only", plan["merge_policy"]["fulltext_candidate_records"])
```

- [ ] **Step 3: Run the failing search-plan tests**

Run:

```bash
node --test packages/qiongli-literature-mcpb/test/tools.test.mjs
uv run python -m unittest tests.test_hybrid_search_router.HybridSearchRouterTests.test_native_fulltext_queries_are_separate_candidate_discovery -v
```

Expected: FAIL because plans do not yet expose `native_fulltext_queries` or `native_fulltext_candidate_schema`.

- [ ] **Step 4: Implement Node native full-text candidate planning**

In `packages/qiongli-literature-mcpb/server/search-plan.mjs`, append these entries to `AGENT_INSTRUCTIONS`:

```javascript
  "Use native_fulltext_queries only to discover candidate URLs; do not mark full text as retrieved from search snippets.",
  "Write native_fulltext_candidates with candidate_only status until retrieval_manifest.csv verifies readable text."
```

Add these helpers after `buildNativeQueries(...)`:

```javascript
function buildNativeFulltextQueries(entries, platform, nativeTools, filters, nativeEnabled) {
  if (!nativeEnabled || entries.length === 0) {
    return [];
  }

  return nativeTools.flatMap((tool) =>
    entries.map((entry) => ({
      tool,
      platform,
      query_id: entry.query_id,
      query: fulltextCandidateQuery(entry.query),
      source: entry.source,
      purpose: "fulltext_candidate_discovery",
      candidate_status: "candidate_only",
      filters: { ...filters },
      expected_candidate_fields: [
        "query_id",
        "source_agent",
        "url",
        "title",
        "doi",
        "access_type",
        "snippet",
        "candidate_status",
        "retrieved_at"
      ],
      provenance_label: `native:${tool}`
    }))
  );
}

function fulltextCandidateQuery(query) {
  return `${query} (PDF OR "full text" OR preprint OR "author manuscript" OR repository OR PMC OR arXiv)`;
}

function nativeFulltextCandidateSchema() {
  return {
    artifact_type: "qiongli_native_fulltext_candidate_schema",
    required: ["query_id", "source_agent", "url", "title", "candidate_status", "retrieved_at"],
    optional: ["doi", "access_type", "snippet", "license", "version_label"],
    status_values: ["candidate_only"],
    evidence_rule: "Search snippets and URLs do not prove retrieved full text. Upgrade only through retrieval_manifest.csv."
  };
}
```

Change `executionSequence(providerQueries, nativeQueries)` to accept `nativeFulltextQueries`:

```javascript
function executionSequence(providerQueries, nativeQueries, nativeFulltextQueries) {
```

Add this block after the existing native-search block:

```javascript
  if (nativeFulltextQueries.length > 0) {
    sequence.push({
      actor: "agent",
      action: "execute platform-native full-text candidate search",
      queries: "native_fulltext_queries"
    });
  }
```

Change the merge step inputs to:

```javascript
    inputs: ["provider_queries", "native_search_queries", "native_fulltext_candidates", "user_corpus"]
```

In `mergePolicy()`, add:

```javascript
    fulltext_candidate_records: "Keep native full-text search outputs as candidate_only until retrieval_manifest.csv verifies readable text.",
```

In `buildHybridSearchPlan(...)`, build the new query array next to `nativeQueries`:

```javascript
  const nativeFulltextQueries = buildNativeFulltextQueries(
    entries,
    platform,
    tools,
    filters,
    ["hybrid_search", "native_only"].includes(mode)
  );
```

Add these fields to the returned plan:

```javascript
    native_fulltext_queries: nativeFulltextQueries,
    native_fulltext_candidate_schema: nativeFulltextCandidateSchema(),
```

And pass the new array to the sequence builder:

```javascript
    execution_sequence: executionSequence(providerQueries, nativeQueries, nativeFulltextQueries),
```

- [ ] **Step 5: Implement Python native full-text candidate planning**

In `packages/python-qiongli/src/qiongli/bridges/hybrid_search_router.py`, append these strings to `AGENT_INSTRUCTIONS`:

```python
    "Use native_fulltext_queries only to discover candidate URLs; do not mark full text as retrieved from search snippets.",
    "Write native_fulltext_candidates with candidate_only status until retrieval_manifest.csv verifies readable text.",
```

Add these helpers after `_native_search_queries(...)`:

```python
def _native_fulltext_queries(
    query_entries: list[dict[str, str]],
    platform: str,
    native_search_tools: list[str],
    filters: dict[str, Any],
) -> list[dict[str, Any]]:
    return [
        {
            "tool": tool,
            "platform": platform,
            "query_id": entry["query_id"],
            "query": _fulltext_candidate_query(entry["query"]),
            "source": entry["source"],
            "purpose": "fulltext_candidate_discovery",
            "candidate_status": "candidate_only",
            "filters": dict(filters),
            "expected_candidate_fields": [
                "query_id",
                "source_agent",
                "url",
                "title",
                "doi",
                "access_type",
                "snippet",
                "candidate_status",
                "retrieved_at",
            ],
            "provenance_label": f"native:{tool}",
        }
        for tool in native_search_tools
        for entry in query_entries
    ]


def _fulltext_candidate_query(query: str) -> str:
    return f'{query} (PDF OR "full text" OR preprint OR "author manuscript" OR repository OR PMC OR arXiv)'


def _native_fulltext_candidate_schema() -> dict[str, Any]:
    return {
        "artifact_type": "qiongli_native_fulltext_candidate_schema",
        "required": ["query_id", "source_agent", "url", "title", "candidate_status", "retrieved_at"],
        "optional": ["doi", "access_type", "snippet", "license", "version_label"],
        "status_values": ["candidate_only"],
        "evidence_rule": "Search snippets and URLs do not prove retrieved full text. Upgrade only through retrieval_manifest.csv.",
    }
```

In `build_hybrid_search_plan(...)`, build the new query array after `native_search_queries`:

```python
    native_fulltext_queries = (
        _native_fulltext_queries(query_entries, platform, native_search_tools, filters)
        if search_execution_mode in {"hybrid_search", "native_only"}
        else []
    )
```

Add these fields to the returned plan:

```python
        "native_fulltext_queries": native_fulltext_queries,
        "native_fulltext_candidate_schema": _native_fulltext_candidate_schema(),
```

Change the execution-sequence call to:

```python
        "execution_sequence": _execution_sequence(provider_queries, native_search_queries, native_fulltext_queries),
```

Change `_execution_sequence(...)` to accept the new argument:

```python
def _execution_sequence(
    provider_queries: list[dict[str, Any]],
    native_search_queries: list[dict[str, Any]],
    native_fulltext_queries: list[dict[str, Any]],
) -> list[dict[str, Any]]:
```

Add this block after the native-search block:

```python
    if native_fulltext_queries:
        sequence.append(
            {
                "actor": "agent",
                "action": "execute platform-native full-text candidate search",
                "queries": "native_fulltext_queries",
            }
        )
```

Change the merge step inputs to:

```python
            "inputs": ["provider_queries", "native_search_queries", "native_fulltext_candidates", "user_corpus"],
```

Add this merge policy field:

```python
            "fulltext_candidate_records": "Keep native full-text search outputs as candidate_only until retrieval_manifest.csv verifies readable text.",
```

- [ ] **Step 6: Run search-plan tests**

Run:

```bash
node --test packages/qiongli-literature-mcpb/test/tools.test.mjs
uv run python -m unittest tests.test_hybrid_search_router -v
```

Expected: PASS.

- [ ] **Step 7: Write failing Zotero companion attachment tests**

In `packages/qiongli-zotero-companion/test/bridge.test.mjs`, update the import list:

```javascript
  normalizeAttachments,
```

Append these tests after `toCompactItem returns no local file paths`:

```javascript
test("toCompactItem exposes attachment summaries without local paths by default", () => {
  const compact = toCompactItem({
    key: "ABC123",
    title: "Local Paper",
    DOI: "10.1000/local",
    attachments: [
      {
        key: "ATT123",
        parent_item_key: "ABC123",
        title: "Local Paper PDF",
        filename: "local-paper.pdf",
        mime_type: "application/pdf",
        link_mode: "imported_file",
        path: "/zotero-fixture/storage/ATT123/local-paper.pdf",
        url: "",
        local_file_available: true
      }
    ]
  });

  assert.equal(compact.attachments.length, 1);
  assert.equal(compact.attachments[0].attachment_key, "ATT123");
  assert.equal(compact.attachments[0].filename, "local-paper.pdf");
  assert.equal(compact.attachments[0].mime_type, "application/pdf");
  assert.equal(compact.attachments[0].local_file_available, true);
  assert.equal(compact.attachments[0].select_uri, "zotero://select/library/items/ATT123");
  assert.equal(Object.hasOwn(compact.attachments[0], "path"), false);
});

test("toCompactItem exposes attachment paths only when explicitly requested", () => {
  const compact = toCompactItem(
    {
      key: "ABC123",
      attachments: [
        {
          key: "ATT123",
          filename: "local-paper.pdf",
          mime_type: "application/pdf",
          path: "/zotero-fixture/storage/ATT123/local-paper.pdf"
        }
      ]
    },
    { include_attachment_paths: true }
  );

  assert.equal(compact.attachments[0].path, "/zotero-fixture/storage/ATT123/local-paper.pdf");
});

test("normalizeAttachments keeps only structured attachment metadata", () => {
  assert.deepEqual(
    normalizeAttachments([
      { key: "A", filename: "a.pdf", mime_type: "application/pdf" },
      { filename: "missing-key.pdf" },
      null
    ]),
    [
      {
        attachment_key: "A",
        title: "",
        filename: "a.pdf",
        mime_type: "application/pdf",
        link_mode: "",
        url: "",
        select_uri: "zotero://select/library/items/A",
        local_file_available: false
      }
    ]
  );
});
```

In the `bootstrap startup registers endpoints from Zotero 8 and 9 global object` test, add attachment methods to `libraryItem`:

```javascript
    getAttachments: () => [99],
```

Add a fake attachment lookup to `Zotero.Items`:

```javascript
      getAsync: async (id) => ({
        key: id === 99 ? "ATT123" : "UNKNOWN",
        itemType: "attachment",
        getField: (field) => ({
          title: "Platform Governance PDF",
          filename: "platform-governance.pdf",
          contentType: "application/pdf",
          url: ""
        })[field] ?? "",
        getFilePath: () => "/zotero-fixture/storage/ATT123/platform-governance.pdf",
        attachmentLinkMode: "imported_file"
      })
```

Add assertions after the `/qiongli/search` response checks:

```javascript
  assert.equal(response.body.results[0].attachments.length, 1);
  assert.equal(response.body.results[0].attachments[0].attachment_key, "ATT123");
  assert.equal(response.body.results[0].attachments[0].mime_type, "application/pdf");
  assert.equal(response.body.results[0].attachments[0].local_file_available, true);
  assert.equal(Object.hasOwn(response.body.results[0].attachments[0], "path"), false);
```

- [ ] **Step 8: Run the failing Zotero companion tests**

Run:

```bash
node --test packages/qiongli-zotero-companion/test/bridge.test.mjs
```

Expected: FAIL because `toCompactItem` does not accept attachment options, `normalizeAttachments` is not exported, and bootstrap does not collect attachments.

- [ ] **Step 9: Implement Zotero companion attachment summaries**

In `packages/qiongli-zotero-companion/chrome/content/qiongli-bridge.js`, update `searchLocalItems(...)` so the query controls path exposure:

```javascript
    .map((item) => toCompactItem(item, query));
```

Change the exported compact mapper to:

```javascript
export function toCompactItem(item = {}, options = {}) {
  return {
    item_key: item.key ?? "",
    title: item.title ?? "",
    doi: normalizeDoi(item.DOI ?? item.doi),
    year: normalizeYear(item.date ?? item.year),
    item_type: item.itemType ?? "",
    select_uri: item.key ? `zotero://select/library/items/${item.key}` : "",
    tags: Array.isArray(item.tags) ? item.tags.map((tag) => tag.tag ?? tag).filter(Boolean) : [],
    collections: Array.isArray(item.collections) ? item.collections : [],
    attachments: normalizeAttachments(item.attachments, options)
  };
}
```

Add this exported helper before `normalizeDoi(...)`:

```javascript
export function normalizeAttachments(value = [], options = {}) {
  const includePaths = options.include_attachment_paths === true || options.includeAttachmentPaths === true;
  const attachments = Array.isArray(value) ? value : [];
  return attachments
    .map((attachment) => {
      if (!attachment || typeof attachment !== "object") {
        return null;
      }
      const key = String(attachment.attachment_key ?? attachment.key ?? "").trim();
      if (!key) {
        return null;
      }
      const normalized = {
        attachment_key: key,
        title: String(attachment.title ?? "").trim(),
        filename: String(attachment.filename ?? "").trim(),
        mime_type: String(attachment.mime_type ?? attachment.contentType ?? "").trim(),
        link_mode: String(attachment.link_mode ?? attachment.linkMode ?? "").trim(),
        url: String(attachment.url ?? "").trim(),
        select_uri: `zotero://select/library/items/${key}`,
        local_file_available: Boolean(attachment.local_file_available ?? attachment.path)
      };
      if (includePaths && attachment.path) {
        normalized.path = String(attachment.path);
      }
      return normalized;
    })
    .filter(Boolean);
}
```

In `packages/qiongli-zotero-companion/bootstrap.js`, change `listItems()` to collect plain objects asynchronously:

```javascript
      const regularItems = asArray(rawItems).filter((item) => typeof item.isRegularItem !== "function" || item.isRegularItem());
      const plainItems = [];
      for (const item of regularItems) {
        plainItems.push(await itemToPlainObject(Zotero, item));
      }
      return plainItems;
```

Change the `/qiongli/search` response mapper to pass the query options:

```javascript
        results: items.filter((item) => itemMatchesQuery(item, query)).map((item) => toCompactItem(item, query))
```

Change `itemToPlainObject(item)` to:

```javascript
async function itemToPlainObject(Zotero, item) {
  return {
    key: item.key,
    itemType: item.itemType,
    title: getField(item, "title"),
    DOI: getField(item, "DOI"),
    url: getField(item, "url"),
    abstractNote: getField(item, "abstractNote"),
    publicationTitle: getField(item, "publicationTitle"),
    date: getField(item, "date"),
    tags: typeof item.getTags === "function" ? item.getTags() : [],
    collections: typeof item.getCollections === "function" ? item.getCollections() : [],
    attachments: await attachmentPlainObjects(Zotero, item)
  };
}
```

Add this helper after `itemToPlainObject(...)`:

```javascript
async function attachmentPlainObjects(Zotero, item) {
  const attachmentIds = typeof item.getAttachments === "function" ? asArray(item.getAttachments()) : [];
  const attachments = [];
  for (const id of attachmentIds) {
    const attachment = typeof Zotero.Items?.getAsync === "function"
      ? await Zotero.Items.getAsync(id)
      : typeof Zotero.Items?.get === "function"
        ? Zotero.Items.get(id)
        : null;
    if (!attachment) {
      continue;
    }
    const path = typeof attachment.getFilePath === "function" ? attachment.getFilePath() : "";
    attachments.push({
      key: attachment.key,
      parent_item_key: item.key,
      title: getField(attachment, "title"),
      filename: getField(attachment, "filename"),
      mime_type: getField(attachment, "contentType") || attachment.attachmentContentType || "",
      link_mode: String(attachment.attachmentLinkMode ?? ""),
      path,
      url: getField(attachment, "url"),
      local_file_available: Boolean(path)
    });
  }
  return attachments;
}
```

Change `toCompactItem(item)` in `bootstrap.js` to match the source module:

```javascript
function toCompactItem(item, options) {
  return {
    item_key: item.key ?? "",
    title: item.title ?? "",
    doi: normalizeDoi(item.DOI ?? item.doi),
    year: parseYear(item.date ?? item.year),
    item_type: item.itemType ?? "",
    select_uri: item.key ? `zotero://select/library/items/${item.key}` : "",
    tags: Array.isArray(item.tags) ? item.tags.map((tag) => tag.tag ?? tag).filter(Boolean) : [],
    collections: Array.isArray(item.collections) ? item.collections : [],
    attachments: normalizeAttachments(item.attachments, options)
  };
}
```

Add the non-exported `normalizeAttachments(...)` helper to `bootstrap.js` before `parseJson(...)`, using the same body as the source helper but without the `export` keyword.

- [ ] **Step 10: Run Zotero companion tests**

Run:

```bash
node --test packages/qiongli-zotero-companion/test/bridge.test.mjs
```

Expected: PASS.

- [ ] **Step 11: Write failing MCPB Zotero normalization tests**

In `packages/qiongli-literature-mcpb/test/zotero.test.mjs`, append:

```javascript
test("normalizeZoteroSourceResults maps attachment summaries to full-text hints", () => {
  const results = normalizeZoteroSourceResults([
    {
      item_key: "ABC123",
      title: "Local Paper",
      doi: "10.1000/local",
      year: 2024,
      abstract: "Local abstract",
      attachments: [
        {
          attachment_key: "ATT123",
          filename: "local-paper.pdf",
          mime_type: "application/pdf",
          url: "",
          select_uri: "zotero://select/library/items/ATT123",
          local_file_available: true
        }
      ]
    }
  ]);

  assert.equal(results[0].provider, "zotero");
  assert.equal(results[0].fulltext_status, "retrieved_zotero");
  assert.equal(results[0].evidence_limit, "full_text");
  assert.equal(results[0].access_url, "zotero://select/library/items/ATT123");
  assert.equal(results[0].zotero.attachments.length, 1);
  assert.equal(results[0].zotero.attachments[0].attachment_key, "ATT123");
});

test("annotateLocalZoteroMatches includes match confidence and attachment status", () => {
  const annotated = annotateLocalZoteroMatches({
    externalResults: [
      { title: "External Paper", doi: "10.1000/match", year: 2024, provider: "openalex" }
    ],
    zoteroResults: [
      {
        title: "Local Paper",
        doi: "10.1000/match",
        year: 2024,
        provider: "zotero",
        fulltext_status: "retrieved_zotero",
        zotero: {
          item_key: "ABC123",
          select_uri: "zotero://select/library/items/ABC123",
          attachments: [{ attachment_key: "ATT123", mime_type: "application/pdf" }]
        }
      }
    ]
  });

  assert.equal(annotated[0].local_zotero_match.item_key, "ABC123");
  assert.equal(annotated[0].local_zotero_match.match_basis, "doi");
  assert.equal(annotated[0].local_zotero_match.match_confidence, 1);
  assert.equal(annotated[0].local_zotero_match.fulltext_status, "retrieved_zotero");
  assert.equal(annotated[0].local_zotero_match.attachments.length, 1);
});
```

- [ ] **Step 12: Run failing MCPB Zotero tests**

Run:

```bash
node --test packages/qiongli-literature-mcpb/test/zotero.test.mjs
```

Expected: FAIL because Zotero source normalization drops attachment and full-text status fields.

- [ ] **Step 13: Implement MCPB Zotero attachment normalization**

In `packages/qiongli-literature-mcpb/server/zotero/search-source.mjs`, add these helpers before `normalizeZoteroSourceResults(...)`:

```javascript
function normalizeZoteroAttachments(value = []) {
  const attachments = Array.isArray(value) ? value : [];
  return attachments
    .map((attachment) => {
      if (!attachment || typeof attachment !== "object") {
        return null;
      }
      const attachmentKey = cleanString(attachment.attachment_key ?? attachment.key);
      if (!attachmentKey) {
        return null;
      }
      return {
        attachment_key: attachmentKey,
        title: cleanString(attachment.title) ?? "",
        filename: cleanString(attachment.filename) ?? "",
        mime_type: cleanString(attachment.mime_type ?? attachment.contentType) ?? "",
        link_mode: cleanString(attachment.link_mode ?? attachment.linkMode) ?? "",
        url: cleanString(attachment.url),
        select_uri: cleanString(attachment.select_uri) ?? `zotero://select/library/items/${attachmentKey}`,
        local_file_available: Boolean(attachment.local_file_available)
      };
    })
    .filter(Boolean);
}

function bestFulltextAttachment(attachments = []) {
  return attachments.find((attachment) => attachment.mime_type === "application/pdf")
    ?? attachments.find((attachment) => attachment.local_file_available)
    ?? null;
}

function zoteroFulltextStatus(attachment) {
  if (!attachment) {
    return "metadata_only";
  }
  return attachment.local_file_available ? "retrieved_zotero" : "not_retrieved:zotero_attachment_candidate";
}

function zoteroEvidenceLimit(item, attachment) {
  if (attachment?.local_file_available) {
    return "full_text";
  }
  return cleanString(item?.abstract ?? item?.abstractNote) ? "abstract_only" : "metadata_only";
}
```

Inside `normalizeZoteroSourceResults(...)`, before returning each mapped item, compute:

```javascript
      const attachments = normalizeZoteroAttachments(item.attachments);
      const fulltextAttachment = bestFulltextAttachment(attachments);
```

Add these fields to the returned object:

```javascript
        access_url: fulltextAttachment?.url ?? fulltextAttachment?.select_uri ?? cleanString(item.url ?? item.URL),
        fulltext_status: zoteroFulltextStatus(fulltextAttachment),
        evidence_limit: zoteroEvidenceLimit(item, fulltextAttachment),
```

Add the attachment summary to the nested `zotero` object:

```javascript
          attachments,
          fulltext_status: zoteroFulltextStatus(fulltextAttachment),
          fulltext_attachment_key: fulltextAttachment?.attachment_key ?? null,
```

In `annotateLocalZoteroMatches(...)`, extend `local_zotero_match` to:

```javascript
      local_zotero_match: {
        item_key: match.zotero?.item_key ?? match.source_id ?? "",
        match_basis: doiMatch ? "doi" : "title_year",
        match_confidence: doiMatch ? 1 : 0.75,
        select_uri: match.zotero?.select_uri ?? "",
        fulltext_status: match.zotero?.fulltext_status ?? match.fulltext_status ?? "metadata_only",
        attachments: match.zotero?.attachments ?? []
      }
```

- [ ] **Step 14: Update Zotero companion README**

In `packages/qiongli-zotero-companion/README.md`, add this section after the endpoint list:

```markdown
## Attachment Metadata

`POST /qiongli/search` returns compact Zotero item records plus attachment summaries when local attachments exist. Attachment summaries include the Zotero attachment key, file name, MIME type, link mode, Zotero select URI, URL, and `local_file_available`.

Raw local file paths are omitted by default. Send `include_attachment_paths: true` only when a local resolver explicitly needs paths for controlled full-text retrieval. Qiongli treats Zotero attachment data as local verification evidence; provider or native search URLs remain `candidate_only` until `retrieval_manifest.csv` records a retrieved or unresolved status.
```

- [ ] **Step 15: Run full native/Zotero focused tests**

Run:

```bash
node --test packages/qiongli-literature-mcpb/test/tools.test.mjs packages/qiongli-literature-mcpb/test/zotero.test.mjs
node --test packages/qiongli-zotero-companion/test/bridge.test.mjs
uv run python -m unittest tests.test_hybrid_search_router -v
```

Expected: PASS.

- [ ] **Step 16: Commit**

```bash
git add \
  packages/qiongli-literature-mcpb/server/search-plan.mjs \
  packages/python-qiongli/src/qiongli/bridges/hybrid_search_router.py \
  packages/qiongli-literature-mcpb/test/tools.test.mjs \
  tests/test_hybrid_search_router.py \
  packages/qiongli-zotero-companion/chrome/content/qiongli-bridge.js \
  packages/qiongli-zotero-companion/bootstrap.js \
  packages/qiongli-zotero-companion/test/bridge.test.mjs \
  packages/qiongli-zotero-companion/README.md \
  packages/qiongli-literature-mcpb/server/zotero/search-source.mjs \
  packages/qiongli-literature-mcpb/test/zotero.test.mjs
git commit -m "feat(literature): verify native full text candidates with zotero"
```

### Task 12: Document Coverage and Completeness Semantics

**Files:**
- Modify: `docs/advanced/mcp-providers-setup.md`
- Modify: `docs/advanced/rigorous-literature-search.md`
- Modify: `content/workflow/references/stage-B-literature.md`
- Modify: `content/templates/search-diagnostics.md`
- Modify: `tests/test_literature_contract.py`
- Modify: `tests/test_mcp_provider_docs.py`

- [ ] **Step 1: Add contract tests for coverage semantics**

Append this test to `tests/test_literature_contract.py`:

```python
    def test_stage_b_distinguishes_discovery_coverage_from_fulltext_access(self) -> None:
        content = (LAYOUT.references / "stage-B-literature.md").read_text(encoding="utf-8")

        self.assertIn("discovery coverage", content)
        self.assertIn("full-text access coverage", content)
        self.assertIn("retrieval_manifest.csv", content)
        self.assertIn("native_fulltext_queries", content)
        self.assertIn("Zotero attachment", content)
        self.assertIn("metadata_only", content)
```

Append this test to `tests/test_mcp_provider_docs.py`:

```python
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
```

- [ ] **Step 2: Run the failing docs tests**

Run:

```bash
uv run python -m unittest tests.test_literature_contract.LiteratureContractTests.test_stage_b_distinguishes_discovery_coverage_from_fulltext_access tests.test_mcp_provider_docs.MCPProviderDocsTests.test_rigorous_search_docs_define_non_exhaustive_coverage_metrics -v
```

Expected: FAIL until docs are updated.

- [ ] **Step 3: Update Stage B reference**

In `content/workflow/references/stage-B-literature.md`, under "Search diagnostics gate", add:

```markdown
### Coverage and access semantics

Discovery coverage and full-text access coverage are separate:

- `discovery coverage`: how broad and reproducible the metadata search was across providers, query variants, years, venues, document types, citation snowballing, and known-item recall.
- `full-text access coverage`: how many sought reports have a controlled `retrieval_manifest.csv` status such as `retrieved_oa`, `retrieved_preprint`, `abstract_only`, or `not_retrieved:<reason>`.
- `native_fulltext_queries`: platform-native LLM search queries that the active agent may execute to discover PDF, PMC, arXiv, repository, author-manuscript, or publisher full-text candidates. These outputs stay `candidate_only` until retrieval status is recorded.
- `Zotero attachment verification`: local Zotero attachment metadata that can distinguish citation-only Zotero matches from records with a local or linked PDF attachment.
- `evidence_limit`: what the workflow is allowed to claim from a record: `full_text`, `abstract_only`, `metadata_only`, or `unavailable`.

No search provider, native LLM search tool, or local Zotero library proves absolute completeness. Review-grade claims require a reproducible search log, deduplication, known-item recall checks, citation snowballing where appropriate, Zotero attachment verification where available, and retrieval status for every included or sought report.
```

- [ ] **Step 4: Update rigorous search docs**

In `docs/advanced/rigorous-literature-search.md`, add a section named `## Coverage Is Audited, Not Guaranteed`:

```markdown
## Coverage Is Audited, Not Guaranteed

No provider can prove absolute completeness. OpenAlex, Semantic Scholar, Crossref, PubMed, and arXiv provide metadata, abstracts, identifiers, citation/reference links, and sometimes open-access or PDF candidates. They do not guarantee that every relevant paper exists in the provider index, that every full text is accessible, or that the returned full text is the version that should be cited.

Platform-native LLM search can improve recall for full-text entry points through `native_fulltext_queries`, especially for PDFs, author manuscripts, PMC pages, arXiv versions, repositories, publisher landing pages, and institutional copies. Treat these as `candidate_only` records with source URL and retrieval time. A native search snippet is not full-text evidence by itself.

Zotero attachment verification is the local-library check. A Zotero item match by DOI or title-year proves that the reference exists locally; a Zotero PDF attachment summary proves that a local or linked full-text candidate exists. The retrieval manifest still records whether that attachment was readable and which version was used.

Use these coverage checks before claiming review-grade search:

- `provider coverage`: at least two productive scholarly providers for broad reviews, unless the protocol justifies a narrower source.
- `known-item recall`: seed papers expected by the reviewer or protocol must be found by at least one query/provider path.
- `query-block coverage`: each required concept block has nonzero hits or an explicit zero-hit explanation.
- `duplicate saturation`: additional providers or query variants mostly return already-seen records after deduplication.
- `snowball coverage`: backward and forward citation checks are logged when the review protocol requires them.
- `native full-text candidate coverage`: `native_fulltext_queries` were executed when platform-native search was available, and their candidate URLs were logged with source-agent provenance.
- `Zotero attachment coverage`: local Zotero matches record match basis, match confidence, and attachment status so citation-only matches are not counted as full-text retrieval.
- `full-text access coverage`: every sought report has a `retrieval_manifest.csv` status and every extracted claim records `evidence_limit`.

Treat full text as confirmed only after retrieval:

- `full_text`: a readable full-text version was retrieved or verified.
- `abstract_only`: only an abstract or structured abstract was available.
- `metadata_only`: only metadata fields were available.
- `unavailable`: no reliable metadata or text source was available.
```

- [ ] **Step 5: Update search diagnostics template**

In `content/templates/search-diagnostics.md`, add fields for:

```markdown
## Coverage Semantics

- discovery_coverage_basis:
- native_fulltext_candidate_coverage:
- zotero_attachment_coverage:
- known_item_recall:
- duplicate_saturation:
- full_text_access_coverage:
- evidence_limit_distribution:
- cannot_claim_absolute_completeness: true
```

- [ ] **Step 6: Run docs tests**

Run:

```bash
uv run python -m unittest tests.test_literature_contract tests.test_mcp_provider_docs -v
```

Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add \
  docs/advanced/mcp-providers-setup.md \
  docs/advanced/rigorous-literature-search.md \
  content/workflow/references/stage-B-literature.md \
  content/templates/search-diagnostics.md \
  tests/test_literature_contract.py \
  tests/test_mcp_provider_docs.py
git commit -m "docs(literature): define search coverage and full text limits"
```

### Task 13: Run Final Verification

**Files:**
- Test-only execution.

- [ ] **Step 1: Run focused Python tests**

Run:

```bash
uv run python -m unittest \
  tests.test_mcp_connectors \
  tests.test_mcp_tool_handlers \
  tests.test_mcp_literature_tools \
  tests.test_literature_search \
  tests.test_hybrid_search_router \
  tests.test_mcp_cli \
  tests.test_mcp_tool_surface_parity \
  tests.test_literature_contract \
  tests.test_mcp_provider_docs \
  tests.test_cli_setup_docs \
  -v
```

Expected: PASS.

- [ ] **Step 2: Run Node MCPB tests**

Run:

```bash
node --test packages/qiongli-literature-mcpb/test/tools.test.mjs packages/qiongli-literature-mcpb/test/config.test.mjs packages/qiongli-literature-mcpb/test/providers.test.mjs packages/qiongli-literature-mcpb/test/zotero.test.mjs
node --test packages/qiongli-zotero-companion/test/bridge.test.mjs
```

Expected: PASS.

- [ ] **Step 3: Run stale wording scan**

Run:

```bash
rg -n "Full runtime commands, including `qiongli_literature_search`|需要 `qiongli_literature_search`、`qiongli_task_plan`|OpenAlex and Semantic Scholar configuration|qiongli_collect_evidence.*provider readiness|provider readiness.*qiongli_collect_evidence" README.md README_CN.md docs content packages tests -S
```

Expected: no matches.

- [ ] **Step 4: Run boundary and secret scan**

Run:

```bash
rg -n "openalex-secret-key|semantic.*secret|api[_-]?key.*[A-Za-z0-9]{20,}|/(Users|private/tmp)/" README.md README_CN.md docs content packages tests -S
```

Expected: no real secrets or machine-local paths. Test fixture strings such as `openalex-secret-key` may appear only inside tests that assert secrets are not echoed.

- [ ] **Step 5: Commit verification-only updates if any were needed**

If verification required small test/doc corrections, commit them:

```bash
git add <corrected-files>
git commit -m "test(mcp): lock provider status parity regressions"
```

If no corrections were needed, do not create an empty commit.

## Release Notes Draft

```markdown
### Fixed

- Clarified that `qiongli_collect_evidence` reports external command-adapter readiness, not built-in OpenAlex/Semantic Scholar/Crossref/PubMed/arXiv provider configuration.
- Added `qiongli_search_plan` to Python full MCP literature tool inventory output.
- Updated MCP and install docs so bundled Node literature MCP and full CLI MCP both advertise provider status, search planning, literature search, and evidence export accurately.
- Added native full-text candidate planning so active agents can use platform-native LLM search for auditable PDF/full-text candidate discovery without treating snippets as retrieved evidence.
- Added Zotero attachment verification planning so local Zotero records can distinguish citation-only matches from verified local PDF/full-text attachment matches.

### Tests

- Added regression coverage for direct `qiongli_collect_evidence` academic-provider calls with configured provider credentials.
- Added cross-surface parity checks for Python full MCP and Node MCPB literature-provider tools.
- Added native full-text query planning and Zotero attachment normalization coverage.
```

## Self-Review

- Spec coverage: The plan covers the root OpenAlex misdiagnosis, Python MCP tool inventory drift, Node/Python surface parity, install docs, workflow guidance, generated guidance, search default widening, full-text candidate preservation, platform-native LLM full-text candidate search, Zotero attachment verification, coverage semantics, and repo-boundary checks.
- Placeholder scan: No task relies on unspecified implementation. Each code-changing task includes concrete snippets and exact verification commands.
- Type consistency: New diagnostics use `MCPEvidence.data` as `dict[str, Any]`, existing `provider_config_summary(...)` return type as `dict[str, str]`, and existing MCP result shapes through `structuredContent`.

Plan complete and saved to `docs/superpowers/plans/2026-07-03-mcp-provider-status-parity.md`. Two execution options:

**1. Subagent-Driven (recommended)** - dispatch a fresh subagent per task, review between tasks, fast iteration.

**2. Inline Execution** - execute tasks in this session using executing-plans, batch execution with checkpoints.
