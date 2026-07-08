# Marketplace Lite Self-Contained MCP Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace marketplace plugin dependence on user-installed Node with a plugin-bundled Rust Literature Provider MCP executable while preserving the Python Full CLI as the complete runtime.

**Architecture:** Add external MCP contracts first, then build a Rust Lite MCP runtime under `packages/qiongli-lite-mcp/` that implements the marketplace-safe literature/provider subset. Marketplace plugin builders package target-specific Rust binaries at a stable plugin-local path, while Python Full CLI remains unchanged except for parity tests and documentation that explain the two runtime profiles.

**Tech Stack:** Rust 1.78+, Cargo, `serde`, `serde_json`, `thiserror`, `reqwest` with `rustls-tls` and blocking client, `quick-xml`, Python 3.12 release scripts and `unittest`, stdio JSON-RPC MCP, existing Node MCP runtime as a temporary parity reference.

---

## Scope Guardrails

- Do not remove `packages/python-qiongli/` or change `qiongli mcp serve --transport stdio`.
- Do not migrate Full CLI to Rust in this project.
- Do not remove `packages/qiongli-literature-mcpb/` in the first implementation pass.
- Do not require users to install Node, Python, npm, pip, Cargo, Rust, or the Qiongli CLI for Marketplace Lite.
- Do not launch local agents from the Rust Lite MCP runtime.
- Do not silently install the Zotero Companion.
- Do not put provider secrets in manifests, artifacts, tests, or logs.
- Do not rely on shell wrapper scripts as the marketplace entrypoint.

## File Structure

- Create `content/mcp-contracts/lite-tools.json`: canonical overlapping Lite tool definitions.
- Create `content/mcp-contracts/provider-config.schema.json`: provider config schema used by Rust Lite and Python Full.
- Create `content/mcp-contracts/literature-result.schema.json`: normalized literature result contract.
- Create `content/mcp-contracts/literature-diagnostics.schema.json`: diagnostics contract.
- Create `content/mcp-contracts/search-plan.schema.json`: search-plan contract.
- Create `content/mcp-contracts/zotero-import-files.schema.json`: Zotero import-file contract.
- Create `content/mcp-contracts/fixtures/`: mocked provider responses and expected normalized output.
- Create `packages/qiongli-lite-mcp/Cargo.toml`: Rust package manifest for the self-contained Lite MCP runtime.
- Create `packages/qiongli-lite-mcp/src/main.rs`: executable entrypoint.
- Create `packages/qiongli-lite-mcp/src/mcp/protocol.rs`: JSON-RPC request/response types and stdio helpers.
- Create `packages/qiongli-lite-mcp/src/mcp/server.rs`: initialize, ping, tools/list, and tools/call dispatch.
- Create `packages/qiongli-lite-mcp/src/tools/definitions.rs`: tool definitions loaded from `content/mcp-contracts/lite-tools.json`.
- Create `packages/qiongli-lite-mcp/src/config/provider_config.rs`: shared provider config path, read, write, summary, redaction.
- Create `packages/qiongli-lite-mcp/src/config/wizard.rs`: local loopback provider setup page.
- Create `packages/qiongli-lite-mcp/src/providers/search.rs`: provider search orchestration and normalization.
- Create `packages/qiongli-lite-mcp/src/providers/openalex.rs`: OpenAlex client.
- Create `packages/qiongli-lite-mcp/src/providers/semantic_scholar.rs`: Semantic Scholar client.
- Create `packages/qiongli-lite-mcp/src/providers/crossref.rs`: Crossref client.
- Create `packages/qiongli-lite-mcp/src/providers/pubmed.rs`: PubMed E-Utilities client.
- Create `packages/qiongli-lite-mcp/src/providers/arxiv.rs`: arXiv Atom client.
- Create `packages/qiongli-lite-mcp/src/searchplan.rs`: search-plan response builder.
- Create `packages/qiongli-lite-mcp/src/zotero/export.rs`: CSL JSON, RIS, BibTeX, and import report exporters.
- Create `packages/qiongli-lite-mcp/src/zotero/companion.rs`: loopback-only Zotero Companion probing and calls.
- Create `packages/qiongli-lite-mcp/src/orchestrator/preview.rs`: preview-only route and task-plan handlers.
- Create `tooling/scripts/build_lite_mcp.py`: platform binary build and staging script.
- Create `scripts/build_lite_mcp.py`: wrapper that delegates to `tooling/scripts/build_lite_mcp.py`.
- Modify `tooling/scripts/build_plugin_artifacts.py`: copy Rust Lite binary into marketplace/direct plugin artifacts and update MCP command entries.
- Modify `content/distribution/platform-targets.yaml`: introduce `marketplace-lite-binary` bundled MCP mode and Rust runtime source metadata.
- Modify `tooling/scripts/validate_marketplace_install.py`: assert marketplace artifacts use plugin-local binary commands.
- Modify `tests/test_plugin_artifacts.py`: assert Codex, Claude Code, and direct Desktop plugin manifests no longer call `node` once binary mode is enabled.
- Modify `tests/test_literature_mcpb_artifact.py`: keep legacy MCPB tests and add binary MCPB tests after the migration switch.
- Create `tests/test_lite_mcp_binary_artifacts.py`: release staging tests for binary names, permissions, and manifest command paths.
- Create `tests/test_lite_mcp_contract.py`: Python-driven black-box tests that run the Rust Lite MCP executable.
- Create `tests/test_mcp_contract_fixtures.py`: Python Full CLI contract fixture validation.
- Modify `docs/advanced/cross-platform-mcp.md`, `docs/advanced/plugin-first-architecture.md`, `docs/guide/install.md`, `README.md`, and `README_CN.md`: describe Rust Marketplace Lite versus Python Full CLI.

## Task 1: Add External MCP Contracts And Fixtures

**Files:**
- Create: `content/mcp-contracts/lite-tools.json`
- Create: `content/mcp-contracts/provider-config.schema.json`
- Create: `content/mcp-contracts/literature-result.schema.json`
- Create: `content/mcp-contracts/literature-diagnostics.schema.json`
- Create: `content/mcp-contracts/search-plan.schema.json`
- Create: `content/mcp-contracts/zotero-import-files.schema.json`
- Create: `content/mcp-contracts/fixtures/openalex-search-response.json`
- Create: `content/mcp-contracts/fixtures/expected-normalized-results.json`
- Create: `tests/test_mcp_contract_fixtures.py`

- [ ] **Step 1: Write failing contract fixture test**

Create `tests/test_mcp_contract_fixtures.py`:

```python
from __future__ import annotations

import json
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
CONTRACT_ROOT = REPO_ROOT / "content" / "mcp-contracts"


class MCPContractFixtureTests(unittest.TestCase):
    def test_lite_tools_contract_declares_required_tools(self) -> None:
        tools = json.loads((CONTRACT_ROOT / "lite-tools.json").read_text(encoding="utf-8"))
        names = {tool["name"] for tool in tools["tools"]}

        self.assertTrue(
            {
                "qiongli_config_status",
                "qiongli_configure_provider",
                "qiongli_save_provider_config",
                "qiongli_open_config_wizard",
                "qiongli_literature_status",
                "qiongli_search_plan",
                "qiongli_literature_search",
                "qiongli_literature_export_evidence",
                "qiongli_zotero_status",
                "qiongli_zotero_export_import_files",
                "qiongli_orchestrator_route",
                "qiongli_task_plan",
            }.issubset(names)
        )

    def test_expected_normalized_results_fixture_has_stable_shape(self) -> None:
        payload = json.loads(
            (CONTRACT_ROOT / "fixtures" / "expected-normalized-results.json").read_text(
                encoding="utf-8"
            )
        )
        first = payload["results"][0]

        self.assertEqual(first["title"], "A Test Paper")
        self.assertEqual(first["doi"], "10.1234/example")
        self.assertEqual(first["year"], 2025)
        self.assertEqual(first["providers"], ["openalex"])
```

- [ ] **Step 2: Run RED**

Run:

```bash
.venv/bin/python -m unittest tests.test_mcp_contract_fixtures -v
```

Expected: fail because contract files do not exist.

- [ ] **Step 3: Add `lite-tools.json`**

Create `content/mcp-contracts/lite-tools.json`:

