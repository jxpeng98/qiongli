import assert from 'node:assert/strict';
import path from 'node:path';
import test from 'node:test';

import { checkPythonRuntime, runPythonCliCommand } from '../lib/python-runtime.mjs';

test('checkPythonRuntime reports missing Python with install guidance', () => {
  const result = checkPythonRuntime({
    candidates: ['python3', 'python'],
    spawnSync: () => ({ status: 127, stdout: '', stderr: 'not found' }),
  });

  assert.equal(result.ok, false);
  assert.match(result.message, /Python runtime not found/);
  assert.match(result.hint, /pipx install qiongli/);
});

test('checkPythonRuntime requires Python 3.12 and PyYAML', () => {
  const calls = [];
  const result = checkPythonRuntime({
    candidates: ['python3'],
    spawnSync: (cmd, args) => {
      calls.push([cmd, args.join(' ')]);
      if (args.join(' ').includes('sys.version_info')) {
        return { status: 0, stdout: '3.12.2\n', stderr: '' };
      }
      return { status: 1, stdout: '', stderr: 'No module named yaml' };
    },
  });

  assert.equal(result.ok, false);
  assert.equal(result.python, 'python3');
  assert.match(result.message, /PyYAML/);
  assert.match(result.hint, /python3 -m pip install PyYAML/);
  assert.equal(calls.length, 2);
});

test('checkPythonRuntime accepts Python 3.12 with PyYAML', () => {
  const result = checkPythonRuntime({
    candidates: ['python3'],
    spawnSync: (_cmd, args) => {
      if (args.join(' ').includes('sys.version_info')) {
        return { status: 0, stdout: '3.12.9\n', stderr: '' };
      }
      return { status: 0, stdout: '', stderr: '' };
    },
  });

  assert.equal(result.ok, true);
  assert.equal(result.python, 'python3');
  assert.equal(result.version, '3.12.9');
});

test('runPythonCliCommand invokes qiongli.cli with packaged PYTHONPATH', () => {
  const calls = [];
  const exitCode = runPythonCliCommand({
    packageRoot: '/pkg',
    args: ['setup', '--dry-run', '--project-dir', '/tmp/project', '--no-doctor'],
    cwd: '/repo',
    env: { PYTHONPATH: '/existing' },
    stdio: 'pipe',
    checkRuntime: () => ({
      ok: true,
      python: 'python3',
      version: '3.12.9',
      message: 'ready',
      hint: '',
    }),
    spawnSync: (cmd, args, options) => {
      calls.push({ cmd, args, options });
      return { status: 5 };
    },
  });

  assert.equal(exitCode, 5);
  assert.equal(calls.length, 1);
  assert.equal(calls[0].cmd, 'python3');
  assert.deepEqual(calls[0].args, [
    '-m',
    'qiongli.cli',
    'setup',
    '--dry-run',
    '--project-dir',
    '/tmp/project',
    '--no-doctor',
  ]);
  assert.equal(calls[0].options.cwd, '/repo');
  assert.equal(calls[0].options.stdio, 'pipe');
  assert.equal(calls[0].options.env.PYTHONPATH, `/pkg/python-runtime${path.delimiter}/existing`);
});
