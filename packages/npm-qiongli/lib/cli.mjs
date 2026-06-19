import { parseArgv } from './args.mjs';
import { packageRoot } from './package-root.mjs';
import { buildCheck, cleanAssets, installSkills, removeAssets } from './installer.mjs';
import {
  checkPythonRuntime,
  runBridgeCommand as defaultRunBridgeCommand,
  runPythonCliCommand as defaultRunPythonCliCommand,
} from './python-runtime.mjs';

const BRIDGE_COMMANDS = new Set(['doctor', 'guidance', 'task-run', 'team-run', 'parallel', 'chain', 'role', 'single', 'code-build', 'task-plan']);
const PYTHON_CLI_COMMANDS = new Set(['setup', 'mcp']);

export async function main(argv, {
  stdout = process.stdout,
  stderr = process.stderr,
  runBridgeCommand = defaultRunBridgeCommand,
  runPythonCliCommand = defaultRunPythonCliCommand,
} = {}) {
  const root = packageRoot();
  let parsed;
  try {
    parsed = parseArgv(argv);
  } catch (error) {
    stderr.write(`[qiongli] ${error.message}\n`);
    return 2;
  }

  if (parsed.options.help || parsed.command === 'help') {
    stdout.write(helpText());
    return 0;
  }

  if (parsed.command === 'install') {
    let result;
    try {
      result = installSkills({
        packageRoot: root,
        target: parsed.options.target,
        mode: parsed.options.mode,
        overwrite: parsed.options.overwrite,
        dryRun: parsed.options.dryRun,
        subject: parsed.options.subject,
        coverage: parsed.options.coverage,
      });
    } catch (error) {
      stderr.write(`[qiongli] ${error.message}\n`);
      return 2;
    }
    printInstallResult(result, stdout);
    return 0;
  }

  if (parsed.command === 'remove') {
    let result;
    try {
      result = removeAssets({
        target: parsed.options.target,
        projectDir: parsed.options.projectDir,
        parts: parsed.options.parts,
        dryRun: parsed.options.dryRun,
      });
    } catch (error) {
      stderr.write(`[qiongli] ${error.message}\n`);
      return 2;
    }
    printRemoveResult(result, stdout);
    return 0;
  }

  if (parsed.command === 'check') {
    let payload;
    try {
      payload = {
        ...buildCheck({ packageRoot: root, subject: parsed.options.subject, coverage: parsed.options.coverage }),
        python_bridge: checkPythonRuntime(),
      };
    } catch (error) {
      stderr.write(`[qiongli] ${error.message}\n`);
      return 2;
    }
    if (parsed.options.json) {
      stdout.write(`${JSON.stringify(payload, null, 2)}\n`);
    } else {
      stdout.write(`Qiongli npm package: ${payload.npm_package.version}\n`);
      stdout.write(`Payload version: ${payload.payload.version || '<unknown>'}\n`);
      stdout.write(`Payload subject: ${payload.payload.subject || '<unknown>'}\n`);
      stdout.write(`Payload coverage: ${payload.payload.coverage || '<unknown>'}\n`);
      stdout.write(`Python bridge: ${payload.python_bridge.ok ? 'ok' : 'warn'} - ${payload.python_bridge.message}\n`);
    }
    return 0;
  }

  if (parsed.command === 'clean') {
    const result = cleanAssets({
      projectDir: parsed.options.projectDir,
      globals: parsed.options.globals,
      dryRun: parsed.options.dryRun,
    });
    stdout.write(`[qiongli] removed ${result.removed.length} stale asset(s)\n`);
    return 0;
  }

  if (parsed.command === 'runtime') {
    if (parsed.rest[0] !== 'doctor') {
      stderr.write('[qiongli] runtime supports only: doctor\n');
      return 2;
    }
    const result = checkPythonRuntime();
    stdout.write(`${result.ok ? '[ok]' : '[warn]'} ${result.message}\n`);
    if (result.hint) {
      stdout.write(`Hint: ${result.hint}\n`);
    }
    return result.ok ? 0 : 1;
  }

  if (PYTHON_CLI_COMMANDS.has(parsed.command)) {
    return runPythonCliCommand({ packageRoot: root, args: [parsed.command, ...parsed.rest] });
  }

  if (BRIDGE_COMMANDS.has(parsed.command)) {
    const args = parsed.command === 'doctor' && !parsed.rest.includes('--cwd')
      ? ['--cwd', parsed.options.cwd || '.', ...parsed.rest]
      : parsed.rest;
    return runBridgeCommand({ packageRoot: root, command: parsed.command, args });
  }

  stderr.write(`[qiongli] unknown command: ${parsed.command}\n`);
  return 2;
}

function printInstallResult(result, stdout) {
  stdout.write('Qiongli npm installer\n');
  stdout.write(`source version: ${result.sourceVersion || '<unknown>'}\n`);
  stdout.write(`source subject: ${result.sourceSubject || '<unknown>'}\n`);
  stdout.write(`source coverage: ${result.sourceCoverage || '<unknown>'}\n`);
  for (const residue of result.legacyResidues) {
    stdout.write(`[legacy:${residue.status}] ${residue.target}: ${residue.legacyName} -> ${residue.path}\n`);
  }
  for (const action of result.actions) {
    stdout.write(`[${action.status}] ${action.label} -> ${action.path} (${action.detail})\n`);
  }
  stdout.write('Restart Codex / Claude Code / Gemini CLI / Antigravity / Hermes to activate changes.\n');
}

function printRemoveResult(result, stdout) {
  stdout.write('Qiongli npm remover\n');
  for (const action of result.actions) {
    stdout.write(`[${action.status}] ${action.label} -> ${action.path} (${action.detail})\n`);
  }
  stdout.write('Restart Codex / Claude Code / Gemini CLI / Antigravity / Hermes to refresh discovery state.\n');
}

function helpText() {
  return `Qiongli npm installer

Usage:
  qiongli install --subject core --target all
  qiongli upgrade --subject economics --coverage complete --target all
  qiongli remove [--target all]
  qiongli uninstall [--target codex]
  qiongli check [--json]
  qiongli clean --project-dir . [--globals]
  qiongli runtime doctor
  qiongli setup [--dry-run] [--project-dir .] [--no-doctor]
  qiongli mcp serve --transport stdio
  qiongli mcp doctor --json
  qiongli mcp upgrade --target all [--dry-run]
  qiongli doctor --cwd .
  qiongli guidance init [--project-dir .]
  qiongli guidance show|trace [--project-dir .]
  qiongli guidance apply --proposal .qiongli/trace/runs/<run_id>/guidance_update_proposal.md
  qiongli task-run ...
  qiongli team-run ...

Options:
  --target codex|claude|gemini|antigravity|hermes|all
  --subject core|economics|accounting|business|finance|economics-accounting
  --coverage complete|focused
  --mode copy|link
  --overwrite
  --dry-run
  --no-doctor
`;
}
