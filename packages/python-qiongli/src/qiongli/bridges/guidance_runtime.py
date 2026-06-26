from __future__ import annotations

import json
import os
import re
import uuid
from dataclasses import asdict, dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from .project_manifest import (
    init_project_manifest,
    load_project_manifest,
    manifest_to_guidance_section,
)


GUIDANCE_MODES = ("off", "read", "propose", "apply")
LOCAL_GUIDANCE_REL = Path(".qiongli") / "local_guidance.md"
GUIDANCE_DIR_REL = Path(".qiongli") / "guidance.d"
GUIDANCE_MANIFEST_REL = Path(".qiongli") / "guidance_manifest.yaml"
TRACE_REL = Path(".qiongli") / "trace"
TRACE_INDEX_REL = TRACE_REL / "index.jsonl"


@dataclass(frozen=True)
class GuidancePaths:
    project_root: Path
    project_guidance: Path
    project_guidance_dir: Path
    project_guidance_manifest: Path
    trace_root: Path
    trace_index: Path
    global_preferences: Path


@dataclass(frozen=True)
class GuidanceState:
    enabled: bool
    mode: str
    project_guidance_file: str
    global_preferences_file: str
    trace_dir: str
    summary: str
    guidance_context: str
    guidance_files_read: list[str]
    guidance_sources: list[dict[str, str]]
    source_order: list[str]
    conflicts: list[str]
    run_id: str = ""
    warnings: list[str] | None = None
    project_manifest: dict[str, Any] | None = None

    def to_packet(self) -> dict[str, Any]:
        payload = asdict(self)
        payload["warnings"] = list(self.warnings or [])
        payload["project_manifest"] = dict(self.project_manifest or {})
        return payload


def resolve_guidance_paths(project_root: Path) -> GuidancePaths:
    root = Path(project_root).expanduser().resolve()
    global_home = os.environ.get("QIONGLI_GUIDANCE_HOME") or os.environ.get("QIONGLI_CONFIG_HOME")
    global_root = Path(global_home).expanduser().resolve() if global_home else Path.home() / ".qiongli"
    return GuidancePaths(
        project_root=root,
        project_guidance=root / LOCAL_GUIDANCE_REL,
        project_guidance_dir=root / GUIDANCE_DIR_REL,
        project_guidance_manifest=root / GUIDANCE_MANIFEST_REL,
        trace_root=root / TRACE_REL,
        trace_index=root / TRACE_INDEX_REL,
        global_preferences=global_root / "preferences.md",
    )


def init_project_guidance(project_root: Path) -> GuidancePaths:
    paths = resolve_guidance_paths(project_root)
    paths.trace_root.mkdir(parents=True, exist_ok=True)
    paths.project_guidance_dir.mkdir(parents=True, exist_ok=True)
    init_project_manifest(paths.project_root)
    if not paths.project_guidance.exists():
        paths.project_guidance.parent.mkdir(parents=True, exist_ok=True)
        paths.project_guidance.write_text(_default_local_guidance(), encoding="utf-8")
    return paths


def guidance_bootstrap_status(project_root: Path, *, mode: str = "propose") -> dict[str, Any]:
    normalized_mode = _normalize_mode(mode)
    paths = resolve_guidance_paths(project_root)
    enabled = normalized_mode != "off"
    return {
        "enabled": enabled,
        "needed": bool(enabled and not paths.project_guidance.is_file()),
        "mode": normalized_mode,
        "project_guidance": _rel(paths.project_root, paths.project_guidance),
        "guidance_dir": _rel(paths.project_root, paths.project_guidance_dir),
        "guidance_fragment_count": len(_iter_project_guidance_fragments(paths)),
        "trace_root": _rel(paths.project_root, paths.trace_root),
        "trace_index": _rel(paths.project_root, paths.trace_index),
    }


def ensure_project_guidance(project_root: Path, *, mode: str = "propose") -> GuidancePaths | None:
    if _normalize_mode(mode) == "off":
        return None
    return init_project_guidance(project_root)