```json
{
  "schema_version": "1.0",
  "tools": [
    {
      "name": "qiongli_config_status",
      "description": "Return redacted Qiongli provider configuration status.",
      "inputSchema": {
        "type": "object",
        "properties": {},
        "additionalProperties": false
      }
    },
    {
      "name": "qiongli_save_provider_config",
      "description": "Save explicit Qiongli provider config values from chat or scripts. Prefer qiongli_configure_provider for API keys.",
      "inputSchema": {
        "type": "object",
        "required": ["provider", "field", "value"],
        "properties": {
          "provider": { "type": "string" },
          "field": { "type": "string" },
          "value": { "type": "string" }
        },
        "additionalProperties": false
      }
    },
    {
      "name": "qiongli_configure_provider",
      "description": "Open a local browser-based setup page for Qiongli provider credentials.",
      "inputSchema": {
        "type": "object",
        "properties": {
          "provider": {
            "type": "string",
            "enum": ["openalex", "semantic_scholar", "semantic-scholar", "crossref", "pubmed"]
          },
          "host": { "type": "string", "default": "127.0.0.1" },
          "port": { "type": "integer", "default": 0 }
        },
        "additionalProperties": false
      }
    },
    {
      "name": "qiongli_open_config_wizard",
      "description": "Compatibility alias for qiongli_configure_provider.",
      "inputSchema": {
        "type": "object",
        "properties": {
          "provider": {
            "type": "string",
            "enum": ["openalex", "semantic_scholar", "semantic-scholar", "crossref", "pubmed"]
          },
          "host": { "type": "string", "default": "127.0.0.1" },
          "port": { "type": "integer", "default": 0 }
        },
        "additionalProperties": false
      }
    },
    {
      "name": "qiongli_literature_status",
      "description": "Report configured literature providers and capability mode without exposing secrets.",
      "inputSchema": {
        "type": "object",
        "properties": {},
        "additionalProperties": false
      }
    },
    {
      "name": "qiongli_search_plan",
      "description": "Plan provider MCP and platform-native literature search routing.",
      "inputSchema": {
        "type": "object",
        "required": ["query"],
        "properties": {
          "query": { "type": "string" },
          "search_mode": { "type": "string" },
          "native_search_usable": { "type": "boolean", "default": false }
        },
        "additionalProperties": false
      }
    },
    {
      "name": "qiongli_literature_search",
      "description": "Search academic literature using configured OpenAlex, Semantic Scholar, Crossref, PubMed, and arXiv providers.",
      "inputSchema": {
        "type": "object",
        "required": ["query"],
        "properties": {
          "query": { "type": "string" },
          "search_mode": { "type": "string" },
          "limit": { "type": "integer" },
          "per_provider_limit": { "type": "integer" },
          "total_limit": { "type": "integer" }
        },
        "additionalProperties": true
      }
    },
    {
      "name": "qiongli_literature_export_evidence",
      "description": "Export an auditable provider capability, search plan, diagnostics, and result snapshot.",
      "inputSchema": {
        "type": "object",
        "properties": {
          "query": { "type": "string" },
          "results": { "type": "array" },
          "diagnostics": { "type": "object" }
        },
        "additionalProperties": true
      }
    },
    {
      "name": "qiongli_zotero_status",
      "description": "Report local Zotero Desktop connector, Qiongli companion, and import-file fallback availability.",
      "inputSchema": {
        "type": "object",
        "properties": {},
        "additionalProperties": true
      }
    },
    {
      "name": "qiongli_zotero_export_import_files",
      "description": "Generate Zotero-compatible CSL-JSON, RIS, BibTeX, and import-report files from Qiongli references.",
      "inputSchema": {
        "type": "object",
        "properties": {
          "records": { "type": "array" },
          "formats": { "type": "array", "items": { "type": "string" } }
        },
        "additionalProperties": true
      }
    },
    {
      "name": "qiongli_orchestrator_route",
      "description": "Preview whether the client should use skill-only routing or Full CLI orchestration.",
      "inputSchema": {
        "type": "object",
        "required": ["request"],
        "properties": {
          "request": { "type": "string" },
          "platform": { "type": "string" }
        },
        "additionalProperties": true
      }
    },
    {
      "name": "qiongli_task_plan",
      "description": "Render a Qiongli task execution plan without launching runtime agents.",
      "inputSchema": {
        "type": "object",
        "required": ["task_id", "paper_type", "topic"],
        "properties": {
          "task_id": { "type": "string" },
          "paper_type": { "type": "string" },
          "topic": { "type": "string" }
        },
        "additionalProperties": true
      }
    }
  ]
}
```

- [ ] **Step 4: Add normalized result fixture**

Create `content/mcp-contracts/fixtures/openalex-search-response.json`:

```json
{
  "results": [
    {
      "id": "https://openalex.org/W1",
      "display_name": "A Test Paper",
      "publication_year": 2025,
      "doi": "https://doi.org/10.1234/example",
      "primary_location": {
        "source": { "display_name": "Journal of Tests" }
      }
    }
  ]
}
```

Create `content/mcp-contracts/fixtures/expected-normalized-results.json`:

```json
{
  "results": [
    {
      "title": "A Test Paper",
      "doi": "10.1234/example",
      "year": 2025,
      "venue": "Journal of Tests",
      "provider": "openalex",
      "providers": ["openalex"]
    }
  ]
}
```

- [ ] **Step 5: Add minimal schema files**

Create each schema file with `"schema_version": "1.0"` and explicit required
fields used by the fixtures. Use this shape for `literature-result.schema.json`:

```json
{
  "schema_version": "1.0",
  "type": "object",
  "required": ["results"],
  "properties": {
    "results": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["title", "provider", "providers"],
        "properties": {
          "title": { "type": "string" },
          "doi": { "type": "string" },
          "year": { "type": "integer" },
          "venue": { "type": "string" },
          "provider": { "type": "string" },
          "providers": { "type": "array", "items": { "type": "string" } }
        }
      }
    }
  }
}
```

- [ ] **Step 6: Run GREEN**

Run:

```bash
.venv/bin/python -m unittest tests.test_mcp_contract_fixtures -v
```

Expected: pass.

- [ ] **Step 7: Commit**

```bash
git add content/mcp-contracts tests/test_mcp_contract_fixtures.py
git commit -m "test(mcp): add lite runtime contracts"
```

## Task 2: Add Contract Tests For No-Runtime Marketplace Manifests

**Files:**
- Modify: `tests/test_plugin_artifacts.py`
- Create: `tests/test_lite_mcp_binary_artifacts.py`
- Modify: `content/distribution/platform-targets.yaml`

- [ ] **Step 1: Add failing Codex manifest assertion**

In `tests/test_plugin_artifacts.py`, update `_assert_codex_mcp_server` so the
stable expected command is plugin-local and not Node:

```python
        self.assertNotEqual(server["command"], "node")
        self.assertEqual(server["command"], "./bin/qiongli-literature-provider")
        self.assertEqual(server["args"], ["--transport", "stdio"])
```

- [ ] **Step 2: Add failing Claude manifest assertion**

In `_assert_claude_manifest_mcp_server` and
`_assert_claude_zip_manifest_mcp_server`, replace the Node expectations with:

```python
        self.assertNotEqual(server["command"], "node")
        self.assertEqual(
            server["command"],
            "${CLAUDE_PLUGIN_ROOT}/bin/qiongli-literature-provider",
        )
        self.assertEqual(server["args"], ["--transport", "stdio"])
        self.assertEqual(server["cwd"], "${CLAUDE_PLUGIN_ROOT}")
```

- [ ] **Step 3: Add artifact binary presence tests**

Create `tests/test_lite_mcp_binary_artifacts.py`:

```python
from __future__ import annotations

import json
import tarfile
import tempfile
import unittest
import zipfile
from pathlib import Path

from scripts.build_plugin_artifacts import build_artifacts
from qiongli.source_layout import RepoLayout


REPO_ROOT = Path(__file__).resolve().parents[1]


class LiteMCPBinaryArtifactTests(unittest.TestCase):
    def test_codex_plugin_contains_lite_mcp_binary(self) -> None:
        tag = (RepoLayout(REPO_ROOT).workflow / "VERSION").read_text(encoding="utf-8").strip()
        with tempfile.TemporaryDirectory() as tmp_dir:
            artifacts = build_artifacts(REPO_ROOT, tag, Path(tmp_dir))
            codex = next(path for path in artifacts if "-codex-plugin-" in path.name)
            with tarfile.open(codex, "r:gz") as archive:
                names = set(archive.getnames())
                member = next(name for name in names if name.endswith("/plugins/qiongli/.mcp.json"))
                manifest = json.loads(archive.extractfile(member).read().decode("utf-8"))

        self.assertTrue(any(name.endswith("/plugins/qiongli/bin/qiongli-literature-provider") for name in names))
        server = manifest["mcpServers"]["qiongli"]
        self.assertEqual(server["command"], "./bin/qiongli-literature-provider")
        self.assertEqual(server["args"], ["--transport", "stdio"])

    def test_direct_desktop_plugin_contains_lite_mcp_binary(self) -> None:
        tag = (RepoLayout(REPO_ROOT).workflow / "VERSION").read_text(encoding="utf-8").strip()
        with tempfile.TemporaryDirectory() as tmp_dir:
            artifacts = build_artifacts(REPO_ROOT, tag, Path(tmp_dir))
            desktop = next(path for path in artifacts if "claude-desktop-plugin" in path.name)
            with zipfile.ZipFile(desktop) as archive:
                names = set(archive.namelist())
                manifest = json.loads(archive.read("qiongli/.claude-plugin/plugin.json").decode("utf-8"))

        self.assertIn("qiongli/bin/qiongli-literature-provider", names)
        server = manifest["mcpServers"]["qiongli"]
        self.assertEqual(server["command"], "${CLAUDE_PLUGIN_ROOT}/bin/qiongli-literature-provider")
        self.assertEqual(server["args"], ["--transport", "stdio"])
```

