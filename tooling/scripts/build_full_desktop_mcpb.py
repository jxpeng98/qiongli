from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import stat
import subprocess
import sys
import tempfile
import tomllib
import zipfile
from pathlib import Path

from build_lite_mcp import current_host_target, target_architecture, target_platform


PACKAGE_RELATIVE = Path("packages/qiongli-full-mcpb")
NATIVE_MANIFEST_RELATIVE = Path("packages/qiongli-native/Cargo.toml")
NATIVE_TARGET_RELATIVE = Path("packages/qiongli-native/target/release")
BINARY_BASENAME = "qiongli"
IDENTITY_FILENAME = "qiongli-full.target.json"
RECEIPT_SUFFIX = ".receipt.json"
MAX_MCP_OUTPUT_BYTES = 4 * 1024 * 1024
EXPECTED_TOOL_COUNT = 30
FORBIDDEN_TOOL_NAMES = {
    "qiongli_agent_backend_status",
    "qiongli_agent_backend_test",
    "qiongli_agent_run",
}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Build a current-host, non-publishing Qiongli Full Desktop MCPB."
    )
    parser.add_argument(
        "--dist-dir",
        type=Path,
        default=Path("dist"),
        help="Directory where the MCPB and local build receipt are written.",
    )
    return parser.parse_args()


def repo_root() -> Path:
    return Path(__file__).resolve().parents[2]


def package_root(root: Path) -> Path:
    return root / PACKAGE_RELATIVE


def native_version(root: Path) -> str:
    with (root / NATIVE_MANIFEST_RELATIVE).open("rb") as handle:
        manifest = tomllib.load(handle)
    workspace = manifest.get("workspace")
    package = workspace.get("package") if isinstance(workspace, dict) else None
    version = package.get("version") if isinstance(package, dict) else None
    if not isinstance(version, str) or not version:
        raise ValueError("native workspace must define workspace.package.version")
    return version


def read_manifest(root: Path) -> dict[str, object]:
    path = root / "manifest.json"
    payload = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(payload, dict):
        raise ValueError("Full Desktop manifest must be an object")
    return payload


def clean_source_commit(root: Path) -> str | None:
    status = subprocess.run(
        ["git", "status", "--short"],
        cwd=root,
        text=True,
        capture_output=True,
        check=True,
    )
    if status.stdout.strip():
        return None
    result = subprocess.run(
        ["git", "rev-parse", "--verify", "HEAD"],
        cwd=root,
        text=True,
        capture_output=True,
        check=True,
    )
    commit = result.stdout.strip()
    if len(commit) != 40 or any(character not in "0123456789abcdef" for character in commit):
        raise ValueError("clean source commit must be 40 lowercase hexadecimal characters")
    return commit


