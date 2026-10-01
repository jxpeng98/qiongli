from pathlib import Path
import json
import os
import subprocess
import tarfile
import tempfile
import unittest
import zipfile
from unittest.mock import patch

from tooling.scripts.native_cli_release import archive_cli, archive_readme, check_windows_imports
from tooling.scripts.native_registry_install_check import check_cli, check_transition, require_transition
from tooling.scripts.native_registry_packages import TARGETS, cli_description


class NativeCliReleaseTests(unittest.TestCase):
    def test_cli_smoke_accepts_only_coherent_named_reader_extension(self):
        version = '2.1.1'
        def check(lite, full, *, local_delta=0, reader_name='qiongli_literature_read_fulltext'):
            def response(argv, *, root, env, input=None, check=True):
                args = argv[1:]
                if args == ['--version']:
                    value = f'qiongli {version}\n'
                elif args == ['--help']:
                    value = 'Usage: qiongli project'
                elif args == ['project', '--help']:
                    value = 'qiongli project'
                elif args == ['content', 'list']:
                    value = json.dumps({'content_version': version, 'pack_sha256': 'a' * 64})
                elif args == ['not-a-command']:
                    return subprocess.CompletedProcess(argv, 1, '', 'error: invalid command')
                else:
                    profile = args[args.index('--profile') + 1]
                    base, extension = (14, lite) if profile == 'lite' else (32, full)
                    if args[1] == 'check':
                        value = json.dumps({'scope': 'local-in-process-protocol',
                            'tool_count': base + extension + local_delta, 'read_only_call': 'passed',
                            'host_session': 'not-checked'})
                    else:
                        names = [f'base_tool_{index}' for index in range(base)]
                        if extension:
                            names.append(reader_name)
                        value = '\n'.join(json.dumps(message) for message in [
                            {'id': 1, 'result': {'serverInfo': {'version': version}}},
                            {'id': 2, 'result': {'tools': [{'name': name} for name in names]}},
                            {'id': 3, 'result': {'structuredContent': {'status': 'ok'}}},
                        ])
                return subprocess.CompletedProcess(argv, 0, value, '')
            with patch('tooling.scripts.native_registry_install_check.run', side_effect=response):
                return check_cli('candidate', version=version, root=Path('.'), env={})
        for extension in (0, 1):
            self.assertEqual(check(extension, extension)['mcp_tools'],
                             {'lite': 14 + extension, 'full': 32 + extension})
        for lite, full in ((0, 1), (1, 0)):
            with self.assertRaisesRegex(ValueError, 'incoherent'):
                check(lite, full)
        with self.assertRaisesRegex(ValueError, 'unsupported MCP'):
            check(1, 1, reader_name='unrelated_extension')
        with self.assertRaises(AssertionError):
            check(1, 1, local_delta=-1)

    def test_transition_probes_both_hosts_and_rejects_wrong_or_absent_v1(self):
        fixture = Path(__file__).resolve().parents[1] / 'packages/qiongli-native/apps/qiongli/tests/fixtures/plugin-source-v1.status.json'
        golden = json.loads(fixture.read_text())
        def response(argv, *, root, env):
            self.assertNotIn('CODEX_HOME', env)
            self.assertNotIn('CLAUDE_CONFIG_DIR', env)
            self.assertNotEqual(env['HOME'], '/real-home')
            self.assertEqual(root.stat().st_mode & 0o077, 0)
            value = dict(golden, destination=str(argv[-1]),
                         target='codex' if argv[-3] == 'codex' else 'claude-code')
            return subprocess.CompletedProcess(argv, 0, json.dumps(value), '')
        with patch('tooling.scripts.native_registry_install_check.run', side_effect=response) as run:
            observed = check_transition(['candidate'], version='2.0.1',
                                        env={'HOME': '/real-home', 'CODEX_HOME': '/real-host'})
            self.assertEqual(run.call_count, 2)
            probe_root = run.call_args.kwargs['root']
            self.assertFalse(probe_root.exists())
            self.assertNotIn(str(probe_root), json.dumps(observed))
            require_transition(observed)
            self.assertIsNone(check_transition(['candidate'], version='2.0.0', env={}))
            self.assertEqual(run.call_count, 2)
        for field, wrong in [('schema_version', 2), ('schema_version', True), ('plugin_id', 'qiongli@qiongli-cli-local'),
                             ('state', 'source-current'), ('target', 'claude'), ('source', {})]:
            changed = json.loads(json.dumps(observed))
            changed['claude'][field] = wrong
            with self.subTest(field=field), self.assertRaises(ValueError):
                require_transition(changed)
        for missing in (None, {}, {'codex': observed['codex']}):
            with self.assertRaises(ValueError):
                require_transition(missing)
        with patch('tooling.scripts.native_registry_install_check.run',
                   return_value=subprocess.CompletedProcess([], 0, json.dumps(dict(golden, schema_version=2)), '')):
            with self.assertRaisesRegex(ValueError, 'actual v1'):
                check_transition(['candidate'], version='2.0.1', env={})
        def boolean_response(*args, **kwargs):
            result = response(*args, **kwargs)
            value = json.loads(result.stdout)
            value['schema_version'] = True
            result.stdout = json.dumps(value)
            return result
        with patch('tooling.scripts.native_registry_install_check.run', side_effect=boolean_response):
            with self.assertRaisesRegex(ValueError, 'transition evidence'):
                check_transition(['candidate'], version='2.0.1', env={})
        with patch('tooling.scripts.native_registry_install_check.run', side_effect=subprocess.CalledProcessError(1, 'probe')) as failed:
            with self.assertRaises(subprocess.CalledProcessError):
                check_transition(['candidate'], version='2.0.1', env={})
            self.assertFalse(failed.call_args.kwargs['root'].exists())
        with patch('tooling.scripts.native_registry_install_check.sys.platform', 'win32'), \
             patch.dict(os.environ, {'SYSTEMROOT': 'C:/Windows'}), \
             patch('tooling.scripts.native_registry_install_check.subprocess.check_output', return_value='"runner","S-1-5-21-123"\n'), \
             patch('tooling.scripts.native_registry_install_check.subprocess.run') as acl, \
             patch('tooling.scripts.native_registry_install_check.run', side_effect=response):
            require_transition(check_transition(['candidate'], version='2.0.1', env={}))
            self.assertEqual(acl.call_args.args[0][2:], ['/inheritance:r', '/grant:r', '*S-1-5-21-123:F'])
            self.assertTrue(acl.call_args.kwargs['check'])

    def test_windows_import_check_rejects_extra_runtimes_and_unreadable_tables(self):
        with patch('tooling.scripts.native_cli_release.shutil.which', return_value='llvm-objdump'), \
             patch('tooling.scripts.native_cli_release.subprocess.check_output') as inspect:
            inspect.return_value = '    DLL Name: KERNEL32.dll\n    DLL Name: api-ms-win-core-synch-l1-2-0.dll\n'
            self.assertEqual(len(check_windows_imports(Path('qiongli.exe'))), 2)
            for extra in ('VCRUNTIME140.dll', 'MSVCP140.dll', 'python312.dll', 'libssl-3-x64.dll'):
                inspect.return_value = f'DLL Name: kernel32.dll\nDLL Name: {extra}\n'
                with self.subTest(extra=extra), self.assertRaisesRegex(ValueError, 'only system DLLs'):
                    check_windows_imports(Path('qiongli.exe'))
            inspect.return_value = 'file format not recognized'
            with self.assertRaisesRegex(ValueError, 'unreadable import table'):
                check_windows_imports(Path('qiongli.exe'))

    def test_archive_preserves_executable_without_host_paths_or_links(self):
        for target, (_, _, executable) in TARGETS.items():
            with self.subTest(target=target), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                binary = root / 'candidate'
                binary.write_bytes(b'candidate executable')
                readme = archive_readme('2.0.0-beta.2', target, 'a' * 40)
                self.assertIn(cli_description().encode(), readme)
                windows = target.endswith('msvc')
                path = root / ('cli.zip' if windows else 'cli.tar.gz')
                archive_cli(path, binary, readme, target)
                if windows:
                    with zipfile.ZipFile(path) as archive:
                        self.assertEqual(archive.namelist(), [executable, 'README.md', 'LICENSE'])
                        archive.extractall(root / 'installed')
                else:
                    with tarfile.open(path) as archive:
                        self.assertEqual(archive.getnames(), [executable, 'README.md', 'LICENSE'])
                        self.assertTrue(all(item.isfile() and item.uid == 0 for item in archive))
                        self.assertEqual(archive.getmember(executable).mode, 0o755)
                        archive.extractall(root / 'installed', filter='data')
                self.assertEqual((root / 'installed' / executable).read_bytes(), binary.read_bytes())
                self.assertEqual((root / 'installed/README.md').read_bytes(), readme)
                command = f'.\\{executable}' if windows else f'./{executable}'
                self.assertIn(f'{command} --version'.encode(), readme)
                self.assertIn(f'{command} content list'.encode(), readme)
                self.assertIn(target.encode(), readme)
                self.assertIn(b'v2.0.0-beta.2', readme)
                self.assertIn(('blob/' + 'a' * 40 + '/docs/guide/cli-2x.md').encode(), readme)
                self.assertNotIn(str(root).encode(), readme)
                link = root / 'linked-candidate'
                link.symlink_to(binary)
                with self.assertRaises(ValueError):
                    archive_cli(root / path.name.replace('cli.', 'linked.'), link, readme, target)


if __name__ == '__main__':
    unittest.main()
