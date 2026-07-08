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
  assert.equal(parsed.options.surface, 'skills');
  assert.equal(parsed.options.subject, 'core');
  assert.equal(parsed.options.coverage, 'complete');
  assert.deepEqual(parsed.rest, []);
});

test('parseArgv parses install surface option', () => {
  const plugin = parseArgv(['install', '--surface', 'plugin']);
  const both = parseArgv(['install', '--surface', 'both']);

  assert.equal(plugin.options.surface, 'plugin');
  assert.equal(both.options.surface, 'both');
});

test('parseArgv accepts auto install target', () => {
  const parsed = parseArgv(['install', '--target', 'auto']);

  assert.equal(parsed.options.target, 'auto');
  assert.deepEqual(parsed.rest, []);
});

test('parseArgv treats refresh commands as install refreshes with overwrite', () => {
  const upgrade = parseArgv(['upgrade', '--target', 'codex', '--subject', 'economics', '--coverage', 'focused']);
  const refresh = parseArgv(['refresh', '--target', 'claude']);
  const update = parseArgv(['update', '--dry-run']);

  assert.equal(upgrade.command, 'install');
  assert.equal(upgrade.options.target, 'codex');
  assert.equal(upgrade.options.subject, 'economics');
  assert.equal(upgrade.options.coverage, 'focused');
  assert.equal(upgrade.options.overwrite, true);
  assert.equal(refresh.command, 'install');
  assert.equal(refresh.options.target, 'claude');
  assert.equal(refresh.options.overwrite, true);
  assert.equal(update.command, 'install');
  assert.equal(update.options.dryRun, true);
  assert.equal(update.options.overwrite, true);
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

test('parseArgv preserves unknown full runtime args in rest', () => {
  const parsed = parseArgv(['task-run', '--task-id', 'B1', '--cwd', '/tmp/project']);

  assert.equal(parsed.command, 'task-run');
  assert.equal(parsed.options.cwd, '/tmp/project');
  assert.deepEqual(parsed.rest, ['--task-id', 'B1']);
});

test('parseArgv parses setup as npm asset setup', () => {
  const parsed = parseArgv(['setup', '--dry-run', '--target', 'codex', '--no-doctor']);

  assert.equal(parsed.command, 'setup');
  assert.equal(parsed.options.dryRun, true);
  assert.equal(parsed.options.target, 'codex');
  assert.deepEqual(parsed.rest, ['--no-doctor']);
});

test('parseArgv keeps self-update unsupported by the npm asset updater', () => {
  const selfUpdate = parseArgv(['self-update', '--channel', 'next', '--dry-run', '--yes']);

  assert.equal(selfUpdate.command, 'self-update');
  assert.equal(selfUpdate.options.dryRun, true);
  assert.deepEqual(selfUpdate.rest, ['--channel', 'next', '--yes']);
});

test('parseArgv delegates mcp without consuming mcp flags', () => {
  const parsed = parseArgv(['mcp', 'serve', '--transport', 'stdio']);

  assert.equal(parsed.command, 'mcp');
  assert.deepEqual(parsed.rest, ['serve', '--transport', 'stdio']);
});

test('parseArgv delegates guidance without consuming guidance flags', () => {
  const parsed = parseArgv(['guidance', 'init', '--project-dir', '/tmp/project']);

  assert.equal(parsed.command, 'guidance');
  assert.equal(parsed.options.projectDir, '/tmp/project');
  assert.deepEqual(parsed.rest, ['init']);
});

test('parseArgv parses project set-subject without Python', () => {
  const parsed = parseArgv(['project', 'set-subject', 'finance', '--project-dir', '/tmp/project']);

  assert.equal(parsed.command, 'project');
  assert.equal(parsed.options.projectCommand, 'set-subject');
  assert.equal(parsed.options.projectDir, '/tmp/project');
  assert.equal(parsed.options.projectSubject, 'finance');
  assert.deepEqual(parsed.rest, []);
});

test('parseArgv parses project set-subject with --subject', () => {
  const parsed = parseArgv(['project', 'set-subject', '--subject', 'finance', '--project-dir', '/tmp/project']);

  assert.equal(parsed.command, 'project');
  assert.equal(parsed.options.projectCommand, 'set-subject');
  assert.equal(parsed.options.projectSubject, 'finance');
  assert.equal(parsed.options.projectDir, '/tmp/project');
  assert.deepEqual(parsed.rest, []);
});

test('parseArgv rejects unsupported surfaces', () => {
  assert.throws(
    () => parseArgv(['install', '--surface', 'wizard']),
    /Unsupported surface: wizard/,
  );
});
