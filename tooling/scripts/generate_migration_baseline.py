#!/usr/bin/env python3
from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import shutil
import stat
import subprocess
import sys
import tarfile
import tempfile
import zipfile
from dataclasses import dataclass
from functools import lru_cache
from pathlib import Path, PurePosixPath
from typing import Any, Iterable, Mapping, Sequence

from tooling.scripts.validate_capability_contract import validate_instance


REPO_ROOT = Path(__file__).resolve().parents[2]
PLAN_RELATIVE = Path("tooling/migration/qiongli-1x-baseline-plan.json")
PLAN_SCHEMA_RELATIVE = Path("tooling/migration/baseline-plan.schema.json")
CANONICAL_OUTPUT_RELATIVE = Path("tooling/migration/baselines/v1.19.0-beta.1")
DESKTOP_RUNTIME_CONTAINER = (
    "qiongli-next-claude-desktop-plugin-v1.19.0-beta.1.zip"
)
RUNTIME_SECRET_CANARY = "QIONGLI_CANARY_DO_NOT_ECHO_runtime_oracle"
HASH_PATTERN = re.compile(r"^[0-9a-f]{40,64}$")
MACHINE_PATH_PATTERN = re.compile(
    r"(?:file://|(?<![A-Za-z0-9/])/(?:Users|home|private/tmp|private/var/folders|"
    r"tmp|var/folders|Volumes|root)/|(?<![A-Za-z0-9+.-])[A-Za-z]:[\\/]|"
    r"\\\\[^\\/\s]+[\\/][^\\/\s]+)"
)
TIMESTAMP_PATTERN = re.compile(
    r"\b20\d{2}-[01]\d-[0-3]\d[Tt][0-2]\d:[0-5]\d:[0-5]\d"
    r"(?:\.\d+)?(?:[Zz]|[+-][0-2]\d:[0-5]\d)\b"
)
PROCESS_ID_PATTERN = re.compile(
    r"\b(?:pid|process[-_ ]?id)\s*[:=]\s*[1-9]\d*\b", re.IGNORECASE
)
SECRET_PATTERNS = (
    re.compile(r"QIONGLI_CANARY_DO_NOT_ECHO"),
    re.compile(r"\b(?:sk[-_]|ghp_|github_pat_)[A-Za-z0-9_-]{12,}\b"),
    re.compile(r"-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----"),
)
CORPUS_DEFINITION = (
    "canonical-json(source,acceptance_receipt,selector_semantics,domains,package_trees,"
    "release_assets,native_identities,oracle_fixtures)"
)


class BaselineError(ValueError):
    """Raised when deterministic baseline capture or verification fails."""


@dataclass(frozen=True)
class GitEntry:
    path: str
    mode: str
    oid: str
    data: bytes

    def manifest_record(self) -> dict[str, Any]:
        return {
            "git_blob_oid": self.oid,
            "mode": self.mode,
            "path": self.path,
            "sha256": _sha256(self.data),
            "size_bytes": len(self.data),
        }

    def projection_record(self) -> dict[str, Any]:
        return {
            "git_blob_oid": self.oid,
            "path": self.path,
            "sha256": _sha256(self.data),
            "size_bytes": len(self.data),
        }


def _canonical_bytes(value: Any) -> bytes:
    return (
        json.dumps(value, ensure_ascii=False, indent=2, sort_keys=True) + "\n"
    ).encode("utf-8")


def _compact_canonical_bytes(value: Any) -> bytes:
    return json.dumps(
        value, ensure_ascii=False, separators=(",", ":"), sort_keys=True
    ).encode("utf-8")


def _sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def _load_json_bytes(data: bytes, *, label: str) -> Any:
    try:
        return json.loads(data.decode("utf-8"))
    except (UnicodeDecodeError, json.JSONDecodeError) as exc:
        raise BaselineError(f"{label} is not canonical UTF-8 JSON: {exc}") from exc


def _load_json_file(path: Path) -> Any:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError as exc:
        raise BaselineError(f"missing JSON file: {path}") from exc
    except json.JSONDecodeError as exc:
        raise BaselineError(f"invalid JSON file {path}: {exc}") from exc


def _git(repo_root: Path, args: Sequence[str], *, input_bytes: bytes | None = None) -> bytes:
    command = ["git", *args]
    environment = dict(os.environ)
    for variable in (
        "GIT_ALTERNATE_OBJECT_DIRECTORIES",
        "GIT_COMMON_DIR",
        "GIT_DIR",
        "GIT_INDEX_FILE",
        "GIT_OBJECT_DIRECTORY",
        "GIT_WORK_TREE",
    ):
        environment.pop(variable, None)
    try:
        completed = subprocess.run(
            command,
            cwd=repo_root,
            env=environment,
            input=input_bytes,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=True,
        )
    except FileNotFoundError as exc:
        raise BaselineError("git is required for migration baseline capture") from exc
    except subprocess.CalledProcessError as exc:
        detail = exc.stderr.decode("utf-8", errors="replace").strip()
        raise BaselineError(f"{' '.join(command)} failed: {detail}") from exc
    return completed.stdout


def _validate_relative_path(value: str, *, label: str) -> str:
    if (
        not value
        or "\\" in value
        or any(ord(character) < 32 for character in value)
        or any(part in {"", ".", ".."} for part in value.split("/"))
    ):
        raise BaselineError(f"{label} must be a non-empty POSIX path: {value!r}")
    path = PurePosixPath(value)
    if path.is_absolute():
        raise BaselineError(f"{label} must stay repository-relative: {value!r}")
    return value


@lru_cache(maxsize=None)
def _selector_regex(selector: str) -> re.Pattern[str]:
    _validate_relative_path(selector, label="selector")
    if "[" in selector or "]" in selector or "{" in selector or "}" in selector:
        raise BaselineError(f"selector uses unsupported expansion syntax: {selector}")
    if "***" in selector:
        raise BaselineError(f"selector uses unsupported star run: {selector}")

    result: list[str] = ["^"]
    index = 0
    while index < len(selector):
        char = selector[index]
        if char == "*" and index + 1 < len(selector) and selector[index + 1] == "*":
            index += 2
            if index < len(selector) and selector[index] == "/":
                result.append("(?:.*/)?")
                index += 1
            else:
                result.append(".*")
            continue
        if char == "*":
            result.append("[^/]*")
        elif char == "?":
            result.append("[^/]")
        else:
            result.append(re.escape(char))
        index += 1
    result.append("$")
    return re.compile("".join(result))


def selector_matches(selector: str, path: str) -> bool:
    """Return the documented repository-relative selector match result."""

    _validate_relative_path(path, label="tree path")
    return _selector_regex(selector).fullmatch(path) is not None


class TagSnapshot:
    def __init__(self, repo_root: Path, tag: str, expected_commit: str) -> None:
        self.repo_root = repo_root.resolve()
        self.tag = tag
        self.expected_commit = expected_commit
        self.tag_object_oid, self.peeled_commit = self._resolve_annotated_tag()
        self.entries = self._load_entries()

    def _resolve_annotated_tag(self) -> tuple[str, str]:
        tag_ref = f"refs/tags/{self.tag}"
        oid = _git(self.repo_root, ["rev-parse", "--verify", tag_ref]).decode().strip()
        if not HASH_PATTERN.fullmatch(oid):
            raise BaselineError(f"invalid tag object id for {self.tag}: {oid!r}")
        object_type = _git(self.repo_root, ["cat-file", "-t", oid]).decode().strip()
        if object_type != "tag":
            raise BaselineError(f"{self.tag} must be an annotated tag, got {object_type!r}")
        tag_object = _git(self.repo_root, ["cat-file", "tag", oid]).decode(
            "utf-8", errors="strict"
        )
        headers: dict[str, str] = {}
        for line in tag_object.splitlines():
            if not line:
                break
            key, _, value = line.partition(" ")
            headers[key] = value
        peeled = headers.get("object", "")
        if headers.get("type") != "commit" or not HASH_PATTERN.fullmatch(peeled):
            raise BaselineError(f"{self.tag} does not annotate a commit")
        if peeled != self.expected_commit:
            raise BaselineError(
                f"{self.tag} peels to {peeled}, expected {self.expected_commit}"
            )
        peeled_type = _git(self.repo_root, ["cat-file", "-t", peeled]).decode().strip()
        if peeled_type != "commit":
            raise BaselineError(f"peeled object {peeled} is not a commit")
        return oid, peeled

    def _load_entries(self) -> dict[str, GitEntry]:
        raw = _git(
            self.repo_root,
            ["ls-tree", "-r", "-z", "--full-tree", self.peeled_commit],
        )
        headers: list[tuple[str, str, str]] = []
        for record in raw.split(b"\0"):
            if not record:
                continue
            try:
                metadata, path_bytes = record.split(b"\t", 1)
                mode, object_type, oid = metadata.decode("ascii").split(" ", 2)
                path = path_bytes.decode("utf-8")
            except (ValueError, UnicodeDecodeError) as exc:
                raise BaselineError("git ls-tree returned an invalid record") from exc
            _validate_relative_path(path, label="tag tree path")
            if object_type != "blob" or mode not in {"100644", "100755"}:
                raise BaselineError(
                    f"unsupported tag tree entry {path}: mode={mode}, type={object_type}"
                )
            if not HASH_PATTERN.fullmatch(oid):
                raise BaselineError(f"invalid Git blob id for {path}: {oid}")
            headers.append((path, mode, oid))
        if not headers:
            raise BaselineError(f"annotated tag {self.tag} has an empty tree")

        blob_map = self._cat_file_batch(sorted({oid for _, _, oid in headers}))
        return {
            path: GitEntry(path=path, mode=mode, oid=oid, data=blob_map[oid])
            for path, mode, oid in sorted(headers)
        }

    def _cat_file_batch(self, oids: Sequence[str]) -> dict[str, bytes]:
        payload = b"".join(oid.encode("ascii") + b"\n" for oid in oids)
        output = _git(self.repo_root, ["cat-file", "--batch"], input_bytes=payload)
        cursor = 0
        result: dict[str, bytes] = {}
        for expected_oid in oids:
            line_end = output.find(b"\n", cursor)
            if line_end < 0:
                raise BaselineError("git cat-file --batch truncated an object header")
            header = output[cursor:line_end].decode("ascii", errors="strict")
            cursor = line_end + 1
            parts = header.split(" ")
            if len(parts) != 3 or parts[0] != expected_oid or parts[1] != "blob":
                raise BaselineError(f"unexpected git cat-file header: {header!r}")
            try:
                size = int(parts[2])
            except ValueError as exc:
                raise BaselineError(f"invalid git cat-file size: {header!r}") from exc
            data = output[cursor : cursor + size]
            cursor += size
            if len(data) != size or output[cursor : cursor + 1] != b"\n":
                raise BaselineError(f"git cat-file truncated blob {expected_oid}")
            cursor += 1
            result[expected_oid] = data
        if cursor != len(output):
            raise BaselineError("git cat-file --batch returned unexpected trailing data")
        return result

    def select(self, selectors: Sequence[str], excludes: Sequence[str]) -> list[GitEntry]:
        selected = [
            entry
            for path, entry in self.entries.items()
            if any(selector_matches(selector, path) for selector in selectors)
            and not any(selector_matches(selector, path) for selector in excludes)
        ]
        return sorted(selected, key=lambda item: item.path)

    def exact(self, path: str) -> GitEntry:
        _validate_relative_path(path, label="tag source path")
        try:
            return self.entries[path]
        except KeyError as exc:
            raise BaselineError(f"accepted tag is missing required source: {path}") from exc


def _read_plan(repo_root: Path) -> dict[str, Any]:
    plan = _load_json_file(repo_root / PLAN_RELATIVE)
    schema = _load_json_file(repo_root / PLAN_SCHEMA_RELATIVE)
    failures = validate_instance(plan, schema)
    if failures:
        raise BaselineError("baseline plan schema validation failed:\n- " + "\n- ".join(failures))
    return plan


