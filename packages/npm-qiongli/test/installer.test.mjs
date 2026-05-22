import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';

import { installSkills, readSkillVersion, resolveTargetPaths } from '../lib/installer.mjs';

function makeTempPackage() {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'qiongli-npm-test-'));
  const workflow = path.join(root, 'payload', 'qiongli-workflow');
  fs.mkdirSync(path.join(workflow, 'workflows'), { recursive: true });
  fs.writeFileSync(path.join(workflow, 'SKILL.md'), '---\nname: qiongli-workflow\n---\n');
  fs.writeFileSync(path.join(workflow, 'VERSION'), 'v9.9.9-beta.1\n');
  fs.writeFileSync(path.join(workflow, 'workflows', 'paper.md'), 'paper workflow\n');
  return { root, workflow };
}

test('resolveTargetPaths uses client home environment overrides', () => {
  const paths = resolveTargetPaths({
    env: {
      HOME: '/home/tester',
      CODEX_HOME: '/x/codex',
      CLAUDE_CODE_HOME: '/x/claude',
      GEMINI_HOME: '/x/gemini',
      ANTIGRAVITY_HOME: '/x/ag',
    },
  });

  assert.equal(paths.codex, path.join('/x/codex', 'skills', 'qiongli-workflow'));
  assert.equal(paths.claude, path.join('/x/claude', 'skills', 'qiongli-workflow'));
  assert.equal(paths.gemini, path.join('/x/gemini', 'skills', 'qiongli-workflow'));
  assert.equal(paths.antigravity, path.join('/x/ag', 'skills', 'qiongli-workflow'));
});

test('installSkills copies managed payload and reports legacy residues', () => {
  const { root } = makeTempPackage();
  const home = fs.mkdtempSync(path.join(os.tmpdir(), 'qiongli-home-'));
  fs.mkdirSync(path.join(home, '.codex', 'skills', 'research-paper-workflow'), { recursive: true });

  const result = installSkills({
    packageRoot: root,
    target: 'codex',
    mode: 'copy',
    env: { HOME: home },
    platform: 'linux',
  });

  const dest = path.join(home, '.codex', 'skills', 'qiongli-workflow');
  assert.equal(readSkillVersion(dest), 'v9.9.9-beta.1');
  assert.equal(fs.readFileSync(path.join(dest, 'workflows', 'paper.md'), 'utf-8'), 'paper workflow\n');
  assert.equal(result.legacyResidues.length, 1);
  assert.equal(result.legacyResidues[0].legacyName, 'research-paper-workflow');
});

test('installSkills updates managed older installs and skips unmanaged installs without overwrite', () => {
  const { root } = makeTempPackage();
  const home = fs.mkdtempSync(path.join(os.tmpdir(), 'qiongli-home-'));
  const dest = path.join(home, '.codex', 'skills', 'qiongli-workflow');
  fs.mkdirSync(dest, { recursive: true });
  fs.writeFileSync(path.join(dest, 'SKILL.md'), '---\nname: qiongli-workflow\n---\n');
  fs.writeFileSync(path.join(dest, 'VERSION'), 'v0.1.0\n');

  const updated = installSkills({
    packageRoot: root,
    target: 'codex',
    mode: 'copy',
    env: { HOME: home },
    platform: 'linux',
  });

  assert.equal(updated.actions[0].status, 'ok');
  assert.equal(readSkillVersion(dest), 'v9.9.9-beta.1');

  fs.rmSync(dest, { recursive: true, force: true });
  fs.mkdirSync(dest, { recursive: true });
  fs.writeFileSync(path.join(dest, 'README.md'), 'user content\n');

  const skipped = installSkills({
    packageRoot: root,
    target: 'codex',
    mode: 'copy',
    env: { HOME: home },
    platform: 'linux',
  });

  assert.equal(skipped.actions[0].status, 'skip');
  assert.equal(fs.readFileSync(path.join(dest, 'README.md'), 'utf-8'), 'user content\n');
});

test('installSkills creates symlink discovery on POSIX and managed copies on Windows', () => {
  const { root } = makeTempPackage();
  const posixHome = fs.mkdtempSync(path.join(os.tmpdir(), 'qiongli-posix-'));
  installSkills({ packageRoot: root, target: 'claude', env: { HOME: posixHome }, platform: 'linux' });
  const posixLink = path.join(posixHome, '.claude', 'commands', 'paper.md');
  assert.equal(fs.lstatSync(posixLink).isSymbolicLink(), true);

  const winHome = fs.mkdtempSync(path.join(os.tmpdir(), 'qiongli-win-'));
  installSkills({ packageRoot: root, target: 'gemini', env: { HOME: winHome }, platform: 'win32' });
  const winCopy = path.join(winHome, '.gemini', 'workflows', 'paper.md');
  assert.equal(fs.lstatSync(winCopy).isFile(), true);
  assert.equal(fs.readFileSync(winCopy, 'utf-8'), 'paper workflow\n');
});
