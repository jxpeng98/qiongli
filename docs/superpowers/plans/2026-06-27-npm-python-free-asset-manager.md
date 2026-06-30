# npm Python-Free Asset Manager Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Reposition the npm package as a Python-free, cross-client asset manager for Qiongli client assets, while directing full runtime commands to pipx/pip.

**Architecture:** The npm CLI remains Node-only and manages pre-materialized workflow, skill, plugin-lite, and lightweight literature MCP assets. Python full runtime remains the source of truth for orchestration, full MCP, provider setup, doctor, task-run, team-run, and custom materialization. The npm CLI must stop implicitly invoking Python and must explain the boundary when users request full-runtime commands.

**Tech Stack:** Node ESM (`node:test`, `fs`, `path`, `readline/promises` if interactive setup is implemented), existing npm package layout under `packages/npm-qiongli`, Python `unittest` only for documentation/package contract tests.

---

## File Structure

- Modify `packages/npm-qiongli/lib/args.mjs`: define the npm-only command contract, parse `--surface`, `--subject`, `--coverage`, project flags, update/refresh aliases, and full-runtime command pass-through for rejection.
- Modify `packages/npm-qiongli/lib/cli.mjs`: remove Python bridge dispatch, route npm-supported commands to Node modules, and route full-runtime commands to a single explanatory message.
- Modify `packages/npm-qiongli/lib/installer.mjs`: add npm asset surfaces (`skills`, `plugin`, `both`), plugin-lite payload resolution, and clearer `check --json` output.
- Create `packages/npm-qiongli/lib/project.mjs`: manage `.qiongli/guidance_manifest.yaml` from Node without requiring Python.
- Create `packages/npm-qiongli/lib/runtime-message.mjs`: centralize the full-runtime-required message and exit code.
- Keep `packages/npm-qiongli/lib/python-runtime.mjs` during the first compatibility release, but stop importing it from the default CLI path.
- Modify `packages/npm-qiongli/test/args.test.mjs`: update parsing tests for npm-only semantics.
- Modify `packages/npm-qiongli/test/cli.test.mjs`: replace Python-dispatch tests with Node-only behavior and full-runtime rejection tests.
- Modify `packages/npm-qiongli/test/installer.test.mjs`: cover plugin-lite asset installation and check payload metadata.
- Create `packages/npm-qiongli/test/project.test.mjs`: cover Node-only `.qiongli/guidance_manifest.yaml` init/status/set-subject.
- Modify `packages/npm-qiongli/README.md`: document npm as Python-free asset manager, not full runtime.
- Modify `README.md`, `README_CN.md`, `docs/guide/install.md`, and `docs/zh/guide/install.md`: align npm, marketplace, and Python full runtime boundaries.
- Modify or add Python contract tests if existing docs tests assert old npm bridge wording: likely `tests/test_npm_package_contract.py`, `tests/test_cli_setup_docs.py`, and `tests/test_mcp_provider_docs.py`.

---

### Task 1: Lock the npm command contract in tests

**Files:**
- Modify: `packages/npm-qiongli/test/args.test.mjs`
- Modify: `packages/npm-qiongli/test/cli.test.mjs`

- [ ] **Step 1: Replace bridge-command expectations with npm-only command parsing tests**

In `packages/npm-qiongli/test/args.test.mjs`, replace the tests that say setup/mcp/guidance are delegated with tests shaped like this:

```js
test('parseArgv parses npm-only install surfaces', () => {
  const parsed = parseArgv([
    'install',
    '--target',
    'codex',
    '--surface',
    'plugin',
    '--subject',
    'finance',
    '--coverage',
    'complete',
    '--dry-run',
  ]);

  assert.equal(parsed.command, 'install');
  assert.equal(parsed.options.target, 'codex');
  assert.equal(parsed.options.surface, 'plugin');
  assert.equal(parsed.options.subject, 'finance');
  assert.equal(parsed.options.coverage, 'complete');
  assert.equal(parsed.options.dryRun, true);
});

test('parseArgv keeps full runtime command args for rejection', () => {
  const parsed = parseArgv(['task-run', '--task-id', 'F3', '--cwd', '/tmp/project']);

  assert.equal(parsed.command, 'task-run');
  assert.deepEqual(parsed.rest, ['--task-id', 'F3', '--cwd', '/tmp/project']);
});

test('parseArgv parses project set-subject without Python', () => {
  const parsed = parseArgv(['project', 'set-subject', 'finance', '--project-dir', '/tmp/project']);

  assert.equal(parsed.command, 'project');
  assert.equal(parsed.options.projectCommand, 'set-subject');
  assert.equal(parsed.options.projectDir, '/tmp/project');
  assert.equal(parsed.options.projectSubject, 'finance');
});
```

