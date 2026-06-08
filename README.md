<div align="center">
  <h1>Qiongli (穷理)</h1>
  <p><strong>Contract-driven academic workflows for Codex, Claude Code, Claude Desktop, and Gemini.</strong></p>
  <p>Plan papers, run literature work, draft manuscripts, execute research code, and audit evidence through one canonical task contract.</p>
  <p>
    <a href="docs/quickstart.md">Quick Start</a> ·
    <a href="docs/guide/install.md">Install</a> ·
    <a href="docs/guide/using-agent-skills.md">Use Skills</a> ·
    <a href="docs/guide/task-recipes.md">Task Recipes</a> ·
    <a href="docs/reference/cli.md">CLI</a> ·
    <a href="docs/architecture.md">Architecture</a>
  </p>
</div>

## What Qiongli Is

Qiongli turns academic work into explicit, reviewable task flows. Instead of asking an agent to improvise a paper end to end, each run is tied to Task IDs, quality gates, role handoffs, and output paths under `RESEARCH/[topic]/`.

Use it for:

- **Paper workflows:** empirical, qualitative, systematic review, RCT/preregistration, theory, and code-first methods routes.
- **Literature rigor:** provider-aware search planning, search diagnostics, materialized search bundles, dedup logs, screening readiness, and snowball readiness.
- **Writing integrity:** claim-evidence mapping, citation risk checks, figures/tables planning, limitations review, proofreading, and rebuttal preparation.
- **Research code discipline:** strict Stage-I `I5 -> I6 -> I7 -> I8` specification, planning, execution, and review artifacts.
- **Multi-agent review:** Codex, Claude, and Gemini orchestration with solo, duo, and triad modes, explicit handoffs, disagreement records, and verification status.

The public name is **Qiongli**, from the Chinese `穷理`: to pursue the underlying principle of a question until its logic, evidence, and limits are clear. The full methodology name is **Qiongli Zhengche** (`穷理证澈`): make evidence chains, citation risk, assumptions, and claim boundaries transparent enough to audit.

## Current Structure

Qiongli now has four deliberately separate surfaces:

| Surface | What it provides | Does it launch local agents? |
|---|---|---|
| Skill / plugin package | Agent instructions, workflow commands, templates, standards, subject overlays, and effective skill markdown | No |
| Literature MCP runtime | Local literature/provider tools such as `qiongli_literature_search`, `qiongli_config_status`, and `qiongli_save_provider_config` | No |
| Full CLI MCP runtime | Python-backed MCP tools including `qiongli_orchestrator_doctor`, `qiongli_task_plan`, and `qiongli_task_run` | Only when explicitly enabled |
| Shell / Python orchestrator | `doctor`, validators, `task-plan`, `task-run`, `team-run`, and code-build routes | Yes, when runtime auth is configured |

This separation matters for Desktop users. A manual Skill ZIP install gives Claude Desktop/Web the Qiongli skill and subject overlays. A `.mcpb` install gives Desktop literature MCP calls. The full local agent runtime is a separate CLI/MCP surface.

## Installation Decision Guide

Start with the smallest surface that matches the job. "Full workflow" means the complete Qiongli methodology, workflows, templates, quality gates, skill registry, and subject overlays. It does not automatically mean local provider calls or local agent execution.