def effective_guidance(project_root: Path, *, mode: str = "propose", run_id: str = "") -> GuidanceState:
    normalized_mode = _normalize_mode(mode)
    paths = resolve_guidance_paths(project_root)
    if normalized_mode == "off":
        return GuidanceState(
            enabled=False,
            mode="off",
            project_guidance_file=_rel(paths.project_root, paths.project_guidance),
            global_preferences_file=str(paths.global_preferences),
            trace_dir="",
            summary="Local guidance disabled for this run.",
            guidance_context="",
            guidance_files_read=[],
            guidance_sources=[],
            source_order=[],
            conflicts=[],
            run_id=run_id,
            warnings=[],
            project_manifest={},
        )

    sections: list[str] = []
    files_read: list[str] = []
    guidance_sources: list[dict[str, str]] = []
    source_order: list[str] = []
    warnings: list[str] = []
    manifest_state = load_project_manifest(paths.project_root)
    sections.append("## Project Manifest\n\n" + manifest_to_guidance_section(manifest_state))
    files_read.append(
        _rel(paths.project_root, manifest_state.path)
        if manifest_state.exists
        else "<implicit-project-manifest>"
    )
    guidance_sources.append(
        {
            "kind": "project-manifest",
            "path": files_read[-1],
            "label": "Project Manifest",
        }
    )
    source_order.append("project-manifest")
    source_specs: list[tuple[str, str, Path]] = [
        ("global-preferences", "Global Preferences", paths.global_preferences),
        ("project-local", "Project Local Guidance", paths.project_guidance),
    ]
    source_specs.extend(
        ("project-fragment", f"Project Guidance Fragment: {path.name}", path)
        for path in _iter_project_guidance_fragments(paths)
    )
    for kind, label, path in source_specs:
        if not path.is_file():
            continue
        try:
            text = path.read_text(encoding="utf-8").strip()
        except OSError as exc:
            warnings.append(f"Failed to read {path}: {exc}")
            continue
        if not text:
            continue
        sections.append(f"## {label}\n\n{text}")
        files_read.append(str(path) if kind == "global-preferences" else _rel(paths.project_root, path))
        guidance_sources.append({"kind": kind, "path": files_read[-1], "label": label})
        source_order.append(kind)

    context = "\n\n".join(sections)
    return GuidanceState(
        enabled=bool(context),
        mode=normalized_mode,
        project_guidance_file=_rel(paths.project_root, paths.project_guidance),
        global_preferences_file=str(paths.global_preferences),
        trace_dir="",
        summary=_summarize_guidance(context, files_read),
        guidance_context=context,
        guidance_files_read=files_read,
        guidance_sources=guidance_sources,
        source_order=source_order,
        conflicts=[],
        run_id=run_id,
        warnings=warnings,
        project_manifest=manifest_state.to_packet(),
    )


def write_guidance_trace(
    *,
    project_root: Path,
    guidance_state: GuidanceState,
    task_packet: dict[str, Any],
    draft_content: str,
    review_content: str,
    merged_analysis: str,
    validator_gate: dict[str, Any],
    applied: bool,
) -> dict[str, Any]:
    paths = init_project_guidance(project_root)
    run_id = guidance_state.run_id or uuid.uuid4().hex
    run_dir = paths.trace_root / "runs" / run_id
    run_dir.mkdir(parents=True, exist_ok=True)

    _write_json(run_dir / "task_packet.json", task_packet)
    _write_json(run_dir / "project_manifest.json", guidance_state.project_manifest or {})
    (run_dir / "guidance_context.md").write_text(
        guidance_state.guidance_context or "Local guidance disabled or empty.\n",
        encoding="utf-8",
    )
    (run_dir / "draft.md").write_text(draft_content or "[no draft produced]\n", encoding="utf-8")
    (run_dir / "review.md").write_text(review_content or "[no review produced]\n", encoding="utf-8")
    (run_dir / "merged_analysis.md").write_text(
        merged_analysis or "[no merged analysis]\n",
        encoding="utf-8",
    )
    _write_json(run_dir / "validator_gate.json", validator_gate)
    (run_dir / "guidance_update_proposal.md").write_text(
        _proposal_text(task_packet, validator_gate, applied),
        encoding="utf-8",
    )
    apply_result: dict[str, Any] = {}
    if applied:
        apply_result = apply_guidance_proposal(
            paths.project_root,
            run_dir / "guidance_update_proposal.md",
        )

    record = {
        "run_id": run_id,
        "created_at": _utc_now(),
        "task_id": str(task_packet.get("task_id", "")),
        "paper_type": str(task_packet.get("paper_type", "")),
        "topic": str(task_packet.get("topic", "")),
        "cwd": str(paths.project_root),
        "guidance_mode": guidance_state.mode,
        "run_dir": _rel(paths.project_root, run_dir),
        "required_outputs": list(task_packet.get("required_outputs", []) or []),
        "found_outputs": list(validator_gate.get("found", []) or []),
        "missing_outputs": list(validator_gate.get("missing", []) or []),
        "guidance_files_read": list(guidance_state.guidance_files_read),
        "guidance_sources": list(guidance_state.guidance_sources),
        "source_order": list(guidance_state.source_order),
        "guidance_conflicts": list(guidance_state.conflicts),
        "project_manifest": dict(guidance_state.project_manifest or {}),
        "guidance_proposal": _rel(paths.project_root, run_dir / "guidance_update_proposal.md"),
        "applied_guidance_update": bool(apply_result.get("applied")) if applied else False,
    }
    if apply_result:
        record["apply_result"] = dict(apply_result)
    paths.trace_index.parent.mkdir(parents=True, exist_ok=True)
    with paths.trace_index.open("a", encoding="utf-8") as handle:
        handle.write(json.dumps(record, ensure_ascii=False, sort_keys=True) + "\n")
    return record


