from __future__ import annotations

import copy
import io
import json
import os
import sys
import tempfile
import unittest
from contextlib import redirect_stdout
from pathlib import Path
from unittest.mock import patch

from tooling.scripts import extract_ctr_201_cli_runtime_inventory as extractor
from tooling.scripts.validate_capability_contract import validate_instance


REPO_ROOT = Path(__file__).resolve().parents[1]
ARTIFACT_PATH = REPO_ROOT / "tooling/migration/ctr-201-cli-runtime.json"
SCHEMA_PATH = REPO_ROOT / "tooling/migration/ctr-201-cli-runtime.schema.json"


class Ctr201CliRuntimeCheckedArtifactTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.schema = json.loads(SCHEMA_PATH.read_text(encoding="utf-8"))
        cls.artifact = json.loads(ARTIFACT_PATH.read_text(encoding="utf-8"))
        cls.case_by_id = {case["id"]: case for case in cls.artifact["cases"]}

    def _artifact(self) -> dict[str, object]:
        return copy.deepcopy(self.artifact)

    def _rehash(self, artifact: dict[str, object]) -> None:
        artifact["integrity"]["payload_sha256"] = extractor.canonical_payload_sha256(
            artifact
        )

    def assertSchemaRejects(self, artifact: dict[str, object]) -> None:  # noqa: N802
        self.assertTrue(validate_instance(artifact, self.schema))

    def assertSemanticRejects(self, artifact: dict[str, object]) -> None:  # noqa: N802
        self._rehash(artifact)
        with self.assertRaises(extractor.RuntimeInventoryMismatch):
            extractor._validate_expected_artifact(artifact, REPO_ROOT)

    def test_artifact_matches_closed_schema_and_semantic_validator(self) -> None:
        self.assertEqual(validate_instance(self.artifact, self.schema), [])
        extractor._validate_expected_artifact(self.artifact, REPO_ROOT)
        self.assertEqual(self.artifact["task_id"], "CTR-201E")
        self.assertEqual(
            self.artifact["status"], "runtime-inventory-freeze-captured"
        )

    def test_source_is_exact_tag_receipt_digest_bound_but_not_signed(self) -> None:
        source = self.artifact["source"]
        self.assertEqual(source["accepted_tag"], "v1.19.0-beta.1")
        self.assertEqual(
            source["tag_object_oid"],
            "e68e3af4c879d8e9053124d1aed625bfcddfdbb4",
        )
        self.assertEqual(
            source["accepted_commit"],
            "8d2e99866ce4c4efb8b3b5e0265c0c1f89a36b0f",
        )
        self.assertEqual(
            source["trust"],
            "exact-tag-object-receipt-and-digest-bound-not-signature-verified",
        )
        self.assertNotIn("signed", json.dumps(source, sort_keys=True).lower())

    def test_public_command_and_entrypoint_matrices_are_complete(self) -> None:
        coverage = self.artifact["coverage"]
        self.assertEqual(coverage["canonical_commands"], 46)
        self.assertEqual(coverage["public_commands"], 49)
        self.assertEqual(coverage["console_entrypoints"], 5)
        self.assertEqual(coverage["help_observations"], 245)
        self.assertEqual(coverage["invalid_usage_observations"], 49)
        self.assertEqual(coverage["zero_argument_observations"], 5)
        self.assertEqual(coverage["json_canonical_commands"], 13)
        self.assertEqual(coverage["dry_run_canonical_commands"], 8)
        self.assertEqual(coverage["dry_run_public_commands"], 11)
        paths = [tuple(row["public_path"]) for row in self.artifact["public_commands"]]
        self.assertEqual(len(paths), len(set(paths)))
        self.assertEqual(
            [row["name"] for row in self.artifact["console_entrypoints"]],
            ["qiongli", "ql", "research-skills", "rsk", "rsw"],
        )

    def test_every_public_path_has_closed_behavior_dispositions(self) -> None:
        case_ids = set(self.case_by_id)
        for row in self.artifact["public_commands"]:
            with self.subTest(path=row["public_path"]):
                self.assertEqual(
                    set(row),
                    {
                        "public_path",
                        "canonical_path",
                        "path_kind",
                        "json_capable",
                        "dry_run_capable",
                        "executable",
                        "help",
                        "behavior",
                        "stdout_stderr",
                        "exit_code",
                        "json",
                        "dry_run",
                        "zero_argument",
                        "error",
                        "side_effects",
                    },
                )
                for field in (
                    "help",
                    "behavior",
                    "stdout_stderr",
                    "exit_code",
                    "json",
                    "dry_run",
                    "zero_argument",
                    "error",
                    "side_effects",
                ):
                    dimension = row[field]
                    self.assertEqual(
                        set(dimension),
                        {"disposition", "case_ids", "reason_code", "decision_id"},
                    )
                    self.assertIn(
                        dimension["disposition"],
                        {
                            "captured",
                            "accepted-oracle",
                            "not-applicable",
                            "explicit-disposition",
                        },
                    )
                    self.assertTrue(dimension["reason_code"])
                    self.assertTrue(set(dimension["case_ids"]).issubset(case_ids))
                    if dimension["disposition"] == "explicit-disposition":
                        self.assertIn(
                            dimension["decision_id"],
                            {"CTR-201E-D001", "CTR-201E-D002"},
                        )
                    else:
                        self.assertIsNone(dimension["decision_id"])

    def test_inventory_only_dispositions_do_not_claim_handler_parity(self) -> None:
        decisions = self.artifact["disposition_decisions"]
        self.assertEqual(
            [row["id"] for row in decisions],
            ["CTR-201E-D001", "CTR-201E-D002", "CTR-201E-D003"],
        )
        self.assertTrue(
            all(
                row["status"] == "approved-for-ctr-201-inventory-only"
                and row["owner_task"] == "LEG-201"
                for row in decisions
            )
        )
        explicit = [
            dimension
            for row in self.artifact["public_commands"]
            for dimension in (
                row["behavior"],
                row["stdout_stderr"],
                row["exit_code"],
                row["json"],
                row["dry_run"],
                row["error"],
                row["side_effects"],
            )
            if dimension["disposition"] == "explicit-disposition"
        ]
        self.assertTrue(explicit)
        self.assertEqual(
            self.artifact["coverage"]["full_handler_runtime_parity"],
            "not-claimed",
        )
        self.assertEqual(
            self.artifact["npm_compatibility"]["disposition_decision_id"],
            "CTR-201E-D003",
        )

    def test_help_and_usage_streams_preserve_accepted_argparse_behavior(self) -> None:
        for row in self.artifact["public_commands"]:
            path = ".".join(row["public_path"])
            help_case = self.case_by_id[f"python.help.{path}"]
            invalid_case = self.case_by_id[f"python.invalid-usage.{path}"]
            with self.subTest(path=path):
                self.assertEqual(help_case["outcome"]["exit_code"], 0)
                self.assertEqual(help_case["outcome"]["stderr_lines"], [])
                self.assertEqual(
                    set(help_case["entrypoint_observations"]),
                    {"qiongli", "ql", "research-skills", "rsk", "rsw"},
                )
                self.assertEqual(invalid_case["outcome"]["exit_code"], 2)
                self.assertEqual(invalid_case["outcome"]["error_class"], "usage-error")
                self.assertEqual(invalid_case["outcome"]["stdout_lines"], [])
                self.assertTrue(invalid_case["outcome"]["stderr_lines"])
        self.assertTrue(
            self.case_by_id["python.help.qiongli.mcp.serve"]["outcome"][
                "stdout_lines"
            ][0].startswith("usage: qiongli serve")
        )
        self.assertIn(
            "self-update",
            self.case_by_id["python.help.qiongli.update"]["outcome"]["stdout_lines"][0],
        )
        for alias in ("uninstall", "delete"):
            self.assertIn(
                "remove",
                self.case_by_id[f"python.help.qiongli.{alias}"]["outcome"][
                    "stdout_lines"
                ][0],
            )

    def test_console_entrypoint_argv0_behavior_is_frozen_separately_from_a8(self) -> None:
        for name in ("qiongli", "ql", "research-skills", "rsk", "rsw"):
            case = self.case_by_id[f"python.entrypoint.{name}.align"]
            self.assertEqual(case["outcome"]["stdout_lines"][0], f"{name} — Quick Reference")
        a8 = self.case_by_id["a8.python.cli-align"]
        self.assertEqual(a8["outcome"]["stdout_lines"][0], "cli.py — Quick Reference")
        self.assertEqual(a8["invocation"]["entrypoint"], "python-module")

    def test_json_error_and_dry_run_boundaries_are_truthful(self) -> None:
        for case_id in (
            "python.handler.provider-list-json",
            "python.handler.mcp-config-example-json",
        ):
            json_outcome = self.case_by_id[case_id]["outcome"]["json"]
            parsed = json.loads(json_outcome["canonical_json"])
            self.assertEqual(
                json.dumps(parsed, ensure_ascii=False, sort_keys=True, separators=(",", ":")),
                json_outcome["canonical_json"],
            )
        domain_error = self.case_by_id["python.handler.provider-invalid-field"]
        self.assertEqual(domain_error["outcome"]["exit_code"], 1)
        self.assertEqual(domain_error["outcome"]["error_class"], "input-error")
        install = next(
            row
            for row in self.artifact["public_commands"]
            if row["public_path"] == ["qiongli", "install"]
        )
        self.assertEqual(install["dry_run"]["disposition"], "accepted-oracle")
        upgrade = next(
            row
            for row in self.artifact["public_commands"]
            if row["public_path"] == ["qiongli", "upgrade"]
        )
        self.assertEqual(upgrade["dry_run"]["disposition"], "explicit-disposition")
        self.assertEqual(
            upgrade["dry_run"]["reason_code"],
            "downloads-before-install-even-in-dry-run",
        )
        zero_argument_paths = {
            tuple(row["public_path"])
            for row in self.artifact["public_commands"]
            if row["zero_argument"]["disposition"] == "captured"
        }
        self.assertEqual(zero_argument_paths, extractor.static_cli.EXPECTED_ZERO_ARGUMENT_COMMANDS)
        for path in zero_argument_paths:
            case = self.case_by_id[f"python.zero-argument.{'.'.join(path)}"]
            self.assertEqual(case["outcome"]["exit_code"], 2)
            self.assertEqual(case["outcome"]["error_class"], "usage-error")

    def test_npm_dispatch_freezes_real_python_divergence_without_claiming_parity(self) -> None:
        npm = self.artifact["npm_compatibility"]
        dispatch = {row["raw_command"]: row for row in npm["dispatch"]}
        for alias in ("update", "refresh", "upgrade"):
            self.assertEqual(dispatch[alias]["normalized_command"], "install")
            self.assertTrue(dispatch[alias]["overwrite"])
        for alias in ("uninstall", "delete"):
            self.assertEqual(dispatch[alias]["normalized_command"], "remove")
        self.assertEqual(npm["handler_runtime_parity"], "pending-LEG-201")
        self.assertEqual(
            npm["python_npm_divergences"][0]["disposition"],
            "frozen-divergence-pending-LEG-201",
        )

    def test_case_effects_have_no_unclassified_write_or_secret(self) -> None:
        rendered = json.dumps(self.artifact, ensure_ascii=False, sort_keys=True)
        self.assertNotIn(str(REPO_ROOT), rendered)
        self.assertNotIn(extractor.CANARY_SECRET, rendered)
        self.assertNotIn("Traceback (most recent call last)", rendered)
        for case in self.artifact["cases"]:
            filesystem = case["effects"]["filesystem"]
            self.assertEqual(
                filesystem["before_tree_sha256"], filesystem["after_tree_sha256"]
            )
            self.assertEqual(filesystem["created"], [])
            self.assertEqual(filesystem["modified"], [])
            self.assertEqual(filesystem["deleted"], [])
            self.assertFalse(filesystem["writes_outside_sandbox"])

    def test_effect_and_stream_provenance_does_not_overstate_a8(self) -> None:
        python_cases = [
            case for case in self.artifact["cases"] if case["id"].startswith("python.")
        ]
        self.assertTrue(python_cases)
        for case in python_cases:
            self.assertEqual(
                case["effects"]["network"], "denied-by-python-audit-hook"
            )
            self.assertEqual(
                case["effects"]["filesystem"]["before_tree_sha256"],
                case["effects"]["filesystem"]["after_tree_sha256"],
            )
        for case in self.artifact["cases"]:
            if case["layer"] != "accepted-a8-oracle":
                continue
            for field in ("network", "process", "browser"):
                self.assertEqual(
                    case["effects"][field],
                    "not-assessed-by-a8-runtime-fixture",
                )
            self.assertIsNone(case["outcome"]["stdout_terminated"])
            self.assertIsNone(case["outcome"]["stderr_terminated"])

    def test_integrity_excludes_integrity_and_is_key_order_stable(self) -> None:
        expected = self.artifact["integrity"]["payload_sha256"]
        self.assertEqual(extractor.canonical_payload_sha256(self.artifact), expected)
        self.assertEqual(
            extractor.case_manifest_sha256(self.artifact["cases"]),
            extractor.EXPECTED_CASE_MANIFEST_SHA256,
        )
        self.assertEqual(
            self.artifact["integrity"]["case_manifest_sha256"],
            extractor.EXPECTED_CASE_MANIFEST_SHA256,
        )
        reordered = dict(reversed(list(self.artifact.items())))
        reordered["integrity"] = {
            **reordered["integrity"],
            "payload_sha256": "0" * 64,
        }
        self.assertEqual(extractor.canonical_payload_sha256(reordered), expected)

    def test_schema_and_semantics_reject_missing_duplicate_and_dangling_rows(self) -> None:
        missing = self._artifact()
        del missing["public_commands"][0]["json"]
        self.assertSchemaRejects(missing)

        duplicate = self._artifact()
        duplicate["public_commands"][1]["public_path"] = duplicate["public_commands"][0][
            "public_path"
        ]
        self.assertSemanticRejects(duplicate)

        dangling = self._artifact()
        dangling["public_commands"][0]["help"]["case_ids"] = ["missing.case"]
        self.assertSemanticRejects(dangling)

    def test_semantics_reject_exit_json_npm_source_and_portability_drift(self) -> None:
        mutations: list[dict[str, object]] = []

        wrong_exit = self._artifact()
        next(
            case
            for case in wrong_exit["cases"]
            if case["id"].startswith("python.invalid-usage.")
        )["outcome"]["exit_code"] = 0
        mutations.append(wrong_exit)

        wrong_json = self._artifact()
        next(
            case
            for case in wrong_json["cases"]
            if case["id"] == "python.handler.provider-list-json"
        )["outcome"]["json"]["canonical_sha256"] = "0" * 64
        mutations.append(wrong_json)

        wrong_npm = self._artifact()
        next(
            row
            for row in wrong_npm["npm_compatibility"]["dispatch"]
            if row["raw_command"] == "update"
        )["normalized_command"] = "self-update"
        mutations.append(wrong_npm)

        wrong_source = self._artifact()
        wrong_source["source"]["ctr_201b"]["payload_sha256"] = "0" * 64
        mutations.append(wrong_source)

        path_leak = self._artifact()
        path_leak["cases"][0]["outcome"]["stdout_lines"] = [str(REPO_ROOT)]
        mutations.append(path_leak)

        secret_leak = self._artifact()
        secret_leak["cases"][0]["outcome"]["stdout_lines"] = [extractor.CANARY_SECRET]
        mutations.append(secret_leak)

        for mutation in mutations:
            with self.subTest(index=mutations.index(mutation)):
                self.assertSemanticRejects(mutation)

    def test_semantics_reject_stream_digest_machine_path_and_secret_tampering(
        self,
    ) -> None:
        mutations: list[dict[str, object]] = []

        help_stream = self._artifact()
        help_stream["cases"][10]["outcome"]["stdout_lines"].append(
            "forged help line"
        )
        mutations.append(help_stream)

        entrypoint_digest = self._artifact()
        entrypoint_digest["console_entrypoints"][0]["root_help_sha256"] = "0" * 64
        mutations.append(entrypoint_digest)

        observation_digest = self._artifact()
        next(
            case
            for case in observation_digest["cases"]
            if case["id"].startswith("python.help.")
        )["entrypoint_observations"]["ql"] = "0" * 64
        mutations.append(observation_digest)

        machine_path = self._artifact()
        machine_path["cases"][0]["outcome"]["stdout_lines"] = [
            "/Users/other-machine/private.txt"
        ]
        mutations.append(machine_path)

        secret = self._artifact()
        secret["cases"][0]["outcome"]["stdout_lines"] = [
            "sk-abcdefghijklmnop"
        ]
        mutations.append(secret)

        for index, mutation in enumerate(mutations):
            with self.subTest(index=index):
                self.assertSemanticRejects(mutation)

    def test_cross_field_guards_reject_source_case_route_and_provenance_drift(
        self,
    ) -> None:
        mutations: list[dict[str, object]] = []

        a8_source = self._artifact()
        a8_source["source"]["a8_manifest"]["sha256"] = "0" * 64
        mutations.append(a8_source)

        python_tree = self._artifact()
        python_tree["source"]["python_package_tree"]["tree_sha256"] = "0" * 64
        mutations.append(python_tree)

        duplicate_entrypoint = self._artifact()
        duplicate_entrypoint["console_entrypoints"][1] = copy.deepcopy(
            duplicate_entrypoint["console_entrypoints"][0]
        )
        mutations.append(duplicate_entrypoint)

        duplicate_taxonomy = self._artifact()
        duplicate_taxonomy["error_taxonomy"][1] = copy.deepcopy(
            duplicate_taxonomy["error_taxonomy"][0]
        )
        mutations.append(duplicate_taxonomy)

        duplicate_npm = self._artifact()
        duplicate_npm["npm_compatibility"]["dispatch"][1] = copy.deepcopy(
            duplicate_npm["npm_compatibility"]["dispatch"][0]
        )
        mutations.append(duplicate_npm)

        a8_case = self._artifact()
        next(
            case for case in a8_case["cases"] if case["id"] == "a8.python.cli-align"
        )["source_case_sha256"] = "0" * 64
        mutations.append(a8_case)

        provenance = self._artifact()
        next(
            case for case in provenance["cases"] if case["id"].startswith("python.help.")
        )["effects"]["network"] = "not-assessed-by-a8-runtime-fixture"
        mutations.append(provenance)

        for index, mutation in enumerate(mutations):
            self._rehash(mutation)
            with self.subTest(index=index), patch.object(
                extractor,
                "EXPECTED_PAYLOAD_SHA256",
                mutation["integrity"]["payload_sha256"],
            ):
                with self.assertRaises(extractor.RuntimeInventoryMismatch):
                    extractor._validate_expected_artifact(mutation, REPO_ROOT)

    def test_case_manifest_rejects_synchronized_python_evidence_forgery(self) -> None:
        mutations: list[dict[str, object]] = []

        help_stream = self._artifact()
        help_case = next(
            case for case in help_stream["cases"] if case["id"].startswith("python.help.")
        )
        help_case["outcome"]["stdout_lines"].append("synchronized forgery")
        help_case["entrypoint_observations"]["qiongli"] = extractor._outcome_sha256(
            help_case["outcome"]
        )
        mutations.append(help_stream)

        observation = self._artifact()
        next(
            case for case in observation["cases"] if case["id"].startswith("python.help.")
        )["entrypoint_observations"]["ql"] = "0" * 64
        mutations.append(observation)

        filesystem = self._artifact()
        filesystem_case = next(
            case for case in filesystem["cases"] if case["id"].startswith("python.help.")
        )
        filesystem_case["effects"]["filesystem"]["before_tree_sha256"] = "1" * 64
        filesystem_case["effects"]["filesystem"]["after_tree_sha256"] = "1" * 64
        mutations.append(filesystem)

        for index, mutation in enumerate(mutations):
            mutation["integrity"]["case_manifest_sha256"] = (
                extractor.case_manifest_sha256(mutation["cases"])
            )
            self._rehash(mutation)
            with self.subTest(index=index), patch.object(
                extractor,
                "EXPECTED_PAYLOAD_SHA256",
                mutation["integrity"]["payload_sha256"],
            ):
                with self.assertRaises(extractor.RuntimeInventoryMismatch):
                    extractor._validate_expected_artifact(mutation, REPO_ROOT)

    def test_cli_reports_checked_drift_as_fail_exit_one(self) -> None:
        drifted = self._artifact()
        drifted["integrity"]["payload_sha256"] = "0" * 64
        stdout = io.StringIO()
        with redirect_stdout(stdout), patch.object(
            extractor, "extract_cli_runtime_inventory", return_value=self.artifact
        ), patch.object(extractor, "_load_checked_output", return_value=drifted):
            self.assertEqual(extractor.main(["--check", "--json"]), 1)
        payload = json.loads(stdout.getvalue())
        self.assertEqual(payload["status"], "fail")
        self.assertEqual(payload["exit_code"], 1)
        self.assertEqual(payload["code"], "accepted-cli-runtime-inventory-mismatch")

    def test_completion_boundary_does_not_claim_rust_or_parent_completion(self) -> None:
        coverage = self.artifact["coverage"]
        self.assertTrue(coverage["cli_inventory_completion_ready"])
        self.assertEqual(coverage["ctr_201"], "in-progress")
        self.assertEqual(coverage["ctr_202"], "not-complete")
        self.assertEqual(coverage["fnd_202"], "not-implemented")
        self.assertEqual(coverage["rust_cli"], "not-implemented")
        self.assertEqual(coverage["cross_platform_runtime_parity"], "not-claimed")
        self.assertEqual(
            self.artifact["compatibility_boundary"]["remaining_ctr_201_blocker"],
            "accepted-source-orchestrator-runtime-closure",
        )


