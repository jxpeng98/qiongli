from __future__ import annotations

import copy
import hashlib
import io
import json
import os
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from contextlib import redirect_stderr, redirect_stdout
from typing import Any, Mapping
from unittest.mock import patch

import yaml

from tooling.scripts import extract_ctr_201_content_inventory as extractor
from tooling.scripts.validate_capability_contract import validate_instance


REPO_ROOT = Path(__file__).resolve().parents[1]
ARTIFACT_PATH = REPO_ROOT / extractor.DEFAULT_OUTPUT_RELATIVE
SCHEMA_PATH = REPO_ROOT / extractor.DEFAULT_SCHEMA_RELATIVE
EXPECTED_PAYLOAD_SHA256 = (
    "d17f37aa96d1896d047b27d197d63f773ae1d644a875722f5262be39593ff304"
)
EXPECTED_SCHEMA_SHA256 = (
    "6f88a56c2a88c51f68a6bb10bce05776d1e06f678ae916739a6e3de96d2b1704"
)


class Ctr201ContentInventoryTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        if os.sys.version_info < (3, 12):
            raise unittest.SkipTest("CTR-201D extraction requires Python 3.12+")
        if yaml.__version__ != "6.0.3":
            raise unittest.SkipTest("CTR-201D extraction requires PyYAML 6.0.3")
        cls.checked_artifact = json.loads(ARTIFACT_PATH.read_text(encoding="utf-8"))
        cls.checked_schema = json.loads(SCHEMA_PATH.read_text(encoding="utf-8"))
        cls.extracted_artifact = extractor.extract_content_inventory(REPO_ROOT)
        cls.extracted_schema = extractor.build_content_schema(cls.extracted_artifact)

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
        if schema.get("type") == "object":
            properties = schema.get("properties")
            self.assertIsInstance(properties, Mapping, path)
            self.assertIs(schema.get("additionalProperties"), False, path)
            self.assertEqual(set(schema.get("required", [])), set(properties), path)
            for key, child in properties.items():
                self.assertIsInstance(child, Mapping, f"{path}.{key}")
                self._assert_recursively_closed(child, path=f"{path}.{key}")
        if schema.get("type") == "array":
            child = schema.get("items")
            self.assertIsInstance(child, Mapping, path)
            self._assert_recursively_closed(child, path=f"{path}[]")
        for index, child in enumerate(schema.get("anyOf", [])):
            self.assertIsInstance(child, Mapping, f"{path}.anyOf[{index}]")
            self._assert_recursively_closed(child, path=f"{path}.anyOf[{index}]")

    def test_checked_artifact_schema_and_hashes_are_deterministic(self) -> None:
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

    def test_artifact_matches_a_recursively_closed_schema(self) -> None:
        self.assertEqual(validate_instance(self.extracted_artifact, self.extracted_schema), [])
        self._assert_recursively_closed(self.extracted_schema)
        unexpected = self._artifact()
        unexpected["unexpected"] = True
        self.assertNotEqual(validate_instance(unexpected, self.extracted_schema), [])
        nested = self._artifact()
        nested["profiles"][0]["materialized_tree"]["entries"][0]["unexpected"] = True
        self.assertNotEqual(validate_instance(nested, self.extracted_schema), [])

    def test_a8_content_tree_and_every_blob_are_frozen(self) -> None:
        source = self.extracted_artifact["source"]
        tree = source["content_tree"]
        self.assertEqual(source["accepted_tag"], extractor.ACCEPTED_TAG)
        self.assertEqual(source["accepted_commit"], extractor.ACCEPTED_COMMIT)
        self.assertEqual(tree["file_count"], 377)
        self.assertEqual(tree["total_bytes"], 1_761_400)
        self.assertEqual(tree["tree_sha256"], extractor.EXPECTED_CONTENT_TREE_SHA256)
        self.assertEqual(len(tree["files"]), 377)
        self.assertEqual(
            [row["path"] for row in tree["files"]],
            sorted((row["path"] for row in tree["files"]), key=lambda value: value.encode("utf-8")),
        )
        self.assertEqual(len({row["path"] for row in tree["files"]}), 377)
        self.assertEqual(sum(row["size_bytes"] for row in tree["files"]), 1_761_400)
        self.assertTrue(all(row["mode"] == "100644" for row in tree["files"]))
        self.assertEqual(
            hashlib.sha256(extractor._canonical_json_bytes(tree["files"])).hexdigest(),
            extractor.EXPECTED_CONTENT_TREE_SHA256,
        )
        roles = {anchor["role"]: anchor for anchor in source["materializer_anchors"]}
        self.assertEqual(roles["accepted-portable-payload-policy"]["mode"], "100755")
        self.assertEqual(len(roles), len(extractor.SOURCE_ANCHORS))

    def test_resource_roots_and_kinds_form_an_exact_partition(self) -> None:
        catalog = self.extracted_artifact["resource_catalog"]
        self.assertEqual(catalog["resource_root_count"], 12)
        self.assertEqual(catalog["resource_kind_count"], 11)
        roots = catalog["roots"]
        self.assertEqual(
            [(row["source"], row["file_count"], row["total_bytes"]) for row in roots],
            [
                ("content/distribution/", 3, 10_246),
                ("content/mcp-contracts/", 28, 74_579),
                ("content/roles/", 10, 13_570),
                ("content/schemas/", 5, 49_270),
                ("content/skills/", 97, 881_422),
                ("content/skills-core.md", 1, 25_853),
                ("content/skills-summary.md", 1, 6_539),
                ("content/standards/", 11, 139_835),
                ("content/subjects/", 77, 182_320),
                ("content/templates/", 92, 148_824),
                ("content/venue-profiles/", 6, 4_720),
                ("content/workflow/", 46, 224_222),
            ],
        )
        self.assertEqual(sum(row["file_count"] for row in roots), 377)
        self.assertEqual(sum(row["total_bytes"] for row in roots), 1_761_400)
        kind_by_id = {row["resource_kind"]: row for row in catalog["kinds"]}
        self.assertEqual(len(kind_by_id), 11)
        self.assertEqual(kind_by_id["skill-summary"]["file_count"], 2)
        self.assertEqual(kind_by_id["skill-summary"]["total_bytes"], 32_392)
        self.assertEqual(
            kind_by_id["skill-summary"]["entries_sha256"],
            "dab5cc7e0d352659234cccca49a2b0640da492f4ad537d9e70cf00cdf6d49889",
        )

    def test_profile_source_closures_and_real_trees_are_exact(self) -> None:
        profiles = self.extracted_artifact["profiles"]
        self.assertEqual([row["profile_id"] for row in profiles], ["skill-only", "marketplace-lite", "full"])
        expected = {
            "skill-only": (341, 1_627_305, "2283d6f5d284dde43225c5fb194e2e714b5e7b34e9c9bb97e753914d968acf26", 178, 708_608, "5b76bc0c02cda7fc18adf2b1afd492e763392ed5fc2a05dac360d1221045f280"),
            "marketplace-lite": (377, 1_761_400, extractor.EXPECTED_CONTENT_TREE_SHA256, 342, 1_600_064, "a854fc61203883132041a43077cc9ea26e62aa28e2c2eeb266777f582b029c6c"),
            "full": (377, 1_761_400, extractor.EXPECTED_CONTENT_TREE_SHA256, 343, 1_602_568, "b5612c713789bbd126829edc1e0646ec2c2387898aa2f5a4c812de0de5aad554"),
        }
        for profile in profiles:
            with self.subTest(profile=profile["profile_id"]):
                closure = profile["source_closure"]
                tree = profile["materialized_tree"]
                values = expected[profile["profile_id"]]
                self.assertEqual(
                    (closure["file_count"], closure["total_bytes"], closure["tree_sha256"]),
                    values[:3],
                )
                self.assertEqual(
                    (tree["file_count"], tree["total_bytes"], tree["tree_sha256"]),
                    values[3:],
                )
                rows = [
                    {key: entry[key] for key in ("path", "mode", "size_bytes", "sha256")}
                    for entry in tree["entries"]
                ]
                self.assertEqual(hashlib.sha256(extractor._canonical_json_bytes(rows)).hexdigest(), tree["tree_sha256"])
                self.assertEqual(sum(tree["origin_counts"].values()), tree["file_count"])
                self.assertEqual(len(tree["entries"]), tree["file_count"])
                self.assertFalse(profile["published_archive_member_parity"] != "not-captured")
        self.assertEqual(profiles[1]["aliases"], ["lite"])
        self.assertEqual(profiles[0]["aliases"], [])
        self.assertEqual(profiles[2]["aliases"], [])

    def test_portable_core_and_total_coverage_are_exact(self) -> None:
        portable = self.extracted_artifact["portable_core"]["materialized_tree"]
        self.assertEqual(
            (portable["file_count"], portable["total_bytes"], portable["tree_sha256"]),
            (263, 1_442_456, "21840d087bd18b1b9d37a03bddf6318a9023c69a0a320ff8bfcea843d4f5b48b"),
        )
        self.assertEqual(
            portable["origin_counts"],
            {"identity-copy": 263, "content-transform": 0, "generated-metadata": 0},
        )
        self.assertEqual(
            self.extracted_artifact["coverage"],
            {
                "accepted_content_file_count": 377,
                "accepted_content_total_bytes": 1_761_400,
                "resource_root_count": 12,
                "resource_kind_count": 11,
                "portable_core_file_count": 263,
                "materialized_profile_count": 3,
                "materialized_output_file_count": 863,
                "identity_output_count": 850,
                "transformed_output_count": 6,
                "generated_output_count": 7,
                "capture_ready": True,
            },
        )

    def test_compatibility_boundary_does_not_claim_archive_or_plugin_parity(self) -> None:
        boundary = self.extracted_artifact["compatibility_boundary"]
        self.assertIs(boundary["a8_generated_tree_evidence"], False)
        self.assertEqual(boundary["published_archive_member_parity"], "not-captured")
        self.assertEqual(boundary["complete_plugin_wrapper_parity"], "not-captured")
        self.assertEqual(boundary["complete_native_binary_parity"], "not-captured")
        self.assertEqual(boundary["extraction_network_sandbox"], "not-proven")
        self.assertEqual(
            boundary["extraction_filesystem_sandbox"],
            "python-audit-write-confined;host-read-isolation-not-proven;os-sandbox-not-proven",
        )
        self.assertIn("plugin-manifests-and-command-wrappers", boundary["excluded_outputs"])
        self.assertIn("published-archive-container-metadata", boundary["excluded_outputs"])
        self.assertIs(boundary["fnd_202_implemented"], False)

    def test_generation_and_check_follow_the_redacted_zero_one_two_contract(self) -> None:
        exit_code, stdout, stderr = self._run_main(["--root", str(REPO_ROOT), "--check", "--json"])
        self.assertEqual((exit_code, stderr), (0, ""))
        result = json.loads(stdout)
        self.assertEqual(result["code"], "accepted-content-inventory-matches")
        self.assertEqual(result["payload_sha256"], EXPECTED_PAYLOAD_SHA256)
        self.assertEqual(result["schema_canonical_sha256"], EXPECTED_SCHEMA_SHA256)

        with tempfile.TemporaryDirectory() as directory, patch.object(
            extractor,
            "extract_content_inventory",
            return_value=self.extracted_artifact,
        ):
            output = Path(directory) / "inventory.json"
            schema = Path(directory) / "inventory.schema.json"
            exit_code, stdout, stderr = self._run_main(
                ["--output", str(output), "--schema-output", str(schema), "--json"]
            )
            self.assertEqual((exit_code, stderr), (0, ""))
            self.assertEqual(output.read_bytes(), ARTIFACT_PATH.read_bytes())
            self.assertEqual(schema.read_bytes(), SCHEMA_PATH.read_bytes())

        exit_code, stdout, stderr = self._run_main(["--unknown-secret-argument", "sensitive-value", "--json"])
        self.assertEqual((exit_code, stderr), (2, ""))
        self.assertEqual(json.loads(stdout)["code"], "invalid-command-usage")
        self.assertNotIn("sensitive-value", stdout + stderr)

        with patch.object(extractor, "extract_content_inventory", side_effect=extractor.ExtractorError("private")):
            exit_code, stdout, stderr = self._run_main(["--output", "not-written.json", "--json"])
        self.assertEqual((exit_code, stderr), (2, ""))
        self.assertEqual(json.loads(stdout)["code"], "content-inventory-extraction-failed")
        self.assertNotIn("private", stdout + stderr)

    def test_check_drift_is_exit_one_and_redacted(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "ctr-201-content.json"
            schema = Path(directory) / Path(extractor.DEFAULT_SCHEMA_RELATIVE).name
            drifted = self._artifact()
            drifted["status"] = "drifted"
            extractor._write_json(output, drifted)
            extractor._write_json(schema, self.extracted_schema)
            with patch.object(
                extractor,
                "extract_content_inventory",
                return_value=self.extracted_artifact,
            ):
                exit_code, stdout, stderr = self._run_main(
                    ["--check", "--output", str(output), "--json"]
                )
            self.assertEqual((exit_code, stderr), (1, ""))
            self.assertEqual(json.loads(stdout)["code"], "accepted-content-inventory-mismatch")

    def test_source_and_materializer_tampering_fail_closed(self) -> None:
        _, rows = extractor._read_manifest(REPO_ROOT)
        bad_rows = copy.deepcopy(rows)
        bad_rows[0]["sha256"] = "0" * 64
        with self.assertRaises(extractor.InventoryMismatch):
            extractor._verify_tree_bindings(REPO_ROOT, bad_rows)

        bindings = {anchor["path"]: b"" for anchor in extractor.SOURCE_ANCHORS}
        bindings["packages/python-qiongli/src/qiongli/__init__.py"] = b"import socket\nsocket.socket()\n"
        bindings["packages/python-qiongli/src/qiongli/source_layout.py"] = b"from pathlib import Path\n"
        bindings["packages/python-qiongli/src/qiongli/subject_materializer.py"] = b"import yaml\n"
        with self.assertRaises(extractor.ExtractorError):
            extractor._verify_executed_reference_safety(bindings)

        dynamic = dict(bindings)
        dynamic["packages/python-qiongli/src/qiongli/__init__.py"] = b"exec('pass')\n"
        with self.assertRaises(extractor.ExtractorError):
            extractor._verify_executed_reference_safety(dynamic)

    def test_portable_paths_reject_traversal_devices_unicode_and_collisions(self) -> None:
        for value in (
            "../escape",
            "/absolute",
            "folder\\file",
            "folder/CON.txt",
            "folder/trailing. ",
            "folder/e\u0301.txt",
        ):
            with self.subTest(value=value), self.assertRaises(extractor.ExtractorError):
                extractor._portable_path(value)

        self.assertEqual(
            extractor._path_collision_key("VERSION"),
            extractor._path_collision_key("version"),
        )
        with self.assertRaises(extractor.ExtractorError):
            extractor._assert_collision_free_paths(["VERSION", "version"])

    def test_materialized_tree_rejects_links_secrets_and_machine_paths(self) -> None:
        with self.assertRaises(extractor.ExtractorError):
            extractor._scan_safe_text("SKILL.md", b"token=sk-abcdefghijklmnop", [])
        with self.assertRaises(extractor.ExtractorError):
            extractor._scan_safe_text("SKILL.md", b"/Users/alice/private/data.csv", [])
        extractor._scan_safe_text("SKILL.md", b"example /Users/name/data.csv", [])

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            target = root / "target"
            target.write_text("safe", encoding="utf-8")
            link = root / "link"
            try:
                link.symlink_to(target)
            except (OSError, NotImplementedError):
                self.skipTest("symlinks are unavailable")
            with self.assertRaises(extractor.ExtractorError):
                extractor._inventory_materialized_tree(root, {}, forbidden_roots=[])

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            target = root / "VERSION"
            target.write_text("safe", encoding="utf-8")
            alias = root / "VERSION.alias"
            try:
                os.link(target, alias)
            except (OSError, NotImplementedError):
                self.skipTest("hard links are unavailable")
            with self.assertRaises(extractor.ExtractorError):
                extractor._inventory_materialized_tree(root, {}, forbidden_roots=[])

    def test_output_paths_reject_aliases_and_symlinks(self) -> None:
        with tempfile.TemporaryDirectory() as directory, patch.object(
            extractor,
            "extract_content_inventory",
            return_value=self.extracted_artifact,
        ):
            same = Path(directory) / "same.json"
            exit_code, stdout, stderr = self._run_main(
                ["--output", str(same), "--schema-output", str(same), "--json"]
            )
            self.assertEqual((exit_code, stderr), (2, ""))
            self.assertEqual(json.loads(stdout)["code"], "invalid-command-usage")
            self.assertFalse(same.exists())

            target = Path(directory) / "target.json"
            target.write_text("{}", encoding="utf-8")
            link = Path(directory) / "linked.json"
            try:
                link.symlink_to(target)
            except (OSError, NotImplementedError):
                self.skipTest("symlinks are unavailable")
            with self.assertRaises(extractor.UsageError):
                extractor._write_json(link, self.extracted_artifact)

            loop = Path(directory) / "QIONGLI_CANARY_DO_NOT_ECHO_loop"
            try:
                loop.symlink_to(loop, target_is_directory=True)
            except (OSError, NotImplementedError):
                self.skipTest("symlinks are unavailable")
            exit_code, stdout, stderr = self._run_main(
                [
                    "--output",
                    str(loop / "artifact.json"),
                    "--schema-output",
                    str(Path(directory) / "schema.json"),
                    "--json",
                ]
            )
            self.assertEqual((exit_code, stderr), (2, ""))
            self.assertEqual(json.loads(stdout)["status"], "error")
            self.assertNotIn("QIONGLI_CANARY_DO_NOT_ECHO", stdout)

    def test_symlink_loop_root_is_redacted_exit_two(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            loop = Path(directory) / "QIONGLI_CANARY_DO_NOT_ECHO_root"
            try:
                loop.symlink_to(loop, target_is_directory=True)
            except (OSError, NotImplementedError):
                self.skipTest("symlinks are unavailable")
            exit_code, stdout, stderr = self._run_main(
                ["--root", str(loop), "--check", "--json"]
            )

        self.assertEqual((exit_code, stderr), (2, ""))
        self.assertEqual(json.loads(stdout)["status"], "error")
        self.assertNotIn("QIONGLI_CANARY_DO_NOT_ECHO", stdout)

    def test_worker_environment_redirects_host_state_and_disables_user_site(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            environment = extractor._worker_environment(root, "token")
            for key in (
                "HOME",
                "USERPROFILE",
                "XDG_CONFIG_HOME",
                "XDG_CACHE_HOME",
                "XDG_DATA_HOME",
                "CODEX_HOME",
                "CLAUDE_CODE_HOME",
                "TMPDIR",
            ):
                self.assertTrue(Path(environment[key]).is_relative_to(root), key)
            self.assertEqual(environment["PATH"], "")
            self.assertEqual(environment["PYTHONNOUSERSITE"], "1")
            self.assertEqual(environment["QIONGLI_CTR201D_WORKER_TOKEN"], "token")
            self.assertNotIn("PYTHONHASHSEED", environment)
            self.assertNotIn("PYTHONDONTWRITEBYTECODE", environment)
            if os.name == "nt" and os.environ.get("SystemRoot"):
                self.assertEqual(environment["SystemRoot"], os.environ["SystemRoot"])

    def test_worker_uses_isolated_no_bytecode_interpreter_flags(self) -> None:
        observed: dict[str, object] = {}
        with tempfile.TemporaryDirectory() as directory:
            worker_root = Path(directory)
            snapshot = worker_root / "accepted"
            snapshot.mkdir()

            def run_worker(command: list[str], **kwargs: object) -> SimpleNamespace:
                observed["command"] = command
                observed["environment"] = kwargs.get("env")
                (worker_root / "output-portable-core").mkdir()
                return SimpleNamespace(returncode=0, stdout=b"", stderr=b"")

            with patch.object(extractor.subprocess, "run", side_effect=run_worker):
                output = extractor._run_materializer_worker(
                    snapshot,
                    worker_root,
                    "portable-core",
                )

        command = observed["command"]
        self.assertIsInstance(command, list)
        self.assertEqual(command[1:3], ["-I", "-B"])
        self.assertEqual(output.name, "output-portable-core")

    def test_worker_generated_text_disables_platform_newline_translation(self) -> None:
        observed: dict[str, object] = {}

        def platform_writer(
            path: Path,
            data: str,
            encoding: str | None = None,
            errors: str | None = None,
            newline: str | None = None,
        ) -> int:
            observed.update(
                path=path,
                data=data,
                encoding=encoding,
                errors=errors,
                newline=newline,
            )
            return len(data)

        with patch.object(
            extractor,
            "_ORIGINAL_PATH_WRITE_TEXT",
            platform_writer,
        ):
            result = extractor._write_text_lf(
                Path("generated.txt"),
                "first\nsecond\n",
                encoding="utf-8",
                newline=None,
            )

        self.assertEqual(result, len("first\nsecond\n"))
        self.assertEqual(observed["data"], "first\nsecond\n")
        self.assertEqual(observed["newline"], "")

    def test_hidden_worker_rejects_broad_roots_and_unauthenticated_code(self) -> None:
        token = "test-token"
        with patch.dict(
            os.environ,
            {"QIONGLI_CTR201D_WORKER_TOKEN": token},
            clear=False,
        ):
            self.assertEqual(
                extractor._worker_main(
                    [
                        "--_worker",
                        token,
                        "full",
                        str(REPO_ROOT),
                        str(REPO_ROOT / "output-full"),
                        str(REPO_ROOT),
                    ]
                ),
                2,
            )
            with tempfile.TemporaryDirectory(prefix="qiongli-ctr201d-") as directory:
                worker_root = Path(directory).resolve()
                snapshot = worker_root / "accepted"
                for binding in extractor.SOURCE_ANCHORS:
                    if binding["role"] not in {
                        "accepted-python-package-init",
                        "accepted-source-layout",
                        "accepted-subject-materializer",
                    }:
                        continue
                    path = snapshot / binding["path"]
                    path.parent.mkdir(parents=True, exist_ok=True)
                    path.write_text("# unauthenticated\n", encoding="utf-8")
                self.assertEqual(
                    extractor._worker_main(
                        [
                            "--_worker",
                            token,
                            "full",
                            str(snapshot),
                            str(worker_root / "output-full"),
                            str(worker_root),
                        ]
                    ),
                    2,
                )


if __name__ == "__main__":
    unittest.main()
