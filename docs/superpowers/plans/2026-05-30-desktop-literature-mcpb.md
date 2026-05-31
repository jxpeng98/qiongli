# Desktop Literature MCPB Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Claude Desktop local `.mcpb` extension that lets Desktop users configure literature provider credentials in Desktop and run Qiongli external academic literature search without installing the Qiongli CLI.

**Architecture:** PR #7 becomes MCPB-only for Desktop provider access. The MCPB is a self-contained Node.js stdio MCP server with a `manifest.json` `user_config` section for OpenAlex email and Semantic Scholar API key. It does not depend on `qiongli` CLI, does not write `~/.config/qiongli/providers.json`, and does not store secrets in the Desktop skill ZIP.

**Tech Stack:** Node.js ESM, `@modelcontextprotocol/sdk`, Node built-in `fetch`, `node:test`, Python stdlib `zipfile` for local artifact validation, Claude MCPB manifest version `0.3`.

---

## Scope Boundary

This plan intentionally does not implement CLI onboarding. The CLI setup wizard belongs in a separate branch and plan.

PR #7 should expose these Desktop MCP tools:

- `qiongli_literature_status`
- `qiongli_literature_search`
- `qiongli_literature_export_evidence`

First-version provider support:

- OpenAlex: real search, configured with optional email.
- Semantic Scholar: real search, configured with optional API key.
- Crossref and PubMed: status schema only, always reported as `missing` or `not_implemented` until later PRs.

Quality constraints:

- Never leak configured secrets in tool output, logs, tests, README examples, or evidence snapshots.
- Never fabricate missing metadata; use `null`, empty arrays, and warning fields.
- Return `provider_connected` only when at least one implemented academic provider is usable.
- Return warnings when only one provider succeeds.
- Return `strategy_only` when no implemented provider is configured or available.

## File Structure

- Create: `packages/qiongli-literature-mcpb/package.json`
  - Owns Node package metadata and MCP SDK dependency for the MCPB package only.
- Create: `packages/qiongli-literature-mcpb/manifest.json`
  - Owns Claude Desktop extension metadata, `user_config`, MCP server launch config, tools list, compatibility.
- Create: `packages/qiongli-literature-mcpb/server/index.mjs`
  - Owns MCP server registration and dispatch only.
- Create: `packages/qiongli-literature-mcpb/server/config.mjs`
  - Reads env values injected by MCPB user config and returns redacted provider status.
- Create: `packages/qiongli-literature-mcpb/server/providers/openalex.mjs`
  - Owns OpenAlex API URL construction and response parsing.
- Create: `packages/qiongli-literature-mcpb/server/providers/semantic-scholar.mjs`
  - Owns Semantic Scholar API URL construction, request headers, and response parsing.
- Create: `packages/qiongli-literature-mcpb/server/normalize.mjs`
  - Owns common result shape and dedupe keys.
- Create: `packages/qiongli-literature-mcpb/server/evidence.mjs`
  - Owns capability mode, provider coverage, and warning calculation.
- Create: `packages/qiongli-literature-mcpb/test/*.test.mjs`
  - Owns unit coverage for config redaction, providers, evidence, and MCP tools.
- Create: `scripts/build_literature_mcpb.py`
  - Builds `dist/qiongli-literature-provider-<version>.mcpb` and validates required files.
- Create: `tests/test_literature_mcpb_artifact.py`
  - Validates manifest, bundled server files, and no accidental secret fixture text.
- Modify: `README.md`, `README_CN.md`, `docs/guide/install.md`, `docs/zh/guide/install.md`
  - Document Desktop MCPB install separately from Desktop skill ZIP.
- Modify: `qiongli-workflow/SKILL.md`, `plugins/qiongli/skills/qiongli-workflow/SKILL.md`, `qiongli/subject_materializer.py`, `scripts/build_plugin_artifacts.py`
  - Keep Desktop skill text aligned with MCPB provider boundary.
- Modify: `tests/test_mcp_provider_docs.py`, `tests/test_claude_desktop_skill_artifact.py`
  - Lock docs and Desktop ZIP text.
- Modify: `qiongli/cli.py`, `tests/test_cli.py`
  - Remove current PR #7 `qiongli companion` CLI command and tests because Desktop MCPB no longer depends on CLI companion.

