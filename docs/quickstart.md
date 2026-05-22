# Quick Start

Use this page when you want to start using Qiongli, not maintain it. The shortest path is:

1. Pick an install surface.
2. Create or open a research workspace.
3. Run a workflow or inspect a task plan.
4. Use quality gates and artifacts to keep the result auditable.

## 1. Pick The Smallest Install Surface

| Situation | Use | Requires Python before install |
|---|---|---|
| One client, minimal setup | Native plugin / extension | No |
| Several clients need global workflow assets | Bootstrap `partial` | No |
| You need `doctor`, validators, or orchestrator task execution | Bootstrap `full` | Yes, Python 3.12+ |
| You prefer npm automation | `npm install -g qiongli` or `npx qiongli@latest` | Only for advanced bridge commands |
| You only need the Python updater CLI | `pipx install qiongli` | Yes |

For full detail, read [Install](/guide/install).

## 2. Install Workflow Assets

If you chose the native plugin path, install Qiongli from the shared Skillsplace marketplace:

```bash
codex plugin marketplace add jxpeng98/skillsplace --ref main
```

```bash
claude plugin marketplace add jxpeng98/skillsplace@main
claude plugin install qiongli@skillsplace
```

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

## 3. Start A Research Workspace

```bash
mkdir my-paper
cd my-paper
```

Then open your client and use a workflow command:

```text
/paper
/lit-review
/paper-write
/code-build
```

These commands are entry UX. The canonical task definitions, expected outputs, quality gates, and role boundaries live in the Qiongli contracts.

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
