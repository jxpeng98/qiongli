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
