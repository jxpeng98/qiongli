import fs from 'node:fs';
import path from 'node:path';
import { posix as pathPosix } from 'node:path';

const OFFICIAL_SUBJECTS = new Set([
  'auto',
  'core',
  'economics',
  'accounting',
  'business',
  'finance',
  'political-economy',
  'geoeconomics',
  'economics-accounting',
]);
const STRICTNESS_CHOICES = new Set(['standard', 'high']);
const MANIFEST_REL = '.qiongli/guidance_manifest.yaml';
const KNOWN_FIELDS = [
  'active_subject',
  'secondary_subjects',
  'venue_profiles',
  'method_lenses',
  'strictness',
];
const LIST_FIELDS = new Set(['secondary_subjects', 'venue_profiles', 'method_lenses']);
const LOCAL_GUIDANCE_TEXT = [
  '# Local Guidance',
  '',
  'Project-local guidance for this repository.',
  'Add durable local rules here when they should supplement canonical workflow guidance.',
  '',
].join('\n');

export function initProject({ projectDir = '.', dryRun = false } = {}) {
  const root = normalizeProjectRoot(projectDir);
  const current = projectStatus({ projectDir: root });
  const actions = buildInitActions(root, current.exists);
  const changed = actions.some((entry) => entry.status === 'create');

  if (!dryRun) {
    ensureProjectScaffolding(root, { writeManifest: !current.exists });
  }

  return {
    action: 'init',
    changed,
    dry_run: dryRun,
    state: dryRun ? current : projectStatus({ projectDir: root }),
    actions,
  };
}

export function projectStatus({ projectDir = '.' } = {}) {
  const root = normalizeProjectRoot(projectDir);
  return loadProjectManifest(root).state;
}

function loadProjectManifest(root) {
  const manifestPath = path.join(root, MANIFEST_REL);
  if (!fs.existsSync(manifestPath)) {
    return {
      state: buildState({
        exists: false,
        manifest: defaultManifest(),
        warnings: [],
      }),
      preservedBlocks: [],
    };
  }

  const parsed = parseManifest(fs.readFileSync(manifestPath, 'utf-8'));
  return {
    state: buildState({
      exists: true,
      manifest: normalizeManifest(parsed.manifest),
      warnings: parsed.warnings,
    }),
    preservedBlocks: parsed.preservedBlocks,
  };
}

export function setProjectSubject({ projectDir = '.', subject, dryRun = false } = {}) {
  const root = normalizeProjectRoot(projectDir);
  const loaded = loadProjectManifest(root);
  const current = loaded.state;
  const nextManifest = normalizeManifest({
    ...current.manifest,
    active_subject: subject,
  });

  if (!dryRun) {
    ensureManifestParent(root);
    fs.writeFileSync(
      path.join(root, MANIFEST_REL),
      renderManifest(nextManifest, { preservedBlocks: loaded.preservedBlocks }),
      'utf-8',
    );
  }

  return {
    action: 'set-subject',
    changed: current.exists ? current.manifest.active_subject !== nextManifest.active_subject : true,
    dry_run: dryRun,
    state: buildState({
      exists: dryRun ? current.exists : true,
      manifest: nextManifest,
      warnings: current.warnings,
    }),
    actions: [
      {
        status: current.exists ? 'update' : 'create',
        path: MANIFEST_REL,
        detail: `active_subject=${nextManifest.active_subject}`,
      },
    ],
  };
}

export function renderProjectResult(result, { json = false } = {}) {
  if (json) {
    return `${JSON.stringify(result, null, 2)}\n`;
  }

  const lines = [
    `qiongli project ${result.action}`,
    `source: ${result.state.exists ? result.state.path : 'implicit defaults'}`,
    `active_subject: ${result.state.manifest.active_subject}`,
    `secondary_subjects: ${displayList(result.state.manifest.secondary_subjects)}`,
    `venue_profiles: ${displayList(result.state.manifest.venue_profiles)}`,
    `method_lenses: ${displayList(result.state.manifest.method_lenses)}`,
    `strictness: ${result.state.manifest.strictness}`,
  ];
  if (result.actions?.length) {
    for (const action of result.actions) {
      lines.push(`[${action.status}] ${action.path}${action.detail ? ` (${action.detail})` : ''}`);
    }
  }
  if (result.state.warnings?.length) {
    lines.push(`warnings: ${result.state.warnings.join('; ')}`);
  }
  return `${lines.join('\n')}\n`;
}

