from __future__ import annotations

import copy
import hashlib
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
    DEFAULT_CLI_ARTIFACT,
    DEFAULT_CLI_SCHEMA,
    EXPECTED_CLI_SCHEMA_CANONICAL_SHA256,
    InventoryConfigError,
    _CLI_EXTRACTION_CACHE,
    _accepted_cli_extraction_bytes,
    _canonical_json_bytes,
    _load_json_file,
    _validate_cli_artifact_semantics,
    _validate_cli_static_semantics,
    _validate_recursively_closed_schema,
    canonical_payload_sha256,
    is_canonical_repository_path,
    load_inventory_documents,
    main,
    validate_inventory,
)


REPO_ROOT = Path(__file__).resolve().parents[1]
DIGEST_BOUND_JSON_PATHS = (
    "content/mcp-contracts/v2/registry.json",
    "tooling/migration/baselines/v1.19.0-beta.1/manifest.json",
    "tooling/migration/baselines/v1.19.0-beta.1/oracles/node-mcpb.json",
    "tooling/migration/baselines/v1.19.0-beta.1/oracles/python-full.json",
    "tooling/migration/baselines/v1.19.0-beta.1/oracles/rust-lite.json",
)


class Ctr201InventoryTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.record, cls.schema = load_inventory_documents(REPO_ROOT)
        cls.cli_artifact = _load_json_file(
            REPO_ROOT, DEFAULT_CLI_ARTIFACT, label="CLI child artifact"
        )
        cls.cli_schema = _load_json_file(
            REPO_ROOT, DEFAULT_CLI_SCHEMA, label="CLI child schema"
        )

    def _record(self) -> dict[str, object]:
        return copy.deepcopy(self.record)

    def _cli_record(self) -> dict[str, object]:
        return copy.deepcopy(self.cli_artifact)

    @staticmethod
    def _rehash(record: dict[str, object]) -> None:
        integrity = record["integrity"]
        assert isinstance(integrity, dict)
        integrity["payload_sha256"] = canonical_payload_sha256(record)

    def _validate_rehashed(self, record: dict[str, object]) -> list[str]:
        self._rehash(record)
        return validate_inventory(REPO_ROOT, record, self.schema)

    def _validate_cli_rehashed(self, artifact: dict[str, object]) -> list[str]:
        self._rehash(artifact)
        return _validate_cli_artifact_semantics(artifact)

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

    def test_cli_child_schema_artifact_and_master_binding_are_exact(self) -> None:
        binding = self.record["cli"]["static_semantics"]
        coverage = self.cli_artifact["coverage"]
        self.assertEqual(validate_instance(self.cli_artifact, self.cli_schema), [])
        self.assertEqual(_validate_recursively_closed_schema(self.cli_schema), [])
        self.assertEqual(_validate_cli_artifact_semantics(self.cli_artifact), [])
        self.assertEqual(_validate_cli_static_semantics(REPO_ROOT, self.record), [])
        self.assertEqual(
            hashlib.sha256(_canonical_json_bytes(self.cli_schema)).hexdigest(),
            EXPECTED_CLI_SCHEMA_CANONICAL_SHA256,
        )
        self.assertEqual(binding["artifact_path"], DEFAULT_CLI_ARTIFACT)
        self.assertEqual(binding["schema_path"], DEFAULT_CLI_SCHEMA)
        self.assertEqual(
            binding["payload_sha256"],
            self.cli_artifact["integrity"]["payload_sha256"],
        )
        self.assertEqual(
            self.cli_artifact["capture_contract"]["ambient_dependency_policy"],
            "disabled-with-deny-use-stubs",
        )
        self.assertEqual(
            [root["subcommand_metadata"] for root in self.cli_artifact["parser_roots"]],
            [
                {"destination": "cmd", "required": True},
                {"destination": "cmd", "required": True},
            ],
        )
        self.assertEqual(
            (
                binding["canonical_command_path_count"],
                binding["public_command_path_count"],
                binding["console_entrypoint_count"],
                binding["argument_action_count"],
                binding["cwd_default_count"],
            ),
            (
                coverage["canonical_command_count"],
                coverage["public_command_count"],
                coverage["console_entrypoint_count"],
                coverage["argument_action_count"],
                coverage["cwd_default_count"],
            ),
        )

    def test_cli_child_preserves_static_only_completion_boundary(self) -> None:
        coverage = self.cli_artifact["coverage"]
        self.assertEqual(coverage["static_semantics"], "captured")
        for field in (
            "formatted_help_output",
            "json_output",
            "runtime_behavior_matrix",
            "exit_code_matrix",
            "dry_run_semantics",
            "error_matrix",
            "side_effect_matrix",
            "legacy_npm_compatibility_surface",
        ):
            self.assertEqual(coverage[field], "incomplete")
        self.assertEqual(coverage["ctr_201"], "in-progress")
        self.assertEqual(coverage["fnd_202"], "not-implemented")
        self.assertFalse(coverage["completion_ready"])

    def test_cli_child_exact_aliases_zero_argument_commands_and_counts(self) -> None:
        commands = self.cli_artifact["commands"]
        aliases = {
            tuple(command["path"]): tuple(command["aliases"])
            for command in commands
            if command["aliases"]
        }
        empty = {
            tuple(command["path"])
            for command in commands
            if not command["arguments"]
        }
        self.assertEqual(
            aliases,
            {
                ("qiongli", "self-update"): ("update",),
                ("qiongli", "remove"): ("uninstall", "delete"),
            },
        )
        self.assertEqual(
            empty,
            {
                ("qiongli", "provider"),
                ("qiongli", "guidance"),
                ("qiongli", "subject"),
                ("qiongli", "project"),
                ("qiongli", "mcp", "config"),
            },
        )
        self.assertEqual(len(commands), 46)
        self.assertEqual(sum(len(command["arguments"]) for command in commands), 164)

    def test_cli_child_loader_rejects_duplicate_nonfinite_and_surrogate_json(self) -> None:
        cases = {
            "duplicate": '{"value":1,"value":2}',
            "nan": '{"value":NaN}',
            "infinity": '{"value":Infinity}',
            "surrogate": '{"value":"\\ud800"}',
        }
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for name, value in cases.items():
                with self.subTest(name=name):
                    path = root / f"{name}.json"
                    path.write_text(value, encoding="utf-8")
                    with self.assertRaises(InventoryConfigError):
                        _load_json_file(root, path.name, label="CLI child test")

    def test_cli_child_loader_rejects_symlinks_and_unsafe_repository_paths(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            target = root / "target.json"
            target.write_text("{}", encoding="utf-8")
            link = root / "link.json"
            try:
                link.symlink_to(target.name)
            except (OSError, NotImplementedError):
                pass
            else:
                with self.assertRaises(InventoryConfigError):
                    _load_json_file(root, link.name, label="CLI child test")

        for value in (
            "../escape.json",
            "nested//artifact.json",
            "C:/artifact.json",
            "CON/artifact.json",
            "nested./artifact.json.",
        ):
            with self.subTest(value=value):
                self.assertFalse(is_canonical_repository_path(value))

    def test_cli_child_rejects_recursive_schema_weakening(self) -> None:
        schema = copy.deepcopy(self.cli_schema)
        schema["$defs"]["argument"]["additionalProperties"] = True
        self.assertEqual(
            _validate_recursively_closed_schema(schema),
            ["CLI child schema must be recursively closed"],
        )

    def test_cli_child_rejects_integrity_and_master_binding_tampering(self) -> None:
        artifact = self._cli_record()
        artifact["commands"][0]["help"] = "changed without rehashing"
        self.assertIn(
            "CLI child canonical payload digest does not match",
            _validate_cli_artifact_semantics(artifact),
        )

        record = self._record()
        record["cli"]["static_semantics"]["schema_canonical_sha256"] = "0" * 64
        self.assertEqual(
            _validate_cli_static_semantics(REPO_ROOT, record),
            ["CLI static-semantics master binding is invalid"],
        )

    def test_cli_child_rejects_projected_path_alias_and_option_collisions(self) -> None:
        artifact = self._cli_record()
        artifact["commands"][1]["path"] = copy.deepcopy(
            artifact["commands"][0]["path"]
        )
        artifact["commands"][1]["segment"] = artifact["commands"][0]["segment"]
        errors = self._validate_cli_rehashed(artifact)
        self.assertIn("CLI child contains a duplicate canonical command path", errors)

        artifact = self._cli_record()
        artifact["commands"][0]["aliases"] = ["upgrade"]
        errors = self._validate_cli_rehashed(artifact)
        self.assertIn("CLI child contains a projected public command collision", errors)

        artifact = self._cli_record()
        command = next(
            item
            for item in artifact["commands"]
            if sum(bool(argument["option_strings"]) for argument in item["arguments"])
            >= 2
        )
        option_arguments = [
            argument for argument in command["arguments"] if argument["option_strings"]
        ]
        option_arguments[1]["option_strings"][0] = option_arguments[0]["option_strings"][0]
        errors = self._validate_cli_rehashed(artifact)
        self.assertIn("CLI child reuses an option string within a command", errors)

    def test_cli_child_rejects_root_delegate_action_and_ordinal_inconsistency(self) -> None:
        artifact = self._cli_record()
        mcp = next(
            command
            for command in artifact["commands"]
            if command["path"] == ["qiongli", "mcp"]
        )
        mcp["delegate"] = None
        errors = self._validate_cli_rehashed(artifact)
        self.assertIn("CLI child parser-root delegation is invalid", errors)

        artifact = self._cli_record()
        argument = next(
            argument
            for command in artifact["commands"]
            for argument in command["arguments"]
        )
        argument["action"] = "help"
        argument["destination"] = None
        errors = self._validate_cli_rehashed(artifact)
        self.assertIn("CLI child includes a non-callable argument action", errors)

        artifact = self._cli_record()
        artifact["commands"][0]["declaration_ordinal"] = 99
        errors = self._validate_cli_rehashed(artifact)
        self.assertIn(
            "CLI child command ordinals must be contiguous per parser root", errors
        )

    def test_cli_child_rejects_zero_argument_and_cwd_count_drift(self) -> None:
        artifact = self._cli_record()
        command = next(item for item in artifact["commands"] if item["arguments"])
        command["arguments"] = []
        errors = self._validate_cli_rehashed(artifact)
        self.assertIn("CLI child zero-argument command inventory is not exact", errors)
        self.assertIn("CLI child argument action count does not match", errors)

        artifact = self._cli_record()
        argument = next(
            argument
            for command in artifact["commands"]
            for argument in command["arguments"]
            if argument["default"] == {"kind": "context", "source": "cwd"}
        )
        argument["default"] = {"kind": "none"}
        errors = self._validate_cli_rehashed(artifact)
        self.assertIn("CLI child cwd-default count does not match", errors)

    def test_cli_child_redacts_machine_secret_and_callable_repr_canaries(self) -> None:
        canaries = (
            "/Users/private/hidden.json",
            "QIONGLI_CANARY_DO_NOT_ECHO_child",
            "<function hidden at 0xdeadbeef>",
        )
        expected = (
            "machine-local path",
            "secret-shaped data",
            "callable representation",
        )
        for canary, diagnostic in zip(canaries, expected, strict=True):
            with self.subTest(diagnostic=diagnostic):
                artifact = self._cli_record()
                artifact["commands"][0]["help"] = canary
                errors = self._validate_cli_rehashed(artifact)
                self.assertTrue(any(diagnostic in error for error in errors), errors)
                self.assertFalse(any(canary in error for error in errors))

    def test_cli_child_extraction_is_cached_and_compared_exactly(self) -> None:
        _CLI_EXTRACTION_CACHE.clear()
        try:
            with patch(
                "tooling.scripts.extract_ctr_201_cli_inventory.extract_cli_inventory",
                return_value=self.cli_artifact,
            ) as extractor:
                expected = _canonical_json_bytes(self.cli_artifact)
                self.assertEqual(_accepted_cli_extraction_bytes(REPO_ROOT), expected)
                self.assertEqual(_accepted_cli_extraction_bytes(REPO_ROOT), expected)
                extractor.assert_called_once_with(REPO_ROOT)
        finally:
            _CLI_EXTRACTION_CACHE.clear()

        with patch(
            "tooling.scripts.validate_ctr_201_inventory._accepted_cli_extraction_bytes",
            return_value=b"{}",
        ):
            self.assertEqual(
                _validate_cli_static_semantics(REPO_ROOT, self.record),
                ["CLI child artifact differs from accepted-source extraction"],
            )

    def test_cli_extraction_unavailable_is_redacted_exit_two(self) -> None:
        canary = "QIONGLI_CANARY_DO_NOT_ECHO_extractor"
        stdout = io.StringIO()
        with patch(
            "tooling.scripts.validate_ctr_201_inventory._accepted_cli_extraction_bytes",
            side_effect=InventoryConfigError(canary),
        ), redirect_stdout(stdout):
            self.assertEqual(main(["--root", str(REPO_ROOT), "--json"]), 2)
        output = stdout.getvalue()
        self.assertEqual(json.loads(output)["status"], "error")
        self.assertNotIn(canary, output)

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

    def test_digest_bound_json_files_are_forced_to_lf_checkout(self) -> None:
        completed = subprocess.run(
            ["git", "check-attr", "eol", "--", *DIGEST_BOUND_JSON_PATHS],
            cwd=REPO_ROOT,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )

        self.assertEqual(completed.returncode, 0, completed.stderr)
        self.assertEqual(
            completed.stdout.splitlines(),
            [f"{path}: eol: lf" for path in DIGEST_BOUND_JSON_PATHS],
        )

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
