import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';

const TARGETS = ['codex', 'claude', 'gemini', 'antigravity'];
const PARTS = ['globals', 'project', 'cli'];
const LEGACY_SKILL_NAME = 'research-paper-workflow';

export function resolveTargetPaths({ env = process.env } = {}) {
  const home = env.HOME || env.USERPROFILE || os.homedir();
  const codexHome = env.CODEX_HOME || path.join(home, '.codex');
  const claudeHome = env.CLAUDE_CODE_HOME || path.join(home, '.claude');
  const geminiHome = env.GEMINI_HOME || path.join(home, '.gemini');
  const antigravityHome = env.ANTIGRAVITY_HOME || path.join(home, '.gemini', 'antigravity');
  return {
    codex: path.join(codexHome, 'skills', 'qiongli-workflow'),
    claude: path.join(claudeHome, 'skills', 'qiongli-workflow'),
    gemini: path.join(geminiHome, 'skills', 'qiongli-workflow'),
    antigravity: path.join(antigravityHome, 'skills', 'qiongli-workflow'),
  };
}

export function readSkillVersion(skillDir) {
  if (!isQiongliSkillDir(skillDir)) {
    return '';
  }
  try {
    return fs.readFileSync(path.join(skillDir, 'VERSION'), 'utf-8').trim();
  } catch {
    return '';
  }
}

export function readSkillSubject(skillDir) {
  if (!isQiongliSkillDir(skillDir)) {
    return '';
  }
  const manifest = readSubjectManifest(skillDir);
  if (typeof manifest.subject === 'string' && manifest.subject.trim()) {
    return manifest.subject.trim();
  }
  try {
    return fs.readFileSync(path.join(skillDir, 'SUBJECT'), 'utf-8').trim() || 'core';
  } catch {
    return 'core';
  }
}

export function readSkillCoverage(skillDir) {
  if (!isQiongliSkillDir(skillDir)) {
    return '';
  }
  const manifest = readSubjectManifest(skillDir);
  if (typeof manifest.coverage === 'string' && manifest.coverage.trim()) {
    return manifest.coverage.trim();
  }
  return 'complete';
}

export function installSkills({
  packageRoot,
  target = 'all',
  mode = 'copy',
  overwrite = false,
  dryRun = false,
  subject = 'core',
  coverage = 'complete',
  env = process.env,
  platform = process.platform,
} = {}) {
  const workflowSrc = resolveSubjectPayload({ packageRoot, subject, coverage });
  if (!fs.existsSync(path.join(workflowSrc, 'SKILL.md'))) {
    throw new Error(`Missing qiongli-workflow payload: ${workflowSrc}`);
  }

  const targetPaths = resolveTargetPaths({ env });
  const selectedTargets = target === 'all' ? TARGETS : [target];
  const sourceVersion = readSkillVersion(workflowSrc);
  const sourceSubject = readSkillSubject(workflowSrc) || subject;
  const sourceCoverage = readSkillCoverage(workflowSrc) || coverage;
  const actions = [];
  const legacyResidues = [];

  for (const item of selectedTargets) {
    const dest = targetPaths[item];
    const legacyPath = path.join(path.dirname(dest), LEGACY_SKILL_NAME);
    if (fs.existsSync(legacyPath)) {
      const status = removeLegacySkillPath(legacyPath, LEGACY_SKILL_NAME, dryRun);
      legacyResidues.push({ target: item, legacyName: LEGACY_SKILL_NAME, path: legacyPath, status });
    }

    actions.push(copySkill({ src: workflowSrc, dest, mode, overwrite, dryRun, sourceVersion, sourceSubject, sourceCoverage }));

    if ((item === 'claude' || item === 'gemini') && actions.at(-1).status !== 'skip') {
      actions.push(...installWorkflowDiscovery({ target: item, skillDest: dest, dryRun, platform }));
    }
  }

  return { sourceVersion, sourceSubject, sourceCoverage, actions, legacyResidues, targetPaths };
}

