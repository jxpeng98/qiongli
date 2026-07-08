const TARGETS = new Set(['codex', 'claude', 'antigravity', 'hermes', 'all', 'auto']);
const MODES = new Set(['copy', 'link']);
const SURFACES = new Set(['skills', 'plugin', 'both']);
const REFRESH_COMMANDS = new Set(['upgrade', 'refresh', 'update']);

export function parseArgv(argv) {
  const [rawCommand = 'help', ...restArgs] = argv;

  let command = rawCommand;
  if (REFRESH_COMMANDS.has(rawCommand)) {
    command = 'install';
  } else if (rawCommand === 'uninstall' || rawCommand === 'delete') {
    command = 'remove';
  }
  const options = {
    target: 'all',
    mode: 'copy',
    projectDir: '.',
    surface: 'skills',
    overwrite: REFRESH_COMMANDS.has(rawCommand),
    dryRun: false,
    json: false,
    globals: false,
    subject: 'core',
    coverage: 'complete',
    parts: '',
    projectCommand: '',
    projectSubject: '',
  };
  const rest = [];
  let i = 0;

  if (command === 'project' && restArgs[0] && !restArgs[0].startsWith('--')) {
    options.projectCommand = restArgs[0];
    i = 1;
    if (
      options.projectCommand === 'set-subject'
      && restArgs[i]
      && !restArgs[i].startsWith('--')
    ) {
      options.projectSubject = restArgs[i];
      i += 1;
    }
  }

  for (; i < restArgs.length; i += 1) {
    const arg = restArgs[i];
    if (arg === '--target') {
      options.target = requireValue(restArgs, i, arg);
      i += 1;
    } else if (arg === '--mode') {
      options.mode = requireValue(restArgs, i, arg);
      i += 1;
    } else if (arg === '--surface') {
      options.surface = requireValue(restArgs, i, arg);
      i += 1;
    } else if (arg === '--project-dir') {
      options.projectDir = requireValue(restArgs, i, arg);
      i += 1;
    } else if (arg === '--subject') {
      const value = requireValue(restArgs, i, arg);
      if (command === 'project' && options.projectCommand === 'set-subject') {
        options.projectSubject = value;
      } else {
        options.subject = value;
      }
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
  if (options.surface && !SURFACES.has(options.surface)) {
    throw new Error(`Unsupported surface: ${options.surface}`);
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
