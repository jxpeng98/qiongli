from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import unittest
import zipfile
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
PACKAGE_ROOT = REPO_ROOT / "packages" / "qiongli-zotero-companion"


class ZoteroCompanionArtifactTests(unittest.TestCase):
    def test_zotero_companion_package_declares_pack_script(self) -> None:
        package = json.loads((PACKAGE_ROOT / "package.json").read_text(encoding="utf-8"))

        self.assertEqual(package["name"], "qiongli-zotero-companion")
        self.assertEqual(package["type"], "module")
        self.assertEqual(
            package["scripts"]["pack:xpi"],
            "python3 ../../scripts/build_zotero_companion.py --dist-dir dist",
        )

    def test_build_zotero_companion_contains_installable_extension_files(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            dist = Path(tmp_dir) / "dist"
            result = subprocess.run(
                [sys.executable, "scripts/build_zotero_companion.py", "--dist-dir", str(dist)],
                cwd=REPO_ROOT,
                text=True,
                capture_output=True,
                check=False,
            )

            self.assertEqual(result.returncode, 0, msg=result.stderr)
            artifact = next(dist.glob("qiongli-zotero-companion-*.xpi"))
            self.assertIn(str(artifact), result.stdout)

            with zipfile.ZipFile(artifact) as zf:
                names = set(zf.namelist())
                text_payloads = {
                    name: zf.read(name).decode("utf-8")
                    for name in names
                    if name.endswith((".json", ".js", ".md"))
                }

        self.assertIn("manifest.json", names)
        self.assertIn("bootstrap.js", names)
        self.assertIn("README.md", names)
        self.assertIn("chrome/content/qiongli-bridge.js", names)
        self.assertFalse(any(name.startswith("test/") or name.startswith("tests/") for name in names))
        self.assertNotIn("package.json", names)
        for content in text_payloads.values():
            self.assertNotIn("/Users/", content)
            self.assertNotIn("/private/tmp", content)
            self.assertNotIn("secret-key", content)


if __name__ == "__main__":
    unittest.main()
