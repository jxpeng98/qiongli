from __future__ import annotations

import json
import os
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tooling.scripts import run_subject_runtime_smoke as smoke
from tooling.scripts.run_subject_runtime_smoke import (
    FIXTURE_DIR,
    SmokeCase,
    _assert_case,
    load_smoke_cases,
    run_smoke_suite,
)


class SubjectRuntimeSmokeTests(unittest.TestCase):
    def test_load_smoke_cases_reads_all_fixtures(self) -> None:
        cases = load_smoke_cases(FIXTURE_DIR)

        names = {case.name for case in cases}
        self.assertEqual(
            names,
            {
                "no_subject_core_only",
                "borrow_finance_lens",
                "suggest_finance_subject",
                "locked_economics_borrow_finance",
                "confirmed_finance_guidance_loaded",
            },
        )
        self.assertEqual([case.name for case in cases], sorted(names))
        self.assertTrue(all(isinstance(case, SmokeCase) for case in cases))

    def test_confirmed_fixture_declares_setup_subject_action(self) -> None:
        cases = load_smoke_cases(FIXTURE_DIR)
        case = next(
            item for item in cases if item.name == "confirmed_finance_guidance_loaded"
        )

        self.assertEqual(
            case.setup_subject_action,
            {
                "action": "confirm",
                "subject": "finance",
                "run_id": "setup-confirm-finance",
            },
        )

    def test_preview_suite_passes_and_writes_inside_isolated_project(self) -> None:
        real_home = os.environ.get("HOME")
        real_lang = os.environ.get("RESEARCH_CLI_LANG")
        with tempfile.TemporaryDirectory() as tmp_dir:
            workspace_root = Path(tmp_dir).resolve()
            report = run_smoke_suite(
                fixture_dir=FIXTURE_DIR,
                workspace_root=workspace_root,
                mode="preview",
                selected_cases=[],
            )

            self.assertEqual(report["summary"]["failed"], 0)
            self.assertEqual(report["summary"]["passed"], 5)
            self.assertEqual(report["mode"], "preview")
            for case in report["cases"]:
                project_root = Path(case["project_root"]).resolve()
                self.assertTrue(project_root.is_relative_to(workspace_root))
                self.assertFalse(case["result"]["run_agents"])
                self.assertEqual(case["status"], "passed")
                case_env = case["environment"]
                self.assertEqual(case_env["RESEARCH_CLI_LANG"], "en")
                self.assertEqual(
                    Path(case_env["HOME"]).resolve(),
                    project_root / ".smoke-home" / "home",
                )
                for key in {
                    "QIONGLI_GUIDANCE_HOME",
                    "QIONGLI_CONFIG_HOME",
                    "CODEX_HOME",
                    "XDG_CONFIG_HOME",
                    "HOME",
                }:
                    self.assertTrue(Path(case_env[key]).resolve().is_relative_to(project_root))

            self.assertEqual(os.environ.get("HOME"), real_home)
            self.assertEqual(os.environ.get("RESEARCH_CLI_LANG"), real_lang)

            created_names = {path.name for path in workspace_root.iterdir()}
            self.assertEqual(
                created_names,
                {
                    "no_subject_core_only",
                    "borrow_finance_lens",
                    "suggest_finance_subject",
                    "locked_economics_borrow_finance",
                    "confirmed_finance_guidance_loaded",
                },
            )

    def test_report_is_json_serializable_and_selected_cases_work(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            report = run_smoke_suite(
                fixture_dir=FIXTURE_DIR,
                workspace_root=Path(tmp_dir),
                mode="preview",
                selected_cases=["suggest_finance_subject"],
            )

        encoded = json.dumps(report, sort_keys=True)
        decoded = json.loads(encoded)
        self.assertEqual(decoded["summary"]["total"], 1)
        self.assertEqual(decoded["summary"]["failed"], 0)
        self.assertEqual(decoded["cases"][0]["name"], "suggest_finance_subject")

    def test_report_schema_version_is_1_1(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            report = run_smoke_suite(
                fixture_dir=FIXTURE_DIR,
                workspace_root=Path(tmp_dir),
                mode="preview",
                selected_cases=["suggest_finance_subject"],
            )

        self.assertEqual(report["schema_version"], "1.1")

    def test_empty_fixture_dir_raises_instead_of_green_zero_case_report(self) -> None:
        with tempfile.TemporaryDirectory() as fixture_dir:
            with tempfile.TemporaryDirectory() as workspace_dir:
                with self.assertRaises(ValueError) as raised:
                    run_smoke_suite(
                        fixture_dir=Path(fixture_dir),
                        workspace_root=Path(workspace_dir),
                        mode="preview",
                    )

        self.assertIn("no subject runtime smoke cases", str(raised.exception))

    def test_unknown_selected_case_still_raises_clear_error(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            with self.assertRaises(ValueError) as raised:
                run_smoke_suite(
                    fixture_dir=FIXTURE_DIR,
                    workspace_root=Path(tmp_dir),
                    mode="preview",
                    selected_cases=["missing_case"],
                )

        self.assertIn("unknown smoke case(s): missing_case", str(raised.exception))

    def test_local_agent_mode_requires_environment_opt_in(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            with mock.patch.dict("os.environ", {}, clear=True):
                with self.assertRaises(RuntimeError) as raised:
                    run_smoke_suite(
                        fixture_dir=FIXTURE_DIR,
                        workspace_root=Path(tmp_dir),
                        mode="local-agent",
                        selected_cases=["suggest_finance_subject"],
                    )

        self.assertIn("QIONGLI_SMOKE_RUN_AGENTS=1", str(raised.exception))

    def test_local_agent_mode_defaults_to_confirmed_finance_case(self) -> None:
        fake_result = {
            "name": "confirmed_finance_guidance_loaded",
            "source": "tests/fixtures/subject_runtime_smoke/confirmed_finance_guidance_loaded.json",
            "project_root": "/tmp/project",
            "status": "passed",
            "failures": [],
            "environment": {},
            "result": {"run_agents": True, "data": {}},
            "local_agent": {"requested": True, "env_opt_in": True},
            "trace_assertions": {},
            "write_boundary": {"known_paths_inside_project": True, "violations": []},
        }
        with tempfile.TemporaryDirectory() as tmp_dir:
            with mock.patch.dict("os.environ", {smoke.LOCAL_AGENT_ENV: "1"}):
                with mock.patch.object(smoke, "run_smoke_case", return_value=fake_result) as run_case:
                    report = run_smoke_suite(
                        fixture_dir=FIXTURE_DIR,
                        workspace_root=Path(tmp_dir),
                        mode="local-agent",
                        selected_cases=[],
                    )

        self.assertEqual(report["schema_version"], "1.1")
        self.assertEqual(report["summary"], {"total": 1, "passed": 1, "failed": 0})
        self.assertEqual(run_case.call_count, 1)
        selected_case = run_case.call_args.args[0]
        self.assertEqual(selected_case.name, "confirmed_finance_guidance_loaded")

    def test_local_agent_case_uses_bounded_task_arguments(self) -> None:
        captured_calls: list[tuple[str, dict[str, object]]] = []

        def fake_call(name: str, args: dict[str, object]) -> dict[str, object]:
            captured_calls.append((name, dict(args)))
            if name == "qiongli_subject_update":
                return {"structuredContent": {"ok": True}, "isError": False}
            return {
                "structuredContent": {
                    "mode": "task-run",
                    "run_agents": True,
                    "data": {
                        "task_packet": {
                            "local_guidance": {
                                "guidance_files_read": [
                                    ".qiongli/guidance.d/subject-runtime.md"
                                ]
                            },
                            "subject_refinement": {
                                "decision": "confirm_subject",
                                "primary_subject": "finance",
                                "loaded_resources": {
                                    "levels": ["subject_overlay", "subject_skill"]
                                },
                                "signals": [],
                                "resource_activation_plan": {},
                            },
                            "runtime_plan": {
                                "primary_agent": "codex",
                                "review_agent": "codex",
                                "fallback_agent": "codex",
                            },
                            "domain": "finance",
                        },
                        "local_guidance_trace": {
                            "run_dir": ".qiongli/trace/runs/run-1",
                            "trace_index": ".qiongli/trace/index.jsonl",
                            "guidance_files_read": [
                                ".qiongli/guidance.d/subject-runtime.md"
                            ],
                        },
                        "routing_notes": ["Runtime plan: draft=codex, review=codex."],
                    },
                },
                "isError": False,
            }

        case = SmokeCase(
            name="confirmed_finance_guidance_loaded",
            manifest=None,
            args={
                "task_id": "C1",
                "paper_type": "empirical",
                "topic": "earnings announcement stock market reaction",
                "context": "Use event-study evidence and Journal of Finance standards.",
                "domain": "auto",
                "guidance_mode": "propose",
                "run_agents": False,
            },
            expected={
                "run_agents": False,
                "decision": "confirm_subject",
                "primary_subject": "finance",
                "effective_domain": "finance",
                "resource_levels": ["subject_overlay", "subject_skill"],
                "guidance_source": ".qiongli/guidance.d/subject-runtime.md",
            },
            source=Path("confirmed_finance_guidance_loaded.json"),
            setup_subject_action={
                "action": "confirm",
                "subject": "finance",
                "run_id": "setup-confirm-finance",
            },
        )
        with tempfile.TemporaryDirectory() as tmp_dir:
            with mock.patch.object(smoke, "call_qiongli_tool", side_effect=fake_call):
                result = smoke.run_smoke_case(case, Path(tmp_dir), "local-agent")

        self.assertEqual(result["status"], "passed", result["failures"])
        task_call = captured_calls[-1]
        self.assertEqual(task_call[0], "qiongli_task_run")
        task_args = task_call[1]
        self.assertIs(task_args["run_agents"], True)
        self.assertEqual(task_args["max_revision_rounds"], 0)
        self.assertEqual(task_args["output_budget"], 1)
        self.assertIs(task_args["skip_validation"], True)
        self.assertEqual(task_args["execution_mode"], "solo")
        self.assertEqual(task_args["controller"], "codex")
        self.assertEqual(task_args["primary"], "codex")
        self.assertEqual(task_args["reviewer"], "codex")

    def test_local_agent_assertion_requires_guidance_trace(self) -> None:
        case = SmokeCase(
            name="confirmed_finance_guidance_loaded",
            manifest=None,
            args={},
            expected={
                "run_agents": True,
                "decision": "confirm_subject",
                "primary_subject": "finance",
                "effective_domain": "finance",
                "resource_levels": [],
                "guidance_source": ".qiongli/guidance.d/subject-runtime.md",
            },
            source=Path("confirmed_finance_guidance_loaded.json"),
        )
        result = {
            "structuredContent": {
                "run_agents": True,
                "data": {
                    "task_packet": {
                        "local_guidance": {
                            "guidance_files_read": [
                                ".qiongli/guidance.d/subject-runtime.md"
                            ]
                        },
                        "subject_refinement": {
                            "decision": "confirm_subject",
                            "primary_subject": "finance",
                            "loaded_resources": {"levels": []},
                            "signals": [],
                            "resource_activation_plan": {},
                        },
                        "domain": "finance",
                    }
                },
            }
        }

        failures = smoke._assert_case(
            case, result, mode="local-agent", project_root=Path("/tmp/project")
        )

        self.assertIn("missing local guidance trace", failures)

    def test_local_agent_report_includes_runtime_metadata(self) -> None:
        case = next(
            item
            for item in load_smoke_cases(FIXTURE_DIR)
            if item.name == "confirmed_finance_guidance_loaded"
        )

        def fake_call(name: str, args: dict[str, object]) -> dict[str, object]:
            if name == "qiongli_subject_update":
                return {"structuredContent": {"ok": True}, "isError": False}
            return {
                "structuredContent": {
                    "mode": "task-run",
                    "run_agents": True,
                    "data": {
                        "task_packet": {
                            "local_guidance": {
                                "guidance_files_read": [
                                    ".qiongli/guidance.d/subject-runtime.md"
                                ]
                            },
                            "subject_refinement": {
                                "decision": "confirm_subject",
                                "primary_subject": "finance",
                                "loaded_resources": {
                                    "levels": ["subject_overlay", "subject_skill"]
                                },
                                "signals": [],
                                "resource_activation_plan": {},
                            },
                            "runtime_plan": {
                                "primary_agent": "codex",
                                "review_agent": "codex",
                                "fallback_agent": "codex",
                            },
                            "domain": "finance",
                        },
                        "local_guidance_trace": {
                            "run_dir": ".qiongli/trace/runs/run-1",
                            "trace_index": ".qiongli/trace/index.jsonl",
                            "guidance_files_read": [
                                ".qiongli/guidance.d/subject-runtime.md"
                            ],
                        },
                        "routing_notes": ["Runtime plan: draft=codex, review=codex."],
                    },
                },
                "isError": False,
            }

        with tempfile.TemporaryDirectory() as tmp_dir:
            with mock.patch.object(smoke, "call_qiongli_tool", side_effect=fake_call):
                result = smoke.run_smoke_case(case, Path(tmp_dir), "local-agent")

        self.assertEqual(result["status"], "passed", result["failures"])
        self.assertEqual(
            result["local_agent"]["runtime_plan"],
            {"primary_agent": "codex", "review_agent": "codex", "fallback_agent": "codex"},
        )
        self.assertTrue(result["trace_assertions"]["trace_written"])
        self.assertTrue(result["trace_assertions"]["subject_guidance_loaded"])
        self.assertTrue(result["trace_assertions"]["subject_refinement_persisted"])

    def test_local_agent_report_treats_malformed_runtime_plan_as_empty_metadata(
        self,
    ) -> None:
        case = next(
            item
            for item in load_smoke_cases(FIXTURE_DIR)
            if item.name == "confirmed_finance_guidance_loaded"
        )

        def fake_call(name: str, args: dict[str, object]) -> dict[str, object]:
            if name == "qiongli_subject_update":
                return {"structuredContent": {"ok": True}, "isError": False}
            return {
                "structuredContent": {
                    "mode": "task-run",
                    "run_agents": True,
                    "data": {
                        "task_packet": {
                            "local_guidance": {
                                "guidance_files_read": [
                                    ".qiongli/guidance.d/subject-runtime.md"
                                ]
                            },
                            "subject_refinement": {
                                "decision": "confirm_subject",
                                "primary_subject": "finance",
                                "loaded_resources": {
                                    "levels": ["subject_overlay", "subject_skill"]
                                },
                                "signals": [],
                                "resource_activation_plan": {},
                            },
                            "runtime_plan": "malformed-runtime-plan",
                            "domain": "finance",
                        },
                        "local_guidance_trace": {
                            "run_dir": ".qiongli/trace/runs/run-1",
                            "trace_index": ".qiongli/trace/index.jsonl",
                            "guidance_files_read": [
                                ".qiongli/guidance.d/subject-runtime.md"
                            ],
                        },
                    },
                },
                "isError": False,
            }

        with tempfile.TemporaryDirectory() as tmp_dir:
            with mock.patch.object(smoke, "call_qiongli_tool", side_effect=fake_call):
                result = smoke.run_smoke_case(case, Path(tmp_dir), "local-agent")

        self.assertEqual(result["status"], "passed", result["failures"])
        self.assertEqual(result["local_agent"]["runtime_plan"], {})

    def test_local_agent_assertion_treats_scalar_guidance_files_as_empty(
        self,
    ) -> None:
        case = SmokeCase(
            name="confirmed_finance_guidance_loaded",
            manifest=None,
            args={},
            expected={
                "run_agents": True,
                "decision": "confirm_subject",
                "primary_subject": "finance",
                "effective_domain": "finance",
                "resource_levels": [],
                "guidance_source": ".qiongli/guidance.d/subject-runtime.md",
            },
            source=Path("confirmed_finance_guidance_loaded.json"),
        )
        result = {
            "structuredContent": {
                "run_agents": True,
                "data": {
                    "task_packet": {
                        "local_guidance": {"guidance_files_read": 123},
                        "subject_refinement": {
                            "decision": "confirm_subject",
                            "primary_subject": "finance",
                            "loaded_resources": {"levels": []},
                            "signals": [],
                            "resource_activation_plan": {},
                        },
                        "domain": "finance",
                    },
                    "local_guidance_trace": {
                        "run_dir": ".qiongli/trace/runs/run-1",
                        "trace_index": ".qiongli/trace/index.jsonl",
                        "guidance_files_read": 456,
                    },
                },
            }
        }

        failures = smoke._assert_case(
            case, result, mode="local-agent", project_root=Path("/tmp/project")
        )

        self.assertIn(
            "missing guidance source '.qiongli/guidance.d/subject-runtime.md'",
            failures,
        )
        self.assertIn(
            "missing local-agent guidance source '.qiongli/guidance.d/subject-runtime.md'",
            failures,
        )

    def test_write_boundary_detects_outside_trace_path(self) -> None:
        project_root = Path("/tmp/project").resolve()
        payload = {
            "data": {
                "local_guidance_trace": {
                    "run_dir": "/tmp/outside/run-1",
                    "trace_index": ".qiongli/trace/index.jsonl",
                }
            }
        }

        result = smoke._write_boundary_report(payload, project_root)

        self.assertFalse(result["known_paths_inside_project"])
        self.assertTrue(any("/tmp/outside/run-1" in item for item in result["violations"]))

    def test_write_boundary_detects_outside_guidance_proposal_path(self) -> None:
        project_root = Path("/tmp/project").resolve()
        payload = {
            "data": {
                "local_guidance_trace": {
                    "run_dir": ".qiongli/trace/runs/run-1",
                    "trace_index": ".qiongli/trace/index.jsonl",
                    "guidance_proposal": "/tmp/outside/guidance-proposal.json",
                }
            }
        }

        result = smoke._write_boundary_report(payload, project_root)

        self.assertFalse(result["known_paths_inside_project"])
        self.assertTrue(
            any("/tmp/outside/guidance-proposal.json" in item for item in result["violations"])
        )

    def test_error_report_includes_rerun_command(self) -> None:
        error = RuntimeError("local-agent smoke requires QIONGLI_SMOKE_RUN_AGENTS=1")

        report = smoke._error_report("local-agent", error)

        self.assertEqual(report["schema_version"], "1.1")
        self.assertIn("rerun_command", report)
        self.assertIn("QIONGLI_SMOKE_RUN_AGENTS=1", report["rerun_command"])
        self.assertIn("--mode local-agent", report["rerun_command"])

    def test_local_agent_write_boundary_failure_includes_case_rerun_command(self) -> None:
        case = next(
            item
            for item in load_smoke_cases(FIXTURE_DIR)
            if item.name == "confirmed_finance_guidance_loaded"
        )

        def fake_call(name: str, args: dict[str, object]) -> dict[str, object]:
            if name == "qiongli_subject_update":
                return {"structuredContent": {"ok": True}, "isError": False}
            return {
                "structuredContent": {
                    "mode": "task-run",
                    "run_agents": True,
                    "data": {
                        "task_packet": {
                            "local_guidance": {
                                "guidance_files_read": [
                                    ".qiongli/guidance.d/subject-runtime.md"
                                ]
                            },
                            "subject_refinement": {
                                "decision": "confirm_subject",
                                "primary_subject": "finance",
                                "loaded_resources": {
                                    "levels": ["subject_overlay", "subject_skill"]
                                },
                                "signals": [],
                                "resource_activation_plan": {},
                            },
                            "domain": "finance",
                        },
                        "local_guidance_trace": {
                            "run_dir": "/tmp/outside/run-1",
                            "trace_index": ".qiongli/trace/index.jsonl",
                            "guidance_files_read": [
                                ".qiongli/guidance.d/subject-runtime.md"
                            ],
                        },
                    },
                },
                "isError": False,
            }

        with tempfile.TemporaryDirectory() as tmp_dir:
            with mock.patch.object(smoke, "call_qiongli_tool", side_effect=fake_call):
                result = smoke.run_smoke_case(case, Path(tmp_dir), "local-agent")

        self.assertEqual(result["status"], "failed")
        self.assertFalse(result["write_boundary"]["known_paths_inside_project"])
        self.assertTrue(
            any("write boundary violation:" in item for item in result["failures"])
        )
        self.assertIn("rerun_command", result)
        self.assertIn("--mode local-agent", result["rerun_command"])
        self.assertIn("--case confirmed_finance_guidance_loaded", result["rerun_command"])

    def test_local_agent_scalar_payload_reports_failure_diagnostics(self) -> None:
        case = next(
            item
            for item in load_smoke_cases(FIXTURE_DIR)
            if item.name == "confirmed_finance_guidance_loaded"
        )

        def fake_call(name: str, args: dict[str, object]) -> dict[str, object]:
            if name == "qiongli_subject_update":
                return {"structuredContent": {"ok": True}, "isError": False}
            return {"structuredContent": "malformed", "isError": False}

        with tempfile.TemporaryDirectory() as tmp_dir:
            with mock.patch.object(smoke, "call_qiongli_tool", side_effect=fake_call):
                result = smoke.run_smoke_case(case, Path(tmp_dir), "local-agent")

        self.assertEqual(result["status"], "failed")
        self.assertIn("tool returned non-object payload", result["failures"])
        self.assertIn("write_boundary", result)
        self.assertIn("rerun_command", result)
        self.assertIn("--mode local-agent", result["rerun_command"])
        self.assertIn("--case confirmed_finance_guidance_loaded", result["rerun_command"])

    def test_non_no_subject_refinement_requires_packet_v2_fields(self) -> None:
        case = SmokeCase(
            name="suggest_finance_subject",
            manifest=None,
            args={},
            expected={
                "run_agents": False,
                "decision": "suggest_subject",
                "primary_subject": "finance",
                "effective_domain": "finance",
                "resource_levels": [],
            },
            source=Path("suggest_finance_subject.json"),
        )
        result = {
            "structuredContent": {
                "run_agents": False,
                "data": {
                    "task_run_preview": {
                        "effective_domain": "finance",
                        "subject_refinement": {
                            "decision": "suggest_subject",
                            "primary_subject": "finance",
                            "loaded_resources": {"levels": []},
                        },
                    }
                },
            }
        }

        failures = _assert_case(case, result)

        self.assertIn("missing signals ledger", failures)
        self.assertIn("missing resource_activation_plan", failures)

    def test_suggest_finance_case_exposes_packet_v2_fields(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            report = run_smoke_suite(
                fixture_dir=FIXTURE_DIR,
                workspace_root=Path(tmp_dir),
                mode="preview",
                selected_cases=["suggest_finance_subject"],
            )

        refinement = report["cases"][0]["result"]["data"]["task_run_preview"][
            "subject_refinement"
        ]
        self.assertTrue(refinement["signals"])
        self.assertEqual(refinement["resource_activation_plan"]["primary_subject"], "finance")

    def test_confirmed_finance_case_loads_materialized_subject_guidance(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            report = run_smoke_suite(
                fixture_dir=FIXTURE_DIR,
                workspace_root=Path(tmp_dir),
                mode="preview",
                selected_cases=["confirmed_finance_guidance_loaded"],
            )

        case = report["cases"][0]
        self.assertEqual(case["status"], "passed", case["failures"])
        guidance = case["result"]["data"]["task_packet"]["local_guidance"]
        self.assertIn(
            ".qiongli/guidance.d/subject-runtime.md",
            guidance["guidance_files_read"],
        )


if __name__ == "__main__":
    unittest.main()
