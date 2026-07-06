from __future__ import annotations

import json
import os
import subprocess
from dataclasses import asdict, dataclass, field
from pathlib import Path
from typing import Any

from bridges.command_runtime import current_python_command, split_command
from bridges.provider_config import (
    provider_config_env,
    provider_config_summary,
    resolve_provider_config,
)
from qiongli.source_layout import RepoLayout, discover_repo_root


@dataclass
class MCPEvidence:
    provider: str
    status: str
    summary: str
    provenance: list[str] = field(default_factory=list)
    data: dict[str, Any] = field(default_factory=dict)

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)


@dataclass
class MCPProviderResolution:
    provider: str
    env_name: str
    source: str
    command: str | None = None
    native_script: str | None = None


DIRECT_LITERATURE_PROVIDER_KEYS = {
    "openalex",
    "semantic_scholar",
    "crossref",
    "pubmed",
    "arxiv",
}


class MCPConnector:
    """Collect evidence from local and external MCP providers."""

    VALID_STATUSES = {"ok", "warning", "error", "not_configured"}

    def __init__(
        self,
        timeout_seconds: int = 20,
        env_prefix: str = "RESEARCH_MCP_",
    ):
        self.timeout_seconds = timeout_seconds
        self.env_prefix = env_prefix

    def collect(
        self,
        provider: str,
        task_packet: dict[str, Any],
        cwd: Path,
    ) -> MCPEvidence:
        if provider == "filesystem":
            return self._collect_filesystem(task_packet, cwd)
        return self._collect_external_command(provider, task_packet, cwd)

    def _collect_filesystem(
        self,
        task_packet: dict[str, Any],
        cwd: Path,
    ) -> MCPEvidence:
        topic = str(task_packet.get("topic", "")).strip()
        artifact_root = str(task_packet.get("artifact_root", "RESEARCH/[topic]/"))
        project_root = cwd / artifact_root.replace("[topic]", topic)
        required_outputs = [
            str(item) for item in task_packet.get("required_outputs", []) if str(item).strip()
        ]

        existing_count = 0
        provenance: list[str] = []
        for rel_path in required_outputs:
            target = project_root / rel_path
            if target.exists():
                existing_count += 1
                provenance.append(str(target))

        summary = (
            f"Detected {existing_count}/{len(required_outputs)} required outputs under "
            f"{project_root}"
        )
        status = "ok" if existing_count == len(required_outputs) else "warning"
        return MCPEvidence(
            provider="filesystem",
            status=status,
            summary=summary,
            provenance=provenance,
            data={
                "project_root": str(project_root),
                "required_output_count": len(required_outputs),
                "existing_output_count": existing_count,
            },
        )

    def _collect_external_command(
        self,
        provider: str,
        task_packet: dict[str, Any],
        cwd: Path,
    ) -> MCPEvidence:
        resolution = self.resolve_provider(provider)
        env_name = resolution.env_name
        command = resolution.command

        if not command:
            config_summary = provider_config_summary(resolve_provider_config(cwd=cwd))
            provider_key = self._provider_config_key(provider)
            data: dict[str, Any] = {
                "not_configured_scope": "external_command_adapter",
                "external_command_env": env_name,
                "provider_config": config_summary,
            }
            if provider_key in DIRECT_LITERATURE_PROVIDER_KEYS:
                data.update(
                    {
                        "provider_config_status": config_summary.get(provider_key, "missing"),
                        "recommended_status_tool": "qiongli_literature_status",
                        "recommended_search_tool": "qiongli_literature_search",
                    }
                )
                summary = (
                    f"The external command adapter for {provider} is not configured. "
                    f"Set {env_name} only if you intend to use a separate external MCP command. "
                    "Use qiongli_literature_status or qiongli_literature_search to check "
                    "the built-in literature provider config."
                )
            else:
                summary = f"External command adapter not configured. Set {env_name}."
            return MCPEvidence(
                provider=provider,
                status="not_configured",
                summary=summary,
                provenance=[env_name],
                data=data,
            )

        payload = {
            "provider": provider,
            "task_packet": task_packet,
        }
        try:
            parsed_cmd = split_command(command)
            child_env = dict(os.environ)
            child_env.update(provider_config_env(resolve_provider_config(cwd=cwd, env=child_env)))
            run_result = subprocess.run(
                parsed_cmd,
                input=json.dumps(payload, ensure_ascii=False),
                capture_output=True,
                text=True,
                cwd=str(cwd),
                env=child_env,
                timeout=self.timeout_seconds,
                check=False,
            )
        except subprocess.TimeoutExpired:
            return MCPEvidence(
                provider=provider,
                status="error",
                summary=f"External MCP command timed out after {self.timeout_seconds}s.",
                provenance=[env_name],
            )
        except OSError as exc:
            return MCPEvidence(
                provider=provider,
                status="error",
                summary=f"External MCP command failed to start: {exc}",
                provenance=[env_name],
            )

        if run_result.returncode != 0:
            stderr = (run_result.stderr or "").strip()
            return MCPEvidence(
                provider=provider,
                status="error",
                summary=f"External MCP command exited with code {run_result.returncode}.",
                provenance=[env_name, stderr] if stderr else [env_name],
            )

        stdout = (run_result.stdout or "").strip()
        if not stdout:
            return MCPEvidence(
                provider=provider,
                status="warning",
                summary="External MCP command returned empty output.",
                provenance=[env_name],
            )

        try:
            parsed = json.loads(stdout)
        except json.JSONDecodeError:
            return MCPEvidence(
                provider=provider,
                status="warning",
                summary="External MCP command returned non-JSON output.",
                provenance=[env_name, stdout[:280]],
            )

        raw_status = str(parsed.get("status", "ok")).strip().lower()
        status = raw_status if raw_status in self.VALID_STATUSES else "warning"
        summary = str(parsed.get("summary", f"External MCP response from {provider}.")).strip()
        raw_provenance = parsed.get("provenance", [])
        if isinstance(raw_provenance, list):
            provenance = [str(item) for item in raw_provenance]
        elif raw_provenance:
            provenance = [str(raw_provenance)]
        else:
            provenance = []
        data = parsed.get("data", {})
        if not isinstance(data, dict):
            data = {"raw_data": data}

        return MCPEvidence(
            provider=provider,
            status=status,
            summary=summary,
            provenance=provenance,
            data=data,
        )

    def _provider_env_var(self, provider: str) -> str:
        key = provider.upper().replace("-", "_")
        return f"{self.env_prefix}{key}_CMD"

    def _provider_config_key(self, provider: str) -> str:
        key = provider.strip().lower().replace("-", "_")
        if key == "semantic_scholar":
            return "semantic_scholar"
        return key

    def _provider_native_script(self, provider: str) -> Path:
        safe_provider_name = provider.replace("-", "_")
        script_name = f"mcp_{safe_provider_name}.py"
        for repo_root in self._repo_root_candidates():
            for candidate in (
                repo_root / "scripts" / script_name,
                repo_root / "tooling" / "scripts" / script_name,
            ):
                if candidate.exists():
                    return candidate
        current = Path(__file__).resolve()
        for parent in current.parents:
            candidate = parent / "scripts" / script_name
            if candidate.exists():
                return candidate
        return current.parents[1] / "scripts" / script_name

    def _repo_root_candidates(self) -> list[Path]:
        roots: list[Path] = []
        for start in (Path.cwd(), Path(__file__).resolve()):
            try:
                root = discover_repo_root(start)
            except ValueError:
                continue
            if not self._looks_like_qiongli_source_root(root):
                continue
            if root not in roots:
                roots.append(root)
        return roots

    def _looks_like_qiongli_source_root(self, root: Path) -> bool:
        layout = RepoLayout(root)
        return (layout.workflow / "SKILL.md").is_file() and (
            layout.standards / "mcp-agent-capability-map.yaml"
        ).is_file()

    def resolve_provider(self, provider: str) -> MCPProviderResolution:
        env_name = self._provider_env_var(provider)
        if provider == "filesystem":
            return MCPProviderResolution(
                provider=provider,
                env_name=env_name,
                source="filesystem",
            )

        command = os.environ.get(env_name, "").strip()
        native_script = self._provider_native_script(provider)
        native_script_str = str(native_script) if native_script.exists() else None

        if command:
            return MCPProviderResolution(
                provider=provider,
                env_name=env_name,
                source="env_override",
                command=command,
                native_script=native_script_str,
            )
        if native_script_str:
            return MCPProviderResolution(
                provider=provider,
                env_name=env_name,
                source="builtin",
                command=current_python_command(native_script_str),
                native_script=native_script_str,
            )
        return MCPProviderResolution(
            provider=provider,
            env_name=env_name,
            source="external_slot",
            native_script=native_script_str,
        )
