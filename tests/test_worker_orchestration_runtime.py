from __future__ import annotations

import argparse
import io
import inspect
import unittest
from contextlib import redirect_stderr
from pathlib import Path
from typing import Any

from bridges.base_bridge import BridgeResponse
from bridges.mcp_connectors import MCPEvidence
from bridges.orchestrator import (
    ModelOrchestrator,
    _add_worker_orchestration_task_run_args,
)
from qiongli.source_layout import RepoLayout


REPO_ROOT = Path(__file__).resolve().parents[1]
DISABLED_WORKER_NOTES = [
    "worker orchestration disabled; no worker execution attempted.",
]


class WorkerCaptureOrchestrator(ModelOrchestrator):
    def __init__(self) -> None:
        super().__init__(standards_dir=RepoLayout(REPO_ROOT).standards)
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


class WorkerOrchestrationRuntimeTests(unittest.TestCase):
    def test_task_run_worker_options_are_keyword_only(self) -> None:
        signature = inspect.signature(ModelOrchestrator.task_run)

        for parameter_name in ("worker_mode", "worker_adapter", "max_workers"):
            self.assertEqual(
                inspect.Parameter.KEYWORD_ONLY,
                signature.parameters[parameter_name].kind,
            )

    def test_parser_accepts_worker_orchestration_flags(self) -> None:
        parser = argparse.ArgumentParser()
        _add_worker_orchestration_task_run_args(parser)

        args = parser.parse_args(
            [
                "--worker-mode",
                "delegated-workers",
                "--worker-adapter",
                "generic-prompt",
                "--max-workers",
                "2",
            ]
        )

        self.assertEqual("delegated-workers", args.worker_mode)
        self.assertEqual("generic-prompt", args.worker_adapter)
        self.assertEqual(2, args.max_workers)

    def test_parser_rejects_non_positive_max_workers(self) -> None:
        parser = argparse.ArgumentParser()
        _add_worker_orchestration_task_run_args(parser)

        with redirect_stderr(io.StringIO()), self.assertRaises(SystemExit):
            parser.parse_args(["--max-workers", "0"])

    def test_task_run_defaults_to_no_worker_plan(self) -> None:
        orchestrator = WorkerCaptureOrchestrator()

        result = orchestrator.task_run(
            task_id="B1",
            paper_type="systematic-review",
            topic="worker-default",
            cwd=REPO_ROOT,
            skip_validation=True,
        )

        packet = result.data["task_packet"]
        self.assertEqual("none", packet["worker_orchestration"]["mode"])
        self.assertEqual("disabled", packet["worker_orchestration"]["status"])
        self.assertEqual("none", packet["worker_orchestration"]["adapter"])
        self.assertEqual([], packet["worker_orchestration"]["workers"])
        self.assertEqual(DISABLED_WORKER_NOTES, packet["worker_orchestration"]["notes"])
        self.assertEqual("none", packet["worker_orchestration"]["requested_mode"])
        self.assertEqual("auto", packet["worker_orchestration"]["requested_adapter"])
        self.assertIsNone(packet["worker_orchestration"]["max_workers"])
        self.assertNotIn("Worker Orchestration", result.merged_analysis)

    def test_task_run_records_requested_worker_options_while_disabled(self) -> None:
        orchestrator = WorkerCaptureOrchestrator()

        result = orchestrator.task_run(
            task_id="B1",
            paper_type="systematic-review",
            topic="worker-requested",
            cwd=REPO_ROOT,
            skip_validation=True,
            worker_mode="none",
            worker_adapter="generic-prompt",
            max_workers=2,
        )

        packet = result.data["task_packet"]
        self.assertEqual("none", packet["worker_orchestration"]["mode"])
        self.assertEqual("disabled", packet["worker_orchestration"]["status"])
        self.assertEqual("none", packet["worker_orchestration"]["adapter"])
        self.assertEqual([], packet["worker_orchestration"]["workers"])
        self.assertEqual(DISABLED_WORKER_NOTES, packet["worker_orchestration"]["notes"])
        self.assertEqual("none", packet["worker_orchestration"]["requested_mode"])
        self.assertEqual(
            "generic_prompt",
            packet["worker_orchestration"]["requested_adapter"],
        )
        self.assertEqual(2, packet["worker_orchestration"]["max_workers"])
        self.assertNotIn("Worker Orchestration", result.merged_analysis)

    def test_task_run_rejects_invalid_worker_options(self) -> None:
        orchestrator = WorkerCaptureOrchestrator()

        with self.assertRaisesRegex(ValueError, "worker_mode must be one of"):
            orchestrator.task_run(
                task_id="B1",
                paper_type="systematic-review",
                topic="worker-invalid-mode",
                cwd=REPO_ROOT,
                skip_validation=True,
                worker_mode="invalid",
            )

        with self.assertRaisesRegex(ValueError, "worker_adapter must be one of"):
            orchestrator.task_run(
                task_id="B1",
                paper_type="systematic-review",
                topic="worker-invalid-adapter",
                cwd=REPO_ROOT,
                skip_validation=True,
                worker_adapter="invalid",
            )

        with self.assertRaisesRegex(ValueError, "max_workers must be a positive int"):
            orchestrator.task_run(
                task_id="B1",
                paper_type="systematic-review",
                topic="worker-invalid-max",
                cwd=REPO_ROOT,
                skip_validation=True,
                max_workers=True,
            )

    def test_task_run_builds_and_executes_generic_worker_plan_when_enabled(self) -> None:
        orchestrator = WorkerCaptureOrchestrator()

        result = orchestrator.task_run(
            task_id="B1",
            paper_type="systematic-review",
            topic="worker-enabled",
            cwd=REPO_ROOT,
            skip_validation=True,
            execution_mode="solo",
            controller="codex",
            primary_agent="codex",
            worker_mode="delegated-workers",
            worker_adapter="generic-prompt",
            max_workers=2,
        )

        packet = result.data["task_packet"]
        worker_state = packet["worker_orchestration"]
        self.assertEqual("delegated_workers", worker_state["mode"])
        self.assertEqual("generic_prompt", worker_state["adapter"])
        self.assertEqual("ok", worker_state["barrier_status"])
        self.assertEqual(2, len(worker_state["workers"]))
        self.assertIn("## Worker Orchestration", result.merged_analysis)
        self.assertIn("Worker barrier status: ok", result.merged_analysis)

        worker_prompts = [
            call["prompt"]
            for call in orchestrator.runtime_calls
            if "Worker packet (JSON):" in call["prompt"]
        ]
        self.assertEqual(2, len(worker_prompts))
        self.assertIn("forbidden_artifacts", worker_prompts[0])

    def test_worker_barrier_degrades_when_allowed(self) -> None:
        class OneWorkerFailsOrchestrator(WorkerCaptureOrchestrator):
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
                if "screening_worker" in prompt:
                    return BridgeResponse.from_error(agent_name, "worker failed")
                return BridgeResponse(success=True, model=agent_name, content=f"{agent_name} ok")

        orchestrator = OneWorkerFailsOrchestrator()
        result = orchestrator.task_run(
            task_id="B1",
            paper_type="systematic-review",
            topic="worker-degraded",
            cwd=REPO_ROOT,
            skip_validation=True,
            execution_mode="solo",
            controller="codex",
            primary_agent="codex",
            worker_mode="delegated-workers",
            worker_adapter="generic-prompt",
            max_workers=3,
        )

        worker_state = result.data["task_packet"]["worker_orchestration"]
        self.assertEqual("degraded", worker_state["barrier_status"])
        self.assertIn("Worker screening_worker failed", "\n".join(worker_state["notes"]))
        self.assertIn("Worker barrier status: degraded", result.merged_analysis)

    def test_worker_barrier_block_short_circuits_before_main_draft(self) -> None:
        class TwoWorkersFailOrchestrator(WorkerCaptureOrchestrator):
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
                if "screening_worker" in prompt or "extraction_worker" in prompt:
                    return BridgeResponse.from_error(agent_name, "worker failed")
                return BridgeResponse(success=True, model=agent_name, content=f"{agent_name} ok")

        orchestrator = TwoWorkersFailOrchestrator()
        result = orchestrator.task_run(
            task_id="B1",
            paper_type="systematic-review",
            topic="worker-blocked",
            cwd=REPO_ROOT,
            skip_validation=True,
            execution_mode="solo",
            controller="codex",
            primary_agent="codex",
            worker_mode="delegated-workers",
            worker_adapter="generic-prompt",
            max_workers=3,
        )

        worker_state = result.data["task_packet"]["worker_orchestration"]
        self.assertEqual("blocked", worker_state["barrier_status"])
        self.assertEqual("blocked", worker_state["status"])
        self.assertEqual(0.0, result.confidence)
        self.assertIn("Worker barrier status: blocked", result.merged_analysis)
        self.assertTrue(orchestrator.runtime_calls)
        self.assertTrue(
            all("Worker packet (JSON):" in call["prompt"] for call in orchestrator.runtime_calls)
        )
        self.assertTrue(result.data["validator_gate"]["skipped"])
        self.assertEqual("worker barrier blocked", result.data["validator_gate"]["reason"])


if __name__ == "__main__":
    unittest.main()
