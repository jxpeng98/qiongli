#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
PYTHON_SOURCE_ROOT = REPO_ROOT / "packages" / "python-qiongli" / "src"
for import_root in (PYTHON_SOURCE_ROOT, REPO_ROOT):
    if str(import_root) not in sys.path:
        sys.path.insert(0, str(import_root))

from qiongli.source_layout import RepoLayout


VERSION_PATTERN = re.compile(
    r"^(?:v)?(?P<major>\d+)\.(?P<minor>\d+)\.(?P<patch>\d+)(?:(?:-beta\.|b)(?P<beta>\d+))?$"
)
PRINTABLE_FIELDS = ("package_version", "skill_version", "repo_version", "npm_version")


def parse_version(raw: str) -> tuple[str, str, str, str]:
    match = VERSION_PATTERN.fullmatch(raw.strip())
    if not match:
        raise ValueError(
            "unsupported version format. Use stable `X.Y.Z` or beta `X.Y.ZbN` / `vX.Y.Z-beta.N`."
        )

    major = int(match.group("major"))
    minor = int(match.group("minor"))
    patch = int(match.group("patch"))
    beta_raw = match.group("beta")

    if beta_raw is None:
        skill_version = f"{major}.{minor}.{patch}"
        package_version = skill_version
        npm_version = skill_version
        repo_version = f"v{skill_version}"
        return package_version, skill_version, repo_version, npm_version

    beta = int(beta_raw)
    package_version = f"{major}.{minor}.{patch}b{beta}"
    skill_version = f"{major}.{minor}.{patch}-beta.{beta}"
    npm_version = skill_version
    repo_version = f"v{skill_version}"
    return package_version, skill_version, repo_version, npm_version


def replace_pattern(path: Path, pattern: re.Pattern[str], replacement: str) -> bool:
    original = path.read_text(encoding="utf-8")
    updated, count = pattern.subn(replacement, original)
    if count == 0:
        raise ValueError(f"no matching version field found in {path}")
    if updated != original:
        path.write_text(updated, encoding="utf-8")
        return True
    return False


def replace_json_versions(path: Path, version: str) -> bool:
    original = path.read_text(encoding="utf-8")
    data = json.loads(original)
    changed = False

    def visit(value: object) -> None:
        nonlocal changed
        if isinstance(value, dict):
            for key, item in value.items():
                if key == "version" and item != version:
                    value[key] = version
                    changed = True
                else:
                    visit(item)
        elif isinstance(value, list):
            for item in value:
                visit(item)

    visit(data)
    if changed:
        path.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")
    return changed


def replace_npm_lock_workspace_version(path: Path, version: str) -> bool:
    original = path.read_text(encoding="utf-8")
    data = json.loads(original)
    packages = data.get("packages")
    if not isinstance(packages, dict):
        raise ValueError(f"missing packages object in {path}")
    workspace = packages.get("packages/npm-qiongli")
    if not isinstance(workspace, dict):
        raise ValueError(f"missing packages/npm-qiongli entry in {path}")
    if workspace.get("version") == version:
        return False
    workspace["version"] = version
    path.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")
    return True


def replace_skill_entrypoint_version(path: Path, repo_version: str) -> bool:
    original = path.read_text(encoding="utf-8")
    label = "Qiongli Next" if re.search(r"(?m)^name:\s*qiongli-next\s*$", original) else "Qiongli"
    updated = original
    description_line = re.compile(r"(?m)^(description:\s*)(.+)$")

    def replace_description(match: re.Match[str]) -> str:
        current = _unquote_yaml_like_string(match.group(2).strip())
        prefix = re.compile(
            r"^(?:Qiongli(?: Next)? version:\s*)"
            r"v?\d+\.\d+\.\d+(?:-beta\.\d+)?\.\s*"
        )
        body = prefix.sub("", current, count=1)
        description = f"{label} version: {repo_version}. {body}"
        return f"{match.group(1)}{json.dumps(description, ensure_ascii=False)}"

    updated, description_count = description_line.subn(replace_description, updated, count=1)
    if description_count == 0:
        raise ValueError(f"no matching description field found in {path}")

    updated, body_count = re.subn(
        r"Installed Qiongli workflow version: `[^`]+`",
        f"Installed Qiongli workflow version: `{repo_version}`",
        updated,
        count=1,
    )
    if body_count == 0:
        updated = updated.rstrip() + f"\n\nInstalled Qiongli workflow version: `{repo_version}`\n"

    if updated != original:
        path.write_text(updated, encoding="utf-8")
        return True
    return False


def _unquote_yaml_like_string(value: str) -> str:
    if len(value) >= 2 and value[0] == value[-1] == '"':
        try:
            decoded = json.loads(value)
        except json.JSONDecodeError:
            return value[1:-1]
        return decoded if isinstance(decoded, str) else value
    if len(value) >= 2 and value[0] == value[-1] == "'":
        return value[1:-1].replace("''", "'")
    return value


def replace_uv_lock_editable_package_version(path: Path, package_name: str, version: str) -> bool:
    original = path.read_text(encoding="utf-8")
    package_block_pattern = re.compile(
        rf'(?ms)^(\[\[package\]\]\nname = "{re.escape(package_name)}"\nversion = ")[^"]+(".*?source = \{{ editable = "\." \}}.*?)(?=^\[\[package\]\]|\Z)'
    )
    updated, count = package_block_pattern.subn(rf"\g<1>{version}\g<2>", original, count=1)
    if count == 0:
        raise ValueError(f"missing editable package {package_name!r} in {path}")
    if updated == original:
        return False
    path.write_text(updated, encoding="utf-8")
    return True


