#!/usr/bin/env node

import { createHash } from 'node:crypto';
import {
  existsSync,
  lstatSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  realpathSync,
  renameSync,
  rmSync,
  writeFileSync
} from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, isAbsolute, join, relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';

const REPOSITORY_ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const SHA256 = /^[0-9a-f]{64}$/;
const PRODUCT_COMMIT = /^[0-9a-f]{40}$/;
const REQUIRED_CHECKS = [
  'exact-source-clean',
  'negative-native-readiness',
  'negative-desktop-readiness',
  'migration-preview-apply',
  'private-state-exclusion',
  'source-inventory-retained',
  'deterministic-rebuild',
  'fresh-process-reopen',
  'canonical-semantic-authority',
  'stable-id-query',
  'relation-query',
  'app-node-artifact-read',
  'app-edge-artifact-read',
  'desktop-readiness-layout',
  'desktop-search-focus',
  'desktop-source-inspection'
].sort();

class AcceptanceError extends Error {
  constructor(reasonCode) {
    super(reasonCode);
    this.reasonCode = reasonCode;
  }
}

function fail(reasonCode) {
  throw new AcceptanceError(reasonCode);
}

function usage() {
  return `PLT-322 migrated-project Graph v1 acceptance

Usage:
  node scripts/plt322_migrated_graph_acceptance.mjs --source <repository-path> --receipt <outside-repository-json>
`;
}

function parseArguments(argv) {
  if (argv.length === 1 && ['-h', '--help'].includes(argv[0])) return { help: true };
  const parsed = { help: false, source: null, receipt: null };
  for (let index = 0; index < argv.length; index += 2) {
    const option = argv[index];
    const value = argv[index + 1];
    if (!['--source', '--receipt'].includes(option) || !value || value.startsWith('--')) {
      fail('plt322-arguments-invalid');
    }
    const key = option === '--source' ? 'source' : 'receipt';
    if (parsed[key] !== null) fail('plt322-arguments-invalid');
    parsed[key] = value;
  }
  if (!parsed.source || !parsed.receipt) fail('plt322-arguments-required');
  return parsed;
}

function run(command, args, options = {}) {
  const outcome = spawnSync(command, args, {
    cwd: options.cwd ?? REPOSITORY_ROOT,
    env: options.env ?? process.env,
    encoding: options.binary ? null : 'utf8',
    input: options.input,
    maxBuffer: 32 * 1024 * 1024,
    timeout: options.timeout ?? 300_000
  });
  if (outcome.error || outcome.status !== 0) {
    if (process.env.QIONGLI_ACCEPTANCE_DIAGNOSTICS === '1') {
      const stdout = Buffer.isBuffer(outcome.stdout)
        ? outcome.stdout.toString('utf8')
        : outcome.stdout ?? '';
      const stderr = Buffer.isBuffer(outcome.stderr)
        ? outcome.stderr.toString('utf8')
        : outcome.stderr ?? '';
      process.stderr.write(`${[
        `command: ${command} ${args.join(' ')}`,
        `status: ${String(outcome.status)}`,
        stdout,
        stderr,
        outcome.error?.message ?? ''
      ].join('\n').slice(0, 32 * 1024)}\n`);
    }
    fail(options.reasonCode ?? 'plt322-command-failed');
  }
  return outcome.stdout;
}

function git(args, options = {}) {
  return run('git', args, { ...options, reasonCode: options.reasonCode ?? 'plt322-git-failed' });
}

function repositorySource(input) {
  let absolute;
  try {
    absolute = realpathSync(resolve(REPOSITORY_ROOT, input));
  } catch {
    fail('plt322-source-missing');
  }
  if (!lstatSync(absolute).isDirectory()) fail('plt322-source-invalid');
  const repositoryRelative = relative(REPOSITORY_ROOT, absolute).replaceAll('\\', '/');
  if (!repositoryRelative
    || repositoryRelative === '..'
    || repositoryRelative.startsWith('../')
    || isAbsolute(repositoryRelative)) {
    fail('plt322-source-outside-repository');
  }
  return { absolute, repositoryRelative };
}

function assertCleanExactSource(source) {
  if (git(['status', '--porcelain=v1', '--untracked-files=all']).trim() !== '') {
    fail('plt322-product-source-dirty');
  }
  const commit = git(['rev-parse', 'HEAD']).trim();
  if (!PRODUCT_COMMIT.test(commit)) fail('plt322-product-commit-invalid');
  const tracked = git([
    'ls-tree', '-r', '--name-only', commit, '--', source.repositoryRelative
  ]).trim().split('\n').filter(Boolean);
  if (tracked.length === 0) fail('plt322-source-not-tracked');
  if (tracked.includes(`${source.repositoryRelative}/context/project_manifest.json`)) {
    fail('plt322-source-already-2x');
  }
  return commit;
}

