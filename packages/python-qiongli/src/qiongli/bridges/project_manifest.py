from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path, PurePosixPath
from typing import Any, Mapping, Sequence

import yaml


OFFICIAL_SUBJECTS = (
    "auto",
    "core",
    "economics",
    "accounting",
    "business",
    "finance",
    "political-economy",
    "geoeconomics",
    "economics-accounting",
)
STRICTNESS_CHOICES = ("standard", "high")
MANIFEST_REL = Path(".qiongli") / "guidance_manifest.yaml"


class ProjectManifestError(ValueError):
    pass


@dataclass(frozen=True)
class ProjectManifest:
    active_subject: str = "auto"
    secondary_subjects: list[str] | None = None
    venue_profiles: list[str] | None = None
    method_lenses: list[str] | None = None
    strictness: str = "standard"

    def normalized(self) -> ProjectManifest:
        return ProjectManifest(
            active_subject=_validate_subject(self.active_subject, field="active_subject"),
            secondary_subjects=_validate_subject_list(
                self.secondary_subjects,
                field="secondary_subjects",
            ),
            venue_profiles=_validate_rel_path_list(self.venue_profiles, field="venue_profiles"),
            method_lenses=_validate_rel_path_list(self.method_lenses, field="method_lenses"),
            strictness=_validate_strictness(self.strictness),
        )

    def to_dict(self) -> dict[str, Any]:
        manifest = self.normalized()
        return {
            "active_subject": manifest.active_subject,
            "secondary_subjects": list(manifest.secondary_subjects or []),
            "venue_profiles": list(manifest.venue_profiles or []),
            "method_lenses": list(manifest.method_lenses or []),
            "strictness": manifest.strictness,
        }


@dataclass(frozen=True)
class ProjectManifestState:
    exists: bool
    path: Path
    project_root: Path
    manifest: ProjectManifest
    warnings: list[str] | None = None

    def to_packet(self) -> dict[str, Any]:
        return {
            "exists": self.exists,
            "path": str(self.path),
            "manifest": self.manifest.to_dict(),
            "warnings": list(self.warnings or []),
        }


def load_project_manifest(project_root: Path) -> ProjectManifestState:
    root = _normalize_project_root(project_root)
    path = root / MANIFEST_REL
    if not path.is_file():
        return ProjectManifestState(
            exists=False,
            path=path,
            project_root=root,
            manifest=ProjectManifest().normalized(),
            warnings=[],
        )

    try:
        loaded = yaml.safe_load(path.read_text(encoding="utf-8"))
    except yaml.YAMLError as exc:
        raise ProjectManifestError(f"Malformed project manifest: {exc}") from exc

    if not isinstance(loaded, Mapping):
        raise ProjectManifestError("Project manifest must be a YAML object")

    manifest, warnings = _manifest_from_mapping(loaded)
    return ProjectManifestState(
        exists=True,
        path=path,
        project_root=root,
        manifest=manifest,
        warnings=warnings,
    )


def init_project_manifest(project_root: Path, *, overwrite: bool = False) -> ProjectManifestState:
    root = _normalize_project_root(project_root)
    path = root / MANIFEST_REL
    if path.exists() and not overwrite:
        return load_project_manifest(root)

    manifest = ProjectManifest().normalized()
    _write_manifest(path, manifest)
    return ProjectManifestState(
        exists=True,
        path=path,
        project_root=root,
        manifest=manifest,
        warnings=[],
    )


def update_project_manifest(
    project_root: Path,
    *,
    active_subject: str | None = None,
    secondary_subjects: Sequence[str] | None = None,
    venue_profiles: Sequence[str] | None = None,
    method_lenses: Sequence[str] | None = None,
    strictness: str | None = None,
) -> ProjectManifestState:
    current = load_project_manifest(project_root)
    manifest = ProjectManifest(
        active_subject=active_subject if active_subject is not None else current.manifest.active_subject,
        secondary_subjects=(
            list(secondary_subjects)
            if secondary_subjects is not None
            else list(current.manifest.secondary_subjects or [])
        ),
        venue_profiles=(
            list(venue_profiles)
            if venue_profiles is not None
            else list(current.manifest.venue_profiles or [])
        ),
        method_lenses=(
            list(method_lenses)
            if method_lenses is not None
            else list(current.manifest.method_lenses or [])
        ),
        strictness=strictness if strictness is not None else current.manifest.strictness,
    ).normalized()
    _write_manifest(current.path, manifest)
    return ProjectManifestState(
        exists=True,
        path=current.path,
        project_root=current.project_root,
        manifest=manifest,
        warnings=list(current.warnings or []),
    )


