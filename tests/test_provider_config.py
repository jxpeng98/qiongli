from __future__ import annotations

import json
import os
import stat
import tempfile
import unittest
from pathlib import Path
from typing import cast
from unittest import mock

from bridges import provider_config as provider_config_module
from bridges.provider_config import (
    ProviderConfigError,
    active_provider_names,
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
    assert provider_capability_mode(resolved) == "provider_connected"


def test_provider_capability_mode_reports_connected_only_when_active() -> None:
    assert provider_capability_mode(
        {
            "providers": {
                "openalex": {"configured": False, "enabled": False},
                "semantic_scholar": {"configured": False, "enabled": False},
                "crossref": {"configured": False, "enabled": False},
                "pubmed": {"configured": False, "enabled": False},
                "arxiv": {"configured": False, "enabled": False},
            }
        }
    ) == "strategy_only"

    assert provider_capability_mode(
        {
            "providers": {
                "openalex": {"configured": True, "enabled": False},
                "semantic_scholar": {"configured": True, "enabled": False},
                "crossref": {"configured": False, "enabled": False},
                "pubmed": {"configured": False, "enabled": False},
                "arxiv": {"configured": False, "enabled": False},
            }
        }
    ) == "strategy_only"

    assert provider_capability_mode(
        {
            "providers": {
                "openalex": {"configured": True, "enabled": True},
                "semantic_scholar": {"configured": False, "enabled": False},
                "crossref": {"configured": False, "enabled": False},
                "pubmed": {"configured": False, "enabled": False},
                "arxiv": {"configured": False, "enabled": False},
            }
        }
    ) == "provider_connected"


def test_provider_config_env_omits_disabled_provider_credentials(
    tmp_path: Path,
    monkeypatch,
) -> None:
    config_home = tmp_path / "config"
    config_home.mkdir()
    (config_home / "providers.json").write_text(
        json.dumps(
            {
                "version": 1,
                "providers": {
                    "semantic_scholar": {
                        "enabled": False,
                        "api_key": "disabled-secret-canary",
                    }
                },
            }
        ),
        encoding="utf-8",
    )
    monkeypatch.setenv("QIONGLI_CONFIG_HOME", str(config_home))

    resolved = resolve_provider_config(cwd=tmp_path, env={})
    emitted = provider_config_env(resolved)

    assert resolved["providers"]["semantic_scholar"]["configured"] is True
    assert resolved["providers"]["semantic_scholar"]["enabled"] is False
    assert provider_capability_mode(resolved) == "provider_connected"
    assert not any("SEMANTIC_SCHOLAR" in name or name == "S2_API_KEY" for name in emitted)
    assert "disabled-secret-canary" not in emitted.values()


def test_all_disabled_providers_are_configured_but_inactive(
    tmp_path: Path,
    monkeypatch,
) -> None:
    config_home = tmp_path / "config"
    config_home.mkdir()
    (config_home / "providers.json").write_text(
        json.dumps(
            {
                "version": 1,
                "providers": {
                    "openalex": {"enabled": False, "api_key": "openalex-canary"},
                    "semantic_scholar": {"enabled": False, "api_key": "s2-canary"},
                    "crossref": {"enabled": False, "email": "crossref@example.com"},
                    "pubmed": {"enabled": False, "api_key": "pubmed-canary"},
                    "arxiv": {"enabled": False},
                },
            }
        ),
        encoding="utf-8",
    )
    monkeypatch.setenv("QIONGLI_CONFIG_HOME", str(config_home))

    resolved = resolve_provider_config(cwd=tmp_path, env={})

    assert all(status == "configured" for status in provider_config_summary(resolved).values())
    assert active_provider_names(resolved) == []
    assert provider_capability_mode(resolved) == "strategy_only"
    assert provider_config_env(resolved) == {}


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


class ProviderConfigSecurityTests(unittest.TestCase):
    def test_fully_qualified_config_home_uses_windows_drive_and_root_semantics(self) -> None:
        for value in (r"C:\config", "C:/config", r"\\server\share\config"):
            with self.subTest(valid=value):
                self.assertTrue(
                    provider_config_module._is_fully_qualified_config_home(
                        value,
                        windows=True,
                    )
                )

        for value in (r"\config", "/config", "C:relative", "relative"):
            with self.subTest(invalid=value):
                self.assertFalse(
                    provider_config_module._is_fully_qualified_config_home(
                        value,
                        windows=True,
                    )
                )

        self.assertTrue(
            provider_config_module._is_fully_qualified_config_home(
                "/config",
                windows=False,
            )
        )
        self.assertFalse(
            provider_config_module._is_fully_qualified_config_home(
                "relative",
                windows=False,
            )
        )

    def test_config_home_expands_platform_home_and_default_never_uses_cwd(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            home = root / "home"
            cwd = root / "worktree"
            home.mkdir()
            cwd.mkdir()
            home_env = (
                {"USERPROFILE": str(home)}
                if os.name == "nt"
                else {"HOME": str(home)}
            )
            original_cwd = Path.cwd()
            try:
                os.chdir(cwd)
                with mock.patch.dict(
                    os.environ,
                    {
                        **home_env,
                        "QIONGLI_CONFIG_HOME": "~/shared-config",
                    },
                    clear=True,
                ):
                    self.assertEqual(
                        global_provider_config_path(),
                        home / "shared-config" / "providers.json",
                    )

                with mock.patch.dict(
                    os.environ,
                    {**home_env, "QIONGLI_CONFIG_HOME": "~"},
                    clear=True,
                ):
                    self.assertEqual(
                        global_provider_config_path(),
                        home / "providers.json",
                    )

                with mock.patch.dict(
                    os.environ,
                    home_env,
                    clear=True,
                ):
                    default_path = global_provider_config_path()
            finally:
                os.chdir(original_cwd)

        self.assertEqual(
            default_path,
            home / ".config" / "qiongli" / "providers.json",
        )
        self.assertNotEqual(default_path.parent, cwd)

    def test_relative_config_home_fails_closed_without_cwd_write_or_path_leak(self) -> None:
        expected_error = (
            "QIONGLI_CONFIG_HOME must be a fully qualified absolute path "
            "or use '~' home notation"
        )
        relative_home = "relative-config-canary"
        invalid_homes = (
            relative_home,
            "~//abs",
            "~/\\abs",
            "~/C:\\abs",
            "~/C:relative",
        )
        secret = "provider-secret-canary"
        observed_errors: list[ProviderConfigError] = []
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            original_cwd = Path.cwd()
            try:
                os.chdir(root)
                with mock.patch.object(
                    provider_config_module,
                    "_read_json_config",
                    return_value={},
                ), mock.patch.object(
                    provider_config_module,
                    "_write_json_config",
                ) as write_json_config:
                    for invalid_home in invalid_homes:
                        with self.subTest(config_home=invalid_home), mock.patch.dict(
                            os.environ,
                            {"QIONGLI_CONFIG_HOME": invalid_home},
                            clear=True,
                        ):
                            with self.assertRaises(ProviderConfigError) as path_error:
                                global_provider_config_path()
                            with self.assertRaises(ProviderConfigError) as write_error:
                                set_provider_value("openalex", "api-key", secret)
                            observed_errors.extend(
                                (path_error.exception, write_error.exception)
                            )
                    write_json_config.assert_not_called()
            finally:
                os.chdir(original_cwd)

            self.assertFalse((root / relative_home).exists())

        for error in observed_errors:
            rendered = str(error)
            self.assertEqual(rendered, expected_error)
            for invalid_home in invalid_homes:
                self.assertNotIn(invalid_home, rendered)
            self.assertNotIn(secret, rendered)
            self.assertNotIn(str(root), rendered)

    def test_redacted_config_only_exposes_typed_public_search_settings(self) -> None:
        canary = "QIONGLI_SEARCH_CANARY_DO_NOT_ECHO"
        redacted = redact_provider_config(
            {
                "providers": {},
                "search": {
                    "minimum_productive_providers": 3,
                    "allow_platform_search_supplement": False,
                    "private_note": canary,
                },
            }
        )

        self.assertEqual(
            redacted["search"],
            {
                "minimum_productive_providers": 3,
                "allow_platform_search_supplement": False,
            },
        )
        self.assertNotIn(canary, json.dumps(redacted, sort_keys=True))

        mistyped = redact_provider_config(
            {
                "providers": {},
                "search": {
                    "minimum_productive_providers": canary,
                    "allow_platform_search_supplement": canary,
                },
            }
        )
        self.assertEqual(mistyped["search"], {})
        self.assertNotIn(canary, json.dumps(mistyped, sort_keys=True))

        nonpositive = redact_provider_config(
            {
                "providers": {},
                "search": {"minimum_productive_providers": 0},
            }
        )
        self.assertEqual(nonpositive["search"], {})

    def test_malformed_global_config_fails_closed_without_overwrite(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            config_home = root / "config"
            config_home.mkdir()
            config_path = config_home / "providers.json"
            malformed = b'{"version":1,"providers":'
            config_path.write_bytes(malformed)
            with mock.patch.dict(
                os.environ,
                {"QIONGLI_CONFIG_HOME": str(config_home)},
                clear=False,
            ):
                with self.assertRaisesRegex(
                    ProviderConfigError,
                    "provider config is not valid JSON",
                ):
                    resolve_provider_config(cwd=root, env={})
                with self.assertRaisesRegex(
                    ProviderConfigError,
                    "provider config is not valid JSON",
                ):
                    set_provider_value("openalex", "api-key", "must-not-be-written")

            self.assertEqual(config_path.read_bytes(), malformed)
            self.assertEqual(list(config_home.glob(".providers.json.*.tmp")), [])

    def test_invalid_known_config_structure_fails_closed_without_overwrite(self) -> None:
        invalid_cases: dict[str, object] = {
            "root": [],
            "version_bool": {"version": True},
            "version_zero": {"version": 0},
            "version_unsupported": {"version": 2},
            "providers": {"providers": []},
            "search": {"search": []},
            "known_provider": {"providers": {"openalex": []}},
            "enabled": {"providers": {"arxiv": {"enabled": "false"}}},
            "credential": {
                "providers": {"semantic-scholar": {"api-key": 7}}
            },
            "provider_alias_collision": {
                "providers": {
                    "semantic-scholar": {"api-key": "first"},
                    "semantic_scholar": {"api_key": "second"},
                }
            },
            "field_alias_collision": {
                "providers": {
                    "semantic_scholar": {
                        "api-key": "first",
                        "api_key": "second",
                    }
                }
            },
            "search_alias_collision": {
                "search": {
                    "minimum-productive-providers": 2,
                    "minimum_productive_providers": 3,
                }
            },
            "minimum_bool": {"search": {"minimum_productive_providers": True}},
            "minimum_zero": {"search": {"minimum_productive_providers": 0}},
            "minimum_negative": {
                "search": {"minimum_productive_providers": -1}
            },
            "supplement": {
                "search": {"allow_platform_search_supplement": "true"}
            },
        }
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            config_home = root / "config"
            config_home.mkdir()
            config_path = config_home / "providers.json"
            with mock.patch.dict(
                os.environ,
                {"QIONGLI_CONFIG_HOME": str(config_home)},
                clear=True,
            ):
                for label, payload in invalid_cases.items():
                    with self.subTest(case=label):
                        original = json.dumps(payload, sort_keys=True).encode("utf-8")
                        config_path.write_bytes(original)
                        with self.assertRaises(ProviderConfigError):
                            resolve_provider_config(cwd=root, env={})
                        with self.assertRaises(ProviderConfigError):
                            set_provider_value(
                                "openalex",
                                "api-key",
                                "must-not-be-written",
                            )
                        self.assertEqual(config_path.read_bytes(), original)
                        self.assertEqual(
                            list(config_home.glob(".providers.json.*.tmp")),
                            [],
                        )

    def test_unknown_future_fields_are_preserved_during_known_field_save(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            config_home = root / "config"
            config_home.mkdir()
            config_path = config_home / "providers.json"
            original = {
                "version": 1,
                "future_top_level": {"opaque": [1, 2, 3]},
                "providers": {
                    "openalex": {
                        "enabled": False,
                        "future_provider_field": {"opaque": True},
                    },
                    "future_provider": ["opaque", {"shape": "unknown"}],
                },
                "search": {"future_search_field": {"opaque": "value"}},
            }
            config_path.write_text(json.dumps(original), encoding="utf-8")
            with mock.patch.dict(
                os.environ,
                {"QIONGLI_CONFIG_HOME": str(config_home)},
                clear=True,
            ):
                set_provider_value("openalex", "api-key", "stored-key")

            persisted = json.loads(config_path.read_text(encoding="utf-8"))
            self.assertEqual(
                persisted["future_top_level"],
                original["future_top_level"],
            )
            self.assertEqual(
                persisted["providers"]["future_provider"],
                original["providers"]["future_provider"],
            )
            self.assertEqual(
                persisted["providers"]["openalex"]["future_provider_field"],
                original["providers"]["openalex"]["future_provider_field"],
            )
            self.assertEqual(persisted["search"], original["search"])
            self.assertEqual(
                persisted["providers"]["openalex"]["api_key"],
                "stored-key",
            )

    def test_legacy_provider_and_field_keys_migrate_to_canonical_on_save(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            config_home = root / "config"
            config_home.mkdir()
            config_path = config_home / "providers.json"
            config_path.write_text(
                json.dumps(
                    {
                        "version": 1,
                        "providers": {
                            "semantic-scholar": {
                                "enabled": False,
                                "api-key": "old-key",
                                "future-field": {"opaque": True},
                            }
                        },
                    }
                ),
                encoding="utf-8",
            )
            with mock.patch.dict(
                os.environ,
                {"QIONGLI_CONFIG_HOME": str(config_home)},
                clear=True,
            ):
                set_provider_value(
                    "semantic_scholar",
                    "api_key",
                    "replacement-key",
                )
                resolved = resolve_provider_config(cwd=root, env={})

            persisted = json.loads(config_path.read_text(encoding="utf-8"))
            providers = persisted["providers"]
            self.assertNotIn("semantic-scholar", providers)
            self.assertIn("semantic_scholar", providers)
            provider = providers["semantic_scholar"]
            self.assertNotIn("api-key", provider)
            self.assertEqual(provider["api_key"], "replacement-key")
            self.assertEqual(provider["future-field"], {"opaque": True})
            self.assertIs(provider["enabled"], True)
            self.assertEqual(
                resolved["providers"]["semantic_scholar"]["api_key"],
                "replacement-key",
            )

    def test_explicitly_disabled_arxiv_is_configured_but_not_active(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            config_home = root / "config"
            config_home.mkdir()
            (config_home / "providers.json").write_text(
                json.dumps(
                    {
                        "version": 1,
                        "providers": {"arxiv": {"enabled": False}},
                    }
                ),
                encoding="utf-8",
            )
            with mock.patch.dict(
                os.environ,
                {"QIONGLI_CONFIG_HOME": str(config_home)},
                clear=True,
            ):
                resolved = resolve_provider_config(cwd=root, env={})

        self.assertIs(resolved["providers"]["arxiv"]["configured"], True)
        self.assertIs(resolved["providers"]["arxiv"]["enabled"], False)
        self.assertEqual(active_provider_names(resolved), [])

    def test_unreadable_global_config_fails_closed_without_overwrite(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            config_home = root / "config"
            config_home.mkdir()
            config_path = config_home / "providers.json"
            original = b'{"version":1,"providers":{}}\n'
            config_path.write_bytes(original)
            real_read_text = Path.read_text

            def deny_provider_config_read(
                path: Path,
                *args: object,
                **kwargs: object,
            ) -> str:
                if path == config_path:
                    raise PermissionError("permission denied")
                return real_read_text(path, *args, **kwargs)

            with mock.patch.dict(
                os.environ,
                {"QIONGLI_CONFIG_HOME": str(config_home)},
                clear=False,
            ), mock.patch.object(Path, "read_text", deny_provider_config_read):
                with self.assertRaisesRegex(
                    ProviderConfigError,
                    "unable to read provider config",
                ):
                    resolve_provider_config(cwd=root, env={})
                with self.assertRaisesRegex(
                    ProviderConfigError,
                    "unable to read provider config",
                ):
                    set_provider_value("openalex", "api-key", "must-not-be-written")

            self.assertEqual(config_path.read_bytes(), original)
            self.assertEqual(list(config_home.glob(".providers.json.*.tmp")), [])

    def test_write_fsyncs_and_atomically_replaces_from_same_directory(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            config_home = Path(tmp_dir) / "config"
            observed: dict[str, object] = {"fsync_calls": 0}
            real_fsync = os.fsync
            real_replace = os.replace

            def observe_fsync(descriptor: int) -> None:
                observed["fsync_calls"] = int(observed["fsync_calls"]) + 1
                real_fsync(descriptor)

            def observe_replace(
                source: os.PathLike[str],
                destination: os.PathLike[str],
            ) -> None:
                source_path = Path(source)
                destination_path = Path(destination)
                observed["source"] = source_path
                observed["destination"] = destination_path
                observed["payload"] = json.loads(source_path.read_text(encoding="utf-8"))
                if os.name == "posix":
                    observed["temporary_mode"] = stat.S_IMODE(source_path.stat().st_mode)
                real_replace(source_path, destination_path)

            with mock.patch.dict(
                os.environ,
                {"QIONGLI_CONFIG_HOME": str(config_home)},
                clear=False,
            ), mock.patch.object(
                provider_config_module.os,
                "fsync",
                observe_fsync,
            ), mock.patch.object(
                provider_config_module.os,
                "replace",
                observe_replace,
            ):
                config_path = set_provider_value(
                    "semantic-scholar",
                    "api-key",
                    "stored-key",
                )

            source_path = observed["source"]
            self.assertIsInstance(source_path, Path)
            source_path = cast(Path, source_path)
            self.assertEqual(source_path.parent, config_home)
            self.assertEqual(observed["destination"], config_path)
            self.assertEqual(observed["fsync_calls"], 1)
            self.assertEqual(
                observed["payload"]["providers"]["semantic_scholar"]["api_key"],
                "stored-key",
            )
            self.assertFalse(source_path.exists())
            if os.name == "posix":
                self.assertEqual(observed["temporary_mode"], 0o600)
                self.assertEqual(stat.S_IMODE(config_path.stat().st_mode), 0o600)

    def test_replace_failure_preserves_original_and_cleans_temporary_file(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            config_home = Path(tmp_dir) / "config"
            config_home.mkdir()
            config_path = config_home / "providers.json"
            original = b'{"version":1,"providers":{"crossref":{"enabled":true}}}\n'
            config_path.write_bytes(original)
            observed: dict[str, Path] = {}

            def fail_replace(
                source: os.PathLike[str],
                destination: os.PathLike[str],
            ) -> None:
                observed["source"] = Path(source)
                observed["destination"] = Path(destination)
                raise OSError("replace failed")

            with mock.patch.dict(
                os.environ,
                {"QIONGLI_CONFIG_HOME": str(config_home)},
                clear=False,
            ), mock.patch.object(
                provider_config_module.os,
                "replace",
                fail_replace,
            ):
                with self.assertRaisesRegex(OSError, "replace failed"):
                    set_provider_value("crossref", "email", "person@example.com")

            self.assertEqual(observed["source"].parent, config_home)
            self.assertEqual(observed["destination"], config_path)
            self.assertFalse(observed["source"].exists())
            self.assertEqual(config_path.read_bytes(), original)


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
        self.assertEqual(provider_capability_mode(resolved), "provider_connected")

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
