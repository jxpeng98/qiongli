# Multi-Client Install Guide (Codex / Claude Code / Gemini)

::: warning Full Functionality Requirement
You do not need to preinstall Python for first install anymore, but the full runtime still depends on:

- `python3`
- `codex`
- `claude`
- `gemini`
- `OPENAI_API_KEY`, `ANTHROPIC_API_KEY`, `GOOGLE_API_KEY`

Without them, asset installation and shell `rsk` maintenance commands still work, but orchestrator execution, `doctor`, validators, and full multi-model flows will be limited.
:::

## 1. Native Plugin And Extension Install

For single-client use, install **Research Skills** through the client-native extension surface. This is the recommended path when you only need the `research-paper-workflow` skill inside one client.

The native plugin/extension distribution does not require `pip`, `pipx`, or the `rsk` CLI. Use the bootstrap/CLI path when you need any of these:

- cross-client global installation
- global slash-command symlinks managed by `rsk`
- `rsk upgrade`, `rsk check`, or `doctor`
- multi-model orchestrator execution

Local development install commands:

```text
# Codex
Install Research Skills from the official Codex plugin marketplace.

# Claude Code
/plugin marketplace add ./path/to/research-skills
/plugin install research-skills@research-skills

# Gemini CLI
gemini extensions install ./path/to/research-skills/plugins/research-skills
```

This repository contains the packaging metadata used for local validation:

- `.agents/plugins/marketplace.json`
- `.claude-plugin/marketplace.json`
- `plugins/research-skills/.codex-plugin/plugin.json`
- `plugins/research-skills/.claude-plugin/plugin.json`
- `plugins/research-skills/gemini-extension.json`
- `plugins/research-skills/skills/research-paper-workflow`

Codex and Claude Code use marketplace catalogs. Gemini CLI uses the official extension system (`gemini-extension.json`) rather than a marketplace JSON.

## 2. Choose `partial` Or `full`

The bootstrap installers now explain these choices interactively if you omit `--profile`.

| Profile | What it installs | Python required before install | Result after install |
|---|---|---|---|
| `partial` | global skills only | No | Assets are ready; orchestrator is not |
| `full` | `partial` + shell CLI + requires existing Python 3.12+ + `doctor` | Yes | Orchestrator runtime is ready |

How `full` works:

- If `python3 >= 3.12` already exists, bootstrap reuses it.
- If Python is missing or too old, bootstrap fails fast and prints installation options.
- On Windows, PowerShell installs directly and only installs Git for Windows via `winget` when shell CLI wrappers need Bash.

### Python prerequisite for `full`

`full` mode requires Python 3.12+ to already be available on PATH. The installer does not install Python or `mise` for you. Install Python using any method you prefer:

- macOS: python.org installer, `brew install python`, `pyenv`, or `mise`
- Windows: python.org installer, `winget install -e --id Python.Python.3.12 --source winget`, Microsoft Store, or pyenv-win
- Linux: distro package manager, `pyenv`, or `mise`

Verify before running `full`:

```bash
python3 --version
```

## 3. Run The Recommended One-Click Bootstrap

### Linux / macOS

Prompt for `partial` or `full`:

```bash
curl -fsSL https://raw.githubusercontent.com/jxpeng98/research-skills/main/scripts/bootstrap_research_skill.sh | bash -s -- --project-dir "$PWD" --target all
```

Force `partial`:

```bash
curl -fsSL https://raw.githubusercontent.com/jxpeng98/research-skills/main/scripts/bootstrap_research_skill.sh | bash -s -- --profile partial --project-dir "$PWD" --target all
```

Force `full`:

```bash
curl -fsSL https://raw.githubusercontent.com/jxpeng98/research-skills/main/scripts/bootstrap_research_skill.sh | bash -s -- --profile full --project-dir "$PWD" --target all
```

Install the latest beta / prerelease:

```bash
curl -fsSL https://raw.githubusercontent.com/jxpeng98/research-skills/main/scripts/bootstrap_research_skill.sh | bash -s -- --beta --profile full --project-dir "$PWD" --target all
```

### Windows PowerShell 7+

If `pwsh` is not installed yet, install it first:

```powershell
winget install --id Microsoft.PowerShell --source winget
```