### Task 1: Reset PR #7 Scope To MCPB-Only

**Files:**
- Modify: `qiongli/cli.py`
- Modify: `tests/test_cli.py`
- Modify: `README.md`, `README_CN.md`, `docs/guide/install.md`, `docs/zh/guide/install.md`

- [ ] Remove `cmd_companion()`, `_companion_status_payload()`, `_cmd_companion_setup()`, and the `companion` argparse subcommand from `qiongli/cli.py`.

- [ ] Remove these companion-specific tests from `tests/test_cli.py`:

```python
def test_companion_doctor_json_reports_strategy_only_without_config(self) -> None:
    ...

def test_companion_setup_and_export_status_json_do_not_leak_secret(self) -> None:
    ...
```

- [ ] Replace docs text that tells Desktop users to run `qiongli companion setup` with MCPB-specific wording:

```markdown
Desktop users who need external provider search should install the Qiongli Literature Provider MCPB. The Desktop skill ZIP remains skill-only; it does not store keys or execute provider calls.
```

- [ ] Run the CLI and docs tests to verify the old companion command is gone and provider config tests still pass:

```bash
uv run --frozen --with pytest pytest tests/test_cli.py tests/test_mcp_provider_docs.py -q
```

Expected: existing provider tests pass; no test references `qiongli companion`.

- [ ] Commit:

```bash
git add qiongli/cli.py tests/test_cli.py README.md README_CN.md docs/guide/install.md docs/zh/guide/install.md
git commit -m "refactor: scope desktop companion to mcpb"
```

### Task 2: Add MCPB Package Skeleton And Manifest

**Files:**
- Create: `packages/qiongli-literature-mcpb/package.json`
- Create: `packages/qiongli-literature-mcpb/manifest.json`
- Create: `packages/qiongli-literature-mcpb/server/index.mjs`
- Create: `packages/qiongli-literature-mcpb/README.md`
- Test: `tests/test_literature_mcpb_artifact.py`

- [ ] Write a failing manifest test:

```python
def test_literature_mcpb_manifest_declares_sensitive_config() -> None:
    manifest = json.loads((REPO_ROOT / "packages/qiongli-literature-mcpb/manifest.json").read_text())
    assert manifest["manifest_version"] == "0.3"
    assert manifest["server"]["type"] == "node"
    assert manifest["server"]["entry_point"] == "server/index.mjs"
    assert manifest["user_config"]["semantic_scholar_api_key"]["sensitive"] is True
    assert manifest["user_config"]["openalex_email"]["type"] == "string"
    assert "qiongli_literature_search" in {tool["name"] for tool in manifest["tools"]}
```

- [ ] Run the failing test:

```bash
uv run --frozen --with pytest pytest tests/test_literature_mcpb_artifact.py::test_literature_mcpb_manifest_declares_sensitive_config -q
```

Expected: FAIL because `manifest.json` does not exist.

- [ ] Create `packages/qiongli-literature-mcpb/package.json`:

```json
{
  "name": "qiongli-literature-provider-mcpb",
  "version": "0.1.0",
  "private": true,
  "type": "module",
  "description": "Claude Desktop MCPB for Qiongli academic literature provider search.",
  "scripts": {
    "test": "node --test test/*.test.mjs",
    "start": "node server/index.mjs"
  },
  "dependencies": {
    "@modelcontextprotocol/sdk": "^1.0.0"
  },
  "devDependencies": {}
}
```

- [ ] Create `manifest.json` using MCPB manifest version `0.3`:

```json
{
  "manifest_version": "0.3",
  "name": "qiongli-literature-provider",
  "display_name": "Qiongli Literature Provider",
  "version": "0.1.0",
  "description": "Local Claude Desktop MCP server for academic literature search through OpenAlex and Semantic Scholar.",
  "author": {
    "name": "Qiongli"
  },
  "server": {
    "type": "node",
    "entry_point": "server/index.mjs",
    "mcp_config": {
      "command": "node",
      "args": ["${__dirname}/server/index.mjs"],
      "env": {
        "QIONGLI_MCPB_OPENALEX_EMAIL": "${user_config.openalex_email}",
        "QIONGLI_MCPB_SEMANTIC_SCHOLAR_API_KEY": "${user_config.semantic_scholar_api_key}",
        "QIONGLI_MCPB_DEFAULT_LIMIT": "${user_config.default_result_limit}"
      }
    }
  },
  "tools": [
    {
      "name": "qiongli_literature_status",
      "description": "Report configured literature providers and capability mode without exposing secrets."
    },
    {
      "name": "qiongli_literature_search",
      "description": "Search academic literature using configured OpenAlex and Semantic Scholar providers."
    },
    {
      "name": "qiongli_literature_export_evidence",
      "description": "Export an auditable provider capability and search evidence snapshot."
    }
  ],
  "compatibility": {
    "claude_desktop": ">=1.0.0",
    "platforms": ["darwin", "win32"],
    "runtimes": {
      "node": ">=18.0.0"
    }
  },
  "user_config": {
    "openalex_email": {
      "type": "string",
      "title": "OpenAlex email",
      "description": "Email address included in OpenAlex requests for polite pool access.",
      "required": false
    },
    "semantic_scholar_api_key": {
      "type": "string",
      "title": "Semantic Scholar API key",
      "description": "Semantic Scholar API key. Leave blank to use only unauthenticated provider paths.",
      "sensitive": true,
      "required": false
    },
    "default_result_limit": {
      "type": "number",
      "title": "Default result limit",
      "description": "Maximum results per provider when a tool call does not specify a limit.",
      "default": 10,
      "min": 1,
      "max": 50,
      "required": false
    }
  },
  "keywords": ["academic", "literature", "openalex", "semantic-scholar", "qiongli"],
  "license": "MIT",
  "privacy_policies": [
    "https://openalex.org/privacy",
    "https://www.semanticscholar.org/product/api/license"
  ]
}
```

- [ ] Create a minimal `server/index.mjs` that starts and responds to `tools/list`; full tools come later.

- [ ] Run:

```bash
uv run --frozen --with pytest pytest tests/test_literature_mcpb_artifact.py -q
```

Expected: PASS for manifest structure tests.

- [ ] Commit:

```bash
git add packages/qiongli-literature-mcpb tests/test_literature_mcpb_artifact.py
git commit -m "feat: add literature mcpb package skeleton"
```

### Task 3: Add Config And Evidence Core

**Files:**
- Create: `packages/qiongli-literature-mcpb/server/config.mjs`
- Create: `packages/qiongli-literature-mcpb/server/evidence.mjs`
- Test: `packages/qiongli-literature-mcpb/test/config.test.mjs`
- Test: `packages/qiongli-literature-mcpb/test/evidence.test.mjs`

- [ ] Write tests that prove secrets are redacted:

```javascript
import test from "node:test";
import assert from "node:assert/strict";
import { readConfig, providerStatus } from "../server/config.mjs";

test("provider status redacts configured secrets", () => {
  const config = readConfig({
    QIONGLI_MCPB_OPENALEX_EMAIL: "person@example.com",
    QIONGLI_MCPB_SEMANTIC_SCHOLAR_API_KEY: "secret-key",
    QIONGLI_MCPB_DEFAULT_LIMIT: "12"
  });
  const status = providerStatus(config);
  assert.equal(status.providers.openalex, "configured");
  assert.equal(status.providers.semantic_scholar, "configured");
  assert.equal(JSON.stringify(status).includes("secret-key"), false);
  assert.equal(JSON.stringify(status).includes("person@example.com"), false);
});
```

- [ ] Write tests for capability warnings:

```javascript
import test from "node:test";
import assert from "node:assert/strict";
import { buildEvidence } from "../server/evidence.mjs";

test("single successful provider produces warning", () => {
  const evidence = buildEvidence({
    attemptedProviders: ["openalex", "semantic_scholar"],
    successfulProviders: ["openalex"],
    failedProviders: ["semantic_scholar"],
    resultCount: 3
  });
  assert.equal(evidence.capability_mode, "provider_connected");
  assert.deepEqual(evidence.warnings, ["single_successful_provider", "partial_provider_failure"]);
});
```

- [ ] Run tests and verify they fail:

```bash
npm --prefix packages/qiongli-literature-mcpb test
```

Expected: FAIL because modules do not exist.

