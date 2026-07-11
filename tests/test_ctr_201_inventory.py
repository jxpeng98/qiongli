from __future__ import annotations

import copy
import io
import json
import os
import subprocess
import sys
import tempfile
import unittest
from contextlib import redirect_stderr, redirect_stdout
from pathlib import Path
from unittest.mock import patch

from tooling.scripts.validate_capability_contract import validate_instance
from tooling.scripts.validate_ctr_201_inventory import (
    InventoryConfigError,
    canonical_payload_sha256,
    load_inventory_documents,
    main,
    validate_inventory,
)


REPO_ROOT = Path(__file__).resolve().parents[1]


class Ctr201InventoryTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.record, cls.schema = load_inventory_documents(REPO_ROOT)

    def _record(self) -> dict[str, object]:
        return copy.deepcopy(self.record)

    @staticmethod
    def _rehash(record: dict[str, object]) -> None:
        integrity = record["integrity"]
        assert isinstance(integrity, dict)
        integrity["payload_sha256"] = canonical_payload_sha256(record)

    def _validate_rehashed(self, record: dict[str, object]) -> list[str]:
        self._rehash(record)
        return validate_inventory(REPO_ROOT, record, self.schema)

    def test_inventory_matches_closed_schema_and_all_semantic_checks(self) -> None:
        self.assertEqual(validate_instance(self.record, self.schema), [])
        self.assertEqual(validate_inventory(REPO_ROOT, self.record, self.schema), [])
        self.assertEqual(self.record["status"], "in-progress")
        self.assertEqual(self.record["completion"]["ctr_201"], "in-progress")
        self.assertEqual(self.record["completion"]["fnd_202"], "not-implemented")
        self.assertFalse(self.record["completion"]["completion_ready"])

    def test_inventory_binds_exact_frozen_a8_and_contract_pilot_facts(self) -> None:
        source = self.record["frozen_source"]
        self.assertEqual(source["accepted_tag"], "v1.19.0-beta.1")
        self.assertEqual(
            source["accepted_commit"],
            "8d2e99866ce4c4efb8b3b5e0265c0c1f89a36b0f",
        )
        self.assertEqual(source["content_tree"]["file_count"], 377)
        contract = self.record["contract_v2"]
        self.assertEqual(
            (
                contract["status"],
                contract["canonical_tool_count"],
                contract["public_name_count"],
                contract["target_canonical_tool_count"],
                contract["target_public_name_count"],
            ),
            ("pilot", 6, 7, 23, 24),
        )

    def test_runtime_surfaces_and_node_only_legacy_names_are_exact(self) -> None:
        mcp = self.record["mcp"]
        self.assertEqual(
            [surface["oracle_id"] for surface in mcp["runtime_surfaces"]],
            ["node-mcpb", "python-full", "rust-lite"],
        )
        self.assertEqual(
            [surface["public_name_count"] for surface in mcp["runtime_surfaces"]],
            [12, 22, 12],
        )
        self.assertEqual(len(mcp["target_public_names"]), 24)
        self.assertEqual(len(mcp["target_canonical_names"]), 23)
        self.assertEqual(
            [entry["public_name"] for entry in mcp["legacy_only"]],
            ["qiongli_zotero_search", "qiongli_zotero_upsert_references"],
        )
        self.assertTrue(
            all(entry["disposition"] == "pending-LEG-201" for entry in mcp["legacy_only"])
        )

    def test_cli_orchestrator_and_content_gaps_block_completion(self) -> None:
        self.assertEqual(self.record["cli"]["status"], "incomplete")
        self.assertEqual(self.record["orchestrator"]["status"], "incomplete")
        self.assertEqual(
            self.record["cli"]["captured_oracle_cases"],
            ["python.cli-align", "python.installer-dry-run"],
        )
        self.assertIn(
            "complete-dry-run-semantics",
            self.record["cli"]["required_not_fully_captured"],
        )
        self.assertIn(
            "duo-mode-preview",
            self.record["orchestrator"]["captured_scope"],
        )
        self.assertIn(
            "all-solo-duo-triad-modes",
            self.record["orchestrator"]["required_not_fully_captured"],
        )
        self.assertFalse(self.record["cli"]["completion_ready"])
        self.assertFalse(self.record["orchestrator"]["completion_ready"])
        self.assertFalse(self.record["content"]["completion_ready"])
        self.assertEqual(
            [profile["status"] for profile in self.record["content"]["profiles"]],
            ["not-ready", "not-ready", "not-ready"],
        )
        self.assertEqual(self.record["content"]["materialization"]["status"], "not-ready")

    def test_canonical_payload_hash_excludes_integrity_and_is_key_order_stable(self) -> None:
        expected = self.record["integrity"]["payload_sha256"]
        self.assertEqual(canonical_payload_sha256(self.record), expected)
        reordered = dict(reversed(list(self.record.items())))
        reordered_integrity = dict(reordered["integrity"])
        reordered_integrity["payload_sha256"] = "f" * 64
        reordered["integrity"] = reordered_integrity
        self.assertEqual(canonical_payload_sha256(reordered), expected)

    def test_rejects_frozen_manifest_tampering_even_with_recomputed_payload_hash(self) -> None:
        record = self._record()
        record["frozen_source"]["manifest_sha256"] = "a" * 64
        errors = self._validate_rehashed(record)
        self.assertTrue(any("accepted A8 digest" in error for error in errors), errors)

    def test_rejects_oracle_surface_tampering_even_with_recomputed_payload_hash(self) -> None:
        record = self._record()
        python_surface = record["mcp"]["runtime_surfaces"][1]
        python_surface["public_names"] = python_surface["public_names"][:-1]
        python_surface["public_name_count"] -= 1
        errors = self._validate_rehashed(record)
        self.assertTrue(any("python-full public MCP surface" in error for error in errors), errors)

    def test_rejects_completion_overclaim(self) -> None:
        record = self._record()
        record["status"] = "complete"
        record["completion"]["ctr_201"] = "complete"
        record["completion"]["fnd_202"] = "implemented"
        record["completion"]["completion_ready"] = True
        errors = self._validate_rehashed(record)
        self.assertTrue(errors)
        self.assertIn("inventory record does not satisfy its closed schema", errors)

    def test_rejects_path_traversal_without_echoing_the_path(self) -> None:
        record = self._record()
        malicious = "tooling/migration/../../private/secret.json"
        record["frozen_source"]["manifest_path"] = malicious
        errors = self._validate_rehashed(record)
        self.assertTrue(errors)
        self.assertFalse(any(malicious in error for error in errors))

    def test_rejects_duplicate_public_name(self) -> None:
        record = self._record()
        names = record["mcp"]["target_public_names"]
        names.append(names[0])
        errors = self._validate_rehashed(record)
        self.assertTrue(errors)
        self.assertIn("inventory record does not satisfy its closed schema", errors)

    def test_rejects_duplicate_content_root_with_semantic_guard(self) -> None:
        record = self._record()
        roots = record["content"]["resource_roots"]
        roots[-1] = copy.deepcopy(roots[0])
        errors = self._validate_rehashed(record)
        self.assertTrue(
            any("resource roots" in error or "duplicate source" in error for error in errors),
            errors,
        )

    def test_rejects_payload_hash_tampering(self) -> None:
        record = self._record()
        record["integrity"]["payload_sha256"] = "0" * 64
        errors = validate_inventory(REPO_ROOT, record, self.schema)
        self.assertIn("inventory canonical payload digest does not match", errors)

    def test_rejects_schema_weakening(self) -> None:
        schema = copy.deepcopy(self.schema)
        schema["additionalProperties"] = True
        errors = validate_inventory(REPO_ROOT, self.record, schema)
        self.assertIn("inventory schema must be a closed object", errors)

    def test_rejects_nested_schema_weakening_and_extra_completion_claim(self) -> None:
        schema = copy.deepcopy(self.schema)
        schema["properties"]["completion"]["additionalProperties"] = True
        record = self._record()
        record["completion"]["native_mcp"] = "complete"
        self._rehash(record)

        errors = validate_inventory(REPO_ROOT, record, schema)
        self.assertIn("inventory schema canonical digest is invalid", errors)

    def test_loader_rejects_duplicate_json_keys(self) -> None:
        record_text = (
            REPO_ROOT / "tooling/migration/ctr-201-inventory.json"
        ).read_text(encoding="utf-8")
        duplicate = record_text.replace(
            '  "status": "in-progress",',
            '  "status": "complete",\n  "status": "in-progress",',
            1,
        )

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "record.json").write_text(duplicate, encoding="utf-8")
            (root / "schema.json").write_text(
                json.dumps(self.schema),
                encoding="utf-8",
            )
            with self.assertRaises(InventoryConfigError):
                load_inventory_documents(
                    root,
                    record_path="record.json",
                    schema_path="schema.json",
                )

    def test_rejects_escaped_lone_surrogate_without_crashing(self) -> None:
        record = self._record()
        record["cli"]["captured_scope"][0] = "\ud800"
        self.assertEqual(
            validate_inventory(REPO_ROOT, record, self.schema),
            ["inventory contains invalid Unicode scalar data"],
        )

        record_text = (
            REPO_ROOT / "tooling/migration/ctr-201-inventory.json"
        ).read_text(encoding="utf-8")
        escaped = record_text.replace(
            '"align-success-outcome"',
            '"\\ud800"',
            1,
        )
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "record.json").write_text(escaped, encoding="utf-8")
            (root / "schema.json").write_text(
                json.dumps(self.schema),
                encoding="utf-8",
            )
            with self.assertRaises(InventoryConfigError):
                load_inventory_documents(
                    root,
                    record_path="record.json",
                    schema_path="schema.json",
                )

    def test_cli_has_stable_zero_one_and_two_exit_codes(self) -> None:
        stdout = io.StringIO()
        with redirect_stdout(stdout):
            self.assertEqual(main(["--root", str(REPO_ROOT), "--json"]), 0)
        self.assertEqual(json.loads(stdout.getvalue())["status"], "pass")

        invalid = self._record()
        invalid["completion"]["fnd_202"] = "implemented"
        self._rehash(invalid)
        stdout = io.StringIO()
        with patch(
            "tooling.scripts.validate_ctr_201_inventory.load_inventory_documents",
            return_value=(invalid, self.schema),
        ), redirect_stdout(stdout):
            self.assertEqual(main(["--root", str(REPO_ROOT), "--json"]), 1)
        self.assertEqual(json.loads(stdout.getvalue())["status"], "fail")

        stdout = io.StringIO()
        with redirect_stdout(stdout):
            self.assertEqual(
                main(
                    [
                        "--root",
                        str(REPO_ROOT),
                        "--record",
                        "/Users/private/QIONGLI_CANARY_DO_NOT_ECHO.json",
                        "--json",
                    ]
                ),
                2,
            )
        output = stdout.getvalue()
        self.assertEqual(json.loads(output)["status"], "error")
        self.assertNotIn("QIONGLI_CANARY_DO_NOT_ECHO", output)
        self.assertNotIn("/Users/", output)

    def test_repository_entrypoint_does_not_depend_on_pythonpath(self) -> None:
        environment = os.environ.copy()
        environment.pop("PYTHONPATH", None)
        completed = subprocess.run(
            [
                sys.executable,
                str(REPO_ROOT / "scripts/validate_ctr_201_inventory.py"),
                "--json",
            ],
            cwd=REPO_ROOT,
            env=environment,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )

        self.assertEqual(completed.returncode, 0, completed.stderr)
        self.assertEqual(completed.stderr, "")
        self.assertEqual(json.loads(completed.stdout)["status"], "pass")

    def test_validation_output_redacts_secret_shaped_record_values(self) -> None:
        record = self._record()
        canary = "QIONGLI_CANARY_DO_NOT_ECHO_semantic_inventory"
        record["cli"]["captured_scope"][0] = canary
        self._rehash(record)
        stdout = io.StringIO()
        with patch(
            "tooling.scripts.validate_ctr_201_inventory.load_inventory_documents",
            return_value=(record, self.schema),
        ), redirect_stdout(stdout):
            self.assertEqual(main(["--root", str(REPO_ROOT), "--json"]), 1)
        self.assertNotIn(canary, stdout.getvalue())

    def test_json_usage_error_is_redacted_and_json_only(self) -> None:
        canary = "QIONGLI_CANARY_DO_NOT_ECHO_argument"
        stdout = io.StringIO()
        stderr = io.StringIO()
        with redirect_stdout(stdout), redirect_stderr(stderr):
            self.assertEqual(main(["--json", f"--unknown={canary}"]), 2)

        payload = json.loads(stdout.getvalue())
        self.assertEqual(payload["status"], "error")
        self.assertEqual(payload["exit_code"], 2)
        self.assertEqual(stderr.getvalue(), "")
        self.assertNotIn(canary, stdout.getvalue())


if __name__ == "__main__":
    unittest.main()
