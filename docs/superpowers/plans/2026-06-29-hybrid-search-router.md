# Hybrid Search Router Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let Qiongli coordinate provider-backed MCP literature search with Codex/Claude platform-native search without making MCP servers call platform search directly.

**Architecture:** Add a deterministic `qiongli_search_plan` MCP tool that returns provider queries, native-search queries, search execution mode, provenance labels, and agent instructions. The MCP tool plans native searches but does not execute Codex/Claude native search; the active agent executes platform-native search when available, then Qiongli workflows merge and log provider, native, and user-corpus evidence separately. Mirror the contract in the Python full MCP and the Node literature MCPB runtime so Codex, Claude Code, and Desktop/plugin bundles expose the same planning semantics.

**Tech Stack:** Python 3.12, unittest, Node.js ESM MCPB runtime, `node:test`, Qiongli workflow Markdown contracts, Codex/Claude local plugin artifacts.

---

## File Structure

- Create `packages/python-qiongli/src/qiongli/bridges/hybrid_search_router.py` for the shared Python search-plan contract and mode selection.
- Modify `packages/python-qiongli/src/qiongli/bridges/literature_mcp_tools.py` to expose `qiongli_search_plan` in full Python MCP.
- Modify `packages/python-qiongli/src/qiongli/bridges/mcp_tool_handlers.py` to route `qiongli_search_plan`.
- Create `tests/test_hybrid_search_router.py` for pure mode-selection tests.
- Modify `tests/test_mcp_tool_handlers.py` and `tests/test_mcp_stdio_server.py` for Python MCP tool exposure.
- Create `packages/qiongli-literature-mcpb/server/search-plan.mjs` for the Node MCPB search-plan contract.
- Modify `packages/qiongli-literature-mcpb/server/index.mjs` and `packages/qiongli-literature-mcpb/manifest.json` to expose `qiongli_search_plan`.
- Modify `packages/qiongli-literature-mcpb/test/tools.test.mjs` and `tests/test_literature_mcpb_artifact.py` for Node MCPB parity.
- Modify workflow source files: `content/workflow/SKILL.md`, `content/workflow/workflows/lit-review.md`, `content/workflow/workflows/paper-read.md`, and `content/skills/B_literature/academic-searcher.md`.
- Modify docs: `docs/advanced/cross-platform-mcp.md`, `docs/advanced/mcp-providers-setup.md`, `docs/guide/troubleshooting.md`, `docs/zh/advanced/mcp-providers-setup.md`, and `docs/zh/guide/troubleshooting.md`.

### Task 1: Add Python Hybrid Search Plan Contract

**Files:**
- Create: `packages/python-qiongli/src/qiongli/bridges/hybrid_search_router.py`
- Create: `tests/test_hybrid_search_router.py`

- [ ] **Step 1: Write failing mode-selection tests**

Create `tests/test_hybrid_search_router.py`:

```python
from __future__ import annotations

import unittest

from bridges.hybrid_search_router import build_hybrid_search_plan


class HybridSearchRouterTests(unittest.TestCase):
    def test_provider_and_native_search_yield_hybrid_mode(self) -> None:
        plan = build_hybrid_search_plan(
            {
                "query": "AI adoption in accounting disclosure",
                "platform": "codex",
                "native_search_available": True,
                "native_search_tools": ["codex_web_search"],
                "fromYear": 2023,
                "toYear": 2026,
                "include_working_papers": True,
            },
            provider_capability_mode="provider_connected",
        )

        self.assertEqual(plan["search_execution_mode"], "hybrid_search")
        self.assertEqual(plan["provider_capability_mode"], "provider_connected")
        self.assertTrue(plan["native_search_available"])
        self.assertIn("mcp:openalex", plan["provenance_labels"])
        self.assertIn("mcp:semantic_scholar", plan["provenance_labels"])
        self.assertIn("native:codex_web_search", plan["provenance_labels"])
        self.assertGreaterEqual(len(plan["provider_queries"]), 1)
        self.assertGreaterEqual(len(plan["native_search_queries"]), 1)
        self.assertEqual(plan["execution_sequence"][0]["actor"], "agent")
        self.assertEqual(plan["execution_sequence"][0]["action"], "call qiongli_literature_status")
        self.assertIn("Do not treat native-search results as provider-reproducible records.", plan["agent_instructions"])

    def test_provider_only_mode_when_native_search_unavailable(self) -> None:
        plan = build_hybrid_search_plan(
            {
                "query": "audit committee expertise disclosure",
                "platform": "cli",
                "native_search_available": False,
            },
            provider_capability_mode="provider_connected",
        )

        self.assertEqual(plan["search_execution_mode"], "provider_connected")
        self.assertFalse(plan["native_search_available"])
        self.assertEqual(plan["native_search_queries"], [])
        self.assertIn("Platform-native search was not declared available.", plan["limitations"])

    def test_native_only_mode_when_provider_missing_but_native_available(self) -> None:
        plan = build_hybrid_search_plan(
            {
                "query": "SSRN corporate disclosure generative AI",
                "platform": "claude_code",
                "native_search_available": True,
                "native_search_tools": ["claude_web_search"],
                "include_working_papers": True,
            },
            provider_capability_mode="strategy_only",
        )

        self.assertEqual(plan["search_execution_mode"], "native_only")
        self.assertIn("native:claude_web_search", plan["provenance_labels"])
        self.assertIn("Provider MCP search is unavailable; native results require explicit provenance labels.", plan["limitations"])

    def test_strategy_only_when_no_provider_and_no_native_search(self) -> None:
        plan = build_hybrid_search_plan(
            {
                "query": "earnings call disclosure",
                "platform": "unknown",
                "native_search_available": False,
            },
            provider_capability_mode="strategy_only",
        )

        self.assertEqual(plan["search_execution_mode"], "strategy_only")
        self.assertEqual(plan["provider_queries"], [])
        self.assertEqual(plan["native_search_queries"], [])
        self.assertIn("No provider MCP search or platform-native search is available.", plan["limitations"])

    def test_empty_query_returns_structured_strategy_gap(self) -> None:
        plan = build_hybrid_search_plan(
            {"query": "  ", "platform": "codex", "native_search_available": True},
            provider_capability_mode="provider_connected",
        )

        self.assertEqual(plan["search_execution_mode"], "strategy_only")
        self.assertIn("Search query is empty.", plan["limitations"])
        self.assertEqual(plan["provider_queries"], [])
        self.assertEqual(plan["native_search_queries"], [])


if __name__ == "__main__":
    unittest.main()
```

