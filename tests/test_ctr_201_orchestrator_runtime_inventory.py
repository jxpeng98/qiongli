from __future__ import annotations

import copy
import io
import json
import math
import sys
import tempfile
import unittest
from contextlib import redirect_stdout
from pathlib import Path
from typing import Any
from unittest.mock import patch

from tooling.scripts import extract_ctr_201_orchestrator_runtime_inventory as extractor
from tooling.scripts.validate_capability_contract import validate_instance


REPO_ROOT = Path(__file__).resolve().parents[1]
ARTIFACT_PATH = REPO_ROOT / "tooling/migration/ctr-201-orchestrator-runtime.json"
SCHEMA_PATH = REPO_ROOT / "tooling/migration/ctr-201-orchestrator-runtime.schema.json"


class Ctr201OrchestratorRuntimeCheckedArtifactTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.schema = json.loads(SCHEMA_PATH.read_text(encoding="utf-8"))
        cls.artifact = json.loads(ARTIFACT_PATH.read_text(encoding="utf-8"))
        cls.case_by_id = {case["id"]: case for case in cls.artifact["cases"]}

    def _artifact(self) -> dict[str, Any]:
        return copy.deepcopy(self.artifact)

    @staticmethod
    def _facts(case: dict[str, Any], field: str) -> dict[str, str]:
        return {row["key"]: row["value"] for row in case[field]}

    @staticmethod
    def _synchronize_digests(artifact: dict[str, Any]) -> None:
        for case in artifact["cases"]:
            case["case_sha256"] = extractor.canonical_case_sha256(case)
        artifact["integrity"]["case_manifest_sha256"] = (
            extractor.case_manifest_sha256(artifact["cases"])
        )
        artifact["integrity"]["payload_sha256"] = (
            extractor.canonical_payload_sha256(artifact)
        )

    def assertSchemaRejects(self, artifact: dict[str, Any]) -> None:  # noqa: N802
        self.assertTrue(validate_instance(artifact, self.schema))

    def assertFullyRehashedSemanticRejects(  # noqa: N802
        self,
        artifact: dict[str, Any],
    ) -> None:
        self._synchronize_digests(artifact)
        with patch.object(
            extractor,
            "EXPECTED_PAYLOAD_SHA256",
            artifact["integrity"]["payload_sha256"],
        ), patch.object(
            extractor,
            "EXPECTED_CASE_MANIFEST_SHA256",
            artifact["integrity"]["case_manifest_sha256"],
        ), self.assertRaises((extractor.InventoryMismatch, extractor.ExtractorError)):
            extractor.validate_runtime_artifact(artifact)

    def test_checked_artifact_matches_closed_schema_and_semantic_validator(self) -> None:
        self.assertEqual(validate_instance(self.artifact, self.schema), [])
        extractor.validate_runtime_artifact(self.artifact)
        self.assertEqual(self.artifact["task_id"], "CTR-201F")
        self.assertEqual(
            self.artifact["status"],
            "runtime-inventory-freeze-captured",
        )
        self.assertEqual(
            self.artifact["integrity"]["payload_sha256"],
            "9232b0a3c2ba223c860244142054940229e435e00735261bd7db834c7a94faab",
        )
        self.assertEqual(
            self.artifact["integrity"]["case_manifest_sha256"],
            "676b7b269889da02bbb928b29fa254e40ca8794f5bf2e199477364f330debddd",
        )

    def test_checked_schema_is_generated_from_the_artifact_and_recursively_closed(
        self,
    ) -> None:
        self.assertEqual(extractor.build_runtime_schema(self.artifact), self.schema)

        object_nodes: list[str] = []

        def visit(node: Any, path: str) -> None:
            if isinstance(node, dict):
                if node.get("type") == "object":
                    object_nodes.append(path)
                    self.assertIs(
                        node.get("additionalProperties"),
                        False,
                        f"open object schema at {path}",
                    )
                    properties = node.get("properties", {})
                    self.assertEqual(
                        set(node.get("required", [])),
                        set(properties),
                        f"partial required set at {path}",
                    )
                for key, value in node.items():
                    visit(value, f"{path}/{key}")
            elif isinstance(node, list):
                for index, value in enumerate(node):
                    visit(value, f"{path}/{index}")

        visit(self.schema, "$")
        self.assertGreaterEqual(len(object_nodes), 20)

        extra = self._artifact()
        extra["unexpected"] = True
        self.assertSchemaRejects(extra)

        nested_extra = self._artifact()
        nested_extra["cases"][0]["outcome"]["unexpected"] = True
        self.assertSchemaRejects(nested_extra)

    def test_source_is_exact_tag_and_dependency_digest_bound(self) -> None:
        source = self.artifact["source"]
        self.assertEqual(source["accepted_tag"], "v1.19.0-beta.1")
        self.assertEqual(
            source["accepted_commit"],
            "8d2e99866ce4c4efb8b3b5e0265c0c1f89a36b0f",
        )
        self.assertEqual(
            source["a8_manifest"]["sha256"],
            extractor.MANIFEST_SHA256,
        )
        self.assertEqual(
            source["python_full_oracle"]["sha256"],
            extractor.PYTHON_ORACLE_SHA256,
        )
        self.assertEqual(
            source["ctr_201c"]["payload_sha256"],
            extractor.STATIC_ARTIFACT_PAYLOAD_SHA256,
        )
        self.assertEqual(
            source["ctr_201d"]["payload_sha256"],
            extractor.CONTENT_ARTIFACT_PAYLOAD_SHA256,
        )
        self.assertEqual(source["package_trees"], [extractor.PYTHON_TREE, extractor.CONTENT_TREE])
        self.assertEqual(len(source["blob_anchors"]), 4)

    def test_case_dimension_and_decision_closure_is_exact(self) -> None:
        cases = self.artifact["cases"]
        dimensions = self.artifact["behavior_dimensions"]
        decisions = self.artifact["disposition_decisions"]

        self.assertEqual([case["id"] for case in cases], list(extractor.CASE_IDS))
        self.assertEqual(
            [case["declaration_ordinal"] for case in cases],
            list(range(len(extractor.CASE_IDS))),
        )
        self.assertEqual(
            [dimension["id"] for dimension in dimensions],
            list(extractor.DIMENSION_IDS),
        )
        self.assertEqual(dimensions, extractor._behavior_dimensions())
        self.assertEqual(decisions, [dict(item) for item in extractor.DISPOSITIONS])

        case_ids = set(extractor.CASE_IDS)
        dimension_ids = set(extractor.DIMENSION_IDS)
        decision_ids = {item["id"] for item in extractor.DISPOSITIONS}
        case_links: set[tuple[str, str]] = set()
        dimension_links: set[tuple[str, str]] = set()
        self.assertEqual(set(dimensions[0]["case_ids"]), case_ids)
        for case in cases:
            self.assertTrue(case["dimension_ids"])
            self.assertTrue(set(case["dimension_ids"]).issubset(dimension_ids))
            self.assertEqual(
                case["dimension_ids"], extractor._case_dimension_ids(case["id"])
            )
            case_links.update(
                (dimension_id, case["id"])
                for dimension_id in case["dimension_ids"]
            )
            self.assertEqual(
                case["case_sha256"],
                extractor.canonical_case_sha256(case),
            )
        for dimension in dimensions:
            self.assertTrue(set(dimension["case_ids"]).issubset(case_ids))
            self.assertTrue(set(dimension["decision_ids"]).issubset(decision_ids))
            dimension_links.update(
                (dimension["id"], case_id) for case_id in dimension["case_ids"]
            )
        self.assertEqual(case_links, dimension_links)

        coverage = self.artifact["coverage"]
        self.assertEqual(coverage["case_count"], 44)
        self.assertEqual(coverage["bounded_runtime_case_count"], 43)
        self.assertEqual(coverage["accepted_a8_case_count"], 1)
        self.assertEqual(coverage["resolved_dimension_count"], 6)
        self.assertEqual(coverage["disposition_decision_count"], 6)
        self.assertEqual(coverage["required_not_fully_captured_count"], 0)
        self.assertTrue(coverage["completion_ready"])

    def test_dual_environment_is_sanitized_and_distinct_without_real_agent_paths(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            first = extractor._worker_environment(root, "a")
            second = extractor._worker_environment(root, "b")

        for environment in (first, second):
            self.assertEqual(environment["TZ"], "UTC")
            self.assertEqual(environment["LANG"], "C.UTF-8")
            self.assertEqual(environment["LC_ALL"], "C.UTF-8")
            self.assertEqual(environment["NO_COLOR"], "1")
            self.assertEqual(environment["CTR201F_CANARY_SECRET"], extractor.CANARY_SECRET)
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
        self.assertNotEqual(first["HOME"], second["HOME"])
        self.assertNotEqual(first["TMPDIR"], second["TMPDIR"])

        contract = self.artifact["capture_contract"]
        self.assertEqual(
            contract["determinism"],
            "two-distinct-temp-roots-and-environments-byte-equivalent",
        )
        self.assertEqual(
            contract["trace_order"],
            "logical-order-preserved-only-concurrent-cohorts-canonicalized",
        )
        self.assertEqual(
            contract["fake_boundary"],
            "bridge-mcp-availability-and-deterministic-identity-injection",
        )
        self.assertEqual(contract["network_policy"], "python-audit-denied")
        self.assertIn("worker-denies-child-processes", contract["process_policy"])
        self.assertIn("worker-denies-sut-writes", contract["write_policy"])

    def test_all_cases_preserve_the_no_sut_write_boundary(self) -> None:
        empty_manifest_sha256 = extractor._sha256(extractor._canonical_json_bytes([]))
        for case in self.artifact["cases"]:
            with self.subTest(case=case["id"]):
                effects = case["effects"]
                self.assertEqual(effects["before_tree_sha256"], effects["after_tree_sha256"])
                self.assertEqual(effects["changed_path_count"], 0)
                self.assertEqual(effects["changed_paths_sha256"], empty_manifest_sha256)
                self.assertIn(
                    case["provenance"],
                    {"accepted-a8-oracle", "accepted-source-bounded-runtime"},
                )

        rendered = json.dumps(self.artifact, ensure_ascii=False, sort_keys=True)
        self.assertNotIn(str(REPO_ROOT), rendered)
        self.assertNotIn(extractor.CANARY_SECRET, rendered)
        self.assertEqual(
            self.artifact["compatibility_boundary"]["real_agent_execution"],
            "not-executed",
        )
        self.assertEqual(
            self.artifact["compatibility_boundary"]["real_provider_network"],
            "not-executed",
        )

    def test_accepted_controller_and_mcp_quirks_are_frozen(self) -> None:
        boolean_case = self.case_by_id["mcp.run-agents-boolean-contract"]
        self.assertEqual(
            self._facts(boolean_case, "input_facts")["accepted_type"],
            "JSON boolean",
        )
        self.assertEqual(
            self._facts(boolean_case, "result_facts")["string_true_rejected"],
            "true",
        )

        advisory = self.case_by_id["mcp.doctor-advisory-run-agents"]
        self.assertEqual(
            self._facts(advisory, "input_facts")["doctor_requirement"],
            "advisory-not-enforced",
        )
        self.assertEqual(
            self._facts(advisory, "result_facts")["doctor_calls_during_task_run"],
            "0",
        )
        self.assertEqual([row["stage"] for row in advisory["trace"]], ["draft", "review"])

        route = self._facts(
            self.case_by_id["mcp.orchestrator-route-matrix"], "result_facts"
        )
        self.assertEqual(route["orchestrator_route"], "orchestrator_mcp")
        self.assertEqual(route["orchestrator_run_agents"], "false")
        self.assertEqual(route["route_safety_claims_doctor_gate"], "true")

        doctor = self._facts(
            self.case_by_id["doctor.sanitized-environment"], "result_facts"
        )
        self.assertEqual(
            json.loads(doctor["cli_statuses"]),
            {"codex": "warning", "claude": "warning", "antigravity": "warning"},
        )

        solo = self.case_by_id["task-run.solo-observed-review"]
        self.assertEqual(self._facts(solo, "input_facts")["execution_mode"], "solo")
        self.assertEqual([row["stage"] for row in solo["trace"]], ["draft", "review"])

        direct_triad = self.case_by_id["task-run.direct-triad-metadata-only"]
        enabled_triad = self.case_by_id["task-run.triad-enabled"]
        self.assertEqual(self._facts(direct_triad, "input_facts")["triad_flag"], "false")
        self.assertNotIn("triad", [row["stage"] for row in direct_triad["trace"]])
        self.assertEqual(self._facts(enabled_triad, "input_facts")["triad_flag"], "true")
        self.assertEqual(
            [row["stage"] for row in enabled_triad["trace"]],
            ["draft", "review", "triad"],
        )

    def test_quality_and_native_worker_accepted_absences_are_frozen(self) -> None:
        quality = self.case_by_id["quality.artifact-existence-gate"]
        self.assertEqual(
            self._facts(quality, "input_facts")["gate_kind"],
            "artifact-existence-only",
        )
        self.assertEqual(
            self._facts(quality, "result_facts")["semantic_execution"],
            "false",
        )
        self.assertEqual(
            self.artifact["compatibility_boundary"]["semantic_quality_gate_execution"],
            "accepted-absent",
        )

        adapter = self.case_by_id["worker.adapter-fallback"]
        self.assertEqual(
            self._facts(adapter, "input_facts")["native_dispatch"],
            "false",
        )
        self.assertEqual(
            json.loads(self._facts(adapter, "result_facts")["effective_adapters"]),
            {
                "auto": "generic_prompt",
                "codex_subagent": "generic_prompt",
                "claude_cowork": "generic_prompt",
            },
        )
        self.assertEqual(
            self.artifact["compatibility_boundary"]["native_worker_adapter_dispatch"],
            "accepted-generic-fallback",
        )
        h3_workers = [
            row
            for row in self.case_by_id["worker.h3-block"]["trace"]
            if row["stage"] == "worker"
        ]
        self.assertEqual([row["success"] for row in h3_workers].count(False), 1)

    def test_review_failure_and_state_boundaries_remain_bounded(self) -> None:
        revision = self.case_by_id["task-run.block-revision-pass"]
        self.assertEqual(
            [row["stage"] for row in revision["trace"]],
            ["draft", "review", "revision", "review"],
        )
        final_block = self.case_by_id["task-run.final-block"]
        self.assertEqual(self._facts(final_block, "input_facts")["scenario"], "final-block")
        replay = self.case_by_id["experience.replay-plan-advisory"]
        self.assertEqual(
            self._facts(replay, "result_facts")["execution_performed"],
            "false",
        )
        self.assertEqual(
            self.artifact["compatibility_boundary"]["task_team_checkpoint_resume"],
            "accepted-absent",
        )

    def test_logical_trace_preserves_only_real_concurrent_cohorts(self) -> None:
        parallel = self.case_by_id["execute.parallel-triad"]["trace"]
        self.assertEqual([row["ordering"] for row in parallel[:3]], ["concurrent"] * 3)
        self.assertEqual(
            {row["logical_cohort_ordinal"] for row in parallel[:3]},
            {0},
        )
        self.assertEqual(parallel[3]["stage"], "parallel-synthesis")
        self.assertEqual(parallel[3]["ordering"], "sequential")

        generic_workers = self.case_by_id["worker.b1-success"]["trace"]
        self.assertTrue(all(row["ordering"] == "sequential" for row in generic_workers))
        self.assertEqual(
            [row["stage"] for row in generic_workers],
            [
                "worker",
                "worker",
                "worker-merge",
                "worker-final-review",
                "draft",
                "review",
            ],
        )

        team = self.case_by_id["team-run.b1-degrade"]["trace"]
        team_workers = [row for row in team if row["stage"] == "worker"]
        self.assertEqual([row["ordering"] for row in team_workers], ["concurrent"] * 3)
        self.assertEqual(
            {row["logical_cohort_ordinal"] for row in team_workers},
            {1},
        )

    def test_worker_team_and_code_build_failure_projections_are_frozen(self) -> None:
        worker_expectations = {
            "worker.b1-degrade": ("degraded", "passed", "PASS"),
            "worker.b1-merge-failure": ("ok", "merge_failed", ""),
            "worker.b1-final-review-failure": ("ok", "failed", ""),
            "worker.b1-final-review-block": ("ok", "blocked", "BLOCK"),
            "worker.h3-block": ("blocked", "skipped", ""),
        }
        for case_id, (barrier, review_status, verdict) in worker_expectations.items():
            with self.subTest(case=case_id):
                facts = self._facts(self.case_by_id[case_id], "result_facts")
                self.assertEqual(facts["worker_barrier_status"], barrier)
                self.assertEqual(facts["worker_final_review_status"], review_status)
                self.assertEqual(facts["worker_final_review_verdict"], verdict)

        team_block = self._facts(
            self.case_by_id["team-run.b1-all-workers-block"], "result_facts"
        )
        self.assertEqual(team_block["barrier_status"], "blocked")
        self.assertEqual(team_block["merge_executed"], "false")
        review_block = self._facts(
            self.case_by_id["team-run.b1-review-block-observed"], "result_facts"
        )
        self.assertEqual(review_block["review_block_observed"], "true")
        self.assertEqual(review_block["confidence"], "0.92")

        advanced_failure = self._facts(
            self.case_by_id["code-build.legacy-advanced-failure"], "result_facts"
        )
        self.assertEqual(advanced_failure["mode"], "chain")
        self.assertEqual(advanced_failure["confidence"], "0.0")
        self.assertEqual(
            self.artifact["compatibility_boundary"]["strict_topic_code_build"],
            "not-captured-no-write-disposition",
        )

    def test_profile_and_experience_facts_are_derived_from_runtime_results(self) -> None:
        profile = self._facts(
            self.case_by_id["profile.builtin-and-custom-resolution"], "result_facts"
        )
        self.assertEqual(profile["parallel_codex_timeout_seconds"], "123")
        self.assertEqual(profile["draft_timeout_seconds"], "123")
        self.assertEqual(profile["parallel_profile_applied"], "true")
        self.assertEqual(profile["review_profile_applied"], "true")

        replay = self._facts(
            self.case_by_id["experience.replay-plan-advisory"], "result_facts"
        )
        self.assertEqual(replay["failed_validator_status"], "failed")
        self.assertEqual(replay["failed_next_action"], "rerun_after_addressing_failures")
        self.assertEqual(replay["passed_validator_status"], "passed")
        self.assertEqual(replay["passed_next_action"], "no_rerun_needed")

    def test_strict_json_loader_rejects_duplicate_nonfinite_and_surrogate_values(self) -> None:
        payloads = (
            b'{"key":1,"key":1}',
            b'{"key":NaN}',
            b'{"key":"\\ud800"}',
            b'{"\\ud800":1}',
            b'{"key":' + (b"[" * 66) + b"0" + (b"]" * 66) + b"}",
        )
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "fixture.json"
            for payload in payloads:
                path.write_bytes(payload)
                with self.subTest(payload=payload), self.assertRaises(extractor.ExtractorError):
                    extractor._load_json(path, label="fixture")

    def test_case_manifest_root_rejects_synchronized_case_and_payload_forgery(self) -> None:
        mutation = self._artifact()
        mutation["cases"][1]["operation"] = "synchronized forged operation"
        mutation["cases"][1]["case_sha256"] = extractor.canonical_case_sha256(
            mutation["cases"][1]
        )
        mutation["integrity"]["case_manifest_sha256"] = extractor.case_manifest_sha256(
            mutation["cases"]
        )
        mutation["integrity"]["payload_sha256"] = extractor.canonical_payload_sha256(
            mutation
        )
        with patch.object(
            extractor,
            "EXPECTED_PAYLOAD_SHA256",
            mutation["integrity"]["payload_sha256"],
        ), self.assertRaises(extractor.InventoryMismatch):
            extractor.validate_runtime_artifact(mutation)

    def test_semantics_reject_source_dimension_decision_and_effect_forgery(self) -> None:
        mutations: list[dict[str, Any]] = []

        source_commit = self._artifact()
        source_commit["source"]["accepted_commit"] = "0" * 40
        mutations.append(source_commit)

        source_tree = self._artifact()
        source_tree["source"]["package_trees"][0]["tree_sha256"] = "0" * 64
        mutations.append(source_tree)

        source_anchor = self._artifact()
        source_anchor["source"]["blob_anchors"][0]["sha256"] = "0" * 64
        mutations.append(source_anchor)

        dimension = self._artifact()
        dimension["behavior_dimensions"][0]["resolution"] = "captured-live-parity"
        mutations.append(dimension)

        decision = self._artifact()
        decision["disposition_decisions"][0]["downstream_tasks"] = []
        mutations.append(decision)

        dangling_dimension = self._artifact()
        dangling_dimension["cases"][0]["dimension_ids"] = ["missing-dimension"]
        mutations.append(dangling_dimension)

        effect = self._artifact()
        effect["cases"][1]["effects"]["after_tree_sha256"] = "1" * 64
        effect["cases"][1]["effects"]["changed_path_count"] = 1
        mutations.append(effect)

        for index, mutation in enumerate(mutations):
            with self.subTest(index=index):
                self.assertFullyRehashedSemanticRejects(mutation)

    def test_semantics_reject_empty_or_asymmetric_case_dimension_links_after_full_rehash(
        self,
    ) -> None:
        empty_case_links = self._artifact()
        empty_case_links["cases"][1]["dimension_ids"] = []

        asymmetric_reverse_links = self._artifact()
        agent_dimension = next(
            dimension
            for dimension in asymmetric_reverse_links["behavior_dimensions"]
            if dimension["id"] == "complete-agent-launch-behavior"
        )
        agent_dimension["case_ids"].remove("mcp.run-agents-boolean-contract")

        for mutation in (empty_case_links, asymmetric_reverse_links):
            with self.subTest(mutation=mutation["cases"][1]["dimension_ids"]):
                self.assertFullyRehashedSemanticRejects(mutation)

    def test_semantics_reject_secret_path_and_callable_repr_after_full_rehash(self) -> None:
        values = (
            "sk-abcdefghijklmnop",
            "/Users/forged/private.txt",
            "<function forged at 0x1234>",
        )
        for value in values:
            mutation = self._artifact()
            mutation["cases"][1]["operation"] = value
            with self.subTest(value=value):
                self.assertFullyRehashedSemanticRejects(mutation)

    def test_nonfinite_and_surrogate_values_fail_canonicalization(self) -> None:
        self.assertFalse(math.isfinite(float("nan")))

        nonfinite = self._artifact()
        nonfinite["cases"][1]["result_facts"][0]["value"] = float("nan")
        with self.assertRaises(extractor.ExtractorError):
            extractor.canonical_case_sha256(nonfinite["cases"][1])

        surrogate = self._artifact()
        surrogate["cases"][1]["operation"] = "\ud800"
        with self.assertRaises((extractor.ExtractorError, UnicodeEncodeError)):
            extractor.canonical_case_sha256(surrogate["cases"][1])

        with self.assertRaises(extractor.ExtractorError):
            extractor._normalize_runtime_value(float("inf"), [])

    def test_case_rejects_unsafe_normalized_results_before_digesting(self) -> None:
        for value in (
            "/Users/fixture/private.txt",
            "sk-abcdefghijklmnop",
            "<function fixture at 0x1234>",
        ):
            with self.subTest(value=value), self.assertRaises(extractor.ExtractorError):
                extractor._case(
                    case_id="fixture",
                    group="fixture",
                    operation="fixture",
                    provenance="accepted-source-bounded-runtime",
                    dimension_ids=["complete-runtime-behavior-matrix"],
                    input_facts=[],
                    result_facts=[],
                    result={"value": value},
                    states=[],
                    before=[],
                    after=[],
                    replacements=[],
                )

    def test_semantics_reject_every_compatibility_false_claim(self) -> None:
        false_claims = {
            "real_agent_execution": "captured",
            "real_provider_network": "captured",
            "real_timeout_signal_cancel": "captured",
            "task_team_checkpoint_resume": "captured",
            "semantic_quality_gate_execution": "captured",
            "native_worker_adapter_dispatch": "native",
            "strict_topic_code_build": "captured",
            "plugin_marketplace_behavior": "captured",
            "rust_implementation": "implemented",
            "cross_platform_runtime_parity": "claimed",
        }
        for field, value in false_claims.items():
            mutation = self._artifact()
            mutation["compatibility_boundary"][field] = value
            with self.subTest(field=field):
                self.assertFullyRehashedSemanticRejects(mutation)

    def test_cli_check_reports_exit_zero_one_and_two(self) -> None:
        stdout = io.StringIO()
        with redirect_stdout(stdout), patch.object(
            extractor,
            "extract_orchestrator_runtime_inventory",
            return_value=self.artifact,
        ):
            self.assertEqual(extractor.main(["--check", "--json"]), 0)
        passed = json.loads(stdout.getvalue())
        self.assertEqual(passed["status"], "pass")
        self.assertEqual(passed["exit_code"], 0)
        self.assertEqual(
            passed["code"],
            "accepted-orchestrator-runtime-inventory-matches",
        )

        drifted = self._artifact()
        drifted["integrity"]["payload_sha256"] = "0" * 64

        def load_drift(path: Path, *, label: str) -> dict[str, Any]:
            del label
            return drifted if path.name == ARTIFACT_PATH.name else self.schema

        stdout = io.StringIO()
        with redirect_stdout(stdout), patch.object(
            extractor,
            "extract_orchestrator_runtime_inventory",
            return_value=self.artifact,
        ), patch.object(extractor, "_load_json", side_effect=load_drift):
            self.assertEqual(extractor.main(["--check", "--json"]), 1)
        failed = json.loads(stdout.getvalue())
        self.assertEqual(failed["status"], "fail")
        self.assertEqual(failed["exit_code"], 1)
        self.assertEqual(
            failed["code"],
            "accepted-orchestrator-runtime-inventory-mismatch",
        )

        stdout = io.StringIO()
        with redirect_stdout(stdout), patch.object(
            extractor,
            "extract_orchestrator_runtime_inventory",
            side_effect=extractor.ExtractorError("unavailable"),
        ):
            self.assertEqual(extractor.main(["--check", "--json"]), 2)
        unavailable = json.loads(stdout.getvalue())
        self.assertEqual(unavailable["status"], "error")
        self.assertEqual(unavailable["exit_code"], 2)
        self.assertEqual(
            unavailable["code"],
            "accepted-orchestrator-runtime-inventory-unavailable",
        )


@unittest.skipUnless(
    sys.platform.startswith("linux"),
    "canonical CTR-201F re-extraction runs only in the Ubuntu full tier",
)
class Ctr201OrchestratorRuntimeCanonicalExtractionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        if sys.version_info[:2] != (3, 12):
            raise unittest.SkipTest("CTR-201F extraction is pinned to Python 3.12")
        cls.checked = json.loads(ARTIFACT_PATH.read_text(encoding="utf-8"))
        cls.checked_schema = json.loads(SCHEMA_PATH.read_text(encoding="utf-8"))
        cls.extracted = extractor.extract_orchestrator_runtime_inventory(REPO_ROOT)

    def test_dual_environment_capture_matches_checked_artifact(self) -> None:
        self.assertEqual(self.extracted, self.checked)

    def test_canonical_schema_matches_checked_schema(self) -> None:
        self.assertEqual(
            extractor.build_runtime_schema(self.extracted),
            self.checked_schema,
        )

    def test_tag_and_fixed_digest_bindings_are_revalidated(self) -> None:
        extractor._verify_tag(REPO_ROOT)
        extractor.validate_runtime_artifact(self.extracted)


if __name__ == "__main__":
    unittest.main(verbosity=2)
