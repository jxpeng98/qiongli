from __future__ import annotations

import json
import re
import shutil
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

import yaml


class SubjectCatalogError(ValueError):
    """Raised when subject catalog metadata is invalid."""


class SubjectMaterializationError(ValueError):
    """Raised when a subject package cannot be generated safely."""


COVERAGE_CHOICES = {"complete", "focused"}


@dataclass(frozen=True)
class SubjectGroup:
    order: int
    heading: str
    subheading: str
    skill_refs: tuple[str, ...]
    stages: tuple[str, ...] = ()
    task_ids: tuple[str, ...] = ()


@dataclass(frozen=True)
class SubjectDefinition:
    id: str
    display_name: str
    package_goal: str
    extends: str | None
    skill_groups: tuple[SubjectGroup, ...]
    domain_profiles: tuple[str, ...] = ()
    venue_profiles: tuple[str, ...] = ()
    template_refs: tuple[str, ...] = ()
    skill_overrides: tuple[dict[str, Any], ...] = ()
    subject_specific_skill_refs: tuple[str, ...] = ()
    skill_refs: tuple[str, ...] = field(init=False)

    def __post_init__(self) -> None:
        refs: list[str] = []
        for group in self.skill_groups:
            for skill_ref in group.skill_refs:
                if skill_ref not in refs:
                    refs.append(skill_ref)
        for skill_ref in self.subject_specific_skill_refs:
            if skill_ref not in refs:
                refs.append(skill_ref)
        object.__setattr__(self, "skill_refs", tuple(refs))


@dataclass(frozen=True)
class SubjectCatalog:
    subjects: dict[str, SubjectDefinition]


@dataclass(frozen=True)
class MaterializeOptions:
    source: Path
    out: Path
    subject: str = "core"
    flavor: str = "full"
    coverage: str = "complete"


def load_subject_catalog(root: Path) -> dict[str, Any]:
    path = Path(root) / "subjects" / "catalog.yaml"
    if not path.is_file():
        raise SubjectCatalogError(f"missing subject catalog: {path}")
    try:
        payload = yaml.safe_load(path.read_text(encoding="utf-8"))
    except yaml.YAMLError as exc:
        raise SubjectCatalogError(f"malformed subject catalog: {exc}") from exc
    if not isinstance(payload, dict) or not isinstance(payload.get("subjects"), dict):
        raise SubjectCatalogError("subjects/catalog.yaml must contain a subjects object")
    return payload


def validate_subject_catalog(root: Path) -> SubjectCatalog:
    root = Path(root)
    payload = load_subject_catalog(root)
    registry_ids = _load_registry_ids(root)
    subjects: dict[str, SubjectDefinition] = {}

    for subject_id, raw_subject in payload["subjects"].items():
        if not isinstance(raw_subject, dict):
            raise SubjectCatalogError(f"subject {subject_id} must be an object")
        if "ids" in raw_subject:
            raise SubjectCatalogError(f"subject {subject_id} must use ordered groups, not ids")
        groups = _parse_groups(subject_id, raw_subject.get("skill_groups"))
        subject = SubjectDefinition(
            id=subject_id,
            display_name=_required_string(raw_subject, "display_name", subject_id),
            package_goal=_required_string(raw_subject, "package_goal", subject_id),
            extends=_optional_string(raw_subject.get("extends")),
            skill_groups=tuple(groups),
            domain_profiles=tuple(_string_list(raw_subject.get("domain_profiles"), "domain_profiles", subject_id)),
            venue_profiles=tuple(_string_list(raw_subject.get("venue_profiles"), "venue_profiles", subject_id)),
            template_refs=tuple(_string_list(raw_subject.get("template_refs"), "template_refs", subject_id)),
            skill_overrides=tuple(_dict_list(raw_subject.get("skill_overrides"), "skill_overrides", subject_id)),
            subject_specific_skill_refs=tuple(
                _string_list(raw_subject.get("subject_specific_skill_refs"), "subject_specific_skill_refs", subject_id)
            ),
        )
        missing_refs = sorted(set(subject.skill_refs) - registry_ids)
        if missing_refs:
            joined = ", ".join(missing_refs)
            raise SubjectCatalogError(f"subject {subject_id} references unknown skills: {joined}")
        subjects[subject_id] = subject

    for subject_id, subject in subjects.items():
        if subject.extends and subject.extends not in subjects:
            raise SubjectCatalogError(f"subject {subject_id} extends unknown subject: {subject.extends}")

    return SubjectCatalog(subjects=subjects)


