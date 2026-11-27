from __future__ import annotations

import contextlib
import copy
import hashlib
import io
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
from urllib.error import URLError

from tooling.scripts import check_native_install_docs as checker


ROOT = Path(__file__).resolve().parents[1]


def published_release(version="2.6.1"):
    """Synthetic API data, never evidence of an actual publication."""
    assets = []
    sums = []
    for target in checker.TARGETS:
        name = f"qiongli-{version}-{target}.{'zip' if target.endswith('msvc') else 'tar.gz'}"
        digest = hashlib.sha256(name.encode()).hexdigest()
        sums.append(f"{digest}  {name}\n")
        assets.append({"name": name, "size": 42, "digest": "sha256:" + digest,
                       "browser_download_url": f"{checker.REPOSITORY}/releases/download/v{version}/{name}"})
    checksums = "".join(sums).encode()
    assets.append({"name": "SHA256SUMS", "size": len(checksums),
                   "digest": "sha256:" + hashlib.sha256(checksums).hexdigest(),
                   "browser_download_url": f"{checker.REPOSITORY}/releases/download/v{version}/SHA256SUMS"})
    return {"tag_name": f"v{version}", "draft": False, "prerelease": False,
            "published_at": "2026-11-13T12:04:35Z", "assets": assets,
            "html_url": f"{checker.REPOSITORY}/releases/tag/v{version}"}, checksums


class NativeInstallDocsTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        # Reuse the actual page layouts, normalizing versions for isolated cases.
        for name in checker.PAGES:
            path = self.root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            text = (ROOT / name).read_text()
            page_version = checker.VERSION.search(text).group(1)
            # Include filenames such as v2.6.2.md as well as visible version pins.
            path.write_text(text.replace(page_version, "2.6.1"))
        notes = self.root / "tooling/release/v2.6.1.md"
        notes.parent.mkdir(parents=True)
        notes.write_text("Synthetic release notes\n")
        self.release, self.checksums = published_release()

    def check(self):
        return checker.check_docs(self.root, self.release, self.checksums)

    def edit(self, name, old, new):
        path = self.root / name
        text = path.read_text()
        self.assertIn(old, text)
        path.write_text(text.replace(old, new))

    def test_current_repository_pages_agree_without_network(self):
        text = (ROOT / "docs/guide/install.md").read_text()
        version = next(iter(checker.VERSION.findall(text)))
        release, checksums = published_release(version)
        with patch.object(checker, "fetch", side_effect=AssertionError("unexpected network")):
            self.assertEqual(checker.check_docs(ROOT, release, checksums), version)

    def test_development_version_does_not_move_stable_installation(self):
        manifest = self.root / "packages/qiongli-native/Cargo.toml"
        manifest.parent.mkdir(parents=True)
        manifest.write_text('[workspace.package]\nversion = "2.9.0-beta.1"\n')
        self.assertEqual(self.check(), "2.6.1")

    def test_original_stale_pages_fail_even_when_they_agree(self):
        for name in checker.PAGES:
            self.edit(name, "2.6.1", "2.5.1")
        with self.assertRaisesRegex(ValueError, "expected published 2.6.1"):
            self.check()

    def test_drift_in_each_language_or_home_page_fails(self):
        for name in checker.PAGES:
            with self.subTest(page=name):
                original = (self.root / name).read_text()
                self.edit(name, "2.6.1", "2.5.1")
                with self.assertRaisesRegex(ValueError, "expected published 2.6.1"):
                    self.check()
                (self.root / name).write_text(original)

    def test_missing_archive_and_wrong_release_url_fail(self):
        for change in ("missing", "wrong-url", "empty", "duplicate"):
            with self.subTest(change=change):
                release = copy.deepcopy(self.release)
                if change == "missing":
                    release["assets"].pop(0)
                elif change == "wrong-url":
                    release["assets"][0]["browser_download_url"] = "https://example.com/archive"
                elif change == "empty":
                    release["assets"][0]["size"] = 0
                else:
                    release["assets"].append(release["assets"][0])
                with self.assertRaises(ValueError):
                    checker.check_docs(self.root, release, self.checksums)

    def test_missing_and_swapped_platform_links_fail(self):
        name = "docs/zh/guide/install.md"
        original = (self.root / name).read_text()
        self.edit(name, "aarch64-apple-darwin.tar.gz", "x86_64-pc-windows-msvc.zip")
        with self.assertRaisesRegex(ValueError, "incorrect download"):
            self.check()
        (self.root / name).write_text(original)
        self.edit(name, "aarch64-apple-darwin.tar.gz", "SWAP")
        self.edit(name, "x86_64-pc-windows-msvc.zip", "aarch64-apple-darwin.tar.gz")
        self.edit(name, "SWAP", "x86_64-pc-windows-msvc.zip")
        with self.assertRaisesRegex(ValueError, "incorrect download"):
            self.check()

    def test_unpinned_registry_commands_fail(self):
        cases = (
            ("README.md", "qiongli@2.6.1", "qiongli@latest"),
            ("docs/guide/install.md", '"qiongli==2.6.1"', "qiongli"),
            ("docs/zh/guide/install.md", "--version 2.6.1 --locked", "--locked"),
        )
        for name, old, new in cases:
            with self.subTest(page=name):
                original = (self.root / name).read_text()
                self.edit(name, old, new)
                with self.assertRaisesRegex(ValueError, "missing.*install"):
                    self.check()
                (self.root / name).write_text(original)

    def test_unpublished_prerelease_and_foreign_release_fail(self):
        for field, value in (("draft", True), ("prerelease", True), ("published_at", None),
                             ("tag_name", "v2.7.0-beta.1"), ("tag_name", "v1.19.1"),
                             ("html_url", "https://github.com/other/repo/releases/tag/v2.6.1")):
            with self.subTest(field=field, value=value):
                release = dict(self.release, **{field: value})
                with self.assertRaises(ValueError):
                    checker.check_docs(self.root, release, self.checksums)

    def test_checksum_identity_and_archive_coverage_fail_closed(self):
        with self.assertRaisesRegex(ValueError, "published digest"):
            checker.check_docs(self.root, self.release, self.checksums + b"\n")
        for data in (b"", self.checksums.splitlines(keepends=True)[0] * 2,
                     self.checksums.replace(self.checksums[:64], b"0" * 64)):
            release = copy.deepcopy(self.release)
            release["assets"][-1]["digest"] = "sha256:" + hashlib.sha256(data).hexdigest()
            with self.assertRaises(ValueError):
                checker.check_docs(self.root, release, data)

    def test_missing_notes_fail_and_historical_notes_are_not_rewritten(self):
        old = self.root / "tooling/release/v2.5.1.md"
        old.write_text("Historical 2.5.1\n")
        self.assertEqual(self.check(), "2.6.1")
        self.assertEqual(old.read_text(), "Historical 2.5.1\n")
        (self.root / "tooling/release/v2.6.1.md").unlink()
        with self.assertRaisesRegex(ValueError, "missing release notes"):
            self.check()

    def test_offline_cli_preserves_inputs_and_never_fetches(self):
        release_path, sums_path = self.root / "release.json", self.root / "SHA256SUMS"
        release_path.write_text(json.dumps(self.release))
        sums_path.write_bytes(self.checksums)
        before = {p.relative_to(self.root): p.read_bytes() for p in self.root.rglob("*") if p.is_file()}
        with patch.object(checker, "fetch", side_effect=AssertionError("unexpected network")):
            with contextlib.redirect_stdout(io.StringIO()) as output:
                self.assertEqual(checker.main(["--root", str(self.root), "--release-json", str(release_path),
                                               "--checksums", str(sums_path)]), 0)
        self.assertIn("supplied release snapshot", output.getvalue())
        self.assertEqual(before, {p.relative_to(self.root): p.read_bytes()
                                  for p in self.root.rglob("*") if p.is_file()})

    def test_live_cli_failure_is_not_a_pass(self):
        for error in (URLError("offline"), ValueError("invalid JSON")):
            with self.subTest(error=error), patch.object(checker, "fetch", side_effect=error):
                with contextlib.redirect_stderr(io.StringIO()):
                    self.assertEqual(checker.main(["--root", str(self.root)]), 1)

    def test_supplied_api_metadata_fetches_only_its_published_checksums(self):
        release_path = self.root / "release.json"
        release_path.write_text(json.dumps(self.release))
        with patch.object(checker, "fetch", return_value=self.checksums) as fetch:
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(checker.main(["--root", str(self.root), "--release-json", str(release_path)]), 0)
        fetch.assert_called_once_with(f"{checker.REPOSITORY}/releases/download/v2.6.1/SHA256SUMS")

    def test_offline_checksums_require_release_metadata(self):
        with patch.object(checker, "fetch", side_effect=AssertionError("unexpected network")):
            with contextlib.redirect_stderr(io.StringIO()), self.assertRaises(SystemExit) as error:
                checker.main(["--checksums", "SHA256SUMS"])
        self.assertEqual(error.exception.code, 2)

    def test_live_cli_uses_published_release_and_its_checksum_url(self):
        with patch.object(checker, "fetch", side_effect=[json.dumps(self.release).encode(), self.checksums]) as fetch:
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(checker.main(["--root", str(self.root)]), 0)
        self.assertEqual(fetch.call_args_list[0].args, (checker.LATEST,))
        self.assertEqual(fetch.call_args_list[1].args,
                         (f"{checker.REPOSITORY}/releases/download/v2.6.1/SHA256SUMS",))


if __name__ == "__main__":
    unittest.main()
