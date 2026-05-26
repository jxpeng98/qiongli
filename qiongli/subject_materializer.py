from __future__ import annotations

from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

import yaml


class SubjectCatalogError(ValueError):
    """Raised when subject catalog metadata is invalid."""


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
    raise NotImplementedError("subject package materialization is implemented in a later task")


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
