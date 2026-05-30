from __future__ import annotations

import json
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
PACKAGE_ROOT = REPO_ROOT / "packages" / "qiongli-literature-mcpb"


def test_literature_mcpb_manifest_declares_sensitive_config() -> None:
    manifest = json.loads((PACKAGE_ROOT / "manifest.json").read_text(encoding="utf-8"))

    assert manifest["manifest_version"] == "0.3"
    assert manifest["name"] == "qiongli-literature-provider"
    assert manifest["display_name"] == "Qiongli Literature Provider"
    assert manifest["server"]["type"] == "node"
    assert manifest["server"]["entry_point"] == "server/index.mjs"
    assert manifest["user_config"]["semantic_scholar_api_key"]["sensitive"] is True
    assert manifest["user_config"]["openalex_email"]["type"] == "string"
    assert manifest["user_config"]["default_result_limit"]["default"] == 10
    assert "qiongli_literature_search" in {tool["name"] for tool in manifest["tools"]}


def test_literature_mcpb_manifest_server_entry_exists() -> None:
    manifest = json.loads((PACKAGE_ROOT / "manifest.json").read_text(encoding="utf-8"))
    server_entry = PACKAGE_ROOT / manifest["server"]["entry_point"]

    assert server_entry.is_file()


def test_literature_mcpb_manifest_declares_expected_tools() -> None:
    manifest = json.loads((PACKAGE_ROOT / "manifest.json").read_text(encoding="utf-8"))

    assert {tool["name"] for tool in manifest["tools"]} == {
        "qiongli_literature_status",
        "qiongli_literature_search",
        "qiongli_literature_export_evidence",
    }
