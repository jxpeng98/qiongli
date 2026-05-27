# qiongli

`qiongli` is the npm/npx installer for the Qiongli academic workflow skills.

## Install

```bash
npm install -g qiongli
qiongli install --subject core --target all
qiongli install --subject economics --target all
qiongli install --subject economics-accounting --target all
```

Update an existing global install with:

```bash
npm install -g qiongli@latest
qiongli upgrade --subject economics --target all
```

Or run without a global install:

```bash
npx qiongli@latest install --subject economics --target all
npx qiongli@latest install --subject economics --coverage focused --target all
```

For prerelease testing:

```bash
npx qiongli@next upgrade --subject economics --target all
```

The npm package contains pre-materialized `core`, `economics`, and `economics-accounting` `qiongli-workflow` subject payloads in both `complete` and `focused` coverage. It does not depend on PyPI for skill installation and does not run `postinstall`.

## Global-first update model

The npm package and the installed workflow assets are separate surfaces:

- `npm install -g qiongli@latest` updates the npm CLI and bundled payload in npm's global package location.
- `qiongli install --subject core --target all` installs the default general package into global AI client skill directories.
- `qiongli install --subject economics --target all` installs the full framework plus economics specialization.
- `qiongli install --subject economics --coverage focused --target all` installs the slimmer economics-focused package.
- `qiongli install --subject economics-accounting --target all` installs the official economics/accounting composite.
- `qiongli upgrade --subject economics --target all` is the same install flow with overwrite enabled, and is the normal command after updating the npm package.
- Project directories are not required for normal install or upgrade. Use project paths only for commands that inspect or clean a specific project, such as `qiongli doctor --cwd .` or `qiongli clean --project-dir .`.

`--subject` defaults to `core`, and `--coverage` defaults to `complete`. Complete coverage keeps the full core framework and adds the selected subject overlays and subject-specific skills. Use `--coverage focused` only when you deliberately want the slim selected subject package. Subject packages are specialized installs, not reduced-quality cuts. Switch subjects or coverage by rerunning `install` or `upgrade` with new flags. `qiongli check --json` reports the bundled payload subject/coverage and installed target subject/coverage.

Global assets are written under client home directories such as:

```text
~/.codex/skills/qiongli-workflow
~/.claude/skills/qiongli-workflow
~/.gemini/skills/qiongli-workflow
```

Advanced bridge commands such as `doctor`, `task-run`, and `team-run` use the Python runtime bundled in the npm package and require Python 3.12+ with `PyYAML`.

Runtime `--custom-dir` customization is not supported by npm in this phase. Use the source checkout and `python3 scripts/materialize_subject_package.py --custom-dir <path>` when you need local custom overlays, profiles, or skills.
