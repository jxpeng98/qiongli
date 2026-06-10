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
  assert.equal(parsed.options.coverage, 'complete');
  assert.deepEqual(parsed.rest, []);
});

test('parseArgv treats upgrade as install with overwrite', () => {
  const parsed = parseArgv(['upgrade', '--target', 'codex', '--subject', 'economics', '--coverage', 'focused']);

  assert.equal(parsed.command, 'install');
  assert.equal(parsed.options.target, 'codex');
  assert.equal(parsed.options.subject, 'economics');
  assert.equal(parsed.options.coverage, 'focused');
  assert.equal(parsed.options.overwrite, true);
});

test('parseArgv accepts hermes as an install target', () => {
  const parsed = parseArgv(['install', '--target', 'hermes', '--dry-run']);

  assert.equal(parsed.command, 'install');
  assert.equal(parsed.options.target, 'hermes');
  assert.equal(parsed.options.dryRun, true);
});

test('parseArgv treats uninstall and delete as remove aliases', () => {
  const uninstall = parseArgv(['uninstall', '--target', 'codex', '--parts', 'globals,project', '--dry-run']);
  const del = parseArgv(['delete', '--target', 'claude']);

  assert.equal(uninstall.command, 'remove');
  assert.equal(uninstall.options.target, 'codex');
  assert.equal(uninstall.options.parts, 'globals,project');
  assert.equal(uninstall.options.dryRun, true);
  assert.equal(del.command, 'remove');
  assert.equal(del.options.target, 'claude');
});

test('parseArgv delegates bridge commands without consuming bridge flags', () => {
  const parsed = parseArgv(['task-run', '--task-id', 'B1', '--cwd', '/tmp/project']);

  assert.equal(parsed.command, 'task-run');
  assert.deepEqual(parsed.rest, ['--task-id', 'B1', '--cwd', '/tmp/project']);
});

test('parseArgv delegates setup without consuming setup flags', () => {
  const parsed = parseArgv(['setup', '--dry-run', '--project-dir', '/tmp/project', '--no-doctor']);

  assert.equal(parsed.command, 'setup');
  assert.deepEqual(parsed.rest, ['--dry-run', '--project-dir', '/tmp/project', '--no-doctor']);
});

test('parseArgv delegates mcp without consuming mcp flags', () => {
  const parsed = parseArgv(['mcp', 'serve', '--transport', 'stdio']);

  assert.equal(parsed.command, 'mcp');
  assert.deepEqual(parsed.rest, ['serve', '--transport', 'stdio']);
});
