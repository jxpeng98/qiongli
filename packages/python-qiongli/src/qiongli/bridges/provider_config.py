from __future__ import annotations

import json
import os
import stat
import tempfile
from pathlib import Path, PureWindowsPath
from typing import Mapping


PROVIDER_FIELDS: dict[str, dict[str, tuple[str, ...]]] = {
    "openalex": {
        "api_key": (
            "QIONGLI_OPENALEX_API_KEY",
            "OPENALEX_API_KEY",
            "QIONGLI_MCPB_OPENALEX_API_KEY",
        ),
        "email": ("QIONGLI_OPENALEX_EMAIL", "OPENALEX_EMAIL", "QIONGLI_MCPB_OPENALEX_EMAIL"),
    },
    "semantic_scholar": {
        "api_key": (
            "QIONGLI_SEMANTIC_SCHOLAR_API_KEY",
            "SEMANTIC_SCHOLAR_API_KEY",
            "S2_API_KEY",
            "QIONGLI_MCPB_SEMANTIC_SCHOLAR_API_KEY",
        ),
    },
    "crossref": {
        "email": ("QIONGLI_CROSSREF_EMAIL", "CROSSREF_EMAIL", "QIONGLI_MCPB_CROSSREF_EMAIL"),
    },
    "pubmed": {
        "api_key": (
            "QIONGLI_NCBI_API_KEY",
            "NCBI_API_KEY",
            "PUBMED_API_KEY",
            "QIONGLI_MCPB_PUBMED_API_KEY",
        ),
    },
    "arxiv": {},
}

SUPPORTED_PROVIDER_CONFIG_VERSIONS = frozenset({1})
SEARCH_PUBLIC_FIELD_TYPES: dict[str, type[object]] = {
    "minimum_productive_providers": int,
    "allow_platform_search_supplement": bool,
}

PROJECT_ENV_FILES = (".env",)
PROJECT_TOML_FILES = ("qiongli.toml", ".qiongli.toml")


class ProviderConfigError(RuntimeError):
    """Raised when the persisted provider config cannot be read safely."""


_CONFIG_HOME_PATH_ERROR = (
    "QIONGLI_CONFIG_HOME must be a fully qualified absolute path "
    "or use '~' home notation"
)
_USER_HOME_RESOLUTION_ERROR = (
    "platform user home directory must be a fully qualified absolute path"
)


def global_provider_config_path() -> Path:
    config_home = os.environ.get("QIONGLI_CONFIG_HOME", "").strip()
    if not config_home:
        root = _platform_user_home() / ".config" / "qiongli"
    elif _is_fully_qualified_config_home(config_home):
        root = Path(config_home)
    elif config_home == "~":
        root = _platform_user_home()
    elif config_home.startswith("~/"):
        suffix = _portable_tilde_suffix(config_home)
        root = _platform_user_home() / Path(suffix)
    else:
        raise ProviderConfigError(_CONFIG_HOME_PATH_ERROR)
    return root / "providers.json"


def _is_fully_qualified_config_home(
    value: str,
    *,
    windows: bool | None = None,
) -> bool:
    use_windows_semantics = os.name == "nt" if windows is None else windows
    if use_windows_semantics:
        candidate = PureWindowsPath(value)
        return bool(candidate.drive and candidate.root)
    return Path(value).is_absolute()


def _portable_tilde_suffix(value: str) -> str:
    suffix = value[2:]
    has_ascii_drive_prefix = (
        len(suffix) >= 2
        and suffix[0].isascii()
        and suffix[0].isalpha()
        and suffix[1] == ":"
    )
    if suffix.startswith(("/", "\\")) or has_ascii_drive_prefix:
        raise ProviderConfigError(_CONFIG_HOME_PATH_ERROR)
    return suffix


def _platform_user_home() -> Path:
    try:
        home = Path.home()
    except (OSError, RuntimeError) as exc:
        raise ProviderConfigError(_USER_HOME_RESOLUTION_ERROR) from exc
    if not _is_fully_qualified_config_home(str(home)):
        raise ProviderConfigError(_USER_HOME_RESOLUTION_ERROR)
    return home


