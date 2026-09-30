#!/usr/bin/env python3
"""Stage native CLI distribution packages; never publish."""
from __future__ import annotations

import argparse
import base64
import csv
import hashlib
import io
import json
import os
from pathlib import Path
import re
import shutil
import stat
import subprocess
import tarfile
import tomllib
import zipfile

try:
    from .release_version import parse_release_version
except ImportError:
    from release_version import parse_release_version

ROOT = Path(__file__).resolve().parents[2]
NATIVE = ROOT / 'packages/qiongli-native'
TARGETS = {
    'aarch64-apple-darwin': ('darwin', 'arm64', 'qiongli'),
    'x86_64-unknown-linux-gnu': ('linux', 'x64', 'qiongli'),
    'x86_64-pc-windows-msvc': ('win32', 'x64', 'qiongli.exe'),
}


def npm_command(*args):
    if os.name == 'nt':
        npm = Path(shutil.which('npm.cmd')).parent / 'node_modules/npm/bin/npm-cli.js'
        return [shutil.which('node'), str(npm), *map(str, args)]
    return ['npm', *map(str, args)]


def regular_bytes(path: Path) -> bytes:
    if path.is_symlink() or not path.is_file():
        raise ValueError(f'expected regular file: {path}')
    return path.read_bytes()


def copy_tree(source: Path, destination: Path) -> None:
    for path in sorted(source.rglob('*')):
        if path.is_symlink():
            raise ValueError(f'symlinks are not package inputs: {path}')
        if path.is_file():
            target = destination / path.relative_to(source)
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(regular_bytes(path))


def stage_cargo(out: Path, version: str) -> Path:
    """Cargo owns archive normalization and dependency ordering after staging."""
    manifest_data = tomllib.loads(regular_bytes(NATIVE / 'Cargo.toml').decode())
    lock = json.loads(regular_bytes(NATIVE / 'crates/qiongli-content/resources/qiongli-core.lock.json'))
    if manifest_data['workspace']['package']['version'] != version or lock['content_version'] != version:
        raise ValueError('Cargo, content and requested versions must match')
    workspace = out / 'cargo-source'
    workspace.mkdir()
    manifest = regular_bytes(NATIVE / 'Cargo.toml').decode().replace('\r\n', '\n')
    (workspace / 'Cargo.toml').write_text(manifest.replace('publish = false', 'publish = ["crates-io"]'), newline='\n')
    shutil.copyfile(NATIVE / 'Cargo.lock', workspace / 'Cargo.lock')
    members = tomllib.loads(manifest)['workspace']['members']
    for member in members:
        source, dest = NATIVE / member, workspace / member
        dest.mkdir(parents=True)
        text = regular_bytes(source / 'Cargo.toml').decode().replace('\r\n', '\n')
        text = re.sub(r'(\{ path = "[^"]+")', rf'\1, version = "={version}"', text)
        # Only the CLI is a registry product. Repository-only tests/examples stay
        # in the checkout; their fixtures are not dependencies of cargo install.
        text = text.replace('[package]\n', '[package]\nautoexamples = false\nautotests = false\nautobenches = false\ninclude = ["Cargo.toml", "src/**", "build.rs", "resources/**", "schemas/**", "icons/**", "package-assets/**", "LICENSE", "README.md"]\n', 1)
        (dest / 'Cargo.toml').write_text(text, newline='\n')
        for directory in ('src', 'resources', 'schemas', 'icons'):
            if (source / directory).is_dir():
                copy_tree(source / directory, dest / directory)
        if (source / 'build.rs').exists():
            shutil.copyfile(source / 'build.rs', dest / 'build.rs')
        shutil.copyfile(ROOT / 'LICENSE', dest / 'LICENSE')
        readme = package_readme(version, 'cargo') if member == 'apps/qiongli' else (
            f'# {source.name} {version}\n\nInternal Rust library for the Qiongli research CLI.\n'
            'Install the `qiongli` crate for the command-line application.\n')
        (dest / 'README.md').write_text(readme)
    assets = workspace / 'apps/qiongli/package-assets'
    copy_tree(ROOT / 'content', assets / 'content')
    # Keep the Rust composer's declared source list as the authority.
    contract = (NATIVE / 'crates/qiongli-platform/src/zotero_companion.rs').read_text()
    paths = re.search(r'ZOTERO_COMPANION_SOURCE_PATHS:.*?= \[(.*?)\];', contract, re.S)
    if paths is None:
        raise ValueError('Zotero source manifest unavailable')
    for relative in re.findall(r'"([^"]+)"', paths[1]):
        target = assets / 'qiongli-zotero-companion' / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(regular_bytes(ROOT / 'packages/qiongli-zotero-companion' / relative))
    return workspace