- [ ] **Step 4: Run RED tests**

Run:

```bash
.venv/bin/python -m unittest tests.test_plugin_artifacts tests.test_lite_mcp_binary_artifacts -v
```

Expected: failures showing manifests still call `node` and plugin artifacts do
not contain `bin/qiongli-literature-provider`.

## Task 3: Scaffold The Rust Lite MCP Runtime

**Files:**
- Create: `packages/qiongli-lite-mcp/Cargo.toml`
- Create: `packages/qiongli-lite-mcp/src/main.rs`
- Create: `packages/qiongli-lite-mcp/src/lib.rs`
- Create: `packages/qiongli-lite-mcp/src/mcp/mod.rs`
- Create: `packages/qiongli-lite-mcp/src/mcp/server.rs`
- Create: `packages/qiongli-lite-mcp/src/tools/mod.rs`
- Create: `packages/qiongli-lite-mcp/src/tools/definitions.rs`
- Create: `packages/qiongli-lite-mcp/tests/mcp_server.rs`

- [ ] **Step 1: Add failing Rust MCP initialize/list integration test**

Create `packages/qiongli-lite-mcp/tests/mcp_server.rs`:

```rust
use qiongli_lite_mcp::mcp::server::{McpRequest, McpServer};
use serde_json::json;

#[test]
fn initialize_and_tools_list_return_json_rpc_results() {
    let server = McpServer::new("qiongli-literature-provider", "0.1.0");

    let initialize = server.handle(McpRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "initialize".to_string(),
        params: Some(json!({"protocolVersion": "2025-11-25"})),
    });
    assert_eq!(initialize["result"]["serverInfo"]["name"], "qiongli-literature-provider");

    let tools = server.handle(McpRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(2)),
        method: "tools/list".to_string(),
        params: Some(json!({})),
    });
    let names: Vec<&str> = tools["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|tool| tool["name"].as_str().unwrap())
        .collect();
    assert!(names.contains(&"qiongli_literature_status"));
}
```

- [ ] **Step 2: Run RED**

Run:

```bash
cd packages/qiongli-lite-mcp && cargo test
```

Expected: fail because the Rust package does not exist.

- [ ] **Step 3: Add Cargo manifest**

Create `packages/qiongli-lite-mcp/Cargo.toml`:

```toml
[package]
name = "qiongli-lite-mcp"
version = "0.1.0"
edition = "2021"
license = "MIT"

[[bin]]
name = "qiongli-literature-provider"
path = "src/main.rs"

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "1"
reqwest = { version = "0.12", default-features = false, features = ["blocking", "json", "rustls-tls"] }
quick-xml = "0.36"
url = "2"
```

- [ ] **Step 4: Add minimal Rust library and server**

Create `packages/qiongli-lite-mcp/src/lib.rs`:

```rust
pub mod mcp;
pub mod tools;
```

Create `packages/qiongli-lite-mcp/src/mcp/mod.rs`:

```rust
pub mod server;
```

Create `packages/qiongli-lite-mcp/src/tools/mod.rs`:

```rust
pub mod definitions;
```

Create `packages/qiongli-lite-mcp/src/tools/definitions.rs`:

```rust
use serde::Serialize;
use serde_json::json;

#[derive(Debug, Clone, Serialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: serde_json::Value,
}

pub fn lite_tool_definitions() -> Vec<ToolDefinition> {
    vec![ToolDefinition {
        name: "qiongli_literature_status".to_string(),
        description: "Report configured literature providers and capability mode without exposing secrets.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {},
            "additionalProperties": false
        }),
    }]
}
```

Create `packages/qiongli-lite-mcp/src/mcp/server.rs`:

```rust
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::tools::definitions::lite_tool_definitions;

#[derive(Debug, Clone, Deserialize)]
pub struct McpRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Debug, Clone)]
pub struct McpServer {
    name: String,
    version: String,
}

impl McpServer {
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
        }
    }

    pub fn handle(&self, request: McpRequest) -> Value {
        match request.method.as_str() {
            "initialize" => self.result(request.id, json!({
                "protocolVersion": request
                    .params
                    .as_ref()
                    .and_then(|params| params.get("protocolVersion"))
                    .and_then(Value::as_str)
                    .unwrap_or("2025-11-25"),
                "capabilities": { "tools": {} },
                "serverInfo": {
                    "name": self.name,
                    "version": self.version
                }
            })),
            "tools/list" => self.result(request.id, json!({
                "tools": lite_tool_definitions()
            })),
            _ => self.error(request.id, -32601, format!("Method not found: {}", request.method)),
        }
    }

    fn result(&self, id: Option<Value>, result: Value) -> Value {
        json!({"jsonrpc": "2.0", "id": id, "result": result})
    }

    fn error(&self, id: Option<Value>, code: i64, message: String) -> Value {
        json!({"jsonrpc": "2.0", "id": id, "error": {"code": code, "message": message}})
    }
}
```

Create `packages/qiongli-lite-mcp/src/main.rs`:

```rust
use std::io::{self, BufRead, Write};

use qiongli_lite_mcp::mcp::server::{McpRequest, McpServer};
use serde_json::json;

fn main() {
    let server = McpServer::new("qiongli-literature-provider", env!("CARGO_PKG_VERSION"));
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let Ok(line) = line else {
            continue;
        };
        if line.trim().is_empty() {
            continue;
        }
        let response = match serde_json::from_str::<McpRequest>(&line) {
            Ok(request) => server.handle(request),
            Err(_) => json!({
                "jsonrpc": "2.0",
                "id": null,
                "error": {"code": -32700, "message": "Parse error"}
            }),
        };
        writeln!(stdout, "{}", response).expect("failed to write MCP response");
        stdout.flush().expect("failed to flush MCP response");
    }
}
```

- [ ] **Step 5: Run GREEN**

Run:

```bash
cd packages/qiongli-lite-mcp && cargo test
```

Expected: pass.

- [ ] **Step 6: Commit**

```bash
git add packages/qiongli-lite-mcp
git commit -m "feat(lite-mcp): scaffold rust MCP runtime"
```

## Task 4: Implement Provider Config Parity In Rust

**Files:**
- Create: `packages/qiongli-lite-mcp/src/config/mod.rs`
- Create: `packages/qiongli-lite-mcp/src/config/provider_config.rs`
- Create: `packages/qiongli-lite-mcp/tests/provider_config.rs`
- Modify: `packages/qiongli-lite-mcp/src/lib.rs`
- Modify: `packages/qiongli-lite-mcp/src/mcp/server.rs`
- Modify: `packages/qiongli-lite-mcp/src/tools/definitions.rs`

- [ ] **Step 1: Write provider config tests**

Create `packages/qiongli-lite-mcp/tests/provider_config.rs`:

```rust
use qiongli_lite_mcp::config::provider_config::{
    provider_config_path, save_provider_value, summary,
};

#[test]
fn provider_config_path_uses_qiongli_config_home() {
    let temp = tempfile_dir();
    std::env::set_var("QIONGLI_CONFIG_HOME", &temp);

    let path = provider_config_path();
    assert_eq!(path, temp.join("providers.json"));

    std::env::remove_var("QIONGLI_CONFIG_HOME");
}

#[test]
fn save_and_summarize_provider_value_without_exposing_secret() {
    let temp = tempfile_dir();
    std::env::set_var("QIONGLI_CONFIG_HOME", &temp);

    save_provider_value("openalex", "api_key", "openalex-secret-key").unwrap();
    let status = summary().unwrap();
    let serialized = serde_json::to_string(&status).unwrap();

    assert_eq!(status.providers["openalex"], "configured");
    assert_eq!(status.providers["arxiv"], "configured");
    assert!(!serialized.contains("openalex-secret-key"));

    std::env::remove_var("QIONGLI_CONFIG_HOME");
}

fn tempfile_dir() -> std::path::PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!(
        "qiongli-lite-mcp-test-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&path).unwrap();
    path
}
```

