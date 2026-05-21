import { spawnSync as nodeSpawnSync } from 'node:child_process';
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

export function runBridgeCommand({ packageRoot, command, args, cwd = process.cwd(), env = process.env, stdio = 'inherit' }) {
  const runtime = checkPythonRuntime();
  if (!runtime.ok) {
    console.error(`[qiongli] ${runtime.message}`);
    console.error(`Hint: ${runtime.hint}`);
    return 1;
  }

  const pythonPath = path.join(packageRoot, 'python-runtime');
  const childEnv = {
    ...env,
    PYTHONPATH: env.PYTHONPATH ? `${pythonPath}${path.delimiter}${env.PYTHONPATH}` : pythonPath,
  };
  const result = nodeSpawnSync(runtime.python, ['-m', 'bridges.orchestrator', command, ...args], {
    cwd,
    env: childEnv,
    stdio,
  });
  return typeof result.status === 'number' ? result.status : 1;
}
