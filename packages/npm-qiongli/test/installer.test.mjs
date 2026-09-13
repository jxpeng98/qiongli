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

const NPM_PLUGIN_MARKER = '.qiongli-npm-lite.json';
const EXPECTED_NPM_PLATFORM_TARGET = Object.freeze({
  target_id: 'npm-plugin-lite',
  artifact_kind: 'npm-package',
  archive_format: 'npm-tarball',
  bundled_mcp_mode: 'none',
  command_surface: 'npx-cli',
  validator: 'npm-plugin-lite',
});

function npmPluginMarker(pluginDir) {
  return path.join(pluginDir, NPM_PLUGIN_MARKER);
}

function npmPluginSidecarMarker(pluginDir) {
  return `${pluginDir}${NPM_PLUGIN_MARKER}`;
}

function makeTempPackage() {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'qiongli-npm-test-'));
  fs.writeFileSync(
    path.join(root, 'package.json'),
    JSON.stringify({ name: 'qiongli', version: '9.9.9-beta.1' }),
  );
  writePlatformTargetRegistry(root);
  const sharedPlugin = createPluginPayload(
    root,
    path.join('payload', 'plugins', 'qiongli'),
    'shared plugin payload\n',
  );
  const codexPlugin = createPluginPayload(
    root,
    path.join('payload', 'plugins', 'codex', 'qiongli'),
    'codex plugin payload\n',
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
    sharedPlugin,
    codexPlugin,
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

function writePlatformTargetRegistry(root, overrides = {}, { targetKey = 'npm-plugin-lite' } = {}) {
  const registry = path.join(root, 'payload', 'content', 'distribution');
  fs.mkdirSync(registry, { recursive: true });
  const target = {
    ...EXPECTED_NPM_PLATFORM_TARGET,
    release_download: {
      guide_label: 'Qiongli npm/npx CLI',
      recommended_key: 'qiongli_cli',
      asset_groups: [],
    },
    ...overrides,
  };
  fs.writeFileSync(
    path.join(registry, 'platform-targets.json'),
    `${JSON.stringify({ schema_version: '1.0', targets: { [targetKey]: target } }, null, 2)}\n`,
  );
}

function createPluginPayload(root, rel, payloadText) {
  const plugin = path.join(root, rel);
  fs.mkdirSync(plugin, { recursive: true });
  fs.mkdirSync(path.join(plugin, '.codex-plugin'), { recursive: true });
  fs.mkdirSync(path.join(plugin, '.claude-plugin'), { recursive: true });
  fs.writeFileSync(path.join(plugin, '.codex-plugin', 'plugin.json'), `${JSON.stringify({ name: 'qiongli', runtime: 'node-lite' })}\n`);
  fs.writeFileSync(path.join(plugin, '.claude-plugin', 'plugin.json'), `${JSON.stringify({ name: 'qiongli', runtime: 'node-lite' })}\n`);
  fs.writeFileSync(path.join(plugin, 'payload.txt'), payloadText);
  return plugin;
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
      HERMES_HOME: '/x/hermes',
    },
  });

  assert.equal(paths.codex, path.join('/x/codex', 'skills', 'qiongli-workflow'));
  assert.equal(paths.claude, path.join('/x/claude', 'skills', 'qiongli-workflow'));
  assert.equal(Object.hasOwn(paths, 'gemini'), false);
  assert.equal(paths.antigravity, path.join('/x/ag', 'skills', 'qiongli-workflow'));
  assert.equal(paths.hermes, path.join('/x/hermes', 'skills', 'qiongli-workflow'));
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

test('installSkills auto target copies only detected client payloads', () => {
  const { root } = makeTempPackage();
  const home = fs.mkdtempSync(path.join(os.tmpdir(), 'qiongli-auto-home-'));
  const binDir = fs.mkdtempSync(path.join(os.tmpdir(), 'qiongli-auto-bin-'));
  const codexBin = path.join(binDir, process.platform === 'win32' ? 'codex.cmd' : 'codex');
  fs.writeFileSync(codexBin, process.platform === 'win32' ? '@echo off\r\n' : '#!/bin/sh\n');
  fs.chmodSync(codexBin, 0o755);

  installSkills({
    packageRoot: root,
    target: 'auto',
    mode: 'copy',
    env: { HOME: home, PATH: binDir },
    platform: process.platform,
  });

  assert.equal(readSkillVersion(path.join(home, '.codex', 'skills', 'qiongli-workflow')), 'v9.9.9-beta.1');
  assert.equal(fs.existsSync(path.join(home, '.claude', 'skills', 'qiongli-workflow')), false);
  assert.equal(fs.existsSync(path.join(home, '.gemini', 'antigravity', 'skills', 'qiongli-workflow')), false);
  assert.equal(fs.existsSync(path.join(home, '.hermes', 'skills', 'qiongli-workflow')), false);
});