def materialize_subject_package(options: MaterializeOptions) -> None:
    source = Path(options.source).resolve()
    out = Path(options.out).resolve()
    if options.flavor not in {"full", "desktop"}:
        raise SubjectMaterializationError(f"unsupported materialization flavor: {options.flavor}")
    if options.coverage not in COVERAGE_CHOICES:
        available = ", ".join(sorted(COVERAGE_CHOICES))
        raise SubjectMaterializationError(f"unsupported coverage: {options.coverage}. Available coverage: {available}")

    catalog = validate_subject_catalog(source)
    try:
        subject = catalog.subjects[options.subject]
    except KeyError as exc:
        available = ", ".join(sorted(catalog.subjects))
        raise SubjectMaterializationError(
            f"Unknown subject '{options.subject}'. Available subjects: {available}"
        ) from exc

    package_root = _package_root(source)
    base_registry = _load_registry_entries(source / "skills" / "registry.yaml")
    subject_registry = _load_all_subject_registry_entries(source)
    subject_skill_sources = _load_subject_skill_sources(source)
    registry_by_id = {entry["id"]: entry for entry in [*base_registry, *subject_registry]}

    if out.exists():
        shutil.rmtree(out)
    out.mkdir(parents=True)

    _copy_common_package_assets(package_root, out)
    _materialize_templates(package_root, out, subject, options.coverage)
    _materialize_venue_profiles(package_root, source, out, subject, options.coverage)
    selected_entries = _selected_registry_entries(subject, base_registry, subject_registry, options.coverage)
    _materialize_skills(
        source=source,
        package_root=package_root,
        out=out,
        subject=subject,
        selected_entries=selected_entries,
        subject_skill_sources=subject_skill_sources,
        coverage=options.coverage,
        include_detailed_skills=options.flavor == "full" or subject.id != "core",
    )
    _write_registry(out, selected_entries)
    _write_subject_markers(package_root, out, subject, options.flavor, options.coverage)
    _assert_no_symlinks(out)
    _assert_selected_skill_refs_exist(subject, registry_by_id)


def _parse_groups(subject_id: str, raw_groups: object) -> list[SubjectGroup]:
    if not isinstance(raw_groups, list) or not raw_groups:
        raise SubjectCatalogError(f"subject {subject_id} must define non-empty skill_groups")
    groups: list[SubjectGroup] = []
    seen_orders: set[int] = set()
    for index, raw_group in enumerate(raw_groups, start=1):
        if not isinstance(raw_group, dict):
            raise SubjectCatalogError(f"subject {subject_id} group {index} must be an object")
        order = raw_group.get("order")
        if not isinstance(order, int):
            raise SubjectCatalogError(f"subject {subject_id} group {index} must define integer order")
        if order in seen_orders:
            raise SubjectCatalogError(f"subject {subject_id} group order {order} is duplicated")
        seen_orders.add(order)
        groups.append(
            SubjectGroup(
                order=order,
                heading=_required_string(raw_group, "heading", f"{subject_id} group {order}"),
                subheading=_required_string(raw_group, "subheading", f"{subject_id} group {order}"),
                skill_refs=tuple(_string_list(raw_group.get("skill_refs"), "skill_refs", f"{subject_id} group {order}")),
                stages=tuple(_string_list(raw_group.get("stages"), "stages", f"{subject_id} group {order}")),
                task_ids=tuple(_string_list(raw_group.get("task_ids"), "task_ids", f"{subject_id} group {order}")),
            )
        )
    orders = sorted(seen_orders)
    expected = list(range(1, len(orders) + 1))
    if orders != expected:
        raise SubjectCatalogError(f"subject {subject_id} group orders must be consecutive starting at 1")
    return sorted(groups, key=lambda group: group.order)