- [ ] **Step 2: Run RED**

Run:

```bash
cd packages/qiongli-lite-mcp && cargo test --test provider_config
```

Expected: fail because the config module is missing.

- [ ] **Step 3: Add config module**

Create `packages/qiongli-lite-mcp/src/config/mod.rs`:

```rust
pub mod provider_config;
```

Add `pub mod config;` to `src/lib.rs`.

Create `packages/qiongli-lite-mcp/src/config/provider_config.rs`:

```rust
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("unsupported provider field: {0}.{1}")]
    UnsupportedField(String, String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct ProviderFile {
    #[serde(default = "default_version")]
    version: u64,
    #[serde(default)]
    providers: BTreeMap<String, BTreeMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProviderSummary {
    pub status: String,
    pub capability_mode: String,
    pub providers: BTreeMap<String, String>,
    pub missing: Vec<String>,
}

fn default_version() -> u64 {
    1
}

pub fn provider_config_path() -> PathBuf {
    if let Ok(root) = std::env::var("QIONGLI_CONFIG_HOME") {
        let trimmed = root.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed).join("providers.json");
        }
    }
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".config")
        .join("qiongli")
        .join("providers.json")
}

pub fn save_provider_value(provider: &str, field: &str, value: &str) -> Result<PathBuf, ConfigError> {
    let provider = normalize(provider);
    let field = normalize(field);
    if !is_supported_field(&provider, &field) {
        return Err(ConfigError::UnsupportedField(provider, field));
    }

    let path = provider_config_path();
    let mut file = read_provider_file()?;
    file.version = 1;
    let entry = file.providers.entry(provider).or_default();
    entry.insert("enabled".to_string(), serde_json::Value::Bool(true));
    entry.insert(field, serde_json::Value::String(value.to_string()));
    write_provider_file(&path, &file)?;
    Ok(path)
}

pub fn summary() -> Result<ProviderSummary, ConfigError> {
    let file = read_provider_file()?;
    let mut providers = BTreeMap::from([
        ("openalex".to_string(), "missing".to_string()),
        ("semantic_scholar".to_string(), "missing".to_string()),
        ("crossref".to_string(), "missing".to_string()),
        ("pubmed".to_string(), "missing".to_string()),
        ("arxiv".to_string(), "configured".to_string()),
    ]);
    let mut missing = Vec::new();

    for (provider, field, aliases) in provider_fields() {
        if configured_field(&file, provider, field, aliases) {
            providers.insert(provider.to_string(), "configured".to_string());
        } else {
            missing.push(format!("{provider}.{field}"));
        }
    }

    let capability_mode = if providers.values().any(|status| status == "configured") {
        "provider_connected"
    } else {
        "strategy_only"
    };

    Ok(ProviderSummary {
        status: "ok".to_string(),
        capability_mode: capability_mode.to_string(),
        providers,
        missing,
    })
}

fn read_provider_file() -> Result<ProviderFile, ConfigError> {
    let path = provider_config_path();
    if !path.is_file() {
        return Ok(ProviderFile::default());
    }
    let text = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&text)?)
}

fn write_provider_file(path: &PathBuf, file: &ProviderFile) -> Result<(), ConfigError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, format!("{}\n", serde_json::to_string_pretty(file)?))?;
    Ok(())
}

fn configured_field(file: &ProviderFile, provider: &str, field: &str, aliases: &[&str]) -> bool {
    if let Some(entry) = file.providers.get(provider) {
        if entry
            .get(field)
            .and_then(serde_json::Value::as_str)
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false)
        {
            return true;
        }
    }
    aliases.iter().any(|alias| {
        std::env::var(alias)
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false)
    })
}

fn is_supported_field(provider: &str, field: &str) -> bool {
    provider_fields()
        .iter()
        .any(|(known_provider, known_field, _)| *known_provider == provider && *known_field == field)
}

fn normalize(value: &str) -> String {
    value.trim().to_ascii_lowercase().replace('-', "_")
}

fn provider_fields() -> Vec<(&'static str, &'static str, Vec<&'static str>)> {
    vec![
        ("openalex", "api_key", vec!["QIONGLI_OPENALEX_API_KEY", "OPENALEX_API_KEY", "QIONGLI_MCPB_OPENALEX_API_KEY"]),
        ("openalex", "email", vec!["QIONGLI_OPENALEX_EMAIL", "OPENALEX_EMAIL", "QIONGLI_MCPB_OPENALEX_EMAIL"]),
        ("semantic_scholar", "api_key", vec!["QIONGLI_SEMANTIC_SCHOLAR_API_KEY", "SEMANTIC_SCHOLAR_API_KEY", "S2_API_KEY", "QIONGLI_MCPB_SEMANTIC_SCHOLAR_API_KEY"]),
        ("crossref", "email", vec!["QIONGLI_CROSSREF_EMAIL", "CROSSREF_EMAIL", "QIONGLI_MCPB_CROSSREF_EMAIL"]),
        ("pubmed", "api_key", vec!["QIONGLI_NCBI_API_KEY", "NCBI_API_KEY", "PUBMED_API_KEY", "QIONGLI_MCPB_PUBMED_API_KEY"]),
    ]
}
```

- [ ] **Step 4: Wire config tools into MCP dispatch**

Add tool definitions for `qiongli_config_status`, `qiongli_save_provider_config`,
`qiongli_configure_provider`, and `qiongli_open_config_wizard`. Update
`McpServer::handle` so `tools/call` dispatches at least `qiongli_config_status`
and `qiongli_save_provider_config`.

- [ ] **Step 5: Run GREEN**

Run:

```bash
cd packages/qiongli-lite-mcp && cargo test
```

Expected: pass.

- [ ] **Step 6: Commit**

```bash
git add packages/qiongli-lite-mcp
git commit -m "feat(lite-mcp): share provider config contract"
```

## Task 5: Add Search Plan And Literature Status

**Files:**
- Create: `packages/qiongli-lite-mcp/src/searchplan.rs`
- Create: `packages/qiongli-lite-mcp/tests/searchplan.rs`
- Modify: `packages/qiongli-lite-mcp/src/lib.rs`
- Modify: `packages/qiongli-lite-mcp/src/mcp/server.rs`
- Modify: `packages/qiongli-lite-mcp/src/tools/definitions.rs`

- [ ] **Step 1: Write search plan tests**

Create `packages/qiongli-lite-mcp/tests/searchplan.rs`:

```rust
use qiongli_lite_mcp::searchplan::{build_search_plan, SearchPlanInput};

#[test]
fn search_plan_records_hybrid_mode_when_provider_and_native_are_available() {
    let plan = build_search_plan(SearchPlanInput {
        query: "platform governance".to_string(),
        search_mode: Some("review".to_string()),
        provider_connected: true,
        native_search_usable: true,
    });

    assert_eq!(plan.search_execution_mode, "hybrid_search");
    assert_eq!(plan.provider_capability_mode, "provider_connected");
    assert_eq!(plan.native_search_queries, vec!["platform governance"]);
}
```

- [ ] **Step 2: Run RED**

Run:

```bash
cd packages/qiongli-lite-mcp && cargo test --test searchplan
```

Expected: fail because `searchplan` does not exist.

- [ ] **Step 3: Implement search plan builder**

Create `packages/qiongli-lite-mcp/src/searchplan.rs`:

```rust
use serde::Serialize;

#[derive(Debug, Clone)]
pub struct SearchPlanInput {
    pub query: String,
    pub search_mode: Option<String>,
    pub provider_connected: bool,
    pub native_search_usable: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchPlan {
    pub query: String,
    pub search_mode: String,
    pub search_execution_mode: String,
    pub provider_capability_mode: String,
    pub native_search_queries: Vec<String>,
}

pub fn build_search_plan(input: SearchPlanInput) -> SearchPlan {
    let search_execution_mode = match (input.provider_connected, input.native_search_usable) {
        (true, true) => "hybrid_search",
        (true, false) => "provider_connected",
        (false, true) => "native_only",
        (false, false) => "strategy_only",
    };
    let provider_capability_mode = if input.provider_connected {
        "provider_connected"
    } else {
        "strategy_only"
    };
    SearchPlan {
        native_search_queries: vec![input.query.clone()],
        query: input.query,
        search_mode: input.search_mode.unwrap_or_else(|| "topic".to_string()),
        search_execution_mode: search_execution_mode.to_string(),
        provider_capability_mode: provider_capability_mode.to_string(),
    }
}
```

