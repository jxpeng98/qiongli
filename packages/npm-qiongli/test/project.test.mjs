import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';

import {
  initProject,
  projectStatus,
  renderProjectResult,
  setProjectSubject,
} from '../lib/project.mjs';

test('projectStatus reports implicit auto when manifest is missing and does not materialize files', () => {
  const projectDir = fs.mkdtempSync(path.join(os.tmpdir(), 'qiongli-project-'));

  const state = projectStatus({ projectDir });

  assert.equal(state.exists, false);
  assert.equal(state.path, '.qiongli/guidance_manifest.yaml');
  assert.deepEqual(state.manifest, {
    active_subject: 'auto',
    secondary_subjects: [],
    venue_profiles: [],
    method_lenses: [],
    strictness: 'standard',
  });
  assert.deepEqual(state.warnings, []);
  assert.equal(fs.existsSync(path.join(projectDir, '.qiongli')), false);
});

test('initProject creates guidance manifest and local guidance scaffolding', () => {
  const projectDir = fs.mkdtempSync(path.join(os.tmpdir(), 'qiongli-project-'));

  const result = initProject({ projectDir });
  const qiongliDir = path.join(projectDir, '.qiongli');
  const manifestPath = path.join(qiongliDir, 'guidance_manifest.yaml');
  const localGuidancePath = path.join(qiongliDir, 'local_guidance.md');

  assert.equal(result.action, 'init');
  assert.equal(result.changed, true);
  assert.equal(result.dry_run, false);
  assert.equal(result.state.exists, true);
  assert.equal(result.state.manifest.active_subject, 'auto');
  assert.equal(fs.existsSync(manifestPath), true);
  assert.equal(fs.existsSync(localGuidancePath), true);
  assert.equal(fs.statSync(path.join(qiongliDir, 'guidance.d')).isDirectory(), true);
  assert.equal(fs.statSync(path.join(qiongliDir, 'trace')).isDirectory(), true);
  assert.match(fs.readFileSync(manifestPath, 'utf-8'), /active_subject: auto/);
  assert.match(fs.readFileSync(manifestPath, 'utf-8'), /strictness: standard/);
  assert.doesNotMatch(fs.readFileSync(manifestPath, 'utf-8'), /guidance_mode/);
  assert.match(fs.readFileSync(localGuidancePath, 'utf-8'), /Project-local guidance for this repository\./);
});

test('initProject dry-run reports planned files without writing them', () => {
  const projectDir = fs.mkdtempSync(path.join(os.tmpdir(), 'qiongli-project-'));

  const result = initProject({ projectDir, dryRun: true });

  assert.equal(result.action, 'init');
  assert.equal(result.changed, true);
  assert.equal(result.dry_run, true);
  assert.equal(result.state.exists, false);
  assert.equal(result.state.manifest.active_subject, 'auto');
  assert.deepEqual(
    result.actions.map((entry) => entry.path),
    [
      '.qiongli/guidance_manifest.yaml',
      '.qiongli/local_guidance.md',
      '.qiongli/guidance.d/',
      '.qiongli/trace/',
    ],
  );
  assert.equal(fs.existsSync(path.join(projectDir, '.qiongli')), false);
});

test('setProjectSubject updates active_subject and preserves known list fields', () => {
  const projectDir = fs.mkdtempSync(path.join(os.tmpdir(), 'qiongli-project-'));
  const qiongliDir = path.join(projectDir, '.qiongli');
  fs.mkdirSync(qiongliDir, { recursive: true });
  fs.writeFileSync(
    path.join(qiongliDir, 'guidance_manifest.yaml'),
    [
      'active_subject: economics',
      'secondary_subjects:',
      '  - accounting',
      'venue_profiles:',
      '  - journal-of-finance',
      'method_lenses:',
      '  - event-study',
      'strictness: high',
      '',
    ].join('\n'),
    'utf-8',
  );

  const result = setProjectSubject({ projectDir, subject: 'finance' });
  const manifestText = fs.readFileSync(path.join(qiongliDir, 'guidance_manifest.yaml'), 'utf-8');

  assert.equal(result.action, 'set-subject');
  assert.equal(result.changed, true);
  assert.equal(result.state.manifest.active_subject, 'finance');
  assert.deepEqual(result.state.manifest.secondary_subjects, ['accounting']);
  assert.deepEqual(result.state.manifest.venue_profiles, ['journal-of-finance']);
  assert.deepEqual(result.state.manifest.method_lenses, ['event-study']);
  assert.equal(result.state.manifest.strictness, 'high');
  assert.match(manifestText, /active_subject: finance/);
  assert.match(manifestText, /secondary_subjects:\n  - accounting/);
  assert.match(manifestText, /venue_profiles:\n  - journal-of-finance/);
  assert.match(manifestText, /method_lenses:\n  - event-study/);
  assert.match(manifestText, /strictness: high/);
});

