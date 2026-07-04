from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from qiongli.bridges.lifecycle_harness import (
    build_lifecycle_report,
    evaluate_stage_gate,
)


class LifecycleHarnessTests(unittest.TestCase):
    def test_stage_gate_blocks_missing_required_artifacts(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            project = Path(tmp_dir)
            result = evaluate_stage_gate(project, "B")

        self.assertEqual(result["stage"], "B")
        self.assertEqual(result["status"], "blocked_missing_artifact")
        self.assertIn("search_strategy.md", result["missing_artifacts"])
        self.assertIn("search_log.md", result["missing_artifacts"])

    def test_stage_gate_does_not_use_metadata_fallback_for_required_artifacts(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            project = Path(tmp_dir)
            _write(project / "context" / "research_state.md", "RQ: Does X affect Y?\nContribution: demo")
            _write(project / "context" / "decision_log.md", "decision_id,stage,decision\nA1,A,RQ locked\n")
            _write(project / "context" / "boundary_review.md", "claim strength: associative")

            result = evaluate_stage_gate(project, "A")

        self.assertEqual(result["status"], "blocked_missing_artifact")
        self.assertIn("framing/research_question.md", result["missing_artifacts"])
        self.assertIn("framing/contribution_statement.md", result["missing_artifacts"])

    def test_stage_gate_requires_stage_handoff_artifact(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            project = Path(tmp_dir)
            _write_complete_empirical_fixture(project)
            (project / "context" / "stage_handoff.md").unlink()

            gate = evaluate_stage_gate(project, "A")
            report = build_lifecycle_report(project, topic="demo", paper_type="empirical")

        self.assertIn("context/stage_handoff.md", gate["missing_artifacts"])
        self.assertNotEqual(report["lifecycle_status"], "ready_for_h5")
        self.assertIn("A:missing_artifact", report["blocking_reasons"])
        self.assertIn("A1", report["recommended_next_tasks"])

    def test_stage_gate_treats_directory_artifact_as_missing(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            project = Path(tmp_dir)
            _write_complete_empirical_fixture(project)
            (project / "context" / "stage_handoff.md").unlink()
            (project / "context" / "stage_handoff.md").mkdir()

            result = evaluate_stage_gate(project, "A")

        self.assertIn("context/stage_handoff.md", result["missing_artifacts"])

    def test_stage_gate_treats_symlink_artifact_as_missing(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            project = Path(tmp_dir)
            _write_complete_empirical_fixture(project)
            handoff = project / "context" / "stage_handoff.md"
            target = project / "context" / "stage_handoff_target.md"
            handoff.unlink()
            _write(target, "Completed Artifacts\n")
            handoff.symlink_to(target)

            result = evaluate_stage_gate(project, "A")

        self.assertEqual(result["status"], "blocked_missing_artifact")
        self.assertIn("context/stage_handoff.md", result["missing_artifacts"])

    def test_clean_empirical_report_recommends_next_h5_when_submission_ready(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            project = Path(tmp_dir)
            _write_complete_empirical_fixture(project)

            report = build_lifecycle_report(project, topic="demo", paper_type="empirical")

        self.assertEqual(report["schema_version"], "1.0")
        self.assertEqual(report["mode"], "preview")
        self.assertEqual(report["topic"], "demo")
        self.assertEqual(report["paper_type"], "empirical")
        self.assertEqual(report["lifecycle_status"], "ready_for_h5")
        self.assertEqual(report["journal_fit"]["status"], "not_run")
        self.assertEqual([], report["blocking_reasons"])
        self.assertIn("H5", report["recommended_next_tasks"])

    def test_report_detects_research_question_drift(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            project = Path(tmp_dir)
            _write(project / "context" / "research_state.md", "RQ: Does X affect Y?")
            _write(project / "manuscript" / "manuscript.md", "# Manuscript\nThis paper studies A and B.")
            _write(
                project / "evidence" / "claim-evidence-ledger.csv",
                "claim_id,claim,evidence_status\nc1,A and B,supported\n",
            )

            report = build_lifecycle_report(project, topic="demo", paper_type="empirical")

        self.assertFalse(report["drift_checks"]["locked_question_preserved"])
        self.assertIn("research_question_drift", report["blocking_reasons"])
        self.assertIn("A1", report["recommended_next_tasks"])

    def test_report_detects_research_question_drift_when_only_generic_terms_overlap(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            project = Path(tmp_dir)
            _write(project / "context" / "research_state.md", "RQ: Does tutoring affect income?")
            _write(
                project / "manuscript" / "manuscript.md",
                "# Manuscript\nThis paper studies affect in unrelated research on sleep quality.",
            )
            _write(
                project / "evidence" / "claim-evidence-ledger.csv",
                "claim_id,claim,evidence_status\nc1,Sleep quality patterns,supported\n",
            )

            report = build_lifecycle_report(project, topic="demo", paper_type="empirical")

        self.assertFalse(report["drift_checks"]["locked_question_preserved"])
        self.assertIn("research_question_drift", report["blocking_reasons"])

    def test_report_blocks_missing_claim_evidence(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            project = Path(tmp_dir)
            _write_complete_empirical_fixture(
                project,
                claim_evidence_status="missing",
            )

            report = build_lifecycle_report(project, topic="demo", paper_type="empirical")

        self.assertEqual(report["drift_checks"]["claim_evidence_coverage"], "partial")
        self.assertIn("missing_claim_evidence", report["blocking_reasons"])
        self.assertIn("F4", report["recommended_next_tasks"])

    def test_report_accepts_canonical_supported_claim_status(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            project = Path(tmp_dir)
            _write_complete_empirical_fixture(project, canonical_claim_status="supported")

            report = build_lifecycle_report(project, topic="demo", paper_type="empirical")

        self.assertEqual(report["drift_checks"]["claim_evidence_coverage"], "complete")
        self.assertNotIn("missing_claim_evidence", report["blocking_reasons"])

    def test_report_blocks_canonical_needs_evidence_claim_status(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            project = Path(tmp_dir)
            _write_complete_empirical_fixture(project, canonical_claim_status="needs_evidence")

            report = build_lifecycle_report(project, topic="demo", paper_type="empirical")

        self.assertEqual(report["drift_checks"]["claim_evidence_coverage"], "partial")
        self.assertIn("missing_claim_evidence", report["blocking_reasons"])
        self.assertIn("F4", report["recommended_next_tasks"])

    def test_report_blocks_unresolved_judge_blocks(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            project = Path(tmp_dir)
            _write_complete_empirical_fixture(
                project,
                fatal_flaw_analysis="Decision: block_submission\nIssue: invalid identification",
            )

            report = build_lifecycle_report(project, topic="demo", paper_type="empirical")

        self.assertEqual(report["drift_checks"]["unresolved_judge_blocks"], 1)
        self.assertIn("unresolved_judge_blocks", report["blocking_reasons"])
        self.assertIn("H4", report["recommended_next_tasks"])

    def test_report_blocks_judge_block_even_when_peer_review_passes(self) -> None:
        for decision in ("Decision: block", "Decision: reopen_stage"):
            with self.subTest(decision=decision):
                with tempfile.TemporaryDirectory() as tmp_dir:
                    project = Path(tmp_dir)
                    _write_complete_empirical_fixture(
                        project,
                        fatal_flaw_analysis=f"{decision}\nIssue: invalid identification",
                    )
                    _write(project / "revision" / "peer_review_simulation.md", "Decision: pass")

                    report = build_lifecycle_report(project, topic="demo", paper_type="empirical")

                self.assertEqual(report["drift_checks"]["unresolved_judge_blocks"], 1)
                self.assertIn("unresolved_judge_blocks", report["blocking_reasons"])
                self.assertIn("H4", report["recommended_next_tasks"])

    def test_report_blocks_judge_revise_decision(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            project = Path(tmp_dir)
            _write_complete_empirical_fixture(
                project,
                fatal_flaw_analysis="Decision: revise\nIssue: unresolved validity concern",
            )

            report = build_lifecycle_report(project, topic="demo", paper_type="empirical")

        self.assertEqual(report["drift_checks"]["unresolved_judge_blocks"], 1)
        self.assertIn("unresolved_judge_blocks", report["blocking_reasons"])
        self.assertIn("H4", report["recommended_next_tasks"])


def _write_complete_empirical_fixture(
    project: Path,
    *,
    claim_evidence_status: str = "supported",
    canonical_claim_status: str | None = None,
    fatal_flaw_analysis: str = "Decision: pass\nNo fatal flaws.",
) -> None:
    _write(project / "context" / "research_state.md", "RQ: Does X affect Y?")
    _write(project / "context" / "decision_log.md", "decision_id,stage,decision\nA1,A,RQ locked\n")
    _write(project / "context" / "boundary_review.md", "claim strength: associative")
    _write(project / "context" / "stage_handoff.md", "Completed Artifacts\n")
    _write(project / "framing" / "research_question.md", "Does X affect Y?")
    _write(project / "framing" / "contribution_statement.md", "X helps explain Y.")
    _write(project / "search_strategy.md", "query: x y")
    _write(project / "search_log.md", "provider: fixture")
    _write(project / "search_results.csv", "title,doi\nA,10.1/a\n")
    _write(project / "dedup_log.csv", "record_id,decision\n1,keep\n")
    _write(project / "retrieval_manifest.csv", "record_id,retrieval_status\n1,abstract_only\n")
    _write(project / "study_design.md", "empirical design")
    _write(project / "analysis_plan.md", "model: y = x")
    _write(project / "manuscript" / "manuscript.md", "# Manuscript\nDoes X affect Y?")
    if canonical_claim_status is None:
        _write(
            project / "evidence" / "claim-evidence-ledger.csv",
            f"claim_id,claim,evidence_status\nc1,Does X affect Y?,{claim_evidence_status}\n",
        )
    else:
        _write(
            project / "evidence" / "claim-evidence-ledger.csv",
            "claim_id,claim_text,claim_type,evidence_type,source_id,source_location,"
            "artifact_path,confidence,limitations,status\n"
            f"c1,Does X affect Y?,finding,paper,Smith2024,p. 4,notes/smith.md,"
            f"high,Single study,{canonical_claim_status}\n",
        )
    _write(project / "reporting_checklist.md", "ready")
    _write(project / "proofread" / "proofread_checklist.md", "ready")
    _write(project / "revision" / "peer_review_simulation.md", "no major flaws")
    _write(project / "revision" / "fatal_flaw_analysis.md", fatal_flaw_analysis)


def _write(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


if __name__ == "__main__":
    unittest.main()
