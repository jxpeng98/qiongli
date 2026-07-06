import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';

import { main } from '../lib/cli.mjs';

test('help describes the npm asset manager and full runtime boundary', async () => {
  const { exitCode, stdout } = await runMain(['help']);

  assert.equal(exitCode, 0);
  assert.match(stdout, /Qiongli npm asset manager/);
  assert.match(stdout, /qiongli setup --target codex \[--surface skills\] \[--dry-run\]/);
  assert.match(stdout, /qiongli project init --project-dir \. \[--dry-run\] \[--json\]/);
  assert.match(stdout, /qiongli project status --project-dir \. \[--json\]/);
  assert.match(stdout, /qiongli project set-subject <subject> --project-dir \. \[--dry-run\] \[--json\]/);
  assert.match(stdout, /Full runtime commands require `pipx install qiongli`/);
  assert.match(stdout, /doctor\|task-run\|team-run\|parallel\|chain\|role\|single\|code-build\|task-plan\|mcp\|provider\|guidance\|customize\|init\|align/);
  assert.match(stdout, /Default core installs adaptive runtime subject refinement/);
  assert.match(stdout, /Non-core subjects are advanced overrides for pre-materialized packages/);
});

test('cli source does not import or call the Python runtime bridge', () => {
  const source = fs.readFileSync(new URL('../lib/cli.mjs', import.meta.url), 'utf-8');

  assert.doesNotMatch(source, /python-runtime\.mjs/);
  assert.doesNotMatch(source, /runPythonCliCommand/);
  assert.doesNotMatch(source, /runBridgeCommand/);
});

test('main runs setup as Node-only asset setup', async (t) => {
  const packageRoot = createMinimalPackageRoot(t);
  const home = fs.mkdtempSync(path.join(os.tmpdir(), 'qiongli-home-'));
  t.after(() => fs.rmSync(home, { recursive: true, force: true }));

  const { exitCode, stdout, stderr } = await runMain(['setup', '--dry-run', '--target', 'codex'], {
    env: {
      HOME: home,
      CODEX_HOME: path.join(home, '.codex'),
    },
    packageRoot,
    runPythonCliCommand: failPythonRunner,
    runBridgeCommand: failPythonRunner,
  });

  assert.equal(exitCode, 0);
  assert.equal(stderr, '');
  assert.match(stdout, /Qiongli npm asset manager/);
  assert.match(stdout, /\[ok\] Skill -> .*qiongli-workflow/);
});

test('main describes default core install as adaptive in npm output', async (t) => {
  const packageRoot = createMinimalPackageRoot(t);
  const home = fs.mkdtempSync(path.join(os.tmpdir(), 'qiongli-home-'));
  t.after(() => fs.rmSync(home, { recursive: true, force: true }));

  const { exitCode, stdout, stderr } = await runMain(['install', '--dry-run', '--target', 'codex'], {
    env: {
      HOME: home,
      CODEX_HOME: path.join(home, '.codex'),
    },
    packageRoot,
    runPythonCliCommand: failPythonRunner,
    runBridgeCommand: failPythonRunner,
  });

  assert.equal(exitCode, 0);
  assert.equal(stderr, '');
  assert.match(stdout, /source subject: core \(adaptive; active_subject defaults to auto\)/);
});

test('main describes non-core npm subject installs as advanced overrides', async (t) => {
  const packageRoot = createMinimalPackageRoot(t);
  createSubjectPayload(packageRoot, 'economics');
  const home = fs.mkdtempSync(path.join(os.tmpdir(), 'qiongli-home-'));
  t.after(() => fs.rmSync(home, { recursive: true, force: true }));

  const { exitCode, stdout, stderr } = await runMain([
    'upgrade',
    '--subject',
    'economics',
    '--dry-run',
    '--target',
    'codex',
  ], {
    env: {
      HOME: home,
      CODEX_HOME: path.join(home, '.codex'),
    },
    packageRoot,
    runPythonCliCommand: failPythonRunner,
    runBridgeCommand: failPythonRunner,
  });

  assert.equal(exitCode, 0);
  assert.equal(stderr, '');
  assert.match(stdout, /source subject: economics \(advanced override\)/);
});