def _load_registry_ids(root: Path) -> set[str]:
    ids: set[str] = set()
    for registry_path in [root / "skills" / "registry.yaml", *sorted((root / "subjects").glob("*/skills/registry.yaml"))]:
        if not registry_path.is_file():
            continue
        try:
            payload = yaml.safe_load(registry_path.read_text(encoding="utf-8")) or {}
        except yaml.YAMLError as exc:
            raise SubjectCatalogError(f"malformed registry {registry_path}: {exc}") from exc
        skills = payload.get("skills")
        if not isinstance(skills, list):
            raise SubjectCatalogError(f"{registry_path} must contain a skills list")
        for entry in skills:
            if isinstance(entry, dict) and isinstance(entry.get("id"), str):
                ids.add(entry["id"])
    return ids


def _package_root(source: Path) -> Path:
    candidate = source / "qiongli-workflow"
    if candidate.is_dir():
        return candidate
    if (source / "SKILL.md").is_file() and (source / "VERSION").is_file():
        return source
    raise SubjectMaterializationError(f"missing qiongli-workflow package under {source}")


def _copy_common_package_assets(package_root: Path, out: Path) -> None:
    for name in ("VERSION", "skills-core.md", "skills-summary.md"):
        src = package_root / name
        if src.is_file():
            _copy_path(src, out / name)
    for dirname in ("workflows", "references", "standards", "roles", "agents"):
        src = package_root / dirname
        if src.exists():
            _copy_path(src, out / dirname)


def _materialize_templates(package_root: Path, out: Path, subject: SubjectDefinition, coverage: str) -> None:
    src_root = package_root / "templates"
    dest_root = out / "templates"
    if not src_root.exists():
        return
    if coverage == "complete" or subject.id == "core" or not subject.template_refs:
        _copy_path(src_root, dest_root)
        return
    for rel in subject.template_refs:
        src = src_root / rel
        if not src.exists():
            raise SubjectMaterializationError(f"subject {subject.id} references missing template: {rel}")
        _copy_path(src, dest_root / rel)


def _materialize_venue_profiles(
    package_root: Path,
    source: Path,
    out: Path,
    subject: SubjectDefinition,
    coverage: str,
) -> None:
    dest_root = out / "venue-profiles"
    src = package_root / "venue-profiles"
    if coverage == "complete" or subject.id == "core" or not subject.venue_profiles:
        if src.exists():
            _copy_path(src, dest_root)
        if coverage != "complete" or subject.id == "core":
            return
    else:
        src = None

    if subject.id == "core" or not subject.venue_profiles:
        return
    for profile in subject.venue_profiles:
        src = _find_venue_profile(package_root, source, subject.id, profile)
        if not src.exists():
            raise SubjectMaterializationError(f"subject {subject.id} references missing venue profile: {profile}")
        _copy_path(src, dest_root / f"{profile}.yaml")


def _find_venue_profile(package_root: Path, source: Path, subject_id: str, profile: str) -> Path:
    current = source / "subjects" / subject_id / "venue-profiles" / f"{profile}.yaml"
    if current.exists():
        return current
    for candidate in sorted((source / "subjects").glob(f"*/venue-profiles/{profile}.yaml")):
        if candidate.exists():
            return candidate
    return package_root / "venue-profiles" / f"{profile}.yaml"


def _materialize_domain_profiles(package_root: Path, out: Path, subject: SubjectDefinition, coverage: str) -> None:
    source = package_root.parent if package_root.name == "qiongli-workflow" else package_root
    src_root = package_root / "skills" / "domain-profiles"
    if not src_root.exists():
        src_root = source / "skills" / "domain-profiles"
    dest_root = out / "skills" / "domain-profiles"
    if not src_root.exists():
        return
    if coverage == "complete" or subject.id == "core" or not subject.domain_profiles:
        _copy_path(src_root, dest_root)
        return
    for profile in subject.domain_profiles:
        src = src_root / f"{profile}.yaml"
        if not src.exists():
            src = source / "skills" / "domain-profiles" / f"{profile}.yaml"
        if not src.exists():
            raise SubjectMaterializationError(f"subject {subject.id} references missing domain profile: {profile}")
        _copy_path(src, dest_root / f"{profile}.yaml")