def _receipt_record(
    repo_root: Path, plan: Mapping[str, Any]
) -> tuple[
    dict[str, Any],
    dict[str, tuple[int, str]],
    dict[str, tuple[int, str]],
]:
    receipt_relative = str(plan["capture_inputs"]["finalized_receipt"])
    _validate_relative_path(receipt_relative, label="acceptance receipt")
    receipt_commit = str(plan["capture_inputs"]["finalized_receipt_commit"])
    expected_oid = str(plan["capture_inputs"]["finalized_receipt_git_blob_oid"])
    expected_sha256 = str(plan["capture_inputs"]["finalized_receipt_sha256"])
    expected_size = plan["capture_inputs"]["finalized_receipt_size_bytes"]
    if not HASH_PATTERN.fullmatch(receipt_commit):
        raise BaselineError("finalized acceptance receipt commit is invalid")
    commit_type = _git(repo_root, ["cat-file", "-t", receipt_commit]).decode().strip()
    if commit_type != "commit":
        raise BaselineError("finalized acceptance receipt source must be a commit")
    tracked_oid = _git(
        repo_root, ["rev-parse", "--verify", f"{receipt_commit}:{receipt_relative}"]
    ).decode().strip()
    if tracked_oid != expected_oid:
        raise BaselineError(
            "finalized acceptance receipt blob does not match the pinned acceptance commit"
        )
    blob_type = _git(repo_root, ["cat-file", "-t", expected_oid]).decode().strip()
    if blob_type != "blob":
        raise BaselineError("finalized acceptance receipt object must be a Git blob")
    data = _git(repo_root, ["cat-file", "blob", expected_oid])
    if _sha256(data) != expected_sha256 or len(data) != expected_size:
        raise BaselineError(
            "finalized acceptance receipt hash/size does not match the capture plan"
        )
    try:
        text = data.decode("utf-8")
    except UnicodeDecodeError as exc:
        raise BaselineError("acceptance receipt must be UTF-8") from exc
    tag = str(plan["release_lineage"]["accepted_tag"])
    commit = str(plan["release_lineage"]["accepted_commit"])
    required_fragments = (
        f"Release Tag: {tag}",
        f"Commit: {commit}",
        "- Owner:",
        "- Reviewer:",
        "Accepted as the final planned feature-bearing Python-led 1.x beta",
    )
    missing = [fragment for fragment in required_fragments if fragment not in text]
    if missing or "- [ ]" in text or "To be confirmed" in text or "TBD" in text:
        detail = ", ".join(missing) if missing else "unchecked or placeholder content"
        raise BaselineError(f"acceptance receipt is not finalized: {detail}")
    oid = _git(repo_root, ["hash-object", "--stdin"], input_bytes=data).decode().strip()
    if not HASH_PATTERN.fullmatch(oid):
        raise BaselineError("git hash-object returned an invalid receipt blob id")
    if oid != expected_oid:
        raise BaselineError(
            "finalized acceptance receipt content does not match the pinned Git blob"
        )
    asset_rows: dict[str, tuple[int, str]] = {}
    row_pattern = re.compile(
        r"^\| `(?P<name>[^`]+)` \| (?P<size>[0-9,]+) \| "
        r"`(?P<sha>[0-9a-f]{64})` \|$"
    )
    for line in text.splitlines():
        match = row_pattern.fullmatch(line)
        if match:
            asset_rows[match.group("name")] = (
                int(match.group("size").replace(",", "")),
                match.group("sha"),
            )
    expected_names = set(plan["capture_inputs"]["release_asset_names"])
    if set(asset_rows) != expected_names:
        raise BaselineError(
            "finalized acceptance receipt asset table does not match the capture plan"
        )
    native_rows: dict[str, tuple[int, str]] = {}
    native_row_pattern = re.compile(
        r"^\| (?P<label>[^|`]+?) \| `(?P<sha>[0-9a-f]{64})` \| "
        r"(?P<size>[0-9,]+) \|$"
    )
    for line in text.splitlines():
        match = native_row_pattern.fullmatch(line)
        if match:
            native_rows[match.group("label")] = (
                int(match.group("size").replace(",", "")),
                match.group("sha"),
            )
    expected_labels = set(plan["capture_inputs"]["native_receipt_labels"].values())
    if set(native_rows) != expected_labels:
        raise BaselineError(
            "finalized acceptance receipt native identity table does not match the capture plan"
        )
    return (
        {
            "git_blob_oid": oid,
            "path": receipt_relative,
            "sha256": _sha256(data),
            "size_bytes": len(data),
            "source_commit": receipt_commit,
            "status": "finalized",
        },
        asset_rows,
        native_rows,
    )


def _file_records(entries: Iterable[GitEntry]) -> list[dict[str, Any]]:
    return [entry.manifest_record() for entry in sorted(entries, key=lambda item: item.path)]


def _domain_records(
    snapshot: TagSnapshot, plan: Mapping[str, Any]
) -> list[dict[str, Any]]:
    excludes = [str(value) for value in plan["inventory"]["exclude_selectors"]]
    records: list[dict[str, Any]] = []
    for domain in plan["inventory"]["domains"]:
        selectors = [str(value) for value in domain["source_selectors"]]
        entries = snapshot.select(selectors, excludes)
        if not entries:
            raise BaselineError(f"inventory domain {domain['id']} selected no tag blobs")
        files = _file_records(entries)
        records.append(
            {
                "file_count": len(files),
                "files": files,
                "id": str(domain["id"]),
                "selectors": selectors,
            }
        )
    return sorted(records, key=lambda value: value["id"])


def _package_tree_records(
    snapshot: TagSnapshot, plan: Mapping[str, Any]
) -> list[dict[str, Any]]:
    records: list[dict[str, Any]] = []
    for root in (str(value) for value in plan["integrity"]["package_tree_roots"]):
        _validate_relative_path(root.rstrip("/"), label="package tree root")
        entries = [entry for path, entry in snapshot.entries.items() if path.startswith(root)]
        if not entries:
            raise BaselineError(f"package tree root selected no tag blobs: {root}")
        files = _file_records(entries)
        records.append(
            {
                "file_count": len(files),
                "files": files,
                "root": root,
                "tree_sha256": _sha256(_compact_canonical_bytes(files)),
            }
        )
    return records


def _safe_archive_name(name: str, *, container: str) -> str:
    normalized = name.rstrip("/")
    if not normalized:
        return normalized
    if "\\" in normalized:
        raise BaselineError(f"unsafe backslash member in {container}: {name}")
    path = PurePosixPath(normalized)
    if (
        path.is_absolute()
        or any(ord(character) < 32 for character in normalized)
        or any(part in {"", ".", ".."} for part in normalized.split("/"))
    ):
        raise BaselineError(f"unsafe archive member in {container}: {name}")
    return normalized


def _archive_member_bytes(path: Path) -> dict[str, bytes]:
    members: dict[str, bytes] = {}
    if zipfile.is_zipfile(path):
        try:
            with zipfile.ZipFile(path) as archive:
                for info in archive.infolist():
                    name = _safe_archive_name(info.filename, container=path.name)
                    if not name or info.is_dir():
                        continue
                    if stat.S_ISLNK(info.external_attr >> 16):
                        raise BaselineError(
                            f"unsupported symlink archive member in {path.name}: {name}"
                        )
                    if name in members:
                        raise BaselineError(f"duplicate archive member in {path.name}: {name}")
                    members[name] = archive.read(info)
        except (OSError, zipfile.BadZipFile) as exc:
            raise BaselineError(f"cannot read ZIP container {path.name}: {exc}") from exc
        return members
    try:
        with tarfile.open(path, mode="r:*") as archive:
            for info in archive.getmembers():
                name = _safe_archive_name(info.name, container=path.name)
                if not name or info.isdir():
                    continue
                if not info.isfile():
                    raise BaselineError(
                        f"unsupported non-file archive member in {path.name}: {name}"
                    )
                if name in members:
                    raise BaselineError(f"duplicate archive member in {path.name}: {name}")
                handle = archive.extractfile(info)
                if handle is None:
                    raise BaselineError(f"cannot read archive member {path.name}:{name}")
                members[name] = handle.read()
    except (OSError, tarfile.TarError) as exc:
        raise BaselineError(f"cannot read native container {path.name}: {exc}") from exc
    return members


def _native_identity_record(container: Path) -> dict[str, Any]:
    members = _archive_member_bytes(container)
    identity_names = sorted(
        name
        for name in members
        if PurePosixPath(name).name == "qiongli-literature-provider.target.json"
    )
    if len(identity_names) != 1:
        raise BaselineError(
            f"{container.name} must contain exactly one native target identity"
        )
    identity_name = identity_names[0]
    identity_bytes = members[identity_name]
    identity = _load_json_bytes(
        identity_bytes, label=f"{container.name}:{identity_name}"
    )
    if not isinstance(identity, dict):
        raise BaselineError(f"native identity in {container.name} must be an object")
    if identity_bytes != _canonical_bytes(identity):
        raise BaselineError(
            f"native identity in {container.name} is not canonical sorted JSON"
        )
    binary_basename = identity.get("binary")
    if not isinstance(binary_basename, str) or not binary_basename:
        raise BaselineError(f"native identity in {container.name} has no binary name")
    binary_name = str(PurePosixPath(identity_name).parent / binary_basename)
    try:
        binary = members[binary_name]
    except KeyError as exc:
        raise BaselineError(
            f"native identity in {container.name} references missing member {binary_name}"
        ) from exc
    expected_sha = identity.get("sha256")
    expected_size = identity.get("size_bytes")
    if expected_sha != _sha256(binary) or expected_size != len(binary):
        raise BaselineError(f"native identity binary hash/size drift in {container.name}")
    return {
        "binary_member": binary_name,
        "container": container.name,
        "identity": identity,
        "identity_document_sha256": _sha256(identity_bytes),
        "identity_member": identity_name,
    }


def _release_records(
    plan: Mapping[str, Any],
    asset_dir: Path,
    accepted_assets: Mapping[str, tuple[int, str]],
    accepted_native: Mapping[str, tuple[int, str]],
) -> tuple[list[dict[str, Any]], list[dict[str, Any]]]:
    expected_names = sorted(str(value) for value in plan["capture_inputs"]["release_asset_names"])
    native_names = sorted(str(value) for value in plan["capture_inputs"]["native_container_names"])
    if not asset_dir.is_dir():
        raise BaselineError(f"release asset directory does not exist: {asset_dir}")
    asset_entries = list(asset_dir.iterdir())
    unsupported_entries = sorted(
        path.name for path in asset_entries if path.is_symlink() or not path.is_file()
    )
    if unsupported_entries:
        raise BaselineError(
            "release asset directory contains unsupported entries: "
            + ", ".join(unsupported_entries)
        )
    actual_names = sorted(path.name for path in asset_entries)
    if actual_names != expected_names:
        missing = sorted(set(expected_names) - set(actual_names))
        extra = sorted(set(actual_names) - set(expected_names))
        raise BaselineError(
            f"release asset set drift: missing={missing or 'none'}, extra={extra or 'none'}"
        )
    assets: list[dict[str, Any]] = []
    for name in expected_names:
        path = asset_dir / name
        if path.is_symlink():
            raise BaselineError(f"release asset must not be a symlink: {name}")
        data = path.read_bytes()
        if not data:
            raise BaselineError(f"release asset is empty: {name}")
        record = {"name": name, "sha256": _sha256(data), "size_bytes": len(data)}
        if accepted_assets.get(name) != (record["size_bytes"], record["sha256"]):
            raise BaselineError(
                f"release asset does not match finalized acceptance evidence: {name}"
            )
        assets.append(record)
    native = [_native_identity_record(asset_dir / name) for name in native_names]
    native = sorted(native, key=lambda value: value["container"])
    _validate_release_evidence(
        plan,
        assets,
        native,
        accepted_assets=accepted_assets,
        accepted_native=accepted_native,
    )
    return assets, native


