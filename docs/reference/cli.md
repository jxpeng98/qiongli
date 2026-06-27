# CLI Command Reference (qiongli)

This document outlines all "executable entry points" (pipx CLI / Python module / Bash scripts) mapping local calls and GitHub CI configurations for the `qiongli` package.

## 0) Command Name Conventions

- `qiongli`: The main CLI (available after pipx/venv installation, or after shell bootstrap install).
- `ql`: short primary alias. `research-skills`, `rsk`, and `rsw`: legacy compatibility aliases, equivalent to `qiongli`.

The rest of this document will use `qiongli` as the example.

---

## 1) How Upstream Repositories are Resolved (Omitting `--repo`)

Many commands need to know "which GitHub repository to query/download releases from." The resolution order for `qiongli` upstream is as follows (highest to lowest priority):

1. CLI Argument: `--repo <owner/repo|Git URL>`
2. Environment Variable: `QIONGLI_REPO=<owner/repo|Git URL>`
3. Legacy environment fallback: `RESEARCH_SKILLS_REPO=<owner/repo|Git URL>`
4. Project Configuration File (searched upwards from the current directory or `--project-dir`):
   - `qiongli.toml`
   - `.qiongli.toml`
5. Package Default (inside the pipx installed package): `qiongli/project.toml` (Injected by CI during publishing)
6. If running inside a `qiongli` repository clone: Inferred from git remote (prioritizes `upstream`, then `origin`)

Supported repo formats:

- `owner/repo`
- `https://github.com/owner/repo.git`
- `git@github.com:owner/repo.git`

We highly recommend committing the upstream configuration to your project repository (useful for CI automation):

```toml
# qiongli.toml
[upstream]
repo = "owner/repo"   # Or url = "https://github.com/owner/repo.git"
```

---

## 2) `qiongli` (Installer & Updater CLI)

There are two distributions of this CLI:
- Python CLI: installed via `pip`/`pipx`
- Shell CLI: installed by `bootstrap_qiongli.sh` into `${QIONGLI_BIN_DIR:-${RESEARCH_SKILLS_BIN_DIR:-~/.local/bin}}` by default
- npm launcher: installed via `npm install -g qiongli`; Node-only `install` remains available, while `setup`, `mcp`, and `self-update` delegate to the bundled Python bridge

### 2.0 Default install model

Current CLI defaults are plugin-first. The short commands:

```bash
qiongli install
qiongli upgrade
```

expand to the full local plugin surface:

| Interface | Default |
|---|---|
| Target | `--target all` |
| Subject package | `--subject core --coverage complete` |
| Runtime profile | `--profile full`, unless `--surface skills` is explicitly set without a profile, in which case the CLI uses `partial` |
| Output surface | `--surface plugin` for `install` and `upgrade` |
| Install mode | `--mode copy` |
| Project directory | Current working directory, used only when project-facing parts are requested |
| Shell CLI wrapper | Enabled by the effective `full` profile; use `--no-cli` to skip wrapper refresh |
| MCP registration | Enabled by the effective `full` profile; plugin-owned Codex/Claude MCP entries are skipped because the local plugin owns them |
| Doctor | Not run by default for direct `install` / `upgrade`; pass `--doctor` or include `doctor` in `--parts` |
| Overwrite | `install` defaults to no overwrite; `upgrade` defaults to overwrite and supports `--no-overwrite` |

`--surface` is the high-level output selector:

| Surface | What it installs |
|---|---|
| `plugin` | CLI-managed local plugins for Codex/Claude Code/Antigravity; Antigravity bundles root `mcp_config.json`; Hermes receives managed MCP config when included by target |
| `skills` | Legacy global `qiongli-workflow` skill directories and workflow discovery where supported |
| `both` | Both legacy global skills and the local plugin surface |

`--parts` is the precise override. When set, it replaces the surface/profile-derived install set with the comma-separated parts you name: `globals`, `plugin`, `project`, `cli`, `mcp`, and `doctor`. `all` and `*` expand to every part. Use this for automation that must touch exactly one surface, for example `--parts mcp` or `--parts plugin,cli`.

### 2.1 `qiongli check` (Check versions/Available updates)

Use Case:
- Outputs the CLI version, local repo version (if run from a clone), and installed versions across supported client surfaces.
- Optional: Queries the upstream latest release tag and determines if an upgrade is needed.

```bash
qiongli check [--repo <owner/repo|url>] [--json] [--strict-network] [--offline]
```

Key Flags:
- `--repo`: Specify upstream (can be omitted, see "Upstream" section).
- `--json`: Output JSON only (useful for CI/Scripts).
- `--strict-network`: Return a failure code if upstream polling fails (defaults to warning and continuing).
- `--offline`: Skip PyPI and upstream release checks; inspect local install surfaces only.

