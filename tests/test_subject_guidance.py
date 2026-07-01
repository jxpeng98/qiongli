from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from bridges.subject_guidance import (
    END_MARKER,
    START_MARKER,
    SUBJECT_GUIDANCE_REL,
    SubjectGuidanceError,
    disable_subject_guidance,
    inspect_subject_guidance,
    write_subject_guidance,
)


class SubjectGuidanceTests(unittest.TestCase):
    def test_inspect_missing_subject_guidance_does_not_create_files(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)

            status = inspect_subject_guidance(root)

            self.assertFalse(status["exists"])
            self.assertEqual(status["managed_block"], "missing")
            self.assertEqual(status["path"], ".qiongli/guidance.d/subject-runtime.md")
            self.assertFalse((root / ".qiongli").exists())

    def test_write_confirmed_subject_guidance_creates_active_block(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)

            status = write_subject_guidance(
                root,
                active_subject="finance",
                subject_mode="confirmed",
                lifecycle_action="confirm",
                source="cli",
                run_id="run-1",
                method_lenses=["event-study", "asset-pricing"],
                resource_activation_plan={
                    "levels": ["core", "subject_overlay", "subject_skill", "method_pack"]
                },
            )

            path = root / SUBJECT_GUIDANCE_REL
            text = path.read_text(encoding="utf-8")
            self.assertTrue(path.is_file())
            self.assertEqual(status["managed_block"], "active")
            self.assertEqual(status["active_subject"], "finance")
            self.assertEqual(status["subject_mode"], "confirmed")
            self.assertTrue(status["updated_at"].endswith("+00:00"))
            self.assertEqual(status["warnings"], [])
            self.assertIn(START_MARKER, text)
            self.assertIn(END_MARKER, text)
            self.assertIn("active_subject: finance", text)
            self.assertIn("subject_mode: confirmed", text)
            self.assertIn("updated_by: cli", text)
            self.assertIn("lifecycle_action: confirm", text)
            self.assertIn("run_id: run-1", text)
            self.assertIn("- event-study", text)
            self.assertIn("- asset-pricing", text)
            self.assertIn("- core: active", text)
            self.assertIn("- subject_overlay: confirmed", text)
            self.assertIn("- subject_skill: confirmed", text)
            self.assertIn("- method_pack: confirmed", text)
            text.encode("ascii")

    def test_write_locked_subject_guidance_marks_replacement_protection(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)

            write_subject_guidance(
                root,
                active_subject="economics",
                subject_mode="locked",
                lifecycle_action="lock",
                source="mcp",
            )

            text = (root / SUBJECT_GUIDANCE_REL).read_text(encoding="utf-8")
            self.assertIn("active_subject: economics", text)
            self.assertIn("subject_mode: locked", text)
            self.assertIn("Do not automatically replace the active subject.", text)

    def test_disable_subject_guidance_writes_disabled_block(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            write_subject_guidance(
                root,
                active_subject="finance",
                subject_mode="confirmed",
                lifecycle_action="confirm",
                source="cli",
            )

            status = disable_subject_guidance(root, lifecycle_action="reset", source="cli")

            text = (root / SUBJECT_GUIDANCE_REL).read_text(encoding="utf-8")
            self.assertEqual(status["managed_block"], "disabled")
            self.assertEqual(status["active_subject"], "auto")
            self.assertEqual(status["subject_mode"], "auto")
            self.assertIn("active_subject: auto", text)
            self.assertIn("subject_mode: auto", text)
            self.assertIn("status: disabled", text)
            self.assertIn("Use adaptive core inference for future runs.", text)

    def test_rewrite_preserves_user_text_outside_managed_block(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            path = root / SUBJECT_GUIDANCE_REL
            path.parent.mkdir(parents=True)
            path.write_text(
                "user prefix\n"
                f"{START_MARKER}\nold block\n{END_MARKER}\n"
                "user suffix\n",
                encoding="utf-8",
            )

            write_subject_guidance(
                root,
                active_subject="finance",
                subject_mode="confirmed",
                lifecycle_action="confirm",
                source="cli",
            )

            text = path.read_text(encoding="utf-8")
            self.assertTrue(text.startswith("user prefix\n"))
            self.assertTrue(text.rstrip().endswith("user suffix"))
            self.assertNotIn("old block", text)
            self.assertEqual(text.count(START_MARKER), 1)
            self.assertEqual(text.count(END_MARKER), 1)

    def test_append_managed_block_when_user_file_has_no_markers(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            path = root / SUBJECT_GUIDANCE_REL
            path.parent.mkdir(parents=True)
            path.write_text("# User Subject Notes\n\n- Keep this note.\n", encoding="utf-8")

            status = write_subject_guidance(
                root,
                active_subject="finance",
                subject_mode="confirmed",
                lifecycle_action="confirm",
                source="cli",
            )

            text = path.read_text(encoding="utf-8")
            self.assertEqual(status["managed_block"], "appended")
            self.assertIn("# User Subject Notes", text)
            self.assertIn("- Keep this note.", text)
            self.assertIn(START_MARKER, text)

    def test_multiple_managed_blocks_fail_closed(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            path = root / SUBJECT_GUIDANCE_REL
            path.parent.mkdir(parents=True)
            path.write_text(
                f"{START_MARKER}\none\n{END_MARKER}\n"
                f"{START_MARKER}\ntwo\n{END_MARKER}\n",
                encoding="utf-8",
            )

            with self.assertRaisesRegex(SubjectGuidanceError, "multiple managed blocks"):
                write_subject_guidance(
                    root,
                    active_subject="finance",
                    subject_mode="confirmed",
                    lifecycle_action="confirm",
                    source="cli",
                )

            self.assertEqual(path.read_text(encoding="utf-8").count(START_MARKER), 2)

    def test_invalid_marker_order_is_reported_and_fails_closed(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            path = root / SUBJECT_GUIDANCE_REL
            path.parent.mkdir(parents=True)
            path.write_text(f"{END_MARKER}\ninvalid\n{START_MARKER}\n", encoding="utf-8")

            status = inspect_subject_guidance(root)

            self.assertTrue(status["exists"])
            self.assertEqual(status["managed_block"], "invalid")
            self.assertTrue(
                any("invalid marker order" in warning for warning in status["warnings"])
            )
            with self.assertRaisesRegex(SubjectGuidanceError, "invalid marker order"):
                write_subject_guidance(
                    root,
                    active_subject="finance",
                    subject_mode="confirmed",
                    lifecycle_action="confirm",
                    source="cli",
                )
            with self.assertRaisesRegex(SubjectGuidanceError, "invalid marker order"):
                disable_subject_guidance(root, lifecycle_action="reset", source="cli")