export function cleanAssets({ projectDir = '.', globals = false, dryRun = false, env = process.env } = {}) {
  const removed = [];
  const projectRoot = path.resolve(projectDir);
  const projectPatterns = [
    ['.agent', 'workflows'],
    ['.agent', 'skills', 'qiongli-workflow'],
    ['.agent', 'skills', LEGACY_SKILL_NAME],
    ['.agents', 'skills', 'qiongli-workflow'],
    ['.agents', 'skills', LEGACY_SKILL_NAME],
    ['.gemini', 'qiongli.md'],
    ['.gemini', 'agent-profiles.example.json'],
    ['CLAUDE.qiongli.md'],
  ];

  for (const parts of projectPatterns) {
    const candidate = path.join(projectRoot, ...parts);
    if (!fs.existsSync(candidate)) {
      continue;
    }
    removePath(candidate, dryRun);
    removed.push(candidate);
  }

  if (globals) {
    const targetPaths = resolveTargetPaths({ env });
    for (const skillDest of Object.values(targetPaths)) {
      const legacyPath = path.join(path.dirname(skillDest), LEGACY_SKILL_NAME);
      if (fs.existsSync(legacyPath) && removeLegacySkillPath(legacyPath, LEGACY_SKILL_NAME, dryRun) !== 'kept') {
        removed.push(legacyPath);
      }
    }
    for (const [target, skillDest] of Object.entries(targetPaths)) {
      if (target !== 'claude' && target !== 'gemini') {
        continue;
      }
      const discoveryDir = discoveryDirectory(target, skillDest);
      if (!fs.existsSync(discoveryDir)) {
        continue;
      }
      for (const name of fs.readdirSync(discoveryDir)) {
        const item = path.join(discoveryDir, name);
        if (isManagedDiscoveryFile(item)) {
          removePath(item, dryRun);
          removed.push(item);
        }
      }
    }
  }

  return { removed };
}

export function removeAssets({
  target = 'all',
  projectDir = '.',
  parts = '',
  dryRun = false,
  env = process.env,
} = {}) {
  validateTarget(target);
  const selectedParts = normalizeParts(parts) || ['globals'];
  const actions = [];

  if (selectedParts.includes('globals')) {
    const targetPaths = resolveTargetPaths({ env });
    const selectedTargets = target === 'all' ? TARGETS : [target];
    for (const item of selectedTargets) {
      const skillDest = targetPaths[item];
      actions.push(...removeWorkflowDiscovery({ target: item, skillDest, dryRun }));
      const legacyPath = path.join(path.dirname(skillDest), LEGACY_SKILL_NAME);
      if (fs.existsSync(legacyPath) && removeLegacySkillPath(legacyPath, LEGACY_SKILL_NAME, dryRun) !== 'kept') {
        actions.push({ label: 'Legacy Skill', status: dryRun ? 'dry-run' : 'removed', path: legacyPath, detail: item });
      }
      if (!fs.existsSync(skillDest)) {
        actions.push({ label: 'Skill', status: 'skip', path: skillDest, detail: 'not installed' });
        continue;
      }
      if (!isQiongliSkillDir(skillDest)) {
        actions.push({ label: 'Skill', status: 'skip', path: skillDest, detail: 'unmanaged qiongli-workflow directory' });
        continue;
      }
      removePath(skillDest, dryRun);
      actions.push({ label: 'Skill', status: dryRun ? 'dry-run' : 'removed', path: skillDest, detail: item });
    }
  }

  if (selectedParts.includes('project')) {
    const result = cleanAssets({ projectDir, globals: false, dryRun, env });
    for (const item of result.removed) {
      actions.push({ label: 'Project', status: dryRun ? 'dry-run' : 'removed', path: item, detail: 'stale asset' });
    }
  }

  if (selectedParts.includes('cli')) {
    actions.push({ label: 'CLI', status: 'skip', path: '<npm package>', detail: 'remove with npm uninstall -g qiongli' });
  }

  return { actions };
}

