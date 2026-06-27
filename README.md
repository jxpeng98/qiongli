<div align="center">
  <h1>Qiongli (穷理)</h1>
  <p><strong>Reviewable academic workflows for Codex, Claude Code, Claude Desktop, Antigravity, and Hermes.</strong></p>
  <p>Turn broad research requests into task IDs, quality gates, evidence trails, role handoffs, and durable outputs under <code>RESEARCH/[topic]/</code>.</p>
  <p>
    <a href="https://www.npmjs.com/package/qiongli"><img alt="npm latest version" src="https://img.shields.io/npm/v/qiongli/latest?style=flat-square&amp;logo=npm&amp;label=npm%20latest"></a>
    <a href="https://www.npmjs.com/package/qiongli?activeTab=versions"><img alt="npm next version" src="https://img.shields.io/npm/v/qiongli/next?style=flat-square&amp;logo=npm&amp;label=npm%20next&amp;color=cb3837"></a>
    <a href="https://pypi.org/project/qiongli/"><img alt="PyPI latest version" src="https://img.shields.io/pypi/v/qiongli?style=flat-square&amp;logo=pypi&amp;label=PyPI%20latest"></a>
  </p>
  <p>
    <a href="README_CN.md">中文 README</a> ·
    <a href="docs/index.md">Docs</a> ·
    <a href="docs/zh/index.md">中文文档</a> ·
    <a href="docs/quickstart.md">Quickstart</a> ·
    <a href="docs/guide/install.md">Install</a> ·
    <a href="docs/reference/cli.md">CLI</a>
  </p>
</div>

## What It Is

Qiongli is a portable academic workflow package plus optional local runtimes. It helps research teams:

- choose the right paper route for empirical, qualitative, systematic review, RCT, theory, and code-first methods work;
- keep literature search, citation risk, methods, writing, and review steps tied to explicit evidence;
- run solo, duo, or triad agent workflows with auditable handoffs and verification status;
- keep full local orchestration separate from lightweight skill/plugin installs.

The name comes from `穷理`: keep asking what principle, evidence, and limit sits underneath a claim.

## Start Here

| Need | Best entry |
|---|---|
| Browse the full documentation | [VitePress Docs](docs/index.md), or run `npm run docs:dev` |
| Read in Chinese | [中文 README](README_CN.md) or [中文文档](docs/zh/index.md) |
| Install Qiongli in one client | [Install Guide](docs/guide/install.md) |
| Get from zero to a first workspace | [Quickstart](docs/quickstart.md) |
| Decide which paper workflow to use | [Task Recipes](docs/guide/task-recipes.md) |
| Use CLI commands, aliases, JSON checks, or automation | [CLI Reference](docs/reference/cli.md) |
| Understand the runtime and package model | [Architecture](docs/architecture.md) |

## Latest Stable Downloads