def sync_versions(root: Path, raw_version: str) -> list[Path]:
    root = root.resolve()
    package_version, skill_version, repo_version, npm_version = parse_version(raw_version)
    changed: list[Path] = []
    layout = RepoLayout(root)

    replacements: list[tuple[Path, re.Pattern[str], str]] = [
        (
            root / "pyproject.toml",
            re.compile(r'^version = "[^"]+"$', re.MULTILINE),
            f'version = "{package_version}"',
        ),
        (
            layout.python_package / "__init__.py",
            re.compile(r'^__version__ = "[^"]+"$', re.MULTILINE),
            f'__version__ = "{package_version}"',
        ),
    ]

    for path, pattern, replacement in replacements:
        if replace_pattern(path, pattern, replacement):
            changed.append(path)

    registry_files = (
        layout.skills / "registry.yaml",
        root / "qiongli-workflow" / "skills" / "registry.yaml",
        root / "packages" / "npm-qiongli" / "payload" / "qiongli-workflow" / "skills" / "registry.yaml",
        root / "packages" / "npm-qiongli" / "python-runtime" / "skills" / "registry.yaml",
    )
    for registry_file in registry_files:
        if not registry_file.exists():
            continue
        if replace_pattern(
            registry_file,
            re.compile(r'^(\s*version:\s*)"?[^"\n]+"?$', re.MULTILINE),
            rf'\g<1>"{skill_version}"',
        ):
            changed.append(registry_file)

    workflow_version_files = (
        layout.workflow / "VERSION",
        root / "qiongli-workflow" / "VERSION",
        root / "packages" / "npm-qiongli" / "payload" / "qiongli-workflow" / "VERSION",
    )
    for workflow_version_file in workflow_version_files:
        if not workflow_version_file.exists():
            continue
        original_repo_version = workflow_version_file.read_text(encoding="utf-8").strip()
        if original_repo_version != repo_version:
            workflow_version_file.write_text(repo_version + "\n", encoding="utf-8")
            changed.append(workflow_version_file)

    skill_entrypoint_files = (
        layout.workflow / "SKILL.md",
        root / "qiongli-workflow" / "SKILL.md",
        root / "packages" / "npm-qiongli" / "payload" / "qiongli-workflow" / "SKILL.md",
    )
    for skill_entrypoint in skill_entrypoint_files:
        if not skill_entrypoint.exists():
            continue
        if replace_skill_entrypoint_version(skill_entrypoint, repo_version):
            changed.append(skill_entrypoint)

    npm_manifest = root / "packages" / "npm-qiongli" / "package.json"
    if npm_manifest.exists():
        if replace_json_versions(npm_manifest, npm_version):
            changed.append(npm_manifest)

    npm_lock = root / "package-lock.json"
    if npm_lock.exists():
        if replace_npm_lock_workspace_version(npm_lock, npm_version):
            changed.append(npm_lock)

    uv_lock = root / "uv.lock"
    if uv_lock.exists():
        if replace_uv_lock_editable_package_version(uv_lock, "qiongli", package_version):
            changed.append(uv_lock)

    bundled_python_init_files = (
        root / "packages" / "npm-qiongli" / "python-runtime" / "qiongli" / "__init__.py",
    )
    for bundled_python_init in bundled_python_init_files:
        if not bundled_python_init.exists():
            continue
        if replace_pattern(
            bundled_python_init,
            re.compile(r'^__version__ = "[^"]+"$', re.MULTILINE),
            f'__version__ = "{package_version}"',
        ):
            changed.append(bundled_python_init)

    return changed


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(
        description="Sync package, portable skill, and skill metadata versions from one release version."
    )
    parser.add_argument("version", help="Stable or beta version, e.g. 0.2.0 or 0.2.0b1")
    parser.add_argument(
        "--print-field",
        choices=PRINTABLE_FIELDS,
        help="Print one normalized version field and exit without writing files.",
    )
    parser.add_argument(
        "--root",
        default=Path(__file__).resolve().parents[2],
        type=Path,
        help="Repository root (defaults to current repo root)",
    )
    args = parser.parse_args(argv)

    package_version, skill_version, repo_version, npm_version = parse_version(args.version)
    if args.print_field:
        print(
            {
                "package_version": package_version,
                "skill_version": skill_version,
                "repo_version": repo_version,
                "npm_version": npm_version,
            }[args.print_field]
        )
        return 0

    root = args.root.resolve()
    changed = sync_versions(root, args.version)

    print("[sync-versions] normalized versions")
    print(f"  - package_version: {package_version}")
    print(f"  - skill_version:   {skill_version}")
    print(f"  - repo_version:    {repo_version}")
    print(f"  - npm_version:     {npm_version}")
    print(f"  - changed_files:   {len(changed)}")
    for path in changed:
        print(f"    - {path.relative_to(root)}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except ValueError as exc:
        print(f"[sync-versions] {exc}", file=sys.stderr)
        raise SystemExit(2)
