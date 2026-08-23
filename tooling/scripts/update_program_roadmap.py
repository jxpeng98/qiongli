#!/usr/bin/env python3
from __future__ import annotations

import argparse
from collections import Counter
from dataclasses import dataclass
from datetime import date
import json
from pathlib import Path, PurePosixPath
import re
import sys
from typing import Any, Sequence


REPO_ROOT = Path(__file__).resolve().parents[2]
ROADMAP_RELATIVE = (
    "docs/superpowers/roadmaps/"
    "2026-08-02-qiongli-2-research-harness-master-roadmap.md"
)
LEDGER_RELATIVE = "docs/superpowers/roadmaps/qiongli-program-ledger-v1.json"
INDEX_RELATIVE = "docs/superpowers/roadmaps/qiongli-current-program-index.md"
SCHEMA_VERSION = "qiongli-program-ledger/v1"
ALLOWED_STATES = (
    "accepted",
    "active",
    "blocked",
    "deferred",
    "proposed",
    "superseded",
)
TOP_LEVEL_KEYS = {"schema_version", "roadmap", "tasks"}
TASK_KEYS = {
    "id",
    "state",
    "owner",
    "dependencies",
    "evidence",
    "commit",
    "run",
    "updated_at",
    "blocker",
}
TASK_PATTERN = re.compile(
    r"^- \[[ xX]\] `(?P<id>[A-Z]+-\d{3,4})` (?P<description>.+)$"
)
MILESTONE_PATTERN = re.compile(
    r"^## \d+\. Milestone (?P<id>M[0-7]) — (?P<title>.+)$"
)
COMMIT_PATTERN = re.compile(r"^[0-9a-f]{40}$")
RUN_PATTERN = re.compile(r"^[1-9][0-9]*$")


class ProgramLedgerError(RuntimeError):
    pass


@dataclass(frozen=True)
class RoadmapTask:
    id: str
    description: str
    milestone: str
    milestone_title: str
    workstream: str


