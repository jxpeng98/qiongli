# qiongli

`qiongli` is the npm/npx installer for the Qiongli academic workflow skills.

## Install

```bash
npm install -g qiongli
qiongli install --subject core --target all
qiongli install --subject economics --target all
```

Update an existing global install with:

```bash
npm install -g qiongli@latest
qiongli upgrade --subject economics --target all
```

Or run without a global install:

```bash
npx qiongli@latest install --subject economics --target all
```

For prerelease testing:

```bash
npx qiongli@next upgrade --subject economics --target all
```

The npm package contains pre-materialized `core` and `economics` `qiongli-workflow` subject payloads and does not depend on PyPI for skill installation.

## Global-first update model

The npm package and the installed workflow assets are separate surfaces:

- `npm install -g qiongli@latest` updates the npm CLI and bundled payload in npm's global package location.
- `qiongli install --subject core --target all` installs the default general package into global AI client skill directories.
- `qiongli install --subject economics --target all` installs the economics-specialized package.
- `qiongli upgrade --subject economics --target all` is the same install flow with overwrite enabled, and is the normal command after updating the npm package.
- Project directories are not required for normal install or upgrade. Use project paths only for commands that inspect or clean a specific project, such as `qiongli doctor --cwd .` or `qiongli clean --project-dir .`.

`--subject` defaults to `core`. Subject packages are specialized installs, not reduced-quality cuts: they share the same workflow contracts and quality gates while using selected profiles and layered overlays. Switch subjects by rerunning `install` or `upgrade` with a different `--subject`. `qiongli check --json` reports the bundled payload subject and installed target subjects.

Global assets are written under client home directories such as:

```text
~/.codex/skills/qiongli-workflow
~/.claude/skills/qiongli-workflow
~/.gemini/skills/qiongli-workflow
```

Advanced bridge commands such as `doctor`, `task-run`, and `team-run` use the Python runtime bundled in the npm package and require Python 3.12+ with `PyYAML`.
