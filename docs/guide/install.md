# Install Qiongli

Qiongli has several installation surfaces because users need different levels of runtime control. Start with the smallest surface that gives you the workflow you need.

## Install Surfaces

| Surface | Best for | Installs | Python required |
|---|---|---|---|
| Native plugin / extension | One client, least setup | Client plugin plus `qiongli-workflow` | No |
| Bootstrap `partial` | Global workflow assets across clients | Skills and slash workflow discovery | No |
| Bootstrap `full` | Runtime checks and orchestration | `partial` plus shell CLI and `doctor` support | Yes, Python 3.12+ |
| npm / npx | Node-based automation | npm CLI plus bundled workflow payload | Only for advanced bridge commands |
| pipx / pip | Python updater CLI | Python CLI distribution | Yes |

## Native Plugin And Extension

Use this when you only need Qiongli inside one supported client.

Codex installs through the shared [Skillsplace](https://github.com/jxpeng98/skillsplace) marketplace:

```bash
codex plugin marketplace add jxpeng98/skillsplace --ref main
codex plugin marketplace list
```

Then install or enable `qiongli` from the Codex plugin UI.

Claude Code uses the same Skillsplace catalog:

```bash
claude plugin marketplace add jxpeng98/skillsplace@main
claude plugin install qiongli@skillsplace
```

Inside an interactive Claude Code session, use:

```text
/plugin marketplace add jxpeng98/skillsplace@main
/plugin install qiongli@skillsplace
```

Gemini CLI still installs the local extension payload directly:

```bash
gemini extensions install ./path/to/qiongli/plugins/qiongli
```

This path does not install the shell CLI, Python bridge, or global slash-command symlinks. Use bootstrap or npm when you need those.

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
```

## npm / npx

Use npm when you want a Node-distributed installer with the workflow payload bundled:

```bash
npm install -g qiongli
qiongli install --target all --project-dir "$PWD"
```

For one-off runs:

```bash
npx qiongli@latest install --target all --project-dir "$PWD"
npx qiongli@latest check --json
```

Prerelease testing remains available through the `next` dist-tag:

```bash
npx qiongli@next install --target all --project-dir "$PWD"
```

## pipx / pip

Use pipx when you specifically want the Python-distributed updater CLI:

```bash
pipx install qiongli
qiongli upgrade --target all
```

Upgrade it with:

```bash
pipx upgrade qiongli
qiongli upgrade --target all --doctor --project-dir /path/to/project
```

## What Gets Installed

Depending on the surface, Qiongli may install:

- `qiongli-workflow` skill assets under client home directories
- workflow command discovery links such as `/paper`, `/lit-review`, `/paper-write`, and `/code-build`
- shell commands `qiongli`, `ql`, and compatibility aliases `research-skills`, `rsk`, `rsw`
- optional project integration files when you explicitly run `qiongli init --project-dir .`

Project-local files are not written by default. The global workflow package can be used from any research workspace.

## Keep Versions Aligned

If you use multiple surfaces, keep plugin, global skill assets, npm payload, and Python CLI aligned:

```bash
qiongli check
qiongli upgrade --target all
```

If you move fully to native plugins and no longer need legacy global slash commands, inspect cleanup first:

```bash
qiongli clean --globals --dry-run
```
