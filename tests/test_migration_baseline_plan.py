from __future__ import annotations

import copy
import json
import subprocess
import unittest
from pathlib import Path

from tooling.scripts.validate_capability_contract import validate_instance


REPO_ROOT = Path(__file__).resolve().parents[1]
PLAN_PATH = REPO_ROOT / "tooling/migration/qiongli-1x-baseline-plan.json"
SCHEMA_PATH = REPO_ROOT / "tooling/migration/baseline-plan.schema.json"
BASELINE_MANIFEST_PATH = (
    REPO_ROOT / "tooling/migration/baselines/v1.19.0-beta.1/manifest.json"
)
EXPECTED_TAG = "v1.19.0-beta.1"
EXPECTED_COMMIT = "8d2e99866ce4c4efb8b3b5e0265c0c1f89a36b0f"


class MigrationBaselinePlanTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.plan = json.loads(PLAN_PATH.read_text(encoding="utf-8"))
        cls.schema = json.loads(SCHEMA_PATH.read_text(encoding="utf-8"))
        cls.baseline_manifest = json.loads(
            BASELINE_MANIFEST_PATH.read_text(encoding="utf-8")
        )

    def _load_frozen_json_reference(self, relative_path: str) -> dict[str, object]:
        records = [
            record
            for domain in self.baseline_manifest["domains"]
            for record in domain["files"]
            if record["path"] == relative_path
        ]
        self.assertEqual(len(records), 1, relative_path)
        result = subprocess.run(
            [
                "git",
                "-C",
                str(REPO_ROOT),
                "cat-file",
                "blob",
                records[0]["git_blob_oid"],
            ],
            check=False,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        payload = json.loads(result.stdout)
        self.assertIsInstance(payload, dict)
        return payload

    def test_plan_matches_schema(self) -> None:
        self.assertEqual(validate_instance(self.plan, self.schema), [])
        self.assertEqual(self.plan["schema_version"], "1.1")
        self.assertEqual(self.plan["status"], "captured")

    def test_plan_pins_the_accepted_annotated_tag_and_finalized_inputs(self) -> None:
        lineage = self.plan["release_lineage"]
        self.assertEqual(lineage["accepted_tag"], EXPECTED_TAG)
        self.assertEqual(lineage["accepted_commit"], EXPECTED_COMMIT)
        self.assertEqual(
            self.plan["capture_inputs"]["finalized_receipt"],
            "tooling/release/acceptance/v1.19.0-beta.1-receipt.md",
        )
        self.assertEqual(
            self.plan["capture_inputs"]["finalized_receipt_commit"],
            "ba4517c8dfd5ce8b551c83b129213e689d32cac4",
        )
        self.assertEqual(
            self.plan["capture_inputs"]["finalized_receipt_git_blob_oid"],
            "605ab151b1621838a85f9909d6877f0f69857fc3",
        )
        self.assertEqual(
            self.plan["capture_inputs"]["finalized_receipt_sha256"],
            "a462dc24d94debfb678038e9ed437bdf04dc75476237cc74a9bf06ac366444e9",
        )
        self.assertEqual(
            self.plan["capture_inputs"]["finalized_receipt_size_bytes"], 6641
        )
        self.assertEqual(len(self.plan["capture_inputs"]["release_asset_names"]), 10)
        self.assertEqual(len(self.plan["capture_inputs"]["native_container_names"]), 5)
        self.assertEqual(
            set(self.plan["capture_inputs"]["native_receipt_labels"]),
            set(self.plan["capture_inputs"]["native_container_names"]),
        )
        self.assertEqual(
            len(set(self.plan["capture_inputs"]["native_receipt_labels"].values())),
            5,
        )
        bindings = self.plan["capture_inputs"]["native_member_bindings"]
        self.assertEqual(set(bindings), set(self.plan["capture_inputs"]["native_container_names"]))
        self.assertTrue(
            all(
                set(binding)
                == {
                    "identity_member",
                    "binary_member",
                    "identity_document_sha256",
                }
                for binding in bindings.values()
            )
        )
        self.assertEqual(
            self.plan["output"]["generated_from"], "accepted-annotated-tag"
        )

    def test_selector_semantics_are_explicit_and_filesystem_independent(self) -> None:
        semantics = self.plan["selector_semantics"]
        self.assertIn("git ls-tree", semantics["universe"])
        self.assertIn("repository-relative", semantics["path_format"])
        self.assertIn("one path segment", semantics["star"])
        self.assertIn("complete path segments", semantics["double_star"])
        self.assertIn("after", semantics["exclude_policy"])
        self.assertIn("filesystem traversal", semantics["unsupported"])

    def test_plan_covers_every_required_inventory_and_normalization_domain(self) -> None:
        inventory_ids = {item["id"] for item in self.plan["inventory"]["domains"]}
        self.assertEqual(
            inventory_ids,
            {
                "mcp",
                "cli",
                "skills",
                "tasks",
                "roles",
                "workflows",
                "subjects",
                "templates",
                "installers",
                "mutable-state",
                "orchestrator-scenarios",
            },
        )
        normalization_ids = {
            item["id"] for item in self.plan["normalization"]["rules"]
        }
        self.assertEqual(
            normalization_ids,
            {"paths", "ports", "timestamps", "process-ids", "secrets"},
        )

    def test_source_references_are_safe_and_resolve_to_repository_anchors(self) -> None:
        referenced_paths = [
            self.plan["contracts"]["capability_registry"],
            self.plan["contracts"]["golden_fixture"],
            self.plan["contracts"]["conformance_test"],
            *(oracle["source_root"] for oracle in self.plan["oracles"]),
        ]
        referenced_paths.extend(
            selector
            for domain in self.plan["inventory"]["domains"]
            for selector in domain["source_selectors"]
        )
        referenced_paths.extend(
            selector
            for oracle in self.plan["oracles"]
            for selector in oracle["projection"]["source_selectors"]
        )

        for reference in referenced_paths:
            with self.subTest(reference=reference):
                relative = Path(reference)
                self.assertFalse(relative.is_absolute())
                self.assertNotIn("..", relative.parts)
                wildcard_at = min(
                    (index for index in (reference.find("*"), reference.find("?")) if index >= 0),
                    default=len(reference),
                )
                anchor_text = reference[:wildcard_at].rstrip("/")
                anchor = Path(anchor_text)
                if not (REPO_ROOT / anchor).exists():
                    anchor = anchor.parent
                self.assertTrue((REPO_ROOT / anchor).exists())

    def test_contract_and_golden_sources_are_referenced_not_duplicated(self) -> None:
        contracts = self.plan["contracts"]
        # The 1.x plan is immutable while its canonical source paths continue to
        # evolve on 2.x. Resolve both references through the frozen manifest so
        # this assertion compares artifacts from the same accepted baseline.
        registry = self._load_frozen_json_reference(
            contracts["capability_registry"]
        )
        fixture = self._load_frozen_json_reference(
            contracts["golden_fixture"]
        )
        registry_smoke_ids = {
            smoke_id
            for tool in registry["tools"]
            for smoke_id in tool["smoke_call_ids"]
        }
        fixture_ids = {
            call["id"] for call in fixture["calls"] if isinstance(call.get("id"), str)
        }
        self.assertEqual(contracts["registry_copy_policy"], "reference-only")
        self.assertEqual(contracts["golden_selection"], "registry.tools[*].smoke_call_ids")
        self.assertTrue(registry_smoke_ids)
        self.assertLessEqual(registry_smoke_ids, fixture_ids)
        keys: set[str] = set()

        def collect_keys(value: object) -> None:
            if isinstance(value, dict):
                keys.update(value)
                for item in value.values():
                    collect_keys(item)
            elif isinstance(value, list):
                for item in value:
                    collect_keys(item)

        collect_keys(self.plan)
        self.assertTrue({"tools", "smoke_call_ids", "error_taxonomy"}.isdisjoint(keys))

    def test_oracles_are_capture_only_and_testkit_is_the_native_consumer(self) -> None:
        oracle_projection = {
            oracle["id"]: (oracle["runtime"], oracle["profile"], oracle["source_root"])
            for oracle in self.plan["oracles"]
        }
        self.assertEqual(
            oracle_projection,
            {
                "python-full": ("python", "full", "packages/python-qiongli"),
                "rust-lite": (
                    "rust",
                    "marketplace-lite",
                    "packages/qiongli-lite-mcp",
                ),
                "node-mcpb": (
                    "node",
                    "legacy-mcpb",
                    "packages/qiongli-literature-mcpb",
                ),
            },
        )
        self.assertTrue(
            all(oracle["production_dependency"] is False for oracle in self.plan["oracles"])
        )
        self.assertEqual(self.plan["consumer"]["id"], "qiongli-testkit")
        self.assertFalse(self.plan["consumer"]["requires_oracle_runtime"])
        self.assertTrue(self.plan["integrity"]["record_file_checksums"])
        self.assertTrue(self.plan["integrity"]["record_package_trees"])
        self.assertLessEqual(
            {"**/__pycache__/**", "**/node_modules/**", "**/target/**"},
            set(self.plan["inventory"]["exclude_selectors"]),
        )
        self.assertEqual(
            self.plan["integrity"]["package_tree_roots"],
            [
                "content/",
                "packages/python-qiongli/",
                "packages/qiongli-lite-mcp/",
                "packages/qiongli-literature-mcpb/",
                "packages/npm-qiongli/",
            ],
        )

    def test_inventory_explicitly_covers_npm_install_and_scenario_fixtures(self) -> None:
        domains = {
            item["id"]: set(item["source_selectors"])
            for item in self.plan["inventory"]["domains"]
        }
        self.assertIn("packages/npm-qiongli/bin/qiongli.mjs", domains["cli"])
        self.assertIn("packages/npm-qiongli/lib/installer.mjs", domains["installers"])
        self.assertIn("tests/test_install_qiongli.py", domains["installers"])
        self.assertIn("evals/controller_modes/**", domains["orchestrator-scenarios"])
        self.assertIn(
            "tests/fixtures/full_cycle_harness/**",
            domains["orchestrator-scenarios"],
        )

    def test_schema_rejects_an_incomplete_capture_plan(self) -> None:
        incomplete = copy.deepcopy(self.plan)
        incomplete["inventory"]["domains"].pop()
        incomplete["integrity"]["record_package_trees"] = False
        failures = validate_instance(incomplete, self.schema)
        self.assertTrue(any("at least 11 items" in failure for failure in failures))
        self.assertTrue(any("expected constant True" in failure for failure in failures))


if __name__ == "__main__":
    unittest.main()
