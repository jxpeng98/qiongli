#!/usr/bin/env python3
from __future__ import annotations

import argparse
from datetime import date
import json
import re
import sys
from pathlib import Path, PurePosixPath, PureWindowsPath
from typing import Any, Sequence


REPO_ROOT = Path(__file__).resolve().parents[2]
DEFAULT_RECORD = REPO_ROOT / "tooling" / "architecture" / "arc-201-decisions.json"
DEFAULT_CURRENT_RECORD = (
    REPO_ROOT / "tooling" / "architecture" / "current-decisions.json"
)
CURRENT_RECORD_KEYS = {"schema_version", "record_type", "branch", "decisions"}
CURRENT_DECISION_KEYS = {"task_id", "adr_number", "title", "path", "status"}
EXPECTED_DECISIONS = (
    ("ARC-201A", "0201", "0201-executable-topology.md"),
    ("ARC-201B", "0202", "0202-rust-native-ui-and-accessibility.md"),
    ("ARC-201C", "0203", "0203-agent-backend-and-tool-host.md"),
    ("ARC-201D", "0204", "0204-versioned-state-and-secret-storage.md"),
    ("ARC-201E", "0205", "0205-deterministic-resource-pack.md"),
    ("ARC-201F", "0206", "0206-declarative-install-plan-and-client-trust.md"),
    ("ARC-201G", "0207", "0207-release-channel-and-artifact-identity.md"),
)
EXPECTED_TASKS = tuple(item[0] for item in EXPECTED_DECISIONS)
REQUIRED_SECTIONS = (
    "Context",
    "Decision drivers",
    "Decision",
    "Alternatives considered",
    "Consequences",
    "Security and privacy",
    "Rollback",
    "Acceptance tests",
    "Follow-up tasks",
)
PLACEHOLDER_PATTERN = re.compile(r"\b(?:TODO|TBD|FIXME|CHANGEME)\b", re.IGNORECASE)
LOCAL_PATH_PATTERN = re.compile(
    r"(?:/Users/[^/\s]+/|/home/[^/\s]+/|[A-Za-z]:\\Users\\|file://)",
    re.IGNORECASE,
)
HEADING_PATTERN = re.compile(r"^## (.+?)\s*$", re.MULTILINE)
TEST_CASE_PATTERN = re.compile(r"(?m)^(?:\d+\.|-)\s+\S")


class DecisionValidationError(ValueError):
    pass


def load_record(path: Path) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise DecisionValidationError(f"cannot load {path}: {error}") from error
    if not isinstance(value, dict):
        raise DecisionValidationError(f"{path} must contain a JSON object")
    return value


def _section_body(content: str, heading: str) -> str:
    match = re.search(
        rf"(?ms)^## {re.escape(heading)}\s*$\n(.*?)(?=^## |\Z)", content
    )
    return match.group(1).strip() if match else ""


def is_canonical_repository_path(relative: str) -> bool:
    if not relative or "\\" in relative:
        return False
    posix = PurePosixPath(relative)
    windows = PureWindowsPath(relative)
    if posix.is_absolute() or windows.is_absolute():
        return False
    if ".." in posix.parts or ".." in windows.parts:
        return False
    return str(posix) == relative


def resolve_adr_path(repo_root: Path, relative: str) -> Path:
    if not is_canonical_repository_path(relative):
        raise DecisionValidationError(
            "ADR path must be a canonical repository-relative POSIX path"
        )
    resolved_repo_root = repo_root.resolve(strict=True)
    decisions_path = repo_root
    for component in ("docs", "architecture", "decisions"):
        decisions_path = decisions_path / component
        if decisions_path.is_symlink():
            raise DecisionValidationError(
                "decisions root must not contain a symbolic-link component"
            )
    decisions_root = decisions_path.resolve(strict=True)
    try:
        decisions_root.relative_to(resolved_repo_root)
    except ValueError as error:
        raise DecisionValidationError(
            "decisions root must resolve inside the repository"
        ) from error
    candidate = repo_root / PurePosixPath(relative)
    if candidate.is_symlink():
        raise DecisionValidationError("ADR path must not be a symbolic link")
    try:
        resolved = candidate.resolve(strict=True)
        resolved.relative_to(decisions_root)
    except (OSError, ValueError, RuntimeError) as error:
        raise DecisionValidationError(
            "ADR path must resolve to a regular file inside the decisions root"
        ) from error
    if not resolved.is_file():
        raise DecisionValidationError("ADR path must resolve to a regular file")
    return resolved


