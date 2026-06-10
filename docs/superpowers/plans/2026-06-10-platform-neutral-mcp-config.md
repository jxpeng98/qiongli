# Platform-Neutral MCP Provider Configuration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make Qiongli provider credential setup a platform-neutral MCP capability for Codex, Claude, Claude Desktop MCPB, and other local stdio MCP clients.

**Architecture:** The bundled Node MCP server exposes `qiongli_configure_provider` as the primary semantic setup tool and keeps `qiongli_open_config_wizard` as a compatibility alias. Status tools return redacted provider state plus a `next_action` object so clients can guide users to the local setup flow without putting API keys in chat.

**Tech Stack:** Node.js ESM MCPB runtime, Python docs/artifact tests, shared `~/.config/qiongli/providers.json` provider config.

---

### Task 1: Add platform-neutral tool contract tests

**Files:**
- Modify: `packages/qiongli-literature-mcpb/test/tools.test.mjs`

- [ ] **Step 1: Write failing tests**

Add tests asserting:
- `TOOL_DECLARATIONS` includes `qiongli_configure_provider`.
- `handleConfigStatus` returns `next_action.tool === "qiongli_configure_provider"` when `semantic_scholar.api_key` is missing.
- `handleSaveProviderConfig` saving `semantic-scholar/api-key` returns a warning and does not echo the raw key.

- [ ] **Step 2: Run test to verify failure**

Run: `npm --prefix packages/qiongli-literature-mcpb test`
Expected: FAIL because `qiongli_configure_provider`, `next_action`, and warning behavior are missing.

### Task 2: Implement platform-neutral setup flow

**Files:**
- Modify: `packages/qiongli-literature-mcpb/server/index.mjs`
- Modify: `packages/qiongli-literature-mcpb/server/config-wizard.mjs`
- Modify: `packages/qiongli-literature-mcpb/manifest.json`
- Modify: `packages/qiongli-literature-mcpb/package.json`

- [ ] **Step 1: Implement minimal code**

Add:
- `qiongli_configure_provider` tool declaration with optional `provider`, `host`, and `port`.
- Handler that calls the same local wizard as `qiongli_open_config_wizard`.
- `next_action` status payloads that point to `qiongli_configure_provider`.
- Save warning for API-key fields while preserving backward-compatible scripted writes.
- Version bump to `0.1.4`.

- [ ] **Step 2: Run test to verify pass**

Run: `npm --prefix packages/qiongli-literature-mcpb test`
Expected: PASS.

### Task 3: Sync bundled plugin runtimes

**Files:**
- Modify: `packages/qiongli-plugin/mcp/qiongli-literature-provider/index.mjs`
- Modify: `packages/qiongli-plugin/mcp/qiongli-literature-provider/config-wizard.mjs`
- Modify: `packages/qiongli-next-plugin/mcp/qiongli-literature-provider/index.mjs`
- Modify: `packages/qiongli-next-plugin/mcp/qiongli-literature-provider/config-wizard.mjs`

- [ ] **Step 1: Copy the MCPB runtime files into both bundled plugin runtimes**

Run:
`cp packages/qiongli-literature-mcpb/server/index.mjs packages/qiongli-plugin/mcp/qiongli-literature-provider/index.mjs`
`cp packages/qiongli-literature-mcpb/server/config-wizard.mjs packages/qiongli-plugin/mcp/qiongli-literature-provider/config-wizard.mjs`
`cp packages/qiongli-literature-mcpb/server/index.mjs packages/qiongli-next-plugin/mcp/qiongli-literature-provider/index.mjs`
`cp packages/qiongli-literature-mcpb/server/config-wizard.mjs packages/qiongli-next-plugin/mcp/qiongli-literature-provider/config-wizard.mjs`

- [ ] **Step 2: Run artifact tests**

Run:
`python3 -m unittest tests.test_literature_mcpb_artifact tests.test_release_downloads tests.test_mcp_provider_docs tests.test_claude_desktop_skill_artifact -v`
Expected: PASS after docs and artifact expectations are updated.

### Task 4: Update docs and generated guidance

**Files:**
- Modify: `README.md`
- Modify: `README_CN.md`
- Modify: `docs/advanced/cross-platform-mcp.md`
- Modify: `docs/guide/install.md`
- Modify: `docs/quickstart.md`
- Modify: `docs/zh/guide/install.md`
- Modify: `docs/zh/quickstart.md`
- Modify: `content/workflow/SKILL.md`
- Modify: `packages/qiongli-next-plugin/skills/qiongli-workflow/SKILL.md`
- Modify: `packages/python-qiongli/src/qiongli/subject_materializer.py`
- Modify: `tooling/scripts/build_plugin_artifacts.py`

- [ ] **Step 1: Update references**

Prefer `qiongli_configure_provider` in user-facing docs and keep `qiongli_open_config_wizard` as a compatibility alias.

- [ ] **Step 2: Run docs tests**

Run:
`python3 -m unittest tests.test_mcp_provider_docs tests.test_cli_setup_docs -v`
Expected: PASS.

### Task 5: Verify and commit

**Files:**
- All modified files in the platform-neutral MCP configuration scope.

- [ ] **Step 1: Run full targeted verification**

Run:
`npm --prefix packages/qiongli-literature-mcpb test`
`python3 -m unittest tests.test_literature_mcpb_artifact tests.test_release_downloads tests.test_mcp_provider_docs tests.test_claude_desktop_skill_artifact tests.test_cli_setup_docs -v`

- [ ] **Step 2: Create conventional commits**

Create logical commits:
- `docs(mcp): plan platform-neutral provider configuration`
- `fix(mcpb): support desktop MCPB stdio attach`
- `feat(mcp): add secure provider configuration flow`
- `docs(mcp): document platform-neutral provider setup`