def _validate_release_evidence(
    plan: Mapping[str, Any],
    assets: Sequence[Mapping[str, Any]],
    native: Sequence[Mapping[str, Any]],
    *,
    accepted_assets: Mapping[str, tuple[int, str]],
    accepted_native: Mapping[str, tuple[int, str]],
) -> None:
    expected_asset_names = sorted(
        str(value) for value in plan["capture_inputs"]["release_asset_names"]
    )
    actual_asset_names = [str(value.get("name")) for value in assets]
    if actual_asset_names != expected_asset_names:
        raise BaselineError("recorded release asset names/order drift from the capture plan")
    for record in assets:
        name = str(record.get("name"))
        actual = (record.get("size_bytes"), record.get("sha256"))
        if accepted_assets.get(name) != actual:
            raise BaselineError(
                f"recorded release asset does not match finalized acceptance evidence: {name}"
            )

    expected_native_names = sorted(
        str(value) for value in plan["capture_inputs"]["native_container_names"]
    )
    actual_native_names = [str(value.get("container")) for value in native]
    if actual_native_names != expected_native_names:
        raise BaselineError("recorded native container names/order drift from the capture plan")
    label_map = plan["capture_inputs"]["native_receipt_labels"]
    binding_map = plan["capture_inputs"]["native_member_bindings"]
    if set(binding_map) != set(expected_native_names):
        raise BaselineError("native member bindings do not match the capture plan containers")
    for record in native:
        container = str(record.get("container"))
        identity = record.get("identity")
        if not isinstance(identity, dict):
            raise BaselineError(f"recorded native identity is invalid: {container}")
        label = label_map.get(container)
        actual = (identity.get("size_bytes"), identity.get("sha256"))
        if not isinstance(label, str) or accepted_native.get(label) != actual:
            raise BaselineError(
                "recorded native identity does not match finalized acceptance "
                f"evidence: {container}"
            )
        identity_member = record.get("identity_member")
        binary_member = record.get("binary_member")
        if not isinstance(identity_member, str) or not isinstance(binary_member, str):
            raise BaselineError(f"recorded native member paths are invalid: {container}")
        if PurePosixPath(identity_member).name != "qiongli-literature-provider.target.json":
            raise BaselineError(f"recorded native identity member name drift: {container}")
        if PurePosixPath(binary_member).parent != PurePosixPath(identity_member).parent:
            raise BaselineError(f"recorded native member parent drift: {container}")
        if PurePosixPath(binary_member).name != identity.get("binary"):
            raise BaselineError(f"recorded native binary member name drift: {container}")
        canonical_identity_sha = _sha256(_canonical_bytes(identity))
        if record.get("identity_document_sha256") != canonical_identity_sha:
            raise BaselineError(f"recorded native identity document hash drift: {container}")
        binding = binding_map.get(container)
        if not isinstance(binding, dict):
            raise BaselineError(f"native member binding is missing: {container}")
        expected_binding = {
            "identity_member": identity_member,
            "binary_member": binary_member,
            "identity_document_sha256": canonical_identity_sha,
        }
        if binding != expected_binding:
            raise BaselineError(f"recorded native member binding drift: {container}")


def _projection_entries(
    snapshot: TagSnapshot, oracle: Mapping[str, Any], excludes: Sequence[str]
) -> list[GitEntry]:
    selectors = [str(value) for value in oracle["projection"]["source_selectors"]]
    entries = snapshot.select(selectors, excludes)
    if not entries:
        raise BaselineError(f"oracle {oracle['id']} selected no canonical tag fixtures")
    return entries


def _runtime_source_tree(snapshot: TagSnapshot, source_root: str) -> dict[str, Any]:
    root = source_root.rstrip("/") + "/"
    files = _file_records(
        entry for path, entry in snapshot.entries.items() if path.startswith(root)
    )
    if not files:
        raise BaselineError(f"runtime source tree is empty: {root}")
    return {
        "kind": "peeled-tag-materialization",
        "tag": snapshot.tag,
        "peeled_commit": snapshot.peeled_commit,
        "source_root": root,
        "source_file_count": len(files),
        "source_tree_sha256": _sha256(_compact_canonical_bytes(files)),
    }


def _materialize_tag_snapshot(snapshot: TagSnapshot, destination: Path) -> None:
    destination.mkdir(parents=True, exist_ok=False)
    for entry in snapshot.entries.values():
        target = destination / entry.path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(entry.data)
        target.chmod(0o755 if entry.mode == "100755" else 0o644)
    # Source-layout discovery intentionally requires a checkout marker. This empty
    # marker is capture scaffolding only; every implementation byte still comes
    # from the immutable peeled tag via ls-tree/cat-file.
    (destination / ".git").mkdir()


def _runtime_file_state(root: Path) -> dict[str, dict[str, Any]]:
    if not root.exists():
        return {}
    records: dict[str, dict[str, Any]] = {}
    for path in sorted(root.rglob("*")):
        if path.is_symlink():
            raise BaselineError(f"runtime sandbox contains a symlink: {path}")
        if not path.is_file():
            continue
        relative = path.relative_to(root).as_posix()
        _validate_relative_path(relative, label="runtime sandbox path")
        data = path.read_bytes()
        records[relative] = {
            "path": relative,
            "sha256": _sha256(data),
            "size_bytes": len(data),
        }
    return records


def _runtime_tree_sha256(state: Mapping[str, Mapping[str, Any]]) -> str:
    return _sha256(
        _compact_canonical_bytes([state[path] for path in sorted(state)])
    )


def _runtime_filesystem_delta(
    before: Mapping[str, Mapping[str, Any]],
    after: Mapping[str, Mapping[str, Any]],
) -> dict[str, Any]:
    created = [dict(after[path]) for path in sorted(set(after) - set(before))]
    deleted = [dict(before[path]) for path in sorted(set(before) - set(after))]
    modified: list[dict[str, Any]] = []
    for path in sorted(set(before) & set(after)):
        prior = before[path]
        current = after[path]
        if prior != current:
            modified.append(
                {
                    "path": path,
                    "before_sha256": prior["sha256"],
                    "before_size_bytes": prior["size_bytes"],
                    "after_sha256": current["sha256"],
                    "after_size_bytes": current["size_bytes"],
                }
            )
    return {
        "before_tree_sha256": _runtime_tree_sha256(before),
        "after_tree_sha256": _runtime_tree_sha256(after),
        "created": created,
        "modified": modified,
        "deleted": deleted,
    }


def _runtime_environment(
    sandbox: Path,
    *,
    config_home: Path | str | None = None,
    extra: Mapping[str, str] | None = None,
) -> dict[str, str]:
    temp_root = sandbox / "tmp"
    temp_root.mkdir(parents=True, exist_ok=True)
    home = sandbox / "home"
    config = config_home if config_home is not None else sandbox / "config"
    environment = {
        "HOME": str(home),
        "USERPROFILE": str(home),
        "APPDATA": str(sandbox / "appdata"),
        "LOCALAPPDATA": str(sandbox / "localappdata"),
        "XDG_CONFIG_HOME": str(sandbox / "xdg-config"),
        "QIONGLI_CONFIG_HOME": str(config),
        "TMPDIR": str(temp_root),
        "TMP": str(temp_root),
        "TEMP": str(temp_root),
        "PATH": "",
        "PYTHONHASHSEED": "0",
        "PYTHONDONTWRITEBYTECODE": "1",
        "PYTHONUTF8": "1",
        "RESEARCH_CLI_LANG": "en",
        "NO_COLOR": "1",
    }
    for name in ("SYSTEMROOT", "WINDIR", "COMSPEC", "PATHEXT"):
        if os.environ.get(name):
            environment[name] = os.environ[name]
    if extra:
        environment.update({str(key): str(value) for key, value in extra.items()})
    return environment


def _runtime_replacements(
    *,
    tag_root: Path,
    sandbox: Path,
    config_home: Path | None = None,
    executable: Path | None = None,
) -> dict[str, str]:
    replacements = {
        str(tag_root): "<TAG_ROOT>",
        str(tag_root.resolve()): "<TAG_ROOT>",
        str(sandbox): "<SANDBOX>",
        str(sandbox.resolve()): "<SANDBOX>",
    }
    # Replace well-known sandbox children before the sandbox root itself. A
    # value such as ``<SANDBOX>/home`` would otherwise still resemble an
    # absolute host path to the strict portability scanner.
    for child, token in (
        ("home", "<HOME>"),
        ("appdata", "<APPDATA>"),
        ("localappdata", "<LOCALAPPDATA>"),
        ("xdg-config", "<XDG_CONFIG_HOME>"),
        ("tmp", "<TEMP_ROOT>"),
    ):
        child_path = sandbox / child
        replacements[str(child_path)] = token
        replacements[str(child_path.resolve())] = token
    if config_home is not None:
        replacements[str(config_home)] = "<CONFIG_ROOT>"
        replacements[str(config_home.resolve())] = "<CONFIG_ROOT>"
    if executable is not None:
        replacements[str(executable)] = "<RUNTIME_EXECUTABLE>"
        replacements[str(executable.resolve())] = "<RUNTIME_EXECUTABLE>"
    return replacements


def _normalize_runtime_string(
    value: str,
    replacements: Mapping[str, str],
    *,
    forbidden_secrets: Sequence[str] = (),
) -> str:
    for secret in forbidden_secrets:
        if secret and secret in value:
            raise BaselineError("runtime oracle output leaked a capture secret")
    normalized = value.replace("\r\n", "\n").replace("\r", "\n")
    for original, replacement in sorted(
        replacements.items(), key=lambda item: len(item[0]), reverse=True
    ):
        if original:
            normalized = normalized.replace(original, replacement)
    normalized = normalized.replace("\\", "/")
    normalized = re.sub(
        r"(<RUNTIME_EXECUTABLE>)\s*\([^\n)]+\)",
        r"\1 (<RUNTIME_VERSION>)",
        normalized,
    )
    return normalized


def _normalize_runtime_value(
    value: Any,
    replacements: Mapping[str, str],
    *,
    forbidden_secrets: Sequence[str] = (),
) -> Any:
    if isinstance(value, dict):
        return {
            str(key): _normalize_runtime_value(
                item,
                replacements,
                forbidden_secrets=forbidden_secrets,
            )
            for key, item in value.items()
        }
    if isinstance(value, list):
        return [
            _normalize_runtime_value(
                item,
                replacements,
                forbidden_secrets=forbidden_secrets,
            )
            for item in value
        ]
    if isinstance(value, str):
        return _normalize_runtime_string(
            value, replacements, forbidden_secrets=forbidden_secrets
        )
    return value


def _run_runtime_process(
    command: Sequence[str],
    *,
    cwd: Path,
    environment: Mapping[str, str],
    sandbox: Path,
    input_text: str = "",
    guard_roots: Sequence[Path] = (),
    forbidden_secrets: Sequence[str] = (),
) -> tuple[subprocess.CompletedProcess[str], dict[str, Any]]:
    before = _runtime_file_state(sandbox)
    guarded_before = {
        str(root): _runtime_file_state(root) for root in guard_roots
    }
    try:
        completed = subprocess.run(
            list(command),
            cwd=cwd,
            env=dict(environment),
            input=input_text,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            encoding="utf-8",
            errors="strict",
            timeout=30,
            check=False,
        )
    except (OSError, subprocess.TimeoutExpired) as exc:
        raise BaselineError(f"runtime oracle command failed: {command[0]}: {exc}") from exc
    for secret in forbidden_secrets:
        if secret and (secret in completed.stdout or secret in completed.stderr):
            raise BaselineError("runtime oracle process echoed a capture secret")
    for root in guard_roots:
        if guarded_before[str(root)] != _runtime_file_state(root):
            raise BaselineError(f"runtime oracle wrote outside its sandbox: {root}")
    after = _runtime_file_state(sandbox)
    return completed, _runtime_filesystem_delta(before, after)


def _jsonrpc_runtime(
    command: Sequence[str],
    requests: Sequence[Mapping[str, Any]],
    *,
    cwd: Path,
    environment: Mapping[str, str],
    sandbox: Path,
    guard_roots: Sequence[Path],
    forbidden_secrets: Sequence[str] = (),
) -> tuple[subprocess.CompletedProcess[str], list[dict[str, Any]], dict[str, Any]]:
    input_text = "".join(
        json.dumps(request, ensure_ascii=False, separators=(",", ":")) + "\n"
        for request in requests
    )
    completed, delta = _run_runtime_process(
        command,
        cwd=cwd,
        environment=environment,
        sandbox=sandbox,
        input_text=input_text,
        guard_roots=guard_roots,
        forbidden_secrets=forbidden_secrets,
    )
    if completed.returncode != 0:
        raise BaselineError(
            f"runtime JSON-RPC process exited {completed.returncode}: {completed.stderr.strip()}"
        )
    responses: list[dict[str, Any]] = []
    for line in completed.stdout.splitlines():
        if not line.strip():
            continue
        response = _load_json_bytes(line.encode("utf-8"), label="runtime JSON-RPC response")
        if not isinstance(response, dict):
            raise BaselineError("runtime JSON-RPC response is not an object")
        responses.append(response)
    if len(responses) != len(requests):
        raise BaselineError(
            f"runtime JSON-RPC response count drift: {len(responses)} != {len(requests)}"
        )
    return completed, responses, delta


def _tool_payload(response: Mapping[str, Any]) -> dict[str, Any]:
    result = response.get("result")
    if not isinstance(result, dict):
        raise BaselineError("runtime tool call has no result object")
    structured = result.get("structuredContent")
    if isinstance(structured, dict):
        return structured
    content = result.get("content")
    if isinstance(content, list) and content and isinstance(content[0], dict):
        text = content[0].get("text")
        if isinstance(text, str):
            payload = _load_json_bytes(text.encode("utf-8"), label="runtime tool payload")
            if isinstance(payload, dict):
                return payload
    raise BaselineError("runtime tool result has no structured payload")