- [ ] Implement `readConfig(env)` and `providerStatus(config)`:

```javascript
export function readConfig(env = process.env) {
  const defaultLimit = Number.parseInt(env.QIONGLI_MCPB_DEFAULT_LIMIT || "10", 10);
  return {
    openalexEmail: String(env.QIONGLI_MCPB_OPENALEX_EMAIL || "").trim(),
    semanticScholarApiKey: String(env.QIONGLI_MCPB_SEMANTIC_SCHOLAR_API_KEY || "").trim(),
    defaultLimit: Number.isFinite(defaultLimit) ? Math.min(Math.max(defaultLimit, 1), 50) : 10
  };
}

export function providerStatus(config) {
  const providers = {
    openalex: config.openalexEmail ? "configured" : "configured_without_email",
    semantic_scholar: config.semanticScholarApiKey ? "configured" : "missing",
    crossref: "not_implemented",
    pubmed: "not_implemented"
  };
  const implementedConnected = providers.openalex.startsWith("configured") || providers.semantic_scholar === "configured";
  return {
    status: "ok",
    capability_mode: implementedConnected ? "provider_connected" : "strategy_only",
    providers
  };
}
```

- [ ] Implement `buildEvidence(input)` with `single_successful_provider`, `partial_provider_failure`, and `all_providers_failed` warnings.

- [ ] Run:

```bash
npm --prefix packages/qiongli-literature-mcpb test
```

Expected: PASS for config and evidence tests.

- [ ] Commit:

```bash
git add packages/qiongli-literature-mcpb/server/config.mjs packages/qiongli-literature-mcpb/server/evidence.mjs packages/qiongli-literature-mcpb/test
git commit -m "feat: add literature mcpb config and evidence core"
```

### Task 4: Implement Provider Clients

**Files:**
- Create: `packages/qiongli-literature-mcpb/server/providers/openalex.mjs`
- Create: `packages/qiongli-literature-mcpb/server/providers/semantic-scholar.mjs`
- Create: `packages/qiongli-literature-mcpb/server/normalize.mjs`
- Test: `packages/qiongli-literature-mcpb/test/providers.test.mjs`

- [ ] Write provider tests using injected `fetch` functions:

```javascript
import test from "node:test";
import assert from "node:assert/strict";
import { searchOpenAlex } from "../server/providers/openalex.mjs";
import { searchSemanticScholar } from "../server/providers/semantic-scholar.mjs";

test("OpenAlex parser normalizes academic records", async () => {
  const fakeFetch = async (url) => ({
    ok: true,
    json: async () => ({
      results: [{
        id: "https://openalex.org/W123",
        doi: "https://doi.org/10.1000/example",
        title: "Example paper",
        publication_year: 2024,
        authorships: [{ author: { display_name: "A. Author" } }],
        primary_location: { source: { display_name: "Journal" }, landing_page_url: "https://example.test" },
        abstract_inverted_index: { Example: [0], abstract: [1] }
      }]
    })
  });
  const result = await searchOpenAlex({ query: "example", limit: 1, email: "person@example.com", fetchImpl: fakeFetch });
  assert.equal(result.results[0].provider, "openalex");
  assert.equal(result.results[0].title, "Example paper");
  assert.equal(result.results[0].doi, "10.1000/example");
});

test("Semantic Scholar sends API key when configured", async () => {
  let capturedHeaders = {};
  const fakeFetch = async (_url, options) => {
    capturedHeaders = options.headers;
    return {
      ok: true,
      json: async () => ({ data: [{ paperId: "S2", title: "S2 paper", year: 2023, authors: [] }] })
    };
  };
  await searchSemanticScholar({ query: "example", limit: 1, apiKey: "secret", fetchImpl: fakeFetch });
  assert.equal(capturedHeaders["x-api-key"], "secret");
});
```

- [ ] Run:

```bash
npm --prefix packages/qiongli-literature-mcpb test
```

Expected: FAIL because provider modules do not exist.

- [ ] Implement provider clients with:
  - `AbortSignal.timeout(15000)` for real network calls.
  - URL encoding with `URL` and `URLSearchParams`.
  - No thrown raw API key values in errors.
  - `null` for missing DOI, abstract, URL, venue, and year.

