from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path, PureWindowsPath
from typing import Any, Mapping

import yaml


RUNTIME_SUBJECT_FILE = "runtime-subject.yaml"
ALLOWED_ACTIVATION_STATUSES = {
    "candidate",
    "eval_ready",
    "runtime_enabled",
    "disabled",
}
PATH_FIELDS = {
    "domain_profile",
    "overlay",
    "subject_skill",
    "evaluation_pack",
}
OBJECT_FIELDS = {
    "signal_groups",
    "method_lenses",
    "near_miss_policy",
    "activation_gate",
}


class SubjectContractValidationError(ValueError):
    pass


@dataclass(frozen=True)
class RuntimeSubjectContract:
    subject: str
    display_name: str
    activation_status: str
    extends: str
    source: str
    domain_profile: str
    overlay: str
    subject_skill: str
    signal_groups: dict[str, list[dict[str, Any]]]
    method_lenses: dict[str, dict[str, Any]]
    evaluation_pack: str
    near_miss_policy: dict[str, Any]
    activation_gate: dict[str, Any]


def load_runtime_subject_contracts(
    subjects_root: Path | str | None = None,
    *,
    recursive: bool | None = None,
    runtime_file: Path | str | None = None,
) -> dict[str, RuntimeSubjectContract]:
    explicit_root = subjects_root is not None
    root = Path(subjects_root) if explicit_root else _default_subjects_root(runtime_file)
    include_nested = explicit_root if recursive is None else recursive
    contracts: dict[str, RuntimeSubjectContract] = {}

    for contract_path in _runtime_subject_paths(root, recursive=include_nested):
        with contract_path.open("r", encoding="utf-8") as handle:
            payload = yaml.safe_load(handle)
        contract = validate_runtime_subject_contract(
            payload,
            source=str(contract_path),
        )
        if contract.subject in contracts:
            raise SubjectContractValidationError(
                f"{contract_path}: duplicate subject {contract.subject!r}",
            )
        contracts[contract.subject] = contract

    return contracts


def subject_activation_status(
    subject: str,
    contracts: Mapping[str, RuntimeSubjectContract] | None = None,
) -> str:
    loaded_contracts = contracts if contracts is not None else load_runtime_subject_contracts()
    contract = loaded_contracts.get(subject)
    if contract is None:
        return "candidate"
    return contract.activation_status


def validate_runtime_subject_contract(
    payload: Mapping[str, Any],
    *,
    source: str,
) -> RuntimeSubjectContract:
    if not isinstance(payload, Mapping):
        raise SubjectContractValidationError(f"{source}: payload must be a mapping")

    subject = _required_string(payload, "subject", source=source)
    display_name = _required_string(payload, "display_name", source=source)
    activation_status = _required_string(payload, "activation_status", source=source)
    if activation_status not in ALLOWED_ACTIVATION_STATUSES:
        allowed = ", ".join(sorted(ALLOWED_ACTIVATION_STATUSES))
        raise SubjectContractValidationError(
            f"{source}: activation_status must be one of {allowed}",
        )

    for field in PATH_FIELDS:
        _validate_relative_path_field(payload, field, source=source)

    objects = {
        field: _optional_mapping(payload, field, source=source)
        for field in OBJECT_FIELDS
    }
    signal_groups = _validate_signal_groups(
        objects["signal_groups"],
        source=source,
    )
    method_lenses = _validate_method_lenses(
        objects["method_lenses"],
        source=source,
    )

    return RuntimeSubjectContract(
        subject=subject,
        display_name=display_name,
        activation_status=activation_status,
        extends=_optional_string(payload, "extends", source=source),
        source=source,
        domain_profile=_optional_string(payload, "domain_profile", source=source),
        overlay=_optional_string(payload, "overlay", source=source),
        subject_skill=_optional_string(payload, "subject_skill", source=source),
        signal_groups=signal_groups,
        method_lenses=method_lenses,
        evaluation_pack=_optional_string(payload, "evaluation_pack", source=source),
        near_miss_policy=dict(objects["near_miss_policy"]),
        activation_gate=dict(objects["activation_gate"]),
    )


def _default_subjects_root(runtime_file: Path | str | None = None) -> Path:
    candidates = _subject_root_candidates(runtime_file)
    for candidate in candidates:
        if candidate.exists():
            return candidate
    return candidates[-1]


