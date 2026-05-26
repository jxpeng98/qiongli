import assert from 'node:assert/strict';
import test from 'node:test';

import { parseArgv } from '../lib/args.mjs';

test('parseArgv parses install options and defaults', () => {
  const parsed = parseArgv(['install', '--target', 'all', '--mode', 'copy', '--project-dir', '.', '--dry-run']);

  assert.equal(parsed.command, 'install');
  assert.equal(parsed.options.target, 'all');
  assert.equal(parsed.options.mode, 'copy');
  assert.equal(parsed.options.projectDir, '.');
  assert.equal(parsed.options.dryRun, true);
  assert.equal(parsed.options.subject, 'core');
  assert.deepEqual(parsed.rest, []);
});

test('parseArgv treats upgrade as install with overwrite', () => {
  const parsed = parseArgv(['upgrade', '--target', 'codex', '--subject', 'economics']);

  assert.equal(parsed.command, 'install');
  assert.equal(parsed.options.target, 'codex');
  assert.equal(parsed.options.subject, 'economics');
  assert.equal(parsed.options.overwrite, true);
});

test('parseArgv delegates bridge commands without consuming bridge flags', () => {
  const parsed = parseArgv(['task-run', '--task-id', 'B1', '--cwd', '/tmp/project']);

  assert.equal(parsed.command, 'task-run');
  assert.deepEqual(parsed.rest, ['--task-id', 'B1', '--cwd', '/tmp/project']);
});
