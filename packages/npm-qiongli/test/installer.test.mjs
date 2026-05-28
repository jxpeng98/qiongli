import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';

import {
  buildCheck,
  installSkills,
  readSkillCoverage,
  readSkillSubject,
  readSkillVersion,
  resolveTargetPaths,
} from '../lib/installer.mjs';

function makeTempPackage() {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'qiongli-npm-test-'));
  fs.writeFileSync(
    path.join(root, 'package.json'),
    JSON.stringify({ name: 'qiongli', version: '9.9.9-beta.1' }),
  );
  const legacyWorkflow = createWorkflow(
    root,
    path.join('payload', 'qiongli-workflow'),
    'core',
    'complete',
    'legacy core workflow\n',
  );
  const coreWorkflow = createWorkflow(
    root,
    path.join('payload', 'subjects', 'core', 'complete', 'qiongli-workflow'),
    'core',
    'complete',
    'core complete workflow\n',
  );
  const coreFocusedWorkflow = createWorkflow(
    root,
    path.join('payload', 'subjects', 'core', 'focused', 'qiongli-workflow'),
    'core',
    'focused',
    'core focused workflow\n',
  );
  const economicsWorkflow = createWorkflow(
    root,
    path.join('payload', 'subjects', 'economics', 'complete', 'qiongli-workflow'),
    'economics',
    'complete',
    'economics complete workflow\n',
  );
  const economicsFocusedWorkflow = createWorkflow(
    root,
    path.join('payload', 'subjects', 'economics', 'focused', 'qiongli-workflow'),
    'economics',
    'focused',
    'economics focused workflow\n',
  );
  const accountingWorkflow = createWorkflow(
    root,
    path.join('payload', 'subjects', 'accounting', 'complete', 'qiongli-workflow'),
    'accounting',
    'complete',
    'accounting complete workflow\n',
  );
  const accountingFocusedWorkflow = createWorkflow(
    root,
    path.join('payload', 'subjects', 'accounting', 'focused', 'qiongli-workflow'),
    'accounting',
    'focused',
    'accounting focused workflow\n',
  );
  return {
    root,
    legacyWorkflow,
    coreWorkflow,
    coreFocusedWorkflow,
    economicsWorkflow,
    economicsFocusedWorkflow,
    accountingWorkflow,
    accountingFocusedWorkflow,
  };
}

function createWorkflow(root, rel, subject, coverage, workflowText) {
  const workflow = path.join(root, rel);
  fs.mkdirSync(path.join(workflow, 'workflows'), { recursive: true });
  fs.writeFileSync(path.join(workflow, 'SKILL.md'), '---\nname: qiongli-workflow\n---\n');
  fs.writeFileSync(path.join(workflow, 'VERSION'), 'v9.9.9-beta.1\n');
  fs.writeFileSync(path.join(workflow, 'SUBJECT'), `${subject}\n`);
  fs.writeFileSync(
    path.join(workflow, 'SUBJECT_MANIFEST.json'),
    `${JSON.stringify({ subject, coverage, flavor: 'full', layers: subject === 'core' ? ['core'] : ['core', subject] })}\n`,
  );
  fs.writeFileSync(path.join(workflow, 'workflows', 'paper.md'), workflowText);
  return workflow;
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
  assert.equal(readSkillSubject(dest), 'core');
  assert.equal(readSkillCoverage(dest), 'complete');
  assert.equal(fs.readFileSync(path.join(dest, 'workflows', 'paper.md'), 'utf-8'), 'core complete workflow\n');
  assert.equal(result.legacyResidues.length, 1);
  assert.equal(result.legacyResidues[0].legacyName, 'research-paper-workflow');
});

test('installSkills installs selected economics complete subject payload', () => {
  const { root } = makeTempPackage();
  const home = fs.mkdtempSync(path.join(os.tmpdir(), 'qiongli-home-'));

  const result = installSkills({
    packageRoot: root,
    target: 'codex',
    subject: 'economics',
    env: { HOME: home },
    platform: 'linux',
  });

  const dest = path.join(home, '.codex', 'skills', 'qiongli-workflow');
  assert.equal(result.sourceSubject, 'economics');
  assert.equal(result.sourceCoverage, 'complete');
  assert.equal(readSkillSubject(dest), 'economics');
  assert.equal(readSkillCoverage(dest), 'complete');
  assert.equal(fs.readFileSync(path.join(dest, 'workflows', 'paper.md'), 'utf-8'), 'economics complete workflow\n');
});

