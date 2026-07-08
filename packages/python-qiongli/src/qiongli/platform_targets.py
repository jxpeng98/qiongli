from __future__ import annotations

import argparse
from dataclasses import dataclass
from fnmatch import fnmatch
from pathlib import Path
import sys
from typing import Any

import yaml


REGISTRY_RELATIVE_PATH = Path("content") / "distribution" / "platform-targets.yaml"
REQUIRED_FIELDS = (
    "display_name",
    "artifact_kind",
    "archive_format",
    "adapter",
    "source_inputs",
    "required_paths",
    "allowed_wrapper_dirs",
    "forbidden_paths",
    "bundled_mcp_mode",
    "command_surface",
    "validator",
)
STRUCTURAL_ARCHIVE_CHECKS = frozenset(
    {
        "marketplace_validation",
        "package_build_validation",
    }
)
CLIENT_ACTIVATION_CHECKS = frozenset(
    {
        "local_install_acceptance",
        "not_applicable",
    }
)
ADAPTER_KINDS = frozenset(
    {
        "plugin",
        "skill-zip",
        "local-plugin",
        "package",
    }
)
ADAPTER_MANIFEST_PLATFORMS = frozenset(
    {
        "codex",
        "claude",
        "none",
    }
)
ADAPTER_MATERIALIZERS = frozenset(
    {
        "plugin_artifacts",
        "desktop_skill_artifacts",
        "local_plugin_installer",
        "npm_package",
        "python_package",
    }
)
ADAPTER_KIND_MANIFEST_PLATFORMS = {
    "plugin": frozenset({"codex", "claude"}),
    "skill-zip": frozenset({"none"}),
    "local-plugin": frozenset({"none"}),
    "package": frozenset({"none"}),
}
ADAPTER_KIND_MATERIALIZERS = {
    "plugin": frozenset({"plugin_artifacts"}),
    "skill-zip": frozenset({"desktop_skill_artifacts"}),
    "local-plugin": frozenset({"local_plugin_installer"}),
    "package": frozenset({"npm_package", "python_package"}),
}


@dataclass(frozen=True)
class PlatformTarget:
    target_id: str
    display_name: str
    artifact_kind: str
    archive_format: str
    adapter: dict[str, str]
    smoke: dict[str, str]
    source_inputs: tuple[str, ...]
    required_paths: tuple[str, ...]
    allowed_wrapper_dirs: tuple[str, ...]
    forbidden_paths: tuple[str, ...]
    bundled_mcp_mode: str
    command_surface: str
    validator: str
    release_download: dict[str, Any]


def load_platform_targets(repo_root: Path | str | None = None) -> dict[str, PlatformTarget]:
    root = _repo_root(repo_root)
    registry_path = root / REGISTRY_RELATIVE_PATH
    if not registry_path.is_file():
        raise ValueError(f"missing platform target registry: {registry_path}")

    payload = yaml.safe_load(registry_path.read_text(encoding="utf-8")) or {}
    if not isinstance(payload, dict):
        raise ValueError(f"{registry_path} must contain a YAML object")
    if payload.get("schema_version") != "1.0":
        raise ValueError(f"{registry_path} schema_version must be 1.0")

    raw_targets = payload.get("targets")
    if not isinstance(raw_targets, dict) or not raw_targets:
        raise ValueError(f"{registry_path} must define targets")

    targets: dict[str, PlatformTarget] = {}
    for target_id, raw_target in raw_targets.items():
        if not isinstance(target_id, str) or not target_id:
            raise ValueError(f"{registry_path} contains an invalid target id: {target_id!r}")
        if not isinstance(raw_target, dict):
            raise ValueError(f"{registry_path} target {target_id} must be an object")
        targets[target_id] = _parse_target(registry_path, target_id, raw_target)
    return targets


def require_platform_target(targets: dict[str, PlatformTarget], target_id: str) -> PlatformTarget:
    try:
        return targets[target_id]
    except KeyError as exc:
        raise ValueError(f"platform target registry missing target: {target_id}") from exc


def validate_platform_target_registry(repo_root: Path | str | None = None) -> list[str]:
    try:
        load_platform_targets(repo_root)
    except ValueError as exc:
        return [str(exc)]
    return []


def missing_required_paths(root: Path, target: PlatformTarget) -> list[str]:
    return [pattern for pattern in target.required_paths if not _pattern_exists(root, pattern)]


def present_forbidden_paths(root: Path, target: PlatformTarget) -> list[str]:
    return [pattern for pattern in target.forbidden_paths if _pattern_exists(root, pattern)]