@unittest.skipUnless(
    sys.platform.startswith("linux"),
    "canonical CTR-201E re-extraction runs only in the Ubuntu full tier",
)
class Ctr201CliRuntimeCanonicalExtractionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        if sys.version_info[:2] != (3, 12):
            raise unittest.SkipTest("CTR-201E extraction is pinned to Python 3.12")
        cls.checked = json.loads(ARTIFACT_PATH.read_text(encoding="utf-8"))
        cls.extracted = extractor.extract_cli_runtime_inventory(REPO_ROOT)

    def test_dual_environment_capture_matches_checked_artifact(self) -> None:
        self.assertEqual(self.extracted, self.checked)

    def test_tag_and_accepted_source_bindings_are_revalidated(self) -> None:
        extractor._verify_tag_binding(REPO_ROOT)
        _, _, _, static_artifact, _ = extractor._read_bound_sources(REPO_ROOT)
        self.assertEqual(
            static_artifact["integrity"]["payload_sha256"],
            self.checked["source"]["ctr_201b"]["payload_sha256"],
        )

    def test_runtime_environment_is_sanitized_and_variant_distinct(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            first = extractor._runtime_environment(root, "a")
            second = extractor._runtime_environment(root, "b")
        for environment in (first, second):
            self.assertEqual(environment["TZ"], "UTC")
            self.assertEqual(environment["COLUMNS"], "80")
            self.assertEqual(environment["CTR201E_CANARY_SECRET"], extractor.CANARY_SECRET)
            for secret_name in (
                "OPENAI_API_KEY",
                "ANTHROPIC_API_KEY",
                "GITHUB_TOKEN",
                "HTTP_PROXY",
                "HTTPS_PROXY",
            ):
                self.assertNotIn(secret_name, environment)
        self.assertEqual(first["PATH"], "")
        self.assertTrue(second["PATH"].endswith("unused-bin"))
        self.assertNotIn("PYTHONHASHSEED", first)
        self.assertNotIn("PYTHONHASHSEED", second)
        self.assertNotEqual(first["HOME"], second["HOME"])

    def test_cli_check_and_generation_are_explicit_and_redacted(self) -> None:
        stdout = io.StringIO()
        with redirect_stdout(stdout), patch.object(
            extractor, "extract_cli_runtime_inventory", return_value=self.checked
        ):
            self.assertEqual(extractor.main(["--check", "--json"]), 0)
        self.assertEqual(
            json.loads(stdout.getvalue())["code"],
            "accepted-cli-runtime-inventory-matches",
        )

        stdout = io.StringIO()
        with redirect_stdout(stdout), patch.object(
            extractor, "extract_cli_runtime_inventory"
        ) as capture:
            self.assertEqual(extractor.main(["--json"]), 2)
        capture.assert_not_called()
        self.assertEqual(
            json.loads(stdout.getvalue())["code"],
            "accepted-cli-runtime-inventory-unavailable",
        )

    def test_tag_binding_fails_closed_when_git_identity_moves(self) -> None:
        completed = type(
            "Completed",
            (),
            {"returncode": 0, "stdout": "0" * 40 + "\n", "stderr": ""},
        )()
        with patch.object(extractor.subprocess, "run", return_value=completed):
            with self.assertRaises(extractor.RuntimeInventoryMismatch):
                extractor._verify_tag_binding(REPO_ROOT)


if __name__ == "__main__":
    unittest.main()
