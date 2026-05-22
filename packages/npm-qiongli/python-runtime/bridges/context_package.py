from __future__ import annotations

import hashlib
import json
from typing import Iterable


def build_context_package(
    task_packet: dict[str, object],
    *,
    controller: str,
    agents: list[str],
) -> dict[str, object]:
    normalized_controller = _normalize_agent_name(controller)
    normalized_agents = [_normalize_agent_name(agent) for agent in agents]

    manifest_without_hash = {
        "task_id": _string_value(task_packet, "task_id"),
        "paper_type": _string_value(task_packet, "paper_type"),
        "topic": _string_value(task_packet, "topic"),
        "controller": normalized_controller,
        "agents": normalized_agents,
    }
    manifest = {
        **manifest_without_hash,
        "input_context_hash": _hash_manifest(manifest_without_hash),
    }

    return {
        "context_manifest": manifest,
        "agent_contexts": {
            "codex": _build_codex_context(task_packet, manifest),
            "claude": _build_claude_context(task_packet, manifest),
            "gemini": _build_gemini_context(task_packet, manifest),
        },
    }


def _normalize_agent_name(value: str) -> str:
    return " ".join(value.strip().lower().split())


def _hash_manifest(manifest: dict[str, object]) -> str:
    encoded_manifest = json.dumps(manifest, sort_keys=True).encode("utf-8")
    return hashlib.sha256(encoded_manifest).hexdigest()


def _string_value(task_packet: dict[str, object], key: str) -> str:
    value = task_packet.get(key, "")
    if value is None:
        return ""
    return str(value)


def _list_value(task_packet: dict[str, object], key: str) -> list[str]:
    value = task_packet.get(key, [])
    if value is None:
        return []
    if isinstance(value, str):
        return [value]
    if isinstance(value, Iterable):
        return [str(item) for item in value]
    return [str(value)]


def _format_list(items: list[str]) -> str:
    if not items:
        return "- None declared"
    return "\n".join(f"- {item}" for item in items)


def _build_header(manifest: dict[str, object]) -> str:
    return "\n".join(
        [
            f"Task: {manifest['task_id']}",
            f"Paper Type: {manifest['paper_type']}",
            f"Topic: {manifest['topic']}",
            f"Controller: {manifest['controller']}",
            f"Agents: {', '.join(manifest['agents'])}",
            f"Input Context Hash: {manifest['input_context_hash']}",
        ]
    )


def _build_codex_context(task_packet: dict[str, object], manifest: dict[str, object]) -> str:
    return "\n\n".join(
        [
            _build_header(manifest),
            "## Declared Write Set\n"
            + _format_list(_list_value(task_packet, "declared_write_set")),
            "## Verification Commands\n"
            + _format_list(_list_value(task_packet, "verification_commands")),
            "## Artifact Paths\n" + _format_list(_list_value(task_packet, "artifact_paths")),
        ]
    )


def _build_claude_context(task_packet: dict[str, object], manifest: dict[str, object]) -> str:
    return "\n\n".join(
        [
            _build_header(manifest),
            f"## Research State\n{_string_value(task_packet, 'research_state') or 'Not provided'}",
            f"## Evidence Ledger\n{_string_value(task_packet, 'evidence_ledger') or 'Not provided'}",
            "## Writing/Review Standards\n"
            + (_string_value(task_packet, "writing_review_standards") or "Not provided"),
        ]
    )


def _build_gemini_context(task_packet: dict[str, object], manifest: dict[str, object]) -> str:
    return "\n\n".join(
        [
            _build_header(manifest),
            f"## Task Packet\n{json.dumps(task_packet, sort_keys=True, indent=2)}",
        ]
    )
