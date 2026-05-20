from __future__ import annotations

import argparse
import unittest
from pathlib import Path
from typing import Any

from bridges.base_bridge import BridgeResponse
from bridges.mcp_connectors import MCPEvidence
from bridges.orchestrator import (
    ModelOrchestrator,
    _add_controller_agnostic_task_run_args,
)


REPO_ROOT = Path(__file__).resolve().parents[1]


class MetadataCaptureOrchestrator(ModelOrchestrator):
    def __init__(self) -> None:
        super().__init__(standards_dir=REPO_ROOT / "standards")
        self.runtime_calls: list[dict[str, Any]] = []

    def _runtime_preflight_error(
        self,
        agent_name: str,
        cwd: Path,
        runtime_options: dict[str, Any] | None = None,
    ) -> str | None:
        return None

    def _execute_runtime_agent(
        self,
        agent_name: str,
        prompt: str,
        cwd: Path,
        runtime_options: dict[str, Any] | None = None,
        profile_directive: str | None = None,
    ) -> BridgeResponse:
        self.runtime_calls.append(
            {
                "agent": agent_name,
                "prompt": prompt,
                "runtime_options": dict(runtime_options or {}),
                "profile_directive": profile_directive or "",
            }
        )
        return BridgeResponse(success=True, model=agent_name, content=f"{agent_name} ok")

    def _collect_mcp_evidence(
        self,
        task_packet: dict[str, Any],
        cwd: Path,
        strict: bool = False,
    ) -> tuple[list[MCPEvidence], list[str]]:
        return [MCPEvidence(provider="filesystem", status="ok", summary="mock")], []


class ControllerAgnosticOrchestrationTests(unittest.TestCase):
    def test_parser_accepts_controller_agnostic_task_run_metadata(self) -> None:
        parser = argparse.ArgumentParser()
        _add_controller_agnostic_task_run_args(parser)

        args = parser.parse_args(
            [
                "--execution-mode",
                "solo",
                "--controller",
                "claude",
                "--primary",
                "claude",
                "--reviewer",
                "codex",
                "--verifier",
                "codex",
                "--solo-role-gates",
                "strict",
            ]
        )

        self.assertEqual("solo", args.execution_mode)
        self.assertEqual("claude", args.controller)
        self.assertEqual("claude", args.primary_agent)
        self.assertEqual("codex", args.review_agent)
        self.assertEqual("codex", args.verifier_agent)
        self.assertEqual("strict", args.solo_role_gates)

    def test_task_run_records_controller_metadata_without_changing_runtime_plan(self) -> None:
        orchestrator = MetadataCaptureOrchestrator()

        result = orchestrator.task_run(
            task_id="F3",
            paper_type="empirical",
            topic="controller-metadata",
            cwd=REPO_ROOT,
            execution_mode="solo",
            controller="Claude",
            primary_agent="claude",
            review_agent="codex",
            verifier_agent="codex",
            solo_role_gates="strict",
            skip_validation=True,
        )

        packet = result.data["task_packet"]
        self.assertEqual("solo", packet["execution_mode"])
        self.assertEqual("claude", packet["controller"])
        self.assertEqual("claude", packet["primary_agent"])
        self.assertEqual("codex", packet["review_agent"])
        self.assertEqual("codex", packet["verifier_agent"])
        self.assertEqual("strict", packet["solo_role_gates"])
        self.assertEqual(
            {
                "execution_mode": "solo",
                "controller": "claude",
                "primary_agent": "claude",
                "review_agent": "codex",
                "verifier_agent": "codex",
                "solo_role_gates": "strict",
            },
            packet["controller_metadata"],
        )

        self.assertEqual("writing-agent", packet["functional_owner"])
        self.assertIn("runtime_plan", packet)
        self.assertGreaterEqual(len(orchestrator.runtime_calls), 2)


if __name__ == "__main__":
    unittest.main()
