# Multi-Client Install Guide (Codex / Claude Code / Gemini)

## 1. Portable Install (No Python Required)

The most portable install path is the shell bootstrapper. It downloads the selected release archive and runs the bundled installer:

```bash
curl -fsSL https://raw.githubusercontent.com/jxpeng98/qiongli/main/scripts/bootstrap_qiongli.sh | bash -s -- \
  --project-dir /path/to/project \
  --target all
```

Requirements:
- `bash`
- `curl` or `wget`
- `tar`

Notes:
- By default this also installs a shell CLI: `qiongli`, `ql`, `research-skills`, `rsk`, `rsw`.
- Default CLI location: `${QIONGLI_BIN_DIR:-${RESEARCH_SKILLS_BIN_DIR:-~/.local/bin}}`.
- Add `--overwrite` when re-installing/upgrading existing targets.
- Use `--no-cli` if you only want the workflow assets.
- Use `--cli-dir <path>` to install the shell CLI elsewhere.
- `--doctor` is optional and only runs when `python3` is available.
- Installer mode selection is `--mode copy|link`. Remote bootstrap only supports `--mode copy`.
- Remote bootstrap only supports `--mode copy`. If you want `--mode link`, clone the repo and use the local installer below.

## 2. Optional npm CLI

If Node.js is already available, you can install the standalone npm CLI instead of the Python CLI:

```bash
npm install -g qiongli
qiongli install --subject core --target all --project-dir /path/to/project
qiongli install --subject economics --target all --project-dir /path/to/project
```

For prerelease testing without a global install:

```bash
npx qiongli@next install --subject economics --target all --project-dir /path/to/project
npx qiongli@next check --json
```

The npm package bundles pre-materialized `core` and `economics` subject payloads. `--subject` defaults to `core`; switch subjects by rerunning install or upgrade with a different `--subject`. Advanced commands such as `qiongli doctor`, `qiongli task-run`, and `qiongli team-run` use the Python bridge source bundled inside the npm package and require Python 3.12+ plus `PyYAML`.

## 3. Optional Python CLI

If Python is already available on the machine, you can install the updater CLI with `pipx`:

```bash
pipx install qiongli
qiongli install --subject economics --target all --project-dir /path/to/project --doctor
```

## 4. Local Repository Installer

If you already have a repository checkout, you can run the installer directly:

```bash
./scripts/install_qiongli.sh --target all --project-dir /path/to/project --install-cli --doctor
```

## Global-First Behaviors & What Gets Installed

Default install/upgrade behavior is purely **global**. Your project directories remain clean.

The installer does two things:
1. **Installs the active subject package:** `qiongli-workflow` is placed into the specific home directories of your AI clients (e.g. `~/.claude/skills/`, `~/.gemini/skills/`). `core` is the default; `economics` is selected with `--subject economics`.
2. **Registers Slash Commands:** It drops lightweight symlinks into the client's discovery paths (e.g. `~/.claude/commands/paper.md` and `~/.gemini/workflows/lit-review.md`).

This means commands like `/paper` and `/study-design` become natively recognized by the AI engines **no matter what folder you are working in**.

_Project-local files (like `.env`) are only written when you explicitly run `qiongli init --project-dir .`._

Home directory overrides:
- `CODEX_HOME`: root directory for Codex skill installation.
- `CLAUDE_CODE_HOME`: root directory for Claude Code skill installation.
- `GEMINI_HOME`: root directory for Gemini skill installation.
- `ANTIGRAVITY_HOME`: root directory for Antigravity global skill installation.

## Common flags

- `--install-cli`: install shell CLI commands (`qiongli`, `ql`, `research-skills`, `rsk`, `rsw`).
- `--no-cli`: skip shell CLI installation.
- `--cli-dir <path>`: choose where the shell CLI is installed (default: `${QIONGLI_BIN_DIR:-${RESEARCH_SKILLS_BIN_DIR:-~/.local/bin}}`).
- `--overwrite`: replace existing installation targets.
- `--dry-run`: preview installation actions only.
- `--doctor`: run `python3 -m bridges.orchestrator doctor --cwd <project>` after install when `python3` is available.

## Zero-Config Usage

Because commands are registered globally, using the system for a new paper is incredibly straightforward:

1. Create an empty directory for your new paper: `mkdir my-new-paper && cd my-new-paper`
2. Start the AI: `claude` or `gemini`
3. Execute a workflow directly: type `/paper` or `/lit-review`.

## Upgrade

- Check updates: `qiongli check --repo <owner>/<repo>`
- Upgrade (no fork / no git clone required): `qiongli upgrade --repo <owner>/<repo> --subject economics --target all` for global refresh.
- Full guide: `guides/basic/upgrade-qiongli.md`

## Verify

```bash
python3 -m bridges.orchestrator doctor --cwd /path/to/project
python3 scripts/validate_research_standard.py --strict
```