def _runtime_case(
    *,
    case_id: str,
    kind: str,
    coverage: Sequence[str],
    source_paths: Sequence[str],
    transport: str,
    operation: str,
    arguments: Mapping[str, Any],
    exit_code: int,
    value: Mapping[str, Any],
    delta: Mapping[str, Any],
    error: Mapping[str, Any] | None = None,
) -> dict[str, Any]:
    changed = any(delta[name] for name in ("created", "modified", "deleted"))
    return {
        "id": case_id,
        "kind": kind,
        "disposition": "captured",
        "coverage": list(coverage),
        "source_paths": list(source_paths),
        "invocation": {
            "transport": transport,
            "operation": operation,
            "arguments": dict(arguments),
        },
        "outcome": {
            "status": "error" if error is not None else "success",
            "exit_code": exit_code,
            "error": dict(error) if error is not None else None,
            "value": dict(value),
        },
        "side_effects": {
            "class": "bounded-write" if changed else "none",
            "filesystem_delta": dict(delta),
            "writes_outside_sandbox": False,
        },
        "assertions": {
            "secret_absent": True,
            "machine_paths_normalized": True,
        },
    }


def _select_python_runtime(repo_root: Path) -> Path:
    candidates: list[Path] = []
    configured = os.environ.get("QIONGLI_BASELINE_PYTHON", "").strip()
    if configured:
        resolved = shutil.which(configured)
        candidates.append(Path(resolved or configured))
    candidates.extend(
        [
            repo_root / ".venv/bin/python",
            repo_root / ".venv/Scripts/python.exe",
            Path(sys.executable),
        ]
    )
    seen: set[str] = set()
    for candidate in candidates:
        key = str(candidate)
        if key in seen or not candidate.is_file():
            continue
        seen.add(key)
        probe = subprocess.run(
            [str(candidate), "-c", "import sys, yaml; assert sys.version_info >= (3, 12)"],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            check=False,
        )
        if probe.returncode == 0:
            # Keep a virtual-environment launcher as the invoked path. Resolving
            # its symlink can discard the venv context and its PyYAML dependency.
            return candidate.absolute()
    raise BaselineError(
        "runtime capture requires Python >=3.12 with PyYAML; set QIONGLI_BASELINE_PYTHON"
    )


def _select_node_runtime() -> Path:
    configured = os.environ.get("QIONGLI_BASELINE_NODE", "").strip()
    resolved = shutil.which(configured or "node")
    if not resolved:
        raise BaselineError("runtime capture requires Node.js >=18")
    probe = subprocess.run(
        [resolved, "-p", "JSON.stringify({execPath:process.execPath,major:Number(process.versions.node.split('.')[0])})"],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        encoding="utf-8",
        errors="strict",
        check=False,
    )
    if probe.returncode != 0:
        raise BaselineError("runtime capture could not resolve the Node.js executable")
    payload = _load_json_bytes(probe.stdout.strip().encode("utf-8"), label="Node.js probe")
    if (
        not isinstance(payload, dict)
        or not isinstance(payload.get("execPath"), str)
        or not isinstance(payload.get("major"), int)
        or payload["major"] < 18
    ):
        raise BaselineError("runtime capture requires Node.js >=18")
    executable = Path(payload["execPath"])
    if not executable.is_absolute() or not executable.is_file():
        raise BaselineError("Node.js probe returned an invalid executable path")
    return executable


def _normalized_lines(
    text: str,
    replacements: Mapping[str, str],
    *,
    forbidden_secrets: Sequence[str] = (),
) -> list[str]:
    normalized = _normalize_runtime_string(
        text, replacements, forbidden_secrets=forbidden_secrets
    )
    return normalized.strip("\n").splitlines() if normalized.strip("\n") else []


def _capture_python_runtime_cases(
    *,
    repo_root: Path,
    tag_root: Path,
    runtime_root: Path,
) -> list[dict[str, Any]]:
    python = _select_python_runtime(repo_root)
    python_source = tag_root / "packages/python-qiongli/src"
    base_command = [str(python)]
    cases: list[dict[str, Any]] = []

    cli_sandbox = runtime_root / "python-cli"
    cli_sandbox.mkdir(parents=True)
    cli_environment = _runtime_environment(
        cli_sandbox, extra={"PYTHONPATH": str(python_source)}
    )
    cli_replacements = _runtime_replacements(
        tag_root=tag_root, sandbox=cli_sandbox, executable=python
    )
    completed, delta = _run_runtime_process(
        [*base_command, "-m", "qiongli.cli", "align"],
        cwd=tag_root,
        environment=cli_environment,
        sandbox=cli_sandbox,
        guard_roots=[tag_root],
    )
    if completed.returncode != 0:
        raise BaselineError(f"Python Full CLI capture failed: {completed.stderr}")
    cases.append(
        _runtime_case(
            case_id="python.cli-align",
            kind="cli-outcome",
            coverage=["cli-command"],
            source_paths=["packages/python-qiongli/src/qiongli/cli.py"],
            transport="cli",
            operation="qiongli align",
            arguments={"argv": ["align"]},
            exit_code=completed.returncode,
            value={
                "stdout_lines": _normalized_lines(
                    completed.stdout, cli_replacements
                ),
                "stderr_lines": _normalized_lines(
                    completed.stderr, cli_replacements
                ),
            },
            delta=delta,
        )
    )

    mcp_command = [*base_command, "-m", "bridges.mcp_server_stdio"]
    mcp_sandbox = runtime_root / "python-mcp-list"
    mcp_sandbox.mkdir(parents=True)
    mcp_environment = _runtime_environment(
        mcp_sandbox, extra={"PYTHONPATH": str(python_source)}
    )
    mcp_replacements = _runtime_replacements(
        tag_root=tag_root, sandbox=mcp_sandbox, executable=python
    )
    completed, responses, delta = _jsonrpc_runtime(
        mcp_command,
        [
            {
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": {},
                    "clientInfo": {"name": "baseline", "version": "1"},
                },
            },
            {"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}},
        ],
        cwd=tag_root,
        environment=mcp_environment,
        sandbox=mcp_sandbox,
        guard_roots=[tag_root],
    )
    initialize = responses[0].get("result")
    listed = responses[1].get("result")
    if not isinstance(initialize, dict) or not isinstance(listed, dict):
        raise BaselineError("Python Full MCP initialize/list capture failed")
    tools = listed.get("tools")
    if not isinstance(tools, list):
        raise BaselineError("Python Full MCP tools/list returned no tools")
    cases.append(
        _runtime_case(
            case_id="python.mcp-initialize-list",
            kind="jsonrpc-outcome",
            coverage=["mcp-initialize-and-list"],
            source_paths=[
                "packages/python-qiongli/src/qiongli/bridges/mcp_server_stdio.py",
                "packages/python-qiongli/src/qiongli/bridges/mcp_tool_handlers.py",
            ],
            transport="jsonrpc-stdio",
            operation="initialize + tools/list",
            arguments={"protocol_version": "2024-11-05"},
            exit_code=completed.returncode,
            value=_normalize_runtime_value(
                {
                    "initialize": initialize,
                    "tool_names": [tool.get("name") for tool in tools if isinstance(tool, dict)],
                    "tool_count": len(tools),
                    "stderr_lines": _normalized_lines(
                        completed.stderr, mcp_replacements
                    ),
                },
                mcp_replacements,
            ),
            delta=delta,
        )
    )

    orchestration_sandbox = runtime_root / "python-orchestration"
    orchestration_sandbox.mkdir(parents=True)
    project_root = orchestration_sandbox / "project"
    project_root.mkdir()
    orchestration_environment = _runtime_environment(
        orchestration_sandbox, extra={"PYTHONPATH": str(python_source)}
    )
    orchestration_replacements = _runtime_replacements(
        tag_root=tag_root,
        sandbox=orchestration_sandbox,
        executable=python,
    )
    completed, responses, delta = _jsonrpc_runtime(
        mcp_command,
        [
            {
                "jsonrpc": "2.0",
                "id": 3,
                "method": "tools/call",
                "params": {
                    "name": "qiongli_task_run",
                    "arguments": {
                        "cwd": str(project_root),
                        "task_id": "F3",
                        "paper_type": "empirical",
                        "topic": "runtime-baseline",
                        "guidance_mode": "off",
                    },
                },
            }
        ],
        cwd=tag_root,
        environment=orchestration_environment,
        sandbox=orchestration_sandbox,
        guard_roots=[tag_root],
    )
    payload = _tool_payload(responses[0])
    data = payload.get("data") if isinstance(payload.get("data"), dict) else {}
    preview = (
        data.get("task_run_preview")
        if isinstance(data.get("task_run_preview"), dict)
        else {}
    )
    task_packet = (
        data.get("task_packet") if isinstance(data.get("task_packet"), dict) else {}
    )
    orchestration_value = {
        "mode": payload.get("mode"),
        "run_agents": payload.get("run_agents"),
        "task_description": payload.get("task_description"),
        "will_launch_agents": preview.get("will_launch_agents"),
        "controller_metadata": preview.get("controller_metadata", {}),
        "effective_domain": preview.get("effective_domain"),
        "task": {
            "task_id": task_packet.get("task_id"),
            "paper_type": task_packet.get("paper_type"),
            "topic": task_packet.get("topic"),
        },
        "stderr_lines": _normalized_lines(
            completed.stderr, orchestration_replacements
        ),
    }
    if orchestration_value["mode"] != "task-run-preview":
        raise BaselineError("Python Full orchestration did not remain preview-first")
    cases.append(
        _runtime_case(
            case_id="python.orchestration-preview",
            kind="orchestration-outcome",
            coverage=["orchestration-preview"],
            source_paths=[
                "packages/python-qiongli/src/qiongli/bridges/orchestrator.py",
                "packages/python-qiongli/src/qiongli/bridges/mcp_tool_handlers.py",
                "content/standards/research-workflow-contract.yaml",
                "content/standards/mcp-agent-capability-map.yaml",
            ],
            transport="jsonrpc-stdio",
            operation="tools/call qiongli_task_run",
            arguments={
                "task_id": "F3",
                "paper_type": "empirical",
                "topic": "runtime-baseline",
                "guidance_mode": "off",
                "run_agents": False,
            },
            exit_code=completed.returncode,
            value=_normalize_runtime_value(
                orchestration_value, orchestration_replacements
            ),
            delta=delta,
        )
    )

    installer_sandbox = runtime_root / "python-installer"
    installer_sandbox.mkdir(parents=True)
    installer_environment = _runtime_environment(
        installer_sandbox, extra={"PYTHONPATH": str(python_source)}
    )
    installer_replacements = _runtime_replacements(
        tag_root=tag_root, sandbox=installer_sandbox, executable=python
    )
    completed, delta = _run_runtime_process(
        [
            *base_command,
            "-m",
            "qiongli.cli",
            "install",
            "--profile",
            "full",
            "--target",
            "codex",
            "--surface",
            "plugin",
            "--dry-run",
        ],
        cwd=tag_root,
        environment=installer_environment,
        sandbox=installer_sandbox,
        guard_roots=[tag_root],
    )
    if completed.returncode != 0:
        raise BaselineError(f"Python Full installer dry-run failed: {completed.stderr}")
    cases.append(
        _runtime_case(
            case_id="python.installer-dry-run",
            kind="installer-outcome",
            coverage=["installer-dry-run"],
            source_paths=[
                "packages/python-qiongli/src/qiongli/cli.py",
                "packages/python-qiongli/src/qiongli/universal_installer.py",
            ],
            transport="cli",
            operation="qiongli install --dry-run",
            arguments={
                "profile": "full",
                "target": "codex",
                "surface": "plugin",
                "dry_run": True,
            },
            exit_code=completed.returncode,
            value={
                "stdout_lines": _normalized_lines(
                    completed.stdout, installer_replacements
                ),
                "stderr_lines": _normalized_lines(
                    completed.stderr, installer_replacements
                ),
            },
            delta=delta,
        )
    )

    mutable_sandbox = runtime_root / "python-mutable-state"
    mutable_sandbox.mkdir(parents=True)
    config_home = mutable_sandbox / "config"
    mutable_environment = _runtime_environment(
        mutable_sandbox,
        config_home=config_home,
        extra={"PYTHONPATH": str(python_source)},
    )
    mutable_replacements = _runtime_replacements(
        tag_root=tag_root,
        sandbox=mutable_sandbox,
        config_home=config_home,
        executable=python,
    )
    completed, responses, delta = _jsonrpc_runtime(
        mcp_command,
        [
            {
                "jsonrpc": "2.0",
                "id": 4,
                "method": "tools/call",
                "params": {
                    "name": "qiongli_save_provider_config",
                    "arguments": {
                        "provider": "semantic-scholar",
                        "field": "api-key",
                        "value": RUNTIME_SECRET_CANARY,
                    },
                },
            },
            {
                "jsonrpc": "2.0",
                "id": 5,
                "method": "tools/call",
                "params": {"name": "qiongli_config_status", "arguments": {}},
            },
        ],
        cwd=tag_root,
        environment=mutable_environment,
        sandbox=mutable_sandbox,
        guard_roots=[tag_root],
        forbidden_secrets=[RUNTIME_SECRET_CANARY],
    )
    save_payload = _tool_payload(responses[0])
    status_payload = _tool_payload(responses[1])
    if not delta["created"] and not delta["modified"]:
        raise BaselineError("Python Full mutable-state capture wrote no bounded state")
    cases.append(
        _runtime_case(
            case_id="python.mutable-provider-state",
            kind="mutable-state-outcome",
            coverage=["mutable-provider-state"],
            source_paths=[
                "packages/python-qiongli/src/qiongli/bridges/mcp_tool_handlers.py",
                "packages/python-qiongli/src/qiongli/bridges/provider_config.py",
            ],
            transport="jsonrpc-stdio",
            operation="save provider config + read redacted status",
            arguments={
                "provider": "semantic-scholar",
                "field": "api-key",
                "value": "<REDACTED>",
            },
            exit_code=completed.returncode,
            value=_normalize_runtime_value(
                {
                    "save": save_payload,
                    "status": status_payload,
                    "redaction_assertion": "capture secret absent from process output",
                },
                mutable_replacements,
                forbidden_secrets=[RUNTIME_SECRET_CANARY],
            ),
            delta=delta,
        )
    )
    return cases


