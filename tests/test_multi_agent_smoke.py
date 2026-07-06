from __future__ import annotations

import sys
import types
import unittest
from datetime import datetime, timezone
from pathlib import Path
from unittest import mock

try:
    import yaml as _yaml  # noqa: F401
except ModuleNotFoundError:
    yaml_stub = types.SimpleNamespace(
        safe_load=lambda *_args, **_kwargs: {},
        safe_dump=lambda *_args, **_kwargs: "",
        YAMLError=ValueError,
    )
    sys.modules["yaml"] = yaml_stub

from qiongli.multi_agent_smoke import (
    FAIL,
    PASS,
    WARN,
    MultiAgentSmokeRunner,
    SmokeCaseResult,
    SmokeReport,
    build_default_report_paths,
    build_parser,
    evaluate_doctor_output,
    overall_status_from_cases,
    render_report_markdown,
)


class MultiAgentSmokeTests(unittest.TestCase):
    def test_overall_status_prioritizes_fail_then_warn(self) -> None:
        self.assertEqual(overall_status_from_cases([SmokeCaseResult("a", PASS, "", 0.1)]), PASS)
        self.assertEqual(
            overall_status_from_cases(
                [SmokeCaseResult("a", PASS, "", 0.1), SmokeCaseResult("b", WARN, "", 0.1)]
            ),
            WARN,
        )
        self.assertEqual(
            overall_status_from_cases(
                [SmokeCaseResult("a", WARN, "", 0.1), SmokeCaseResult("b", FAIL, "", 0.1)]
            ),
            FAIL,
        )

    def test_build_default_report_paths_uses_output_test_runtime(self) -> None:
        json_path, md_path = build_default_report_paths(
            Path("/tmp/repo"),
            datetime(2026, 4, 11, 12, 0, tzinfo=timezone.utc),
        )
        self.assertEqual(json_path, Path("/tmp/repo/output/test_runtime/multi_agent_smoke_20260411T120000Z.json"))
        self.assertEqual(md_path, Path("/tmp/repo/output/test_runtime/multi_agent_smoke_20260411T120000Z.md"))

    def test_evaluate_doctor_output_requires_expected_markers(self) -> None:
        merged = "\n".join(
            [
                "- [OK] Working directory: /tmp/repo",
                "- [OK] CLI codex: /tmp/codex",
                "- [OK] CLI claude: /tmp/claude",
                "- [OK] Env OPENAI_API_KEY: configured",
                "- [OK] Env ANTHROPIC_API_KEY: configured",
            ]
        )
        status, detail = evaluate_doctor_output(
            merged,
            codex_required=True,
            claude_required=True,
            antigravity_required=False,
            codex_auth_ready=False,
            claude_auth_ready=False,
            antigravity_auth_ready=False,
        )
        self.assertEqual(status, PASS)
        self.assertIn("expected readiness markers", detail)

    def test_evaluate_doctor_output_reports_missing_claude_auth(self) -> None:
        merged = "\n".join(
            [
                "- [OK] Working directory: /tmp/repo",
                "- [OK] CLI codex: /tmp/codex",
                "- [OK] CLI claude: /tmp/claude",
                "- [OK] Env OPENAI_API_KEY: configured",
            ]
        )
        status, detail = evaluate_doctor_output(
            merged,
            codex_required=True,
            claude_required=True,
            antigravity_required=False,
            codex_auth_ready=False,
            claude_auth_ready=False,
            antigravity_auth_ready=False,
        )
        self.assertEqual(status, FAIL)
        self.assertIn("ANTHROPIC_API_KEY not configured", detail)

    def test_render_report_markdown_includes_case_lines(self) -> None:
        report = SmokeReport(
            generated_at="2026-04-11T12:00:00+00:00",
            cwd="/tmp/repo",
            topic="demo",
            codex_required=True,
            claude_required=True,
            antigravity_required=False,
            environment={
                "codex_cli": True,
                "codex_auth_ready": True,
                "codex_auth_detail": "Logged in using ChatGPT",
                "claude_cli": True,
                "claude_auth_ready": True,
                "claude_auth_detail": "ANTHROPIC_API_KEY configured",
                "antigravity_cli": False,
                "antigravity_auth_ready": False,
                "antigravity_auth_detail": "antigravity CLI not found in PATH",
                "openai_api_key": True,
                "anthropic_api_key": True,
            },
            cases=[
                SmokeCaseResult("doctor", PASS, "doctor ok", 0.1),
                SmokeCaseResult("claude_runtime", WARN, "claude warn", 0.2),
            ],
        )
        markdown = render_report_markdown(report)
        self.assertIn("# Multi-Agent Smoke Report", markdown)
        self.assertIn("`PASS` doctor: doctor ok", markdown)
        self.assertIn("`WARN` claude_runtime: claude warn", markdown)

    def test_parallel_smoke_requires_environment_opt_in(self) -> None:
        args = build_parser().parse_args(["--run-parallel"])
        runner = MultiAgentSmokeRunner(args)
        skip_detail = "QIONGLI_SMOKE_RUN_AGENTS=1"

        def fail_if_parallel_called() -> tuple[str, str, dict[str, object]]:
            raise AssertionError("parallel runtime case should not launch")

        with mock.patch.dict("os.environ", {}, clear=True):
            with mock.patch.object(runner, "_write_reports"):
                with mock.patch.object(
                    runner,
                    "_case_doctor",
                    return_value=(PASS, "doctor ok", {}),
                ):
                    with mock.patch.object(
                        runner,
                        "_case_codex_runtime",
                        return_value=(PASS, "codex ok", {}),
                    ):
                        with mock.patch.object(
                            runner,
                            "_case_claude_runtime",
                            return_value=(PASS, "claude ok", {}),
                        ):
                            with mock.patch.object(
                                runner,
                                "_case_antigravity_runtime",
                                return_value=(WARN, "antigravity skipped", {}),
                            ):
                                with mock.patch.object(
                                    runner,
                                    "_case_parallel_codex_claude_antigravity",
                                    side_effect=fail_if_parallel_called,
                                ):
                                    report = runner.run()

        parallel = report.cases[-1]
        self.assertEqual(parallel.name, "parallel_codex_claude_antigravity")
        self.assertEqual(parallel.status, WARN)
        self.assertIn(skip_detail, parallel.detail)

    def test_parallel_smoke_runs_when_environment_opt_in_is_present(self) -> None:
        args = build_parser().parse_args(["--run-parallel"])
        runner = MultiAgentSmokeRunner(args)

        with mock.patch.dict("os.environ", {"QIONGLI_SMOKE_RUN_AGENTS": "1"}, clear=True):
            with mock.patch.object(runner, "_write_reports"):
                with mock.patch.object(
                    runner,
                    "_case_doctor",
                    return_value=(PASS, "doctor ok", {}),
                ):
                    with mock.patch.object(
                        runner,
                        "_case_codex_runtime",
                        return_value=(PASS, "codex ok", {}),
                    ):
                        with mock.patch.object(
                            runner,
                            "_case_claude_runtime",
                            return_value=(PASS, "claude ok", {}),
                        ):
                            with mock.patch.object(
                                runner,
                                "_case_antigravity_runtime",
                                return_value=(WARN, "antigravity skipped", {}),
                            ):
                                with mock.patch.object(
                                    runner,
                                    "_case_parallel_codex_claude_antigravity",
                                    return_value=(PASS, "parallel ok", {}),
                                ) as parallel_case:
                                    report = runner.run()

        parallel = report.cases[-1]
        self.assertEqual(parallel.name, "parallel_codex_claude_antigravity")
        self.assertEqual(parallel.status, PASS)
        parallel_case.assert_called_once_with()


if __name__ == "__main__":
    unittest.main()
