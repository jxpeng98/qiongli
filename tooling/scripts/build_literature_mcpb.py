from __future__ import annotations

import argparse
import json
import shutil
import sys
import tempfile
import zipfile
from pathlib import Path

from build_lite_mcp import build_current_platform


PACKAGE_RELATIVE = Path("packages/qiongli-literature-mcpb")
REQUIRED_FILES = (
    Path("manifest.json"),
    Path("README.md"),
)
LEGACY_NODE_REQUIRED_FILES = (
    Path("manifest.json"),
    Path("package.json"),
    Path("README.md"),
)
SECRET_FIXTURES = ("secret-key", "desktop-secret", "api-key-value")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Build the Qiongli Literature MCPB.")
    parser.add_argument(
        "--dist-dir",
        type=Path,
        default=Path("dist"),
        help="Directory where the .mcpb artifact should be written.",
    )
    parser.add_argument(
        "--legacy-node",
        action="store_true",
        help="Package the legacy Node MCPB runtime instead of the bundled Rust Lite binary.",
    )
    return parser.parse_args()


def repo_root() -> Path:
    return Path(__file__).resolve().parents[2]


def package_root() -> Path:
    return repo_root() / PACKAGE_RELATIVE


def read_manifest(root: Path) -> dict[str, object]:
    manifest_path = root / "manifest.json"
    if not manifest_path.is_file():
        raise ValueError(f"Missing required file: {manifest_path}")
    with manifest_path.open(encoding="utf-8") as handle:
        return json.load(handle)


def validate_required_files(root: Path, manifest: dict[str, object], *, legacy_node: bool = False) -> None:
    required_files = LEGACY_NODE_REQUIRED_FILES if legacy_node else REQUIRED_FILES
    missing = [str(path) for path in required_files if not (root / path).is_file()]
    server = manifest.get("server")
    if not isinstance(server, dict):
        raise ValueError("manifest.json must define a server object")

    entry_point = server.get("entry_point")
    if not isinstance(entry_point, str) or not entry_point:
        raise ValueError("manifest.json must define server.entry_point")

    entry_path = Path(entry_point)
    if entry_path.is_absolute() or ".." in entry_path.parts:
        raise ValueError("server.entry_point must be a package-relative path")
    if not (root / entry_path).is_file():
        missing.append(entry_point)

    if legacy_node:
        server_dir = root / "server"
        if not server_dir.is_dir():
            missing.append("server/")

    if missing:
        raise ValueError("Missing required files: " + ", ".join(missing))


def iter_package_files(root: Path) -> list[Path]:
    return sorted(
        (path for path in root.rglob("*") if path.is_file()),
        key=lambda path: path.relative_to(root).as_posix(),
    )


def validate_no_secret_fixtures(root: Path, files: list[Path]) -> None:
    for path in files:
        relative = path.relative_to(root).as_posix()
        if relative == "manifest.json":
            continue

        try:
            content = path.read_text(encoding="utf-8")
        except UnicodeDecodeError:
            continue

        for fixture in SECRET_FIXTURES:
            if fixture in content:
                raise ValueError(f"Refusing to package fixture secret {fixture!r} in {relative}")


def artifact_path(dist_dir: Path, manifest: dict[str, object]) -> Path:
    name = manifest.get("name")
    version = manifest.get("version")
    if not isinstance(name, str) or not name:
        raise ValueError("manifest.json must define a string name")
    if not isinstance(version, str) or not version:
        raise ValueError("manifest.json must define a string version")
    return dist_dir / f"{name}-{version}.mcpb"


def _copy_package_file(root: Path, staging: Path, relative: Path) -> None:
    source = root / relative
    dest = staging / relative
    dest.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(source, dest)


def _write_manifest(staging: Path, manifest: dict[str, object]) -> None:
    (staging / "manifest.json").write_text(
        json.dumps(manifest, indent=2, ensure_ascii=False) + "\n",
        encoding="utf-8",
    )


def _legacy_node_manifest(manifest: dict[str, object]) -> dict[str, object]:
    legacy = json.loads(json.dumps(manifest))
    server = legacy["server"]
    server["type"] = "node"
    server["entry_point"] = "server/index.mjs"
    server["mcp_config"]["command"] = "node"
    server["mcp_config"]["args"] = ["${__dirname}/server/index.mjs"]
    compatibility = legacy.setdefault("compatibility", {})
    compatibility["platforms"] = ["darwin", "win32"]
    compatibility["runtimes"] = {"node": ">=18.0.0"}
    return legacy


def _stage_binary_package(root: Path, staging: Path, manifest: dict[str, object]) -> None:
    _write_manifest(staging, manifest)
    _copy_package_file(root, staging, Path("README.md"))
    build_current_platform(repo_root(), staging / "bin")


def _stage_legacy_node_package(root: Path, staging: Path, manifest: dict[str, object]) -> None:
    _write_manifest(staging, _legacy_node_manifest(manifest))
    for relative in (Path("README.md"), Path("package.json")):
        _copy_package_file(root, staging, relative)
    server_root = root / "server"
    if not server_root.is_dir():
        raise ValueError(f"Missing required directory: {server_root}")
    shutil.copytree(server_root, staging / "server")


def build(root: Path, dist_dir: Path, *, legacy_node: bool = False) -> Path:
    source_manifest = read_manifest(root)

    with tempfile.TemporaryDirectory(prefix="qiongli-literature-mcpb-") as tmp:
        staging = Path(tmp) / root.name
        staging.mkdir(parents=True)
        if legacy_node:
            _stage_legacy_node_package(root, staging, source_manifest)
        else:
            _stage_binary_package(root, staging, source_manifest)

        manifest = read_manifest(staging)
        validate_required_files(staging, manifest, legacy_node=legacy_node)

        files = iter_package_files(staging)
        validate_no_secret_fixtures(staging, files)

        dist_dir.mkdir(parents=True, exist_ok=True)
        path = artifact_path(dist_dir, manifest)
        temporary = path.with_suffix(path.suffix + ".tmp")
        if temporary.exists():
            temporary.unlink()

        with zipfile.ZipFile(temporary, "w", compression=zipfile.ZIP_DEFLATED) as archive:
            for source in files:
                archive.write(source, source.relative_to(staging).as_posix())

    shutil.move(str(temporary), path)
    return path


def main() -> int:
    args = parse_args()
    try:
        path = build(package_root(), args.dist_dir, legacy_node=args.legacy_node)
    except (OSError, ValueError, json.JSONDecodeError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 1

    print(path)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
