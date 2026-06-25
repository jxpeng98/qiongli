from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from bridges.provider_config import (
    global_provider_config_path,
    provider_config_env,
    provider_config_summary,
    provider_capability_mode,
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
                    "openalex": {
                        "enabled": True,
                        "api_key": "global-openalex-key",
                        "email": "global@example.com",
                    },
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
    assert resolved["providers"]["openalex"]["api_key"] == "global-openalex-key"
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
        "QIONGLI_OPENALEX_API_KEY=project-openalex-key\n"
        "QIONGLI_OPENALEX_EMAIL=project@example.com\n",
        encoding="utf-8",
    )
    monkeypatch.setenv("QIONGLI_CONFIG_HOME", str(config_home))

    resolved = resolve_provider_config(cwd=project, env={})

    assert resolved["providers"]["semantic_scholar"]["api_key"] == "project-key"
    assert resolved["providers"]["openalex"]["api_key"] == "project-openalex-key"
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
            "QIONGLI_OPENALEX_API_KEY": "openalex-secret-key",
            "QIONGLI_SEMANTIC_SCHOLAR_API_KEY": "secret-demo-key",
            "QIONGLI_OPENALEX_EMAIL": "user@example.com",
        },
    )

    redacted = redact_provider_config(resolved)
    rendered = json.dumps(redacted, sort_keys=True)

    assert "openalex-secret-key" not in rendered
    assert "secret-demo-key" not in rendered
    assert "user@example.com" not in rendered
    assert redacted["providers"]["semantic_scholar"]["fields"]["api_key"] == "configured"
    assert redacted["providers"]["openalex"]["fields"]["api_key"] == "configured"
    assert redacted["providers"]["openalex"]["fields"]["email"] == "configured"


def test_openalex_email_alone_does_not_mark_provider_configured(tmp_path: Path, monkeypatch) -> None:
    monkeypatch.setenv("QIONGLI_CONFIG_HOME", str(tmp_path / "config"))
    set_provider_value("openalex", "email", "user@example.com")

    resolved = resolve_provider_config(cwd=tmp_path, env={})
    redacted = redact_provider_config(resolved)

    assert resolved["providers"]["openalex"]["configured"] is False
    assert redacted["providers"]["openalex"]["configured"] is False
    assert redacted["providers"]["openalex"]["fields"]["email"] == "configured"
    assert redacted["providers"]["openalex"]["fields"]["api_key"] == "missing"


def test_arxiv_is_available_without_provider_credentials(tmp_path: Path, monkeypatch) -> None:
    monkeypatch.setenv("QIONGLI_CONFIG_HOME", str(tmp_path / "config"))

    resolved = resolve_provider_config(cwd=tmp_path, env={})
    redacted = redact_provider_config(resolved)

    assert resolved["providers"]["arxiv"]["enabled"] is True
    assert resolved["providers"]["arxiv"]["configured"] is True
    assert redacted["providers"]["arxiv"]["configured"] is True
    assert redacted["providers"]["arxiv"]["fields"] == {}
    assert provider_config_summary(resolved)["arxiv"] == "configured"
    assert provider_capability_mode(provider_config_summary(resolved)) == "provider_connected"


def test_provider_capability_mode_reports_connected_only_when_configured() -> None:
    assert provider_capability_mode(
        {
            "openalex": "missing",
            "semantic_scholar": "missing",
            "crossref": "missing",
            "pubmed": "missing",
            "arxiv": "missing",
        }
    ) == "strategy_only"

    assert provider_capability_mode(
        {
            "openalex": "configured",
            "semantic_scholar": "configured",
            "crossref": "missing",
            "pubmed": "missing",
            "arxiv": "missing",
        }
    ) == "provider_connected"

    assert provider_capability_mode(
        {
            "openalex": "missing",
            "semantic_scholar": "missing",
            "crossref": "missing",
            "pubmed": "missing",
            "arxiv": "configured",
        }
    ) == "provider_connected"


