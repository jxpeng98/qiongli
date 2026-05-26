# Install Qiongli

Qiongli has several installation surfaces because users need different levels of runtime control. Start with the smallest surface that gives you the workflow you need.

## Install Surfaces

| Surface | Best for | Installs | Python required |
|---|---|---|---|
| Native plugin / extension | One client, least setup | Client plugin plus `qiongli-workflow` | No |
| Claude Desktop Skill ZIP | Claude Desktop or Claude.ai, especially when you do not want to use a code/CLI environment | Personal `qiongli` Skill upload | No |
| Bootstrap `partial` | Global workflow assets across clients | Skills and workflow discovery where supported | No |
| Bootstrap `full` | Runtime checks and orchestration | `partial` plus shell CLI and `doctor` support | Yes, Python 3.12+ |
| npm / npx | Node-based automation | npm CLI plus bundled workflow payload | Only for advanced bridge commands |
| pipx / pip | Python updater CLI | Python CLI distribution | Yes |

The user-visible skill name is `qiongli`. The installed directory is still `qiongli-workflow` for compatibility with existing clients and release artifacts. `core` is the default subject; `economics` is the first specialized subject package.

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

Claude Desktop and Claude.ai do not install third-party Claude Code plugin marketplaces. If you use Desktop or the web app and are not familiar with a code/CLI environment, use the release ZIP path instead. It requires no terminal commands:

1. Download `qiongli-claude-desktop-skill-core-<tag>.zip` or `qiongli-claude-desktop-skill-economics-<tag>.zip` from the GitHub Release assets.
2. In Claude Desktop, drag the ZIP into the Skills upload/install flow, or open `Customize > Skills`, click `+`, choose `Create skill`, then `Upload a skill`.
3. In Claude.ai, use the same `Customize > Skills` upload flow and select the same ZIP.
4. Enable the uploaded `qiongli` skill.

The release ZIP is a subject-specialized Desktop/Web package, not a reduced-quality cut. It preserves executable workflows, templates, standards, selected profiles, `skills-summary.md`, and `skills-core.md`; the economics ZIP also includes selected effective skill markdown generated with layered overlays. Detailed canonical source remains available in the Codex / Claude Code / Gemini plugin packages and source repository.

Gemini CLI still installs the local extension payload directly:

```bash
gemini extensions install ./path/to/qiongli/plugins/qiongli
```

This path does not install the shell CLI, Python bridge, or global slash-command symlinks. Use bootstrap or npm when you need those.

## Use After Install

Restart the target client after installing or upgrading. Then use the entrypoint that client exposes:

| Client | Discovery | Invocation |
|---|---|---|
| Codex | `/skills` should list `qiongli` | `$qiongli <research task>` |
| Claude Code | Plugin UI, `/plugin`, or global command discovery | `/paper`, `/lit-review`, `/paper-write`, `/code-build` |
| Gemini CLI | Extension list or global workflow discovery | `/paper`, `/lit-review`, `/paper-write`, `/code-build` |
| Shell | `qiongli check` | `qiongli doctor`, `qiongli upgrade`, `python3 -m bridges.orchestrator ...` |

Codex does not expose a custom `/qiongli` slash command. Use `/skills` to confirm the skill exists, then invoke `$qiongli`.

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
qiongli install --subject core --target all --project-dir "$PWD"
qiongli install --subject economics --target all --project-dir "$PWD"
```

For one-off runs:

```bash
npx qiongli@latest install --subject economics --target all --project-dir "$PWD"
npx qiongli@latest check --json
```

Prerelease testing remains available through the `next` dist-tag:

```bash
npx qiongli@next install --subject economics --target all --project-dir "$PWD"
```

## pipx / pip

Use pipx when you specifically want the Python-distributed updater CLI:

```bash
pipx install qiongli
qiongli install --subject core --target all
qiongli install --subject economics --target all
```

Upgrade it with:

```bash
pipx upgrade qiongli
qiongli upgrade --subject economics --target all --doctor --project-dir /path/to/project
```

`--subject` defaults to `core`. To switch a client from core to economics, rerun `install` or `upgrade` with `--subject economics`; rerun with `--subject core` to switch back. `qiongli check --json` reports the active installed subject per target, and legacy installs without a `SUBJECT` file are treated as `core`.

## What Gets Installed

Depending on the surface, Qiongli may install:

- `qiongli-workflow` skill assets under client home directories, visible to users as `qiongli`
- workflow command discovery links such as `/paper`, `/lit-review`, `/paper-write`, and `/code-build` in clients that support that discovery model
- shell commands `qiongli`, `ql`, and compatibility aliases `research-skills`, `rsk`, `rsw`
- optional project integration files when you explicitly run `qiongli init --project-dir .`

Project-local files are not written by default. The global workflow package can be used from any research workspace.

For invocation details, see [Using Agent Skills](/guide/using-agent-skills).

## Keep Versions Aligned

If you use multiple surfaces, keep plugin, global skill assets, npm payload, and Python CLI aligned:

```bash
qiongli check
qiongli upgrade --subject core --target all
```

If you move fully to native plugins and no longer need legacy global slash commands, inspect cleanup first:

```bash
qiongli clean --globals --dry-run
```