Add `pub mod searchplan;` to `src/lib.rs`.

- [ ] **Step 4: Add literature status and search plan MCP handlers**

Wire `qiongli_literature_status` to provider config summary and
`qiongli_search_plan` to `build_search_plan`. Include provider capabilities for
OpenAlex, Semantic Scholar, Crossref, PubMed, and arXiv.

- [ ] **Step 5: Run GREEN**

Run:

```bash
cd packages/qiongli-lite-mcp && cargo test
```

Expected: pass.

- [ ] **Step 6: Commit**

```bash
git add packages/qiongli-lite-mcp
git commit -m "feat(lite-mcp): add literature status and planning"
```

## Task 6: Add Provider Search Clients

**Files:**
- Create: `packages/qiongli-lite-mcp/src/providers/mod.rs`
- Create: `packages/qiongli-lite-mcp/src/providers/search.rs`
- Create: `packages/qiongli-lite-mcp/src/providers/openalex.rs`
- Create: `packages/qiongli-lite-mcp/src/providers/semantic_scholar.rs`
- Create: `packages/qiongli-lite-mcp/src/providers/crossref.rs`
- Create: `packages/qiongli-lite-mcp/src/providers/pubmed.rs`
- Create: `packages/qiongli-lite-mcp/src/providers/arxiv.rs`
- Create: `packages/qiongli-lite-mcp/tests/providers.rs`
- Modify: `packages/qiongli-lite-mcp/src/lib.rs`
- Modify: `packages/qiongli-lite-mcp/src/mcp/server.rs`

- [ ] **Step 1: Write OpenAlex normalization test**

Create `packages/qiongli-lite-mcp/tests/providers.rs`:

```rust
use qiongli_lite_mcp::providers::openalex::normalize_openalex_response;

#[test]
fn openalex_response_normalizes_title_year_and_doi() {
    let fixture = include_str!("../../../content/mcp-contracts/fixtures/openalex-search-response.json");
    let results = normalize_openalex_response(fixture).unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "A Test Paper");
    assert_eq!(results[0].doi.as_deref(), Some("10.1234/example"));
    assert_eq!(results[0].year, Some(2025));
    assert_eq!(results[0].provider, "openalex");
}
```

- [ ] **Step 2: Run RED**

Run:

```bash
cd packages/qiongli-lite-mcp && cargo test --test providers
```

Expected: fail because provider modules are missing.

- [ ] **Step 3: Add common provider types**

Create `packages/qiongli-lite-mcp/src/providers/mod.rs`:

```rust
pub mod arxiv;
pub mod crossref;
pub mod openalex;
pub mod pubmed;
pub mod search;
pub mod semantic_scholar;
```

Create `packages/qiongli-lite-mcp/src/providers/search.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct SearchInput {
    pub query: String,
    pub search_mode: Option<String>,
    pub limit: Option<usize>,
    pub per_provider_limit: Option<usize>,
    pub total_limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LiteratureResult {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doi: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub venue: Option<String>,
    pub provider: String,
    pub providers: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchDiagnostics {
    pub status: String,
    pub provider_counts: std::collections::BTreeMap<String, usize>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchOutput {
    pub status: String,
    pub results: Vec<LiteratureResult>,
    pub diagnostics: SearchDiagnostics,
}
```

Add `pub mod providers;` to `src/lib.rs`.

- [ ] **Step 4: Implement OpenAlex normalizer**

Create `packages/qiongli-lite-mcp/src/providers/openalex.rs`:

```rust
use serde::Deserialize;
use thiserror::Error;

use crate::providers::search::LiteratureResult;

#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Deserialize)]
struct OpenAlexResponse {
    #[serde(default)]
    results: Vec<OpenAlexWork>,
}

#[derive(Debug, Deserialize)]
struct OpenAlexWork {
    display_name: Option<String>,
    publication_year: Option<i64>,
    doi: Option<String>,
    primary_location: Option<OpenAlexLocation>,
}

#[derive(Debug, Deserialize)]
struct OpenAlexLocation {
    source: Option<OpenAlexSource>,
}

#[derive(Debug, Deserialize)]
struct OpenAlexSource {
    display_name: Option<String>,
}

pub fn normalize_openalex_response(payload: &str) -> Result<Vec<LiteratureResult>, ProviderError> {
    let response: OpenAlexResponse = serde_json::from_str(payload)?;
    Ok(response
        .results
        .into_iter()
        .filter_map(|work| {
            let title = work.display_name?;
            Some(LiteratureResult {
                title,
                doi: work.doi.as_deref().map(normalize_doi),
                year: work.publication_year,
                venue: work
                    .primary_location
                    .and_then(|location| location.source)
                    .and_then(|source| source.display_name),
                provider: "openalex".to_string(),
                providers: vec!["openalex".to_string()],
            })
        })
        .collect())
}

fn normalize_doi(raw: &str) -> String {
    raw.trim()
        .trim_start_matches("https://doi.org/")
        .trim_start_matches("http://doi.org/")
        .trim_start_matches("doi:")
        .to_ascii_lowercase()
}
```

- [ ] **Step 5: Add remaining provider tests and normalizers**

Add tests in `tests/providers.rs` for:

- `semantic_scholar_response_normalizes_title_year_and_doi`
- `crossref_response_normalizes_title_year_and_doi`
- `pubmed_summary_response_normalizes_title_year_and_doi`
- `arxiv_atom_response_normalizes_title_year_and_url`

Create one fixture per provider under `content/mcp-contracts/fixtures/`. Each
normalizer returns `Vec<LiteratureResult>` and must not perform network calls.

- [ ] **Step 6: Add network clients after normalizers pass**

Add one client function per provider:

```rust
pub fn search_openalex(client: &reqwest::blocking::Client, base_url: &str, input: &SearchInput) -> Result<Vec<LiteratureResult>, ProviderError>
```

Use the same pattern for Semantic Scholar, Crossref, PubMed, and arXiv. Unit
tests for network clients must use local mock HTTP servers or pure response
normalizers; no unit test may call an external provider.

- [ ] **Step 7: Wire `qiongli_literature_search`**

Add a tool definition and dispatch handler. The handler returns:

```json
{
  "status": "ok",
  "search_plan": {},
  "diagnostics": {},
  "results": []
}
```

- [ ] **Step 8: Run GREEN**

Run:

```bash
cd packages/qiongli-lite-mcp && cargo test
```

Expected: pass.

- [ ] **Step 9: Commit**

```bash
git add content/mcp-contracts packages/qiongli-lite-mcp
git commit -m "feat(lite-mcp): add rust provider search"
```

## Task 7: Add Evidence Export And Zotero Import Files

**Files:**
- Create: `packages/qiongli-lite-mcp/src/zotero/mod.rs`
- Create: `packages/qiongli-lite-mcp/src/zotero/export.rs`
- Create: `packages/qiongli-lite-mcp/tests/zotero_export.rs`
- Modify: `packages/qiongli-lite-mcp/src/lib.rs`
- Modify: `packages/qiongli-lite-mcp/src/mcp/server.rs`
- Modify: `packages/qiongli-lite-mcp/src/tools/definitions.rs`

- [ ] **Step 1: Write Zotero export tests**

Create `packages/qiongli-lite-mcp/tests/zotero_export.rs`:

```rust
use qiongli_lite_mcp::providers::search::LiteratureResult;
use qiongli_lite_mcp::zotero::export::export_import_files;

#[test]
fn export_import_files_includes_ris_bibtex_csl_and_report() {
    let files = export_import_files(vec![LiteratureResult {
        title: "A Test Paper".to_string(),
        doi: Some("10.1234/example".to_string()),
        year: Some(2025),
        venue: Some("Journal of Tests".to_string()),
        provider: "openalex".to_string(),
        providers: vec!["openalex".to_string()],
    }]);

    assert!(files.contains_key("references.json"));
    assert!(files.contains_key("references.ris"));
    assert!(files.contains_key("bibliography.bib"));
    assert!(files.contains_key("zotero-import-report.md"));
    assert!(files["references.ris"].contains("TY  - JOUR"));
    assert!(files["bibliography.bib"].contains("@article"));
}
```

- [ ] **Step 2: Run RED**

Run:

```bash
cd packages/qiongli-lite-mcp && cargo test --test zotero_export
```

Expected: fail because Zotero export module is missing.

- [ ] **Step 3: Implement Zotero export files**

Create `packages/qiongli-lite-mcp/src/zotero/mod.rs`:

```rust
pub mod export;
```

