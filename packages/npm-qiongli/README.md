# qiongli

`qiongli` is the npm/npx installer for the Qiongli academic workflow skills.

## Install

```bash
npm install -g qiongli
qiongli install --target all
```

Or run without a global install:

```bash
npx qiongli@beta install --target all
```

The npm package contains the complete `qiongli-workflow` skill payload and does not depend on PyPI for skill installation.

Advanced bridge commands such as `doctor`, `task-run`, and `team-run` use the Python runtime bundled in the npm package and require Python 3.12+ with `PyYAML`.