test('main treats update and refresh as overwrite install refreshes', async (t) => {
  const packageRoot = createMinimalPackageRoot(t);
  const home = fs.mkdtempSync(path.join(os.tmpdir(), 'qiongli-home-'));
  const codexSkill = path.join(home, '.codex', 'skills', 'qiongli-workflow');
  fs.mkdirSync(codexSkill, { recursive: true });
  fs.writeFileSync(path.join(codexSkill, 'USER_FILE'), 'unmanaged\n');
  t.after(() => fs.rmSync(home, { recursive: true, force: true }));

  for (const command of ['update', 'refresh']) {
    const { exitCode, stdout } = await runMain([command, '--target', 'codex', '--dry-run'], {
      env: {
        HOME: home,
        CODEX_HOME: path.join(home, '.codex'),
      },
      packageRoot,
      runPythonCliCommand: failPythonRunner,
      runBridgeCommand: failPythonRunner,
    });

    assert.equal(exitCode, 0);
    assert.match(stdout, /\[ok\] Skill -> .*qiongli-workflow/);
  }
});

test('main rejects legacy update flags with full-runtime guidance', async () => {
  const { exitCode, stdout, stderr } = await runMain(['update', '--yes', '--channel', 'next']);

  assert.equal(exitCode, 1);
  assert.equal(stdout, '');
  assert.match(stderr, /legacy update flags are not supported by the npm asset manager/);
  assert.match(stderr, /Use `qiongli install`, `qiongli refresh`, or `qiongli upgrade` for npm asset refreshes/);
  assert.match(stderr, /Use `qiongli self-update` from the full runtime after `pipx install qiongli`/);
});

test('main rejects legacy update flags passed with equals syntax', async () => {
  const { exitCode, stdout, stderr } = await runMain(['update', '--channel=next']);

  assert.equal(exitCode, 1);
  assert.equal(stdout, '');
  assert.match(stderr, /legacy update flags are not supported by the npm asset manager/);
});

test('main treats npm upgrade as content-only install refresh', async (t) => {
  const packageRoot = createMinimalPackageRoot(t);
  const home = fs.mkdtempSync(path.join(os.tmpdir(), 'qiongli-home-'));
  t.after(() => fs.rmSync(home, { recursive: true, force: true }));

  const { exitCode } = await runMain(['upgrade', '--target', 'codex', '--dry-run'], {
    env: {
      HOME: home,
      CODEX_HOME: path.join(home, '.codex'),
    },
    packageRoot,
    runPythonCliCommand: failPythonRunner,
    runBridgeCommand: failPythonRunner,
  });

  assert.equal(exitCode, 0);
});

test('main rejects unsupported npm asset-manager flags instead of ignoring them', async () => {
  for (const argv of [
    ['upgrade', '--ref', 'v1.11.0', '--target', 'codex'],
    ['setup', '--no-doctor'],
    ['install', '--profile', 'full'],
  ]) {
    const { exitCode, stdout, stderr } = await runMain(argv);

    assert.equal(exitCode, 2, argv.join(' '));
    assert.equal(stdout, '');
    assert.match(stderr, /unsupported npm asset-manager argument/);
    assert.match(stderr, /pipx install qiongli/);
  }
});

test('full runtime commands reject with pipx guidance', async () => {
  const commands = [
    ['doctor'],
    ['task-run', '--task-id', 'F3'],
    ['team-run'],
    ['parallel'],
    ['chain'],
    ['role'],
    ['single'],
    ['code-build'],
    ['task-plan'],
    ['mcp', 'serve', '--transport', 'stdio'],
    ['provider', 'setup'],
    ['guidance', 'init'],
    ['customize'],
    ['init', '--project-dir', '.'],
    ['align', '--repo', 'owner/repo'],
  ];

  for (const argv of commands) {
    const { exitCode, stdout, stderr } = await runMain(argv, {
      runPythonCliCommand: failPythonRunner,
      runBridgeCommand: failPythonRunner,
    });

    assert.equal(exitCode, 1, argv[0]);
    assert.equal(stdout, '');
    assert.match(stderr, new RegExp(`${argv[0]}.*requires Qiongli full runtime`));
    assert.match(stderr, /pipx install qiongli/);
  }
});

test('self-update rejects with full runtime guidance instead of Python dispatch', async () => {
  const { exitCode, stderr } = await runMain(['self-update', '--channel', 'next', '--yes'], {
    runPythonCliCommand: failPythonRunner,
    runBridgeCommand: failPythonRunner,
  });

  assert.equal(exitCode, 1);
  assert.match(stderr, /self-update.*requires Qiongli full runtime/);
  assert.match(stderr, /pipx install qiongli/);
});

