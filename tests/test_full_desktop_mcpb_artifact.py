from __future__ import annotations

import hashlib
import json
import os
import subprocess
import sys
import tempfile
import tomllib
import unittest
import zipfile
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
PACKAGE_ROOT = REPO_ROOT / "packages" / "qiongli-full-mcpb"
HOST_TOOL_NAMES = [
    "qiongli_orchestration_doctor",
    "qiongli_orchestration_start",
    "qiongli_orchestration_next",
    "qiongli_orchestration_read",
    "qiongli_orchestration_submit",
    "qiongli_orchestration_runs",
    "qiongli_orchestration_action",
    "qiongli_worker_orchestration_runs",
    "qiongli_worker_orchestration_action",
]
DIRECT_MODEL_TOOL_NAMES = {
    "qiongli_agent_backend_status",
    "qiongli_agent_backend_test",
    "qiongli_agent_run",
}


def native_version() -> str:
    with (REPO_ROOT / "packages/qiongli-native/Cargo.toml").open("rb") as handle:
        manifest = tomllib.load(handle)
    return manifest["workspace"]["package"]["version"]


def contract_names(filename: str) -> list[str]:
    contract = json.loads(
        (REPO_ROOT / "content/mcp-contracts" / filename).read_text(encoding="utf-8")
    )
    return [tool["name"] for tool in contract["tools"]]


class FullDesktopMcpbArtifactTests(unittest.TestCase):
    def test_source_manifest_declares_only_the_full_host_boundary(self) -> None:
        manifest = json.loads((PACKAGE_ROOT / "manifest.json").read_text(encoding="utf-8"))

        self.assertEqual(manifest["manifest_version"], "0.3")
        self.assertEqual(manifest["name"], "qiongli-full-runtime")
        self.assertEqual(manifest["version"], native_version())
        self.assertEqual(manifest["server"]["type"], "stdio")
        self.assertEqual(
            manifest["server"]["mcp_config"]["args"],
            ["mcp", "serve", "--profile", "full", "--transport", "stdio"],
        )
        self.assertEqual(manifest["server"]["mcp_config"]["env"], {})
        self.assertEqual(manifest["tools"], [])
        serialized = json.dumps(manifest)
        self.assertNotIn("api_key", serialized)
        self.assertNotIn("provider_endpoint", serialized)
        self.assertNotIn("model_name", serialized)

    def test_readme_separates_local_full_lite_and_remote_surfaces(self) -> None:
        readme = (PACKAGE_ROOT / "README.md").read_text(encoding="utf-8")
        normalized = " ".join(readme.split())

        self.assertIn("Claude Desktop owns the model login", normalized)
        self.assertIn("qiongli-literature-provider-*.mcpb", normalized)
        self.assertIn("Marketplace Lite", normalized)
        self.assertIn("Codex Cloud", normalized)
        self.assertIn("remote worker", normalized)
        self.assertIn("publication_allowed: false", normalized)

    def test_builder_packages_and_probes_the_exact_current_host_full_runtime(self) -> None:
        expected_names = (
            contract_names("lite-tools.json")
            + contract_names("full-project-tools.json")
            + HOST_TOOL_NAMES
        )
        self.assertEqual(len(expected_names), 30)

        with tempfile.TemporaryDirectory() as temporary:
            temporary_root = Path(temporary)
            dist = temporary_root / "dist"
            result = subprocess.run(
                [
                    sys.executable,
                    "scripts/build_full_desktop_mcpb.py",
                    "--dist-dir",
                    str(dist),
                ],
                cwd=REPO_ROOT,
                text=True,
                capture_output=True,
                check=False,
            )
            self.assertEqual(result.returncode, 0, msg=result.stderr)

            artifact = dist / f"qiongli-full-runtime-{native_version()}.mcpb"
            receipt_path = dist / f"qiongli-full-runtime-{native_version()}.receipt.json"
            self.assertTrue(artifact.is_file())
            self.assertTrue(receipt_path.is_file())
            self.assertIn(str(artifact), result.stdout)
            self.assertIn(str(receipt_path), result.stdout)

            extract_root = temporary_root / "extracted"
            with zipfile.ZipFile(artifact) as archive:
                names = set(archive.namelist())
                manifest = json.loads(archive.read("manifest.json").decode("utf-8"))
                identity = json.loads(
                    archive.read("bin/qiongli-full.target.json").decode("utf-8")
                )
                binary_member = f"bin/{identity['binary']}"
                binary_bytes = archive.read(binary_member)
                archive.extractall(extract_root)

            self.assertEqual(
                names,
                {
                    "LICENSE",
                    "README.md",
                    "manifest.json",
                    "bin/qiongli-full.target.json",
                    binary_member,
                },
            )
            self.assertEqual(
                [tool["name"] for tool in manifest["tools"]],
                expected_names,
            )
            self.assertFalse(
                DIRECT_MODEL_TOOL_NAMES.intersection(
                    tool["name"] for tool in manifest["tools"]
                )
            )
            self.assertEqual(identity["runtime_profile"], "full")
            self.assertEqual(identity["runtime_implementation"], "rust")
            self.assertEqual(identity["target_policy"], "current-host-only")
            self.assertIs(identity["publication_allowed"], False)
            self.assertEqual(identity["component_version"], native_version())
            self.assertEqual(identity["sha256"], hashlib.sha256(binary_bytes).hexdigest())
            self.assertEqual(identity["size_bytes"], len(binary_bytes))
            self.assertEqual(
                manifest["compatibility"]["platforms"],
                [identity["platform"]],
            )
            self.assertEqual(
                manifest["compatibility"]["architectures"],
                [identity["architecture"]],
            )
            self.assertEqual(
                manifest["compatibility"]["target_triple"],
                identity["target_triple"],
            )
            self.assertEqual(
                manifest["compatibility"]["runtimes"],
                {"native": "bundled-rust-full-mcp"},
            )

            receipt = json.loads(receipt_path.read_text(encoding="utf-8"))
            self.assertEqual(receipt["status"], "built-current-host-nonpublishing")
            self.assertIs(receipt["publication_allowed"], False)
            self.assertEqual(receipt["binary_sha256"], identity["sha256"])
            self.assertEqual(
                receipt["artifact_sha256"],
                hashlib.sha256(artifact.read_bytes()).hexdigest(),
            )
            self.assertEqual(receipt["artifact_size_bytes"], artifact.stat().st_size)

            binary = extract_root / binary_member
            if os.name != "nt":
                binary.chmod(0o755)
            probe_home = temporary_root / "probe-home"
            probe_home.mkdir()
            requests = [
                {"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}},
                {"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}},
            ]
            probe = subprocess.run(
                [
                    str(binary),
                    "mcp",
                    "serve",
                    "--profile",
                    "full",
                    "--transport",
                    "stdio",
                ],
                cwd=probe_home,
                env={
                    "HOME": str(probe_home),
                    "PATH": "",
                    "QIONGLI_CONFIG_HOME": str(probe_home / "config"),
                },
                input="".join(json.dumps(request) + "\n" for request in requests),
                text=True,
                capture_output=True,
                check=False,
            )
            self.assertEqual(probe.returncode, 0, msg=probe.stderr)
            self.assertEqual(probe.stderr, "")
            responses = [json.loads(line) for line in probe.stdout.splitlines() if line]
            self.assertEqual(
                [tool["name"] for tool in responses[1]["result"]["tools"]],
                expected_names,
            )


if __name__ == "__main__":
    unittest.main()