NPM_INSTALL_REVIEW = """import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
if (process.stdin.isTTY && process.stdout.isTTY) {
  const result = spawnSync(process.execPath, [fileURLToPath(new URL('./qiongli.mjs', import.meta.url)), 'install', 'migrate', '--interactive'], { stdio: 'inherit' });
  if (result.error || result.status !== 0) console.error('Installation review was skipped or cancelled. Run the installed Qiongli with install migrate --interactive to review later.');
}
"""


NPM_LAUNCHER = '''#!/usr/bin/env node
import { spawn } from 'node:child_process';
import { fileURLToPath } from 'node:url';
const targets = { 'darwin-arm64': 'aarch64-apple-darwin/qiongli', 'linux-x64': 'x86_64-unknown-linux-gnu/qiongli', 'win32-x64': 'x86_64-pc-windows-msvc/qiongli.exe' };
const target = targets[`${process.platform}-${process.arch}`];
if (!target) {
  console.error(`Unsupported Qiongli platform: ${process.platform}/${process.arch}`);
  process.exit(1);
}
const child = spawn(fileURLToPath(new URL(`../native/${target}`, import.meta.url)), process.argv.slice(2), { stdio: 'inherit' });
for (const signal of ['SIGINT', 'SIGTERM', 'SIGHUP']) process.on(signal, () => child.kill(signal));
child.on('error', () => { console.error('Unable to start the packaged Qiongli executable.'); process.exitCode = 1; });
child.on('close', (code, signal) => {
  if (signal) { process.removeAllListeners(signal); process.kill(process.pid, signal); }
  else process.exitCode = code ?? 1;
});
'''
PYTHON_LAUNCHER = '''import os
from pathlib import Path
import sys


def main():
    executable = Path(__file__).parent / "bin" / ("qiongli.exe" if os.name == "nt" else "qiongli")
    if os.name == "nt":
        import subprocess
        raise SystemExit(subprocess.call([str(executable), *sys.argv[1:]]))
    os.execv(str(executable), [str(executable), *sys.argv[1:]])
'''


def wheel(out: Path, version: str, platform_tag: str, binary: bytes, readme: str) -> Path:
    dist = f'qiongli-{version}.dist-info'
    binary_name = 'qiongli.exe' if platform_tag == 'win_amd64' else 'qiongli'
    files = {
        'qiongli_native/__init__.py': PYTHON_LAUNCHER.encode(),
        f'qiongli_native/bin/{binary_name}': binary,
        f'{dist}/METADATA': (f'Metadata-Version: 2.4\nName: qiongli\nVersion: {version}\nSummary: {cli_description()}\nRequires-Python: >=3.9\nLicense-Expression: MIT\nLicense-File: LICENSE\nDescription-Content-Type: text/markdown\n\n{readme}').encode(),
        f'{dist}/WHEEL': f'Wheel-Version: 1.0\nGenerator: qiongli-native-registry-packages\nRoot-Is-Purelib: false\nTag: py3-none-{platform_tag}\n'.encode(),
        f'{dist}/entry_points.txt': b'[console_scripts]\nqiongli = qiongli_native:main\nql = qiongli_native:main\n',
        f'{dist}/licenses/LICENSE': regular_bytes(ROOT / 'LICENSE'),
    }
    record = io.StringIO(newline='')
    writer = csv.writer(record, lineterminator='\n')
    for name, data in sorted(files.items()):
        digest = base64.urlsafe_b64encode(hashlib.sha256(data).digest()).rstrip(b'=').decode()
        writer.writerow([name, f'sha256={digest}', len(data)])
    writer.writerow([f'{dist}/RECORD', '', ''])
    files[f'{dist}/RECORD'] = record.getvalue().encode()
    path = out / f'qiongli-{version}-py3-none-{platform_tag}.whl'
    with zipfile.ZipFile(path, 'x', compression=zipfile.ZIP_DEFLATED) as archive:
        for name, data in sorted(files.items()):
            info = zipfile.ZipInfo(name, (1980, 1, 1, 0, 0, 0))
            info.create_system = 3
            mode = 0o755 if name == f'qiongli_native/bin/{binary_name}' else 0o644
            info.external_attr = (stat.S_IFREG | mode) << 16
            info.compress_type = zipfile.ZIP_DEFLATED
            archive.writestr(info, data)
    return path


