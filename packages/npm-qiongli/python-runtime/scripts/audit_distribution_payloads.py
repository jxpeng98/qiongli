#!/usr/bin/env python3
from __future__ import annotations

import argparse
import hashlib
import json
from dataclasses import dataclass
from pathlib import Path


EXCLUDED_NAMES = {
    ".DS_Store",
    "__pycache__",
    ".pytest_cache",
    ".mypy_cache",
    "node_modules",
    "dist",
    "build",
}
EXCLUDED_SUFFIXES = {".pyc", ".pyo"}
SKILL_SYNC_DIRS = ("skills", "templates", "standards", "roles", "venue-profiles")
SKILL_SYNC_FILES = ("skills-core.md", "skills-summary.md")
SKILL_PACKAGE_EXCLUDED_NAMES = {"CLAUDE.project.md"}
NPM_RUNTIME_DIRS = (
    "bridges",
    "qiongli",
    "scripts",
    "standards",
    "templates",
    "roles",
    "venue-profiles",
    "skills",
    "subjects",
    "pipelines",
    "schemas",
    "evals",
)
NPM_RUNTIME_FILES = ("skills-core.md", "skills-summary.md", "LICENSE")
NPM_RUNTIME_EXTRA_EXCLUDES = {
    "qiongli": {"payload"},
}


@dataclass(frozen=True)
class AuditIssue:
    label: str
    detail: str


def _is_excluded(path: Path, root: Path, extra_names: set[str] | None = None) -> bool:
    names = EXCLUDED_NAMES | (extra_names or set())
    try:
        rel = path.relative_to(root)
    except ValueError:
        return False
    return any(part in names for part in rel.parts) or any(path.name.endswith(suffix) for suffix in EXCLUDED_SUFFIXES)


def _hash_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def _file_map(root: Path, *, extra_excluded_names: set[str] | None = None) -> tuple[dict[str, str], list[AuditIssue]]:
    files: dict[str, str] = {}
    issues: list[AuditIssue] = []
    if not root.exists():
        issues.append(AuditIssue("missing", str(root)))
        return files, issues
    if root.is_symlink():
        issues.append(AuditIssue("symlink", str(root)))
        return files, issues
    if root.is_file():
        files[root.name] = _hash_file(root)
        return files, issues

    for item in sorted(root.rglob("*")):
        if _is_excluded(item, root, extra_excluded_names):
            continue
        if item.is_symlink():
            issues.append(AuditIssue("symlink", str(item)))
            continue
        if item.is_file():
            files[item.relative_to(root).as_posix()] = _hash_file(item)
    return files, issues


def _compare_trees(
    left: Path,
    right: Path,
    label: str,
    *,
    extra_excluded_names: set[str] | None = None,
) -> list[AuditIssue]:
    issues: list[AuditIssue] = []
    left_files, left_issues = _file_map(left, extra_excluded_names=extra_excluded_names)
    right_files, right_issues = _file_map(right, extra_excluded_names=extra_excluded_names)
    issues.extend(AuditIssue(label, f"{issue.label}: {issue.detail}") for issue in left_issues)
    issues.extend(AuditIssue(label, f"{issue.label}: {issue.detail}") for issue in right_issues)

    left_keys = set(left_files)
    right_keys = set(right_files)
    for missing in sorted(left_keys - right_keys):
        issues.append(AuditIssue(label, f"missing in {right}: {missing}"))
    for extra in sorted(right_keys - left_keys):
        issues.append(AuditIssue(label, f"extra in {right}: {extra}"))
    for rel in sorted(left_keys & right_keys):
        if left_files[rel] != right_files[rel]:
            issues.append(AuditIssue(label, f"content mismatch: {rel}"))
    return issues


