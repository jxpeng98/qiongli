# qiongli

`qiongli` is the npm/npx installer for the Qiongli academic workflow skills.

## Install

```bash
npm install -g qiongli
qiongli setup
qiongli install --target all
qiongli install --subject economics --target all
qiongli install --subject accounting --target all
qiongli install --subject business --target all
qiongli install --subject finance --target all
qiongli install --subject political-economy --target all
qiongli install --subject geoeconomics --target all
qiongli install --subject economics-accounting --target all
qiongli install --subject economics --coverage focused --target all
```

`qiongli setup` uses the bundled Python bridge and requires Python 3.12+ with `PyYAML`.
Use explicit `qiongli install ...` commands when you only need Node-based asset installation.

Update an existing global install with:

```bash
qiongli self-update --dry-run
qiongli self-update --yes
```

`qiongli self-update` delegates to `npm install -g qiongli@latest`, then refreshes the full local plugin/MCP surface with `qiongli install --target all --surface plugin --profile full --overwrite`. Use `--channel next` for the prerelease dist-tag. Marketplace plugins remain managed by the client plugin manager.

Or run without a global install:

```bash
npx qiongli@latest install --subject economics --target all
npx qiongli@latest install --subject economics --coverage focused --target all
```

For prerelease testing:

```bash
npx qiongli@next upgrade --subject economics --target all
```

Remove CLI-installed workflow assets before switching install channels:

```bash
qiongli remove --target all --dry-run
qiongli remove --target all
qiongli uninstall --target codex
```

`qiongli remove` deletes CLI-installed global skill assets and workflow discovery links only. It does not uninstall marketplace plugins such as `qiongli` or `qiongli-next`; remove those through the client/plugin manager that installed them.

The npm package contains pre-materialized `core`, `economics`, `accounting`, `business`, `finance`, `political-economy`, `geoeconomics`, and `economics-accounting` `qiongli-workflow` subject payloads in both `complete` and `focused` coverage. It does not depend on PyPI for skill installation and does not run `postinstall`.

`qiongli setup` is the interactive guided path for choosing install or upgrade, runtime surface, subject, coverage, `--mode copy|link`, install scope, CLI directory, `--overwrite` / `--no-overwrite`, optional upgrade source, literature provider setup, and doctor verification. Provider setup defaults to one local browser page with key-access guidance for OpenAlex, Semantic Scholar, Crossref, PubMed, and arXiv; use `--provider-mode prompt` for terminal-only entry or `--provider-mode skip` to bypass it. Each prompt includes a short `Tip:` comment. It delegates to the bundled Python bridge, so it requires Python 3.12+ with `PyYAML`. If you only need Node-based asset installation, use explicit `qiongli install ...` commands.

## Global-first update model

The npm package and the installed workflow assets are separate surfaces:

- `npm install -g qiongli@latest` updates the npm CLI and bundled payload in npm's global package location.
- `qiongli self-update --yes` performs that npm package update and then refreshes installed full local plugin/MCP surfaces from the new bundled payload.
- `qiongli install --target all` installs the default `core/complete` package into global AI client skill directories.
- `qiongli install --subject economics --target all` installs the full framework plus economics specialization.
- `qiongli install --subject accounting --target all` installs the full framework plus accounting specialization.
- `qiongli install --subject business --target all` installs the full framework plus business/management specialization.
- `qiongli install --subject finance --target all` installs the full framework plus finance specialization.
- `qiongli install --subject political-economy --target all` installs the full framework plus political economy specialization.
- `qiongli install --subject geoeconomics --target all` installs the full framework plus geoeconomics specialization.
- `qiongli install --subject economics --coverage focused --target all` installs the slimmer economics-focused package.
- `qiongli install --subject economics-accounting --target all` installs the official economics/accounting composite.
- `qiongli upgrade --subject accounting --target all` is the release-archive refresh path with overwrite enabled. Use it when you intentionally want to download an upstream GitHub release rather than updating the npm package first.
- `qiongli remove --target all` removes CLI-installed global workflow assets and generated discovery links while preserving marketplace plugins and unmanaged user files.
- Project directories are not required for normal install or upgrade. Use project paths only for commands that inspect or clean a specific project, such as `qiongli doctor --cwd .` or `qiongli clean --project-dir .`.

`--subject` defaults to `core`, and `--coverage` defaults to `complete`; the default install is `core/complete`. `--subject economics`, `--subject business`, `--subject finance`, `--subject political-economy`, and `--subject geoeconomics` mean complete specialized installs, not reduced packages. `--subject accounting` means `accounting/complete`, full framework plus accounting specialization. Use `--coverage focused` only when you deliberately want the slim selected subject package and the Desktop/Web ZIP shape. Current official subjects are `core`, `economics`, `accounting`, `business`, `finance`, `political-economy`, `geoeconomics`, and the named composite `economics-accounting`; composites are not arbitrary comma-separated stacking. Public Desktop ZIP subjects are `core`, `economics`, `business`, `finance`, `political-economy`, `geoeconomics`, and `economics-accounting`, with no standalone accounting Desktop ZIP in this phase. Subject packages are specialized installs, not reduced-quality cuts. Switch subjects or coverage by rerunning `install` or `upgrade` with new flags. `qiongli check --json` reports the bundled payload subject/coverage and installed target subject/coverage.

Global assets are written under client home directories such as:

```text
~/.codex/skills/qiongli-workflow
~/.claude/skills/qiongli-workflow
~/.gemini/antigravity/skills/qiongli-workflow
~/.hermes/skills/qiongli-workflow
```

Advanced bridge commands such as `setup`, `doctor`, `task-run`, and `team-run` use the Python runtime bundled in the npm package and require Python 3.12+ with `PyYAML`.

## MCP server

`qiongli mcp serve --transport stdio` is the unified full CLI MCP server. It exposes literature provider tools plus orchestrator and task-run tools from one server. The zero-dependency Node literature-provider MCP bundled by plugins remains a marketplace/MCPB fallback for environments that cannot run the full CLI.

The Python CLI path (`qiongli install --profile full --target codex`) performs managed Codex MCP registration for the unified server. The npm asset installer stays conservative: explicit npm installs can report MCP guidance with `qiongli install --parts globals,mcp --dry-run`, but they do not rewrite client MCP config.

The npm launcher also delegates MCP commands to the bundled Python bridge:

```bash
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

When agents are launched, the first non-`off` task run initializes `.qiongli/local_guidance.md` and `.qiongli/trace/` if needed. Formal task outputs still belong under `RESEARCH/[topic]/...`. Project-local guidance traces are written separately under `.qiongli/trace/` so missing formal outputs remain auditable without modifying bundled skills or workflow payloads.

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
