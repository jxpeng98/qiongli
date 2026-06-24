from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]


class ReleaseDownloadsTests(unittest.TestCase):
    def test_stable_download_section_updater_rewrites_docs(self) -> None:
        targets = {
            "README.md": "## Latest Stable Downloads",
            "README_CN.md": "## 最新稳定版下载",
            "docs/index.md": "## Latest Stable Downloads",
            "docs/zh/index.md": "## 最新稳定版下载",
            "docs/guide/install.md": "## Latest Stable Downloads",
            "docs/zh/guide/install.md": "## 最新稳定版下载",
        }

        with tempfile.TemporaryDirectory() as tmp_dir:
            docs_root = Path(tmp_dir)
            for relative_path, heading in targets.items():
                path = docs_root / relative_path
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(
                    "\n".join(
                        [
                            f"# Fixture for {relative_path}",
                            "",
                            heading,
                            "",
                            "Current stable release: [v1.5.0](https://github.com/jxpeng98/qiongli/releases/tag/v1.5.0).",
                            "",
                            "| Need | Link or command |",
                            "|---|---|",
                            "| npm CLI | stale v1.5.0 link |",
                            "",
                            "## After",
                            "",
                            "Keep this trailing section.",
                            "",
                        ]
                    ),
                    encoding="utf-8",
                )

            result = subprocess.run(
                [
                    sys.executable,
                    "scripts/update_stable_download_sections.py",
                    "--tag",
                    "v1.6.0",
                    "--root",
                    str(docs_root),
                    "--asset-root",
                    str(REPO_ROOT),
                ],
                cwd=REPO_ROOT,
                text=True,
                capture_output=True,
                check=False,
            )

            self.assertEqual(result.returncode, 0, msg=result.stderr)
            self.assertIn("updated stable download sections: 6 files", result.stdout)

            for relative_path, heading in targets.items():
                content = (docs_root / relative_path).read_text(encoding="utf-8")
                self.assertIn(f"# Fixture for {relative_path}", content)
                self.assertIn(heading, content)
                self.assertIn("## After", content)
                self.assertIn("Keep this trailing section.", content)
                self.assertNotIn("v1.5.0", content)
                self.assertIn("[v1.6.0](https://github.com/jxpeng98/qiongli/releases/tag/v1.6.0)", content)
                self.assertIn("qiongli-claude-desktop-skill-core-v1.6.0.zip", content)
                self.assertIn("qiongli-literature-provider-0.1.4.mcpb", content)
                self.assertIn("qiongli-zotero-companion-0.2.2.xpi", content)
                self.assertIn("qiongli-downloads-v1.6.0.md", content)

            english = (docs_root / "README.md").read_text(encoding="utf-8")
            chinese = (docs_root / "README_CN.md").read_text(encoding="utf-8")
            self.assertIn("Current stable release:", english)
            self.assertIn("| All release assets |", english)
            self.assertIn("当前稳定版是", chinese)
            self.assertIn("| 全部 release assets |", chinese)

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
        self.assertIn("## Direct downloads", guide)
        self.assertIn("Start here", guide)
        self.assertIn("https://github.com/jxpeng98/qiongli/releases/download/v1.1.0-beta.2/qiongli-next-claude-desktop-skill-core-v1.1.0-beta.2.zip", guide)
        self.assertIn("npx qiongli@next install --target all", guide)
        self.assertIn("Use the marketplace command; do not download a plugin tarball", guide)
        self.assertIn("qiongli-next-claude-desktop-skill-core-v1.1.0-beta.2.zip", guide)
        self.assertIn("qiongli-literature-provider-0.1.4.mcpb", guide)
        self.assertIn("qiongli-zotero-companion-0.2.2.xpi", guide)
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
        self.assertEqual(
            index["recommended"]["zotero_desktop_companion"]["asset"],
            "qiongli-zotero-companion-0.2.2.xpi",
        )
        self.assertEqual(
            index["assets"]["zotero_desktop_companion"],
            "qiongli-zotero-companion-0.2.2.xpi",
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
        self.assertIn("qiongli-zotero-companion-0.2.2.xpi", notes)
        self.assertIn("Claude plugin ZIPs", notes)

    def test_stable_release_notes_include_category_downloads_and_changelog(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            note_path = Path(tmp_dir) / "stable-notes.md"
            result = subprocess.run(
                [
                    sys.executable,
                    "scripts/generate_stable_release_notes.py",
                    "--tag",
                    "v1.5.0",
                    "--output",
                    str(note_path),
                ],
                cwd=REPO_ROOT,
                text=True,
                capture_output=True,
                check=False,
            )

            self.assertEqual(result.returncode, 0, msg=result.stderr)
            notes = note_path.read_text(encoding="utf-8")

        self.assertIn("## Release Category", notes)
        self.assertIn("## Download Guide", notes)
        self.assertIn("npm install -g qiongli@latest", notes)
        self.assertIn("pipx install qiongli", notes)
        self.assertIn("qiongli-claude-desktop-skill-core-v1.5.0.zip", notes)
        self.assertIn("qiongli-literature-provider-0.1.4.mcpb", notes)
        self.assertIn("qiongli-zotero-companion-0.2.2.xpi", notes)
        self.assertIn("qiongli-downloads-v1.5.0.md", notes)
        self.assertIn("## Changelog", notes)
        self.assertIn("### [1.5.0] - 2026-06-23", notes)


if __name__ == "__main__":
    unittest.main()