def validate_binary(binary: bytes, target: str) -> None:
    if target == 'aarch64-apple-darwin':
        valid = binary[:8] == bytes.fromhex('cffaedfe0c000001')
    elif target == 'x86_64-unknown-linux-gnu':
        valid = binary[:6] == b'\x7fELF\x02\x01' and binary[18:20] == b'\x3e\x00'
    elif target == 'x86_64-pc-windows-msvc':
        offset = int.from_bytes(binary[60:64], 'little')
        valid = binary[:2] == b'MZ' and offset >= 64 and binary[offset:offset+6] == b'PE\x00\x00\x64\x86'
    else:
        valid = False
    if not valid:
        raise ValueError(f'executable format/architecture does not match {target}')


def cli_description() -> str:
    return tomllib.loads(regular_bytes(NATIVE / 'apps/qiongli/Cargo.toml').decode())['package']['description']


def package_readme(version: str, channel: str) -> str:
    identity = parse_release_version(version)
    instructions = {
        'npm': f"""Requires Node.js 18+. The package includes the native executables; installation
and first launch do not download another runtime.

```sh
npm install --global qiongli@{identity.npm_version}
```

To follow the {identity.npm_dist_tag} channel, use `npm install --global qiongli@{identity.npm_dist_tag}`.
The optional installation review needs a terminal. Allow it for one invocation
with `--allow-scripts=qiongli --foreground-scripts`, or use `--ignore-scripts`
and run `qiongli setup` afterward. Skipping this review does not disable the CLI.
Use `npm uninstall --global qiongli` to remove this package.
""",
        'pypi': f"""Requires Python 3.9+. The wheel includes the native executable and a small
Python launcher; no additional Python packages are required.

```sh
python -m pip install --upgrade "qiongli=={identity.package_version}"
```

Use `python -m pip uninstall qiongli` to remove this package. Run `qiongli setup`
after installation to review other CLI copies visible on this computer.
""",
        'cargo': f"""Builds the CLI from source with Rust 1.97+ and a native linker.
The compiled executable includes its research resources and needs no Rust runtime.

```sh
cargo install qiongli --version {version} --locked
```

Prereleases use an exact version; Cargo has no dist-tags. Use `cargo uninstall qiongli`
to remove this package. Run `qiongli setup` after installation to review other CLI copies.
""",
    }
    if channel not in instructions:
        raise ValueError('unknown package documentation channel')
    if channel == 'npm' and requires_deepseek_npm(version):
        instructions[channel] += f"""
## DeepSeek Harness Plugin

In Desktop's Add plugin dialog, select Official npm registry and enter
`qiongli@{identity.npm_version}`. The same package includes 22 Skill entries and
Full MCP; a separate global CLI installation or install script is not required.
The official local CLI can install it into the Desktop profile:

```sh
dsh plugin --profile desktop add qiongli@{identity.npm_version}
```

Replace `desktop` with your own profile name when needed. For a local install,
replace the package specification with the absolute path to this `.tgz` archive.
DSH 0.2 upgrades require removing the installed `qiongli` Plugin, then installing
the new version. Start a fresh session and check the Skill catalog and MCP tools.
"""
    return f"""# Qiongli {version}

{cli_description()}

Qiongli includes research Skills, templates and Lite/Full MCP. Your Host supplies
the model and its credentials; no Qiongli desktop App is required. The supported
binary targets are macOS Apple Silicon, Windows x64 and Linux x64 (glibc 2.35+).

## Install and update

{instructions[channel]}
## Use the CLI

`qiongli` and `ql` share the same commands. Start with:

```sh
qiongli --version
qiongli install
qiongli mcp check
qiongli doctor
qiongli content
qiongli help install plugin
```

In a terminal, `install` opens the guide. `install plugin` installs or refreshes
the bundled Plugin using its registered source directory. File changes and official
Host registration require separate confirmations; `upgrade plugin` is an alias.
The Host loads Skills and Full MCP from its Plugin cache, with no copy into
`.agents/skills`. `install skills` only exports guidance to `.qiongli-skills`.
Start a new Host session after updating; registration alone does not prove its
tools are available.

Use `--json` for scripts. Scripts and MCP never prompt. `qiongli setup` reviews
visible installations without deleting files, changing PATH or replacing model settings.
If another CLI takes precedence, invoke the selected executable by its full path.
Research writes retain preview, explicit approval and revision checks. Keep earlier
research records and backups when migrating; upgrading a package does not authorize cleanup.

[Command and installation guide](https://github.com/jxpeng98/qiongli/blob/{identity.repo_tag}/docs/guide/cli-2x.md)
"""


