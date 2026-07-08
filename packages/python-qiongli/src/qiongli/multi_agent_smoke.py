from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
import time
from dataclasses import asdict, dataclass, field
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from qiongli.source_layout import RepoLayout


REPO_ROOT = Path(__file__).resolve().parents[1]
DEFAULT_TIMEOUT_SECONDS = 60.0
LOCAL_AGENT_ENV = "QIONGLI_SMOKE_RUN_AGENTS"
PASS = "PASS"
WARN = "WARN"
FAIL = "FAIL"


@dataclass
class SmokeCaseResult:
    name: str
    status: str
    detail: str
    duration_seconds: float
    data: dict[str, Any] = field(default_factory=dict)


@dataclass
class SmokeReport:
    generated_at: str
    cwd: str
    topic: str
    codex_required: bool
    claude_required: bool
    antigravity_required: bool
    cases: list[SmokeCaseResult] = field(default_factory=list)
    environment: dict[str, Any] = field(default_factory=dict)
    outputs: dict[str, str] = field(default_factory=dict)

    @property
    def overall_status(self) -> str:
        return overall_status_from_cases(self.cases)

    def to_dict(self) -> dict[str, Any]:
        return {
            "generated_at": self.generated_at,
            "cwd": self.cwd,
            "topic": self.topic,
            "codex_required": self.codex_required,
            "claude_required": self.claude_required,
            "antigravity_required": self.antigravity_required,
            "overall_status": self.overall_status,
            "environment": self.environment,
            "outputs": self.outputs,
            "cases": [asdict(item) for item in self.cases],
        }


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Run a local Codex/Claude/Antigravity multi-agent smoke harness.",
    )
    parser.add_argument("--cwd", default=str(REPO_ROOT), help="Target working directory.")
    parser.add_argument("--topic", default="multi-agent-smoke", help="Topic label stored in the report.")
    parser.add_argument(
        "--timeout-seconds",
        type=float,
        default=DEFAULT_TIMEOUT_SECONDS,
        help="Per-runtime timeout for Codex, Claude, and Antigravity prompt probes.",
    )
    parser.add_argument(
        "--run-parallel",
        action="store_true",
        help="Also run an orchestrator parallel smoke with Codex, Claude, and Antigravity.",
    )
    parser.add_argument(
        "--strict-claude",
        action="store_true",
        help="Fail the run when Claude is unavailable instead of recording a warning.",
    )
    parser.add_argument(
        "--strict-antigravity",
        action="store_true",
        help="Fail the run when Antigravity is unavailable instead of recording a warning.",
    )
    parser.add_argument(
        "--json-report",
        default="",
        help="Optional explicit JSON report path. Defaults to output/test_runtime/.",
    )
    parser.add_argument(
        "--md-report",
        default="",
        help="Optional explicit markdown report path. Defaults to output/test_runtime/.",
    )
    return parser


def overall_status_from_cases(cases: list[SmokeCaseResult]) -> str:
    if any(case.status == FAIL for case in cases):
        return FAIL
    if any(case.status == WARN for case in cases):
        return WARN
    return PASS


def build_default_report_paths(root: Path, generated_at: datetime) -> tuple[Path, Path]:
    stamp = generated_at.strftime("%Y%m%dT%H%M%SZ")
    out_dir = root / "output" / "test_runtime"
    return (
        out_dir / f"multi_agent_smoke_{stamp}.json",
        out_dir / f"multi_agent_smoke_{stamp}.md",
    )


def render_report_markdown(report: SmokeReport) -> str:
    lines = [
        "# Multi-Agent Smoke Report",
        "",
        f"- Generated at: {report.generated_at}",
        f"- Working directory: `{report.cwd}`",
        f"- Topic: `{report.topic}`",
        f"- Overall status: `{report.overall_status}`",
        "",
        "## Environment",
        "",
        f"- Codex CLI in PATH: `{report.environment.get('codex_cli', False)}`",
        f"- Codex auth ready: `{report.environment.get('codex_auth_ready', False)}`",
        f"- Codex auth detail: `{report.environment.get('codex_auth_detail', '')}`",
        f"- Claude CLI in PATH: `{report.environment.get('claude_cli', False)}`",
        f"- Claude auth ready: `{report.environment.get('claude_auth_ready', False)}`",
        f"- Claude auth detail: `{report.environment.get('claude_auth_detail', '')}`",
        f"- Antigravity CLI in PATH: `{report.environment.get('antigravity_cli', False)}`",
        f"- Antigravity auth ready: `{report.environment.get('antigravity_auth_ready', False)}`",
        f"- Antigravity auth detail: `{report.environment.get('antigravity_auth_detail', '')}`",
        f"- OPENAI_API_KEY set: `{report.environment.get('openai_api_key', False)}`",
        f"- ANTHROPIC_API_KEY set: `{report.environment.get('anthropic_api_key', False)}`",
        "",
        "## Cases",
        "",
    ]
    for case in report.cases:
        lines.append(f"- `{case.status}` {case.name}: {case.detail}")
    return "\n".join(lines) + "\n"


