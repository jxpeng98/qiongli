from __future__ import annotations

import copy
import subprocess
import tempfile
import unittest
from pathlib import Path

from scripts.validate_2x_branch_point import (
    DEFAULT_RECORD,
    REPO_ROOT,
    git_blob_bytes,
    load_record,
    validate_record,
)


class BranchPointEvidenceTests(unittest.TestCase):
    def test_repository_branch_point_evidence_is_valid(self) -> None:
        self.assertEqual(validate_record(REPO_ROOT, load_record(DEFAULT_RECORD)), [])

    def test_changed_run_head_is_rejected(self) -> None:
        record = copy.deepcopy(load_record(DEFAULT_RECORD))
        record["branch_validation"]["ci"]["head_sha"] = "0" * 40
        errors = validate_record(REPO_ROOT, record)
        self.assertTrue(any("head_sha" in error for error in errors))

    def test_changed_run_id_and_url_are_rejected(self) -> None:
        record = copy.deepcopy(load_record(DEFAULT_RECORD))
        record["source_validation"]["ci"]["run_id"] += 1
        errors = validate_record(REPO_ROOT, record)
        self.assertTrue(any("run_id" in error for error in errors))
        self.assertTrue(any("URL does not match" in error for error in errors))

    def test_changed_ruleset_is_rejected(self) -> None:
        record = copy.deepcopy(load_record(DEFAULT_RECORD))
        record["protection"]["native_development"]["ruleset_id"] += 1
        errors = validate_record(REPO_ROOT, record)
        self.assertTrue(any("ruleset_id" in error for error in errors))
        self.assertTrue(any("URL does not match" in error for error in errors))

    def test_changed_manifest_digest_is_rejected(self) -> None:
        record = copy.deepcopy(load_record(DEFAULT_RECORD))
        record["baseline"]["manifest_sha256"] = "0" * 64
        errors = validate_record(REPO_ROOT, record)
        self.assertTrue(any("manifest_sha256" in error for error in errors))
        self.assertTrue(any("manifest SHA-256" in error for error in errors))

    def test_malformed_nested_evidence_returns_errors_instead_of_crashing(self) -> None:
        record = copy.deepcopy(load_record(DEFAULT_RECORD))
        record["branch_validation"] = []
        errors = validate_record(REPO_ROOT, record)
        self.assertTrue(errors)
        self.assertTrue(any("workflow evidence" in error for error in errors))

    def test_schema_invalid_extra_ruleset_field_is_rejected(self) -> None:
        record = copy.deepcopy(load_record(DEFAULT_RECORD))
        record["protection"]["dev"]["unexpected"] = True
        errors = validate_record(REPO_ROOT, record)
        self.assertTrue(any("protection.dev keys differ" in error for error in errors))

    def test_schema_invalid_policy_text_is_rejected(self) -> None:
        record = copy.deepcopy(load_record(DEFAULT_RECORD))
        record["branch_validation_policy"]["evidence_binding"] = 42
        errors = validate_record(REPO_ROOT, record)
        self.assertTrue(any("evidence_binding" in error for error in errors))

    def test_git_blob_hash_is_independent_of_worktree_line_endings(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            subprocess.run(["git", "init", str(root)], check=True, capture_output=True)
            subprocess.run(
                ["git", "-C", str(root), "config", "user.email", "test@example.com"],
                check=True,
            )
            subprocess.run(
                ["git", "-C", str(root), "config", "user.name", "Qiongli Test"],
                check=True,
            )
            path = root / "manifest.json"
            canonical = b'{\n  "value": true\n}\n'
            path.write_bytes(canonical)
            subprocess.run(
                ["git", "-C", str(root), "add", "manifest.json"], check=True
            )
            subprocess.run(
                ["git", "-C", str(root), "commit", "-m", "test fixture"],
                check=True,
                capture_output=True,
            )
            path.write_bytes(canonical.replace(b"\n", b"\r\n"))
            self.assertEqual(git_blob_bytes(root, "manifest.json"), canonical)


if __name__ == "__main__":
    unittest.main()
