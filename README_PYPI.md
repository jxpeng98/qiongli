# qiongli

`qiongli` is the lightweight updater CLI for **Qiongli** (`穷理`), a contract-driven academic workflow system for Codex, Claude Code, and Gemini.

The full system name is **Qiongli Zhengche** (`穷理证澈`): Qiongli names the public research workflow, while Zhengche names the evidence-governance method that keeps claims, citations, assumptions, and output paths auditable.

## What it does

- Install or refresh global `qiongli-workflow` skill assets
- Select the active subject package with `--subject core|economics|economics-accounting`
- Choose package coverage with `--coverage complete|focused` (`complete` is the default)
- Upgrade assets to newer upstream versions
- Support `codex`, `claude`, `gemini`, or `all` targets
- Run doctor checks before/after installation

## Installation

For a global CLI command, `pipx` is recommended:

```bash
pipx install qiongli
```

Upgrade an existing PyPI install with:

```bash
pipx upgrade qiongli
```

You can also install with `pip`:

```bash
pip install qiongli
```

If you use `pip` inside an active virtual environment, the CLI is installed into that environment. That only affects where the Python command lives; it does not make the workflow assets project-local.

## Global-first update model

Current releases separate the package install from workflow asset installation:

- `pipx install qiongli` / `pipx upgrade qiongli` updates the lightweight Python CLI.
- `qiongli install --subject core --target all` installs the default general package.
- `qiongli install --subject economics --target all` installs the full framework plus economics specialization.
- `qiongli install --subject economics --coverage focused --target all` installs the slimmer economics-focused package.
- `qiongli install --subject economics-accounting --target all` installs the official economics/accounting composite.
- `qiongli upgrade --subject economics --target all` refreshes the active global `qiongli-workflow` skill assets for Codex, Claude Code, Gemini, and Antigravity.
- `--project-dir` is not required for normal global upgrades. It only selects a project when you run `doctor`, load project-level `qiongli.toml` / `.qiongli.toml`, or explicitly write project files with `qiongli init` / `--parts project`.

Global assets are written under client home directories such as:

```text
~/.codex/skills/qiongli-workflow
~/.claude/skills/qiongli-workflow
~/.gemini/skills/qiongli-workflow
```

## CLI

Main command and aliases:

- `qiongli`
- `ql`
- `research-skills` (legacy)
- `rsk`
- `rsw`

### Check updates

```bash
qiongli check
```

### Upgrade assets

```bash
qiongli install --subject core --target all
qiongli install --subject economics --target all
qiongli install --subject economics --coverage focused --target all
qiongli install --subject economics-accounting --target all
qiongli upgrade --subject economics --target all
```

Run `doctor` only when you want to check a specific project runtime:

```bash
qiongli upgrade --subject economics --target all --doctor --project-dir /path/to/project
```

The package includes a default upstream repo (`jxpeng98/qiongli`), so `--repo` is optional.
Use `--repo` only when you want to override the default.

## Override default repo (optional)

The CLI resolves upstream repo in this order:

1. `--repo` argument
2. `QIONGLI_REPO` environment variable
3. legacy `RESEARCH_SKILLS_REPO` environment variable
4. `qiongli.toml` or `.qiongli.toml` in your project path
5. Packaged default (`qiongli/project.toml`)

### Option A: Global override

Add this to your shell profile (`~/.zshrc`, `~/.bashrc`, etc.):

```bash
export QIONGLI_REPO="<owner>/<repo>"
```

Then reload shell:

```bash
source ~/.zshrc
```

Now you can run:

```bash
qiongli check
qiongli upgrade --subject core --target all
```

### Option B: Project-level override

Create `qiongli.toml` in your project root:

```toml
[upstream]
repo = "jxpeng98/qiongli"
url = "https://github.com/<owner>/<repo>"
```

This keeps the override local to that project.

## Typical usage

```bash
# Install or update the CLI from PyPI
pipx install qiongli
pipx upgrade qiongli

# Refresh global workflow assets
qiongli install --subject core --target all
qiongli install --subject economics --target all
qiongli install --subject economics-accounting --target all

# Optional: initialize project-local files only when you want them
qiongli init --project-dir /path/to/project
```

`--subject` defaults to `core` and `--coverage` defaults to `complete`. With complete coverage, a specialized install keeps the full core framework and adds the requested subject overlays and subject-specific skills, so `qiongli install --subject economics` is full Qiongli plus economics specialization. Use `--coverage focused` only when you deliberately want the slimmer selected subject package used by Desktop/Web ZIPs. Subject packages are specialized installs, not reduced-quality cuts; switching subjects or coverage is another install/upgrade with the new flags. Older installs without a `SUBJECT_MANIFEST.json` or `SUBJECT` marker are treated as legacy `core` / `complete`.

Local customization is supported through the source materializer:

```bash
python3 scripts/materialize_subject_package.py \
  --subject economics \
  --coverage complete \
  --source . \
  --custom-dir /path/to/custom-qiongli \
  --out /tmp/qiongli-workflow
```

The custom directory can add local overlays, profiles, registry entries, and custom skill markdown for that generated output only. It does not modify the canonical source tree.

## Links

- Repository: https://github.com/jxpeng98/qiongli
- Issues: https://github.com/jxpeng98/qiongli/issues
