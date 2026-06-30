# Unified Full CLI MCP Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make `qiongli mcp serve` the single full MCP server by merging core literature tools into the Python-backed CLI MCP, and make `qiongli install --profile full --target codex` register that unified MCP.

**Architecture:** Add a Python literature MCP layer that reuses existing query planning, normalization, diagnostics, provider config, and evidence-bundle helpers. Keep marketplace Node literature MCP as lite/no-CLI fallback, while full CLI MCP becomes the complete product surface. Add a conservative MCP install part that can update managed Codex MCP config and print config fragments for targets without a stable writer.

**Tech Stack:** Python 3.12+, `unittest`, stdio JSON-RPC MCP server, `urllib`, existing Qiongli provider/query/diagnostics modules, Codex TOML config fragments, npm installer parity tests.

---

## Scope Guardrails

- Do not remove the Node literature MCP from marketplace plugin or MCPB packages.
- Do not require Node for `qiongli mcp serve`.
- Do not require provider API keys for MCP startup, install, or doctor.
- Do not launch local Codex/Claude/Antigravity agents during install or doctor.
- Do not overwrite unmanaged user MCP config entries.
- Automatic MCP config writing is Codex-only in this first cut; other targets print exact config fragments.
- Keep all provider secrets out of manifests, config examples, test fixtures, and output logs.

## File Structure

- Create `packages/python-qiongli/src/qiongli/bridges/literature_mcp_tools.py`: Python handlers and tool definitions for `qiongli_literature_*`.
- Create `packages/python-qiongli/src/qiongli/bridges/providers/openalex_client.py`: OpenAlex search client.
- Create `packages/python-qiongli/src/qiongli/bridges/providers/crossref_client.py`: Crossref search client.
- Create `packages/python-qiongli/src/qiongli/bridges/providers/pubmed_client.py`: PubMed search client.
- Modify `packages/python-qiongli/src/qiongli/bridges/mcp_tool_handlers.py`: merge literature tool definitions and dispatch handlers.
- Modify `packages/python-qiongli/src/qiongli/bridges/mcp_cli.py`: include literature tools in config/doctor output.
- Create `packages/python-qiongli/src/qiongli/bridges/mcp_client_config.py`: managed Codex MCP config writer and fragments for other clients.
- Modify `packages/python-qiongli/src/qiongli/universal_installer.py`: add `mcp` install/remove part and full-profile defaults.
- Modify `packages/npm-qiongli/lib/installer.mjs`: add npm-side `mcp` part reporting or delegate guidance so npm CLI output matches Python CLI semantics.
- Modify `packages/npm-qiongli/README.md`, `docs/guide/install.md`, `docs/zh/guide/install.md`, `docs/advanced/cross-platform-mcp.md`, `docs/advanced/plugin-first-architecture.md`: update product positioning.
- Create `tests/test_mcp_literature_tools.py`: literature handler tests.
- Create `tests/test_provider_clients.py`: provider client request/normalization tests.
- Create `tests/test_mcp_client_config.py`: Codex config writer tests.
- Modify `tests/test_mcp_tool_handlers.py`: unified MCP tool-list tests.
- Modify `tests/test_mcp_stdio_server.py`: stdio list/call coverage for literature tools.
- Modify `tests/test_universal_installer.py`: full profile includes managed MCP registration.
- Modify `packages/npm-qiongli/test/installer.test.mjs`: npm full install reports MCP part consistently.

## Task 1: Add Failing Tests For Unified Literature Tools In Full MCP

**Files:**
- Modify: `tests/test_mcp_tool_handlers.py`
- Create: `tests/test_mcp_literature_tools.py`
- Modify: `tests/test_mcp_stdio_server.py`

- [ ] **Step 1: Add tool definition expectations**

In `tests/test_mcp_tool_handlers.py`, update `test_tool_definitions_include_config_and_evidence_tools` to require the literature tools:

```python
        self.assertTrue(
            {
                "qiongli_literature_status",
                "qiongli_literature_search",
                "qiongli_literature_export_evidence",
                "qiongli_config_status",
                "qiongli_save_provider_config",
                "qiongli_collect_evidence",
                "qiongli_list_provider_env",
                "qiongli_test_provider",
                "qiongli_configure_provider",
                "qiongli_open_config_wizard",
                "qiongli_orchestrator_route",
                "qiongli_orchestrator_doctor",
                "qiongli_task_plan",
                "qiongli_task_run",
            }.issubset(names)
        )
```

- [ ] **Step 2: Add literature status handler test**

Create `tests/test_mcp_literature_tools.py`:

```python
from __future__ import annotations

import json
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
            with mock.patch.dict("os.environ", {"QIONGLI_CONFIG_HOME": str(root / "config")}, clear=False):
                set_provider_value("openalex", "api-key", "openalex-secret-key")
                result = call_qiongli_tool("qiongli_literature_status", {})

        payload = result["structuredContent"]
        rendered = json.dumps(payload, sort_keys=True)
        self.assertFalse(result["isError"])
        self.assertEqual(payload["providers"]["openalex"], "configured")
        self.assertIn("openalex", payload["capabilities"])
        self.assertIn("semantic_scholar", payload["capabilities"])
        self.assertIn("crossref", payload["capabilities"])
        self.assertIn("pubmed", payload["capabilities"])
        self.assertNotIn("openalex-secret-key", rendered)
```

