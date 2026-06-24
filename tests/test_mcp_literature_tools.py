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
            with mock.patch.dict(
                "os.environ",
                {"QIONGLI_CONFIG_HOME": str(root / "config")},
                clear=False,
            ):
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
