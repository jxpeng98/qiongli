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
        current = tomllib.loads((publish.ROOT / 'packages/qiongli-native/Cargo.toml').read_text())['workspace']['package']['version']
        for version, outcome in product((current, '2.0.0'), ('ci-failure', 'asset-failure', 'wrong-main', 'success')):
            commit, tag = 'a' * 40, f'v{version}'
            stable = not publish.parse_release_version(tag).is_prerelease
            if outcome == 'wrong-main' and not stable:
                continue
            with self.subTest(version=version, outcome=outcome), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary) / 'source'
                native = root / 'packages/qiongli-native'
                native.mkdir(parents=True)
                (native / 'Cargo.toml').write_text(f'[workspace.package]\nversion = "{version}"\n')
                notes = root / 'tooling/release'
                notes.mkdir(parents=True)
                (notes / f'{tag}.md').write_text('Reviewed fixture notes')
                calls = []
                def command(*args):
                    args = tuple(map(str, args))
                    calls.append(args)
                    if args[:2] == ('git', 'rev-parse'):
                        return commit
                    if args[:2] == ('gh', 'api'):
                        if '/commits/' in args[2]:
                            return json.dumps({'sha': 'b' * 40 if args[2].endswith('/main') and outcome == 'wrong-main' else commit})
                        if '/actions/workflows/' in args[2]:
                            return json.dumps({'workflow_runs': [dict(id=4, head_sha=commit, head_branch=tag,
                                event='workflow_dispatch', status='completed', html_url='https://example.invalid/run',
                                conclusion='failure' if outcome == 'ci-failure' else 'success')]})
                        return '[]'
                    if args[:3] == ('gh', 'run', 'download'):
                        assets = Path(temporary) / 'native-release-assets'
                        assets.mkdir()
                        (assets / 'verified-asset').write_text('fixture')
                        (assets / 'release-manifest.json').write_text(json.dumps({'artifacts': [{'file': 'verified-asset'}]}))
                        (assets / 'SHA256SUMS').write_text('fixture checksum')
                        for unlisted in ('qiongli-codex-plugin-v2.0.1.tar.gz', 'arbitrary.txt', 'registry-packages.json'):
                            (assets / unlisted).write_text('unverified')
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
                    self.assertEqual([args[2] for args in mutations], ['create', 'edit'])
                    uploaded = {Path(arg).name for arg in mutations[0] if str(Path(temporary) / 'native-release-assets') in arg}
                    self.assertEqual(uploaded, {'verified-asset', 'release-manifest.json', 'SHA256SUMS'})
                    self.assertEqual('--prerelease' in mutations[0], not stable)
                    self.assertEqual('--latest' in mutations[1], stable)
                    self.assertIn('--prerelease=false' if stable else '--prerelease', mutations[1])
                    verification = next(args for args in calls if '--require-ci' in args)
                    self.assertLess(calls.index(verification), calls.index(mutations[0]))
                    dispatches = [args for args in calls if 'publish_release=true' in args]
                    self.assertEqual(len(dispatches), 3)
                    self.assertTrue(all(calls.index(args) > calls.index(mutations[-1]) for args in dispatches))
                else:
                    self.assertEqual(mutations, [])
                    self.assertFalse(any('publish_release=true' in args for args in calls))
                    if outcome == 'wrong-main':
                        self.assertFalse(any(args[:3] == ('gh', 'workflow', 'run') for args in calls))

    def test_run_selection_binds_source_tag_and_dispatch(self):
        good = dict(id=5, head_sha='source', head_branch='v2.0.0-beta.2', event='workflow_dispatch')
        unrelated = [good | {'id': 8, 'head_sha': 'other'},
                     good | {'id': 9, 'head_branch': '2.x'},
                     good | {'id': 10, 'event': 'push'}]
        self.assertIsNone(publish.select_run(unrelated, 'source', 'v2.0.0-beta.2'))
        self.assertEqual(publish.select_run(unrelated + [good], 'source', 'v2.0.0-beta.2'), good)

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


if __name__ == '__main__':
    unittest.main()
