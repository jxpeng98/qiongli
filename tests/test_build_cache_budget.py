import argparse
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

from tooling.scripts.check_build_cache_budget import CACHES, check_budget, positive_gib


class BuildCacheBudgetTests(unittest.TestCase):
    def test_budget_boundaries_and_unrelated_artifacts_are_preserved(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            cache = root / CACHES[0]
            cache.mkdir(parents=True)
            (cache / 'dependency.rlib').write_bytes(b'12345')
            release = root / 'packages/qiongli-native/target/release/qiongli'
            release.parent.mkdir(parents=True)
            release.write_bytes(b'keep released executable')
            with patch('tooling.scripts.check_build_cache_budget.shutil.disk_usage', return_value=SimpleNamespace(free=10)):
                self.assertTrue(check_budget(root, 5, 10)['within_budget'])
                self.assertFalse(check_budget(root, 4, 10)['within_budget'])
                self.assertFalse(check_budget(root, 5, 11)['within_budget'])
            self.assertEqual(release.read_bytes(), b'keep released executable')
            self.assertEqual((cache / 'dependency.rlib').read_bytes(), b'12345')

    def test_symlinks_refuse_measurement_instead_of_following_external_data(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            cache = root / CACHES[0]
            cache.parent.mkdir(parents=True)
            cache.symlink_to(root, target_is_directory=True)
            with self.assertRaises(ValueError):
                check_budget(root, 100, 1)
            self.assertTrue(cache.is_symlink())

    def test_nonfinite_or_nonpositive_budgets_are_rejected(self):
        for value in ('nan', 'inf', '0', '-1'):
            with self.subTest(value=value), self.assertRaises(argparse.ArgumentTypeError):
                positive_gib(value)