export function buildCheck({ packageRoot, subject = 'core', coverage = 'complete', env = process.env } = {}) {
  const packageJson = JSON.parse(fs.readFileSync(path.join(packageRoot, 'package.json'), 'utf-8'));
  const payload = resolveSubjectPayload({ packageRoot, subject, coverage });
  const targetPaths = resolveTargetPaths({ env });
  const installed = {};
  for (const [target, skillDir] of Object.entries(targetPaths)) {
    installed[target] = {
      path: skillDir,
      installed: isQiongliSkillDir(skillDir),
      version: readSkillVersion(skillDir) || null,
      subject: readSkillSubject(skillDir) || null,
      coverage: readSkillCoverage(skillDir) || null,
    };
  }
  return {
    npm_package: {
      name: packageJson.name,
      version: packageJson.version,
    },
    payload: {
      path: payload,
      version: readSkillVersion(payload),
      subject: readSkillSubject(payload) || subject,
      coverage: readSkillCoverage(payload) || coverage,
      available_subjects: availableSubjects(packageRoot),
      available_coverage: availableCoverage(packageRoot),
    },
    installed,
  };
}

function copySkill({ src, dest, mode, overwrite, dryRun, sourceVersion, sourceSubject, sourceCoverage }) {
  if (fs.existsSync(dest)) {
    if (!overwrite) {
      if (isQiongliSkillDir(dest)) {
        const destVersion = readSkillVersion(dest);
        const destSubject = readSkillSubject(dest);
        const destCoverage = readSkillCoverage(dest);
        if (destVersion === sourceVersion && destSubject === sourceSubject && destCoverage === sourceCoverage) {
          return {
            label: 'Skill',
            status: 'skip',
            path: dest,
            detail: `already current ${sourceVersion} (${sourceSubject}/${sourceCoverage})`,
          };
        }
      } else {
        return { label: 'Skill', status: 'skip', path: dest, detail: 'use --overwrite for unmanaged directory' };
      }
    }
    removePath(dest, dryRun);
  }

  if (!dryRun) {
    fs.mkdirSync(path.dirname(dest), { recursive: true });
    if (mode === 'link') {
      fs.symlinkSync(src, dest, process.platform === 'win32' ? 'junction' : 'dir');
    } else {
      fs.cpSync(src, dest, { recursive: true, force: true });
    }
  }

  return { label: 'Skill', status: 'ok', path: dest, detail: `installed ${sourceVersion} (${sourceSubject}/${sourceCoverage})` };
}

function resolveSubjectPayload({ packageRoot, subject, coverage = 'complete' }) {
  const requested = subject || 'core';
  const requestedCoverage = coverage || 'complete';
  const subjectPayload = path.join(packageRoot, 'payload', 'subjects', requested, requestedCoverage, 'qiongli-workflow');
  if (fs.existsSync(path.join(subjectPayload, 'SKILL.md'))) {
    return subjectPayload;
  }
  const legacySubjectPayload = path.join(packageRoot, 'payload', 'subjects', requested, 'qiongli-workflow');
  if (requestedCoverage === 'complete' && fs.existsSync(path.join(legacySubjectPayload, 'SKILL.md'))) {
    return legacySubjectPayload;
  }
  const legacyCore = path.join(packageRoot, 'payload', 'qiongli-workflow');
  if (requested === 'core' && requestedCoverage === 'complete' && fs.existsSync(path.join(legacyCore, 'SKILL.md'))) {
    return legacyCore;
  }
  const subjects = availableSubjects(packageRoot);
  if (!subjects.includes(requested)) {
    throw new Error(`Unknown subject '${requested}'. Available subjects: ${subjects.join(', ') || 'core'}`);
  }
  const coverageOptions = availableCoverage(packageRoot)[requested] || [];
  throw new Error(
    `Unknown coverage '${requestedCoverage}' for subject '${requested}'. Available coverage: ${coverageOptions.join(', ') || 'complete'}`,
  );
}

function availableSubjects(packageRoot) {
  const subjectsDir = path.join(packageRoot, 'payload', 'subjects');
  const subjects = [];
  if (fs.existsSync(subjectsDir)) {
    for (const name of fs.readdirSync(subjectsDir)) {
      const legacyWorkflow = path.join(subjectsDir, name, 'qiongli-workflow');
      const coverageRoot = path.join(subjectsDir, name);
      const hasCoverage = fs.readdirSync(coverageRoot).some((coverageName) => {
        const workflow = path.join(coverageRoot, coverageName, 'qiongli-workflow');
        return fs.existsSync(path.join(workflow, 'SKILL.md'));
      });
      if (fs.existsSync(path.join(legacyWorkflow, 'SKILL.md')) || hasCoverage) {
        subjects.push(name);
      }
    }
  }
  const legacyCore = path.join(packageRoot, 'payload', 'qiongli-workflow');
  if (!subjects.includes('core') && fs.existsSync(path.join(legacyCore, 'SKILL.md'))) {
    subjects.push('core');
  }
  return subjects.sort();
}

