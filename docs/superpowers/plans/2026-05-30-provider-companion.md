# Provider Companion Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a minimal local provider companion so Desktop users can configure literature provider keys outside the skill ZIP and export an auditable status snapshot.

**Architecture:** The companion reuses the existing provider config resolver and writes the same global `providers.json` as the CLI/provider commands. Desktop ZIPs remain skill-only and read no secrets; they only instruct users to run companion commands and record `provider_connected` or `strategy_only`.

**Tech Stack:** Python CLI via `qiongli/cli.py`, provider config helpers in `bridges/provider_config.py`, pytest/unittest coverage, Markdown install docs, Desktop ZIP materializer tests.

---

### Task 1: Companion CLI

**Files:**
- Modify: `qiongli/cli.py`
- Test: `tests/test_cli.py`

- [x] Add failing CLI tests for `qiongli companion doctor --json` and `qiongli companion export-status --json`.
- [x] Verify the tests fail because the `companion` command is missing.
- [x] Add `cmd_companion()`, `_companion_status_payload()`, and `_cmd_companion_setup()`.
- [x] Register `companion setup`, `companion doctor --json`, and `companion export-status --json`.
- [x] Verify companion JSON redacts provider values and returns `status`, `config_path`, `providers`, and `capability_mode`.
- [x] Commit with `feat: add provider companion cli`.

### Task 2: Desktop Companion Documentation

**Files:**
- Modify: `README.md`, `README_CN.md`
- Modify: `docs/guide/install.md`, `docs/zh/guide/install.md`
- Modify: `qiongli-workflow/SKILL.md`
- Modify: `qiongli/subject_materializer.py`
- Modify: `scripts/build_plugin_artifacts.py`
- Test: `tests/test_mcp_provider_docs.py`, `tests/test_claude_desktop_skill_artifact.py`

- [x] Add failing docs tests requiring `qiongli companion setup`, `qiongli companion doctor --json`, and `qiongli companion export-status --json`.
- [x] Add install docs explaining the Desktop companion path.
- [x] Add Desktop ZIP skill text explaining the companion commands.
- [x] Verify Desktop ZIP output still stays within the 180-file budget.
- [x] Commit with `docs: document provider companion setup`.

### Task 3: Verification

**Files:**
- Verify all touched companion, provider, Desktop, and search tests.

- [ ] Run:
  ```bash
  uv run --with pytest pytest tests/test_provider_config.py tests/test_cli.py tests/test_s2_client.py tests/test_literature_search.py tests/test_literature_search_quality_audit.py tests/test_mcp_connectors.py tests/test_mcp_provider_docs.py tests/test_claude_desktop_skill_artifact.py tests/test_orchestrator_workflows.py -q
  ```
- [ ] Restore `uv.lock` if test execution updates it without semantic dependency changes.
- [ ] Confirm `git status --short --branch` is clean after commits.