- [ ] **Step 3: Add literature search handler test with mocked executor**

Append to `tests/test_mcp_literature_tools.py`:

```python
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
```

- [ ] **Step 4: Add evidence export handler test**

Append:

```python
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
```

- [ ] **Step 5: Add stdio tools/list smoke expectation**

In `tests/test_mcp_stdio_server.py`, extend the existing `tools/list` assertion to include:

```python
        self.assertIn("qiongli_literature_status", names)
        self.assertIn("qiongli_literature_search", names)
        self.assertIn("qiongli_literature_export_evidence", names)
```

- [ ] **Step 6: Run tests and confirm failure**

Run:

```bash
uv run python -m unittest tests.test_mcp_tool_handlers tests.test_mcp_literature_tools tests.test_mcp_stdio_server -v
```

Expected: fails because `qiongli_literature_*` tools and `bridges.literature_mcp_tools` do not exist.

## Task 2: Implement Literature Tool Definitions And Dispatch

**Files:**
- Create: `packages/python-qiongli/src/qiongli/bridges/literature_mcp_tools.py`
- Modify: `packages/python-qiongli/src/qiongli/bridges/mcp_tool_handlers.py`
- Test: `tests/test_mcp_literature_tools.py`
- Test: `tests/test_mcp_tool_handlers.py`

- [ ] **Step 1: Create literature MCP tool module**

Create `packages/python-qiongli/src/qiongli/bridges/literature_mcp_tools.py`:

```python
from __future__ import annotations

from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from bridges.provider_config import (
    provider_capability_mode,
    provider_config_summary,
    redact_provider_config,
    resolve_provider_config,
)


LITERATURE_PROVIDER_CAPABILITIES: dict[str, dict[str, Any]] = {
    "openalex": {
        "status": "implemented",
        "capabilities": ["topic_search", "doi_lookup", "year_filter", "document_type_filter", "venue_metadata"],
    },
    "semantic_scholar": {
        "status": "implemented",
        "capabilities": ["topic_search", "title_lookup", "doi_lookup", "year_filter", "venue_metadata"],
    },
    "crossref": {
        "status": "implemented",
        "capabilities": ["topic_search", "doi_lookup", "year_filter", "document_type_filter", "reference_metadata"],
    },
    "pubmed": {
        "status": "implemented",
        "capabilities": ["topic_search", "doi_lookup", "biomedical_topic_search", "year_filter"],
    },
}


LITERATURE_TOOL_DEFINITIONS: list[dict[str, Any]] = [
    {
        "name": "qiongli_literature_status",
        "description": "Report configured literature providers and capability mode without exposing secrets.",
        "inputSchema": {"type": "object", "properties": {}, "additionalProperties": False},
    },
    {
        "name": "qiongli_literature_search",
        "description": "Search academic literature using the full Qiongli CLI MCP provider stack.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "query": {"type": "string"},
                "limit": {"type": "number"},
                "per_provider_limit": {"type": "number"},
                "total_limit": {"type": "number"},
                "search_mode": {
                    "type": "string",
                    "enum": ["auto", "topic", "title", "doi", "review", "systematic_review"],
                },
                "exact_title": {"type": "boolean"},
                "fromYear": {"type": ["integer", "string"]},
                "toYear": {"type": ["integer", "string"]},
                "venue_filter": {"type": "string"},
                "document_types": {"type": "array", "items": {"type": "string"}},
                "query_variants": {"type": "array", "items": {"type": "string"}},
            },
            "additionalProperties": True,
        },
    },
    {
        "name": "qiongli_literature_export_evidence",
        "description": "Export an auditable provider capability and search evidence snapshot.",
        "inputSchema": {"type": "object", "properties": {}, "additionalProperties": True},
    },
]


def handle_literature_status(args: dict[str, Any]) -> dict[str, Any]:
    cwd = _cwd_from_args(args)
    config = resolve_provider_config(cwd=cwd)
    summary = provider_config_summary(config)
    return {
        "providers": summary,
        "capability_mode": provider_capability_mode(summary),
        "capabilities": LITERATURE_PROVIDER_CAPABILITIES,
        "redacted_config": redact_provider_config(config),
    }


def handle_literature_search(args: dict[str, Any]) -> dict[str, Any]:
    return run_literature_search(args)


def handle_literature_export_evidence(args: dict[str, Any]) -> dict[str, Any]:
    results = args.get("results", args.get("search_results", []))
    if not isinstance(results, list):
        results = []
    return {
        "artifact_type": "qiongli_literature_evidence_snapshot",
        "exported_at": datetime.now(timezone.utc).replace(microsecond=0).isoformat(),
        "query": str(args.get("query", "") or "").strip(),
        "provider_status": args.get("provider_status", {}),
        "search_plan": args.get("search_plan", args.get("query_plan", {})),
        "diagnostics": args.get("diagnostics", args.get("search_diagnostics", {})),
        "result_count": len(results),
        "results": results,
    }


def run_literature_search(args: dict[str, Any]) -> dict[str, Any]:
    from bridges.providers.literature_search import run_scholarly_search
    from bridges.providers.s2_client import search_paper

    task_packet = _task_packet_from_search_args(args)
    return run_scholarly_search(task_packet, search_paper)


def _task_packet_from_search_args(args: dict[str, Any]) -> dict[str, Any]:
    query = str(args.get("query", "") or "").strip()
    variants = args.get("query_variants", args.get("queryVariants", []))
    keywords = [query] if query else []
    if isinstance(variants, list):
        keywords.extend(str(item).strip() for item in variants if str(item).strip())
    return {
        "topic": query or "literature-search",
        "research_question": query,
        "keywords": keywords,
        "paper_type": _paper_type_from_search_mode(args.get("search_mode", args.get("searchMode"))),
        "search_mode": args.get("search_mode", args.get("searchMode", "auto")),
        "year_start": args.get("fromYear"),
        "year_end": args.get("toYear"),
        "venue_profile": args.get("venue_filter", args.get("venueFilter", "")),
        "publication_type": _first_document_type(args.get("document_types", args.get("documentTypes"))),
        "limit": args.get("limit"),
        "per_provider_limit": args.get("per_provider_limit", args.get("perProviderLimit")),
    }


def _paper_type_from_search_mode(search_mode: Any) -> str:
    return "systematic-review" if str(search_mode).strip() == "systematic_review" else "empirical"


def _first_document_type(raw: Any) -> str:
    if isinstance(raw, list) and raw:
        return str(raw[0])
    return str(raw or "")


def _cwd_from_args(args: dict[str, Any]) -> Path:
    raw = str(args.get("cwd", "") or "").strip()
    return Path(raw).expanduser().resolve() if raw else Path.cwd()
```

