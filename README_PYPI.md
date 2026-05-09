# qiongli-installer

`qiongli-installer` is the lightweight updater CLI for **Qiongli** (`穷理`), a contract-driven academic workflow system for Codex, Claude Code, and Gemini.

The full system name is **Qiongli Zhengche** (`穷理证澈`): Qiongli names the public research workflow, while Zhengche names the evidence-governance method that keeps claims, citations, assumptions, and output paths auditable.

## What it does

- Install or refresh global `qiongli-workflow` skill assets
- Upgrade assets to newer upstream versions
- Support `codex`, `claude`, `gemini`, or `all` targets
- Run doctor checks before/after installation

## Installation

```bash
pip install qiongli-installer
```

Or with `pipx`:

```bash
pipx install qiongli-installer
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
qiongli upgrade --project-dir /path/to/project --target all --doctor
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
qiongli upgrade --project-dir /path/to/project --target all --doctor
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
# Install from PyPI
pipx install qiongli-installer

# Upgrade assets into your project
qiongli upgrade --project-dir /path/to/project --target all --doctor
```

## Links

- Repository: https://github.com/jxpeng98/qiongli
- Issues: https://github.com/jxpeng98/qiongli/issues