def validate_adr(path: Path, entry: dict[str, Any]) -> list[str]:
    errors: list[str] = []
    try:
        content = path.read_text(encoding="utf-8")
    except OSError as error:
        return [f"{path}: cannot read ADR: {error}"]

    task_id = entry.get("task_id")
    number = entry.get("adr_number")
    status = entry.get("status")
    expected_title = f"# ADR {number}: {entry.get('title')}"
    actual_title = content.splitlines()[0] if content else ""
    if actual_title != expected_title:
        errors.append(f"{path}: title must be {expected_title!r}")
    if f"- Status: {status}" not in content:
        errors.append(f"{path}: metadata must declare '- Status: {status}'")
    if f"- Task ID: `{task_id}`" not in content:
        errors.append(f"{path}: metadata must declare Task ID {task_id}")
    date_match = re.search(r"(?m)^- Date: (\d{4}-\d{2}-\d{2})$", content)
    if not date_match:
        errors.append(f"{path}: metadata must contain an ISO decision date")
    else:
        try:
            date.fromisoformat(date_match.group(1))
        except ValueError:
            errors.append(f"{path}: metadata decision date is not a real date")
    if not re.search(r"(?m)^- Owners: \S.+$", content):
        errors.append(f"{path}: metadata must name an owner")

    headings = HEADING_PATTERN.findall(content)
    for section in REQUIRED_SECTIONS:
        count = headings.count(section)
        if count != 1:
            errors.append(
                f"{path}: expected exactly one '## {section}' section, found {count}"
            )
            continue
        body = _section_body(content, section)
        if len(body) < 40:
            errors.append(f"{path}: '## {section}' is not decision-grade content")

    acceptance = _section_body(content, "Acceptance tests")
    if len(TEST_CASE_PATTERN.findall(acceptance)) < 3:
        errors.append(f"{path}: acceptance must contain at least three explicit tests")
    if PLACEHOLDER_PATTERN.search(content):
        errors.append(f"{path}: contains an unresolved placeholder marker")
    if LOCAL_PATH_PATTERN.search(content):
        errors.append(f"{path}: contains a machine-specific absolute path")
    return errors


def validate_record(repo_root: Path, record: dict[str, Any]) -> list[str]:
    errors: list[str] = []
    if record.get("schema_version") != "1.0":
        errors.append("decision record schema_version must be '1.0'")
    if record.get("record_type") != "qiongli-architecture-decision-set":
        errors.append("decision record_type is invalid")
    if record.get("branch") != "2.x":
        errors.append("decision record must be bound to branch '2.x'")
    if record.get("status") != "accepted":
        errors.append("decision set must be accepted before native scaffolding")

    decisions = record.get("decisions")
    if not isinstance(decisions, list):
        return [*errors, "decision record must contain a decisions array"]
    task_ids = [entry.get("task_id") for entry in decisions if isinstance(entry, dict)]
    if tuple(task_ids) != EXPECTED_TASKS:
        errors.append(
            "decision record must contain ARC-201A through ARC-201G exactly once "
            "and in order"
        )

    seen_paths: set[str] = set()
    seen_numbers: set[str] = set()
    for index, entry in enumerate(decisions):
        if not isinstance(entry, dict):
            errors.append(f"decisions[{index}] must be an object")
            continue
        task_id = entry.get("task_id", f"decisions[{index}]")
        expected = EXPECTED_DECISIONS[index] if index < len(EXPECTED_DECISIONS) else None
        if expected is not None:
            expected_task, expected_number, expected_filename = expected
            if task_id != expected_task:
                errors.append(f"decisions[{index}]: task_id must be {expected_task}")
            if entry.get("adr_number") != expected_number:
                errors.append(f"{task_id}: adr_number must be {expected_number}")
        relative = entry.get("path")
        if not isinstance(relative, str):
            errors.append(f"{task_id}: path must be a string")
            continue
        if expected is not None:
            expected_path = f"docs/architecture/decisions/{expected_filename}"
            if relative != expected_path:
                errors.append(f"{task_id}: ADR path must be {expected_path}")
        if not relative.startswith("docs/architecture/decisions/"):
            errors.append(f"{task_id}: ADR must live under docs/architecture/decisions/")
        if relative in seen_paths:
            errors.append(f"{task_id}: duplicate ADR path {relative}")
        seen_paths.add(relative)
        number = entry.get("adr_number")
        if not isinstance(number, str):
            errors.append(f"{task_id}: adr_number must be a string")
        elif number in seen_numbers:
            errors.append(f"{task_id}: duplicate ADR number {number}")
        else:
            seen_numbers.add(number)
        if entry.get("status") != "Accepted":
            errors.append(f"{task_id}: ADR status must be Accepted")
        summary = entry.get("decision_summary")
        if not isinstance(summary, str) or len(summary.strip()) < 40:
            errors.append(f"{task_id}: decision_summary is missing or too short")
        unblocks = entry.get("unblocks")
        if not isinstance(unblocks, list) or not unblocks or not all(
            isinstance(item, str) and item for item in unblocks
        ):
            errors.append(f"{task_id}: unblocks must be a non-empty string array")
        try:
            adr_path = resolve_adr_path(repo_root, relative)
        except DecisionValidationError as error:
            errors.append(f"{task_id}: {error}")
            continue
        errors.extend(validate_adr(adr_path, entry))
    return errors