def resolve_provider_config(
    *,
    cwd: Path | str | None = None,
    env: Mapping[str, str] | None = None,
) -> dict[str, object]:
    base = _empty_config()
    _merge_config(base, _read_json_config(global_provider_config_path()))
    if cwd is not None:
        project_root = Path(cwd)
        _merge_config(base, _read_project_toml_config(project_root))
        _merge_config(base, _read_project_env_config(project_root))
    _merge_config(base, _config_from_env(os.environ if env is None else env))
    _finalize_config(base)
    return base


def redact_provider_config(config: Mapping[str, object]) -> dict[str, object]:
    providers = config.get("providers", {})
    if not isinstance(providers, Mapping):
        providers = {}
    redacted: dict[str, object] = {
        "providers": {},
        "search": _redacted_search_config(config.get("search")),
    }
    redacted_providers: dict[str, object] = {}
    for provider, field_map in PROVIDER_FIELDS.items():
        raw = providers.get(provider, {})
        if not isinstance(raw, Mapping):
            raw = {}
        fields = {
            field: "configured" if str(raw.get(field, "")).strip() else "missing"
            for field in field_map
        }
        redacted_providers[provider] = {
            "enabled": bool(raw.get("enabled", False)),
            "configured": bool(raw.get("configured", False)),
            "fields": fields,
        }
    redacted["providers"] = redacted_providers
    return redacted


def provider_config_summary(config: Mapping[str, object]) -> dict[str, str]:
    redacted = redact_provider_config(config)
    providers = redacted.get("providers", {})
    if not isinstance(providers, Mapping):
        return {}
    summary: dict[str, str] = {}
    for provider, raw in providers.items():
        if isinstance(raw, Mapping) and raw.get("configured"):
            summary[str(provider)] = "configured"
        else:
            summary[str(provider)] = "missing"
    return summary


def active_provider_names(config: Mapping[str, object]) -> list[str]:
    providers = config.get("providers", {})
    if not isinstance(providers, Mapping):
        return []
    active: list[str] = []
    for provider in PROVIDER_FIELDS:
        raw_provider = providers.get(provider)
        if (
            isinstance(raw_provider, Mapping)
            and raw_provider.get("enabled") is True
            and raw_provider.get("configured") is True
        ):
            active.append(provider)
    return active


def provider_config_env(config: Mapping[str, object]) -> dict[str, str]:
    providers = config.get("providers", {})
    if not isinstance(providers, Mapping):
        providers = {}
    env: dict[str, str] = {}
    for provider, field_map in PROVIDER_FIELDS.items():
        raw = providers.get(provider, {})
        if not isinstance(raw, Mapping):
            continue
        if raw.get("enabled") is not True or raw.get("configured") is not True:
            continue
        for field, aliases in field_map.items():
            value = str(raw.get(field, "")).strip()
            if not value:
                continue
            for alias in aliases:
                env[alias] = value
    return env


def provider_capability_mode(config: Mapping[str, object]) -> str:
    """Report provider connectivity from configured *and enabled* state."""

    return "provider_connected" if active_provider_names(config) else "strategy_only"


def set_provider_value(provider: str, field: str, value: str, *, project_dir: Path | str | None = None) -> Path:
    if project_dir is not None:
        raise NotImplementedError("project provider writes are not implemented yet")
    provider_id = _normalize_provider(provider)
    field_id = _normalize_field(field)
    _assert_known_field(provider_id, field_id)
    path = global_provider_config_path()
    config = _read_json_config(path)
    raw_provider = _provider_entry_for_write(config, provider_id, create=True)
    if raw_provider is None:  # pragma: no cover - create=True always returns an entry.
        raise ProviderConfigError("provider configuration could not be prepared for writing")
    raw_provider["enabled"] = True
    _remove_normalized_key(raw_provider, field_id)
    raw_provider[field_id] = str(value)
    _write_json_config(path, config)
    return path


def unset_provider_value(provider: str, field: str, *, project_dir: Path | str | None = None) -> Path:
    if project_dir is not None:
        raise NotImplementedError("project provider writes are not implemented yet")
    provider_id = _normalize_provider(provider)
    field_id = _normalize_field(field)
    _assert_known_field(provider_id, field_id)
    path = global_provider_config_path()
    config = _read_json_config(path)
    raw_provider = _provider_entry_for_write(config, provider_id, create=False)
    if raw_provider is not None:
        _remove_normalized_key(raw_provider, field_id)
    _write_json_config(path, config)
    return path