def build_native_binary(root: Path, staging_bin: Path) -> tuple[Path, str]:
    target = current_host_target(root)
    command = [
        "cargo",
        "build",
        "--locked",
        "--release",
        "--manifest-path",
        str(root / NATIVE_MANIFEST_RELATIVE),
        "--package",
        "qiongli",
        "--bin",
        "qiongli",
    ]
    environment = os.environ.copy()
    source_commit = clean_source_commit(root)
    if source_commit is None:
        environment.pop("QIONGLI_NATIVE_SOURCE_COMMIT", None)
    else:
        environment["QIONGLI_NATIVE_SOURCE_COMMIT"] = source_commit
    subprocess.run(command, cwd=root, env=environment, check=True)

    windows = os.name == "nt"
    binary_name = f"{BINARY_BASENAME}.exe" if windows else BINARY_BASENAME
    source = root / NATIVE_TARGET_RELATIVE / binary_name
    if not source.is_file() or source.is_symlink():
        raise ValueError(f"native Full MCP binary was not produced: {source}")
    staging_bin.mkdir(parents=True, exist_ok=True)
    binary = staging_bin / binary_name
    shutil.copy2(source, binary)
    if not windows:
        binary.chmod(binary.stat().st_mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)
    return binary, target


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def write_identity(
    binary: Path,
    target: str,
    version: str,
    source_commit: str | None,
) -> dict[str, object]:
    binary_bytes = binary.read_bytes()
    identity: dict[str, object] = {
        "schema_version": "1.0",
        "component_version": version,
        "runtime_profile": "full",
        "runtime_implementation": "rust",
        "target_policy": "current-host-only",
        "target_triple": target,
        "platform": target_platform(target),
        "architecture": target_architecture(target),
        "binary": binary.name,
        "sha256": sha256_bytes(binary_bytes),
        "size_bytes": len(binary_bytes),
        "source_commit": source_commit,
        "publication_allowed": False,
    }
    (binary.parent / IDENTITY_FILENAME).write_text(
        json.dumps(identity, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    return identity


def probe_tools(binary: Path) -> list[dict[str, str]]:
    requests = (
        {"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}},
        {"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}},
    )
    input_text = "".join(json.dumps(request, separators=(",", ":")) + "\n" for request in requests)
    with tempfile.TemporaryDirectory(prefix="qiongli-full-mcp-probe-") as temporary:
        private_root = Path(temporary)
        environment = {
            "HOME": str(private_root),
            "PATH": "",
            "QIONGLI_CONFIG_HOME": str(private_root / "config"),
        }
        if os.name == "nt":
            environment["USERPROFILE"] = str(private_root)
        result = subprocess.run(
            [
                str(binary),
                "mcp",
                "serve",
                "--profile",
                "full",
                "--transport",
                "stdio",
            ],
            cwd=private_root,
            env=environment,
            input=input_text,
            text=True,
            capture_output=True,
            check=False,
        )
    if result.returncode != 0 or result.stderr:
        raise ValueError("native Full MCP probe failed")
    if len(result.stdout.encode("utf-8")) > MAX_MCP_OUTPUT_BYTES:
        raise ValueError("native Full MCP probe exceeded the output limit")
    responses = [json.loads(line) for line in result.stdout.splitlines() if line]
    if len(responses) != 2:
        raise ValueError("native Full MCP probe returned an unexpected response count")
    tools = responses[1].get("result", {}).get("tools")
    if not isinstance(tools, list):
        raise ValueError("native Full MCP probe did not return tools")
    projected: list[dict[str, str]] = []
    for tool in tools:
        if not isinstance(tool, dict):
            raise ValueError("native Full MCP tool must be an object")
        name = tool.get("name")
        description = tool.get("description")
        if not isinstance(name, str) or not name:
            raise ValueError("native Full MCP tool name must be non-empty")
        if not isinstance(description, str) or not description:
            raise ValueError(f"native Full MCP tool description is missing: {name}")
        projected.append({"name": name, "description": description})
    names = [tool["name"] for tool in projected]
    if len(names) != EXPECTED_TOOL_COUNT or len(set(names)) != len(names):
        raise ValueError("native Full MCP tool inventory count or uniqueness drifted")
    if FORBIDDEN_TOOL_NAMES.intersection(names):
        raise ValueError("native Full MCP unexpectedly advertised a direct-model tool")
    return projected


def render_manifest(
    source: dict[str, object],
    identity: dict[str, object],
    tools: list[dict[str, str]],
) -> dict[str, object]:
    manifest = json.loads(json.dumps(source))
    version = identity["component_version"]
    if manifest.get("version") != version:
        raise ValueError("Full Desktop manifest version must match the native workspace")
    binary = identity["binary"]
    server = manifest.get("server")
    if not isinstance(server, dict):
        raise ValueError("Full Desktop manifest must define server")
    server["entry_point"] = f"bin/{binary}"
    mcp_config = server.get("mcp_config")
    if not isinstance(mcp_config, dict):
        raise ValueError("Full Desktop manifest must define server.mcp_config")
    mcp_config["command"] = f"${{__dirname}}/bin/{binary}"
    mcp_config["args"] = [
        "mcp",
        "serve",
        "--profile",
        "full",
        "--transport",
        "stdio",
    ]
    mcp_config["env"] = {}
    manifest["tools"] = tools
    compatibility = manifest.get("compatibility")
    if not isinstance(compatibility, dict):
        raise ValueError("Full Desktop manifest must define compatibility")
    compatibility["platforms"] = [identity["platform"]]
    compatibility["architectures"] = [identity["architecture"]]
    compatibility["target_triple"] = identity["target_triple"]
    compatibility["runtimes"] = {"native": "bundled-rust-full-mcp"}
    return manifest


def iter_files(root: Path) -> list[Path]:
    return sorted(
        (path for path in root.rglob("*") if path.is_file()),
        key=lambda path: path.relative_to(root).as_posix(),
    )


def write_archive(staging: Path, destination: Path) -> None:
    temporary = destination.with_suffix(destination.suffix + ".tmp")
    if temporary.exists():
        temporary.unlink()
    with zipfile.ZipFile(temporary, "w", compression=zipfile.ZIP_DEFLATED) as archive:
        for source in iter_files(staging):
            relative = source.relative_to(staging).as_posix()
            info = zipfile.ZipInfo(relative, date_time=(1980, 1, 1, 0, 0, 0))
            mode = 0o755 if relative.startswith("bin/") and source.name != IDENTITY_FILENAME else 0o644
            info.external_attr = (stat.S_IFREG | mode) << 16
            info.compress_type = zipfile.ZIP_DEFLATED
            archive.writestr(info, source.read_bytes())
    shutil.move(str(temporary), destination)


def write_receipt(
    artifact: Path,
    identity: dict[str, object],
    package_name: str,
    version: str,
) -> Path:
    artifact_bytes = artifact.read_bytes()
    receipt = {
        "schema_version": 1,
        "record_type": "qiongli-full-desktop-mcpb-build",
        "status": "built-current-host-nonpublishing",
        "package_name": package_name,
        "version": version,
        "target_triple": identity["target_triple"],
        "platform": identity["platform"],
        "architecture": identity["architecture"],
        "source_commit": identity["source_commit"],
        "binary_sha256": identity["sha256"],
        "artifact_sha256": sha256_bytes(artifact_bytes),
        "artifact_size_bytes": len(artifact_bytes),
        "publication_allowed": False,
    }
    path = artifact.with_suffix(RECEIPT_SUFFIX)
    temporary = path.with_suffix(path.suffix + ".tmp")
    temporary.write_text(
        json.dumps(receipt, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    shutil.move(str(temporary), path)
    return path


def build(root: Path, dist_dir: Path) -> tuple[Path, Path]:
    root = root.resolve()
    source_root = package_root(root)
    source_manifest = read_manifest(source_root)
    version = native_version(root)
    package_name = source_manifest.get("name")
    if not isinstance(package_name, str) or not package_name:
        raise ValueError("Full Desktop manifest name must be non-empty")

    with tempfile.TemporaryDirectory(prefix="qiongli-full-desktop-mcpb-") as temporary:
        staging = Path(temporary) / package_name
        staging.mkdir(parents=True)
        shutil.copy2(source_root / "README.md", staging / "README.md")
        shutil.copy2(root / "LICENSE", staging / "LICENSE")
        binary, target = build_native_binary(root, staging / "bin")
        source_commit = clean_source_commit(root)
        identity = write_identity(binary, target, version, source_commit)
        manifest = render_manifest(source_manifest, identity, probe_tools(binary))
        (staging / "manifest.json").write_text(
            json.dumps(manifest, indent=2, ensure_ascii=False) + "\n",
            encoding="utf-8",
        )

        dist_dir = dist_dir.resolve()
        dist_dir.mkdir(parents=True, exist_ok=True)
        artifact = dist_dir / f"{package_name}-{version}.mcpb"
        write_archive(staging, artifact)

    receipt = write_receipt(artifact, identity, package_name, version)
    return artifact, receipt


def main() -> int:
    arguments = parse_args()
    try:
        artifact, receipt = build(repo_root(), arguments.dist_dir)
    except (
        OSError,
        ValueError,
        json.JSONDecodeError,
        subprocess.CalledProcessError,
    ) as error:
        print(f"error: {error}", file=sys.stderr)
        return 1
    print(artifact)
    print(receipt)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
