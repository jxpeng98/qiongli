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

### 2.1 `qiongli check` (Check versions/Available updates)

Use Case:
- Outputs the CLI version, local repo version (if run from a clone), and installed versions across all 3 client directories.
- Optional: Queries the upstream latest release tag and determines if an upgrade is needed.

```bash
qiongli check [--repo <owner/repo|url>] [--json] [--strict-network]
```

Key Flags:
- `--repo`: Specify upstream (can be omitted, see "Upstream" section).
- `--json`: Output JSON only (useful for CI/Scripts).
- `--strict-network`: Return a failure code if upstream polling fails (defaults to warning and continuing).

JSON output includes the active installed subject and coverage for each target. Older managed installs that do not have a `SUBJECT_MANIFEST.json` or `SUBJECT` marker are reported as legacy `core` / `complete`.

Exit Codes:
- `0`: No updates available / upstream check bypassed.
- `1`: Update available.
- `2`: Invalid argument.

### 2.2 `qiongli setup` (Interactive CLI setup wizard)

Use Case:
- Recommended first command after installing the CLI with npm, pipx, pip, or bootstrap.
- Guides CLI, Codex, Claude Code, and Antigravity users through install vs upgrade, runtime surface, subject, coverage, install mode, install scope, overwrite policy, upgrade source, optional provider key setup, and doctor verification.

```bash
qiongli setup [--project-dir <path>] [--dry-run] [--no-doctor]
```

Examples:

```bash
qiongli setup
qiongli setup --dry-run
qiongli setup --project-dir "$PWD" --no-doctor
```

When invoked through the npm launcher, `qiongli setup` uses the bundled Python bridge and requires Python 3.12+ plus `PyYAML`. The explicit `qiongli install ...` npm command remains available for Node-only asset installation.

Wizard choices:
- Setup path: `install` or `upgrade`.
- Runtime surface: `cli`, `codex`, `claude-code`, `antigravity`, or `multi-platform`.
- Subject: `core`, `economics`, `accounting`, `business`, `finance`, `political-economy`, `geoeconomics`, or `economics-accounting`.
- Coverage: `complete` or `focused`.
- Install mode: `--mode copy` for normal use, or `--mode link` for local development.
- Install scope: `all`, `globals`, `project`, or `cli`.
- Shell CLI directory when the selected scope includes CLI wrappers.
- Overwrite policy: `--overwrite` for install refreshes, or `--no-overwrite` when upgrading without replacing managed files.
- Upgrade source: latest stable, latest beta, optional `--repo`, explicit `--ref`, and `--ref-type tag|branch`.
- Optional provider keys for literature provider credentials.
- Doctor verification unless `--no-doctor` is set.

Every prompt includes a short `Tip:` comment that explains why the choice matters and which install or upgrade behavior it changes.

Provider keys entered through setup use the same provider config as `qiongli provider setup` and `qiongli provider doctor`. Secrets are stored outside generated research artifacts. Setup configures credentials and runs doctor/capability checks; it does not promise that an external literature search will run.

### 2.2.1 `qiongli mcp` (Cross-platform MCP server)

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

Default `stdio` mode is local and does not require a remote server. HTTP mode can also run locally; use a remote server only when the client cannot launch local MCP commands or when you need a managed shared endpoint. Codex, Claude Code, Antigravity, or another local MCP client should call `qiongli_orchestrator_route` when deciding whether to upgrade from skill-only routing to full orchestrator tools. `qiongli_task_run` defaults to preview mode and launches local model CLIs only when the MCP caller explicitly sets JSON boolean `run_agents: true`. The tool accepts `guidance_mode: "off" | "read" | "propose" | "apply"`; preview responses echo the effective task-run arguments and report whether project guidance will be bootstrapped, but do not create files or launch agents.

### 2.3 `qiongli install` (Install bundled subject payload)

Use Case:
- Installs the subject payload bundled inside the PyPI package into global client skill directories.
- Defaults to `--subject core --coverage complete`; use `--subject economics`, `--subject political-economy`, or `--subject geoeconomics` for full Qiongli plus the selected subject specialization.

```bash
qiongli install \
  [--profile partial|full] \
  [--subject core|economics|accounting|business|finance|political-economy|geoeconomics|economics-accounting] \
  [--coverage complete|focused] \
  [--target codex|claude|antigravity|hermes|all] \
  [--surface skills|plugin|both] \
  [--mode copy|link] \
  [--project-dir <path>] \
  [--overwrite] \
  [--doctor] \
  [--dry-run]
```

Examples:

```bash
qiongli install --target all
qiongli install --subject economics --target all
qiongli install --subject accounting --target all
qiongli install --subject political-economy --target all
qiongli install --subject geoeconomics --target all
qiongli install --subject economics-accounting --target all
qiongli install --subject economics --coverage focused --target all
qiongli install --profile full --target codex --surface plugin
qiongli install --profile full --target all --surface plugin
qiongli install --profile full --target all --surface both
```

