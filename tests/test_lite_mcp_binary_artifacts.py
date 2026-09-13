from __future__ import annotations

import hashlib
import json
import os
import tarfile
import tempfile
import tomllib
import unittest
import zipfile
from pathlib import Path
from unittest import mock

from qiongli.source_layout import RepoLayout
from scripts.build_plugin_artifacts import build_artifacts
from tooling.scripts.release_version import parse_release_version


REPO_ROOT = Path(__file__).resolve().parents[1]
with (REPO_ROOT / "packages" / "qiongli-lite-mcp" / "Cargo.toml").open("rb") as handle:
    EXPECTED_RUST_COMPONENT_VERSION = tomllib.load(handle)["package"]["version"]


class LiteMCPBinaryArtifactTests(unittest.TestCase):
    def test_build_lite_mcp_stages_current_platform_binary(self) -> None:
        from tooling.scripts.build_lite_mcp import (
            TARGET_IDENTITY_FILENAME,
            build_current_platform,
            current_host_target,
        )

        with tempfile.TemporaryDirectory() as tmp_dir:
            binary = build_current_platform(REPO_ROOT, Path(tmp_dir))
            identity_path = binary.parent / TARGET_IDENTITY_FILENAME
            identity = json.loads(identity_path.read_text(encoding="utf-8"))
            self.assertTrue(binary.is_file())
            self.assertEqual(binary.name, "qiongli-literature-provider")
            self.assertTrue(os.access(binary, os.X_OK))
            self.assertEqual(identity["runtime_profile"], "lite")
            self.assertEqual(identity["runtime_implementation"], "rust")
            self.assertEqual(identity["target_policy"], "current-host-only")
            self.assertEqual(identity["component_version"], EXPECTED_RUST_COMPONENT_VERSION)
            self.assertEqual(identity["target_triple"], current_host_target(REPO_ROOT))
            self.assertEqual(identity["binary"], binary.name)
            self.assertEqual(identity["size_bytes"], binary.stat().st_size)
            self.assertEqual(identity["sha256"], hashlib.sha256(binary.read_bytes()).hexdigest())

    def test_target_build_is_locked_and_writes_target_identity(self) -> None:
        from tooling.scripts import build_lite_mcp

        target = "x86_64-pc-windows-msvc"
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir) / "root"
            out = Path(tmp_dir) / "out"
            manifest = root / "packages" / "qiongli-lite-mcp" / "Cargo.toml"
            manifest.parent.mkdir(parents=True)
            manifest.write_text("[package]\nname='fixture'\nversion='0.0.0'\n", encoding="utf-8")
            built = manifest.parent / "target" / target / "release" / "qiongli-literature-provider.exe"
            built.parent.mkdir(parents=True)
            built.write_bytes(b"fixture-windows-binary")

            with mock.patch.object(build_lite_mcp, "_run_cargo") as run_cargo:
                binary = build_lite_mcp.build_target(root, out, target)

            cargo_args = run_cargo.call_args.args[1]
            self.assertIn("--locked", cargo_args)
            self.assertEqual(binary.name, "qiongli-literature-provider.exe")
            identity = json.loads(
                (binary.parent / build_lite_mcp.TARGET_IDENTITY_FILENAME).read_text(encoding="utf-8")
            )

        self.assertEqual(identity["target_triple"], target)
        self.assertEqual(identity["target_policy"], "current-host-only")
        self.assertEqual(identity["component_version"], "0.0.0")
        self.assertEqual(identity["platform"], "win32")
        self.assertEqual(identity["architecture"], "x86_64")

    def test_codex_plugin_contains_lite_mcp_binary(self) -> None:
        tag = (RepoLayout(REPO_ROOT).workflow / "VERSION").read_text(encoding="utf-8").strip()
        plugin_name = "qiongli-next" if parse_release_version(tag).is_prerelease else "qiongli"
        with tempfile.TemporaryDirectory() as tmp_dir:
            artifacts = build_artifacts(REPO_ROOT, tag, Path(tmp_dir))
            codex = next(
                path for path in artifacts if path.name.startswith(f"{plugin_name}-codex-plugin-")
            )
            with tarfile.open(codex, "r:gz") as archive:
                names = set(archive.getnames())
                member = next(
                    name for name in names if name.endswith(f"/plugins/{plugin_name}/.mcp.json")
                )
                extracted = archive.extractfile(member)
                self.assertIsNotNone(extracted, msg=f"missing tar member: {member}")
                assert extracted is not None
                manifest = json.loads(extracted.read().decode("utf-8"))
                identity_member = next(
                    name
                    for name in names
                    if name.endswith(
                        f"/plugins/{plugin_name}/bin/qiongli-literature-provider.target.json"
                    )
                )
                identity_file = archive.extractfile(identity_member)
                self.assertIsNotNone(identity_file, msg=f"missing tar member: {identity_member}")
                assert identity_file is not None
                identity = json.loads(identity_file.read().decode("utf-8"))

        self.assertTrue(
            any(
                name.endswith(f"/plugins/{plugin_name}/bin/qiongli-literature-provider")
                for name in names
            )
        )
        server = manifest["mcpServers"][plugin_name]
        self.assertEqual(server["command"], "./bin/qiongli-literature-provider")
        self.assertEqual(server["args"], ["--transport", "stdio"])
        self.assertEqual(identity["runtime_profile"], "lite")
        self.assertEqual(identity["runtime_implementation"], "rust")
        self.assertEqual(identity["component_version"], EXPECTED_RUST_COMPONENT_VERSION)
        self.assertEqual(identity["binary"], "qiongli-literature-provider")

    def test_direct_desktop_plugin_contains_lite_mcp_binary(self) -> None:
        tag = (RepoLayout(REPO_ROOT).workflow / "VERSION").read_text(encoding="utf-8").strip()
        plugin_name = "qiongli-next" if parse_release_version(tag).is_prerelease else "qiongli"
        with tempfile.TemporaryDirectory() as tmp_dir:
            artifacts = build_artifacts(REPO_ROOT, tag, Path(tmp_dir))
            desktop = next(
                path
                for path in artifacts
                if path.name.startswith(f"{plugin_name}-claude-desktop-plugin-")
            )
            with zipfile.ZipFile(desktop) as archive:
                names = set(archive.namelist())
                manifest_member = next(
                    name for name in names if name.endswith("/.claude-plugin/plugin.json")
                )
                manifest = json.loads(
                    archive.read(manifest_member).decode("utf-8")
                )
                identity_member = next(
                    name
                    for name in names
                    if name.endswith("/bin/qiongli-literature-provider.target.json")
                )
                identity = json.loads(archive.read(identity_member).decode("utf-8"))

        self.assertTrue(
            any(name.endswith("/bin/qiongli-literature-provider") for name in names)
        )
        server = manifest["mcpServers"][plugin_name]
        self.assertEqual(
            server["command"],
            "${CLAUDE_PLUGIN_ROOT}/bin/qiongli-literature-provider",
        )
        self.assertEqual(server["args"], ["--transport", "stdio"])
        self.assertEqual(identity["runtime_profile"], "lite")
        self.assertEqual(identity["runtime_implementation"], "rust")
        self.assertEqual(identity["component_version"], EXPECTED_RUST_COMPONENT_VERSION)


if __name__ == "__main__":
    unittest.main()
