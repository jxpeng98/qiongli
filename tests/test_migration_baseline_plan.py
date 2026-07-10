from __future__ import annotations

import copy
import json
import unittest
from pathlib import Path

from tooling.scripts.validate_capability_contract import validate_instance


REPO_ROOT = Path(__file__).resolve().parents[1]
PLAN_PATH = REPO_ROOT / "tooling/migration/qiongli-1x-baseline-plan.json"
SCHEMA_PATH = REPO_ROOT / "tooling/migration/baseline-plan.schema.json"


class MigrationBaselinePlanTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.plan = json.loads(PLAN_PATH.read_text(encoding="utf-8"))
        cls.schema = json.loads(SCHEMA_PATH.read_text(encoding="utf-8"))

    def test_plan_matches_schema(self) -> None:
        self.assertEqual(validate_instance(self.plan, self.schema), [])

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
        registry = json.loads(
            (REPO_ROOT / contracts["capability_registry"]).read_text(encoding="utf-8")
        )
        fixture = json.loads(
            (REPO_ROOT / contracts["golden_fixture"]).read_text(encoding="utf-8")
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

    def test_schema_rejects_an_incomplete_capture_plan(self) -> None:
        incomplete = copy.deepcopy(self.plan)
        incomplete["inventory"]["domains"].pop()
        incomplete["integrity"]["record_package_trees"] = False
        failures = validate_instance(incomplete, self.schema)
        self.assertTrue(any("at least 11 items" in failure for failure in failures))
        self.assertTrue(any("expected constant True" in failure for failure in failures))


if __name__ == "__main__":
    unittest.main()