def _selected_registry_entries(
    subject: SubjectDefinition,
    base_registry: list[dict[str, Any]],
    subject_registry: list[dict[str, Any]],
    coverage: str,
) -> list[dict[str, Any]]:
    if subject.id == "core":
        return [dict(entry) for entry in base_registry]
    by_id = {entry["id"]: entry for entry in [*base_registry, *subject_registry]}
    if coverage == "complete":
        selected = [dict(entry) for entry in base_registry]
        for skill_ref in subject.subject_specific_skill_refs:
            if skill_ref not in by_id:
                raise SubjectMaterializationError(f"subject {subject.id} references unknown skill: {skill_ref}")
            selected.append(dict(by_id[skill_ref]))
        return _dedupe_registry_entries(selected)
    selected: list[dict[str, Any]] = []
    for skill_ref in subject.skill_refs:
        if skill_ref not in by_id:
            raise SubjectMaterializationError(f"subject {subject.id} references unknown skill: {skill_ref}")
        selected.append(dict(by_id[skill_ref]))
    return selected


def _dedupe_registry_entries(entries: list[dict[str, Any]]) -> list[dict[str, Any]]:
    selected: list[dict[str, Any]] = []
    seen: set[str] = set()
    for entry in entries:
        skill_id = str(entry["id"])
        if skill_id in seen:
            continue
        seen.add(skill_id)
        selected.append(entry)
    return selected


def _materialize_skills(
    *,
    source: Path,
    package_root: Path,
    out: Path,
    subject: SubjectDefinition,
    selected_entries: list[dict[str, Any]],
    subject_skill_sources: dict[str, Path],
    coverage: str,
    include_detailed_skills: bool,
) -> None:
    _materialize_domain_profiles(package_root, out, subject, coverage)
    if not include_detailed_skills:
        return

    overrides = {str(item.get("skill")): item for item in subject.skill_overrides}
    for entry in selected_entries:
        skill_id = str(entry["id"])
        rel = Path(str(entry["file"]))
        dest = out / rel
        src = package_root / rel
        if not src.exists():
            src = source / rel
        if not src.exists() and skill_id in subject_skill_sources:
            src = subject_skill_sources[skill_id]
        if not src.exists():
            raise SubjectMaterializationError(f"missing skill source for {skill_id}: {src}")

        text = src.read_text(encoding="utf-8")
        if skill_id in overrides:
            text = _apply_overlay(source, subject, skill_id, text, overrides[skill_id])
        dest.parent.mkdir(parents=True, exist_ok=True)
        dest.write_text(text, encoding="utf-8")


def _apply_overlay(
    source: Path,
    subject: SubjectDefinition,
    skill_id: str,
    base_text: str,
    override: dict[str, Any],
) -> str:
    overlay_rel = override.get("overlay")
    if not isinstance(overlay_rel, str) or not overlay_rel.strip():
        raise SubjectMaterializationError(f"skill override for {skill_id} must define overlay")
    overlay_path = source / "subjects" / subject.id / overlay_rel
    if not overlay_path.is_file():
        raise SubjectMaterializationError(f"missing overlay for {skill_id}: {overlay_path}")
    overlay_text = overlay_path.read_text(encoding="utf-8").strip()
    mode = str(override.get("mode") or "append")
    if mode == "append":
        return base_text.rstrip() + "\n\n" + overlay_text + "\n"
    if mode == "replace_sections":
        raw_sections = override.get("sections")
        if not isinstance(raw_sections, list) or not raw_sections:
            raise SubjectMaterializationError(f"replace_sections override for {skill_id} must list sections")
        result = base_text
        for section in raw_sections:
            if not isinstance(section, str) or not section.strip():
                raise SubjectMaterializationError(f"replace_sections override for {skill_id} has invalid section")
            result = _replace_markdown_section(result, overlay_text, section.strip(), skill_id)
        return result
    raise SubjectMaterializationError(f"unsupported override mode for {skill_id}: {mode}")