def _read_text(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except (OSError, UnicodeError) as error:
        raise ProgramLedgerError(f"cannot read {path}: {error}") from error


def parse_roadmap(path: Path) -> tuple[RoadmapTask, ...]:
    tasks: list[RoadmapTask] = []
    milestone: tuple[str, str] | None = None
    lines = _read_text(path).splitlines()
    index = 0
    while index < len(lines):
        line = lines[index]
        line_number = index + 1
        milestone_match = MILESTONE_PATTERN.fullmatch(line)
        if milestone_match:
            milestone = (
                milestone_match.group("id"),
                milestone_match.group("title"),
            )
            index += 1
            continue
        task_match = TASK_PATTERN.fullmatch(line)
        if not task_match:
            index += 1
            continue
        if milestone is None:
            raise ProgramLedgerError(
                f"roadmap task at line {line_number} has no milestone"
            )
        task_id = task_match.group("id")
        description = task_match.group("description")
        index += 1
        continuation: list[str] = []
        while index < len(lines) and lines[index].startswith("  "):
            value = lines[index].strip()
            if value:
                continuation.append(value)
            index += 1
        if continuation:
            description += " " + " ".join(continuation)
        tasks.append(
            RoadmapTask(
                id=task_id,
                description=description,
                milestone=milestone[0],
                milestone_title=milestone[1],
                workstream=task_id.split("-", 1)[0],
            )
        )
    if not tasks:
        raise ProgramLedgerError("roadmap contains no canonical task rows")
    counts = Counter(task.id for task in tasks)
    duplicates = sorted(task_id for task_id, count in counts.items() if count > 1)
    if duplicates:
        raise ProgramLedgerError(
            "roadmap contains duplicate task ID: " + ", ".join(duplicates)
        )
    return tuple(tasks)


def _reject_duplicate_json_keys(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    value: dict[str, Any] = {}
    for key, item in pairs:
        if key in value:
            raise ProgramLedgerError(f"duplicate JSON key: {key}")
        value[key] = item
    return value


def _load_json(path: Path) -> dict[str, Any]:
    try:
        value = json.loads(
            _read_text(path), object_pairs_hook=_reject_duplicate_json_keys
        )
    except json.JSONDecodeError as error:
        raise ProgramLedgerError(f"invalid JSON in {path}: {error}") from error
    if not isinstance(value, dict):
        raise ProgramLedgerError("program ledger must be a JSON object")
    return value


def _canonical_relative_path(value: object, field: str) -> str:
    if not isinstance(value, str) or not value or "\\" in value:
        raise ProgramLedgerError(f"{field} must be a repository-relative POSIX path")
    path = PurePosixPath(value)
    if path.is_absolute() or ".." in path.parts or str(path) != value:
        raise ProgramLedgerError(f"{field} must be a repository-relative POSIX path")
    return value


def _require_string(value: object, field: str, *, nonempty: bool = False) -> str:
    if not isinstance(value, str) or (nonempty and not value.strip()):
        suffix = "non-empty " if nonempty else ""
        raise ProgramLedgerError(f"{field} must be a {suffix}string")
    return value


def _require_string_list(value: object, field: str) -> list[str]:
    if not isinstance(value, list) or any(not isinstance(item, str) for item in value):
        raise ProgramLedgerError(f"{field} must be a string array")
    if len(value) != len(set(value)):
        raise ProgramLedgerError(f"{field} contains duplicate values")
    return value


def _validate_date(value: object, task_id: str) -> str:
    text = _require_string(value, f"{task_id}.updated_at", nonempty=True)
    try:
        parsed = date.fromisoformat(text)
    except ValueError as error:
        raise ProgramLedgerError(
            f"{task_id}.updated_at must be an ISO date"
        ) from error
    if parsed.isoformat() != text:
        raise ProgramLedgerError(f"{task_id}.updated_at must be an ISO date")
    return text


def _validate_evidence(repo_root: Path, task_id: str, values: list[str]) -> None:
    root = repo_root.resolve()
    for raw_path in values:
        relative = _canonical_relative_path(raw_path, f"{task_id}.evidence")
        path = (root / relative).resolve()
        if not path.is_relative_to(root) or not path.is_file():
            raise ProgramLedgerError(
                f"{task_id} repository evidence does not exist: {relative}"
            )


def _validate_row(repo_root: Path, raw: object, index: int) -> dict[str, Any]:
    if not isinstance(raw, dict):
        raise ProgramLedgerError(f"tasks[{index}] must be an object")
    keys = set(raw)
    if keys != TASK_KEYS:
        missing = sorted(TASK_KEYS - keys)
        extra = sorted(keys - TASK_KEYS)
        detail = []
        if missing:
            detail.append("missing " + ", ".join(missing))
        if extra:
            detail.append("unexpected " + ", ".join(extra))
        raise ProgramLedgerError(f"tasks[{index}] has invalid fields: {'; '.join(detail)}")

    task_id = _require_string(raw["id"], f"tasks[{index}].id", nonempty=True)
    if not re.fullmatch(r"[A-Z]+-\d{3,4}", task_id):
        raise ProgramLedgerError(f"tasks[{index}].id is invalid: {task_id}")
    state = _require_string(raw["state"], f"{task_id}.state", nonempty=True)
    if state not in ALLOWED_STATES:
        raise ProgramLedgerError(f"invalid state for {task_id}: {state}")
    _require_string(raw["owner"], f"{task_id}.owner", nonempty=True)
    dependencies = _require_string_list(
        raw["dependencies"], f"{task_id}.dependencies"
    )
    if task_id in dependencies:
        raise ProgramLedgerError(f"{task_id} cannot depend on itself")
    evidence = _require_string_list(raw["evidence"], f"{task_id}.evidence")
    _validate_evidence(repo_root, task_id, evidence)
    commit = _require_string(raw["commit"], f"{task_id}.commit")
    run = _require_string(raw["run"], f"{task_id}.run")
    blocker = _require_string(raw["blocker"], f"{task_id}.blocker")
    _validate_date(raw["updated_at"], task_id)

    if commit and not COMMIT_PATTERN.fullmatch(commit):
        raise ProgramLedgerError(f"{task_id}.commit must be an exact commit SHA")
    if run and not RUN_PATTERN.fullmatch(run):
        raise ProgramLedgerError(f"{task_id}.run must be an exact Actions run ID")
    if state == "accepted":
        if not evidence:
            raise ProgramLedgerError(f"{task_id} accepted state requires repository evidence")
        if not COMMIT_PATTERN.fullmatch(commit):
            raise ProgramLedgerError(f"{task_id} accepted state requires an exact commit")
        if not RUN_PATTERN.fullmatch(run):
            raise ProgramLedgerError(f"{task_id} accepted state requires an Actions run")
    if state == "blocked" and not blocker.strip():
        raise ProgramLedgerError(f"{task_id} blocked state requires a blocker")
    return raw


def _validate_dependencies(rows: Sequence[dict[str, Any]]) -> None:
    known = {row["id"] for row in rows}
    dependencies = {row["id"]: row["dependencies"] for row in rows}
    for task_id, values in dependencies.items():
        unknown = sorted(set(values) - known)
        if unknown:
            raise ProgramLedgerError(
                f"{task_id} has unknown dependency: {', '.join(unknown)}"
            )

    visiting: set[str] = set()
    visited: set[str] = set()

    def visit(task_id: str, path: tuple[str, ...]) -> None:
        if task_id in visiting:
            cycle_start = path.index(task_id)
            cycle = (*path[cycle_start:], task_id)
            raise ProgramLedgerError("dependency cycle: " + " -> ".join(cycle))
        if task_id in visited:
            return
        visiting.add(task_id)
        for dependency in dependencies[task_id]:
            visit(dependency, (*path, task_id))
        visiting.remove(task_id)
        visited.add(task_id)

    for task_id in dependencies:
        visit(task_id, ())


def validate_program(
    repo_root: Path, roadmap_path: Path, ledger_path: Path
) -> tuple[tuple[RoadmapTask, ...], tuple[dict[str, Any], ...]]:
    tasks = parse_roadmap(roadmap_path)
    document = _load_json(ledger_path)
    if set(document) != TOP_LEVEL_KEYS:
        raise ProgramLedgerError("program ledger has invalid top-level fields")
    if document["schema_version"] != SCHEMA_VERSION:
        raise ProgramLedgerError(
            f"schema_version must be {SCHEMA_VERSION!r}"
        )
    try:
        expected_roadmap = roadmap_path.resolve().relative_to(repo_root.resolve()).as_posix()
    except ValueError as error:
        raise ProgramLedgerError("roadmap must be inside the repository") from error
    if document["roadmap"] != expected_roadmap:
        raise ProgramLedgerError(f"roadmap must be {expected_roadmap!r}")
    raw_rows = document["tasks"]
    if not isinstance(raw_rows, list):
        raise ProgramLedgerError("tasks must be an array")
    rows = tuple(
        _validate_row(repo_root, raw, index) for index, raw in enumerate(raw_rows)
    )
    counts = Counter(row["id"] for row in rows)
    duplicates = sorted(task_id for task_id, count in counts.items() if count > 1)
    if duplicates:
        raise ProgramLedgerError("duplicate task ID: " + ", ".join(duplicates))
    roadmap_ids = [task.id for task in tasks]
    ledger_ids = [row["id"] for row in rows]
    if roadmap_ids != ledger_ids:
        missing = sorted(set(roadmap_ids) - set(ledger_ids))
        extra = sorted(set(ledger_ids) - set(roadmap_ids))
        detail = []
        if missing:
            detail.append("missing " + ", ".join(missing))
        if extra:
            detail.append("unexpected " + ", ".join(extra))
        if not detail:
            detail.append("task order differs")
        raise ProgramLedgerError(
            "ledger IDs do not match roadmap: " + "; ".join(detail)
        )
    _validate_dependencies(rows)
    return tasks, rows


def _escape_table(value: str) -> str:
    return value.replace("|", "\\|").replace("\n", " ")


def _status_detail(row: dict[str, Any]) -> str:
    parts: list[str] = []
    if row["evidence"]:
        parts.append("evidence: " + "<br>".join(f"`{path}`" for path in row["evidence"]))
    if row["commit"]:
        parts.append(f"commit `{row['commit'][:12]}`")
    if row["run"]:
        parts.append(f"run `{row['run']}`")
    if row["blocker"]:
        parts.append("blocker: " + _escape_table(row["blocker"]))
    return "<br>".join(parts) or "—"


def render_index(
    tasks: Sequence[RoadmapTask], rows: Sequence[dict[str, Any]]
) -> str:
    row_by_id = {row["id"]: row for row in rows}
    counts = Counter(row["state"] for row in rows)
    updated_at = max(row["updated_at"] for row in rows)
    lines = [
        "# Qiongli current program index",
        "",
        "<!-- Generated by tooling/scripts/update_program_roadmap.py. Do not edit. -->",
        "",
        f"Ledger schema: `{SCHEMA_VERSION}`",
        f"Ledger update: `{updated_at}`",
        f"Tasks: `{len(tasks)}`",
        "",
        "The [master roadmap](2026-08-02-qiongli-2-research-harness-master-roadmap.md)",
        "owns descriptions and ordering. The",
        "[program ledger](qiongli-program-ledger-v1.json) owns live task state.",
        "Markdown checkboxes are presentation only.",
        "",
        "## State summary",
        "",
        "| State | Count | Meaning |",
        "|---|---:|---|",
        f"| `accepted` | {counts['accepted']} | Exact repository evidence, commit and CI run are recorded. |",
        f"| `active` | {counts['active']} | The bounded task is currently being implemented or integrated. |",
        f"| `blocked` | {counts['blocked']} | Work cannot advance until the recorded blocker clears. |",
        f"| `deferred` | {counts['deferred']} | Intentionally held behind a milestone or policy gate. |",
        f"| `proposed` | {counts['proposed']} | Ordered work that has not entered implementation. |",
        f"| `superseded` | {counts['superseded']} | Replaced by another recorded task or decision. |",
        "",
        "## Tasks",
    ]
    current_milestone: str | None = None
    current_workstream: str | None = None
    for task in tasks:
        row = row_by_id[task.id]
        if task.milestone != current_milestone:
            current_milestone = task.milestone
            current_workstream = None
            lines.extend(
                ["", f"## {task.milestone} — {task.milestone_title}"]
            )
        if task.workstream != current_workstream:
            current_workstream = task.workstream
            lines.extend(
                [
                    "",
                    f"### {task.workstream}",
                    "",
                    "| ID | State | Owner | Description | Dependencies | Evidence / blocker |",
                    "|---|---|---|---|---|---|",
                ]
            )
        dependencies = (
            "<br>".join(f"`{value}`" for value in row["dependencies"])
            if row["dependencies"]
            else "—"
        )
        lines.append(
            f"| `{task.id}` | `{row['state']}` | `{_escape_table(row['owner'])}` | "
            f"{_escape_table(task.description)} | {dependencies} | {_status_detail(row)} |"
        )
    return "\n".join(lines) + "\n"


def require_current_index(index_path: Path, rendered: str) -> None:
    if not index_path.is_file() or _read_text(index_path) != rendered:
        raise ProgramLedgerError(
            "generated program index is stale; run "
            "python3 tooling/scripts/update_program_roadmap.py"
        )


def _run(check: bool) -> None:
    tasks, rows = validate_program(
        REPO_ROOT,
        REPO_ROOT / ROADMAP_RELATIVE,
        REPO_ROOT / LEDGER_RELATIVE,
    )
    rendered = render_index(tasks, rows)
    index_path = REPO_ROOT / INDEX_RELATIVE
    if check:
        require_current_index(index_path, rendered)
        print(f"program roadmap: {len(tasks)} tasks; generated index is current")
        return
    index_path.write_text(rendered, encoding="utf-8", newline="\n")
    print(f"wrote {INDEX_RELATIVE} ({len(tasks)} tasks)")


def main(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Validate the program ledger and generate its current index."
    )
    parser.add_argument(
        "--check", action="store_true", help="fail when the generated index is stale"
    )
    arguments = parser.parse_args(argv)
    try:
        _run(arguments.check)
    except ProgramLedgerError as error:
        print(f"program roadmap error: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
