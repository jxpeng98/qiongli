from __future__ import annotations

import hashlib
import json
import unittest
from pathlib import Path

from qiongli.source_layout import RepoLayout

from bridges.context_package import build_context_package


REPO_ROOT = Path(__file__).resolve().parents[1]
TEMPLATE_PATH = RepoLayout(REPO_ROOT).templates / "context-manifest.json"


class ContextPackageBuilderTests(unittest.TestCase):
    def test_builds_manifest_with_normalized_participants_and_stable_hash(self) -> None:
        packet = {
            "task_id": "P3-T3.1",
            "paper_type": "systematic_review",
            "topic": "AI assisted evidence synthesis",
            "declared_write_set": ["bridges/context_package.py"],
            "verification_commands": ["python3 -m unittest tests.test_context_package_builder -v"],
            "artifact_paths": ["templates/context-manifest.json"],
        }

        package = build_context_package(
            packet,
            controller=" Codex ",
            agents=["Claude", " gemini ", "CODEX"],
        )

        manifest = package["context_manifest"]
        expected_manifest_without_hash = {
            "task_id": "P3-T3.1",
            "paper_type": "systematic_review",
            "topic": "AI assisted evidence synthesis",
            "controller": "codex",
            "agents": ["claude", "codex"],
        }
        expected_hash = hashlib.sha256(
            json.dumps(expected_manifest_without_hash, sort_keys=True).encode("utf-8")
        ).hexdigest()

        self.assertEqual(
            {
                **expected_manifest_without_hash,
                "input_context_hash": expected_hash,
            },
            manifest,
        )

    def test_builds_agent_specific_context_sections(self) -> None:
        packet = {
            "task_id": "P3-T3.1",
            "paper_type": "empirical",
            "topic": "Research tooling",
            "declared_write_set": ["bridges/context_package.py", "tests/test_context_package_builder.py"],
            "verification_commands": ["python3 -m unittest tests.test_context_package_builder -v"],
            "artifact_paths": ["templates/context-manifest.json"],
            "research_state": "Draft context package contract.",
            "evidence_ledger": "claims/evidence-ledger.md",
            "writing_review_standards": "Use auditable claims and explicit limitations.",
        }

        contexts = build_context_package(
            packet,
            controller="claude",
            agents=["codex", "claude", "gemini"],
        )["agent_contexts"]

        self.assertEqual({"codex", "claude"}, set(contexts))

        codex_context = contexts["codex"]
        for required in ("Declared Write Set", "Verification Commands", "Artifact Paths"):
            with self.subTest(agent="codex", required=required):
                self.assertIn(required, codex_context)
        self.assertIn("bridges/context_package.py", codex_context)

        claude_context = contexts["claude"]
        for required in ("Research State", "Evidence Ledger", "Writing/Review Standards"):
            with self.subTest(agent="claude", required=required):
                self.assertIn(required, claude_context)
        self.assertIn("Draft context package contract.", claude_context)

    def test_context_package_includes_boundary_review_for_all_agents(self) -> None:
        task_packet = {
            "task_id": "F3",
            "paper_type": "empirical",
            "topic": "ai-writing",
            "boundary_review": {
                "artifact": "context/boundary_review.md",
                "status": "answered",
                "existing_review": "- locked_decision: Claims are associative, not causal.",
            },
        }

        package = build_context_package(
            task_packet,
            controller="codex",
            agents=["codex", "claude", "gemini"],
        )

        self.assertIn("Claims are associative, not causal", package["agent_contexts"]["codex"])
        self.assertIn("Claims are associative, not causal", package["agent_contexts"]["claude"])
        self.assertNotIn("gemini", package["agent_contexts"])

    def test_context_package_includes_writing_harness_for_all_agents(self) -> None:
        task_packet = {
            "task_id": "F3",
            "paper_type": "empirical",
            "topic": "ai-writing",
            "writing_harness": {
                "enabled": True,
                "mode": "incremental-mainline",
                "required_preflight": ["boundary", "story_spine", "non_goals"],
                "loop": "write_review_confirm",
                "chunk_unit": "section_or_paragraph_cluster",
                "block_conditions": ["mainline_drift", "missing_support"],
            },
        }

        package = build_context_package(
            task_packet,
            controller="codex",
            agents=["codex", "claude", "gemini"],
        )

        self.assertNotIn("gemini", package["agent_contexts"])
        for agent in ("codex", "claude"):
            with self.subTest(agent=agent):
                context = package["agent_contexts"][agent]
                self.assertIn("Writing Harness", context)
                self.assertIn("incremental-mainline", context)
                self.assertIn("story_spine", context)
                self.assertIn("mainline_drift", context)

    def test_template_scaffold_declares_manifest_fields(self) -> None:
        self.assertTrue(TEMPLATE_PATH.exists(), f"Missing required artifact: {TEMPLATE_PATH}")

        template = json.loads(TEMPLATE_PATH.read_text(encoding="utf-8"))

        self.assertEqual(
            {"task_id", "paper_type", "topic", "controller", "agents", "input_context_hash"},
            set(template),
        )
        self.assertEqual([], template["agents"])


if __name__ == "__main__":
    unittest.main()
