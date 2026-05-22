# Quick Start Guide

This guide focuses on stable user-facing entrypoints. You do not need to understand `skills/`, `roles/`, or `pipelines/` to start using the system.

If you do want a detailed map of what each internal skill section contains, see [Skills Guide](/reference/skills).
If you want scenario-driven routes such as literature review, empirical design, qualitative fieldwork writing, code-first methods, or rebuttal prep, see [Task Recipes](/guide/task-recipes).

::: warning Full Functionality Requirement
If you want the full system, install and configure all of the following:

- `python3` 3.12+
- `codex`
- `claude`
- `gemini`
- `OPENAI_API_KEY`, `ANTHROPIC_API_KEY`, `GOOGLE_API_KEY`

Without them, you can still install workflow assets and use shell `qiongli check|upgrade|align`, but `doctor`, validators, tests, and full orchestrator execution will be limited.
:::

## 1. Native Plugin And Extension Install

For single-client use, install **Qiongli** through the client-native surface. This installs the `qiongli-workflow` skill without requiring `pip`, `pipx`, or the `rsk` CLI.

```text
# Codex
Install Qiongli from the official Codex plugin marketplace.

# Claude Code
/plugin marketplace add ./path/to/qiongli
/plugin install qiongli@qiongli

# Gemini CLI
gemini extensions install ./path/to/qiongli/plugins/qiongli
```

Use the bootstrap path below when you need cross-client global installation, shell maintenance commands, or the orchestrator runtime.

## 2. Global One-Click Install

The recommended path is the one-click bootstrap. Choose the install profile explicitly when you know what you need:

- `partial`: installs global skill assets and slash workflow discovery. It does not require Python.
- `full`: installs everything in `partial`, plus the shell CLI (`qiongli`, `ql`, `research-skills`, `rsk`, `rsw`) and optional `doctor` validation. It requires Python 3.12+ to already exist on PATH.

The installer does not install Python or `mise`. If you need `full`, install Python first using python.org, Homebrew, winget, Microsoft Store, pyenv, mise, or your OS package manager.

Linux / macOS:

```bash
# Prompt for partial or full
curl -fsSL https://raw.githubusercontent.com/jxpeng98/qiongli/main/scripts/bootstrap_qiongli.sh | bash -s -- --project-dir "$PWD" --target all

# Force the lightweight profile
curl -fsSL https://raw.githubusercontent.com/jxpeng98/qiongli/main/scripts/bootstrap_qiongli.sh | bash -s -- --profile partial --project-dir "$PWD" --target all

# Force the full runtime profile
curl -fsSL https://raw.githubusercontent.com/jxpeng98/qiongli/main/scripts/bootstrap_qiongli.sh | bash -s -- --profile full --project-dir "$PWD" --target all
```

Windows PowerShell 7+:

```powershell
winget install --id Microsoft.PowerShell --source winget
Invoke-WebRequest https://raw.githubusercontent.com/jxpeng98/qiongli/main/scripts/bootstrap_qiongli.ps1 -OutFile .\bootstrap_qiongli.ps1

# Prompt for partial or full
pwsh -ExecutionPolicy Bypass -File .\bootstrap_qiongli.ps1 -ProjectDir "$PWD" -Target all

# Force the lightweight profile
pwsh -ExecutionPolicy Bypass -File .\bootstrap_qiongli.ps1 -Profile partial -ProjectDir "$PWD" -Target all

# Force the full runtime profile
pwsh -ExecutionPolicy Bypass -File .\bootstrap_qiongli.ps1 -Profile full -ProjectDir "$PWD" -Target all
```

The bootstrap installs `qiongli-workflow` globally into the configuration directories of Codex, Claude Code, Gemini, and Antigravity when selected. Project-local files are only written when you explicitly run `qiongli init --project-dir .`.

## 3. Zero-Config Usage

Because the commands are registered globally, using the system is now instantaneous.

1. **Create an empty workspace:** `mkdir my-new-paper && cd my-new-paper`
2. **Start your AI agent:** Type `claude` or `gemini` in your terminal.
3. **Execute a workflow:** Type `/paper` or `/lit-review`.

The AI will seamlessly fetch the global skill package in the background.

## 4. Advanced Entry Modes

| Entry mode | Use when | Entry |
|---|---|---|
| Native plugin / extension | You want the easiest single-client install | Codex marketplace, Claude marketplace, or Gemini extension |
| Slash commands | You want `/paper`, `/lit-review`, etc. | Included by default via global symlinks |
| Orchestrator CLI | You want explicit automated task routing and validation | `python3 -m bridges.orchestrator task-plan|task-run|doctor` |
| Installer / updater CLI | You want install or refresh commands after bootstrap | `qiongli`, `ql`, `research-skills`, `rsk`, `rsw` |

## 5. Choose a Paper Type

The canonical paper-type pipelines are:

| Paper type | Pipeline | Typical use |
|---|---|---|
| `systematic-review` | `systematic-review-prisma` | PRISMA-style evidence review |
| `empirical` | `empirical-study` | Standard empirical research paper |
| `qualitative` | `qualitative-study` | Interview, case, ethnographic, or process-oriented qualitative paper |
| `empirical` | `rct-prereg` | RCT with preregistration and reporting checks |
| `theory` | `theory-paper` | Conceptual or theory-building paper |
| `methods` | `code-first-methods` | Methods paper where code is a first-class deliverable |

## 6. Plan Before You Run

Inspect prerequisites and routing before execution:

```bash
python3 -m bridges.orchestrator task-plan \
  --task-id F3 \
  --paper-type empirical \
  --topic ai-in-education \
  --cwd .
```

`task-plan` shows:

- contract outputs
- prerequisite tasks
- functional owner
- functional handoff trace
- runtime plan (`draft` / `review` / `fallback`)

## 7. Run a Canonical Task

Execute one task with routing, MCP evidence collection, and review:

```bash
python3 -m bridges.orchestrator task-run \
  --task-id F3 \
  --paper-type empirical \
  --topic ai-in-education \
  --cwd . \
  --triad
```

Common options:

- `--mcp-strict`: block if required MCP providers are unavailable
- `--skills-strict`: block if required internal skill specs are missing
- `--triad`: request a third independent audit when available
- `--profile`, `--draft-profile`, `--review-profile`, `--triad-profile`: select execution profiles
- `--focus-output` and `--output-budget`: reduce auxiliary artifact spread for this run by keeping only a smaller active output set
- `--research-depth deep` plus `--max-rounds`: force a narrower, more adversarial evidence-expansion and revision process

## 8. Use Slash Commands When You Want UX

After `qiongli upgrade`, workflow slash-commands are globally registered via symlinks:

- **Claude Code**: `~/.claude/commands/*.md`
- **Gemini CLI**: `~/.gemini/workflows/*.md`

No per-project setup required. Available commands:

```text
/paper
/lit-review
/paper-read
/find-gap
/study-design
/paper-write
/submission-prep
/academic-present
```

These commands are entry UX only. The canonical task and artifact truth still lives in `standards/`.

To remove all symlinks: `qiongli clean --globals`.

## 9. Know When to Switch to Maintainer Docs

Use this guide for operation.
Switch to maintainer docs only when you are changing the system itself:

- Architecture and layer model: [Architecture](/architecture)
- Edit rules and decision boundaries: [Conventions](/conventions)
- Where to modify specific behavior: [Extend Qiongli](/advanced/extend-qiongli)