- [ ] **Step 2: Merge tool definitions into full MCP**

In `packages/python-qiongli/src/qiongli/bridges/mcp_tool_handlers.py`, add:

```python
from bridges.literature_mcp_tools import (
    LITERATURE_TOOL_DEFINITIONS,
    handle_literature_export_evidence,
    handle_literature_search,
    handle_literature_status,
)
```

After the existing `MCP_TOOL_DEFINITIONS` list literal is defined, append:

```python
MCP_TOOL_DEFINITIONS = [*LITERATURE_TOOL_DEFINITIONS, *MCP_TOOL_DEFINITIONS]
```

In `call_qiongli_tool`, add handlers:

```python
        "qiongli_literature_status": handle_literature_status,
        "qiongli_literature_search": handle_literature_search,
        "qiongli_literature_export_evidence": handle_literature_export_evidence,
```

- [ ] **Step 3: Run focused tests**

Run:

```bash
uv run python -m unittest tests.test_mcp_tool_handlers tests.test_mcp_literature_tools tests.test_mcp_stdio_server -v
```

Expected: tool definition/status/export tests pass; search test passes with the mocked executor.

- [ ] **Step 4: Commit**

Run:

```bash
git add packages/python-qiongli/src/qiongli/bridges/literature_mcp_tools.py packages/python-qiongli/src/qiongli/bridges/mcp_tool_handlers.py tests/test_mcp_tool_handlers.py tests/test_mcp_literature_tools.py tests/test_mcp_stdio_server.py
git commit -m "feat(mcp): expose literature tools from full cli server"
```

Expected: commit succeeds.

## Task 3: Add Provider Client Tests For OpenAlex, Crossref, And PubMed

**Files:**
- Create: `tests/test_provider_clients.py`
- Create: `packages/python-qiongli/src/qiongli/bridges/providers/openalex_client.py`
- Create: `packages/python-qiongli/src/qiongli/bridges/providers/crossref_client.py`
- Create: `packages/python-qiongli/src/qiongli/bridges/providers/pubmed_client.py`

- [ ] **Step 1: Add URL construction and normalization tests**

Create `tests/test_provider_clients.py`:

```python
from __future__ import annotations

import json
import unittest
from unittest import mock

from bridges.providers import crossref_client, openalex_client, pubmed_client


class ProviderClientTests(unittest.TestCase):
    def test_openalex_search_uses_query_filters_and_normalizes_results(self) -> None:
        captured: dict[str, str] = {}

        def fake_urlopen(request, timeout=0):
            captured["url"] = request.full_url
            return _FakeResponse({"results": [{"id": "https://openalex.org/W1", "display_name": "OpenAlex Paper", "publication_year": 2024}]})

        with mock.patch("urllib.request.urlopen", fake_urlopen):
            result = openalex_client.search({"translated_query": "AI education", "filters": {"year_start": 2020}}, 3)

        self.assertIn("search=AI%20education", captured["url"])
        self.assertIn("per-page=3", captured["url"])
        self.assertEqual(result["data"][0]["title"], "OpenAlex Paper")
        self.assertEqual(result["data"][0]["provider"], "openalex")

    def test_crossref_search_normalizes_items(self) -> None:
        def fake_urlopen(request, timeout=0):
            return _FakeResponse({"message": {"items": [{"DOI": "10.1/demo", "title": ["Crossref Paper"], "issued": {"date-parts": [[2023]]}}]}})

        with mock.patch("urllib.request.urlopen", fake_urlopen):
            result = crossref_client.search({"translated_query": "AI education", "filters": {}}, 2)

        self.assertEqual(result["data"][0]["title"], "Crossref Paper")
        self.assertEqual(result["data"][0]["doi"], "10.1/demo")
        self.assertEqual(result["data"][0]["provider"], "crossref")

    def test_pubmed_search_normalizes_esearch_and_esummary(self) -> None:
        responses = [
            {"esearchresult": {"idlist": ["123"]}},
            {"result": {"123": {"title": "PubMed Paper", "pubdate": "2022 Jan", "source": "Demo Journal"}}},
        ]

        def fake_urlopen(request, timeout=0):
            return _FakeResponse(responses.pop(0))

        with mock.patch("urllib.request.urlopen", fake_urlopen):
            result = pubmed_client.search({"translated_query": "AI education", "filters": {}}, 5)

        self.assertEqual(result["data"][0]["title"], "PubMed Paper")
        self.assertEqual(result["data"][0]["year"], 2022)
        self.assertEqual(result["data"][0]["provider"], "pubmed")


class _FakeResponse:
    def __init__(self, payload: dict):
        self.payload = payload

    def __enter__(self):
        return self

    def __exit__(self, *_args):
        return False

    def read(self) -> bytes:
        return json.dumps(self.payload).encode("utf-8")
```

- [ ] **Step 2: Run tests and confirm failure**

Run:

```bash
uv run python -m unittest tests.test_provider_clients -v
```

Expected: fails because the provider client modules do not exist.

- [ ] **Step 3: Implement provider clients**

Create each provider client with this public function:

```python
def search(translation: dict[str, object], limit: int) -> dict[str, object]:
    ...
```

Implementation requirements:

- Return `{"data": [normalized records...]}` on success.
- Return `{"error": "...", "data": []}` on HTTP or JSON failure.
- Normalize fields to keys consumed by `normalize_search_hit`: `title`, `authors`, `year`, `abstract`, `url`, `venue`, `doi`, `provider`, `external_ids`.
- Use provider config for optional OpenAlex email, Crossref email, and PubMed API key.
- Keep request timeouts and retry behavior conservative; match `s2_client.py` style.

- [ ] **Step 4: Run provider tests**

Run:

```bash
uv run python -m unittest tests.test_provider_clients -v
```

Expected: all provider client tests pass.

- [ ] **Step 5: Commit**

Run:

```bash
git add tests/test_provider_clients.py packages/python-qiongli/src/qiongli/bridges/providers/openalex_client.py packages/python-qiongli/src/qiongli/bridges/providers/crossref_client.py packages/python-qiongli/src/qiongli/bridges/providers/pubmed_client.py
git commit -m "feat(mcp): add python literature provider clients"
```

Expected: commit succeeds.

## Task 4: Wire Multi-Provider Literature Search Into Full MCP

**Files:**
- Modify: `packages/python-qiongli/src/qiongli/bridges/literature_mcp_tools.py`
- Modify: `tests/test_mcp_literature_tools.py`

- [ ] **Step 1: Add multi-provider search test**

Append to `tests/test_mcp_literature_tools.py`:

```python
    def test_literature_search_uses_configured_provider_clients(self) -> None:
        calls: list[str] = []

        def fake_provider(name: str):
            def _search(_translation, _limit):
                calls.append(name)
                return {"data": [{"title": f"{name} paper", "provider": name, "year": 2024}]}
            return _search

        with mock.patch("bridges.literature_mcp_tools._configured_provider_fns") as configured:
            configured.return_value = {
                "semantic_scholar": fake_provider("semantic_scholar"),
                "openalex": fake_provider("openalex"),
                "crossref": fake_provider("crossref"),
                "pubmed": fake_provider("pubmed"),
            }
            result = call_qiongli_tool("qiongli_literature_search", {"query": "AI education", "limit": 4})

        payload = result["structuredContent"]
        self.assertFalse(result["isError"])
        self.assertEqual(set(calls), {"semantic_scholar", "openalex", "crossref", "pubmed"})
        self.assertIn("search_diagnostics", payload["data"])
```

- [ ] **Step 2: Implement provider selection**

In `literature_mcp_tools.py`, update `run_literature_search`:

```python
def run_literature_search(args: dict[str, Any]) -> dict[str, Any]:
    from bridges.providers.literature_search import run_scholarly_search
    from bridges.providers.s2_client import search_paper

    task_packet = _task_packet_from_search_args(args)
    provider_fns = _configured_provider_fns(args)
    if provider_fns:
        return run_scholarly_search(task_packet, search_paper, provider_fns=provider_fns)
    return run_scholarly_search(task_packet, search_paper)
```

Add:

```python
def _configured_provider_fns(args: dict[str, Any]) -> dict[str, Any]:
    from bridges.providers import crossref_client, openalex_client, pubmed_client, s2_client

    requested = args.get("providers")
    if isinstance(requested, list) and requested:
        provider_names = {str(item).strip().lower().replace("-", "_") for item in requested}
    else:
        provider_names = {"semantic_scholar", "openalex", "crossref", "pubmed"}

    mapping = {
        "semantic_scholar": _s2_provider_search,
        "openalex": openalex_client.search,
        "crossref": crossref_client.search,
        "pubmed": pubmed_client.search,
    }
    return {name: fn for name, fn in mapping.items() if name in provider_names}


def _s2_provider_search(translation: dict[str, object], limit: int) -> dict[str, object]:
    from bridges.providers.s2_client import search_paper

    filters = translation.get("filters", {})
    filters = filters if isinstance(filters, dict) else {}
    return search_paper(
        str(translation.get("translated_query", "") or ""),
        limit,
        year_start=filters.get("year_start"),
        year_end=filters.get("year_end"),
        publication_type=filters.get("publication_type"),
        venue=filters.get("venue"),
    )
```

- [ ] **Step 3: Run literature tests**

Run:

```bash
uv run python -m unittest tests.test_mcp_literature_tools tests.test_literature_search tests.test_literature_query_planning tests.test_literature_search_quality_audit -v
```

Expected: all tests pass.

- [ ] **Step 4: Commit**

Run:

```bash
git add packages/python-qiongli/src/qiongli/bridges/literature_mcp_tools.py tests/test_mcp_literature_tools.py
git commit -m "feat(mcp): route literature search through full cli providers"
```

Expected: commit succeeds.

## Task 5: Add Managed MCP Config Writer For Codex

**Files:**
- Create: `packages/python-qiongli/src/qiongli/bridges/mcp_client_config.py`
- Create: `tests/test_mcp_client_config.py`

- [ ] **Step 1: Add Codex config writer tests**

Create `tests/test_mcp_client_config.py`:

```python
from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from bridges.mcp_client_config import install_mcp_config, remove_mcp_config


class MCPClientConfigTests(unittest.TestCase):
    def test_codex_dry_run_reports_managed_entry_without_writing(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            codex_home = Path(tmp_dir) / "codex"
            result = install_mcp_config("codex", codex_home=codex_home, dry_run=True)

        self.assertEqual(result["status"], "dry-run")
        self.assertEqual(result["server"]["command"], "qiongli")
        self.assertFalse((codex_home / "config.toml").exists())

    def test_codex_install_writes_managed_qiongli_entry(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            codex_home = Path(tmp_dir) / "codex"
            result = install_mcp_config("codex", codex_home=codex_home, dry_run=False)
            config_text = (codex_home / "config.toml").read_text(encoding="utf-8")

        self.assertEqual(result["status"], "installed")
        self.assertIn("# BEGIN QIONGLI MANAGED MCP", config_text)
        self.assertIn("[mcp_servers.qiongli]", config_text)
        self.assertIn('command = "qiongli"', config_text)

    def test_codex_install_skips_unmanaged_existing_entry(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            codex_home = Path(tmp_dir) / "codex"
            codex_home.mkdir(parents=True)
            config = codex_home / "config.toml"
            config.write_text('[mcp_servers.qiongli]\ncommand = "custom"\n', encoding="utf-8")

            result = install_mcp_config("codex", codex_home=codex_home, dry_run=False)

        self.assertEqual(result["status"], "skip")
        self.assertIn("unmanaged", result["detail"])
        self.assertEqual(config.read_text(encoding="utf-8"), '[mcp_servers.qiongli]\ncommand = "custom"\n')

    def test_codex_remove_deletes_managed_block_only(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            codex_home = Path(tmp_dir) / "codex"
            install_mcp_config("codex", codex_home=codex_home, dry_run=False)
            result = remove_mcp_config("codex", codex_home=codex_home, dry_run=False)
            config_text = (codex_home / "config.toml").read_text(encoding="utf-8")

        self.assertEqual(result["status"], "removed")
        self.assertNotIn("QIONGLI MANAGED MCP", config_text)
```

- [ ] **Step 2: Run tests and confirm failure**

Run:

```bash
uv run python -m unittest tests.test_mcp_client_config -v
```

Expected: fails because `bridges.mcp_client_config` does not exist.

- [ ] **Step 3: Implement MCP config writer**

Create `packages/python-qiongli/src/qiongli/bridges/mcp_client_config.py` with:

```python
from __future__ import annotations

import os
import re
from pathlib import Path
from typing import Any


BEGIN = "# BEGIN QIONGLI MANAGED MCP"
END = "# END QIONGLI MANAGED MCP"
MANAGED_RE = re.compile(r"\n?# BEGIN QIONGLI MANAGED MCP\n.*?# END QIONGLI MANAGED MCP\n?", re.S)
UNMANAGED_QIONGLI_RE = re.compile(r"^\[mcp_servers\.qiongli\]\s*$", re.M)


def qiongli_mcp_server_entry() -> dict[str, Any]:
    return {"command": "qiongli", "args": ["mcp", "serve", "--transport", "stdio"]}


def install_mcp_config(
    target: str,
    *,
    codex_home: Path | None = None,
    dry_run: bool = False,
    overwrite: bool = False,
) -> dict[str, Any]:
    if target != "codex":
        return {
            "target": target,
            "status": "fragment",
            "server": qiongli_mcp_server_entry(),
            "detail": "automatic MCP config writing is only supported for codex in this release",
        }

    config_path = (codex_home or Path(os.environ.get("CODEX_HOME", str(Path.home() / ".codex")))) / "config.toml"
    existing = config_path.read_text(encoding="utf-8") if config_path.exists() else ""
    block = _managed_block()

    if MANAGED_RE.search(existing):
        updated = MANAGED_RE.sub("\n" + block + "\n", existing).strip() + "\n"
        status = "updated"
    elif UNMANAGED_QIONGLI_RE.search(existing) and not overwrite:
        return {
            "target": target,
            "status": "skip",
            "path": str(config_path),
            "server": qiongli_mcp_server_entry(),
            "detail": "unmanaged qiongli MCP entry exists; pass overwrite_mcp to replace it",
        }
    else:
        updated = (existing.rstrip() + "\n\n" + block + "\n").lstrip()
        status = "installed"

    if dry_run:
        return {"target": target, "status": "dry-run", "path": str(config_path), "server": qiongli_mcp_server_entry()}

    config_path.parent.mkdir(parents=True, exist_ok=True)
    config_path.write_text(updated, encoding="utf-8")
    return {"target": target, "status": status, "path": str(config_path), "server": qiongli_mcp_server_entry()}


def remove_mcp_config(
    target: str,
    *,
    codex_home: Path | None = None,
    dry_run: bool = False,
) -> dict[str, Any]:
    if target != "codex":
        return {"target": target, "status": "skip", "detail": "automatic MCP config removal is only supported for codex"}

    config_path = (codex_home or Path(os.environ.get("CODEX_HOME", str(Path.home() / ".codex")))) / "config.toml"
    if not config_path.exists():
        return {"target": target, "status": "skip", "path": str(config_path), "detail": "config file not found"}
    existing = config_path.read_text(encoding="utf-8")
    updated = MANAGED_RE.sub("\n", existing).strip() + "\n"
    if updated == existing:
        return {"target": target, "status": "skip", "path": str(config_path), "detail": "managed entry not found"}
    if not dry_run:
        config_path.write_text(updated, encoding="utf-8")
    return {"target": target, "status": "dry-run" if dry_run else "removed", "path": str(config_path)}


def _managed_block() -> str:
    return "\n".join(
        [
            BEGIN,
            "[mcp_servers.qiongli]",
            'command = "qiongli"',
            'args = ["mcp", "serve", "--transport", "stdio"]',
            END,
        ]
    )
```

- [ ] **Step 4: Run config writer tests**

Run:

```bash
uv run python -m unittest tests.test_mcp_client_config -v
```

Expected: all tests pass.

- [ ] **Step 5: Commit**

Run:

```bash
git add packages/python-qiongli/src/qiongli/bridges/mcp_client_config.py tests/test_mcp_client_config.py
git commit -m "feat(installer): add managed codex mcp config writer"
```

Expected: commit succeeds.

## Task 6: Make Full Install Include MCP Registration

**Files:**
- Modify: `packages/python-qiongli/src/qiongli/universal_installer.py`
- Modify: `tests/test_universal_installer.py`

- [ ] **Step 1: Add failing installer tests**

In `tests/test_universal_installer.py`, add:

```python
    def test_full_profile_dry_run_reports_mcp_registration(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            temp_root = Path(tmp_dir)
            project_dir = temp_root / "project"
            project_dir.mkdir(parents=True)
            codex_home = temp_root / "codex-home"

            env = os.environ.copy()
            env["CODEX_HOME"] = str(codex_home)
            env["PATH"] = ""

            stdout = io.StringIO()
            with mock.patch.dict(os.environ, env, clear=True):
                with contextlib.redirect_stdout(stdout):
                    result = install(
                        InstallOptions(
                            repo_root=REPO_ROOT,
                            project_dir=project_dir,
                            target="codex",
                            profile="full",
                            dry_run=True,
                            doctor=False,
                        )
                    )

            self.assertEqual(result, 0)
            rendered = stdout.getvalue()
            self.assertIn("== MCP ==", rendered)
            self.assertIn("qiongli mcp serve --transport stdio", rendered)
            self.assertFalse((codex_home / "config.toml").exists())

    def test_remove_mcp_part_removes_managed_codex_config(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            temp_root = Path(tmp_dir)
            project_dir = temp_root / "project"
            project_dir.mkdir(parents=True)
            codex_home = temp_root / "codex-home"

            env = os.environ.copy()
            env["CODEX_HOME"] = str(codex_home)
            env["PATH"] = ""

            with mock.patch.dict(os.environ, env, clear=True):
                install(
                    InstallOptions(
                        repo_root=REPO_ROOT,
                        project_dir=project_dir,
                        target="codex",
                        profile="full",
                        doctor=False,
                    )
                )
                result = remove(RemoveOptions(project_dir=project_dir, target="codex", parts=("mcp",)))

            self.assertEqual(result, 0)
            self.assertNotIn("QIONGLI MANAGED MCP", (codex_home / "config.toml").read_text(encoding="utf-8"))
```