| Install path | Use it when | Advantages | Trade-offs |
|---|---|---|---|
| **Codex marketplace plugin**: `codex plugin marketplace add jxpeng98/skillsplace --ref main` | You use Codex and want Qiongli as a native skill/plugin. | Installs the Qiongli skill, subject packages such as `qiongli-economics`, and the bundled zero-dependency literature MCP registration/runtime. No Python needed for skill use or bundled literature MCP. | Full Python-backed orchestrator MCP still requires npm/pipx/bootstrap `full` plus `qiongli mcp serve --transport stdio`. |
| **Claude Code marketplace plugin**: `claude plugin marketplace add jxpeng98/skillsplace@main` | You use Claude Code and want the full Qiongli workflow from the marketplace. | Installs the full `subject/complete` workflow package for `qiongli` and subject entries such as `qiongli-economics@skillsplace`; includes slash workflow commands like `/paper`, `/lit-review`, and `/code-build`, plus the same zero-dependency Node literature MCP runtime as Codex for provider, search, and status tools. No Python needed for skill/command use or bundled literature MCP. | Full Python-backed tools such as `qiongli_task_run` still require npm/pipx/bootstrap `full` plus `qiongli mcp serve --transport stdio`. |
| **Claude Desktop/Web Skill ZIP** | You want Qiongli in Claude Desktop or Claude.ai without a code environment. | No terminal required. Good for skill-guided paper planning, writing, review, and focused subject packages. | Focused package kept under Desktop upload limits; skill-only, no secrets, no provider calls, no local agent execution. |
| **Claude Desktop Literature MCPB**: `qiongli-literature-provider.mcpb` | Desktop needs local OpenAlex/Semantic Scholar search and provider key configuration. | No Python or npm install; pairs cleanly with the Desktop Skill ZIP. | Literature/provider tools only. It does not install Qiongli skills and does not launch orchestrator agents. |
| **npm / npx**: `npm install -g qiongli` or `npx qiongli@latest ...` | You want scriptable installs, upgrades, and prebuilt complete/focused subject payloads through Node. | Good default for cross-client asset installation; no PyPI dependency for skill payloads. | Advanced bridge commands such as `setup`, `doctor`, `task-run`, and `mcp` need Python 3.12+ with `PyYAML`. |
| **Bootstrap `partial`** | You want portable workflow assets and command discovery across clients without Python. | Simple shell/PowerShell path for skills and workflow discovery links. | No runtime validation, no Python bridge, no local orchestrator execution. |
| **Bootstrap `full` / pipx / pip Python CLI** | You need `doctor`, validators, local `task-plan`, `task-run`, `team-run`, or full CLI MCP. | Most complete runtime surface; enables local checks and Python-backed orchestration. | Requires Python 3.12+, model CLIs in `PATH`, and runtime auth for actual agent execution. |

Claude Code marketplace status: yes, Claude Code can install the full Qiongli methodology through Skillsplace for core and subject `complete` packages. Codex and Claude Code both install the skill/command package plus the bundled zero-dependency Node literature MCP runtime for provider, search, and status tools. The full Python-backed orchestration MCP remains separate and still requires npm/pipx/bootstrap `full` plus `qiongli mcp serve --transport stdio`.

Prerelease testing uses the separate `qiongli-next` marketplace entry. It installs only the core Qiongli workflow for Codex and Claude Code, keeps the bundled literature MCP runtime, and pairs with `qiongli-next-claude-desktop-skill-core-<tag>.zip` plus `qiongli-literature-provider-<version>.mcpb` for Claude Desktop. CLI prerelease testing uses `npx qiongli@next ...`.

Detailed install instructions live in [docs/guide/install.md](docs/guide/install.md). The quickest route from nothing installed is [docs/quickstart.md](docs/quickstart.md).

## Install Snippets

Native plugin installs:

```bash
# Codex
codex plugin marketplace add jxpeng98/skillsplace --ref main
codex plugin marketplace list

# Claude Code
claude plugin marketplace add jxpeng98/skillsplace@main
claude plugin install qiongli@skillsplace
claude plugin install qiongli-economics@skillsplace

# Prerelease channel
claude plugin install qiongli-next@skillsplace
```

npm / npx installs:

```bash
npm install -g qiongli
qiongli install --target all --project-dir "$PWD"
qiongli install --subject economics --target all --project-dir "$PWD"
qiongli install --subject accounting --target all --project-dir "$PWD"
qiongli install --subject economics-accounting --target all --project-dir "$PWD"
qiongli install --subject economics --coverage focused --target all --project-dir "$PWD"

# Prerelease testing
npx qiongli@next install --target all --project-dir "$PWD"
npx qiongli@next check --json
```