Subject packages are specialized installs, not reduced-quality cuts. Default install is `core/complete`. `--subject economics`, `--subject business`, `--subject finance`, `--subject political-economy`, and `--subject geoeconomics` mean complete specialized installs, not reduced packages. `--subject accounting` means `accounting/complete`, full framework plus accounting specialization. Focused coverage selects the subject profile set and active effective skills for deliberate slim installs and Desktop/Web ZIPs. Current official subjects are `core`, `economics`, `accounting`, `business`, `finance`, `political-economy`, `geoeconomics`, and the named composite `economics-accounting`; `political-economy` and `geoeconomics` are independent subject choices, not a composite. Official composites are not arbitrary comma-separated stacking. Public Desktop ZIP subjects are `core`, `economics`, `business`, `finance`, `political-economy`, `geoeconomics`, and `economics-accounting`, with no standalone accounting Desktop ZIP in this phase. Switch subjects or coverage by rerunning `install` or `upgrade` with new flags.

`--surface plugin` installs a local client-native plugin bundle backed by the full Python MCP server. This is next release or source checkout behavior until the next stable release is published; current stable v1.7.0 does not include this flag. For Codex, the CLI writes a personal marketplace entry and plugin `.mcp.json` that launches `qiongli mcp serve --transport stdio`. Marketplace-installed plugins stay on the lite no-Python path with the bundled Node literature provider.

### 2.4 `qiongli upgrade` (Download release & execute installers)

Use Case:
- Downloads the upstream release (defaults to latest tag `.tar.gz`).
- Extracts it and executes `scripts/install_qiongli.sh`.
- Defaults to refreshing global skill directories only; project assets are opt-in.

```bash
qiongli upgrade \
  [--repo <owner/repo|url>] \
  [--ref <tag-or-branch>] \
  [--ref-type tag|branch] \
  [--subject core|economics|accounting|business|finance|political-economy|geoeconomics|economics-accounting] \
  [--coverage complete|focused] \
  [--target codex|claude|antigravity|hermes|all] \
  [--project-dir <path>] \
  [--no-overwrite] \
  [--doctor] \
  [--dry-run]
```

Notes:
- `--project-dir` matters when you also request project-facing surfaces, such as `--parts project`.
- Default `upgrade` now behaves as a global refresh. Use `qiongli init --project-dir .` for project bootstrap, or `qiongli upgrade --parts project ...` when you explicitly want project files rewritten.
- `--subject` defaults to `core` and `--coverage` defaults to `complete`; use `--subject economics` for full Qiongli plus economics specialization, `--subject accounting` for full Qiongli plus accounting specialization, or add `--coverage focused` for the slim selected package.
- Example: `qiongli upgrade --subject accounting --target all`.
- After global install, `upgrade` creates workflow discovery symlinks under `~/.claude/commands/*.md` → enables direct `/paper`, `/lit-review`, etc. invocation in Claude Code.
- Shell CLI uses the bundled bootstrap helper and does not require Python.
- The command exits with the error code returned by the underlying installer.

### 2.5 `qiongli align` (Quick Reference Guide)

Use Case: Prints an overview of "what pipx installed / paths modified by upgrades / common commands".

```bash
qiongli align [--repo <owner/repo|url>]
```

### 2.6 `qiongli init` (Project Bootstrap)

Use Case: Creates project-local `.env` configuration in your project directory.

```bash
qiongli init [--project-dir <path>] [--target all|codex|claude|antigravity|hermes] [--dry-run]
```

Notes:
- Only creates project-facing assets (`.env`). Does not touch global skill directories.
- Safe to run multiple times; will not overwrite existing files unless `--overwrite` is passed.

### 2.7 `qiongli remove` (Remove CLI-installed assets)

Use Case: Removes assets installed by the CLI so you can switch cleanly between npm/PyPI/bootstrap installs and native marketplace plugins.

```bash
qiongli remove \
  [--target codex|claude|antigravity|hermes|all] \
  [--surface skills|plugin|both] \
  [--parts globals|project|cli] \
  [--project-dir <path>] \
  [--cli-dir <path>] \
  [--dry-run]
```

Examples:

```bash
qiongli remove --target all --dry-run
qiongli remove --target codex
qiongli remove --target codex --surface plugin
qiongli remove --parts globals,project --project-dir "$PWD"
qiongli remove --parts cli --cli-dir ~/.local/bin
qiongli uninstall --target all
qiongli delete --target claude
```

Notes:
- `remove` defaults to `--parts globals` and deletes CLI-installed `qiongli-workflow` skill directories plus generated workflow discovery links.
- It skips unmanaged `qiongli-workflow` directories that do not look like Qiongli package payloads.
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

  Formal research artifacts still belong under `RESEARCH/[topic]/...`. The first non-`off` task run automatically initializes `.qiongli/local_guidance.md` and `.qiongli/trace/` when they are missing. The orchestrator prompts runtime agents to create the required files; if an agent only returns text and does not write those files, the validator reports them as missing. Guidance trace bundles are written separately under `.qiongli/trace/runs/<run_id>/` so the run remains auditable even when formal outputs are incomplete.

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
  Project-local customization lives in `.qiongli/local_guidance.md`; run trace records live in `.qiongli/trace/index.jsonl` and `.qiongli/trace/runs/<run_id>/`. These files are intentionally separate from canonical workflow contracts, bundled skills, and release payloads.
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