- [ ] **Step 2: Run tests and confirm failure**

Run:

```bash
uv run python -m unittest tests.test_universal_installer -v
```

Expected: fails because `mcp` is not a supported part and full profile does not register MCP.

- [ ] **Step 3: Add `mcp` part and full-profile default**

In `universal_installer.py`:

```python
PART_CHOICES = ("globals", "project", "cli", "mcp", "doctor")
```

Update `profile_defaults("full")` to include MCP behavior. If the existing return type stays boolean-only, add a helper:

```python
def default_parts_for_profile(profile: str | None) -> tuple[str, ...] | None:
    if profile == "partial":
        return ("globals", "project")
    if profile == "full":
        return ("globals", "project", "cli", "mcp", "doctor")
    return None
```

Use this when `options.parts is None`.

- [ ] **Step 4: Install MCP part**

Add:

```python
def _install_mcp(options: InstallOptions) -> None:
    from bridges.mcp_client_config import install_mcp_config

    _print_section("MCP")
    for target_name in _selected_target_names(options.target):
        result = install_mcp_config(target_name, dry_run=options.dry_run)
        detail = result.get("path") or result.get("detail") or ""
        server = result.get("server", {})
        command = " ".join([str(server.get("command", "")), *[str(item) for item in server.get("args", [])]]).strip()
        suffix = f"{detail} ({command})" if command else str(detail)
        _print_result("MCP", suffix, "ok" if result["status"] in {"installed", "updated", "dry-run", "fragment"} else "skip")
```

Call `_install_mcp(options)` when `mcp` is included in selected parts.

- [ ] **Step 5: Remove MCP part**

In `remove`, call `remove_mcp_config` when `mcp` is included in selected parts and print the result under an `MCP` section.

- [ ] **Step 6: Run installer tests**

Run:

```bash
uv run python -m unittest tests.test_universal_installer tests.test_mcp_client_config -v
```

Expected: all tests pass.

- [ ] **Step 7: Commit**

Run:

```bash
git add packages/python-qiongli/src/qiongli/universal_installer.py tests/test_universal_installer.py
git commit -m "feat(installer): include unified mcp in full installs"
```

Expected: commit succeeds.

## Task 7: Update MCP Doctor And Config Example Output

**Files:**
- Modify: `packages/python-qiongli/src/qiongli/bridges/mcp_cli.py`
- Modify: `tests/test_mcp_cli.py`

- [ ] **Step 1: Add expected literature tools to config example test**

In `tests/test_mcp_cli.py`, update the relevant config example test to assert:

```python
        self.assertIn("qiongli_literature_search", payload["literature_tools"])
        self.assertIn("qiongli_task_run", payload["orchestrator_tools"])
```

- [ ] **Step 2: Add literature status to doctor JSON test**

Assert the doctor payload includes:

```python
        self.assertIn("literature_tools_available", payload)
        self.assertTrue(payload["literature_tools_available"])
```

- [ ] **Step 3: Run test and confirm failure**

Run:

```bash
uv run python -m unittest tests.test_mcp_cli -v
```

Expected: fails because these payload fields do not exist.

- [ ] **Step 4: Implement doctor/config payload fields**

In `mcp_cli.py`, extend `_doctor_payload`:

```python
        "literature_tools_available": True,
        "orchestrator_tools_available": True,
        "literature_tools": [
            "qiongli_literature_status",
            "qiongli_literature_search",
            "qiongli_literature_export_evidence",
        ],
        "orchestrator_tools": [
            "qiongli_orchestrator_route",
            "qiongli_orchestrator_doctor",
            "qiongli_task_plan",
            "qiongli_task_run",
        ],
```

Extend `config_example` return payload with the same `literature_tools` and `orchestrator_tools` lists.

- [ ] **Step 5: Run tests**

Run:

```bash
uv run python -m unittest tests.test_mcp_cli tests.test_mcp_tool_handlers -v
```

Expected: all tests pass.

- [ ] **Step 6: Commit**

Run:

```bash
git add packages/python-qiongli/src/qiongli/bridges/mcp_cli.py tests/test_mcp_cli.py
git commit -m "docs(mcp): report unified cli mcp capabilities"
```

Expected: commit succeeds.

## Task 8: Align npm Installer Output With Full MCP Semantics

**Files:**
- Modify: `packages/npm-qiongli/lib/installer.mjs`
- Modify: `packages/npm-qiongli/test/installer.test.mjs`
- Modify: `packages/npm-qiongli/README.md`

- [ ] **Step 1: Add npm installer expectation**

In `packages/npm-qiongli/test/installer.test.mjs`, add a test:

```javascript
test('full install parts include unified MCP guidance', () => {
  const root = makePackageRoot();
  const result = installSkills({
    packageRoot: root,
    target: 'codex',
    parts: 'globals,mcp',
    dryRun: true,
    env: { HOME: root },
    platform: 'linux'
  });

  assert.ok(result.actions.some((action) => action.label === 'MCP'));
  assert.match(
    result.actions.find((action) => action.label === 'MCP').detail,
    /qiongli mcp serve --transport stdio/
  );
});
```

