import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';

const TARGETS = ['codex', 'claude', 'antigravity', 'hermes'];
const PARTS = ['globals', 'project', 'cli', 'mcp'];
const LEGACY_SKILL_NAME = 'research-paper-workflow';
const NPM_PLUGIN_MARKER = '.qiongli-npm-lite.json';

export function resolveTargetPaths({ env = process.env } = {}) {
  const home = env.HOME || env.USERPROFILE || os.homedir();
  const codexHome = env.CODEX_HOME || path.join(home, '.codex');
  const claudeHome = env.CLAUDE_CODE_HOME || path.join(home, '.claude');
  const antigravityHome = env.ANTIGRAVITY_HOME || path.join(home, '.gemini', 'antigravity');
  const hermesHome = env.HERMES_HOME || path.join(home, '.hermes');
  return {
    codex: path.join(codexHome, 'skills', 'qiongli-workflow'),
    claude: path.join(claudeHome, 'skills', 'qiongli-workflow'),
    antigravity: path.join(antigravityHome, 'skills', 'qiongli-workflow'),
    hermes: path.join(hermesHome, 'skills', 'qiongli-workflow'),
  };
}

function resolvePluginTargetPaths({ env = process.env } = {}) {
  const home = env.HOME || env.USERPROFILE || os.homedir();
  const claudeHome = env.CLAUDE_CODE_HOME || path.join(home, '.claude');
  const antigravityHome = env.ANTIGRAVITY_HOME || path.join(home, '.gemini', 'antigravity');
  const hermesHome = env.HERMES_HOME || path.join(home, '.hermes');
  return {
    codex: env.CODEX_PLUGIN_HOME || path.join(home, 'plugins', 'qiongli'),
    claude: env.CLAUDE_CODE_PLUGIN_HOME || path.join(claudeHome, 'plugins', 'qiongli'),
    antigravity: env.ANTIGRAVITY_PLUGIN_HOME || path.join(antigravityHome, 'plugins', 'qiongli'),
    hermes: env.HERMES_PLUGIN_HOME || path.join(hermesHome, 'plugins', 'qiongli'),
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
  surface = 'skills',
  overwrite = false,
  dryRun = false,
  subject = 'core',
  coverage = 'complete',
  parts = '',
  env = process.env,
  platform = process.platform,
} = {}) {
  const selectedSurface = normalizeSurface(surface);
  const installSkillSurface = selectedSurface === 'skills' || selectedSurface === 'both';
  const installPluginSurface = selectedSurface === 'plugin' || selectedSurface === 'both';
  const targetPaths = resolveTargetPaths({ env });
  const pluginTargetPaths = resolvePluginTargetPaths({ env });
  const selectedTargets = target === 'all' ? TARGETS : [target];
  let sourceVersion = readPackageVersion(packageRoot);
  let sourceSubject = subject;
  let sourceCoverage = coverage;
  const actions = [];
  const legacyResidues = [];
  const selectedParts = normalizeParts(parts);
  const installGlobals = !selectedParts || selectedParts.includes('globals');
  const installMcp = selectedParts?.includes('mcp') || false;

  let workflowSrc;
  if (installSkillSurface) {
    workflowSrc = resolveSubjectPayload({ packageRoot, subject, coverage });
    if (!fs.existsSync(path.join(workflowSrc, 'SKILL.md'))) {
      throw new Error(`Missing qiongli-workflow payload: ${workflowSrc}`);
    }
    sourceVersion = readSkillVersion(workflowSrc) || sourceVersion;
    sourceSubject = readSkillSubject(workflowSrc) || sourceSubject;
    sourceCoverage = readSkillCoverage(workflowSrc) || sourceCoverage;
  }

  if (installGlobals) {
    if (installSkillSurface) {
      for (const item of selectedTargets) {
        const dest = targetPaths[item];
        const legacyPath = path.join(path.dirname(dest), LEGACY_SKILL_NAME);
        if (fs.existsSync(legacyPath)) {
          const status = removeLegacySkillPath(legacyPath, LEGACY_SKILL_NAME, dryRun);
          legacyResidues.push({ target: item, legacyName: LEGACY_SKILL_NAME, path: legacyPath, status });
        }

        actions.push(copySkill({ src: workflowSrc, dest, mode, overwrite, dryRun, sourceVersion, sourceSubject, sourceCoverage }));

        if (item === 'claude' && actions.at(-1).status !== 'skip') {
          actions.push(...installWorkflowDiscovery({ target: item, skillDest: dest, dryRun, platform }));
        }
      }
    }

    if (installPluginSurface) {
      for (const item of selectedTargets) {
        const pluginSrc = resolvePluginPayload({ packageRoot, target: item });
        const dest = pluginTargetPaths[item];
        if (!pluginSrc) {
          actions.push(pluginUnavailableAction({ target: item, path: dest }));
          continue;
        }
        actions.push(copyPlugin({ src: pluginSrc, dest, mode, overwrite, dryRun, detail: item, platform, sourceVersion }));
      }
    }
  }

  if (installMcp) {
    actions.push(mcpGuidanceAction({ dryRun }));
  }

  return { sourceVersion, sourceSubject, sourceCoverage, actions, legacyResidues, targetPaths, pluginTargetPaths };
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
      if (target !== 'claude') {
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
  surface = 'skills',
  parts = '',
  dryRun = false,
  env = process.env,
} = {}) {
  validateTarget(target);
  const selectedSurface = normalizeSurface(surface);
  const removeSkillSurface = selectedSurface === 'skills' || selectedSurface === 'both';
  const removePluginSurface = selectedSurface === 'plugin' || selectedSurface === 'both';
  const selectedParts = normalizeParts(parts) || ['globals'];
  const actions = [];

  if (selectedParts.includes('globals')) {
    const targetPaths = resolveTargetPaths({ env });
    const pluginTargetPaths = resolvePluginTargetPaths({ env });
    const selectedTargets = target === 'all' ? TARGETS : [target];
    for (const item of selectedTargets) {
      if (removeSkillSurface) {
        const skillDest = targetPaths[item];
        actions.push(...removeWorkflowDiscovery({ target: item, skillDest, dryRun }));
        const legacyPath = path.join(path.dirname(skillDest), LEGACY_SKILL_NAME);
        if (fs.existsSync(legacyPath) && removeLegacySkillPath(legacyPath, LEGACY_SKILL_NAME, dryRun) !== 'kept') {
          actions.push({ label: 'Legacy Skill', status: dryRun ? 'dry-run' : 'removed', path: legacyPath, detail: item });
        }
        if (!fs.existsSync(skillDest)) {
          actions.push({ label: 'Skill', status: 'skip', path: skillDest, detail: 'not installed' });
        } else if (!isQiongliSkillDir(skillDest)) {
          actions.push({ label: 'Skill', status: 'skip', path: skillDest, detail: 'unmanaged qiongli-workflow directory' });
        } else {
          removePath(skillDest, dryRun);
          actions.push({ label: 'Skill', status: dryRun ? 'dry-run' : 'removed', path: skillDest, detail: item });
        }
      }

      if (removePluginSurface) {
        const pluginDest = pluginTargetPaths[item];
        if (!pathExistsOrSymlink(pluginDest)) {
          actions.push({ label: 'Plugin', status: 'skip', path: pluginDest, detail: 'not installed' });
        } else if (!isNpmManagedPluginDir(pluginDest)) {
          actions.push({ label: 'Plugin', status: 'skip', path: pluginDest, detail: 'unmanaged qiongli plugin directory' });
        } else {
          removeNpmPluginMarker(pluginDest, dryRun);
          removePath(pluginDest, dryRun);
          actions.push({ label: 'Plugin', status: dryRun ? 'dry-run' : 'removed', path: pluginDest, detail: item });
        }
      }
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

  if (selectedParts.includes('mcp')) {
    actions.push({
      label: 'MCP',
      status: 'manual',
      path: '<client config>',
      detail: 'Use qiongli remove --parts mcp through the Python CLI to remove managed MCP config',
    });
  }

  return { actions };
}

export function buildCheck({ packageRoot, subject = 'core', coverage = 'complete', env = process.env } = {}) {
  const packageJson = JSON.parse(fs.readFileSync(path.join(packageRoot, 'package.json'), 'utf-8'));
  const payload = resolveSubjectPayload({ packageRoot, subject, coverage });
  const targetPaths = resolveTargetPaths({ env });
  const pluginTargetPaths = resolvePluginTargetPaths({ env });
  const installed = {};
  for (const [target, skillDir] of Object.entries(targetPaths)) {
    const skillInstalled = isQiongliSkillDir(skillDir);
    const pluginDir = pluginTargetPaths[target];
    const pluginMarker = readNpmPluginMarker(pluginDir);
    const pluginInstalled = isQiongliPluginDir(pluginDir) && Boolean(pluginMarker);
    const pluginVersion = pluginMarker?.version || null;
    const skillVersion = readSkillVersion(skillDir) || null;
    const version = skillVersion || pluginVersion;
    const installedSubject = readSkillSubject(skillDir) || null;
    const installedCoverage = readSkillCoverage(skillDir) || null;
    installed[target] = {
      path: skillDir,
      installed: skillInstalled || pluginInstalled,
      surface: pluginInstalled ? (skillInstalled ? 'both' : 'plugin') : skillInstalled ? 'skills' : 'none',
      version,
      subject: installedSubject,
      coverage: installedCoverage,
      skill: {
        path: skillDir,
        installed: skillInstalled,
        version: skillVersion,
        subject: installedSubject,
        coverage: installedCoverage,
      },
      plugin: {
        path: pluginDir,
        installed: pluginInstalled,
        managed: pluginInstalled,
        version: pluginVersion,
        target: pluginMarker?.target || target,
      },
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

function copyPlugin({ src, dest, mode, overwrite, dryRun, detail, platform, sourceVersion }) {
  if (pathExistsOrSymlink(dest)) {
    if (!isNpmManagedPluginDir(dest)) {
      return { label: 'Plugin', status: 'skip', path: dest, detail: 'unmanaged qiongli plugin directory' };
    }
    if (!overwrite) {
      return { label: 'Plugin', status: 'skip', path: dest, detail: `already current ${detail}` };
    }
    removeNpmPluginMarker(dest, dryRun);
    removePath(dest, dryRun);
  }

  if (!dryRun) {
    fs.mkdirSync(path.dirname(dest), { recursive: true });
    if (mode === 'link') {
      fs.symlinkSync(src, dest, platform === 'win32' ? 'junction' : 'dir');
    } else {
      fs.cpSync(src, dest, { recursive: true, force: true });
    }
    writeNpmPluginMarker(dest, { target: detail, sourceVersion });
  }

  return { label: 'Plugin', status: 'ok', path: dest, detail: `installed ${detail}` };
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

function resolvePluginPayload({ packageRoot, target }) {
  if (target === 'hermes') {
    return '';
  }
  const targetPayload = path.join(packageRoot, 'payload', 'plugins', target, 'qiongli');
  if (isPluginPayloadDir(targetPayload, target)) {
    return targetPayload;
  }
  const sharedPayload = path.join(packageRoot, 'payload', 'plugins', 'qiongli');
  if (isPluginPayloadDir(sharedPayload, target)) {
    return sharedPayload;
  }
  return '';
}

function pluginUnavailableAction({ target, path: pluginPath }) {
  if (target === 'hermes') {
    return {
      label: 'Plugin',
      status: 'skip',
      path: pluginPath,
      detail: 'Hermes has no npm plugin-lite surface',
    };
  }
  return {
    label: 'Plugin',
    status: 'skip',
    path: pluginPath,
    detail: `plugin payload not bundled for ${target}`,
  };
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

function mcpGuidanceAction({ dryRun }) {
  return {
    label: 'MCP',
    status: dryRun ? 'dry-run' : 'manual',
    path: '<client config>',
    detail: 'Use qiongli mcp serve --transport stdio as the unified full MCP server',
  };
}

function removeWorkflowDiscovery({ target, skillDest, dryRun }) {
  if (target !== 'claude') {
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
  return path.join(clientHome, 'commands');
}

function isQiongliSkillDir(skillDir) {
  try {
    const content = fs.readFileSync(path.join(skillDir, 'SKILL.md'), 'utf-8');
    return /^name:\s*(qiongli|qiongli-workflow)\s*$/m.test(content);
  } catch {
    return false;
  }
}

function isQiongliPluginDir(pluginDir) {
  for (const rel of [
    'manifest.json',
    'plugin.json',
    path.join('.codex-plugin', 'plugin.json'),
    path.join('.claude-plugin', 'plugin.json'),
  ]) {
    try {
      const payload = JSON.parse(fs.readFileSync(path.join(pluginDir, rel), 'utf-8'));
      if (payload && typeof payload === 'object' && payload.name === 'qiongli') {
        return true;
      }
    } catch {
      continue;
    }
  }
  return false;
}

function isNpmManagedPluginDir(pluginDir) {
  const marker = readNpmPluginMarker(pluginDir);
  if (!marker) {
    return false;
  }
  if (pathExistsOrSymlink(pluginDir) && fs.lstatSync(pluginDir).isSymbolicLink()) {
    return true;
  }
  return isQiongliPluginDir(pluginDir);
}

function readNpmPluginMarker(pluginDir) {
  for (const markerPath of [pluginMarkerPath(pluginDir), pluginSidecarMarkerPath(pluginDir)]) {
    try {
      const marker = JSON.parse(fs.readFileSync(markerPath, 'utf-8'));
      if (
        marker
        && typeof marker === 'object'
        && marker.managed_by === 'qiongli-npm'
        && marker.surface === 'plugin-lite'
      ) {
        return marker;
      }
    } catch {
      continue;
    }
  }
  return null;
}

function writeNpmPluginMarker(pluginDir, { target, sourceVersion } = {}) {
  const marker = {
    managed_by: 'qiongli-npm',
    surface: 'plugin-lite',
    target,
    version: sourceVersion || '',
  };
  const markerPath = fs.lstatSync(pluginDir).isSymbolicLink()
    ? pluginSidecarMarkerPath(pluginDir)
    : pluginMarkerPath(pluginDir);
  fs.writeFileSync(markerPath, `${JSON.stringify(marker, null, 2)}\n`);
}

function removeNpmPluginMarker(pluginDir, dryRun) {
  if (dryRun) {
    return;
  }
  for (const markerPath of [pluginMarkerPath(pluginDir), pluginSidecarMarkerPath(pluginDir)]) {
    fs.rmSync(markerPath, { force: true });
  }
}

function pluginMarkerPath(pluginDir) {
  return path.join(pluginDir, NPM_PLUGIN_MARKER);
}

function pluginSidecarMarkerPath(pluginDir) {
  return `${pluginDir}${NPM_PLUGIN_MARKER}`;
}

function isPluginPayloadDir(pluginDir, target) {
  if (!fs.existsSync(pluginDir)) {
    return false;
  }
  return pluginManifestForTarget(pluginDir, target);
}

function pluginManifestForTarget(pluginDir, target) {
  const manifestsByTarget = {
    codex: [path.join('.codex-plugin', 'plugin.json'), 'manifest.json'],
    claude: [path.join('.claude-plugin', 'plugin.json'), 'manifest.json'],
    antigravity: ['plugin.json'],
  };
  for (const rel of manifestsByTarget[target] || []) {
    try {
      const payload = JSON.parse(fs.readFileSync(path.join(pluginDir, rel), 'utf-8'));
      if (payload && typeof payload === 'object' && payload.name === 'qiongli') {
        return true;
      }
    } catch {
      continue;
    }
  }
  return false;
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

function normalizeSurface(surface) {
  const normalized = surface || 'skills';
  if (normalized === 'skills' || normalized === 'plugin' || normalized === 'both') {
    return normalized;
  }
  throw new Error(`Unsupported surface: ${normalized}`);
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

function readPackageVersion(packageRoot) {
  try {
    const packageJson = JSON.parse(fs.readFileSync(path.join(packageRoot, 'package.json'), 'utf-8'));
    return typeof packageJson.version === 'string' ? packageJson.version : '';
  } catch {
    return '';
  }
}

function escapeRegExp(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}
