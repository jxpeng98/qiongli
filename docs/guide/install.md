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

The user-visible skill name is `qiongli`. The installed directory is still `qiongli-workflow` for compatibility with existing clients and release artifacts. `core` is the default subject, so the default install is `core/complete`. Specialized CLI/npm installs default to `coverage=complete`, meaning full Qiongli plus the requested subject specialization.

## Native Plugin And Extension

Use this when you only need Qiongli inside one supported client.

Codex installs through the shared [Skillsplace](https://github.com/jxpeng98/skillsplace) marketplace:

```bash
codex plugin marketplace add jxpeng98/skillsplace --ref main
codex plugin marketplace list
```

Then install or enable `qiongli` from the Codex plugin UI for the default core package. Subject entries such as `qiongli-economics`, `qiongli-accounting`, `qiongli-business`, `qiongli-finance`, `qiongli-political-economy`, `qiongli-geoeconomics`, and `qiongli-economics-accounting` install the corresponding `subject/complete` package from the same marketplace.

Claude Code uses the same Skillsplace catalog:

```bash
claude plugin marketplace add jxpeng98/skillsplace@main
claude plugin install qiongli@skillsplace
# Subject-specialized install:
claude plugin install qiongli-economics@skillsplace
```

Inside an interactive Claude Code session, use:

```text
/plugin marketplace add jxpeng98/skillsplace@main
/plugin install qiongli@skillsplace
/plugin install qiongli-economics@skillsplace
```

Claude Desktop and Claude.ai do not install third-party Claude Code plugin marketplaces. If you use Desktop or the web app and are not familiar with a code/CLI environment, use the release ZIP path instead. It requires no terminal commands:

1. Download `qiongli-claude-desktop-skill-core-<tag>.zip`, `qiongli-claude-desktop-skill-economics-<tag>.zip`, `qiongli-claude-desktop-skill-business-<tag>.zip`, `qiongli-claude-desktop-skill-finance-<tag>.zip`, `qiongli-claude-desktop-skill-political-economy-<tag>.zip`, `qiongli-claude-desktop-skill-geoeconomics-<tag>.zip`, or `qiongli-claude-desktop-skill-economics-accounting-<tag>.zip` from the GitHub Release assets. Public Desktop ZIP subjects in this phase are `core`, `economics`, `business`, `finance`, `political-economy`, `geoeconomics`, and `economics-accounting`; there is no standalone accounting Desktop ZIP yet.
2. In Claude Desktop, drag the ZIP into the Skills upload/install flow, or open `Customize > Skills`, click `+`, choose `Create skill`, then `Upload a skill`.
3. In Claude.ai, use the same `Customize > Skills` upload flow and select the same ZIP.
4. Enable the uploaded `qiongli` skill.

The release ZIP uses `coverage=focused` to stay under the current 180-file upload budget. It is a subject-specialized Desktop/Web package, not a reduced-quality cut. It preserves executable workflows, templates, standards, selected profiles, `skills-summary.md`, and `skills-core.md`; specialized ZIPs also include selected effective skill markdown generated with layered overlays. Detailed canonical source remains available through CLI/npm `coverage=complete`, the Codex / Claude Code / Gemini plugin packages, and the source repository.

Literature provider keys are configured outside the Desktop ZIP. CLI, Codex, and Claude Code users can run `qiongli provider setup`, then verify `provider_connected` or `strategy_only` with `qiongli provider doctor`. Desktop users who also install the local CLI can use the provider companion path: run `qiongli companion setup`, verify with `qiongli companion doctor --json`, and share `qiongli companion export-status --json` when the Desktop workflow needs an auditable status snapshot. The Desktop/Web ZIP is a skill-only package: it cannot store API keys or execute OpenAlex, Semantic Scholar, Crossref, or PubMed calls by itself. Desktop users need a provider companion or platform-native search before claiming `provider_connected`; otherwise record the run as `strategy_only` and treat platform search or user-supplied corpus as the evidence source.

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
qiongli install --subject accounting --target all --project-dir "$PWD"
qiongli install --subject economics-accounting --target all --project-dir "$PWD"
qiongli install --subject economics --coverage focused --target all --project-dir "$PWD"
```

For one-off runs:

```bash
npx qiongli@latest install --subject economics --target all --project-dir "$PWD"
npx qiongli@latest install --subject economics --coverage focused --target all --project-dir "$PWD"
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
qiongli install --target all
qiongli install --subject economics --target all
qiongli install --subject accounting --target all
qiongli install --subject political-economy --target all
qiongli install --subject geoeconomics --target all
qiongli install --subject economics-accounting --target all
```

Upgrade it with:

```bash
pipx upgrade qiongli
qiongli upgrade --subject accounting --target all --doctor --project-dir /path/to/project
```

`--subject` defaults to `core`, and `--coverage` defaults to `complete`. Use complete when you are unsure: `--subject economics`, `--subject business`, `--subject finance`, `--subject political-economy`, and `--subject geoeconomics` mean complete specialized installs, not reduced packages, and `--subject accounting` means `accounting/complete`, full framework plus accounting specialization. Use `--coverage focused` for deliberate slim installs and Desktop/Web-equivalent packages. Current official subjects are `core`, `economics`, `accounting`, `business`, `finance`, `political-economy`, `geoeconomics`, and the named composite `economics-accounting`; `political-economy` and `geoeconomics` are independent subject choices, not a composite. Official composite subjects are not arbitrary comma-separated stacking. To switch a client from one subject or coverage to another, rerun `install` or `upgrade` with new flags. `qiongli check --json` reports the active installed subject and coverage per target; legacy installs without a `SUBJECT_MANIFEST.json` or `SUBJECT` file are treated as `core` / `complete`.

Create a custom scaffold before materializing local overlays:

```bash
qiongli customize --subject economics --name my-econ-lab --out ./qiongli-custom/econ-lab
```

Local custom overlays are supported by the source materializer:

```bash
python3 scripts/materialize_subject_package.py \
  --subject economics \
  --coverage complete \
  --source . \
  --custom-dir /path/to/custom-qiongli \
  --out /tmp/qiongli-workflow
```

Use this when you need local overlays, profiles, registry entries, or custom skill markdown. Custom overlays affect generated output only and do not rewrite canonical source files. `qiongli customize` plus `--custom-dir` materialization is for the Python/source checkout workflow; npm runtime installs pre-generated payloads and do not accept `--custom-dir` in this phase.

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

If you move fully to native plugins and no longer need legacy global skill directories or slash discovery, inspect cleanup first:

```bash
qiongli clean --globals --dry-run
```