def validate_current_record(repo_root: Path, record: dict[str, Any]) -> list[str]:
    errors: list[str] = []
    if set(record) != CURRENT_RECORD_KEYS:
        errors.append(
            "current decision registry must contain exactly "
            f"{sorted(CURRENT_RECORD_KEYS)}"
        )
    if record.get("schema_version") != "1.0":
        errors.append("current decision registry schema_version must be '1.0'")
    if record.get("record_type") != "qiongli-current-architecture-decision-registry":
        errors.append("current decision registry record_type is invalid")
    if record.get("branch") != "2.x":
        errors.append("current decision registry must be bound to branch '2.x'")

    decisions = record.get("decisions")
    if not isinstance(decisions, list):
        return [*errors, "current decision registry must contain a decisions array"]

    decisions_root = repo_root / "docs" / "architecture" / "decisions"
    actual_paths = [
        path.relative_to(repo_root).as_posix()
        for path in sorted(decisions_root.glob("[0-9][0-9][0-9][0-9]-*.md"))
        if path.is_file()
    ]
    registered_paths: list[str] = []
    seen_tasks: set[str] = set()
    seen_numbers: set[str] = set()
    seen_paths: set[str] = set()
    numbers: list[str] = []

    for index, entry in enumerate(decisions):
        if not isinstance(entry, dict):
            errors.append(f"current decisions[{index}] must be an object")
            continue
        if set(entry) != CURRENT_DECISION_KEYS:
            errors.append(
                f"current decisions[{index}] must contain exactly "
                f"{sorted(CURRENT_DECISION_KEYS)}"
            )

        task_id = entry.get("task_id")
        if not isinstance(task_id, str) or not task_id:
            errors.append(f"current decisions[{index}]: task_id must be a string")
        elif task_id in seen_tasks:
            errors.append(f"current decisions[{index}]: duplicate task ID {task_id}")
        else:
            seen_tasks.add(task_id)

        number = entry.get("adr_number")
        if not isinstance(number, str) or not re.fullmatch(r"\d{4}", number):
            errors.append(f"{task_id or index}: adr_number must be four digits")
        else:
            numbers.append(number)
            if number in seen_numbers:
                errors.append(f"{task_id or index}: duplicate ADR number {number}")
            seen_numbers.add(number)

        relative = entry.get("path")
        if not isinstance(relative, str):
            errors.append(f"{task_id or index}: path must be a string")
            continue
        registered_paths.append(relative)
        if relative in seen_paths:
            errors.append(f"{task_id or index}: duplicate ADR path {relative}")
        seen_paths.add(relative)
        if isinstance(number, str) and not PurePosixPath(relative).name.startswith(
            f"{number}-"
        ):
            errors.append(f"{task_id or index}: ADR number must match its filename")
        if entry.get("status") != "Accepted":
            errors.append(f"{task_id or index}: ADR status must be Accepted")

        try:
            adr_path = resolve_adr_path(repo_root, relative)
            content = adr_path.read_text(encoding="utf-8")
        except (DecisionValidationError, OSError) as error:
            errors.append(f"{task_id or index}: {error}")
            continue
        title = entry.get("title")
        expected_title = f"# ADR {number}: {title}"
        actual_title = content.splitlines()[0] if content else ""
        if actual_title != expected_title:
            errors.append(f"{relative}: title must be {expected_title!r}")
        if f"- Status: {entry.get('status')}" not in content:
            errors.append(f"{relative}: status metadata does not match the registry")
        if f"- Task ID: `{task_id}`" not in content:
            errors.append(f"{relative}: task metadata does not match the registry")

    if registered_paths != actual_paths:
        errors.append(
            "current decision paths must exactly match every numbered ADR Markdown "
            "file in filename order"
        )
    if numbers != sorted(numbers):
        errors.append("current decision ADR numbers must be in ascending order")
    return errors


def main(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Validate the frozen ARC-201 set and current ADR registry."
    )
    parser.add_argument("--record", type=Path, default=DEFAULT_RECORD)
    parser.add_argument(
        "--current-record", type=Path, default=DEFAULT_CURRENT_RECORD
    )
    args = parser.parse_args(argv)

    try:
        record = load_record(args.record)
        current_record = load_record(args.current_record)
    except DecisionValidationError as error:
        print(f"[arc-201] {error}", file=sys.stderr)
        return 2
    errors = [
        *validate_record(REPO_ROOT, record),
        *validate_current_record(REPO_ROOT, current_record),
    ]
    if errors:
        for error in errors:
            print(f"[arc-201] FAIL: {error}", file=sys.stderr)
        print(f"[arc-201] {len(errors)} validation error(s)", file=sys.stderr)
        return 1
    print(
        "[arc-201] PASS: 7 frozen decisions and "
        f"{len(current_record['decisions'])} current decisions are complete"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