test('installSkills auto target fails when no client CLI is detected', () => {
  const { root } = makeTempPackage();
  const home = fs.mkdtempSync(path.join(os.tmpdir(), 'qiongli-auto-empty-home-'));

  assert.throws(
    () => installSkills({
      packageRoot: root,
      target: 'auto',
      mode: 'copy',
      env: { HOME: home, PATH: '' },
      platform: 'linux',
    }),
    /--target auto/,
  );
});

test('installSkills installs plugin-only surface from target-specific plugin payload', () => {
  const { root } = makeTempPackage();
  const home = fs.mkdtempSync(path.join(os.tmpdir(), 'qiongli-plugin-home-'));

  const result = installSkills({
    packageRoot: root,
    target: 'codex',
    surface: 'plugin',
    env: { HOME: home },
    platform: 'linux',
  });

  const pluginDest = path.join(home, 'plugins', 'qiongli');
  const skillDest = path.join(home, '.codex', 'skills', 'qiongli-workflow');
  assert.equal(fs.readFileSync(path.join(pluginDest, 'payload.txt'), 'utf-8'), 'codex plugin payload\n');
  assert.deepEqual(JSON.parse(fs.readFileSync(npmPluginMarker(pluginDest), 'utf-8')), {
    managed_by: 'qiongli-npm',
    surface: 'plugin-lite',
    target: 'codex',
    version: '9.9.9-beta.1',
    platform_target: EXPECTED_NPM_PLATFORM_TARGET,
  });
  assert.equal(fs.existsSync(skillDest), false);
  assert.deepEqual(
    result.actions.map((action) => ({ label: action.label, path: action.path })),
    [{ label: 'Plugin', path: pluginDest }],
  );
});

test('installSkills records npm plugin-lite platform target metadata in marker', () => {
  const { root } = makeTempPackage();
  const home = fs.mkdtempSync(path.join(os.tmpdir(), 'qiongli-plugin-target-home-'));

  installSkills({
    packageRoot: root,
    target: 'codex',
    surface: 'plugin',
    env: { HOME: home },
    platform: 'linux',
  });

  const pluginDest = path.join(home, 'plugins', 'qiongli');
  const marker = JSON.parse(fs.readFileSync(npmPluginMarker(pluginDest), 'utf-8'));

  assert.deepEqual(marker.platform_target, EXPECTED_NPM_PLATFORM_TARGET);
});

test('installSkills uses bundled npm platform target registry values', () => {
  const { root } = makeTempPackage();
  writePlatformTargetRegistry(root, {
    target_id: 'fake-npm-plugin-lite',
    artifact_kind: 'fake-package',
    archive_format: 'fake-tarball',
    bundled_mcp_mode: 'fake-none',
    command_surface: 'fake-cli',
    validator: 'fake-validator',
  });
  const home = fs.mkdtempSync(path.join(os.tmpdir(), 'qiongli-plugin-fake-target-home-'));

  installSkills({
    packageRoot: root,
    target: 'codex',
    surface: 'plugin',
    env: { HOME: home },
    platform: 'linux',
  });

  const pluginDest = path.join(home, 'plugins', 'qiongli');
  const marker = JSON.parse(fs.readFileSync(npmPluginMarker(pluginDest), 'utf-8'));

  assert.equal(marker.platform_target.target_id, 'fake-npm-plugin-lite');
  assert.equal(marker.platform_target.validator, 'fake-validator');
});

test('installSkills selects npm platform target by registry recommended key', () => {
  const { root } = makeTempPackage();
  writePlatformTargetRegistry(
    root,
    {
      target_id: 'fixture-npm-target',
      artifact_kind: 'fixture-package',
      archive_format: 'fixture-tarball',
      bundled_mcp_mode: 'fixture-none',
      command_surface: 'fixture-cli',
      validator: 'fixture-validator',
    },
    { targetKey: 'fixture-npm-target' },
  );
  const home = fs.mkdtempSync(path.join(os.tmpdir(), 'qiongli-plugin-recommended-target-home-'));

  installSkills({
    packageRoot: root,
    target: 'codex',
    surface: 'plugin',
    env: { HOME: home },
    platform: 'linux',
  });

  const pluginDest = path.join(home, 'plugins', 'qiongli');
  const marker = JSON.parse(fs.readFileSync(npmPluginMarker(pluginDest), 'utf-8'));

  assert.equal(marker.platform_target.target_id, 'fixture-npm-target');
  assert.equal(marker.platform_target.validator, 'fixture-validator');
});

