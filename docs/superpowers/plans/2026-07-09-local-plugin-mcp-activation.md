# Local Plugin MCP Activation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add local startup validation for plugin-bundled Qiongli Lite MCP servers.

**Architecture:** Keep existing Codex and Claude plugin packaging, then validate the same MCP declarations clients consume by launching the bundled Rust executable and checking `tools/list`. This turns release validation from archive-only inspection into local server activation evidence.

**Tech Stack:** Python `unittest`, `subprocess`, release validator scripts, Rust Lite MCP binary built by existing artifact materialization.

---

### Task 1: Add Failing Activation Tests

**Files:**
- Modify: `tests/test_plugin_distribution_contract.py`

- [x] Add Codex plugin startup test that materializes the plugin and calls `validator._assert_plugin_mcp_server_launches(...)`.
- [x] Add Claude plugin startup test with the same helper and `platform="claude"`.
- [x] Run the two tests and verify they fail because the helper does not exist.

### Task 2: Implement MCP Startup Validator

**Files:**
- Modify: `tooling/scripts/validate_marketplace_install.py`

- [x] Add MCP config loading for Codex `.mcp.json` and Claude inline or plugin-root `.mcp.json`.
- [x] Resolve plugin-local variables such as `${CLAUDE_PLUGIN_ROOT}`.
- [x] Start the declared stdio command with `subprocess.run`.
- [x] Send `initialize` and `tools/list`.
- [x] Require `qiongli_literature_status`, `qiongli_literature_search`, and `qiongli_task_plan`.

### Task 3: Wire Release Validation

**Files:**
- Modify: `tooling/scripts/validate_marketplace_install.py`
- Modify: `tests/test_plugin_distribution_contract.py`

- [x] Call the startup validator for Codex and Claude marketplace artifacts.
- [x] Call it for Claude Desktop direct plugin artifacts.
- [x] Update validator output expectations to include `MCP startup checked`.

### Task 4: Verify

**Files:**
- Verify: `tests/test_plugin_distribution_contract.py`
- Verify: `tooling/scripts/validate_marketplace_install.py`

- [x] Run the focused startup tests.
- [x] Run marketplace artifact validation tests.
- [x] Run `scripts/validate_marketplace_install.py --dist-dir <tmp>`.
- [x] Check git status for intended changes only.
