import { parseArgv } from './args.mjs';
import { packageRoot } from './package-root.mjs';
import { buildCheck, cleanAssets, installSkills, removeAssets } from './installer.mjs';
import { initProject, projectStatus, renderProjectResult, setProjectSubject } from './project.mjs';
import {
  FULL_RUNTIME_COMMANDS,
  hasLegacyUpdateFlags,
  writeFullRuntimeRequired,
  writeLegacyUpdateFlagGuidance,
} from './runtime-message.mjs';

const INSTALL_COMMANDS = new Set(['install', 'setup']);

export async function main(argv, {
  stdout = process.stdout,
  stderr = process.stderr,
  env = process.env,
  packageRoot: packageRootOverride,
} = {}) {
  const root = packageRootOverride || packageRoot();
  const [rawCommand = 'help'] = argv;
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

  if (rawCommand === 'update' && hasLegacyUpdateFlags(parsed.rest)) {
    writeLegacyUpdateFlagGuidance(stderr);
    return 1;
  }

  if (INSTALL_COMMANDS.has(parsed.command)) {
    if (parsed.rest.length) {
      writeUnsupportedAssetArguments(parsed.rest, stderr);
      return 2;
    }
    let result;
    try {
      result = installSkills({
        packageRoot: root,
        target: parsed.options.target,
        mode: parsed.options.mode,
        surface: parsed.options.surface,
        overwrite: parsed.options.overwrite,
        dryRun: parsed.options.dryRun,
        subject: parsed.options.subject,
        coverage: parsed.options.coverage,
        parts: parsed.options.parts,
        env,
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
        surface: parsed.options.surface,
        parts: parsed.options.parts,
        dryRun: parsed.options.dryRun,
        env,
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
        ...buildCheck({ packageRoot: root, subject: parsed.options.subject, coverage: parsed.options.coverage, env }),
        python_bridge: pythonBridgeStatus(),
        npm_cli: npmCliStatus(),
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
      stdout.write(`Npm CLI: ${payload.npm_cli.role} (${payload.npm_cli.python_free ? 'Python-free' : 'Python required'})\n`);
      stdout.write(`Full runtime: ${payload.npm_cli.full_runtime_install}\n`);
    }
    return 0;
  }

  if (parsed.command === 'clean') {
    const result = cleanAssets({
      projectDir: parsed.options.projectDir,
      globals: parsed.options.globals,
      dryRun: parsed.options.dryRun,
      env,
    });
    stdout.write(`[qiongli] removed ${result.removed.length} stale asset(s)\n`);
    return 0;
  }

  if (parsed.command === 'runtime') {
    if (parsed.rest[0] !== 'doctor') {
      stderr.write('[qiongli] runtime supports only: doctor\n');
      return 2;
    }
    stdout.write('[ok] Qiongli npm asset manager is installed.\n');
    stdout.write('Full runtime: pipx install qiongli\n');
    return 0;
  }

  if (parsed.command === 'project') {
    let result;
    try {
      if (parsed.options.projectCommand === 'init') {
        result = initProject({
          projectDir: parsed.options.projectDir,
          dryRun: parsed.options.dryRun,
        });
      } else if (parsed.options.projectCommand === 'status') {
        result = {
          action: 'status',
          changed: false,
          dry_run: false,
          state: projectStatus({ projectDir: parsed.options.projectDir }),
          actions: [],
        };
      } else if (parsed.options.projectCommand === 'set-subject') {
        if (!parsed.options.projectSubject) {
          stderr.write('[qiongli] project set-subject requires a subject\n');
          return 2;
        }
        result = setProjectSubject({
          projectDir: parsed.options.projectDir,
          subject: parsed.options.projectSubject,
          dryRun: parsed.options.dryRun,
        });
      } else {
        stderr.write('[qiongli] project supports: init, status, set-subject\n');
        return 2;
      }
    } catch (error) {
      stderr.write(`[qiongli] ${error.message}\n`);
      return 2;
    }
    stdout.write(renderProjectResult(result, { json: parsed.options.json }));
    return 0;
  }

  if (FULL_RUNTIME_COMMANDS.has(parsed.command)) {
    writeFullRuntimeRequired(parsed.command, stderr);
    return 1;
  }

  stderr.write(`[qiongli] unknown command: ${parsed.command}\n`);
  return 2;
}

function printInstallResult(result, stdout) {
  stdout.write('Qiongli npm asset manager\n');
  stdout.write(`source version: ${result.sourceVersion || '<unknown>'}\n`);
  stdout.write(`source subject: ${subjectLabel(result.sourceSubject)}\n`);
  stdout.write(`source coverage: ${result.sourceCoverage || '<unknown>'}\n`);
  for (const residue of result.legacyResidues) {
    stdout.write(`[legacy:${residue.status}] ${residue.target}: ${residue.legacyName} -> ${residue.path}\n`);
  }
  for (const action of result.actions) {
    stdout.write(`[${action.status}] ${action.label} -> ${action.path} (${action.detail})\n`);
  }
  stdout.write('Restart Codex / Claude Code / Antigravity / Hermes to activate changes.\n');
}

function printRemoveResult(result, stdout) {
  stdout.write('Qiongli npm remover\n');
  for (const action of result.actions) {
    stdout.write(`[${action.status}] ${action.label} -> ${action.path} (${action.detail})\n`);
  }
  stdout.write('Restart Codex / Claude Code / Antigravity / Hermes to refresh discovery state.\n');
}

function helpText() {
  return `Qiongli npm asset manager

Usage:
  qiongli install --subject core --target all [--surface skills]
  qiongli setup --target codex [--surface skills] [--dry-run]
  qiongli update --target all [--dry-run]
  qiongli refresh --target all [--dry-run]
  qiongli upgrade --subject economics --coverage complete --target all
  qiongli remove [--target all]
  qiongli uninstall [--target codex]
  qiongli check [--json]
  qiongli clean --project-dir . [--globals]
  qiongli project init --project-dir . [--dry-run] [--json]
  qiongli project status --project-dir . [--json]
  qiongli project set-subject <subject> --project-dir . [--dry-run] [--json]
  qiongli runtime doctor

Full runtime commands require \`pipx install qiongli\`:
  doctor|task-run|team-run|parallel|chain|role|single|code-build|task-plan|mcp|provider|guidance|customize|init|align

Options:
  --target codex|claude|antigravity|hermes|all
  --surface skills|plugin|both
  --subject core|economics|accounting|business|finance|political-economy|geoeconomics|economics-accounting
            Default core installs adaptive runtime subject refinement.
            Non-core subjects are advanced overrides for pre-materialized packages.
  --coverage complete|focused
  --mode copy|link
  --parts globals,project,cli,mcp
  --overwrite
  --dry-run
`;
}

function subjectLabel(subject) {
  if (!subject) {
    return '<unknown>';
  }
  return subject === 'core'
    ? 'core (adaptive; active_subject defaults to auto)'
    : `${subject} (advanced override)`;
}

function npmCliStatus() {
  return {
    role: 'asset-manager',
    python_free: true,
    full_runtime_install: 'pipx install qiongli',
  };
}

function pythonBridgeStatus() {
  return {
    deprecated: true,
    bundled: false,
    managed_by_npm: false,
    message: 'Not bundled or managed by this npm asset-manager path.',
  };
}

function writeUnsupportedAssetArguments(args, stderr) {
  stderr.write(`[qiongli] unsupported npm asset-manager argument: ${args[0]}\n`);
  stderr.write('[qiongli] Full runtime setup/update/archive options require: pipx install qiongli\n');
}
