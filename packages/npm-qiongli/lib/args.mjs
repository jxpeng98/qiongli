const TARGETS = new Set(['codex', 'claude', 'antigravity', 'hermes', 'all']);
const MODES = new Set(['copy', 'link']);
const BRIDGE_COMMANDS = new Set(['doctor', 'guidance', 'task-run', 'team-run', 'parallel', 'chain', 'role', 'single', 'code-build', 'task-plan']);
const PYTHON_CLI_COMMANDS = new Set(['setup', 'mcp', 'self-update', 'update']);

export function parseArgv(argv) {
  const [rawCommand = 'help', ...restArgs] = argv;
  if (BRIDGE_COMMANDS.has(rawCommand) || PYTHON_CLI_COMMANDS.has(rawCommand)) {
    return { command: rawCommand, options: {}, rest: restArgs };
  }

  let command = rawCommand;
  if (rawCommand === 'upgrade') {
    command = 'install';
  } else if (rawCommand === 'uninstall' || rawCommand === 'delete') {
    command = 'remove';
  }
  const options = {
    target: 'all',
    mode: 'copy',
    projectDir: '.',
    overwrite: rawCommand === 'upgrade',
    dryRun: false,
    json: false,
    globals: false,
    subject: 'core',
    coverage: 'complete',
    parts: '',
  };
  const rest = [];

  for (let i = 0; i < restArgs.length; i += 1) {
    const arg = restArgs[i];
    if (arg === '--target') {
      options.target = requireValue(restArgs, i, arg);
      i += 1;
    } else if (arg === '--mode') {
      options.mode = requireValue(restArgs, i, arg);
      i += 1;
    } else if (arg === '--project-dir') {
      options.projectDir = requireValue(restArgs, i, arg);
      i += 1;
    } else if (arg === '--subject') {
      options.subject = requireValue(restArgs, i, arg);
      i += 1;
    } else if (arg === '--coverage') {
      options.coverage = requireValue(restArgs, i, arg);
      i += 1;
    } else if (arg === '--cwd') {
      options.cwd = requireValue(restArgs, i, arg);
      i += 1;
    } else if (arg === '--parts') {
      options.parts = requireValue(restArgs, i, arg);
      i += 1;
    } else if (arg === '--overwrite') {
      options.overwrite = true;
    } else if (arg === '--dry-run') {
      options.dryRun = true;
    } else if (arg === '--json') {
      options.json = true;
    } else if (arg === '--globals') {
      options.globals = true;
    } else if (arg === '-h' || arg === '--help') {
      options.help = true;
    } else {
      rest.push(arg);
    }
  }

  if (options.target && !TARGETS.has(options.target)) {
    throw new Error(`Unsupported target: ${options.target}`);
  }
  if (options.mode && !MODES.has(options.mode)) {
    throw new Error(`Unsupported mode: ${options.mode}`);
  }
  return { command, options, rest };
}

function requireValue(args, index, flag) {
  const value = args[index + 1];
  if (!value || value.startsWith('--')) {
    throw new Error(`Missing value for ${flag}`);
  }
  return value;
}
