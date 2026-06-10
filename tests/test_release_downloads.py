from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]


class ReleaseDownloadsTests(unittest.TestCase):
    def test_generates_human_and_machine_download_guides(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            out_dir = Path(tmp_dir)
            result = subprocess.run(
                [
                    sys.executable,
                    "scripts/generate_release_downloads.py",
                    "--tag",
                    "v1.1.0-beta.2",
                    "--out-dir",
                    str(out_dir),
                ],
                cwd=REPO_ROOT,
                text=True,
                capture_output=True,
                check=False,
            )

            self.assertEqual(result.returncode, 0, msg=result.stderr)
            guide_path = out_dir / "qiongli-downloads-v1.1.0-beta.2.md"
            index_path = out_dir / "qiongli-downloads-v1.1.0-beta.2.json"
            self.assertIn(str(guide_path), result.stdout)
            self.assertIn(str(index_path), result.stdout)

            guide = guide_path.read_text(encoding="utf-8")
            index = json.loads(index_path.read_text(encoding="utf-8"))

        self.assertIn("# Qiongli v1.1.0-beta.2 Download Guide", guide)
        self.assertIn("Start here", guide)
        self.assertIn("npx qiongli@next install --target all", guide)
        self.assertIn("Use the marketplace command; do not download a plugin tarball", guide)
        self.assertIn("qiongli-next-claude-desktop-skill-core-v1.1.0-beta.2.zip", guide)
        self.assertIn("qiongli-literature-provider-0.1.4.mcpb", guide)
        self.assertIn("qiongli-next-claude-plugin-v1.1.0-beta.2.zip", guide)
        self.assertIn("qiongli-downloads-v1.1.0-beta.2.json", guide)

        self.assertEqual(index["tag"], "v1.1.0-beta.2")
        self.assertEqual(index["channel"], "next")
        self.assertEqual(index["release_url"], "https://github.com/jxpeng98/qiongli/releases/tag/v1.1.0-beta.2")
        self.assertEqual(index["recommended"]["qiongli_cli"]["install"], "npm_next")
        self.assertEqual(index["recommended"]["codex"]["install"], "marketplace")
        self.assertEqual(index["recommended"]["codex"]["plugin"], "qiongli-next")
        self.assertEqual(index["recommended"]["claude_code"]["install"], "marketplace")
        self.assertEqual(index["recommended"]["claude_code"]["plugin"], "qiongli-next")
        self.assertEqual(
            index["recommended"]["claude_desktop_literature_mcpb"]["asset"],
            "qiongli-literature-provider-0.1.4.mcpb",
        )
        self.assertIn(
            "qiongli-next-claude-desktop-skill-core-v1.1.0-beta.2.zip",
            index["assets"]["claude_desktop_skills"],
        )
        self.assertIn(
            "qiongli-next-claude-plugin-v1.1.0-beta.2.tar.gz",
            index["assets"]["maintainer_plugin_tarballs"],
        )
        self.assertIn(
            "qiongli-next-claude-plugin-v1.1.0-beta.2.zip",
            index["assets"]["maintainer_plugin_zips"],
        )
        self.assertNotIn(
            "qiongli-economics-claude-plugin-v1.1.0-beta.2.tar.gz",
            index["assets"]["maintainer_plugin_tarballs"],
        )
        self.assertNotIn(
            "qiongli-economics-claude-plugin-v1.1.0-beta.2.zip",
            index["assets"]["maintainer_plugin_zips"],
        )

    def test_release_notes_include_download_guide_section(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            note_path = Path(tmp_dir) / "notes.md"
            result = subprocess.run(
                [
                    "bash",
                    "scripts/generate_release_notes.sh",
                    "--tag",
                    "v1.1.0-beta.2",
                    "--from-tag",
                    "v1.1.0-beta.1",
                    "--output",
                    str(note_path),
                    "--overwrite",
                ],
                cwd=REPO_ROOT,
                text=True,
                capture_output=True,
                check=False,
            )

            self.assertEqual(result.returncode, 0, msg=result.stderr)
            notes = note_path.read_text(encoding="utf-8")

        self.assertIn("## Download Guide", notes)
        self.assertIn("Most users should not download plugin tarballs manually", notes)
        self.assertIn("qiongli-downloads-v1.1.0-beta.2.md", notes)
        self.assertIn("qiongli-next-claude-desktop-skill-core-v1.1.0-beta.2.zip", notes)
        self.assertIn("qiongli-literature-provider-0.1.4.mcpb", notes)
        self.assertIn("Claude plugin ZIPs", notes)


if __name__ == "__main__":
    unittest.main()