def _compare_files(left: Path, right: Path, label: str) -> list[AuditIssue]:
    issues: list[AuditIssue] = []
    for path in (left, right):
        if not path.exists():
            issues.append(AuditIssue(label, f"missing: {path}"))
        elif path.is_symlink():
            issues.append(AuditIssue(label, f"symlink: {path}"))
        elif not path.is_file():
            issues.append(AuditIssue(label, f"not a file: {path}"))
    if issues:
        return issues
    if _hash_file(left) != _hash_file(right):
        issues.append(AuditIssue(label, f"content mismatch: {left} != {right}"))
    return issues


def _assert_no_symlinks(root: Path, label: str) -> list[AuditIssue]:
    issues: list[AuditIssue] = []
    if not root.exists():
        issues.append(AuditIssue(label, f"missing: {root}"))
        return issues
    if root.is_symlink():
        return [AuditIssue(label, f"symlink: {root}")]
    for item in sorted(root.rglob("*")):
        if item.is_symlink():
            issues.append(AuditIssue(label, f"symlink: {item}"))
    return issues


def audit(root: Path) -> list[AuditIssue]:
    root = root.resolve()
    workflow = root / "qiongli-workflow"
    plugin_workflow = root / "plugins" / "qiongli" / "skills" / "qiongli-workflow"
    npm_payload = root / "packages" / "npm-qiongli" / "payload" / "qiongli-workflow"
    npm_runtime = root / "packages" / "npm-qiongli" / "python-runtime"

    issues: list[AuditIssue] = []

    for label, path in (
        ("portable package", workflow),
        ("plugin mirror", plugin_workflow),
        ("npm payload", npm_payload),
        ("npm python-runtime", npm_runtime),
    ):
        issues.extend(_assert_no_symlinks(path, label))

    issues.extend(_compare_trees(workflow, plugin_workflow, "plugin mirror vs portable package"))
    issues.extend(_compare_trees(workflow, npm_payload, "npm payload vs portable package"))

    for dir_name in SKILL_SYNC_DIRS:
        extra_excludes = SKILL_PACKAGE_EXCLUDED_NAMES if dir_name == "templates" else set()
        issues.extend(
            _compare_trees(
                root / dir_name,
                workflow / dir_name,
                f"portable {dir_name}/ vs source {dir_name}/",
                extra_excluded_names=extra_excludes,
            )
        )
    for file_name in SKILL_SYNC_FILES:
        issues.extend(_compare_files(root / file_name, workflow / file_name, f"portable {file_name} vs source"))

    for dir_name in NPM_RUNTIME_DIRS:
        src = root / dir_name
        if src.exists():
            issues.extend(
                _compare_trees(
                    src,
                    npm_runtime / dir_name,
                    f"npm runtime {dir_name}/ vs source {dir_name}/",
                    extra_excluded_names=NPM_RUNTIME_EXTRA_EXCLUDES.get(dir_name),
                )
            )
    for file_name in NPM_RUNTIME_FILES:
        src = root / file_name
        if src.exists():
            issues.extend(_compare_files(src, npm_runtime / file_name, f"npm runtime {file_name} vs source"))
    issues.extend(_compare_files(root / "LICENSE", root / "packages" / "npm-qiongli" / "LICENSE", "npm LICENSE vs source"))

    package_json = root / "packages" / "npm-qiongli" / "package.json"
    workflow_version = workflow / "VERSION"
    if package_json.exists() and workflow_version.exists():
        npm_version = json.loads(package_json.read_text(encoding="utf-8"))["version"]
        expected = workflow_version.read_text(encoding="utf-8").strip().removeprefix("v")
        if npm_version != expected:
            issues.append(AuditIssue("npm package version", f"expected {expected}, found {npm_version}"))

    return issues


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Audit generated distribution payloads against canonical repo sources.")
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    args = parser.parse_args(argv)

    issues = audit(args.root)
    if issues:
        for issue in issues:
            print(f"[FAIL] {issue.label}: {issue.detail}")
        return 1
    print("[OK] distribution payloads match canonical sources and contain no symlinks")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