def _capture_node_runtime_cases(
    *,
    tag_root: Path,
    runtime_root: Path,
) -> list[dict[str, Any]]:
    node = _select_node_runtime()
    server = tag_root / "packages/qiongli-literature-mcpb/server/index.mjs"
    command = [str(node), str(server)]
    cases: list[dict[str, Any]] = []

    list_sandbox = runtime_root / "node-mcp-list"
    list_sandbox.mkdir(parents=True)
    list_environment = _runtime_environment(list_sandbox)
    list_replacements = _runtime_replacements(
        tag_root=tag_root, sandbox=list_sandbox, executable=node
    )
    completed, responses, delta = _jsonrpc_runtime(
        command,
        [
            {
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": {},
                    "clientInfo": {"name": "baseline", "version": "1"},
                },
            },
            {"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}},
        ],
        cwd=tag_root,
        environment=list_environment,
        sandbox=list_sandbox,
        guard_roots=[tag_root],
    )
    initialize = responses[0].get("result")
    listed = responses[1].get("result")
    if not isinstance(initialize, dict) or not isinstance(listed, dict):
        raise BaselineError("legacy Node MCPB initialize/list capture failed")
    tools = listed.get("tools")
    if not isinstance(tools, list):
        raise BaselineError("legacy Node MCPB tools/list returned no tools")
    cases.append(
        _runtime_case(
            case_id="node.mcp-initialize-list",
            kind="jsonrpc-outcome",
            coverage=["mcp-initialize-and-list"],
            source_paths=[
                "packages/qiongli-literature-mcpb/server/index.mjs",
                "packages/qiongli-literature-mcpb/server/stdio.mjs",
            ],
            transport="jsonrpc-stdio",
            operation="initialize + tools/list",
            arguments={"protocol_version": "2024-11-05"},
            exit_code=completed.returncode,
            value=_normalize_runtime_value(
                {
                    "initialize": initialize,
                    "tool_names": [tool.get("name") for tool in tools if isinstance(tool, dict)],
                    "tool_count": len(tools),
                    "stderr_lines": _normalized_lines(
                        completed.stderr, list_replacements
                    ),
                },
                list_replacements,
            ),
            delta=delta,
        )
    )

    normalization_sandbox = runtime_root / "node-normalization"
    normalization_sandbox.mkdir(parents=True)
    normalization_environment = _runtime_environment(normalization_sandbox)
    normalization_replacements = _runtime_replacements(
        tag_root=tag_root, sandbox=normalization_sandbox, executable=node
    )
    driver = """
import { readFile } from 'node:fs/promises';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
const root = process.cwd();
const moduleUrl = pathToFileURL(path.join(root, 'packages/qiongli-literature-mcpb/server/providers/crossref.mjs')).href;
const { searchCrossref } = await import(moduleUrl);
const fixture = JSON.parse(await readFile(path.join(root, 'content/mcp-contracts/fixtures/crossref-search-response.json'), 'utf8'));
const response = await searchCrossref({
  query: 'runtime baseline',
  limit: 1,
  fetchImpl: async () => ({ ok: true, status: 200, async json() { return fixture; } })
});
process.stdout.write(JSON.stringify(response) + '\\n');
""".strip()
    completed, delta = _run_runtime_process(
        [str(node), "--input-type=module", "--eval", driver],
        cwd=tag_root,
        environment=normalization_environment,
        sandbox=normalization_sandbox,
        guard_roots=[tag_root],
    )
    if completed.returncode != 0:
        raise BaselineError(f"legacy Node normalization capture failed: {completed.stderr}")
    normalized_result = _load_json_bytes(
        completed.stdout.strip().encode("utf-8"), label="legacy Node normalization result"
    )
    if not isinstance(normalized_result, dict):
        raise BaselineError("legacy Node normalization result is not an object")
    cases.append(
        _runtime_case(
            case_id="node.crossref-normalization",
            kind="provider-normalization-outcome",
            coverage=["provider-normalization"],
            source_paths=[
                "packages/qiongli-literature-mcpb/server/providers/crossref.mjs",
                "packages/qiongli-literature-mcpb/server/providers/http.mjs",
                "packages/qiongli-literature-mcpb/server/normalize.mjs",
                "content/mcp-contracts/fixtures/crossref-search-response.json",
            ],
            transport="module-fixture-driver",
            operation="searchCrossref with canonical mocked response",
            arguments={
                "query": "runtime baseline",
                "limit": 1,
                "fixture": "content/mcp-contracts/fixtures/crossref-search-response.json",
                "network": False,
            },
            exit_code=completed.returncode,
            value=_normalize_runtime_value(
                {
                    "result": normalized_result,
                    "stderr_lines": _normalized_lines(
                        completed.stderr, normalization_replacements
                    ),
                },
                normalization_replacements,
            ),
            delta=delta,
        )
    )

    plan_sandbox = runtime_root / "node-search-plan"
    plan_sandbox.mkdir(parents=True)
    plan_environment = _runtime_environment(plan_sandbox)
    plan_replacements = _runtime_replacements(
        tag_root=tag_root, sandbox=plan_sandbox, executable=node
    )
    completed, responses, delta = _jsonrpc_runtime(
        command,
        [
            {
                "jsonrpc": "2.0",
                "id": 3,
                "method": "tools/call",
                "params": {
                    "name": "qiongli_search_plan",
                    "arguments": {
                        "query": "runtime oracle",
                        "platform": "codex",
                        "native_search_available": True,
                        "native_search_tools": ["codex_web_search"],
                    },
                },
            }
        ],
        cwd=tag_root,
        environment=plan_environment,
        sandbox=plan_sandbox,
        guard_roots=[tag_root],
    )
    payload = _tool_payload(responses[0])
    plan_value = {
        key: payload.get(key)
        for key in (
            "artifact_type",
            "query",
            "platform",
            "provider_capability_mode",
            "search_execution_mode",
            "provider_queries",
            "native_search_queries",
            "limitations",
        )
    }
    cases.append(
        _runtime_case(
            case_id="node.search-plan",
            kind="search-plan-outcome",
            coverage=["search-plan"],
            source_paths=[
                "packages/qiongli-literature-mcpb/server/index.mjs",
                "packages/qiongli-literature-mcpb/server/search-plan.mjs",
            ],
            transport="jsonrpc-stdio",
            operation="tools/call qiongli_search_plan",
            arguments={
                "query": "runtime oracle",
                "platform": "codex",
                "native_search_available": True,
                "native_search_tools": ["codex_web_search"],
            },
            exit_code=completed.returncode,
            value=_normalize_runtime_value(
                {
                    "plan": plan_value,
                    "stderr_lines": _normalized_lines(
                        completed.stderr, plan_replacements
                    ),
                },
                plan_replacements,
            ),
            delta=delta,
        )
    )

    malformed_sandbox = runtime_root / "node-malformed-config"
    malformed_sandbox.mkdir(parents=True)
    malformed_config = malformed_sandbox / "config"
    malformed_config.mkdir()
    (malformed_config / "providers.json").write_text(
        "{not-json " + RUNTIME_SECRET_CANARY,
        encoding="utf-8",
    )
    malformed_environment = _runtime_environment(
        malformed_sandbox, config_home=malformed_config
    )
    malformed_replacements = _runtime_replacements(
        tag_root=tag_root,
        sandbox=malformed_sandbox,
        config_home=malformed_config,
        executable=node,
    )
    completed, responses, delta = _jsonrpc_runtime(
        command,
        [
            {
                "jsonrpc": "2.0",
                "id": 4,
                "method": "tools/call",
                "params": {"name": "qiongli_config_status", "arguments": {}},
            }
        ],
        cwd=tag_root,
        environment=malformed_environment,
        sandbox=malformed_sandbox,
        guard_roots=[tag_root],
        forbidden_secrets=[RUNTIME_SECRET_CANARY],
    )
    error = responses[0].get("error")
    if not isinstance(error, dict):
        raise BaselineError("legacy Node malformed config did not fail closed")
    cases.append(
        _runtime_case(
            case_id="node.malformed-config-fail-closed",
            kind="fail-closed-outcome",
            coverage=["malformed-config-fail-closed"],
            source_paths=[
                "packages/qiongli-literature-mcpb/server/config.mjs",
                "packages/qiongli-literature-mcpb/server/stdio.mjs",
            ],
            transport="jsonrpc-stdio",
            operation="tools/call qiongli_config_status with malformed config",
            arguments={"config_fixture": "malformed-redacted-json"},
            exit_code=completed.returncode,
            value={
                "config_preserved": delta["before_tree_sha256"]
                == delta["after_tree_sha256"],
                "stderr_lines": _normalized_lines(
                    completed.stderr,
                    malformed_replacements,
                    forbidden_secrets=[RUNTIME_SECRET_CANARY],
                ),
            },
            delta=delta,
            error={"code": int(error.get("code", -32603)), "message": str(error.get("message", ""))},
        )
    )

    unsupported_sandbox = runtime_root / "node-unsupported-config"
    unsupported_sandbox.mkdir(parents=True)
    unsupported_environment = _runtime_environment(
        unsupported_sandbox, config_home="relative-config-home"
    )
    unsupported_replacements = _runtime_replacements(
        tag_root=tag_root, sandbox=unsupported_sandbox, executable=node
    )
    completed, responses, delta = _jsonrpc_runtime(
        command,
        [
            {
                "jsonrpc": "2.0",
                "id": 5,
                "method": "tools/call",
                "params": {"name": "qiongli_config_status", "arguments": {}},
            }
        ],
        cwd=tag_root,
        environment=unsupported_environment,
        sandbox=unsupported_sandbox,
        guard_roots=[tag_root],
    )
    error = responses[0].get("error")
    if not isinstance(error, dict):
        raise BaselineError("legacy Node unsupported config home did not fail closed")
    cases.append(
        _runtime_case(
            case_id="node.unsupported-config-fail-closed",
            kind="fail-closed-outcome",
            coverage=["unsupported-config-fail-closed"],
            source_paths=[
                "packages/qiongli-literature-mcpb/server/config.mjs",
                "packages/qiongli-literature-mcpb/server/stdio.mjs",
            ],
            transport="jsonrpc-stdio",
            operation="tools/call qiongli_config_status with relative config home",
            arguments={"config_home": "relative-config-home"},
            exit_code=completed.returncode,
            value={
                "filesystem_unchanged": delta["before_tree_sha256"]
                == delta["after_tree_sha256"],
                "stderr_lines": _normalized_lines(
                    completed.stderr, unsupported_replacements
                ),
            },
            delta=delta,
            error={"code": int(error.get("code", -32603)), "message": str(error.get("message", ""))},
        )
    )
    return cases


