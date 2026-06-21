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
    supported_agents = {"codex", "claude"}
    normalized_agents = [
        normalized
        for normalized in (_normalize_agent_name(agent) for agent in agents)
        if normalized in supported_agents
    ]

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
        "agent_contexts": _build_agent_contexts(task_packet, manifest, normalized_agents),
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


def _boundary_review_text(task_packet: dict[str, object]) -> str:
    boundary = task_packet.get("boundary_review", {})
    if not isinstance(boundary, dict):
        return "Not provided"
    existing = str(boundary.get("existing_review", "")).strip()
    if existing:
        return existing
    status = str(boundary.get("status", "")).strip()
    artifact = str(boundary.get("artifact", "")).strip()
    if status or artifact:
        return f"status: {status or 'unknown'}\nartifact: {artifact or 'context/boundary_review.md'}"
    return "Not provided"


def _writing_harness_text(task_packet: dict[str, object]) -> str:
    harness = task_packet.get("writing_harness", {})
    if not isinstance(harness, dict):
        harness = {}
    mode = str(harness.get("mode", "incremental-mainline") or "incremental-mainline")
    loop = str(harness.get("loop", "write_review_confirm") or "write_review_confirm")
    required_preflight = _format_inline_list(harness.get("required_preflight", []))
    block_conditions = _format_inline_list(harness.get("block_conditions", []))
    return "\n".join(
        [
            "Writing Harness Contract applies to Stage F drafting and revision.",
            f"- mode: {mode}",
            f"- required_preflight: {required_preflight}",
            f"- loop: {loop}",
            f"- block_conditions: {block_conditions}",
            "- Lock the Story Spine before prose: central claim, argumentative mainline, section jobs, non-goals, and evidence threshold.",
            "- Work in section or paragraph-cluster chunks.",
            "- For each chunk, run write -> review -> confirm before continuing.",
            "- Ask the next blocking boundary/grill question when the mainline, claim strength, or evidence threshold is unclear.",
            "- Block or revise mainline drift, logic jumps, missing support, and generic or vague claims.",
        ]
    )


def _format_inline_list(value: object) -> str:
    if isinstance(value, str):
        return value
    if isinstance(value, Iterable):
        rendered = [str(item) for item in value if str(item).strip()]
        return ", ".join(rendered) if rendered else "-"
    return str(value) if value is not None else "-"


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
            f"## Boundary Review\n{_boundary_review_text(task_packet)}",
            f"## Writing Harness\n{_writing_harness_text(task_packet)}",
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
            f"## Boundary Review\n{_boundary_review_text(task_packet)}",
            f"## Writing Harness\n{_writing_harness_text(task_packet)}",
        ]
    )


def _build_agent_contexts(
    task_packet: dict[str, object],
    manifest: dict[str, object],
    agents: list[str],
) -> dict[str, str]:
    contexts: dict[str, str] = {}
    if "codex" in agents:
        contexts["codex"] = _build_codex_context(task_packet, manifest)
    if "claude" in agents:
        contexts["claude"] = _build_claude_context(task_packet, manifest)
    return contexts
