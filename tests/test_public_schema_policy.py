from __future__ import annotations

import copy
import unittest

from tooling.scripts.validate_arc_201_adrs import (
    DEFAULT_CURRENT_RECORD,
    REPO_ROOT,
    load_record,
    validate_adr,
)
from tooling.scripts.validate_public_schema_policy import (
    DEFAULT_POLICY,
    load_policy,
    validate_policy,
)


def _change(**overrides: object) -> dict[str, object]:
    change: dict[str, object] = {
        "change_id": "app-ipc/snapshot-v19",
        "contract_id": "app-ipc/snapshot",
        "from_version": "18",
        "to_version": "19",
        "classification": "additive",
        "rust_sources": [
            "packages/qiongli-native/apps/qiongli/src/desktop_api.rs"
        ],
        "generated_schema": (
            "content/mcp-contracts/v2/schemas/"
            "qiongli_search_plan.input.schema.json"
        ),
        "golden_fixtures": [
            "packages/qiongli-native/apps/qiongli/"
            "examples/app_api_contract_fixture.rs"
        ],
        "consumer_checks": ["packages/qiongli-app-api/tests/client.test.ts"],
        "migration_path": None,
        "removal_gate": None,
    }
    change.update(overrides)
    return change


class PublicSchemaPolicyTests(unittest.TestCase):
    def setUp(self) -> None:
        self.policy = load_policy(DEFAULT_POLICY)

    def assert_policy_error(
        self, policy: dict[str, object], message: str
    ) -> None:
        self.assertTrue(
            any(message in error for error in validate_policy(REPO_ROOT, policy)),
            message,
        )

    def test_repository_policy_and_adr_are_valid(self) -> None:
        self.assertEqual(validate_policy(REPO_ROOT, self.policy), [])
        self.assertEqual(self.policy["schema_version"], "1.1")
        self.assertEqual(
            [contract["id"] for contract in self.policy["contracts"]],
            ["app-ipc", "mcp-tools", "public-cli-json"],
        )

        entry = load_record(DEFAULT_CURRENT_RECORD)["decisions"][-1]
        self.assertEqual(entry["adr_number"], "0216")
        self.assertEqual(validate_adr(REPO_ROOT / entry["path"], entry), [])

    def test_evaluation_truth_runs_policy_validation(self) -> None:
        workflow = (REPO_ROOT / ".github/workflows/evaluation-truth.yml").read_text(
            encoding="utf-8"
        )
        self.assertIn(
            "python tooling/scripts/validate_public_schema_policy.py", workflow
        )
        self.assertIn("tests.test_public_schema_policy", workflow)

    def test_boundary_inventory_is_closed_and_ordered(self) -> None:
        for mutation in ("missing", "duplicate", "reordered"):
            with self.subTest(mutation=mutation):
                policy = copy.deepcopy(self.policy)
                contracts = policy["contracts"]
                if mutation == "missing":
                    contracts.pop()
                elif mutation == "duplicate":
                    contracts[2] = copy.deepcopy(contracts[1])
                else:
                    contracts[0], contracts[1] = contracts[1], contracts[0]
                self.assert_policy_error(policy, "exactly once and in order")

    def test_unknown_policy_fields_and_values_are_rejected(self) -> None:
        mutations = (
            (lambda policy: policy.update(unexpected=True), "contain exactly"),
            (
                lambda policy: policy["authority"].update(language="typescript"),
                "exact Rust-owned",
            ),
            (
                lambda policy: policy["contracts"][0]["baseline"].update(
                    authority_state="generated"
                ),
                "authority_state is invalid",
            ),
        )
        for mutate, message in mutations:
            with self.subTest(message=message):
                policy = copy.deepcopy(self.policy)
                mutate(policy)
                self.assert_policy_error(policy, message)

    def test_compatibility_window_is_closed_and_exact(self) -> None:
        for key in self.policy["compatibility_window"]:
            with self.subTest(key=key):
                policy = copy.deepcopy(self.policy)
                policy["compatibility_window"][key] = "weaker"
                self.assert_policy_error(policy, f"compatibility_window.{key}")

        policy = copy.deepcopy(self.policy)
        policy["compatibility_window"].pop("public_id_removal")
        self.assert_policy_error(policy, "compatibility_window must contain exactly")

    def test_release_freezes_are_exact_and_source_bound(self) -> None:
        for index, contract in enumerate(self.policy["contracts"]):
            for key in contract["release_freeze"]:
                with self.subTest(contract=contract["id"], key=key):
                    policy = copy.deepcopy(self.policy)
                    policy["contracts"][index]["release_freeze"][key] = "wrong"
                    self.assert_policy_error(
                        policy, f"{contract['id']}.release_freeze.{key}"
                    )

        policy = copy.deepcopy(self.policy)
        policy["contracts"][0]["release_freeze"]["definition_version"] = "18"
        self.assert_policy_error(policy, "Rust App schema version does not match")

        policy = copy.deepcopy(self.policy)
        policy["contracts"][1]["release_freeze"].update(
            identity="wrong", definition_version="wrong"
        )
        self.assert_policy_error(policy, "MCP capability registry does not match")
        self.assert_policy_error(policy, "MCP capability registry schema does not match")

        policy = copy.deepcopy(self.policy)
        del policy["contracts"][0]["release_freeze"]["support_window"]
        self.assert_policy_error(
            policy, "app-ipc.release_freeze must contain exactly"
        )

    def test_non_repository_paths_are_rejected(self) -> None:
        for path in ("../outside.json", "/tmp/outside.json", "missing.json"):
            with self.subTest(path=path):
                policy = copy.deepcopy(self.policy)
                policy["contracts"][0]["rust_sources"] = [path]
                self.assert_policy_error(policy, "rust_sources[0]")

    def test_additive_change_is_accepted(self) -> None:
        policy = copy.deepcopy(self.policy)
        policy["contracts"][0]["changes"] = [_change()]
        self.assertEqual(validate_policy(REPO_ROOT, policy), [])

    def test_change_record_is_closed_and_has_generated_evidence(self) -> None:
        for key in (
            "from_version",
            "rust_sources",
            "generated_schema",
            "golden_fixtures",
            "consumer_checks",
        ):
            with self.subTest(key=key):
                policy = copy.deepcopy(self.policy)
                change = _change()
                del change[key]
                policy["contracts"][0]["changes"] = [change]
                self.assert_policy_error(policy, "must contain exactly")

        policy = copy.deepcopy(self.policy)
        policy["contracts"][0]["changes"] = [
            _change(generated_schema="tooling/architecture/public-schema-policy.json")
        ]
        self.assert_policy_error(policy, "Draft 2020-12")

    def test_unknown_class_and_missing_breaking_controls_are_rejected(self) -> None:
        cases = (
            ({"classification": "breaking"}, "classification is unknown"),
            (
                {"classification": "migratable-breaking"},
                "requires a migration_path",
            ),
            (
                {"classification": "unsupported-breaking"},
                "requires a separate removal_gate",
            ),
        )
        for overrides, message in cases:
            with self.subTest(message=message):
                policy = copy.deepcopy(self.policy)
                policy["contracts"][0]["changes"] = [_change(**overrides)]
                self.assert_policy_error(policy, message)

    def test_breaking_controls_are_repository_evidence(self) -> None:
        cases = (
            {
                "classification": "migratable-breaking",
                "migration_path": "packages/qiongli-app-api/src/schema.ts",
            },
            {
                "classification": "unsupported-breaking",
                "removal_gate": (
                    "docs/superpowers/acceptance/"
                    "2026-08-01-qiongli-alpha3-readiness.md"
                ),
            },
        )
        for overrides in cases:
            with self.subTest(classification=overrides["classification"]):
                policy = copy.deepcopy(self.policy)
                policy["contracts"][0]["changes"] = [_change(**overrides)]
                self.assertEqual(validate_policy(REPO_ROOT, policy), [])

    def test_change_identity_and_predecessor_chain_are_enforced(self) -> None:
        policy = copy.deepcopy(self.policy)
        policy["contracts"][0]["changes"] = [
            _change(),
            _change(from_version="17", to_version="20"),
        ]
        errors = validate_policy(REPO_ROOT, policy)
        self.assertTrue(any("is duplicated" in error for error in errors))
        self.assertTrue(any("must equal prior to_version" in error for error in errors))


if __name__ == "__main__":
    unittest.main()
