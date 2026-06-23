from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import unittest
import zipfile
from pathlib import Path

import scripts.build_zotero_companion as zotero_builder


REPO_ROOT = Path(__file__).resolve().parents[1]
PACKAGE_ROOT = REPO_ROOT / "packages" / "qiongli-zotero-companion"
ZOTERO_UPDATE_URL = (
    "https://github.com/jxpeng98/qiongli/releases/latest/download/"
    "qiongli-zotero-companion-updates.json"
)
COMPANION_DISPLAY_NAME = "Qiongli Zotero Companion"


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
        manifest = json.loads(text_payloads["manifest.json"])
        self.assertNotIn("browser_specific_settings", manifest)
        self.assertEqual(manifest["manifest_version"], 2)
        self.assertEqual(manifest["name"], COMPANION_DISPLAY_NAME)
        self.assertIn("Zotero 9.0.4", manifest["description"])
        self.assertEqual(manifest["version"], "0.2.2")
        self.assertEqual(manifest["applications"]["zotero"]["update_url"], ZOTERO_UPDATE_URL)
        self.assertEqual(manifest["applications"]["zotero"]["strict_min_version"], "8.0")
        self.assertEqual(manifest["applications"]["zotero"]["strict_max_version"], "9.0.*")
        self.assertFalse(any(name.startswith("test/") or name.startswith("tests/") for name in names))
        self.assertNotIn("package.json", names)
        for content in text_payloads.values():
            self.assertNotIn("/Users/", content)
            self.assertNotIn("/private/tmp", content)
            self.assertNotIn("secret-key", content)

    def test_manifest_validation_requires_zotero_8_9_update_metadata(self) -> None:
        manifest = {
            "name": COMPANION_DISPLAY_NAME,
            "version": "0.2.2",
            "description": "Tested with Zotero 9.0.4.",
            "applications": {
                "zotero": {
                    "id": "qiongli-zotero-companion@qiongli.local",
                    "strict_min_version": "8.0",
                    "strict_max_version": "9.0.*",
                }
            },
        }

        with self.assertRaisesRegex(ValueError, "applications.zotero.update_url"):
            zotero_builder.validate_manifest(manifest)


if __name__ == "__main__":
    unittest.main()
