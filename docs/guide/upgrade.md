# Upgrade / Auto-Upgrade Guide (No Fork Required)

This guide explains how to:
1) Check for new versions.
2) Automate the upgrade process.
3) Complete the upgrade without needing to fork or `git clone` the repository.

## 0) Choose an Upgrade Entry Point

### Option A: Shell bootstrap (No Python required)

This path only needs `bash` plus `curl`/`wget` and `tar`:

```bash
curl -fsSL https://raw.githubusercontent.com/jxpeng98/qiongli/main/scripts/bootstrap_qiongli.sh | bash -s -- \
  --repo <owner>/<repo> \
  --project-dir /path/to/project \
  --target all \
  --overwrite
```

Notes:
- The bootstrapper downloads the selected release archive and runs `scripts/install_qiongli.sh` from inside it.
- By default it also installs the shell CLI: `qiongli`, `ql`, `research-skills`, `rsk`, `rsw`.
- Default shell CLI location: `${QIONGLI_BIN_DIR:-${RESEARCH_SKILLS_BIN_DIR:-~/.local/bin}}`.
- Use `--no-cli` to skip shell CLI installation, or `--cli-dir <path>` to change the install location.
- `--doctor` is optional and only runs when `python3` exists.
- Remote bootstrap supports `--mode copy` only. Use a local clone for `--mode link`.

### Option B: Python CLI (optional)

This repository also provides a `pyproject.toml` package for people who want a reusable updater CLI:

```bash
pipx install qiongli
# This provides 3 equivalent commands (choose any):
# - qiongli
# - rsk
# - rsw
# You can also set `QIONGLI_REPO=<owner>/<repo>` to omit the --repo flag
qiongli check --repo <owner>/<repo>
qiongli upgrade --repo <owner>/<repo> --target all --doctor
qiongli init --project-dir /path/to/project
```

> Note: pip installs/upgrades the "updater CLI." The actual refresh of global client skill directories is still performed by `qiongli upgrade` (or `qiongli upgrade`). Project-local files are explicit: use `qiongli init` or `qiongli upgrade --parts project ...` when you want them rewritten.

## 1) What exactly are you upgrading?

This project has one type of "installation target":

- **Local skill directories for the supported clients** (so Codex / Claude Code / Antigravity / Hermes recognize the skill globally)
  - Codex: `${CODEX_HOME:-~/.codex}/skills/qiongli-workflow`
  - Claude: `${CLAUDE_CODE_HOME:-~/.claude}/skills/qiongli-workflow`
  - Antigravity (global): `${ANTIGRAVITY_HOME:-~/.gemini/antigravity}/skills/qiongli-workflow`
  - Hermes: `${HERMES_HOME:-~/.hermes}/skills/qiongli-workflow`

Upgrading simply means **overwriting these target paths with the new version**.

_Project-local integrations (like `.env`) are only refreshed when you explicitly run `qiongli init --project-dir .` or add `--parts project`._

---

## 2) Checking for new versions (Recommended)

```bash
# If QIONGLI_REPO is set, --repo can be omitted
qiongli check --repo <owner>/<repo>
# Or run within the repository (equivalent):
python3 scripts/qiongli_update.py check --repo <owner>/<repo>
```

Details:
- `--repo` is used to query the latest GitHub release tag.
- If it detects that "local/installed version < latest version", the command returns exit code `1` (which is useful for automation scripts).
- You can set a default upstream to omit `--repo`:
  - Envrionment variable: `export QIONGLI_REPO=<owner>/<repo>`
  - If you run this inside a `qiongli` clone with a configured git remote (prioritizes `upstream`, then `origin`), `--repo` can be omitted.
  - Or add a `qiongli.toml` file to your project root (easy to commit to your project repo, great for CI).

Example (in project root):

```toml
# qiongli.toml
[upstream]
repo = "<owner>/<repo>" # Or Git URL
```

Afterward, you can run:

```bash
qiongli check
qiongli upgrade --target all --doctor
qiongli init --project-dir .
```

---

## 3) Automatic Upgrade (No fork, no git clone required)

This directly downloads the GitHub release archive and executes the installation script inside it:

```bash
curl -fsSL https://raw.githubusercontent.com/jxpeng98/qiongli/main/scripts/bootstrap_qiongli.sh | bash -s -- \
  --repo <owner>/<repo> \
  --project-dir /path/to/your/project \
  --target all \
  --overwrite
```

Or, if Python is available:

```bash
# If QIONGLI_REPO is set, --repo can be omitted
qiongli upgrade \
  --repo <owner>/<repo> \
  --target all \
  --mode copy \
  --doctor

# Or run within the repository (equivalent):
python3 scripts/qiongli_update.py upgrade \
  --repo <owner>/<repo> \
  --target all \
  --mode copy \
  --doctor
```

Key points:
- This method **does not rely on git** and does not require you to clone the repository locally.
- The shell bootstrap path **does not rely on Python**.
- The shell CLI itself can run `check`, `upgrade`, and `align` without Python.
- Default upgrade is plugin-first from v1.9.0 onward: it installs the full local plugin surface, then cleans old global skills and Codex/Claude standalone MCP configs after the new install succeeds.
- Add `--surface skills --profile partial` when you explicitly want the old skills-only upgrade path.
- Add `--parts project` when you explicitly want to refresh project-local workflow assets.
- For private repositories or if you hit API rate limits, it is recommended to set: `GITHUB_TOKEN` or `GH_TOKEN`.
- It defaults to using the "latest release tag", but both shell bootstrap and `qiongli upgrade` accept explicit refs:
  - `--ref v0.1.0-beta.6 --ref-type tag`
  - `--ref main --ref-type branch`

It is recommended to restart your clients (Codex / Claude Code / Antigravity / Hermes) after upgrading.

---

## 4) Alternative "Auto-Upgrade": Link Installation + Git Pull (Best for long-term maintenance)

If you are willing to keep a local clone of the repository (no fork needed, just clone once), this is recommended:

1) When installing, use `--mode link` (creates symlinks, meaning future updates don't require re-running the install script):

```bash
./scripts/install_qiongli.sh --target all --mode link --overwrite
python3 -m qiongli.cli init --project-dir /path/to/project --target all --overwrite
```

2) When updating, simply run:

```bash
git pull
```

Because the installation targets are symlinks, updating the repository contents automatically syncs the skill and workflows to the latest version across selected clients.

---

## 5) Automation Suggestions (Optional)

You can use cron/CI to do a "weekly check + upgrade if available":

1) Check periodically:
```bash
qiongli check --repo <owner>/<repo>
```
2) If exit code is 1, execute upgrade:
```bash
qiongli upgrade --repo <owner>/<repo> --target all
qiongli init --project-dir /path/to/project
```

If you want this upgrade detection integrated as a Codex Automation (run periodically and generate inbox results), just let me know the run frequency and target project paths.