def requires_deepseek_npm(version: str) -> bool:
    return tuple(map(int, version.split('-')[0].split('.'))) >= (2, 1, 0)


def npm_package(out: Path, binaries: dict[str, Path], version: str,
                *, plugin_content: tuple[dict, dict[str, bytes]] | None = None) -> Path:
    identity = parse_release_version(version)
    if requires_deepseek_npm(version) and plugin_content is None:
        raise ValueError('2.1+ npm packages require source-bound Plugin content')
    npm = out / 'npm'
    (npm / 'bin').mkdir(parents=True)
    (npm / 'bin/qiongli.mjs').write_text(NPM_LAUNCHER)
    (npm / 'bin/qiongli.mjs').chmod(0o755)
    (npm / 'bin/install.mjs').write_text(NPM_INSTALL_REVIEW)
    for target, binary in binaries.items():
        data = regular_bytes(binary)
        validate_binary(data, target)
        dest = npm / 'native' / target / TARGETS[target][2]
        dest.parent.mkdir(parents=True)
        dest.write_bytes(data)
        dest.chmod(0o755)
    (npm / 'README.md').write_text(package_readme(version, 'npm'))
    shutil.copyfile(ROOT / 'LICENSE', npm / 'LICENSE')
    # ponytail: bundle three binaries in one package; split only if download size becomes a problem.
    manifest = {
        'name': 'qiongli', 'version': identity.npm_version, 'description': cli_description(),
        'type': 'module', 'license': 'MIT',
        'repository': {'type': 'git', 'url': 'git+https://github.com/jxpeng98/qiongli.git'},
        'bin': {'qiongli': 'bin/qiongli.mjs', 'ql': 'bin/qiongli.mjs'},
        'os': sorted({TARGETS[t][0] for t in binaries}), 'cpu': sorted({TARGETS[t][1] for t in binaries}),
        'scripts': {'postinstall': 'node bin/install.mjs'},
        'engines': {'node': '>=18'}, 'files': ['bin/', 'native/', 'README.md', 'LICENSE'],
        'publishConfig': {'access': 'public', 'tag': identity.npm_dist_tag},
    }
    if requires_deepseek_npm(version):
        try:
            from .native_marketplace_plugins import deepseek_npm_files, verify_deepseek_npm
        except ImportError:
            from native_marketplace_plugins import deepseek_npm_files, verify_deepseek_npm
        metadata, content = plugin_content
        binary_bytes = {target: regular_bytes(binary) for target, binary in binaries.items()}
        bundle = deepseek_npm_files(metadata, content, version, binary_bytes)
        verify_deepseek_npm(bundle, version, metadata['source_commit'], binary_bytes)
        for name, data in bundle.items():
            dest = npm / 'dsh' / name
            dest.parent.mkdir(parents=True, exist_ok=True)
            dest.write_bytes(data)
        manifest.update(main='dsh/index.mjs', dsh={'bundle': {'patch': './dsh/cordis.patch.yml'}})
        manifest['files'].append('dsh/')
    (npm / 'package.json').write_text(json.dumps(manifest, indent=2) + '\n')
    packed = json.loads(subprocess.check_output(npm_command(
        'pack', '--json', '--ignore-scripts', '--pack-destination', out,
        '--cache', out / 'npm-cache'), cwd=npm, text=True))
    tarball = out / packed[0]['filename']
    with tarfile.open(tarball) as archive:
        for target, binary in binaries.items():
            if archive.extractfile(f'package/native/{target}/{TARGETS[target][2]}').read() != regular_bytes(binary):
                raise ValueError('npm archive changed the candidate executable')
    return tarball


