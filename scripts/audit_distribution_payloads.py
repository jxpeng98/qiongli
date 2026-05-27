#!/usr/bin/env python3
from __future__ import annotations

import argparse
import hashlib
import json
from dataclasses import dataclass
from pathlib import Path

from qiongli.subject_materializer import validate_subject_catalog


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
PYTHON_PAYLOAD_SOURCE_DIRS = ("skills", "subjects")
PYTHON_PAYLOAD_EXTRA_EXCLUDES = {
    "subjects": {"complete", "focused"},
}
GENERATED_PACKAGE_DIRS = ("skills", "templates", "standards", "roles", "venue-profiles")
GENERATED_PACKAGE_FILES = ("skills-core.md", "skills-summary.md")
GENERATED_PACKAGE_EXCLUDED_NAMES = set(GENERATED_PACKAGE_DIRS) | set(GENERATED_PACKAGE_FILES)


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


def _read_json(path: Path, label: str) -> tuple[dict[str, object] | None, list[AuditIssue]]:
    if not path.is_file():
        return None, [AuditIssue(label, f"missing: {path}")]
    if path.is_symlink():
        return None, [AuditIssue(label, f"symlink: {path}")]
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        return None, [AuditIssue(label, f"malformed JSON: {path}: {exc}")]
    if not isinstance(payload, dict):
        return None, [AuditIssue(label, f"manifest must be a JSON object: {path}")]
    return payload, []


def _audit_subject_payloads(root: Path, subjects_root: Path, label: str) -> list[AuditIssue]:
    issues: list[AuditIssue] = []
    if not subjects_root.exists():
        issues.append(AuditIssue(label, f"missing: {subjects_root}"))
        return issues

    catalog = validate_subject_catalog(root)
    for subject in sorted(catalog.subjects):
        for coverage in ("complete", "focused"):
            workflow = subjects_root / subject / coverage / "qiongli-workflow"
            manifest_path = workflow / "SUBJECT_MANIFEST.json"
            manifest, manifest_issues = _read_json(manifest_path, label)
            issues.extend(manifest_issues)
            if manifest is None:
                continue
            if manifest.get("subject") != subject:
                issues.append(AuditIssue(label, f"{manifest_path} expected subject {subject}, found {manifest.get('subject')}"))
            if manifest.get("coverage") != coverage:
                issues.append(AuditIssue(label, f"{manifest_path} expected coverage {coverage}, found {manifest.get('coverage')}"))
    return issues


def _audit_generated_skill_package(root: Path, generated: Path, label: str) -> list[AuditIssue]:
    issues: list[AuditIssue] = []
    workflow = root / "qiongli-workflow"
    issues.extend(
        _compare_trees(
            workflow,
            generated,
            f"{label} base package vs portable package",
            extra_excluded_names=GENERATED_PACKAGE_EXCLUDED_NAMES,
        )
    )
    for dir_name in GENERATED_PACKAGE_DIRS:
        extra_excludes = SKILL_PACKAGE_EXCLUDED_NAMES if dir_name == "templates" else set()
        issues.extend(
            _compare_trees(
                root / dir_name,
                generated / dir_name,
                f"{label} {dir_name}/ vs source {dir_name}/",
                extra_excluded_names=extra_excludes,
            )
        )
    for file_name in GENERATED_PACKAGE_FILES:
        issues.extend(_compare_files(root / file_name, generated / file_name, f"{label} {file_name} vs source"))
    return issues


def audit(root: Path) -> list[AuditIssue]:
    root = root.resolve()
    workflow = root / "qiongli-workflow"
    plugin_workflow = root / "plugins" / "qiongli" / "skills" / "qiongli-workflow"
    npm_payload = root / "packages" / "npm-qiongli" / "payload" / "qiongli-workflow"
    npm_payload_subjects = root / "packages" / "npm-qiongli" / "payload" / "subjects"
    npm_runtime = root / "packages" / "npm-qiongli" / "python-runtime"
    python_payload = root / "qiongli" / "payload"

    issues: list[AuditIssue] = []

    for label, path in (
        ("portable package", workflow),
        ("plugin mirror", plugin_workflow),
        ("npm payload", npm_payload),
        ("npm subject payloads", npm_payload_subjects),
        ("npm python-runtime", npm_runtime),
        ("python bundled payload", python_payload),
    ):
        issues.extend(_assert_no_symlinks(path, label))

    issues.extend(_audit_generated_skill_package(root, plugin_workflow, "plugin mirror"))
    issues.extend(_audit_generated_skill_package(root, npm_payload, "npm payload"))
    issues.extend(_audit_generated_skill_package(root, python_payload / "qiongli-workflow", "python payload"))

    for dir_name in PYTHON_PAYLOAD_SOURCE_DIRS:
        issues.extend(
            _compare_trees(
                root / dir_name,
                python_payload / dir_name,
                f"python payload {dir_name}/ vs source {dir_name}/",
                extra_excluded_names=PYTHON_PAYLOAD_EXTRA_EXCLUDES.get(dir_name),
            )
        )

    issues.extend(_audit_subject_payloads(root, npm_payload_subjects, "npm subject payload coverage"))
    if python_payload.exists():
        issues.extend(_audit_subject_payloads(root, python_payload / "subjects", "python subject payload coverage"))

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