Download and prompt for `partial` or `full`:

```powershell
Invoke-WebRequest https://raw.githubusercontent.com/jxpeng98/research-skills/main/scripts/bootstrap_research_skill.ps1 -OutFile .\bootstrap_research_skill.ps1
pwsh -ExecutionPolicy Bypass -File .\bootstrap_research_skill.ps1 -ProjectDir "$PWD" -Target all
```

Force `partial`:

```powershell
pwsh -ExecutionPolicy Bypass -File .\bootstrap_research_skill.ps1 -Profile partial -ProjectDir "$PWD" -Target all
```

Force `full`:

```powershell
pwsh -ExecutionPolicy Bypass -File .\bootstrap_research_skill.ps1 -Profile full -ProjectDir "$PWD" -Target all
```

Install the latest beta / prerelease:

```powershell
pwsh -ExecutionPolicy Bypass -File .\bootstrap_research_skill.ps1 -Beta -Profile full -ProjectDir "$PWD" -Target all
```

What bootstrap installs:

- workflow assets for Codex / Claude Code / Gemini
- project integration files such as `.agent/workflows/`, `CLAUDE.md`, `.gemini/` when you run `rsk init` or `--parts project`
- shell CLI commands `research-skills`, `rsk`, `rsw` in `full` mode

## 4. Optional Local Installers

If Python is already available, you can use the local cross-platform Python installer:

```bash
python3 scripts/bootstrap_research_skill.py --profile partial --project-dir .
python3 scripts/bootstrap_research_skill.py --profile full --project-dir .
```

If you already have a local repository checkout on Linux or macOS, you can also use the local shell installer:

```bash
./scripts/install_research_skill.sh --profile partial --target all --project-dir /path/to/project
./scripts/install_research_skill.sh --profile full --target all --project-dir /path/to/project
```

The `pip` / `pipx` path is still available for the updater CLI, but it is no longer the recommended first-install path:

```bash
pipx install research-skills-installer
rsk upgrade --target all --doctor
rsk init --project-dir /path/to/project
```

## 6. Global-First Behaviors & What Gets Installed

Default install/upgrade behavior is purely **global**. Your project directories remain completely clean.

The installer does two things:
1. **Installs the Core Package:** `research-paper-workflow` is placed into the specific home directories of your AI clients (e.g. `~/.claude/skills/`, `~/.gemini/skills/`).
2. **Registers Slash Commands:** It drops lightweight symlinks into the client's discovery paths (e.g. `~/.claude/commands/paper.md` and `~/.gemini/workflows/lit-review.md`).

This means commands like `/paper` and `/study-design` become natively recognized by the AI engines **no matter what folder you are working in**.

_Project-local files (like `.env`) are only written when you explicitly run `rsk init --project-dir .`._

## 7. Common Flags

- `--profile partial|full`: choose the install preset explicitly instead of using the prompt.
- `--target codex|claude|gemini|antigravity|all`: limit installation scope.
- `--beta`: install the latest beta / prerelease tag when `--ref` is omitted.
- `--install-cli`: install shell CLI commands even outside `full`.
- `--no-cli`: skip shell CLI installation even in `full`.
- `--cli-dir <path>`: choose where the shell CLI is installed. Default: `${RESEARCH_SKILLS_BIN_DIR:-~/.local/bin}`.
- `--overwrite`: replace existing installation targets.
- `--dry-run`: preview installation actions only.
- `--doctor`: run `python3 -m bridges.orchestrator doctor --cwd <project>` after install.

## 8. Zero-Config Usage

Because commands are registered globally, using the system for a new paper is incredibly straightforward:

1. Create an empty directory for your new paper: `mkdir my-new-paper && cd my-new-paper`
2. Start the AI: `claude` or `gemini`
3. Execute a workflow directly: type `/paper` or `/lit-review`.

The model will seamlessly load the global skill assets without cluttering your workspace with boilerplate templates.

## 9. Upgrading and Verifying

To update everything to the latest global release across all AI clients (no need to navigate to each project):

```bash
rsk upgrade --target all --doctor
```

_Note: The shell CLI (`rsk check`, `rsk upgrade`, `rsk clean`) runs without Python. However, `--doctor`, test validators, and `bridges.orchestrator` require a valid Python 3 environment._