def _accepted_rust_runtime_source(
    release_assets: Sequence[Mapping[str, Any]],
    native_identities: Sequence[Mapping[str, Any]],
) -> dict[str, Any]:
    asset = next(
        (
            record
            for record in release_assets
            if record.get("name") == DESKTOP_RUNTIME_CONTAINER
        ),
        None,
    )
    native = next(
        (
            record
            for record in native_identities
            if record.get("container") == DESKTOP_RUNTIME_CONTAINER
        ),
        None,
    )
    if not isinstance(asset, Mapping) or not isinstance(native, Mapping):
        raise BaselineError("accepted Desktop Rust runtime evidence is missing")
    identity = native.get("identity")
    if not isinstance(identity, Mapping):
        raise BaselineError("accepted Desktop Rust runtime identity is invalid")
    return {
        "kind": "accepted-release-binary",
        "container": DESKTOP_RUNTIME_CONTAINER,
        "container_sha256": str(asset.get("sha256")),
        "identity_member": str(native.get("identity_member")),
        "identity_document_sha256": str(native.get("identity_document_sha256")),
        "binary_member": str(native.get("binary_member")),
        "binary_sha256": str(identity.get("sha256")),
        "binary_size_bytes": int(identity.get("size_bytes", 0)),
        "target_triple": str(identity.get("target_triple")),
    }


def _capture_rust_runtime_cases(
    *,
    tag_root: Path,
    runtime_root: Path,
    asset_dir: Path,
    runtime_source: Mapping[str, Any],
) -> list[dict[str, Any]]:
    members = _archive_member_bytes(asset_dir / DESKTOP_RUNTIME_CONTAINER)
    binary_member = str(runtime_source["binary_member"])
    binary_data = members.get(binary_member)
    if not isinstance(binary_data, bytes):
        raise BaselineError("accepted Desktop plugin omits its Rust runtime binary")
    if (
        _sha256(binary_data) != runtime_source["binary_sha256"]
        or len(binary_data) != runtime_source["binary_size_bytes"]
    ):
        raise BaselineError("accepted Desktop Rust runtime binary identity drift")
    binary_root = runtime_root / "accepted-rust-binary"
    binary_root.mkdir(parents=True)
    binary = binary_root / "qiongli-literature-provider"
    binary.write_bytes(binary_data)
    binary.chmod(
        stat.S_IRUSR
        | stat.S_IWUSR
        | stat.S_IXUSR
        | stat.S_IRGRP
        | stat.S_IXGRP
        | stat.S_IROTH
        | stat.S_IXOTH
    )
    command = [str(binary), "--transport", "stdio"]
    cases: list[dict[str, Any]] = []

    list_sandbox = runtime_root / "rust-mcp-list"
    list_sandbox.mkdir(parents=True)
    list_environment = _runtime_environment(list_sandbox)
    list_replacements = _runtime_replacements(
        tag_root=tag_root, sandbox=list_sandbox, executable=binary
    )
    completed, responses, delta = _jsonrpc_runtime(
        command,
        [
            {
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": {},
                    "clientInfo": {"name": "baseline", "version": "1"},
                },
            },
            {"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}},
        ],
        cwd=binary_root,
        environment=list_environment,
        sandbox=list_sandbox,
        guard_roots=[binary_root],
    )
    initialize = responses[0].get("result")
    listed = responses[1].get("result")
    if not isinstance(initialize, dict) or not isinstance(listed, dict):
        raise BaselineError("accepted Rust MCP initialize/list capture failed")
    tools = listed.get("tools")
    if not isinstance(tools, list):
        raise BaselineError("accepted Rust MCP tools/list returned no tools")
    cases.append(
        _runtime_case(
            case_id="rust.mcp-initialize-list",
            kind="jsonrpc-outcome",
            coverage=["mcp-initialize-and-list"],
            source_paths=[
                "packages/qiongli-lite-mcp/src/main.rs",
                "packages/qiongli-lite-mcp/src/mcp/server.rs",
                "packages/qiongli-lite-mcp/src/tools/definitions.rs",
            ],
            transport="jsonrpc-stdio",
            operation="initialize + tools/list",
            arguments={"protocol_version": "2024-11-05"},
            exit_code=completed.returncode,
            value=_normalize_runtime_value(
                {
                    "initialize": initialize,
                    "tool_names": [tool.get("name") for tool in tools if isinstance(tool, dict)],
                    "tool_count": len(tools),
                    "stderr_lines": _normalized_lines(
                        completed.stderr, list_replacements
                    ),
                },
                list_replacements,
            ),
            delta=delta,
        )
    )

    status_sandbox = runtime_root / "rust-config-status"
    status_sandbox.mkdir(parents=True)
    config_home = status_sandbox / "config"
    status_environment = _runtime_environment(
        status_sandbox, config_home=config_home
    )
    status_replacements = _runtime_replacements(
        tag_root=tag_root,
        sandbox=status_sandbox,
        config_home=config_home,
        executable=binary,
    )
    completed, responses, delta = _jsonrpc_runtime(
        command,
        [
            {
                "jsonrpc": "2.0",
                "id": 3,
                "method": "tools/call",
                "params": {"name": "qiongli_config_status", "arguments": {}},
            }
        ],
        cwd=binary_root,
        environment=status_environment,
        sandbox=status_sandbox,
        guard_roots=[binary_root],
    )
    payload = _tool_payload(responses[0])
    cases.append(
        _runtime_case(
            case_id="rust.config-status",
            kind="config-status-outcome",
            coverage=["config-status"],
            source_paths=[
                "packages/qiongli-lite-mcp/src/config/provider_config.rs",
                "packages/qiongli-lite-mcp/src/mcp/server.rs",
            ],
            transport="jsonrpc-stdio",
            operation="tools/call qiongli_config_status",
            arguments={},
            exit_code=completed.returncode,
            value=_normalize_runtime_value(
                {
                    "status": payload.get("status"),
                    "capability_mode": payload.get("capability_mode"),
                    "providers": payload.get("providers", {}),
                    "missing": payload.get("missing", []),
                    "config_path": payload.get("config_path"),
                    "stderr_lines": _normalized_lines(
                        completed.stderr, status_replacements
                    ),
                },
                status_replacements,
            ),
            delta=delta,
        )
    )

    plan_sandbox = runtime_root / "rust-search-plan"
    plan_sandbox.mkdir(parents=True)
    plan_environment = _runtime_environment(plan_sandbox)
    plan_replacements = _runtime_replacements(
        tag_root=tag_root, sandbox=plan_sandbox, executable=binary
    )
    completed, responses, delta = _jsonrpc_runtime(
        command,
        [
            {
                "jsonrpc": "2.0",
                "id": 4,
                "method": "tools/call",
                "params": {
                    "name": "qiongli_search_plan",
                    "arguments": {
                        "query": "runtime oracle",
                        "platform": "codex",
                        "native_search_available": True,
                        "native_search_tools": ["codex_web_search"],
                    },
                },
            }
        ],
        cwd=binary_root,
        environment=plan_environment,
        sandbox=plan_sandbox,
        guard_roots=[binary_root],
    )
    payload = _tool_payload(responses[0])
    plan_value = {
        key: payload.get(key)
        for key in (
            "artifact_type",
            "query",
            "platform",
            "provider_capability_mode",
            "search_execution_mode",
            "provider_queries",
            "native_search_queries",
            "limitations",
        )
    }
    cases.append(
        _runtime_case(
            case_id="rust.search-plan",
            kind="search-plan-outcome",
            coverage=["search-plan"],
            source_paths=[
                "packages/qiongli-lite-mcp/src/searchplan.rs",
                "packages/qiongli-lite-mcp/src/mcp/server.rs",
            ],
            transport="jsonrpc-stdio",
            operation="tools/call qiongli_search_plan",
            arguments={
                "query": "runtime oracle",
                "platform": "codex",
                "native_search_available": True,
                "native_search_tools": ["codex_web_search"],
            },
            exit_code=completed.returncode,
            value=_normalize_runtime_value(
                {
                    "plan": plan_value,
                    "stderr_lines": _normalized_lines(
                        completed.stderr, plan_replacements
                    ),
                },
                plan_replacements,
            ),
            delta=delta,
        )
    )

    error_sandbox = runtime_root / "rust-jsonrpc-error"
    error_sandbox.mkdir(parents=True)
    error_environment = _runtime_environment(error_sandbox)
    error_replacements = _runtime_replacements(
        tag_root=tag_root, sandbox=error_sandbox, executable=binary
    )
    completed, responses, delta = _jsonrpc_runtime(
        command,
        [
            {
                "jsonrpc": "2.0",
                "id": 5,
                "method": "tools/call",
                "params": {"name": "does_not_exist", "arguments": {}},
            }
        ],
        cwd=binary_root,
        environment=error_environment,
        sandbox=error_sandbox,
        guard_roots=[binary_root],
    )
    error = responses[0].get("error")
    if not isinstance(error, dict):
        raise BaselineError("accepted Rust MCP unknown tool did not return an error")
    cases.append(
        _runtime_case(
            case_id="rust.unknown-tool-error",
            kind="jsonrpc-error-outcome",
            coverage=["jsonrpc-error"],
            source_paths=[
                "packages/qiongli-lite-mcp/src/mcp/protocol.rs",
                "packages/qiongli-lite-mcp/src/mcp/server.rs",
            ],
            transport="jsonrpc-stdio",
            operation="tools/call unknown tool",
            arguments={"name": "does_not_exist"},
            exit_code=completed.returncode,
            value={
                "stderr_lines": _normalized_lines(
                    completed.stderr, error_replacements
                )
            },
            delta=delta,
            error={"code": int(error.get("code", -32601)), "message": str(error.get("message", ""))},
        )
    )

    redaction_sandbox = runtime_root / "rust-secret-redaction"
    redaction_sandbox.mkdir(parents=True)
    redaction_environment = _runtime_environment(
        redaction_sandbox,
        extra={"QIONGLI_MCPB_OPENALEX_API_KEY": RUNTIME_SECRET_CANARY},
    )
    redaction_replacements = _runtime_replacements(
        tag_root=tag_root, sandbox=redaction_sandbox, executable=binary
    )
    completed, responses, delta = _jsonrpc_runtime(
        command,
        [
            {
                "jsonrpc": "2.0",
                "id": 6,
                "method": "tools/call",
                "params": {"name": "qiongli_config_status", "arguments": {}},
            }
        ],
        cwd=binary_root,
        environment=redaction_environment,
        sandbox=redaction_sandbox,
        guard_roots=[binary_root],
        forbidden_secrets=[RUNTIME_SECRET_CANARY],
    )
    payload = _tool_payload(responses[0])
    providers = payload.get("providers")
    if not isinstance(providers, dict) or providers.get("openalex") != "configured":
        raise BaselineError("accepted Rust MCP did not consume the redaction canary")
    cases.append(
        _runtime_case(
            case_id="rust.secret-redaction",
            kind="redaction-outcome",
            coverage=["secret-redaction"],
            source_paths=[
                "packages/qiongli-lite-mcp/src/config/provider_config.rs",
                "packages/qiongli-lite-mcp/src/mcp/server.rs",
            ],
            transport="jsonrpc-stdio",
            operation="tools/call qiongli_config_status with provider secret",
            arguments={"QIONGLI_MCPB_OPENALEX_API_KEY": "<REDACTED>"},
            exit_code=completed.returncode,
            value=_normalize_runtime_value(
                {
                    "providers": providers,
                    "redacted_config": payload.get("redacted_config", {}),
                    "redaction_assertion": "capture secret absent from process output",
                    "stderr_lines": _normalized_lines(
                        completed.stderr,
                        redaction_replacements,
                        forbidden_secrets=[RUNTIME_SECRET_CANARY],
                    ),
                },
                redaction_replacements,
                forbidden_secrets=[RUNTIME_SECRET_CANARY],
            ),
            delta=delta,
        )
    )
    return cases


def _runtime_oracle_document(
    *,
    snapshot: TagSnapshot,
    plan: Mapping[str, Any],
    plan_oracle: Mapping[str, Any],
    runtime_source: Mapping[str, Any],
    cases: Sequence[Mapping[str, Any]],
) -> dict[str, Any]:
    excludes = [str(value) for value in plan["inventory"]["exclude_selectors"]]
    entries = _projection_entries(snapshot, plan_oracle, excludes)
    required = [str(value) for value in plan_oracle["required_coverage"]]
    return {
        "$schema": "../../../oracle-fixture.schema.json",
        "schema_version": "2.0",
        "oracle_id": str(plan_oracle["id"]),
        "runtime_origin": str(plan_oracle["runtime"]),
        "profile": str(plan_oracle["profile"]),
        "capture_kind": "accepted-runtime-outcomes",
        "source": {
            "access_method": "git-ls-tree-and-cat-file",
            "peeled_commit": snapshot.peeled_commit,
            "projections": [entry.projection_record() for entry in entries],
            "runtime_source": dict(runtime_source),
            "tag": snapshot.tag,
        },
        "coverage": {
            "required_capabilities": required,
            "captured_capabilities": required,
            "accepted_gaps": json.loads(json.dumps(plan_oracle["accepted_gaps"])),
        },
        "cases": sorted(
            [json.loads(json.dumps(case)) for case in cases],
            key=lambda value: value["id"],
        ),
    }


