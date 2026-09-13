from __future__ import annotations

import ast
import builtins
import copy
import hashlib
import importlib
import io
import json
import os
import runpy
import subprocess
import sys
import tempfile
import unittest
from contextlib import redirect_stderr, redirect_stdout
from pathlib import Path
from types import SimpleNamespace
from typing import Any, Mapping
from unittest.mock import patch

import yaml

from tooling.scripts import extract_ctr_201_orchestrator_inventory as extractor
from tooling.scripts.validate_capability_contract import validate_instance


REPO_ROOT = Path(__file__).resolve().parents[1]
ARTIFACT_PATH = REPO_ROOT / extractor.DEFAULT_OUTPUT_RELATIVE
SCHEMA_PATH = REPO_ROOT / extractor.DEFAULT_SCHEMA_RELATIVE
EXPECTED_PAYLOAD_SHA256 = (
    "508ed0f92a511a0a9a6daa33598ce891222540b15e5aa207984db97319fe2c5e"
)
EXPECTED_SCHEMA_SHA256 = (
    "0473158288cf35d4a10e39cfc741fd5b4cb38a49c68209aaea48337d52782510"
)


class Ctr201OrchestratorInventoryTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        if sys.version_info[:2] != (3, 12):
            raise unittest.SkipTest("CTR-201C extraction is pinned to Python 3.12")
        if yaml.__version__ != "6.0.3":
            raise unittest.SkipTest("CTR-201C extraction is pinned to PyYAML 6.0.3")
        cls.checked_artifact = json.loads(ARTIFACT_PATH.read_text(encoding="utf-8"))
        cls.checked_schema = json.loads(SCHEMA_PATH.read_text(encoding="utf-8"))
        cls.extracted_artifact = extractor.extract_orchestrator_inventory(REPO_ROOT)
        cls.extracted_schema = extractor.build_orchestrator_schema(
            cls.extracted_artifact
        )

    def _artifact(self) -> dict[str, Any]:
        return copy.deepcopy(self.extracted_artifact)

    def _run_main(self, arguments: list[str]) -> tuple[int, str, str]:
        stdout_bytes = io.BytesIO()
        stdout = io.TextIOWrapper(
            stdout_bytes,
            encoding="utf-8",
            newline="\n",
            write_through=True,
        )
        stderr = io.StringIO()
        with redirect_stdout(stdout), redirect_stderr(stderr):
            exit_code = extractor.main(arguments)
        stdout.flush()
        return exit_code, stdout_bytes.getvalue().decode("utf-8"), stderr.getvalue()

    def _assert_recursively_closed(
        self, schema: Mapping[str, Any], *, path: str = "$"
    ) -> None:
        schema_type = schema.get("type")
        if schema_type == "object":
            properties = schema.get("properties")
            self.assertIsInstance(properties, Mapping, path)
            self.assertIs(schema.get("additionalProperties"), False, path)
            self.assertEqual(
                set(schema.get("required", [])),
                set(properties),
                path,
            )
            for key, child in properties.items():
                self.assertIsInstance(child, Mapping, f"{path}.{key}")
                self._assert_recursively_closed(child, path=f"{path}.{key}")
        if schema_type == "array":
            if schema.get("maxItems") == 0:
                self.assertNotIn("items", schema, path)
            else:
                child = schema.get("items")
                self.assertIsInstance(child, Mapping, path)
                self._assert_recursively_closed(child, path=f"{path}[]")
        for index, child in enumerate(schema.get("anyOf", [])):
            self.assertIsInstance(child, Mapping, f"{path}.anyOf[{index}]")
            self._assert_recursively_closed(child, path=f"{path}.anyOf[{index}]")

    def test_artifact_schema_and_hashes_are_exactly_deterministic(self) -> None:
        self.assertEqual(self.extracted_artifact, self.checked_artifact)
        self.assertEqual(self.extracted_schema, self.checked_schema)
        self.assertEqual(
            extractor.canonical_payload_sha256(self.extracted_artifact),
            EXPECTED_PAYLOAD_SHA256,
        )
        self.assertEqual(
            self.extracted_artifact["integrity"]["payload_sha256"],
            EXPECTED_PAYLOAD_SHA256,
        )
        self.assertEqual(
            extractor.canonical_schema_sha256(self.extracted_schema),
            EXPECTED_SCHEMA_SHA256,
        )
        reordered = dict(reversed(list(self.extracted_artifact.items())))
        self.assertEqual(
            extractor.canonical_payload_sha256(reordered),
            EXPECTED_PAYLOAD_SHA256,
        )

    def test_artifact_matches_a_recursively_closed_valid_schema(self) -> None:
        self.assertEqual(
            validate_instance(self.extracted_artifact, self.extracted_schema), []
        )
        self._assert_recursively_closed(self.extracted_schema)
        unexpected = self._artifact()
        unexpected["unexpected"] = True
        self.assertNotEqual(validate_instance(unexpected, self.extracted_schema), [])
        self.assertEqual(self.extracted_artifact["task_id"], "CTR-201C")
        self.assertEqual(
            self.extracted_artifact["status"], "static-contract-captured"
        )

    def test_generation_and_check_use_canonical_artifact_and_schema(self) -> None:
        exit_code, stdout, stderr = self._run_main(
            ["--root", str(REPO_ROOT), "--check", "--json"]
        )
        self.assertEqual((exit_code, stderr), (0, ""))
        result = json.loads(stdout)
        self.assertEqual(result["code"], "accepted-orchestrator-inventory-matches")
        self.assertEqual(result["payload_sha256"], EXPECTED_PAYLOAD_SHA256)
        self.assertEqual(result["schema_canonical_sha256"], EXPECTED_SCHEMA_SHA256)

        with tempfile.TemporaryDirectory() as directory:
            output_path = Path(directory) / "inventory.json"
            schema_path = Path(directory) / "inventory.schema.json"
            with patch.object(
                extractor,
                "extract_orchestrator_inventory",
                return_value=self.extracted_artifact,
            ):
                exit_code, stdout, stderr = self._run_main(
                    [
                        "--output",
                        str(output_path),
                        "--schema-output",
                        str(schema_path),
                        "--json",
                    ]
                )
            self.assertEqual((exit_code, stderr), (0, ""))
            self.assertEqual(
                json.loads(stdout)["code"],
                "accepted-orchestrator-inventory-written",
            )
            self.assertEqual(output_path.read_bytes(), ARTIFACT_PATH.read_bytes())
            self.assertEqual(schema_path.read_bytes(), SCHEMA_PATH.read_bytes())

    def test_generation_rejects_same_and_case_alias_output_paths(self) -> None:
        with tempfile.TemporaryDirectory() as directory, patch.object(
            extractor,
            "extract_orchestrator_inventory",
            return_value=self.extracted_artifact,
        ):
            for artifact_name, schema_name in (
                ("same.json", "same.json"),
                ("inventory.json", "INVENTORY.JSON"),
            ):
                with self.subTest(
                    artifact_name=artifact_name,
                    schema_name=schema_name,
                ):
                    exit_code, stdout, stderr = self._run_main(
                        [
                            "--output",
                            str(Path(directory) / artifact_name),
                            "--schema-output",
                            str(Path(directory) / schema_name),
                            "--json",
                        ]
                    )
                    self.assertEqual((exit_code, stderr), (2, ""))
                    self.assertEqual(
                        json.loads(stdout)["code"],
                        "accepted-orchestrator-inventory-unavailable",
                    )
                    self.assertFalse((Path(directory) / artifact_name).exists())
                    self.assertFalse((Path(directory) / schema_name).exists())

    def test_check_fails_closed_for_artifact_and_schema_drift(self) -> None:
        for drift_target in ("artifact", "schema"):
            with (
                self.subTest(drift_target=drift_target),
                tempfile.TemporaryDirectory() as directory,
            ):
                output_path = Path(directory) / "ctr-201-orchestrator.json"
                schema_path = Path(directory) / Path(
                    extractor.DEFAULT_SCHEMA_RELATIVE
                ).name
                artifact = self._artifact()
                schema = copy.deepcopy(self.extracted_schema)
                if drift_target == "artifact":
                    artifact["status"] = "drifted"
                else:
                    schema["$id"] = "https://example.invalid/drifted.json"
                extractor._write_json(output_path, artifact)
                extractor._write_json(schema_path, schema)
                with patch.object(
                    extractor,
                    "extract_orchestrator_inventory",
                    return_value=self.extracted_artifact,
                ):
                    exit_code, stdout, stderr = self._run_main(
                        ["--check", "--output", str(output_path), "--json"]
                    )
                self.assertEqual((exit_code, stderr), (1, ""))
                self.assertEqual(
                    json.loads(stdout)["code"],
                    "accepted-orchestrator-inventory-mismatch",
                )

    def test_manifest_digest_and_accepted_source_identity_fail_closed(self) -> None:
        manifest_bytes = (REPO_ROOT / extractor.MANIFEST_RELATIVE).read_bytes()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / extractor.MANIFEST_RELATIVE
            path.parent.mkdir(parents=True)
            path.write_bytes(manifest_bytes + b"\n")
            with self.assertRaises(extractor.InventoryMismatch):
                extractor._read_manifest(root)

            document = json.loads(manifest_bytes)
            document["source"]["tag"] = "v0.0.0-invalid"
            modified = json.dumps(document, sort_keys=True).encode("utf-8")
            path.write_bytes(modified)
            with (
                patch.object(
                    extractor,
                    "MANIFEST_SHA256",
                    hashlib.sha256(modified).hexdigest(),
                ),
                self.assertRaises(extractor.InventoryMismatch),
            ):
                extractor._read_manifest(root)

    def test_tag_and_path_to_blob_bindings_fail_closed(self) -> None:
        good_tag = SimpleNamespace(
            returncode=0,
            stdout=(extractor.ACCEPTED_COMMIT + "\n").encode("ascii"),
            stderr=b"",
        )
        unused_tree = SimpleNamespace(returncode=0, stdout=b"", stderr=b"")
        bad_tag = SimpleNamespace(returncode=0, stdout=b"0" * 40 + b"\n", stderr=b"")
        with (
            patch.object(
                extractor.subprocess,
                "run",
                side_effect=[bad_tag, unused_tree],
            ),
            self.assertRaises(extractor.InventoryMismatch),
        ):
            extractor._verify_tag_and_tree(REPO_ROOT)

        missing_tag = SimpleNamespace(
            returncode=128,
            stdout=b"",
            stderr=b"fatal: accepted tag is unavailable\n",
        )
        with (
            patch.object(
                extractor.subprocess,
                "run",
                side_effect=[missing_tag, unused_tree],
            ),
            self.assertRaises(extractor.ExtractorError),
        ):
            extractor._verify_tag_and_tree(REPO_ROOT)

        expected_tree = b"".join(
            f"{item['mode']} blob {item['git_blob_oid']}\t{item['path']}\0".encode(
                "utf-8"
            )
            for item in sorted(
                extractor.SOURCE_BINDINGS,
                key=lambda value: str(value["path"]),
            )
        )
        traced_tag = SimpleNamespace(
            returncode=0,
            stdout=(extractor.ACCEPTED_COMMIT + "\n").encode("ascii"),
            stderr=b"trace: benign rev-parse diagnostics\n",
        )
        traced_tree = SimpleNamespace(
            returncode=0,
            stdout=expected_tree,
            stderr=b"trace: benign ls-tree diagnostics\n",
        )
        with patch.object(
            extractor.subprocess,
            "run",
            side_effect=[traced_tag, traced_tree],
        ):
            extractor._verify_tag_and_tree(REPO_ROOT)

        bad_tree = SimpleNamespace(returncode=0, stdout=b"", stderr=b"")
        with (
            patch.object(
                extractor.subprocess,
                "run",
                side_effect=[good_tag, bad_tree],
            ),
            self.assertRaises(extractor.InventoryMismatch),
        ):
            extractor._verify_tag_and_tree(REPO_ROOT)

    def test_git_reader_environment_disables_ambient_mutation_and_lazy_fetch(self) -> None:
        with patch.dict(
            os.environ,
            {
                "GIT_TRACE": "1",
                "GIT_OBJECT_DIRECTORY": "/private/tmp/untrusted-objects",
                "GIT_CONFIG_COUNT": "1",
            },
        ):
            environment = extractor._git_environment()
        self.assertNotIn("GIT_TRACE", environment)
        self.assertNotIn("GIT_OBJECT_DIRECTORY", environment)
        self.assertEqual(environment["GIT_CONFIG_COUNT"], "0")
        self.assertEqual(environment["GIT_CONFIG_NOSYSTEM"], "1")
        self.assertEqual(environment["GIT_CONFIG_GLOBAL"], os.devnull)
        self.assertEqual(environment["GIT_NO_LAZY_FETCH"], "1")
        self.assertEqual(environment["GIT_NO_REPLACE_OBJECTS"], "1")
        self.assertEqual(environment["GIT_OPTIONAL_LOCKS"], "0")
        self.assertEqual(environment["GIT_TERMINAL_PROMPT"], "0")

    def test_blob_reader_rejects_missing_truncated_and_digest_drift(self) -> None:
        binding = {
            "path": "accepted.py",
            "git_blob_oid": "1" * 40,
            "sha256": hashlib.sha256(b"abc").hexdigest(),
            "size_bytes": 3,
        }
        responses = (
            (
                SimpleNamespace(
                    returncode=0,
                    stdout=("1" * 40 + " missing\n").encode("ascii"),
                    stderr=b"",
                ),
                extractor.ExtractorError,
            ),
            (
                SimpleNamespace(
                    returncode=0,
                    stdout=("1" * 40 + " blob 3\n").encode("ascii") + b"ab\n",
                    stderr=b"",
                ),
                extractor.ExtractorError,
            ),
            (
                SimpleNamespace(
                    returncode=0,
                    stdout=("1" * 40 + " blob 3\n").encode("ascii") + b"abd\n",
                    stderr=b"",
                ),
                extractor.InventoryMismatch,
            ),
        )
        for completed, error_type in responses:
            with (
                self.subTest(error_type=error_type.__name__),
                patch.object(extractor, "SOURCE_BINDINGS", (binding,)),
                patch.object(extractor.subprocess, "run", return_value=completed),
                self.assertRaises(error_type),
            ):
                extractor._cat_file_blobs(REPO_ROOT)

    def test_oracle_digest_and_exact_outcome_fail_closed(self) -> None:
        oracle_bytes = (REPO_ROOT / extractor.PYTHON_ORACLE_RELATIVE).read_bytes()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / extractor.PYTHON_ORACLE_RELATIVE
            path.parent.mkdir(parents=True)
            path.write_bytes(oracle_bytes + b"\n")
            with self.assertRaises(extractor.InventoryMismatch):
                extractor._oracle(root)

            document = json.loads(oracle_bytes)
            case = next(
                item
                for item in document["cases"]
                if item["id"] == "python.orchestration-preview"
            )
            case["outcome"]["value"]["will_launch_agents"] = True
            modified = json.dumps(document, sort_keys=True).encode("utf-8")
            path.write_bytes(modified)
            with (
                patch.object(
                    extractor,
                    "PYTHON_ORACLE_SHA256",
                    hashlib.sha256(modified).hexdigest(),
                ),
                self.assertRaises(extractor.InventoryMismatch),
            ):
                extractor._oracle(root)

    def test_yaml_loader_accepts_only_the_authenticated_identical_duplicate(self) -> None:
        key = "academic-context-maintainer"
        payload = f"{key}:\n  file: skills/a.md\n{key}:\n  file: skills/a.md\n".encode()
        self.assertEqual(
            extractor._load_yaml_bytes(
                payload,
                allowed_identical_duplicates={key: 1},
            ),
            {key: {"file": "skills/a.md"}},
        )
        invalid_payloads = (
            payload,
            f"{key}: 1\n{key}: 2\n".encode(),
        )
        for invalid in invalid_payloads:
            with self.subTest(invalid=invalid), self.assertRaises(
                extractor.ExtractorError
            ):
                extractor._load_yaml_bytes(invalid)
        with self.assertRaises(extractor.ExtractorError):
            extractor._load_yaml_bytes(
                f"{key}: 1\n".encode(),
                allowed_identical_duplicates={key: 1},
            )

    def test_yaml_loader_rejects_alias_anchor_tags_keys_merge_and_nonfinite(self) -> None:
        payloads = {
            "alias": b"value: *undefined\n",
            "anchor": b"value: &anchor 1\n",
            "custom-tag": b"value: !unsafe 1\n",
            "timestamp": b"value: 2026-07-11\n",
            "binary": b"value: !!binary YQ==\n",
            "non-string-key": b"1: value\n",
            "merge": b"value:\n  <<: {nested: true}\n",
            "nonfinite": b"value: .nan\n",
        }
        for label, payload in payloads.items():
            with self.subTest(label=label), self.assertRaises(
                extractor.ExtractorError
            ):
                extractor._load_yaml_bytes(payload)

    def test_yaml_loader_enforces_nesting_and_node_limits(self) -> None:
        deeply_nested = "\n".join(
            ["  " * index + f"level_{index}:" for index in range(40)]
            + ["  " * 40 + "value"]
        ).encode("utf-8")
        with self.assertRaises(extractor.ExtractorError):
            extractor._load_yaml_bytes(deeply_nested)
        with (
            patch.object(extractor, "MAX_YAML_NODES", 2),
            self.assertRaises(extractor.ExtractorError),
        ):
            extractor._load_yaml_bytes(b"one: 1\ntwo: 2\n")

    def test_ast_literal_parser_never_evaluates_dynamic_python(self) -> None:
        safe = ast.parse(
            "{'text': 'value', 'items': [1, True, None], 'choice': KNOWN}",
            mode="eval",
        ).body
        self.assertEqual(
            extractor._literal(safe, {"KNOWN": "accepted"}),
            {
                "text": "value",
                "items": [1, True, None],
                "choice": "accepted",
            },
        )
        unsafe = (
            "dangerous()",
            "1 + 2",
            "UNKNOWN",
            "{'a': 1, **extra}",
            "{'a': 1, 'a': 2}",
            "{1: 'non-string-key'}",
        )
        for expression in unsafe:
            with self.subTest(expression=expression), self.assertRaises(
                extractor.ExtractorError
            ):
                extractor._literal(ast.parse(expression, mode="eval").body)
        with (
            patch.object(extractor, "MAX_AST_NODES", 1),
            self.assertRaises(extractor.ExtractorError),
        ):
            extractor._parse_python(b"value = 1\n", label="untrusted.py")

    def test_exact_counts_and_static_semantic_distinctions_are_preserved(self) -> None:
        artifact = self.extracted_artifact
        coverage = artifact["coverage"]
        for field, expected in extractor.EXPECTED_COUNTS.items():
            self.assertEqual(coverage[field], expected, field)
        self.assertEqual(
            artifact["compatibility"]["collaboration_modes"],
            ["parallel", "chain", "role", "single"],
        )
        self.assertEqual(
            artifact["compatibility"]["controller_modes"],
            ["solo", "duo", "triad"],
        )
        self.assertNotEqual(
            artifact["compatibility"]["collaboration_modes"],
            artifact["compatibility"]["controller_modes"],
        )
        self.assertEqual(
            [row["external"] for row in artifact["compatibility"]["worker_mode_pairs"]],
            ["none", "auto", "delegated-workers", "review-swarm"],
        )
        self.assertEqual(
            [row["normalized"] for row in artifact["compatibility"]["worker_mode_pairs"]],
            ["none", "auto", "delegated_workers", "review_swarm"],
        )
        self.assertEqual(
            [
                row["requires_runtime_resolution"]
                for row in artifact["compatibility"]["worker_mode_pairs"]
            ],
            [False, True, False, False],
        )
        adapter_status = {
            row["external"]: row["dispatch_status"]
            for row in artifact["compatibility"]["worker_adapter_pairs"]
        }
        self.assertEqual(
            adapter_status["codex-subagent"],
            "recognized-fallback-to-generic-prompt",
        )
        self.assertEqual(
            adapter_status["claude-cowork"],
            "recognized-fallback-to-generic-prompt",
        )
        self.assertEqual(
            artifact["compatibility"]["worker_default_status"],
            "disabled-unless-explicitly-requested",
        )
        self.assertEqual(
            artifact["compatibility"]["doctor_gate"],
            "route-sequence-advisory-not-enforced-by-task-run",
        )
        self.assertEqual(
            artifact["compatibility"]["quality_gate_runtime"],
            "task-declared-and-artifact-existence-only-not-semantic-policy-execution",
        )
        self.assertEqual(
            [row["task_id"] for row in artifact["routing"]["team_runs"]],
            ["B1", "H3"],
        )
        self.assertEqual(
            [row["task_id"] for row in artifact["routing"]["worker_configs"]],
            ["B1", "H3"],
        )
        self.assertEqual(len(artifact["routing"]["skills"]), 82)
        self.assertEqual(len(artifact["routing"]["logical_mcp_capabilities"]), 11)
        self.assertFalse(artifact["module_surface"]["mcp_escalation"]["run_agents_default"])
        self.assertEqual(artifact["oracle"]["outcome"]["execution_mode"], "duo")
        self.assertFalse(artifact["oracle"]["outcome"]["will_launch_agents"])
        self.assertFalse(artifact["coverage"]["completion_ready"])
        self.assertEqual(artifact["coverage"]["ctr_201"], "in-progress")
        self.assertEqual(artifact["coverage"]["fnd_202"], "not-implemented")

    def test_extraction_does_not_import_or_execute_accepted_source(self) -> None:
        blobs = extractor._cat_file_blobs(REPO_ROOT)
        git_commands: list[list[str]] = []
        original_import = builtins.__import__

        def reject_accepted_source_import(
            name: str, *arguments: Any, **keywords: Any
        ) -> Any:
            if name == "qiongli" or name.startswith("qiongli."):
                raise AssertionError("accepted source import")
            return original_import(name, *arguments, **keywords)

        def static_git_reader(
            command: list[str], **_arguments: Any
        ) -> SimpleNamespace:
            git_commands.append(command)
            operation = command[1]
            if operation == "rev-parse":
                return SimpleNamespace(
                    returncode=0,
                    stdout=(extractor.ACCEPTED_COMMIT + "\n").encode("ascii"),
                    stderr=b"",
                )
            if operation == "ls-tree":
                tree = b"".join(
                    f"{item['mode']} blob {item['git_blob_oid']}\t{item['path']}\0".encode(
                        "utf-8"
                    )
                    for item in sorted(
                        extractor.SOURCE_BINDINGS,
                        key=lambda value: str(value["path"]),
                    )
                )
                return SimpleNamespace(returncode=0, stdout=tree, stderr=b"")
            if operation == "cat-file":
                batch = b"".join(
                    f"{item['git_blob_oid']} blob {item['size_bytes']}\n".encode(
                        "ascii"
                    )
                    + blobs[str(item["path"])]
                    + b"\n"
                    for item in sorted(
                        extractor.SOURCE_BINDINGS,
                        key=lambda value: str(value["path"]),
                    )
                )
                return SimpleNamespace(returncode=0, stdout=batch, stderr=b"")
            raise AssertionError(f"unexpected Git operation: {operation}")

        with (
            patch.object(importlib, "import_module", side_effect=AssertionError("import")),
            patch.object(runpy, "run_path", side_effect=AssertionError("run-path")),
            patch.object(builtins, "eval", side_effect=AssertionError("eval")),
            patch.object(builtins, "exec", side_effect=AssertionError("exec")),
            patch.object(
                builtins,
                "__import__",
                side_effect=reject_accepted_source_import,
            ),
            patch.object(extractor.subprocess, "run", side_effect=static_git_reader),
        ):
            extracted = extractor.extract_orchestrator_inventory(REPO_ROOT)
        self.assertEqual(extracted, self.extracted_artifact)
        self.assertEqual(
            [command[:2] for command in git_commands],
            [["git", "rev-parse"], ["git", "ls-tree"], ["git", "cat-file"]],
        )

    def test_cli_results_are_stable_redacted_and_path_independent(self) -> None:
        secret = "/private/workspace/secret-token-value"
        with patch.object(
            extractor,
            "extract_orchestrator_inventory",
            side_effect=extractor.InventoryMismatch(secret),
        ):
            exit_code, stdout, stderr = self._run_main(["--check", "--json"])
        self.assertEqual((exit_code, stderr), (1, ""))
        self.assertNotIn(secret, stdout)
        self.assertEqual(
            json.loads(stdout),
            {
                "code": "accepted-orchestrator-inventory-mismatch",
                "ctr_201": "in-progress",
                "exit_code": 1,
                "fnd_202": "not-implemented",
                "status": "fail",
            },
        )

        with patch.object(
            extractor,
            "extract_orchestrator_inventory",
            side_effect=extractor.ExtractorError(secret),
        ):
            exit_code, stdout, stderr = self._run_main(["--check"])
        self.assertEqual((exit_code, stdout), (2, ""))
        self.assertEqual(stderr, "accepted-orchestrator-inventory-unavailable\n")
        self.assertNotIn(secret, stderr)

        exit_code, stdout, stderr = self._run_main(
            ["--json", "--unknown", secret]
        )
        self.assertEqual((exit_code, stderr), (2, ""))
        self.assertEqual(
            json.loads(stdout)["code"],
            "accepted-orchestrator-inventory-unavailable",
        )
        self.assertNotIn(secret, stdout)

        exit_code, stdout, stderr = self._run_main(
            ["--root", str(REPO_ROOT), "--check", "--json"]
        )
        self.assertEqual((exit_code, stderr), (0, ""))
        self.assertNotIn(str(REPO_ROOT), stdout)

    def test_missing_pyyaml_stays_inside_the_redacted_exit_contract(self) -> None:
        environment = os.environ.copy()
        environment.pop("PYTHONHOME", None)
        environment.pop("PYTHONPATH", None)
        completed = subprocess.run(
            [
                sys.executable,
                "-S",
                str(REPO_ROOT / "scripts" / "extract_ctr_201_orchestrator_inventory.py"),
                "--check",
                "--json",
            ],
            cwd=REPO_ROOT,
            env=environment,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )
        self.assertEqual(completed.returncode, 2)
        self.assertEqual(completed.stderr, "")
        self.assertEqual(
            json.loads(completed.stdout),
            {
                "code": "accepted-orchestrator-inventory-unavailable",
                "ctr_201": "in-progress",
                "exit_code": 2,
                "fnd_202": "not-implemented",
                "status": "error",
            },
        )
        self.assertNotIn(str(REPO_ROOT), completed.stdout + completed.stderr)


if __name__ == "__main__":
    unittest.main()
