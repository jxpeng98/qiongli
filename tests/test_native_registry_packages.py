from __future__ import annotations

import base64
import csv
import hashlib
import io
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import tarfile
import tomllib
import unittest
from unittest.mock import patch
import zipfile

import yaml

from tooling.scripts import native_registry_packages as packages


class NativeRegistryPackagesTests(unittest.TestCase):
    def test_pip_resolves_native_wheels_instead_of_falling_back_to_legacy(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            index = root / 'index'
            index.mkdir()
            # Model the existing universal 1.x package alongside platform-only 2.x.
            packages.wheel(index, '1.17.0', 'any', b'legacy fixture', 'fixture')
            tags = ['macosx_11_0_arm64', 'manylinux_2_35_x86_64', 'win_amd64']
            for tag in tags:
                packages.wheel(index, '2.1.2', tag, b'native fixture', 'fixture')
            count = 0
            def resolve(tag, requirement='qiongli>=2,<3', python='3.12'):
                nonlocal count
                count += 1
                destination = root / str(count)
                result = subprocess.run([
                    sys.executable, '-m', 'pip', '--isolated', '--disable-pip-version-check',
                    'download', '--no-index', '--no-deps', '--only-binary=:all:',
                    '--find-links', str(index), '--dest', str(destination),
                    '--platform', tag, '--python-version', python, requirement,
                ], capture_output=True, text=True, timeout=30)
                return result, list(destination.glob('*.whl'))
            result, files = resolve('manylinux_2_35_aarch64', 'qiongli')
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual([p.name for p in files], ['qiongli-1.17.0-py3-none-any.whl'])
            tags.append('manylinux_2_35_aarch64')
            packages.wheel(index, '2.1.2', tags[-1], b'native ARM64 fixture', 'fixture')
            for tag in tags:
                for python in ('3.9', '3.12'):
                    result, files = resolve(tag, python=python)
                    self.assertEqual(result.returncode, 0, result.stderr)
                    self.assertEqual([p.name for p in files], [f'qiongli-2.1.2-py3-none-{tag}.whl'])
            for tag in ('manylinux_2_34_aarch64', 'musllinux_1_2_aarch64', 'win_arm64'):
                result, files = resolve(tag)
                self.assertNotEqual(result.returncode, 0)
                self.assertEqual(files, [])

    def test_host_target_and_distribution_jobs_cover_every_supported_pair(self):
        for target, system, machine in [
            ('aarch64-apple-darwin', 'Darwin', 'arm64'),
            ('x86_64-unknown-linux-gnu', 'Linux', 'x86_64'),
            ('aarch64-unknown-linux-gnu', 'Linux', 'aarch64'),
            ('aarch64-unknown-linux-gnu', 'Linux', 'arm64'),
            ('x86_64-pc-windows-msvc', 'Windows', 'AMD64'),
            (None, 'Linux', 'armv7l'),
            (None, 'Windows', 'ARM64'),
        ]:
            with self.subTest(system=system, machine=machine), \
                    patch.object(packages.platform, 'system', return_value=system), \
                    patch.object(packages.platform, 'machine', return_value=machine):
                self.assertEqual(packages.host_target(), target)
        workflow = yaml.safe_load((packages.ROOT / '.github/workflows/native-cli-distribution.yml').read_text())
        build = workflow['jobs']['build']['strategy']['matrix']['include']
        self.assertEqual({row['target'] for row in build}, set(packages.TARGETS))
        self.assertIn({'os': 'ubuntu-22.04-arm', 'target': 'aarch64-unknown-linux-gnu'}, build)
        self.assertEqual(set(workflow['jobs']['install']['strategy']['matrix']['os']),
                         {row['os'] for row in build})
        cargo = yaml.safe_load((packages.ROOT / '.github/workflows/publish-cargo.yml').read_text())
        for job in ('qualify', 'public-install'):
            self.assertEqual(set(cargo['jobs'][job]['strategy']['matrix']['os']),
                             {row['os'] for row in build})

    def test_npm_launcher_selects_each_architecture_and_rejects_unsupported_pairs(self):
        # Exercise the emitted launcher in Node while replacing only process/child IO.
        source = packages.NPM_LAUNCHER.replace("import { spawn } from 'node:child_process';", '''
const spawn = (command, args, options) => {
  console.log(JSON.stringify({command, args, options}));
  return {kill() {}, on() {}};
};''')
        with tempfile.TemporaryDirectory(prefix='qiongli path # ') as temporary:
            root = Path(temporary)
            launcher = root / 'bin/qiongli.mjs'
            launcher.parent.mkdir()
            launcher.write_text(source)
            for target, (system, arch, binary) in packages.TARGETS.items():
                script = "Object.defineProperty(process,'platform',{value:process.argv[1]});" \
                         "Object.defineProperty(process,'arch',{value:process.argv[2]});" \
                         "process.argv=['node','qiongli','--version','argument with spaces'];" \
                         "await import(process.env.TEST_LAUNCHER);"
                env = dict(os.environ, TEST_LAUNCHER=launcher.as_uri())
                result = subprocess.run(['node', '--input-type=module', '-e', script, system, arch],
                                        env=env, capture_output=True, text=True, check=True)
                observed = json.loads(result.stdout)
                self.assertEqual(Path(observed['command']), root / 'native' / target / binary)
                self.assertEqual(observed['args'], ['--version', 'argument with spaces'])
                self.assertEqual(observed['options'], {'stdio': 'inherit'})
            for system, arch in [('linux', 'arm'), ('win32', 'arm64'), ('darwin', 'x64')]:
                result = subprocess.run(['node', '--input-type=module', '-e', script, system, arch],
                                        env=env, capture_output=True, text=True)
                self.assertNotEqual(result.returncode, 0)
                self.assertIn('Unsupported Qiongli platform', result.stderr)
                self.assertEqual(result.stdout, '')

    def test_linux_wheels_audit_the_matching_architecture_and_refuse_binary_changes(self):
        version = '2.0.0-beta.3'
        for target, machine in [('x86_64-unknown-linux-gnu', b'\x3e\x00'),
                                ('aarch64-unknown-linux-gnu', b'\xb7\x00')]:
            for tamper in (False, True):
                with self.subTest(target=target, tamper=tamper), tempfile.TemporaryDirectory() as temporary:
                    root = Path(temporary)
                    binary = root / 'candidate'
                    data = b'\x7fELF\x02\x01' + bytes(12) + machine + bytes(80)
                    binary.write_bytes(data)
                    wrong = 'aarch64-unknown-linux-gnu' if target.startswith('x86_64') else 'x86_64-unknown-linux-gnu'
                    with self.assertRaisesRegex(ValueError, 'architecture'):
                        packages.validate_binary(data, wrong)
                    def audit(argv, **kwargs):
                        self.assertEqual(argv[:5], ['auditwheel', 'repair', '--only-plat', '--plat',
                                                  packages.LINUX_WHEEL_TAGS[target]])
                        destination = Path(argv[6])
                        destination.mkdir()
                        packages.wheel(destination, '2.0.0b3', argv[4], data + (b'changed' if tamper else b''), 'test')
                    with patch.object(packages.subprocess, 'check_output', side_effect=[
                            f'qiongli {version}', json.dumps({'content_version': version})]), \
                            patch.object(packages.subprocess, 'run', side_effect=audit), \
                            patch.object(packages, 'npm_package', return_value=root / 'fixture.tgz'):
                        if tamper:
                            with self.assertRaisesRegex(ValueError, 'shared library bundling'):
                                packages.binary_packages(root, binary, version, target)
                        else:
                            _, wheel = packages.binary_packages(root, binary, version, target)
                            self.assertTrue(wheel.name.endswith(packages.LINUX_WHEEL_TAGS[target] + '.whl'))

    def test_binary_content_version_mismatch_refuses_before_packaging(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            binary = root / 'candidate'
            binary.write_bytes(bytes.fromhex('cffaedfe0c000001'))
            with patch.object(packages.subprocess, 'check_output', side_effect=[
                'qiongli 2.0.0-beta.3', '{"content_version":"2.0.0-beta.999"}',
            ]), self.assertRaisesRegex(ValueError, 'embedded content version'):
                packages.binary_packages(root, binary, '2.0.0-beta.3')
            self.assertEqual(list(root.iterdir()), [binary])

    def test_channel_docs_share_cli_identity_without_cross_channel_installers(self):
        version = '2.0.0-beta.3'
        for channel, command, other_commands in [
            ('npm', 'npm install --global qiongli@2.0.0-beta.3', ['python -m pip', 'cargo install']),
            ('pypi', 'python -m pip install --upgrade "qiongli==2.0.0b3"', ['npm install', 'cargo install']),
            ('cargo', 'cargo install qiongli --version 2.0.0-beta.3 --locked', ['npm install', 'python -m pip']),
        ]:
            with self.subTest(channel=channel):
                readme = packages.package_readme(version, channel)
                self.assertIn(packages.cli_description(), readme)
                self.assertIn(command, readme)
                for other in other_commands:
                    self.assertNotIn(other, readme)
                self.assertIn('qiongli help install plugin', readme)
        with self.assertRaises(ValueError):
            packages.package_readme(version, 'unknown')
        with tempfile.TemporaryDirectory() as temporary:
            out = Path(temporary)
            with self.assertRaisesRegex(ValueError, 'versions must match'):
                packages.stage_cargo(out, '2.0.0-beta.999')
            self.assertEqual(list(out.iterdir()), [])

    def test_install_review_skips_noninteractive_installation(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / 'install.mjs').write_text(packages.NPM_INSTALL_REVIEW)
            (root / 'qiongli.mjs').write_text("throw new Error('must not launch without a terminal');")
            node = subprocess.check_output(['node', '-p', 'process.execPath'], text=True).strip()
            result = subprocess.run([node, str(root / 'install.mjs')], input='', capture_output=True, text=True, timeout=10)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(result.stdout + result.stderr, '')

    def test_cargo_publication_requires_ci_token_without_exposing_it(self):
        workflow = yaml.safe_load((packages.ROOT / '.github/workflows/publish-cargo.yml').read_text())
        publish = workflow['jobs']['publish']
        self.assertEqual(publish['env']['CARGO_REGISTRY_TOKEN'], '${{ secrets.CARGO_REGISTRY_TOKEN }}')
        credentials = next(step for step in publish['steps'] if step.get('id') == 'credentials')
        upload = next(step for step in publish['steps'] if step.get('id') == 'upload')
        self.assertLess(publish['steps'].index(credentials), publish['steps'].index(upload))
        self.assertNotIn('if', upload)
        self.assertIn('cargo publish', upload['run'])
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / 'outputs'
            env = {key: value for key, value in os.environ.items() if key != 'CARGO_REGISTRY_TOKEN'}
            env['GITHUB_OUTPUT'] = str(output)
            for token in (None, '', 'test-only-token-not-a-credential'):
                with self.subTest(token_present=bool(token)):
                    if token is not None:
                        env['CARGO_REGISTRY_TOKEN'] = token
                    result = subprocess.run(['bash', '-e', '-c', credentials['run']],
                                            env=env, capture_output=True, text=True)
                    if token:
                        self.assertEqual(result.returncode, 0, result.stderr)
                        self.assertNotIn(token, result.stdout + result.stderr)
                    else:
                        self.assertNotEqual(result.returncode, 0)
                        self.assertIn('::error::', result.stdout)
                    self.assertFalse(output.exists())

    def test_cargo_staging_normalizes_windows_manifests(self):
        version = tomllib.loads((packages.NATIVE / 'Cargo.toml').read_text())['workspace']['package']['version']
        read = packages.regular_bytes
        def windows_bytes(path):
            data = read(path)
            return data.replace(b'\r\n', b'\n').replace(b'\n', b'\r\n') if path.name == 'Cargo.toml' else data
        with tempfile.TemporaryDirectory() as temporary, patch.object(packages, 'regular_bytes', side_effect=windows_bytes):
            source = packages.stage_cargo(Path(temporary), version)
            for path in source.rglob('Cargo.toml'):
                data = path.read_bytes()
                self.assertNotIn(b'\r', data)
                manifest = tomllib.loads(data.decode())
                if 'package' in manifest:
                    self.assertFalse(manifest['package']['autoexamples'])
                    self.assertFalse(manifest['package']['autotests'])
                    self.assertIn('package-assets/**', manifest['package']['include'])

    def test_three_platform_npm_and_windows_wheel(self):
        binary_data = {
            'aarch64-apple-darwin': bytes.fromhex('cffaedfe0c000001') + bytes(100),
            'x86_64-unknown-linux-gnu': b'\x7fELF\x02\x01' + bytes(12) + b'\x3e\x00' + bytes(80),
            'x86_64-pc-windows-msvc': b'MZ' + bytes(58) + (64).to_bytes(4, 'little') + b'PE\x00\x00\x64\x86',
            'aarch64-unknown-linux-gnu': b'\x7fELF\x02\x01' + bytes(12) + b'\xb7\x00' + bytes(80),
        }
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            binaries = {}
            for target, data in binary_data.items():
                path = root / target
                path.write_bytes(data)
                binaries[target] = path
                packages.validate_binary(data, target)
                with self.assertRaises(ValueError):
                    packages.validate_binary(b'invalid', target)
            tarball = packages.npm_package(root / 'npm-out', binaries, '2.0.0-beta.1')
            with tarfile.open(tarball) as archive:
                import json
                manifest = json.load(archive.extractfile('package/package.json'))
                self.assertEqual(manifest['version'], '2.0.0-beta.1')
                self.assertEqual(manifest['description'], packages.cli_description())
                self.assertEqual(archive.extractfile('package/README.md').read().decode(),
                                 packages.package_readme('2.0.0-beta.1', 'npm'))
                self.assertFalse(any('qiongli_native/' in name or name.endswith('.py')
                                     for name in archive.getnames()))
                self.assertEqual(manifest['publishConfig']['tag'], 'next')
                self.assertEqual(manifest['scripts'], {'postinstall': 'node bin/install.mjs'})
                self.assertEqual(archive.extractfile('package/bin/install.mjs').read(), packages.NPM_INSTALL_REVIEW.encode())
                self.assertEqual(set(manifest['os']), {'darwin', 'linux', 'win32'})
                for target, data in binary_data.items():
                    self.assertEqual(archive.extractfile(f'package/native/{target}/{packages.TARGETS[target][2]}').read(), data)
            whl = packages.wheel(root, '2.0.0b1', 'win_amd64', binary_data['x86_64-pc-windows-msvc'], 'test')
            with zipfile.ZipFile(whl) as archive:
                self.assertIn('qiongli_native/bin/qiongli.exe', archive.namelist())
                self.assertIn(b'Version: 2.0.0b1', archive.read('qiongli-2.0.0b1.dist-info/METADATA'))

    def test_source_assets_and_wheel_record_preserve_the_native_inputs(self):
        version = tomllib.loads((packages.NATIVE / 'Cargo.toml').read_text())['workspace']['package']['version']
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = packages.stage_cargo(root, version)
            manifest = tomllib.loads((source / 'apps/qiongli/Cargo.toml').read_text())
            self.assertEqual(manifest['dependencies']['qiongli-content']['version'], f'={version}')
            self.assertEqual((source / 'apps/qiongli/package-assets/content/workflow/SKILL.md').read_bytes(), (packages.ROOT / 'content/workflow/SKILL.md').read_bytes())
            self.assertTrue((source / 'apps/qiongli/package-assets/qiongli-zotero-companion/manifest.json').is_file())
            self.assertEqual((source / 'apps/qiongli/README.md').read_text(),
                             packages.package_readme(version, 'cargo'))
            wheel = packages.wheel(root, '2.0.0a6', 'macosx_11_0_arm64', b'native-bytes', 'Test package')
            with zipfile.ZipFile(wheel) as archive:
                records = csv.reader(io.StringIO(archive.read('qiongli-2.0.0a6.dist-info/RECORD').decode()))
                for name, digest, size in records:
                    if not digest:
                        self.assertTrue(name.endswith('/RECORD'))
                        continue
                    data = archive.read(name)
                    self.assertEqual(int(size), len(data))
                    self.assertEqual(digest, 'sha256=' + base64.urlsafe_b64encode(hashlib.sha256(data).digest()).rstrip(b'=').decode())
                self.assertEqual(archive.getinfo('qiongli_native/bin/qiongli').external_attr >> 16 & 0o777, 0o755)
                self.assertFalse(any(name.endswith(('.mjs', '.rs')) or 'node_modules' in name
                                     for name in archive.namelist()))
            (root / 'link').symlink_to(wheel)
            with self.assertRaises(ValueError):
                packages.regular_bytes(root / 'link')
            with self.assertRaises(ValueError):
                packages.binary_packages(root, wheel, version)


if __name__ == '__main__':
    unittest.main()