def _replace_markdown_section(base_text: str, overlay_text: str, section: str, skill_id: str) -> str:
    base_range = _find_section_range(base_text, section)
    if base_range is None:
        raise SubjectMaterializationError(f"{skill_id}: base skill missing replace_sections section: {section}")
    overlay_range = _find_section_range(overlay_text, section)
    if overlay_range is None:
        raise SubjectMaterializationError(f"{skill_id}: overlay missing replace_sections section: {section}")
    base_start, base_end = base_range
    overlay_start, overlay_end = overlay_range
    replacement = overlay_text[overlay_start:overlay_end].strip() + "\n"
    return base_text[:base_start].rstrip() + "\n\n" + replacement + base_text[base_end:].lstrip()


def _find_section_range(text: str, section: str) -> tuple[int, int] | None:
    heading_re = re.compile(r"(?m)^(?P<marker>#{2})\s+(?P<title>.+?)\s*$")
    wanted = _normalize_heading(section)
    for match in heading_re.finditer(text):
        title = _normalize_heading(match.group("title"))
        if title != wanted and not title.startswith(wanted + " "):
            continue
        next_match = heading_re.search(text, match.end())
        end = next_match.start() if next_match else len(text)
        return match.start(), end
    return None


def _normalize_heading(value: str) -> str:
    return re.sub(r"\s+", " ", value.strip().lower())


def _write_registry(out: Path, entries: list[dict[str, Any]]) -> None:
    registry_path = out / "skills" / "registry.yaml"
    registry_path.parent.mkdir(parents=True, exist_ok=True)
    registry_path.write_text(yaml.safe_dump({"skills": entries}, sort_keys=False, allow_unicode=True), encoding="utf-8")


def _write_subject_markers(
    package_root: Path,
    out: Path,
    subject: SubjectDefinition,
    flavor: str,
    coverage: str,
) -> None:
    version = (package_root / "VERSION").read_text(encoding="utf-8").strip()
    (out / "VERSION").write_text(version + "\n", encoding="utf-8")
    (out / "SUBJECT").write_text(subject.id + "\n", encoding="utf-8")
    (out / "SUBJECT_MANIFEST.json").write_text(
        json.dumps(
            {
                "subject": subject.id,
                "coverage": coverage,
                "flavor": flavor,
                "layers": _subject_layers(subject),
            },
            indent=2,
            sort_keys=True,
        )
        + "\n",
        encoding="utf-8",
    )
    (out / "SKILL.md").write_text(_render_skill_md(subject, flavor), encoding="utf-8")


def _subject_layers(subject: SubjectDefinition) -> list[str]:
    layers: list[str] = []
    if subject.extends:
        layers.append(subject.extends)
    layers.append(subject.id)
    return layers


def _render_skill_md(subject: SubjectDefinition, flavor: str) -> str:
    map_title = f"{subject.display_name.removeprefix('Qiongli ').strip() or subject.display_name} Workflow Map"
    lines = [
        "---",
        "name: qiongli",
        f"description: {subject.package_goal}",
        "---",
        "",
        f"# {subject.display_name}",
        "",
        subject.package_goal,
        "",
        f"## {map_title}",
        "",
    ]
    for group in subject.skill_groups:
        lines.extend(
            [
                f"### {group.order}. {group.heading}",
                group.subheading,
                "",
            ]
        )
    lines.extend(
        [
            "## Required Behavior",
            "",
            "- Use the canonical task and output definitions in `references/workflow-contract.md`.",
            "- Keep stage labels and task IDs unchanged across models.",
            "- When a workflow references `templates/<name>.md`, load the template from `templates/`.",
            "- Use `skills/registry.yaml` as the active skill list for this subject package.",
            "",
            "## Skill Loading Strategy",
            "",
        ]
    )
    if flavor == "desktop" and subject.id == "core":
        lines.extend(
            [
                "This Desktop/Web package uses `skills-summary.md`, `skills-core.md`, and `skills/registry.yaml`.",
                "Detailed per-skill markdown files are omitted to stay under upload limits.",
                "",
            ]
        )
    else:
        lines.extend(
            [
                "Use `skills-summary.md` for quick lookup, `skills-core.md` for consolidated guidance, and detailed files under `skills/` for active subject skills.",
                "",
            ]
        )
    return "\n".join(lines)