`qiongli check` is plugin-aware. It reports `surface=plugin` for CLI-managed Codex/Claude Code/Antigravity local plugins, `surface=mcp` for Hermes or MCP-only managed configs, `surface=legacy_skill` for old global skill directories, and `surface=none` when no Qiongli surface is found. JSON output keeps the compatibility fields `installed`, `version`, `subject`, `coverage`, and `path`, and adds nested `plugin`, `skill`, and `mcp` objects for diagnostics. Plugin diagnostics include `active`, `enabled`, `plugin_id`, and `activation_detail` where the client CLI can be queried, so file installation can be distinguished from an enabled client plugin. Older managed installs that do not have a `SUBJECT_MANIFEST.json` or `SUBJECT` marker are reported as legacy `core` / `complete`.

If Codex lists `qiongli` in the Personal marketplace but the details page says `Plugin detail unavailable`, the marketplace entry was found but Codex could not read the local plugin payload. Check the local plugin root named by `qiongli check --json`; common causes are an invalid `.codex-plugin/plugin.json`, invalid YAML frontmatter in `skills/qiongli-workflow/SKILL.md`, or a missing local path. Reinstall CLI-managed Codex plugin payloads with:

```bash
qiongli install --target codex --surface plugin --overwrite
```

Exit Codes:
- `0`: No updates available / upstream check bypassed.
- `1`: Update available.
- `2`: Invalid argument.

### 2.2 `qiongli setup` (Interactive CLI setup wizard)

Use Case:
- Recommended first command after installing the CLI with npm, pipx, pip, or bootstrap.
- Guides CLI, Codex, Claude Code, Antigravity, and Hermes users through install vs upgrade, runtime surface, subject, coverage, install mode, install scope, overwrite policy, upgrade source, optional provider key setup, and doctor verification.

```bash
qiongli setup [--project-dir <path>] [--dry-run] [--no-doctor] [--provider-mode page|prompt|skip] [--no-browser]
```

Examples:

```bash
qiongli setup
qiongli setup --dry-run
qiongli setup --project-dir "$PWD" --no-doctor
qiongli setup --provider-mode prompt --no-browser
```

When invoked through the npm launcher, `qiongli setup` uses the bundled Python bridge and requires Python 3.12+ plus `PyYAML`. The explicit `qiongli install ...` npm command remains available for Node-only asset installation.

Wizard choices:
- Setup path: `install` or `upgrade`.
- Runtime surface: `cli`, `codex`, `claude-code`, `antigravity`, `hermes`, or `multi-platform`.
- Subject: `core`, `economics`, `accounting`, `business`, `finance`, `political-economy`, `geoeconomics`, or `economics-accounting`.
- Coverage: `complete` or `focused`.
- Install mode: `--mode copy` for normal use, or `--mode link` for local development.
- Install scope: `all`, `globals`, `project`, or `cli`.
- Shell CLI directory when the selected scope includes CLI wrappers.
- Overwrite policy: `--overwrite` for install refreshes, or `--no-overwrite` when upgrading without replacing managed files.
- Upgrade source: latest stable, latest beta, optional `--repo`, explicit `--ref`, and `--ref-type tag|branch`.
- Optional provider setup for literature provider credentials. The default `page` mode opens one local browser page with fields and access guidance for OpenAlex, Semantic Scholar, Crossref, PubMed, and arXiv. Use `--no-browser` to print the local URL only, `--provider-mode prompt` for terminal-only key entry, or `--provider-mode skip` to bypass provider setup.
- Doctor verification unless `--no-doctor` is set.

Every prompt includes a short `Tip:` comment that explains why the choice matters and which install or upgrade behavior it changes.

Provider keys saved through setup use the same provider config as `qiongli provider setup` and `qiongli provider doctor`. The local setup page includes links and short steps for obtaining each supported credential; arXiv is marked as available without an API key. Secrets are stored outside generated research artifacts. Setup configures credentials and runs doctor/capability checks; it does not promise that an external literature search will run.

### 2.2.1 `qiongli self-update` / `qiongli update` (Update package and refresh surfaces)

Use Case:
- Updates the CLI package through the package manager that installed it.
- After the package update succeeds, refreshes the installed full local plugin/MCP surface from the newly installed package payload.
- Keeps native marketplace plugins separate; they remain managed by Codex, Claude Code, or the relevant client plugin manager.

```bash
qiongli self-update [--channel stable|next] [--target codex|claude|antigravity|hermes|all] [--surface skills|plugin|both] [--profile partial|full] [--dry-run] [--yes]
qiongli update --dry-run
```

