from __future__ import annotations

import importlib
import json
import tempfile
import unittest
from pathlib import Path
from unittest import mock

import bridges.mcp_tool_handlers as tool_handlers
from bridges import project_manifest, subject_lifecycle
from bridges.mcp_tool_handlers import MCP_TOOL_DEFINITIONS, call_qiongli_tool
from bridges.provider_config import set_provider_value


class _PreviewStubResult:
    mode = "task-plan"
    confidence = 0.8
    merged_analysis = "preview"
    recommendations: list[str] = []

    def __init__(self) -> None:
        self.data = {
            "task_id": "F3",
            "paper_type": "empirical",
            "topic": "my-topic",
            "artifact_root": "RESEARCH/[topic]/",
            "runtime_plan": {
                "primary_agent": "codex",
                "review_agent": "claude",
                "fallback_agent": "claude",
            },
        }


class _PreviewStubOrchestrator:
    def __init__(self) -> None:
        self.loaded_domain = ""

    def task_plan(self, **_kwargs: object) -> _PreviewStubResult:
        return _PreviewStubResult()

    def _build_controller_metadata(self, **_kwargs: object) -> dict[str, str]:
        return {
            "execution_mode": "duo",
            "controller": "codex",
            "primary_agent": "",
            "review_agent": "",
            "verifier_agent": "",
            "solo_role_gates": "standard",
        }

    def _controller_runtime_overrides(self, _metadata: dict[str, str]) -> dict[str, str]:
        return {}

    def _load_domain_profile_context(self, domain: str) -> dict[str, str]:
        self.loaded_domain = domain
        return {
            "requested_domain": domain,
            "domain": domain,
            "status": "loaded" if domain != "auto" else "auto",
            "display_name": domain.title() if domain != "auto" else "Auto-detect",
        }

    def _build_domain_packet_fields(self, domain_context: dict[str, str]) -> dict[str, str]:
        return {
            "domain": domain_context["domain"],
            "requested_domain": domain_context["requested_domain"],
            "domain_profile_status": domain_context["status"],
            "domain_profile_display_name": domain_context["display_name"],
        }


