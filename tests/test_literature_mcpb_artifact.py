from __future__ import annotations

import json
import subprocess
import sys
import zipfile
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


def test_literature_mcpb_manifest_package_structure_is_consistent() -> None:
    manifest = json.loads((PACKAGE_ROOT / "manifest.json").read_text(encoding="utf-8"))
    package = json.loads((PACKAGE_ROOT / "package.json").read_text(encoding="utf-8"))

    assert manifest["name"] == "qiongli-literature-provider"
    assert manifest["version"] == package["version"]
    assert package["type"] == "module"
    assert package["private"] is True
    assert package["scripts"]["start"] == "node server/index.mjs"
    assert "@modelcontextprotocol/sdk" in package["dependencies"]


def test_build_literature_mcpb_contains_required_files(tmp_path: Path) -> None:
    dist = tmp_path / "dist"
    result = subprocess.run(
        [sys.executable, "scripts/build_literature_mcpb.py", "--dist-dir", str(dist)],
        cwd=REPO_ROOT,
        text=True,
        capture_output=True,
    )

    assert result.returncode == 0, result.stderr
    artifact = next(dist.glob("qiongli-literature-provider-*.mcpb"))
    assert str(artifact) in result.stdout

    with zipfile.ZipFile(artifact) as zf:
        names = set(zf.namelist())

    assert "manifest.json" in names
    assert "package.json" in names
    assert "README.md" in names
    assert "server/index.mjs" in names
    assert "server/config.mjs" in names


def test_build_literature_mcpb_excludes_tests(tmp_path: Path) -> None:
    dist = tmp_path / "dist"
    result = subprocess.run(
        [sys.executable, "scripts/build_literature_mcpb.py", "--dist-dir", str(dist)],
        cwd=REPO_ROOT,
        text=True,
        capture_output=True,
    )

    assert result.returncode == 0, result.stderr
    artifact = next(dist.glob("qiongli-literature-provider-*.mcpb"))

    with zipfile.ZipFile(artifact) as zf:
        names = set(zf.namelist())

    assert not any(name.startswith("test/") or name.startswith("tests/") for name in names)
