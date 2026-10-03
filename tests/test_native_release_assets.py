import hashlib
import json
from pathlib import Path
import tempfile
import unittest
import zipfile
from unittest.mock import patch

from tooling.scripts.native_cli_release import archive_cli
from tooling.scripts.native_registry_packages import TARGETS, LEGACY_TARGETS, wheel
from tooling.scripts.native_release_assets import assemble, verify, packet_targets


class NativeReleaseAssetsTests(unittest.TestCase):
    def test_legacy_roster_is_limited_to_immutable_historical_versions(self):
        self.assertEqual(packet_targets({}, '2.1.1'), LEGACY_TARGETS)
        self.assertEqual(packet_targets({'schema_version': 2}, '2.1.2'), TARGETS)
        for version, manifest in [('2.1.2', {}), ('2.2.0-beta.1', {}),
                                  ('2.1.1', {'schema_version': True}),
                                  ('2.1.1', {'schema_version': 3})]:
            with self.subTest(version=version, manifest=manifest), self.assertRaisesRegex(ValueError, 'schema_version 2'):
                packet_targets(manifest, version)

    def test_21_requires_target_native_deepseek_install_evidence(self):
        version, commit = '2.1.0', 'a' * 40
        receipts = [{'version': version, 'source_commit': commit, 'target': target,
                     'checks': {'cli_mcp_tests': 'passed', 'npm_wheel_local_install': 'passed',
                                'archive_smoke': {'content_pack_sha256': 'b' * 64,
                                                  'mcp_tools': {'lite': 14, 'full': 32}}}}
                    for target in TARGETS]
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / 'release-manifest.json').write_text(json.dumps({
                'schema_version': 2, 'version': version, 'source_commit': commit, 'targets': list(TARGETS),
                'target_evidence': receipts,
            }))
            with self.assertRaisesRegex(ValueError, 'target-native DeepSeek'):
                verify(root, version, commit)
            for extension in (0, 1):
                for receipt in receipts:
                    counts = {'lite': 14 + extension, 'full': 32 + extension}
                    receipt['checks']['archive_smoke']['mcp_tools'] = counts
                    receipt['checks']['registry_install'] = {'npm': {'mcp_tools': counts,
                        'deepseek_plugin': {'skills': 22, 'mcp_tools': counts['full'],
                                            'content_pack_sha256': 'b' * 64}}}
                def save():
                    (root / 'release-manifest.json').write_text(json.dumps({
                        'schema_version': 2, 'version': version, 'source_commit': commit, 'targets': list(TARGETS),
                        'target_evidence': receipts,
                    }))
                save()
                # Stop at asset verification: this isolates the receipt gate,
                # without claiming these minimal fixtures are release packages.
                with patch('tooling.scripts.native_release_assets.checked_assets',
                           side_effect=RuntimeError('asset verification reached')):
                    with self.assertRaisesRegex(RuntimeError, 'asset verification reached'):
                        verify(root, version, commit)
                    receipts[0]['checks']['registry_install']['npm']['deepseek_plugin']['mcp_tools'] = 33 - extension
                    save()
                    with self.assertRaisesRegex(ValueError, 'target-native DeepSeek'):
                        verify(root, version, commit)

    def test_assembly_requires_one_source_and_refuses_modified_assets(self):
        version, commit = '2.0.0-alpha.7', 'a' * 40
        fixtures = {
            'aarch64-apple-darwin': (bytes.fromhex('cffaedfe0c000001') + bytes(100), 'macosx_11_0_arm64'),
            'x86_64-unknown-linux-gnu': (b'\x7fELF\x02\x01' + bytes(12) + b'\x3e\x00' + bytes(80), 'manylinux_2_35_x86_64'),
            'x86_64-pc-windows-msvc': (b'MZ' + bytes(58) + (64).to_bytes(4, 'little') + b'PE\x00\x00\x64\x86', 'win_amd64'),
            'aarch64-unknown-linux-gnu': (b'\x7fELF\x02\x01' + bytes(12) + b'\xb7\x00' + bytes(80), 'manylinux_2_35_aarch64'),
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
                                      'npm_wheel_local_install': 'passed', 'archive_smoke': {
                                          'version': version, 'mcp_tools': {'lite': 14, 'full': 32}}},
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
            with patch('tooling.scripts.native_release_assets.NPM_LAUNCHER', 'incomplete target dispatch'):
                with self.assertRaisesRegex(ValueError, 'supported target dispatch'):
                    verify(root / 'assets', version, commit)
            self.assertEqual(set(wheels), set(TARGETS))
            self.assertEqual(len(manifest['artifacts']), 9)
            manifest_path = root / 'assets/release-manifest.json'
            incomplete = json.loads(json.dumps(manifest))
            incomplete['targets'].remove('aarch64-unknown-linux-gnu')
            incomplete['target_evidence'] = [r for r in incomplete['target_evidence']
                                             if r['target'] != 'aarch64-unknown-linux-gnu']
            manifest_path.write_text(json.dumps(incomplete))
            with self.assertRaisesRegex(ValueError, 'all required OS/architecture'):
                verify(root / 'assets', version, commit)
            manifest_path.write_text(json.dumps(manifest))
            # A valid wheel tag must also carry the corresponding executable.
            arm_wheel = wheels['aarch64-unknown-linux-gnu']
            original_wheel = arm_wheel.read_bytes()
            with zipfile.ZipFile(arm_wheel) as archive:
                entries = [(info, archive.read(info.filename)) for info in archive.infolist()]
            with zipfile.ZipFile(arm_wheel, 'w') as archive:
                for info, data in entries:
                    archive.writestr(info, fixtures['x86_64-unknown-linux-gnu'][0]
                                     if info.filename == 'qiongli_native/bin/qiongli' else data)
            changed = json.loads(json.dumps(manifest))
            artifact = next(a for a in changed['artifacts'] if a['file'] == arm_wheel.name)
            artifact.update(sha256=hashlib.sha256(arm_wheel.read_bytes()).hexdigest(), bytes=arm_wheel.stat().st_size)
            manifest_path.write_text(json.dumps(changed))
            with self.assertRaisesRegex(ValueError, 'wheel executable differs'):
                verify(root / 'assets', version, commit)
            arm_wheel.write_bytes(original_wheel)
            manifest_path.write_text(json.dumps(manifest))
            # Existing three-target packets remain readable without adding assets.
            historical = root / 'historical/assets'
            with patch('tooling.scripts.native_release_assets.TARGETS', LEGACY_TARGETS):
                assemble(root / 'targets', historical, version, commit)
            old_manifest = json.loads((historical / 'release-manifest.json').read_text())
            del old_manifest['schema_version']
            (historical / 'release-manifest.json').write_text(json.dumps(old_manifest))
            self.assertEqual(set(verify(historical, version, commit)[2]), set(LEGACY_TARGETS))
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
                self.assertEqual(len(packet['artifacts']), 11)
                verify_plugin.return_value = {'pack_sha256': 'c' * 64}
                with self.assertRaisesRegex(ValueError, 'differs from CLI'):
                    verify(plugin_assets, version, commit)
                verify_plugin.return_value = {'pack_sha256': pack_hash}
                packet['target_evidence'][0]['checks']['archive_smoke']['content_pack_sha256'] = 'c' * 64
                (plugin_assets / 'release-manifest.json').write_text(json.dumps(packet))
                with self.assertRaisesRegex(ValueError, 'all CLI content packs'):
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