def plugin_manifest_platform(target: PlatformTarget) -> str:
    value = target.adapter.get("plugin_manifest_platform")
    if not isinstance(value, str) or not value:
        raise ValueError(
            f"platform target {target.target_id} adapter.plugin_manifest_platform must be a non-empty string"
        )
    return value


def remove_path_pattern(root: Path, pattern: str) -> None:
    clean_pattern = pattern.rstrip("/")
    if pattern.endswith("/"):
        if _has_wildcard(clean_pattern):
            for path in root.glob(clean_pattern):
                if path.is_dir():
                    _remove_path(path)
        else:
            path = root / clean_pattern
            if path.is_dir():
                _remove_path(path)
        return

    for path in root.glob(clean_pattern):
        if path.exists() or path.is_symlink():
            _remove_path(path)


def _parse_target(registry_path: Path, target_id: str, raw_target: dict[str, Any]) -> PlatformTarget:
    missing = [field for field in REQUIRED_FIELDS if field not in raw_target]
    if missing:
        raise ValueError(f"{registry_path} target {target_id} missing fields: {', '.join(missing)}")

    return PlatformTarget(
        target_id=target_id,
        display_name=_required_string(registry_path, target_id, raw_target, "display_name"),
        artifact_kind=_required_string(registry_path, target_id, raw_target, "artifact_kind"),
        archive_format=_required_string(registry_path, target_id, raw_target, "archive_format"),
        adapter=_adapter_mapping(registry_path, target_id, raw_target),
        smoke=_smoke_mapping(registry_path, target_id, raw_target),
        source_inputs=_string_tuple(registry_path, target_id, raw_target, "source_inputs"),
        required_paths=_string_tuple(
            registry_path,
            target_id,
            raw_target,
            "required_paths",
            empty_message="missing positive required_path checks",
        ),
        allowed_wrapper_dirs=_string_tuple(registry_path, target_id, raw_target, "allowed_wrapper_dirs"),
        forbidden_paths=_string_tuple(
            registry_path,
            target_id,
            raw_target,
            "forbidden_paths",
            empty_message="missing negative forbidden_path checks",
        ),
        bundled_mcp_mode=_required_string(registry_path, target_id, raw_target, "bundled_mcp_mode"),
        command_surface=_required_string(registry_path, target_id, raw_target, "command_surface"),
        validator=_required_string(registry_path, target_id, raw_target, "validator"),
        release_download=_release_download_mapping(registry_path, target_id, raw_target),
    )


def _required_string(registry_path: Path, target_id: str, raw_target: dict[str, Any], field: str) -> str:
    value = raw_target.get(field)
    if not isinstance(value, str) or not value:
        raise ValueError(f"{registry_path} target {target_id}.{field} must be a non-empty string")
    return value


def _string_tuple(
    registry_path: Path,
    target_id: str,
    raw_target: dict[str, Any],
    field: str,
    *,
    empty_message: str | None = None,
) -> tuple[str, ...]:
    value = raw_target.get(field)
    if not isinstance(value, list):
        raise ValueError(f"{registry_path} target {target_id}.{field} must be a list")
    strings: list[str] = []
    for item in value:
        if not isinstance(item, str) or not item:
            raise ValueError(f"{registry_path} target {target_id}.{field} must contain only non-empty strings")
        strings.append(item)
    if empty_message is not None and not strings:
        raise ValueError(f"{registry_path} target {target_id} {empty_message}")
    return tuple(strings)


def _release_download_mapping(registry_path: Path, target_id: str, raw_target: dict[str, Any]) -> dict[str, Any]:
    field = "release_download"
    value = raw_target.get(field)
    if not isinstance(value, dict):
        raise ValueError(f"{registry_path} target {target_id}.{field} must be an object")
    for required_string in ("guide_label", "recommended_key"):
        if not isinstance(value.get(required_string), str) or not value[required_string]:
            raise ValueError(
                f"{registry_path} target {target_id}.{field}.{required_string} must be a non-empty string"
            )
    asset_groups = value.get("asset_groups")
    if not isinstance(asset_groups, list) or not all(isinstance(item, str) for item in asset_groups):
        raise ValueError(f"{registry_path} target {target_id}.{field}.asset_groups must be a list of strings")
    return dict(value)


