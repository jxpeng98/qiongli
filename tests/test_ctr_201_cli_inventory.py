from __future__ import annotations

import argparse
import copy
import io
import json
import os
import sys
import tempfile
import unittest
from contextlib import redirect_stdout
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

from tooling.scripts import extract_ctr_201_cli_inventory as extractor
from tooling.scripts.validate_capability_contract import validate_instance


REPO_ROOT = Path(__file__).resolve().parents[1]
SCHEMA_PATH = REPO_ROOT / "tooling/migration/ctr-201-cli.schema.json"


class Ctr201CliInventoryTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        if sys.version_info[:2] != (3, 12):
            raise unittest.SkipTest("CTR-201B extraction is pinned to Python 3.12")
        cls.schema = json.loads(SCHEMA_PATH.read_text(encoding="utf-8"))
        cls.artifact = extractor.extract_cli_inventory(REPO_ROOT)

    def _artifact(self) -> dict[str, object]:
        return copy.deepcopy(self.artifact)

    def _run_main(self, arguments: list[str]) -> tuple[int, str]:
        stdout = io.StringIO()
        with redirect_stdout(stdout):
            exit_code = extractor.main(arguments)
        return exit_code, stdout.getvalue()

    def test_artifact_matches_closed_schema_and_static_only_boundary(self) -> None:
        self.assertEqual(validate_instance(self.artifact, self.schema), [])
        self.assertEqual(self.artifact["task_id"], "CTR-201B")
        self.assertEqual(self.artifact["status"], "static-semantics-captured")
        self.assertEqual(self.artifact["capture_contract"]["python_version"], "python3.12")
        self.assertEqual(
            self.artifact["capture_contract"]["ambient_dependency_policy"],
            "disabled-with-deny-use-stubs",
        )
        self.assertEqual(
            self.artifact["capture_contract"]["side_effect_policy"],
            "read-only-no-network-no-process",
        )
        coverage = self.artifact["coverage"]
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

    def test_source_is_bound_to_accepted_a8_blobs_and_pyproject_commit_path(self) -> None:
        source = self.artifact["source"]
        self.assertEqual(source["accepted_tag"], "v1.19.0-beta.1")
        self.assertEqual(
            source["accepted_commit"],
            "8d2e99866ce4c4efb8b3b5e0265c0c1f89a36b0f",
        )
        self.assertEqual(source["package_tree"]["file_count"], 76)
        self.assertEqual(
            [anchor["role"] for anchor in source["blob_anchors"]],
            ["cli-parser", "mcp-cli-parser", "console-entrypoints"],
        )
        self.assertEqual(
            source["blob_anchors"][2]["git_blob_oid"],
            "4fc00c6a21b5c7e8a9ffc1ac58698b9d2bd087a5",
        )
        extractor._verify_pyproject_commit_binding(REPO_ROOT)

    def test_pyproject_commit_path_binding_fails_closed(self) -> None:
        completed = SimpleNamespace(returncode=0, stdout=b"", stderr=b"")
        with patch.object(extractor.subprocess, "run", return_value=completed):
            with self.assertRaises(extractor.InventoryMismatch):
                extractor._verify_pyproject_commit_binding(REPO_ROOT)

        unavailable = SimpleNamespace(returncode=128, stdout=b"", stderr=b"missing")
        with patch.object(extractor.subprocess, "run", return_value=unavailable):
            with self.assertRaises(extractor.ExtractorError):
                extractor._verify_pyproject_commit_binding(REPO_ROOT)

    def test_argparse_allowlist_uses_exact_type_identity(self) -> None:
        fake_store_action = object.__new__(type("_StoreAction", (), {}))
        with self.assertRaises(extractor.ExtractorError):
            extractor._action_kind(fake_store_action)

        custom_formatter = argparse.ArgumentParser(
            formatter_class=argparse.RawTextHelpFormatter
        )
        with self.assertRaises(extractor.ExtractorError):
            extractor._require_allowlisted_parser(custom_formatter)

    def test_yaml_stub_allows_import_but_rejects_ambient_yaml_use(self) -> None:
        stub = extractor._make_yaml_deny_use_stub()
        self.assertIs(stub.YAMLError, extractor._DeniedYamlUseError)
        with self.assertRaises(extractor.ExtractorError):
            getattr(stub, "safe_load")

    def test_worker_audit_guard_rejects_all_process_launch_families(self) -> None:
        for event in (
            "os.exec",
            "os.execve",
            "os.chdir",
            "os.fchdir",
            "os.fork",
            "os.forkpty",
            "os.posix_spawn",
            "os.spawn",
            "os.spawnve",
            "os.startfile",
            "os.system",
            "pty.spawn",
            "subprocess.Popen",
        ):
            with self.subTest(event=event):
                with self.assertRaises(PermissionError):
                    extractor._guard_worker_audit_event(event, (), REPO_ROOT)

    def test_worker_audit_guard_rejects_outside_temp_writes_and_mutations(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            write_root = Path(directory)
            inside = write_root / "inside"
            outside = write_root.parent / "outside-ctr-201b"
            extractor._guard_worker_audit_event("open", (str(outside), "r", 0), write_root)
            for path in (inside, outside):
                with self.subTest(path=path), self.assertRaises(PermissionError):
                    extractor._guard_worker_audit_event(
                        "open",
                        (str(path), "w", os.O_WRONLY | os.O_CREAT),
                        write_root,
                    )
            with self.assertRaises(PermissionError):
                extractor._guard_worker_audit_event(
                    "open",
                    ("relative.json", None, os.O_WRONLY | os.O_CREAT),
                    write_root,
                )

            for event in extractor._MUTATION_EVENTS:
                with self.subTest(event=event), self.assertRaises(PermissionError):
                    extractor._guard_worker_audit_event(event, (), write_root)

            with self.assertRaises(PermissionError):
                extractor._guard_worker_audit_event(
                    "os.chmod",
                    (str(inside), 0o600, 3),
                    write_root,
                )

    def test_console_entrypoints_preserve_toml_declaration_order(self) -> None:
        self.assertEqual(
            self.artifact["console_entrypoints"],
            [
                {
                    "name": name,
                    "target": "qiongli.cli:main",
                    "declaration_ordinal": ordinal,
                }
                for ordinal, name in enumerate(
                    ("qiongli", "ql", "research-skills", "rsk", "rsw")
                )
            ],
        )

    def test_parser_roots_commands_aliases_arguments_and_delegate_are_exact(self) -> None:
        self.assertEqual(
            [root["path"] for root in self.artifact["parser_roots"]],
            [["qiongli"], ["qiongli", "mcp"]],
        )
        self.assertEqual(
            [root["description"] for root in self.artifact["parser_roots"]],
            [
                "Install/upgrade qiongli client skills without requiring a git fork.",
                "Run and configure the Qiongli cross-platform MCP server.",
            ],
        )
        self.assertTrue(
            all(
                root["subcommand_metadata"]
                == {"destination": "cmd", "required": True}
                for root in self.artifact["parser_roots"]
            )
        )
        commands = self.artifact["commands"]
        self.assertEqual(len(commands), 46)
        self.assertEqual(sum(1 + len(command["aliases"]) for command in commands), 49)
        self.assertEqual(sum(len(command["arguments"]) for command in commands), 164)
        self.assertTrue(
            all(
                argument["action"] != "help"
                for command in commands
                for argument in command["arguments"]
            )
        )
        aliases = {tuple(command["path"]): command["aliases"] for command in commands}
        self.assertEqual(aliases[("qiongli", "self-update")], ["update"])
        self.assertEqual(aliases[("qiongli", "remove")], ["uninstall", "delete"])
        zero_arguments = {
            tuple(command["path"])
            for command in commands
            if command["arguments"] == []
        }
        self.assertEqual(zero_arguments, extractor.EXPECTED_ZERO_ARGUMENT_COMMANDS)
        delegates = {
            tuple(command["path"]): command["delegate"]
            for command in commands
            if command["delegate"] is not None
        }
        self.assertEqual(
            delegates,
            {
                ("qiongli", "mcp"): {
                    "kind": "parser-root",
                    "parser_root_id": "qiongli-mcp-cli",
                    "argument_destination": "mcp_args",
                }
            },
        )

    def test_dynamic_cwd_defaults_are_symbolic_and_environment_independent(self) -> None:
        cwd_defaults = [
            argument["default"]
            for command in self.artifact["commands"]
            for argument in command["arguments"]
            if argument["default"] == {"kind": "context", "source": "cwd"}
        ]
        self.assertEqual(len(cwd_defaults), 27)
        rendered = json.dumps(self.artifact, sort_keys=True)
        self.assertNotIn(str(REPO_ROOT), rendered)
        self.assertNotIn("object at 0x", rendered)

    def test_payload_hash_excludes_integrity_and_is_key_order_stable(self) -> None:
        expected = self.artifact["integrity"]["payload_sha256"]
        self.assertEqual(extractor.canonical_payload_sha256(self.artifact), expected)
        reordered = dict(reversed(list(self.artifact.items())))
        reordered["integrity"] = {
            **reordered["integrity"],
            "payload_sha256": "0" * 64,
        }
        self.assertEqual(extractor.canonical_payload_sha256(reordered), expected)

    def test_json_loader_rejects_duplicate_keys_and_nonfinite_numbers(self) -> None:
        for payload in (b'{"a":1,"a":2}', b'{"a":NaN}', b'{"a":Infinity}'):
            with self.subTest(payload=payload):
                with self.assertRaises(extractor.ExtractorError):
                    extractor._load_json_bytes(payload)

    def test_python_version_claim_is_enforced(self) -> None:
        with patch.object(extractor.sys, "version_info", (3, 13, 0)):
            with self.assertRaises(extractor.ExtractorError):
                extractor._require_python_312()

    def test_check_without_output_uses_canonical_artifact_path(self) -> None:
        with (
            patch.object(extractor, "extract_cli_inventory", return_value=self.artifact),
            patch.object(extractor, "_load_checked_output", return_value=self.artifact) as loader,
        ):
            exit_code, output = self._run_main(
                ["--root", str(REPO_ROOT), "--check", "--json"]
            )
        self.assertEqual(exit_code, 0)
        loader.assert_called_once_with(REPO_ROOT / extractor.DEFAULT_OUTPUT_RELATIVE)
        payload = json.loads(output)
        self.assertEqual(payload["status"], "pass")
        self.assertEqual(payload["code"], "accepted-cli-inventory-matches")

    def test_generation_requires_explicit_output_and_writes_canonical_record(self) -> None:
        with patch.object(extractor, "extract_cli_inventory") as capture:
            exit_code, output = self._run_main(["--root", str(REPO_ROOT), "--json"])
        self.assertEqual(exit_code, 2)
        capture.assert_not_called()
        self.assertEqual(
            json.loads(output),
            {
                "code": "accepted-cli-inventory-unavailable",
                "ctr_201": "in-progress",
                "exit_code": 2,
                "fnd_202": "not-implemented",
                "status": "error",
            },
        )
        with tempfile.TemporaryDirectory() as directory:
            output_path = Path(directory) / "inventory.json"
            with patch.object(
                extractor,
                "extract_cli_inventory",
                return_value=self.artifact,
            ):
                exit_code, output = self._run_main(
                    ["--output", str(output_path), "--json"]
                )
            self.assertEqual(exit_code, 0)
            self.assertEqual(
                json.loads(output)["code"],
                "accepted-cli-inventory-written",
            )
            self.assertEqual(json.loads(output_path.read_text(encoding="utf-8")), self.artifact)

    def test_cli_json_failures_are_stable_and_redacted(self) -> None:
        secret = "private-token-value"
        with patch.object(
            extractor,
            "extract_cli_inventory",
            side_effect=extractor.InventoryMismatch(secret),
        ):
            exit_code, output = self._run_main(["--check", "--json"])
        self.assertEqual(exit_code, 1)
        self.assertNotIn(secret, output)
        self.assertEqual(json.loads(output)["code"], "accepted-cli-inventory-mismatch")

        exit_code, output = self._run_main(["--json", "--unknown", secret])
        self.assertEqual(exit_code, 2)
        self.assertNotIn(secret, output)
        self.assertEqual(json.loads(output)["code"], "accepted-cli-inventory-unavailable")

    def test_cli_text_results_are_stable_and_do_not_echo_internal_errors(self) -> None:
        secret = "private-internal-path"
        with patch.object(
            extractor,
            "extract_cli_inventory",
            side_effect=extractor.ExtractorError(secret),
        ):
            exit_code, output = self._run_main(["--check"])
        self.assertEqual(exit_code, 2)
        self.assertEqual(
            output,
            "[ctr-201b-cli] ERROR: accepted-cli-inventory-unavailable\n",
        )
        self.assertNotIn(secret, output)


if __name__ == "__main__":
    unittest.main()
