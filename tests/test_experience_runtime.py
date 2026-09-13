from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from bridges.experience_runtime import (
    build_experience_record,
    experience_lessons,
    experience_metrics,
    experience_schema_compatibility,
    experience_summary,
    generate_skill_reinforcement_candidate,
    promote_experience,
    query_experience,
    redact_experience_payload,
    replay_experience_plan,
    select_prior_experience,
    show_experience,
    write_experience_record,
)


class ExperienceRuntimeTests(unittest.TestCase):
    def test_redact_experience_payload_removes_common_credential_keys(self) -> None:
        credential_keys = (
            "token",
            "access_token",
            "accessToken",
            "access_key",
            "accessKey",
            "auth",
            "authorization",
            "bearer",
            "QIONGLI_OPENALEX_API_KEY",
            "service_private_key",
            "servicePrivateKey",
            "service_client_secret",
            "serviceClientSecret",
            "sessionToken",
            "refreshToken",
            "authToken",
            "idToken",
        )
        payload = {
            "nested": {
                key: f"CANARY_{index}"
                for index, key in enumerate(credential_keys)
            },
            "token_budget": 4096,
            "public_key": "kept",
        }

        redacted = redact_experience_payload(payload)

        self.assertEqual(redacted["nested"], {})
        self.assertEqual(redacted["token_budget"], 4096)
        self.assertEqual(redacted["public_key"], "kept")

    def _write_experience_fixture(
        self,
        root: Path,
        *,
        run_id: str,
        task_id: str,
        topic: str,
        validator_status: str,
        failure_modes: list[str] | None = None,
        reusable_guidance: list[str] | None = None,
        required_skills: list[str] | None = None,
        stage: str = "",
        inputs: dict[str, object] | None = None,
        outputs: dict[str, object] | None = None,
        quality: dict[str, object] | None = None,
        experience: dict[str, object] | None = None,
        execution: dict[str, object] | None = None,
    ) -> dict[str, object]:
        run_dir = root / ".qiongli" / "trace" / "runs" / run_id
        run_dir.mkdir(parents=True)
        record: dict[str, object] = {
            "schema_version": "1.0",
            "run_id": run_id,
            "created_at": "2026-07-06T12:00:00Z",
            "project_root": str(root),
            "task": {
                "task_id": task_id,
                "paper_type": "systematic-review",
                "topic": topic,
                "workflow": "",
                "stage": stage,
            },
            "execution": {
                "run_agents": False,
                "execution_mode": "solo",
                "worker_mode": "none",
            },
            "inputs": {
                "guidance_sources": [
                    {"kind": "project-local", "path": ".qiongli/local_guidance.md"}
                ],
                "required_skills": required_skills or [],
            },
            "outputs": {
                "required_outputs": ["search_diagnostics.md"],
                "found_outputs": [],
                "missing_outputs": ["search_diagnostics.md"]
                if validator_status == "failed"
                else [],
                "trace_files": [f".qiongli/trace/runs/{run_id}/validator_gate.json"],
            },
            "quality": {
                "validator_status": validator_status,
                "review_status": "unknown",
                "blocking_issues": [],
                "warnings": [],
                "confidence": 0.0,
            },
            "experience": {
                "lessons": [],
                "failure_modes": failure_modes or [],
                "reusable_guidance": reusable_guidance or [],
                "promotion_candidates": [],
            },
            "privacy": {
                "redaction_status": "not_needed",
                "contains_user_corpus": False,
                "contains_provider_metadata": False,
            },
        }
        for section_name, section_updates in (
            ("inputs", inputs),
            ("outputs", outputs),
            ("quality", quality),
            ("experience", experience),
            ("execution", execution),
        ):
            if section_updates is None:
                continue
            section = record.get(section_name)
            if isinstance(section, dict):
                section.update(section_updates)
            else:
                record[section_name] = dict(section_updates)
        (run_dir / "experience_record.json").write_text(
            json.dumps(record, ensure_ascii=False, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )
        index_path = root / ".qiongli" / "trace" / "experience.jsonl"
        index_path.parent.mkdir(parents=True, exist_ok=True)
        with index_path.open("a", encoding="utf-8") as handle:
            handle.write(json.dumps(record, ensure_ascii=False, sort_keys=True) + "\n")
        return record

    def test_build_experience_record_captures_task_quality_and_failure_modes(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            run_dir = root / ".qiongli" / "trace" / "runs" / "run-1"
            run_dir.mkdir(parents=True)
            guidance_trace = {
                "run_id": "run-1",
                "created_at": "2026-07-06T12:00:00Z",
                "run_dir": ".qiongli/trace/runs/run-1",
                "guidance_mode": "propose",
                "guidance_files_read": [".qiongli/local_guidance.md"],
                "guidance_sources": [
                    {"kind": "project-local", "path": ".qiongli/local_guidance.md"}
                ],
                "guidance_proposal": ".qiongli/trace/runs/run-1/guidance_update_proposal.md",
                "applied_guidance_update": False,
                "project_manifest": {"manifest": {"active_subject": "auto"}},
                "subject_refinement": {
                    "decision": "no_subject",
                    "summary": "Core guidance only.",
                },
            }
            task_packet = {
                "task_id": "B1",
                "paper_type": "systematic-review",
                "topic": "ai-writing",
                "required_outputs": ["search_diagnostics.md"],
                "worker_orchestration": {"status": "disabled", "mode": "none"},
                "controller_metadata": {
                    "execution_mode": "solo",
                    "controller": "codex",
                    "primary_agent": "codex",
                    "review_agent": "claude",
                    "verifier_agent": "",
                },
            }
            validator_gate = {
                "passed": False,
                "found": [],
                "missing": ["search_diagnostics.md"],
                "checked": 1,
            }

            record = build_experience_record(
                project_root=root,
                run_dir=run_dir,
                guidance_trace=guidance_trace,
                task_packet=task_packet,
                validator_gate=validator_gate,
            )

        self.assertEqual(record["schema_version"], "1.0")
        self.assertEqual(record["run_id"], "run-1")
        self.assertEqual(record["task"]["task_id"], "B1")
        self.assertEqual(record["execution"]["execution_mode"], "solo")
        self.assertEqual(record["quality"]["validator_status"], "failed")
        self.assertIn(
            "missing_required_output:search_diagnostics.md",
            record["experience"]["failure_modes"],
        )
        self.assertEqual(record["outputs"]["missing_outputs"], ["search_diagnostics.md"])
        self.assertEqual(record["privacy"]["redaction_status"], "not_needed")
        self.assertEqual(
            record["experience"]["guidance_update"]["proposal_path"],
            ".qiongli/trace/runs/run-1/guidance_update_proposal.md",
        )
        self.assertFalse(record["experience"]["guidance_update"]["applied"])
        self.assertEqual(record["experience"]["guidance_update"]["mode"], "propose")

    def test_build_experience_record_captures_worker_merge_and_review_state(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            run_dir = root / ".qiongli" / "trace" / "runs" / "run-workers"
            run_dir.mkdir(parents=True)
            record = build_experience_record(
                project_root=root,
                run_dir=run_dir,
                guidance_trace={
                    "run_id": "run-workers",
                    "created_at": "2026-07-06T12:00:00Z",
                },
                task_packet={
                    "task_id": "B1",
                    "paper_type": "systematic-review",
                    "topic": "worker-review",
                    "run_agents": True,
                    "worker_orchestration": {
                        "mode": "delegated_workers",
                        "status": "blocked",
                        "barrier_status": "ok",
                        "workers": [{"id": "w1"}, {"id": "w2"}],
                        "merge_status": "passed",
                        "merge_review_status": "blocked",
                        "merge_review_verdict": "BLOCK",
                        "blocking_issues": ["unsafe merge"],
                    },
                },
                validator_gate={
                    "passed": False,
                    "found": [],
                    "missing": [],
                    "checked": 1,
                },
            )

        self.assertTrue(record["execution"]["run_agents"])
        self.assertEqual(record["execution"]["worker_mode"], "delegated_workers")
        self.assertEqual(record["execution"]["worker_status"], "blocked")
        self.assertEqual(record["execution"]["worker_count"], 2)
        self.assertEqual(record["execution"]["worker_merge_status"], "passed")
        self.assertEqual(record["execution"]["worker_final_review_status"], "blocked")
        self.assertEqual(record["quality"]["review_status"], "blocked")
        self.assertIn("unsafe merge", record["quality"]["blocking_issues"])
        self.assertIn(
            "worker_final_review_blocked",
            record["experience"]["failure_modes"],
        )

    def test_write_experience_record_writes_run_file_and_jsonl_index(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            run_dir = root / ".qiongli" / "trace" / "runs" / "run-2"
            run_dir.mkdir(parents=True)
            trace = {
                "run_id": "run-2",
                "created_at": "2026-07-06T12:00:00Z",
                "run_dir": ".qiongli/trace/runs/run-2",
            }
            packet = {"task_id": "F3", "paper_type": "empirical", "topic": "demo"}
            gate = {
                "passed": True,
                "found": ["manuscript/manuscript.md"],
                "missing": [],
                "checked": 1,
            }

            result = write_experience_record(
                project_root=root,
                run_dir=run_dir,
                guidance_trace=trace,
                task_packet=packet,
                validator_gate=gate,
            )

            record_path = root / result["experience_record"]
            index_path = root / result["experience_index"]
            record = json.loads(record_path.read_text(encoding="utf-8"))
            rows = [
                json.loads(line)
                for line in index_path.read_text(encoding="utf-8").splitlines()
            ]

        self.assertEqual(record["run_id"], "run-2")
        self.assertEqual(rows[0]["run_id"], "run-2")
        self.assertEqual(result["experience_status"], "written")

    def test_write_experience_record_rejects_run_directory_symlink_escape(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            base = Path(tmp_dir)
            root = base / "project"
            runs_root = root / ".qiongli" / "trace" / "runs"
            outside = base / "outside-run"
            runs_root.mkdir(parents=True)
            outside.mkdir()
            escaped_run = runs_root / "escaped-run"
            escaped_run.symlink_to(outside, target_is_directory=True)

            with self.assertRaisesRegex(ValueError, "managed project paths"):
                write_experience_record(
                    project_root=root,
                    run_dir=escaped_run,
                    guidance_trace={"run_id": "escaped-run"},
                    task_packet={},
                    validator_gate={},
                )

            self.assertFalse((outside / "experience_record.json").exists())

    def test_write_experience_record_rejects_index_symlink_escape(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            base = Path(tmp_dir)
            root = base / "project"
            run_dir = root / ".qiongli" / "trace" / "runs" / "safe-run"
            run_dir.mkdir(parents=True)
            outside_index = base / "outside-experience.jsonl"
            outside_index.write_text("outside-canary\n", encoding="utf-8")
            index_path = root / ".qiongli" / "trace" / "experience.jsonl"
            index_path.symlink_to(outside_index)

            with self.assertRaisesRegex(ValueError, "managed project paths"):
                write_experience_record(
                    project_root=root,
                    run_dir=run_dir,
                    guidance_trace={"run_id": "safe-run"},
                    task_packet={},
                    validator_gate={},
                )

            self.assertEqual(outside_index.read_text(encoding="utf-8"), "outside-canary\n")
            self.assertFalse((run_dir / "experience_record.json").exists())

    def test_experience_summary_tolerates_malformed_jsonl_rows(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            trace_root = root / ".qiongli" / "trace"
            trace_root.mkdir(parents=True)
            (trace_root / "experience.jsonl").write_text(
                '{"run_id": "ok", "task": {"task_id": "B1"}}\nnot-json\n',
                encoding="utf-8",
            )

            summary = experience_summary(root)

        self.assertEqual(summary["run_count"], 1)
        self.assertEqual(summary["malformed_count"], 1)
        self.assertEqual(summary["runs"][0]["run_id"], "ok")

    def test_query_experience_filters_by_task_status_and_failure_mode(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            self._write_experience_fixture(
                root,
                run_id="failed-b1",
                task_id="B1",
                topic="ai-writing",
                validator_status="failed",
                failure_modes=["missing_required_output:search_diagnostics.md"],
                reusable_guidance=[
                    "Write search diagnostics before claiming review-grade coverage."
                ],
            )
            self._write_experience_fixture(
                root,
                run_id="passed-b1",
                task_id="B1",
                topic="ai-writing",
                validator_status="passed",
            )
            self._write_experience_fixture(
                root,
                run_id="failed-f3",
                task_id="F3",
                topic="ai-writing",
                validator_status="failed",
                failure_modes=["missing_required_output:manuscript/manuscript.md"],
            )

            result = query_experience(
                root,
                task_id="B1",
                validator_status="failed",
                failure_mode="missing_required_output:search_diagnostics.md",
            )

        self.assertEqual(result["run_count"], 1)
        self.assertEqual(result["records"][0]["run_id"], "failed-b1")
        self.assertEqual(result["filters"]["task_id"], "B1")

    def test_show_lessons_replay_and_prior_experience_use_local_records(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            self._write_experience_fixture(
                root,
                run_id="failed-b1",
                task_id="B1",
                topic="ai-writing",
                validator_status="failed",
                failure_modes=["missing_required_output:search_diagnostics.md"],
                reusable_guidance=[
                    "Write search diagnostics before claiming review-grade coverage."
                ],
            )

            shown = show_experience(root, "failed-b1")
            lessons = experience_lessons(root, task_id="B1")
            replay = replay_experience_plan(root, "failed-b1")
            prior = select_prior_experience(root, task_id="B1", topic="ai-writing", limit=1)

        self.assertEqual(shown["record"]["run_id"], "failed-b1")
        self.assertEqual(lessons["records"][0]["run_id"], "failed-b1")
        self.assertEqual(
            lessons["records"][0]["reusable_guidance"],
            ["Write search diagnostics before claiming review-grade coverage."],
        )
        self.assertEqual(replay["next_action"], "rerun_after_addressing_failures")
        self.assertEqual(replay["validator_status"], "failed")
        self.assertEqual(
            prior["records"][0]["failure_modes"],
            ["missing_required_output:search_diagnostics.md"],
        )
        self.assertEqual(prior["query"]["limit"], 1)

    def test_show_experience_rejects_record_directory_symlink_escape(self) -> None:
        canary = "QIONGLI_OUTSIDE_RECORD_CANARY_DO_NOT_READ"
        with tempfile.TemporaryDirectory() as tmp_dir:
            base = Path(tmp_dir)
            root = base / "project"
            runs_root = root / ".qiongli" / "trace" / "runs"
            outside_run = base / "outside-run"
            runs_root.mkdir(parents=True)
            outside_run.mkdir()
            (outside_run / "experience_record.json").write_text(
                json.dumps({"run_id": "linked-run", "api_key": canary}),
                encoding="utf-8",
            )
            (runs_root / "linked-run").symlink_to(outside_run, target_is_directory=True)

            with self.assertRaisesRegex(ValueError, "managed project paths") as caught:
                show_experience(root, "linked-run")

        self.assertNotIn(canary, str(caught.exception))

    def test_show_experience_rejects_record_file_symlink_escape(self) -> None:
        canary = "QIONGLI_OUTSIDE_FILE_CANARY_DO_NOT_READ"
        with tempfile.TemporaryDirectory() as tmp_dir:
            base = Path(tmp_dir)
            root = base / "project"
            run_dir = root / ".qiongli" / "trace" / "runs" / "linked-file"
            run_dir.mkdir(parents=True)
            outside_record = base / "outside-record.json"
            outside_record.write_text(
                json.dumps({"run_id": "linked-file", "api_key": canary}),
                encoding="utf-8",
            )
            (run_dir / "experience_record.json").symlink_to(outside_record)

            with self.assertRaisesRegex(ValueError, "managed project paths") as caught:
                show_experience(root, "linked-file")

        self.assertNotIn(canary, str(caught.exception))

    def test_experience_readers_reject_index_symlink_escape(self) -> None:
        canary = "QIONGLI_OUTSIDE_INDEX_CANARY_DO_NOT_READ"
        with tempfile.TemporaryDirectory() as tmp_dir:
            base = Path(tmp_dir)
            root = base / "project"
            trace_root = root / ".qiongli" / "trace"
            trace_root.mkdir(parents=True)
            outside_index = base / "outside-index.jsonl"
            outside_index.write_text(
                json.dumps({"run_id": "outside-run", "api_key": canary}) + "\n",
                encoding="utf-8",
            )
            (trace_root / "experience.jsonl").symlink_to(outside_index)

            readers = (
                lambda: query_experience(root),
                lambda: experience_lessons(root),
                lambda: show_experience(root, "missing-run"),
            )
            for reader in readers:
                with self.subTest(reader=reader):
                    with self.assertRaisesRegex(ValueError, "managed project paths") as caught:
                        reader()
                    self.assertNotIn(canary, str(caught.exception))

    def test_experience_readers_reject_in_project_managed_path_redirects(self) -> None:
        canary = "QIONGLI_IN_PROJECT_REDIRECT_CANARY_DO_NOT_READ"
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir) / "project"
            qiongli_root = root / ".qiongli"
            qiongli_root.mkdir(parents=True)
            redirected_index = root / "experience.jsonl"
            redirected_index.write_text(
                json.dumps({"run_id": "redirected", "api_key": canary}) + "\n",
                encoding="utf-8",
            )
            (qiongli_root / "trace").symlink_to(root, target_is_directory=True)

            with self.assertRaisesRegex(ValueError, "managed project paths") as caught:
                query_experience(root)

        self.assertNotIn(canary, str(caught.exception))

    def test_experience_readers_reject_in_trace_index_symlink(self) -> None:
        canary = "QIONGLI_IN_TRACE_INDEX_CANARY_DO_NOT_READ"
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir) / "project"
            trace_root = root / ".qiongli" / "trace"
            trace_root.mkdir(parents=True)
            redirected_index = trace_root / "redirected.jsonl"
            redirected_index.write_text(
                json.dumps({"run_id": "redirected", "api_key": canary}) + "\n",
                encoding="utf-8",
            )
            (trace_root / "experience.jsonl").symlink_to(redirected_index)

            with self.assertRaisesRegex(ValueError, "managed project paths") as caught:
                experience_lessons(root)

        self.assertNotIn(canary, str(caught.exception))

    def test_skill_reinforcement_candidate_requires_repeated_experience_support(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            (root / "content" / "skills").mkdir(parents=True)
            self._write_experience_fixture(
                root,
                run_id="failed-b1-a",
                task_id="B1",
                topic="ai-writing",
                validator_status="failed",
                failure_modes=["missing_required_output:search_diagnostics.md"],
                required_skills=["literature-search"],
            )
            self._write_experience_fixture(
                root,
                run_id="failed-b1-b",
                task_id="B1",
                topic="ai-writing",
                validator_status="failed",
                failure_modes=["missing_required_output:search_diagnostics.md"],
                required_skills=["literature-search"],
            )

            result = generate_skill_reinforcement_candidate(
                root,
                task_id="B1",
                min_support=2,
            )
            candidate_path = root / result["candidate_path"]
            text = candidate_path.read_text(encoding="utf-8")

        self.assertEqual(result["status"], "candidate_written")
        self.assertIn("literature-search", result["affected_skill_ids"])
        self.assertIn("failed-b1-a", text)
        self.assertIn("failed-b1-b", text)
        self.assertIn("Required Eval Or Regression Test", text)
        self.assertIn("Rollback Path", text)

    def test_promote_experience_writes_canonical_candidate_without_source_edits(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            skills_dir = root / "content" / "skills"
            skills_dir.mkdir(parents=True)
            skill_source = skills_dir / "literature-search.md"
            skill_source.write_text("# Literature Search\n", encoding="utf-8")
            self._write_experience_fixture(
                root,
                run_id="failed-b1-a",
                task_id="B1",
                topic="ai-writing",
                validator_status="failed",
                failure_modes=["missing_required_output:search_diagnostics.md"],
                required_skills=["literature-search"],
            )
            self._write_experience_fixture(
                root,
                run_id="failed-b1-b",
                task_id="B1",
                topic="ai-writing",
                validator_status="failed",
                failure_modes=["missing_required_output:search_diagnostics.md"],
                required_skills=["literature-search"],
            )

            result = promote_experience(
                root,
                scope="canonical-candidate",
                task_id="B1",
                min_support=2,
                test_plan="Add a regression test for search diagnostics output.",
            )
            skill_source_text = skill_source.read_text(encoding="utf-8")

        self.assertEqual(result["status"], "candidate_written")
        self.assertEqual(skill_source_text, "# Literature Search\n")

    def test_promote_experience_user_global_requires_explicit_approval(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            with self.assertRaisesRegex(ValueError, "requires explicit approval"):
                promote_experience(root, scope="user-global")

            result = promote_experience(root, scope="user-global", approved=True)

        self.assertEqual(result["status"], "manual_global_review_required")
        self.assertEqual(result["scope"], "user-global")

    def test_experience_metrics_summarize_validator_and_missing_artifact_rates(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            self._write_experience_fixture(
                root,
                run_id="failed-b1",
                task_id="B1",
                topic="ai-writing",
                validator_status="failed",
                failure_modes=["missing_required_output:search_diagnostics.md"],
            )
            self._write_experience_fixture(
                root,
                run_id="passed-b1",
                task_id="B1",
                topic="ai-writing",
                validator_status="passed",
            )

            metrics = experience_metrics(root)

        b1 = metrics["validator"]["by_task"]["B1"]
        self.assertEqual(b1["total_runs"], 2)
        self.assertEqual(b1["passed"], 1)
        self.assertEqual(b1["missing_artifact_runs"], 1)
        self.assertEqual(b1["pass_rate"], 0.5)
        self.assertEqual(
            metrics["failure_modes"]["missing_required_output:search_diagnostics.md"],
            1,
        )

    def test_experience_metrics_summarize_stage_11_learning_rates(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            self._write_experience_fixture(
                root,
                run_id="accepted-b1",
                task_id="B1",
                topic="ai-writing",
                validator_status="passed",
                inputs={
                    "subject_refinement": {
                        "evidence_sources": {
                            "user_action": {
                                "latest_action": {
                                    "action": "confirm",
                                    "subject": "finance",
                                }
                            }
                        }
                    }
                },
                outputs={
                    "required_outputs": ["search_diagnostics.md"],
                    "found_outputs": ["search_diagnostics.md"],
                    "missing_outputs": [],
                },
                experience={
                    "guidance_update": {
                        "proposal_path": ".qiongli/trace/runs/accepted-b1/guidance_update_proposal.md",
                        "applied": True,
                        "mode": "apply",
                    }
                },
            )
            self._write_experience_fixture(
                root,
                run_id="dismissed-b1",
                task_id="B1",
                topic="ai-writing",
                validator_status="failed",
                failure_modes=["missing_required_output:search_diagnostics.md"],
                inputs={
                    "subject_refinement": {
                        "evidence_sources": {
                            "user_action": {
                                "latest_action": {
                                    "action": "dismiss",
                                    "subject": "finance",
                                }
                            }
                        }
                    }
                },
                quality={
                    "review_status": "blocked",
                    "blocking_issues": ["search diagnostics missing"],
                },
                experience={
                    "guidance_update": {
                        "proposal_path": ".qiongli/trace/runs/dismissed-b1/guidance_update_proposal.md",
                        "applied": False,
                        "mode": "propose",
                    }
                },
            )
            self._write_experience_fixture(
                root,
                run_id="passed-f3",
                task_id="F3",
                topic="ai-writing",
                validator_status="passed",
                outputs={
                    "required_outputs": ["manuscript/manuscript.md"],
                    "found_outputs": ["manuscript/manuscript.md"],
                    "missing_outputs": [],
                },
            )

            metrics = experience_metrics(root)

        self.assertEqual(metrics["guidance"]["proposal_runs"], 2)
        self.assertEqual(metrics["guidance"]["accepted_runs"], 1)
        self.assertEqual(metrics["guidance"]["acceptance_rate"], 0.5)
        self.assertEqual(metrics["subject_routing"]["confirmation_count"], 1)
        self.assertEqual(metrics["subject_routing"]["dismissal_count"], 1)
        self.assertEqual(metrics["subject_routing"]["correction_count"], 1)
        self.assertEqual(metrics["subject_routing"]["correction_rate"], 0.5)
        self.assertEqual(metrics["review"]["blocker_count"], 1)
        self.assertEqual(metrics["literature_diagnostics"]["checked_runs"], 2)
        self.assertEqual(metrics["literature_diagnostics"]["failure_count"], 1)
        self.assertEqual(metrics["literature_diagnostics"]["failure_rate"], 0.5)

    def test_experience_schema_compatibility_accepts_current_records(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            self._write_experience_fixture(
                root,
                run_id="passed-b1",
                task_id="B1",
                topic="ai-writing",
                validator_status="passed",
            )

            report = experience_schema_compatibility(root)

        self.assertTrue(report["ok"])
        self.assertEqual(report["checked_records"], 2)
        self.assertEqual(report["malformed_count"], 0)
        self.assertEqual(report["errors"], [])

    def test_experience_schema_compatibility_reports_malformed_records(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            index_path = root / ".qiongli" / "trace" / "experience.jsonl"
            index_path.parent.mkdir(parents=True)
            index_path.write_text(
                json.dumps(
                    {
                        "schema_version": "1.0",
                        "run_id": "bad-record",
                        "execution": {},
                        "inputs": {},
                        "outputs": {},
                        "quality": {},
                        "experience": {},
                        "privacy": {},
                    }
                )
                + "\nnot-json\n",
                encoding="utf-8",
            )

            report = experience_schema_compatibility(root)

        self.assertFalse(report["ok"])
        self.assertEqual(report["checked_records"], 1)
        self.assertEqual(report["malformed_count"], 1)
        self.assertTrue(any("missing required object: task" in error for error in report["errors"]))


if __name__ == "__main__":
    unittest.main()