- [ ] **Step 2: Run test to verify red**

Run:

```bash
python3 -m unittest tests.test_hybrid_search_router -v
```

Expected: FAIL with `ModuleNotFoundError: No module named 'bridges.hybrid_search_router'`.

- [ ] **Step 3: Implement the Python router**

Create `packages/python-qiongli/src/qiongli/bridges/hybrid_search_router.py`:

```python
from __future__ import annotations

from typing import Any

PROVIDER_LABELS = ("mcp:semantic_scholar", "mcp:openalex", "mcp:crossref", "mcp:pubmed", "mcp:arxiv")
DEFAULT_NATIVE_TOOLS = {
    "codex": "codex_web_search",
    "claude": "claude_web_search",
    "claude_code": "claude_web_search",
    "antigravity": "antigravity_search",
}


def build_hybrid_search_plan(
    args: dict[str, Any],
    *,
    provider_capability_mode: str,
) -> dict[str, Any]:
    query = str(args.get("query", "") or "").strip()
    platform = _platform(args.get("platform"))
    native_available = bool(args.get("native_search_available", False))
    native_tools = _native_tools(args, platform, native_available)
    limitations: list[str] = []

    if not query:
        return _empty_plan(
            query=query,
            platform=platform,
            provider_capability_mode=provider_capability_mode,
            native_available=native_available,
            native_tools=native_tools,
            limitations=["Search query is empty."],
        )

    provider_connected = provider_capability_mode == "provider_connected"
    if provider_connected and native_available:
        mode = "hybrid_search"
    elif provider_connected:
        mode = "provider_connected"
        limitations.append("Platform-native search was not declared available.")
    elif native_available:
        mode = "native_only"
        limitations.append("Provider MCP search is unavailable; native results require explicit provenance labels.")
    else:
        return _empty_plan(
            query=query,
            platform=platform,
            provider_capability_mode=provider_capability_mode,
            native_available=native_available,
            native_tools=native_tools,
            limitations=["No provider MCP search or platform-native search is available."],
        )

    provider_queries = _provider_queries(args, query) if provider_connected else []
    native_queries = _native_queries(args, query) if native_available else []
    provenance = []
    if provider_queries:
        provenance.extend(PROVIDER_LABELS)
    provenance.extend(f"native:{tool}" for tool in native_tools)
    provenance.append("user_corpus")

    return {
        "artifact_type": "qiongli_hybrid_search_plan",
        "query": query,
        "platform": platform,
        "search_execution_mode": mode,
        "provider_capability_mode": provider_capability_mode,
        "native_search_available": native_available,
        "native_search_tools": native_tools,
        "provider_queries": provider_queries,
        "native_search_queries": native_queries,
        "provenance_labels": provenance,
        "execution_sequence": _execution_sequence(mode),
        "agent_instructions": _agent_instructions(),
        "merge_policy": {
            "dedupe_order": ["doi", "provider_id", "title_year_author"],
            "preserve_source_rows": True,
            "require_search_log": True,
        },
        "limitations": limitations,
    }


def _empty_plan(
    *,
    query: str,
    platform: str,
    provider_capability_mode: str,
    native_available: bool,
    native_tools: list[str],
    limitations: list[str],
) -> dict[str, Any]:
    return {
        "artifact_type": "qiongli_hybrid_search_plan",
        "query": query,
        "platform": platform,
        "search_execution_mode": "strategy_only",
        "provider_capability_mode": provider_capability_mode,
        "native_search_available": native_available,
        "native_search_tools": native_tools,
        "provider_queries": [],
        "native_search_queries": [],
        "provenance_labels": ["user_corpus"],
        "execution_sequence": _execution_sequence("strategy_only"),
        "agent_instructions": _agent_instructions(),
        "merge_policy": {
            "dedupe_order": ["doi", "provider_id", "title_year_author"],
            "preserve_source_rows": True,
            "require_search_log": True,
        },
        "limitations": limitations,
    }


def _provider_queries(args: dict[str, Any], query: str) -> list[dict[str, Any]]:
    return [
        {
            "query_id": "P1",
            "query": query,
            "search_mode": str(args.get("search_mode", args.get("searchMode", "review")) or "review"),
            "fromYear": args.get("fromYear"),
            "toYear": args.get("toYear"),
            "venue_filter": args.get("venue_filter", args.get("venueFilter", "")),
            "document_types": args.get("document_types", args.get("documentTypes", [])),
        }
    ]


def _native_queries(args: dict[str, Any], query: str) -> list[dict[str, Any]]:
    queries = [
        {
            "query_id": "N1",
            "query": query,
            "purpose": "current web/PDF/working-paper supplement",
        }
    ]
    if bool(args.get("include_working_papers", args.get("includeWorkingPapers", False))):
        queries.append(
            {
                "query_id": "N2",
                "query": f'{query} SSRN OR NBER OR CEPR OR working paper filetype:pdf',
                "purpose": "working paper and author PDF supplement",
            }
        )
    return queries


def _native_tools(args: dict[str, Any], platform: str, native_available: bool) -> list[str]:
    raw = args.get("native_search_tools", args.get("nativeSearchTools", []))
    if isinstance(raw, list):
        tools = [str(item).strip() for item in raw if str(item).strip()]
    else:
        tools = []
    if not tools and native_available:
        tools = [DEFAULT_NATIVE_TOOLS.get(platform, "platform_native_search")]
    return tools


def _execution_sequence(mode: str) -> list[dict[str, str]]:
    sequence = [
        {"actor": "agent", "action": "call qiongli_literature_status"},
        {"actor": "agent", "action": "call qiongli_search_plan"},
    ]
    if mode in {"provider_connected", "hybrid_search"}:
        sequence.append({"actor": "agent", "action": "call qiongli_literature_search for provider_queries"})
    if mode in {"native_only", "hybrid_search"}:
        sequence.append({"actor": "agent", "action": "execute platform-native search for native_search_queries"})
    sequence.append({"actor": "agent", "action": "merge, dedupe, and write provenance-labelled search_log.md"})
    return sequence


def _agent_instructions() -> list[str]:
    return [
        "MCP servers must not call Codex or Claude native search directly.",
        "The active agent executes native_search_queries only when the platform exposes native search.",
        "Do not treat native-search results as provider-reproducible records.",
        "Write provider, native, and user-corpus records with distinct provenance labels.",
    ]


def _platform(raw: Any) -> str:
    value = str(raw or "unknown").strip().lower().replace("-", "_")
    return value if value in {"codex", "claude", "claude_code", "antigravity", "cli"} else "unknown"
```