- [ ] Implement `normalizeResult(raw)` and `dedupeResults(results)` in `normalize.mjs`.

- [ ] Run:

```bash
npm --prefix packages/qiongli-literature-mcpb test
```

Expected: PASS.

- [ ] Commit:

```bash
git add packages/qiongli-literature-mcpb/server/providers packages/qiongli-literature-mcpb/server/normalize.mjs packages/qiongli-literature-mcpb/test/providers.test.mjs
git commit -m "feat: add mcpb literature provider clients"
```

### Task 5: Implement MCP Tool Server

**Files:**
- Modify: `packages/qiongli-literature-mcpb/server/index.mjs`
- Test: `packages/qiongli-literature-mcpb/test/tools.test.mjs`

- [ ] Write tool handler tests that call exported pure handlers without spawning stdio:

```javascript
import test from "node:test";
import assert from "node:assert/strict";
import { handleStatus, handleSearch, handleExportEvidence } from "../server/index.mjs";

test("status tool returns redacted provider mode", async () => {
  const payload = await handleStatus({ env: { QIONGLI_MCPB_SEMANTIC_SCHOLAR_API_KEY: "secret" } });
  assert.equal(payload.capability_mode, "provider_connected");
  assert.equal(JSON.stringify(payload).includes("secret"), false);
});

test("search tool rejects blank query", async () => {
  await assert.rejects(
    () => handleSearch({ query: "   " }, { env: {}, fetchImpl: async () => ({ ok: true, json: async () => ({}) }) }),
    /query is required/
  );
});
```

- [ ] Run tests and verify they fail:

```bash
npm --prefix packages/qiongli-literature-mcpb test
```

Expected: FAIL because handlers are missing.

- [ ] Implement:
  - `handleStatus(context)`
  - `handleSearch(input, context)`
  - `handleExportEvidence(input, context)`
  - MCP SDK stdio server registration.

- [ ] Use this input schema for `qiongli_literature_search`:

```json
{
  "type": "object",
  "properties": {
    "query": { "type": "string" },
    "limit": { "type": "number", "minimum": 1, "maximum": 50 },
    "from_year": { "type": "number" },
    "to_year": { "type": "number" }
  },
  "required": ["query"]
}
```

- [ ] Ensure tool output has this top-level shape:

```json
{
  "status": "ok",
  "capability_mode": "provider_connected",
  "providers": {
    "attempted": ["openalex", "semantic_scholar"],
    "successful": ["openalex"],
    "failed": ["semantic_scholar"]
  },
  "warnings": ["single_successful_provider"],
  "results": []
}
```

- [ ] Run:

```bash
npm --prefix packages/qiongli-literature-mcpb test
```

Expected: PASS.

- [ ] Commit:

```bash
git add packages/qiongli-literature-mcpb/server/index.mjs packages/qiongli-literature-mcpb/test/tools.test.mjs
git commit -m "feat: expose literature mcp tools"
```

### Task 6: Build And Validate `.mcpb` Artifact

**Files:**
- Create: `scripts/build_literature_mcpb.py`
- Modify: `tests/test_literature_mcpb_artifact.py`
- Modify: `package.json`

- [ ] Add a Python artifact test:

```python
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
    with zipfile.ZipFile(artifact) as zf:
        names = set(zf.namelist())
    assert "manifest.json" in names
    assert "server/index.mjs" in names
    assert "package.json" in names
```

- [ ] Run and verify failure:

```bash
uv run --frozen --with pytest pytest tests/test_literature_mcpb_artifact.py -q
```

Expected: FAIL because build script is missing.

- [ ] Implement `scripts/build_literature_mcpb.py` using only `argparse`, `json`, `shutil`, and `zipfile`. It must:
  - read `packages/qiongli-literature-mcpb/manifest.json`
  - create `dist/qiongli-literature-provider-<version>.mcpb`
  - include `manifest.json`, `package.json`, `server/**`, `README.md`
  - include `node_modules/**` only when present
  - fail if `semantic_scholar_api_key` appears outside `manifest.json` or tests

- [ ] Add root npm script:

```json
"mcpb:pack": "python3 scripts/build_literature_mcpb.py --dist-dir dist"
```

- [ ] Run:

```bash
uv run --frozen --with pytest pytest tests/test_literature_mcpb_artifact.py -q
```

Expected: PASS.

- [ ] Run:

```bash
npm --prefix packages/qiongli-literature-mcpb test
```

Expected: PASS.

- [ ] Commit:

```bash
git add scripts/build_literature_mcpb.py tests/test_literature_mcpb_artifact.py package.json
git commit -m "build: package literature mcpb artifact"
```

### Task 7: Update Desktop Documentation And Skill Boundaries

**Files:**
- Modify: `README.md`, `README_CN.md`
- Modify: `docs/guide/install.md`, `docs/zh/guide/install.md`
- Modify: `qiongli-workflow/SKILL.md`
- Modify: `plugins/qiongli/skills/qiongli-workflow/SKILL.md`
- Modify: `qiongli/subject_materializer.py`
- Modify: `scripts/build_plugin_artifacts.py`
- Test: `tests/test_mcp_provider_docs.py`
- Test: `tests/test_claude_desktop_skill_artifact.py`

- [ ] Update tests to require:

```python
for expected in (
    "qiongli-literature-provider",
    ".mcpb",
    "OpenAlex",
    "Semantic Scholar",
    "provider_connected",
    "strategy_only",
):
    self.assertIn(expected, content)
self.assertNotIn("qiongli companion setup", content)
```

- [ ] Run:

```bash
uv run --frozen --with pytest pytest tests/test_mcp_provider_docs.py tests/test_claude_desktop_skill_artifact.py -q
```

Expected: FAIL until docs are updated.

- [ ] Update docs to describe two separate Desktop assets:
  - Desktop skill ZIP: workflows, prompts, templates, no secrets.
  - Literature MCPB: local provider search, Desktop config UI, sensitive key handling.

- [ ] Update skill text:

```markdown
Claude Desktop/Web focused ZIPs remain skill-only packages. Claude Desktop users who need external provider search should install the Qiongli Literature Provider `.mcpb` local extension and configure provider keys in the extension settings. If no MCPB or platform-native search is available, record the workflow as `strategy_only`.
```

- [ ] Run:

```bash
uv run --frozen --with pytest pytest tests/test_mcp_provider_docs.py tests/test_claude_desktop_skill_artifact.py -q
```

Expected: PASS.

- [ ] Verify mirror alignment:

```bash
diff -u qiongli-workflow/SKILL.md plugins/qiongli/skills/qiongli-workflow/SKILL.md
```

Expected: no output and exit code 0.

- [ ] Commit:

```bash
git add README.md README_CN.md docs/guide/install.md docs/zh/guide/install.md qiongli-workflow/SKILL.md plugins/qiongli/skills/qiongli-workflow/SKILL.md qiongli/subject_materializer.py scripts/build_plugin_artifacts.py tests/test_mcp_provider_docs.py tests/test_claude_desktop_skill_artifact.py
git commit -m "docs: document desktop literature mcpb"
```

### Task 8: Final Verification For PR #7

**Files:**
- Verify only.

- [ ] Run Node tests:

```bash
npm --prefix packages/qiongli-literature-mcpb test
```

Expected: all tests pass.

- [ ] Run Python tests:

```bash
uv run --frozen --with pytest pytest tests/test_literature_mcpb_artifact.py tests/test_mcp_provider_docs.py tests/test_claude_desktop_skill_artifact.py tests/test_cli.py -q
```

Expected: all tests pass.

- [ ] Run artifact build:

```bash
python3 scripts/build_literature_mcpb.py --dist-dir dist
```

Expected: `dist/qiongli-literature-provider-0.1.0.mcpb` exists.

- [ ] Check git state:

```bash
git status --short --branch
```

Expected: clean branch, except ignored `dist/` output if not tracked.

- [ ] Push PR #7:

```bash
git push --force-with-lease origin feat/provider-companion
```

- [ ] Keep PR #7 as draft until a manual Claude Desktop install test has been run with the generated `.mcpb`.

## Self-Review

- Spec coverage: Desktop local provider access without CLI is covered by MCPB manifest, Node server, provider clients, packaging, and docs.
- Placeholder scan: no `TBD` or future-only implementation steps remain in first-version scope.
- Scope check: CLI onboarding is excluded and handled by a separate plan.