def _expected_runtime_source(
    *,
    snapshot: TagSnapshot,
    oracle_id: str,
    release_assets: Sequence[Mapping[str, Any]],
    native_identities: Sequence[Mapping[str, Any]],
) -> dict[str, Any]:
    if oracle_id == "python-full":
        return _runtime_source_tree(snapshot, "packages/python-qiongli")
    if oracle_id == "node-mcpb":
        return _runtime_source_tree(snapshot, "packages/qiongli-literature-mcpb")
    if oracle_id == "rust-lite":
        return _accepted_rust_runtime_source(release_assets, native_identities)
    raise BaselineError(f"unsupported runtime oracle id: {oracle_id}")


def _validate_runtime_oracle_documents(
    *,
    snapshot: TagSnapshot,
    plan: Mapping[str, Any],
    documents: Mapping[str, Mapping[str, Any]],
    release_assets: Sequence[Mapping[str, Any]],
    native_identities: Sequence[Mapping[str, Any]],
) -> None:
    expected_paths = {
        "oracles/python-full.json",
        "oracles/rust-lite.json",
        "oracles/node-mcpb.json",
    }
    if set(documents) != {
        "oracles/python-full.json",
        "oracles/rust-lite.json",
        "oracles/node-mcpb.json",
    }:
        raise BaselineError("capture must contain exactly three named runtime oracles")
    excludes = [str(value) for value in plan["inventory"]["exclude_selectors"]]
    plan_by_id = {str(oracle["id"]): oracle for oracle in plan["oracles"]}
    for path in sorted(expected_paths):
        document = documents[path]
        oracle_id = path.removeprefix("oracles/").removesuffix(".json")
        plan_oracle = plan_by_id.get(oracle_id)
        if not isinstance(plan_oracle, Mapping):
            raise BaselineError(f"runtime oracle is absent from the plan: {oracle_id}")
        if (
            document.get("oracle_id") != oracle_id
            or document.get("runtime_origin") != plan_oracle["runtime"]
            or document.get("profile") != plan_oracle["profile"]
            or document.get("capture_kind") != "accepted-runtime-outcomes"
            or document.get("schema_version") != "2.0"
        ):
            raise BaselineError(f"runtime oracle metadata drift: {oracle_id}")
        source = document.get("source")
        if not isinstance(source, Mapping):
            raise BaselineError(f"runtime oracle source is invalid: {oracle_id}")
        entries = _projection_entries(snapshot, plan_oracle, excludes)
        expected_projections = [entry.projection_record() for entry in entries]
        if source.get("projections") != expected_projections:
            raise BaselineError(f"runtime oracle source projection drift: {oracle_id}")
        expected_source = _expected_runtime_source(
            snapshot=snapshot,
            oracle_id=oracle_id,
            release_assets=release_assets,
            native_identities=native_identities,
        )
        if source.get("runtime_source") != expected_source:
            raise BaselineError(f"runtime oracle implementation identity drift: {oracle_id}")
        if (
            source.get("tag") != snapshot.tag
            or source.get("peeled_commit") != snapshot.peeled_commit
            or source.get("access_method") != "git-ls-tree-and-cat-file"
        ):
            raise BaselineError(f"runtime oracle tag identity drift: {oracle_id}")

        coverage = document.get("coverage")
        cases = document.get("cases")
        if not isinstance(coverage, Mapping) or not isinstance(cases, list) or not cases:
            raise BaselineError(f"runtime oracle coverage/cases are invalid: {oracle_id}")
        required = [str(value) for value in plan_oracle["required_coverage"]]
        if (
            coverage.get("required_capabilities") != required
            or coverage.get("captured_capabilities") != required
            or coverage.get("accepted_gaps") != plan_oracle["accepted_gaps"]
        ):
            raise BaselineError(f"runtime oracle coverage drift: {oracle_id}")
        case_ids: set[str] = set()
        captured: set[str] = set()
        projection_paths = {entry.path for entry in entries}
        for case in cases:
            if not isinstance(case, Mapping):
                raise BaselineError(f"runtime oracle case is invalid: {oracle_id}")
            case_id = case.get("id")
            if not isinstance(case_id, str) or not case_id or case_id in case_ids:
                raise BaselineError(f"runtime oracle case id drift: {oracle_id}")
            case_ids.add(case_id)
            case_coverage = case.get("coverage")
            if not isinstance(case_coverage, list) or not case_coverage:
                raise BaselineError(f"runtime oracle case has no coverage: {case_id}")
            captured.update(str(value) for value in case_coverage)
            if not set(case_coverage).issubset(set(required)):
                raise BaselineError(f"runtime oracle case exceeds planned coverage: {case_id}")
            source_paths = case.get("source_paths")
            if not isinstance(source_paths, list) or not set(source_paths).issubset(
                projection_paths
            ):
                raise BaselineError(f"runtime oracle case source drift: {case_id}")
            outcome = case.get("outcome")
            side_effects = case.get("side_effects")
            if not isinstance(outcome, Mapping) or not isinstance(side_effects, Mapping):
                raise BaselineError(f"runtime oracle case evidence is invalid: {case_id}")
            error = outcome.get("error")
            if (outcome.get("status") == "error") != isinstance(error, Mapping):
                raise BaselineError(f"runtime oracle case error disposition drift: {case_id}")
            delta = side_effects.get("filesystem_delta")
            if not isinstance(delta, Mapping):
                raise BaselineError(f"runtime oracle filesystem delta is missing: {case_id}")
            changed = any(delta.get(name) for name in ("created", "modified", "deleted"))
            expected_class = "bounded-write" if changed else "none"
            if (
                side_effects.get("class") != expected_class
                or side_effects.get("writes_outside_sandbox") is not False
            ):
                raise BaselineError(f"runtime oracle side-effect disposition drift: {case_id}")
            if not changed and delta.get("before_tree_sha256") != delta.get(
                "after_tree_sha256"
            ):
                raise BaselineError(f"runtime oracle no-write tree hash drift: {case_id}")
        if captured != set(required):
            raise BaselineError(f"runtime oracle required coverage is incomplete: {oracle_id}")
        _scan_portability(document)


def _oracle_documents(
    snapshot: TagSnapshot,
    plan: Mapping[str, Any],
    *,
    repo_root: Path,
    asset_dir: Path,
    release_assets: Sequence[Mapping[str, Any]],
    native_identities: Sequence[Mapping[str, Any]],
) -> dict[str, dict[str, Any]]:
    with tempfile.TemporaryDirectory(prefix="qiongli-runtime-oracle-") as temp_name:
        runtime_root = Path(temp_name)
        tag_root = runtime_root / "accepted-tag"
        _materialize_tag_snapshot(snapshot, tag_root)
        plan_by_id = {str(oracle["id"]): oracle for oracle in plan["oracles"]}
        python_cases = _capture_python_runtime_cases(
            repo_root=repo_root,
            tag_root=tag_root,
            runtime_root=runtime_root,
        )
        node_cases = _capture_node_runtime_cases(
            tag_root=tag_root,
            runtime_root=runtime_root,
        )
        rust_source = _accepted_rust_runtime_source(
            release_assets, native_identities
        )
        rust_cases = _capture_rust_runtime_cases(
            tag_root=tag_root,
            runtime_root=runtime_root,
            asset_dir=asset_dir,
            runtime_source=rust_source,
        )
        documents = {
            "oracles/python-full.json": _runtime_oracle_document(
                snapshot=snapshot,
                plan=plan,
                plan_oracle=plan_by_id["python-full"],
                runtime_source=_runtime_source_tree(
                    snapshot, "packages/python-qiongli"
                ),
                cases=python_cases,
            ),
            "oracles/node-mcpb.json": _runtime_oracle_document(
                snapshot=snapshot,
                plan=plan,
                plan_oracle=plan_by_id["node-mcpb"],
                runtime_source=_runtime_source_tree(
                    snapshot, "packages/qiongli-literature-mcpb"
                ),
                cases=node_cases,
            ),
            "oracles/rust-lite.json": _runtime_oracle_document(
                snapshot=snapshot,
                plan=plan,
                plan_oracle=plan_by_id["rust-lite"],
                runtime_source=rust_source,
                cases=rust_cases,
            ),
        }
    _validate_runtime_oracle_documents(
        snapshot=snapshot,
        plan=plan,
        documents=documents,
        release_assets=release_assets,
        native_identities=native_identities,
    )
    return documents


def _scan_portability(value: Any, *, path: str = "$") -> None:
    if isinstance(value, dict):
        for key, item in value.items():
            _scan_portability(str(key), path=f"{path}.<key>")
            _scan_portability(item, path=f"{path}.{key}")
        return
    if isinstance(value, list):
        for index, item in enumerate(value):
            _scan_portability(item, path=f"{path}[{index}]")
        return
    if not isinstance(value, str):
        return
    if MACHINE_PATH_PATTERN.search(value):
        raise BaselineError(f"machine-local path leaked into baseline at {path}")
    if TIMESTAMP_PATTERN.search(value):
        raise BaselineError(f"wall-clock timestamp leaked into baseline at {path}")
    if PROCESS_ID_PATTERN.search(value):
        raise BaselineError(f"process identifier leaked into baseline at {path}")
    for pattern in SECRET_PATTERNS:
        if pattern.search(value):
            raise BaselineError(f"secret-shaped value leaked into baseline at {path}")


def _oracle_descriptor(path: str, document: Mapping[str, Any]) -> dict[str, Any]:
    data = _canonical_bytes(document)
    return {
        "accepted_gap_count": len(document["coverage"]["accepted_gaps"]),
        "case_count": len(document["cases"]),
        "captured_capabilities": list(
            document["coverage"]["captured_capabilities"]
        ),
        "oracle_id": str(document["oracle_id"]),
        "path": path,
        "runtime_source_kind": str(
            document["source"]["runtime_source"]["kind"]
        ),
        "sha256": _sha256(data),
        "size_bytes": len(data),
    }


def _corpus_payload(manifest: Mapping[str, Any]) -> dict[str, Any]:
    return {
        key: manifest[key]
        for key in (
            "source",
            "acceptance_receipt",
            "selector_semantics",
            "domains",
            "package_trees",
            "release_assets",
            "native_identities",
            "oracle_fixtures",
        )
    }


def build_capture(
    *,
    repo_root: Path = REPO_ROOT,
    asset_dir: Path | None = None,
    recorded_release_evidence: tuple[
        Sequence[Mapping[str, Any]], Sequence[Mapping[str, Any]]
    ]
    | None = None,
    recorded_oracle_documents: Mapping[str, Mapping[str, Any]] | None = None,
) -> dict[str, bytes]:
    repo_root = repo_root.resolve()
    plan = _read_plan(repo_root)
    tag = str(plan["release_lineage"]["accepted_tag"])
    expected_commit = str(plan["release_lineage"]["accepted_commit"])
    snapshot = TagSnapshot(repo_root, tag, expected_commit)
    receipt, accepted_assets, accepted_native = _receipt_record(repo_root, plan)
    domains = _domain_records(snapshot, plan)
    package_trees = _package_tree_records(snapshot, plan)
    if recorded_release_evidence is None:
        if asset_dir is None:
            raise BaselineError("capture requires an explicit release asset directory")
        release_assets, native_identities = _release_records(
            plan,
            asset_dir.resolve(),
            accepted_assets,
            accepted_native,
        )
    else:
        release_assets = json.loads(json.dumps(recorded_release_evidence[0]))
        native_identities = json.loads(json.dumps(recorded_release_evidence[1]))
        _validate_release_evidence(
            plan,
            release_assets,
            native_identities,
            accepted_assets=accepted_assets,
            accepted_native=accepted_native,
        )
    if recorded_oracle_documents is None:
        if recorded_release_evidence is not None:
            raise BaselineError(
                "offline replay requires recorded runtime oracle documents"
            )
        if asset_dir is None:
            raise BaselineError("runtime oracle capture requires release assets")
        oracle_documents = _oracle_documents(
            snapshot,
            plan,
            repo_root=repo_root,
            asset_dir=asset_dir.resolve(),
            release_assets=release_assets,
            native_identities=native_identities,
        )
    else:
        if recorded_release_evidence is None:
            raise BaselineError(
                "recorded runtime oracle documents require recorded release evidence"
            )
        oracle_documents = json.loads(json.dumps(recorded_oracle_documents))
        _validate_runtime_oracle_documents(
            snapshot=snapshot,
            plan=plan,
            documents=oracle_documents,
            release_assets=release_assets,
            native_identities=native_identities,
        )
    oracle_fixtures = [
        _oracle_descriptor(path, oracle_documents[path]) for path in sorted(oracle_documents)
    ]
    source = {
        "peeled_commit": snapshot.peeled_commit,
        "tag": snapshot.tag,
        "tag_object_oid": snapshot.tag_object_oid,
        "tag_type": "annotated",
        "tree_access": "git-ls-tree-and-cat-file",
    }
    manifest: dict[str, Any] = {
        "$schema": "../../baseline-manifest.schema.json",
        "acceptance_receipt": receipt,
        "domains": domains,
        "format_id": str(plan["output"]["format_id"]),
        "native_identities": native_identities,
        "oracle_fixtures": oracle_fixtures,
        "package_trees": package_trees,
        "release_assets": release_assets,
        "schema_version": "1.0",
        "selector_semantics": plan["selector_semantics"],
        "source": source,
    }
    manifest["integrity"] = {
        "algorithm": "sha256",
        "corpus_definition": CORPUS_DEFINITION,
        "corpus_sha256": _sha256(_compact_canonical_bytes(_corpus_payload(manifest))),
    }
    documents: dict[str, bytes] = {
        path: _canonical_bytes(document) for path, document in oracle_documents.items()
    }
    documents["manifest.json"] = _canonical_bytes(manifest)
    _validate_capture_documents(repo_root, documents)
    return documents


