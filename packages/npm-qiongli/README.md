# qiongli

`qiongli` on npm is the Python-free asset manager for Qiongli client assets.

## Install

```bash
npm install -g qiongli
qiongli setup
qiongli install --target all --surface skills
qiongli check --json
qiongli project init --project-dir .
qiongli project set-subject finance --project-dir .
qiongli project status --project-dir .
```

`qiongli setup` is npm asset setup. It stays on the Python-free asset manager path and installs client assets only. For full runtime commands such as `doctor`, `mcp serve`, `provider setup`, `task-run`, or `customize`, install the full runtime first:

```bash
pipx install qiongli
```

Use explicit `qiongli install ...` commands when you only need scripted asset installation. For normal asset/project use, install once and keep per-project subject behavior in `.qiongli/guidance_manifest.yaml`.

Update an existing global install with:

```bash
qiongli update
qiongli refresh
qiongli upgrade --target all
```

`qiongli update`, `qiongli refresh`, and `qiongli upgrade` all reapply bundled assets from the currently installed npm package. `upgrade` is treated as an overwrite refresh. They do not update the npm package or the full Python CLI. Selected release archives, package self-update, and `qiongli self-update` require the full runtime: `pipx install qiongli`.

Or run without a global install:

```bash
npx qiongli@latest install --target all
npx qiongli@latest project status --project-dir .
```

For prerelease testing:

```bash
npx qiongli@next install --target all
```

Remove CLI-installed workflow assets before switching install channels:

```bash
qiongli remove --target all --dry-run
qiongli remove --target all
qiongli uninstall --target codex
```

`qiongli remove` deletes CLI-installed global skill assets and workflow discovery links only. It does not uninstall marketplace plugins such as `qiongli` or `qiongli-next`; remove those through the client/plugin manager that installed them.

The npm package contains pre-materialized `core`, `economics`, `accounting`, `business`, `finance`, `political-economy`, `geoeconomics`, and `economics-accounting` `qiongli-workflow` subject payloads in both `complete` and `focused` coverage. It does not depend on PyPI for skill installation and does not run `postinstall`.

`qiongli setup` in the npm package is client-asset setup only. It uses the same Python-free asset path as `qiongli install`, `qiongli update`, `qiongli refresh`, and `qiongli project ...`. The interactive setup wizard, provider setup, doctor checks, and full MCP/orchestrator commands require the full runtime: `pipx install qiongli`.

## Global-first update model

The npm package and the installed workflow assets are separate surfaces:

- `npm install -g qiongli@latest` updates the npm CLI and bundled payload in npm's global package location.
- `qiongli update`, `qiongli refresh`, and `qiongli upgrade` reapply bundled assets from the current npm package; they do not update the npm package.
- `qiongli install --target all` installs the stable skills surface used across projects. Plugin-lite output is opt-in with `--surface plugin` or `--surface both` where bundled and supported.
- `qiongli project init --project-dir .` creates `.qiongli/guidance_manifest.yaml` for a project.
- `qiongli project set-subject finance --project-dir .` changes ordinary project subject behavior without reinstalling a package.
- Missing `.qiongli/guidance_manifest.yaml` means implicit `active_subject: auto`: Qiongli uses core guidance, infers temporary subject or method lenses from the task, and writes auditable proposals before changing project-local state.
- `qiongli remove --target all` removes CLI-installed global workflow assets and generated discovery links while preserving marketplace plugins and unmanaged user files.
- Project directories are not required for normal install or upgrade. Use project paths only for commands that inspect or clean a specific project, such as `qiongli doctor --cwd .` or `qiongli clean --project-dir .`.

## Subject lifecycle controls

The npm package can inspect and edit the lightweight `qiongli project ...`
manifest, but subject lifecycle controls live in the full runtime: `pipx install qiongli`.

```bash
pipx install qiongli
qiongli subject confirm finance --cwd .
qiongli subject confirm finance --cwd . --propose-only --json
```

MCP clients that need `qiongli_subject_status` or `qiongli_subject_update`
should use the full runtime server. Read-only clients can call
`qiongli_subject_update` with `read_only: true` to export a proposed action
without writing `.qiongli` project files.

Advanced compatibility, Desktop ZIP, focused package, release payload, and install-surface testing examples:

```bash
qiongli install --subject economics --target all
qiongli install --subject accounting --target all
qiongli install --subject business --target all
qiongli install --subject finance --target all
qiongli install --subject political-economy --target all
qiongli install --subject geoeconomics --target all
qiongli install --subject economics --coverage focused --target all
qiongli install --subject economics-accounting --target all
qiongli upgrade --subject accounting --target all
```