- [ ] **Step 2: Run test and confirm failure**

Run:

```bash
node --test packages/npm-qiongli/test/installer.test.mjs
```

Expected: fails because npm installer does not accept/report `mcp`.

- [ ] **Step 3: Add npm `mcp` install part**

In `packages/npm-qiongli/lib/installer.mjs`, add `mcp` to `PARTS` and make `installSkills` append a dry-run/action record:

```javascript
actions.push({
  label: 'MCP',
  status: dryRun ? 'dry-run' : 'manual',
  path: '<client config>',
  detail: 'Use qiongli mcp serve --transport stdio as the unified full MCP server'
});
```

Keep npm conservative: do not write Codex config from Node in this first cut unless the Python config writer is invoked through an explicit future command.

- [ ] **Step 4: Update npm README**

In `packages/npm-qiongli/README.md`, state that `qiongli install --profile full` through the Python CLI path performs managed Codex MCP registration, while npm installer output now points to the same unified full MCP command.

- [ ] **Step 5: Run npm tests**

Run:

```bash
node --test packages/npm-qiongli/test/installer.test.mjs packages/npm-qiongli/test/args.test.mjs packages/npm-qiongli/test/cli.test.mjs
```

Expected: all tests pass.

- [ ] **Step 6: Commit**

Run:

```bash
git add packages/npm-qiongli/lib/installer.mjs packages/npm-qiongli/test/installer.test.mjs packages/npm-qiongli/README.md
git commit -m "feat(npm): surface unified mcp install guidance"
```

Expected: commit succeeds.

## Task 9: Update User-Facing Documentation

**Files:**
- Modify: `docs/guide/install.md`
- Modify: `docs/zh/guide/install.md`
- Modify: `docs/advanced/cross-platform-mcp.md`
- Modify: `docs/advanced/plugin-first-architecture.md`
- Modify: `README.md`
- Test: `tests/test_cli_setup_docs.py`
- Test: `tests/test_mcp_provider_docs.py`

- [ ] **Step 1: Update install guide positioning**

In both English and Chinese install guides, rewrite the install-entry table around:

- Marketplace plugin: lite/no-CLI fallback.
- `qiongli install --profile full`: full local product.
- MCPB: Claude Desktop literature fallback.

Include this explicit command:

```bash
qiongli install --profile full --target codex
qiongli mcp doctor --json
```

- [ ] **Step 2: Update cross-platform MCP guide**

State that `qiongli mcp serve --transport stdio` is the complete full MCP and exposes literature plus orchestrator tools. Keep the Node literature MCP section as marketplace/MCPB fallback.

- [ ] **Step 3: Update plugin-first architecture doc**

Clarify that plugin-first is the marketplace distribution architecture, not the full-product architecture. Add:

```text
For full local Qiongli, the CLI full profile is canonical. Marketplace plugins remain client-native lite installs.
```

- [ ] **Step 4: Run docs tests**

Run:

```bash
uv run python -m unittest tests.test_cli_setup_docs tests.test_mcp_provider_docs tests.test_distribution_materialization_docs -v
```

Expected: all tests pass.

- [ ] **Step 5: Commit**

Run:

```bash
git add README.md docs/guide/install.md docs/zh/guide/install.md docs/advanced/cross-platform-mcp.md docs/advanced/plugin-first-architecture.md
git commit -m "docs: define cli full as complete qiongli entry"
```

Expected: commit succeeds.

## Task 10: Full Verification

**Files:**
- Verify only.

- [ ] **Step 1: Run Python MCP and installer tests**

Run:

```bash
uv run python -m unittest tests.test_mcp_tool_handlers tests.test_mcp_literature_tools tests.test_mcp_stdio_server tests.test_mcp_cli tests.test_mcp_client_config tests.test_universal_installer tests.test_provider_clients -v
```

Expected: all tests pass.

- [ ] **Step 2: Run literature regression tests**

Run:

```bash
uv run python -m unittest tests.test_literature_search tests.test_literature_query_planning tests.test_literature_search_quality_audit tests.test_literature_artifact_materialization -v
```

Expected: all tests pass.

- [ ] **Step 3: Run npm package tests**

Run:

```bash
node --test packages/npm-qiongli/test/*.test.mjs
```

Expected: all tests pass.

- [ ] **Step 4: Run full stdio MCP smoke manually**

Run:

```bash
printf '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}\n' | qiongli mcp serve --transport stdio
```

Expected: JSON output includes `qiongli_literature_search` and `qiongli_task_run`.

- [ ] **Step 5: Run full install dry-run**

Run:

```bash
qiongli install --profile full --target codex --dry-run --project-dir .
```

Expected: output includes workflow assets, shell CLI, MCP registration for `qiongli mcp serve --transport stdio`, and doctor step planning.

- [ ] **Step 6: Final status check**

Run:

```bash
git status --short
```

Expected: no uncommitted changes unless intentionally left for review.
