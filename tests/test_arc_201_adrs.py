from __future__ import annotations

import copy
import tempfile
import unittest
from pathlib import Path

from scripts.validate_arc_201_adrs import (
    DEFAULT_CURRENT_RECORD,
    DEFAULT_RECORD,
    REPO_ROOT,
    is_canonical_repository_path,
    load_record,
    resolve_adr_path,
    validate_adr,
    validate_current_record,
    validate_record,
)


class Arc201DecisionTests(unittest.TestCase):
    def test_repository_decision_set_is_complete(self) -> None:
        errors = validate_record(REPO_ROOT, load_record(DEFAULT_RECORD))
        self.assertEqual(errors, [])

    def test_current_architecture_overview_is_truthful(self) -> None:
        overview = " ".join(
            (REPO_ROOT / "docs/architecture.md")
            .read_text(encoding="utf-8")
            .split()
        )
        for statement in (
            "Rust-native product",
            "Tauri 2 / Svelte 5",
            "Native services own",
            "ADR 0210",
            "ADR 0211",
            "ADR 0217",
            "ADR 0218",
            "independent CLI build/package acceptance remains open",
            "Same-device collaboration precedes any optional cross-device synchronization",
        ):
            with self.subTest(statement=statement):
                self.assertIn(statement, overview)

    def test_current_decision_registry_is_complete(self) -> None:
        errors = validate_current_record(
            REPO_ROOT, load_record(DEFAULT_CURRENT_RECORD)
        )
        self.assertEqual(errors, [])

    def test_2x_evaluation_ci_validates_architecture_truth(self) -> None:
        workflow = (REPO_ROOT / ".github/workflows/evaluation-truth.yml").read_text(
            encoding="utf-8"
        )
        self.assertIn("python scripts/validate_arc_201_adrs.py", workflow)
        self.assertIn("tests.test_arc_201_adrs", workflow)
        self.assertIn("tests.test_frozen_2x_architecture_baseline", workflow)

    def test_current_registry_missing_or_extra_decision_is_rejected(self) -> None:
        for mutation in ("missing", "extra"):
            with self.subTest(mutation=mutation):
                record = copy.deepcopy(load_record(DEFAULT_CURRENT_RECORD))
                if mutation == "missing":
                    record["decisions"].pop()
                else:
                    record["decisions"].append(copy.deepcopy(record["decisions"][-1]))
                errors = validate_current_record(REPO_ROOT, record)
                self.assertTrue(
                    any(
                        "exactly match every numbered ADR" in error
                        for error in errors
                    )
                )

    def test_current_registry_duplicate_identity_is_rejected(self) -> None:
        record = copy.deepcopy(load_record(DEFAULT_CURRENT_RECORD))
        record["decisions"][1]["adr_number"] = "0201"
        record["decisions"][1]["task_id"] = "ARC-201A"
        errors = validate_current_record(REPO_ROOT, record)
        self.assertTrue(any("duplicate ADR number 0201" in error for error in errors))
        self.assertTrue(any("duplicate task ID ARC-201A" in error for error in errors))

    def test_current_registry_reordered_decisions_are_rejected(self) -> None:
        record = copy.deepcopy(load_record(DEFAULT_CURRENT_RECORD))
        record["decisions"][0], record["decisions"][1] = (
            record["decisions"][1],
            record["decisions"][0],
        )
        errors = validate_current_record(REPO_ROOT, record)
        self.assertTrue(any("filename order" in error for error in errors))
        self.assertTrue(any("ascending order" in error for error in errors))

    def test_current_registry_path_mismatch_is_rejected(self) -> None:
        record = copy.deepcopy(load_record(DEFAULT_CURRENT_RECORD))
        record["decisions"][7]["path"] = record["decisions"][8]["path"]
        errors = validate_current_record(REPO_ROOT, record)
        self.assertTrue(any("ADR number must match its filename" in error for error in errors))
        self.assertTrue(any("filename order" in error for error in errors))

    def test_current_registry_metadata_mismatch_is_rejected(self) -> None:
        record = copy.deepcopy(load_record(DEFAULT_CURRENT_RECORD))
        record["decisions"][-1]["title"] = "Wrong title"
        record["decisions"][-1]["task_id"] = "WRONG-1"
        record["decisions"][-1]["status"] = "Proposed"
        errors = validate_current_record(REPO_ROOT, record)
        self.assertTrue(any("title must be" in error for error in errors))
        self.assertTrue(any("task metadata" in error for error in errors))
        self.assertTrue(any("ADR status must be Accepted" in error for error in errors))

    def test_current_registry_rejects_unknown_fields(self) -> None:
        record = copy.deepcopy(load_record(DEFAULT_CURRENT_RECORD))
        record["unexpected"] = True
        record["decisions"][0]["unexpected"] = True
        errors = validate_current_record(REPO_ROOT, record)
        self.assertEqual(sum("must contain exactly" in error for error in errors), 2)

    def test_duplicate_task_is_rejected(self) -> None:
        record = copy.deepcopy(load_record(DEFAULT_RECORD))
        record["decisions"][1]["task_id"] = "ARC-201A"
        errors = validate_record(REPO_ROOT, record)
        self.assertTrue(any("ARC-201A through ARC-201G" in error for error in errors))

    def test_duplicate_or_wrong_adr_number_is_rejected(self) -> None:
        record = copy.deepcopy(load_record(DEFAULT_RECORD))
        record["decisions"][1]["adr_number"] = "0201"
        errors = validate_record(REPO_ROOT, record)
        self.assertTrue(any("adr_number must be 0202" in error for error in errors))
        self.assertTrue(any("duplicate ADR number 0201" in error for error in errors))

    def test_wrong_adr_filename_is_rejected(self) -> None:
        record = copy.deepcopy(load_record(DEFAULT_RECORD))
        record["decisions"][1]["path"] = (
            "docs/architecture/decisions/0201-executable-topology.md"
        )
        errors = validate_record(REPO_ROOT, record)
        self.assertTrue(any("ADR path must be" in error for error in errors))

    def test_missing_required_section_is_rejected(self) -> None:
        entry = copy.deepcopy(load_record(DEFAULT_RECORD)["decisions"][1])
        source = REPO_ROOT / entry["path"]
        content = source.read_text(encoding="utf-8").replace(
            "## Rollback", "## Recovery", 1
        )
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "decision.md"
            path.write_text(content, encoding="utf-8")
            errors = validate_adr(path, entry)
        self.assertTrue(any("## Rollback" in error for error in errors))

    def test_placeholder_is_rejected(self) -> None:
        entry = copy.deepcopy(load_record(DEFAULT_RECORD)["decisions"][1])
        source = REPO_ROOT / entry["path"]
        content = source.read_text(encoding="utf-8") + "\ntodo: revisit.\n"
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "decision.md"
            path.write_text(content, encoding="utf-8")
            errors = validate_adr(path, entry)
        self.assertTrue(any("placeholder" in error for error in errors))

    def test_invalid_calendar_date_is_rejected(self) -> None:
        entry = copy.deepcopy(load_record(DEFAULT_RECORD)["decisions"][1])
        source = REPO_ROOT / entry["path"]
        content = source.read_text(encoding="utf-8").replace(
            "- Date: 2026-07-11", "- Date: 2026-02-30", 1
        )
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "decision.md"
            path.write_text(content, encoding="utf-8")
            errors = validate_adr(path, entry)
        self.assertTrue(any("not a real date" in error for error in errors))

    def test_cross_platform_absolute_and_noncanonical_paths_are_rejected(self) -> None:
        invalid = (
            "/home/user/decision.md",
            "C:\\Users\\user\\decision.md",
            "\\\\server\\share\\decision.md",
            "docs/architecture/decisions/../outside.md",
            "docs\\architecture\\decisions\\0201.md",
            "docs//architecture/decisions/0201.md",
        )
        for path in invalid:
            with self.subTest(path=path):
                self.assertFalse(is_canonical_repository_path(path))

    def test_symlinked_adr_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            decision_root = root / "docs" / "architecture" / "decisions"
            decision_root.mkdir(parents=True)
            outside = root / "outside.md"
            outside.write_text("outside", encoding="utf-8")
            link = decision_root / "0202-rust-native-ui-and-accessibility.md"
            try:
                link.symlink_to(outside)
            except (OSError, NotImplementedError):
                self.skipTest("symlink creation is unavailable on this platform")
            with self.assertRaisesRegex(ValueError, "symbolic link"):
                resolve_adr_path(
                    root,
                    "docs/architecture/decisions/"
                    "0202-rust-native-ui-and-accessibility.md",
                )

    def test_symlinked_decisions_directory_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as repository_directory:
            with tempfile.TemporaryDirectory() as outside_directory:
                root = Path(repository_directory)
                architecture = root / "docs" / "architecture"
                architecture.mkdir(parents=True)
                outside = Path(outside_directory)
                filename = "0202-rust-native-ui-and-accessibility.md"
                (outside / filename).write_text("outside", encoding="utf-8")
                try:
                    (architecture / "decisions").symlink_to(
                        outside, target_is_directory=True
                    )
                except (OSError, NotImplementedError):
                    self.skipTest("directory symlinks are unavailable on this platform")
                with self.assertRaisesRegex(ValueError, "symbolic-link component"):
                    resolve_adr_path(
                        root, f"docs/architecture/decisions/{filename}"
                    )


if __name__ == "__main__":
    unittest.main()
