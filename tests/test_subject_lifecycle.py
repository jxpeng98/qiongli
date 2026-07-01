from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

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

    @staticmethod
    def _manifest_path(root: Path) -> Path:
        return root / ".qiongli" / "guidance_manifest.yaml"

    @staticmethod
    def _state_path(root: Path) -> Path:
        return root / ".qiongli" / "trace" / "subject_evidence.json"


if __name__ == "__main__":
    unittest.main()