function buildInitActions(root, manifestExists) {
  return [
    {
      status: manifestExists ? 'exists' : 'create',
      path: MANIFEST_REL,
      detail: manifestExists ? 'kept existing manifest' : 'write default manifest',
    },
    buildPathAction(root, '.qiongli/local_guidance.md'),
    buildPathAction(root, '.qiongli/guidance.d/', { directory: true }),
    buildPathAction(root, '.qiongli/trace/', { directory: true }),
  ];
}

function buildPathAction(root, relativePath, { directory = false } = {}) {
  const absolutePath = path.join(root, relativePath.replace(/\/$/, ''));
  return {
    status: fs.existsSync(absolutePath) ? 'exists' : 'create',
    path: relativePath,
    detail: directory ? 'ensure directory' : 'ensure file',
  };
}

function ensureProjectScaffolding(root, { writeManifest }) {
  const qiongliDir = path.join(root, '.qiongli');
  fs.mkdirSync(qiongliDir, { recursive: true });
  fs.mkdirSync(path.join(qiongliDir, 'guidance.d'), { recursive: true });
  fs.mkdirSync(path.join(qiongliDir, 'trace'), { recursive: true });
  ensureFile(path.join(qiongliDir, 'local_guidance.md'), LOCAL_GUIDANCE_TEXT);
  if (writeManifest) {
    fs.writeFileSync(path.join(qiongliDir, 'guidance_manifest.yaml'), renderManifest(defaultManifest()), 'utf-8');
  }
}

function ensureManifestParent(root) {
  fs.mkdirSync(path.join(root, '.qiongli'), { recursive: true });
}

function ensureFile(filePath, content) {
  if (!fs.existsSync(filePath)) {
    fs.writeFileSync(filePath, content, 'utf-8');
  }
}

function buildState({ exists, manifest, warnings }) {
  return {
    exists,
    path: MANIFEST_REL,
    manifest,
    warnings: Array.isArray(warnings) ? warnings : [],
  };
}

function defaultManifest() {
  return {
    active_subject: 'auto',
    secondary_subjects: [],
    venue_profiles: [],
    method_lenses: [],
    strictness: 'standard',
  };
}

function normalizeManifest(manifest = {}) {
  const normalized = {
    active_subject: validateSubject(manifest.active_subject ?? 'auto', 'active_subject'),
    secondary_subjects: validateSubjectList(manifest.secondary_subjects, 'secondary_subjects'),
    venue_profiles: validateRelPathList(manifest.venue_profiles, 'venue_profiles'),
    method_lenses: validateRelPathList(manifest.method_lenses, 'method_lenses'),
    strictness: validateStrictness(manifest.strictness ?? 'standard'),
  };
  return normalized;
}

function validateSubject(value, field) {
  if (typeof value !== 'string') {
    throw new Error(`Unsupported ${field}: ${String(value)}`);
  }
  const normalized = value.trim();
  if (!OFFICIAL_SUBJECTS.has(normalized)) {
    throw new Error(`Unsupported ${field}: ${normalized}`);
  }
  return normalized;
}

function validateSubjectList(value, field) {
  return validateList(value, field).map((entry) => validateSubject(entry, field));
}

function validateRelPathList(value, field) {
  return validateList(value, field).map((entry) => validateRelPath(entry, field));
}

function validateList(value, field) {
  if (value == null) {
    return [];
  }
  if (!Array.isArray(value)) {
    throw new Error(`${field} must be a list`);
  }
  return value;
}

function validateRelPath(value, field) {
  if (typeof value !== 'string') {
    throw new Error(`${field} entries must be strings`);
  }
  const normalized = value.trim();
  const parsed = pathPosix.parse(normalized);
  const parts = normalized.split('/');
  if (
    !normalized
    || normalized.startsWith('/')
    || normalized.startsWith('\\')
    || normalized.includes('\\')
    || parsed.root
    || parts.some((part) => part === '' || part === '.' || part === '..')
  ) {
    throw new Error(`Unsupported ${field} entry: ${value}`);
  }
  return normalized;
}

