# Install Qiongli

Qiongli has several installation surfaces because users need different levels of runtime control. Start with the smallest surface that gives you the workflow you need.

## Latest Stable Downloads

Current stable release: [v1.6.0](https://github.com/jxpeng98/qiongli/releases/tag/v1.6.0). These direct links cover the common install paths; use the download guide for subject-specific Desktop ZIPs and maintainer artifacts.

| Need | Link or command |
|---|---|
| npm CLI | [`qiongli@1.6.0`](https://www.npmjs.com/package/qiongli/v/1.6.0): `npm install -g qiongli@latest` |
| PyPI CLI | [`qiongli 1.6.0`](https://pypi.org/project/qiongli/1.6.0/): `pipx install qiongli` |
| Claude Desktop/Web core skill | [`qiongli-claude-desktop-skill-core-v1.6.0.zip`](https://github.com/jxpeng98/qiongli/releases/download/v1.6.0/qiongli-claude-desktop-skill-core-v1.6.0.zip) |
| Claude Desktop literature MCPB | [`qiongli-literature-provider-0.1.4.mcpb`](https://github.com/jxpeng98/qiongli/releases/download/v1.6.0/qiongli-literature-provider-0.1.4.mcpb) |
| Zotero Desktop companion | [`qiongli-zotero-companion-0.2.2.xpi`](https://github.com/jxpeng98/qiongli/releases/download/v1.6.0/qiongli-zotero-companion-0.2.2.xpi) |
| All release assets | [Download guide](https://github.com/jxpeng98/qiongli/releases/download/v1.6.0/qiongli-downloads-v1.6.0.md) and [GitHub Release](https://github.com/jxpeng98/qiongli/releases/tag/v1.6.0) |

## Install Surfaces

| Surface | Best for | Installs | Python required |
|---|---|---|---|
| Marketplace plugin / extension | One client, least setup, or no local CLI environment | Client plugin plus `qiongli-workflow`; Codex and Claude Code include the bundled Node literature MCP as a lite/no-CLI fallback | No for skill use or bundled literature MCP; Python/CLI only for full local Qiongli |
| Claude Desktop Skill ZIP | Claude Desktop or Claude.ai, especially when you do not want to use a code/CLI environment | Personal `qiongli` Skill upload | No |
| `qiongli install --profile full` | Full local Qiongli in Codex or another local client | Skills, shell CLI, provider config flow, doctor checks, and the unified full MCP server | Yes, Python 3.12+ |
| Bootstrap `partial` | Global workflow assets across clients | Skills and workflow discovery where supported | No |
| Bootstrap `full` | Runtime checks and orchestration from release scripts | `partial` plus shell CLI, MCP registration part, and `doctor` support | Yes, Python 3.12+ |
| npm / npx | Node-based automation | npm CLI plus bundled workflow payload | Only for advanced bridge commands |
| pipx / pip | Python updater CLI | Python CLI distribution | Yes |

The user-visible skill name is `qiongli`. The installed directory is still `qiongli-workflow` for compatibility with existing clients and release artifacts. `core` is the default subject, so the default install is `core/complete`. Specialized CLI/npm installs default to `coverage=complete`, meaning full Qiongli plus the requested subject specialization.

## Native Plugin And Extension

Use this when you only need Qiongli inside one supported client.

Codex installs through the shared [Skillsplace](https://github.com/jxpeng98/skillsplace) marketplace:

```bash
codex plugin marketplace add jxpeng98/skillsplace --ref main
codex plugin marketplace list
```

Then install or enable `qiongli` from the Codex plugin UI for the default core package. Subject entries such as `qiongli-economics`, `qiongli-accounting`, `qiongli-business`, `qiongli-finance`, `qiongli-political-economy`, `qiongli-geoeconomics`, and `qiongli-economics-accounting` install the corresponding `subject/complete` package from the same marketplace.

The Codex plugin bundles its MCP registration through `.mcp.json` and includes a zero-dependency Node literature-provider server under `mcp/qiongli-literature-provider/`. Codex users do not need to hand-write a separate MCP config or install the `qiongli` CLI for those bundled literature tools. Provider keys remain outside the plugin and can be configured with the bundled local setup tool `qiongli_configure_provider`, with `qiongli_save_provider_config`, or with `qiongli mcp configure` / `qiongli provider setup` when the CLI is installed. For full local Qiongli, the CLI full profile is canonical: it registers the Python-backed MCP server that exposes literature plus orchestrator tools from one `qiongli mcp serve --transport stdio` process.

Codex currently treats plugin-bundled MCP servers as plugin assets: the settings UI can enable the server and manage tool policy, but it is not the right place to add provider keys for this bundled server. Claude Desktop MCPB, Claude Code, Cursor-style clients, and other local stdio MCP clients should use the same Qiongli provider setup contract. Configure keys through the Qiongli provider config instead:

1. Ask Codex to run `qiongli_config_status` and note the redacted status plus `config_path`.
2. Ask the client to run `qiongli_configure_provider`, then open the returned `127.0.0.1` URL.
3. Enter the OpenAlex API key, optional OpenAlex email, and Semantic Scholar API key in the local browser page. The page writes the shared provider config without putting secrets in the conversation.
4. Re-run `qiongli_config_status` or `qiongli_literature_status`; credentials should be reported only as `configured` or `missing`, never printed in full.

Do not put provider keys in `.mcp.json`, `.codex-plugin/plugin.json`, release ZIPs, or research artifacts. The same shared provider config is read by the Codex plugin MCP, the Claude Code plugin MCP, the Claude Desktop MCPB, and the full CLI MCP server.

Claude Code uses the same Skillsplace catalog:

```bash
claude plugin marketplace add jxpeng98/skillsplace@main
claude plugin install qiongli@skillsplace
# Subject-specialized install:
claude plugin install qiongli-economics@skillsplace
```

Inside an interactive Claude Code session, use:

```text
/plugin marketplace add jxpeng98/skillsplace@main
/plugin install qiongli@skillsplace
/plugin install qiongli-economics@skillsplace
```

The Claude Code plugin also bundles the zero-dependency Node literature-provider MCP runtime under `mcp/qiongli-literature-provider/`, using the same provider, search, and status tools as the Codex plugin. It covers literature-provider MCP without installing the `qiongli` CLI. Full Python-backed tools, including `qiongli_literature_search`, `qiongli_orchestrator_route`, `qiongli_task_plan`, `qiongli_task_run`, and `qiongli_orchestrator_doctor`, require the npm, pipx/pip, or bootstrap `full` CLI runtime and `qiongli mcp serve --transport stdio`.

Claude Desktop and Claude.ai do not install third-party Claude Code plugin marketplaces. If you use Desktop or the web app and are not familiar with a code/CLI environment, use the release ZIP path instead. It requires no terminal commands:

1. Download `qiongli-claude-desktop-skill-core-<tag>.zip`, `qiongli-claude-desktop-skill-economics-<tag>.zip`, `qiongli-claude-desktop-skill-business-<tag>.zip`, `qiongli-claude-desktop-skill-finance-<tag>.zip`, `qiongli-claude-desktop-skill-political-economy-<tag>.zip`, `qiongli-claude-desktop-skill-geoeconomics-<tag>.zip`, or `qiongli-claude-desktop-skill-economics-accounting-<tag>.zip` from the GitHub Release assets. Public Desktop ZIP subjects in this phase are `core`, `economics`, `business`, `finance`, `political-economy`, `geoeconomics`, and `economics-accounting`; there is no standalone accounting Desktop ZIP yet.
2. In Claude Desktop, drag the ZIP into the Skills upload/install flow, or open `Customize > Skills`, click `+`, choose `Create skill`, then `Upload a skill`.
3. In Claude.ai, use the same `Customize > Skills` upload flow and select the same ZIP.
4. Enable the uploaded `qiongli` skill.

The release ZIP uses `coverage=focused` to stay under the current 180-file upload budget. It is a subject-specialized Desktop/Web package, not a reduced-quality cut. It preserves executable workflows, prompts, templates, standards, selected profiles, `skills-summary.md`, and `skills-core.md`; specialized ZIPs also include selected effective skill markdown generated with layered overlays. This Desktop skill ZIP is skill-only: it contains workflows/prompts/templates, stores no secrets, and does not execute provider calls. Detailed canonical source remains available through CLI/npm `coverage=complete`, the Codex / Claude Code plugin packages, and the source repository.

The Qiongli Literature Provider `.mcpb` (`qiongli-literature-provider.mcpb`) is a separate Claude Desktop local provider asset. It runs Desktop literature search through OpenAlex, Semantic Scholar, Crossref, and PubMed, exposes a Desktop configuration UI for provider credentials, and uses Claude Desktop sensitive-field handling instead of putting keys in the skill ZIP. It supports query variants, finance/economics deep-search routing, pagination, retry diagnostics, and limited citation/reference metadata expansion. It contains its own zero-dependency Node stdio server, so Desktop users do not need the `qiongli` CLI or an npm install to use this MCPB. CLI, Codex, and Claude Code users can still run `qiongli provider setup`, then verify `provider_connected` or `strategy_only` with `qiongli provider doctor`. Desktop users need the `qiongli-literature-provider` MCPB or platform-native search before claiming `provider_connected`; if no MCPB or platform-native search is available, record the run as `strategy_only` and treat platform search or user-supplied corpus as the evidence source. Finance/economics data APIs such as FRED and SEC EDGAR belong in a separate data MCP surface, documented in [Finance/Economics Data MCP Boundary](../advanced/finance-econ-data-mcp.md).

Manual Desktop installs can combine two local assets:

- Skill ZIP: enables Qiongli agent instructions, workflows, subject overlays, and skill guidance inside Claude Desktop/Web.
- Literature MCPB: enables local literature MCP calls and provider configuration.

Those two assets do not by themselves expose the full Python-backed orchestrator. If a local client should call `qiongli_orchestrator_route`, `qiongli_task_plan`, `qiongli_task_run`, or `qiongli_orchestrator_doctor` as MCP tools, install the npm, pipx/pip, or bootstrap `full` CLI runtime and configure:

```bash
qiongli install --profile full --target codex
qiongli mcp serve --transport stdio
qiongli mcp doctor --json
```

Use `qiongli_orchestrator_route` from Codex, Claude Code, Antigravity, or another local MCP client when deciding whether a request should move from skill-only execution to the full orchestrator. `qiongli_task_run` defaults to preview mode. It launches local runtime agents only when the MCP caller explicitly sends JSON boolean `run_agents: true` and the local runtime passes `doctor`.

## Use After Install

Restart the target client after installing or upgrading. Then use the entrypoint that client exposes:

| Client | Discovery | Invocation |
|---|---|---|
| Codex | `/skills` should list `qiongli` | `$qiongli <research task>` |
| Claude Code | Plugin UI, `/plugin`, or global command discovery | `/paper`, `/lit-review`, `/paper-write`, `/code-build` |
| Shell | `qiongli check` | `qiongli doctor`, `qiongli upgrade`, `python3 -m bridges.orchestrator ...` |

Codex does not expose a custom `/qiongli` slash command. Use `/skills` to confirm the skill exists, then invoke `$qiongli`.

## Bootstrap Partial

Use `partial` for the cross-client workflow package without Python:

```bash
curl -fsSL https://raw.githubusercontent.com/jxpeng98/qiongli/main/scripts/bootstrap_qiongli.sh | bash -s -- --profile partial --project-dir "$PWD" --target all
```

Windows PowerShell 7+:

```powershell
winget install --id Microsoft.PowerShell --source winget
Invoke-WebRequest https://raw.githubusercontent.com/jxpeng98/qiongli/main/scripts/bootstrap_qiongli.ps1 -OutFile .\bootstrap_qiongli.ps1
pwsh -ExecutionPolicy Bypass -File .\bootstrap_qiongli.ps1 -Profile partial -ProjectDir "$PWD" -Target all
```

`partial` installs workflow assets and discovery links. It does not require Python and does not run full runtime validation.

## Bootstrap Full

Use `full` when you need local validation or orchestrated task execution:

```bash
curl -fsSL https://raw.githubusercontent.com/jxpeng98/qiongli/main/scripts/bootstrap_qiongli.sh | bash -s -- --profile full --project-dir "$PWD" --target all
```

Windows PowerShell 7+:

```powershell
pwsh -ExecutionPolicy Bypass -File .\bootstrap_qiongli.ps1 -Profile full -ProjectDir "$PWD" -Target all
```

`full` requires Python 3.12+ to already be on `PATH`. It does not install Python or `mise`.

After `full`, check a workspace:

```bash
qiongli doctor --project-dir .
python3 -m bridges.orchestrator doctor --cwd .
qiongli mcp doctor --json
```

## npm / npx

Use npm when you want a Node-distributed installer with the workflow payload bundled:

```bash
npm install -g qiongli
qiongli install --subject core --target all --project-dir "$PWD"
qiongli install --subject economics --target all --project-dir "$PWD"
qiongli install --subject accounting --target all --project-dir "$PWD"
qiongli install --subject economics-accounting --target all --project-dir "$PWD"
qiongli install --subject economics --coverage focused --target all --project-dir "$PWD"
```

For one-off runs:

```bash
npx qiongli@latest install --subject economics --target all --project-dir "$PWD"
npx qiongli@latest install --subject economics --coverage focused --target all --project-dir "$PWD"
npx qiongli@latest check --json
```

Prerelease testing remains available through the `next` dist-tag:

```bash
npx qiongli@next install --subject economics --target all --project-dir "$PWD"
```

Remove CLI-installed assets before switching fully to marketplace plugins:

```bash
qiongli remove --target all --dry-run
qiongli remove --target all
```

`qiongli remove` only removes CLI-installed global workflow assets and discovery links. Native marketplace plugins remain managed by the client/plugin manager that installed them.

## Recommended CLI Setup Wizard

After installing the CLI with npm, pipx, pip, or the bootstrap script, run the interactive setup wizard before hand-writing install flags:

```bash
qiongli setup
qiongli setup --dry-run
qiongli setup --project-dir "$PWD" --no-doctor
```

The wizard guides CLI, Codex, Claude Code, and Antigravity users through:

- setup path: `install` for first-time bundled asset installation, or `upgrade` for an upstream refresh
- runtime surface: CLI, Codex, Claude Code, Antigravity, or multi-platform
- subject choice
- coverage choice: `complete` or `focused`
- install mode: `--mode copy` for normal use, or `--mode link` for local checkout development
- install scope: `all`, `globals`, `project`, or `cli`
- shell CLI directory when CLI wrappers are enabled
- overwrite policy: `--overwrite` for replacing managed installs, or `--no-overwrite` on upgrade when you want to preserve existing managed files
- upgrade source: latest stable, latest beta, an explicit `--ref` tag, an explicit `--ref-type branch`, and optional `--repo`
- optional literature provider key setup
- doctor verification, unless `--no-doctor` is set

Every prompt prints a short `Tip:` comment explaining what the choice changes, so new users can follow the install or upgrade path without knowing the full CLI flag set first.

Provider keys entered through setup use the same provider config as `qiongli provider setup` and `qiongli provider doctor`. Secrets are stored outside generated research artifacts. The provider step configures credentials and runs doctor/capability checks; it does not guarantee external search results.

On npm installs, `qiongli setup` delegates to the bundled Python bridge and therefore requires Python 3.12+ plus `PyYAML`. Use explicit `qiongli install ...` commands when you want the Node-only asset installer.

## pipx / pip

Use pipx when you specifically want the Python-distributed updater CLI:

```bash
pipx install qiongli
qiongli setup
qiongli install --subject economics --target all
qiongli install --subject accounting --target all
qiongli install --subject political-economy --target all
qiongli install --subject geoeconomics --target all
qiongli install --subject economics-accounting --target all
```

`qiongli setup` can guide the same choices interactively. Scriptable installs can still use `qiongli upgrade` or explicit `qiongli install ...` commands as shown here.

Upgrade it with:

```bash
pipx upgrade qiongli
qiongli upgrade --subject accounting --target all --doctor --project-dir /path/to/project
```

`--subject` defaults to `core`, and `--coverage` defaults to `complete`. Use complete when you are unsure: `--subject economics`, `--subject business`, `--subject finance`, `--subject political-economy`, and `--subject geoeconomics` mean complete specialized installs, not reduced packages, and `--subject accounting` means `accounting/complete`, full framework plus accounting specialization. Use `--coverage focused` for deliberate slim installs and Desktop/Web-equivalent packages. Current official subjects are `core`, `economics`, `accounting`, `business`, `finance`, `political-economy`, `geoeconomics`, and the named composite `economics-accounting`; `political-economy` and `geoeconomics` are independent subject choices, not a composite. Official composite subjects are not arbitrary comma-separated stacking. To switch a client from one subject or coverage to another, rerun `install` or `upgrade` with new flags. `qiongli check --json` reports the active installed subject and coverage per target; legacy installs without a `SUBJECT_MANIFEST.json` or `SUBJECT` file are treated as `core` / `complete`.

Create a custom scaffold before materializing local overlays:

```bash
qiongli customize --subject economics --name my-econ-lab --out ./qiongli-custom/econ-lab
```

Local custom overlays are supported by the source materializer:

```bash
python3 scripts/materialize_subject_package.py \
  --subject economics \
  --coverage complete \
  --source . \
  --custom-dir /path/to/custom-qiongli \
  --out /tmp/qiongli-workflow
```

Use this when you need local overlays, profiles, registry entries, or custom skill markdown. Custom overlays affect generated output only and do not rewrite canonical source files. `qiongli customize` plus `--custom-dir` materialization is for the Python/source checkout workflow; npm runtime installs pre-generated payloads and do not accept `--custom-dir` in this phase.

## What Gets Installed

Depending on the surface, Qiongli may install:

- `qiongli-workflow` skill assets under client home directories, visible to users as `qiongli`
- workflow command discovery links such as `/paper`, `/lit-review`, `/paper-write`, and `/code-build` in clients that support that discovery model
- shell commands `qiongli`, `ql`, and compatibility aliases `research-skills`, `rsk`, `rsw`
- optional project integration files when you explicitly run `qiongli init --project-dir .`

Project-local files are not written by default. The global workflow package can be used from any research workspace.

For invocation details, see [Using Agent Skills](/guide/using-agent-skills).

## Keep Versions Aligned

If you use multiple surfaces, keep plugin, global skill assets, npm payload, and Python CLI aligned:

```bash
qiongli check
qiongli upgrade --subject core --target all
```

If you move fully to native plugins and no longer need legacy global skill directories or slash discovery, inspect cleanup first:

```bash
qiongli remove --target all --dry-run
```