Default behavior:
- `--channel stable` delegates to `npm install -g qiongli@latest`, `pipx upgrade qiongli`, or `python -m pip install --upgrade qiongli`, depending on the detected install channel.
- `--channel next` delegates to `npm install -g qiongli@next` or enables Python prerelease upgrades with `--pre`.
- Refresh defaults to `qiongli install --target all --surface plugin --profile full --overwrite`. This is intentional: after the package manager updates the CLI package, the bundled payload is already local, so the refresh should not download another release archive.
- `--dry-run` prints the detected channel and exact commands without executing them.
- Without `--yes`, the command prints the plan and exits without making changes.
- `--no-refresh` skips the installed surface refresh, and `--skip-check` skips the final `qiongli check --offline`.

Source checkouts do not self-modify. When source mode is detected, update with `git pull`, then run `qiongli install --overwrite` for the surfaces you want to refresh.

### 2.2.2 `qiongli doctor` (Runtime and client integration health)

Use Case:
- Runs the Python orchestrator doctor for the selected project directory.
- Prints a non-fatal client integration summary using the same plugin/MCP/legacy skill discovery as `qiongli check`.

```bash
qiongli doctor --cwd .
```

`doctor` validates runtime pieces such as project files, provider/orchestrator readiness, and local model CLI availability. It does not install or remove plugins. Missing optional client integrations are reported in the summary but do not by themselves make `doctor` fail; the exit code remains the orchestrator doctor exit code.

### 2.2.3 `qiongli mcp` (Cross-platform MCP server)

Use Case:
- Runs the local Qiongli MCP server for desktop or agent clients that support MCP.
- Lets CLI users and desktop-only users configure the same provider keys.
- Generates client config examples without embedding secrets.

```bash
qiongli mcp serve --transport stdio
qiongli mcp serve --transport http --host 127.0.0.1 --port 8765
qiongli mcp configure --provider openalex --field email --value you@example.com
qiongli mcp doctor --json
qiongli mcp config example --target codex --json
qiongli mcp config example --target claude-code --json
qiongli mcp config example --target antigravity --json
qiongli mcp config example --target hermes --json
qiongli mcp wizard
```

MCP tools exposed by the server:
- `qiongli_config_status`
- `qiongli_save_provider_config`
- `qiongli_collect_evidence`
- `qiongli_list_provider_env`
- `qiongli_test_provider`
- `qiongli_configure_provider`
- `qiongli_orchestrator_route`
- `qiongli_orchestrator_doctor`
- `qiongli_task_plan`
- `qiongli_task_run`

Default `stdio` mode is local and does not require a remote server. HTTP mode can also run locally; use a remote server only when the client cannot launch local MCP commands or when you need a managed shared endpoint. Codex, Claude Code, Antigravity, Hermes, or another local MCP client should call `qiongli_orchestrator_route` when deciding whether to upgrade from skill-only routing to full orchestrator tools. `qiongli_task_run` defaults to preview mode and launches local model CLIs only when the MCP caller explicitly sets JSON boolean `run_agents: true`. The tool accepts `guidance_mode: "off" | "read" | "propose" | "apply"`; preview responses echo the effective task-run arguments and report whether project guidance will be bootstrapped, but do not create files or launch agents.

### 2.3 `qiongli install` (Install bundled subject payload)

Use Case:
- Installs the subject payload bundled inside the PyPI/npm/source checkout as the current local Qiongli surface.
- Defaults to the full plugin surface: local plugins for Codex/Claude Code/Antigravity, bundled Antigravity plugin MCP, managed Hermes MCP config, and a refreshed shell CLI wrapper unless `--no-cli` is set.
- Does not migrate or remove old global skills by default; use `qiongli upgrade` for automatic migration or `qiongli remove` for explicit cleanup.

```bash
qiongli install \
  [--profile partial|full] \
  [--subject core|economics|accounting|business|finance|political-economy|geoeconomics|economics-accounting] \
  [--coverage complete|focused] \
  [--target codex|claude|antigravity|hermes|all] \
  [--surface skills|plugin|both] \
  [--mode copy|link] \
  [--project-dir <path>] \
  [--install-cli | --no-cli] \
  [--cli-dir <path>] \
  [--overwrite] \
  [--doctor] \
  [--parts globals,plugin,project,cli,mcp,doctor] \
  [--dry-run]
```

Examples:

```bash
qiongli install --target all
qiongli install --surface skills --profile partial --target all
qiongli install --profile full --target codex --surface plugin
qiongli install --profile full --target all --surface plugin
qiongli install --profile full --target all --surface both
qiongli install --parts mcp --target hermes
```

For normal CLI/local plugin use, install Qiongli once and set subject behavior per project with `qiongli project ...`. Subject install flags are retained for legacy and advanced compatibility cases: focused Claude Desktop/Web ZIPs, deliberately narrow packages, release payloads, and install-surface testing.