Bootstrap installs:

```bash
# Linux / macOS
curl -fsSL https://raw.githubusercontent.com/jxpeng98/qiongli/main/scripts/bootstrap_qiongli.sh | bash -s -- --profile partial --project-dir "$PWD" --target all
curl -fsSL https://raw.githubusercontent.com/jxpeng98/qiongli/main/scripts/bootstrap_qiongli.sh | bash -s -- --profile full --project-dir "$PWD" --target all
```

```powershell
# Windows PowerShell 7+
pwsh -ExecutionPolicy Bypass -File .\bootstrap_qiongli.ps1 -Profile partial -ProjectDir "$PWD" -Target all
pwsh -ExecutionPolicy Bypass -File .\bootstrap_qiongli.ps1 -Profile full -ProjectDir "$PWD" -Target all
```

Use `partial` for skills and workflow discovery. Use `full` when you also need the shell CLI, `doctor`, validators, or local orchestrator execution.

## Recommended CLI Setup Wizard

After npm, pipx, pip, or bootstrap installs, use the setup wizard when you want help choosing an install or upgrade path:

```bash
qiongli setup
qiongli setup --dry-run
qiongli setup --project-dir "$PWD" --no-doctor
qiongli install --target all --project-dir "$PWD"
```

The wizard covers `install` and `upgrade`, runtime surface (`cli`, `codex`, `claude-code`, or `multi-platform`), subject, coverage, `--mode copy|link`, install scope, CLI directory / shell CLI location, `--overwrite` / `--no-overwrite`, optional provider config, and doctor verification unless `--no-doctor` is used.

On npm installs, `qiongli setup` delegates to the bundled Python bridge and therefore requires Python 3.12+ plus `PyYAML`. If you only need Node-based asset installation, use explicit `qiongli install ...` commands.

Provider keys entered through `qiongli setup` use the same provider config as `qiongli provider setup` and `qiongli provider doctor`. Secrets are stored in provider configuration outside generated research artifacts. Setup configures credentials and runs doctor/capability checks; it does not by itself guarantee external search results.

## Manual Desktop And MCP Boundaries

Claude Desktop / Claude.ai can use Qiongli without a terminal:

1. Download a release Skill ZIP such as `qiongli-claude-desktop-skill-core-<tag>.zip` or `qiongli-claude-desktop-skill-economics-<tag>.zip`.
2. Upload it through the Desktop/Web Skills flow.
3. Enable the uploaded `qiongli` skill.

The Desktop/Web ZIP uses `coverage=focused` to stay under the current 180-file upload budget. It is subject-specialized, not lower quality. It preserves workflows, prompts, templates, standards, selected profiles, `skills-summary.md`, `skills-core.md`, and selected effective skill markdown generated with layered overlays.

This Desktop skill ZIP is **skill-only**: it stores no secrets and does not execute provider calls. To add local Desktop literature tools, install the separate Qiongli Literature Provider `.mcpb`:

```text
qiongli-literature-provider.mcpb
```

The MCPB runs a zero-dependency Node stdio server for OpenAlex and Semantic Scholar search, provider status, and provider key saving. Desktop users need `qiongli-literature-provider` MCPB or platform-native search before claiming `provider_connected`; otherwise record the run as `strategy_only` and treat platform search or a user-supplied corpus as the evidence source.

The MCPB does not launch orchestrator agents. To expose the full Python-backed agent runtime through MCP, install the npm, pipx/pip, or bootstrap `full` CLI runtime and configure:

```bash
qiongli mcp serve --transport stdio
qiongli mcp config example --target codex --json
qiongli mcp config example --target claude-code --json
```

The full CLI MCP server exposes:

- `qiongli_config_status`, `qiongli_save_provider_config`, and `qiongli_collect_evidence`
- `qiongli_orchestrator_doctor`
- `qiongli_task_plan`
- `qiongli_task_run`