test('installSkills installs selected economics focused subject payload', () => {
  const { root } = makeTempPackage();
  const home = fs.mkdtempSync(path.join(os.tmpdir(), 'qiongli-home-'));

  installSkills({
    packageRoot: root,
    target: 'codex',
    subject: 'economics',
    coverage: 'focused',
    env: { HOME: home },
    platform: 'linux',
  });

  const dest = path.join(home, '.codex', 'skills', 'qiongli-workflow');
  assert.equal(readSkillSubject(dest), 'economics');
  assert.equal(readSkillCoverage(dest), 'focused');
  assert.equal(fs.readFileSync(path.join(dest, 'workflows', 'paper.md'), 'utf-8'), 'economics focused workflow\n');
});

test('installSkills installs selected accounting complete subject payload', () => {
  const { root } = makeTempPackage();
  const home = fs.mkdtempSync(path.join(os.tmpdir(), 'qiongli-home-'));

  const result = installSkills({
    packageRoot: root,
    target: 'codex',
    subject: 'accounting',
    env: { HOME: home },
    platform: 'linux',
  });

  const dest = path.join(home, '.codex', 'skills', 'qiongli-workflow');
  assert.equal(result.sourceSubject, 'accounting');
  assert.equal(result.sourceCoverage, 'complete');
  assert.equal(readSkillSubject(dest), 'accounting');
  assert.equal(readSkillCoverage(dest), 'complete');
  assert.equal(fs.readFileSync(path.join(dest, 'workflows', 'paper.md'), 'utf-8'), 'accounting complete workflow\n');
});

test('installSkills installs selected accounting focused subject payload', () => {
  const { root } = makeTempPackage();
  const home = fs.mkdtempSync(path.join(os.tmpdir(), 'qiongli-home-'));

  installSkills({
    packageRoot: root,
    target: 'codex',
    subject: 'accounting',
    coverage: 'focused',
    env: { HOME: home },
    platform: 'linux',
  });

  const dest = path.join(home, '.codex', 'skills', 'qiongli-workflow');
  assert.equal(readSkillSubject(dest), 'accounting');
  assert.equal(readSkillCoverage(dest), 'focused');
  assert.equal(fs.readFileSync(path.join(dest, 'workflows', 'paper.md'), 'utf-8'), 'accounting focused workflow\n');
});

test('installSkills reports available subjects for unknown subject', () => {
  const { root } = makeTempPackage();

  assert.throws(
    () => installSkills({ packageRoot: root, target: 'codex', subject: 'biomedical' }),
    /Unknown subject 'biomedical'\. Available subjects: accounting, core, economics/,
  );
});

test('installSkills reports available coverage for unknown coverage', () => {
  const { root } = makeTempPackage();

  assert.throws(
    () => installSkills({ packageRoot: root, target: 'codex', subject: 'economics', coverage: 'wide' }),
    /Unknown coverage 'wide' for subject 'economics'\. Available coverage: complete, focused/,
  );
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

  fs.writeFileSync(path.join(dest, 'SUBJECT'), 'core\n');
  const sameVersionDifferentSubject = installSkills({
    packageRoot: root,
    target: 'codex',
    subject: 'economics',
    mode: 'copy',
    env: { HOME: home },
    platform: 'linux',
  });

  assert.equal(sameVersionDifferentSubject.actions[0].status, 'ok');
  assert.equal(readSkillSubject(dest), 'economics');

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
  assert.equal(fs.readFileSync(winCopy, 'utf-8'), 'core complete workflow\n');
});

test('buildCheck reports payload and installed subjects', () => {
  const { root } = makeTempPackage();
  const home = fs.mkdtempSync(path.join(os.tmpdir(), 'qiongli-home-'));
  installSkills({ packageRoot: root, target: 'codex', subject: 'economics', env: { HOME: home }, platform: 'linux' });

  const result = buildCheck({ packageRoot: root, env: { HOME: home } });

  assert.equal(result.payload.subject, 'core');
  assert.equal(result.payload.coverage, 'complete');
  assert.deepEqual(result.payload.available_subjects, ['accounting', 'core', 'economics']);
  assert.deepEqual(result.payload.available_coverage.accounting, ['complete', 'focused']);
  assert.deepEqual(result.payload.available_coverage.core, ['complete', 'focused']);
  assert.deepEqual(result.payload.available_coverage.economics, ['complete', 'focused']);
  assert.equal(result.installed.codex.subject, 'economics');
  assert.equal(result.installed.codex.coverage, 'complete');
  assert.equal(result.installed.codex.version, 'v9.9.9-beta.1');
});
