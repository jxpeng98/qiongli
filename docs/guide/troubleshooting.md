# 🚨 Troubleshooting & Unified Error Codes Guide

This document lists all standard `ERR-RS-*` error codes you might encounter while using the Qiongli Orchestrator CLI, along with their root causes and standard solutions.

> **Pro Tip:** If you encounter any unexpected behavior, your first step should always be to run the interactive doctor: 
> `python3 -m bridges.orchestrator doctor --cwd .`

---

## Client Skill Discovery

### Codex does not respond to `/qiongli`.

- **Cause**: Codex does not expose Qiongli as a custom slash command. Qiongli is a Codex skill, not a Codex slash command.
- **Fix**:
  - Restart Codex after installing or upgrading.
  - Run `/skills` and confirm that `qiongli` is listed.
  - Invoke the main skill with `$qiongli`, for example `$qiongli plan a literature review on ai-in-education`.
  - For plugin-first installs, you can also invoke generated workflow wrappers such as `$qiongli-lit-review ai-in-education`.
  - If `/skills` only shows `research-paper-workflow`, refresh the current package with `qiongli upgrade --target codex --overwrite`, restart Codex, and check again.
- **Note**: The install directory may still be named `qiongli-workflow`; that is expected. The user-visible skill name should be `qiongli`.

### Codex does not show `/lit-review`, `/academic-write`, or other workflow commands.

- **Cause**: CLI-managed Codex plugin installs bundle workflow wrappers under `commands/`, but current Codex plugin discovery does not show them as separate `/lit-review` slash entries. Codex uses skill entries instead, so Qiongli generates `qiongli-*` wrapper skills for plugin installs.
- **Fix**:
  - Use `$qiongli-lit-review <topic>` when the wrapper is visible in `/skills`.
  - Use `$qiongli plan a literature review on <topic>` or `$qiongli run lit-review on <topic>` for the main skill.
  - You can also write the natural request directly, for example "Use Qiongli to run a literature review on <topic>"; the skill entrypoint is expected to route it to Stage B / `lit-review`.
  - After upgrading to a build that includes Codex wrapper skills, reinstall the local Codex plugin with `qiongli install --target codex --surface plugin --overwrite` and restart Codex.
- **Note**: Codex still will not show `/lit-review` as a slash command. It should show `qiongli-lit-review` as a skill wrapper for plugin installs; skills-only installs can continue using `$qiongli run lit-review on <topic>`.

### Codex literature tasks say `strategy_only` even though Qiongli is installed.

- **Cause**: The active Codex session may not have loaded Qiongli MCP tools, or the workflow skipped the literature capability preflight.
- **Fix**:
  - Run `qiongli check --json` and confirm `installed.codex.plugin_mcp.installed` is true.
  - In the Codex session, ask Qiongli to call `qiongli_literature_status`; `capability_mode: provider_connected` means provider-backed literature workflow is available.
  - Ask the workflow to write `qiongli_search_plan` before search execution. If provider MCP is unavailable but Codex native search is available, use `search_execution_mode: native_only`; if both provider MCP and native search are available, use `search_execution_mode: hybrid_search`.
  - Keep `provider_capability_mode` separate from `search_execution_mode`: provider capability can be `strategy_only` while execution is still `native_only` or `hybrid_search`.
  - MCP servers must not call Codex or Claude native search directly. The active agent executes `native_search_queries`, and logs native search as `native:codex_web_search` or `native:claude_web_search`.
  - If `qiongli_literature_status` is not visible, restart Codex and check again.
  - If plugin MCP remains invisible after restart, install the explicit standalone fallback with `qiongli install --target codex --parts mcp`.

### Claude Code does not show `/paper` or `/lit-review`.

- **Cause**: The workflow command discovery files were not installed, the client was not restarted, or only the Codex-facing skill package was installed.
- **Fix**:
  - For global multi-client usage, run `qiongli upgrade --target all --overwrite`.
  - Restart Claude Code.
  - If you installed only the native plugin, confirm that the plugin is enabled in the client.

---

## Environment & Authentication (ENV)

These errors occur when the Orchestrator cannot locate a required CLI binary (like `claude` or `codex`) or the corresponding API keys in your environment.

### `[ERR-RS-ENV-001]` Missing or invalid API key.
- **Cause**: An active AI agent was requested, but its authentication key is missing from the environment.
- **Fix**: Check your `.env` file or export the keys directly on your terminal.
  - Claude: `export ANTHROPIC_API_KEY="sk-ant-..."`
  - Codex: `export OPENAI_API_KEY="sk-proj-..."`

### `[ERR-RS-ENV-002]` Required CLI tool is not installed or not in PATH.
- **Cause**: The Node.js binary wrappers for the models are not installed.
- **Fix**: Install the underlying CLI tools globally:
  - `npm install -g @anthropic-ai/claude-code`
  - Install the Codex CLI from the official OpenAI distribution, then ensure `codex` is on `PATH`.

