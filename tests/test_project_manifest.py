from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from bridges.project_manifest import (
    ProjectManifestError,
    init_project_manifest,
    load_project_manifest,
    update_project_manifest,
)


class ProjectManifestTests(unittest.TestCase):
    def test_missing_manifest_returns_implicit_auto_subject(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            state = load_project_manifest(Path(tmp_dir))

        self.assertFalse(state.exists)
        self.assertEqual(state.manifest.active_subject, "auto")
        self.assertEqual(state.manifest.strictness, "standard")
        self.assertEqual(state.manifest.secondary_subjects, [])
        self.assertEqual(state.manifest.venue_profiles, [])
        self.assertEqual(state.manifest.method_lenses, [])

    def test_init_project_manifest_writes_default_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            state = init_project_manifest(root)

            self.assertTrue(state.path.is_file())
            text = state.path.read_text(encoding="utf-8")
            self.assertIn("active_subject: auto", text)
            self.assertIn("strictness: standard", text)

    def test_update_project_manifest_sets_subject_venue_and_methods(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            init_project_manifest(root)

            state = update_project_manifest(
                root,
                active_subject="finance",
                venue_profiles=["journal-of-finance"],
                method_lenses=["asset-pricing", "event-study"],
                strictness="high",
            )

            self.assertEqual(state.manifest.active_subject, "finance")
            self.assertEqual(state.manifest.venue_profiles, ["journal-of-finance"])
            self.assertEqual(state.manifest.method_lenses, ["asset-pricing", "event-study"])
            self.assertEqual(state.manifest.strictness, "high")

    def test_invalid_subject_fails_closed(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            (root / ".qiongli").mkdir()
            (root / ".qiongli" / "guidance_manifest.yaml").write_text(
                "active_subject: unknown-field\n",
                encoding="utf-8",
            )

            with self.assertRaisesRegex(ProjectManifestError, "Unsupported active_subject"):
                load_project_manifest(root)


if __name__ == "__main__":
    unittest.main()