Subject packages are specialized installs, not reduced-quality cuts. Default install is `core/complete`. `--subject economics`, `--subject business`, `--subject finance`, `--subject political-economy`, and `--subject geoeconomics` mean complete specialized installs, not reduced packages. `--subject accounting` means `accounting/complete`, full framework plus accounting specialization. Focused coverage selects the subject profile set and active effective skills for deliberate slim installs and Desktop/Web ZIPs. Current official subjects are `core`, `economics`, `accounting`, `business`, `finance`, `political-economy`, `geoeconomics`, and the named composite `economics-accounting`; `political-economy` and `geoeconomics` are independent subject choices, not a composite. Official composites are not arbitrary comma-separated stacking. Public Desktop ZIP subjects are `core`, `economics`, `business`, `finance`, `political-economy`, `geoeconomics`, and `economics-accounting`, with no standalone accounting Desktop ZIP in this phase. Change ordinary project subject behavior with `qiongli project set-subject`; switch installed subject or coverage only when you are intentionally refreshing a specialized package.

`qiongli install` defaults to `--profile full --surface plugin` from v1.9.0 onward. For Codex, the CLI writes a personal marketplace entry, places the plugin payload at `~/plugins/qiongli`, writes plugin `.mcp.json` that launches `qiongli mcp serve --transport stdio`, and runs `codex plugin add qiongli@personal` when the Codex CLI is available. For Claude Code, it writes a local marketplace under `~/.qiongli/plugins/claude-code`, places the plugin payload at `plugins/qiongli`, and runs `claude plugin marketplace add ...` plus `claude plugin install qiongli@qiongli-local --scope user` when the Claude CLI is available. For Antigravity, it writes a root `plugin.json` plugin bundle under `~/.qiongli/plugins/antigravity/qiongli`, writes the full MCP server config to the plugin root `mcp_config.json`, and runs `antigravity plugin install <path>` when available. With `--target all`, Codex/Claude Code/Antigravity use local plugins while Hermes receives managed full MCP client config. Marketplace-installed plugins stay on the lite no-Python path with the bundled Node literature provider. Use `--surface skills --profile partial` for the old skills-only layout.

Install behavior details:
- `--surface plugin --target all` installs CLI-managed local plugins for Codex, Claude Code, and Antigravity; Antigravity's plugin includes root `mcp_config.json`, while Hermes receives managed client-level MCP config.
- `--surface skills --profile partial` installs only legacy skill directories and workflow discovery; it does not install the shell wrapper or MCP config unless `--parts cli` or `--parts mcp` is used.
- `--surface both` keeps legacy global skills available while also installing local plugins; use this only when you intentionally want both discovery paths.
- `--parts` wins over `--surface`. For example, `--parts mcp --target antigravity` writes only the Antigravity MCP config, and `--parts project` writes only project-facing files.
- `--doctor` is explicit for direct install. The interactive setup wizard may offer doctor verification by default, but `qiongli install` itself does not run doctor unless requested.

### 2.4 `qiongli upgrade` (Download release & execute installers)

Use Case:
- Downloads the upstream release (defaults to latest tag `.tar.gz`).
- Extracts it and runs the packaged Python installer.
- Defaults to the full local plugin surface and migrates old global skills / Codex/Claude standalone MCP configs after the new install succeeds.

```bash
qiongli upgrade \
  [--repo <owner/repo|url>] \
  [--ref <tag-or-branch>] \
  [--ref-type tag|branch] \
  [--profile partial|full] \
  [--subject core|economics|accounting|business|finance|political-economy|geoeconomics|economics-accounting] \
  [--coverage complete|focused] \
  [--target codex|claude|antigravity|hermes|all] \
  [--surface skills|plugin|both] \
  [--project-dir <path>] \
  [--install-cli | --no-cli] \
  [--cli-dir <path>] \
  [--overwrite | --no-overwrite] \
  [--doctor] \
  [--parts globals,plugin,project,cli,mcp,doctor] \
  [--dry-run]
```

