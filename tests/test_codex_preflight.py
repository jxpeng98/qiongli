from pathlib import Path
import json
import os
import sys
import tempfile
import unittest
from unittest.mock import patch

from evals.research_journey import codex_preflight as owner


class CodexPreflightTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(prefix='qiongli-preflight-test-')
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.guidance = self.root / 'public-skill.md'
        self.guidance.write_text('# Public synthetic guidance\n')
        self.executable = self.root / 'fake-codex'
        self.output = self.root / 'output'

    def fake(self, behavior='pass'):
        self.executable.write_text(f'''#!{sys.executable}
import os,sys,time,signal,json
from pathlib import Path
behavior={behavior!r}
assert not any('TOKEN' in k or 'API_KEY' in k for k in os.environ)
assert os.environ['CODEX_HOME'].endswith('/codex-home')
assert not (Path(os.environ['CODEX_HOME'])/'auth.json').exists()
if sys.argv[1]=='--version':
 print('codex-cli synthetic')
 if behavior=='missing-config': (Path(os.environ['CODEX_HOME'])/'config.toml').unlink()
elif '/usr/bin/true' in sys.argv:
 if behavior=='refused': sys.exit(1)
 if behavior=='timeout':
  signal.signal(signal.SIGTERM,signal.SIG_IGN)
  time.sleep(30)
elif '/usr/bin/cat' in sys.argv:
 p=Path(sys.argv[-1])
 if behavior=='drift':p.write_text('synthetic changed public guidance')
 sys.stdout.write('wrong' if behavior=='mismatch' else p.read_text())
else:sys.exit(2)
''')
        self.executable.chmod(0o755)

    def test_real_fake_process_success_uses_isolated_environment_and_full_file_bytes(self):
        self.fake()
        with patch.dict(os.environ, {'TEST_TOKEN': 'synthetic', 'OPENAI_API_KEY': 'synthetic',
                                    'CODEX_HOME': '/do-not-use-real-home'}):
            result = owner.check(self.executable, [self.guidance], self.output)
        self.assertEqual(result['status'], 'passed')
        self.assertEqual([r['name'] for r in result['checks']], ['version', 'sandbox', 'guidance-0'])
        self.assertTrue(result['authentication_path_absent'])
        self.assertEqual(result['model_calls'], 0)
        self.assertEqual(result['effective_plugin_policy'], 'not-checked')
        self.assertEqual((self.output / 'guidance-0/events.jsonl').read_bytes(), self.guidance.read_bytes())
        self.assertEqual(json.loads((self.output / 'preflight.json').read_text()), result)

    def test_true_failure_stops_before_any_guidance_read(self):
        self.fake('refused')
        result = owner.check(self.executable, [self.guidance], self.output)
        self.assertEqual(result['status'], 'blocked')
        self.assertEqual(result['blocker'], 'sandbox')
        self.assertFalse((self.output / 'guidance-0').exists())

    def test_mismatching_cat_or_source_drift_cannot_pass(self):
        for behavior in ('mismatch', 'drift'):
            with self.subTest(behavior=behavior):
                self.guidance.write_text('# Public synthetic guidance\n')
                self.fake(behavior)
                result = owner.check(self.executable, [self.guidance], self.root / behavior)
                self.assertEqual(result['status'], 'blocked')
                if behavior == 'drift':
                    self.assertFalse(result['guidance_unchanged'])

    def test_timeout_kills_owned_process_group_and_stops_reads(self):
        self.fake('timeout')
        result = owner.check(self.executable, [self.guidance], self.output, timeout_seconds=1)
        self.assertEqual(result['status'], 'blocked')
        check = result['checks'][-1]
        self.assertEqual(check['termination_reason'], 'timeout')
        self.assertTrue(check['process_cleanup']['reaped'])
        self.assertTrue(check['process_cleanup']['group_gone'])
        self.assertFalse((self.output / 'guidance-0').exists())

    def test_spawn_error_is_recorded_as_blocked(self):
        self.executable.write_bytes(b'not an executable format')
        self.executable.chmod(0o755)
        result = owner.check(self.executable, [self.guidance], self.output)
        self.assertEqual(result['status'], 'blocked')
        self.assertIsNotNone(result['checks'][0]['driver_error'])
        self.assertIsNone(result['checks'][0]['process_cleanup'])

    def test_existing_output_is_not_overwritten(self):
        self.fake()
        self.output.mkdir()
        sentinel = self.output / 'retain'
        sentinel.write_bytes(b'keep')
        with self.assertRaises(ValueError):
            owner.check(self.executable, [self.guidance], self.output)
        self.assertEqual(sentinel.read_bytes(), b'keep')

    def test_invalid_timeout_or_public_file_refuses_before_start(self):
        self.fake()
        for timeout in (False, 0, 31, 1.5):
            with self.subTest(timeout=timeout), self.assertRaises(ValueError):
                owner.check(self.executable, [self.guidance], self.output, timeout)
            self.assertFalse(self.output.exists())
        for content in (b'', b'\xff', b'x' * 512001):
            self.guidance.write_bytes(content)
            with self.assertRaises((ValueError, UnicodeDecodeError)):
                owner.check(self.executable, [self.guidance], self.output)
            self.assertFalse(self.output.exists())
        link = self.root / 'linked.md'
        link.symlink_to(self.guidance)
        with self.assertRaises(ValueError):
            owner.check(self.executable, [link], self.output)
        self.assertFalse(self.output.exists())

    def test_missing_generated_config_is_blocked_and_receipt_is_preserved(self):
        self.fake('missing-config')
        result = owner.check(self.executable, [self.guidance], self.output)
        self.assertEqual(result['status'], 'blocked')
        self.assertFalse(result['config_unchanged'])
        self.assertEqual(result['blocker'], 'profile-or-guidance-changed')
        self.assertEqual(json.loads((self.output / 'preflight.json').read_text()), result)

    def test_sandbox_profile_grants_only_exact_runtime_and_public_files(self):
        import tomllib
        self.fake()
        result = owner.check(self.executable, [self.guidance], self.output)
        self.assertEqual(result['status'], 'passed')
        config = tomllib.loads((self.output / 'codex-home/config.toml').read_text())
        profile = config['permissions'][owner.PROFILE]
        self.assertEqual(profile['filesystem'], {
            ':minimal': 'read', str(self.executable.resolve()): 'read',
            str(self.guidance.resolve()): 'read'
        })
        self.assertEqual(profile['network'], {'enabled': False})
        self.assertEqual(config['approval_policy'], 'never')