- [ ] **Step 2: Replace Python dispatch tests with full-runtime rejection tests**

In `packages/npm-qiongli/test/cli.test.mjs`, replace tests like `main dispatches setup to Python CLI runner` and `main dispatches task-run to Python bridge runner` with:

```js
test('full runtime commands do not invoke Python from npm CLI', async () => {
  let stderr = '';
  const exitCode = await main(['task-run', '--task-id', 'F3'], {
    stdout: { write: () => {} },
    stderr: { write: (chunk) => { stderr += chunk; } },
    runPythonCliCommand: () => {
      throw new Error('Python should not be called');
    },
    runBridgeCommand: () => {
      throw new Error('Python bridge should not be called');
    },
  });

  assert.equal(exitCode, 1);
  assert.match(stderr, /requires Qiongli full runtime/);
  assert.match(stderr, /pipx install qiongli/);
});
```

- [ ] **Step 3: Add setup boundary tests**

Use `setup` as a Node-only asset setup command, not a Python wizard:

```js
test('setup is npm asset setup and does not call Python', async (t) => {
  const packageRoot = createMinimalPackageRoot(t);
  const home = fs.mkdtempSync(path.join(os.tmpdir(), 'qiongli-home-'));
  t.after(() => fs.rmSync(home, { recursive: true, force: true }));

  const exitCode = await main(['setup', '--target', 'codex', '--dry-run'], {
    stdout: { write: () => {} },
    stderr: { write: () => {} },
    env: { HOME: home, CODEX_HOME: path.join(home, '.codex') },
    packageRoot,
    runPythonCliCommand: () => {
      throw new Error('Python should not be called');
    },
  });

  assert.equal(exitCode, 0);
});
```

- [ ] **Step 4: Run parser and CLI tests to verify they fail**

Run:

```bash
npm --prefix packages/npm-qiongli test -- test/args.test.mjs test/cli.test.mjs
```

Expected: FAIL because `parseArgv` still delegates Python commands and `cli.mjs` still calls Python bridge functions.

- [ ] **Step 5: Commit only the failing tests**

```bash
git add packages/npm-qiongli/test/args.test.mjs packages/npm-qiongli/test/cli.test.mjs
git commit -m "test(npm): define python-free cli contract"
```

---

### Task 2: Implement full-runtime rejection and npm-only command dispatch

**Files:**
- Modify: `packages/npm-qiongli/lib/args.mjs`
- Modify: `packages/npm-qiongli/lib/cli.mjs`
- Create: `packages/npm-qiongli/lib/runtime-message.mjs`

- [ ] **Step 1: Add the runtime boundary message module**

Create `packages/npm-qiongli/lib/runtime-message.mjs`:

```js
export const FULL_RUNTIME_COMMANDS = new Set([
  'doctor',
  'task-run',
  'team-run',
  'parallel',
  'chain',
  'role',
  'single',
  'code-build',
  'task-plan',
  'mcp',
  'provider',
  'guidance',
  'customize',
]);

export function writeFullRuntimeRequired(command, stderr) {
  stderr.write(`[qiongli] \`${command}\` requires Qiongli full runtime.\n`);
  stderr.write('[qiongli] Install it with: pipx install qiongli\n');
  stderr.write('[qiongli] npm is the Python-free asset manager for client skills/plugins only.\n');
}
```

- [ ] **Step 2: Update `args.mjs` command groups**

In `packages/npm-qiongli/lib/args.mjs`, define:

```js
const TARGETS = new Set(['codex', 'claude', 'antigravity', 'hermes', 'all']);
const MODES = new Set(['copy', 'link']);
const SURFACES = new Set(['skills', 'plugin', 'both']);
const NPM_COMMANDS = new Set(['install', 'setup', 'refresh', 'update', 'upgrade', 'remove', 'uninstall', 'delete', 'check', 'clean', 'runtime', 'project', 'help']);
const FULL_RUNTIME_COMMANDS = new Set(['doctor', 'task-run', 'team-run', 'parallel', 'chain', 'role', 'single', 'code-build', 'task-plan', 'mcp', 'provider', 'guidance', 'customize']);
```

Parse `--surface` with default `skills` for npm `install` in the first implementation unless Task 4 completes plugin-lite install in the same branch:

```js
const options = {
  target: 'all',
  mode: 'copy',
  surface: 'skills',
  projectDir: '.',
  overwrite: rawCommand === 'upgrade' || rawCommand === 'refresh' || rawCommand === 'update',
  dryRun: false,
  json: false,
  globals: false,
  subject: 'core',
  coverage: 'complete',
  parts: '',
};
```

For `project`, parse subcommands directly:

```js
if (rawCommand === 'project') {
  const [projectCommand = 'help', ...projectRest] = restArgs;
  options.projectCommand = projectCommand;
  for (let i = 0; i < projectRest.length; i += 1) {
    const arg = projectRest[i];
    if (arg === '--project-dir') {
      options.projectDir = requireValue(projectRest, i, arg);
      i += 1;
    } else if (arg === '--json') {
      options.json = true;
    } else if (!options.projectSubject && projectCommand === 'set-subject') {
      options.projectSubject = arg;
    } else {
      rest.push(arg);
    }
  }
  return { command: 'project', options, rest };
}
```

- [ ] **Step 3: Remove default Python dispatch from `cli.mjs`**

In `packages/npm-qiongli/lib/cli.mjs`, remove `BRIDGE_COMMANDS`, `PYTHON_CLI_COMMANDS`, and the imports from `python-runtime.mjs`. Import:

```js
import { FULL_RUNTIME_COMMANDS, writeFullRuntimeRequired } from './runtime-message.mjs';
```

Route unsupported full-runtime commands before the unknown-command branch:

```js
if (FULL_RUNTIME_COMMANDS.has(parsed.command)) {
  writeFullRuntimeRequired(parsed.command, stderr);
  return 1;
}
```

- [ ] **Step 4: Route `setup`, `refresh`, and `update` to npm asset behavior**

In `cli.mjs`, treat:

```js
const installLikeCommands = new Set(['install', 'setup', 'refresh', 'update']);
```

For these commands, call `installSkills(...)` with the parsed options. `setup` should print a short npm-specific header before the install output:

```js
if (parsed.command === 'setup') {
  stdout.write('Qiongli npm asset setup\n');
}
```

Keep `upgrade` as a compatibility alias to install with overwrite through `parseArgv`.

- [ ] **Step 5: Run tests**

Run:

```bash
npm --prefix packages/npm-qiongli test -- test/args.test.mjs test/cli.test.mjs
```

Expected: PASS for parser/CLI tests introduced in Task 1.

- [ ] **Step 6: Commit**

```bash
git add packages/npm-qiongli/lib/args.mjs packages/npm-qiongli/lib/cli.mjs packages/npm-qiongli/lib/runtime-message.mjs packages/npm-qiongli/test/args.test.mjs packages/npm-qiongli/test/cli.test.mjs
git commit -m "refactor(npm): make cli python-free by default"
```

---

### Task 3: Add Node-only project guidance manifest commands

**Files:**
- Create: `packages/npm-qiongli/lib/project.mjs`
- Create: `packages/npm-qiongli/test/project.test.mjs`
- Modify: `packages/npm-qiongli/lib/cli.mjs`

- [ ] **Step 1: Write tests for project init/status/set-subject**

Create `packages/npm-qiongli/test/project.test.mjs`:

```js
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';

import { initProject, projectStatus, setProjectSubject } from '../lib/project.mjs';

test('initProject creates a default guidance manifest', () => {
  const projectDir = fs.mkdtempSync(path.join(os.tmpdir(), 'qiongli-project-'));
  const result = initProject({ projectDir });

  const manifestPath = path.join(projectDir, '.qiongli', 'guidance_manifest.yaml');
  assert.equal(result.status, 'created');
  assert.equal(result.path, manifestPath);
  assert.match(fs.readFileSync(manifestPath, 'utf-8'), /active_subject: auto/);
});