function availableCoverage(packageRoot) {
  const subjectsDir = path.join(packageRoot, 'payload', 'subjects');
  const result = {};
  if (fs.existsSync(subjectsDir)) {
    for (const subject of fs.readdirSync(subjectsDir)) {
      const subjectRoot = path.join(subjectsDir, subject);
      const coverage = [];
      for (const name of fs.readdirSync(subjectRoot)) {
        const workflow = path.join(subjectRoot, name, 'qiongli-workflow');
        if (fs.existsSync(path.join(workflow, 'SKILL.md'))) {
          coverage.push(name);
        }
      }
      if (fs.existsSync(path.join(subjectRoot, 'qiongli-workflow', 'SKILL.md'))) {
        coverage.push('complete');
      }
      if (coverage.length > 0) {
        result[subject] = [...new Set(coverage)].sort();
      }
    }
  }
  const legacyCore = path.join(packageRoot, 'payload', 'qiongli-workflow');
  if (fs.existsSync(path.join(legacyCore, 'SKILL.md'))) {
    result.core = [...new Set([...(result.core || []), 'complete'])].sort();
  }
  return result;
}

function readSubjectManifest(skillDir) {
  try {
    const payload = JSON.parse(fs.readFileSync(path.join(skillDir, 'SUBJECT_MANIFEST.json'), 'utf-8'));
    return payload && typeof payload === 'object' && !Array.isArray(payload) ? payload : {};
  } catch {
    return {};
  }
}

function installWorkflowDiscovery({ target, skillDest, dryRun, platform }) {
  const workflowsDir = path.join(skillDest, 'workflows');
  if (!fs.existsSync(workflowsDir)) {
    return [];
  }
  const actions = [];
  const discoveryDir = discoveryDirectory(target, skillDest);
  if (!dryRun) {
    fs.mkdirSync(discoveryDir, { recursive: true });
  }

  for (const file of fs.readdirSync(workflowsDir).filter((name) => name.endsWith('.md'))) {
    const src = path.join(workflowsDir, file);
    const dest = path.join(discoveryDir, file);
    if (!dryRun && fs.existsSync(dest)) {
      fs.rmSync(dest, { force: true, recursive: true });
    }
    if (!dryRun) {
      if (platform === 'win32') {
        fs.copyFileSync(src, dest);
      } else {
        fs.symlinkSync(src, dest);
      }
    }
    actions.push({ label: 'Workflow', status: 'ok', path: dest, detail: platform === 'win32' ? 'copied' : 'linked' });
  }
  return actions;
}

function removeWorkflowDiscovery({ target, skillDest, dryRun }) {
  if (target !== 'claude' && target !== 'gemini') {
    return [];
  }
  const workflowsDir = path.join(skillDest, 'workflows');
  const discoveryDir = discoveryDirectory(target, skillDest);
  if (!fs.existsSync(discoveryDir)) {
    return [];
  }
  const actions = [];
  const handled = new Set();
  const workflowNames = fs.existsSync(workflowsDir)
    ? fs.readdirSync(workflowsDir).filter((item) => item.endsWith('.md'))
    : [];
  for (const name of workflowNames) {
    const candidate = path.join(discoveryDir, name);
    handled.add(candidate);
    if (!pathExistsOrSymlink(candidate)) {
      continue;
    }
    if (!isManagedDiscoveryFileForSource(candidate, path.join(workflowsDir, name))) {
      actions.push({ label: 'Workflow', status: 'skip', path: candidate, detail: 'user-customized' });
      continue;
    }
    removePath(candidate, dryRun);
    actions.push({ label: 'Workflow', status: dryRun ? 'dry-run' : 'removed', path: candidate, detail: target });
  }
  for (const name of fs.readdirSync(discoveryDir).filter((item) => item.endsWith('.md'))) {
    const candidate = path.join(discoveryDir, name);
    if (handled.has(candidate) || !isManagedDiscoverySymlink(candidate)) {
      continue;
    }
    removePath(candidate, dryRun);
    actions.push({ label: 'Workflow', status: dryRun ? 'dry-run' : 'removed', path: candidate, detail: target });
  }
  return actions;
}