### `qiongli doctor` from npm exits with full-runtime guidance.
- **Cause**: The npm CLI is a Python-free asset manager. It does not run `doctor`, provider setup, orchestrator, or the unified MCP server.
- **Fix**: Install the full runtime through pipx, then rerun the command:

```bash
pipx install qiongli
qiongli doctor
```

### `[ERR-RS-ENV-003]` `curl: (60) SSL certificate problem: certificate is not yet valid`.
- **Cause**: The install command reached GitHub, but TLS validation failed before the script could be downloaded. This usually means the machine clock is wrong, the CA certificate bundle is stale, or a proxy is intercepting HTTPS traffic.
- **Fix**:
  - Check the system clock first: `date -u` and `timedatectl status`.
  - Refresh CA certificates:
    - Debian/Ubuntu: `sudo apt-get update && sudo apt-get install --reinstall ca-certificates curl`
    - RHEL/CentOS/Fedora: `sudo dnf reinstall ca-certificates curl` then `sudo update-ca-trust`
  - If you are behind a corporate proxy, install the proxy root certificate into the system trust store.
  - Retry with the exact script name and shell expansion:
    - `curl -fsSL https://raw.githubusercontent.com/jxpeng98/qiongli/main/scripts/bootstrap_qiongli.sh | bash -s -- --project-dir "$PWD" --target all`
  - Avoid `curl -k` unless you are doing a temporary local test and understand the security tradeoff.

---

## Configuration & Standards (CFG)

These errors occur when the YAML contracts or JSON config maps contain invalid settings, or you requested a task/profile that cannot be mapped.

### `[ERR-RS-CFG-001]` Unknown or invalid agent profile specified.
- **Cause**: You passed a `--profile` (e.g., `--profile fast-writer`) that does not exist in `standards/agent-profiles.example.json`.
- **Fix**: Check your spelling. Use `task-run --help` or view the JSON file to see acceptable profiles (e.g., `default`, `academic-strict`, `bilingual-collaborator`).

### `[ERR-RS-CFG-002]` Could not read or parse standard YAML contracts.
- **Cause**: The `standards/` directory is missing critical standard files (like `research-workflow-contract.yaml`), or the YAML syntax is broken.
- **Fix**: Restore the `.yaml` files from version control. Run `python3 scripts/validate_research_standard.py --strict` to pinpoint YAML syntax errors.

### `doctor` reports missing standards files.
- **Cause**: The Python runtime cannot resolve the bundled `standards/` directory.
- **Fix**: Upgrade to a version with runtime standards discovery, then rerun:

```bash
qiongli doctor --cwd .
```

If you are running from source, use the repo root as the working directory or run through `uv run python -m bridges.orchestrator doctor --cwd .`.

### `[ERR-RS-CFG-003]` Unknown Task ID or invalid phase logic requested.
- **Cause**: You tried to run `rsk task-run --task-id X99`, but `X99` does not exist in the platform mapping.
- **Fix**: Refer to `standards/research-workflow-contract.yaml` to see all valid tasks (A1-I8).

---

## Execution & Orchestration (EXE)

These errors happen during the actual generation and agent runtime phase.

### `[ERR-RS-EXE-001]` All required parallel agents failed to execute.
- **Cause**: You requested a `parallel` execution, but all agents crashed or returned safety constraint violations immediately.
- **Fix**: Check the agent logs in `.agent/logs`. This usually happens if the `<cwd>` directory is completely empty and the agent lacks context, or if API rate limits were exhausted simultaneously.

### `[ERR-RS-EXE-002]` Subprocess execution timed out.
- **Cause**: A task exceeded the hardcoded maximum wait time (usually several minutes).
- **Fix**: The scope of the work is too large for one prompt. Break down your `task.md` into smaller sub-tasks.

---

## Model Context Protocol (MCP)

These errors specifically relate to the tools the AI uses to interact with your file system, search engines, or codebase.

### `[ERR-RS-MCP-001]` Required MCP provider is not configured.
- **Cause**: A task (like `scholarly-search`) explicitly demands an MCP tool that you have not configured into the environment. 
- **Fix**: Review `.env.example`. Set the appropriate environment variable (e.g., `RESEARCH_MCP_METADATA_REGISTRY_CMD="..."`). Alternatively, run without `--mcp-strict`.

### `[ERR-RS-MCP-002]` MCP provider command failed to execute or returned an error.
- **Cause**: The MCP server you configured crashed, returned non-JSON output, or timed out.
- **Fix**: If using Zotero MCP or other community MCPs, ensure that node server is actually running and your local API keys are valid. test the MCP independently using `doctor`.