def _adapter_mapping(registry_path: Path, target_id: str, raw_target: dict[str, Any]) -> dict[str, str]:
    field = "adapter"
    value = raw_target.get(field)
    if not isinstance(value, dict):
        raise ValueError(f"{registry_path} target {target_id}.{field} must be an object")
    for required_string in ("kind", "plugin_manifest_platform", "materializer"):
        if not isinstance(value.get(required_string), str) or not value[required_string]:
            raise ValueError(
                f"{registry_path} target {target_id}.{field}.{required_string} must be a non-empty string"
            )
    strings: dict[str, str] = {}
    for key, item in value.items():
        if not isinstance(key, str) or not isinstance(item, str) or not item:
            raise ValueError(f"{registry_path} target {target_id}.{field} must contain only string values")
        strings[key] = item
    kind = strings["kind"]
    if kind not in ADAPTER_KINDS:
        allowed = ", ".join(sorted(ADAPTER_KINDS))
        raise ValueError(
            f"{registry_path} target {target_id}.{field}.kind must be one of: {allowed}"
        )
    manifest_platform = strings["plugin_manifest_platform"]
    if manifest_platform not in ADAPTER_MANIFEST_PLATFORMS:
        allowed = ", ".join(sorted(ADAPTER_MANIFEST_PLATFORMS))
        raise ValueError(
            f"{registry_path} target {target_id}.{field}.plugin_manifest_platform must be one of: {allowed}"
        )
    materializer = strings["materializer"]
    if materializer not in ADAPTER_MATERIALIZERS:
        allowed = ", ".join(sorted(ADAPTER_MATERIALIZERS))
        raise ValueError(
            f"{registry_path} target {target_id}.{field}.materializer must be one of: {allowed}"
        )
    allowed_manifest_platforms = ADAPTER_KIND_MANIFEST_PLATFORMS[kind]
    if manifest_platform not in allowed_manifest_platforms:
        allowed = ", ".join(sorted(allowed_manifest_platforms))
        raise ValueError(
            f"{registry_path} target {target_id}.{field}.plugin_manifest_platform={manifest_platform} "
            f"is not valid for adapter.kind={kind}; expected one of: {allowed}"
        )
    allowed_materializers = ADAPTER_KIND_MATERIALIZERS[kind]
    if materializer not in allowed_materializers:
        allowed = ", ".join(sorted(allowed_materializers))
        raise ValueError(
            f"{registry_path} target {target_id}.{field}.materializer={materializer} "
            f"is not valid for adapter.kind={kind}; expected one of: {allowed}"
        )
    return strings


def _smoke_mapping(registry_path: Path, target_id: str, raw_target: dict[str, Any]) -> dict[str, str]:
    field = "smoke"
    value = raw_target.get(field)
    if not isinstance(value, dict):
        raise ValueError(f"{registry_path} target {target_id}.{field} must be an object")

    structural_archive_check = value.get("structural_archive_check")
    if structural_archive_check not in STRUCTURAL_ARCHIVE_CHECKS:
        allowed = ", ".join(sorted(STRUCTURAL_ARCHIVE_CHECKS))
        raise ValueError(
            f"{registry_path} target {target_id}.{field}.structural_archive_check "
            f"must be one of: {allowed}"
        )

    client_activation_check = value.get("client_activation_check")
    if client_activation_check not in CLIENT_ACTIVATION_CHECKS:
        allowed = ", ".join(sorted(CLIENT_ACTIVATION_CHECKS))
        raise ValueError(
            f"{registry_path} target {target_id}.{field}.client_activation_check "
            f"must be one of: {allowed}"
        )

    return {
        "structural_archive_check": str(structural_archive_check),
        "client_activation_check": str(client_activation_check),
    }


def _pattern_exists(root: Path, pattern: str) -> bool:
    clean_pattern = pattern.rstrip("/")
    if pattern.endswith("/"):
        if _has_wildcard(clean_pattern):
            return any(path.is_dir() for path in root.glob(clean_pattern))
        return (root / clean_pattern).is_dir()
    if _has_wildcard(clean_pattern):
        return any(_path_matches_pattern(path, root, clean_pattern) for path in root.rglob("*"))
    path = root / clean_pattern
    return path.exists() or path.is_symlink()


def _path_matches_pattern(path: Path, root: Path, pattern: str) -> bool:
    relative = path.relative_to(root).as_posix()
    return fnmatch(relative, pattern)


def _has_wildcard(pattern: str) -> bool:
    return any(char in pattern for char in "*?[")


def _remove_path(path: Path) -> None:
    if path.is_dir() and not path.is_symlink():
        import shutil

        shutil.rmtree(path)
    else:
        path.unlink()


def _repo_root(repo_root: Path | str | None) -> Path:
    if repo_root is not None:
        return Path(repo_root).resolve()
    return Path(__file__).resolve().parents[4]


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Validate the Qiongli platform target registry schema.")
    parser.add_argument("--root", type=Path, default=_repo_root(None), help="Repository or materialized distribution root.")
    args = parser.parse_args(argv)

    failures = validate_platform_target_registry(args.root)
    if failures:
        for failure in failures:
            print(f"[FAIL] platform target registry: {failure}")
        return 1
    print("[OK] platform target registry schema valid")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
