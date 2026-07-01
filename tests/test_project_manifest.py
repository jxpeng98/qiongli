from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

import yaml

from bridges.project_manifest import (
    ProjectManifestError,
    init_project_manifest,
    load_project_manifest,
    manifest_to_guidance_section,
    update_project_manifest,
)


class ProjectManifestTests(unittest.TestCase):
    def test_missing_manifest_returns_implicit_auto_subject(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            state = load_project_manifest(Path(tmp_dir))

        self.assertFalse(state.exists)
        self.assertEqual(state.manifest.active_subject, "auto")
        self.assertEqual(state.manifest.subject_mode, "auto")
        self.assertEqual(state.manifest.strictness, "standard")
        self.assertEqual(state.manifest.secondary_subjects, [])
        self.assertEqual(state.manifest.venue_profiles, [])
        self.assertEqual(state.manifest.method_lenses, [])

    def test_missing_manifest_packet_uses_project_relative_path(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            state = load_project_manifest(Path(tmp_dir))

        self.assertEqual(state.to_packet()["path"], ".qiongli/guidance_manifest.yaml")

    def test_init_project_manifest_writes_default_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            state = init_project_manifest(root)

            self.assertTrue(state.path.is_file())
            text = state.path.read_text(encoding="utf-8")
            self.assertIn("active_subject: auto", text)
            self.assertIn("subject_mode: auto", text)
            self.assertIn("strictness: standard", text)

    def test_existing_manifest_packet_uses_project_relative_path(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            state = init_project_manifest(Path(tmp_dir))

        self.assertEqual(state.to_packet()["path"], ".qiongli/guidance_manifest.yaml")

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

    def test_load_project_manifest_reads_locked_subject_mode(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            manifest_path = root / ".qiongli" / "guidance_manifest.yaml"
            manifest_path.parent.mkdir()
            manifest_path.write_text(
                "active_subject: finance\n"
                "subject_mode: locked\n",
                encoding="utf-8",
            )

            state = load_project_manifest(root)

        self.assertEqual(state.manifest.active_subject, "finance")
        self.assertEqual(state.manifest.subject_mode, "locked")

    def test_legacy_non_auto_manifest_without_subject_mode_becomes_confirmed(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            manifest_path = root / ".qiongli" / "guidance_manifest.yaml"
            manifest_path.parent.mkdir()
            manifest_path.write_text("active_subject: finance\n", encoding="utf-8")

            state = load_project_manifest(root)

        self.assertEqual(state.manifest.active_subject, "finance")
        self.assertEqual(state.manifest.subject_mode, "confirmed")

    def test_update_project_manifest_sets_locked_subject_mode(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)

            state = update_project_manifest(
                root,
                active_subject="finance",
                subject_mode="locked",
            )
            payload = yaml.safe_load(state.path.read_text(encoding="utf-8")) or {}

        self.assertEqual(state.manifest.active_subject, "finance")
        self.assertEqual(state.manifest.subject_mode, "locked")
        self.assertEqual(payload.get("subject_mode"), "locked")

    def test_update_project_manifest_sets_concrete_subject_to_confirmed_by_default(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)

            state = update_project_manifest(root, active_subject="finance")
            payload = yaml.safe_load(state.path.read_text(encoding="utf-8")) or {}

        self.assertEqual(state.manifest.active_subject, "finance")
        self.assertEqual(state.manifest.subject_mode, "confirmed")
        self.assertEqual(payload.get("subject_mode"), "confirmed")

    def test_manifest_guidance_section_includes_subject_mode(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            state = update_project_manifest(
                root,
                active_subject="finance",
                subject_mode="confirmed",
            )

        self.assertIn("- subject_mode: confirmed", manifest_to_guidance_section(state))

    def test_update_project_manifest_preserves_unknown_fields(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            manifest_path = root / ".qiongli" / "guidance_manifest.yaml"
            manifest_path.parent.mkdir()
            manifest_path.write_text(
                "active_subject: finance\n"
                "future_field:\n"
                "  owner: local\n"
                "future_list:\n"
                "  - alpha\n",
                encoding="utf-8",
            )

            state = update_project_manifest(root, strictness="high")
            payload = yaml.safe_load(manifest_path.read_text(encoding="utf-8")) or {}

        self.assertIn("Ignored unsupported manifest field: future_field", state.warnings)
        self.assertIn("Ignored unsupported manifest field: future_list", state.warnings)
        self.assertEqual(payload.get("future_field"), {"owner": "local"})
        self.assertEqual(payload.get("future_list"), ["alpha"])
        self.assertEqual(payload.get("strictness"), "high")

    def test_update_project_manifest_preserves_unspecified_lists(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            update_project_manifest(
                root,
                active_subject="finance",
                secondary_subjects=["economics", "accounting"],
                venue_profiles=["journal-of-finance"],
                method_lenses=["asset-pricing", "event-study"],
            )

            state = update_project_manifest(root, strictness="high")

        self.assertEqual(state.manifest.secondary_subjects, ["economics", "accounting"])
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