test('installSkills does not overwrite unmarked qiongli plugin directories', () => {
  const { root } = makeTempPackage();
  const home = fs.mkdtempSync(path.join(os.tmpdir(), 'qiongli-plugin-collision-home-'));
  const pluginDest = path.join(home, 'plugins', 'qiongli');
  fs.mkdirSync(path.join(pluginDest, '.codex-plugin'), { recursive: true });
  fs.writeFileSync(path.join(pluginDest, '.codex-plugin', 'plugin.json'), `${JSON.stringify({ name: 'qiongli' })}\n`);
  fs.writeFileSync(path.join(pluginDest, 'payload.txt'), 'user full plugin\n');

  const result = installSkills({
    packageRoot: root,
    target: 'codex',
    surface: 'plugin',
    overwrite: true,
    env: { HOME: home },
    platform: 'linux',
  });

  assert.equal(fs.readFileSync(path.join(pluginDest, 'payload.txt'), 'utf-8'), 'user full plugin\n');
  assert.deepEqual(
    result.actions.map((action) => ({ label: action.label, status: action.status, path: action.path, detail: action.detail })),
    [{ label: 'Plugin', status: 'skip', path: pluginDest, detail: 'unmanaged qiongli plugin directory' }],
  );
});

test('installSkills installs both skill and plugin surfaces', () => {
  const { root } = makeTempPackage();
  const home = fs.mkdtempSync(path.join(os.tmpdir(), 'qiongli-both-home-'));

  const result = installSkills({
    packageRoot: root,
    target: 'codex',
    surface: 'both',
    env: { HOME: home },
    platform: 'linux',
  });

  const pluginDest = path.join(home, 'plugins', 'qiongli');
  const skillDest = path.join(home, '.codex', 'skills', 'qiongli-workflow');
  assert.equal(fs.existsSync(skillDest), true);
  assert.equal(fs.readFileSync(path.join(pluginDest, 'payload.txt'), 'utf-8'), 'codex plugin payload\n');
  assert.equal(result.actions.some((action) => action.label === 'Skill' && action.path === skillDest), true);
  assert.equal(result.actions.some((action) => action.label === 'Plugin' && action.path === pluginDest), true);
});

test('installSkills falls back to shared plugin payload when target-specific payload is absent', () => {
  const { root } = makeTempPackage();
  const home = fs.mkdtempSync(path.join(os.tmpdir(), 'qiongli-shared-plugin-home-'));

  installSkills({
    packageRoot: root,
    target: 'claude',
    surface: 'plugin',
    env: { HOME: home },
    platform: 'linux',
  });

  const pluginDest = path.join(home, '.claude', 'plugins', 'qiongli');
  assert.equal(fs.readFileSync(path.join(pluginDest, 'payload.txt'), 'utf-8'), 'shared plugin payload\n');
});

test('installSkills skips plugin surface when payload lacks target manifest', () => {
  const { root } = makeTempPackage();
  const home = fs.mkdtempSync(path.join(os.tmpdir(), 'qiongli-unsupported-plugin-home-'));

  const result = installSkills({
    packageRoot: root,
    target: 'antigravity',
    surface: 'plugin',
    env: { HOME: home },
    platform: 'linux',
  });

  const pluginDest = path.join(home, '.gemini', 'antigravity', 'plugins', 'qiongli');
  assert.equal(fs.existsSync(pluginDest), false);
  assert.deepEqual(
    result.actions.map((action) => ({ label: action.label, status: action.status, path: action.path, detail: action.detail })),
    [{ label: 'Plugin', status: 'skip', path: pluginDest, detail: 'plugin payload not bundled for antigravity' }],
  );
});

test('installSkills skips Hermes plugin surface because Hermes has no plugin-lite target', () => {
  const { root } = makeTempPackage();
  const home = fs.mkdtempSync(path.join(os.tmpdir(), 'qiongli-hermes-plugin-home-'));

  const result = installSkills({
    packageRoot: root,
    target: 'hermes',
    surface: 'plugin',
    env: { HOME: home },
    platform: 'linux',
  });

  const pluginDest = path.join(home, '.hermes', 'plugins', 'qiongli');
  assert.equal(fs.existsSync(pluginDest), false);
  assert.deepEqual(
    result.actions.map((action) => ({ label: action.label, status: action.status, path: action.path, detail: action.detail })),
    [{ label: 'Plugin', status: 'skip', path: pluginDest, detail: 'Hermes has no npm plugin-lite surface' }],
  );
});