test('setProjectSubject writes selected active subject', () => {
  const projectDir = fs.mkdtempSync(path.join(os.tmpdir(), 'qiongli-project-'));
  setProjectSubject({ projectDir, subject: 'finance' });

  const status = projectStatus({ projectDir });
  assert.equal(status.exists, true);
  assert.equal(status.activeSubject, 'finance');
});

test('projectStatus reports implicit auto when manifest is missing', () => {
  const projectDir = fs.mkdtempSync(path.join(os.tmpdir(), 'qiongli-project-'));
  const status = projectStatus({ projectDir });

  assert.equal(status.exists, false);
  assert.equal(status.activeSubject, 'auto');
});
```

- [ ] **Step 2: Implement `project.mjs` with conservative YAML writing**

Create `packages/npm-qiongli/lib/project.mjs`:

```js
import fs from 'node:fs';
import path from 'node:path';

const SUBJECTS = new Set(['auto', 'core', 'economics', 'accounting', 'business', 'finance', 'political-economy', 'geoeconomics', 'economics-accounting']);

export function manifestPath(projectDir = '.') {
  return path.join(path.resolve(projectDir), '.qiongli', 'guidance_manifest.yaml');
}

export function initProject({ projectDir = '.', dryRun = false } = {}) {
  const target = manifestPath(projectDir);
  if (fs.existsSync(target)) {
    return { status: 'exists', path: target };
  }
  if (!dryRun) {
    fs.mkdirSync(path.dirname(target), { recursive: true });
    fs.writeFileSync(target, renderManifest({ activeSubject: 'auto' }));
  }
  return { status: dryRun ? 'dry-run' : 'created', path: target };
}

export function setProjectSubject({ projectDir = '.', subject, dryRun = false } = {}) {
  const normalized = String(subject || '').trim();
  if (!SUBJECTS.has(normalized)) {
    throw new Error(`Unsupported subject: ${normalized}`);
  }
  const target = manifestPath(projectDir);
  if (!dryRun) {
    fs.mkdirSync(path.dirname(target), { recursive: true });
    fs.writeFileSync(target, renderManifest({ activeSubject: normalized }));
  }
  return { status: dryRun ? 'dry-run' : 'updated', path: target, activeSubject: normalized };
}

export function projectStatus({ projectDir = '.' } = {}) {
  const target = manifestPath(projectDir);
  if (!fs.existsSync(target)) {
    return { exists: false, path: target, activeSubject: 'auto' };
  }
  const raw = fs.readFileSync(target, 'utf-8');
  const match = raw.match(/^active_subject:\s*([A-Za-z0-9_-]+)\s*$/m);
  return { exists: true, path: target, activeSubject: match ? match[1] : 'auto' };
}

