import hashlib
import json
from pathlib import Path
import tempfile
import subprocess
import copy
import unittest
from unittest.mock import patch

from tooling.scripts.native_cli_release import archive_cli
from tooling.scripts.native_registry_packages import TARGETS, wheel
from tooling.scripts.native_release_assets import assemble, verify
from tooling.scripts import native_release_assets as assets


class NativeReleaseAssetsTests(unittest.TestCase):
    def local_fixture(self):
        check = {'version': '2.1.0-beta.1', 'content_pack_sha256': 'b' * 64,
                 'invalid_command_rejected': True, 'mcp_tools': {'lite': 14, 'full': 32}}
        plugins = {'codex': {'status': 'passed'}, 'claude': {'status': 'passed'}}
        manifest = {'version': check['version'], 'source_commit': 'a' * 40, 'artifacts': [],
                    'target_evidence': [{'target': 'aarch64-apple-darwin',
                        'checks': {'archive_smoke': check, 'marketplace_plugins': plugins}}]}
        manifest['local_macos'] = {'status': 'passed', 'execution': 'manual-local',
            'target': 'aarch64-apple-darwin', 'source_commit': manifest['source_commit'],
            'version': manifest['version'], 'packet_sha256': assets.packet_digest(manifest),
            'cargo_dry_run': 'passed', 'marketplace_plugins': plugins,
            'checks': {name: dict(check) for name in ('npm', 'pypi', 'cargo_archives', 'cargo_alias')}}
        return manifest

    def test_local_evidence_rejects_missing_stale_and_incomplete_observations(self):
        manifest = self.local_fixture()
        assets.require_local_macos(manifest)
        changes = [lambda m: m.pop('local_macos'),
                   lambda m: m.update(source_commit='c' * 40),
                   lambda m: m['artifacts'].append({'file': 'changed'}),
                   lambda m: m['local_macos'].update(execution='github-actions'),
                   lambda m: m['local_macos'].update(target='x86_64-unknown-linux-gnu'),
                   lambda m: m['local_macos'].pop('cargo_dry_run'),
                   lambda m: m['local_macos']['checks'].pop('cargo_alias'),
                   lambda m: m['local_macos']['checks']['npm'].update(invalid_command_rejected=False),
                   lambda m: m['local_macos']['checks']['pypi'].update(content_pack_sha256='c' * 64),
                   lambda m: m['local_macos'].update(marketplace_plugins={})]
        for change in changes:
            bad = copy.deepcopy(manifest)
            change(bad)
            with self.subTest(change=change), self.assertRaises(ValueError):
                assets.require_local_macos(bad)

    def test_ci_requires_both_targets_on_the_exact_packet_not_only_a_green_build(self):
        manifest = self.local_fixture()
        commit = manifest['source_commit']
        run = {'id': 42, 'head_sha': commit, 'conclusion': 'success',
               'event': 'workflow_dispatch', 'head_branch': 'v' + manifest['version']}
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / 'release-manifest.json').write_text(json.dumps(manifest))
            digest = hashlib.sha256((root / 'release-manifest.json').read_bytes()).hexdigest()
            for failure in (None, 'build-only', 'stale-packet', 'missing-windows', 'wrong-target', 'wrong-source'):
                def download(command, **kwargs):
                    directory = Path(command[-1])
                    for os_name, target in [('ubuntu-22.04', 'x86_64-unknown-linux-gnu'), ('windows-2022', 'x86_64-pc-windows-msvc')]:
                        if failure == 'build-only' or failure == 'missing-windows' and os_name == 'windows-2022':
                            continue
                        folder = directory / f'install-{os_name}'
                        folder.mkdir()
                        receipt = {'status': 'passed', 'version': manifest['version'],
                            'source_commit': 'c' * 40 if failure == 'wrong-source' else commit,
                            'target': 'aarch64-apple-darwin' if failure == 'wrong-target' else target,
                            'manifest_sha256': 'c' * 64 if failure == 'stale-packet' else digest}
                        (folder / 'install-check.json').write_text(json.dumps(receipt))
                    return subprocess.CompletedProcess(command, 0)
                with patch.dict(assets.os.environ, {'GITHUB_REPOSITORY': 'owner/repo'}), \
                     patch.object(assets.subprocess, 'check_output', return_value=json.dumps({'workflow_runs': [run]})), \
                     patch.object(assets.subprocess, 'run', side_effect=download):
                    if failure:
                        with self.subTest(failure=failure), self.assertRaisesRegex(ValueError, 'exact packet'):
                            assets.require_remote_installs(root, manifest, commit)
                    else:
                        assets.require_remote_installs(root, manifest, commit)

    def test_local_qualification_writes_receipt_only_after_real_owners_succeed(self):
        for failure in (None, 'cargo', 'changed-source'):
            with self.subTest(failure=failure), tempfile.TemporaryDirectory() as temporary:
                root, out = Path(temporary) / 'assets', Path(temporary) / 'checks'
                root.mkdir()
                manifest = self.local_fixture()
                observed = manifest.pop('local_macos')
                before = json.dumps(manifest).encode()
                (root / 'release-manifest.json').write_bytes(before)
                def command(argv, **kwargs):
                    if argv[0] == 'cargo':
                        self.assertIn('--dry-run', argv)
                        if failure == 'cargo':
                            raise subprocess.CalledProcessError(1, argv)
                    if str(argv[1]).endswith('native_registry_install_check.py'):
                        folder = Path(argv[argv.index('--out-dir') + 1])
                        folder.mkdir(parents=True)
                        cargo = '--cargo-only' in argv
                        names = ('cargo_archives', 'cargo_alias') if cargo else ('npm', 'pypi')
                        (folder / 'install-check.json').write_text(json.dumps({'checks': {
                            name: observed['checks'][name] for name in names}}))
                with patch.object(assets.platform, 'system', return_value='Darwin'), \
                     patch.object(assets.platform, 'machine', return_value='arm64'), \
                     patch.dict(assets.os.environ, {}, clear=True), \
                     patch.object(assets, 'check_source', side_effect=[None, ValueError('changed-source')] if failure == 'changed-source' else None), \
                     patch.object(assets, 'verify', return_value=(manifest, root / 'npm.tgz', {'aarch64-apple-darwin': root / 'mac.whl'})), \
                     patch.object(assets, 'check_plugins', return_value=observed['marketplace_plugins']), \
                     patch.object(assets.subprocess, 'run', side_effect=command):
                    if failure:
                        with self.assertRaises((ValueError, subprocess.CalledProcessError)):
                            assets.install_packet(root, out, manifest['version'], manifest['source_commit'], local_macos=True)
                        self.assertEqual((root / 'release-manifest.json').read_bytes(), before)
                        self.assertFalse((out / 'install-check.json').exists())
                    else:
                        assets.install_packet(root, out, manifest['version'], manifest['source_commit'], local_macos=True)
                        assets.require_local_macos(json.loads((root / 'release-manifest.json').read_text()))
                        self.assertTrue((out / 'install-check.json').is_file())
        for system, machine, env in [('Linux', 'x86_64', {}), ('Darwin', 'arm64', {'GITHUB_ACTIONS': 'true'})]:
            with patch.object(assets.platform, 'system', return_value=system), \
                 patch.object(assets.platform, 'machine', return_value=machine), \
                 patch.dict(assets.os.environ, env, clear=True), patch.object(assets, 'check_source') as check:
                with self.assertRaisesRegex(ValueError, 'manually'):
                    assets.install_packet(Path('.'), Path('unused'), '2.0.1', 'a' * 40, local_macos=True)
                check.assert_not_called()

    def test_assembly_requires_one_source_and_refuses_modified_assets(self):
        version, commit = '2.0.0-alpha.7', 'a' * 40
        fixtures = {
            'aarch64-apple-darwin': (bytes.fromhex('cffaedfe0c000001') + bytes(100), 'macosx_11_0_arm64'),
            'x86_64-unknown-linux-gnu': (b'\x7fELF\x02\x01' + bytes(12) + b'\x3e\x00' + bytes(80), 'manylinux_2_35_x86_64'),
            'x86_64-pc-windows-msvc': (b'MZ' + bytes(58) + (64).to_bytes(4, 'little') + b'PE\x00\x00\x64\x86', 'win_amd64'),
        }
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            for target, (data, tag) in fixtures.items():
                folder = root / 'targets' / target
                folder.mkdir(parents=True)
                binary = root / target
                binary.write_bytes(data)
                extension = 'zip' if target.endswith('msvc') else 'tar.gz'
                archive = folder / f'qiongli-{version}-{target}.{extension}'
                archive_cli(archive, binary, b'fixture', target)
                whl = wheel(folder, '2.0.0a7', tag, data, 'fixture')
                receipt = {'version': version, 'source_commit': commit, 'target': target,
                           'checks': {'cli_mcp_tests': 'passed', 'cli_clippy': 'passed',
                                      'npm_wheel_local_install': 'passed', 'archive_smoke': {'version': version}},
                           'artifacts': [{'file': p.name, 'sha256': hashlib.sha256(p.read_bytes()).hexdigest(),
                                          'bytes': p.stat().st_size} for p in (archive, whl)]}
                (folder / 'release-manifest.json').write_text(json.dumps(receipt))
            with self.assertRaisesRegex(ValueError, 'mixed version or source'):
                assemble(root / 'targets', root / 'wrong-source', version, 'b' * 40)
            assemble(root / 'targets', root / 'assets', version, commit)
            manifest, npm, wheels = verify(root / 'assets', version, commit)
            with patch('tooling.scripts.native_release_assets.NPM_INSTALL_REVIEW', 'unexpected script'):
                with self.assertRaisesRegex(ValueError, 'installation review bytes'):
                    verify(root / 'assets', version, commit)
            self.assertEqual(set(wheels), set(TARGETS))
            self.assertEqual(len(manifest['artifacts']), 7)
            # The Plugin verifier owns archive internals; this owner must bind
            # both Plugin archives to every native executable's observed pack.
            pack_hash = 'b' * 64
            for target in fixtures:
                folder = root / 'targets' / target
                receipt = json.loads((folder / 'release-manifest.json').read_text())
                receipt['checks']['archive_smoke']['content_pack_sha256'] = pack_hash
                if target.endswith('linux-gnu'):
                    for host in ('codex', 'claude'):
                        plugin = folder / f'qiongli-next-{host}-plugin-v{version}.tar.gz'
                        plugin.write_bytes(host.encode())
                        receipt['artifacts'].append({'file': plugin.name,
                            'sha256': hashlib.sha256(plugin.read_bytes()).hexdigest(),
                            'bytes': plugin.stat().st_size})
                (folder / 'release-manifest.json').write_text(json.dumps(receipt))
            plugin_assets = root / 'with-plugins/assets'
            with patch('tooling.scripts.native_release_assets.verify_archive',
                       return_value={'pack_sha256': pack_hash}) as verify_plugin:
                assemble(root / 'targets', plugin_assets, version, commit)
                packet, _, _ = verify(plugin_assets, version, commit)
                self.assertEqual(len(packet['artifacts']), 9)
                verify_plugin.return_value = {'pack_sha256': 'c' * 64}
                with self.assertRaisesRegex(ValueError, 'differs from CLI'):
                    verify(plugin_assets, version, commit)
                verify_plugin.return_value = {'pack_sha256': pack_hash}
                packet['target_evidence'][0]['checks']['archive_smoke']['content_pack_sha256'] = 'c' * 64
                (plugin_assets / 'release-manifest.json').write_text(json.dumps(packet))
                with self.assertRaisesRegex(ValueError, 'all three CLI content packs'):
                    verify(plugin_assets, version, commit)
                packet['artifacts'] = [a for a in packet['artifacts'] if '-claude-plugin-' not in a['file']]
                (plugin_assets / 'release-manifest.json').write_text(json.dumps(packet))
                with self.assertRaisesRegex(ValueError, 'both marketplace Plugin archives'):
                    verify(plugin_assets, version, commit)
            original = npm.read_bytes()
            npm.write_bytes(original + b'tamper')
            with self.assertRaisesRegex(ValueError, 'digest/size mismatch'):
                verify(root / 'assets', version, commit)
            npm.write_bytes(original)
            manifest['target_evidence'][1] = manifest['target_evidence'][0]
            (root / 'assets/release-manifest.json').write_text(json.dumps(manifest))
            with self.assertRaisesRegex(ValueError, 'duplicate or missing'):
                verify(root / 'assets', version, commit)


if __name__ == '__main__':
    unittest.main()
