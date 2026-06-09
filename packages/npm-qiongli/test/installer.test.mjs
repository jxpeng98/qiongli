import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';

import {
  buildCheck,
  cleanAssets,
  installSkills,
  readSkillCoverage,
  readSkillSubject,
  readSkillVersion,
  removeAssets,
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
  const businessWorkflow = createWorkflow(
    root,
    path.join('payload', 'subjects', 'business', 'complete', 'qiongli-workflow'),
    'business',
    'complete',
    'business complete workflow\n',
  );
  const businessFocusedWorkflow = createWorkflow(
    root,
    path.join('payload', 'subjects', 'business', 'focused', 'qiongli-workflow'),
    'business',
    'focused',
    'business focused workflow\n',
  );
  const financeWorkflow = createWorkflow(
    root,
    path.join('payload', 'subjects', 'finance', 'complete', 'qiongli-workflow'),
    'finance',
    'complete',
    'finance complete workflow\n',
  );
  const financeFocusedWorkflow = createWorkflow(
    root,
    path.join('payload', 'subjects', 'finance', 'focused', 'qiongli-workflow'),
    'finance',
    'focused',
    'finance focused workflow\n',
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
    businessWorkflow,
    businessFocusedWorkflow,
    financeWorkflow,
    financeFocusedWorkflow,
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

test('installSkills copies managed payload and removes legacy residues', () => {
  const { root } = makeTempPackage();
  const home = fs.mkdtempSync(path.join(os.tmpdir(), 'qiongli-home-'));
  const legacyDir = path.join(home, '.codex', 'skills', 'research-paper-workflow');
  fs.mkdirSync(legacyDir, { recursive: true });
  fs.writeFileSync(path.join(legacyDir, 'SKILL.md'), '---\nname: research-paper-workflow\n---\n');

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
  assert.equal(result.legacyResidues[0].status, 'removed');
  assert.equal(fs.existsSync(legacyDir), false);
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

test('installSkills installs selected business complete subject payload', () => {
  const { root } = makeTempPackage();
  const home = fs.mkdtempSync(path.join(os.tmpdir(), 'qiongli-home-'));

  const result = installSkills({
    packageRoot: root,
    target: 'codex',
    subject: 'business',
    env: { HOME: home },
    platform: 'linux',
  });

  const dest = path.join(home, '.codex', 'skills', 'qiongli-workflow');
  assert.equal(result.sourceSubject, 'business');
  assert.equal(result.sourceCoverage, 'complete');
  assert.equal(readSkillSubject(dest), 'business');
  assert.equal(readSkillCoverage(dest), 'complete');
  assert.equal(fs.readFileSync(path.join(dest, 'workflows', 'paper.md'), 'utf-8'), 'business complete workflow\n');
});

test('installSkills installs selected finance focused subject payload', () => {
  const { root } = makeTempPackage();
  const home = fs.mkdtempSync(path.join(os.tmpdir(), 'qiongli-home-'));

  installSkills({
    packageRoot: root,
    target: 'codex',
    subject: 'finance',
    coverage: 'focused',
    env: { HOME: home },
    platform: 'linux',
  });

  const dest = path.join(home, '.codex', 'skills', 'qiongli-workflow');
  assert.equal(readSkillSubject(dest), 'finance');
  assert.equal(readSkillCoverage(dest), 'focused');
  assert.equal(fs.readFileSync(path.join(dest, 'workflows', 'paper.md'), 'utf-8'), 'finance focused workflow\n');
});

test('installSkills reports available subjects for unknown subject', () => {
  const { root } = makeTempPackage();

  assert.throws(
    () => installSkills({ packageRoot: root, target: 'codex', subject: 'biomedical' }),
    /Unknown subject 'biomedical'\. Available subjects: accounting, business, core, economics, finance/,
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
  assert.deepEqual(result.payload.available_subjects, ['accounting', 'business', 'core', 'economics', 'finance']);
  assert.deepEqual(result.payload.available_coverage.accounting, ['complete', 'focused']);
  assert.deepEqual(result.payload.available_coverage.business, ['complete', 'focused']);
  assert.deepEqual(result.payload.available_coverage.core, ['complete', 'focused']);
  assert.deepEqual(result.payload.available_coverage.economics, ['complete', 'focused']);
  assert.deepEqual(result.payload.available_coverage.finance, ['complete', 'focused']);
  assert.equal(result.installed.codex.subject, 'economics');
  assert.equal(result.installed.codex.coverage, 'complete');
  assert.equal(result.installed.codex.version, 'v9.9.9-beta.1');
});

test('cleanAssets globals removes legacy skill directories', () => {
  const home = fs.mkdtempSync(path.join(os.tmpdir(), 'qiongli-clean-home-'));
  const legacyDir = path.join(home, '.codex', 'skills', 'research-paper-workflow');
  fs.mkdirSync(legacyDir, { recursive: true });
  fs.writeFileSync(path.join(legacyDir, 'SKILL.md'), '---\nname: research-paper-workflow\n---\n');

  const result = cleanAssets({ projectDir: home, globals: true, env: { HOME: home } });

  assert.equal(fs.existsSync(legacyDir), false);
  assert.equal(result.removed.includes(legacyDir), true);
});

test('removeAssets removes managed skills and discovery while preserving user files', () => {
  const { root } = makeTempPackage();
  const home = fs.mkdtempSync(path.join(os.tmpdir(), 'qiongli-remove-home-'));
  installSkills({ packageRoot: root, target: 'claude', env: { HOME: home }, platform: 'linux' });
  const skillDir = path.join(home, '.claude', 'skills', 'qiongli-workflow');
  const commandLink = path.join(home, '.claude', 'commands', 'paper.md');
  const customCommand = path.join(home, '.claude', 'commands', 'custom.md');
  fs.writeFileSync(customCommand, 'user command\n');

  const result = removeAssets({ target: 'claude', env: { HOME: home }, platform: 'linux' });

  assert.equal(fs.existsSync(skillDir), false);
  assert.equal(fs.existsSync(commandLink), false);
  assert.equal(fs.existsSync(customCommand), true);
  assert.equal(result.actions.some((action) => action.label === 'Skill' && action.status === 'removed'), true);
  assert.equal(result.actions.some((action) => action.label === 'Workflow' && action.status === 'removed'), true);
});

test('removeAssets skips unmanaged qiongli-workflow directories', () => {
  const home = fs.mkdtempSync(path.join(os.tmpdir(), 'qiongli-remove-home-'));
  const unmanaged = path.join(home, '.codex', 'skills', 'qiongli-workflow');
  fs.mkdirSync(unmanaged, { recursive: true });
  fs.writeFileSync(path.join(unmanaged, 'README.md'), 'user content\n');

  const result = removeAssets({ target: 'codex', env: { HOME: home } });

  assert.equal(fs.existsSync(unmanaged), true);
  assert.equal(result.actions[0].status, 'skip');
});

test('removeAssets removes stale discovery symlinks without skill directory', () => {
  const home = fs.mkdtempSync(path.join(os.tmpdir(), 'qiongli-remove-home-'));
  const commandDir = path.join(home, '.claude', 'commands');
  fs.mkdirSync(commandDir, { recursive: true });
  const stale = path.join(commandDir, 'paper.md');
  fs.symlinkSync(path.join(home, '.claude', 'skills', 'qiongli-workflow', 'workflows', 'paper.md'), stale);

  const result = removeAssets({ target: 'claude', env: { HOME: home } });

  assert.equal(fs.existsSync(stale), false);
  assert.equal(result.actions.some((action) => action.label === 'Workflow' && action.status === 'removed'), true);
});

test('removeAssets rejects unsupported targets and parts', () => {
  assert.throws(() => removeAssets({ target: 'unknown' }), /Unsupported target/);
  assert.throws(() => removeAssets({ parts: 'globals,unknown' }), /Unsupported install part/);
});