def _provider_entry_for_write(
    config: dict[str, object],
    provider: str,
    *,
    create: bool,
) -> dict[str, object] | None:
    providers = config.get("providers")
    if providers is None:
        if not create:
            return None
        providers = {}
        config["providers"] = providers
    if not isinstance(providers, dict):
        raise ProviderConfigError("provider config providers must be an object")

    matched_key = next(
        (
            raw_provider
            for raw_provider in providers
            if _normalize_provider(str(raw_provider)) == provider
        ),
        None,
    )
    if matched_key is None:
        if not create:
            return None
        raw_config: object = {}
    else:
        raw_config = providers.pop(matched_key)
    if not isinstance(raw_config, dict):
        raise ProviderConfigError(
            f"provider config providers.{provider} must be an object"
        )
    _canonicalize_known_provider_fields(raw_config, provider)
    providers[provider] = raw_config
    return raw_config


def _canonicalize_known_provider_fields(
    config: dict[str, object],
    provider: str,
) -> None:
    known_fields = {"enabled", *PROVIDER_FIELDS[provider]}
    for raw_field in list(config):
        field = _normalize_field(str(raw_field))
        if field in known_fields and raw_field != field:
            config[field] = config.pop(raw_field)


def _remove_normalized_key(config: dict[str, object], field: str) -> None:
    for raw_field in list(config):
        if _normalize_field(str(raw_field)) == field:
            config.pop(raw_field)


def _empty_config() -> dict[str, object]:
    return {
        "version": 1,
        "providers": {provider: {} for provider in PROVIDER_FIELDS},
        "search": {
            "minimum_productive_providers": 2,
            "allow_platform_search_supplement": True,
        },
    }


def _read_json_config(path: Path) -> dict[str, object]:
    try:
        text = path.read_text(encoding="utf-8")
    except FileNotFoundError:
        return {}
    except OSError as exc:
        raise ProviderConfigError(f"unable to read provider config: {path}") from exc
    try:
        payload = json.loads(text)
    except json.JSONDecodeError as exc:
        raise ProviderConfigError(f"provider config is not valid JSON: {path}") from exc
    if not isinstance(payload, dict):
        raise ProviderConfigError(f"provider config root must be an object: {path}")
    _validate_json_config(payload, path)
    return payload


def _validate_json_config(config: Mapping[str, object], path: Path) -> None:
    if "version" in config:
        version = config["version"]
        if isinstance(version, bool) or not isinstance(version, int) or version < 1:
            raise ProviderConfigError(
                f"provider config version must be a positive integer: {path}"
            )
        if version not in SUPPORTED_PROVIDER_CONFIG_VERSIONS:
            raise ProviderConfigError(
                f"provider config version is not supported: {path}"
            )

    providers = config.get("providers")
    if "providers" in config and not isinstance(providers, Mapping):
        raise ProviderConfigError(
            f"provider config providers must be an object: {path}"
        )
    if isinstance(providers, Mapping):
        seen_providers: set[str] = set()
        for raw_provider, raw_config in providers.items():
            provider = _normalize_provider(str(raw_provider))
            if provider not in PROVIDER_FIELDS:
                continue
            if provider in seen_providers:
                raise ProviderConfigError(
                    f"provider config has conflicting keys for providers.{provider}: {path}"
                )
            seen_providers.add(provider)
            if not isinstance(raw_config, Mapping):
                raise ProviderConfigError(
                    f"provider config providers.{provider} must be an object: {path}"
                )
            seen_fields: set[str] = set()
            for raw_field, value in raw_config.items():
                field = _normalize_field(str(raw_field))
                is_known_field = (
                    field == "enabled" or field in PROVIDER_FIELDS[provider]
                )
                if is_known_field and field in seen_fields:
                    raise ProviderConfigError(
                        "provider config has conflicting keys for "
                        f"providers.{provider}.{field}: {path}"
                    )
                if is_known_field:
                    seen_fields.add(field)
                if field == "enabled" and not isinstance(value, bool):
                    raise ProviderConfigError(
                        f"provider config providers.{provider}.enabled must be a boolean: {path}"
                    )
                if field in PROVIDER_FIELDS[provider] and not isinstance(value, str):
                    raise ProviderConfigError(
                        f"provider config providers.{provider}.{field} must be a string: {path}"
                    )

    search = config.get("search")
    if "search" in config and not isinstance(search, Mapping):
        raise ProviderConfigError(
            f"provider config search must be an object: {path}"
        )
    if isinstance(search, Mapping):
        seen_search_fields: set[str] = set()
        for raw_field, value in search.items():
            field = _normalize_field(str(raw_field))
            expected_type = SEARCH_PUBLIC_FIELD_TYPES.get(field)
            if expected_type is None:
                continue
            if field in seen_search_fields:
                raise ProviderConfigError(
                    f"provider config has conflicting keys for search.{field}: {path}"
                )
            seen_search_fields.add(field)
            if expected_type is int:
                valid = isinstance(value, int) and not isinstance(value, bool)
            else:
                valid = isinstance(value, expected_type)
            if not valid:
                type_name = "an integer" if expected_type is int else "a boolean"
                raise ProviderConfigError(
                    f"provider config search.{field} must be {type_name}: {path}"
                )
            if (
                field == "minimum_productive_providers"
                and isinstance(value, int)
                and value < 1
            ):
                raise ProviderConfigError(
                    "provider config search.minimum_productive_providers "
                    f"must be a positive integer: {path}"
                )