Notes:
- `qiongli upgrade` is a release-archive refresh command. To update the installed npm/pipx/pip package first, use `qiongli self-update --yes`; it updates the package, then runs `qiongli install --overwrite` to refresh client surfaces from the new local package payload.
- `--project-dir` matters when you also request project-facing surfaces, such as `--parts project`.
- Default `upgrade` behaves like `--profile full --surface plugin`: it installs/refreshes local plugins for Codex, Claude Code, and Antigravity, bundles the Antigravity MCP config in the plugin, writes managed Hermes MCP config, then removes legacy global skills and standalone Codex/Claude/Antigravity MCP configs only after installation succeeds.
- Use `qiongli upgrade --surface skills --profile partial ...` when you explicitly want to keep the old skills-only upgrade path.
- Use `qiongli upgrade --surface both ...` when you intentionally want to keep legacy global skills alongside the plugin surface; this does not run the plugin migration cleanup.
- Migration cleanup runs only after a successful effective `--surface plugin` upgrade when the selected parts are omitted or include `plugin`. Failed installs never remove old assets.
- Migration cleanup removes legacy global `qiongli-workflow` skill directories, Claude Code workflow discovery links, and standalone Codex/Claude/Antigravity MCP config. It leaves Hermes MCP config in place because Hermes still uses client-level managed MCP config.
- Use `qiongli init --project-dir .` for project bootstrap, or `qiongli upgrade --parts project ...` when you explicitly want project files rewritten.
- `--subject` defaults to `core` and `--coverage` defaults to `complete`; use subject install flags for specialized package refreshes, focused Desktop/Web ZIP payloads, or compatibility testing. For ordinary per-project subject selection, use `qiongli project set-subject`.
- Example: `qiongli upgrade --subject accounting --target all`.
- Example: `qiongli upgrade --target all` refreshes the full local plugin surface without switching to the marketplace lite plugin.
- Legacy skills-only upgrades create workflow discovery symlinks under `~/.claude/commands/*.md` for direct `/paper`, `/lit-review`, etc. invocation in Claude Code.
- Shell CLI uses the bundled bootstrap helper. The full plugin/MCP path requires a Python-capable `qiongli` runtime because plugins launch `qiongli mcp serve --transport stdio`.
- The command exits with the error code returned by the underlying installer.

### 2.5 `qiongli align` (Quick Reference Guide)

Use Case: Prints an overview of "what pipx installed / paths modified by upgrades / common commands".

```bash
qiongli align [--repo <owner/repo|url>]
```

### 2.6 `qiongli init` (Project Bootstrap)

Use Case: Creates project-local `.env` configuration in your project directory.

```bash
qiongli init \
  [--project-dir <path>] \
  [--target all|codex|claude|antigravity|hermes] \
  [--mode copy|link] \
  [--overwrite] \
  [--doctor] \
  [--parts project,doctor] \
  [--dry-run]
```

Notes:
- Defaults to `--parts project` and only creates project-facing assets (`.env`). It does not touch global skill directories, local plugins, or MCP configs unless explicit parts are passed.
- Safe to run multiple times; will not overwrite existing files unless `--overwrite` is passed.

### 2.7 `qiongli remove` (Remove CLI-installed assets)

Use Case: Removes assets installed by the CLI so you can switch cleanly between npm/PyPI/bootstrap installs and native marketplace plugins.

```bash
qiongli remove \
  [--target codex|claude|antigravity|hermes|all] \
  [--surface skills|plugin|both] \
  [--parts globals|project|cli|mcp|plugin] \
  [--project-dir <path>] \
  [--cli-dir <path>] \
  [--dry-run]
```

Examples:

```bash
qiongli remove --target all --dry-run
qiongli remove --target codex
qiongli remove --target codex --surface plugin
qiongli remove --parts plugin --target codex
qiongli remove --parts globals,project --project-dir "$PWD"
qiongli remove --parts cli --cli-dir ~/.local/bin
qiongli uninstall --target all
qiongli delete --target claude
```

Notes:
- `remove` defaults to `--parts globals` and deletes CLI-installed `qiongli-workflow` skill directories plus generated workflow discovery links.
- It skips unmanaged `qiongli-workflow` directories that do not look like Qiongli package payloads.
- Plugin removal deletes only CLI-managed local full plugin roots marked with `.qiongli-managed.json`, and Codex marketplace entries marked with `metadata.managedBy = "qiongli-cli"`.
- `--surface plugin` removes only the CLI-managed local plugin surface. It does not remove MCP client config; use `--parts mcp` for that.
- `--surface both` removes legacy global skills plus CLI-managed local plugins, but still leaves MCP config unless `--parts mcp` is included.
- It does not uninstall marketplace plugins such as `qiongli` or `qiongli-next`; remove those through the Codex, Claude Code, or Claude Desktop plugin manager.
- Use `--parts project` when you also want the old project-local cleanup performed by `qiongli clean`.
- Use `--parts cli` only when you installed shell wrappers through the full CLI/bootstrap path.

### 2.8 `qiongli clean` (Remove Stale Assets)

Use Case: Removes stale project-local assets left from older installations.

```bash
qiongli clean [--project-dir <path>] [--dry-run] [--globals]
```

Flags:
- `--project-dir`: Directory to clean (default: current dir). Removes `.agent/workflows/`, `.agents/skills/qiongli-workflow/`, `CLAUDE.qiongli.md`, `.gemini/qiongli.md`, and template-matching `CLAUDE.md`.
- `--globals`: Also remove workflow discovery symlinks from `~/.claude/commands/`. Only removes symlinks that point to `qiongli-workflow` — user-created commands are preserved.
- `--dry-run`: Show what would be removed without deleting.

### 2.9 `qiongli doctor` (Environment Preflight)

Use Case: Runs orchestrator preflight checks (CLIs, API keys, MCP wiring).