Create `packages/qiongli-lite-mcp/src/zotero/export.rs`:

```rust
use std::collections::BTreeMap;

use crate::providers::search::LiteratureResult;

pub fn export_import_files(records: Vec<LiteratureResult>) -> BTreeMap<String, String> {
    let mut files = BTreeMap::new();
    files.insert(
        "references.json".to_string(),
        serde_json::to_string_pretty(&records).unwrap_or_else(|_| "[]".to_string()),
    );
    files.insert("references.ris".to_string(), to_ris(&records));
    files.insert("bibliography.bib".to_string(), to_bibtex(&records));
    files.insert("zotero-import-report.md".to_string(), to_report(&records));
    files
}

fn to_ris(records: &[LiteratureResult]) -> String {
    records
        .iter()
        .map(|record| {
            format!(
                "TY  - JOUR\nTI  - {}\nPY  - {}\nDO  - {}\nER  -\n",
                record.title,
                record.year.map(|year| year.to_string()).unwrap_or_default(),
                record.doi.clone().unwrap_or_default()
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn to_bibtex(records: &[LiteratureResult]) -> String {
    records
        .iter()
        .enumerate()
        .map(|(index, record)| {
            format!(
                "@article{{qiongli{},\n  title = {{{}}},\n  year = {{{}}},\n  doi = {{{}}}\n}}\n",
                index + 1,
                record.title,
                record.year.map(|year| year.to_string()).unwrap_or_default(),
                record.doi.clone().unwrap_or_default()
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn to_report(records: &[LiteratureResult]) -> String {
    format!("# Zotero Import Report\n\nRecords: {}\n", records.len())
}
```

Add `pub mod zotero;` to `src/lib.rs`.

- [ ] **Step 4: Wire evidence export and Zotero export MCP handlers**

Add `qiongli_literature_export_evidence` and
`qiongli_zotero_export_import_files` dispatch handlers. They must not require
Zotero Desktop or the Zotero Companion.

- [ ] **Step 5: Run GREEN**

Run:

```bash
cd packages/qiongli-lite-mcp && cargo test
```

Expected: pass.

- [ ] **Step 6: Commit**

```bash
git add packages/qiongli-lite-mcp
git commit -m "feat(lite-mcp): export evidence and Zotero files"
```

## Task 8: Add Optional Zotero Companion Bridge

**Files:**
- Create: `packages/qiongli-lite-mcp/src/zotero/companion.rs`
- Create: `packages/qiongli-lite-mcp/tests/zotero_companion.rs`
- Modify: `packages/qiongli-lite-mcp/src/zotero/mod.rs`
- Modify: `packages/qiongli-lite-mcp/src/mcp/server.rs`
- Modify: `packages/qiongli-lite-mcp/src/tools/definitions.rs`

- [ ] **Step 1: Write loopback enforcement test**

Create `packages/qiongli-lite-mcp/tests/zotero_companion.rs`:

```rust
use qiongli_lite_mcp::zotero::companion::CompanionClient;

#[test]
fn rejects_non_loopback_connector_url() {
    let result = CompanionClient::new("http://example.com:23119");
    assert!(result.is_err());
}

#[test]
fn accepts_loopback_connector_url() {
    let result = CompanionClient::new("http://127.0.0.1:23119");
    assert!(result.is_ok());
}
```

- [ ] **Step 2: Run RED**

Run:

```bash
cd packages/qiongli-lite-mcp && cargo test --test zotero_companion
```

Expected: fail because Companion client is missing.

- [ ] **Step 3: Implement loopback-only Companion client**

Create `packages/qiongli-lite-mcp/src/zotero/companion.rs`:

```rust
use thiserror::Error;
use url::Url;

#[derive(Debug, Error)]
pub enum CompanionError {
    #[error("Zotero connector URL must use localhost or 127.0.0.1")]
    NonLoopback,
    #[error("invalid URL: {0}")]
    InvalidUrl(#[from] url::ParseError),
}

#[derive(Debug, Clone)]
pub struct CompanionClient {
    base_url: Url,
}

impl CompanionClient {
    pub fn new(raw: &str) -> Result<Self, CompanionError> {
        let base_url = Url::parse(raw)?;
        let host = base_url.host_str().unwrap_or("");
        if host != "127.0.0.1" && host != "localhost" {
            return Err(CompanionError::NonLoopback);
        }
        Ok(Self { base_url })
    }

    pub fn base_url(&self) -> &Url {
        &self.base_url
    }
}
```

Add `pub mod companion;` to `src/zotero/mod.rs`.

- [ ] **Step 4: Wire Zotero status**

Add `qiongli_zotero_status`. It returns:

```json
{
  "status": "fallback_only",
  "connector": {"available": false},
  "companion": {"available": false},
  "fallback_import_files": {
    "available": true,
    "formats": ["references.json", "references.ris", "bibliography.bib", "zotero-import-report.md"]
  }
}
```

When a loopback connector is reachable and Companion endpoint responds, return
`status: "ok"`.

- [ ] **Step 5: Run GREEN**

Run:

```bash
cd packages/qiongli-lite-mcp && cargo test
```

Expected: pass.

- [ ] **Step 6: Commit**

```bash
git add packages/qiongli-lite-mcp
git commit -m "feat(lite-mcp): support optional Zotero companion"
```

## Task 9: Add Preview-Only Orchestrator Tools

**Files:**
- Create: `packages/qiongli-lite-mcp/src/orchestrator/mod.rs`
- Create: `packages/qiongli-lite-mcp/src/orchestrator/preview.rs`
- Create: `packages/qiongli-lite-mcp/tests/orchestrator_preview.rs`
- Modify: `packages/qiongli-lite-mcp/src/lib.rs`
- Modify: `packages/qiongli-lite-mcp/src/mcp/server.rs`
- Modify: `packages/qiongli-lite-mcp/src/tools/definitions.rs`

- [ ] **Step 1: Write preview behavior tests**

Create `packages/qiongli-lite-mcp/tests/orchestrator_preview.rs`:

```rust
use qiongli_lite_mcp::orchestrator::preview::{build_task_plan, TaskPlanInput};

#[test]
fn task_plan_preview_does_not_allow_agent_execution() {
    let plan = build_task_plan(TaskPlanInput {
        task_id: "B1".to_string(),
        paper_type: "systematic-review".to_string(),
        topic: "ai-feedback".to_string(),
    });

    assert_eq!(plan.mode, "preview");
    assert_eq!(plan.runtime_profile, "marketplace_lite");
    assert!(!plan.run_agents_allowed);
}
```

- [ ] **Step 2: Run RED**

Run:

```bash
cd packages/qiongli-lite-mcp && cargo test --test orchestrator_preview
```

Expected: fail because preview module is missing.

- [ ] **Step 3: Implement preview module**

Create `packages/qiongli-lite-mcp/src/orchestrator/mod.rs`:

```rust
pub mod preview;
```

Create `packages/qiongli-lite-mcp/src/orchestrator/preview.rs`:

```rust
use serde::Serialize;

#[derive(Debug, Clone)]
pub struct TaskPlanInput {
    pub task_id: String,
    pub paper_type: String,
    pub topic: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TaskPlanPreview {
    pub mode: String,
    pub runtime_profile: String,
    pub run_agents_allowed: bool,
    pub task_id: String,
    pub paper_type: String,
    pub topic: String,
}

pub fn build_task_plan(input: TaskPlanInput) -> TaskPlanPreview {
    TaskPlanPreview {
        mode: "preview".to_string(),
        runtime_profile: "marketplace_lite".to_string(),
        run_agents_allowed: false,
        task_id: input.task_id,
        paper_type: input.paper_type,
        topic: input.topic,
    }
}
```

Add `pub mod orchestrator;` to `src/lib.rs`.

- [ ] **Step 4: Wire route and task-plan MCP handlers**

Add `qiongli_orchestrator_route` and `qiongli_task_plan`. Both return
`runtime_profile: "marketplace_lite"` and never launch local agents.

- [ ] **Step 5: Run GREEN**

Run:

```bash
cd packages/qiongli-lite-mcp && cargo test
```

Expected: pass.

- [ ] **Step 6: Commit**

```bash
git add packages/qiongli-lite-mcp
git commit -m "feat(lite-mcp): add preview workflow tools"
```

## Task 10: Build Platform Binaries For Release Staging

**Files:**
- Create: `tooling/scripts/build_lite_mcp.py`
- Create: `scripts/build_lite_mcp.py`
- Create: `tests/test_lite_mcp_binary_artifacts.py`
- Modify: `tooling/scripts/build_plugin_artifacts.py`

