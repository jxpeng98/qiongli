from __future__ import annotations

import argparse
import json
import shutil
import sys
import zipfile
from pathlib import Path


PACKAGE_RELATIVE = Path("packages/qiongli-literature-mcpb")
REQUIRED_FILES = (
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
    return parser.parse_args()


def package_root() -> Path:
    return Path(__file__).resolve().parents[1] / PACKAGE_RELATIVE


def read_manifest(root: Path) -> dict[str, object]:
    manifest_path = root / "manifest.json"
    if not manifest_path.is_file():
        raise ValueError(f"Missing required file: {manifest_path}")
    with manifest_path.open(encoding="utf-8") as handle:
        return json.load(handle)


def validate_required_files(root: Path, manifest: dict[str, object]) -> None:
    missing = [str(path) for path in REQUIRED_FILES if not (root / path).is_file()]
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

    server_dir = root / "server"
    if not server_dir.is_dir():
        missing.append("server/")

    if missing:
        raise ValueError("Missing required files: " + ", ".join(missing))


def iter_package_files(root: Path) -> list[Path]:
    files = [root / path for path in REQUIRED_FILES]

    for directory_name in ("server", "node_modules"):
        directory = root / directory_name
        if directory.exists():
            files.extend(path for path in directory.rglob("*") if path.is_file())

    return sorted(set(files), key=lambda path: path.relative_to(root).as_posix())


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


def build(root: Path, dist_dir: Path) -> Path:
    manifest = read_manifest(root)
    validate_required_files(root, manifest)

    files = iter_package_files(root)
    validate_no_secret_fixtures(root, files)

    dist_dir.mkdir(parents=True, exist_ok=True)
    path = artifact_path(dist_dir, manifest)
    temporary = path.with_suffix(path.suffix + ".tmp")
    if temporary.exists():
        temporary.unlink()

    with zipfile.ZipFile(temporary, "w", compression=zipfile.ZIP_DEFLATED) as archive:
        for source in files:
            archive.write(source, source.relative_to(root).as_posix())

    shutil.move(str(temporary), path)
    return path


def main() -> int:
    args = parse_args()
    try:
        path = build(package_root(), args.dist_dir)
    except (OSError, ValueError, json.JSONDecodeError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 1

    print(path)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
