from __future__ import annotations

import copy
import tempfile
import unittest
from pathlib import Path

from tooling.scripts.validate_authorization_policy import (
    AuthorizationPolicyError,
    DEFAULT_POLICY,
    DEFAULT_SCHEMA,
    EXPECTED_ACTIONS,
    EXPECTED_PLANES,
    EXPECTED_ROLES,
    REPO_ROOT,
    load_document,
    validate_policy,
    validate_receipt,
)


class AuthorizationPolicyTests(unittest.TestCase):
    def setUp(self) -> None:
        self.policy = load_document(DEFAULT_POLICY)
        self.schema = load_document(DEFAULT_SCHEMA)
        self.receipt = copy.deepcopy(self.schema["examples"][0])

    def assert_policy_error(
        self,
        policy: dict[str, object],
        message: str,
        schema: dict[str, object] | None = None,
    ) -> None:
        errors = validate_policy(
            REPO_ROOT, policy, schema if schema is not None else self.schema
        )
        self.assertTrue(any(message in error for error in errors), errors)

    def assert_receipt_error(
        self, receipt: dict[str, object], message: str
    ) -> None:
        errors = validate_receipt(self.policy, receipt)
        self.assertTrue(any(message in error for error in errors), errors)

    def test_repository_policy_schema_and_example_are_valid(self) -> None:
        self.assertEqual(validate_policy(REPO_ROOT, self.policy, self.schema), [])
        self.assertEqual(validate_receipt(self.policy, self.receipt), [])
        self.assertEqual(
            tuple(plane["id"] for plane in self.policy["planes"]),
            EXPECTED_PLANES,
        )
        self.assertEqual(
            tuple(role["id"] for role in self.policy["roles"]), EXPECTED_ROLES
        )
        self.assertEqual(
            tuple(action["id"] for action in self.policy["actions"]),
            EXPECTED_ACTIONS,
        )

    def test_evaluation_truth_runs_authorization_checks(self) -> None:
        workflow = (REPO_ROOT / ".github/workflows/evaluation-truth.yml").read_text(
            encoding="utf-8"
        )
        self.assertIn(
            "python tooling/scripts/validate_authorization_policy.py", workflow
        )
        self.assertIn("tests.test_authorization_policy", workflow)

    def test_closed_inventories_reject_missing_duplicate_and_reordered_ids(self) -> None:
        for inventory in ("planes", "roles", "actions"):
            for mutation in ("missing", "duplicate", "reordered"):
                with self.subTest(inventory=inventory, mutation=mutation):
                    policy = copy.deepcopy(self.policy)
                    values = policy[inventory]
                    if mutation == "missing":
                        values.pop()
                    elif mutation == "duplicate":
                        values[-1] = copy.deepcopy(values[0])
                    else:
                        values[0], values[1] = values[1], values[0]
                    self.assert_policy_error(policy, "closed inventory")

    def test_role_and_action_references_fail_closed(self) -> None:
        mutations = (
            (
                lambda policy: policy["actions"][0]["executor_roles"].append(
                    "unknown-role"
                ),
                "unknown role",
            ),
            (
                lambda policy: policy["actions"][0]["authorizer_roles"].append(
                    "agent-ci-principal"
                ),
                "Agent/CI principal cannot be an authorizer",
            ),
            (
                lambda policy: policy["actions"][0].update(plane="repository"),
                "action namespace",
            ),
            (
                lambda policy: policy["actions"][0].update(
                    default_rule="allow-by-default"
                ),
                "retain its v1 rule",
            ),
            (
                lambda policy: policy["actions"][0].update(unexpected=True),
                "contain exactly",
            ),
        )
        for mutate, message in mutations:
            with self.subTest(message=message):
                policy = copy.deepcopy(self.policy)
                mutate(policy)
                self.assert_policy_error(policy, message)

    def test_non_transitive_rules_are_exact_and_negative(self) -> None:
        for mutation in (
            "missing",
            "duplicate",
            "reordered",
            "unknown",
            "positive",
        ):
            with self.subTest(mutation=mutation):
                policy = copy.deepcopy(self.policy)
                rules = policy["non_transitive_rules"]
                if mutation == "missing":
                    rules.pop()
                elif mutation == "duplicate":
                    rules.append(copy.deepcopy(rules[-1]))
                elif mutation == "reordered":
                    rules[0], rules[1] = rules[1], rules[0]
                elif mutation == "unknown":
                    rules[0]["source"] = "ci-red"
                else:
                    target = rules[0].pop("does_not_authorize")
                    rules[0]["authorizes"] = target
                self.assert_policy_error(policy, "closed negative transitions")

    def test_schema_is_closed_bounded_and_requires_digest_and_expiry(self) -> None:
        mutations = (
            lambda schema: schema.update({"$schema": "draft-07"}),
            lambda schema: schema.update(additionalProperties=True),
            lambda schema: schema["required"].remove("expires_at"),
            lambda schema: schema["properties"]["authorization_id"].update(
                pattern=".*"
            ),
            lambda schema: schema["properties"]["action"]["enum"].append(
                "repository.force-push"
            ),
            lambda schema: schema["properties"]["authorizer_role"]["enum"].append(
                "agent-ci-principal"
            ),
            lambda schema: schema["allOf"][0]["anyOf"][1]["properties"][
                "artifact_digests_sha256"
            ].update(minItems=0),
        )
        for index, mutate in enumerate(mutations):
            with self.subTest(index=index):
                schema = copy.deepcopy(self.schema)
                mutate(schema)
                self.assertNotEqual(
                    validate_policy(REPO_ROOT, self.policy, schema), []
                )

    def test_receipt_rejects_missing_digest_expiry_and_unknown_or_unsafe_data(self) -> None:
        cases = (
            (
                lambda receipt: receipt.update(
                    plan_digest_sha256=None,
                    artifact_digests_sha256=[],
                ),
                "requires a plan or artifact digest",
            ),
            (
                lambda receipt: receipt.pop("expires_at"),
                "contain exactly",
            ),
            (
                lambda receipt: receipt.update(
                    expires_at=receipt["issued_at"]
                ),
                "later than issued_at",
            ),
            (
                lambda receipt: receipt.update(
                    authorizer_role="agent-ci-principal"
                ),
                "unknown or Agent/CI",
            ),
            (
                lambda receipt: receipt.update(action="repository.force-push"),
                "action is unknown",
            ),
            (
                lambda receipt: receipt.update(unexpected=True),
                "contain exactly",
            ),
            (
                lambda receipt: receipt.update(evidence_refs=["/tmp/secret"]),
                "machine-absolute paths",
            ),
        )
        for mutate, message in cases:
            with self.subTest(message=message):
                receipt = copy.deepcopy(self.receipt)
                mutate(receipt)
                self.assert_receipt_error(receipt, message)

    def test_policy_evidence_paths_must_be_canonical_repository_files(self) -> None:
        for path in ("../outside.md", "/tmp/secret", "missing.md"):
            with self.subTest(path=path):
                policy = copy.deepcopy(self.policy)
                policy["evidence"] = [path]
                self.assert_policy_error(policy, "evidence[0]")

    def test_duplicate_json_keys_are_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "duplicate.json"
            path.write_text(
                '{"schema_version":"1.0","schema_version":"2.0"}',
                encoding="utf-8",
            )
            with self.assertRaisesRegex(
                AuthorizationPolicyError, "duplicate JSON key"
            ):
                load_document(path)


if __name__ == "__main__":
    unittest.main()