test('setProjectSubject ignores and preserves unknown manifest blocks', () => {
  const projectDir = fs.mkdtempSync(path.join(os.tmpdir(), 'qiongli-project-'));
  const qiongliDir = path.join(projectDir, '.qiongli');
  fs.mkdirSync(qiongliDir, { recursive: true });
  fs.writeFileSync(
    path.join(qiongliDir, 'guidance_manifest.yaml'),
    [
      'active_subject: economics',
      'future_field:',
      '  owner: local',
      '  nested:',
      '    enabled: true',
      'future_list:',
      '  - alpha',
      'strictness: standard',
      '',
    ].join('\n'),
    'utf-8',
  );

  const status = projectStatus({ projectDir });
  const result = setProjectSubject({ projectDir, subject: 'finance' });
  const manifestText = fs.readFileSync(path.join(qiongliDir, 'guidance_manifest.yaml'), 'utf-8');

  assert.equal(status.manifest.active_subject, 'economics');
  assert.deepEqual(status.warnings, [
    'Ignored unsupported manifest field: future_field',
    'Ignored unsupported manifest field: future_list',
  ]);
  assert.equal(result.state.manifest.active_subject, 'finance');
  assert.match(manifestText, /future_field:\n  owner: local\n  nested:\n    enabled: true/);
  assert.match(manifestText, /future_list:\n  - alpha/);
  assert.match(manifestText, /active_subject: finance/);
});

test('setProjectSubject accepts --subject style input and supports dry-run', () => {
  const projectDir = fs.mkdtempSync(path.join(os.tmpdir(), 'qiongli-project-'));

  const result = setProjectSubject({ projectDir, subject: 'finance', dryRun: true });

  assert.equal(result.action, 'set-subject');
  assert.equal(result.changed, true);
  assert.equal(result.dry_run, true);
  assert.equal(result.state.exists, false);
  assert.equal(result.state.manifest.active_subject, 'finance');
  assert.equal(fs.existsSync(path.join(projectDir, '.qiongli', 'guidance_manifest.yaml')), false);
});

test('setProjectSubject rejects unsupported subjects', () => {
  const projectDir = fs.mkdtempSync(path.join(os.tmpdir(), 'qiongli-project-'));

  assert.throws(
    () => setProjectSubject({ projectDir, subject: 'history' }),
    /Unsupported active_subject: history/,
  );
});

test('project module remains Node-only and does not call Python', () => {
  const source = fs.readFileSync(new URL('../lib/project.mjs', import.meta.url), 'utf-8');

  assert.doesNotMatch(source, /python-runtime\.mjs/);
  assert.doesNotMatch(source, /runPythonCliCommand/);
  assert.doesNotMatch(source, /spawnSync|execFileSync|execSync/);
});

test('renderProjectResult returns JSON packets when requested', () => {
  const json = renderProjectResult({
    action: 'status',
    changed: false,
    dry_run: false,
    state: projectStatus({ projectDir: fs.mkdtempSync(path.join(os.tmpdir(), 'qiongli-project-')) }),
    actions: [],
  }, { json: true });

  const payload = JSON.parse(json);
  assert.equal(payload.action, 'status');
  assert.equal(payload.state.manifest.active_subject, 'auto');
});