def codex_auth_status(timeout_seconds: float = 5.0) -> tuple[bool, str]:
    if os.environ.get("OPENAI_API_KEY", "").strip():
        return True, "OPENAI_API_KEY configured"
    if not shutil.which("codex"):
        return False, "codex CLI not found in PATH"
    try:
        completed = subprocess.run(
            ["codex", "login", "status"],
            capture_output=True,
            text=True,
            timeout=timeout_seconds,
            check=False,
        )
    except (OSError, subprocess.TimeoutExpired) as exc:
        return False, f"codex login status failed: {exc}"
    combined = "\n".join(
        part.strip()
        for part in (completed.stdout or "", completed.stderr or "")
        if part.strip()
    )
    if completed.returncode == 0 and "Logged in" in combined:
        return True, combined.splitlines()[-1]
    detail = combined.splitlines()[-1] if combined.strip() else "codex login status did not confirm authentication"
    return False, detail


def claude_auth_status() -> tuple[bool, str]:
    if os.environ.get("ANTHROPIC_API_KEY", "").strip():
        return True, "ANTHROPIC_API_KEY configured"
    if not shutil.which("claude"):
        return False, "claude CLI not found in PATH"
    return True, "claude CLI found; API key status not checked by smoke harness"


def antigravity_auth_status() -> tuple[bool, str]:
    if not shutil.which("antigravity"):
        return False, "antigravity CLI not found in PATH"
    return True, "antigravity CLI found; local login/configuration is managed by Antigravity"


def evaluate_doctor_output(
    merged_analysis: str,
    *,
    codex_required: bool,
    claude_required: bool,
    antigravity_required: bool,
    codex_auth_ready: bool,
    claude_auth_ready: bool,
    antigravity_auth_ready: bool,
) -> tuple[str, str]:
    text = merged_analysis
    missing: list[str] = []
    if "Working directory" not in text:
        missing.append("Working directory check missing")
    if codex_required:
        if "[OK] CLI codex:" not in text:
            missing.append("CLI codex not OK")
        if not codex_auth_ready and "[OK] Env OPENAI_API_KEY: configured" not in text:
            missing.append("OPENAI_API_KEY not configured")
    if claude_required:
        if "[OK] CLI claude:" not in text:
            missing.append("CLI claude not OK")
        if not claude_auth_ready and "[OK] Env ANTHROPIC_API_KEY: configured" not in text:
            missing.append("ANTHROPIC_API_KEY not configured")
    if antigravity_required:
        if "[OK] CLI antigravity:" not in text:
            missing.append("CLI antigravity not OK")
        if not antigravity_auth_ready and "[OK] Auth antigravity:" not in text:
            missing.append("Antigravity local CLI auth/configuration not ready")
    if missing:
        return FAIL, "; ".join(missing)
    return PASS, "doctor output contained all expected readiness markers"


