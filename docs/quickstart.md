# Quick Start

Use this page when you want to start using Qiongli, not maintain it. The shortest path is:

1. Pick an install surface.
2. Create or open a research workspace.
3. Run a workflow or inspect a task plan.
4. Use quality gates and artifacts to keep the result auditable.

## 1. Pick The Smallest Install Surface

| Situation | Use | Requires Python before install |
|---|---|---|
| Claude Desktop/Web with no CLI | Focused subject Desktop ZIP, such as `qiongli-claude-desktop-skill-economics-<tag>.zip` | No |
| One client, minimal setup | Native plugin / extension | No |
| Several clients need global workflow assets | Bootstrap `partial` | No |
| You need `doctor`, validators, or orchestrator task execution | Bootstrap `full` | Yes, Python 3.12+ |
| You prefer npm automation | `npm install -g qiongli` or `npx qiongli@latest` | Only for advanced bridge commands |
| You only need the Python updater CLI | `pipx install qiongli` | Yes |

For full detail, read [Install](/guide/install).

## 2. Install Workflow Assets

If you use Claude Desktop or Claude.ai and do not want to work in a code/CLI environment, download the focused subject ZIP you need from the GitHub Release assets. Public Desktop ZIP subjects in this phase are `core`, `economics`, `business`, `finance`, `political-economy`, `geoeconomics`, and `economics-accounting`; there is no standalone accounting Desktop ZIP yet. Use `qiongli-claude-desktop-skill-core-<tag>.zip` for the default workflow, `qiongli-claude-desktop-skill-economics-<tag>.zip` for economics, `qiongli-claude-desktop-skill-political-economy-<tag>.zip` for political economy, `qiongli-claude-desktop-skill-geoeconomics-<tag>.zip` for geoeconomics, `qiongli-claude-desktop-skill-business-<tag>.zip` for business, `qiongli-claude-desktop-skill-finance-<tag>.zip` for finance, or `qiongli-claude-desktop-skill-economics-accounting-<tag>.zip` for the official economics/accounting composite. Drag the ZIP into Claude Desktop's Skills upload/install flow, or use `Customize > Skills > + > Create skill > Upload a skill`. The same ZIP upload flow also works in Claude.ai.

The Desktop/Web ZIP uses `coverage=focused` to stay under upload limits. It is a subject-specialized package, not a reduced-quality cut. It keeps workflows, templates, standards, selected profiles, `skills-summary.md`, and `skills-core.md`; specialized ZIPs also include selected effective skill markdown generated with layered overlays.

If you chose the native plugin path for Codex or Claude Code, install Qiongli from the shared Skillsplace marketplace:

```bash
codex plugin marketplace add jxpeng98/skillsplace --ref main
```

```bash
claude plugin marketplace add jxpeng98/skillsplace@main
claude plugin install qiongli@skillsplace
```

For bundled literature MCP installs, provider keys are configured through the MCP tools, not the client MCP settings UI. Ask Codex, Claude Desktop, Claude Code, or another local MCP client to run `qiongli_config_status`, then use `qiongli_configure_provider` to open a local setup page for `openalex.email` or `semantic_scholar.api_key`. This keeps secrets out of plugin manifests, release artifacts, and chat context.

For cross-client global workflow assets, use the bootstrap installer.

Linux / macOS:

```bash
curl -fsSL https://raw.githubusercontent.com/jxpeng98/qiongli/main/scripts/bootstrap_qiongli.sh | bash -s -- --profile partial --project-dir "$PWD" --target all
```

Windows PowerShell 7+:

```powershell
winget install --id Microsoft.PowerShell --source winget
Invoke-WebRequest https://raw.githubusercontent.com/jxpeng98/qiongli/main/scripts/bootstrap_qiongli.ps1 -OutFile .\bootstrap_qiongli.ps1
pwsh -ExecutionPolicy Bypass -File .\bootstrap_qiongli.ps1 -Profile partial -ProjectDir "$PWD" -Target all
```

Use `--profile full` instead of `partial` when Python 3.12+ is already installed and you want runtime checks, validators, or orchestrated tasks.

For npm or pipx installs, `--subject` defaults to `core` and `--coverage` defaults to `complete`:

```bash
qiongli install --subject economics --target all
qiongli install --subject accounting --target all
npx qiongli@latest install --subject economics --target all
qiongli install --subject political-economy --target all
qiongli install --subject geoeconomics --target all
qiongli install --subject economics-accounting --target all
qiongli install --subject economics --coverage focused --target all
qiongli upgrade --subject accounting --target all
qiongli remove --target all --dry-run
qiongli customize --subject economics --name my-econ-lab --out ./qiongli-custom/econ-lab
qiongli check --json
```

Use the default complete coverage when you are unsure: `qiongli install --target all` means `core/complete`, `--subject economics`, `--subject business`, `--subject finance`, `--subject political-economy`, and `--subject geoeconomics` mean complete specialized installs, and `--subject accounting` means `accounting/complete`, full framework plus accounting specialization. Use `--coverage focused` only when you deliberately want the slimmer selected subject package and Desktop/Web ZIP shape. `political-economy` and `geoeconomics` are independent subjects, not a composite. Official composite subjects such as `economics-accounting` are named subjects, not arbitrary comma-separated stacking. Switch subjects or coverage by rerunning `install` or `upgrade` with new flags. Custom overlays affect generated output only and do not rewrite canonical source files; `qiongli customize` plus `--custom-dir` materialization is for the Python/source checkout workflow, while npm runtime installs pre-generated payloads in this phase.

Use `qiongli remove` when you need to remove CLI-installed global assets before relying only on marketplace plugins.

## 3. Start A Research Workspace

```bash
mkdir my-paper
cd my-paper
```

Then open your client and use the entrypoint that client supports.

Codex:

```text
/skills
$qiongli plan an empirical paper on ai-in-education
```

Claude Code or Gemini CLI:

```text
/paper
/lit-review
/paper-write
/code-build
```

Codex uses `/skills` for discovery and `$qiongli` for execution; it does not register a custom `/qiongli` slash command. Claude Code and Gemini expose slash workflow entries when command/workflow discovery is installed. These entries are UX wrappers. The canonical task definitions, expected outputs, quality gates, and role boundaries live in the Qiongli contracts.

See [Using Agent Skills](/guide/using-agent-skills) for the full client-by-client usage matrix.

## 4. Choose A Research Route

| Paper type | Pipeline | Start when |
|---|---|---|
| `systematic-review` | `systematic-review-prisma` | You need PRISMA-style search, screening, extraction, and synthesis. |
| `empirical` | `empirical-study` | You need a standard empirical research paper route. |
| `qualitative` | `qualitative-study` | You need interview, case, ethnographic, or process-oriented outputs. |
| `empirical` | `rct-prereg` | You need RCT preregistration and reporting checks. |
| `theory` | `theory-paper` | You need conceptual development, mechanisms, and contribution framing. |
| `methods` | `code-first-methods` | You need research code and method artifacts to be first-class deliverables. |

See [Task Recipes](/guide/task-recipes) for scenario-level routing.

## 5. Inspect Before You Run

If you installed the `full` runtime, inspect the task route first:

```bash
python3 -m bridges.orchestrator task-plan \
  --task-id F3 \
  --paper-type empirical \
  --topic ai-in-education \
  --cwd .
```

`task-plan` shows:

- required and optional outputs
- prerequisite tasks
- functional owner and handoff trace
- runtime plan for draft, review, fallback, and verification

## 6. Run A Canonical Task

```bash
python3 -m bridges.orchestrator task-run \
  --task-id F3 \
  --paper-type empirical \
  --topic ai-in-education \
  --cwd . \
  --triad
```

Useful switches:

- `--mcp-strict`: block when required external evidence providers are unavailable.
- `--skills-strict`: block when required internal skill specs are missing.
- `--triad`: request a third independent audit when available.
- `--focus-output` and `--output-budget`: reduce auxiliary artifact spread for one run.
- `--research-depth deep` plus `--max-rounds`: force deeper evidence expansion and revision.

## 7. Understand The Quality Gates

Qiongli is useful because it leaves reviewable traces:

- literature search diagnostics and materialized search bundles
- claim-evidence maps and citation risk artifacts
- method diagnostics and reporting checks
- code spec, plan, execution, review, and reproducibility artifacts
- handoffs, disagreement records, and verification status for multi-agent work

Use [Architecture](/architecture) when you need to understand how those contracts fit together.
