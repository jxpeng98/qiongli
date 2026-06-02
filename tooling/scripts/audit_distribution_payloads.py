#!/usr/bin/env python3
from __future__ import annotations

import argparse
import hashlib
import json
import shutil
import sys
import tempfile
from dataclasses import dataclass
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
PYTHON_SOURCE_ROOT = REPO_ROOT / "packages" / "python-qiongli" / "src"
for import_root in (PYTHON_SOURCE_ROOT, REPO_ROOT):
    if str(import_root) not in sys.path:
        sys.path.insert(0, str(import_root))

from qiongli.source_layout import RepoLayout
from qiongli.subject_materializer import MaterializeOptions, materialize_subject_package, validate_subject_catalog


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


def _copy_path(src: Path, dest: Path, *, extra_excluded_names: set[str] | None = None) -> None:
    if src.is_dir():
        shutil.copytree(
            src,
            dest,
            ignore=lambda _copy_src, names: {
                name for name in names if extra_excluded_names and name in extra_excluded_names
            },
        )
        return
    dest.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(src, dest)


def _build_materialize_source(root: Path, tmp_root: Path) -> Path:
    layout = RepoLayout(root)
    source = tmp_root / "source"
    _copy_path(layout.workflow, source / "qiongli-workflow")
    _copy_path(layout.skills, source / "skills")
    _copy_path(layout.subjects, source / "subjects")
    package_dirs = {
        "skills": layout.skills,
        "templates": layout.templates,
        "standards": layout.standards,
        "roles": layout.roles,
        "venue-profiles": layout.venue_profiles,
    }
    for item, src in package_dirs.items():
        if src.exists():
            extra_excludes = SKILL_PACKAGE_EXCLUDED_NAMES if item == "templates" else None
            _copy_path(src, source / "qiongli-workflow" / item, extra_excluded_names=extra_excludes)
    package_files = {
        "skills-core.md": layout.skills_core,
        "skills-summary.md": layout.skills_summary,
    }
    for item, src in package_files.items():
        if src.exists():
            _copy_path(src, source / "qiongli-workflow" / item)
    return source


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
    with tempfile.TemporaryDirectory(prefix="qiongli-subject-payload-audit-") as tmp:
        tmp_root = Path(tmp)
        materialize_source = _build_materialize_source(root, tmp_root / "materialize")
        expected_root = tmp_root / "expected"
        for subject in sorted(catalog.subjects):
            for coverage in ("complete", "focused"):
                workflow = subjects_root / subject / coverage / "qiongli-workflow"
                subject_label = f"{label} {subject}/{coverage}"
                manifest_path = workflow / "SUBJECT_MANIFEST.json"
                manifest, manifest_issues = _read_json(manifest_path, subject_label)
                issues.extend(manifest_issues)
                if manifest is not None:
                    if manifest.get("subject") != subject:
                        issues.append(
                            AuditIssue(
                                subject_label,
                                f"{manifest_path} expected subject {subject}, found {manifest.get('subject')}",
                            )
                        )
                    if manifest.get("coverage") != coverage:
                        issues.append(
                            AuditIssue(
                                subject_label,
                                f"{manifest_path} expected coverage {coverage}, found {manifest.get('coverage')}",
                            )
                        )

                expected = expected_root / subject / coverage / "qiongli-workflow"
                materialize_subject_package(
                    MaterializeOptions(
                        source=materialize_source,
                        out=expected,
                        subject=subject,
                        flavor="full",
                        coverage=coverage,
                    )
                )
                issues.extend(_compare_trees(expected, workflow, subject_label))
    return issues


def _audit_generated_skill_package(root: Path, generated: Path, label: str) -> list[AuditIssue]:
    issues: list[AuditIssue] = []
    layout = RepoLayout(root)
    legacy_workflow = root / "qiongli-workflow"
    workflow = legacy_workflow if legacy_workflow.exists() else layout.workflow
    issues.extend(
        _compare_trees(
            workflow,
            generated,
            f"{label} base package vs portable package",
            extra_excluded_names=GENERATED_PACKAGE_EXCLUDED_NAMES,
        )
    )
    source_dirs = {
        "skills": layout.skills,
        "templates": layout.templates,
        "standards": layout.standards,
        "roles": layout.roles,
        "venue-profiles": layout.venue_profiles,
    }
    for dir_name in GENERATED_PACKAGE_DIRS:
        extra_excludes = SKILL_PACKAGE_EXCLUDED_NAMES if dir_name == "templates" else set()
        issues.extend(
            _compare_trees(
                source_dirs[dir_name],
                generated / dir_name,
                f"{label} {dir_name}/ vs source {dir_name}/",
                extra_excluded_names=extra_excludes,
            )
        )
    source_files = {
        "skills-core.md": layout.skills_core,
        "skills-summary.md": layout.skills_summary,
    }
    for file_name in GENERATED_PACKAGE_FILES:
        issues.extend(_compare_files(source_files[file_name], generated / file_name, f"{label} {file_name} vs source"))
    return issues


def audit(root: Path) -> list[AuditIssue]:
    root = root.resolve()
    layout = RepoLayout(root)
    workflow = root / "qiongli-workflow"
    plugin_workflow = root / "plugins" / "qiongli" / "skills" / "qiongli-workflow"
    npm_payload = root / "packages" / "npm-qiongli" / "payload" / "qiongli-workflow"
    npm_payload_subjects = root / "packages" / "npm-qiongli" / "payload" / "subjects"
    npm_runtime = root / "packages" / "npm-qiongli" / "python-runtime"
    python_payload = layout.python_package / "payload"

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
        source_dirs = {
            "skills": layout.skills,
            "subjects": layout.subjects,
        }
        issues.extend(
            _compare_trees(
                source_dirs[dir_name],
                python_payload / dir_name,
                f"python payload {dir_name}/ vs source {dir_name}/",
                extra_excluded_names=PYTHON_PAYLOAD_EXTRA_EXCLUDES.get(dir_name),
            )
        )

    issues.extend(_audit_subject_payloads(root, npm_payload_subjects, "npm subject payload coverage"))
    if python_payload.exists():
        issues.extend(_audit_subject_payloads(root, python_payload / "subjects", "python subject payload coverage"))

    for dir_name in NPM_RUNTIME_DIRS:
        runtime_sources = {
            "bridges": layout.bridges_compat_package,
            "qiongli": layout.python_package,
            "scripts": layout.scripts,
            "standards": layout.standards,
            "templates": layout.templates,
            "roles": layout.roles,
            "venue-profiles": layout.venue_profiles,
            "skills": layout.skills,
            "subjects": layout.subjects,
            "pipelines": layout.pipelines,
            "schemas": layout.schemas,
            "evals": root / "evals",
        }
        src = runtime_sources[dir_name]
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
        runtime_files = {
            "skills-core.md": layout.skills_core,
            "skills-summary.md": layout.skills_summary,
            "LICENSE": root / "LICENSE",
        }
        src = runtime_files[file_name]
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
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[2])
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