class MultiAgentSmokeRunner:
    def __init__(self, args: argparse.Namespace) -> None:
        self.args = args
        self.cwd = Path(args.cwd).resolve()
        self.timeout_seconds = float(args.timeout_seconds)
        now = datetime.now(timezone.utc).replace(microsecond=0)
        json_path, md_path = build_default_report_paths(REPO_ROOT, now)
        if args.json_report:
            json_path = Path(args.json_report).resolve()
        if args.md_report:
            md_path = Path(args.md_report).resolve()
        self.report = SmokeReport(
            generated_at=now.isoformat(),
            cwd=str(self.cwd),
            topic=str(args.topic),
            codex_required=True,
            claude_required=True,
            antigravity_required=bool(args.strict_antigravity),
            outputs={
                "json_report": str(json_path),
                "markdown_report": str(md_path),
            },
        )

    def run(self) -> SmokeReport:
        self._snapshot_environment()
        try:
            self._run_case("doctor", self._case_doctor)
            self._run_case("codex_runtime", self._case_codex_runtime)
            self._run_case("claude_runtime", self._case_claude_runtime)
            self._run_case("antigravity_runtime", self._case_antigravity_runtime)
            if self.args.run_parallel:
                if os.environ.get(LOCAL_AGENT_ENV) == "1":
                    self._run_case(
                        "parallel_codex_claude_antigravity",
                        self._case_parallel_codex_claude_antigravity,
                    )
                else:
                    self._run_case(
                        "parallel_codex_claude_antigravity",
                        self._case_parallel_opt_in_missing,
                    )
        finally:
            self._write_reports()
        return self.report

    def _snapshot_environment(self) -> None:
        codex_auth_ready, codex_auth_detail = codex_auth_status()
        claude_auth_ready, claude_auth_detail = claude_auth_status()
        antigravity_auth_ready, antigravity_auth_detail = antigravity_auth_status()
        self.report.environment = {
            "codex_cli": bool(shutil.which("codex")),
            "codex_auth_ready": codex_auth_ready,
            "codex_auth_detail": codex_auth_detail,
            "claude_cli": bool(shutil.which("claude")),
            "claude_auth_ready": claude_auth_ready,
            "claude_auth_detail": claude_auth_detail,
            "antigravity_cli": bool(shutil.which("antigravity")),
            "antigravity_auth_ready": antigravity_auth_ready,
            "antigravity_auth_detail": antigravity_auth_detail,
            "openai_api_key": bool(os.environ.get("OPENAI_API_KEY", "").strip()),
            "anthropic_api_key": bool(os.environ.get("ANTHROPIC_API_KEY", "").strip()),
        }

    def _run_case(self, name: str, fn) -> None:
        started_at = time.monotonic()
        try:
            status, detail, data = fn()
        except Exception as exc:
            status = FAIL
            detail = f"{type(exc).__name__}: {exc}"
            data = {}
        duration = time.monotonic() - started_at
        self.report.cases.append(
            SmokeCaseResult(
                name=name,
                status=status,
                detail=detail,
                duration_seconds=round(duration, 3),
                data=data,
            )
        )

    def _case_doctor(self) -> tuple[str, str, dict[str, Any]]:
        orchestrator = self._make_orchestrator()
        result = orchestrator.doctor(self.cwd)
        status, detail = evaluate_doctor_output(
            result.merged_analysis,
            codex_required=True,
            claude_required=True,
            antigravity_required=bool(self.args.strict_antigravity),
            codex_auth_ready=bool(self.report.environment.get("codex_auth_ready")),
            claude_auth_ready=bool(self.report.environment.get("claude_auth_ready")),
            antigravity_auth_ready=bool(self.report.environment.get("antigravity_auth_ready")),
        )
        return status, detail, {
            "confidence": result.confidence,
            "merged_analysis": result.merged_analysis,
        }

    def _case_codex_runtime(self) -> tuple[str, str, dict[str, Any]]:
        orchestrator = self._make_orchestrator()
        response = orchestrator._execute_runtime_agent(
            "codex",
            "Return only the token CODEX_SMOKE_OK.",
            self.cwd,
            {"non_interactive": True, "timeout_seconds": self.timeout_seconds},
        )
        if response.success and "CODEX_SMOKE_OK" in response.content:
            return PASS, "Codex runtime returned the smoke token.", {"content": response.content}
        return FAIL, response.error or "Codex runtime did not return the smoke token.", {
            "content": response.content,
            "error": response.error,
        }

    def _case_claude_runtime(self) -> tuple[str, str, dict[str, Any]]:
        orchestrator = self._make_orchestrator()
        response = orchestrator._execute_runtime_agent(
            "claude",
            "Return only the token CLAUDE_SMOKE_OK.",
            self.cwd,
            {"non_interactive": True, "timeout_seconds": self.timeout_seconds},
        )
        if response.success and "CLAUDE_SMOKE_OK" in response.content:
            return PASS, "Claude runtime returned the smoke token.", {"content": response.content}
        status = FAIL if self.args.strict_claude else WARN
        return status, response.error or "Claude runtime did not return the smoke token.", {
            "content": response.content,
            "error": response.error,
        }

    def _case_antigravity_runtime(self) -> tuple[str, str, dict[str, Any]]:
        orchestrator = self._make_orchestrator()
        response = orchestrator._execute_runtime_agent(
            "antigravity",
            "Return only the token ANTIGRAVITY_SMOKE_OK.",
            self.cwd,
            {"non_interactive": True, "timeout_seconds": self.timeout_seconds},
        )
        if response.success and "ANTIGRAVITY_SMOKE_OK" in response.content:
            return PASS, "Antigravity runtime returned the smoke token.", {
                "content": response.content
            }
        status = FAIL if self.args.strict_antigravity else WARN
        return status, response.error or "Antigravity runtime did not return the smoke token.", {
            "content": response.content,
            "error": response.error,
        }

    def _case_parallel_opt_in_missing(self) -> tuple[str, str, dict[str, Any]]:
        return (
            WARN,
            (
                "parallel runtime smoke skipped; set "
                f"{LOCAL_AGENT_ENV}=1 with --run-parallel to launch local agents"
            ),
            {"required_env": LOCAL_AGENT_ENV, "run_parallel": True},
        )

    def _case_parallel_codex_claude_antigravity(self) -> tuple[str, str, dict[str, Any]]:
        from bridges.orchestrator import CollaborationMode

        orchestrator = self._make_orchestrator()
        result = orchestrator.execute(
            mode=CollaborationMode.PARALLEL,
            cwd=self.cwd,
            prompt="Return one short analysis sentence mentioning the token PARALLEL_SMOKE_OK.",
            parallel_summarizer="codex",
            profile_file=self._smoke_profile_file(),
            profile="smoke-codex-claude-antigravity",
            summarizer_profile="smoke-codex-claude-antigravity",
        )
        codex_ok = bool(result.codex_response and result.codex_response.success)
        claude_ok = bool(result.claude_response and result.claude_response.success)
        antigravity_ok = bool(
            result.antigravity_response and result.antigravity_response.success
        )
        if codex_ok and claude_ok and antigravity_ok:
            detail = (
                "Parallel mode succeeded with Codex, Claude, and Antigravity "
                "participation."
            )
            return PASS, detail, {
                "merged_analysis": result.merged_analysis,
            }
        if codex_ok and not claude_ok:
            detail = (
                "Parallel mode succeeded for Codex, but Claude did not participate "
                "successfully."
            )
            status = FAIL if self.args.strict_claude else WARN
            return status, detail, {"merged_analysis": result.merged_analysis}
        if codex_ok and claude_ok and not antigravity_ok:
            detail = (
                "Parallel mode succeeded for Codex and Claude, but Antigravity did not "
                "participate successfully."
            )
            status = FAIL if self.args.strict_antigravity else WARN
            return status, detail, {"merged_analysis": result.merged_analysis}
        return FAIL, "Parallel mode did not complete with Codex as the primary execution path.", {
            "merged_analysis": result.merged_analysis,
        }

    def _make_orchestrator(self):
        from bridges.orchestrator import ModelOrchestrator

        return ModelOrchestrator(standards_dir=RepoLayout(REPO_ROOT).standards)

    def _smoke_profile_file(self) -> Path:
        out_dir = Path(self.report.outputs["json_report"]).resolve().parent
        out_dir.mkdir(parents=True, exist_ok=True)
        profile_path = out_dir / "multi_agent_smoke_profile.json"
        payload = {
            "profiles": {
                "smoke-codex-claude-antigravity": {
                    "persona": "Smoke test profile for local Codex/Claude/Antigravity validation.",
                    "analysis_style": "Return short direct answers for smoke validation.",
                    "summary_style": "Return short direct answers for smoke validation.",
                    "runtime_options": {
                        "codex": {
                            "non_interactive": True,
                            "timeout_seconds": self.timeout_seconds,
                        },
                        "claude": {
                            "non_interactive": True,
                            "timeout_seconds": self.timeout_seconds,
                        },
                        "antigravity": {
                            "non_interactive": True,
                            "timeout_seconds": self.timeout_seconds,
                            "sandbox": True,
                        },
                    },
                }
            }
        }
        profile_path.write_text(json.dumps(payload, indent=2, ensure_ascii=False), encoding="utf-8")
        self.report.outputs["profile_file"] = str(profile_path)
        return profile_path

    def _write_reports(self) -> None:
        json_path = Path(self.report.outputs["json_report"]).resolve()
        md_path = Path(self.report.outputs["markdown_report"]).resolve()
        json_path.parent.mkdir(parents=True, exist_ok=True)
        md_path.parent.mkdir(parents=True, exist_ok=True)
        json_path.write_text(
            json.dumps(self.report.to_dict(), indent=2, ensure_ascii=False),
            encoding="utf-8",
        )
        md_path.write_text(render_report_markdown(self.report), encoding="utf-8")


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    runner = MultiAgentSmokeRunner(args)
    report = runner.run()
    print(f"[multi-agent-smoke] {report.overall_status}")
    print(f"[multi-agent-smoke] json: {report.outputs['json_report']}")
    print(f"[multi-agent-smoke] md: {report.outputs['markdown_report']}")
    return 0 if report.overall_status != FAIL else 1


if __name__ == "__main__":
    raise SystemExit(main())
