from __future__ import annotations

import json
import os
import stat
from pathlib import Path
from typing import Mapping


PROVIDER_FIELDS: dict[str, dict[str, tuple[str, ...]]] = {
    "openalex": {
        "email": ("QIONGLI_OPENALEX_EMAIL", "OPENALEX_EMAIL"),
    },
    "semantic_scholar": {
        "api_key": ("QIONGLI_SEMANTIC_SCHOLAR_API_KEY", "SEMANTIC_SCHOLAR_API_KEY", "S2_API_KEY"),
    },
    "crossref": {
        "email": ("QIONGLI_CROSSREF_EMAIL", "CROSSREF_EMAIL"),
    },
    "pubmed": {
        "api_key": ("QIONGLI_NCBI_API_KEY", "NCBI_API_KEY", "PUBMED_API_KEY"),
    },
}

PROJECT_ENV_FILES = (".env",)
PROJECT_TOML_FILES = ("qiongli.toml", ".qiongli.toml")


def global_provider_config_path() -> Path:
    config_home = os.environ.get("QIONGLI_CONFIG_HOME", "").strip()
    root = Path(config_home).expanduser() if config_home else Path.home() / ".config" / "qiongli"
    return root / "providers.json"


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
    redacted: dict[str, object] = {"providers": {}, "search": dict(config.get("search", {}) or {})}
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


def provider_capability_mode(summary: Mapping[str, str]) -> str:
    academic_providers = ("openalex", "semantic_scholar", "crossref", "pubmed")
    if any(summary.get(provider) == "configured" for provider in academic_providers):
        return "provider_connected"
    return "strategy_only"


def set_provider_value(provider: str, field: str, value: str, *, project_dir: Path | str | None = None) -> Path:
    if project_dir is not None:
        raise NotImplementedError("project provider writes are not implemented yet")
    provider_id = _normalize_provider(provider)
    field_id = _normalize_field(field)
    _assert_known_field(provider_id, field_id)
    path = global_provider_config_path()
    config = _read_json_config(path)
    providers = config.setdefault("providers", {})
    if not isinstance(providers, dict):
        providers = {}
        config["providers"] = providers
    raw_provider = providers.setdefault(provider_id, {})
    if not isinstance(raw_provider, dict):
        raw_provider = {}
        providers[provider_id] = raw_provider
    raw_provider["enabled"] = True
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
    providers = config.get("providers", {})
    if isinstance(providers, dict):
        raw_provider = providers.get(provider_id, {})
        if isinstance(raw_provider, dict):
            raw_provider.pop(field_id, None)
    _write_json_config(path, config)
    return path


def _empty_config() -> dict[str, object]:
    return {
        "version": 1,
        "providers": {
            provider: {"enabled": False, "configured": False}
            for provider in PROVIDER_FIELDS
        },
        "search": {
            "minimum_productive_providers": 2,
            "allow_platform_search_supplement": True,
        },
    }


def _read_json_config(path: Path) -> dict[str, object]:
    if not path.is_file():
        return {}
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return {}
    return payload if isinstance(payload, dict) else {}


def _write_json_config(path: Path, config: Mapping[str, object]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(config, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    try:
        path.chmod(stat.S_IRUSR | stat.S_IWUSR)
    except OSError:
        pass


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
