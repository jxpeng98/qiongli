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