`qiongli_task_run` defaults to preview mode. It launches local Codex, Claude, or Gemini processes only when the MCP caller explicitly sends JSON boolean `run_agents: true` and the local runtime passes `doctor`.

See [Cross-Platform MCP Server](docs/advanced/cross-platform-mcp.md) and [MCP Providers Setup](docs/advanced/mcp-providers-setup.md).

## Subject Packages And Overlays

The user-visible skill name is `qiongli`. The installed directory remains `qiongli-workflow` for compatibility with existing clients and release artifacts.

Current official subjects:

- `core`
- `economics`
- `accounting`
- `business`
- `finance`
- `political-economy`
- `geoeconomics`
- `economics-accounting`

Default install means `core/complete`. CLI/npm specialized installs default to `coverage=complete`, so `--subject economics` means the full framework plus economics specialization, not a reduced package. Use `--coverage focused` only for deliberate slim packages and Desktop/Web ZIP-equivalent packages.

Subject specialization is layered:

- `core` owns shared workflow contracts, generic skills, templates, standards, and quality gates.
- Subject packages add discipline depth through selected profiles, append overlays, declared section replacements, and subject-specific skills.
- Effective packages are generated from `skill_refs`, subject overlays, layered section overrides, and optional local custom overlays.
- Generic skill source files are not duplicated.

Public Desktop ZIP subjects in this phase are `core`, `economics`, `business`, `finance`, `political-economy`, `geoeconomics`, and `economics-accounting`; there is no standalone accounting Desktop ZIP yet.

Local customization is available for source/Python workflows:

```bash
qiongli customize --subject economics --name my-econ-lab --out ./qiongli-custom/econ-lab
python3 scripts/materialize_subject_package.py --subject economics --custom-dir ./qiongli-custom/econ-lab --source . --out /tmp/qiongli-workflow
```

npm runtime installs use pre-generated payloads and do not accept runtime `--custom-dir` in this phase.

See [Subject Packaging Model](docs/advanced/subject-packaging-model.md).

## Workflow Runtime

Qiongli writes durable research artifacts under `RESEARCH/[topic]/`. The most common runtime checks are:

```bash
python3 -m bridges.orchestrator doctor --cwd .
python3 -m bridges.orchestrator task-plan --task-id F3 --paper-type empirical --topic ai-in-education --cwd .
python3 -m bridges.orchestrator task-run --task-id F3 --paper-type empirical --topic ai-in-education --cwd .
```

Useful controls:

- `--execution-mode solo|duo|triad`
- `--controller codex|claude|gemini`
- `--primary`, `--reviewer`, and `--verifier`
- `--solo-role-gates strict|standard|off`
- `--mcp-strict` and `--skills-strict`
- `--research-depth deep`
- `--only-target <id>`

Full functionality requires a real Python runtime plus model CLIs in `PATH`: `python3`, `codex`, `claude`, and `gemini`. You also need runtime authentication. `codex` can run with `OPENAI_API_KEY` or an existing ChatGPT/Codex login, `claude` uses `ANTHROPIC_API_KEY`, and Gemini direct mode requires non-interactive auth such as `GEMINI_API_KEY` or Vertex env auth. Google-login-only Gemini automation should use the resident broker path described in [docs/guide/multi-agent.md](docs/guide/multi-agent.md).

Without these runtime pieces, you can still install assets and use shell `qiongli check|upgrade|align`, but `doctor`, validators, tests, and full orchestrator execution will be partial or unavailable.

## Academic Idea Boundary

Qiongli includes an Academic Idea Funnel and Academic Grill Loop before early-stage paper outputs. This is an academic adaptation of Matt Pocock's `grill-me` interaction pattern, not a generic grill-me clone. The adaptation turns one-question-at-a-time clarification into academic idea-discovery: claim strength, evidence threshold, rival explanations, feasibility, venue/reviewer risk, and the handoff into `context/boundary_review.md`.

The funnel artifact is:

```text
RESEARCH/[topic]/context/idea_funnel.md
```

