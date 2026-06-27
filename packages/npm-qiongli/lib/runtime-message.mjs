export const FULL_RUNTIME_COMMANDS = new Set([
  'doctor',
  'task-run',
  'team-run',
  'parallel',
  'chain',
  'role',
  'single',
  'code-build',
  'task-plan',
  'mcp',
  'provider',
  'guidance',
  'customize',
  'init',
  'align',
  'self-update',
]);

const LEGACY_UPDATE_FLAGS = new Set(['--yes', '--no-refresh', '--channel', '--skip-check']);

export function writeFullRuntimeRequired(command, stderr = process.stderr) {
  const label = command ? `Command '${command}'` : 'This command';
  stderr.write(`[qiongli] ${label} requires Qiongli full runtime.\n`);
  stderr.write('[qiongli] Install the full runtime with: pipx install qiongli\n');
  stderr.write('[qiongli] The npm package is a Python-free asset manager for installed Qiongli assets.\n');
}

export function hasLegacyUpdateFlags(args = []) {
  return args.some((arg) => {
    if (LEGACY_UPDATE_FLAGS.has(arg)) {
      return true;
    }
    const [flag] = String(arg).split('=', 1);
    return LEGACY_UPDATE_FLAGS.has(flag);
  });
}

export function writeLegacyUpdateFlagGuidance(stderr = process.stderr) {
  stderr.write('[qiongli] legacy update flags are not supported by the npm asset manager.\n');
  stderr.write('[qiongli] Use `qiongli install`, `qiongli refresh`, or `qiongli upgrade` for npm asset refreshes.\n');
  stderr.write('[qiongli] Use `qiongli self-update` from the full runtime after `pipx install qiongli`.\n');
}