def _write_json_config(path: Path, config: Mapping[str, object]) -> None:
    text = json.dumps(config, indent=2, sort_keys=True) + "\n"
    path.parent.mkdir(parents=True, exist_ok=True)
    descriptor, temporary_name = tempfile.mkstemp(
        dir=path.parent,
        prefix=f".{path.name}.",
        suffix=".tmp",
    )
    temporary_path = Path(temporary_name)
    try:
        if os.name == "posix":
            os.fchmod(descriptor, stat.S_IRUSR | stat.S_IWUSR)
        with os.fdopen(descriptor, "w", encoding="utf-8", newline="\n") as handle:
            descriptor = -1
            handle.write(text)
            handle.flush()
            os.fsync(handle.fileno())
        os.replace(temporary_path, path)
    except BaseException:
        if descriptor >= 0:
            try:
                os.close(descriptor)
            except OSError:
                pass
        try:
            temporary_path.unlink()
        except FileNotFoundError:
            pass
        raise


def _redacted_search_config(raw_search: object) -> dict[str, object]:
    if not isinstance(raw_search, Mapping):
        return {}
    redacted: dict[str, object] = {}
    minimum = raw_search.get("minimum_productive_providers")
    if (
        isinstance(minimum, int)
        and not isinstance(minimum, bool)
        and minimum >= 1
    ):
        redacted["minimum_productive_providers"] = minimum
    allow_supplement = raw_search.get("allow_platform_search_supplement")
    if isinstance(allow_supplement, bool):
        redacted["allow_platform_search_supplement"] = allow_supplement
    return redacted


def _read_project_env_config(root: Path) -> dict[str, object]:
    env_values: dict[str, str] = {}
    for path in _project_paths(root, PROJECT_ENV_FILES):
        env_values.update(_parse_env_file(path))
    return _config_from_env(env_values)


def _read_project_toml_config(root: Path) -> dict[str, object]:
    config: dict[str, object] = {}
    for path in _project_paths(root, PROJECT_TOML_FILES):
        _merge_config(config, _parse_minimal_provider_toml(path))
    return config


def _project_paths(root: Path, names: tuple[str, ...]) -> list[Path]:
    candidates = [root, *root.parents]
    paths: list[Path] = []
    for candidate in candidates:
        for name in names:
            path = candidate / name
            if path.is_file():
                paths.append(path)
    return list(reversed(paths))


def _parse_env_file(path: Path) -> dict[str, str]:
    values: dict[str, str] = {}
    try:
        lines = path.read_text(encoding="utf-8").splitlines()
    except OSError:
        return values
    for raw_line in lines:
        line = raw_line.strip()
        if not line or line.startswith("#") or "=" not in line:
            continue
        key, raw_value = line.split("=", 1)
        key = key.strip()
        value = raw_value.strip()
        if "#" in value:
            value = value.split("#", 1)[0].strip()
        values[key] = _strip_quotes(value)
    return values