def test_set_and_unset_provider_value_round_trip_global_config(tmp_path: Path, monkeypatch) -> None:
    config_home = tmp_path / "config"
    monkeypatch.setenv("QIONGLI_CONFIG_HOME", str(config_home))

    set_provider_value("semantic-scholar", "api-key", "stored-key")
    resolved = resolve_provider_config(cwd=tmp_path, env={})

    assert resolved["providers"]["semantic_scholar"]["api_key"] == "stored-key"

    unset_provider_value("semantic-scholar", "api-key")
    resolved_after_unset = resolve_provider_config(cwd=tmp_path, env={})

    assert resolved_after_unset["providers"]["semantic_scholar"]["configured"] is False


def test_provider_config_env_emits_primary_and_legacy_aliases_without_redaction(
    tmp_path: Path,
    monkeypatch,
) -> None:
    monkeypatch.setenv("QIONGLI_CONFIG_HOME", str(tmp_path / "config"))
    set_provider_value("openalex", "api-key", "openalex-secret-key")
    set_provider_value("openalex", "email", "user@example.com")
    set_provider_value("semantic-scholar", "api-key", "stored-key")

    config = resolve_provider_config(cwd=tmp_path, env={})
    env = provider_config_env(config)

    assert env["QIONGLI_OPENALEX_API_KEY"] == "openalex-secret-key"
    assert env["OPENALEX_API_KEY"] == "openalex-secret-key"
    assert env["QIONGLI_MCPB_OPENALEX_API_KEY"] == "openalex-secret-key"
    assert env["QIONGLI_OPENALEX_EMAIL"] == "user@example.com"
    assert env["OPENALEX_EMAIL"] == "user@example.com"
    assert env["QIONGLI_MCPB_OPENALEX_EMAIL"] == "user@example.com"
    assert env["QIONGLI_SEMANTIC_SCHOLAR_API_KEY"] == "stored-key"
    assert env["SEMANTIC_SCHOLAR_API_KEY"] == "stored-key"
    assert env["S2_API_KEY"] == "stored-key"


class ProviderConfigEnvTests(unittest.TestCase):
    def test_arxiv_is_available_without_provider_credentials(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            with mock.patch.dict(
                "os.environ",
                {"QIONGLI_CONFIG_HOME": str(root / "config")},
                clear=False,
            ):
                resolved = resolve_provider_config(cwd=root, env={})
                redacted = redact_provider_config(resolved)

        self.assertIs(resolved["providers"]["arxiv"]["enabled"], True)
        self.assertIs(resolved["providers"]["arxiv"]["configured"], True)
        self.assertIs(redacted["providers"]["arxiv"]["configured"], True)
        self.assertEqual(redacted["providers"]["arxiv"]["fields"], {})
        self.assertEqual(provider_config_summary(resolved)["arxiv"], "configured")
        self.assertEqual(provider_capability_mode(provider_config_summary(resolved)), "provider_connected")

    def test_provider_config_env_emits_primary_and_legacy_aliases_without_redaction(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            with mock.patch.dict(
                "os.environ",
                {"QIONGLI_CONFIG_HOME": str(root / "config")},
                clear=False,
            ):
                set_provider_value("openalex", "api-key", "openalex-secret-key")
                set_provider_value("openalex", "email", "user@example.com")
                set_provider_value("semantic-scholar", "api-key", "stored-key")

                config = resolve_provider_config(cwd=root, env={})
                env = provider_config_env(config)

        self.assertEqual(env["QIONGLI_OPENALEX_API_KEY"], "openalex-secret-key")
        self.assertEqual(env["OPENALEX_API_KEY"], "openalex-secret-key")
        self.assertEqual(env["QIONGLI_MCPB_OPENALEX_API_KEY"], "openalex-secret-key")
        self.assertEqual(env["QIONGLI_OPENALEX_EMAIL"], "user@example.com")
        self.assertEqual(env["OPENALEX_EMAIL"], "user@example.com")
        self.assertEqual(env["QIONGLI_MCPB_OPENALEX_EMAIL"], "user@example.com")
        self.assertEqual(env["QIONGLI_SEMANTIC_SCHOLAR_API_KEY"], "stored-key")
        self.assertEqual(env["SEMANTIC_SCHOLAR_API_KEY"], "stored-key")
        self.assertEqual(env["S2_API_KEY"], "stored-key")
