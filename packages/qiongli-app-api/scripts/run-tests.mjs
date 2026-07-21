import { spawnSync } from 'node:child_process';
import { existsSync, mkdtempSync, rmdirSync, unlinkSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const packageRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const nativeManifest = resolve(packageRoot, '../qiongli-native/Cargo.toml');
const cargo = process.env.CARGO || 'cargo';
const generated = spawnSync(cargo, [
  'run',
  '--quiet',
  '--locked',
  '--manifest-path',
  nativeManifest,
  '--package',
  'qiongli',
  '--example',
  'app_api_contract_fixture'
], {
  cwd: packageRoot,
  encoding: 'utf8',
  maxBuffer: 1024 * 1024
});

if (generated.error) {
  throw generated.error;
}
if (generated.status !== 0) {
  process.stderr.write(generated.stderr);
  exitLikeChild(generated, 'Rust App API contract generator');
}

try {
  JSON.parse(generated.stdout);
} catch (error) {
  process.stderr.write(`Rust App API contract generator emitted invalid JSON: ${String(error)}\n`);
  process.exit(1);
}

const vitestCli = join(packageRoot, 'node_modules', 'vitest', 'vitest.mjs');
const fixtureRoot = mkdtempSync(join(tmpdir(), 'qiongli-app-contract-'));
const fixtureModule = join(fixtureRoot, 'contract.mjs');
let tests;
try {
  writeFileSync(fixtureModule, `export default ${generated.stdout};\n`, { mode: 0o600 });
  tests = spawnSync(process.execPath, [vitestCli, 'run', ...process.argv.slice(2)], {
    cwd: packageRoot,
    env: {
      ...process.env,
      QIONGLI_APP_CONTRACT_MODULE: pathToFileURL(fixtureModule).href
    },
    stdio: 'inherit'
  });
} finally {
  if (existsSync(fixtureModule)) unlinkSync(fixtureModule);
  rmdirSync(fixtureRoot);
}

if (tests.error) {
  throw tests.error;
}
exitLikeChild(tests, 'Vitest');

function exitLikeChild(result, label) {
  if (result.signal) {
    process.stderr.write(`${label} terminated by ${result.signal}\n`);
    process.kill(process.pid, result.signal);
    process.exit(1);
  }
  process.exit(result.status ?? 1);
}
