#!/usr/bin/env python3
"""Assemble or verify one three-platform CLI release packet; never publish."""
import argparse
import hashlib
import json
import platform
import os
import subprocess
import sys
import tempfile
import re
from pathlib import Path
import shutil
import tarfile
import zipfile

try:
    from .native_registry_install_check import require_transition
    from .native_registry_packages import ROOT, NATIVE, NPM_INSTALL_REVIEW, TARGETS, npm_package, parse_release_version, regular_bytes, validate_binary
    from .native_marketplace_plugins import PLATFORMS, archive_name, verify_archive, check_plugins, find_archive
except ImportError:
    from native_registry_install_check import require_transition
    from native_registry_packages import ROOT, NATIVE, NPM_INSTALL_REVIEW, TARGETS, npm_package, parse_release_version, regular_bytes, validate_binary
    from native_marketplace_plugins import PLATFORMS, archive_name, verify_archive, check_plugins, find_archive


def checked_assets(root, manifest):
    assets = {}
    for item in manifest['artifacts']:
        name = item['file']
        if Path(name).name != name or '/' in name or '\\' in name or name in {'.', '..', ''} or name in assets:
            raise ValueError('unsafe or duplicate artifact name')
        path = root / name
        data = regular_bytes(path)
        if hashlib.sha256(data).hexdigest() != item['sha256'] or len(data) != item['bytes']:
            raise ValueError(f'artifact digest/size mismatch: {name}')
        assets[name] = path
    return assets


def check_identity(manifest, version, commit):
    if manifest['version'] != version or manifest['source_commit'] != commit:
        raise ValueError('mixed version or source commit')


def cli_binary(archive, target):
    name = TARGETS[target][2]
    if target.endswith('msvc'):
        with zipfile.ZipFile(archive) as packet:
            data = packet.read(name)
    else:
        with tarfile.open(archive) as packet:
            member = packet.getmember(name)
            if not member.isfile():
                raise ValueError('archive executable is not regular')
            data = packet.extractfile(member).read()
    validate_binary(data, target)
    return data


