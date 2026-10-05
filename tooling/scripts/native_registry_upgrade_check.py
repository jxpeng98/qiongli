#!/usr/bin/env python3
"""Upgrade disposable published pip/npm installs to the checked candidate."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

try:
    from .native_registry_install_check import check_cli, run
    from .native_registry_packages import host_target, npm_command, parse_release_version
except ImportError:
    from native_registry_install_check import check_cli, run
    from native_registry_packages import host_target, npm_command, parse_release_version


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def predecessor_versions(target: str | None, version: str | None) -> dict[str, str]:
    if version is not None:
        identity = parse_release_version(version)
        return {'pypi': identity.package_version, 'npm': identity.npm_version}
    # Preserve the historical Linux ARM64 upgrade: 2.1.1 has no wheel for it.
    return {'pypi': '1.17.0' if target == 'aarch64-unknown-linux-gnu' else '2.1.1',
            'npm': '2.1.1'}


class PredecessorUnavailable(RuntimeError):
    def __init__(self, ecosystem: str):
        super().__init__('selected predecessor could not be downloaded')
        self.ecosystem = ecosystem


def download_predecessor(argv, *, ecosystem, root, env):
    try:
        return run(argv, root=root, env=env)
    except (subprocess.CalledProcessError, subprocess.TimeoutExpired) as error:
        # Do not substitute a historical version or expose process environments.
        raise PredecessorUnavailable(ecosystem) from error


def upgrade(packages: Path, out_dir: Path, predecessor_version: str | None = None) -> dict:
    target = host_target()
    versions = predecessor_versions(target, predecessor_version)
    packages = packages.resolve()
    receipt = json.loads((packages / 'registry-packages.json').read_text())
    artifacts = {}
    for artifact in receipt['artifacts']:
        path = packages / artifact['file']
        if (path.parent != packages or path.is_symlink() or not path.is_file()
                or sha256(path) != artifact['sha256']):
            raise ValueError('candidate package integrity mismatch')
        if path.suffix in ('.whl', '.tgz') and path.suffix in artifacts:
            raise ValueError('ambiguous candidate package selection')
        artifacts[path.suffix] = path
    if '.whl' not in artifacts or '.tgz' not in artifacts:
        raise ValueError('upgrade check requires target-selected wheel and npm package')
    root = out_dir.resolve()
    root.mkdir(parents=True, exist_ok=False)
    home = root / 'home'
    home.mkdir(mode=0o700)
    env = os.environ.copy()
    env.update(HOME=str(home), USERPROFILE=str(home), QIONGLI_CONFIG_HOME=str(home / 'config'),
               XDG_CONFIG_HOME=str(home / 'config'), PIP_DISABLE_PIP_VERSION_CHECK='1')
    if os.name == 'nt':
        env.update(APPDATA=str(home / 'AppData/Roaming'), LOCALAPPDATA=str(home / 'AppData/Local'))
    for key in ('CODEX_HOME', 'CLAUDE_CONFIG_DIR', 'PYTHONPATH', 'PYTHONHOME'):
        env.pop(key, None)
    canaries = {
        'research/paper-notes.md': b'# Prior reviewed notes\nKeep these exact bytes.\n',
        'research/source-packet.json': b'{"source":"supplied-fixture","reviewed":true}\n',
        'config/host-model-settings.json': b'{"model":"user-selected-fixture","profile":"keep"}\n',
    }
    for relative, data in canaries.items():
        path = home / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)

    result = {'status': 'unqualified', 'version': receipt['version'], 'target': target,
              'scope': 'package replacement and fixture byte retention; not project-format migration or live Host acceptance',
              'predecessor_selection': 'explicit' if predecessor_version is not None else 'historical-default',
              'selected_predecessors': versions, 'predecessors': [],
              'candidate_packages': receipt['artifacts'], 'retained': {}, 'checks': {}}
    try:
        check_upgrade(root, home, env, artifacts, receipt['version'], versions, canaries, result)
        result['status'] = 'passed'
    except PredecessorUnavailable as error:
        result.update(reason='predecessor-unavailable', ecosystem=error.ecosystem)
    except (ValueError, OSError, AssertionError, subprocess.SubprocessError) as error:
        result.update(status='failed', reason='upgrade-check-failed', error_type=type(error).__name__)
    (root / 'registry-upgrade-check.json').write_text(json.dumps(result, indent=2) + '\n')
    return result


def check_upgrade(root, home, env, artifacts, version, versions, canaries, result):

    python_root = root / 'python'
    run([sys.executable, '-m', 'venv', python_root], root=root, env=env)
    python_bin = python_root / ('Scripts' if os.name == 'nt' else 'bin')
    suffix = '.exe' if os.name == 'nt' else ''
    python = python_bin / ('python' + suffix)
    downloads = root / 'predecessors'
    downloads.mkdir()
    pip_version = versions['pypi']
    download_predecessor([python, '-m', 'pip', '--isolated', 'download', '--no-deps', '--only-binary=:all:',
         '--index-url', 'https://pypi.org/simple', '--dest', downloads,
         f'qiongli=={pip_version}'], ecosystem='pypi', root=root, env=env)
    old_wheel, = downloads.glob('*.whl')
    result['predecessors'].append({'ecosystem': 'pypi', 'version': pip_version,
                                  'file': old_wheel.name, 'sha256': sha256(old_wheel)})
    run([python, '-m', 'pip', '--isolated', 'install', '--no-index', '--no-deps', old_wheel],
        root=root, env=env)
    observed = run([python, '-c', 'import importlib.metadata; print(importlib.metadata.version("qiongli"))'],
                   root=root, env=env).stdout.strip()
    if observed != pip_version:
        raise ValueError('wrong pip predecessor installed')
    run([python, '-m', 'pip', '--isolated', 'install', '--upgrade', '--no-index', '--no-deps',
         artifacts['.whl']], root=root, env=env)
    result['checks']['pypi'] = check_cli(python_bin / ('qiongli' + suffix), version=version, root=root, env=env)
    run([python_bin / ('ql' + suffix), '--version'], root=root, env=env)

    npm_version = versions['npm']
    download_predecessor(npm_command('pack', f'qiongli@{npm_version}', '--ignore-scripts', '--pack-destination', downloads,
                    '--registry', 'https://registry.npmjs.org', '--cache', root / 'npm-cache'),
        ecosystem='npm', root=root, env=env)
    old_npm, = downloads.glob('*.tgz')
    result['predecessors'].append({'ecosystem': 'npm', 'version': npm_version,
                                  'file': old_npm.name, 'sha256': sha256(old_npm)})
    prefix = root / 'npm'
    for package in (old_npm, artifacts['.tgz']):
        run(npm_command('install', '--global', '--prefix', prefix, '--cache', root / 'npm-cache',
                        '--ignore-scripts', '--no-audit', '--no-fund', '--offline', package), root=root, env=env)
        if package == old_npm:
            installed = prefix / ('node_modules/qiongli' if os.name == 'nt' else 'lib/node_modules/qiongli')
            if json.loads((installed / 'package.json').read_text())['version'] != npm_version:
                raise ValueError('wrong npm predecessor installed')
    node = subprocess.check_output(['node', '-p', 'process.execPath'], text=True).strip()
    command = ([node, prefix / 'node_modules/qiongli/bin/qiongli.mjs'] if os.name == 'nt'
               else prefix / 'bin/qiongli')
    result['checks']['npm'] = check_cli(command, version=version, root=root, env=env)
    for relative, expected in canaries.items():
        if (home / relative).read_bytes() != expected:
            raise ValueError(f'upgrade changed retained data: {relative}')
    result['retained'] = {relative: sha256(home / relative) for relative in canaries}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--packages', type=Path, required=True)
    parser.add_argument('--out-dir', type=Path, required=True)
    parser.add_argument('--predecessor-version', help='exact shared pip/npm predecessor; omit for historical platform defaults')
    args = parser.parse_args()
    result = upgrade(args.packages, args.out_dir, args.predecessor_version)
    print(json.dumps(result, indent=2))
    if result['status'] != 'passed':
        raise SystemExit(1)


if __name__ == '__main__':
    main()
