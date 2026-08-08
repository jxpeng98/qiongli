import hashlib
import json
import shutil
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
    def test_packaged_sources_are_lf_materialized(self) -> None:
        relative_paths = [
            (PACKAGE_ROOT / path).relative_to(REPO_ROOT).as_posix()
            for path in zotero_builder.REQUIRED_FILES
        ]
        result = subprocess.run(
            ["git", "check-attr", "text", "eol", "--", *relative_paths],
            cwd=REPO_ROOT,
            text=True,
            capture_output=True,
            check=False,
        )

        self.assertEqual(result.returncode, 0, msg=result.stderr)
        attributes = result.stdout.splitlines()
        for relative in relative_paths:
            self.assertIn(f"{relative}: text: set", attributes)
            self.assertIn(f"{relative}: eol: lf", attributes)
            self.assertNotIn(b"\r", (REPO_ROOT / relative).read_bytes())

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
                [
                    sys.executable,
                    "scripts/build_zotero_companion.py",
                    "--dist-dir",
                    str(dist),
                    "--release-tag",
                    "v2.0.0-alpha.1",
                    "--repo",
                    "jxpeng98/qiongli",
                ],
                cwd=REPO_ROOT,
                text=True,
                capture_output=True,
                check=False,
            )

            self.assertEqual(result.returncode, 0, msg=result.stderr)
            artifact = next(dist.glob("qiongli-zotero-companion-*.xpi"))
            self.assertIn(str(artifact), result.stdout)
            artifact_manifest_path = dist / "qiongli-zotero-companion.manifest.json"
            self.assertIn(str(artifact_manifest_path), result.stdout)
            artifact_manifest_bytes = artifact_manifest_path.read_bytes()
            artifact_manifest = json.loads(artifact_manifest_bytes)
            update_manifest_path = dist / "qiongli-zotero-companion-updates.json"
            self.assertIn(str(update_manifest_path), result.stdout)
            update_manifest_bytes = update_manifest_path.read_bytes()
            update_manifest = json.loads(update_manifest_bytes)
            artifact_bytes = artifact.read_bytes()
            artifact_size = artifact.stat().st_size
            artifact_name = artifact.name

            with zipfile.ZipFile(artifact) as zf:
                names = set(zf.namelist())
                zip_infos = zf.infolist()
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
        self.assertEqual(manifest["version"], "0.3.0")
        self.assertEqual(manifest["applications"]["zotero"]["update_url"], ZOTERO_UPDATE_URL)
        self.assertEqual(manifest["applications"]["zotero"]["strict_min_version"], "8.0")
        self.assertEqual(manifest["applications"]["zotero"]["strict_max_version"], "9.0.*")
        self.assertEqual(artifact_manifest["schema_version"], 1)
        self.assertEqual(
            artifact_manifest["record_type"],
            "qiongli-zotero-companion-artifact",
        )
        self.assertEqual(artifact_manifest["status"], "assembled-unpublished")
        self.assertEqual(artifact_manifest["companion_version"], "0.3.0")
        self.assertEqual(artifact_manifest["endpoint_version"], "2")
        self.assertEqual(artifact_manifest["artifact_file"], artifact_name)
        self.assertEqual(artifact_manifest["artifact_size_bytes"], artifact_size)
        self.assertEqual(
            artifact_manifest["artifact_sha256"],
            hashlib.sha256(artifact_bytes).hexdigest(),
        )
        self.assertEqual(
            artifact_manifest_bytes,
            json.dumps(
                artifact_manifest,
                ensure_ascii=False,
                separators=(",", ":"),
                sort_keys=True,
            ).encode("utf-8"),
        )
        update = update_manifest["addons"][
            "qiongli-zotero-companion@qiongli.local"
        ]["updates"]
        self.assertEqual(len(update), 1)
        self.assertEqual(update[0]["version"], "0.3.0")
        self.assertEqual(
            update[0]["update_link"],
            "https://github.com/jxpeng98/qiongli/releases/download/"
            "v2.0.0-alpha.1/qiongli-zotero-companion-0.3.0.xpi",
        )
        self.assertEqual(
            update[0]["update_hash"],
            f"sha256:{artifact_manifest['artifact_sha256']}",
        )
        self.assertEqual(
            update[0]["applications"]["zotero"],
            {
                "strict_min_version": "8.0",
                "strict_max_version": "9.0.*",
            },
        )
        self.assertEqual(
            update_manifest_bytes,
            json.dumps(
                update_manifest,
                ensure_ascii=False,
                separators=(",", ":"),
                sort_keys=True,
            ).encode("utf-8"),
        )
        self.assertEqual([info.filename for info in zip_infos], sorted(names))
        self.assertTrue(all(info.compress_type == zipfile.ZIP_STORED for info in zip_infos))
        self.assertTrue(all(info.date_time == (1980, 1, 1, 0, 0, 0) for info in zip_infos))
        self.assertFalse(any(name.startswith("test/") or name.startswith("tests/") for name in names))
        self.assertNotIn("package.json", names)
        for content in text_payloads.values():
            self.assertNotIn("/Users/", content)
            self.assertNotIn("/private/tmp", content)
            self.assertNotIn("secret-key", content)

    def test_build_is_byte_for_byte_deterministic(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            first = Path(tmp_dir) / "first"
            second = Path(tmp_dir) / "second"
            first_artifact = zotero_builder.build(
                PACKAGE_ROOT,
                first,
                release_tag="v2.0.0-alpha.1",
            )
            second_artifact = zotero_builder.build(
                PACKAGE_ROOT,
                second,
                release_tag="v2.0.0-alpha.1",
            )

            self.assertEqual(first_artifact.read_bytes(), second_artifact.read_bytes())
            self.assertEqual(
                zotero_builder.artifact_manifest_path(first).read_bytes(),
                zotero_builder.artifact_manifest_path(second).read_bytes(),
            )
            self.assertEqual(
                zotero_builder.update_manifest_path(first).read_bytes(),
                zotero_builder.update_manifest_path(second).read_bytes(),
            )

    def test_build_rejects_crlf_source_materialization(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            source = Path(tmp_dir) / "source"
            shutil.copytree(PACKAGE_ROOT, source)
            readme = source / "README.md"
            readme.write_bytes(readme.read_bytes().replace(b"\n", b"\r\n"))

            with self.assertRaisesRegex(ValueError, "non-LF line endings"):
                zotero_builder.build(source, Path(tmp_dir) / "dist")

    def test_update_manifest_rejects_untrusted_repository_slug(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            with self.assertRaisesRegex(ValueError, "owner/repository"):
                zotero_builder.build(
                    PACKAGE_ROOT,
                    Path(tmp_dir),
                    release_tag="v2.0.0-alpha.1",
                    repo_slug="jxpeng98/qiongli?download=1",
                )

    def test_manifest_validation_requires_zotero_8_9_update_metadata(self) -> None:
        manifest = {
            "name": COMPANION_DISPLAY_NAME,
            "version": "0.3.0",
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

    def test_manifest_validation_rejects_version_path_traversal(self) -> None:
        manifest = {
            "name": COMPANION_DISPLAY_NAME,
            "version": "../../0.3.0",
            "description": "Tested with Zotero 9.0.4.",
        }

        with self.assertRaisesRegex(ValueError, "safe semantic version"):
            zotero_builder.validate_manifest(manifest)


if __name__ == "__main__":
    unittest.main()