Current stable release: [v1.12.0](https://github.com/jxpeng98/qiongli/releases/tag/v1.12.0). These direct links cover the common install paths; use the download guide for subject-specific Desktop ZIPs and maintainer artifacts.

| Need | Link or command |
|---|---|
| npm CLI | [`qiongli@1.12.0`](https://www.npmjs.com/package/qiongli/v/1.12.0): `npm install -g qiongli@latest` |
| PyPI CLI | [`qiongli 1.12.0`](https://pypi.org/project/qiongli/1.12.0/): `pipx install qiongli` |
| Claude Desktop/Web core skill | [`qiongli-claude-desktop-skill-core-v1.12.0.zip`](https://github.com/jxpeng98/qiongli/releases/download/v1.12.0/qiongli-claude-desktop-skill-core-v1.12.0.zip) |
| Claude Desktop literature MCPB | [`qiongli-literature-provider-0.1.5.mcpb`](https://github.com/jxpeng98/qiongli/releases/download/v1.12.0/qiongli-literature-provider-0.1.5.mcpb) |
| Zotero Desktop companion | [`qiongli-zotero-companion-0.2.2.xpi`](https://github.com/jxpeng98/qiongli/releases/download/v1.12.0/qiongli-zotero-companion-0.2.2.xpi) |
| All release assets | [Download guide](https://github.com/jxpeng98/qiongli/releases/download/v1.12.0/qiongli-downloads-v1.12.0.md) and [GitHub Release](https://github.com/jxpeng98/qiongli/releases/tag/v1.12.0) |

## Install Fast

The default CLI install now prepares the full local plugin surface for supported clients:

```bash
npm install -g qiongli
qiongli install --target all
qiongli check --offline
```

For scripted installs, keep the project directory explicit:

```bash
qiongli install --target all --project-dir "$PWD"
```

Use project-local subject guidance instead of reinstalling packages for every topic:

```bash
qiongli project init --project-dir "$PWD"
qiongli project set-subject finance --project-dir "$PWD"
qiongli project status --project-dir "$PWD"
```

For skill-only or no-Python paths, use the install guide. It covers Codex and Claude Code marketplace plugins, Claude Desktop Skill ZIPs, the literature MCPB, bootstrap partial/full, npm/npx, pipx, and pip.

## Recommended CLI Setup Wizard

Use the wizard when you want the CLI to help choose an install and upgrade path:

```bash
qiongli setup
qiongli setup --dry-run
qiongli setup --project-dir "$PWD" --no-doctor
```

It covers runtime surface, subject, coverage, `--mode copy|link`, shell CLI / CLI directory choices, `--overwrite` / `--no-overwrite`, optional provider config, and doctor verification. On npm installs, `qiongli setup` delegates through the bundled Python bridge and requires Python 3.12+ plus `PyYAML`. If you only need scriptable asset installation, run `qiongli install ...` directly.

## Update Or Refresh

`qiongli update` updates the installed CLI package first, then asks whether to refresh installed local plugins/assets from the new package:

```bash
qiongli update
qiongli update --yes
qiongli update --no-refresh
```

`qiongli upgrade` is different: it refreshes local content/assets from the current package or a selected release archive. It does not upgrade the npm, pipx, or pip package.

```bash
qiongli upgrade --ref v1.11.0 --target all
```

## Runtime Boundary

Installing Qiongli assets is intentionally lighter than running full orchestration.

| Surface | Use it for | Needs Python/model CLIs? |
|---|---|---|
| Skill or plugin package | prompts, task routes, templates, standards, subject overlays | No |
| Literature MCPB / bundled literature MCP | provider status, local search, evidence export | No Python |
| Full local plugin or CLI MCP | `doctor`, provider config, `task-plan`, `task-run`, orchestrator tools | Yes |
| Shell/Python CLI | validators, release checks, local orchestration, package maintenance | Yes |

Actual agent execution starts only when the runtime is configured and an execution command explicitly enables it. Previews and checks are designed to be inspectable before side effects.

## Research Boundaries

Qiongli includes the Academic Idea Funnel and Academic Grill Loop as an academic adaptation of Matt Pocock's `grill-me` idea-discovery pattern. It is tuned for academic idea-discovery, so it asks about evidence, rival explanations, feasibility, venue fit, and boundary review before drafting.

Provider credentials stay in provider config, not generated skill bundles. Use `qiongli provider setup` for OpenAlex and Semantic Scholar, then `qiongli provider doctor` to verify. The `qiongli-literature-provider` `.mcpb` exposes `qiongli_config_status`, `qiongli_configure_provider`, and `qiongli_save_provider_config` for Codex/Desktop flows; statuses include `provider_connected` and `strategy_only`. Skill-only installs can still use strategy fallback, and runtime checks keep a 180-second ceiling for external provider probes.

## Documentation Map

- [Guide](docs/guide/index.md): install, usage, upgrade, troubleshooting, and runtime choices.
- [Quickstart](docs/quickstart.md): smallest install surface and first research route.
- [Using Agent Skills](docs/guide/using-agent-skills.md): what to type in Codex, Claude Code, Antigravity, Hermes, and shell.
- [Task Recipes](docs/guide/task-recipes.md): scenario-based paper routes.
- [Reference](docs/reference/index.md): CLI behavior and skill catalog.
- [Advanced](docs/advanced/index.md): MCP providers, Zotero, subject packaging, and plugin-first distribution.
- [Maintainer](docs/maintainer/index.md): release policy, naming policy, and contributor guidance.

## Development

Common checks:

```bash
python3 -m unittest tests.test_self_update tests.test_cli tests.test_cli_setup_docs
python3 -m unittest tests.test_materialize_distribution_payloads tests.test_npm_package_contract
npm --prefix packages/npm-qiongli test
npm run docs:build
git diff --check
```

Maintainer contract anchors:

- The canonical contract lives with the workflow standards; packaged installs expose `standards/research-workflow-contract.yaml` and `standards/mcp-agent-capability-map.yaml`.
- Run `python3 scripts/validate_research_standard.py --strict` before release-facing changes.
- Subject package changes must pass staged materialization and npm package contract tests, including `tests.test_materialize_distribution_payloads` and `tests.test_npm_package_contract`.
- Agent routing details live in [Agent-Skill Collaboration](docs/advanced/agent-skill-collaboration.md).
- The legacy shell installer remains at `scripts/install_qiongli.sh`; most users should prefer the install guide or `qiongli install`.

Routine releases go through:

```bash
./scripts/release_automation.sh publish --version <version> --from-tag <previous-tag>
```

## Credit

Qiongli adapts useful workflow ideas from strict agent planning/review systems, Claude skill packaging, and academic review practices. Thanks to the [linux.do](https://linux.do/) community for practical AI tooling discussion and feedback.