def binary_packages(out: Path, binary_path: Path, version: str,
                    target: str = 'aarch64-apple-darwin',
                    *, plugin_content: tuple[dict, dict[str, bytes]] | None = None) -> list[Path]:
    binary = regular_bytes(binary_path)
    validate_binary(binary, target)
    reported = subprocess.check_output([str(binary_path), '--version'], text=True).strip()
    if reported != f'qiongli {version}':
        raise ValueError('executable version does not match the native workspace')
    content = json.loads(subprocess.check_output([str(binary_path), 'content', 'list', '--json'], text=True))
    if content['content_version'] != version:
        raise ValueError('embedded content version does not match the executable')
    if target == 'aarch64-apple-darwin':
        commands = subprocess.check_output(['otool', '-l', str(binary_path)], text=True)
        versions = re.findall(r'^\s*minos (\d+)\.(\d+)(?:\.\d+)?$', commands, re.M)
        if len(versions) != 1:
            raise ValueError('cannot determine one macOS deployment target')
        major, minor = versions[0]
        platform_tag = f'macosx_{major}_{minor}_arm64'
    else:
        platform_tag = 'win_amd64' if target.endswith('msvc') else 'linux_x86_64'
    identity = parse_release_version(version)
    whl = wheel(out, identity.package_version, platform_tag, binary, package_readme(version, 'pypi'))
    if platform_tag == 'linux_x86_64':
        repaired = out / 'manylinux'
        subprocess.run(['auditwheel', 'repair', '--only-plat', '--plat', 'manylinux_2_35_x86_64',
                        '--wheel-dir', str(repaired), str(whl)], check=True)
        files = list(repaired.glob('*.whl'))
        if len(files) != 1:
            raise ValueError('expected exactly one audited Linux wheel')
        with zipfile.ZipFile(files[0]) as archive:
            if archive.read('qiongli_native/bin/qiongli') != binary:
                raise ValueError('Linux executable needs shared library bundling in standalone/npm too')
        whl.unlink()
        whl = Path(shutil.move(files[0], out / files[0].name))
    return [npm_package(out, {target: binary_path}, version, plugin_content=plugin_content), whl]


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--out-dir', required=True, type=Path)
    parser.add_argument('--package-cargo', action='store_true', help='also create Cargo archives without claiming verification')
    parser.add_argument('--binary', type=Path, help='optional target-native candidate')
    parser.add_argument('--target', choices=TARGETS, default='aarch64-apple-darwin')
    parser.add_argument('--plugin-content', type=Path, help='source-bound native content export for 2.1+ npm packages')
    args = parser.parse_args()
    out = args.out_dir.expanduser().absolute()
    if out.exists() or out.is_symlink() or ROOT == out.resolve() or ROOT in out.resolve().parents:
        parser.error('--out-dir must be a new directory outside the source checkout')
    version = tomllib.loads((NATIVE / 'Cargo.toml').read_text())['workspace']['package']['version']
    if parse_release_version(version).release_line != 'native-2x':
        parser.error('expected native 2.x version')
    out.mkdir(parents=True)
    workspace = stage_cargo(out, version)
    plugin_content = None
    if args.plugin_content:
        try:
            from .native_marketplace_plugins import EXPORT, read_content
        except ImportError:
            from native_marketplace_plugins import EXPORT, read_content
        commit = json.loads(regular_bytes(args.plugin_content / EXPORT))['source_commit']
        plugin_content = read_content(args.plugin_content, version, commit)
    artifacts = binary_packages(out, args.binary.absolute(), version, args.target,
                                plugin_content=plugin_content) if args.binary else []
    if args.package_cargo:
        subprocess.run(['cargo', 'package', '--manifest-path', str(workspace / 'Cargo.toml'),
                        '--workspace', '--no-default-features', '--no-verify', '--offline',
                        '--allow-dirty', '--target-dir', str(out / 'cargo-build')], check=True)
        for archive in sorted((out / 'cargo-build/package').glob('*.crate')):
            target = out / archive.name
            shutil.copyfile(archive, target)
            artifacts.append(target)
    receipt = {'version': version, 'status': 'staged-unpublished', 'cargo_source': str(workspace),
               'binary_target': args.target if args.binary else None,
               'binary_sha256': hashlib.sha256(regular_bytes(args.binary)).hexdigest() if args.binary else None,
               'cargo_verification': 'not-run',
               'artifacts': [{'file': p.name, 'sha256': hashlib.sha256(p.read_bytes()).hexdigest(), 'bytes': p.stat().st_size} for p in artifacts]}
    (out / 'registry-packages.json').write_text(json.dumps(receipt, indent=2) + '\n')
    print(json.dumps(receipt, indent=2))


if __name__ == '__main__':
    main()