test('installSkills installs the payload into Hermes home', () => {
  const { root } = makeTempPackage();
  const home = fs.mkdtempSync(path.join(os.tmpdir(), 'qiongli-hermes-home-'));
  const hermesHome = path.join(home, '.custom-hermes');

  const result = installSkills({
    packageRoot: root,
    target: 'hermes',
    mode: 'copy',
    env: { HOME: home, HERMES_HOME: hermesHome },
    platform: 'linux',
  });

  const dest = path.join(hermesHome, 'skills', 'qiongli-workflow');
  assert.equal(readSkillVersion(dest), 'v9.9.9-beta.1');
  assert.equal(readSkillSubject(dest), 'core');
  assert.equal(readSkillCoverage(dest), 'complete');
  assert.equal(fs.readFileSync(path.join(dest, 'workflows', 'paper.md'), 'utf-8'), 'core complete workflow\n');
  assert.equal(result.targetPaths.hermes, dest);
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

test('full install parts include unified MCP guidance', () => {
  const { root } = makeTempPackage();

  const result = installSkills({
    packageRoot: root,
    target: 'codex',
    parts: 'globals,mcp',
    dryRun: true,
    env: { HOME: root },
    platform: 'linux',
  });

  const mcpAction = result.actions.find((action) => action.label === 'MCP');
  assert.ok(mcpAction);
  assert.match(mcpAction.detail, /qiongli mcp serve --transport stdio/);
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
  installSkills({ packageRoot: root, target: 'claude', env: { HOME: winHome }, platform: 'win32' });
  const winCopy = path.join(winHome, '.claude', 'commands', 'paper.md');
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
  assert.equal(result.installed.codex.surface, 'skills');
  assert.equal(result.installed.codex.skill.installed, true);
  assert.equal(result.installed.codex.plugin.installed, false);
});

test('buildCheck reports plugin-only installs', () => {
  const { root } = makeTempPackage();
  const home = fs.mkdtempSync(path.join(os.tmpdir(), 'qiongli-plugin-check-home-'));
  installSkills({ packageRoot: root, target: 'codex', surface: 'plugin', env: { HOME: home }, platform: 'linux' });

  const result = buildCheck({ packageRoot: root, env: { HOME: home } });

  assert.equal(result.installed.codex.installed, true);
  assert.equal(result.installed.codex.surface, 'plugin');
  assert.equal(result.installed.codex.version, '9.9.9-beta.1');
  assert.equal(result.installed.codex.skill.installed, false);
  assert.equal(result.installed.codex.skill.version, null);
  assert.equal(result.installed.codex.plugin.installed, true);
  assert.equal(result.installed.codex.plugin.managed, true);
  assert.equal(result.installed.codex.plugin.version, '9.9.9-beta.1');
  assert.equal(result.installed.codex.plugin.target, 'codex');
  assert.deepEqual(result.installed.codex.plugin.platform_target, EXPECTED_NPM_PLATFORM_TARGET);
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

test('removeAssets removes plugin-only surface with default globals part', () => {
  const { root } = makeTempPackage();
  const home = fs.mkdtempSync(path.join(os.tmpdir(), 'qiongli-remove-plugin-home-'));
  installSkills({ packageRoot: root, target: 'codex', surface: 'plugin', env: { HOME: home }, platform: 'linux' });

  const pluginDest = path.join(home, 'plugins', 'qiongli');
  const result = removeAssets({ target: 'codex', surface: 'plugin', env: { HOME: home } });

  assert.equal(fs.existsSync(pluginDest), false);
  assert.deepEqual(
    result.actions.map((action) => ({ label: action.label, status: action.status, path: action.path })),
    [{ label: 'Plugin', status: 'removed', path: pluginDest }],
  );
});

test('removeAssets removes broken link-mode plugin-only installs', () => {
  const { root } = makeTempPackage();
  const home = fs.mkdtempSync(path.join(os.tmpdir(), 'qiongli-remove-plugin-link-home-'));
  installSkills({
    packageRoot: root,
    target: 'codex',
    surface: 'plugin',
    mode: 'link',
    env: { HOME: home },
    platform: 'linux',
  });

  const pluginDest = path.join(home, 'plugins', 'qiongli');
  const sidecarMarker = npmPluginSidecarMarker(pluginDest);
  assert.equal(fs.lstatSync(pluginDest).isSymbolicLink(), true);
  assert.equal(fs.existsSync(sidecarMarker), true);
  fs.rmSync(root, { recursive: true, force: true });
  assert.equal(fs.existsSync(pluginDest), false);
  assert.equal(fs.lstatSync(pluginDest).isSymbolicLink(), true);

  const result = removeAssets({ target: 'codex', surface: 'plugin', env: { HOME: home } });

  assert.throws(() => fs.lstatSync(pluginDest), /ENOENT/);
  assert.equal(fs.existsSync(sidecarMarker), false);
  assert.deepEqual(
    result.actions.map((action) => ({ label: action.label, status: action.status, path: action.path })),
    [{ label: 'Plugin', status: 'removed', path: pluginDest }],
  );
});

test('installSkills overwrites broken link-mode plugin-only installs', () => {
  const firstPackage = makeTempPackage();
  const home = fs.mkdtempSync(path.join(os.tmpdir(), 'qiongli-overwrite-plugin-link-home-'));
  installSkills({
    packageRoot: firstPackage.root,
    target: 'codex',
    surface: 'plugin',
    mode: 'link',
    env: { HOME: home },
    platform: 'linux',
  });

  const pluginDest = path.join(home, 'plugins', 'qiongli');
  fs.rmSync(firstPackage.root, { recursive: true, force: true });
  assert.equal(fs.existsSync(pluginDest), false);
  assert.equal(fs.lstatSync(pluginDest).isSymbolicLink(), true);

  const secondPackage = makeTempPackage();
  const result = installSkills({
    packageRoot: secondPackage.root,
    target: 'codex',
    surface: 'plugin',
    mode: 'link',
    overwrite: true,
    env: { HOME: home },
    platform: 'linux',
  });

  assert.equal(fs.lstatSync(pluginDest).isSymbolicLink(), true);
  assert.equal(fs.readFileSync(path.join(pluginDest, 'payload.txt'), 'utf-8'), 'codex plugin payload\n');
  assert.equal(fs.existsSync(npmPluginSidecarMarker(pluginDest)), true);
  assert.deepEqual(
    result.actions.map((action) => ({ label: action.label, status: action.status, path: action.path })),
    [{ label: 'Plugin', status: 'ok', path: pluginDest }],
  );
});

test('removeAssets skips unmarked qiongli plugin directories', () => {
  const home = fs.mkdtempSync(path.join(os.tmpdir(), 'qiongli-remove-plugin-collision-home-'));
  const pluginDest = path.join(home, 'plugins', 'qiongli');
  fs.mkdirSync(path.join(pluginDest, '.codex-plugin'), { recursive: true });
  fs.writeFileSync(path.join(pluginDest, '.codex-plugin', 'plugin.json'), `${JSON.stringify({ name: 'qiongli' })}\n`);
  fs.writeFileSync(path.join(pluginDest, 'payload.txt'), 'user full plugin\n');

  const result = removeAssets({ target: 'codex', surface: 'plugin', env: { HOME: home } });

  assert.equal(fs.existsSync(pluginDest), true);
  assert.equal(fs.readFileSync(path.join(pluginDest, 'payload.txt'), 'utf-8'), 'user full plugin\n');
  assert.deepEqual(
    result.actions.map((action) => ({ label: action.label, status: action.status, path: action.path, detail: action.detail })),
    [{ label: 'Plugin', status: 'skip', path: pluginDest, detail: 'unmanaged qiongli plugin directory' }],
  );
});

test('removeAssets removes both surfaces when requested', () => {
  const { root } = makeTempPackage();
  const home = fs.mkdtempSync(path.join(os.tmpdir(), 'qiongli-remove-both-home-'));
  installSkills({ packageRoot: root, target: 'claude', surface: 'both', env: { HOME: home }, platform: 'linux' });

  const pluginDest = path.join(home, '.claude', 'plugins', 'qiongli');
  const skillDest = path.join(home, '.claude', 'skills', 'qiongli-workflow');
  const result = removeAssets({ target: 'claude', surface: 'both', env: { HOME: home } });

  assert.equal(fs.existsSync(pluginDest), false);
  assert.equal(fs.existsSync(skillDest), false);
  assert.equal(result.actions.some((action) => action.label === 'Plugin' && action.status === 'removed' && action.path === pluginDest), true);
  assert.equal(result.actions.some((action) => action.label === 'Skill' && action.status === 'removed' && action.path === skillDest), true);
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
