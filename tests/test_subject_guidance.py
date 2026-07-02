from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from unittest import mock

from bridges import subject_guidance as subject_guidance_module
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
            write_subject_guidance(
                root,
                active_subject="economics",
                subject_mode="locked",
                lifecycle_action="lock",
                source="cli",
            )
            valid_managed_text = path.read_text(encoding="utf-8")
            path.write_text(
                f"user prefix\n{valid_managed_text}user suffix\n",
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
            self.assertNotIn("active_subject: economics", text)
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

    def test_write_converts_oserror_to_subject_guidance_error(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)

            with mock.patch.object(Path, "write_text", side_effect=OSError("disk full")):
                with self.assertRaisesRegex(
                    SubjectGuidanceError,
                    r"\.qiongli/guidance\.d/subject-runtime\.md.*disk full",
                ):
                    write_subject_guidance(
                        root,
                        active_subject="finance",
                        subject_mode="confirmed",
                        lifecycle_action="confirm",
                        source="cli",
                    )

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

    def test_malformed_managed_block_metadata_inspects_as_invalid(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            path = root / SUBJECT_GUIDANCE_REL
            path.parent.mkdir(parents=True)
            path.write_text(f"{START_MARKER}\nold block\n{END_MARKER}\n", encoding="utf-8")

            status = inspect_subject_guidance(root)

            self.assertTrue(status["exists"])
            self.assertEqual(status["managed_block"], "invalid")
            self.assertIsNone(status["active_subject"])
            self.assertIsNone(status["subject_mode"])
            self.assertIsNone(status["updated_at"])
            self.assertTrue(status["warnings"])

    def test_malformed_managed_block_metadata_fails_closed_for_writes(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            path = root / SUBJECT_GUIDANCE_REL
            path.parent.mkdir(parents=True)
            path.write_text(f"{START_MARKER}\nold block\n{END_MARKER}\n", encoding="utf-8")

            with self.assertRaisesRegex(SubjectGuidanceError, "invalid managed metadata"):
                write_subject_guidance(
                    root,
                    active_subject="finance",
                    subject_mode="confirmed",
                    lifecycle_action="confirm",
                    source="cli",
                )
            with self.assertRaisesRegex(SubjectGuidanceError, "invalid managed metadata"):
                disable_subject_guidance(root, lifecycle_action="reset", source="cli")

            self.assertEqual(
                path.read_text(encoding="utf-8"),
                f"{START_MARKER}\nold block\n{END_MARKER}\n",
            )

    def test_write_rejects_symlinked_qiongli_dir_without_creating_outside_target(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            base = Path(tmp_dir)
            root = base / "project"
            root.mkdir()
            outside_qiongli = base / "outside-qiongli"
            outside_qiongli.mkdir()
            self._symlink_or_skip(
                outside_qiongli,
                root / ".qiongli",
                target_is_directory=True,
            )

            with self.assertRaisesRegex(SubjectGuidanceError, "symlink.*\\.qiongli"):
                write_subject_guidance(
                    root,
                    active_subject="finance",
                    subject_mode="confirmed",
                    lifecycle_action="confirm",
                    source="cli",
                )

            self.assertFalse((outside_qiongli / "guidance.d" / "subject-runtime.md").exists())

    def test_disable_rejects_symlinked_guidance_dir_without_mutating_target(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            base = Path(tmp_dir)
            root = base / "project"
            root.mkdir()
            (root / ".qiongli").mkdir()
            outside_guidance_dir = base / "outside-guidance"
            outside_guidance_dir.mkdir()
            outside_file = outside_guidance_dir / "subject-runtime.md"
            outside_file.write_text("outside original\n", encoding="utf-8")
            self._symlink_or_skip(
                outside_guidance_dir,
                root / ".qiongli" / "guidance.d",
                target_is_directory=True,
            )

            with self.assertRaisesRegex(SubjectGuidanceError, "symlink.*guidance\\.d"):
                disable_subject_guidance(root, lifecycle_action="reset", source="cli")

            self.assertEqual(outside_file.read_text(encoding="utf-8"), "outside original\n")

    def test_write_rejects_symlinked_guidance_file_without_mutating_target(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            base = Path(tmp_dir)
            root = base / "project"
            root.mkdir()
            path = root / SUBJECT_GUIDANCE_REL
            path.parent.mkdir(parents=True)
            outside_file = base / "outside-runtime.md"
            outside_file.write_text("outside original\n", encoding="utf-8")
            self._symlink_or_skip(outside_file, path)

            with self.assertRaisesRegex(SubjectGuidanceError, "symlink.*subject-runtime\\.md"):
                write_subject_guidance(
                    root,
                    active_subject="finance",
                    subject_mode="confirmed",
                    lifecycle_action="confirm",
                    source="cli",
                )

            self.assertEqual(outside_file.read_text(encoding="utf-8"), "outside original\n")

    def test_write_rejects_managed_marker_in_method_lenses_without_creating_file(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)

            with self.assertRaisesRegex(SubjectGuidanceError, "managed marker"):
                write_subject_guidance(
                    root,
                    active_subject="finance",
                    subject_mode="confirmed",
                    lifecycle_action="confirm",
                    source="cli",
                    method_lenses=[START_MARKER],
                )

            self.assertFalse((root / SUBJECT_GUIDANCE_REL).exists())

    def test_write_rejects_managed_marker_in_resource_levels_without_mutation(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            write_subject_guidance(
                root,
                active_subject="finance",
                subject_mode="confirmed",
                lifecycle_action="confirm",
                source="cli",
            )
            path = root / SUBJECT_GUIDANCE_REL
            original_text = path.read_text(encoding="utf-8")

            with self.assertRaisesRegex(SubjectGuidanceError, "managed marker"):
                write_subject_guidance(
                    root,
                    active_subject="finance",
                    subject_mode="confirmed",
                    lifecycle_action="confirm",
                    source="cli",
                    resource_activation_plan={
                        "levels": {
                            f"subject_overlay {END_MARKER}": "confirmed",
                            "method_pack": START_MARKER,
                        }
                    },
                )

            self.assertEqual(path.read_text(encoding="utf-8"), original_text)

    def test_write_rejects_managed_marker_in_subject_mode_without_creating_file(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)

            with self.assertRaisesRegex(SubjectGuidanceError, "managed marker"):
                write_subject_guidance(
                    root,
                    active_subject="finance",
                    subject_mode=START_MARKER,
                    lifecycle_action="confirm",
                    source="cli",
                    resource_activation_plan={"levels": ["subject_overlay"]},
                )

            self.assertFalse((root / SUBJECT_GUIDANCE_REL).exists())

    def test_resource_activation_rejects_managed_marker_in_subject_mode(self) -> None:
        with self.assertRaisesRegex(SubjectGuidanceError, "managed marker"):
            subject_guidance_module._render_resource_activation(  # noqa: SLF001
                {"levels": ["subject_overlay"]},
                subject_mode=START_MARKER,
            )

    def _symlink_or_skip(
        self,
        target: Path,
        link: Path,
        *,
        target_is_directory: bool = False,
    ) -> None:
        try:
            link.symlink_to(target, target_is_directory=target_is_directory)
        except OSError as exc:
            self.skipTest(f"symlink unavailable: {exc}")
