from __future__ import annotations

import io
from pathlib import Path
import tarfile
import tempfile
import unittest
from unittest.mock import patch

from tooling.scripts.native_registry_install_check import install_cargo_archives


class NativeCargoArchivesTests(unittest.TestCase):
    def install(self, *, count=10, missing=False, extra=False, duplicate=False,
                lock_version='2.2.0'):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            packages = root / 'packages'
            packages.mkdir()
            names = ['qiongli'] + [f'qiongli-library-{i}' for i in range(count - 1)]
            lock = 'version = 4\n' + ''.join(
                f'[[package]]\nname = "{name}"\nversion = "{lock_version}"\n'
                for name in names)
            archive_names = names[:-1] if missing else names.copy()
            if extra:
                archive_names.append('qiongli-unexpected')
            artifacts = []
            for name in archive_names:
                stem = f'{name}-2.2.0'
                path = packages / f'{stem}.crate'
                with tarfile.open(path, 'w:gz') as archive:
                    files = {'Cargo.toml': f'[package]\nname = "{name}"\nversion = "2.2.0"\n'}
                    if name == 'qiongli':
                        files['Cargo.lock'] = lock
                    for filename, content in files.items():
                        data = content.encode()
                        member = tarfile.TarInfo(f'{stem}/{filename}')
                        member.size = len(data)
                        archive.addfile(member, io.BytesIO(data))
                artifacts.append({'file': path.name})
            if duplicate:
                artifacts.append(artifacts[-1])
            with patch('tooling.scripts.native_registry_install_check.subprocess.run') as run:
                try:
                    install_cargo_archives(packages, {'version': '2.2.0', 'artifacts': artifacts},
                                           root, {}, root / 'target')
                except ValueError:
                    run.assert_not_called()
                    raise
                run.assert_called_once()
                command = run.call_args.args[0]
                self.assertIn('--offline', command)
                self.assertIn('--locked', command)
                config = (root / 'cargo-archive-patches.toml').read_text()
                self.assertEqual(config.count('path ='), count - 1)

    def test_complete_old_and_growing_workspace_closures(self):
        for count in (9, 10, 11):
            with self.subTest(count=count):
                self.install(count=count)

    def test_missing_archive_is_rejected_before_install(self):
        with self.assertRaisesRegex(ValueError, 'lockfile closure'):
            self.install(missing=True)

    def test_unexpected_archive_is_rejected_before_install(self):
        with self.assertRaisesRegex(ValueError, 'lockfile closure'):
            self.install(extra=True)

    def test_duplicate_archive_is_rejected_before_install(self):
        with self.assertRaisesRegex(ValueError, 'duplicate'):
            self.install(duplicate=True)

    def test_mixed_lockfile_versions_are_rejected_before_install(self):
        with self.assertRaisesRegex(ValueError, 'lockfile closure'):
            self.install(lock_version='2.1.1')