def assemble(root, out, version, commit):
    out.mkdir(parents=True, exist_ok=False)
    binaries, receipts = {}, []
    for target in TARGETS:
        source = root / target
        receipt = json.loads(regular_bytes(source / 'release-manifest.json'))
        check_identity(receipt, version, commit)
        checks = receipt['checks']
        if (receipt['target'] != target or checks['cli_mcp_tests'] != 'passed'
                or checks['npm_wheel_local_install'] != 'passed'
                or checks['archive_smoke']['version'] != version):
            raise ValueError(f'missing target-native checks: {target}')
        if target.endswith('linux-gnu') and checks['cli_clippy'] != 'passed':
            raise ValueError('missing Linux Clippy gate')
        files = checked_assets(source, receipt)
        extension = 'zip' if target.endswith('msvc') else 'tar.gz'
        cli_name = f'qiongli-{version}-{target}.{extension}'
        archive = files[cli_name]
        binary_name = TARGETS[target][2]
        data = cli_binary(archive, target)
        binary = out.parent / 'npm-binaries' / target / binary_name
        binary.parent.mkdir(parents=True, exist_ok=True)
        binary.write_bytes(data)
        binary.chmod(0o755)
        binaries[target] = binary
        wheels = [p for name, p in files.items() if name.endswith('.whl')]
        if len(wheels) != 1:
            raise ValueError('expected one wheel per target')
        for path in [archive, *wheels]:
            shutil.copyfile(path, out / path.name)
        plugins = [find_archive(files, host, version, target) for host in PLATFORMS]
        if any(plugins):
            if not all(plugins):
                raise ValueError('both marketplace Plugin archives are required for each target')
            for path in plugins:
                provenance = verify_archive(path, version, commit)
                if provenance['binary_sha256'] != hashlib.sha256(data).hexdigest():
                    raise ValueError('marketplace executable differs from CLI executable')
                shutil.copyfile(path, out / path.name)
        # Historical alpha.8 packets retain their immutable npm-bridge pair.
        if target.endswith('linux-gnu'):
            legacy = [files.get(archive_name(host, version)) for host in PLATFORMS]
            if any(legacy):
                if not all(legacy):
                    raise ValueError('both marketplace Plugin archives are required')
                for path in legacy:
                    verify_archive(path, version, commit)
                    shutil.copyfile(path, out / path.name)
        receipts.append(receipt)
    npm_work = out.parent / 'npm-combined'
    npm_work.mkdir(exist_ok=False)
    packed = npm_package(npm_work, binaries, version)
    shutil.copyfile(packed, out / packed.name)
    output_files = {p.name: p for p in out.iterdir()}
    native_plugins = [find_archive(output_files, host, version, target) for target in TARGETS for host in PLATFORMS]
    if any(native_plugins):
        if not all(native_plugins):
            raise ValueError('all six target-specific marketplace archives are required')
        index = marketplace_index(version, commit, [verify_archive(p, version, commit) for p in native_plugins])
        (out / 'marketplace-plugins.json').write_text(json.dumps(index, indent=2) + '\n')
    manifest = {'version': version, 'source_commit': commit, 'targets': list(TARGETS),
                'status': 'three-platform-packaged', 'target_evidence': receipts,
                'artifacts': [{'file': p.name, 'sha256': hashlib.sha256(p.read_bytes()).hexdigest(),
                               'bytes': p.stat().st_size} for p in sorted(out.iterdir())]}
    (out / 'release-manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
    (out / 'SHA256SUMS').write_text(''.join(f'{hashlib.sha256(p.read_bytes()).hexdigest()}  {p.name}\n'
                                         for p in sorted(out.iterdir())))


def marketplace_index(version, commit, plugins):
    return {'schema_version': 1, 'version': version, 'source_commit': commit,
            'plugins': [{'name': p['plugin_name'], 'host': p['platform'],
                         'target': p['target'], 'artifact': p['artifact'],
                         'sha256': p['sha256'], 'binary_sha256': p['binary_sha256'],
                         'distribution_ref': f"{p['platform']}/{p['target']}/v{version}",
                         'plugin_path': 'plugins/' + p['plugin_name']} for p in plugins]}


def packet_digest(manifest):
    payload = {k: v for k, v in manifest.items() if k != 'local_macos'}
    return hashlib.sha256(json.dumps(payload, sort_keys=True).encode()).hexdigest()


def require_local_macos(manifest):
    observed = manifest.get('local_macos', {})
    if (observed.get('status') != 'passed' or observed.get('execution') != 'manual-local'
            or observed.get('target') != 'aarch64-apple-darwin'
            or observed.get('packet_sha256') != packet_digest(manifest)
            or observed.get('source_commit') != manifest['source_commit']
            or observed.get('version') != manifest['version']
            or observed.get('cargo_dry_run') != 'passed'):
        raise ValueError('missing or stale local macOS qualification')
    mac = next(r for r in manifest['target_evidence'] if r['target'] == 'aarch64-apple-darwin')
    pack = mac['checks']['archive_smoke']['content_pack_sha256']
    for name in ('npm', 'pypi', 'cargo_archives', 'cargo_alias'):
        check = observed.get('checks', {}).get(name, {})
        if (check.get('version') != manifest['version'] or check.get('content_pack_sha256') != pack
                or check.get('mcp_tools') != {'lite': 14, 'full': 32}
                or check.get('invalid_command_rejected') is not True):
            raise ValueError('incomplete local macOS package checks')
        if manifest['version'] == '2.0.1':
            require_transition(check.get('plugin_source_transition'))
    if observed.get('marketplace_plugins') != mac['checks']['marketplace_plugins']:
        raise ValueError('local macOS Plugin checks do not match qualified target')


def check_source(commit):
    def git(*args):
        return subprocess.check_output(['git', *args], cwd=ROOT, text=True).strip()
    if git('rev-parse', 'HEAD') != commit or git('status', '--porcelain'):
        raise ValueError('installation evidence requires the exact clean source')


def install_packet(root, out, version, commit, *, local_macos=False):
    target = {('Darwin', 'arm64'): 'aarch64-apple-darwin', ('Linux', 'x86_64'): 'x86_64-unknown-linux-gnu',
              ('Windows', 'AMD64'): 'x86_64-pc-windows-msvc'}[(platform.system(), platform.machine())]
    if local_macos and (target != 'aarch64-apple-darwin' or os.environ.get('GITHUB_ACTIONS') == 'true'):
        raise ValueError('macOS qualification must run manually on an Apple Silicon Mac')
    check_source(commit)
    manifest, npm, wheels = verify(root, version, commit)
    if not local_macos:
        require_local_macos(manifest)
    before = regular_bytes(root / 'release-manifest.json')
    if out.exists() or out.is_symlink() or ROOT in out.resolve().parents or out.resolve() == ROOT:
        raise ValueError('install output must be new and outside the checkout')
    out.mkdir(parents=True)
    checks = check_plugins(root, version, commit, target)
    selected = {npm.name, wheels[target].name}
    (root / 'registry-packages.json').write_text(json.dumps({
        'version': version, 'artifacts': [a for a in manifest['artifacts'] if a['file'] in selected]}))
    def run(*args):
        subprocess.run(list(map(str, args)), cwd=NATIVE, check=True)
    run(sys.executable, ROOT / 'scripts/native_registry_install_check.py', '--packages', root,
        '--out-dir', out / 'registry', '--version', version)
    installed = json.loads((out / 'registry/install-check.json').read_text())['checks']
    result = {'status': 'passed', 'version': version, 'source_commit': commit, 'target': target,
              'manifest_sha256': hashlib.sha256(before).hexdigest(), 'checks': installed,
              'marketplace_plugins': checks}
    if local_macos:
        run(sys.executable, ROOT / 'scripts/native_registry_packages.py', '--out-dir', out / 'cargo', '--package-cargo')
        run('cargo', 'publish', '--manifest-path', out / 'cargo/cargo-source/Cargo.toml',
            '--workspace', '--no-default-features', '--allow-dirty', '--locked', '--dry-run',
            '--target-dir', out / 'cargo-build')
        run(sys.executable, ROOT / 'scripts/native_registry_install_check.py', '--packages', out / 'cargo',
            '--out-dir', out / 'cargo-install', '--cargo-only', '--cargo-archives',
            '--cargo-target-dir', out / 'cargo-build', '--version', version)
        installed.update(json.loads((out / 'cargo-install/install-check.json').read_text())['checks'])
        result.update(execution='manual-local', packet_sha256=packet_digest(manifest), cargo_dry_run='passed')
    verify(root, version, commit)
    check_source(commit)
    if regular_bytes(root / 'release-manifest.json') != before:
        raise ValueError('packet changed during installation checks')
    if local_macos:
        manifest['local_macos'] = result
        require_local_macos(manifest)
        (root / 'release-manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
        names = ['release-manifest.json', *(a['file'] for a in manifest['artifacts'])]
        (root / 'SHA256SUMS').write_text(''.join(
            f'{hashlib.sha256(regular_bytes(root / name)).hexdigest()}  {name}\n' for name in sorted(names)))
    (out / 'install-check.json').write_text(json.dumps(result, indent=2) + '\n')


def require_remote_installs(root, manifest, commit):
    require_local_macos(manifest)
    repo = os.environ['GITHUB_REPOSITORY']
    runs = json.loads(subprocess.check_output(['gh', 'api',
        f'repos/{repo}/actions/workflows/native-cli-distribution.yml/runs?head_sha={commit}&status=success'], text=True))
    digest = hashlib.sha256(regular_bytes(root / 'release-manifest.json')).hexdigest()
    for run in runs['workflow_runs']:
        if (run['head_sha'] != commit or run['conclusion'] != 'success' or run['event'] != 'workflow_dispatch'
                or run['head_branch'] != 'v' + manifest['version']):
            continue
        with tempfile.TemporaryDirectory() as directory:
            downloaded = subprocess.run(['gh', 'run', 'download', str(run['id']), '--repo', repo,
                '--pattern', 'install-*', '--dir', directory], capture_output=True, text=True)
            if downloaded.returncode:
                continue
            expected = {'ubuntu-22.04': 'x86_64-unknown-linux-gnu', 'windows-2022': 'x86_64-pc-windows-msvc'}
            try:
                receipts = {os_name: json.loads(regular_bytes(Path(directory) / f'install-{os_name}/install-check.json'))
                            for os_name in expected}
            except (OSError, ValueError):
                continue
            if all(r.get('status') == 'passed' and r.get('target') == expected[name]
                   and r.get('source_commit') == commit and r.get('version') == manifest['version']
                   and r.get('manifest_sha256') == digest for name, r in receipts.items()):
                return
    raise ValueError('no successful Linux/Windows combined-install run for this exact packet')


def verify(root, version, commit):
    manifest = json.loads(regular_bytes(root / 'release-manifest.json'))
    check_identity(manifest, version, commit)
    if set(manifest['targets']) != set(TARGETS) or len(manifest['target_evidence']) != len(TARGETS):
        raise ValueError('all three platforms are required before registry publication')
    seen = set()
    for receipt in manifest['target_evidence']:
        check_identity(receipt, version, commit)
        seen.add(receipt['target'])
        if receipt['checks']['cli_mcp_tests'] != 'passed' or receipt['checks']['npm_wheel_local_install'] != 'passed':
            raise ValueError('target-native qualification is incomplete')
    if seen != set(TARGETS):
        raise ValueError('duplicate or missing target evidence')
    files = checked_assets(root, manifest)
    identity = parse_release_version(version)
    npm = files[f'qiongli-{identity.npm_version}.tgz']
    binary_hashes = {}
    with tarfile.open(npm) as packet:
        metadata = json.load(packet.extractfile('package/package.json'))
        if metadata['version'] != identity.npm_version or metadata['publishConfig']['tag'] != identity.npm_dist_tag:
            raise ValueError('npm version/channel mismatch')
        if metadata['name'] != 'qiongli':
            raise ValueError('unexpected npm package')
        scripts = metadata.get('scripts')
        if scripts:
            if scripts != {'postinstall': 'node bin/install.mjs'}:
                raise ValueError('unexpected npm install scripts')
            if packet.extractfile('package/bin/install.mjs').read() != NPM_INSTALL_REVIEW.encode():
                raise ValueError('unexpected npm installation review bytes')
        for target, (_, _, binary) in TARGETS.items():
            data = packet.extractfile(f'package/native/{target}/{binary}').read()
            validate_binary(data, target)
            binary_hashes[target] = hashlib.sha256(data).hexdigest()
            extension = 'zip' if target.endswith('msvc') else 'tar.gz'
            if cli_binary(files[f'qiongli-{version}-{target}.{extension}'], target) != data:
                raise ValueError('CLI executable differs from npm executable')
    expected = {'aarch64-apple-darwin': 'macosx_', 'x86_64-unknown-linux-gnu': 'manylinux_2_35_',
                'x86_64-pc-windows-msvc': 'win_amd64'}
    wheels = {}
    for target, marker in expected.items():
        matches = [p for name, p in files.items() if name.endswith('.whl') and marker in name]
        if len(matches) != 1 or not matches[0].name.startswith(f'qiongli-{identity.package_version}-'):
            raise ValueError('wheel version/target mismatch')
        with zipfile.ZipFile(matches[0]) as packet:
            metadata = packet.read(f'qiongli-{identity.package_version}.dist-info/METADATA').decode()
            if f'\nVersion: {identity.package_version}\n' not in metadata:
                raise ValueError('wheel metadata version mismatch')
        wheels[target] = matches[0]
    legacy_names = {archive_name(host, version) for host in PLATFORMS}
    native_names = [p.name if p else archive_name(host, version, target)
                    for target in TARGETS for host in PLATFORMS
                    for p in [find_archive(files, host, version, target)]]
    legacy, native = legacy_names.intersection(files), set(native_names).intersection(files)
    if legacy and native:
        raise ValueError('mixed legacy and target-specific marketplace packages')
    if legacy and legacy != legacy_names:
        raise ValueError('both marketplace Plugin archives are required')
    if version == '2.0.1':
        for receipt in manifest['target_evidence']:
            checks = receipt['checks']
            require_transition(checks.get('archive_smoke', {}).get('plugin_source_transition'))
            for package in ('npm', 'pypi'):
                installed = checks.get('registry_install', {}).get(package, {})
                if (installed.get('version') != version or
                        installed.get('content_pack_sha256') != checks['archive_smoke'].get('content_pack_sha256')):
                    raise ValueError('missing or mismatched 2.0.1 installed package evidence')
                require_transition(installed.get('plugin_source_transition'))
            for host in PLATFORMS:
                require_transition(checks.get('marketplace_plugins', {}).get(host, {}).get('plugin_source_transition'))
        next_names = {f'qiongli-next-{host}-plugin-v{version}-{target}.tar.gz'
                      for target in TARGETS for host in PLATFORMS}
        if native != next_names:
            raise ValueError('2.0.1 requires all six Next marketplace archives')
    required_native = any('marketplace_plugins' in r['checks'] for r in manifest['target_evidence'])
    if (native or required_native) and native != set(native_names):
        raise ValueError('all six target-specific marketplace archives are required')
    if native:
        for receipt in manifest['target_evidence']:
            smoke = receipt['checks']['archive_smoke']
            if smoke.get('runtime_path') != 'empty':
                raise ValueError('missing target-native empty-PATH CLI smoke evidence')
            if receipt['target'].endswith('msvc') and not smoke.get('windows_system_dlls'):
                raise ValueError('missing Windows system-DLL inspection evidence')
    if legacy or native:
        pack_hashes = {receipt['checks']['archive_smoke'].get('content_pack_sha256')
                       for receipt in manifest['target_evidence']}
        if len(pack_hashes) != 1 or None in pack_hashes:
            raise ValueError('all three CLI content packs must match the marketplace Plugins')
        verified = []
        for name in (native_names if native else sorted(legacy)):
            provenance = verify_archive(files[name], version, commit)
            if provenance['pack_sha256'] not in pack_hashes:
                raise ValueError('marketplace Plugin content differs from CLI content')
            if native:
                target, host = provenance['target'], provenance['platform']
                if provenance['binary_sha256'] != binary_hashes[target]:
                    raise ValueError('marketplace executable differs from CLI executable')
                receipt = next(r for r in manifest['target_evidence'] if r['target'] == target)
                observed = receipt['checks'].get('marketplace_plugins', {}).get(host, {})
                if (observed.get('status') != 'passed' or observed.get('runtime_path') != 'empty'
                        or observed.get('mcp_tools') != 14 or observed.get('sha256') != provenance['sha256']
                        or observed.get('binary_sha256') != provenance['binary_sha256']):
                    raise ValueError('missing target-native marketplace smoke evidence')
            verified.append(provenance)
        if version == '2.0.1':
            next_ids = {'aarch64-apple-darwin': 'qiongli-next-macos-arm64',
                        'x86_64-unknown-linux-gnu': 'qiongli-next-linux-x64',
                        'x86_64-pc-windows-msvc': 'qiongli-next-windows-x64'}
            if 'marketplace-plugins.json' not in files:
                raise ValueError('marketplace platform index mismatch')
            index = json.loads(regular_bytes(files['marketplace-plugins.json']))
            if any(p['name'] != next_ids.get(p['target']) or p['plugin_path'] != 'plugins/' + p['name']
                   for p in index['plugins']):
                raise ValueError('2.0.1 requires legacy Next marketplace identities')
        if native and ('marketplace-plugins.json' not in files or
                       json.loads(regular_bytes(files['marketplace-plugins.json'])) != marketplace_index(version, commit, verified)):
            raise ValueError('marketplace platform index mismatch')
    if len(files) != 7 + len(legacy) + len(native) + bool(native):
        raise ValueError('unexpected CLI/registry/marketplace artifact set')
    return manifest, npm, wheels


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('mode', choices=['assemble', 'verify', 'install-input', 'check-install', 'qualify-macos'])
    parser.add_argument('--root', required=True, type=Path)
    parser.add_argument('--out', type=Path)
    parser.add_argument('--version', required=True)
    parser.add_argument('--commit', required=True)
    parser.add_argument('--require-ci', action='store_true')
    parser.add_argument('--require-local-macos', action='store_true')
    args = parser.parse_args()
    identity = parse_release_version(args.version)
    if identity.release_line != 'native-2x':
        parser.error('native release required')
    if not re.fullmatch(r'(?:[a-f0-9]{40}|[a-f0-9]{64})', args.commit):
        parser.error('an exact source commit is required')
    if args.mode == 'assemble':
        if not args.out:
            parser.error('--out is required')
        assemble(args.root, args.out, identity.version, args.commit)
        return
    if args.mode in ('qualify-macos', 'check-install'):
        if not args.out:
            parser.error('--out is required')
        install_packet(args.root.resolve(), args.out.resolve(), identity.version, args.commit,
                       local_macos=args.mode == 'qualify-macos')
        return
    manifest, npm, wheels = verify(args.root, identity.version, args.commit)
    if args.require_local_macos:
        require_local_macos(manifest)
    if args.require_ci:
        require_remote_installs(args.root, manifest, args.commit)
    if args.mode == 'install-input':
        target = {('Darwin', 'arm64'): 'aarch64-apple-darwin', ('Linux', 'x86_64'): 'x86_64-unknown-linux-gnu',
                  ('Windows', 'AMD64'): 'x86_64-pc-windows-msvc'}[(platform.system(), platform.machine())]
        selected = {npm.name, wheels[target].name}
        if any(a['file'] == 'marketplace-plugins.json' for a in manifest['artifacts']):
            checks = check_plugins(args.root, identity.version, args.commit, target)
            (args.root / 'marketplace-install-check.json').write_text(json.dumps(checks, indent=2) + '\n')
        (args.root / 'registry-packages.json').write_text(json.dumps({
            'version': identity.version, 'artifacts': [a for a in manifest['artifacts'] if a['file'] in selected]}))
    print(f'Verified three-platform packet {identity.version} at {args.commit}')


if __name__ == '__main__':
    main()
