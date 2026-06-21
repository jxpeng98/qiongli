from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path
from unittest import mock

import bridges.mcp_tool_handlers as tool_handlers
from bridges.mcp_tool_handlers import MCP_TOOL_DEFINITIONS, call_qiongli_tool
from bridges.provider_config import set_provider_value


class MCPToolHandlerTests(unittest.TestCase):
    def test_tool_definitions_include_config_and_evidence_tools(self) -> None:
        names = {tool["name"] for tool in MCP_TOOL_DEFINITIONS}

        self.assertTrue(
            {
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
            }.issubset(names)
        )

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
                    "primary_agent": "gemini",
                    "review_agent": "gemini",
                    "fallback_agent": "gemini",
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
                    "fallback_agent": "gemini",
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
                    "fallback_agent": "gemini",
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
            self.assertFalse((root / ".qiongli").exists())

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
                    "fallback_agent": "gemini",
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
                    "verifier": "gemini",
                },
            )

        self.assertFalse(result["isError"])
        preview = result["structuredContent"]["data"]["task_run_preview"]
        self.assertIs(stub.controller_kwargs["triad"], True)
        self.assertEqual(preview["controller_metadata"]["triad"], "True")

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
                    "fallback_agent": "gemini",
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
                    "fallback_agent": "gemini",
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
                        "verifier": "gemini",
                        "run_agents": True,
                    },
                )

        self.assertFalse(result["isError"])
        self.assertIs(stub.kwargs["triad"], True)

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
                        "verifier": "gemini",
                        "run_agents": True,
                    },
                )

        self.assertFalse(result["isError"])
        self.assertIs(stub.kwargs["triad"], False)


if __name__ == "__main__":
    unittest.main()