def _subject_root_candidates(runtime_file: Path | str | None = None) -> list[Path]:
    runtime_path = Path(runtime_file) if runtime_file is not None else Path(__file__)
    runtime_path = runtime_path.expanduser().resolve()
    candidates: list[Path] = []

    for parent in runtime_path.parents:
        candidates.append(parent / "content" / "subjects")
    for parent in runtime_path.parents:
        candidates.append(parent / "subjects")
    for parent in runtime_path.parents:
        candidates.append(parent / "payload" / "subjects")
    candidates.append(Path("content") / "subjects")

    return _unique_paths(candidates)


def _runtime_subject_paths(root: Path, *, recursive: bool) -> list[Path]:
    if recursive:
        return sorted(root.rglob(RUNTIME_SUBJECT_FILE))
    return sorted(root.glob(f"*/{RUNTIME_SUBJECT_FILE}"))


def _required_string(payload: Mapping[str, Any], field: str, *, source: str) -> str:
    value = payload.get(field)
    if not isinstance(value, str) or not value.strip():
        raise SubjectContractValidationError(
            f"{source}: required field {field!r} must be a non-empty string",
        )
    return value.strip()


def _optional_string(payload: Mapping[str, Any], field: str, *, source: str) -> str:
    value = payload.get(field)
    if value is None:
        return ""
    if isinstance(value, str):
        return value.strip()
    raise SubjectContractValidationError(
        f"{source}: field {field!r} must be a string when present",
    )


def _optional_mapping(
    payload: Mapping[str, Any],
    field: str,
    *,
    source: str,
) -> Mapping[str, Any]:
    value = payload.get(field, {})
    if not isinstance(value, Mapping):
        raise SubjectContractValidationError(
            f"{source}: field {field!r} must be a mapping",
        )
    return value


def _validate_relative_path_field(
    payload: Mapping[str, Any],
    field: str,
    *,
    source: str,
) -> None:
    value = payload.get(field)
    if value is None:
        return
    _validate_relative_path_value(value, field=field, source=source)


def _validate_relative_path_value(value: Any, *, field: str, source: str) -> None:
    if not isinstance(value, str):
        raise SubjectContractValidationError(
            f"{source}: path field {field!r} must be a string",
        )

    normalized = value.strip()
    posix_path = Path(normalized)
    windows_path = PureWindowsPath(normalized)
    if posix_path.is_absolute() or windows_path.is_absolute():
        raise SubjectContractValidationError(
            f"{source}: absolute path is not allowed in {field!r}",
        )
    if ".." in posix_path.parts or ".." in windows_path.parts:
        raise SubjectContractValidationError(
            f"{source}: path escape is not allowed in {field!r}",
        )


def _validate_signal_groups(
    signal_groups: Mapping[str, Any],
    *,
    source: str,
) -> dict[str, list[dict[str, Any]]]:
    validated: dict[str, list[dict[str, Any]]] = {}
    for group_name, group_values in signal_groups.items():
        if not isinstance(group_values, list):
            raise SubjectContractValidationError(
                f"{source}: signal_groups value for {group_name!r} must be a list",
            )
        validated_values: list[dict[str, Any]] = []
        for index, entry in enumerate(group_values):
            if not isinstance(entry, Mapping):
                raise SubjectContractValidationError(
                    f"{source}: signal_groups entry {group_name!r}[{index}] must be a mapping",
                )
            validated_values.append(dict(entry))
        validated[str(group_name)] = validated_values
    return validated


def _validate_method_lenses(
    method_lenses: Mapping[str, Any],
    *,
    source: str,
) -> dict[str, dict[str, Any]]:
    validated: dict[str, dict[str, Any]] = {}
    for lens_name, lens_config in method_lenses.items():
        if not isinstance(lens_config, Mapping):
            raise SubjectContractValidationError(
                f"{source}: method_lenses value for {lens_name!r} must be a mapping",
            )
        if "resource" in lens_config:
            _validate_relative_path_value(
                lens_config["resource"],
                field=f"method_lenses[{lens_name!r}].resource",
                source=source,
            )
        validated[str(lens_name)] = dict(lens_config)
    return validated


def _unique_paths(paths: list[Path]) -> list[Path]:
    seen: set[str] = set()
    unique: list[Path] = []
    for path in paths:
        marker = str(path)
        if marker in seen:
            continue
        seen.add(marker)
        unique.append(path)
    return unique
