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
    from .native_registry_packages import host_target, npm_command
except ImportError:
    from native_registry_install_check import check_cli, run
    from native_registry_packages import host_target, npm_command


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--packages', type=Path, required=True)
    parser.add_argument('--out-dir', type=Path, required=True)
    args = parser.parse_args()
    packages = args.packages.resolve()
    receipt = json.loads((packages / 'registry-packages.json').read_text())
    artifacts = {}
    for artifact in receipt['artifacts']:
        path = packages / artifact['file']
        if path.parent != packages or path.is_symlink() or sha256(path) != artifact['sha256']:
            raise ValueError('candidate package integrity mismatch')
        artifacts[path.suffix] = path
    if '.whl' not in artifacts or '.tgz' not in artifacts:
        raise ValueError('upgrade check requires target-selected wheel and npm package')
    root = args.out_dir.resolve()
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

    python_root = root / 'python'
    run([sys.executable, '-m', 'venv', python_root], root=root, env=env)
    python_bin = python_root / ('Scripts' if os.name == 'nt' else 'bin')
    suffix = '.exe' if os.name == 'nt' else ''
    python = python_bin / ('python' + suffix)
    downloads = root / 'predecessors'
    downloads.mkdir()
    # Published 2.1.1 has no ARM64 Linux wheel: reproduce that user's 1.x upgrade.
    pip_version = '1.17.0' if host_target() == 'aarch64-unknown-linux-gnu' else '2.1.1'
    run([python, '-m', 'pip', '--isolated', 'download', '--no-deps', '--only-binary=:all:',
         '--index-url', 'https://pypi.org/simple', '--dest', downloads,
         f'qiongli=={pip_version}'], root=root, env=env)
    old_wheel, = downloads.glob('*.whl')
    run([python, '-m', 'pip', '--isolated', 'install', '--no-index', '--no-deps', old_wheel],
        root=root, env=env)
    observed = run([python, '-c', 'import importlib.metadata; print(importlib.metadata.version("qiongli"))'],
                   root=root, env=env).stdout.strip()
    if observed != pip_version:
        raise ValueError('wrong pip predecessor installed')
    run([python, '-m', 'pip', '--isolated', 'install', '--upgrade', '--no-index', '--no-deps',
         artifacts['.whl']], root=root, env=env)
    pip_check = check_cli(python_bin / ('qiongli' + suffix), version=receipt['version'], root=root, env=env)
    run([python_bin / ('ql' + suffix), '--version'], root=root, env=env)

    run(npm_command('pack', 'qiongli@2.1.1', '--ignore-scripts', '--pack-destination', downloads,
                    '--registry', 'https://registry.npmjs.org', '--cache', root / 'npm-cache'),
        root=root, env=env)
    old_npm, = downloads.glob('*.tgz')
    prefix = root / 'npm'
    for package in (old_npm, artifacts['.tgz']):
        run(npm_command('install', '--global', '--prefix', prefix, '--cache', root / 'npm-cache',
                        '--ignore-scripts', '--no-audit', '--no-fund', '--offline', package), root=root, env=env)
        if package == old_npm:
            installed = prefix / ('node_modules/qiongli' if os.name == 'nt' else 'lib/node_modules/qiongli')
            if json.loads((installed / 'package.json').read_text())['version'] != '2.1.1':
                raise ValueError('wrong npm predecessor installed')
    node = subprocess.check_output(['node', '-p', 'process.execPath'], text=True).strip()
    command = ([node, prefix / 'node_modules/qiongli/bin/qiongli.mjs'] if os.name == 'nt'
               else prefix / 'bin/qiongli')
    npm_check = check_cli(command, version=receipt['version'], root=root, env=env)
    for relative, expected in canaries.items():
        if (home / relative).read_bytes() != expected:
            raise ValueError(f'upgrade changed retained data: {relative}')
    result = {'status': 'passed', 'version': receipt['version'],
              'scope': 'package replacement and fixture byte retention; not project-format migration or live Host acceptance',
              'predecessors': [{'file': path.name, 'sha256': sha256(path)} for path in (old_wheel, old_npm)],
              'candidate_packages': receipt['artifacts'],
              'retained': {relative: sha256(home / relative) for relative in canaries},
              'checks': {'pypi': pip_check, 'npm': npm_check}}
    (root / 'registry-upgrade-check.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(result, indent=2))


if __name__ == '__main__':
    main()
