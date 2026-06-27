import { spawnSync as nodeSpawnSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import path from 'node:path';

export function checkPythonRuntime({
  candidates = ['python3', 'python'],
  spawnSync = nodeSpawnSync,
} = {}) {
  for (const candidate of candidates) {
    const version = spawnSync(candidate, [
      '-c',
      'import sys; print(f"{sys.version_info.major}.{sys.version_info.minor}.{sys.version_info.micro}")',
    ], { encoding: 'utf-8' });
    if (version.status !== 0) {
      continue;
    }

    const rawVersion = String(version.stdout || '').trim();
    const [major, minor] = rawVersion.split('.').map((item) => Number.parseInt(item, 10));
    if (major < 3 || (major === 3 && minor < 12)) {
      return {
        ok: false,
        python: candidate,
        version: rawVersion,
        message: `Python bridge requires Python 3.12+, found ${rawVersion}.`,
        hint: 'Install Python 3.12+ or use `pipx install qiongli`.',
      };
    }

    const yaml = spawnSync(candidate, ['-c', 'import yaml'], { encoding: 'utf-8' });
    if (yaml.status !== 0) {
      return {
        ok: false,
        python: candidate,
        version: rawVersion,
        message: 'Python bridge requires PyYAML.',
        hint: `${candidate} -m pip install PyYAML, or use \`pipx install qiongli\`.`,
      };
    }

    return {
      ok: true,
      python: candidate,
      version: rawVersion,
      message: `Python bridge ready (${candidate} ${rawVersion}).`,
      hint: '',
    };
  }

  return {
    ok: false,
    python: '',
    version: '',
    message: 'Python runtime not found.',
    hint: 'Install Python 3.12+ and PyYAML, or use `pipx install qiongli`.',
  };
}

export function runBridgeCommand({
  packageRoot,
  command,
  args,
  cwd = process.cwd(),
  env = process.env,
  stdio = 'inherit',
  checkRuntime = checkPythonRuntime,
  spawnSync = nodeSpawnSync,
}) {
  const runtime = checkRuntime();
  if (!runtime.ok) {
    console.error(`[qiongli] ${runtime.message}`);
    console.error(`Hint: ${runtime.hint}`);
    return 1;
  }

  const childEnv = buildPythonRuntimeEnv({ packageRoot, env });
  const result = spawnSync(runtime.python, ['-m', 'bridges.orchestrator', command, ...args], {
    cwd,
    env: childEnv,
    stdio,
  });
  return typeof result.status === 'number' ? result.status : 1;
}

export function runPythonCliCommand({
  packageRoot,
  args,
  cwd = process.cwd(),
  env = process.env,
  stdio = 'inherit',
  checkRuntime = checkPythonRuntime,
  spawnSync = nodeSpawnSync,
}) {
  const runtime = checkRuntime();
  if (!runtime.ok) {
    console.error(`[qiongli] ${runtime.message}`);
    console.error(`Hint: ${runtime.hint}`);
    return 1;
  }

  const childEnv = buildPythonRuntimeEnv({ packageRoot, env });
  const result = spawnSync(runtime.python, ['-m', 'qiongli.cli', ...args], {
    cwd,
    env: childEnv,
    stdio,
  });
  return typeof result.status === 'number' ? result.status : 1;
}

function buildPythonRuntimeEnv({ packageRoot, env }) {
  const pythonPath = path.join(packageRoot, 'python-runtime');
  const childEnv = {
    ...env,
    PYTHONPATH: env.PYTHONPATH ? `${pythonPath}${path.delimiter}${env.PYTHONPATH}` : pythonPath,
  };
  delete childEnv.QIONGLI_NPM_PACKAGE_VERSION;
  const npmPackageVersion = readNpmPackageVersion(packageRoot);
  if (npmPackageVersion) {
    childEnv.QIONGLI_NPM_PACKAGE_VERSION = npmPackageVersion;
  }
  return childEnv;
}

function readNpmPackageVersion(packageRoot) {
  try {
    const raw = readFileSync(path.join(packageRoot, 'package.json'), 'utf-8');
    const packageJson = JSON.parse(raw);
    const version = typeof packageJson.version === 'string' ? packageJson.version.trim() : '';
    return isNpmSemverLikeVersion(version) ? version : '';
  } catch {
    return '';
  }
}

function isNpmSemverLikeVersion(version) {
  return /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?(?:\+[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?$/.test(version);
}
