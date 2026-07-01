from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from bridges.subject_guidance import END_MARKER, START_MARKER, SUBJECT_GUIDANCE_REL
from bridges.subject_lifecycle import (
    SubjectLifecycleError,
    apply_subject_action,
    subject_status,
)


class SubjectLifecycleTests(unittest.TestCase):
    def test_subject_status_on_new_project_reports_auto_without_creating_files(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)

            result = subject_status(root)

            self.assertEqual(result["project_root"], str(root.resolve()))
            self.assertFalse(result["manifest_exists"])
            self.assertEqual(result["manifest"]["active_subject"], "auto")
            self.assertEqual(result["manifest"]["subject_mode"], "auto")
            self.assertEqual(result["state"]["schema_version"], "1.0")
            self.assertEqual(result["state"]["subjects"], {})
            self.assertEqual(result["state"]["dismissed_subjects"], {})
            self.assertEqual(result["state"]["lifecycle_events"], [])
            self.assertFalse(self._manifest_path(root).exists())
            self.assertFalse(self._state_path(root).exists())

    def test_status_reports_missing_subject_guidance_without_creating_qiongli(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)

            result = subject_status(root)

            self.assertEqual(result["subject_guidance"]["path"], SUBJECT_GUIDANCE_REL.as_posix())
            self.assertFalse(result["subject_guidance"]["exists"])
            self.assertEqual(result["subject_guidance"]["managed_block"], "missing")
            self.assertFalse((root / ".qiongli").exists())

    def test_confirm_updates_manifest_and_records_event(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)

            result = apply_subject_action(
                root,
                "confirm",
                "finance",
                source="cli",
                run_id="run-1",
            )

            self.assertTrue(self._manifest_path(root).is_file())
            self.assertTrue(self._state_path(root).is_file())
            self.assertTrue(result["manifest_exists"])
            self.assertEqual(result["manifest"]["active_subject"], "finance")
            self.assertEqual(result["manifest"]["subject_mode"], "confirmed")
            self.assertEqual(len(result["state"]["lifecycle_events"]), 1)
            event = result["state"]["lifecycle_events"][0]
            self.assertEqual(event["action"], "confirm")
            self.assertEqual(event["subject"], "finance")
            self.assertEqual(event["source"], "cli")
            self.assertEqual(event["run_id"], "run-1")
            self.assertTrue(event["created_at"].endswith("+00:00"))

    def test_confirm_materializes_active_subject_guidance(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            self._manifest_path(root).parent.mkdir(parents=True)
            self._manifest_path(root).write_text(
                "method_lenses:\n- event-study\n- asset-pricing\n",
                encoding="utf-8",
            )

            result = apply_subject_action(
                root,
                "confirm",
                "finance",
                source="cli",
                run_id="run-1",
            )

            text = self._guidance_path(root).read_text(encoding="utf-8")
            self.assertEqual(result["subject_guidance"]["managed_block"], "active")
            self.assertEqual(result["subject_guidance"]["active_subject"], "finance")
            self.assertEqual(result["subject_guidance"]["subject_mode"], "confirmed")
            self.assertIn("active_subject: finance", text)
            self.assertIn("subject_mode: confirmed", text)
            self.assertIn("updated_by: cli", text)
            self.assertIn("run_id: run-1", text)
            self.assertIn("- event-study", text)
            self.assertIn("- asset-pricing", text)

    def test_dismiss_writes_only_state_file_without_creating_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)

            result = apply_subject_action(root, "dismiss", "finance", source="mcp")

            self.assertFalse(self._manifest_path(root).exists())
            self.assertTrue(self._state_path(root).is_file())
            self.assertFalse(result["manifest_exists"])
            self.assertEqual(result["manifest"]["active_subject"], "auto")
            self.assertIn("finance", result["state"]["dismissed_subjects"])
            self.assertEqual(
                result["state"]["dismissed_subjects"]["finance"]["source"],
                "mcp",
            )
            self.assertEqual(
                result["state"]["dismissed_subjects"]["finance"]["last_suggestion_count"],
                0,
            )
            self.assertEqual(result["state"]["lifecycle_events"][0]["action"], "dismiss")

    def test_dismiss_does_not_create_or_modify_subject_guidance(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)

            apply_subject_action(root, "dismiss", "finance", source="mcp")

            self.assertFalse(self._guidance_path(root).exists())

        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            apply_subject_action(root, "confirm", "finance", source="cli")
            before = self._guidance_path(root).read_text(encoding="utf-8")

            apply_subject_action(root, "dismiss", "economics", source="mcp")

            after = self._guidance_path(root).read_text(encoding="utf-8")
            self.assertEqual(after, before)

    def test_dismiss_stores_last_suggestion_count_from_existing_record(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            self._state_path(root).parent.mkdir(parents=True)
            self._state_path(root).write_text(
                json.dumps(
                    {
                        "schema_version": "1.0",
                        "subjects": {"finance": {"suggestion_count": 3}},
                    }
                ),
                encoding="utf-8",
            )

            result = apply_subject_action(root, "dismiss", "finance", run_id="run-2")

            dismissed = result["state"]["dismissed_subjects"]["finance"]
            self.assertEqual(dismissed["last_suggestion_count"], 3)
            self.assertEqual(dismissed["run_id"], "run-2")

    def test_dismiss_preserves_unknown_top_level_state_fields(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            self._state_path(root).parent.mkdir(parents=True)
            self._state_path(root).write_text(
                json.dumps(
                    {
                        "schema_version": "1.0",
                        "subjects": {"finance": {"suggestion_count": 1}},
                        "future_field": {"keep": True},
                    }
                ),
                encoding="utf-8",
            )

            apply_subject_action(root, "dismiss", "finance")
            stored = json.loads(self._state_path(root).read_text(encoding="utf-8"))

            self.assertIn("future_field", stored)
            self.assertEqual(stored["future_field"], {"keep": True})

    def test_lock_then_unlock_transitions_concrete_subject_to_confirmed(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)

            locked = apply_subject_action(root, "lock", "finance")
            unlocked = apply_subject_action(root, "unlock", source="cli", run_id="unlock-1")

            self.assertEqual(locked["manifest"]["subject_mode"], "locked")
            self.assertEqual(unlocked["manifest"]["active_subject"], "finance")
            self.assertEqual(unlocked["manifest"]["subject_mode"], "confirmed")
            self.assertEqual(unlocked["state"]["lifecycle_events"][-1]["action"], "unlock")
            self.assertIsNone(unlocked["state"]["lifecycle_events"][-1]["subject"])

    def test_lock_materializes_locked_guidance_with_locked_text(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)

            result = apply_subject_action(root, "lock", "finance", source="cli")

            text = self._guidance_path(root).read_text(encoding="utf-8")
            self.assertEqual(result["subject_guidance"]["managed_block"], "active")
            self.assertEqual(result["subject_guidance"]["subject_mode"], "locked")
            self.assertIn("subject_mode: locked", text)
            self.assertIn("Do not automatically replace the active subject.", text)

    def test_unlock_rewrites_locked_guidance_to_confirmed(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            apply_subject_action(root, "lock", "finance", source="cli")

            result = apply_subject_action(root, "unlock", source="cli", run_id="unlock-1")

            text = self._guidance_path(root).read_text(encoding="utf-8")
            self.assertEqual(result["subject_guidance"]["managed_block"], "active")
            self.assertEqual(result["subject_guidance"]["active_subject"], "finance")
            self.assertEqual(result["subject_guidance"]["subject_mode"], "confirmed")
            self.assertIn("active_subject: finance", text)
            self.assertIn("subject_mode: confirmed", text)
            self.assertNotIn("subject_mode: locked", text)
            self.assertNotIn("Do not automatically replace the active subject.", text)
            self.assertIn("run_id: unlock-1", text)

    def test_unlock_core_locked_manifest_returns_to_auto(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            self._manifest_path(root).parent.mkdir(parents=True)
            self._manifest_path(root).write_text(
                "active_subject: core\nsubject_mode: locked\n",
                encoding="utf-8",
            )

            result = apply_subject_action(root, "unlock")

            self.assertEqual(result["manifest"]["active_subject"], "auto")
            self.assertEqual(result["manifest"]["subject_mode"], "auto")

    def test_unlock_auto_or_core_disables_subject_guidance(self) -> None:
        for active_subject in ("auto", "core"):
            with self.subTest(active_subject=active_subject):
                with tempfile.TemporaryDirectory() as tmp_dir:
                    root = Path(tmp_dir)
                    self._guidance_path(root).parent.mkdir(parents=True)
                    self._guidance_path(root).write_text(
                        f"# Qiongli Subject Runtime Guidance\n\n{START_MARKER}\n"
                        "schema_version: 1.0\n"
                        "managed_by: qiongli\n"
                        "active_subject: finance\n"
                        "subject_mode: locked\n"
                        "updated_at: 2026-07-01T12:00:00+00:00\n"
                        f"{END_MARKER}\n",
                        encoding="utf-8",
                    )
                    self._manifest_path(root).write_text(
                        "active_subject: "
                        f"{active_subject}\n"
                        "subject_mode: "
                        f"{'auto' if active_subject == 'auto' else 'locked'}\n",
                        encoding="utf-8",
                    )

                    result = apply_subject_action(root, "unlock", source="cli")

                    text = self._guidance_path(root).read_text(encoding="utf-8")
                    self.assertEqual(result["manifest"]["active_subject"], "auto")
                    self.assertEqual(result["manifest"]["subject_mode"], "auto")
                    self.assertEqual(result["subject_guidance"]["managed_block"], "disabled")
                    self.assertIn("status: disabled", text)
                    self.assertIn("lifecycle_action: unlock", text)

    def test_unlock_rejects_non_empty_subject(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)

            with self.assertRaisesRegex(SubjectLifecycleError, "does not accept a subject"):
                apply_subject_action(root, "unlock", "finance")

    def test_reset_returns_to_auto_and_clears_subject_lifecycle_state(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            self._state_path(root).parent.mkdir(parents=True)
            self._state_path(root).write_text(
                json.dumps(
                    {
                        "schema_version": "1.0",
                        "subjects": {"finance": {"suggestion_count": 2}},
                        "dismissed_subjects": {"finance": {"source": "user"}},
                        "warnings": ["keep this warning"],
                    }
                ),
                encoding="utf-8",
            )
            apply_subject_action(root, "lock", "finance")

            result = apply_subject_action(root, "reset", source="cli")

            self.assertEqual(result["manifest"]["active_subject"], "auto")
            self.assertEqual(result["manifest"]["subject_mode"], "auto")
            self.assertEqual(result["manifest"]["secondary_subjects"], [])
            self.assertEqual(result["manifest"]["venue_profiles"], [])
            self.assertEqual(result["manifest"]["method_lenses"], [])
            self.assertEqual(result["manifest"]["strictness"], "standard")
            self.assertEqual(result["state"]["subjects"], {})
            self.assertEqual(result["state"]["dismissed_subjects"], {})
            self.assertEqual(result["state"]["warnings"], ["keep this warning"])
            self.assertEqual(result["state"]["lifecycle_events"][-1]["action"], "reset")

    def test_reset_disables_subject_guidance(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            apply_subject_action(root, "confirm", "finance", source="cli")

            result = apply_subject_action(root, "reset", source="cli", run_id="reset-1")

            text = self._guidance_path(root).read_text(encoding="utf-8")
            self.assertEqual(result["subject_guidance"]["managed_block"], "disabled")
            self.assertEqual(result["subject_guidance"]["active_subject"], "auto")
            self.assertEqual(result["subject_guidance"]["subject_mode"], "auto")
            self.assertIn("status: disabled", text)
            self.assertIn("lifecycle_action: reset", text)
            self.assertIn("run_id: reset-1", text)

    def test_reset_rejects_non_empty_subject(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)

            with self.assertRaisesRegex(SubjectLifecycleError, "does not accept a subject"):
                apply_subject_action(root, "reset", "finance")

    def test_subjectless_actions_normalize_blank_subject_to_none(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)

            unlocked = apply_subject_action(root, "unlock", "  ")
            reset = apply_subject_action(root, "reset", "\t")

            self.assertIsNone(unlocked["state"]["lifecycle_events"][-1]["subject"])
            self.assertIsNone(reset["state"]["lifecycle_events"][-1]["subject"])

    def test_invalid_subject_is_rejected_for_concrete_subject_actions(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)

            for action in ("confirm", "dismiss", "lock"):
                for subject in ("not-a-subject", "auto", "core"):
                    with self.subTest(action=action, subject=subject):
                        with self.assertRaises(SubjectLifecycleError):
                            apply_subject_action(root, action, subject)

    def test_subject_is_required_for_concrete_subject_actions(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)

            for action in ("confirm", "dismiss", "lock"):
                with self.subTest(action=action):
                    with self.assertRaisesRegex(SubjectLifecycleError, "requires a subject"):
                        apply_subject_action(root, action)

    def test_malformed_subject_evidence_does_not_crash_status_or_action(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            self._state_path(root).parent.mkdir(parents=True)
            self._state_path(root).write_text("{not json", encoding="utf-8")

            status = subject_status(root)
            result = apply_subject_action(root, "dismiss", "finance")
            stored = json.loads(self._state_path(root).read_text(encoding="utf-8"))

            self.assertTrue(
                any(
                    "Invalid subject evidence memory" in warning
                    for warning in status["state"]["warnings"]
                )
            )
            self.assertIn("finance", result["state"]["dismissed_subjects"])
            self.assertIn("finance", stored["dismissed_subjects"])

    def test_guidance_write_error_is_converted_to_lifecycle_error(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            self._manifest_path(root).parent.mkdir(parents=True)
            self._manifest_path(root).write_text(
                "method_lenses:\n"
                f"- {START_MARKER}\n",
                encoding="utf-8",
            )

            with self.assertRaisesRegex(
                SubjectLifecycleError,
                "^Failed to update subject guidance:",
            ):
                apply_subject_action(root, "confirm", "finance", source="cli")

            self.assertFalse(self._state_path(root).exists())

    def test_marker_method_lens_fails_before_manifest_or_evidence_writes(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            self._manifest_path(root).parent.mkdir(parents=True)
            original_manifest = (
                "active_subject: auto\n"
                "subject_mode: auto\n"
                "method_lenses:\n"
                f"- {START_MARKER}\n"
            )
            self._manifest_path(root).write_text(original_manifest, encoding="utf-8")

            with self.assertRaisesRegex(
                SubjectLifecycleError,
                "^Failed to update subject guidance:.*managed marker",
            ):
                apply_subject_action(root, "confirm", "finance", source="cli")

            self.assertEqual(
                self._manifest_path(root).read_text(encoding="utf-8"),
                original_manifest,
            )
            self.assertFalse(self._state_path(root).exists())

    def test_lock_marker_method_lens_fails_before_manifest_or_evidence_writes(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            self._manifest_path(root).parent.mkdir(parents=True)
            original_manifest = (
                "active_subject: auto\n"
                "subject_mode: auto\n"
                "method_lenses:\n"
                f"- {START_MARKER}\n"
            )
            self._manifest_path(root).write_text(original_manifest, encoding="utf-8")

            with self.assertRaisesRegex(
                SubjectLifecycleError,
                "^Failed to update subject guidance:.*managed marker",
            ):
                apply_subject_action(root, "lock", "finance", source="cli")

            self.assertEqual(
                self._manifest_path(root).read_text(encoding="utf-8"),
                original_manifest,
            )
            self.assertFalse(self._state_path(root).exists())

    def test_concrete_unlock_marker_method_lens_fails_before_writes(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            self._manifest_path(root).parent.mkdir(parents=True)
            original_manifest = (
                "active_subject: finance\n"
                "subject_mode: locked\n"
                "method_lenses:\n"
                f"- {START_MARKER}\n"
            )
            self._manifest_path(root).write_text(original_manifest, encoding="utf-8")

            with self.assertRaisesRegex(
                SubjectLifecycleError,
                "^Failed to update subject guidance:.*managed marker",
            ):
                apply_subject_action(root, "unlock", source="cli")

            self.assertEqual(
                self._manifest_path(root).read_text(encoding="utf-8"),
                original_manifest,
            )
            self.assertFalse(self._state_path(root).exists())

    def test_invalid_existing_guidance_fails_before_manifest_or_evidence_writes(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            self._guidance_path(root).parent.mkdir(parents=True)
            original_guidance = f"{END_MARKER}\n{START_MARKER}\n"
            self._guidance_path(root).write_text(original_guidance, encoding="utf-8")

            with self.assertRaisesRegex(
                SubjectLifecycleError,
                "^Failed to update subject guidance:",
            ):
                apply_subject_action(root, "confirm", "finance", source="cli")

            self.assertFalse(self._manifest_path(root).exists())
            self.assertFalse(self._state_path(root).exists())
            self.assertEqual(
                self._guidance_path(root).read_text(encoding="utf-8"),
                original_guidance,
            )

    def test_symlinked_manifest_path_fails_before_lifecycle_writes(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            outside_manifest = root / "outside-manifest.yaml"
            original_manifest = "active_subject: auto\nsubject_mode: auto\n"
            outside_manifest.write_text(original_manifest, encoding="utf-8")
            self._manifest_path(root).parent.mkdir(parents=True)
            try:
                self._manifest_path(root).symlink_to(outside_manifest)
            except OSError as exc:
                self.skipTest(f"symlink creation unavailable: {exc}")

            with self.assertRaisesRegex(
                SubjectLifecycleError,
                r"symlink.*\.qiongli/guidance_manifest\.yaml",
            ):
                apply_subject_action(root, "confirm", "finance", source="cli")

            self.assertEqual(outside_manifest.read_text(encoding="utf-8"), original_manifest)
            self.assertFalse(self._state_path(root).exists())
            self.assertFalse(self._guidance_path(root).exists())

    def test_symlinked_evidence_path_fails_before_lifecycle_writes(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            outside_state = root / "outside-subject-evidence.json"
            original_state = json.dumps({"schema_version": "1.0", "subjects": {}})
            outside_state.write_text(original_state, encoding="utf-8")
            self._state_path(root).parent.mkdir(parents=True)
            try:
                self._state_path(root).symlink_to(outside_state)
            except OSError as exc:
                self.skipTest(f"symlink creation unavailable: {exc}")

            with self.assertRaisesRegex(
                SubjectLifecycleError,
                r"symlink.*\.qiongli/trace/subject_evidence\.json",
            ):
                apply_subject_action(root, "dismiss", "finance", source="cli")

            self.assertEqual(outside_state.read_text(encoding="utf-8"), original_state)
            self.assertFalse(self._manifest_path(root).exists())
            self.assertFalse(self._guidance_path(root).exists())

    @staticmethod
    def _manifest_path(root: Path) -> Path:
        return root / ".qiongli" / "guidance_manifest.yaml"

    @staticmethod
    def _state_path(root: Path) -> Path:
        return root / ".qiongli" / "trace" / "subject_evidence.json"

    @staticmethod
    def _guidance_path(root: Path) -> Path:
        return root / SUBJECT_GUIDANCE_REL


if __name__ == "__main__":
    unittest.main()
