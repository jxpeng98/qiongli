# Multi-Client Install Guide (Codex / Claude Code / Gemini)

::: warning Full Functionality Requirement
You can install the lightweight `partial` profile without Python. The `full` profile and the complete runtime still depend on:

- `python3` 3.12+
- `codex`
- `claude`
- `gemini`
- `OPENAI_API_KEY`, `ANTHROPIC_API_KEY`, `GOOGLE_API_KEY`

Without them, asset installation and shell `rsk` maintenance commands still work, but orchestrator execution, `doctor`, validators, and full multi-model flows will be limited.
:::

## 1. Native Plugin And Extension Install

For single-client use, install **Qiongli** through the client-native extension surface. This is the recommended path when you only need the `qiongli-workflow` skill inside one client.

The native plugin/extension distribution does not require `pip`, `pipx`, or the `rsk` CLI. Use the bootstrap/CLI path when you need any of these:

- cross-client global installation
- global slash-command symlinks managed by `rsk`
- `rsk upgrade`, `rsk check`, or `doctor`
- multi-model orchestrator execution

Local development install commands:

```text
# Codex
Install Qiongli from the official Codex plugin marketplace.

# Claude Code
/plugin marketplace add ./path/to/qiongli
/plugin install qiongli@qiongli

# Gemini CLI
gemini extensions install ./path/to/qiongli/plugins/qiongli
```

This repository contains the packaging metadata used for local validation:

- `.agents/plugins/marketplace.json`
- `.claude-plugin/marketplace.json`
- `plugins/qiongli/.codex-plugin/plugin.json`
- `plugins/qiongli/.claude-plugin/plugin.json`
- `plugins/qiongli/gemini-extension.json`
- `plugins/qiongli/commands/*.md`
- `plugins/qiongli/skills/qiongli-workflow`

In this layout, the plugin is the install/discovery container and `qiongli-workflow` is the portable skill package. The plugin command files are thin wrappers; the real workflow logic stays in `skills/qiongli-workflow/workflows/*.md`.

Codex and Claude Code use marketplace catalogs. Gemini CLI uses the official extension system (`gemini-extension.json`) rather than a marketplace JSON.

## 2. Compatibility With Existing Global Skills

The native plugin/extension install is compatible with older `partial` / `full` installs, but it does not automatically migrate or delete them.

These are separate installation surfaces:

| Surface | Typical path | Managed by |
|---|---|---|
| Native plugin bundle | client plugin/extension store, containing `plugins/qiongli/skills/qiongli-workflow` | Codex / Claude Code / Gemini plugin system |
| Legacy global skill install | `~/.codex/skills/qiongli-workflow`, `~/.claude/skills/qiongli-workflow`, `~/.gemini/skills/qiongli-workflow` | `rsk`, bootstrap, or local installer |
| Legacy slash-command discovery | `~/.claude/commands/*.md`, `~/.gemini/workflows/*.md` | `rsk` symlink management |

Use this rule:

- New users who only need client-native skills and `/paper`-style workflows should install the official plugin/extension.
- Existing `partial` / `full` users can install the plugin alongside their current global skills.
- Users who still need `qiongli`, `rsk`, `rsw`, `doctor`, validators, or `bridges.orchestrator` should keep the `full` runtime and run `rsk upgrade --target all --doctor` so the global skill package stays aligned with the plugin version.
- Users who are moving fully to the official plugin and no longer need old global slash discovery should inspect cleanup with `rsk clean --globals --dry-run`.

If both old global skills and the new plugin are installed, keep their versions aligned to avoid loading different `qiongli-workflow` copies in different clients or command paths.

## 3. Choose `partial` Or `full`

The bootstrap installers now explain these choices interactively if you omit `--profile`. Choose the smallest profile that matches what you need.

| Profile | What it installs | Python required before install | Result after install |
|---|---|---|---|
| `partial` | global `qiongli-workflow` skill assets for Codex / Claude Code / Gemini, plus workflow discovery links | No | Slash workflows such as `/paper` and `/lit-review` are ready |
| `full` | everything in `partial`, shell CLI commands `qiongli` / `rsk` / `rsw`, and optional `doctor` validation | Yes, Python 3.12+ | Full orchestrator runtime is ready |

Choose `partial` when:

- you only need native client skills and slash workflows
- Python is not installed yet
- you want the lowest-friction install on Windows or a locked-down machine

Choose `full` when:

- you want `rsk upgrade`, `rsk init`, `rsk doctor`, or `qiongli`
- you want `python3 -m bridges.orchestrator task-plan|task-run|doctor`
- you want local validators, unit tests, or multi-model orchestration

How `full` works:

- If `python3 >= 3.12` already exists, bootstrap reuses it.
- If Python is missing or too old, bootstrap fails fast and prints installation options. It does not install Python or `mise`.
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

## 4. Run The Recommended One-Click Bootstrap

### Linux / macOS

Prompt for `partial` or `full`:

```bash
curl -fsSL https://raw.githubusercontent.com/jxpeng98/qiongli/main/scripts/bootstrap_qiongli.sh | bash -s -- --project-dir "$PWD" --target all
```

Force `partial`:

```bash
curl -fsSL https://raw.githubusercontent.com/jxpeng98/qiongli/main/scripts/bootstrap_qiongli.sh | bash -s -- --profile partial --project-dir "$PWD" --target all
```

Force `full`:

```bash
curl -fsSL https://raw.githubusercontent.com/jxpeng98/qiongli/main/scripts/bootstrap_qiongli.sh | bash -s -- --profile full --project-dir "$PWD" --target all
```