def _parse_minimal_provider_toml(path: Path) -> dict[str, object]:
    config: dict[str, object] = {}
    section = ""
    try:
        lines = path.read_text(encoding="utf-8").splitlines()
    except OSError:
        return config
    for raw_line in lines:
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        if line.startswith("[") and line.endswith("]"):
            section = line[1:-1].strip()
            continue
        if "=" not in line:
            continue
        key, raw_value = line.split("=", 1)
        key = _normalize_field(key)
        value = _coerce_toml_value(raw_value)
        if section.startswith("providers."):
            provider_id = _normalize_provider(section.split(".", 1)[1])
            if provider_id in PROVIDER_FIELDS:
                providers = config.setdefault("providers", {})
                if isinstance(providers, dict):
                    provider_cfg = providers.setdefault(provider_id, {})
                    if isinstance(provider_cfg, dict):
                        provider_cfg[key] = value
        elif section == "search":
            search = config.setdefault("search", {})
            if isinstance(search, dict):
                search[key] = value
    return config


def _config_from_env(env: Mapping[str, str]) -> dict[str, object]:
    providers: dict[str, dict[str, object]] = {}
    for provider, fields in PROVIDER_FIELDS.items():
        for field, aliases in fields.items():
            value = _first_env_value(env, aliases)
            if value:
                provider_cfg = providers.setdefault(provider, {"enabled": True})
                provider_cfg[field] = value
    return {"providers": providers} if providers else {}


def _first_env_value(env: Mapping[str, str], aliases: tuple[str, ...]) -> str:
    for alias in aliases:
        value = str(env.get(alias, "")).strip()
        if value:
            return value
    return ""


def _merge_config(target: dict[str, object], overlay: Mapping[str, object]) -> None:
    if not overlay:
        return
    providers = overlay.get("providers")
    if isinstance(providers, Mapping):
        target_providers = target.setdefault("providers", {})
        if isinstance(target_providers, dict):
            for provider, raw_provider in providers.items():
                provider_id = _normalize_provider(str(provider))
                if provider_id not in PROVIDER_FIELDS or not isinstance(raw_provider, Mapping):
                    continue
                current = target_providers.setdefault(provider_id, {"enabled": False, "configured": False})
                if isinstance(current, dict):
                    for key, value in raw_provider.items():
                        field = _normalize_field(str(key))
                        if field == "enabled" or field in PROVIDER_FIELDS[provider_id]:
                            current[field] = value
    search = overlay.get("search")
    if isinstance(search, Mapping):
        target_search = target.setdefault("search", {})
        if isinstance(target_search, dict):
            for key, value in search.items():
                target_search[_normalize_field(str(key))] = value


def _finalize_config(config: dict[str, object]) -> None:
    providers = config.setdefault("providers", {})
    if not isinstance(providers, dict):
        return
    for provider, fields in PROVIDER_FIELDS.items():
        raw = providers.setdefault(provider, {})
        if not isinstance(raw, dict):
            raw = {}
            providers[provider] = raw
        if provider == "openalex":
            configured = bool(str(raw.get("api_key", "")).strip())
        elif not fields:
            configured = True
        else:
            configured = any(str(raw.get(field, "")).strip() for field in fields)
        raw["configured"] = configured
        raw["enabled"] = bool(raw.get("enabled", configured))


def _normalize_provider(value: str) -> str:
    normalized = value.strip().lower().replace("-", "_")
    aliases = {"s2": "semantic_scholar", "semanticscholar": "semantic_scholar", "ncbi": "pubmed"}
    return aliases.get(normalized, normalized)


def _normalize_field(value: str) -> str:
    return value.strip().lower().replace("-", "_")


def _assert_known_field(provider: str, field: str) -> None:
    if provider not in PROVIDER_FIELDS:
        raise ValueError(f"unknown provider: {provider}")
    if field not in PROVIDER_FIELDS[provider]:
        raise ValueError(f"unknown field for {provider}: {field}")


def _strip_quotes(value: str) -> str:
    if len(value) >= 2 and ((value.startswith('"') and value.endswith('"')) or (value.startswith("'") and value.endswith("'"))):
        return value[1:-1]
    return value


def _coerce_toml_value(raw_value: str) -> object:
    value = raw_value.strip()
    if "#" in value:
        value = value.split("#", 1)[0].strip()
    lowered = value.lower()
    if lowered == "true":
        return True
    if lowered == "false":
        return False
    if value.isdigit():
        return int(value)
    return _strip_quotes(value)