def guidance_trace_summary(project_root: Path, *, limit: int = 20) -> dict[str, Any]:
    paths = resolve_guidance_paths(project_root)
    if not paths.trace_index.is_file():
        return {"project_dir": str(paths.project_root), "run_count": 0, "runs": []}
    rows: list[dict[str, Any]] = []
    for line in paths.trace_index.read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        try:
            parsed = json.loads(line)
        except json.JSONDecodeError:
            continue
        if isinstance(parsed, dict):
            rows.append(parsed)
    return {
        "project_dir": str(paths.project_root),
        "run_count": len(rows),
        "runs": rows[-max(0, limit):],
    }


def apply_guidance_proposal(project_root: Path, proposal: Path) -> dict[str, Any]:
    paths = init_project_guidance(project_root)
    proposal_path = Path(proposal)
    if not proposal_path.is_absolute():
        proposal_path = (paths.project_root / proposal_path).resolve()
    if not proposal_path.is_file():
        raise FileNotFoundError(f"Guidance proposal not found: {proposal_path}")

    proposal_text = proposal_path.read_text(encoding="utf-8")
    proposed_changes = _extract_proposed_changes(proposal_text)
    if not proposed_changes:
        return {
            "applied": False,
            "reason": "proposal contains no proposed changes",
            "proposal": str(proposal_path),
        }

    guidance_text = paths.project_guidance.read_text(encoding="utf-8")
    run_id = _run_id_from_proposal_path(paths.project_root, proposal_path)
    addition = "\n".join(
        [
            "",
            f"### Applied Proposal: {run_id}",
            "",
            "- source: `" + _rel(paths.project_root, proposal_path) + "`",
            "- applied_at: " + _utc_now(),
            "",
            "#### Proposed Changes",
            "",
            proposed_changes,
            "",
        ]
    )
    paths.project_guidance.write_text(guidance_text.rstrip() + "\n" + addition, encoding="utf-8")
    return {
        "applied": True,
        "proposal": _rel(paths.project_root, proposal_path),
        "project_guidance": _rel(paths.project_root, paths.project_guidance),
        "run_id": run_id,
    }


def create_guidance_fragment(project_root: Path, name: str) -> dict[str, Any]:
    paths = init_project_guidance(project_root)
    slug = _slugify_guidance_name(name)
    if not slug:
        raise ValueError("Guidance fragment name must contain letters or numbers.")
    path = paths.project_guidance_dir / f"{slug}.md"
    if path.exists():
        raise FileExistsError(f"Guidance fragment already exists: {_rel(paths.project_root, path)}")
    path.write_text(
        "\n".join(
            [
                f"# {slug.replace('-', ' ').title()}",
                "",
                "## Scope",
                "",
                "- Describe when this project guidance applies.",
                "",
                "## Guidance",
                "",
                "- Add one stable project rule.",
                "",
                "## Evidence",
                "",
                "- Link to trace runs, project artifacts, or explicit user decisions.",
                "",
            ]
        ),
        encoding="utf-8",
    )
    return {"created": True, "path": _rel(paths.project_root, path)}


def list_project_guidance_sources(project_root: Path) -> dict[str, Any]:
    state = effective_guidance(project_root, mode="read")
    return {
        "project_dir": str(Path(project_root).expanduser().resolve()),
        "sources": list(state.guidance_sources),
        "files_read": list(state.guidance_files_read),
    }


def lint_project_guidance(project_root: Path) -> dict[str, Any]:
    root = Path(project_root).expanduser().resolve()
    state = effective_guidance(root, mode="read")
    forbidden_patterns = [
        ("required outputs", re.compile(r"\b(ignore|skip|override)\b.{0,80}\brequired outputs?\b", re.I)),
        ("evidence gates", re.compile(r"\b(ignore|skip|override)\b.{0,80}\bevidence gates?\b", re.I)),
        ("quality gates", re.compile(r"\b(ignore|skip|override)\b.{0,80}\bquality gates?\b", re.I)),
        ("safety checks", re.compile(r"\b(ignore|skip|override)\b.{0,80}\bsafety checks?\b", re.I)),
    ]
    findings: list[dict[str, str]] = []
    for source in state.guidance_sources:
        path_text = source["path"]
        path = Path(path_text)
        full_path = path if path.is_absolute() else root / path
        text = full_path.read_text(encoding="utf-8") if full_path.is_file() else ""
        for label, pattern in forbidden_patterns:
            if pattern.search(text):
                findings.append(
                    {
                        "path": path_text,
                        "severity": "error",
                        "message": f"Guidance appears to weaken {label}.",
                    }
                )
    return {"ok": not findings, "findings": findings}