Install the latest beta / prerelease:

```bash
curl -fsSL https://raw.githubusercontent.com/jxpeng98/qiongli/main/scripts/bootstrap_qiongli.sh | bash -s -- --beta --profile full --project-dir "$PWD" --target all
```

### Windows PowerShell 7+

If `pwsh` is not installed yet, install it first:

```powershell
winget install --id Microsoft.PowerShell --source winget
```

Download and prompt for `partial` or `full`:

```powershell
Invoke-WebRequest https://raw.githubusercontent.com/jxpeng98/qiongli/main/scripts/bootstrap_qiongli.ps1 -OutFile .\bootstrap_qiongli.ps1
pwsh -ExecutionPolicy Bypass -File .\bootstrap_qiongli.ps1 -ProjectDir "$PWD" -Target all
```

Force `partial`:

```powershell
pwsh -ExecutionPolicy Bypass -File .\bootstrap_qiongli.ps1 -Profile partial -ProjectDir "$PWD" -Target all
```

Force `full`:

```powershell
pwsh -ExecutionPolicy Bypass -File .\bootstrap_qiongli.ps1 -Profile full -ProjectDir "$PWD" -Target all
```

Install the latest beta / prerelease:

```powershell
pwsh -ExecutionPolicy Bypass -File .\bootstrap_qiongli.ps1 -Beta -Profile full -ProjectDir "$PWD" -Target all
```

What bootstrap installs:

- workflow assets for Codex / Claude Code / Gemini
- project integration files such as `.agent/workflows/`, `CLAUDE.md`, `.gemini/` when you run `rsk init` or `--parts project`
- shell CLI commands `qiongli`, `rsk`, `rsw` in `full` mode

## 5. Use The Installed Skills

Both `partial` and `full` are global-first. After installation:

1. Create or open a research workspace: `mkdir my-paper && cd my-paper`.
2. Start a supported client such as Claude Code or Gemini CLI.
3. Run a workflow command such as `/paper`, `/lit-review`, `/paper-write`, or `/code-build`.

The model loads the global `qiongli-workflow` package. Your project directory stays clean unless you explicitly initialize it:

```bash
rsk init --project-dir .
```

## 6. Optional Local Installers

If Python is already available, you can use the local cross-platform Python installer:

```bash
python3 scripts/bootstrap_qiongli.py --profile partial --project-dir .
python3 scripts/bootstrap_qiongli.py --profile full --project-dir .
```

If you already have a local repository checkout on Linux or macOS, you can also use the local shell installer:

```bash
./scripts/install_qiongli.sh --profile partial --target all --project-dir /path/to/project
./scripts/install_qiongli.sh --profile full --target all --project-dir /path/to/project
```

The `pip` / `pipx` path is still available for the updater CLI, but it is no longer the recommended first-install path:

```bash
pipx install qiongli-installer
rsk upgrade --target all --doctor
rsk init --project-dir /path/to/project
```

If you installed `partial` first and later add Python 3.12+, rerun bootstrap with `--profile full` or use `rsk upgrade --target all --doctor` after the shell CLI exists.

## 7. Global-First Behaviors & What Gets Installed

Default install/upgrade behavior is purely **global**. Your project directories remain completely clean.

The installer does two things:
1. **Installs the Core Package:** `qiongli-workflow` is placed into the specific home directories of your AI clients (e.g. `~/.claude/skills/`, `~/.gemini/skills/`).
2. **Registers Slash Commands:** It drops lightweight symlinks into the client's discovery paths (e.g. `~/.claude/commands/paper.md` and `~/.gemini/workflows/lit-review.md`).

This means commands like `/paper` and `/study-design` become natively recognized by the AI engines **no matter what folder you are working in**.

_Project-local files (like `.env`) are only written when you explicitly run `rsk init --project-dir .`._

## 8. Common Flags

- `--profile partial|full`: choose the install preset explicitly instead of using the prompt.
- `--target codex|claude|gemini|antigravity|all`: limit installation scope.
- `--beta`: install the latest beta / prerelease tag when `--ref` is omitted.
- `--install-cli`: install shell CLI commands even outside `full`.
- `--no-cli`: skip shell CLI installation even in `full`.
- `--cli-dir <path>`: choose where the shell CLI is installed. Default: `${RESEARCH_SKILLS_BIN_DIR:-~/.local/bin}`.
- `--overwrite`: replace existing installation targets.
- `--dry-run`: preview installation actions only.
- `--doctor`: run `python3 -m bridges.orchestrator doctor --cwd <project>` after install.

## 9. Zero-Config Usage

Because commands are registered globally, using the system for a new paper is incredibly straightforward:

1. Create an empty directory for your new paper: `mkdir my-new-paper && cd my-new-paper`
2. Start the AI: `claude` or `gemini`
3. Execute a workflow directly: type `/paper` or `/lit-review`.

The model will seamlessly load the global skill assets without cluttering your workspace with boilerplate templates.

## 10. Upgrading and Verifying

To update everything to the latest global release across all AI clients (no need to navigate to each project):

```bash
rsk upgrade --target all --doctor
```

To preview removal of old global slash-command symlinks after adopting the official plugin:

```bash
rsk clean --globals --dry-run
```

_Note: The shell CLI (`rsk check`, `rsk upgrade`, `rsk clean`) runs without Python. However, `--doctor`, test validators, and `bridges.orchestrator` require a valid Python 3 environment._
