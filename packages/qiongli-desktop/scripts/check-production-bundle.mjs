import {
  readdirSync,
  readFileSync,
  statSync
} from 'node:fs';
import { spawnSync } from 'node:child_process';
import {
  dirname,
  join
} from 'node:path';
import { fileURLToPath } from 'node:url';

const packageRoot = dirname(dirname(fileURLToPath(import.meta.url)));
const clientRoot = join(packageRoot, '.svelte-kit', 'output', 'client');
const manifestPath = join(clientRoot, '.vite', 'manifest.json');
const manifest = JSON.parse(readFileSync(manifestPath, 'utf8'));
// The shadcn-svelte shell is intentionally richer than the pre-migration UI.
// Keep modest headroom above the post-migration deferred baseline (368 KiB).
const maximumShellBytes = 400 * 1024;
const maximumEnglishCatalogBytes = 96 * 1024;
const maximumChineseCatalogBytes = 96 * 1024;
const maximumValidatedClientBytes = 165 * 1024;
// These whole-client limits intentionally leave less than 7% headroom above
// the Alpha 3 planning baseline. A new feature must defer, replace, or remove
// code instead of silently growing the installed product.
const maximumClientBytes = 2000 * 1024;
const maximumJavaScriptBytes = 1650 * 1024;
const maximumCssBytes = 245 * 1024;
const maximumJavaScriptAssetBytes = 460 * 1024;
const maximumClientFiles = 90;
const fixtureMarkers = [
  'dev-fixture-command-unsupported',
  '10.1000/qiongli-fixture',
  'fixture-consolidation-acknowledgement'
];

const shellEntry = Object.entries(manifest)
  .find(([, entry]) => entry.name === 'nodes/0');
if (!shellEntry) fail('missing SvelteKit shell entry');

const shellFiles = new Set();
visitEntry(shellEntry[0]);
const shellBytes = [...shellFiles].reduce(
  (total, file) => total + statSync(join(clientRoot, file)).size,
  0
);
if (shellBytes > maximumShellBytes) {
  fail(
    `shared shell is ${formatBytes(shellBytes)}; budget is ${formatBytes(maximumShellBytes)}`
  );
}

const clientFiles = collectFiles(clientRoot);
const clientBytes = totalBytes(clientFiles);
const javaScriptFiles = clientFiles.filter((file) => file.endsWith('.js'));
const javaScriptBytes = totalBytes(javaScriptFiles);
const cssFiles = clientFiles.filter((file) => file.endsWith('.css'));
const cssBytes = totalBytes(cssFiles);
const largestJavaScriptBytes = Math.max(
  0,
  ...javaScriptFiles.map((file) => statSync(file).size)
);

assertBudget('client output', clientBytes, maximumClientBytes);
assertBudget('client JavaScript', javaScriptBytes, maximumJavaScriptBytes);
assertBudget('client CSS', cssBytes, maximumCssBytes);
assertBudget(
  'largest JavaScript asset',
  largestJavaScriptBytes,
  maximumJavaScriptAssetBytes
);
if (clientFiles.length > maximumClientFiles) {
  fail(`client emitted ${clientFiles.length} files; budget is ${maximumClientFiles}`);
}

assertDynamicEntry('src/lib/components/app/ConfirmationDialog.svelte');
assertDynamicEntry('src/lib/features/academic-graph/cytoscape-adapter.ts');
// Tauri freezes Object.prototype before deferred UI modules load.
assertFrozenPrototypeCytoscapeImport();
assertDynamicEntry('src/lib/i18n/locales/en.ts');
assertOutsideShell('src/lib/i18n/locales/en.ts');
assertMaximumEntrySize(
  'src/lib/i18n/locales/en.ts',
  maximumEnglishCatalogBytes
);
assertDynamicEntry('src/lib/i18n/locales/zh-CN.ts');
assertOutsideShell('src/lib/i18n/locales/zh-CN.ts');
assertMaximumEntrySize(
  'src/lib/i18n/locales/zh-CN.ts',
  maximumChineseCatalogBytes
);
assertDynamicEntry('src/lib/validated-app-client.ts');
assertOutsideShell('src/lib/validated-app-client.ts');
assertMaximumEntrySize(
  'src/lib/validated-app-client.ts',
  maximumValidatedClientBytes
);

if (findEntry('src/lib/dev-transport.ts')) {
  fail('development fixture transport was emitted into the production manifest');
}

for (const file of new Set(
  Object.values(manifest)
    .map((entry) => entry.file)
    .filter((file) => file.endsWith('.js'))
)) {
  const source = readFileSync(join(clientRoot, file), 'utf8');
  const marker = fixtureMarkers.find((candidate) => source.includes(candidate));
  if (marker) fail(`production asset ${file} contains fixture marker ${marker}`);
}

console.log(
  `Desktop bundle contract passed: ${formatBytes(clientBytes)} in ${clientFiles.length} files; `
  + `${formatBytes(javaScriptBytes)} JavaScript; ${formatBytes(cssBytes)} CSS; `
  + `${formatBytes(largestJavaScriptBytes)} largest JavaScript asset; `
  + `${formatBytes(shellBytes)} shared shell; `
  + 'validated client, locale catalogs, confirmation dialog, and Cytoscape deferred; '
  + 'development fixture excluded.'
);

function collectFiles(directory) {
  return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const path = join(directory, entry.name);
    return entry.isDirectory() ? collectFiles(path) : [path];
  });
}

function totalBytes(files) {
  return files.reduce((total, file) => total + statSync(file).size, 0);
}

function assertBudget(label, bytes, maximumBytes) {
  if (bytes > maximumBytes) {
    fail(`${label} is ${formatBytes(bytes)}; budget is ${formatBytes(maximumBytes)}`);
  }
}

function visitEntry(key) {
  const entry = manifest[key];
  if (!entry || shellFiles.has(entry.file)) return;
  if (entry.file.endsWith('.js')) shellFiles.add(entry.file);
  for (const dependency of entry.imports ?? []) visitEntry(dependency);
}

function assertDynamicEntry(suffix) {
  const match = findEntry(suffix);
  if (!match || match[1].isDynamicEntry !== true) {
    fail(`${suffix} must remain a dynamic production entry`);
  }
}

function assertOutsideShell(suffix) {
  const match = findEntry(suffix);
  if (!match || shellFiles.has(match[1].file)) {
    fail(`${suffix} must stay outside the shared shell`);
  }
}

function assertMaximumEntrySize(suffix, maximumBytes) {
  const match = findEntry(suffix);
  if (!match) fail(`missing production entry ${suffix}`);
  const bytes = statSync(join(clientRoot, match[1].file)).size;
  if (bytes > maximumBytes) {
    fail(
      `${suffix} is ${formatBytes(bytes)}; budget is ${formatBytes(maximumBytes)}`
    );
  }
}

function assertFrozenPrototypeCytoscapeImport() {
  const result = spawnSync(
    process.execPath,
    [
      '--input-type=module',
      '--eval',
      "Object.freeze(Object.prototype); await import('cytoscape');"
    ],
    { cwd: packageRoot, encoding: 'utf8' }
  );
  if (result.status !== 0) {
    fail(`Cytoscape cannot load with Tauri prototype hardening: ${result.stderr.trim()}`);
  }
}

function findEntry(suffix) {
  return Object.entries(manifest)
    .find(([key, entry]) => key.endsWith(suffix) || entry.src?.endsWith(suffix));
}

function formatBytes(bytes) {
  return `${(bytes / 1024).toFixed(1)} KiB`;
}

function fail(message) {
  throw new Error(`desktop-production-bundle-contract: ${message}`);
}
