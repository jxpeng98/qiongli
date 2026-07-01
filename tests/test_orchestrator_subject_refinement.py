from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from unittest import mock

from bridges.base_bridge import BridgeResponse, CollaborationResult
from bridges.orchestrator import ModelOrchestrator


class OrchestratorSubjectRefinementTests(unittest.TestCase):
    def _run_c1_task(
        self,
        *,
        topic: str,
        context: str,
        domain: str = "auto",
    ) -> tuple[CollaborationResult, mock.Mock]:
        orchestrator = ModelOrchestrator()

        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            with mock.patch.object(orchestrator, "task_plan") as task_plan:
                task_plan.return_value = CollaborationResult(
                    mode="task-plan",
                    task_description=f"C1 empirical {topic}",
                    confidence=0.8,
                    merged_analysis="plan",
                    recommendations=[],
                    data={
                        "functional_handoff_trace": [],
                        "functional_owner_chain": [],
                    },
                )
                with mock.patch.object(orchestrator, "_execute_runtime_agent") as execute:
                    execute.side_effect = [
                        BridgeResponse(success=True, model="codex", content="draft"),
                        BridgeResponse(
                            success=True,
                            model="claude",
                            content="PASS\n\nRecommendations:\n- ok",
                        ),
                    ]
                    result = orchestrator.task_run(
                        task_id="C1",
                        paper_type="empirical",
                        topic=topic,
                        context=context,
                        domain=domain,
                        cwd=root,
                        skip_validation=True,
                        max_revision_rounds=0,
                    )
                    return result, execute

        raise AssertionError("temporary task run exited unexpectedly")

    def test_builds_subject_refinement_for_real_task_run_packet(self) -> None:
        result, execute = self._run_c1_task(
            topic="earnings announcement stock market reaction",
            context=(
                "CRSP abnormal returns event study for Journal of Finance "
                "framing."
            ),
        )

        packet = result.data["task_packet"]
        self.assertEqual(packet["subject_refinement"]["decision"], "suggest_subject")
        self.assertEqual(packet["subject_refinement"]["primary_subject"], "finance")
        self.assertEqual(packet["domain"], "finance")
        self.assertEqual(packet["requested_domain"], "auto")
        draft_prompt = execute.call_args_list[0].args[1]
        review_prompt = execute.call_args_list[1].args[1]
        self.assertIn("Runtime subject refinement:", draft_prompt)
        self.assertIn("Decision: suggest_subject", draft_prompt)
        self.assertIn("Runtime subject refinement:", review_prompt)
        self.assertIn("Decision: suggest_subject", review_prompt)

    def test_explicit_domain_override_is_preserved_when_subject_is_suggested(self) -> None:
        result, _execute = self._run_c1_task(
            topic="earnings announcement stock market reaction",
            context=(
                "CRSP abnormal returns event study for Journal of Finance "
                "framing."
            ),
            domain="economics",
        )

        packet = result.data["task_packet"]
        self.assertEqual(packet["subject_refinement"]["decision"], "suggest_subject")
        self.assertEqual(packet["subject_refinement"]["primary_subject"], "finance")
        self.assertEqual(packet["domain"], "economics")

    def test_borrowed_lens_does_not_switch_auto_domain(self) -> None:
        result, _execute = self._run_c1_task(
            topic="announcement timing study",
            context=(
                "Use an event-study timing lens around policy announcements "
                "with qualitative adoption outcomes."
            ),
        )

        packet = result.data["task_packet"]
        self.assertEqual(packet["subject_refinement"]["decision"], "borrow_lens")
        self.assertEqual(packet["subject_refinement"]["primary_subject"], "auto")
        self.assertEqual(packet["domain"], "auto")


if __name__ == "__main__":
    unittest.main()