- [ ] **Step 1: Add build script test**

Extend `tests/test_lite_mcp_binary_artifacts.py`:

```python
    def test_build_lite_mcp_stages_current_platform_binary(self) -> None:
        from tooling.scripts.build_lite_mcp import build_current_platform

        with tempfile.TemporaryDirectory() as tmp_dir:
            binary = build_current_platform(REPO_ROOT, Path(tmp_dir))

        self.assertTrue(binary.is_file())
        self.assertEqual(binary.name, "qiongli-literature-provider")
```

- [ ] **Step 2: Run RED**

Run:

```bash
.venv/bin/python -m unittest tests.test_lite_mcp_binary_artifacts -v
```

Expected: fail because `tooling/scripts/build_lite_mcp.py` is missing.

- [ ] **Step 3: Implement Rust build script**

Create `tooling/scripts/build_lite_mcp.py` with:

- `build_current_platform(root: Path, out_dir: Path) -> Path`
- `build_target(root: Path, out_dir: Path, target: str) -> Path`
- `build_all_platforms(root: Path, out_dir: Path) -> list[Path]`
- target matrix:
  - `aarch64-apple-darwin`
  - `x86_64-apple-darwin`
  - `x86_64-unknown-linux-gnu`
  - `aarch64-unknown-linux-gnu`
  - `x86_64-pc-windows-msvc`
- command: `cargo build --release --manifest-path packages/qiongli-lite-mcp/Cargo.toml`
- target-specific command: `cargo build --release --target <target> --manifest-path packages/qiongli-lite-mcp/Cargo.toml`
- executable permission fix for non-Windows artifacts

Create `scripts/build_lite_mcp.py` using the wrapper style used by the existing
`scripts/build_literature_mcpb.py`.

- [ ] **Step 4: Run GREEN**

Run:

```bash
.venv/bin/python -m unittest tests.test_lite_mcp_binary_artifacts -v
```

Expected: pass on the current development platform.

- [ ] **Step 5: Commit**

```bash
git add tooling/scripts/build_lite_mcp.py scripts/build_lite_mcp.py tests/test_lite_mcp_binary_artifacts.py
git commit -m "build(lite-mcp): stage rust binaries"
```

## Task 11: Switch Marketplace Plugin Packaging To Rust Lite Binary

**Files:**
- Modify: `tooling/scripts/build_plugin_artifacts.py`
- Modify: `content/distribution/platform-targets.yaml`
- Modify: `tooling/scripts/validate_marketplace_install.py`
- Modify: `tests/test_plugin_artifacts.py`
- Modify: `tests/test_lite_mcp_binary_artifacts.py`

- [ ] **Step 1: Update manifest writers**

Change `_write_claude_manifest` so `mcpServers` uses:

```python
"mcpServers": {
    plugin.mcp_server_name: {
        "command": "${CLAUDE_PLUGIN_ROOT}/bin/qiongli-literature-provider",
        "args": ["--transport", "stdio"],
        "cwd": "${CLAUDE_PLUGIN_ROOT}",
    }
}
```

Change `_write_codex_mcp_manifest` so it uses:

```python
"mcpServers": {
    server_name: {
        "command": "./bin/qiongli-literature-provider",
        "args": ["--transport", "stdio"],
        "cwd": ".",
        "startup_timeout_sec": 20,
        "tool_timeout_sec": 60,
    }
}
```

- [ ] **Step 2: Add binary copy helper**

Add `_copy_lite_mcp_runtime(root, dest_plugin_root)` that calls
`build_current_platform(root, work_dir)` during local artifact tests and copies
the resulting executable to:

```python
dest_plugin_root / "bin" / "qiongli-literature-provider"
```

- [ ] **Step 3: Update platform targets**

Set Codex, Claude Code, and Claude Desktop direct plugin targets:

```yaml
bundled_mcp_mode: marketplace-lite-binary
source_inputs:
  - content/distribution/plugins.yaml
  - content/workflow/**
  - packages/qiongli-lite-mcp/**
required_paths:
  - bin/qiongli-literature-provider
```

Remove `mcp/qiongli-literature-provider/index.mjs` from required paths for
these marketplace-lite binary targets.

- [ ] **Step 4: Update validator**

In `tooling/scripts/validate_marketplace_install.py`, assert that marketplace
plugin manifests do not include:

- `"command": "node"`
- `"command": "python"`
- `"command": "qiongli"`
- `mcp/qiongli-literature-provider/index.mjs`

Assert the expected binary command path for each platform manifest.

- [ ] **Step 5: Run GREEN**

Run:

```bash
.venv/bin/python -m unittest tests.test_plugin_artifacts tests.test_lite_mcp_binary_artifacts -v
.venv/bin/python tooling/scripts/validate_marketplace_install.py --skip-client-activation
```

Expected: pass.

- [ ] **Step 6: Commit**

```bash
git add tooling/scripts/build_plugin_artifacts.py content/distribution/platform-targets.yaml tooling/scripts/validate_marketplace_install.py tests/test_plugin_artifacts.py tests/test_lite_mcp_binary_artifacts.py
git commit -m "feat(marketplace): bundle rust lite MCP executable"
```

## Task 12: Add Black-Box MCP Binary Contract Tests

**Files:**
- Create: `tests/test_lite_mcp_contract.py`
- Modify: `tooling/scripts/build_lite_mcp.py`

- [ ] **Step 1: Add subprocess MCP contract test**

Create `tests/test_lite_mcp_contract.py`:

```python
from __future__ import annotations

import json
import subprocess
import tempfile
import unittest
from pathlib import Path

from tooling.scripts.build_lite_mcp import build_current_platform


REPO_ROOT = Path(__file__).resolve().parents[1]


class LiteMCPContractTests(unittest.TestCase):
    def test_binary_initializes_and_lists_tools_without_node_or_python(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            binary = build_current_platform(REPO_ROOT, Path(tmp_dir))
            process = subprocess.run(
                [str(binary), "--transport", "stdio"],
                input=(
                    '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25"}}\n'
                    '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}\n'
                ),
                text=True,
                capture_output=True,
                check=False,
                timeout=10,
            )

        self.assertEqual(process.returncode, 0, msg=process.stderr)
        lines = [json.loads(line) for line in process.stdout.splitlines() if line.strip()]
        self.assertEqual(lines[0]["result"]["serverInfo"]["name"], "qiongli-literature-provider")
        names = {tool["name"] for tool in lines[1]["result"]["tools"]}
        self.assertIn("qiongli_literature_status", names)
        self.assertIn("qiongli_literature_search", names)
        self.assertIn("qiongli_zotero_export_import_files", names)
```

- [ ] **Step 2: Add config status black-box test**

Append a test that sets `QIONGLI_CONFIG_HOME` to a temp directory, calls
`qiongli_save_provider_config`, then calls `qiongli_config_status` and asserts
the raw secret does not appear in stdout.

- [ ] **Step 3: Run GREEN**

Run:

```bash
.venv/bin/python -m unittest tests.test_lite_mcp_contract -v
```

Expected: pass.

- [ ] **Step 4: Commit**

```bash
git add tests/test_lite_mcp_contract.py tooling/scripts/build_lite_mcp.py
git commit -m "test(lite-mcp): verify rust binary MCP contract"
```

## Task 13: Add Python Full CLI Parity Tests

**Files:**
- Create: `tests/test_lite_full_mcp_parity.py`
- Modify: `tests/test_mcp_contract_fixtures.py`

- [ ] **Step 1: Add tools/list parity test**

Create `tests/test_lite_full_mcp_parity.py`:

```python
from __future__ import annotations

import json
import subprocess
import tempfile
import unittest
from pathlib import Path

from tooling.scripts.build_lite_mcp import build_current_platform


REPO_ROOT = Path(__file__).resolve().parents[1]


class LiteFullMCPParityTests(unittest.TestCase):
    def test_lite_tools_are_subset_of_full_cli_tools(self) -> None:
        contract = json.loads(
            (REPO_ROOT / "content" / "mcp-contracts" / "lite-tools.json").read_text(
                encoding="utf-8"
            )
        )
        expected_lite_names = {tool["name"] for tool in contract["tools"]}

        with tempfile.TemporaryDirectory() as tmp_dir:
            binary = build_current_platform(REPO_ROOT, Path(tmp_dir))
            lite = subprocess.run(
                [str(binary), "--transport", "stdio"],
                input='{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}\n',
                text=True,
                capture_output=True,
                check=False,
                timeout=10,
            )

        lite_names = {
            tool["name"]
            for tool in json.loads(lite.stdout.splitlines()[0])["result"]["tools"]
        }

        self.assertTrue(expected_lite_names.issubset(lite_names))
```