def manifest_to_guidance_section(state: ProjectManifestState) -> str:
    manifest = state.manifest.normalized()
    source = _rel(state.project_root, state.path) if state.exists else "implicit defaults"
    rows = [
        "## Project Manifest",
        "",
        f"- source: {source}",
        f"- active_subject: {manifest.active_subject}",
        f"- secondary_subjects: {_display_list(manifest.secondary_subjects)}",
        f"- venue_profiles: {_display_list(manifest.venue_profiles)}",
        f"- method_lenses: {_display_list(manifest.method_lenses)}",
        f"- strictness: {manifest.strictness}",
    ]
    warnings = list(state.warnings or [])
    if warnings:
        rows.append(f"- warnings: {'; '.join(warnings)}")
    return "\n".join(rows)


def _manifest_from_mapping(payload: Mapping[Any, Any]) -> tuple[ProjectManifest, list[str]]:
    allowed = {
        "active_subject",
        "secondary_subjects",
        "venue_profiles",
        "method_lenses",
        "strictness",
    }
    warnings = [
        f"Ignored unsupported manifest field: {key}"
        for key in sorted(str(key) for key in payload.keys() if str(key) not in allowed)
    ]
    manifest = ProjectManifest(
        active_subject=payload.get("active_subject", "auto"),
        secondary_subjects=payload.get("secondary_subjects"),
        venue_profiles=payload.get("venue_profiles"),
        method_lenses=payload.get("method_lenses"),
        strictness=payload.get("strictness", "standard"),
    ).normalized()
    return manifest, warnings


def _write_manifest(path: Path, manifest: ProjectManifest) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        yaml.safe_dump(manifest.to_dict(), sort_keys=False, allow_unicode=True),
        encoding="utf-8",
    )


def _normalize_project_root(project_root: Path) -> Path:
    return Path(project_root).expanduser().resolve()


def _validate_subject(value: Any, *, field: str) -> str:
    if not isinstance(value, str):
        raise ProjectManifestError(f"Unsupported {field}: {value!r}")
    normalized = value.strip()
    if normalized not in OFFICIAL_SUBJECTS:
        raise ProjectManifestError(f"Unsupported {field}: {normalized}")
    return normalized


def _validate_subject_list(value: Any, *, field: str) -> list[str]:
    values = _validate_list(value, field=field)
    return [_validate_subject(item, field=field) for item in values]


def _validate_rel_path_list(value: Any, *, field: str) -> list[str]:
    values = _validate_list(value, field=field)
    return [_validate_rel_path(item, field=field) for item in values]


def _validate_list(value: Any, *, field: str) -> list[Any]:
    if value is None:
        return []
    if isinstance(value, (str, bytes)) or not isinstance(value, Sequence):
        raise ProjectManifestError(f"{field} must be a list")
    return list(value)


def _validate_rel_path(value: Any, *, field: str) -> str:
    if not isinstance(value, str):
        raise ProjectManifestError(f"{field} entries must be strings")
    normalized = value.strip()
    path = PurePosixPath(normalized)
    if (
        not normalized
        or normalized.startswith(("/", "\\"))
        or "\\" in normalized
        or path.is_absolute()
        or not path.parts
        or any(part in {"", ".", ".."} for part in path.parts)
    ):
        raise ProjectManifestError(f"Unsupported {field} entry: {value!r}")
    return normalized


def _validate_strictness(value: Any) -> str:
    if not isinstance(value, str):
        raise ProjectManifestError(f"Unsupported strictness: {value!r}")
    normalized = value.strip()
    if normalized not in STRICTNESS_CHOICES:
        raise ProjectManifestError(f"Unsupported strictness: {normalized}")
    return normalized


def _display_list(values: list[str] | None) -> str:
    return ", ".join(values or []) if values else "none"


def _rel(root: Path, path: Path) -> str:
    try:
        return str(path.relative_to(root))
    except ValueError:
        return str(path)
