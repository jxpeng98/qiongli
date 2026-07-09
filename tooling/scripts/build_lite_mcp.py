#!/usr/bin/env python3
from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import stat
import subprocess
import sys
import tomllib
from functools import lru_cache
from pathlib import Path


TARGETS = (
    "aarch64-apple-darwin",
    "x86_64-apple-darwin",
    "x86_64-unknown-linux-gnu",
    "aarch64-unknown-linux-gnu",
    "x86_64-pc-windows-msvc",
)
PACKAGE_RELATIVE = Path("packages") / "qiongli-lite-mcp"
BIN_BASENAME = "qiongli-literature-provider"
TARGET_IDENTITY_FILENAME = f"{BIN_BASENAME}.target.json"


def _manifest_path(root: Path) -> Path:
    return root / PACKAGE_RELATIVE / "Cargo.toml"


def _target_dir(root: Path) -> Path:
    return root / PACKAGE_RELATIVE / "target"


def _binary_name(*, windows: bool = False) -> str:
    return f"{BIN_BASENAME}.exe" if windows else BIN_BASENAME


def _run_cargo(root: Path, args: list[str]) -> None:
    subprocess.run(args, cwd=root, check=True)


@lru_cache(maxsize=1)
def current_host_target(root: Path) -> str:
    # Keep the caller's trusted working directory. Materialized release trees can
    # contain tool-manager configuration that is intentionally not trusted yet.
    _ = root
    result = subprocess.run(
        ["rustc", "-vV"],
        text=True,
        capture_output=True,
        check=True,
    )
    for line in result.stdout.splitlines():
        if line.startswith("host: "):
            target = line.removeprefix("host: ").strip()
            if target not in TARGETS:
                raise ValueError(f"unsupported current Lite MCP host target: {target}")
            return target
    raise ValueError("rustc -vV did not report a host target")


def target_platform(target: str) -> str:
    if target.endswith("apple-darwin"):
        return "darwin"
    if target.endswith("pc-windows-msvc"):
        return "win32"
    if target.endswith("unknown-linux-gnu"):
        return "linux"
    raise ValueError(f"unsupported Lite MCP target platform: {target}")


def target_architecture(target: str) -> str:
    architecture = target.split("-", 1)[0]
    if architecture not in {"aarch64", "x86_64"}:
        raise ValueError(f"unsupported Lite MCP target architecture: {target}")
    return architecture


def target_identity_path(binary: Path) -> Path:
    return binary.parent / TARGET_IDENTITY_FILENAME


def component_version(root: Path) -> str:
    manifest_path = _manifest_path(root.resolve())
    with manifest_path.open("rb") as handle:
        manifest = tomllib.load(handle)
    package = manifest.get("package")
    version = package.get("version") if isinstance(package, dict) else None
    if not isinstance(version, str) or not version:
        raise ValueError(f"{manifest_path} must define package.version")
    return version


def write_target_identity(binary: Path, target: str, component_version: str) -> Path:
    if not binary.is_file():
        raise FileNotFoundError(f"Lite MCP binary identity source does not exist: {binary}")
    if not component_version:
        raise ValueError("Lite MCP target identity component_version must be non-empty")
    digest = hashlib.sha256(binary.read_bytes()).hexdigest()
    identity = {
        "schema_version": "1.0",
        "component_version": component_version,
        "runtime_profile": "lite",
        "runtime_implementation": "rust",
        "target_policy": "current-host-only",
        "target_triple": target,
        "platform": target_platform(target),
        "architecture": target_architecture(target),
        "binary": binary.name,
        "sha256": digest,
        "size_bytes": binary.stat().st_size,
    }
    path = target_identity_path(binary)
    path.write_text(json.dumps(identity, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return path


def read_target_identity(binary_or_identity: Path) -> dict[str, object]:
    path = (
        binary_or_identity
        if binary_or_identity.name == TARGET_IDENTITY_FILENAME
        else target_identity_path(binary_or_identity)
    )
    payload = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(payload, dict):
        raise ValueError(f"Lite MCP target identity must be an object: {path}")
    return payload


def _stage_binary(
    source: Path,
    out_dir: Path,
    *,
    target: str,
    component_version: str,
    windows: bool = False,
) -> Path:
    if not source.is_file():
        raise FileNotFoundError(f"Lite MCP binary was not built: {source}")
    out_dir.mkdir(parents=True, exist_ok=True)
    staged = out_dir / _binary_name(windows=windows)
    shutil.copy2(source, staged)
    if not windows:
        staged.chmod(staged.stat().st_mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)
    write_target_identity(staged, target, component_version)
    return staged


def build_current_platform(root: Path, out_dir: Path) -> Path:
    root = root.resolve()
    manifest = _manifest_path(root)
    target = current_host_target(root)
    _run_cargo(
        root,
        [
            "cargo",
            "build",
            "--locked",
            "--release",
            "--manifest-path",
            str(manifest),
        ],
    )
    windows = os.name == "nt"
    built = _target_dir(root) / "release" / _binary_name(windows=windows)
    return _stage_binary(
        built,
        out_dir.resolve(),
        target=target,
        component_version=component_version(root),
        windows=windows,
    )


def build_target(root: Path, out_dir: Path, target: str) -> Path:
    if target not in TARGETS:
        raise ValueError(f"unsupported Lite MCP target: {target}")
    root = root.resolve()
    manifest = _manifest_path(root)
    _run_cargo(
        root,
        [
            "cargo",
            "build",
            "--locked",
            "--release",
            "--target",
            target,
            "--manifest-path",
            str(manifest),
        ],
    )
    windows = target.endswith("windows-msvc")
    built = _target_dir(root) / target / "release" / _binary_name(windows=windows)
    return _stage_binary(
        built,
        out_dir.resolve() / target,
        target=target,
        component_version=component_version(root),
        windows=windows,
    )


def build_all_platforms(root: Path, out_dir: Path) -> list[Path]:
    return [build_target(root, out_dir, target) for target in TARGETS]


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Build and stage Qiongli Lite MCP binaries.")
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[2])
    parser.add_argument("--out-dir", type=Path, default=Path("dist") / "lite-mcp")
    parser.add_argument("--target", choices=(*TARGETS, "current", "all"), default="current")
    args = parser.parse_args(argv)

    if args.target == "current":
        artifacts = [build_current_platform(args.root, args.out_dir)]
    elif args.target == "all":
        artifacts = build_all_platforms(args.root, args.out_dir)
    else:
        artifacts = [build_target(args.root, args.out_dir, args.target)]

    for artifact in artifacts:
        print(artifact)
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
