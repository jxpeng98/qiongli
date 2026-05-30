from __future__ import annotations

import json
from pathlib import Path

from bridges.provider_config import (
    global_provider_config_path,
    redact_provider_config,
    resolve_provider_config,
    set_provider_value,
    unset_provider_value,
)


def test_resolve_provider_config_uses_global_config(tmp_path: Path, monkeypatch) -> None:
    config_home = tmp_path / "config"
    config_path = config_home / "providers.json"
    config_path.parent.mkdir(parents=True)
    config_path.write_text(
        json.dumps(
            {
                "version": 1,
                "providers": {
                    "openalex": {"enabled": True, "email": "global@example.com"},
                    "semantic_scholar": {"enabled": True, "api_key": "global-s2-key"},
                },
            }
        ),
        encoding="utf-8",
    )
    monkeypatch.setenv("QIONGLI_CONFIG_HOME", str(config_home))
    monkeypatch.delenv("QIONGLI_OPENALEX_EMAIL", raising=False)
    monkeypatch.delenv("QIONGLI_SEMANTIC_SCHOLAR_API_KEY", raising=False)
    monkeypatch.delenv("SEMANTIC_SCHOLAR_API_KEY", raising=False)

    resolved = resolve_provider_config(cwd=tmp_path, env={})

    assert resolved["providers"]["openalex"]["email"] == "global@example.com"
    assert resolved["providers"]["openalex"]["configured"] is True
    assert resolved["providers"]["semantic_scholar"]["api_key"] == "global-s2-key"
    assert resolved["providers"]["semantic_scholar"]["configured"] is True
    assert global_provider_config_path() == config_path


def test_project_env_overrides_global_config(tmp_path: Path, monkeypatch) -> None:
    config_home = tmp_path / "config"
    config_path = config_home / "providers.json"
    config_path.parent.mkdir(parents=True)
    config_path.write_text(
        json.dumps({"version": 1, "providers": {"semantic_scholar": {"api_key": "global-key"}}}),
        encoding="utf-8",
    )
    project = tmp_path / "project"
    project.mkdir()
    (project / ".env").write_text(
        "QIONGLI_SEMANTIC_SCHOLAR_API_KEY=project-key\n"
        "QIONGLI_OPENALEX_EMAIL=project@example.com\n",
        encoding="utf-8",
    )
    monkeypatch.setenv("QIONGLI_CONFIG_HOME", str(config_home))

    resolved = resolve_provider_config(cwd=project, env={})

    assert resolved["providers"]["semantic_scholar"]["api_key"] == "project-key"
    assert resolved["providers"]["openalex"]["email"] == "project@example.com"


def test_process_env_overrides_project_config_and_legacy_s2_env(tmp_path: Path, monkeypatch) -> None:
    (tmp_path / ".env").write_text("QIONGLI_SEMANTIC_SCHOLAR_API_KEY=project-key\n", encoding="utf-8")
    monkeypatch.setenv("QIONGLI_CONFIG_HOME", str(tmp_path / "config"))

    resolved = resolve_provider_config(
        cwd=tmp_path,
        env={
            "SEMANTIC_SCHOLAR_API_KEY": "legacy-key",
            "QIONGLI_SEMANTIC_SCHOLAR_API_KEY": "env-key",
        },
    )

    assert resolved["providers"]["semantic_scholar"]["api_key"] == "env-key"

    legacy_only = resolve_provider_config(
        cwd=tmp_path,
        env={"SEMANTIC_SCHOLAR_API_KEY": "legacy-key"},
    )

    assert legacy_only["providers"]["semantic_scholar"]["api_key"] == "legacy-key"


def test_redacted_config_never_exposes_secret_values(tmp_path: Path, monkeypatch) -> None:
    monkeypatch.setenv("QIONGLI_CONFIG_HOME", str(tmp_path / "config"))

    resolved = resolve_provider_config(
        cwd=tmp_path,
        env={
            "QIONGLI_SEMANTIC_SCHOLAR_API_KEY": "secret-demo-key",
            "QIONGLI_OPENALEX_EMAIL": "user@example.com",
        },
    )

    redacted = redact_provider_config(resolved)
    rendered = json.dumps(redacted, sort_keys=True)

    assert "secret-demo-key" not in rendered
    assert "user@example.com" not in rendered
    assert redacted["providers"]["semantic_scholar"]["fields"]["api_key"] == "configured"
    assert redacted["providers"]["openalex"]["fields"]["email"] == "configured"


def test_set_and_unset_provider_value_round_trip_global_config(tmp_path: Path, monkeypatch) -> None:
    config_home = tmp_path / "config"
    monkeypatch.setenv("QIONGLI_CONFIG_HOME", str(config_home))

    set_provider_value("semantic-scholar", "api-key", "stored-key")
    resolved = resolve_provider_config(cwd=tmp_path, env={})

    assert resolved["providers"]["semantic_scholar"]["api_key"] == "stored-key"

    unset_provider_value("semantic-scholar", "api-key")
    resolved_after_unset = resolve_provider_config(cwd=tmp_path, env={})

    assert resolved_after_unset["providers"]["semantic_scholar"]["configured"] is False