```bash
qiongli doctor [--cwd <path>]
```

### 2.10 `qiongli customize` (Create a custom subject overlay)

Use Case:
- Creates a local custom overlay scaffold for the Python/source checkout materialization workflow.
- Custom overlays affect generated output only and do not rewrite canonical source files.
- npm runtime installs use pre-generated payloads in this phase and do not materialize `--custom-dir` overlays at install time.

```bash
qiongli customize --subject economics --name my-econ-lab --out ./qiongli-custom/econ-lab
python3 scripts/materialize_subject_package.py --subject economics --custom-dir ./qiongli-custom/econ-lab --source . --out /tmp/qiongli-workflow
```

Developer subject-depth workflow: when adding or deepening a subject, update `subjects/catalog.yaml`, subject overlays, subject-specific registry and markdown, selected domain and venue profiles, subject eval fixtures, specialization audit expected terms, materializer tests, npm package contract tests against staged materialization when the subject is installable through npm, and release validation if the subject has a Desktop/Web artifact.

---

## 3) Orchestrator CLI: `python3 -m bridges.orchestrator`

This is the execution entry point for "Parallel Fallbacks & Task-Run Contract Execution".

```bash
python3 -m bridges.orchestrator <mode> [args...]
```

Available modes:

- `doctor`: Environment Preflight Checks
  ```bash
  python3 -m bridges.orchestrator doctor --cwd .
  ```
- `parallel`: 3-Agent Parallel Analysis + Synthesis (Auto-downgrades to dual/single if unavailable)
  ```bash
  python3 -m bridges.orchestrator parallel \
    --prompt "Analyze this study design" \
    --cwd . \
    --summarizer claude \
    --profile-file standards/agent-profiles.example.json \
    --profile default
  ```
- `task-run`: Standard pipeline execution via Task ID (plan -> evidence -> draft -> review -> gates -> write to RESEARCH/)
  ```bash
  python3 -m bridges.orchestrator task-run \
    --task-id F3 \
    --paper-type empirical \
    --topic your-topic \
    --cwd . \
    --triad
  ```
  Common parameters:
  - `--domain <name>`: inject a runtime domain profile (for example `econ`, `cs`, `psychology`) into the task packet and prompts
  - `--venue <name>` / `--context <text>`
  - `--mcp-strict` / `--skills-strict`
  - `--profile-file <path>` + `--profile <name>` (along with `--draft-profile` / `--review-profile` / `--triad-profile`)
  - `--focus-output <path>` (repeatable) + `--output-budget <n>`: narrow this run to a smaller active output set and defer the rest of the contract outputs explicitly
  - `--research-depth standard|deep` + `--max-rounds <n>`: increase evidence-expansion pressure and enforce a deeper review/revision loop
  - `--only-target <id>` (repeatable): for structured Stage-I tasks `I4`-`I8`, reload the existing artifact under `RESEARCH/[topic]/code/` and rerun only the named actionable targets
  - `--skip-validation`: disable strict MCP/skill availability checks and skip the artifact validator gate for fast iteration; the run will emit an explicit warning and mark `validator_gate.skipped=true`
  - `--guidance-mode off|read|propose|apply`: control project-local guidance under `.qiongli/`; default `propose` reads guidance when present, writes a trace bundle, and produces a conservative update proposal
  - `--update-academic-context`: for supported stage-close tasks (`A5`, `B6`, `C5`, `D3`, `E5`, `F6`, `H4`), append `context/research_state.md` and `context/decision_log.md` to this run's active outputs and inject stage-specific academic continuity guidance into the draft prompt
  - Built-in profiles now include `focused-delivery` and `deep-research` in addition to `default`, `rapid-draft`, and `strict-review`

  Formal research artifacts still belong under `RESEARCH/[topic]/...`. Project subject context comes from `.qiongli/guidance_manifest.yaml`; when it is missing, the effective subject is `active_subject: auto`. The first non-`off` task run automatically initializes `.qiongli/local_guidance.md` and `.qiongli/trace/` when they are missing. The orchestrator prompts runtime agents to create the required files; if an agent only returns text and does not write those files, the validator reports them as missing. Guidance trace bundles are written separately under `.qiongli/trace/runs/<run_id>/` so the run remains auditable even when formal outputs are incomplete.

  Example: reduce artifact sprawl but keep stronger review pressure
  ```bash
  python3 -m bridges.orchestrator task-run \
    --task-id F3 \
    --paper-type empirical \
    --topic your-topic \
    --cwd . \
    --focus-output manuscript/manuscript.md \
    --research-depth deep \
    --draft-profile deep-research \
    --review-profile strict-review \
    --triad-profile deep-research \
    --triad \
    --max-rounds 4
  ```
  Example: rerun only a blocked Stage-I planning step
  ```bash
  python3 -m bridges.orchestrator task-run \
    --task-id I6 \
    --paper-type methods \
    --topic llm-bias \
    --cwd . \
    --only-target S1
  ```
  Example: force a stage-close run to refresh project-level academic continuity artifacts
  ```bash
  python3 -m bridges.orchestrator task-run \
    --task-id F6 \
    --paper-type empirical \
    --topic your-topic \
    --cwd . \
    --update-academic-context
  ```
