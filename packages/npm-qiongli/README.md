# qiongli

`qiongli` is the npm/npx installer for the Qiongli academic workflow skills.

## Install

```bash
npm install -g qiongli
qiongli install --target all
```

Update an existing global install with:

```bash
npm install -g qiongli@latest
qiongli upgrade --target all
```

Or run without a global install:

```bash
npx qiongli@latest install --target all
```

For prerelease testing:

```bash
npx qiongli@next upgrade --target all
```

The npm package contains the complete `qiongli-workflow` skill payload and does not depend on PyPI for skill installation.

## Global-first update model

The npm package and the installed workflow assets are separate surfaces:

- `npm install -g qiongli@latest` updates the npm CLI and bundled payload in npm's global package location.
- `qiongli install --target all` installs the bundled `qiongli-workflow` payload into global AI client skill directories.
- `qiongli upgrade --target all` is the same install flow with overwrite enabled, and is the normal command after updating the npm package.
- Project directories are not required for normal install or upgrade. Use project paths only for commands that inspect or clean a specific project, such as `qiongli doctor --cwd .` or `qiongli clean --project-dir .`.

Global assets are written under client home directories such as:

```text
~/.codex/skills/qiongli-workflow
~/.claude/skills/qiongli-workflow
~/.gemini/skills/qiongli-workflow
```

Advanced bridge commands such as `doctor`, `task-run`, and `team-run` use the Python runtime bundled in the npm package and require Python 3.12+ with `PyYAML`.