function renderManifest({ activeSubject }) {
  return [
    '# Managed by qiongli npm asset manager.',
    `active_subject: ${activeSubject}`,
    'guidance_mode: read',
    '',
  ].join('\n');
}
```

- [ ] **Step 3: Route project commands from `cli.mjs`**

In `packages/npm-qiongli/lib/cli.mjs`, import:

```js
import { initProject, projectStatus, setProjectSubject } from './project.mjs';
```

Add command handling:

```js
if (parsed.command === 'project') {
  try {
    const projectCommand = parsed.options.projectCommand;
    if (projectCommand === 'init') {
      const result = initProject({ projectDir: parsed.options.projectDir, dryRun: parsed.options.dryRun });
      stdout.write(`[qiongli] project manifest ${result.status}: ${result.path}\n`);
      return 0;
    }
    if (projectCommand === 'status') {
      const result = projectStatus({ projectDir: parsed.options.projectDir });
      if (parsed.options.json) {
        stdout.write(`${JSON.stringify(result, null, 2)}\n`);
      } else {
        stdout.write(`Project manifest: ${result.exists ? 'present' : 'implicit'}\n`);
        stdout.write(`Active subject: ${result.activeSubject}\n`);
        stdout.write(`Path: ${result.path}\n`);
      }
      return 0;
    }
    if (projectCommand === 'set-subject') {
      const result = setProjectSubject({
        projectDir: parsed.options.projectDir,
        subject: parsed.options.projectSubject,
        dryRun: parsed.options.dryRun,
      });
      stdout.write(`[qiongli] active subject ${result.status}: ${result.activeSubject}\n`);
      stdout.write(`Path: ${result.path}\n`);
      return 0;
    }
    stderr.write('[qiongli] project supports: init, status, set-subject\n');
    return 2;
  } catch (error) {
    stderr.write(`[qiongli] ${error.message}\n`);
    return 2;
  }
}
```

- [ ] **Step 4: Run project tests**

Run:

```bash
node --test packages/npm-qiongli/test/project.test.mjs
npm --prefix packages/npm-qiongli test -- test/cli.test.mjs
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add packages/npm-qiongli/lib/project.mjs packages/npm-qiongli/lib/cli.mjs packages/npm-qiongli/test/project.test.mjs
git commit -m "feat(npm): add node-only project guidance commands"
```

---

### Task 4: Add plugin-lite asset installation support

**Files:**
- Modify: `packages/npm-qiongli/lib/installer.mjs`
- Modify: `packages/npm-qiongli/test/installer.test.mjs`

- [ ] **Step 1: Add installer tests for plugin surface**

In `packages/npm-qiongli/test/installer.test.mjs`, extend `makeTempPackage()` to create minimal plugin payloads:

```js
function createPluginPayload(root, target) {
  const pluginRoot = path.join(root, 'payload', 'plugins', target, 'qiongli');
  fs.mkdirSync(path.join(pluginRoot, '.codex-plugin'), { recursive: true });
  fs.writeFileSync(path.join(pluginRoot, '.codex-plugin', 'plugin.json'), JSON.stringify({ name: 'qiongli' }));
  fs.writeFileSync(path.join(pluginRoot, 'README.md'), `${target} plugin\n`);
  return pluginRoot;
}
```

Call it for `codex`, `claude`, and `antigravity` inside `makeTempPackage()`.

Add:

```js
test('installSkills installs codex plugin-lite surface without Python', () => {
  const { root } = makeTempPackage();
  const home = fs.mkdtempSync(path.join(os.tmpdir(), 'qiongli-home-'));
  const result = installSkills({
    packageRoot: root,
    target: 'codex',
    surface: 'plugin',
    env: {
      HOME: home,
      CODEX_HOME: path.join(home, '.codex'),
      QIONGLI_NPM_PLUGIN_ROOT: path.join(home, 'plugins'),
    },
    platform: 'linux',
  });

  const pluginRoot = path.join(home, 'plugins', 'qiongli');
  assert.equal(fs.existsSync(path.join(pluginRoot, '.codex-plugin', 'plugin.json')), true);
  assert.equal(result.actions.some((action) => action.label === 'Plugin'), true);
});
```

- [ ] **Step 2: Add surface support to `installSkills`**

Change `installSkills` signature in `installer.mjs`:

```js
export function installSkills({
  packageRoot,
  target = 'all',
  mode = 'copy',
  surface = 'skills',
  overwrite = false,
  dryRun = false,
  subject = 'core',
  coverage = 'complete',
  parts = '',
  env = process.env,
  platform = process.platform,
} = {}) {
```

Set:

```js
const installSkillsSurface = surface === 'skills' || surface === 'both';
const installPluginSurface = surface === 'plugin' || surface === 'both';
```

Use `installSkillsSurface` around the existing global skill install block.

- [ ] **Step 3: Implement plugin payload resolution**

Add functions:

```js
function resolvePluginPayload({ packageRoot, target }) {
  const candidate = path.join(packageRoot, 'payload', 'plugins', target, 'qiongli');
  if (fs.existsSync(candidate)) {
    return candidate;
  }
  return '';
}

function pluginDest({ target, env = process.env }) {
  const home = env.HOME || env.USERPROFILE || os.homedir();
  if (env.QIONGLI_NPM_PLUGIN_ROOT) {
    return path.join(env.QIONGLI_NPM_PLUGIN_ROOT, 'qiongli');
  }
  if (target === 'codex') {
    return path.join(env.CODEX_HOME || path.join(home, '.codex'), 'plugins', 'qiongli');
  }
  if (target === 'claude') {
    return path.join(env.CLAUDE_CODE_HOME || path.join(home, '.claude'), 'plugins', 'qiongli');
  }
  if (target === 'antigravity') {
    return path.join(env.ANTIGRAVITY_HOME || path.join(home, '.gemini', 'antigravity'), 'plugins', 'qiongli');
  }
  return '';
}
```

For Hermes, plugin surface should return an action explaining MCP/full plugin is not a Python-free plugin asset:

```js
actions.push({ label: 'Plugin', status: 'skip', path: '<hermes>', detail: 'Hermes uses skills or full runtime MCP config' });
```

- [ ] **Step 4: Copy plugin payloads with managed overwrite rules**

Inside the plugin install block:

```js
if (installPluginSurface) {
  for (const item of selectedTargets) {
    if (item === 'hermes') {
      actions.push({ label: 'Plugin', status: 'skip', path: '<hermes>', detail: 'Hermes has no npm plugin-lite surface' });
      continue;
    }
    const src = resolvePluginPayload({ packageRoot, target: item });
    const dest = pluginDest({ target: item, env });
    if (!src) {
      actions.push({ label: 'Plugin', status: 'skip', path: dest || `<${item}>`, detail: 'plugin payload not bundled' });
      continue;
    }
    actions.push(copyManagedDirectory({ src, dest, mode, overwrite, dryRun, label: 'Plugin', detail: item }));
  }
}
```

Refactor the existing skill copy helper or add `copyManagedDirectory(...)` so skill and plugin copy behavior is shared.

- [ ] **Step 5: Run installer tests**

Run:

```bash
npm --prefix packages/npm-qiongli test -- test/installer.test.mjs
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add packages/npm-qiongli/lib/installer.mjs packages/npm-qiongli/test/installer.test.mjs
git commit -m "feat(npm): support python-free plugin asset installs"
```

---

### Task 5: Update npm README and root docs to the new positioning

**Files:**
- Modify: `packages/npm-qiongli/README.md`
- Modify: `README.md`
- Modify: `README_CN.md`
- Modify: `docs/guide/install.md`
- Modify: `docs/zh/guide/install.md`

- [ ] **Step 1: Update npm README opening**

In `packages/npm-qiongli/README.md`, replace the opening with:

```markdown
`qiongli` on npm is the Python-free asset manager for Qiongli client assets.

Use it when you want npm/npx to install, refresh, check, or remove Qiongli skills,
plugin-lite payloads, workflow prompts, templates, and lightweight literature MCP
assets across clients without configuring each marketplace.

It is not the full Qiongli runtime. Commands such as `doctor`, `task-run`,
`team-run`, `mcp serve`, `provider setup`, and `customize` require:

```bash
pipx install qiongli
```
```

- [ ] **Step 2: Replace bridge/runtime wording**

Remove wording that says:

```markdown
`qiongli setup` uses the bundled Python bridge
Advanced bridge commands such as `setup`, `doctor`, `task-run`, and `team-run`
The npm launcher also delegates MCP commands to the bundled Python bridge
```

Replace with:

```markdown
`qiongli setup` is npm asset setup. It installs Python-free client assets only.
Full runtime commands exit with guidance to install the Python runtime with pipx.
```

- [ ] **Step 3: Update install guide comparison table**

In English and Chinese install guides, set npm row to:

```markdown
| npm / npx | Python-free cross-client asset management, scripts, dotfiles, CI | npm CLI plus pre-materialized skills/plugin-lite payloads/light literature MCP assets | No |
```

For Python full runtime row, make clear it owns:

```markdown
doctor, task-run, team-run, full MCP server, provider setup, custom overlays
```

- [ ] **Step 4: Add explicit boundary examples**

Add English examples:

```markdown
Use npm for assets:

```bash
npm install -g qiongli
qiongli install --target all
qiongli check --json
```

Use pipx for full runtime:

```bash
pipx install qiongli
qiongli doctor --cwd .
qiongli mcp serve --transport stdio
```
```

Add equivalent Chinese examples in `README_CN.md` and `docs/zh/guide/install.md`.

- [ ] **Step 5: Run docs tests that protect install wording**

Run:

```bash
python3 -m unittest tests.test_npm_package_contract tests.test_cli_setup_docs tests.test_mcp_provider_docs tests.test_release_downloads
npm run docs:build
```

Expected: PASS after updating assertions for the new npm boundary if needed.

- [ ] **Step 6: Commit**

```bash
git add packages/npm-qiongli/README.md README.md README_CN.md docs/guide/install.md docs/zh/guide/install.md tests/test_npm_package_contract.py tests/test_cli_setup_docs.py tests/test_mcp_provider_docs.py
git commit -m "docs(npm): clarify python-free asset manager boundary"
```

---

### Task 6: Update Python-side contract tests for npm boundary

**Files:**
- Modify: `tests/test_npm_package_contract.py`
- Modify: `tests/test_distribution_materialization_docs.py` if it asserts old npm wording
- Modify: `tests/test_cli_setup_docs.py` if it expects npm setup to use Python bridge

- [ ] **Step 1: Find old bridge assumptions**

Run:

```bash
rg -n "Python bridge|bundled Python|npm setup|qiongli setup|task-run|mcp serve|npm payload|postinstall|project set-subject" tests docs README.md README_CN.md packages/npm-qiongli/README.md
```

Expected: identify all tests/docs that still describe npm as a Python bridge.

- [ ] **Step 2: Change npm contract tests to assert Python-free behavior**

In `tests/test_npm_package_contract.py`, add or update assertions like:

```python
def test_npm_docs_define_python_free_asset_manager_boundary(self) -> None:
    readme = (REPO_ROOT / "packages" / "npm-qiongli" / "README.md").read_text(encoding="utf-8")

    self.assertIn("Python-free asset manager", readme)
    self.assertIn("pipx install qiongli", readme)
    self.assertIn("does not run `postinstall`", readme)
    self.assertNotIn("delegates MCP commands to the bundled Python bridge", readme)
```

If an existing test asserts `qiongli setup` delegates to Python, replace it with:

```python
def test_npm_setup_is_asset_setup_not_full_runtime(self) -> None:
    readme = (REPO_ROOT / "packages" / "npm-qiongli" / "README.md").read_text(encoding="utf-8")

    self.assertIn("`qiongli setup` is npm asset setup", readme)
    self.assertIn("Full runtime commands", readme)
```

- [ ] **Step 3: Update setup docs tests**

If `tests/test_cli_setup_docs.py` currently requires npm Python bridge wording, replace that assertion with:

```python
self.assertIn("npm asset setup", content)
self.assertIn("pipx install qiongli", content)
self.assertIn("full runtime", content)
```

- [ ] **Step 4: Run targeted Python tests**

Run:

```bash
python3 -m unittest tests.test_npm_package_contract tests.test_cli_setup_docs tests.test_distribution_materialization_docs
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add tests/test_npm_package_contract.py tests/test_cli_setup_docs.py tests/test_distribution_materialization_docs.py
git commit -m "test(npm): enforce python-free asset manager docs"
```

---

### Task 7: Remove default package references to Python bridge from npm path

**Files:**
- Modify: `packages/npm-qiongli/package.json`
- Modify: `packages/npm-qiongli/test/python-runtime.test.mjs`
- Modify: release/materialization tests only if they assert python-runtime must be used by npm CLI

- [ ] **Step 1: Keep `python-runtime` bundled only as transitional data or stop shipping it**

For the first non-breaking release, do not delete `python-runtime` from generated package payload unless package contract tests are updated and release risk is accepted. Instead, make it unused by the npm CLI.

In `packages/npm-qiongli/package.json`, change the description:

```json
"description": "Python-free Qiongli client asset manager for npm and npx."
```

- [ ] **Step 2: Add a CLI test that prevents reintroducing Python bridge imports**

In `packages/npm-qiongli/test/cli.test.mjs`, add:

```js
test('cli module does not import python-runtime by default', () => {
  const cliSource = fs.readFileSync(new URL('../lib/cli.mjs', import.meta.url), 'utf-8');

  assert.doesNotMatch(cliSource, /python-runtime\.mjs/);
  assert.doesNotMatch(cliSource, /runPythonCliCommand/);
  assert.doesNotMatch(cliSource, /runBridgeCommand/);
});
```

- [ ] **Step 3: Adjust python-runtime tests**

Leave `packages/npm-qiongli/test/python-runtime.test.mjs` in place only if the module is still shipped for transition. Rename test descriptions to clarify it is legacy/transitional:

```js
test('legacy python-runtime helper still builds isolated PYTHONPATH when directly imported', () => {
  // existing assertion body
});
```

If the implementation removes `python-runtime.mjs` in a breaking branch, delete this test file and update `package.json` exports/files accordingly in the same commit.

- [ ] **Step 4: Run npm tests**

Run:

```bash
npm --prefix packages/npm-qiongli test
```

Expected: all Node tests pass.

- [ ] **Step 5: Commit**

```bash
git add packages/npm-qiongli/package.json packages/npm-qiongli/test/cli.test.mjs packages/npm-qiongli/test/python-runtime.test.mjs
git commit -m "chore(npm): prevent implicit python bridge usage"
```

---

### Task 8: Full verification and release-readiness checks

**Files:**
- No code changes expected unless verification finds a defect.

- [ ] **Step 1: Run npm package tests**

Run:

```bash
npm --prefix packages/npm-qiongli test
```

Expected: all Node tests pass.

- [ ] **Step 2: Run Python contract tests affected by npm/docs**

Run:

```bash
python3 -m unittest tests.test_npm_package_contract tests.test_cli_setup_docs tests.test_release_downloads tests.test_distribution_materialization_docs tests.test_mcp_provider_docs
```

Expected: all tests pass.

- [ ] **Step 3: Run materialization and package payload tests**

Run:

```bash
python3 -m unittest tests.test_materialize_distribution_payloads tests.test_distribution_payloads tests.test_plugin_distribution_contract tests.test_plugin_manifests
```

Expected: all tests pass.

- [ ] **Step 4: Build docs**

Run:

```bash
npm run docs:build
```

Expected: VitePress build completes. Existing syntax-highlighting fallback warnings are acceptable if no new warnings indicate broken links or failed pages.

- [ ] **Step 5: Run whitespace check**

Run:

```bash
git diff --check
```

Expected: no output.

- [ ] **Step 6: Run release-preflight smoke subset if preparing a release**

Run:

```bash
python3 scripts/validate_research_standard.py --strict
./scripts/release_ready.sh --version 1.13.0 --from-tag v1.12.0 --skip-bump
```

Expected: validator summary reports zero failures; release-ready completes for the recommended `1.13.0` compatibility release. If the implementation removes transitional Python bridge files from the npm package, replace this with the agreed `2.0.0` release command during release prep.

- [ ] **Step 7: Commit final verification fixes if needed**

Only if verification required edits:

```bash
git add packages/npm-qiongli README.md README_CN.md docs/guide/install.md docs/zh/guide/install.md tests/test_npm_package_contract.py tests/test_cli_setup_docs.py tests/test_distribution_materialization_docs.py tests/test_mcp_provider_docs.py
git commit -m "fix(npm): align asset manager verification"
```

---

## Release Recommendation

Ship this as `1.13.0` if `python-runtime` remains bundled but unused by default and full-runtime commands return guidance instead of dispatching Python.

Ship as `2.0.0` only if the npm package removes `python-runtime` entirely or changes existing command behavior in a way that cannot be treated as clarification plus deprecation.

The safer path is:

1. `1.13.0`: npm becomes Python-free by default, docs and tests enforce the boundary, transitional files may remain.
2. Later `2.0.0`: remove transitional Python bridge files from the npm package and shrink the package payload.

---

## Self-Review

- Spec coverage: covers npm positioning, marketplace distinction, Python full runtime distinction, command contract, Node-only project commands, plugin-lite install, docs, tests, and release impact.
- Placeholder scan: no implementation step relies on unspecified "add tests" or "handle errors"; each task names files, test commands, and concrete snippets.
- Type consistency: command names use `install`, `setup`, `refresh`, `update`, `project`, and full-runtime rejection consistently; asset surface names use `skills`, `plugin`, and `both`.