- `task-plan`: Renders the dependency execution order based on the contract
  ```bash
  python3 -m bridges.orchestrator task-plan --task-id F3 --paper-type empirical --topic your-topic --cwd .
  ```

### Project subject guidance: `qiongli project`

Use `qiongli project` to keep subject, venue, method-lens, and strictness context in the project instead of reinstalling Qiongli for every paper.

```bash
qiongli project init --project-dir .
qiongli project set-subject finance --project-dir .
qiongli project set-venue journal-of-finance --project-dir .
qiongli project set-method-lens event-study --project-dir .
qiongli project status --project-dir .
```

These commands read and write `.qiongli/guidance_manifest.yaml`. The manifest can include `active_subject`, `secondary_subjects`, `venue_profiles`, `method_lenses`, and `strictness`. If the file is missing, the effective default is `active_subject: auto`: Qiongli remains usable without setup, uses core guidance, and may infer temporary subject or method lenses from the current task.

Qiongli does not silently persist a subject switch. Persistent project changes come only from explicit `qiongli project ...` commands or from accepted guidance proposals. Task runs may propose manifest or local-guidance updates for audit, but unaccepted proposals do not change project-local state.

- `guidance`: Manage project-local guidance and trace bundles
  ```bash
  qiongli guidance init --project-dir .
  qiongli guidance show --project-dir .
  qiongli guidance add --project-dir . --name writing-style
  qiongli guidance list --project-dir .
  qiongli guidance lint --project-dir .
  qiongli guidance trace --project-dir .
  qiongli guidance apply \
    --project-dir . \
    --proposal .qiongli/trace/runs/<run_id>/guidance_update_proposal.md
  ```
  Project-local subject context lives in `.qiongli/guidance_manifest.yaml`; project-local customization lives in `.qiongli/local_guidance.md`; run trace records live in `.qiongli/trace/index.jsonl` and `.qiongli/trace/runs/<run_id>/`. These files are intentionally separate from canonical workflow contracts, bundled skills, and release payloads.
  Guidance proposals are project-local by default. A proposal may suggest `user-global` or `canonical-candidate`, but `qiongli guidance apply` only writes `.qiongli/local_guidance.md`. Promoting a rule to `~/.qiongli/preferences.md` or canonical source requires an explicit future command or normal repository PR.
- `code-build`: Academic code workflow entry point
  ```bash
  python3 -m bridges.orchestrator code-build \
    --method "Staggered DID" \
    --topic policy-effects \
    --domain econ \
    --focus full \
    --cwd .
  ```
  Key parameters:
  - `--topic <slug>`: when present, `code-build` routes into strict Stage-I workflow instead of the legacy prompt-only path
  - `--focus <name>`: map into `I1`/`I2`/`I3`/`I4`/`I5`/`I6`/`I7`/`I8`, or use `full` for `I5 -> I6 -> I7 -> I8`
  - `--domain <name>`: inject the matching `skills/domain-profiles/*.yaml`
  - `--paper-type <type>`: workflow paper type used by strict Stage-I routing
  - `--triad`: add the third independent audit on the final strict review pass
  - `--paper <path-or-url>`: optional paper reference carried into the task context
  - `--only-target <selector>` (repeatable): targeted follow-up mode
    - single-stage focus: use bare target IDs such as `S1` or `P1-01`
    - `--focus full`: use `STAGE_ID:TARGET` selectors such as `I5:decision-1` or `I8:P1-01`

  Example: run only the spec phase for an advanced CS method
  ```bash
  python3 -m bridges.orchestrator code-build \
    --method "Transformer Fine-Tuning" \
    --topic llm-bias \
    --domain cs \
    --tier advanced \
    --focus code_specification \
    --paper-type methods \
    --cwd .
  ```
  Example: rerun only specific full-flow targets
  ```bash
  python3 -m bridges.orchestrator code-build \
    --method "Transformer Fine-Tuning" \
    --topic llm-bias \
    --domain cs \
    --focus full \
    --only-target I5:decision-1 \
    --only-target I8:P1-01 \
    --cwd .
  ```
- `single`: Single-model execution (Quick debug/runs)
  ```bash
  python3 -m bridges.orchestrator single --prompt "..." --cwd . --model codex
  ```
- `chain`: Iterative refinement (One builds, the other verifies)
  ```bash
  python3 -m bridges.orchestrator chain --prompt "..." --cwd . --generator codex
  ```