def _load_registry_entries(path: Path) -> list[dict[str, Any]]:
    if not path.is_file():
        return []
    try:
        payload = yaml.safe_load(path.read_text(encoding="utf-8")) or {}
    except yaml.YAMLError as exc:
        raise SubjectMaterializationError(f"malformed registry {path}: {exc}") from exc
    skills = payload.get("skills")
    if not isinstance(skills, list):
        raise SubjectMaterializationError(f"{path} must contain a skills list")
    entries: list[dict[str, Any]] = []
    for entry in skills:
        if not isinstance(entry, dict) or not isinstance(entry.get("id"), str):
            raise SubjectMaterializationError(f"{path} contains invalid skill registry entry")
        entries.append(dict(entry))
    return entries


def _load_all_subject_registry_entries(source: Path) -> list[dict[str, Any]]:
    entries: list[dict[str, Any]] = []
    for registry_path in sorted((source / "subjects").glob("*/skills/registry.yaml")):
        entries.extend(_load_registry_entries(registry_path))
    return entries


def _load_subject_skill_sources(source: Path) -> dict[str, Path]:
    sources: dict[str, Path] = {}
    for registry_path in sorted((source / "subjects").glob("*/skills/registry.yaml")):
        subject_root = registry_path.parents[1]
        for entry in _load_registry_entries(registry_path):
            skill_id = str(entry["id"])
            sources.setdefault(skill_id, subject_root / "skills" / f"{skill_id}.md")
    return sources


def _assert_selected_skill_refs_exist(subject: SubjectDefinition, registry_by_id: dict[str, dict[str, Any]]) -> None:
    missing = sorted(set(subject.skill_refs) - set(registry_by_id))
    if missing:
        raise SubjectMaterializationError(f"subject {subject.id} references unknown skills: {', '.join(missing)}")


def _copy_path(src: Path, dest: Path) -> None:
    if src.is_symlink():
        raise SubjectMaterializationError(f"refusing to copy symlink: {src}")
    if src.is_dir():
        shutil.copytree(src, dest, ignore=_ignore_generated)
    else:
        dest.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(src, dest)


def _ignore_generated(_src: str, names: list[str]) -> set[str]:
    ignored = {".DS_Store", "__pycache__", ".pytest_cache", ".mypy_cache", "node_modules", "dist", "build"} & set(names)
    ignored.update(name for name in names if name.endswith((".pyc", ".pyo")))
    return ignored


def _assert_no_symlinks(root: Path) -> None:
    for item in root.rglob("*"):
        if item.is_symlink():
            raise SubjectMaterializationError(f"generated package contains symlink: {item}")


def _required_string(mapping: dict[str, object], key: str, label: str) -> str:
    value = mapping.get(key)
    if not isinstance(value, str) or not value.strip():
        raise SubjectCatalogError(f"{label} missing required string field: {key}")
    return value.strip()


def _optional_string(value: object) -> str | None:
    if value is None:
        return None
    if not isinstance(value, str) or not value.strip():
        raise SubjectCatalogError("optional string field must be non-empty when provided")
    return value.strip()


def _string_list(value: object, key: str, label: str) -> list[str]:
    if value is None:
        return []
    if not isinstance(value, list):
        raise SubjectCatalogError(f"{label} field {key} must be a list")
    result: list[str] = []
    for item in value:
        if not isinstance(item, str) or not item.strip():
            raise SubjectCatalogError(f"{label} field {key} must contain only non-empty strings")
        result.append(item.strip())
    return result


def _dict_list(value: object, key: str, label: str) -> list[dict[str, Any]]:
    if value is None:
        return []
    if not isinstance(value, list):
        raise SubjectCatalogError(f"{label} field {key} must be a list")
    result: list[dict[str, Any]] = []
    for item in value:
        if not isinstance(item, dict):
            raise SubjectCatalogError(f"{label} field {key} must contain only objects")
        result.append(dict(item))
    return result