def _validate_capture_documents(repo_root: Path, documents: Mapping[str, bytes]) -> None:
    manifest_schema = _load_json_file(repo_root / "tooling/migration/baseline-manifest.schema.json")
    oracle_schema = _load_json_file(repo_root / "tooling/migration/oracle-fixture.schema.json")
    expected_paths = {
        "manifest.json",
        "oracles/python-full.json",
        "oracles/rust-lite.json",
        "oracles/node-mcpb.json",
    }
    if set(documents) != expected_paths:
        raise BaselineError(
            f"baseline output file set drift: expected={sorted(expected_paths)}, "
            f"actual={sorted(documents)}"
        )
    parsed = {
        path: _load_json_bytes(data, label=path) for path, data in documents.items()
    }
    manifest = parsed["manifest.json"]
    failures = validate_instance(manifest, manifest_schema)
    if failures:
        raise BaselineError("manifest schema validation failed:\n- " + "\n- ".join(failures))
    for path in sorted(expected_paths - {"manifest.json"}):
        failures = validate_instance(parsed[path], oracle_schema)
        if failures:
            raise BaselineError(f"{path} schema validation failed:\n- " + "\n- ".join(failures))
        if not parsed[path]["cases"]:
            raise BaselineError(f"oracle fixture is empty: {path}")
    descriptors = {item["path"]: item for item in manifest["oracle_fixtures"]}
    for path in sorted(expected_paths - {"manifest.json"}):
        descriptor = descriptors.get(path)
        if not isinstance(descriptor, dict):
            raise BaselineError(f"manifest omits oracle fixture descriptor: {path}")
        data = documents[path]
        if descriptor["sha256"] != _sha256(data) or descriptor["size_bytes"] != len(data):
            raise BaselineError(f"manifest oracle fixture hash/size drift: {path}")
        if descriptor["case_count"] != len(parsed[path]["cases"]):
            raise BaselineError(f"manifest oracle fixture case-count drift: {path}")
        oracle = parsed[path]
        if descriptor["oracle_id"] != oracle["oracle_id"]:
            raise BaselineError(f"manifest oracle fixture id drift: {path}")
        if descriptor["accepted_gap_count"] != len(
            oracle["coverage"]["accepted_gaps"]
        ):
            raise BaselineError(f"manifest oracle fixture accepted-gap drift: {path}")
        if descriptor["captured_capabilities"] != oracle["coverage"][
            "captured_capabilities"
        ]:
            raise BaselineError(f"manifest oracle fixture coverage drift: {path}")
        if descriptor["runtime_source_kind"] != oracle["source"][
            "runtime_source"
        ]["kind"]:
            raise BaselineError(f"manifest oracle fixture runtime-source drift: {path}")
    expected_corpus = _sha256(_compact_canonical_bytes(_corpus_payload(manifest)))
    if manifest["integrity"]["corpus_sha256"] != expected_corpus:
        raise BaselineError("manifest corpus digest drift")
    _scan_portability(parsed)


def _assert_no_symlink_descendant(path: Path, *, anchor: Path) -> None:
    if not path.is_relative_to(anchor):
        raise BaselineError(f"baseline output escapes its trusted root: {path}")
    current = anchor
    for part in path.relative_to(anchor).parts:
        current /= part
        try:
            metadata = os.lstat(current)
        except FileNotFoundError:
            break
        if stat.S_ISLNK(metadata.st_mode):
            raise BaselineError(f"baseline output path contains a symlink: {current}")


def _safe_output_dir(output_dir: Path, *, repo_root: Path = REPO_ROOT) -> Path:
    lexical = Path(os.path.abspath(output_dir))
    resolved = output_dir.resolve()
    repository = repo_root.resolve()
    canonical = repository / CANONICAL_OUTPUT_RELATIVE
    if resolved == Path(resolved.anchor) or resolved == repository:
        raise BaselineError(f"refusing unsafe baseline output directory: {resolved}")
    if output_dir.is_symlink():
        raise BaselineError(f"baseline output directory must not be a symlink: {output_dir}")
    if lexical.is_relative_to(repository):
        if lexical != canonical:
            raise BaselineError(
                "repository baseline output must be the canonical versioned directory: "
                f"{canonical}"
            )
        _assert_no_symlink_descendant(lexical, anchor=repository)
        if resolved != canonical or not resolved.is_relative_to(repository):
            raise BaselineError(
                "canonical baseline output must resolve inside the repository without aliases"
            )
        return resolved
    if resolved.is_relative_to(repository):
        raise BaselineError(
            "repository baseline output must use the canonical lexical path: "
            f"{canonical}"
        )

    temporary_roots = {
        Path(os.path.abspath(tempfile.gettempdir())),
        Path(os.path.abspath("/tmp")),
        Path(os.path.abspath("/private/tmp")),
    }
    trusted_root = next(
        (
            root
            for root in sorted(temporary_roots, key=lambda item: len(item.parts), reverse=True)
            if lexical.is_relative_to(root)
            and len(lexical.relative_to(root).parts) >= 2
            and resolved.is_relative_to(root.resolve())
        ),
        None,
    )
    if trusted_root is None:
        raise BaselineError(
            "non-repository baseline output must be nested under a temporary root"
        )
    _assert_no_symlink_descendant(lexical, anchor=trusted_root)
    return resolved


def _assert_replaceable_output(path: Path) -> None:
    if not path.exists():
        return
    if path.is_symlink() or not path.is_dir():
        raise BaselineError(f"baseline output must be a real directory: {path}")
    entries = list(path.iterdir())
    if not entries:
        return
    manifest_path = path / "manifest.json"
    if manifest_path.is_symlink() or not manifest_path.is_file():
        raise BaselineError(
            f"refusing to replace non-baseline non-empty directory: {path}"
        )
    manifest = _load_json_file(manifest_path)
    source = manifest.get("source") if isinstance(manifest, dict) else None
    if (
        not isinstance(source, dict)
        or manifest.get("format_id") != "qiongli-migration-baseline-v1"
        or source.get("tag") != "v1.19.0-beta.1"
        or source.get("peeled_commit")
        != "8d2e99866ce4c4efb8b3b5e0265c0c1f89a36b0f"
    ):
        raise BaselineError(f"refusing to replace unrecognized baseline directory: {path}")


def write_capture(
    output_dir: Path,
    documents: Mapping[str, bytes],
    *,
    repo_root: Path = REPO_ROOT,
) -> None:
    output_dir = _safe_output_dir(output_dir, repo_root=repo_root)
    _assert_replaceable_output(output_dir)
    output_dir.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(
        dir=output_dir.parent, prefix=f".{output_dir.name}.capture-"
    ) as temp_name:
        temp_root = Path(temp_name)
        for relative, data in sorted(documents.items()):
            _validate_relative_path(relative, label="baseline output path")
            destination = temp_root / relative
            destination.parent.mkdir(parents=True, exist_ok=True)
            destination.write_bytes(data)
        staged = output_dir.parent / f".{output_dir.name}.ready-{os.getpid()}"
        if staged.exists():
            _assert_replaceable_output(staged)
            shutil.rmtree(staged)
        os.replace(temp_root, staged)
        if output_dir.exists():
            _assert_replaceable_output(output_dir)
            shutil.rmtree(output_dir)
        os.replace(staged, output_dir)


def _read_output_documents(
    output_dir: Path, *, repo_root: Path = REPO_ROOT
) -> dict[str, bytes]:
    output_dir = _safe_output_dir(output_dir, repo_root=repo_root)
    if not output_dir.is_dir():
        raise BaselineError(f"baseline output directory does not exist: {output_dir}")
    documents: dict[str, bytes] = {}
    for path in sorted(output_dir.rglob("*")):
        if path.is_symlink():
            raise BaselineError(f"baseline output contains a symlink: {path}")
        if path.is_dir():
            continue
        relative = path.relative_to(output_dir).as_posix()
        _validate_relative_path(relative, label="baseline output path")
        documents[relative] = path.read_bytes()
    return documents


def verify_capture(
    *, output_dir: Path, repo_root: Path = REPO_ROOT
) -> None:
    actual = _read_output_documents(output_dir, repo_root=repo_root)
    _validate_capture_documents(repo_root.resolve(), actual)
    manifest = _load_json_bytes(actual["manifest.json"], label="manifest.json")
    recorded_oracles = {
        path: _load_json_bytes(actual[path], label=path)
        for path in (
            "oracles/python-full.json",
            "oracles/rust-lite.json",
            "oracles/node-mcpb.json",
        )
    }
    expected = build_capture(
        repo_root=repo_root,
        recorded_release_evidence=(
            manifest["release_assets"],
            manifest["native_identities"],
        ),
        recorded_oracle_documents=recorded_oracles,
    )
    if set(actual) != set(expected):
        raise BaselineError(
            f"baseline output file set drift: expected={sorted(expected)}, actual={sorted(actual)}"
        )
    drift = [path for path in sorted(expected) if actual[path] != expected[path]]
    if drift:
        raise BaselineError(f"baseline byte drift: {', '.join(drift)}")


def _default_output_dir(repo_root: Path, plan: Mapping[str, Any]) -> Path:
    manifest = Path(str(plan["output"]["manifest_path"]))
    return repo_root / manifest.parent


def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Capture or verify the deterministic Qiongli 1.x migration baseline."
    )
    parser.add_argument("--repo-root", type=Path, default=REPO_ROOT)
    subparsers = parser.add_subparsers(dest="command", required=True)
    for command in ("capture", "verify"):
        help_text = (
            "Capture or replay runtime outcomes from accepted release assets."
            if command == "capture"
            else "Validate offline structure and source integrity without rerunning runtimes."
        )
        child = subparsers.add_parser(
            command,
            help=help_text,
            description=help_text,
        )
        child.add_argument("--output-dir", type=Path)
        if command == "capture":
            child.add_argument("--asset-dir", type=Path, required=True)
            child.add_argument(
                "--check",
                action="store_true",
                help="Recompute and compare without writing generated files.",
            )
    return parser


def main(argv: Sequence[str] | None = None) -> int:
    args = _parser().parse_args(argv)
    try:
        repo_root = args.repo_root.resolve()
        plan = _read_plan(repo_root)
        output_dir = args.output_dir or _default_output_dir(repo_root, plan)
        if args.command == "capture" and not args.check:
            documents = build_capture(repo_root=repo_root, asset_dir=args.asset_dir)
            write_capture(output_dir, documents, repo_root=repo_root)
            print(f"captured deterministic migration baseline: {output_dir}")
        else:
            if args.command == "capture":
                verify_capture(output_dir=output_dir, repo_root=repo_root)
                expected = build_capture(repo_root=repo_root, asset_dir=args.asset_dir)
                actual = _read_output_documents(output_dir, repo_root=repo_root)
                drift = [
                    path
                    for path in sorted(expected)
                    if path not in actual or actual[path] != expected[path]
                ]
                if set(actual) != set(expected) or drift:
                    raise BaselineError(
                        "capture --check differs from release-asset recapture: "
                        + ", ".join(drift or sorted(set(actual) ^ set(expected)))
                    )
            else:
                verify_capture(output_dir=output_dir, repo_root=repo_root)
            verb = "capture --check" if args.command == "capture" else "verify"
            print(f"migration baseline {verb} passed: {output_dir}")
        return 0
    except BaselineError as exc:
        print(f"migration baseline error: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