function materializeExactSource(root, source, commit) {
  const snapshot = join(root, 'snapshot');
  mkdirSync(snapshot, { mode: 0o700 });
  const archive = git([
    'archive', '--format=tar', commit, '--', source.repositoryRelative
  ], { binary: true, reasonCode: 'plt322-source-archive-failed' });
  run('tar', ['-xf', '-', '-C', snapshot], {
    input: archive,
    reasonCode: 'plt322-source-materialization-failed'
  });
  const exactSource = join(snapshot, source.repositoryRelative);
  const inventory = inventoryDigest(exactSource);
  if (existsSync(join(exactSource, '.qiongli'))
    || existsSync(join(exactSource, '.claude'))
    || existsSync(join(exactSource, 'context', 'project_manifest.json'))) {
    fail('plt322-exact-source-boundary-invalid');
  }
  mkdirSync(join(exactSource, '.qiongli'), { mode: 0o700 });
  writeFileSync(join(exactSource, '.qiongli', 'session.json'), '{}\n', { mode: 0o600 });
  mkdirSync(join(exactSource, '.claude', 'transcripts'), { recursive: true, mode: 0o700 });
  writeFileSync(
    join(exactSource, '.claude', 'transcripts', 'acceptance.txt'),
    'synthetic private exclusion marker\n',
    { mode: 0o600 }
  );
  return { exactSource, inventory };
}

function inventoryDigest(root) {
  const hash = createHash('sha256');
  const visit = (directory, prefix = '') => {
    for (const entry of readdirSync(directory, { withFileTypes: true })
      .sort((left, right) => left.name < right.name ? -1 : left.name > right.name ? 1 : 0)) {
      const path = join(directory, entry.name);
      const relativePath = prefix ? `${prefix}/${entry.name}` : entry.name;
      if (entry.isDirectory()) visit(path, relativePath);
      else {
        const metadata = lstatSync(path);
        if (!entry.isFile() || metadata.isSymbolicLink()) fail('plt322-source-entry-invalid');
        const bytes = readFileSync(path);
        hash.update(relativePath).update('\0').update(String(bytes.length)).update('\0').update(bytes);
      }
    }
  };
  visit(root);
  return hash.digest('hex');
}

function readFacts(path) {
  let bytes;
  try {
    bytes = readFileSync(path);
  } catch {
    fail('plt322-representative-check-skipped');
  }
  if (bytes.length < 2 || bytes.length > 256 * 1024) fail('plt322-facts-invalid');
  try {
    return JSON.parse(bytes.toString('utf8'));
  } catch {
    fail('plt322-facts-invalid');
  }
}

function validateFacts(facts, sourceInventoryDigest) {
  const digests = [
    facts.migrationPlanDigest,
    facts.sourceInventoryDigest,
    facts.migrationInputDigest,
    facts.analysisResultsDigest,
    facts.projectionDigest,
    facts.graphSourceDigest
  ];
  if (facts.schemaVersion !== 1
    || !/^prj_[0-9a-f]{32}$/.test(facts.projectId ?? '')
    || !/^grp_[0-9a-f]{64}$/.test(facts.projectionId ?? '')
    || !/^gix_[0-9a-f]{64}$/.test(facts.indexId ?? '')
    || !digests.every((digest) => SHA256.test(digest ?? ''))
    || facts.sourceInventoryDigest !== sourceInventoryDigest
    || facts.sourceRetained !== true
    || facts.excludedEntryCount < 2
    || !Number.isSafeInteger(facts.copiedFileCount)
    || facts.copiedFileCount < 1
    || facts.nodeCount < 1
    || facts.semanticNodeCount < 1
    || facts.edgeCount < 1
    || facts.diagnosticCount !== 0
    || facts.readinessState !== 'visualizable'
    || facts.reasonCode !== 'academic-graph-visualizable'
    || !Array.isArray(facts.nodeTypes)
    || !Array.isArray(facts.relations)
    || !facts.relations.some((relation) => relation !== 'contains')
    || !Array.isArray(facts.checks)) {
    fail('plt322-facts-invalid');
  }
  return facts;
}

function assertRequiredChecks(factsChecks) {
  const observed = [...new Set([
    'exact-source-clean',
    'negative-native-readiness',
    'negative-desktop-readiness',
    ...factsChecks
  ])].sort();
  if (JSON.stringify(observed) !== JSON.stringify(REQUIRED_CHECKS)) {
    fail('plt322-required-check-skipped');
  }
  return observed;
}

function writeReceipt(path, receipt) {
  const absolute = resolve(REPOSITORY_ROOT, path);
  const repositoryRelative = relative(REPOSITORY_ROOT, absolute);
  if (!repositoryRelative.startsWith('..') || isAbsolute(repositoryRelative)) {
    fail('plt322-receipt-must-be-outside-repository');
  }
  if (existsSync(absolute) && lstatSync(absolute).isSymbolicLink()) {
    fail('plt322-receipt-path-invalid');
  }
  mkdirSync(dirname(absolute), { recursive: true, mode: 0o700 });
  const body = JSON.stringify(receipt);
  const rendered = `${JSON.stringify({
    ...receipt,
    evidenceDigest: sha256(Buffer.from(body))
  }, null, 2)}\n`;
  assertRedacted(JSON.parse(rendered));
  const temporary = `${absolute}.tmp-${process.pid}`;
  writeFileSync(temporary, rendered, { mode: 0o600 });
  renameSync(temporary, absolute);
  return absolute;
}

