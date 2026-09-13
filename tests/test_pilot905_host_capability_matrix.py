from __future__ import annotations

import hashlib
import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
MATRIX_PATH = (
    ROOT
    / "docs"
    / "superpowers"
    / "acceptance"
    / "2026-08-30-qiongli-pilot905-host-capability-matrix.json"
)
HOST_IDS = (
    "codex-cli",
    "claude-code",
    "codex-desktop",
    "claude-desktop",
    "antigravity",
    "generic-local-mcp",
)
CAPABILITY_IDS = (
    "plugin_lifecycle",
    "skill_discovery",
    "lite_mcp",
    "full_mcp",
    "authenticated_model_execution",
    "project_read",
    "graph_read",
    "structured_output",
    "native_subagents",
    "conversation_non_retention",
    "cleanup",
)
STATUS_ORDER = ("observed-present", "observed-absent", "not-observed")
STATUSES = set(STATUS_ORDER)


class Pilot905HostCapabilityMatrixTests(unittest.TestCase):
    def test_matrix_is_closed_receipt_bound_and_published_bilingually(self) -> None:
        matrix = json.loads(MATRIX_PATH.read_text(encoding="utf-8"))
        self.assertEqual(matrix["documentKind"], "qiongli-observed-model-host-capability-matrix")
        self.assertFalse(matrix["publicationAllowed"])
        self.assertEqual(matrix["statusVocabulary"], list(STATUS_ORDER))
        self.assertEqual(matrix["capabilityOrder"], list(CAPABILITY_IDS))

        evidence = {record["id"]: record for record in matrix["evidence"]}
        self.assertEqual(
            tuple(evidence),
            (
                "codex-claude-mcp-compatibility-2026-08-24",
                "pilot-903-codex-real-project-2026-08-30",
            ),
        )
        for record in evidence.values():
            relative = Path(record["path"])
            self.assertFalse(relative.is_absolute())
            self.assertNotIn("..", relative.parts)
            payload = (ROOT / relative).read_bytes()
            self.assertEqual(hashlib.sha256(payload).hexdigest(), record["sha256"])
            self.assertRegex(record["productSource"], r"^[0-9a-f]{40}$")
            rendered_evidence = payload.decode("utf-8")
            self.assertIn(record["productSource"], rendered_evidence)
            for observed_host in record["hosts"]:
                self.assertIn(observed_host["version"], rendered_evidence)
            self.assertFalse(record["publicationAllowed"])

        hosts = {host["id"]: host for host in matrix["hosts"]}
        self.assertEqual(tuple(hosts), HOST_IDS)
        for host_id, host in hosts.items():
            self.assertEqual(host["modelIdentity"], "not-recorded")
            self.assertEqual(tuple(host["capabilities"]), CAPABILITY_IDS)
            evidence_versions = {
                observed_host["version"]
                for record in evidence.values()
                for observed_host in record["hosts"]
                if observed_host["id"] == host_id
            }
            self.assertEqual(set(host["observedVersions"]), evidence_versions)
            for cell in host["capabilities"].values():
                self.assertIn(cell["status"], STATUSES)
                evidence_ids = cell["evidenceIds"]
                if cell["status"] == "not-observed":
                    self.assertEqual(evidence_ids, [])
                else:
                    self.assertTrue(evidence_ids)
                    for evidence_id in evidence_ids:
                        self.assertIn(evidence_id, evidence)
                        observed_hosts = {
                            item["id"] for item in evidence[evidence_id]["hosts"]
                        }
                        self.assertIn(host_id, observed_hosts)

        self.assertEqual(
            hosts["codex-cli"]["capabilities"]["native_subagents"]["status"],
            "observed-absent",
        )
        self.assertEqual(
            hosts["claude-code"]["capabilities"]["authenticated_model_execution"]["status"],
            "not-observed",
        )
        for host_id in HOST_IDS[2:]:
            self.assertTrue(
                all(
                    cell["status"] == "not-observed"
                    for cell in hosts[host_id]["capabilities"].values()
                )
            )

        payload = dict(matrix)
        expected_digest = payload.pop("receiptPayloadSha256")
        canonical = json.dumps(
            payload,
            ensure_ascii=False,
            separators=(",", ":"),
            sort_keys=True,
        ).encode("utf-8")
        self.assertEqual(hashlib.sha256(canonical).hexdigest(), expected_digest)

        forbidden_keys = {"prompt", "response", "conversation", "credentials", "username"}
        pending: list[object] = [matrix]
        while pending:
            value = pending.pop()
            if isinstance(value, dict):
                self.assertTrue(forbidden_keys.isdisjoint(value))
                pending.extend(value.values())
            elif isinstance(value, list):
                pending.extend(value)
            elif isinstance(value, str):
                self.assertFalse(value.startswith(("/", "~")))
                self.assertNotIn("\n", value)

        receipt_name = MATRIX_PATH.name
        for page in (
            ROOT / "docs" / "guide" / "agent-host-capability-matrix.md",
            ROOT / "docs" / "zh" / "guide" / "agent-host-capability-matrix.md",
        ):
            text = page.read_text(encoding="utf-8")
            self.assertIn(receipt_name, text)
            for host in matrix["hosts"]:
                self.assertIn(host["displayName"], text)
        self.assertIn(
            "/guide/agent-host-capability-matrix",
            (ROOT / "docs" / "index.md").read_text(encoding="utf-8"),
        )
        self.assertIn(
            "/zh/guide/agent-host-capability-matrix",
            (ROOT / "docs" / "zh" / "index.md").read_text(encoding="utf-8"),
        )


if __name__ == "__main__":
    unittest.main()
