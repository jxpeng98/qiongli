#!/usr/bin/env python3
"""Assemble or verify one three-platform CLI release packet; never publish."""
import argparse
import hashlib
import json
import os
import subprocess
import re
from pathlib import Path
import shutil
import tarfile
import zipfile

try:
    from .native_registry_install_check import require_transition
    from .native_registry_packages import NPM_INSTALL_REVIEW, NPM_LAUNCHER, TARGETS, LEGACY_TARGETS, LINUX_WHEEL_TAGS, host_target, npm_package, parse_release_version, regular_bytes, validate_binary, requires_deepseek_npm
    from .native_marketplace_plugins import PLATFORMS, archive_name, verify_archive, check_plugins, find_archive, read_plugin_content, verify_deepseek_npm
except ImportError:
    from native_registry_install_check import require_transition
    from native_registry_packages import NPM_INSTALL_REVIEW, NPM_LAUNCHER, TARGETS, LEGACY_TARGETS, LINUX_WHEEL_TAGS, host_target, npm_package, parse_release_version, regular_bytes, validate_binary, requires_deepseek_npm
    from native_marketplace_plugins import PLATFORMS, archive_name, verify_archive, check_plugins, find_archive, read_plugin_content, verify_deepseek_npm


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
    binaries, receipts, plugin_content = {}, [], None
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
            if requires_deepseek_npm(version) and plugin_content is None:
                plugin_content = read_plugin_content(plugins[0], version, commit)
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
    packed = npm_package(npm_work, binaries, version, plugin_content=plugin_content)
    shutil.copyfile(packed, out / packed.name)
    output_files = {p.name: p for p in out.iterdir()}
    native_plugins = [find_archive(output_files, host, version, target) for target in TARGETS for host in PLATFORMS]
    if any(native_plugins):
        if not all(native_plugins):
            raise ValueError('all target-specific marketplace archives are required')
        index = marketplace_index(version, commit, [verify_archive(p, version, commit) for p in native_plugins])
        (out / 'marketplace-plugins.json').write_text(json.dumps(index, indent=2) + '\n')
    manifest = {'schema_version': 2, 'version': version, 'source_commit': commit, 'targets': list(TARGETS),
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


def verify(root, version, commit):
    manifest = json.loads(regular_bytes(root / 'release-manifest.json'))
    check_identity(manifest, version, commit)
    targets = packet_targets(manifest, version)
    if (set(manifest['targets']) != set(targets) or len(manifest['targets']) != len(targets)
            or len(manifest['target_evidence']) != len(targets)):
        raise ValueError('all required OS/architecture targets are required before registry publication')
    seen = set()
    for receipt in manifest['target_evidence']:
        check_identity(receipt, version, commit)
        seen.add(receipt['target'])
        if receipt['checks']['cli_mcp_tests'] != 'passed' or receipt['checks']['npm_wheel_local_install'] != 'passed':
            raise ValueError('target-native qualification is incomplete')
        if requires_deepseek_npm(version):
            dsh = receipt['checks'].get('registry_install', {}).get('npm', {}).get('deepseek_plugin', {})
            if (dsh.get('skills') != 22 or dsh.get('mcp_tools') != 32 or
                    dsh.get('content_pack_sha256') != receipt['checks']['archive_smoke'].get('content_pack_sha256')):
                raise ValueError('missing target-native DeepSeek npm installation evidence')
    if seen != set(targets):
        raise ValueError('duplicate or missing target evidence')
    files = checked_assets(root, manifest)
    identity = parse_release_version(version)
    npm = files[f'qiongli-{identity.npm_version}.tgz']
    binary_hashes, binaries = {}, {}
    with tarfile.open(npm) as packet:
        metadata = json.load(packet.extractfile('package/package.json'))
        if metadata['version'] != identity.npm_version or metadata['publishConfig']['tag'] != identity.npm_dist_tag:
            raise ValueError('npm version/channel mismatch')
        if metadata['name'] != 'qiongli':
            raise ValueError('unexpected npm package')
        if manifest.get('schema_version') == 2 and packet.extractfile('package/bin/qiongli.mjs').read() != NPM_LAUNCHER.encode():
            raise ValueError('npm launcher does not match the supported target dispatch')
        scripts = metadata.get('scripts')
        if scripts:
            if scripts != {'postinstall': 'node bin/install.mjs'}:
                raise ValueError('unexpected npm install scripts')
            if packet.extractfile('package/bin/install.mjs').read() != NPM_INSTALL_REVIEW.encode():
                raise ValueError('unexpected npm installation review bytes')
        for target, (_, _, binary) in targets.items():
            data = packet.extractfile(f'package/native/{target}/{binary}').read()
            validate_binary(data, target)
            binaries[target] = data
            binary_hashes[target] = hashlib.sha256(data).hexdigest()
            extension = 'zip' if target.endswith('msvc') else 'tar.gz'
            if cli_binary(files[f'qiongli-{version}-{target}.{extension}'], target) != data:
                raise ValueError('CLI executable differs from npm executable')
        if requires_deepseek_npm(version):
            if (metadata.get('main') != 'dsh/index.mjs' or
                    metadata.get('dsh') != {'bundle': {'patch': './dsh/cordis.patch.yml'}} or
                    'dsh/' not in metadata.get('files', [])):
                raise ValueError('missing DeepSeek npm Plugin entry')
            bundle = {}
            for member in packet:
                if not member.name.startswith('package/dsh/'):
                    continue
                name = member.name.removeprefix('package/dsh/')
                if not member.isfile() or name in bundle:
                    raise ValueError('unsafe or duplicate DeepSeek npm member')
                bundle[name] = packet.extractfile(member).read()
            observed = verify_deepseek_npm(bundle, version, commit, binaries)
            if any(observed['pack_sha256'] != r['checks']['archive_smoke'].get('content_pack_sha256')
                   for r in manifest['target_evidence']):
                raise ValueError('DeepSeek npm content differs from CLI content')
    expected = {'aarch64-apple-darwin': r'macosx_\d+_\d+_arm64',
                'x86_64-pc-windows-msvc': 'win_amd64', **LINUX_WHEEL_TAGS}
    wheels = {}
    for target in targets:
        pattern = re.escape(f'qiongli-{identity.package_version}-py3-none-') + expected[target] + r'\.whl'
        matches = [p for name, p in files.items() if re.fullmatch(pattern, name)]
        if len(matches) != 1:
            raise ValueError('wheel version/target mismatch')
        with zipfile.ZipFile(matches[0]) as packet:
            metadata = packet.read(f'qiongli-{identity.package_version}.dist-info/METADATA').decode()
            if f'\nVersion: {identity.package_version}\n' not in metadata:
                raise ValueError('wheel metadata version mismatch')
            if packet.read(f'qiongli_native/bin/{targets[target][2]}') != binaries[target]:
                raise ValueError('wheel executable differs from CLI/npm executable')
            tag = matches[0].name.removeprefix(f'qiongli-{identity.package_version}-').removesuffix('.whl')
            if f'\nTag: {tag}\n' not in packet.read(f'qiongli-{identity.package_version}.dist-info/WHEEL').decode():
                raise ValueError('wheel compatibility tag mismatch')
        wheels[target] = matches[0]
    legacy_names = {archive_name(host, version) for host in PLATFORMS}
    native_names = [p.name if p else archive_name(host, version, target)
                    for target in targets for host in PLATFORMS
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
                      for target in targets for host in PLATFORMS}
        if native != next_names:
            raise ValueError('2.0.1 requires all target-specific Next marketplace archives')
    required_native = any('marketplace_plugins' in r['checks'] for r in manifest['target_evidence'])
    if (native or required_native) and native != set(native_names):
        raise ValueError('all target-specific marketplace archives are required')
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
            raise ValueError('all CLI content packs must match the marketplace Plugins')
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
                        'aarch64-unknown-linux-gnu': 'qiongli-next-linux-arm64',
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
    if len(files) != 1 + 2 * len(targets) + len(legacy) + len(native) + bool(native):
        raise ValueError('unexpected CLI/registry/marketplace artifact set')
    return manifest, npm, wheels


def packet_targets(manifest, version):
    """New packets require ARM64; immutable releases through 2.1.1 keep their roster."""
    if type(manifest.get('schema_version')) is int and manifest['schema_version'] == 2:
        return TARGETS
    if 'schema_version' not in manifest and tuple(map(int, version.split('-')[0].split('.'))) <= (2, 1, 1):
        return LEGACY_TARGETS
    raise ValueError('release packet requires schema_version 2 with Linux ARM64')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('mode', choices=['assemble', 'verify', 'install-input'])
    parser.add_argument('--root', required=True, type=Path)
    parser.add_argument('--out', type=Path)
    parser.add_argument('--version', required=True)
    parser.add_argument('--commit', required=True)
    parser.add_argument('--require-ci', action='store_true')
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
    manifest, npm, wheels = verify(args.root, identity.version, args.commit)
    if args.require_ci:
        repo = os.environ['GITHUB_REPOSITORY']
        runs = json.loads(subprocess.check_output(['gh', 'api',
            f'repos/{repo}/actions/workflows/native-cli-distribution.yml/runs?head_sha={args.commit}&status=success'], text=True))
        if not any(r['head_sha'] == args.commit and r['conclusion'] == 'success' for r in runs['workflow_runs']):
            raise ValueError('no successful three-platform combined-install CI run at this source')
    if args.mode == 'install-input':
        target = host_target()
        if target not in wheels:
            raise ValueError('release has no wheel for the current OS/architecture')
        selected = {npm.name, wheels[target].name}
        if any(a['file'] == 'marketplace-plugins.json' for a in manifest['artifacts']):
            checks = check_plugins(args.root, identity.version, args.commit, target)
            (args.root / 'marketplace-install-check.json').write_text(json.dumps(checks, indent=2) + '\n')
        (args.root / 'registry-packages.json').write_text(json.dumps({
            'version': identity.version, 'artifacts': [a for a in manifest['artifacts'] if a['file'] in selected]}))
    print(f'Verified three-platform packet {identity.version} at {args.commit}')


if __name__ == '__main__':
    main()
