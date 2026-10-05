from __future__ import annotations

import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


class NativeRegistryUpgradeCheckTests(unittest.TestCase):
    def test_invalid_packets_fail_before_creating_installation_or_download_roots(self):
        script = Path(__file__).resolve().parents[1] / 'tooling/scripts/native_registry_upgrade_check.py'
        for tampered in (False, True):
            with self.subTest(tampered=tampered), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                package = root / 'candidate.whl'
                package.write_bytes(b'changed bytes')
                receipt = {'version': '2.2.0', 'artifacts': []}
                if tampered:
                    receipt['artifacts'] = [{'file': package.name, 'sha256': 'f' * 64}]
                (root / 'registry-packages.json').write_text(json.dumps(receipt))
                output = root / 'must-not-be-created'
                result = subprocess.run([sys.executable, str(script), '--packages', str(root),
                                         '--out-dir', str(output)], capture_output=True, text=True)
                self.assertNotEqual(result.returncode, 0)
                self.assertFalse(output.exists())
                self.assertIn('integrity mismatch' if tampered else 'requires target-selected', result.stderr)


class ExplicitPredecessorUpgradeTests(unittest.TestCase):
    def setUp(self):
        from tooling.scripts import native_registry_upgrade_check
        self.owner = native_registry_upgrade_check
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.packages = self.root / 'packages'
        self.packages.mkdir()
        self.output = self.root / 'upgrade'
        self.calls = []
        self.failure = None
        self.wrong_version = None
        self.drift = False
        self.receipt = {'version': '2.4.0', 'target': 'aarch64-unknown-linux-gnu', 'artifacts': []}
        self.add_candidate('candidate.whl', b'synthetic wheel')
        self.add_candidate('candidate.tgz', b'synthetic npm package')
        self.save_receipt()

    def add_candidate(self, name, content):
        import hashlib
        (self.packages / name).write_bytes(content)
        self.receipt['artifacts'].append({'file': name, 'sha256': hashlib.sha256(content).hexdigest()})

    def save_receipt(self):
        (self.packages / 'registry-packages.json').write_text(json.dumps(self.receipt))

    def execute(self, predecessor='2.3.0'):
        from unittest.mock import patch
        selected = self.owner.predecessor_versions(self.receipt['target'], predecessor)

        def fake_run(argv, *, root, env, **kwargs):
            argv = list(map(str, argv))
            self.calls.append(argv)
            self.assertEqual(env['HOME'], str(self.output / 'home'))
            if 'download' in argv:
                if self.failure == 'pip':
                    raise subprocess.CalledProcessError(1, argv, stderr='synthetic predecessor unavailable')
                if self.failure == 'timeout':
                    raise subprocess.TimeoutExpired(argv, 1)
                destination = Path(argv[argv.index('--dest') + 1])
                (destination / 'prior.whl').write_bytes(b'public synthetic predecessor wheel')
            if 'pack' in argv:
                if self.failure == 'npm':
                    raise subprocess.CalledProcessError(1, argv, stderr='synthetic predecessor unavailable')
                destination = Path(argv[argv.index('--pack-destination') + 1])
                (destination / 'prior.tgz').write_bytes(b'public synthetic predecessor npm')
            if 'install' in argv and '--prefix' in argv:
                prefix = Path(argv[argv.index('--prefix') + 1])
                installed = prefix / ('node_modules/qiongli' if sys.platform == 'win32' else 'lib/node_modules/qiongli')
                installed.mkdir(parents=True, exist_ok=True)
                version = self.wrong_version if self.wrong_version == 'npm-wrong' else selected['npm']
                (installed / 'package.json').write_text(json.dumps({'version': version}))
            if '-c' in argv and 'importlib.metadata' in argv[argv.index('-c') + 1]:
                return subprocess.CompletedProcess(argv, 0, stdout=selected['pypi'] if self.wrong_version == 'npm-wrong' else (self.wrong_version or selected['pypi']))
            return subprocess.CompletedProcess(argv, 0, stdout='')

        def fake_check(*args, **kwargs):
            if self.drift:
                (self.output / 'home/research/paper-notes.md').write_text('synthetic changed canary')
            return {'status': 'passed', 'version': kwargs['version']}

        with patch.object(self.owner, 'run', side_effect=fake_run), patch.object(
            self.owner, 'check_cli', side_effect=fake_check
        ), patch.object(self.owner.subprocess, 'check_output', return_value='/synthetic/node\n'), patch.object(
            self.owner, 'host_target', return_value=self.receipt['target']
        ):
            return self.owner.upgrade(self.packages, self.output, predecessor_version=predecessor)

    def test_explicit_predecessor_executes_exact_versions_and_retains_fixture_bytes(self):
        import hashlib
        result = self.execute()
        self.assertEqual(result['status'], 'passed')
        self.assertEqual(result['selected_predecessors'], {'pypi': '2.3.0', 'npm': '2.3.0'})
        self.assertEqual(result['predecessor_selection'], 'explicit')
        self.assertEqual(result['target'], self.receipt['target'])
        self.assertTrue(any('qiongli==2.3.0' in call for call in self.calls))
        self.assertTrue(any('qiongli@2.3.0' in call for call in self.calls))
        self.assertFalse(any('qiongli==1.17.0' in call or 'qiongli@2.1.1' in call for call in self.calls))
        self.assertEqual({p['ecosystem'] for p in result['predecessors']}, {'pypi', 'npm'})
        for predecessor in result['predecessors']:
            self.assertEqual(predecessor['version'], '2.3.0')
            self.assertEqual(predecessor['sha256'], hashlib.sha256(
                (self.output / 'predecessors' / predecessor['file']).read_bytes()
            ).hexdigest())
        for name, digest in result['retained'].items():
            self.assertEqual(hashlib.sha256((self.output / 'home' / name).read_bytes()).hexdigest(), digest)
        self.assertEqual(json.loads((self.output / 'registry-upgrade-check.json').read_text()), result)

    def test_historical_defaults_preserve_target_specific_predecessors(self):
        self.assertEqual(self.owner.predecessor_versions('aarch64-unknown-linux-gnu', None),
                         {'pypi': '1.17.0', 'npm': '2.1.1'})
        self.assertEqual(self.owner.predecessor_versions('x86_64-unknown-linux-gnu', None),
                         {'pypi': '2.1.1', 'npm': '2.1.1'})
        result = self.execute(None)
        self.assertEqual(result['predecessor_selection'], 'historical-default')
        self.assertTrue(any('qiongli==1.17.0' in call for call in self.calls))

    def test_prerelease_normalization_uses_release_version_owner(self):
        self.assertEqual(self.owner.predecessor_versions(None, 'v2.3.0'),
                         {'pypi': '2.3.0', 'npm': '2.3.0'})
        self.assertEqual(self.owner.predecessor_versions(None, '2.4.0-beta.1'),
                         {'pypi': '2.4.0b1', 'npm': '2.4.0-beta.1'})

    def test_invalid_predecessor_refuses_before_output_creation(self):
        for version in ('garbage', '../2.3.0', ''):
            with self.subTest(version=version), self.assertRaises(ValueError):
                self.owner.upgrade(self.packages, self.output, predecessor_version=version)
            self.assertFalse(self.output.exists())

    def test_missing_tampered_linked_escape_and_duplicate_candidates_refuse_before_output(self):
        import hashlib
        original = json.loads(json.dumps(self.receipt))
        for mutation in ('missing', 'tampered', 'symlink', 'escape', 'duplicate-wheel', 'duplicate-npm'):
            with self.subTest(mutation=mutation):
                self.receipt = json.loads(json.dumps(original))
                wheel = self.packages / 'candidate.whl'
                if wheel.exists() or wheel.is_symlink():
                    wheel.unlink()
                wheel.write_bytes(b'synthetic wheel')
                if mutation == 'missing':
                    wheel.unlink()
                elif mutation == 'tampered':
                    wheel.write_bytes(b'tampered')
                elif mutation == 'symlink':
                    external = self.root / 'external.whl'
                    external.write_bytes(b'synthetic wheel')
                    wheel.unlink()
                    wheel.symlink_to(external)
                elif mutation == 'escape':
                    external = self.root / 'escape.whl'
                    external.write_bytes(b'synthetic wheel')
                    self.receipt['artifacts'][0]['file'] = '../escape.whl'
                else:
                    self.add_candidate('second.whl' if mutation == 'duplicate-wheel' else 'second.tgz', b'second')
                self.save_receipt()
                with self.assertRaises((ValueError, FileNotFoundError)):
                    self.owner.upgrade(self.packages, self.output, predecessor_version='2.3.0')
                self.assertFalse(self.output.exists())

    def test_unavailable_pip_and_npm_preserve_selected_version_without_fallback(self):
        for failure, ecosystem in (('pip', 'pypi'), ('npm', 'npm'), ('timeout', 'pypi')):
            with self.subTest(failure=failure):
                self.output = self.root / ('upgrade-' + failure)
                self.failure = failure
                self.calls = []
                result = self.execute()
                self.assertEqual(result['status'], 'unqualified')
                self.assertEqual(result['reason'], 'predecessor-unavailable')
                self.assertEqual(result['ecosystem'], ecosystem)
                self.assertEqual(result['selected_predecessors'], {'pypi': '2.3.0', 'npm': '2.3.0'})
                self.assertFalse(any('qiongli==1.17.0' in call or 'qiongli@2.1.1' in call for call in self.calls))

    def test_wrong_pip_and_npm_predecessor_installations_are_failed(self):
        for mismatch in ('9.9.9', 'npm-wrong'):
            with self.subTest(mismatch=mismatch):
                self.output = self.root / ('wrong-' + mismatch)
                self.wrong_version = mismatch
                result = self.execute()
                self.assertEqual(result['status'], 'failed')
                self.assertEqual(result['reason'], 'upgrade-check-failed')

    def test_retained_research_data_drift_is_failed(self):
        self.drift = True
        result = self.execute()
        self.assertEqual(result['status'], 'failed')
        self.assertEqual(result['reason'], 'upgrade-check-failed')

    def test_cli_exits_nonzero_for_failed_or_unqualified_receipts(self):
        import contextlib
        import io
        from unittest.mock import patch
        for status in ('failed', 'unqualified'):
            with self.subTest(status=status), patch.object(
                self.owner, 'upgrade', return_value={'status': status}
            ), patch.object(sys, 'argv', [
                'upgrade-check', '--packages', str(self.packages),
                '--out-dir', str(self.output), '--predecessor-version', '2.3.0'
            ]), contextlib.redirect_stdout(io.StringIO()), self.assertRaises(SystemExit) as raised:
                self.owner.main()
            self.assertEqual(raised.exception.code, 1)

    def test_formal_receipt_with_multiple_cargo_archives_keeps_binary_selection(self):
        self.add_candidate('qiongli-project-2.4.0.crate', b'synthetic project source archive')
        self.add_candidate('qiongli-runtime-2.4.0.crate', b'synthetic runtime source archive')
        self.save_receipt()
        result = self.execute()
        self.assertEqual(result['status'], 'passed')
        self.assertEqual(result['candidate_packages'], self.receipt['artifacts'])