The boundary artifact is:

```text
RESEARCH/[topic]/context/boundary_review.md
```

Source credit: [Matt Pocock's skills repository](https://github.com/mattpocock/skills).

## Repository Map

| Path | Role |
|---|---|
| `content/workflow/` | Portable skill package source: `SKILL.md`, workflows, references, agents, standards |
| `content/skills/` | Canonical academic capability cards and registry |
| `content/subjects/` | Subject catalog, overlays, subject-specific skills, and materialization rules |
| `content/templates/` | Research artifact templates |
| `content/standards/` | Canonical contracts including `standards/research-workflow-contract.yaml`, `standards/mcp-agent-capability-map.yaml`, schemas, and quality gates |
| `packages/python-qiongli/` | Python CLI/runtime package and bridge modules |
| `packages/npm-qiongli/` | npm installer, bundled payload, and Python bridge entry |
| `packages/qiongli-plugin/` | Codex/Claude Code/Gemini plugin source and bundled literature MCP runtime |
| `packages/qiongli-literature-mcpb/` | Claude Desktop literature-provider MCPB package source |
| `docs/` | User, advanced, reference, architecture, and maintainer documentation |
| `tests/` | Contract, materialization, MCP, runtime, package, and orchestration tests |

Generated outputs are intentionally not normal feature-review targets. Normal feature PRs should update canonical source, tests, and documentation only. Release automation performs staged materialization into temporary roots, and subject changes should keep staged materialization plus npm package contract tests up to date.

When adding or deepening a subject, update `content/subjects/catalog.yaml`, subject overlays, subject-specific registry/markdown, selected profiles, eval fixtures, specialization audit expected terms, materializer tests, npm package contract tests against staged materialization, and release validation if the subject has a Desktop/Web artifact.

For agent and skill collaboration details, see [Agent-Skill Collaboration](docs/advanced/agent-skill-collaboration.md). The legacy shell installer source remains available at `scripts/install_qiongli.sh`; most users should prefer the install paths in [docs/guide/install.md](docs/guide/install.md).

## Verification

Common local checks:

```bash
uv run python -m unittest tests.test_mcp_cli tests.test_mcp_tool_handlers tests.test_mcp_stdio_server -v
uv run python -m unittest tests.test_orchestrator_workflows tests.test_controller_agnostic_orchestration -v
uv run python scripts/validate_research_standard.py --strict
npm test --prefix packages/npm-qiongli
git diff --check
```

For runtime readiness:

```bash
uv run python -m bridges.orchestrator doctor --cwd .
```

## More Documentation

- [Quick Start](docs/quickstart.md)
- [Install Guide](docs/guide/install.md)
- [Using Agent Skills](docs/guide/using-agent-skills.md)
- [Task Recipes](docs/guide/task-recipes.md)
- [Multi-Agent Runtime Guide](docs/guide/multi-agent.md)
- [CLI Reference](docs/reference/cli.md)
- [Architecture](docs/architecture.md)
- [Cross-Platform MCP Server](docs/advanced/cross-platform-mcp.md)
- [Plugin-First Architecture](docs/advanced/plugin-first-architecture.md)
- [Controller Modes](docs/advanced/controller-modes.md)
- [Subject Packaging Model](docs/advanced/subject-packaging-model.md)

## Design Lineage

Qiongli borrows useful ideas from existing agent-workflow projects while adapting them to academic research:

- [fengshao1227/ccg-workflow](https://github.com/fengshao1227/ccg-workflow): strict phase separation, spec -> plan -> execute -> review, and constrained execution.
- [GuDaStudio/skills](https://github.com/GuDaStudio/skills): reusable Claude-oriented collaboration skill packaging.
- [Matt Pocock's `grill-me` skill](https://github.com/mattpocock/skills/blob/main/skills/productivity/grill-me/SKILL.md): one-question-at-a-time clarification, adapted here into the Academic Idea Funnel and Academic Grill Loop for defensible scholarly idea formation.