Add a second test for Python Full CLI after the Full CLI tool list is easy to
launch in CI. It should assert that overlapping Lite tool names are present in
`qiongli mcp serve --transport stdio`.

- [ ] **Step 2: Run parity tests**

Run:

```bash
.venv/bin/python -m unittest tests.test_lite_full_mcp_parity -v
```

Expected: pass after the Lite binary tools/list is contract-complete.

- [ ] **Step 3: Commit**

```bash
git add tests/test_lite_full_mcp_parity.py tests/test_mcp_contract_fixtures.py
git commit -m "test(mcp): compare lite and full tool contracts"
```

## Task 14: Update MCPB To Reuse The Rust Lite Runtime

**Files:**
- Modify: `packages/qiongli-literature-mcpb/manifest.json`
- Modify: `packages/qiongli-literature-mcpb/README.md`
- Modify: `tooling/scripts/build_literature_mcpb.py`
- Modify: `tests/test_literature_mcpb_artifact.py`

- [ ] **Step 1: Add binary MCPB tests while keeping legacy tests explicit**

In `tests/test_literature_mcpb_artifact.py`, add:

```python
    def test_literature_mcpb_uses_self_contained_runtime(self) -> None:
        manifest = json.loads((PACKAGE_ROOT / "manifest.json").read_text(encoding="utf-8"))
        mcp_config = manifest["server"]["mcp_config"]

        self.assertNotEqual(mcp_config["command"], "node")
        self.assertIn("qiongli-literature-provider", mcp_config["command"])
        self.assertEqual(mcp_config["args"], ["--transport", "stdio"])
```

- [ ] **Step 2: Run RED**

Run:

```bash
.venv/bin/python -m unittest tests.test_literature_mcpb_artifact -v
```

Expected: fail because MCPB still declares Node.

- [ ] **Step 3: Update MCPB build to include Rust binary**

Change `tooling/scripts/build_literature_mcpb.py` so it stages:

```text
manifest.json
README.md
bin/qiongli-literature-provider
```

Keep Node server packaging behind a `--legacy-node` flag for one release train.

- [ ] **Step 4: Update MCPB manifest and README**

Set `server.mcp_config.command` to the package-local binary path supported by
Claude Desktop MCPB. Keep user config fields unchanged.

- [ ] **Step 5: Run GREEN**

Run:

```bash
.venv/bin/python -m unittest tests.test_literature_mcpb_artifact -v
```

Expected: pass.

- [ ] **Step 6: Commit**

```bash
git add packages/qiongli-literature-mcpb/manifest.json packages/qiongli-literature-mcpb/README.md tooling/scripts/build_literature_mcpb.py tests/test_literature_mcpb_artifact.py
git commit -m "feat(mcpb): package rust lite MCP executable"
```

## Task 15: Document Rust Marketplace Lite Versus Python Full CLI

**Files:**
- Modify: `docs/advanced/cross-platform-mcp.md`
- Modify: `docs/advanced/plugin-first-architecture.md`
- Modify: `docs/guide/install.md`
- Modify: `README.md`
- Modify: `README_CN.md`
- Modify: `content/workflow/SKILL.md`

- [ ] **Step 1: Update product model docs**

Add this table to `docs/advanced/cross-platform-mcp.md`:

```markdown
| Runtime profile | Install path | Requires user Node/Python | Main capabilities |
|---|---|---:|---|
| Marketplace Lite | Codex, Claude Code, or Claude Desktop plugin marketplace/direct plugin | No | Rust-built Literature Provider MCP, config wizard, search planning, evidence export, Zotero import files, optional Zotero Companion bridge |
| Full CLI | `qiongli install --profile full` or `qiongli mcp serve --transport stdio` | Python Qiongli runtime | Full MCP tools, orchestrator, task-run, project guidance, local agent execution, doctor checks |
```

- [ ] **Step 2: Update install guide language**

Replace language that says marketplace plugins bundle Node MCP with language
that says marketplace plugins bundle a Rust-built self-contained local Lite MCP
executable.

- [ ] **Step 3: Update workflow skill runtime notes**

In `content/workflow/SKILL.md`, replace Node-specific Desktop guidance with:

```markdown
- Marketplace Lite plugin installs include a Rust-built self-contained local Literature Provider MCP runtime. They do not require user-installed Node, Python, npm, pip, Cargo, Rust, or the full Qiongli CLI for provider search tools.
- Full local orchestration remains a Python Full CLI capability. Use `qiongli mcp serve --transport stdio` when `qiongli_task_run` or local agent execution is required.
- Zotero local-library search and direct writes require Zotero Desktop plus the Qiongli Zotero Companion. Without the Companion, use the Lite MCP import-file export tools.
```

- [ ] **Step 4: Run docs checks**

Run:

```bash
.venv/bin/python -m unittest tests.test_literature_contract tests.test_plugin_distribution_contract -v
git diff --check
```

Expected: pass.

- [ ] **Step 5: Commit**

```bash
git add docs/advanced/cross-platform-mcp.md docs/advanced/plugin-first-architecture.md docs/guide/install.md README.md README_CN.md content/workflow/SKILL.md
git commit -m "docs(mcp): explain rust marketplace lite runtime"
```

## Task 16: Full Regression And Release Boundary Review

**Files:**
- Test only
- Review: all files changed by Tasks 1-15

- [ ] **Step 1: Run Rust tests**

```bash
cd packages/qiongli-lite-mcp && cargo test
```

Expected: pass.

- [ ] **Step 2: Run Python MCP and packaging tests**

```bash
.venv/bin/python -m unittest \
  tests.test_mcp_contract_fixtures \
  tests.test_lite_mcp_contract \
  tests.test_lite_mcp_binary_artifacts \
  tests.test_lite_full_mcp_parity \
  tests.test_plugin_artifacts \
  tests.test_literature_mcpb_artifact \
  tests.test_plugin_distribution_contract \
  tests.test_mcp_client_config \
  tests.test_mcp_literature_tools \
  -v
```

Expected: pass.

- [ ] **Step 3: Run platform target validation**

```bash
.venv/bin/python tooling/scripts/validate_platform_targets.py
```

Expected: no validation errors.

- [ ] **Step 4: Run marketplace validator**

```bash
.venv/bin/python tooling/scripts/validate_marketplace_install.py --skip-client-activation
```

Expected: no validation errors and marketplace binary mode reported for Codex,
Claude Code, and Claude Desktop direct plugin artifacts.

- [ ] **Step 5: Run secret and boundary scan**

```bash
rg -n "(/[U]sers/|/[p]rivate/|BEGI[N] (RSA|OPENSSH|EC|DSA) PRIVATE KEY|secre[t]:|toke[n]:|passwor[d]:|OPENALEX_API_KEY=.*|S2_API_KEY=.*)" \
  content/mcp-contracts \
  packages/qiongli-lite-mcp \
  tooling/scripts/build_lite_mcp.py \
  tooling/scripts/build_plugin_artifacts.py \
  packages/qiongli-literature-mcpb \
  docs/advanced/cross-platform-mcp.md \
  README.md \
  README_CN.md
```

Expected: no matches.

- [ ] **Step 6: Run diff hygiene**

```bash
git diff --check
```

Expected: no whitespace errors.

- [ ] **Step 7: Commit final integration adjustments**

```bash
git add content/mcp-contracts packages/qiongli-lite-mcp tooling/scripts scripts tests docs README.md README_CN.md content
git commit -m "chore(lite-mcp): verify rust marketplace runtime"
```

## Execution Notes

- Rust and Cargo are maintainer/development dependencies only. End users of
  Marketplace Lite must not need Rust, Cargo, Node, or Python.
- If a marketplace cannot select platform-specific artifacts, publish separate
  platform-specific plugin artifacts or keep that marketplace on the legacy Node
  provider until platform selection is available.
- Keep Full CLI tests in every regression run. A passing Rust Lite MCP build is
  not a substitute for Python full-runtime verification.
- Do not start a Full CLI Rust migration from this plan. That requires a
  separate spec because it would touch orchestrator, installer, project
  guidance, local agent execution, and release automation.

## Self-Review

- Spec coverage: the plan covers Rust self-contained runtime, marketplace
  packaging, MCPB migration, provider config parity, Zotero boundaries, preview
  orchestrator tools, Python Full CLI preservation, docs, and release
  validation.
- Plan scan: tasks name concrete files, commands, expected failures, expected
  pass states, and commit boundaries.
- Boundary check: changes stay in Qiongli source, generated release scripts,
  contracts, and docs. No marketplace catalog repository or local machine path
  is copied into this repository.