- [ ] **Step 4: Run pure router tests**

Run:

```bash
python3 -m unittest tests.test_hybrid_search_router -v
```

Expected: PASS.

### Task 2: Expose `qiongli_search_plan` In Python MCP

**Files:**
- Modify: `packages/python-qiongli/src/qiongli/bridges/literature_mcp_tools.py`
- Modify: `packages/python-qiongli/src/qiongli/bridges/mcp_tool_handlers.py`
- Modify: `tests/test_mcp_tool_handlers.py`
- Modify: `tests/test_mcp_stdio_server.py`

- [ ] **Step 1: Write failing MCP tool handler tests**

In `tests/test_mcp_tool_handlers.py`, update `test_tool_definitions_include_config_and_evidence_tools` expected names to include:

```python
"qiongli_search_plan",
```

Add this test:

```python
    def test_search_plan_tool_returns_hybrid_plan_without_running_native_search(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            with mock.patch.dict(
                "os.environ",
                {"QIONGLI_CONFIG_HOME": str(root / "config")},
                clear=False,
            ):
                set_provider_value("openalex", "api-key", "openalex-secret-key")
                result = call_qiongli_tool(
                    "qiongli_search_plan",
                    {
                        "query": "AI disclosure accounting working papers",
                        "platform": "codex",
                        "native_search_available": True,
                        "native_search_tools": ["codex_web_search"],
                        "include_working_papers": True,
                    },
                )

        payload = result["structuredContent"]
        rendered = json.dumps(result, sort_keys=True)
        self.assertFalse(result["isError"])
        self.assertEqual(payload["search_execution_mode"], "hybrid_search")
        self.assertEqual(payload["provider_capability_mode"], "provider_connected")
        self.assertIn("native:codex_web_search", payload["provenance_labels"])
        self.assertIn("call qiongli_literature_search", json.dumps(payload["execution_sequence"]))
        self.assertIn("execute platform-native search", json.dumps(payload["execution_sequence"]))
        self.assertNotIn("openalex-secret-key", rendered)
```

In `tests/test_mcp_stdio_server.py`, add:

```python
self.assertIn("qiongli_search_plan", tool_names)
```

- [ ] **Step 2: Run tests to verify red**

Run:

```bash
python3 -m unittest tests.test_mcp_tool_handlers tests.test_mcp_stdio_server -v
```

Expected: FAIL because `qiongli_search_plan` is not defined or routed.

- [ ] **Step 3: Add Python MCP tool definition and handler**

In `packages/python-qiongli/src/qiongli/bridges/literature_mcp_tools.py`, import the router:

```python
from bridges.hybrid_search_router import build_hybrid_search_plan
```

Add this tool declaration immediately after `qiongli_literature_status`:

```python
    {
        "name": "qiongli_search_plan",
        "description": (
            "Plan hybrid Qiongli literature search across MCP providers and platform-native search. "
            "This tool returns native-search instructions but does not execute Codex or Claude native search."
        ),
        "inputSchema": {
            "type": "object",
            "properties": {
                "query": {"type": "string"},
                "platform": {
                    "type": "string",
                    "enum": ["codex", "claude", "claude_code", "antigravity", "cli", "unknown"],
                    "default": "unknown",
                },
                "native_search_available": {"type": "boolean", "default": False},
                "native_search_tools": {"type": "array", "items": {"type": "string"}},
                "include_working_papers": {"type": "boolean", "default": False},
                "fromYear": {"type": ["integer", "string"]},
                "toYear": {"type": ["integer", "string"]},
                "search_mode": {
                    "type": "string",
                    "enum": ["auto", "topic", "title", "doi", "review", "systematic_review"],
                },
                "venue_filter": {"type": "string"},
                "document_types": {"type": "array", "items": {"type": "string"}},
            },
            "additionalProperties": True,
        },
    },
```

Add:

```python
def handle_search_plan(args: dict[str, Any]) -> dict[str, Any]:
    status = handle_literature_status(args)
    return build_hybrid_search_plan(
        args,
        provider_capability_mode=str(status.get("capability_mode") or "strategy_only"),
    )
```

In `packages/python-qiongli/src/qiongli/bridges/mcp_tool_handlers.py`, import and route:

```python
    handle_search_plan,
```

Add to `handlers`:

```python
"qiongli_search_plan": handle_search_plan,
```

- [ ] **Step 4: Run MCP Python tests**

Run:

```bash
python3 -m unittest tests.test_hybrid_search_router tests.test_mcp_tool_handlers tests.test_mcp_stdio_server -v
```

Expected: PASS.

### Task 3: Mirror `qiongli_search_plan` In Node Literature MCPB

**Files:**
- Create: `packages/qiongli-literature-mcpb/server/search-plan.mjs`
- Modify: `packages/qiongli-literature-mcpb/server/index.mjs`
- Modify: `packages/qiongli-literature-mcpb/manifest.json`
- Modify: `packages/qiongli-literature-mcpb/test/tools.test.mjs`
- Modify: `tests/test_literature_mcpb_artifact.py`

- [ ] **Step 1: Write failing Node MCPB tests**

In `packages/qiongli-literature-mcpb/test/tools.test.mjs`, update the tool declaration name list to include `"qiongli_search_plan"` immediately after `"qiongli_literature_status"`.

Add this import:

```js
  handleSearchPlan,
```

Add these tests:

```js
test("search plan returns hybrid mode without executing native search", () => {
  const plan = handleSearchPlan({
    query: "AI disclosure accounting working papers",
    platform: "codex",
    native_search_available: true,
    native_search_tools: ["codex_web_search"],
    include_working_papers: true
  }, {
    env: {
      QIONGLI_MCPB_OPENALEX_API_KEY: "openalex-secret"
    }
  });

  const rendered = JSON.stringify(plan);
  assert.equal(plan.search_execution_mode, "hybrid_search");
  assert.equal(plan.provider_capability_mode, "provider_connected");
  assert.ok(plan.provenance_labels.includes("native:codex_web_search"));
  assert.ok(JSON.stringify(plan.execution_sequence).includes("execute platform-native search"));
  assert.ok(!rendered.includes("openalex-secret"));
});

test("search plan returns native-only mode when providers are missing but native search is available", () => {
  const plan = handleSearchPlan({
    query: "SSRN generative AI disclosure",
    platform: "claude_code",
    native_search_available: true,
    native_search_tools: ["claude_web_search"]
  }, {
    env: {}
  });

  assert.equal(plan.search_execution_mode, "native_only");
  assert.equal(plan.provider_capability_mode, "strategy_only");
  assert.ok(plan.limitations.includes("Provider MCP search is unavailable; native results require explicit provenance labels."));
});
```

In `tests/test_literature_mcpb_artifact.py`, add `"qiongli_search_plan"` to `test_literature_mcpb_manifest_declares_expected_tools`.

- [ ] **Step 2: Run Node and artifact tests to verify red**

Run:

```bash
npm --prefix packages/qiongli-literature-mcpb test
python3 -m unittest tests.test_literature_mcpb_artifact -v
```

Expected: FAIL because Node MCPB has no `qiongli_search_plan` declaration, handler, or manifest entry.

- [ ] **Step 3: Implement Node search-plan helper**

Create `packages/qiongli-literature-mcpb/server/search-plan.mjs`:

```js
const PROVIDER_LABELS = ["mcp:semantic_scholar", "mcp:openalex", "mcp:crossref", "mcp:pubmed", "mcp:arxiv"];
const DEFAULT_NATIVE_TOOLS = {
  codex: "codex_web_search",
  claude: "claude_web_search",
  claude_code: "claude_web_search",
  antigravity: "antigravity_search"
};

export function buildHybridSearchPlan(input = {}, providerCapabilityMode = "strategy_only") {
  const query = String(input.query || "").trim();
  const platform = normalizePlatform(input.platform);
  const nativeSearchAvailable = Boolean(input.native_search_available || input.nativeSearchAvailable);
  const nativeSearchTools = nativeTools(input, platform, nativeSearchAvailable);

  if (!query) {
    return emptyPlan({
      query,
      platform,
      providerCapabilityMode,
      nativeSearchAvailable,
      nativeSearchTools,
      limitations: ["Search query is empty."]
    });
  }

  const providerConnected = providerCapabilityMode === "provider_connected";
  let searchExecutionMode = "strategy_only";
  const limitations = [];

  if (providerConnected && nativeSearchAvailable) {
    searchExecutionMode = "hybrid_search";
  } else if (providerConnected) {
    searchExecutionMode = "provider_connected";
    limitations.push("Platform-native search was not declared available.");
  } else if (nativeSearchAvailable) {
    searchExecutionMode = "native_only";
    limitations.push("Provider MCP search is unavailable; native results require explicit provenance labels.");
  } else {
    return emptyPlan({
      query,
      platform,
      providerCapabilityMode,
      nativeSearchAvailable,
      nativeSearchTools,
      limitations: ["No provider MCP search or platform-native search is available."]
    });
  }

  const providerQueries = providerConnected ? providerQueriesFor(input, query) : [];
  const nativeSearchQueries = nativeSearchAvailable ? nativeQueriesFor(input, query) : [];
  const provenanceLabels = [];
  if (providerQueries.length > 0) {
    provenanceLabels.push(...PROVIDER_LABELS);
  }
  provenanceLabels.push(...nativeSearchTools.map((tool) => `native:${tool}`));
  provenanceLabels.push("user_corpus");

  return {
    artifact_type: "qiongli_hybrid_search_plan",
    query,
    platform,
    search_execution_mode: searchExecutionMode,
    provider_capability_mode: providerCapabilityMode,
    native_search_available: nativeSearchAvailable,
    native_search_tools: nativeSearchTools,
    provider_queries: providerQueries,
    native_search_queries: nativeSearchQueries,
    provenance_labels: provenanceLabels,
    execution_sequence: executionSequence(searchExecutionMode),
    agent_instructions: agentInstructions(),
    merge_policy: mergePolicy(),
    limitations
  };
}

function emptyPlan({ query, platform, providerCapabilityMode, nativeSearchAvailable, nativeSearchTools, limitations }) {
  return {
    artifact_type: "qiongli_hybrid_search_plan",
    query,
    platform,
    search_execution_mode: "strategy_only",
    provider_capability_mode: providerCapabilityMode,
    native_search_available: nativeSearchAvailable,
    native_search_tools: nativeSearchTools,
    provider_queries: [],
    native_search_queries: [],
    provenance_labels: ["user_corpus"],
    execution_sequence: executionSequence("strategy_only"),
    agent_instructions: agentInstructions(),
    merge_policy: mergePolicy(),
    limitations
  };
}

function providerQueriesFor(input, query) {
  return [{
    query_id: "P1",
    query,
    search_mode: String(input.search_mode || input.searchMode || "review"),
    fromYear: input.fromYear,
    toYear: input.toYear,
    venue_filter: input.venue_filter || input.venueFilter || "",
    document_types: input.document_types || input.documentTypes || []
  }];
}

function nativeQueriesFor(input, query) {
  const queries = [{
    query_id: "N1",
    query,
    purpose: "current web/PDF/working-paper supplement"
  }];
  if (Boolean(input.include_working_papers || input.includeWorkingPapers)) {
    queries.push({
      query_id: "N2",
      query: `${query} SSRN OR NBER OR CEPR OR working paper filetype:pdf`,
      purpose: "working paper and author PDF supplement"
    });
  }
  return queries;
}

function nativeTools(input, platform, nativeSearchAvailable) {
  const raw = input.native_search_tools || input.nativeSearchTools || [];
  const tools = Array.isArray(raw) ? raw.map((item) => String(item).trim()).filter(Boolean) : [];
  if (tools.length === 0 && nativeSearchAvailable) {
    return [DEFAULT_NATIVE_TOOLS[platform] || "platform_native_search"];
  }
  return tools;
}

function executionSequence(mode) {
  const sequence = [
    { actor: "agent", action: "call qiongli_literature_status" },
    { actor: "agent", action: "call qiongli_search_plan" }
  ];
  if (["provider_connected", "hybrid_search"].includes(mode)) {
    sequence.push({ actor: "agent", action: "call qiongli_literature_search for provider_queries" });
  }
  if (["native_only", "hybrid_search"].includes(mode)) {
    sequence.push({ actor: "agent", action: "execute platform-native search for native_search_queries" });
  }
  sequence.push({ actor: "agent", action: "merge, dedupe, and write provenance-labelled search_log.md" });
  return sequence;
}

function agentInstructions() {
  return [
    "MCP servers must not call Codex or Claude native search directly.",
    "The active agent executes native_search_queries only when the platform exposes native search.",
    "Do not treat native-search results as provider-reproducible records.",
    "Write provider, native, and user-corpus records with distinct provenance labels."
  ];
}

function mergePolicy() {
  return {
    dedupe_order: ["doi", "provider_id", "title_year_author"],
    preserve_source_rows: true,
    require_search_log: true
  };
}

function normalizePlatform(raw) {
  const value = String(raw || "unknown").trim().toLowerCase().replace(/-/g, "_");
  return ["codex", "claude", "claude_code", "antigravity", "cli"].includes(value) ? value : "unknown";
}
```

- [ ] **Step 4: Wire Node MCPB declarations and handler**

In `packages/qiongli-literature-mcpb/server/index.mjs`, import:

```js
import { buildHybridSearchPlan } from "./search-plan.mjs";
```

Add a `TOOL_DECLARATIONS` entry immediately after `qiongli_literature_status`:

```js
  {
    name: "qiongli_search_plan",
    description: "Plan hybrid Qiongli literature search across MCP providers and platform-native search. This tool returns native-search instructions but does not execute Codex or Claude native search.",
    inputSchema: {
      type: "object",
      additionalProperties: true,
      properties: {
        query: { type: "string" },
        platform: {
          type: "string",
          enum: ["codex", "claude", "claude_code", "antigravity", "cli", "unknown"],
          default: "unknown"
        },
        native_search_available: { type: "boolean", default: false },
        native_search_tools: { type: "array", items: { type: "string" } },
        include_working_papers: { type: "boolean", default: false },
        fromYear: { type: ["integer", "string"] },
        toYear: { type: ["integer", "string"] },
        search_mode: {
          type: "string",
          enum: ["auto", "topic", "title", "doi", "review", "systematic_review"]
        },
        venue_filter: { type: "string" },
        document_types: { type: "array", items: { type: "string" } }
      }
    }
  },
```

Export:

```js
export function handleSearchPlan(input = {}, context = {}) {
  const status = handleStatus(context);
  return buildHybridSearchPlan(input, status.capability_mode || "strategy_only");
}
```

In `handleToolCall`, add before `qiongli_literature_search`:

```js
  if (name === "qiongli_search_plan") {
    return toolResult(handleSearchPlan(input, context));
  }
```

In `packages/qiongli-literature-mcpb/manifest.json`, add a matching `qiongli_search_plan` tool entry after `qiongli_literature_status`.

- [ ] **Step 5: Run Node MCPB parity tests**

Run:

```bash
npm --prefix packages/qiongli-literature-mcpb test
python3 -m unittest tests.test_literature_mcpb_artifact tests.test_plugin_distribution_contract tests.test_plugin_artifacts -v
```

Expected: PASS. Plugin artifact tests should pass because `tooling/scripts/build_plugin_artifacts.py` copies `packages/qiongli-literature-mcpb/server/` into plugin payloads.

### Task 4: Update Workflow Contracts For Hybrid Search Execution

**Files:**
- Modify: `content/workflow/SKILL.md`
- Modify: `content/workflow/workflows/lit-review.md`
- Modify: `content/workflow/workflows/paper-read.md`
- Modify: `content/skills/B_literature/academic-searcher.md`
- Modify: `tests/test_literature_contract.py`
- Modify: `tests/test_mcp_provider_docs.py`
- Modify if generated text requires it: `packages/python-qiongli/src/qiongli/subject_materializer.py`

- [ ] **Step 1: Write failing workflow contract tests**

In `tests/test_literature_contract.py`, add:

```python
    def test_literature_workflows_document_hybrid_search_router(self) -> None:
        workflow_paths = (
            LAYOUT.workflow / "SKILL.md",
            LAYOUT.workflow / "workflows" / "lit-review.md",
            LAYOUT.workflow / "workflows" / "paper-read.md",
            LAYOUT.skills / "B_literature" / "academic-searcher.md",
        )

        required_tokens = (
            "qiongli_search_plan",
            "hybrid_search",
            "native_only",
            "provider_connected",
            "strategy_only",
            "provenance labels",
            "MCP servers must not call Codex or Claude native search directly",
        )

        for path in workflow_paths:
            with self.subTest(path=str(path.relative_to(REPO_ROOT))):
                content = path.read_text(encoding="utf-8")
                for token in required_tokens:
                    self.assertIn(token, content)
```

In `tests/test_mcp_provider_docs.py`, add:

```python
    def test_docs_explain_hybrid_search_router_boundary(self) -> None:
        content = (REPO_ROOT / "content" / "workflow" / "SKILL.md").read_text(encoding="utf-8")
        self.assertIn("qiongli_search_plan", content)
        self.assertIn("hybrid_search", content)
        self.assertIn("native_only", content)
        self.assertIn("MCP servers must not call Codex or Claude native search directly", content)
```

- [ ] **Step 2: Run tests to verify red**

Run:

```bash
python3 -m unittest tests.test_literature_contract tests.test_mcp_provider_docs -v
```

Expected: FAIL because the workflow docs do not yet define `qiongli_search_plan`, `hybrid_search`, or `native_only`.

- [ ] **Step 3: Update `content/workflow/SKILL.md`**

Under `## Literature Provider Configuration`, add:

```markdown
- For literature search, literature review, paper reading metadata lookup, gap finding, citation snowballing, and evidence synthesis, use `qiongli_search_plan` after `qiongli_literature_status` when the tool is visible. The plan selects `search_execution_mode`: `hybrid_search` when provider MCP and platform-native search are both available; `provider_connected` when only provider MCP is available; `native_only` when provider MCP is unavailable but Codex/Claude platform-native search is available; and `strategy_only` only when neither provider MCP nor platform-native search nor user corpus is available.
- MCP servers must not call Codex or Claude native search directly. The active agent executes `native_search_queries` returned by `qiongli_search_plan` with the platform-native search capability when available, then merges those results with provider records.
- Preserve provenance labels in all search artifacts: `mcp:openalex`, `mcp:semantic_scholar`, `mcp:crossref`, `mcp:pubmed`, `mcp:arxiv`, `native:codex_web_search`, `native:claude_web_search`, and `user_corpus`. Native-search results are supplemental unless the workflow explicitly marks the run as `native_only`; do not present them as provider-reproducible records.
```

- [ ] **Step 4: Update `lit-review.md` execution phases**

In `content/workflow/workflows/lit-review.md`, replace the Phase 2 item that records only `provider_connected` / `strategy_only` with:

```markdown
4. Call `qiongli_literature_status` when visible, then call `qiongli_search_plan` with:
   - `query`
   - `platform`
   - `native_search_available`
   - `native_search_tools`
   - date range and document filters
5. Record `search_execution_mode` as one of `hybrid_search`, `provider_connected`, `native_only`, or `strategy_only`.
6. Record provider capability mode separately as `provider_connected` or `strategy_only`.
```

In Phase 3, replace the supplemental native-search item with:

```markdown
4. If `search_execution_mode` is `hybrid_search` or `native_only`, execute the `native_search_queries` returned by `qiongli_search_plan` with the active platform's native search capability. MCP servers must not call Codex or Claude native search directly.
5. Log provider and native searches in `search_log.md` with distinct provenance labels, then normalize and dedupe records without merging away source provenance.
```

- [ ] **Step 5: Update `paper-read.md` retrieval rules**

In `content/workflow/workflows/paper-read.md`, replace the capability-mode paragraph in Step 1 with:

```markdown
Record `search_execution_mode` from `qiongli_search_plan` as `hybrid_search`, `provider_connected`, `native_only`, or `strategy_only`, and record provider capability mode separately as `provider_connected` or `strategy_only`. If no MCP/provider, platform-native search, or user-supplied metadata is available, use user-supplied metadata only and keep that evidence boundary visible. MCP servers must not call Codex or Claude native search directly; the active agent executes platform-native lookups when available.
```

- [ ] **Step 6: Update `academic-searcher.md` ownership boundary**

In `content/skills/B_literature/academic-searcher.md`, add after the provider ownership table:

```markdown
Hybrid search coordination belongs to the Qiongli workflow/router layer. Use `qiongli_search_plan` to decide whether a run is `hybrid_search`, `provider_connected`, `native_only`, or `strategy_only`. The MCP/provider layer owns provider calls and raw provider hit capture; the active agent owns any platform-native search calls; this skill owns logging, normalization, deduplication, and diagnostics. MCP servers must not call Codex or Claude native search directly.
```

- [ ] **Step 7: Run workflow contract tests**

Run:

```bash
python3 -m unittest tests.test_literature_contract tests.test_mcp_provider_docs tests.test_subject_materializer -v
```

Expected: PASS. If subject materialization tests fail because hardcoded provider guidance in `subject_materializer.py` is missing the new text, add the same concise hybrid-search bullets to the materializer's injected fallback text.

### Task 5: Update User-Facing Docs

**Files:**
- Modify: `docs/advanced/cross-platform-mcp.md`
- Modify: `docs/advanced/mcp-providers-setup.md`
- Modify: `docs/guide/troubleshooting.md`
- Modify: `docs/zh/advanced/mcp-providers-setup.md`
- Modify: `docs/zh/guide/troubleshooting.md`
- Modify: `tests/test_mcp_provider_docs.py`
- Modify: `tests/test_cli_setup_docs.py`

- [ ] **Step 1: Write failing docs tests**

In `tests/test_mcp_provider_docs.py`, add:

```python
    def test_provider_docs_explain_hybrid_search_modes(self) -> None:
        docs = {
            "docs/advanced/cross-platform-mcp.md": (REPO_ROOT / "docs" / "advanced" / "cross-platform-mcp.md").read_text(encoding="utf-8"),
            "docs/advanced/mcp-providers-setup.md": (REPO_ROOT / "docs" / "advanced" / "mcp-providers-setup.md").read_text(encoding="utf-8"),
            "docs/zh/advanced/mcp-providers-setup.md": (REPO_ROOT / "docs" / "zh" / "advanced" / "mcp-providers-setup.md").read_text(encoding="utf-8"),
        }

        for label, content in docs.items():
            with self.subTest(label=label):
                self.assertIn("qiongli_search_plan", content)
                self.assertIn("hybrid_search", content)
                self.assertIn("native_only", content)
                self.assertIn("MCP servers", content)
                self.assertIn("native search", content)
```

In `tests/test_cli_setup_docs.py`, extend the troubleshooting tests to assert:

```python
self.assertIn("hybrid_search", troubleshooting)
self.assertIn("native_only", troubleshooting)
```

and in the Chinese troubleshooting test:

```python
self.assertIn("hybrid_search", troubleshooting)
self.assertIn("native_only", troubleshooting)
```

- [ ] **Step 2: Run docs tests to verify red**

Run:

```bash
python3 -m unittest tests.test_mcp_provider_docs tests.test_cli_setup_docs -v
```

Expected: FAIL because the docs do not yet describe `qiongli_search_plan` and hybrid modes.

- [ ] **Step 3: Update English docs**

In `docs/advanced/cross-platform-mcp.md`, add to the literature tools list:

```markdown
- `qiongli_search_plan`: returns provider queries, platform-native search queries, `search_execution_mode`, provenance labels, and merge instructions. It never executes Codex or Claude native search itself.
```

In `docs/advanced/mcp-providers-setup.md`, add a "Hybrid Search Router" section:

```markdown
## Hybrid Search Router

Use `qiongli_search_plan` after `qiongli_literature_status` for literature review, paper lookup, gap finding, citation snowballing, and evidence synthesis. The tool returns one of four execution modes:

| Mode | Meaning |
| --- | --- |
| `hybrid_search` | Provider MCP and platform-native search are both available. |
| `provider_connected` | Provider MCP is available; platform-native search was not declared available. |
| `native_only` | Provider MCP is unavailable, but Codex/Claude platform-native search is available. |
| `strategy_only` | Neither provider MCP nor platform-native search nor user corpus is available. |

MCP servers must not call Codex or Claude native search directly. The active agent executes native search queries returned by `qiongli_search_plan`, then logs them with provenance labels such as `native:codex_web_search` or `native:claude_web_search`. Provider records keep labels such as `mcp:openalex` and `mcp:semantic_scholar`.
```

In `docs/guide/troubleshooting.md`, update the `strategy_only` entry with:

```markdown
  - If platform-native search is available, ask Qiongli to call `qiongli_search_plan` and use `native_only` or `hybrid_search` rather than `strategy_only`.
```

- [ ] **Step 4: Update Chinese docs**

In `docs/zh/advanced/mcp-providers-setup.md`, add:

```markdown
## Hybrid Search Router

文献综述、单篇论文查找、gap finding、citation snowballing 和 evidence synthesis 应在 `qiongli_literature_status` 之后调用 `qiongli_search_plan`。该工具返回四种执行模式：

| Mode | 含义 |
| --- | --- |
| `hybrid_search` | Provider MCP 和平台原生搜索都可用。 |
| `provider_connected` | Provider MCP 可用，但当前未声明平台原生搜索可用。 |
| `native_only` | Provider MCP 不可用，但 Codex/Claude 平台原生搜索可用。 |
| `strategy_only` | Provider MCP、平台原生搜索和用户语料都不可用。 |

MCP servers 不能直接调用 Codex 或 Claude 的 native search。当前 agent 负责执行 `qiongli_search_plan` 返回的 native search queries，然后用 `native:codex_web_search` 或 `native:claude_web_search` 等 provenance labels 记录。Provider records 保留 `mcp:openalex`、`mcp:semantic_scholar` 等 labels。
```

In `docs/zh/guide/troubleshooting.md`, update the `strategy_only` entry with:

```markdown
  - 如果当前平台有原生搜索能力，要求 Qiongli 调用 `qiongli_search_plan`，并使用 `native_only` 或 `hybrid_search`，不要直接降级为 `strategy_only`。
```

- [ ] **Step 5: Run docs tests**

Run:

```bash
python3 -m unittest tests.test_mcp_provider_docs tests.test_cli_setup_docs -v
```

Expected: PASS.

### Task 6: Verify Packaging, Release Artifacts, And Boundaries

**Files:**
- Read/verify all modified files.

- [ ] **Step 1: Run focused Python and Node tests**

Run:

```bash
python3 -m unittest \
  tests.test_hybrid_search_router \
  tests.test_mcp_tool_handlers \
  tests.test_mcp_stdio_server \
  tests.test_literature_contract \
  tests.test_mcp_provider_docs \
  tests.test_cli_setup_docs \
  tests.test_literature_mcpb_artifact \
  tests.test_plugin_distribution_contract \
  tests.test_plugin_artifacts \
  tests.test_subject_materializer \
  -v
npm --prefix packages/qiongli-literature-mcpb test
```

Expected: PASS.

- [ ] **Step 2: Run syntax and whitespace checks**

Run:

```bash
python3 -m py_compile \
  packages/python-qiongli/src/qiongli/bridges/hybrid_search_router.py \
  packages/python-qiongli/src/qiongli/bridges/literature_mcp_tools.py \
  packages/python-qiongli/src/qiongli/bridges/mcp_tool_handlers.py
node --check packages/qiongli-literature-mcpb/server/search-plan.mjs
node --check packages/qiongli-literature-mcpb/server/index.mjs
git diff --check
```

Expected: all commands exit 0.

- [ ] **Step 3: Build MCPB and plugin artifact checks**

Run:

```bash
python3 scripts/build_literature_mcpb.py --dist-dir /tmp/qiongli-mcpb-check
python3 -m unittest tests.test_literature_mcpb_artifact tests.test_plugin_distribution_contract tests.test_plugin_artifacts -v
```

Expected: the MCPB artifact includes `server/search-plan.mjs`, and materialized Codex/Claude plugin payloads include the updated `mcp/qiongli-literature-provider/search-plan.mjs` copied from the MCPB server directory.

- [ ] **Step 4: Repository boundary review**

Run:

```bash
git status --short --untracked-files=all
git diff --name-only
rg -n "sk-[A-Za-z0-9]|api[_-]?key\\s*[:=]\\s*['\\\"][^'\\\"]+|/Users/pengjiaxin|BEGIN RSA|BEGIN OPENSSH|password\\s*[:=]" \
  packages/python-qiongli/src/qiongli/bridges \
  packages/qiongli-literature-mcpb \
  content/workflow \
  content/skills/B_literature \
  docs/advanced \
  docs/guide \
  docs/zh/advanced \
  docs/zh/guide \
  tests
```

Expected findings:
- No provider secrets or real API keys are introduced.
- No machine-specific absolute paths are committed.
- No generated plugin payload directories are edited directly; plugin payload updates come from `packages/qiongli-literature-mcpb/server/` via the existing artifact builder.
- The new `qiongli_search_plan` contract is implemented in both Python full MCP and Node MCPB.
- The tool returns native-search instructions but does not execute platform-native search.

- [ ] **Step 5: Commit after existing work is separated**

Because the current working tree already contains wrapper-skill and Codex MCP UX changes, inspect `git status --short --untracked-files=all` before staging. Commit this hybrid router work separately from earlier changes:

```bash
git add \
  packages/python-qiongli/src/qiongli/bridges/hybrid_search_router.py \
  packages/python-qiongli/src/qiongli/bridges/literature_mcp_tools.py \
  packages/python-qiongli/src/qiongli/bridges/mcp_tool_handlers.py \
  packages/qiongli-literature-mcpb/server/search-plan.mjs \
  packages/qiongli-literature-mcpb/server/index.mjs \
  packages/qiongli-literature-mcpb/manifest.json \
  content/workflow/SKILL.md \
  content/workflow/workflows/lit-review.md \
  content/workflow/workflows/paper-read.md \
  content/skills/B_literature/academic-searcher.md \
  docs/advanced/cross-platform-mcp.md \
  docs/advanced/mcp-providers-setup.md \
  docs/guide/troubleshooting.md \
  docs/zh/advanced/mcp-providers-setup.md \
  docs/zh/guide/troubleshooting.md \
  tests/test_hybrid_search_router.py \
  tests/test_mcp_tool_handlers.py \
  tests/test_mcp_stdio_server.py \
  tests/test_literature_contract.py \
  tests/test_mcp_provider_docs.py \
  tests/test_cli_setup_docs.py \
  tests/test_literature_mcpb_artifact.py \
  packages/qiongli-literature-mcpb/test/tools.test.mjs
git commit -m "feat(search): add hybrid literature search planning"
```

If `content/workflow/SKILL.md`, `docs/guide/troubleshooting.md`, or shared tests also contain previous Codex MCP UX edits, stage hunks carefully so each commit remains reviewable.

## Self-Review

**Spec coverage:** The plan keeps MCP provider search and Codex/Claude native search collaborative without changing MCP call direction. `qiongli_search_plan` plans native searches, agents execute them, and workflows merge results with provenance labels.

**Placeholder scan:** The plan contains concrete file paths, commands, expected outcomes, and code snippets. It avoids open-ended placeholders and keeps implementation steps directly executable.

**Type consistency:** The canonical field is `search_execution_mode`, with values `hybrid_search`, `provider_connected`, `native_only`, and `strategy_only`. Provider status remains `provider_capability_mode`, preserving the existing `provider_connected` / `strategy_only` provider capability contract.