def _default_local_guidance() -> str:
    return "\n".join(
        [
            "# Qiongli Local Guidance",
            "",
            "## Scope",
            "",
            "- This file contains project-local guidance for future Qiongli runs.",
            "- It must not override canonical task contracts, required outputs, evidence gates, or safety checks.",
            "",
            "## Active Guidance",
            "",
            "- No project-specific guidance recorded yet.",
            "",
            "## Artifact Policy",
            "",
            "- Keep run traces under `.qiongli/trace/` in this project.",
            "- Treat `RESEARCH/[topic]/...` as the authoritative location for formal research artifacts.",
            "",
            "## Project Preferences",
            "",
            "- No project-specific preferences recorded yet.",
            "",
            "## Trace Anchors",
            "",
            "- See `.qiongli/trace/index.jsonl` for run-level trace records.",
            "",
            "## Revision History",
            "",
            "- Initial local guidance scaffold.",
            "",
        ]
    )


def _normalize_mode(mode: str) -> str:
    normalized = str(mode or "propose").strip().lower()
    if normalized not in GUIDANCE_MODES:
        available = ", ".join(GUIDANCE_MODES)
        raise ValueError(f"Unsupported guidance mode: {mode}. Available: {available}")
    return normalized


def _iter_project_guidance_fragments(paths: GuidancePaths) -> list[Path]:
    if not paths.project_guidance_dir.is_dir():
        return []
    return sorted(
        path
        for path in paths.project_guidance_dir.glob("*.md")
        if path.is_file() and not path.name.startswith(".")
    )


def _slugify_guidance_name(name: str) -> str:
    normalized = re.sub(r"[^a-z0-9]+", "-", str(name).strip().lower())
    return normalized.strip("-")


def _rel(root: Path, path: Path) -> str:
    try:
        return path.resolve().relative_to(root.resolve()).as_posix()
    except ValueError:
        return str(path)


def _summarize_guidance(context: str, files_read: list[str]) -> str:
    if not context:
        return "No local guidance configured."
    return "Loaded guidance from " + ", ".join(files_read) + "."


def _write_json(path: Path, payload: Any) -> None:
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def _proposal_text(task_packet: dict[str, Any], validator_gate: dict[str, Any], applied: bool) -> str:
    missing = list(validator_gate.get("missing", []) or [])
    lines = [
        "# Guidance Update Proposal",
        "",
        "## Proposed Changes",
        "",
    ]
    if missing:
        lines.extend(
            [
                "- Keep future runs aware that required outputs were missing and preserve trace bundles for audit.",
                "",
                "## Evidence Source",
                "",
                f"- task_id: `{task_packet.get('task_id', '')}`",
                f"- topic: `{task_packet.get('topic', '')}`",
                "- missing_outputs: " + ", ".join(f"`{item}`" for item in missing),
            ]
        )
    else:
        lines.extend(
            [
                "- No guidance changes proposed from this run.",
                "",
                "## Evidence Source",
                "",
                f"- task_id: `{task_packet.get('task_id', '')}`",
                f"- topic: `{task_packet.get('topic', '')}`",
            ]
        )
    lines.extend(
        [
            "",
            "## Affected Future Behavior",
            "",
            "- Future runs may use this trace to understand output coverage and artifact-policy risk.",
            "",
            "## Rejected Alternatives",
            "",
            "- Do not update canonical skills or workflow contracts from a project-local run.",
            "",
            "## Suggested Target",
            "",
            "- project-local",
            "",
            "## Conflict Check",
            "",
            "- Do not apply if the proposal weakens required outputs, evidence gates, quality gates, or safety checks.",
            "",
            "## Applied",
            "",
            "true" if applied else "false",
            "",
        ]
    )
    return "\n".join(lines)


def _extract_proposed_changes(proposal_text: str) -> str:
    match = re.search(
        r"^## Proposed Changes\s*\n(?P<body>.*?)(?=^## |\Z)",
        proposal_text,
        flags=re.MULTILINE | re.DOTALL,
    )
    if not match:
        return ""
    return match.group("body").strip()


def _run_id_from_proposal_path(project_root: Path, proposal_path: Path) -> str:
    try:
        relative_parts = proposal_path.resolve().relative_to(project_root.resolve()).parts
    except ValueError:
        return proposal_path.parent.name
    if len(relative_parts) >= 5 and relative_parts[0] == ".qiongli" and relative_parts[1] == "trace":
        return relative_parts[3]
    return proposal_path.parent.name


def _utc_now() -> str:
    return datetime.now(timezone.utc).replace(microsecond=0).isoformat()
