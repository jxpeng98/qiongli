import assert from 'node:assert/strict';
import test from 'node:test';

import { main } from '../lib/cli.mjs';

test('help documents all official subject options', async () => {
  let output = '';
  const exitCode = await main(['help'], {
    stdout: { write: (chunk) => { output += chunk; } },
    stderr: { write: () => {} },
  });

  assert.equal(exitCode, 0);
  assert.match(output, /--subject core\|economics\|accounting\|business\|finance\|economics-accounting/);
});

test('help documents setup command and options', async () => {
  let output = '';
  const exitCode = await main(['help'], {
    stdout: { write: (chunk) => { output += chunk; } },
    stderr: { write: () => {} },
  });

  assert.equal(exitCode, 0);
  assert.match(output, /qiongli setup \[--dry-run\] \[--project-dir \.\] \[--no-doctor\]/);
  assert.match(output, /--no-doctor/);
});

test('main dispatches setup to Python CLI runner and returns its code', async () => {
  const calls = [];
  const exitCode = await main(['setup', '--dry-run', '--project-dir', '/tmp/project', '--no-doctor'], {
    stdout: { write: () => {} },
    stderr: { write: () => {} },
    runPythonCliCommand: ({ args }) => {
      calls.push(args);
      return 7;
    },
  });

  assert.equal(exitCode, 7);
  assert.deepEqual(calls, [
    ['setup', '--dry-run', '--project-dir', '/tmp/project', '--no-doctor'],
  ]);
});

test('main dispatches mcp to Python CLI runner and returns its code', async () => {
  const calls = [];
  const exitCode = await main(['mcp', 'serve', '--transport', 'stdio'], {
    stdout: { write: () => {} },
    stderr: { write: () => {} },
    runPythonCliCommand: ({ args }) => {
      calls.push(args);
      return 9;
    },
  });

  assert.equal(exitCode, 9);
  assert.deepEqual(calls, [
    ['mcp', 'serve', '--transport', 'stdio'],
  ]);
});

test('main dispatches task-run to Python bridge runner and preserves args', async () => {
  const calls = [];
  const exitCode = await main([
    'task-run',
    '--task-id',
    'F3',
    '--paper-type',
    'empirical',
    '--topic',
    'ai-in-education',
    '--cwd',
    '/tmp/project',
    '--execution-mode',
    'duo',
    '--primary',
    'codex',
    '--reviewer',
    'claude',
  ], {
    stdout: { write: () => {} },
    stderr: { write: () => {} },
    runBridgeCommand: ({ command, args }) => {
      calls.push({ command, args });
      return 11;
    },
  });

  assert.equal(exitCode, 11);
  assert.deepEqual(calls, [{
    command: 'task-run',
    args: [
      '--task-id',
      'F3',
      '--paper-type',
      'empirical',
      '--topic',
      'ai-in-education',
      '--cwd',
      '/tmp/project',
      '--execution-mode',
      'duo',
      '--primary',
      'codex',
      '--reviewer',
      'claude',
    ],
  }]);
});

test('main injects default cwd for doctor bridge command', async () => {
  const calls = [];
  const exitCode = await main(['doctor'], {
    stdout: { write: () => {} },
    stderr: { write: () => {} },
    runBridgeCommand: ({ command, args }) => {
      calls.push({ command, args });
      return 13;
    },
  });

  assert.equal(exitCode, 13);
  assert.equal(calls[0].command, 'doctor');
  assert.deepEqual(calls[0].args, ['--cwd', '.']);
});

test('runBridgeCommand invokes bridges.orchestrator with packaged PYTHONPATH', async () => {
  const calls = [];
  const { runBridgeCommand } = await import('../lib/python-runtime.mjs');
  const exitCode = runBridgeCommand({
    packageRoot: '/pkg',
    command: 'task-run',
    args: ['--task-id', 'F3', '--cwd', '/tmp/project'],
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
      return { status: 17 };
    },
  });

  assert.equal(exitCode, 17);
  assert.equal(calls[0].cmd, 'python3');
  assert.deepEqual(calls[0].args, [
    '-m',
    'bridges.orchestrator',
    'task-run',
    '--task-id',
    'F3',
    '--cwd',
    '/tmp/project',
  ]);
});