function discoveryDirectory(target, skillDest) {
  const clientHome = path.dirname(path.dirname(skillDest));
  return path.join(clientHome, target === 'claude' ? 'commands' : 'workflows');
}

function isQiongliSkillDir(skillDir) {
  try {
    const content = fs.readFileSync(path.join(skillDir, 'SKILL.md'), 'utf-8');
    return /^name:\s*(qiongli|qiongli-workflow)\s*$/m.test(content);
  } catch {
    return false;
  }
}

function isLegacySkillPath(skillDir, legacyName) {
  try {
    if (fs.lstatSync(skillDir).isSymbolicLink()) {
      return true;
    }
    const content = fs.readFileSync(path.join(skillDir, 'SKILL.md'), 'utf-8');
    return new RegExp(`^name:\\s*${escapeRegExp(legacyName)}\\s*$`, 'm').test(content);
  } catch {
    return false;
  }
}

function removeLegacySkillPath(legacyPath, legacyName, dryRun) {
  if (!isLegacySkillPath(legacyPath, legacyName)) {
    return 'kept';
  }
  removePath(legacyPath, dryRun);
  return dryRun ? 'dry-run' : 'removed';
}

function isManagedDiscoveryFile(item) {
  try {
    if (fs.lstatSync(item).isSymbolicLink()) {
      const target = symlinkTarget(item);
      return target.includes('qiongli-workflow') || target.includes(LEGACY_SKILL_NAME);
    }
    const content = fs.readFileSync(item, 'utf-8');
    return content.includes('qiongli-workflow') || content.includes('Qiongli');
  } catch {
    return false;
  }
}

function isManagedDiscoveryFileForSource(item, source) {
  try {
    if (fs.lstatSync(item).isSymbolicLink()) {
      const target = symlinkTarget(item);
      return target.includes('qiongli-workflow') || target.includes(LEGACY_SKILL_NAME);
    }
    if (fs.existsSync(source) && fs.readFileSync(item).equals(fs.readFileSync(source))) {
      return true;
    }
    const content = fs.readFileSync(item, 'utf-8');
    return content.includes('qiongli-workflow') || content.includes('Qiongli');
  } catch {
    return false;
  }
}

function isManagedDiscoverySymlink(item) {
  try {
    if (!fs.lstatSync(item).isSymbolicLink()) {
      return false;
    }
    const target = symlinkTarget(item);
    return target.includes('qiongli-workflow') || target.includes(LEGACY_SKILL_NAME);
  } catch {
    return false;
  }
}

function pathExistsOrSymlink(item) {
  try {
    fs.lstatSync(item);
    return true;
  } catch {
    return false;
  }
}

function symlinkTarget(item) {
  try {
    return fs.realpathSync(item);
  } catch {
    return fs.readlinkSync(item);
  }
}

function validateTarget(target) {
  if (target === 'all' || TARGETS.includes(target)) {
    return;
  }
  throw new Error(`Unsupported target: ${target}`);
}

function normalizeParts(parts) {
  if (!parts) {
    return null;
  }
  const parsed = String(parts)
    .split(',')
    .map((item) => item.trim().toLowerCase())
    .filter(Boolean)
    .map((item) => (item === 'global' ? 'globals' : item));
  if (parsed.includes('all') || parsed.includes('*')) {
    return PARTS;
  }
  for (const item of parsed) {
    if (!PARTS.includes(item)) {
      throw new Error(`Unsupported install part: ${item}`);
    }
  }
  return [...new Set(parsed)];
}

function removePath(target, dryRun) {
  if (dryRun) {
    return;
  }
  fs.rmSync(target, { recursive: true, force: true });
}

function escapeRegExp(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}