function assertRedacted(value) {
  const visit = (item) => {
    if (typeof item === 'string') {
      if (item.includes(REPOSITORY_ROOT)
        || item.startsWith('/Users/')
        || item.startsWith('/private/')
        || item.startsWith('/tmp/')) {
        fail('plt322-receipt-path-leak');
      }
      return;
    }
    if (Array.isArray(item)) item.forEach(visit);
    else if (item && typeof item === 'object') Object.values(item).forEach(visit);
  };
  visit(value);
}

function sha256(bytes) {
  return createHash('sha256').update(bytes).digest('hex');
}

function main() {
  const args = parseArguments(process.argv.slice(2));
  if (args.help) {
    process.stdout.write(usage());
    return;
  }
  const source = repositorySource(args.source);
  const productCommit = assertCleanExactSource(source);
  const executionRoot = mkdtempSync(join(realpathSync(tmpdir()), 'qiongli-plt322-coordinator-'));
  try {
    const materialized = materializeExactSource(executionRoot, source, productCommit);
    const factsPath = join(executionRoot, 'representative-facts.json');
    run('cargo', [
      'test', '--locked', '--manifest-path', 'packages/qiongli-native/Cargo.toml',
      '-p', 'qiongli-project', 'academic_graph_readiness'
    ], { reasonCode: 'plt322-native-negative-controls-failed' });
    run('cargo', [
      'build', '--locked', '--manifest-path', 'packages/qiongli-native/Cargo.toml',
      '--package', 'qiongli', '--bin', 'qiongli'
    ], { reasonCode: 'plt322-native-build-failed' });
    const vitest = join(
      REPOSITORY_ROOT,
      'packages/qiongli-desktop/node_modules/.bin/vitest'
    );
    run(vitest, [
      'run',
      'src/lib/features/academic-graph/readiness.test.ts'
    ], {
      cwd: join(REPOSITORY_ROOT, 'packages/qiongli-desktop'),
      reasonCode: 'plt322-desktop-negative-controls-failed',
      env: process.env
    });
    const native = join(
      REPOSITORY_ROOT,
      'packages/qiongli-native/target/debug',
      process.platform === 'win32' ? 'qiongli.exe' : 'qiongli'
    );
    run(vitest, [
      'run',
      'src/lib/features/academic-graph/representative-migrated-project.acceptance.test.ts'
    ], {
      cwd: join(REPOSITORY_ROOT, 'packages/qiongli-desktop'),
      env: {
        ...process.env,
        PLT322_MIGRATION_SOURCE: materialized.exactSource,
        PLT322_NATIVE_BINARY: native,
        PLT322_FACTS_PATH: factsPath,
        PLT322_SOURCE_INVENTORY_DIGEST: materialized.inventory
      },
      timeout: 180_000,
      reasonCode: 'plt322-representative-project-failed'
    });
    const facts = validateFacts(readFacts(factsPath), materialized.inventory);
    const checks = assertRequiredChecks(facts.checks);
    const receiptPath = writeReceipt(args.receipt, {
      schemaVersion: 1,
      documentKind: 'qiongli-plt322-migrated-graph-acceptance',
      status: 'passed',
      productCommit,
      source: {
        repositoryRelativePath: source.repositoryRelative,
        inventoryDigest: facts.sourceInventoryDigest,
        migrationInputDigest: facts.migrationInputDigest
      },
      analysisResultsDigest: facts.analysisResultsDigest,
      migration: {
        projectId: facts.projectId,
        planDigest: facts.migrationPlanDigest,
        copiedFileCount: facts.copiedFileCount,
        excludedEntryCount: facts.excludedEntryCount,
        sourceRetained: facts.sourceRetained
      },
      graph: {
        projectionId: facts.projectionId,
        projectionDigest: facts.projectionDigest,
        graphSourceDigest: facts.graphSourceDigest,
        indexId: facts.indexId,
        nodeCount: facts.nodeCount,
        semanticNodeCount: facts.semanticNodeCount,
        edgeCount: facts.edgeCount,
        diagnosticCount: facts.diagnosticCount,
        readinessState: facts.readinessState,
        reasonCode: facts.reasonCode,
        nodeTypes: facts.nodeTypes,
        relations: facts.relations
      },
      checkIds: checks
    });
    process.stdout.write(`${JSON.stringify({ status: 'passed', receipt: receiptPath })}\n`);
  } finally {
    rmSync(executionRoot, { recursive: true, force: true });
  }
}

try {
  main();
} catch (error) {
  const reason = error instanceof AcceptanceError
    ? error.reasonCode
    : 'plt322-acceptance-unexpected';
  if (!(error instanceof AcceptanceError)
    && process.env.QIONGLI_ACCEPTANCE_DIAGNOSTICS === '1') {
    process.stderr.write(`${error?.stack ?? String(error)}\n`);
  }
  process.stderr.write(`${reason}\n`);
  process.exitCode = 1;
}