- `role`: Execution split by specialized roles
  ```bash
  python3 -m bridges.orchestrator role --cwd . --codex-task "..." --claude-task "..."
  ```

---

## 4) Bash Scripts (Non-pipx)

### 4.1 Remote Bootstrap Installer: `./scripts/bootstrap_qiongli.sh`

Use case:
- Install or refresh skills on machines without Python.
- Downloads a GitHub release/branch archive, extracts it, and then runs `scripts/install_qiongli.sh` from that archive.

```bash
./scripts/bootstrap_qiongli.sh \
  --repo owner/repo \
  --target all \
  --project-dir /path/to/project \
  --overwrite
```

Notes:
- Requires `bash` and either `curl` or `wget`, plus `tar`.
- Supports `--ref <tag-or-branch>` with `--ref-type tag|branch`.
- Installs shell CLI commands by default: `qiongli`, `ql`, `research-skills`, `rsk`, `rsw`.
- Use `--no-cli` to skip shell CLI installation, or `--cli-dir <path>` to choose the install location.
- Remote bootstrap supports `--mode copy` only.
- `--doctor` auto-skips when `python3` is unavailable.

### 4.2 Installer Script: `./scripts/install_qiongli.sh`

```bash
./scripts/install_qiongli.sh \
  --target all \
  --mode copy \
  --project-dir /path/to/project \
  --install-cli \
  --overwrite \
  --doctor
```

Notes:
- This is the local-repository installer.
- The copy/link install path no longer requires Python.
- Add `--install-cli` to also install the shell CLI into `${QIONGLI_BIN_DIR:-${RESEARCH_SKILLS_BIN_DIR:-~/.local/bin}}` or `--cli-dir <path>`.
- `--doctor` runs `python3 -m bridges.orchestrator doctor --cwd <project>` only when `python3` exists.

### 4.3 Release Automation: `./scripts/release_automation.sh`

```bash
./scripts/release_automation.sh publish --version 0.1.0 --from-tag v0.1.0-beta.X
./scripts/release_automation.sh pre  --tag v0.1.0-beta.X
./scripts/release_automation.sh post --tag v0.1.0-beta.X --create-release
```

Recommended:

- use `publish` as the only routine release entrypoint
- use `pre` / `post` only for diagnostics or recovery
- let `publish` own commit, branch push, branch CI/check gate, tag push, tag publish wait, GitHub Release, and acceptance receipt
- do not create or push the release tag until the release-prep commit has passed `CI` and `Checkout Install Check`
- stable releases publish from the matching `CHANGELOG.md` section
- beta / prerelease releases publish from `tooling/release/<tag>.md`

Also executable individually:

```bash
./scripts/release_preflight.sh [--tag v0.1.0-beta.X] [--quick] [--skip-smoke] [--maintainer-smoke] [--no-strict]
./scripts/release_postflight.sh --tag v0.1.0-beta.X [--skip-remote] [--skip-ci-status] [--wait-ci] [--ci-timeout-seconds 900] [--ci-timeout-mode soft] [--create-release]
```

`publish` always uses hard CI gates before tag creation and before GitHub Release creation. For manual `post` diagnostics or recovery, `--ci-timeout-mode soft` can record unresolved CI as `pending` in the acceptance receipt, but it is not valid for routine publishing.

### 4.4 Beta smoke tests: `./scripts/run_beta_smoke.sh`

```bash
./scripts/run_beta_smoke.sh
./scripts/run_beta_smoke.sh --tier release
./scripts/run_beta_smoke.sh --tier maintainer
```

This smoke entrypoint supports two tiers:

- `release`: built-in literature pipeline smoke + `doctor`
- `maintainer`: everything in `release`, plus `parallel` and `task-run` profile-path checks

Release preflight now uses the `release` tier by default. Use `--maintainer-smoke` in release tooling when you explicitly want the heavier maintainer checks.

### 4.5 Literature smoke: `./scripts/run_literature_smoke.sh`

```bash
./scripts/run_literature_smoke.sh
```

### 4.6 CI Default Upstream Injector: `./scripts/inject_project_toml.sh`

Executed by GitHub actions during packaging to hardcode the repo slug into `qiongli/project.toml`.

```bash
bash scripts/inject_project_toml.sh

# Or override the repo slug dynamically during builds
QIONGLI_REPO_SLUG="other-owner/other-repo" bash scripts/inject_project_toml.sh
```

---

## 5) Validators (Recommended before CI/Deployment)

```bash
python3 scripts/validate_research_standard.py --strict
python3 -m unittest tests.test_orchestrator_workflows -v
```

Project Artifact Validator (run inside your actual project output directory):

```bash
python3 scripts/validate_project_artifacts.py \
  --cwd /path/to/project \
  --topic your-topic \
  --task-id H1 \
  --strict
```