test('check reports npm asset-manager status without Python bridge diagnostics', async (t) => {
  const packageRoot = createMinimalPackageRoot(t);
  const home = fs.mkdtempSync(path.join(os.tmpdir(), 'qiongli-home-'));
  t.after(() => fs.rmSync(home, { recursive: true, force: true }));

  const { exitCode, stdout } = await runMain(['check', '--json'], {
    env: {
      HOME: home,
      CODEX_HOME: path.join(home, '.codex'),
    },
    packageRoot,
    runPythonCliCommand: failPythonRunner,
    runBridgeCommand: failPythonRunner,
  });
  const payload = JSON.parse(stdout);

  assert.equal(exitCode, 0);
  assert.equal(payload.npm_cli.role, 'asset-manager');
  assert.equal(payload.npm_cli.python_free, true);
  assert.equal(payload.npm_cli.full_runtime_install, 'pipx install qiongli');
  assert.deepEqual(payload.python_bridge, {
    bundled: false,
    deprecated: true,
    managed_by_npm: false,
    message: 'Not bundled or managed by this npm asset-manager path.',
  });
});

test('runtime doctor remains a Node-only npm asset-manager diagnostic', async () => {
  const { exitCode, stdout, stderr } = await runMain(['runtime', 'doctor'], {
    runPythonCliCommand: failPythonRunner,
    runBridgeCommand: failPythonRunner,
  });

  assert.equal(exitCode, 0);
  assert.equal(stderr, '');
  assert.match(stdout, /\[ok\] Qiongli npm asset manager is installed/);
  assert.match(stdout, /Full runtime: pipx install qiongli/);
});

test('project status runs as a Node-only npm-lite command', async (t) => {
  const projectDir = fs.mkdtempSync(path.join(os.tmpdir(), 'qiongli-project-'));
  t.after(() => fs.rmSync(projectDir, { recursive: true, force: true }));

  const { exitCode, stdout, stderr } = await runMain(['project', 'status', '--project-dir', projectDir, '--json'], {
    runPythonCliCommand: failPythonRunner,
    runBridgeCommand: failPythonRunner,
  });
  const payload = JSON.parse(stdout);

  assert.equal(exitCode, 0);
  assert.equal(stderr, '');
  assert.equal(payload.action, 'status');
  assert.equal(payload.state.exists, false);
  assert.equal(payload.state.manifest.active_subject, 'auto');
});

async function runMain(argv, options = {}) {
  let stdout = '';
  let stderr = '';
  const exitCode = await main(argv, {
    stdout: { write: (chunk) => { stdout += chunk; } },
    stderr: { write: (chunk) => { stderr += chunk; } },
    ...options,
  });
  return { exitCode, stdout, stderr };
}

function failPythonRunner() {
  throw new Error('Python runner should not be called from npm CLI default dispatch');
}

function createMinimalPackageRoot(t) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'qiongli-package-'));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));

  const workflow = path.join(root, 'payload', 'qiongli-workflow');
  const codexPlugin = path.join(root, 'payload', 'plugins', 'codex', 'qiongli');
  fs.mkdirSync(workflow, { recursive: true });
  fs.mkdirSync(codexPlugin, { recursive: true });
  fs.writeFileSync(path.join(root, 'package.json'), JSON.stringify({ name: 'qiongli', version: '0.0.0-test' }));
  fs.writeFileSync(path.join(workflow, 'SKILL.md'), '---\nname: qiongli-workflow\n---\n');
  fs.writeFileSync(path.join(workflow, 'VERSION'), '0.0.0-test\n');
  fs.writeFileSync(path.join(workflow, 'SUBJECT'), 'core\n');
  fs.writeFileSync(path.join(codexPlugin, 'manifest.json'), `${JSON.stringify({ name: 'qiongli', runtime: 'node-lite' })}\n`);
  return root;
}

function createSubjectPayload(root, subject) {
  const workflow = path.join(root, 'payload', 'subjects', subject, 'complete', 'qiongli-workflow');
  fs.mkdirSync(workflow, { recursive: true });
  fs.writeFileSync(path.join(workflow, 'SKILL.md'), '---\nname: qiongli-workflow\n---\n');
  fs.writeFileSync(path.join(workflow, 'VERSION'), '0.0.0-test\n');
  fs.writeFileSync(path.join(workflow, 'SUBJECT'), `${subject}\n`);
}
