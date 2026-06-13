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
        if prompt.startswith("Final-review the merged worker output."):
            return BridgeResponse(
                success=True,
                model=agent_name,
                content="Verdict: PASS\nConfidence: 0.9",
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
        self.assertEqual("delegated_workers", worker_state["orchestration_mode"])
        self.assertEqual("delegated_workers", worker_state["mode"])
        self.assertEqual(worker_state["orchestration_mode"], worker_state["mode"])
        self.assertEqual("codex", worker_state["controller_runtime"])
        self.assertEqual("generic_prompt", worker_state["platform_adapter"])
        self.assertEqual("generic_prompt", worker_state["adapter"])
        self.assertEqual(worker_state["platform_adapter"], worker_state["adapter"])
        self.assertEqual("B1", worker_state["task_id"])
        self.assertEqual("systematic-review", worker_state["paper_type"])
        self.assertEqual("worker-enabled", worker_state["topic"])
        self.assertIsInstance(worker_state["merge"], dict)
        self.assertIn("agent", worker_state["merge"])
        self.assertIn("policy", worker_state["merge"])
        self.assertIn("output_artifacts", worker_state["merge"])
        self.assertIsInstance(worker_state["final_review"], dict)
        self.assertIn("reviewer", worker_state["final_review"])
        self.assertIn("gate", worker_state["final_review"])
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

    def test_task_run_worker_plan_paths_are_run_scoped(self) -> None:
        orchestrator = WorkerCaptureOrchestrator()

        result = orchestrator.task_run(
            task_id="B1",
            paper_type="systematic-review",
            topic="worker-run-scoped",
            cwd=REPO_ROOT,
            skip_validation=True,
            execution_mode="solo",
            controller="codex",
            primary_agent="codex",
            worker_mode="delegated-workers",
            worker_adapter="generic-prompt",
            max_workers=2,
        )

        worker_state = result.data["task_packet"]["worker_orchestration"]
        run_id = worker_state["run_id"]
        self.assertTrue(run_id)
        for worker in worker_state["workers"]:
            expected_root = f"RESEARCH/worker-run-scoped/runs/{run_id}/workers/{worker['id']}/"
            self.assertEqual(expected_root, worker["worker_root"])
            self.assertEqual([expected_root], worker["allowed_artifacts"])
            self.assertNotIn("//", worker["worker_root"])
        self.assertEqual(
            [f"RESEARCH/worker-run-scoped/runs/{run_id}/worker-merge-report.md"],
            worker_state["merge"]["output_artifacts"],
        )

    def test_repeated_worker_runs_get_distinct_run_scoped_paths(self) -> None:
        orchestrator = WorkerCaptureOrchestrator()
        results = [
            orchestrator.task_run(
                task_id="B1",
                paper_type="systematic-review",
                topic="worker-retry",
                cwd=REPO_ROOT,
                skip_validation=True,
                execution_mode="solo",
                controller="codex",
                primary_agent="codex",
                worker_mode="delegated-workers",
                worker_adapter="generic-prompt",
                max_workers=2,
            )
            for _ in range(2)
        ]

        first_state = results[0].data["task_packet"]["worker_orchestration"]
        second_state = results[1].data["task_packet"]["worker_orchestration"]
        first_run_id = first_state["run_id"]
        second_run_id = second_state["run_id"]

        self.assertTrue(first_run_id)
        self.assertTrue(second_run_id)
        self.assertNotEqual(first_run_id, second_run_id)

        first_worker_roots = [worker["worker_root"] for worker in first_state["workers"]]
        second_worker_roots = [worker["worker_root"] for worker in second_state["workers"]]
        first_allowed_artifacts = [
            artifact
            for worker in first_state["workers"]
            for artifact in worker["allowed_artifacts"]
        ]
        second_allowed_artifacts = [
            artifact
            for worker in second_state["workers"]
            for artifact in worker["allowed_artifacts"]
        ]
        first_merge_artifacts = first_state["merge"]["output_artifacts"]
        second_merge_artifacts = second_state["merge"]["output_artifacts"]

        self.assertNotEqual(first_worker_roots, second_worker_roots)
        self.assertNotEqual(first_allowed_artifacts, second_allowed_artifacts)
        self.assertNotEqual(first_merge_artifacts, second_merge_artifacts)
        for path in first_worker_roots + first_allowed_artifacts + first_merge_artifacts:
            self.assertIn(f"/runs/{first_run_id}/", path)
        for path in second_worker_roots + second_allowed_artifacts + second_merge_artifacts:
            self.assertIn(f"/runs/{second_run_id}/", path)

    def test_worker_execution_runs_merge_and_final_review(self) -> None:
        orchestrator = WorkerCaptureOrchestrator()

        result = orchestrator.task_run(
            task_id="B1",
            paper_type="systematic-review",
            topic="worker-merge",
            cwd=REPO_ROOT,
            skip_validation=True,
            execution_mode="duo",
            controller="codex",
            primary_agent="codex",
            review_agent="claude",
            worker_mode="delegated-workers",
            worker_adapter="generic-prompt",
            max_workers=2,
        )

        merge_calls = [
            call
            for call in orchestrator.runtime_calls
            if call["prompt"].startswith("Merge worker results for this Qiongli task.")
        ]
        final_review_calls = [
            call
            for call in orchestrator.runtime_calls
            if call["prompt"].startswith("Final-review the merged worker output.")
        ]
        prompts = "\n\n".join(call["prompt"] for call in orchestrator.runtime_calls)
        self.assertIn("Merge worker results for this Qiongli task.", prompts)
        self.assertIn("Final-review the merged worker output.", prompts)
        self.assertEqual(1, len(merge_calls))
        self.assertEqual("codex", merge_calls[0]["agent"])
        self.assertEqual(1, len(final_review_calls))
        self.assertEqual("claude", final_review_calls[0]["agent"])
        worker_state = result.data["task_packet"]["worker_orchestration"]
        self.assertIn("merge_review_status", worker_state)
        self.assertEqual("passed", worker_state["merge_review_status"])

    def test_worker_merge_runtime_failure_short_circuits_before_main_draft(self) -> None:
        class FailingMergeOrchestrator(WorkerCaptureOrchestrator):
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
                if "Worker packet (JSON):" in prompt:
                    return BridgeResponse(
                        success=True,
                        model=agent_name,
                        content="worker ok",
                    )
                if prompt.startswith("Merge worker results for this Qiongli task."):
                    return BridgeResponse.from_error(agent_name, "merge runtime failed")
                return BridgeResponse(success=True, model=agent_name, content=f"{agent_name} ok")

        orchestrator = FailingMergeOrchestrator()
        result = orchestrator.task_run(
            task_id="B1",
            paper_type="systematic-review",
            topic="worker-merge-failed",
            cwd=REPO_ROOT,
            skip_validation=True,
            execution_mode="duo",
            controller="codex",
            primary_agent="codex",
            review_agent="claude",
            worker_mode="delegated-workers",
            worker_adapter="generic-prompt",
            max_workers=2,
        )

        worker_state = result.data["task_packet"]["worker_orchestration"]
        self.assertEqual("failed", worker_state["merge_status"])
        self.assertEqual("merge_failed", worker_state["merge_review_status"])
        self.assertEqual("blocked", worker_state["status"])
        self.assertEqual(0.0, result.confidence)
        self.assertIn("Worker merge status: failed", result.merged_analysis)
        self.assertIn("worker merge failed", result.merged_analysis)
        self.assertTrue(result.data["validator_gate"]["skipped"])
        self.assertEqual("worker merge failed", result.data["validator_gate"]["reason"])
        self.assertEqual(
            "worker merge failed",
            result.data["review_loop_state"]["stopped_reason"],
        )
        self.assertEqual(3, len(orchestrator.runtime_calls))
        for call in orchestrator.runtime_calls:
            prompt = call["prompt"]
            is_worker_prompt = "Worker packet (JSON):" in prompt
            is_worker_merge = prompt.startswith("Merge worker results for this Qiongli task.")
            if "Task packet (JSON):" in prompt:
                self.assertTrue(is_worker_prompt or is_worker_merge)

    def test_worker_final_review_runtime_failure_short_circuits_before_main_draft(self) -> None:
        class FailingFinalReviewOrchestrator(WorkerCaptureOrchestrator):
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
                if "Worker packet (JSON):" in prompt:
                    return BridgeResponse(
                        success=True,
                        model=agent_name,
                        content="worker ok",
                    )
                if prompt.startswith("Merge worker results for this Qiongli task."):
                    return BridgeResponse(
                        success=True,
                        model=agent_name,
                        content="merged worker output",
                    )
                if prompt.startswith("Final-review the merged worker output."):
                    return BridgeResponse.from_error(agent_name, "final review runtime failed")
                return BridgeResponse(success=True, model=agent_name, content=f"{agent_name} ok")

        orchestrator = FailingFinalReviewOrchestrator()
        result = orchestrator.task_run(
            task_id="B1",
            paper_type="systematic-review",
            topic="worker-final-review-failed",
            cwd=REPO_ROOT,
            skip_validation=True,
            execution_mode="duo",
            controller="codex",
            primary_agent="codex",
            review_agent="claude",
            worker_mode="delegated-workers",
            worker_adapter="generic-prompt",
            max_workers=2,
        )

        worker_state = result.data["task_packet"]["worker_orchestration"]
        self.assertEqual("passed", worker_state["merge_status"])
        self.assertEqual("failed", worker_state["merge_review_status"])
        self.assertEqual("blocked", worker_state["status"])
        self.assertEqual(0.0, result.confidence)
        self.assertIn("Worker final review status: failed", result.merged_analysis)
        self.assertIn("worker final review failed", result.merged_analysis)
        self.assertTrue(result.data["validator_gate"]["skipped"])
        self.assertEqual("worker final review failed", result.data["validator_gate"]["reason"])
        self.assertEqual(
            "worker final review failed",
            result.data["review_loop_state"]["stopped_reason"],
        )
        self.assertEqual(4, len(orchestrator.runtime_calls))
        for call in orchestrator.runtime_calls:
            prompt = call["prompt"]
            is_worker_prompt = "Worker packet (JSON):" in prompt
            is_worker_merge = prompt.startswith("Merge worker results for this Qiongli task.")
            is_worker_review = prompt.startswith("Final-review the merged worker output.")
            if "Task packet (JSON):" in prompt:
                self.assertTrue(is_worker_prompt or is_worker_merge or is_worker_review)

    def test_worker_final_review_block_short_circuits_before_main_draft(self) -> None:
        class BlockingFinalReviewOrchestrator(WorkerCaptureOrchestrator):
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
                if "Worker packet (JSON):" in prompt:
                    return BridgeResponse(
                        success=True,
                        model=agent_name,
                        content="worker ok",
                    )
                if prompt.startswith("Merge worker results for this Qiongli task."):
                    return BridgeResponse(
                        success=True,
                        model=agent_name,
                        content="merged worker output",
                    )
                if prompt.startswith("Final-review the merged worker output."):
                    return BridgeResponse(
                        success=True,
                        model=agent_name,
                        content=(
                            "Verdict: BLOCK\n"
                            "Confidence: 0.2\n"
                            "Blocking Issues\n"
                            "- unsafe merge"
                        ),
                    )
                return BridgeResponse(success=True, model=agent_name, content=f"{agent_name} ok")

        orchestrator = BlockingFinalReviewOrchestrator()
        result = orchestrator.task_run(
            task_id="B1",
            paper_type="systematic-review",
            topic="worker-final-review-block",
            cwd=REPO_ROOT,
            skip_validation=True,
            execution_mode="duo",
            controller="codex",
            primary_agent="codex",
            review_agent="claude",
            worker_mode="delegated-workers",
            worker_adapter="generic-prompt",
            max_workers=2,
        )

        worker_state = result.data["task_packet"]["worker_orchestration"]
        self.assertEqual("BLOCK", worker_state["merge_review_verdict"])
        self.assertEqual("blocked", worker_state["merge_review_status"])
        self.assertEqual(0.2, worker_state["merge_review_confidence"])
        self.assertEqual("blocked", worker_state["status"])
        self.assertEqual(0.0, result.confidence)
        self.assertIn("Worker final review status: blocked", result.merged_analysis)
        self.assertIn("worker final review blocked", result.merged_analysis)
        self.assertTrue(result.data["validator_gate"]["skipped"])
        self.assertEqual("worker final review blocked", result.data["validator_gate"]["reason"])
        self.assertEqual(
            "worker final review blocked",
            result.data["review_loop_state"]["stopped_reason"],
        )
        self.assertEqual(4, len(orchestrator.runtime_calls))
        for call in orchestrator.runtime_calls:
            prompt = call["prompt"]
            is_worker_prompt = "Worker packet (JSON):" in prompt
            is_worker_merge = prompt.startswith("Merge worker results for this Qiongli task.")
            is_worker_review = prompt.startswith("Final-review the merged worker output.")
            if "Task packet (JSON):" in prompt:
                self.assertTrue(is_worker_prompt or is_worker_merge or is_worker_review)

    def test_worker_merge_and_final_review_prompts_are_bounded(self) -> None:
        large_worker_content = "worker-output-" + ("x" * 20000)
        large_merge_content = "merge-output-" + ("y" * 20000)

        class LargeOutputOrchestrator(WorkerCaptureOrchestrator):
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
                if "Worker packet (JSON):" in prompt:
                    return BridgeResponse(
                        success=True,
                        model=agent_name,
                        content=large_worker_content,
                    )
                if prompt.startswith("Merge worker results for this Qiongli task."):
                    return BridgeResponse(
                        success=True,
                        model=agent_name,
                        content=large_merge_content,
                    )
                if prompt.startswith("Final-review the merged worker output."):
                    return BridgeResponse(
                        success=True,
                        model=agent_name,
                        content="Verdict: PASS\nConfidence: 0.9",
                    )
                return BridgeResponse(success=True, model=agent_name, content=f"{agent_name} ok")

        orchestrator = LargeOutputOrchestrator()
        result = orchestrator.task_run(
            task_id="B1",
            paper_type="systematic-review",
            topic="worker-large-prompts",
            cwd=REPO_ROOT,
            skip_validation=True,
            execution_mode="duo",
            controller="codex",
            primary_agent="codex",
            review_agent="claude",
            worker_mode="delegated-workers",
            worker_adapter="generic-prompt",
            max_workers=2,
        )

        merge_prompt = next(
            call["prompt"]
            for call in orchestrator.runtime_calls
            if call["prompt"].startswith("Merge worker results for this Qiongli task.")
        )
        final_review_prompt = next(
            call["prompt"]
            for call in orchestrator.runtime_calls
            if call["prompt"].startswith("Final-review the merged worker output.")
        )

        self.assertNotIn(large_worker_content, merge_prompt)
        self.assertIn("worker-output-" + ("x" * 100), merge_prompt)
        self.assertIn("truncated", merge_prompt.lower())
        self.assertIn(str(len(large_worker_content)), merge_prompt)
        self.assertNotIn(large_merge_content, final_review_prompt)
        self.assertIn("merge-output-" + ("y" * 100), final_review_prompt)
        self.assertIn("truncated", final_review_prompt.lower())
        for prompt in (merge_prompt, final_review_prompt):
            self.assertNotIn('"merge_content"', prompt)
            self.assertNotIn('"merge_review_content"', prompt)
            self.assertNotIn('"worker_orchestration"', prompt)
        self.assertNotIn(large_worker_content, result.merged_analysis)
        self.assertNotIn(large_merge_content, result.merged_analysis)
        self.assertIn("[truncated:", result.merged_analysis)

        worker_state = result.data["task_packet"]["worker_orchestration"]
        first_worker_result = worker_state["worker_results"][0]
        self.assertTrue(first_worker_result["content_truncated"])
        self.assertEqual(
            len(large_worker_content),
            first_worker_result["content_original_length"],
        )
        self.assertTrue(worker_state["merge_content_truncated"])
        self.assertEqual(
            len(large_merge_content),
            worker_state["merge_content_original_length"],
        )

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
                if prompt.startswith("Final-review the merged worker output."):
                    return BridgeResponse(
                        success=True,
                        model=agent_name,
                        content="Verdict: PASS\nConfidence: 0.9",
                    )
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
