import hashlib
import unittest
import contextlib
import io
import tomllib
import json
import subprocess
import tempfile
from itertools import product
from pathlib import Path
from unittest.mock import patch

import yaml

from tooling.scripts import native_release_publish as publish


class NativeReleasePublishTests(unittest.TestCase):
    def test_publication_waits_for_ci_and_verified_assets(self):
        outcomes = ('ci-failure', 'asset-failure', 'wrong-main', 'published', 'extra-asset',
                    'draft-change', 'main-change', 'tag-change', 'bad-checksums', 'success')
        for version, outcome in product(('2.0.1', '2.1.0-beta.1'), outcomes):
            commit, tag = 'a' * 40, f'v{version}'
            stable = not publish.parse_release_version(tag).is_prerelease
            if outcome in ('wrong-main', 'main-change') and not stable:
                continue
            with self.subTest(version=version, outcome=outcome), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary) / 'source'
                native = root / 'packages/qiongli-native'
                native.mkdir(parents=True)
                (native / 'Cargo.toml').write_text(f'[workspace.package]\nversion = "{version}"\n')
                notes = root / 'tooling/release'
                notes.mkdir(parents=True)
                (notes / f'{tag}.md').write_text('Reviewed fixture notes')
                calls, dispatched = [], False
                def command(*args):
                    nonlocal dispatched
                    args = tuple(map(str, args))
                    calls.append(args)
                    if args[:2] == ('git', 'rev-parse'):
                        return commit
                    if args[:3] == ('gh', 'workflow', 'run') and args[3] == 'native-cli-distribution.yml':
                        dispatched = True
                    if args[:2] == ('gh', 'api'):
                        if '/commits/' in args[2]:
                            wrong = (args[2].endswith('/main') and (outcome == 'wrong-main' or (dispatched and outcome == 'main-change'))
                                     or dispatched and outcome == 'tag-change' and args[2].endswith('/' + tag))
                            return json.dumps({'sha': 'b' * 40 if wrong else commit})
                        if '/actions/workflows/' in args[2]:
                            return json.dumps({'workflow_runs': [dict(id=4, head_sha=commit, head_branch=tag,
                                event='workflow_dispatch', status='completed', html_url='https://example.invalid/run',
                                conclusion='failure' if outcome == 'ci-failure' else 'success')] if dispatched else []})
                    if args[:3] == ('gh', 'release', 'view'):
                        return json.dumps({'isDraft': outcome != 'published', 'tagName': tag})
                    if args[:3] == ('gh', 'release', 'download'):
                        assets = Path(args[args.index('--dir') + 1])
                        assets.mkdir()
                        (assets / 'verified-asset').write_text('fixture')
                        manifest = {'artifacts': [{'file': 'verified-asset'}]}
                        if outcome == 'draft-change' and dispatched:
                            manifest['changed'] = True
                        (assets / 'release-manifest.json').write_text(json.dumps(manifest))
                        sums = ''.join(f'{hashlib.sha256((assets / name).read_bytes()).hexdigest()}  {name}\n'
                                       for name in ('release-manifest.json', 'verified-asset'))
                        (assets / 'SHA256SUMS').write_text('wrong' if outcome == 'bad-checksums' else sums)
                        if outcome == 'extra-asset':
                            (assets / 'unverified-plugin.tar.gz').write_text('unverified')
                    if len(args) > 1 and args[1].endswith('native_release_assets.py') and outcome == 'asset-failure':
                        raise subprocess.CalledProcessError(1, args)
                    return ''
                env = dict(GITHUB_ACTIONS='true', GITHUB_SHA=commit, GITHUB_REF=f'refs/tags/{tag}',
                           GITHUB_REPOSITORY='owner/repo', RUNNER_TEMP=temporary)
                with patch.dict(publish.os.environ, env, clear=True), patch.object(publish, 'ROOT', root), patch.object(publish, 'run', side_effect=command), patch('sys.argv', ['publish', '--tag', tag]), contextlib.redirect_stdout(io.StringIO()):
                    if outcome == 'success':
                        publish.main()
                    else:
                        with self.assertRaises((ValueError, RuntimeError, subprocess.CalledProcessError)):
                            publish.main()
                mutations = [args for args in calls if args[:2] == ('gh', 'release') and args[2] in ('create', 'edit')]
                if outcome == 'success':
                    self.assertEqual([args[2] for args in mutations], ['edit'])
                    self.assertIn('--draft=false', mutations[0])
                    self.assertIn('--prerelease=false' if stable else '--prerelease', mutations[0])
                    self.assertEqual('--latest' in mutations[0], stable)
                    verification = next(args for args in calls if '--require-ci' in args)
                    self.assertLess(calls.index(verification), calls.index(mutations[0]))
                    self.assertTrue(any('verify_release=true' in args for args in calls))
                    dispatches = [args for args in calls if 'publish_release=true' in args]
                    self.assertEqual(len(dispatches), 3)
                    self.assertTrue(all(calls.index(args) > calls.index(mutations[0]) for args in dispatches))
                else:
                    self.assertEqual(mutations, [])
                    self.assertFalse(any('publish_release=true' in args for args in calls))
                    if outcome in ('wrong-main', 'published', 'extra-asset', 'asset-failure', 'bad-checksums'):
                        self.assertFalse(dispatched)

    def test_run_selection_binds_source_tag_and_dispatch(self):
        good = dict(id=5, head_sha='source', head_branch='v2.0.0-beta.2', event='workflow_dispatch')
        unrelated = [good | {'id': 8, 'head_sha': 'other'},
                     good | {'id': 9, 'head_branch': '2.x'},
                     good | {'id': 10, 'event': 'push'}]
        self.assertIsNone(publish.select_run(unrelated, 'source', 'v2.0.0-beta.2'))
        self.assertEqual(publish.select_run(unrelated + [good], 'source', 'v2.0.0-beta.2'), good)
        self.assertIsNone(publish.select_run([good], 'source', 'v2.0.0-beta.2', after_id=5))

    def test_local_invocation_cannot_publish(self):
        with patch.dict(publish.os.environ, {}, clear=True), patch.object(publish, 'run', return_value='source') as run:
            with patch('sys.argv', ['publish', '--tag', 'v2.0.0-beta.2']), self.assertRaises(SystemExit):
                publish.main()
            self.assertEqual(run.call_count, 1)
            self.assertEqual(run.call_args.args, ('git', 'rev-parse', 'HEAD'))

    def test_dispatch_preserves_registry_gates(self):
        root = Path(__file__).resolve().parents[1]
        for name in ('publish-npm.yml', 'publish-pypi.yml', 'publish-cargo.yml'):
            workflow = yaml.safe_load((root / '.github/workflows' / name).read_text())
            events = workflow.get('on', workflow.get(True))
            self.assertFalse(events['workflow_dispatch']['inputs']['publish_release']['default'])
            job = workflow['jobs'].get('publish-native', workflow['jobs'].get('publish'))
            self.assertIn("github.ref_type == 'tag'", job['if'])
            self.assertIn('inputs.publish_release', job['if'])
            self.assertTrue(any('--require-ci' in step.get('run', '') for step in job['steps']))
            self.assertIn('environment', job)
        npm = (root / '.github/workflows/publish-npm.yml').read_text()
        self.assertIn('"$channel" == next || "$channel" == latest', npm)
        self.assertIn('--tag "$channel" --provenance', npm)

    def test_distribution_keeps_two_hosted_targets_and_a_local_packet_gate(self):
        root = Path(__file__).resolve().parents[1]
        for path in (root / '.github/workflows').glob('*.yml'):
            jobs = yaml.safe_load(path.read_text())['jobs']
            for name, job in jobs.items():
                with self.subTest(workflow=path.name, job=name):
                    self.assertNotIn('macos', str(job.get('runs-on', '')).lower())
                    self.assertNotIn('macos', str(job.get('strategy', {}).get('matrix', {})).lower())
        workflow = yaml.safe_load((root / '.github/workflows/native-cli-distribution.yml').read_text())
        jobs = workflow['jobs']
        self.assertEqual({r['target'] for r in jobs['build']['strategy']['matrix']['include']},
                         {'x86_64-unknown-linux-gnu', 'x86_64-pc-windows-msvc'})
        self.assertEqual(jobs['install']['strategy']['matrix']['os'], ['ubuntu-22.04', 'windows-2022'])
        self.assertIn('verify_release', jobs['install']['if'])
        self.assertIn("github.ref_type == 'tag'", jobs['install']['if'])
        self.assertEqual(jobs['install']['needs'], 'candidate')
        candidate = jobs['candidate']
        self.assertEqual(candidate['permissions'], {'contents': 'write'})
        self.assertFalse(any('checkout' in step.get('uses', '') for step in candidate['steps']))
        self.assertEqual(workflow['permissions'], {'contents': 'read'})
        build_commands = '\n'.join(s.get('run', '') for s in jobs['build']['steps'])
        install_commands = '\n'.join(s.get('run', '') for s in jobs['install']['steps'])
        self.assertIn('tests.test_native_release_assets', build_commands)
        self.assertIn('check-install', install_commands)
        self.assertIn('twine check', install_commands)
        cargo = yaml.safe_load((root / '.github/workflows/publish-cargo.yml').read_text())['jobs']
        for name in ('qualify', 'public-install'):
            self.assertEqual(cargo[name]['strategy']['matrix']['os'], ['ubuntu-22.04', 'windows-2022'])


if __name__ == '__main__':
    unittest.main()