class MCPToolHandlerTests(unittest.TestCase):
    def _call_task_run_preview(
        self,
        args: dict[str, object],
        *,
        stub: _PreviewStubOrchestrator | None = None,
    ) -> tuple[dict[str, object], _PreviewStubOrchestrator]:
        preview_stub = stub or _PreviewStubOrchestrator()
        with mock.patch.object(tool_handlers, "ModelOrchestrator", return_value=preview_stub):
            result = call_qiongli_tool("qiongli_task_run", args)
        self.assertFalse(result["isError"])
        return result, preview_stub

    def _write_journal_fit_fixture(self, root: Path, *, venue_dir: str = "venues") -> None:
        (root / "manuscript").mkdir()
        (root / "manuscript" / "manuscript.md").write_text(
            "A design science contribution with validated analytics methods.",
            encoding="utf-8",
        )
        (root / "framing").mkdir()
        (root / "framing" / "contribution_statement.md").write_text(
            "This paper offers a design science contribution.",
            encoding="utf-8",
        )
        (root / "study_design.md").write_text(
            "The study uses validated analytics methods.",
            encoding="utf-8",
        )
        (root / "evidence").mkdir()
        (root / "evidence" / "claim-evidence-ledger.csv").write_text(
            "claim_id,status\nc1,supported\n",
            encoding="utf-8",
        )
        venues = root / venue_dir
        venues.mkdir()
        (venues / "primary.yaml").write_text(
            "\n".join(
                [
                    "venue_id: primary",
                    "community:",
                    "  - design science",
                    "contribution_expectations:",
                    "  - design science contribution",
                    "methods_expectations:",
                    "  - validated analytics methods",
                    "evidence_standards:",
                    "  - supported",
                ]
            ),
            encoding="utf-8",
        )
        (venues / "backup.yaml").write_text(
            "\n".join(
                [
                    "venue_id: backup",
                    "community:",
                    "  - unrelated community",
                ]
            ),
            encoding="utf-8",
        )

    def _write_experience_fixture(self, root: Path) -> None:
        run_id = "failed-b1"
        run_dir = root / ".qiongli" / "trace" / "runs" / run_id
        run_dir.mkdir(parents=True)
        record = {
            "schema_version": "1.0",
            "run_id": run_id,
            "created_at": "2026-07-06T12:00:00Z",
            "project_root": str(root),
            "task": {
                "task_id": "B1",
                "paper_type": "systematic-review",
                "topic": "ai-writing",
                "workflow": "",
                "stage": "",
            },
            "execution": {"run_agents": False, "execution_mode": "solo", "worker_mode": "none"},
            "inputs": {"guidance_sources": []},
            "outputs": {
                "required_outputs": ["search_diagnostics.md"],
                "found_outputs": [],
                "missing_outputs": ["search_diagnostics.md"],
                "trace_files": [".qiongli/trace/runs/failed-b1/validator_gate.json"],
            },
            "quality": {
                "validator_status": "failed",
                "review_status": "unknown",
                "blocking_issues": [],
                "warnings": [],
                "confidence": 0.0,
            },
            "experience": {
                "lessons": [],
                "failure_modes": ["missing_required_output:search_diagnostics.md"],
                "reusable_guidance": [
                    "Write search diagnostics before claiming review-grade coverage."
                ],
                "promotion_candidates": [],
            },
            "privacy": {
                "redaction_status": "not_needed",
                "contains_user_corpus": False,
                "contains_provider_metadata": False,
            },
        }
        (run_dir / "experience_record.json").write_text(
            json.dumps(record, ensure_ascii=False, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )
        index_path = root / ".qiongli" / "trace" / "experience.jsonl"
        index_path.parent.mkdir(parents=True, exist_ok=True)
        index_path.write_text(
            json.dumps(record, ensure_ascii=False, sort_keys=True) + "\n",
            encoding="utf-8",
        )

    def test_tool_definitions_include_config_and_evidence_tools(self) -> None:
        ordered_names = [tool["name"] for tool in MCP_TOOL_DEFINITIONS]
        names = set(ordered_names)

        self.assertTrue(
            {
                "qiongli_literature_status",
                "qiongli_search_plan",
                "qiongli_literature_search",
                "qiongli_literature_export_evidence",
                "qiongli_config_status",
                "qiongli_save_provider_config",
                "qiongli_collect_evidence",
                "qiongli_list_provider_env",
                "qiongli_test_provider",
                "qiongli_configure_provider",
                "qiongli_open_config_wizard",
                "qiongli_orchestrator_route",
                "qiongli_orchestrator_doctor",
                "qiongli_task_plan",
                "qiongli_task_run",
                "qiongli_subject_status",
                "qiongli_subject_update",
                "qiongli_experience_query",
                "qiongli_experience_show",
                "qiongli_experience_lessons",
            }.issubset(names)
        )
        status_index = ordered_names.index("qiongli_literature_status")
        self.assertEqual(ordered_names[status_index + 1], "qiongli_search_plan")
        search_plan_schema = next(
            tool["inputSchema"]["properties"]
            for tool in MCP_TOOL_DEFINITIONS
            if tool["name"] == "qiongli_search_plan"
        )
        for alias in (
            "nativeSearchAvailable",
            "nativeSearchTools",
            "includeWorkingPapers",
            "searchMode",
            "venueFilter",
            "documentTypes",
            "queryVariants",
        ):
            self.assertIn(alias, search_plan_schema)

    def test_experience_mcp_tools_query_show_and_lessons(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            self._write_experience_fixture(root)

            query = call_qiongli_tool(
                "qiongli_experience_query",
                {
                    "cwd": str(root),
                    "task_id": "B1",
                    "validator_status": "failed",
                    "failure_mode": "missing_required_output:search_diagnostics.md",
                },
            )
            shown = call_qiongli_tool(
                "qiongli_experience_show",
                {"cwd": str(root), "run_id": "failed-b1"},
            )
            lessons = call_qiongli_tool(
                "qiongli_experience_lessons",
                {"cwd": str(root), "task_id": "B1"},
            )

        self.assertFalse(query["isError"], query)
        self.assertEqual(query["structuredContent"]["run_count"], 1)
        self.assertEqual(query["structuredContent"]["records"][0]["run_id"], "failed-b1")
        self.assertFalse(shown["isError"], shown)
        self.assertEqual(shown["structuredContent"]["record"]["run_id"], "failed-b1")
        self.assertFalse(lessons["isError"], lessons)
        self.assertEqual(
            lessons["structuredContent"]["records"][0]["reusable_guidance"],
            ["Write search diagnostics before claiming review-grade coverage."],
        )

    def test_tool_definitions_include_subject_lifecycle_tools(self) -> None:
        definitions = {tool["name"]: tool for tool in MCP_TOOL_DEFINITIONS}
        expected_actions = [
            action
            for action in tool_handlers.SUBJECT_LIFECYCLE_ACTION_ORDER
            if action in subject_lifecycle.ACTIONS
        ]
        expected_subjects = [
            subject for subject in project_manifest.OFFICIAL_SUBJECTS if subject not in {"auto", "core"}
        ]

        self.assertIn("qiongli_subject_status", definitions)
        self.assertIn("qiongli_subject_update", definitions)
        update_schema = definitions["qiongli_subject_update"]["inputSchema"]
        self.assertEqual(update_schema["required"], ["action"])
        self.assertEqual(
            update_schema["properties"]["action"]["enum"],
            expected_actions,
        )
        self.assertEqual(
            update_schema["properties"]["subject"]["enum"],
            expected_subjects,
        )
        self.assertIn("read_only", update_schema["properties"])

    def test_tool_definitions_include_full_cycle_preview_tools(self) -> None:
        definitions = {tool["name"]: tool for tool in MCP_TOOL_DEFINITIONS}

        lifecycle_schema = definitions["qiongli_lifecycle_plan"]["inputSchema"]
        lifecycle_properties = lifecycle_schema["properties"]
        self.assertEqual(lifecycle_schema["type"], "object")
        self.assertEqual(lifecycle_properties["cwd"]["type"], "string")
        self.assertEqual(lifecycle_properties["topic"]["type"], "string")
        self.assertEqual(lifecycle_properties["paper_type"]["type"], "string")
        self.assertEqual(lifecycle_properties["mode"]["type"], "string")
        self.assertEqual(lifecycle_properties["mode"]["enum"], ["preview"])

        journal_schema = definitions["qiongli_journal_fit_recommend"]["inputSchema"]
        journal_properties = journal_schema["properties"]
        self.assertEqual(journal_schema["type"], "object")
        self.assertEqual(journal_properties["cwd"]["type"], "string")
        self.assertEqual(journal_properties["venue_roots"]["type"], "array")
        self.assertEqual(journal_properties["venue_roots"]["items"]["type"], "string")
        self.assertEqual(journal_properties["limit"]["type"], "integer")

    def test_lifecycle_plan_tool_returns_preview_report_without_launching_agents(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            (root / "manuscript").mkdir()
            (root / "manuscript" / "manuscript.md").write_text("draft", encoding="utf-8")

            with mock.patch.object(
                tool_handlers,
                "ModelOrchestrator",
                side_effect=AssertionError("lifecycle preview must not launch agents"),
            ):
                result = call_qiongli_tool(
                    "qiongli_lifecycle_plan",
                    {"cwd": str(root), "topic": "demo", "paper_type": "empirical"},
                )

        self.assertFalse(result.get("isError"), result)
        payload = result["structuredContent"]
        self.assertEqual(payload["schema_version"], "1.0")
        self.assertEqual(payload["mode"], "preview")
        self.assertEqual(payload["topic"], "demo")
        self.assertEqual(payload["paper_type"], "empirical")

    def test_journal_fit_tool_blocks_missing_manuscript(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            result = call_qiongli_tool(
                "qiongli_journal_fit_recommend",
                {"cwd": str(root), "venue_roots": []},
            )

        self.assertFalse(result.get("isError"), result)
        payload = result["structuredContent"]
        self.assertEqual(payload["schema_version"], "1.0")
        self.assertEqual(payload["status"], "blocked")
        self.assertIn("missing manuscript/manuscript.md", payload["blocking_reasons"])

    def test_journal_fit_tool_defaults_venue_roots_and_honors_limit(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            self._write_journal_fit_fixture(root)

            result = call_qiongli_tool(
                "qiongli_journal_fit_recommend",
                {"cwd": str(root), "limit": 1},
            )

        payload = result["structuredContent"]
        self.assertFalse(result.get("isError"), result)
        self.assertEqual(payload["status"], "ok")
        self.assertEqual(len(payload["ranked_venues"]), 1)
        self.assertEqual(payload["ranked_venues"][0]["venue_id"], "primary")
        self.assertEqual(payload["ranked_venues"][0]["source"], "venues/primary.yaml")
        self.assertFalse(Path(payload["ranked_venues"][0]["source"]).is_absolute())

    def test_journal_fit_tool_resolves_relative_venue_roots_against_cwd(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            self._write_journal_fit_fixture(root, venue_dir="profiles")

            result = call_qiongli_tool(
                "qiongli_journal_fit_recommend",
                {"cwd": str(root), "venue_roots": ["profiles"], "limit": 1},
            )

        payload = result["structuredContent"]
        self.assertFalse(result.get("isError"), result)
        self.assertEqual(payload["status"], "ok")
        self.assertEqual(len(payload["ranked_venues"]), 1)
        self.assertEqual(payload["ranked_venues"][0]["venue_id"], "primary")
        self.assertEqual(payload["ranked_venues"][0]["source"], "profiles/primary.yaml")
        self.assertFalse(Path(payload["ranked_venues"][0]["source"]).is_absolute())

    def test_journal_fit_tool_rejects_invalid_venue_roots(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            for venue_roots in ("venues", [123], [None]):
                with self.subTest(venue_roots=venue_roots):
                    result = call_qiongli_tool(
                        "qiongli_journal_fit_recommend",
                        {"cwd": str(root), "venue_roots": venue_roots},
                    )

                    self.assertTrue(result["isError"])
                    self.assertIn("venue_roots", result["structuredContent"]["error"])

    def test_journal_fit_tool_limit_zero_returns_empty_ranked_venues(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            self._write_journal_fit_fixture(root)

            result = call_qiongli_tool(
                "qiongli_journal_fit_recommend",
                {"cwd": str(root), "limit": 0},
            )

        payload = result["structuredContent"]
        self.assertFalse(result.get("isError"), result)
        self.assertEqual(payload["status"], "ok")
        self.assertEqual(payload["ranked_venues"], [])

    def test_journal_fit_tool_rejects_invalid_limit_values(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            for limit in (-1, True, "1", 1.5):
                with self.subTest(limit=limit):
                    result = call_qiongli_tool(
                        "qiongli_journal_fit_recommend",
                        {"cwd": str(root), "limit": limit},
                    )

                    self.assertTrue(result["isError"])
                    self.assertIn("limit", result["structuredContent"]["error"])

    def test_subject_lifecycle_schema_derives_enums_from_shared_constants(self) -> None:
        original_module = tool_handlers
        patched_subjects = ("auto", "core", "economics", "finance", "new-official-subject")
        try:
            with (
                mock.patch.object(subject_lifecycle, "ACTIONS", {"unlock", "confirm", "reset"}),
                mock.patch.object(project_manifest, "OFFICIAL_SUBJECTS", patched_subjects),
            ):
                reloaded = importlib.reload(tool_handlers)
                definitions = {tool["name"]: tool for tool in reloaded.MCP_TOOL_DEFINITIONS}
                update_schema = definitions["qiongli_subject_update"]["inputSchema"]
                expected_actions = [
                    action
                    for action in reloaded.SUBJECT_LIFECYCLE_ACTION_ORDER
                    if action in subject_lifecycle.ACTIONS
                ]
                expected_subjects = [
                    subject for subject in patched_subjects if subject not in {"auto", "core"}
                ]

                self.assertEqual(
                    update_schema["properties"]["action"]["enum"],
                    expected_actions,
                )
                self.assertEqual(
                    update_schema["properties"]["subject"]["enum"],
                    expected_subjects,
                )
        finally:
            importlib.reload(original_module)

    def test_subject_status_reports_auto_state_without_creating_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            manifest_path = root / ".qiongli" / "guidance_manifest.yaml"

            result = call_qiongli_tool("qiongli_subject_status", {"cwd": str(root)})

            payload = result["structuredContent"]
            self.assertFalse(result["isError"])
            self.assertEqual(payload["manifest"]["active_subject"], "auto")
            self.assertEqual(payload["manifest"]["subject_mode"], "auto")
            self.assertIn("state", payload)
            self.assertFalse(manifest_path.exists())

    def test_subject_update_confirm_finance_writes_confirmed_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)

            result = call_qiongli_tool(
                "qiongli_subject_update",
                {"cwd": str(root), "action": "confirm", "subject": "finance"},
            )

            payload = result["structuredContent"]
            self.assertFalse(result["isError"])
            self.assertEqual(payload["manifest"]["active_subject"], "finance")
            self.assertEqual(payload["manifest"]["subject_mode"], "confirmed")
            self.assertTrue((root / ".qiongli" / "guidance_manifest.yaml").exists())

    def test_subject_update_read_only_exports_proposed_action_without_writing_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)

            result = call_qiongli_tool(
                "qiongli_subject_update",
                {
                    "cwd": str(root),
                    "action": "confirm",
                    "subject": "finance",
                    "read_only": True,
                    "run_id": "run-1",
                },
            )

            payload = result["structuredContent"]
            self.assertFalse(result["isError"])
            self.assertEqual(payload["write_mode"], "proposed")
            self.assertFalse((root / ".qiongli" / "guidance_manifest.yaml").exists())
            self.assertEqual(payload["proposed_action"]["action"], "confirm")
            self.assertEqual(payload["proposed_action"]["subject"], "finance")
            self.assertEqual(payload["proposed_action"]["source"], "mcp")
            self.assertIn("qiongli subject confirm finance", payload["proposed_action"]["apply_command"])

    def test_subject_update_returns_materialized_guidance_status(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)

            result = call_qiongli_tool(
                "qiongli_subject_update",
                {"cwd": str(root), "action": "confirm", "subject": "finance"},
            )

            payload = result["structuredContent"]
            guidance = payload["subject_guidance"]
            self.assertFalse(result["isError"])
            self.assertTrue(guidance["exists"])
            self.assertEqual(guidance["managed_block"], "active")
            self.assertEqual(guidance["active_subject"], "finance")

    def test_subject_status_returns_materialized_guidance_status(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            call_qiongli_tool(
                "qiongli_subject_update",
                {"cwd": str(root), "action": "lock", "subject": "economics"},
            )

            result = call_qiongli_tool("qiongli_subject_status", {"cwd": str(root)})

            payload = result["structuredContent"]
            guidance = payload["subject_guidance"]
            self.assertFalse(result["isError"])
            self.assertEqual(guidance["active_subject"], "economics")
            self.assertEqual(guidance["subject_mode"], "locked")

    def test_subject_update_lock_finance_writes_locked_manifest_with_run_id(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)

            result = call_qiongli_tool(
                "qiongli_subject_update",
                {
                    "cwd": str(root),
                    "action": "lock",
                    "subject": "finance",
                    "run_id": 123,
                },
            )

            payload = result["structuredContent"]
            self.assertFalse(result["isError"])
            self.assertEqual(payload["manifest"]["active_subject"], "finance")
            self.assertEqual(payload["manifest"]["subject_mode"], "locked")
            self.assertEqual(payload["state"]["lifecycle_events"][-1]["source"], "mcp")
            self.assertEqual(payload["state"]["lifecycle_events"][-1]["run_id"], "123")

    def test_subject_update_unlock_after_lock_returns_confirmed_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            call_qiongli_tool(
                "qiongli_subject_update",
                {"cwd": str(root), "action": "lock", "subject": "finance"},
            )

            result = call_qiongli_tool(
                "qiongli_subject_update",
                {"cwd": str(root), "action": "unlock", "run_id": "unlock-1"},
            )

            payload = result["structuredContent"]
            self.assertFalse(result["isError"])
            self.assertEqual(payload["manifest"]["active_subject"], "finance")
            self.assertEqual(payload["manifest"]["subject_mode"], "confirmed")
            self.assertEqual(payload["state"]["lifecycle_events"][-1]["action"], "unlock")
            self.assertEqual(payload["state"]["lifecycle_events"][-1]["source"], "mcp")
            self.assertEqual(payload["state"]["lifecycle_events"][-1]["run_id"], "unlock-1")

    def test_subject_update_dismiss_economics_records_mcp_source_without_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)

            result = call_qiongli_tool(
                "qiongli_subject_update",
                {"cwd": str(root), "action": "dismiss", "subject": "economics"},
            )

            payload = result["structuredContent"]
            self.assertFalse(result["isError"])
            self.assertEqual(
                payload["state"]["dismissed_subjects"]["economics"]["source"],
                "mcp",
            )
            self.assertFalse((root / ".qiongli" / "guidance_manifest.yaml").exists())

    def test_subject_update_reset_after_confirm_restores_auto_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            call_qiongli_tool(
                "qiongli_subject_update",
                {"cwd": str(root), "action": "confirm", "subject": "finance"},
            )

            result = call_qiongli_tool(
                "qiongli_subject_update",
                {"cwd": str(root), "action": "reset"},
            )

            payload = result["structuredContent"]
            self.assertFalse(result["isError"])
            self.assertEqual(payload["manifest"]["active_subject"], "auto")
            self.assertEqual(payload["manifest"]["subject_mode"], "auto")

    def test_subject_update_invalid_action_returns_tool_error(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            result = call_qiongli_tool(
                "qiongli_subject_update",
                {"cwd": tmp_dir, "action": "archive", "subject": "finance"},
            )

        self.assertTrue(result["isError"])
        self.assertIn("Unsupported subject lifecycle action", result["structuredContent"]["error"])

    def test_subject_update_confirm_rejects_auto_or_missing_subject(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            auto_result = call_qiongli_tool(
                "qiongli_subject_update",
                {"cwd": tmp_dir, "action": "confirm", "subject": "auto"},
            )
            missing_result = call_qiongli_tool(
                "qiongli_subject_update",
                {"cwd": tmp_dir, "action": "confirm"},
            )

        self.assertTrue(auto_result["isError"])
        self.assertIn(
            "confirm requires a concrete official subject",
            auto_result["structuredContent"]["error"],
        )
        self.assertTrue(missing_result["isError"])
        self.assertIn("confirm requires a subject", missing_result["structuredContent"]["error"])

    def test_orchestrator_route_recommends_mcp_sequence_for_codex_claude_duo(self) -> None:
        result = call_qiongli_tool(
            "qiongli_orchestrator_route",
            {
                "request": "Use Codex and Claude Code to write and independently review the F3 discussion section.",
                "platform": "codex",
                "cwd": "/tmp/demo",
                "task_id": "F3",
                "paper_type": "empirical",
                "topic": "ai-in-education",
                "execution_mode": "duo",
                "controller": "codex",
                "primary": "codex",
                "reviewer": "claude",
            },
        )

        payload = result["structuredContent"]
        self.assertFalse(result["isError"])
        self.assertEqual(payload["route"], "orchestrator_mcp")
        self.assertEqual(payload["recommended_tool"], "qiongli_task_run")
        self.assertTrue(payload["requires_full_runtime"])
        self.assertIn("Codex", payload["platform_note"])
        self.assertIn("Claude Code", payload["platform_note"])
        self.assertEqual(
            [step["tool"] for step in payload["sequence"]],
            [
                "qiongli_orchestrator_doctor",
                "qiongli_task_plan",
                "qiongli_task_run",
            ],
        )
        task_run_args = payload["sequence"][2]["args"]
        self.assertEqual(task_run_args["task_id"], "F3")
        self.assertEqual(task_run_args["execution_mode"], "duo")
        self.assertEqual(task_run_args["controller"], "codex")
        self.assertEqual(task_run_args["primary"], "codex")
        self.assertEqual(task_run_args["reviewer"], "claude")
        self.assertFalse(task_run_args["run_agents"])

    def test_orchestrator_route_keeps_simple_request_on_skill_workflow(self) -> None:
        result = call_qiongli_tool(
            "qiongli_orchestrator_route",
            {
                "request": "Tighten one paragraph for clarity without launching agents.",
                "platform": "claude_code",
            },
        )

        payload = result["structuredContent"]
        self.assertEqual(payload["route"], "skill_workflow")
        self.assertEqual(payload["recommended_tool"], "qiongli_task_plan")
        self.assertFalse(payload["requires_full_runtime"])
        self.assertIn("skill", payload["why"][0])

    def test_tool_definitions_replace_gemini_runtime_with_antigravity(self) -> None:
        for tool in MCP_TOOL_DEFINITIONS:
            if tool["name"] not in {"qiongli_orchestrator_route", "qiongli_task_run"}:
                continue
            properties = tool["inputSchema"]["properties"]
            for field in ("controller", "primary", "reviewer", "verifier"):
                with self.subTest(tool=tool["name"], field=field):
                    self.assertNotIn("gemini", properties[field]["enum"])
                    self.assertEqual(["codex", "claude", "antigravity"], properties[field]["enum"])

    def test_config_status_redacts_saved_provider_values(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            with mock.patch.dict(
                "os.environ",
                {"QIONGLI_CONFIG_HOME": str(root / "config")},
                clear=False,
            ):
                set_provider_value("openalex", "api-key", "openalex-secret-key")
                set_provider_value("semantic-scholar", "api-key", "secret-demo-key")
                result = call_qiongli_tool("qiongli_config_status", {"cwd": str(root)})

        rendered = json.dumps(result, sort_keys=True)
        self.assertEqual(
            result["structuredContent"]["providers"]["openalex"],
            "configured",
        )
        self.assertEqual(
            result["structuredContent"]["providers"]["semantic_scholar"],
            "configured",
        )
        self.assertEqual(result["structuredContent"]["capability_mode"], "provider_connected")
        self.assertNotIn("openalex-secret-key", rendered)
        self.assertNotIn("secret-demo-key", rendered)

    def test_config_status_suggests_platform_neutral_configure_tool(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            with mock.patch.dict(
                "os.environ",
                {"QIONGLI_CONFIG_HOME": str(root / "config")},
                clear=False,
            ):
                result = call_qiongli_tool("qiongli_config_status", {"cwd": str(root)})

        payload = result["structuredContent"]
        self.assertEqual(payload["providers"]["openalex"], "missing")
        self.assertEqual(payload["providers"]["semantic_scholar"], "missing")
        self.assertEqual(payload["missing"], ["openalex.api_key", "semantic_scholar.api_key"])
        self.assertEqual(payload["next_action"]["tool"], "qiongli_configure_provider")
        self.assertEqual(payload["next_action"]["args"], {"provider": "openalex"})

    def test_save_provider_config_persists_to_shared_config(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            with mock.patch.dict(
                "os.environ",
                {"QIONGLI_CONFIG_HOME": str(root / "config")},
                clear=False,
            ):
                result = call_qiongli_tool(
                    "qiongli_save_provider_config",
                    {"provider": "openalex", "field": "api-key", "value": "openalex-secret-key"},
                )
                status = call_qiongli_tool("qiongli_config_status", {"cwd": str(root)})

        rendered = json.dumps(result, sort_keys=True)
        self.assertEqual(result["structuredContent"]["provider"], "openalex")
        self.assertEqual(result["structuredContent"]["field"], "api_key")
        self.assertEqual(status["structuredContent"]["providers"]["openalex"], "configured")
        self.assertNotIn("openalex-secret-key", rendered)

    def test_save_provider_config_warns_for_chat_api_key_input(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            with mock.patch.dict(
                "os.environ",
                {"QIONGLI_CONFIG_HOME": str(root / "config")},
                clear=False,
            ):
                result = call_qiongli_tool(
                    "qiongli_save_provider_config",
                    {"provider": "semantic-scholar", "field": "api-key", "value": "secret-demo-key"},
                )

        rendered = json.dumps(result, sort_keys=True)
        self.assertEqual(result["structuredContent"]["provider"], "semantic_scholar")
        self.assertEqual(result["structuredContent"]["field"], "api_key")
        self.assertIn("Prefer qiongli_configure_provider", result["structuredContent"]["warning"])
        self.assertNotIn("secret-demo-key", rendered)

    def test_collect_evidence_tool_uses_existing_connector(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            project_root = root / "RESEARCH" / "demo-topic"
            project_root.mkdir(parents=True)
            (project_root / "analysis.md").write_text("# analysis\n", encoding="utf-8")

            result = call_qiongli_tool(
                "qiongli_collect_evidence",
                {
                    "cwd": str(root),
                    "provider": "filesystem",
                    "task_packet": {
                        "topic": "demo-topic",
                        "artifact_root": "RESEARCH/[topic]/",
                        "required_outputs": ["analysis.md"],
                    },
                },
            )

        self.assertEqual(result["structuredContent"]["evidence"]["status"], "ok")
        self.assertEqual(
            result["structuredContent"]["evidence"]["data"]["existing_output_count"],
            1,
        )

    def test_collect_evidence_description_names_external_command_boundary(self) -> None:
        definitions = {tool["name"]: tool for tool in MCP_TOOL_DEFINITIONS}
        description = definitions["qiongli_collect_evidence"]["description"]

        self.assertIn("external command adapters", description)
        self.assertIn("qiongli_literature_status", description)
        self.assertIn("qiongli_literature_search", description)

    def test_collect_evidence_openalex_not_configured_is_not_provider_config_status(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            with mock.patch.dict(
                "os.environ",
                {
                    "QIONGLI_CONFIG_HOME": str(root / "config"),
                    "RESEARCH_MCP_OPENALEX_CMD": "",
                },
                clear=False,
            ):
                set_provider_value("openalex", "api-key", "openalex-secret-key")
                result = call_qiongli_tool(
                    "qiongli_collect_evidence",
                    {
                        "cwd": str(root),
                        "provider": "openalex",
                        "task_packet": {"topic": "demo-topic"},
                    },
                )

        payload = result["structuredContent"]["evidence"]
        rendered = json.dumps(result, sort_keys=True)
        self.assertFalse(result["isError"])
        self.assertEqual(payload["status"], "not_configured")
        self.assertEqual(payload["data"]["not_configured_scope"], "external_command_adapter")
        self.assertEqual(payload["data"]["provider_config_status"], "configured")
        self.assertEqual(payload["data"]["recommended_status_tool"], "qiongli_literature_status")
        self.assertEqual(payload["data"]["recommended_search_tool"], "qiongli_literature_search")
        self.assertNotIn("openalex-secret-key", rendered)

    def test_list_provider_env_returns_aliases_not_values(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            with mock.patch.dict(
                "os.environ",
                {"QIONGLI_CONFIG_HOME": str(root / "config")},
                clear=False,
            ):
                set_provider_value("semantic-scholar", "api-key", "secret-demo-key")
                result = call_qiongli_tool("qiongli_list_provider_env", {})

        rendered = json.dumps(result, sort_keys=True)
        aliases = result["structuredContent"]["providers"]["openalex"]["api_key"]
        self.assertIn("QIONGLI_OPENALEX_API_KEY", aliases)
        self.assertIn("OPENALEX_API_KEY", aliases)
        aliases = result["structuredContent"]["providers"]["semantic_scholar"]["api_key"]
        self.assertIn("QIONGLI_SEMANTIC_SCHOLAR_API_KEY", aliases)
        self.assertIn("S2_API_KEY", aliases)
        self.assertNotIn("secret-demo-key", rendered)

    def test_search_plan_tool_uses_status_capability_without_leaking_secrets(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            with mock.patch.dict(
                "os.environ",
                {"QIONGLI_CONFIG_HOME": str(root / "config")},
                clear=False,
            ):
                set_provider_value("openalex", "api-key", "openalex-secret-key")
                result = call_qiongli_tool(
                    "qiongli_search_plan",
                    {
                        "cwd": str(root),
                        "query": "AI feedback in education",
                        "platform": "codex",
                        "native_search_available": True,
                    },
                )

        payload = result["structuredContent"]
        rendered = json.dumps(result, sort_keys=True)
        self.assertFalse(result["isError"])
        self.assertEqual(payload["artifact_type"], "qiongli_hybrid_search_plan")
        self.assertEqual(payload["search_execution_mode"], "hybrid_search")
        self.assertEqual(payload["provider_capability_mode"], "provider_connected")
        self.assertEqual(payload["native_search_tools"], ["codex_web_search"])
        self.assertEqual(
            [query["provider"] for query in payload["provider_queries"]],
            ["openalex", "arxiv"],
        )
        self.assertEqual(payload["provenance_labels"]["provider"], ["mcp:openalex", "mcp:arxiv"])
        self.assertNotIn("openalex-secret-key", rendered)

    def test_open_config_wizard_returns_local_url(self) -> None:
        class StubWizard:
            url = "http://127.0.0.1:8765/?token=abc"
            host = "127.0.0.1"
            port = 8765
            token = "abc"
            config_path = "/tmp/qiongli/providers.json"

        with mock.patch.object(
            tool_handlers,
            "start_config_wizard",
            lambda **_: StubWizard(),
        ):
            result = call_qiongli_tool(
                "qiongli_open_config_wizard",
                {"host": "127.0.0.1", "port": 0},
            )

        self.assertEqual(
            result["structuredContent"]["url"],
            "http://127.0.0.1:8765/?token=abc",
        )
        self.assertEqual(result["structuredContent"]["config_path"], "/tmp/qiongli/providers.json")

    def test_configure_provider_returns_local_wizard_url(self) -> None:
        class StubWizard:
            url = "http://127.0.0.1:8765/?token=abc"
            host = "127.0.0.1"
            port = 8765
            token = "abc"
            config_path = "/tmp/qiongli/providers.json"

        with mock.patch.object(
            tool_handlers,
            "start_config_wizard",
            lambda **_: StubWizard(),
        ):
            result = call_qiongli_tool(
                "qiongli_configure_provider",
                {"provider": "semantic_scholar", "host": "127.0.0.1", "port": 0},
            )

        self.assertEqual(
            result["structuredContent"]["url"],
            "http://127.0.0.1:8765/?token=abc",
        )
        self.assertEqual(result["structuredContent"]["provider"], "semantic_scholar")
        self.assertEqual(result["structuredContent"]["config_path"], "/tmp/qiongli/providers.json")

    def test_orchestrator_doctor_tool_returns_structured_result(self) -> None:
        class StubResult:
            mode = "doctor"
            confidence = 1.0
            merged_analysis = "doctor ok"
            recommendations = ["ready"]
            data = {"checks": [{"status": "ok", "label": "Working directory"}]}

        class StubOrchestrator:
            def doctor(self, cwd: Path) -> StubResult:
                self.cwd = cwd
                return StubResult()

        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            with mock.patch.object(tool_handlers, "ModelOrchestrator", return_value=StubOrchestrator()):
                result = call_qiongli_tool("qiongli_orchestrator_doctor", {"cwd": str(root)})

        payload = result["structuredContent"]
        self.assertFalse(result["isError"])
        self.assertEqual(payload["mode"], "doctor")
        self.assertEqual(payload["data"]["checks"][0]["status"], "ok")

    def test_task_plan_tool_uses_orchestrator_without_running_agents(self) -> None:
        class StubResult:
            mode = "task-plan"
            confidence = 0.9
            merged_analysis = "plan ok"
            recommendations: list[str] = []
            data = {
                "task_packet": {"task_id": "F3", "topic": "my-topic"},
                "runtime_plan": {"draft": "codex", "review": "claude"},
            }

        class StubOrchestrator:
            def task_plan(self, **kwargs: object) -> StubResult:
                self.kwargs = kwargs
                return StubResult()

        stub = StubOrchestrator()
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            with mock.patch.object(tool_handlers, "ModelOrchestrator", return_value=stub):
                result = call_qiongli_tool(
                    "qiongli_task_plan",
                    {
                        "cwd": str(root),
                        "task_id": "F3",
                        "paper_type": "empirical",
                        "topic": "my-topic",
                        "primary": "codex",
                        "reviewer": "claude",
                    },
                )

        payload = result["structuredContent"]
        self.assertEqual(payload["mode"], "task-plan")
        self.assertEqual(payload["data"]["runtime_plan"]["draft"], "codex")
        self.assertEqual(stub.kwargs["task_id"], "F3")
        self.assertEqual(stub.kwargs["topic"], "my-topic")

    def test_task_run_tool_defaults_to_preview_until_agents_are_enabled(self) -> None:
        class StubResult:
            mode = "task-plan"
            confidence = 0.8
            merged_analysis = "preview"
            recommendations: list[str] = []
            data = {"task_packet": {"task_id": "F3"}}

        class StubOrchestrator:
            ran_agents = False

            def task_plan(self, **_kwargs: object) -> StubResult:
                return StubResult()

            def task_run(self, **_kwargs: object) -> StubResult:
                self.ran_agents = True
                return StubResult()

            def _build_controller_metadata(self, **_kwargs: object) -> dict[str, str]:
                return {
                    "execution_mode": "duo",
                    "controller": "codex",
                    "primary_agent": "",
                    "review_agent": "",
                    "verifier_agent": "",
                    "solo_role_gates": "standard",
                }

            def _controller_runtime_overrides(self, _metadata: dict[str, str]) -> dict[str, str]:
                return {}

        stub = StubOrchestrator()
        with mock.patch.object(tool_handlers, "ModelOrchestrator", return_value=stub):
            result = call_qiongli_tool(
                "qiongli_task_run",
                {
                    "task_id": "F3",
                    "paper_type": "empirical",
                    "topic": "my-topic",
                    "cwd": ".",
                },
            )

        payload = result["structuredContent"]
        self.assertEqual(payload["mode"], "task-run-preview")
        self.assertFalse(payload["run_agents"])
        self.assertFalse(stub.ran_agents)

    def test_task_run_tool_rejects_non_boolean_run_agents_gate(self) -> None:
        class StubResult:
            mode = "task-run"
            confidence = 0.95
            merged_analysis = "run ok"
            recommendations: list[str] = []
            data: dict[str, object] = {}

        class StubOrchestrator:
            ran_agents = False

            def task_run(self, **_kwargs: object) -> StubResult:
                self.ran_agents = True
                return StubResult()

        for unsafe_value in ("true", "preview", 1):
            with self.subTest(unsafe_value=unsafe_value):
                stub = StubOrchestrator()
                with mock.patch.object(tool_handlers, "ModelOrchestrator", return_value=stub):
                    result = call_qiongli_tool(
                        "qiongli_task_run",
                        {
                            "task_id": "F3",
                            "paper_type": "empirical",
                            "topic": "my-topic",
                            "cwd": ".",
                            "run_agents": unsafe_value,
                        },
                    )

                self.assertTrue(result["isError"])
                self.assertIn("run_agents must be the JSON boolean", result["structuredContent"]["error"])
                self.assertFalse(stub.ran_agents)

    def test_task_run_preview_exposes_effective_runtime_options(self) -> None:
        class StubResult:
            mode = "task-plan"
            confidence = 0.8
            merged_analysis = "preview"
            recommendations: list[str] = []
            data = {
                "task_id": "F3",
                "paper_type": "empirical",
                "topic": "my-topic",
                "artifact_root": "RESEARCH/[topic]/",
                "runtime_plan": {
                    "primary_agent": "codex",
                    "review_agent": "claude",
                    "fallback_agent": "claude",
                },
            }

        class StubOrchestrator:
            def task_plan(self, **_kwargs: object) -> StubResult:
                return StubResult()

            def _build_controller_metadata(self, **kwargs: object) -> dict[str, str]:
                self.controller_kwargs = kwargs
                return {
                    "execution_mode": str(kwargs["execution_mode"]),
                    "controller": str(kwargs["controller"]),
                    "primary_agent": str(kwargs["primary_agent"]),
                    "review_agent": str(kwargs["review_agent"]),
                    "verifier_agent": str(kwargs["verifier_agent"] or ""),
                    "solo_role_gates": str(kwargs["solo_role_gates"]),
                }

            def _controller_runtime_overrides(self, metadata: dict[str, str]) -> dict[str, str]:
                return {
                    "primary_agent": metadata["primary_agent"],
                    "review_agent": metadata["review_agent"],
                }

        stub = StubOrchestrator()
        with mock.patch.object(tool_handlers, "ModelOrchestrator", return_value=stub):
            result = call_qiongli_tool(
                "qiongli_task_run",
                {
                    "task_id": "F3",
                    "paper_type": "empirical",
                    "topic": "my-topic",
                    "cwd": ".",
                    "execution_mode": "duo",
                    "controller": "claude",
                    "primary": "codex",
                    "reviewer": "claude",
                    "solo_role_gates": "strict",
                },
            )

        payload = result["structuredContent"]
        preview = payload["data"]["task_run_preview"]
        self.assertEqual(preview["controller_metadata"]["controller"], "claude")
        self.assertEqual(preview["effective_runtime_plan"]["primary_agent"], "codex")
        self.assertEqual(preview["effective_runtime_plan"]["review_agent"], "claude")
        self.assertEqual(preview["task_run_arguments"]["primary_agent"], "codex")
        self.assertEqual(preview["task_run_arguments"]["review_agent"], "claude")
        self.assertFalse(preview["will_launch_agents"])

    def test_task_run_preview_accepts_guidance_mode(self) -> None:
        class StubResult:
            mode = "task-plan"
            confidence = 0.8
            merged_analysis = "preview"
            recommendations: list[str] = []
            data = {
                "task_id": "F3",
                "paper_type": "empirical",
                "topic": "my-topic",
                "artifact_root": "RESEARCH/[topic]/",
                "runtime_plan": {
                    "primary_agent": "codex",
                    "review_agent": "claude",
                    "fallback_agent": "claude",
                },
            }

        class StubOrchestrator:
            def task_plan(self, **_kwargs: object) -> StubResult:
                return StubResult()

            def _build_controller_metadata(self, **_kwargs: object) -> dict[str, str]:
                return {
                    "execution_mode": "duo",
                    "controller": "codex",
                    "primary_agent": "",
                    "review_agent": "",
                    "verifier_agent": "",
                    "solo_role_gates": "standard",
                }

            def _controller_runtime_overrides(self, _metadata: dict[str, str]) -> dict[str, str]:
                return {}

        stub = StubOrchestrator()
        with mock.patch.object(tool_handlers, "ModelOrchestrator", return_value=stub):
            result = call_qiongli_tool(
                "qiongli_task_run",
                {
                    "task_id": "F3",
                    "paper_type": "empirical",
                    "topic": "my-topic",
                    "cwd": ".",
                    "guidance_mode": "read",
                },
            )

        preview = result["structuredContent"]["data"]["task_run_preview"]
        self.assertEqual(preview["task_run_arguments"]["guidance_mode"], "read")

    def test_task_run_preview_reports_guidance_bootstrap_without_writing(self) -> None:
        class StubResult:
            mode = "task-plan"
            confidence = 0.8
            merged_analysis = "preview"
            recommendations: list[str] = []
            data = {
                "task_id": "F3",
                "paper_type": "empirical",
                "topic": "my-topic",
                "artifact_root": "RESEARCH/[topic]/",
                "runtime_plan": {
                    "primary_agent": "codex",
                    "review_agent": "claude",
                    "fallback_agent": "claude",
                },
            }

        class StubOrchestrator:
            def task_plan(self, **_kwargs: object) -> StubResult:
                return StubResult()

            def _build_controller_metadata(self, **_kwargs: object) -> dict[str, str]:
                return {
                    "execution_mode": "duo",
                    "controller": "codex",
                    "primary_agent": "",
                    "review_agent": "",
                    "verifier_agent": "",
                    "solo_role_gates": "standard",
                }

            def _controller_runtime_overrides(self, _metadata: dict[str, str]) -> dict[str, str]:
                return {}

        stub = StubOrchestrator()
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            with mock.patch.object(tool_handlers, "ModelOrchestrator", return_value=stub):
                result = call_qiongli_tool(
                    "qiongli_task_run",
                    {
                        "task_id": "F3",
                        "paper_type": "empirical",
                        "topic": "my-topic",
                        "cwd": str(root),
                    },
                )

            preview = result["structuredContent"]["data"]["task_run_preview"]
            self.assertTrue(preview["guidance_bootstrap"]["needed"])
            self.assertEqual(
                preview["guidance_bootstrap"]["project_guidance"],
                ".qiongli/local_guidance.md",
            )
            self.assertEqual(preview["guidance_bootstrap"]["guidance_dir"], ".qiongli/guidance.d")
            self.assertEqual(preview["guidance_bootstrap"]["guidance_fragment_count"], 0)
            self.assertFalse((root / ".qiongli").exists())

    def test_task_run_preview_packet_includes_materialized_subject_guidance(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            update_result = call_qiongli_tool(
                "qiongli_subject_update",
                {
                    "cwd": str(root),
                    "action": "confirm",
                    "subject": "finance",
                    "run_id": "setup-confirm-finance",
                },
            )
            self.assertFalse(update_result["isError"])

            result, _stub = self._call_task_run_preview(
                {
                    "task_id": "C1",
                    "paper_type": "empirical",
                    "topic": "earnings announcement stock market reaction",
                    "context": (
                        "Use event-study evidence and Journal of Finance standards "
                        "for this empirical paper."
                    ),
                    "domain": "auto",
                    "guidance_mode": "propose",
                    "cwd": str(root),
                },
            )

        task_packet = result["structuredContent"]["data"]["task_packet"]
        guidance = task_packet["local_guidance"]
        self.assertIn(
            ".qiongli/guidance.d/subject-runtime.md",
            guidance["guidance_files_read"],
        )

    def test_task_run_preview_maps_triad_execution_mode_to_metadata(self) -> None:
        class StubResult:
            mode = "task-plan"
            confidence = 0.8
            merged_analysis = "preview"
            recommendations: list[str] = []
            data = {
                "task_id": "F3",
                "paper_type": "empirical",
                "topic": "my-topic",
                "artifact_root": "RESEARCH/[topic]/",
                "runtime_plan": {
                    "primary_agent": "codex",
                    "review_agent": "claude",
                    "fallback_agent": "claude",
                },
            }

        class StubOrchestrator:
            def task_plan(self, **_kwargs: object) -> StubResult:
                return StubResult()

            def _build_controller_metadata(self, **kwargs: object) -> dict[str, str]:
                self.controller_kwargs = kwargs
                return {
                    "execution_mode": str(kwargs["execution_mode"]),
                    "controller": str(kwargs["controller"]),
                    "primary_agent": str(kwargs["primary_agent"]),
                    "review_agent": str(kwargs["review_agent"]),
                    "verifier_agent": str(kwargs["verifier_agent"] or ""),
                    "solo_role_gates": str(kwargs["solo_role_gates"]),
                    "triad": str(kwargs["triad"]),
                }

            def _controller_runtime_overrides(self, _metadata: dict[str, str]) -> dict[str, str]:
                return {}

        stub = StubOrchestrator()
        with mock.patch.object(tool_handlers, "ModelOrchestrator", return_value=stub):
            result = call_qiongli_tool(
                "qiongli_task_run",
                {
                    "task_id": "F3",
                    "paper_type": "empirical",
                    "topic": "my-topic",
                    "cwd": ".",
                    "execution_mode": "triad",
                    "controller": "codex",
                    "primary": "codex",
                    "reviewer": "claude",
                    "verifier": "antigravity",
                },
            )

        self.assertFalse(result["isError"])
        preview = result["structuredContent"]["data"]["task_run_preview"]
        self.assertIs(stub.controller_kwargs["triad"], True)
        self.assertEqual(preview["controller_metadata"]["triad"], "True")
        self.assertEqual(preview["task_run_arguments"]["verifier_agent"], "antigravity")

    def test_task_run_preview_includes_domain_context_in_task_packet(self) -> None:
        class StubResult:
            mode = "task-plan"
            confidence = 0.8
            merged_analysis = "preview"
            recommendations: list[str] = []
            data = {
                "task_id": "F3",
                "paper_type": "empirical",
                "topic": "my-topic",
                "artifact_root": "RESEARCH/[topic]/",
                "runtime_plan": {
                    "primary_agent": "codex",
                    "review_agent": "claude",
                    "fallback_agent": "claude",
                },
            }

        class StubOrchestrator:
            def task_plan(self, **_kwargs: object) -> StubResult:
                return StubResult()

            def _build_controller_metadata(self, **_kwargs: object) -> dict[str, str]:
                return {
                    "execution_mode": "duo",
                    "controller": "codex",
                    "primary_agent": "",
                    "review_agent": "",
                    "verifier_agent": "",
                    "solo_role_gates": "standard",
                }

            def _controller_runtime_overrides(self, _metadata: dict[str, str]) -> dict[str, str]:
                return {}

            def _load_domain_profile_context(self, domain: str) -> dict[str, str]:
                return {
                    "requested_domain": domain,
                    "domain": "finance",
                    "status": "loaded",
                    "display_name": "Finance",
                }

            def _build_domain_packet_fields(self, domain_context: dict[str, str]) -> dict[str, str]:
                return {
                    "domain": domain_context["domain"],
                    "requested_domain": domain_context["requested_domain"],
                    "domain_profile_status": domain_context["status"],
                    "domain_profile_display_name": domain_context["display_name"],
                }

        stub = StubOrchestrator()
        with mock.patch.object(tool_handlers, "ModelOrchestrator", return_value=stub):
            result = call_qiongli_tool(
                "qiongli_task_run",
                {
                    "task_id": "F3",
                    "paper_type": "empirical",
                    "topic": "my-topic",
                    "cwd": ".",
                    "domain": "finance",
                },
            )

        task_packet = result["structuredContent"]["data"]["task_packet"]
        self.assertEqual(task_packet["domain"], "finance")
        self.assertEqual(task_packet["requested_domain"], "finance")
        self.assertIn("domain_profile_status", task_packet)
        self.assertIn("domain_profile_display_name", task_packet)

    def test_task_run_preview_includes_auto_domain_context_in_task_packet(self) -> None:
        class StubResult:
            mode = "task-plan"
            confidence = 0.8
            merged_analysis = "preview"
            recommendations: list[str] = []
            data = {
                "task_id": "F3",
                "paper_type": "empirical",
                "topic": "my-topic",
                "artifact_root": "RESEARCH/[topic]/",
                "runtime_plan": {
                    "primary_agent": "codex",
                    "review_agent": "claude",
                    "fallback_agent": "claude",
                },
            }

        class StubOrchestrator:
            def task_plan(self, **_kwargs: object) -> StubResult:
                return StubResult()

            def _build_controller_metadata(self, **_kwargs: object) -> dict[str, str]:
                return {
                    "execution_mode": "duo",
                    "controller": "codex",
                    "primary_agent": "",
                    "review_agent": "",
                    "verifier_agent": "",
                    "solo_role_gates": "standard",
                }

            def _controller_runtime_overrides(self, _metadata: dict[str, str]) -> dict[str, str]:
                return {}

            def _load_domain_profile_context(self, domain: str) -> dict[str, str]:
                return {
                    "requested_domain": domain,
                    "domain": "auto",
                    "status": "auto",
                    "display_name": "Auto-detect",
                }

            def _build_domain_packet_fields(self, domain_context: dict[str, str]) -> dict[str, str]:
                return {
                    "domain": domain_context["domain"],
                    "requested_domain": domain_context["requested_domain"],
                    "domain_profile_status": domain_context["status"],
                    "domain_profile_display_name": domain_context["display_name"],
                }

        stub = StubOrchestrator()
        with mock.patch.object(tool_handlers, "ModelOrchestrator", return_value=stub):
            result = call_qiongli_tool(
                "qiongli_task_run",
                {
                    "task_id": "F3",
                    "paper_type": "empirical",
                    "topic": "my-topic",
                    "cwd": ".",
                },
            )

        task_packet = result["structuredContent"]["data"]["task_packet"]
        self.assertEqual(task_packet["domain"], "auto")
        self.assertEqual(task_packet["requested_domain"], "auto")
        self.assertIn("domain_profile_status", task_packet)
        self.assertIn("domain_profile_display_name", task_packet)

    def test_task_run_preview_exposes_borrowed_subject_refinement_without_domain_switch(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            result, stub = self._call_task_run_preview(
                {
                    "task_id": "F3",
                    "paper_type": "empirical",
                    "topic": "management disclosure around announcements",
                    "cwd": tmp_dir,
                    "context": "Use an event study design for the disclosure event.",
                },
            )

        payload = result["structuredContent"]
        preview = payload["data"]["task_run_preview"]
        task_packet = payload["data"]["task_packet"]
        refinement = preview["subject_refinement"]
        self.assertEqual(refinement["decision"], "borrow_lens")
        self.assertEqual(refinement["primary_subject"], "auto")
        self.assertEqual(refinement["domain"], "auto")
        self.assertEqual(refinement["borrowed_lenses"][0]["source_subject"], "finance")
        self.assertEqual(refinement["borrowed_lenses"][0]["lens"], "event-study")
        self.assertEqual(preview["effective_domain"], "auto")
        self.assertEqual(task_packet["domain"], "auto")
        self.assertEqual(task_packet["requested_domain"], "auto")
        self.assertEqual(task_packet["subject_refinement"], refinement)
        self.assertEqual(stub.loaded_domain, "auto")

    def test_task_run_preview_uses_suggested_subject_for_temporary_finance_domain_context(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            result, stub = self._call_task_run_preview(
                {
                    "task_id": "F3",
                    "paper_type": "empirical",
                    "topic": "CRSP abnormal returns event study",
                    "cwd": tmp_dir,
                    "venue": "Journal of Finance",
                    "context": "Estimate abnormal returns using CRSP data.",
                },
            )

        payload = result["structuredContent"]
        preview = payload["data"]["task_run_preview"]
        task_packet = payload["data"]["task_packet"]
        refinement = preview["subject_refinement"]
        self.assertEqual(refinement["decision"], "suggest_subject")
        self.assertEqual(refinement["primary_subject"], "finance")
        self.assertEqual(refinement["domain"], "finance")
        self.assertIn("resource_activation_plan", refinement)
        self.assertEqual(refinement["resource_activation_plan"]["primary_subject"], "finance")
        self.assertIn("signals", refinement)
        self.assertIn(
            "finance.method.event-study",
            {signal["id"] for signal in refinement["signals"]},
        )
        self.assertEqual(preview["effective_domain"], "finance")
        self.assertEqual(task_packet["domain"], "finance")
        self.assertEqual(task_packet["requested_domain"], "auto")
        self.assertEqual(task_packet["subject_refinement"], refinement)
        self.assertEqual(stub.loaded_domain, "finance")

    def test_task_run_preview_explicit_domain_overrides_finance_subject_refinement(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            result, stub = self._call_task_run_preview(
                {
                    "task_id": "F3",
                    "paper_type": "empirical",
                    "topic": "CRSP abnormal returns event study",
                    "cwd": tmp_dir,
                    "domain": "economics",
                    "venue": "Journal of Finance",
                    "context": "Estimate abnormal returns using CRSP data.",
                },
            )

        payload = result["structuredContent"]
        preview = payload["data"]["task_run_preview"]
        task_packet = payload["data"]["task_packet"]
        refinement = preview["subject_refinement"]
        self.assertEqual(refinement["decision"], "suggest_subject")
        self.assertEqual(refinement["primary_subject"], "finance")
        self.assertEqual(refinement["domain"], "finance")
        self.assertEqual(preview["effective_domain"], "economics")
        self.assertEqual(task_packet["domain"], "economics")
        self.assertEqual(task_packet["requested_domain"], "economics")
        self.assertEqual(task_packet["subject_refinement"], refinement)
        self.assertEqual(stub.loaded_domain, "economics")

    def test_task_run_preview_reports_project_manifest_state(self) -> None:
        class StubResult:
            mode = "task-plan"
            confidence = 0.8
            merged_analysis = "preview"
            recommendations: list[str] = []
            data = {
                "task_id": "F3",
                "paper_type": "empirical",
                "topic": "my-topic",
                "artifact_root": "RESEARCH/[topic]/",
                "runtime_plan": {
                    "primary_agent": "codex",
                    "review_agent": "claude",
                    "fallback_agent": "claude",
                },
            }

        class StubOrchestrator:
            def task_plan(self, **_kwargs: object) -> StubResult:
                return StubResult()

            def _build_controller_metadata(self, **_kwargs: object) -> dict[str, str]:
                return {
                    "execution_mode": "duo",
                    "controller": "codex",
                    "primary_agent": "",
                    "review_agent": "",
                    "verifier_agent": "",
                    "solo_role_gates": "standard",
                }

            def _controller_runtime_overrides(self, _metadata: dict[str, str]) -> dict[str, str]:
                return {}

        stub = StubOrchestrator()
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            (root / ".qiongli").mkdir()
            (root / ".qiongli" / "guidance_manifest.yaml").write_text(
                "active_subject: finance\n",
                encoding="utf-8",
            )
            with mock.patch.object(tool_handlers, "ModelOrchestrator", return_value=stub):
                result = call_qiongli_tool(
                    "qiongli_task_run",
                    {
                        "task_id": "F3",
                        "paper_type": "empirical",
                        "topic": "my-topic",
                        "cwd": str(root),
                    },
                )

        preview = result["structuredContent"]["data"]["task_run_preview"]
        self.assertEqual(preview["project_manifest"]["manifest"]["active_subject"], "finance")

    def test_task_run_preview_default_guidance_malformed_manifest_falls_back_to_implicit_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            (root / ".qiongli").mkdir()
            (root / ".qiongli" / "guidance_manifest.yaml").write_text(
                "active_subject: [finance\n",
                encoding="utf-8",
            )
            result, _stub = self._call_task_run_preview(
                {
                    "task_id": "F3",
                    "paper_type": "empirical",
                    "topic": "my-topic",
                    "cwd": str(root),
                },
            )

        preview = result["structuredContent"]["data"]["task_run_preview"]
        self.assertEqual(preview["project_subject"]["effective_subject"], "auto")
        self.assertEqual(preview["project_subject"]["domain"], "auto")
        self.assertFalse(preview["project_manifest"]["exists"])
        self.assertEqual(preview["project_manifest"]["manifest"]["active_subject"], "auto")
        self.assertIn(
            "Malformed project manifest",
            " ".join(preview["project_manifest"]["warnings"]),
        )
        self.assertEqual(preview["subject_refinement"]["decision"], "no_subject")
        task_packet = result["structuredContent"]["data"]["task_packet"]
        self.assertEqual(task_packet["subject_refinement"], preview["subject_refinement"])

    def test_task_run_preview_guidance_off_uses_implicit_project_manifest(self) -> None:
        class StubResult:
            mode = "task-plan"
            confidence = 0.8
            merged_analysis = "preview"
            recommendations: list[str] = []
            data = {
                "task_id": "F3",
                "paper_type": "empirical",
                "topic": "my-topic",
                "artifact_root": "RESEARCH/[topic]/",
                "runtime_plan": {
                    "primary_agent": "codex",
                    "review_agent": "claude",
                    "fallback_agent": "claude",
                },
            }

        class StubOrchestrator:
            def task_plan(self, **_kwargs: object) -> StubResult:
                return StubResult()

            def _build_controller_metadata(self, **_kwargs: object) -> dict[str, str]:
                return {
                    "execution_mode": "duo",
                    "controller": "codex",
                    "primary_agent": "",
                    "review_agent": "",
                    "verifier_agent": "",
                    "solo_role_gates": "standard",
                }

            def _controller_runtime_overrides(self, _metadata: dict[str, str]) -> dict[str, str]:
                return {}

        stub = StubOrchestrator()
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            (root / ".qiongli").mkdir()
            (root / ".qiongli" / "guidance_manifest.yaml").write_text(
                "active_subject: [finance\n",
                encoding="utf-8",
            )
            with mock.patch.object(tool_handlers, "ModelOrchestrator", return_value=stub):
                result = call_qiongli_tool(
                    "qiongli_task_run",
                    {
                        "task_id": "F3",
                        "paper_type": "empirical",
                        "topic": "my-topic",
                        "cwd": str(root),
                        "guidance_mode": "off",
                    },
                )

        preview = result["structuredContent"]["data"]["task_run_preview"]
        self.assertEqual(preview["project_subject"]["effective_subject"], "auto")
        self.assertEqual(preview["project_subject"]["domain"], "auto")
        self.assertFalse(preview["project_manifest"]["exists"])
        self.assertEqual(preview["project_manifest"]["manifest"]["active_subject"], "auto")
        self.assertEqual(preview["subject_refinement"]["decision"], "no_subject")
        task_packet = result["structuredContent"]["data"]["task_packet"]
        self.assertEqual(task_packet["subject_refinement"], preview["subject_refinement"])

    def test_task_run_preview_uses_project_manifest_subject_for_auto_domain(self) -> None:
        class StubResult:
            mode = "task-plan"
            confidence = 0.8
            merged_analysis = "preview"
            recommendations: list[str] = []
            data = {
                "task_id": "F3",
                "paper_type": "empirical",
                "topic": "my-topic",
                "artifact_root": "RESEARCH/[topic]/",
                "runtime_plan": {
                    "primary_agent": "codex",
                    "review_agent": "claude",
                    "fallback_agent": "claude",
                },
            }

        class StubOrchestrator:
            def __init__(self) -> None:
                self.loaded_domain = ""

            def task_plan(self, **_kwargs: object) -> StubResult:
                return StubResult()

            def _build_controller_metadata(self, **_kwargs: object) -> dict[str, str]:
                return {
                    "execution_mode": "duo",
                    "controller": "codex",
                    "primary_agent": "",
                    "review_agent": "",
                    "verifier_agent": "",
                    "solo_role_gates": "standard",
                }

            def _controller_runtime_overrides(self, _metadata: dict[str, str]) -> dict[str, str]:
                return {}

            def _load_domain_profile_context(self, domain: str) -> dict[str, str]:
                self.loaded_domain = domain
                return {
                    "requested_domain": domain,
                    "domain": domain,
                    "status": "loaded",
                    "display_name": domain.title(),
                }

            def _build_domain_packet_fields(self, domain_context: dict[str, str]) -> dict[str, str]:
                return {
                    "domain": domain_context["domain"],
                    "requested_domain": domain_context["requested_domain"],
                    "domain_profile_status": domain_context["status"],
                    "domain_profile_display_name": domain_context["display_name"],
                }

        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            (root / ".qiongli").mkdir()
            (root / ".qiongli" / "guidance_manifest.yaml").write_text(
                "active_subject: finance\n",
                encoding="utf-8",
            )
            stub = StubOrchestrator()
            with mock.patch.object(tool_handlers, "ModelOrchestrator", return_value=stub):
                result = call_qiongli_tool(
                    "qiongli_task_run",
                    {
                        "task_id": "F3",
                        "paper_type": "empirical",
                        "topic": "my-topic",
                        "cwd": str(root),
                    },
                )

        preview = result["structuredContent"]["data"]["task_run_preview"]
        task_packet = result["structuredContent"]["data"]["task_packet"]
        self.assertEqual(task_packet["project_subject"]["effective_subject"], "finance")
        self.assertEqual(task_packet["project_subject"]["domain"], "finance")
        self.assertEqual(task_packet["domain"], "finance")
        self.assertEqual(preview["effective_domain"], "finance")
        self.assertEqual(stub.loaded_domain, "finance")

    def test_task_run_tool_can_launch_agents_when_explicitly_enabled(self) -> None:
        class StubResult:
            mode = "task-run"
            confidence = 0.95
            merged_analysis = "run ok"
            recommendations: list[str] = []
            data = {"runtime_plan": {"draft": "codex", "review": "claude"}}

        class StubOrchestrator:
            def task_run(self, **kwargs: object) -> StubResult:
                self.kwargs = kwargs
                return StubResult()

        stub = StubOrchestrator()
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            with mock.patch.object(tool_handlers, "ModelOrchestrator", return_value=stub):
                result = call_qiongli_tool(
                    "qiongli_task_run",
                    {
                        "cwd": str(root),
                        "task_id": "F3",
                        "paper_type": "empirical",
                        "topic": "my-topic",
                        "execution_mode": "duo",
                        "controller": "codex",
                        "primary": "codex",
                        "reviewer": "claude",
                        "run_agents": True,
                    },
                )

        payload = result["structuredContent"]
        self.assertEqual(payload["mode"], "task-run")
        self.assertTrue(payload["run_agents"])
        self.assertEqual(stub.kwargs["execution_mode"], "duo")
        self.assertEqual(stub.kwargs["controller"], "codex")

    def test_task_run_tool_passes_bounded_runtime_options(self) -> None:
        class StubResult:
            mode = "task-run"
            confidence = 0.95
            merged_analysis = "run ok"
            recommendations: list[str] = []
            data = {"runtime_plan": {"draft": "codex", "review": "codex"}}

        class StubOrchestrator:
            def task_run(self, **kwargs: object) -> StubResult:
                self.kwargs = kwargs
                return StubResult()

        stub = StubOrchestrator()
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            with mock.patch.object(tool_handlers, "ModelOrchestrator", return_value=stub):
                result = call_qiongli_tool(
                    "qiongli_task_run",
                    {
                        "cwd": str(root),
                        "task_id": "C1",
                        "paper_type": "empirical",
                        "topic": "smoke topic",
                        "run_agents": True,
                        "max_revision_rounds": 0,
                        "output_budget": 1,
                        "skip_validation": True,
                    },
                )

        self.assertFalse(result["isError"])
        self.assertEqual(stub.kwargs["max_revision_rounds"], 0)
        self.assertEqual(stub.kwargs["output_budget"], 1)
        self.assertIs(stub.kwargs["skip_validation"], True)

    def test_task_run_tool_maps_triad_execution_mode_to_triad_flag(self) -> None:
        class StubResult:
            mode = "task-run"
            confidence = 0.95
            merged_analysis = "run ok"
            recommendations: list[str] = []
            data = {"runtime_plan": {"draft": "codex", "review": "claude"}}

        class StubOrchestrator:
            def task_run(self, **kwargs: object) -> StubResult:
                self.kwargs = kwargs
                return StubResult()

        stub = StubOrchestrator()
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            with mock.patch.object(tool_handlers, "ModelOrchestrator", return_value=stub):
                result = call_qiongli_tool(
                    "qiongli_task_run",
                    {
                        "cwd": str(root),
                        "task_id": "F3",
                        "paper_type": "empirical",
                        "topic": "my-topic",
                        "execution_mode": "triad",
                        "controller": "codex",
                        "primary": "codex",
                        "reviewer": "claude",
                        "verifier": "antigravity",
                        "run_agents": True,
                    },
                )

        self.assertFalse(result["isError"])
        self.assertIs(stub.kwargs["triad"], True)
        self.assertEqual(stub.kwargs["verifier_agent"], "antigravity")

    def test_task_run_tool_explicit_false_triad_overrides_triad_execution_mode(self) -> None:
        class StubResult:
            mode = "task-run"
            confidence = 0.95
            merged_analysis = "run ok"
            recommendations: list[str] = []
            data = {"runtime_plan": {"draft": "codex", "review": "claude"}}

        class StubOrchestrator:
            def task_run(self, **kwargs: object) -> StubResult:
                self.kwargs = kwargs
                return StubResult()

        stub = StubOrchestrator()
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            with mock.patch.object(tool_handlers, "ModelOrchestrator", return_value=stub):
                result = call_qiongli_tool(
                    "qiongli_task_run",
                    {
                        "cwd": str(root),
                        "task_id": "F3",
                        "paper_type": "empirical",
                        "topic": "my-topic",
                        "execution_mode": "triad",
                        "triad": False,
                        "controller": "codex",
                        "primary": "codex",
                        "reviewer": "claude",
                        "verifier": "antigravity",
                        "run_agents": True,
                    },
                )

        self.assertFalse(result["isError"])
        self.assertIs(stub.kwargs["triad"], False)
        self.assertEqual(stub.kwargs["verifier_agent"], "antigravity")


if __name__ == "__main__":
    unittest.main()
