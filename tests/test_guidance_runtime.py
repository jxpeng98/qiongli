from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from bridges.guidance_runtime import (
    apply_guidance_proposal,
    effective_guidance,
    guidance_bootstrap_status,
    guidance_trace_summary,
    init_project_guidance,
    write_guidance_trace,
)


class GuidanceRuntimeTests(unittest.TestCase):
    def test_init_project_guidance_creates_project_local_files(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)

            paths = init_project_guidance(root)

            self.assertEqual(paths.project_guidance, root.resolve() / ".qiongli" / "local_guidance.md")
            self.assertTrue(paths.project_guidance.is_file())
            self.assertTrue((root / ".qiongli" / "trace").is_dir())
            self.assertIn("# Qiongli Local Guidance", paths.project_guidance.read_text(encoding="utf-8"))

    def test_effective_guidance_project_overrides_global_preferences(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir) / "project"
            root.mkdir()
            home = Path(tmp_dir) / "home"
            home.mkdir()
            (home / "preferences.md").write_text(
                "# Qiongli User Preferences\n\n## Artifact Preferences\n- Prefer compact outputs.\n",
                encoding="utf-8",
            )
            init_project_guidance(root)
            (root / ".qiongli" / "local_guidance.md").write_text(
                "# Qiongli Local Guidance\n\n## Artifact Policy\n- Keep trace bundles in the project.\n",
                encoding="utf-8",
            )

            with mock.patch.dict("os.environ", {"QIONGLI_GUIDANCE_HOME": str(home)}):
                state = effective_guidance(root, mode="read")

            self.assertTrue(state.enabled)
            self.assertEqual(state.mode, "read")
            self.assertIn("Prefer compact outputs", state.guidance_context)
            self.assertIn("Keep trace bundles in the project", state.guidance_context)
            self.assertEqual(state.project_guidance_file, ".qiongli/local_guidance.md")
            self.assertEqual(state.trace_dir, "")

    def test_effective_guidance_reads_project_guidance_fragments_in_stable_order(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            init_project_guidance(root)
            guidance_dir = root / ".qiongli" / "guidance.d"
            guidance_dir.mkdir(parents=True, exist_ok=True)
            (guidance_dir / "writing-style.md").write_text(
                "# Writing Style\n\n- Prefer claim-first paragraphs.\n",
                encoding="utf-8",
            )
            (guidance_dir / "artifact-policy.md").write_text(
                "# Artifact Policy Extension\n\n- Keep scratch notes outside formal outputs.\n",
                encoding="utf-8",
            )

            state = effective_guidance(root, mode="read")

            self.assertTrue(state.enabled)
            self.assertEqual(
                state.guidance_files_read,
                [
                    ".qiongli/local_guidance.md",
                    ".qiongli/guidance.d/artifact-policy.md",
                    ".qiongli/guidance.d/writing-style.md",
                ],
            )
            self.assertIn("Keep scratch notes outside formal outputs", state.guidance_context)
            self.assertIn("Prefer claim-first paragraphs", state.guidance_context)
            self.assertEqual(state.source_order[-2:], ["project-fragment", "project-fragment"])
            self.assertEqual(state.guidance_sources[-1]["path"], ".qiongli/guidance.d/writing-style.md")

    def test_effective_guidance_off_mode_skips_guidance_reads(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            init_project_guidance(root)

            state = effective_guidance(root, mode="off")

            self.assertFalse(state.enabled)
            self.assertEqual(state.guidance_context, "")
            self.assertEqual(state.guidance_files_read, [])

    def test_guidance_bootstrap_status_reports_missing_files_without_writing(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)

            status = guidance_bootstrap_status(root, mode="propose")

            self.assertTrue(status["needed"])
            self.assertFalse((root / ".qiongli").exists())
            self.assertEqual(status["project_guidance"], ".qiongli/local_guidance.md")
            self.assertEqual(status["trace_root"], ".qiongli/trace")

    def test_write_guidance_trace_creates_linked_bundle_and_index(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            init_project_guidance(root)
            state = effective_guidance(root, mode="propose", run_id="run-123")

            trace = write_guidance_trace(
                project_root=root,
                guidance_state=state,
                task_packet={
                    "task_id": "F3",
                    "paper_type": "empirical",
                    "topic": "ai-writing",
                    "required_outputs": ["manuscript/manuscript.md"],
                },
                draft_content="draft body",
                review_content="review body",
                merged_analysis="merged body",
                validator_gate={
                    "passed": False,
                    "found": [],
                    "missing": ["manuscript/manuscript.md"],
                    "checked": 1,
                },
                applied=False,
            )

            run_dir = root / ".qiongli" / "trace" / "runs" / "run-123"
            self.assertEqual(trace["run_dir"], ".qiongli/trace/runs/run-123")
            for filename in (
                "task_packet.json",
                "guidance_context.md",
                "draft.md",
                "review.md",
                "merged_analysis.md",
                "validator_gate.json",
                "guidance_update_proposal.md",
            ):
                self.assertTrue((run_dir / filename).is_file(), filename)
            index_rows = [
                json.loads(line)
                for line in (root / ".qiongli" / "trace" / "index.jsonl")
                .read_text(encoding="utf-8")
                .splitlines()
            ]
            self.assertEqual(index_rows[0]["run_id"], "run-123")
            self.assertEqual(index_rows[0]["missing_outputs"], ["manuscript/manuscript.md"])

    def test_apply_guidance_proposal_appends_revision_history_only_to_project_file(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            init_project_guidance(root)
            proposal = root / ".qiongli" / "trace" / "runs" / "run-1" / "guidance_update_proposal.md"
            proposal.parent.mkdir(parents=True)
            proposal.write_text(
                "# Guidance Update Proposal\n\n## Proposed Changes\n\n- Prefer project-local trace bundles.\n",
                encoding="utf-8",
            )

            result = apply_guidance_proposal(root, proposal)

            text = (root / ".qiongli" / "local_guidance.md").read_text(encoding="utf-8")
            self.assertTrue(result["applied"])
            self.assertIn("Prefer project-local trace bundles", text)
            self.assertIn("run-1", text)

    def test_guidance_trace_summary_returns_recent_index_rows(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            init_project_guidance(root)
            state = effective_guidance(root, mode="propose", run_id="run-summary")
            write_guidance_trace(
                project_root=root,
                guidance_state=state,
                task_packet={"task_id": "A1", "paper_type": "theory", "topic": "trace-topic"},
                draft_content="draft",
                review_content="",
                merged_analysis="merged",
                validator_gate={"passed": True, "found": [], "missing": [], "checked": 0},
                applied=False,
            )

            summary = guidance_trace_summary(root)

            self.assertEqual(summary["project_dir"], str(root.resolve()))
            self.assertEqual(summary["run_count"], 1)
            self.assertEqual(summary["runs"][0]["run_id"], "run-summary")


if __name__ == "__main__":
    unittest.main()