function validateStrictness(value) {
  if (typeof value !== 'string') {
    throw new Error(`Unsupported strictness: ${String(value)}`);
  }
  const normalized = value.trim();
  if (!STRICTNESS_CHOICES.has(normalized)) {
    throw new Error(`Unsupported strictness: ${normalized}`);
  }
  return normalized;
}

function renderManifest(manifest, { preservedBlocks = [] } = {}) {
  const normalized = normalizeManifest(manifest);
  const blocks = [
    ...preservedBlocks,
    `active_subject: ${normalized.active_subject}`,
    renderListField('secondary_subjects', normalized.secondary_subjects),
    renderListField('venue_profiles', normalized.venue_profiles),
    renderListField('method_lenses', normalized.method_lenses),
    `strictness: ${normalized.strictness}`,
  ].filter(Boolean);
  return `${blocks.join('\n')}\n`;
}

function renderListField(key, values) {
  if (!values.length) {
    return `${key}: []`;
  }
  return `${key}:\n${values.map((value) => `  - ${value}`).join('\n')}`;
}

function parseManifest(text) {
  const manifest = defaultManifest();
  const warnings = [];
  const preservedBlocks = [];

  for (const block of splitTopLevelBlocks(text)) {
    const contentLines = block.filter((line) => line.trim() && !line.trimStart().startsWith('#'));
    if (!contentLines.length) {
      continue;
    }

    const firstLine = contentLines[0];
    const fieldMatch = firstLine.match(/^([A-Za-z_][A-Za-z0-9_]*):(?:\s*(.*))?$/);
    if (!fieldMatch) {
      throw new Error(`Malformed project manifest: ${firstLine}`);
    }

    const [, key, rawValue = ''] = fieldMatch;

    if (!KNOWN_FIELDS.includes(key)) {
      warnings.push(`Ignored unsupported manifest field: ${key}`);
      preservedBlocks.push(trimTrailingBlankLines(block).join('\n'));
      continue;
    }

    if (LIST_FIELDS.has(key)) {
      if (!rawValue) {
        manifest[key] = parseBlockList(contentLines.slice(1), key);
      } else {
        manifest[key] = parseInlineList(rawValue, key);
      }
      continue;
    }

    if (contentLines.length > 1) {
      throw new Error(`Malformed project manifest: ${key} must be a scalar`);
    }
    manifest[key] = parseScalar(rawValue);
  }

  return { manifest, warnings, preservedBlocks };
}

function splitTopLevelBlocks(text) {
  const blocks = [];
  let current = [];
  for (const line of text.split(/\r?\n/)) {
    if (/^[A-Za-z_][A-Za-z0-9_]*:/.test(line) && current.length) {
      blocks.push(current);
      current = [line];
    } else {
      current.push(line);
    }
  }
  if (current.length) {
    blocks.push(current);
  }
  return blocks;
}

function parseBlockList(lines, field) {
  const values = [];
  for (const line of lines) {
    if (!line.trim() || line.trimStart().startsWith('#')) {
      continue;
    }
    const listMatch = line.match(/^\s*-\s*(.+?)\s*$/);
    if (!listMatch) {
      throw new Error(`Malformed project manifest: ${field} must use block list entries`);
    }
    values.push(parseScalar(listMatch[1]));
  }
  return values;
}

function trimTrailingBlankLines(lines) {
  const trimmed = [...lines];
  while (trimmed.length && !trimmed.at(-1).trim()) {
    trimmed.pop();
  }
  return trimmed;
}

function parseInlineList(rawValue, field) {
  const trimmed = rawValue.trim();
  if (trimmed === '[]') {
    return [];
  }
  const bracketMatch = trimmed.match(/^\[(.*)\]$/);
  if (!bracketMatch) {
    throw new Error(`Malformed project manifest: ${field} must use [] or block list form`);
  }
  const body = bracketMatch[1].trim();
  if (!body) {
    return [];
  }
  return body.split(',').map((item) => parseScalar(item.trim())).filter(Boolean);
}

function parseScalar(value) {
  const trimmed = value.trim();
  if (
    (trimmed.startsWith('"') && trimmed.endsWith('"'))
    || (trimmed.startsWith('\'') && trimmed.endsWith('\''))
  ) {
    return trimmed.slice(1, -1);
  }
  return trimmed;
}

function displayList(values) {
  return values.length ? values.join(', ') : 'none';
}

function normalizeProjectRoot(projectDir) {
  return path.resolve(projectDir);
}