`--subject` defaults to `core`, and `--coverage` defaults to `complete`, but subject install flags are advanced compatibility controls. `--subject economics`, `--subject business`, `--subject finance`, `--subject political-economy`, and `--subject geoeconomics` mean complete specialized installs, not reduced packages. `--subject accounting` means `accounting/complete`, full framework plus accounting specialization. Use `--coverage focused` only when you deliberately want the slim selected subject package and the Desktop/Web ZIP shape. Current official subjects are `core`, `economics`, `accounting`, `business`, `finance`, `political-economy`, `geoeconomics`, and the named composite `economics-accounting`; composites are not arbitrary comma-separated stacking. Public Desktop ZIP subjects are `core`, `economics`, `business`, `finance`, `political-economy`, `geoeconomics`, and `economics-accounting`, with no standalone accounting Desktop ZIP in this phase. Subject packages are specialized installs, not reduced-quality cuts. Ordinary project subject behavior changes with `qiongli project set-subject`; installed subject or coverage changes are only intentional specialized package refreshes. `qiongli check --json` reports the bundled payload subject/coverage and installed target subject/coverage.

Global assets are written under client home directories such as:

```text
~/.codex/skills/qiongli-workflow
~/.claude/skills/qiongli-workflow
~/.gemini/antigravity/skills/qiongli-workflow
~/.hermes/skills/qiongli-workflow
```

Transitional `python-runtime/` files still ship in the npm package for compatibility checks, but npm CLI dispatch stays on the Python-free asset path. Full runtime commands such as `doctor`, `task-run`, `team-run`, `mcp serve`, `provider setup`, and `customize` require `pipx install qiongli`.

## MCP server

`qiongli mcp serve --transport stdio` is the unified full CLI MCP server. It exposes literature provider tools plus orchestrator and task-run tools from one server. The zero-dependency Node literature-provider MCP bundled by plugins remains a marketplace/MCPB fallback for environments that do not need the full runtime.

The full runtime path (`pipx install qiongli`, then `qiongli install --profile full --target codex`) performs managed Codex MCP registration for the unified server. The npm asset installer stays conservative: explicit npm installs can report MCP guidance with `qiongli install --parts globals,mcp --dry-run`, but they do not run the full server or rewrite client MCP config.

Install the full runtime before running MCP commands:

```bash
pipx install qiongli
qiongli mcp serve --transport stdio
qiongli mcp doctor --json
qiongli mcp config example --target codex --json
qiongli mcp config example --target claude-code --json
qiongli mcp config example --target hermes --json
```

The full CLI MCP server exposes literature tools plus orchestrator tools:

- `qiongli_literature_status`
- `qiongli_literature_search`
- `qiongli_literature_export_evidence`
- `qiongli_orchestrator_doctor`
- `qiongli_orchestrator_route`
- `qiongli_task_plan`
- `qiongli_task_run`

`qiongli_task_run` defaults to preview mode and launches local Codex or Claude processes only when the MCP caller explicitly sends JSON boolean `run_agents: true`. It accepts `guidance_mode` (`off`, `read`, `propose`, or `apply`) for the project-local `.qiongli/` guidance layer. Preview mode echoes the selected task-run arguments and bootstrap status, but does not create formal `RESEARCH/[topic]/...` artifacts or `.qiongli/` files.

When agents are launched, project subject context comes from `.qiongli/guidance_manifest.yaml`; if it is missing, the effective subject is `active_subject: auto`. The first non-`off` task run initializes `.qiongli/local_guidance.md` and `.qiongli/trace/` if needed. Formal task outputs still belong under `RESEARCH/[topic]/...`. Project-local guidance traces are written separately under `.qiongli/trace/` so missing formal outputs remain auditable without modifying bundled skills or workflow payloads.

Use `stdio` when the desktop client can launch a local command. Use HTTP only for clients that require an endpoint:

```bash
qiongli mcp serve --transport http --host 127.0.0.1 --port 8765
```

Provider keys can be configured with `qiongli mcp configure ...`, `qiongli provider setup`, or the MCP tool `qiongli_configure_provider`. The setup page includes links and short steps for getting each supported key, and marks arXiv as available without an API key. Status and tool output are redacted; raw key values are not printed.

Runtime `--custom-dir` customization is not supported by npm in this phase. Use the source checkout when you need local custom overlays, profiles, or skills:

```bash
qiongli customize --subject economics --name my-econ-lab --out ./qiongli-custom/econ-lab
python3 scripts/materialize_subject_package.py --subject economics --custom-dir ./qiongli-custom/econ-lab --source . --out /tmp/qiongli-workflow
```

Custom overlays affect generated output only and do not rewrite canonical source files.
