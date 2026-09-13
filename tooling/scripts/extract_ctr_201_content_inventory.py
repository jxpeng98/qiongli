#!/usr/bin/env python3
from __future__ import annotations

import argparse
import ast
import hashlib
import io
import json
import os
from pathlib import Path, PurePosixPath
import re
import shutil
import stat
import subprocess
import sys
import tempfile
import unicodedata
from typing import Any, Mapping, Sequence

try:
    import yaml
except ModuleNotFoundError:  # pragma: no cover - reported by _require_toolchain
    yaml = None  # type: ignore[assignment]


REPO_ROOT = Path(__file__).resolve().parents[2]
MANIFEST_RELATIVE = "tooling/migration/baselines/v1.19.0-beta.1/manifest.json"
MANIFEST_SHA256 = "77bb7628d43a496c995e4b0a8daf6a624847b62e96948c0461affe89002da131"
DEFAULT_OUTPUT_RELATIVE = "tooling/migration/ctr-201-content.json"
DEFAULT_SCHEMA_RELATIVE = "tooling/migration/ctr-201-content.schema.json"
ACCEPTED_TAG = "v1.19.0-beta.1"
ACCEPTED_COMMIT = "8d2e99866ce4c4efb8b3b5e0265c0c1f89a36b0f"
ACCEPTED_TAG_OBJECT = "e68e3af4c879d8e9053124d1aed625bfcddfdbb4"
EXPECTED_CONTENT_FILE_COUNT = 377
EXPECTED_CONTENT_TOTAL_BYTES = 1_761_400
EXPECTED_CONTENT_TREE_SHA256 = (
    "4659cbcd839c3f8eb3798a64981b7ec2180cf766566fcf439ac892eb32a8a5a8"
)
ARTIFACT_SCHEMA = "./ctr-201-content.schema.json"
SCHEMA_VERSION = "1.0"
RECORD_TYPE = "qiongli-ctr-201-content-materialization-inventory"
CANONICALIZATION = "utf-8-json-sorted-keys-compact-excluding-integrity"
TREE_CANONICALIZATION = (
    "sha256-of-utf8-compact-sorted-key-json-over-utf8-byte-sorted-"
    "path-mode-size_bytes-sha256-entry-array"
)
MAX_BLOB_BYTES = 8 * 1024 * 1024
MAX_TREE_BYTES = 16 * 1024 * 1024
MAX_TREE_ENTRIES = 2_000
MAX_PATH_BYTES = 512
MAX_PATH_DEPTH = 32


RESOURCE_ROOTS: tuple[Mapping[str, Any], ...] = (
    {
        "source": "content/distribution/",
        "match": "prefix",
        "resource_kind": "target-metadata",
        "file_count": 3,
        "total_bytes": 10_246,
        "entries_sha256": "e8e1095cb67ed518a64c7fa62d4d79ba2fcb6f27a6006920c4d4bac4883bc316",
    },
    {
        "source": "content/mcp-contracts/",
        "match": "prefix",
        "resource_kind": "mcp-contract",
        "file_count": 28,
        "total_bytes": 74_579,
        "entries_sha256": "3351a56914e76c184a03ee9f7e9344dfa435b2e316468d6b4757e79b6dc67e45",
    },
    {
        "source": "content/roles/",
        "match": "prefix",
        "resource_kind": "role",
        "file_count": 10,
        "total_bytes": 13_570,
        "entries_sha256": "910c465e0cb213f5834aedf70475d0d9f9a17caf5e4a46db70d46b40f26015c6",
    },
    {
        "source": "content/schemas/",
        "match": "prefix",
        "resource_kind": "schema",
        "file_count": 5,
        "total_bytes": 49_270,
        "entries_sha256": "f65399010efd7c09f930e7a4d42c646b967a183ed69f7b01615f71f5084b1867",
    },
    {
        "source": "content/skills/",
        "match": "prefix",
        "resource_kind": "skill",
        "file_count": 97,
        "total_bytes": 881_422,
        "entries_sha256": "00b1f3192405a53c3df89163eec42f8a847c8ce5a9b0b2578d6a7ddbe4fab5ed",
    },
    {
        "source": "content/skills-core.md",
        "match": "exact",
        "resource_kind": "skill-summary",
        "file_count": 1,
        "total_bytes": 25_853,
        "entries_sha256": "24876a1ef568d12a50a4965047a730fbc5b6d6281f2441fa91cd8762ecce64e7",
    },
    {
        "source": "content/skills-summary.md",
        "match": "exact",
        "resource_kind": "skill-summary",
        "file_count": 1,
        "total_bytes": 6_539,
        "entries_sha256": "1ee42f0f4214c2ce044c3281dd52b96e41eb98b2e43811dd0aaf55c3ab038815",
    },
    {
        "source": "content/standards/",
        "match": "prefix",
        "resource_kind": "standard",
        "file_count": 11,
        "total_bytes": 139_835,
        "entries_sha256": "8480cfd1db0010208a87f5e93a87c71245b60b9c5cdeba341c6ade794b2df5a8",
    },
    {
        "source": "content/subjects/",
        "match": "prefix",
        "resource_kind": "subject",
        "file_count": 77,
        "total_bytes": 182_320,
        "entries_sha256": "7e6b9f511bf35042843ecd7b372feefa6ce8c3617e7d96b49e5f07b95b1be8d3",
    },
    {
        "source": "content/templates/",
        "match": "prefix",
        "resource_kind": "template",
        "file_count": 92,
        "total_bytes": 148_824,
        "entries_sha256": "61faed4829ffebd767862bd25d0d17bfdb2f06f2a9ee60c1c7810b0824c8a9a1",
    },
    {
        "source": "content/venue-profiles/",
        "match": "prefix",
        "resource_kind": "venue-profile",
        "file_count": 6,
        "total_bytes": 4_720,
        "entries_sha256": "d6e698377329d6417f1282c4ab7b53669bbde9a15e753d83e18c6f3911e18808",
    },
    {
        "source": "content/workflow/",
        "match": "prefix",
        "resource_kind": "workflow",
        "file_count": 46,
        "total_bytes": 224_222,
        "entries_sha256": "92f24659d60e8f4e40a468c3e98ed707b7dd0a403127b473c0c9d2fd6238aca6",
    },
)

RESOURCE_KIND_ORDER = (
    "target-metadata",
    "mcp-contract",
    "role",
    "schema",
    "skill",
    "skill-summary",
    "standard",
    "subject",
    "template",
    "venue-profile",
    "workflow",
)
SKILL_SOURCE_KINDS = (
    "role",
    "skill",
    "skill-summary",
    "standard",
    "subject",
    "template",
    "venue-profile",
    "workflow",
)

SOURCE_ANCHORS: tuple[Mapping[str, Any], ...] = (
    {
        "role": "accepted-python-package-init",
        "path": "packages/python-qiongli/src/qiongli/__init__.py",
        "mode": "100644",
        "git_blob_oid": "205d5ce54341f4cc78222275ed63f9b371a8d477",
        "sha256": "1f26e8f8063e4f54db80e2039cec2a488e69e44a6be5b5dadde5ea7d30a4745d",
        "size_bytes": 88,
    },
    {
        "role": "accepted-source-layout",
        "path": "packages/python-qiongli/src/qiongli/source_layout.py",
        "mode": "100644",
        "git_blob_oid": "6195f93f6597c7a546a82178790d79f571bda6f0",
        "sha256": "7ba7912744b0076d448323fb5d7122c368315dea97ea766000fc5802f9e9b0fe",
        "size_bytes": 7_492,
    },
    {
        "role": "accepted-subject-materializer",
        "path": "packages/python-qiongli/src/qiongli/subject_materializer.py",
        "mode": "100644",
        "git_blob_oid": "6c4821894c26aa336a1a5e2ef797ffd218296a30",
        "sha256": "7fa4e6b6591b489bc445ae97787474e0134a8fa748c563603f4d37d3be0a1470",
        "size_bytes": 47_416,
    },
    {
        "role": "accepted-plugin-materializer-policy",
        "path": "tooling/scripts/build_plugin_artifacts.py",
        "mode": "100644",
        "git_blob_oid": "0a18a083dfec73e8bddc13f961d9a70c1260e6e2",
        "sha256": "59563be3b4e3ee33ccb1bf66980639c3603291aa84a21f3edd9c484cf4aa85bb",
        "size_bytes": 59_409,
    },
    {
        "role": "accepted-portable-payload-policy",
        "path": "tooling/scripts/sync_npm_package_payload.py",
        "mode": "100755",
        "git_blob_oid": "df209ada7aaa5b3b857985298c5a3c3d67687210",
        "sha256": "db086a4a9b4ce66711b68de67612b510139f5eb71878df8fd3a0f69edcb41d71",
        "size_bytes": 13_090,
    },
    {
        "role": "accepted-full-installer-policy",
        "path": "packages/python-qiongli/src/qiongli/universal_installer.py",
        "mode": "100644",
        "git_blob_oid": "c68d9087d7d32a55b123df67a352a74c231c58df",
        "sha256": "e098bc896e962d6990e830d67216c017e5f747e8fa89b47b8eadf0ab670b7a9a",
        "size_bytes": 60_333,
    },
    {
        "role": "accepted-local-plugin-policy",
        "path": "packages/python-qiongli/src/qiongli/local_plugin_installer.py",
        "mode": "100644",
        "git_blob_oid": "f996649a87ffed54ead09dc527efda024983bacd",
        "sha256": "193ef75364def009afa1158115907b715f3e48519048cd051c3f97654eea40a7",
        "size_bytes": 26_406,
    },
    {
        "role": "accepted-python-lock",
        "path": "uv.lock",
        "mode": "100644",
        "git_blob_oid": "2460232d3ca1f3fa206dc673f608966df1b30650",
        "sha256": "1edf91752bb8eece7acb1322ce0ba1f4ebba72681bbe8f1a4aa275f61df15352",
        "size_bytes": 12_949,
    },
    {
        "role": "accepted-python-project",
        "path": "pyproject.toml",
        "mode": "100644",
        "git_blob_oid": "4fc00c6a21b5c7e8a9ffc1ac58698b9d2bd087a5",
        "sha256": "3001a8c7e6002e6fca928a7748dbb556e95c3cb6b9cc864a0a8d4294638f5ebf",
        "size_bytes": 1_315,
    },
    {
        "role": "accepted-install-manifest",
        "path": "tooling/install/install_manifest.tsv",
        "mode": "100644",
        "git_blob_oid": "7dcc10771543bb080f6c83de81c5bbba14db2755",
        "sha256": "6f43e4b878a4139967bedac093d30f9ac0359f9cd849b8ac051bbd674940ec11",
        "size_bytes": 417,
    },
)

PROFILE_FACTS: tuple[Mapping[str, Any], ...] = (
    {
        "profile_id": "skill-only",
        "aliases": [],
        "variant_id": "qiongli-next-prerelease-core-desktop-focused",
        "subject": "core",
        "flavor": "desktop",
        "coverage": "focused",
        "skill_name": "qiongli-next",
        "worker_plan": "skill-only",
        "source_file_count": 341,
        "source_total_bytes": 1_627_305,
        "source_tree_sha256": "2283d6f5d284dde43225c5fb194e2e714b5e7b34e9c9bb97e753914d968acf26",
        "included_resource_kinds": list(SKILL_SOURCE_KINDS),
        "file_count": 178,
        "total_bytes": 708_608,
        "tree_sha256": "5b76bc0c02cda7fc18adf2b1afd492e763392ed5fc2a05dac360d1221045f280",
        "origin_counts": {"identity-copy": 173, "content-transform": 2, "generated-metadata": 3},
        "pipeline": [
            "stage-plugin-source-excluding-templates/CLAUDE.project.md",
            "materialize-subject:core:desktop:focused",
            "rewrite-skill-entrypoint:qiongli-next",
        ],
    },
    {
        "profile_id": "marketplace-lite",
        "aliases": ["lite"],
        "variant_id": "qiongli-next-prerelease-core-full-complete",
        "subject": "core",
        "flavor": "full",
        "coverage": "complete",
        "skill_name": "qiongli-next",
        "worker_plan": "marketplace-lite",
        "source_file_count": 377,
        "source_total_bytes": EXPECTED_CONTENT_TOTAL_BYTES,
        "source_tree_sha256": EXPECTED_CONTENT_TREE_SHA256,
        "included_resource_kinds": list(RESOURCE_KIND_ORDER),
        "file_count": 342,
        "total_bytes": 1_600_064,
        "tree_sha256": "a854fc61203883132041a43077cc9ea26e62aa28e2c2eeb266777f582b029c6c",
        "origin_counts": {"identity-copy": 338, "content-transform": 2, "generated-metadata": 2},
        "pipeline": [
            "stage-plugin-source-excluding-templates/CLAUDE.project.md",
            "materialize-subject:core:full:complete",
            "rewrite-skill-entrypoint:qiongli-next",
        ],
    },
    {
        "profile_id": "full",
        "aliases": [],
        "variant_id": "qiongli-local-core-full-complete",
        "subject": "core",
        "flavor": "full",
        "coverage": "complete",
        "skill_name": "qiongli",
        "worker_plan": "full",
        "source_file_count": 377,
        "source_total_bytes": EXPECTED_CONTENT_TOTAL_BYTES,
        "source_tree_sha256": EXPECTED_CONTENT_TREE_SHA256,
        "included_resource_kinds": list(RESOURCE_KIND_ORDER),
        "file_count": 343,
        "total_bytes": 1_602_568,
        "tree_sha256": "b5612c713789bbd126829edc1e0646ec2c2387898aa2f5a4c812de0de5aad554",
        "origin_counts": {"identity-copy": 339, "content-transform": 2, "generated-metadata": 2},
        "pipeline": ["materialize-subject-from-accepted-root:core:full:complete"],
    },
)

PORTABLE_CORE_FACTS: Mapping[str, Any] = {
    "variant_id": "legacy-portable-core",
    "worker_plan": "portable-core",
    "file_count": 263,
    "total_bytes": 1_442_456,
    "tree_sha256": "21840d087bd18b1b9d37a03bddf6318a9023c69a0a320ff8bfcea843d4f5b48b",
    "origin_counts": {"identity-copy": 263, "content-transform": 0, "generated-metadata": 0},
}

RESERVED_WINDOWS_NAMES = frozenset(
    {"CON", "PRN", "AUX", "NUL"}
    | {f"COM{index}" for index in range(1, 10)}
    | {f"LPT{index}" for index in range(1, 10)}
)
SECRET_PATTERNS = (
    re.compile(r"QIONGLI_CANARY_DO_NOT_ECHO"),
    re.compile(r"\b(?:sk-|ghp_|github_pat_)[A-Za-z0-9_-]{12,}\b"),
    re.compile(r"-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----"),
)
MACHINE_PATH_PATTERNS = (
    re.compile(r"/Users/(?!name(?:/|\b)|user(?:/|\b)|you(?:/|\b)|username(?:/|\b))[A-Za-z0-9._-]+/"),
    re.compile(r"/home/(?!name(?:/|\b)|user(?:/|\b)|you(?:/|\b)|username(?:/|\b))[A-Za-z0-9._-]+/"),
    re.compile(r"[A-Za-z]:\\Users\\(?!name(?:\\|\b)|user(?:\\|\b)|you(?:\\|\b)|username(?:\\|\b))[A-Za-z0-9._-]+\\"),
)


class ExtractorError(RuntimeError):
    """The authenticated source or isolated oracle cannot be evaluated safely."""


class InventoryMismatch(RuntimeError):
    """An authenticated or generated identity differs from the accepted facts."""


class UsageError(RuntimeError):
    """Public command usage is invalid and must be reported without its input."""


class _RedactedArgumentParser(argparse.ArgumentParser):
    def error(self, _message: str) -> None:  # pragma: no cover - exercised through main
        raise UsageError("invalid command usage")


_ORIGINAL_PATH_WRITE_TEXT = Path.write_text


def _write_text_lf(
    path: Path,
    data: str,
    encoding: str | None = None,
    errors: str | None = None,
    newline: str | None = None,
) -> int:
    """Write generated reference output without platform newline translation."""

    del newline
    return _ORIGINAL_PATH_WRITE_TEXT(
        path,
        data,
        encoding=encoding,
        errors=errors,
        newline="",
    )


def _require_toolchain() -> None:
    if sys.version_info < (3, 12):
        raise ExtractorError("CTR-201D extraction requires Python 3.12 or newer")
    if yaml is None or getattr(yaml, "__version__", "") != "6.0.3":
        raise ExtractorError("CTR-201D extraction requires PyYAML 6.0.3")


def _sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def _canonical_json_bytes(value: Any) -> bytes:
    try:
        return json.dumps(
            value,
            ensure_ascii=False,
            sort_keys=True,
            separators=(",", ":"),
            allow_nan=False,
        ).encode("utf-8")
    except (TypeError, ValueError, UnicodeEncodeError) as error:
        raise ExtractorError("value cannot be serialized canonically") from error


def canonical_payload_sha256(record: Mapping[str, Any]) -> str:
    payload = {key: value for key, value in record.items() if key != "integrity"}
    return _sha256(_canonical_json_bytes(payload))


def canonical_schema_sha256(schema: Mapping[str, Any]) -> str:
    return _sha256(_canonical_json_bytes(schema))


def _reject_duplicate_object(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise ExtractorError("JSON document contains a duplicate key")
        result[key] = value
    return result


def _reject_nonfinite_constant(_value: str) -> None:
    raise ExtractorError("JSON document contains a non-finite number")


def _load_json_bytes(data: bytes) -> Mapping[str, Any]:
    try:
        value = json.loads(
            data.decode("utf-8"),
            object_pairs_hook=_reject_duplicate_object,
            parse_constant=_reject_nonfinite_constant,
        )
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise ExtractorError("JSON document is invalid") from error
    if not isinstance(value, Mapping):
        raise ExtractorError("JSON document must contain an object")
    return value


def _portable_path(raw: Any, *, allow_trailing_slash: bool = False) -> str:
    if not isinstance(raw, str) or not raw or "\\" in raw or "\x00" in raw:
        raise ExtractorError("path is not portable")
    trailing = raw.endswith("/")
    candidate = raw[:-1] if trailing else raw
    if trailing and not allow_trailing_slash:
        raise ExtractorError("path is not portable")
    if unicodedata.normalize("NFC", candidate) != candidate:
        raise ExtractorError("path is not Unicode NFC")
    path = PurePosixPath(candidate)
    if path.is_absolute() or any(part in {"", ".", ".."} for part in path.parts):
        raise ExtractorError("path is not portable")
    if len(path.parts) > MAX_PATH_DEPTH or len(candidate.encode("utf-8")) > MAX_PATH_BYTES:
        raise ExtractorError("path exceeds the portable limit")
    for part in path.parts:
        if part.endswith((" ", ".")) or ":" in part or any(ord(char) < 32 for char in part):
            raise ExtractorError("path is not portable")
        if part.split(".", 1)[0].upper() in RESERVED_WINDOWS_NAMES:
            raise ExtractorError("path uses a reserved device name")
    normalized = path.as_posix() + ("/" if trailing else "")
    if normalized != raw:
        raise ExtractorError("path is not canonical")
    return normalized


def _path_collision_key(path: str) -> str:
    return unicodedata.normalize("NFC", path).casefold()


def _assert_collision_free_paths(paths: Sequence[str]) -> None:
    keys: set[str] = set()
    for path in paths:
        canonical = _portable_path(path)
        key = _path_collision_key(canonical)
        if key in keys:
            raise ExtractorError("portable path collision detected")
        keys.add(key)


def _git_environment() -> dict[str, str]:
    environment = {
        key: value for key, value in os.environ.items() if not key.upper().startswith("GIT_")
    }
    environment.update(
        {
            "GIT_CONFIG_COUNT": "0",
            "GIT_CONFIG_NOSYSTEM": "1",
            "GIT_CONFIG_GLOBAL": os.devnull,
            "GIT_NO_LAZY_FETCH": "1",
            "GIT_NO_REPLACE_OBJECTS": "1",
            "GIT_OPTIONAL_LOCKS": "0",
            "GIT_TERMINAL_PROMPT": "0",
            "LANG": "C",
            "LC_ALL": "C",
        }
    )
    return environment


def _run_git(repo_root: Path, arguments: Sequence[str], *, input_bytes: bytes | None = None) -> bytes:
    try:
        completed = subprocess.run(
            ["git", *arguments],
            cwd=repo_root,
            env=_git_environment(),
            input=input_bytes,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
            timeout=30,
        )
    except (OSError, subprocess.TimeoutExpired) as error:
        raise ExtractorError("authenticated Git reader is unavailable") from error
    if completed.returncode != 0:
        raise ExtractorError("authenticated Git reader failed")
    if len(completed.stdout) > MAX_TREE_BYTES * 4:
        raise ExtractorError("authenticated Git response exceeds the size limit")
    return completed.stdout


def _read_manifest(repo_root: Path) -> tuple[Mapping[str, Any], list[dict[str, Any]]]:
    path = repo_root / MANIFEST_RELATIVE
    try:
        data = path.read_bytes()
    except OSError as error:
        raise ExtractorError("accepted A8 manifest is unavailable") from error
    if _sha256(data) != MANIFEST_SHA256:
        raise InventoryMismatch("accepted A8 manifest digest differs")
    manifest = _load_json_bytes(data)
    source = manifest.get("source")
    if not isinstance(source, Mapping):
        raise InventoryMismatch("accepted A8 source binding is invalid")
    expected_source = {
        "tag": ACCEPTED_TAG,
        "peeled_commit": ACCEPTED_COMMIT,
        "tag_object_oid": ACCEPTED_TAG_OBJECT,
        "tag_type": "annotated",
        "tree_access": "git-ls-tree-and-cat-file",
    }
    if dict(source) != expected_source:
        raise InventoryMismatch("accepted A8 source binding differs")
    package_trees = manifest.get("package_trees")
    if not isinstance(package_trees, list):
        raise InventoryMismatch("accepted A8 package trees are invalid")
    matches = [
        item for item in package_trees if isinstance(item, Mapping) and item.get("root") == "content/"
    ]
    if len(matches) != 1:
        raise InventoryMismatch("accepted A8 content tree is missing or duplicated")
    tree = matches[0]
    files = tree.get("files")
    if not isinstance(files, list) or not all(isinstance(item, Mapping) for item in files):
        raise InventoryMismatch("accepted A8 content file inventory is invalid")
    rows = [dict(item) for item in files]
    if (
        tree.get("file_count") != EXPECTED_CONTENT_FILE_COUNT
        or tree.get("tree_sha256") != EXPECTED_CONTENT_TREE_SHA256
        or len(rows) != EXPECTED_CONTENT_FILE_COUNT
    ):
        raise InventoryMismatch("accepted A8 content identity differs")
    return manifest, rows


def _verify_tag(repo_root: Path) -> None:
    if _run_git(repo_root, ["cat-file", "-t", ACCEPTED_TAG]).strip() != b"tag":
        raise InventoryMismatch("accepted tag is not annotated")
    if _run_git(repo_root, ["rev-parse", ACCEPTED_TAG]).strip() != ACCEPTED_TAG_OBJECT.encode("ascii"):
        raise InventoryMismatch("accepted tag object differs")
    if _run_git(repo_root, ["rev-parse", f"{ACCEPTED_TAG}^{{}}" ]).strip() != ACCEPTED_COMMIT.encode("ascii"):
        raise InventoryMismatch("accepted tag commit differs")


def _validate_source_record(record: Mapping[str, Any]) -> dict[str, Any]:
    required = {"path", "mode", "git_blob_oid", "sha256", "size_bytes"}
    if set(record) != required:
        raise InventoryMismatch("accepted content entry shape differs")
    path = _portable_path(record.get("path"))
    mode = record.get("mode")
    oid = record.get("git_blob_oid")
    digest = record.get("sha256")
    size = record.get("size_bytes")
    if mode != "100644":
        raise InventoryMismatch("accepted content entry is not a regular data file")
    if not isinstance(oid, str) or re.fullmatch(r"[0-9a-f]{40}", oid) is None:
        raise InventoryMismatch("accepted content blob identity is invalid")
    if not isinstance(digest, str) or re.fullmatch(r"[0-9a-f]{64}", digest) is None:
        raise InventoryMismatch("accepted content digest is invalid")
    if not isinstance(size, int) or isinstance(size, bool) or size < 0 or size > MAX_BLOB_BYTES:
        raise InventoryMismatch("accepted content size is invalid")
    return {
        "git_blob_oid": oid,
        "mode": mode,
        "path": path,
        "sha256": digest,
        "size_bytes": size,
    }


def _verify_tree_bindings(repo_root: Path, rows: Sequence[Mapping[str, Any]]) -> list[dict[str, Any]]:
    normalized = [_validate_source_record(row) for row in rows]
    if normalized != sorted(normalized, key=lambda item: item["path"].encode("utf-8")):
        raise InventoryMismatch("accepted content inventory ordering differs")
    paths = [item["path"] for item in normalized]
    if len(paths) != len(set(paths)):
        raise InventoryMismatch("accepted content inventory contains a path collision")
    try:
        _assert_collision_free_paths(paths)
    except ExtractorError as error:
        raise InventoryMismatch("accepted content inventory contains a path collision") from error
    if sum(int(item["size_bytes"]) for item in normalized) != EXPECTED_CONTENT_TOTAL_BYTES:
        raise InventoryMismatch("accepted content byte count differs")
    if _sha256(_canonical_json_bytes(normalized)) != EXPECTED_CONTENT_TREE_SHA256:
        raise InventoryMismatch("accepted content tree digest differs")

    tree_bytes = _run_git(repo_root, ["ls-tree", "-r", "-z", ACCEPTED_COMMIT, "--", "content"])
    observed: dict[str, tuple[str, str]] = {}
    for raw in tree_bytes.split(b"\0"):
        if not raw:
            continue
        try:
            metadata, path_bytes = raw.split(b"\t", 1)
            mode_bytes, kind, oid_bytes = metadata.split(b" ", 2)
            path = path_bytes.decode("utf-8")
            mode = mode_bytes.decode("ascii")
            oid = oid_bytes.decode("ascii")
        except (ValueError, UnicodeDecodeError) as error:
            raise ExtractorError("accepted Git tree response is invalid") from error
        if kind != b"blob":
            raise InventoryMismatch("accepted content tree contains a non-blob entry")
        _portable_path(path)
        if path in observed:
            raise InventoryMismatch("accepted Git tree contains a duplicate path")
        observed[path] = (mode, oid)
    expected = {item["path"]: (item["mode"], item["git_blob_oid"]) for item in normalized}
    if observed != expected:
        raise InventoryMismatch("accepted Git content bindings differ from A8")
    return normalized


def _anchor_records(repo_root: Path) -> list[dict[str, Any]]:
    records: list[dict[str, Any]] = []
    for raw in SOURCE_ANCHORS:
        record = dict(raw)
        path = _portable_path(record.get("path"))
        output = _run_git(repo_root, ["ls-tree", "-z", ACCEPTED_COMMIT, "--", path])
        expected = (
            f"{record['mode']} blob {record['git_blob_oid']}\t{path}\0".encode("utf-8")
        )
        if output != expected:
            raise InventoryMismatch("accepted materializer anchor binding differs")
        records.append(record)
    return records


def _verify_executed_reference_safety(anchor_blobs: Mapping[str, bytes]) -> None:
    executed_paths = (
        "packages/python-qiongli/src/qiongli/__init__.py",
        "packages/python-qiongli/src/qiongli/source_layout.py",
        "packages/python-qiongli/src/qiongli/subject_materializer.py",
    )
    allowed_import_roots = {
        "__future__",
        "dataclasses",
        "json",
        "pathlib",
        "qiongli",
        "re",
        "shutil",
        "typing",
        "yaml",
    }
    forbidden_calls = {"compile", "eval", "exec", "__import__"}
    forbidden_roots = {
        "asyncio",
        "http",
        "importlib",
        "requests",
        "socket",
        "subprocess",
        "urllib",
    }
    for path in executed_paths:
        data = anchor_blobs.get(path)
        if data is None:
            raise InventoryMismatch("executed materializer source is not authenticated")
        try:
            tree = ast.parse(data.decode("utf-8"), filename=path)
        except (UnicodeDecodeError, SyntaxError) as error:
            raise ExtractorError("executed materializer source is not valid Python") from error
        if len(list(ast.walk(tree))) > 20_000:
            raise ExtractorError("executed materializer source exceeds the AST limit")
        for node in ast.walk(tree):
            if isinstance(node, ast.Import):
                roots = {alias.name.split(".", 1)[0] for alias in node.names}
                if not roots <= allowed_import_roots:
                    raise ExtractorError("executed materializer imports an unapproved module")
            elif isinstance(node, ast.ImportFrom):
                root = (node.module or "").split(".", 1)[0]
                if root not in allowed_import_roots:
                    raise ExtractorError("executed materializer imports an unapproved module")
            elif isinstance(node, ast.Call):
                if isinstance(node.func, ast.Name) and node.func.id in forbidden_calls:
                    raise ExtractorError("executed materializer uses dynamic execution")
                current: ast.AST = node.func
                while isinstance(current, ast.Attribute):
                    current = current.value
                if isinstance(current, ast.Name) and current.id in forbidden_roots:
                    raise ExtractorError("executed materializer has a forbidden call path")


def _cat_file_blobs(repo_root: Path, records: Sequence[Mapping[str, Any]]) -> dict[str, bytes]:
    unique_by_oid: dict[str, Mapping[str, Any]] = {}
    for record in records:
        oid = str(record["git_blob_oid"])
        previous = unique_by_oid.get(oid)
        if previous is not None and (
            previous.get("sha256") != record.get("sha256")
            or previous.get("size_bytes") != record.get("size_bytes")
        ):
            raise InventoryMismatch("accepted blob identity is ambiguous")
        unique_by_oid[oid] = record
    ordered = sorted(unique_by_oid.values(), key=lambda item: str(item["git_blob_oid"]))
    request = b"".join(f"{item['git_blob_oid']}\n".encode("ascii") for item in ordered)
    response = _run_git(repo_root, ["cat-file", "--batch"], input_bytes=request)
    stream = io.BytesIO(response)
    by_oid: dict[str, bytes] = {}
    for expected in ordered:
        header = stream.readline()
        tokens = header[:-1].split() if header.endswith(b"\n") else []
        if len(tokens) != 3:
            raise ExtractorError("accepted Git blob response is invalid")
        try:
            oid = tokens[0].decode("ascii")
            kind = tokens[1].decode("ascii")
            size = int(tokens[2].decode("ascii"))
        except (UnicodeDecodeError, ValueError) as error:
            raise ExtractorError("accepted Git blob response is invalid") from error
        if oid != expected["git_blob_oid"] or kind != "blob" or size != expected["size_bytes"]:
            raise InventoryMismatch("accepted Git blob header differs")
        data = stream.read(size)
        terminator = stream.read(1)
        if len(data) != size or terminator != b"\n":
            raise ExtractorError("accepted Git blob response is truncated")
        if _sha256(data) != expected["sha256"]:
            raise InventoryMismatch("accepted Git blob digest differs")
        by_oid[oid] = data
    if stream.read(1):
        raise ExtractorError("accepted Git blob response has trailing data")
    return {str(record["path"]): by_oid[str(record["git_blob_oid"])] for record in records}


def _matches_root(path: str, root: Mapping[str, Any]) -> bool:
    source = str(root["source"])
    return path == source if root["match"] == "exact" else path.startswith(source)


def _resource_catalog(rows: Sequence[Mapping[str, Any]]) -> tuple[list[dict[str, Any]], list[dict[str, Any]]]:
    roots: list[dict[str, Any]] = []
    for expected in RESOURCE_ROOTS:
        selected = [dict(item) for item in rows if _matches_root(str(item["path"]), expected)]
        actual = {
            "source": expected["source"],
            "match": expected["match"],
            "resource_kind": expected["resource_kind"],
            "file_count": len(selected),
            "total_bytes": sum(int(item["size_bytes"]) for item in selected),
            "entries_sha256": _sha256(_canonical_json_bytes(selected)),
        }
        if actual != dict(expected):
            raise InventoryMismatch("accepted resource-root identity differs")
        roots.append(actual)
    for row in rows:
        if sum(_matches_root(str(row["path"]), root) for root in roots) != 1:
            raise InventoryMismatch("accepted content file does not have exactly one resource kind")

    kinds: list[dict[str, Any]] = []
    for kind in RESOURCE_KIND_ORDER:
        kind_roots = [root for root in roots if root["resource_kind"] == kind]
        selected = [
            dict(row)
            for row in rows
            if any(_matches_root(str(row["path"]), root) for root in kind_roots)
        ]
        kinds.append(
            {
                "resource_kind": kind,
                "source_roots": [str(root["source"]) for root in kind_roots],
                "file_count": len(selected),
                "total_bytes": sum(int(item["size_bytes"]) for item in selected),
                "entries_sha256": _sha256(_canonical_json_bytes(selected)),
            }
        )
    if len(kinds) != 11:
        raise InventoryMismatch("accepted resource-kind count differs")
    return roots, kinds


def _write_snapshot_file(root: Path, relative: str, data: bytes) -> None:
    path = root / PurePosixPath(_portable_path(relative))
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.exists() or path.is_symlink():
        raise ExtractorError("temporary accepted snapshot contains a collision")
    path.write_bytes(data)
    path.chmod(0o444)


def _materialize_snapshot(
    root: Path,
    content_rows: Sequence[Mapping[str, Any]],
    content_blobs: Mapping[str, bytes],
    anchor_blobs: Mapping[str, bytes],
) -> None:
    for row in content_rows:
        path = str(row["path"])
        _write_snapshot_file(root, path, content_blobs[path])
    executable_anchor_roles = {
        "accepted-python-package-init",
        "accepted-source-layout",
        "accepted-subject-materializer",
    }
    for anchor in SOURCE_ANCHORS:
        if anchor["role"] in executable_anchor_roles:
            path = str(anchor["path"])
            _write_snapshot_file(root, path, anchor_blobs[path])
    for directory in sorted(
        (item for item in root.rglob("*") if item.is_dir()),
        key=lambda item: len(item.parts),
        reverse=True,
    ):
        directory.chmod(0o555)
    root.chmod(0o555)


def _worker_environment(worker_root: Path, token: str) -> dict[str, str]:
    home = worker_root / "home"
    tmp = worker_root / "tmp"
    for path in (home, tmp):
        path.mkdir(parents=True, exist_ok=True)
    environment = {
        "HOME": str(home),
        "USERPROFILE": str(home),
        "XDG_CONFIG_HOME": str(home / "config"),
        "XDG_CACHE_HOME": str(home / "cache"),
        "XDG_DATA_HOME": str(home / "data"),
        "CODEX_HOME": str(home / "codex"),
        "CLAUDE_CODE_HOME": str(home / "claude"),
        "ANTIGRAVITY_HOME": str(home / "antigravity"),
        "HERMES_HOME": str(home / "hermes"),
        "TMP": str(tmp),
        "TEMP": str(tmp),
        "TMPDIR": str(tmp),
        "PATH": "",
        "LANG": "C.UTF-8",
        "LC_ALL": "C.UTF-8",
        "PYTHONNOUSERSITE": "1",
        "NO_PROXY": "*",
        "no_proxy": "*",
        "QIONGLI_CTR201D_WORKER_TOKEN": token,
    }
    if os.name == "nt" and os.environ.get("SystemRoot"):
        environment["SystemRoot"] = os.environ["SystemRoot"]
    return environment


def _run_materializer_worker(snapshot: Path, worker_root: Path, plan: str) -> Path:
    output = worker_root / f"output-{plan}"
    token = hashlib.sha256(os.urandom(32)).hexdigest()
    command = [
        sys.executable,
        "-I",
        "-B",
        str(Path(__file__).resolve()),
        "--_worker",
        token,
        plan,
        str(snapshot),
        str(output),
        str(worker_root),
    ]
    try:
        completed = subprocess.run(
            command,
            cwd=worker_root,
            env=_worker_environment(worker_root, token),
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
            timeout=45,
        )
    except (OSError, subprocess.TimeoutExpired) as error:
        raise ExtractorError("accepted materializer worker is unavailable") from error
    if completed.returncode != 0 or completed.stdout or len(completed.stderr) > 16_384:
        raise ExtractorError("accepted materializer worker failed")
    if not output.is_dir() or output.is_symlink():
        raise ExtractorError("accepted materializer worker did not produce a regular tree")
    return output


def _copy_tree(source: Path, destination: Path, *, exclude_claude_project: bool = False) -> None:
    def ignore(current: str, names: list[str]) -> set[str]:
        if exclude_claude_project and Path(current).resolve() == source.resolve():
            return {"CLAUDE.project.md"} & set(names)
        return set()

    shutil.copytree(source, destination, symlinks=False, ignore=ignore)


def _stage_plugin_source(snapshot: Path, stage: Path) -> Path:
    content = snapshot / "content"
    package = stage / "qiongli-workflow"
    _copy_tree(content / "workflow", package)
    for directory in (package, *[item for item in package.rglob("*") if item.is_dir()]):
        directory.chmod(0o755)
    _copy_tree(content / "skills", stage / "skills")
    _copy_tree(content / "subjects", stage / "subjects")
    _copy_tree(content / "skills", package / "skills")
    _copy_tree(content / "templates", package / "templates", exclude_claude_project=True)
    _copy_tree(content / "standards", package / "standards")
    _copy_tree(content / "roles", package / "roles")
    _copy_tree(content / "venue-profiles", package / "venue-profiles")
    shutil.copy2(content / "skills-core.md", package / "skills-core.md")
    shutil.copy2(content / "skills-summary.md", package / "skills-summary.md")
    for item in stage.rglob("*"):
        item.chmod(0o755 if item.is_dir() else 0o644)
    return stage


def _stage_full_source(snapshot: Path, stage: Path) -> Path:
    _copy_tree(snapshot / "content", stage / "content")
    for item in stage.rglob("*"):
        item.chmod(0o755 if item.is_dir() else 0o644)
    return stage


def _rewrite_next_skill(skill_root: Path) -> None:
    skill_path = skill_root / "SKILL.md"
    text = skill_path.read_text(encoding="utf-8")
    text = re.sub(r"(?m)^name:\s*qiongli\s*$", "name: qiongli-next", text)
    text = text.replace("Qiongli version:", "Qiongli Next version:", 1)
    text = text.replace("$qiongli", "$qiongli-next")
    if "$qiongli-next" not in text:
        text = (
            text.rstrip()
            + "\n\n## Prerelease Invocation\n\n"
            + "Invoke this beta package as `$qiongli-next` when testing the next Qiongli core workflow.\n"
        )
    skill_path.write_text(text, encoding="utf-8")


def _worker_main(arguments: Sequence[str]) -> int:
    if len(arguments) != 6 or arguments[0] != "--_worker":
        return 2
    _, token, plan, snapshot_raw, output_raw, worker_root_raw = arguments
    if not token or os.environ.get("QIONGLI_CTR201D_WORKER_TOKEN") != token:
        return 2
    if plan not in {"portable-core", "skill-only", "marketplace-lite", "full"}:
        return 2
    _require_toolchain()
    snapshot = Path(snapshot_raw).resolve()
    output = Path(output_raw).resolve()
    worker_root = Path(worker_root_raw).resolve()
    if (
        not worker_root.name.startswith("qiongli-ctr201d-")
        or snapshot != worker_root / "accepted"
        or output != worker_root / f"output-{plan}"
    ):
        return 2
    if output.exists() or output.is_symlink():
        return 2
    executed_blobs: dict[str, bytes] = {}
    for binding in SOURCE_ANCHORS:
        if binding["role"] not in {
            "accepted-python-package-init",
            "accepted-source-layout",
            "accepted-subject-materializer",
        }:
            continue
        candidate = snapshot / PurePosixPath(str(binding["path"]))
        if candidate.is_symlink() or not candidate.is_file():
            return 2
        try:
            data = candidate.read_bytes()
        except OSError:
            return 2
        if len(data) != binding["size_bytes"] or _sha256(data) != binding["sha256"]:
            return 2
        executed_blobs[str(binding["path"])] = data
    _verify_executed_reference_safety(executed_blobs)
    package_root = snapshot / "packages" / "python-qiongli" / "src"
    sys.dont_write_bytecode = True
    sys.path.insert(0, str(package_root))

    def audit_worker(event: str, event_args: tuple[Any, ...]) -> None:
        if event.startswith("socket.") or event in {
            "os.system",
            "pty.spawn",
            "subprocess.Popen",
        }:
            raise RuntimeError("worker network and process launch are disabled")
        if event in {"os.link", "os.symlink"}:
            raise RuntimeError("worker links are disabled")
        mutation_positions = {
            "os.chmod": (0,),
            "os.chown": (0,),
            "os.mkdir": (0,),
            "os.remove": (0,),
            "os.rename": (0, 1),
            "os.rmdir": (0,),
            "os.truncate": (0,),
            "os.utime": (0,),
        }

        def require_ephemeral_path(raw: Any) -> None:
            if not isinstance(raw, (str, bytes, os.PathLike)):
                return
            candidate = Path(raw).resolve(strict=False)
            if candidate == snapshot or snapshot in candidate.parents:
                raise RuntimeError("worker cannot mutate the authenticated snapshot")
            if candidate != worker_root and worker_root not in candidate.parents:
                raise RuntimeError("worker mutation escaped the ephemeral root")

        for position in mutation_positions.get(event, ()):
            if position < len(event_args):
                require_ephemeral_path(event_args[position])
        if event != "open" or not event_args or not isinstance(event_args[0], (str, bytes, os.PathLike)):
            return
        mode = event_args[1] if len(event_args) > 1 else None
        flags = event_args[2] if len(event_args) > 2 else 0
        writing = (
            isinstance(mode, str) and any(marker in mode for marker in ("w", "a", "x", "+"))
        ) or (
            isinstance(flags, int)
            and bool(flags & (os.O_WRONLY | os.O_RDWR | os.O_CREAT | os.O_TRUNC | os.O_APPEND))
        )
        if not writing:
            return
        require_ephemeral_path(event_args[0])

    sys.addaudithook(audit_worker)

    Path.write_text = _write_text_lf
    from qiongli.subject_materializer import MaterializeOptions, materialize_subject_package

    if plan == "full":
        source = _stage_full_source(snapshot, worker_root / "stage-full")
    else:
        stage = worker_root / f"stage-{plan}"
        source = _stage_plugin_source(snapshot, stage)
        if plan == "portable-core":
            shutil.copytree(source / "qiongli-workflow", output, symlinks=False)
            return 0
    flavor = "desktop" if plan == "skill-only" else "full"
    coverage = "focused" if plan == "skill-only" else "complete"
    materialize_subject_package(
        MaterializeOptions(
            source=source,
            out=output,
            subject="core",
            flavor=flavor,
            coverage=coverage,
        )
    )
    if plan in {"skill-only", "marketplace-lite"}:
        _rewrite_next_skill(output)
    return 0


def _scan_safe_text(path: str, data: bytes, forbidden_roots: Sequence[str]) -> None:
    try:
        text = data.decode("utf-8")
    except UnicodeDecodeError as error:
        raise ExtractorError("materialized skill payload is not UTF-8") from error
    for pattern in SECRET_PATTERNS:
        if pattern.search(text):
            raise ExtractorError("materialized skill payload contains secret-like data")
    for pattern in MACHINE_PATH_PATTERNS:
        if pattern.search(text):
            raise ExtractorError("materialized skill payload contains a machine-specific path")
    for root in forbidden_roots:
        if root and root in text:
            raise ExtractorError("materialized skill payload contains an extraction path")
    if "\x00" in text or any(0xD800 <= ord(character) <= 0xDFFF for character in text):
        raise ExtractorError("materialized skill payload contains invalid text")
    _portable_path(path)


def _source_path_for_output(path: str) -> str | None:
    if path in {"skills-core.md", "skills-summary.md"}:
        return f"content/{path}"
    first, separator, rest = path.partition("/")
    roots = {
        "agents": "content/workflow/agents",
        "references": "content/workflow/references",
        "roles": "content/roles",
        "skills": "content/skills",
        "standards": "content/standards",
        "subjects": "content/subjects",
        "templates": "content/templates",
        "venue-profiles": "content/venue-profiles",
        "workflows": "content/workflow/workflows",
    }
    if separator and first in roots:
        return f"{roots[first]}/{rest}"
    if path in {"SKILL.md", "VERSION"}:
        return f"content/workflow/{path}"
    return None


def _origin_for_entry(path: str, data: bytes, content_blobs: Mapping[str, bytes]) -> dict[str, Any]:
    source_path = _source_path_for_output(path)
    if source_path is not None and source_path in content_blobs and content_blobs[source_path] == data:
        return {"origin_class": "identity-copy", "source_paths": [source_path]}
    if path == "SKILL.md":
        return {
            "origin_class": "content-transform",
            "source_paths": ["content/subjects/catalog.yaml", "content/workflow/VERSION"],
        }
    if path == "skills/registry.yaml":
        return {
            "origin_class": "content-transform",
            "source_paths": ["content/skills/registry.yaml"],
        }
    generated_sources = {
        "SUBJECT": ["content/subjects/catalog.yaml"],
        "SUBJECT_MANIFEST.json": ["content/subjects/catalog.yaml", "content/workflow/VERSION"],
        "subjects/refinement-index.yaml": ["content/subjects/catalog.yaml"],
    }
    if path in generated_sources:
        return {"origin_class": "generated-metadata", "source_paths": generated_sources[path]}
    if source_path is not None and source_path in content_blobs:
        raise InventoryMismatch("unexpected accepted-source transformation was observed")
    raise InventoryMismatch("materialized output has no accepted-source provenance")


def _inventory_materialized_tree(
    root: Path,
    content_blobs: Mapping[str, bytes],
    *,
    forbidden_roots: Sequence[str],
) -> dict[str, Any]:
    if root.is_symlink() or not root.is_dir():
        raise ExtractorError("materialized output root is unsafe")
    paths: list[Path] = []
    for item in root.rglob("*"):
        try:
            metadata = item.lstat()
        except OSError as error:
            raise ExtractorError("materialized output could not be inspected") from error
        if stat.S_ISLNK(metadata.st_mode):
            raise ExtractorError("materialized output contains a symlink")
        if stat.S_ISDIR(metadata.st_mode):
            continue
        if not stat.S_ISREG(metadata.st_mode):
            raise ExtractorError("materialized output contains a non-regular file")
        if metadata.st_nlink != 1:
            raise ExtractorError("materialized output contains a hard-link alias")
        paths.append(item)
    if len(paths) > MAX_TREE_ENTRIES:
        raise ExtractorError("materialized output exceeds the entry limit")
    paths.sort(key=lambda item: item.relative_to(root).as_posix().encode("utf-8"))
    entries: list[dict[str, Any]] = []
    collision_keys: set[str] = set()
    total_bytes = 0
    top_level: dict[str, int] = {}
    origin_counts = {"identity-copy": 0, "content-transform": 0, "generated-metadata": 0}
    for item in paths:
        relative = _portable_path(item.relative_to(root).as_posix())
        collision_key = _path_collision_key(relative)
        if collision_key in collision_keys:
            raise ExtractorError("materialized output contains a portable path collision")
        collision_keys.add(collision_key)
        try:
            data = item.read_bytes()
        except OSError as error:
            raise ExtractorError("materialized output could not be read") from error
        if len(data) > MAX_BLOB_BYTES:
            raise ExtractorError("materialized output file exceeds the size limit")
        total_bytes += len(data)
        if total_bytes > MAX_TREE_BYTES:
            raise ExtractorError("materialized output exceeds the byte limit")
        _scan_safe_text(relative, data, forbidden_roots)
        origin = _origin_for_entry(relative, data, content_blobs)
        origin_counts[str(origin["origin_class"])] += 1
        top = relative.split("/", 1)[0]
        top_level[top] = top_level.get(top, 0) + 1
        entries.append(
            {
                "path": relative,
                "mode": "0644",
                "size_bytes": len(data),
                "sha256": _sha256(data),
                "origin": origin,
            }
        )
    digest_rows = [
        {key: entry[key] for key in ("path", "mode", "size_bytes", "sha256")}
        for entry in entries
    ]
    return {
        "root": "normalized-skill-root",
        "file_count": len(entries),
        "total_bytes": total_bytes,
        "tree_sha256": _sha256(_canonical_json_bytes(digest_rows)),
        "entries": entries,
        "top_level_counts": [
            {"name": name, "file_count": top_level[name]}
            for name in sorted(top_level, key=lambda value: value.encode("utf-8"))
        ],
        "origin_counts": origin_counts,
    }


def _assert_tree_facts(tree: Mapping[str, Any], expected: Mapping[str, Any]) -> None:
    actual = {
        "file_count": tree.get("file_count"),
        "total_bytes": tree.get("total_bytes"),
        "tree_sha256": tree.get("tree_sha256"),
        "origin_counts": tree.get("origin_counts"),
    }
    wanted = {key: expected[key] for key in actual}
    if actual != wanted:
        raise InventoryMismatch("accepted materialized tree identity differs")


def _source_closure(rows: Sequence[Mapping[str, Any]], profile: Mapping[str, Any]) -> dict[str, Any]:
    kinds = list(profile["included_resource_kinds"])
    selected = [
        dict(row)
        for row in rows
        if any(
            root["resource_kind"] in kinds and _matches_root(str(row["path"]), root)
            for root in RESOURCE_ROOTS
        )
    ]
    closure = {
        "included_resource_kinds": kinds,
        "file_count": len(selected),
        "total_bytes": sum(int(item["size_bytes"]) for item in selected),
        "tree_sha256": _sha256(_canonical_json_bytes(selected)),
    }
    expected = {
        "included_resource_kinds": kinds,
        "file_count": profile["source_file_count"],
        "total_bytes": profile["source_total_bytes"],
        "tree_sha256": profile["source_tree_sha256"],
    }
    if closure != expected:
        raise InventoryMismatch("accepted profile source closure differs")
    return closure


def extract_content_inventory(repo_root: Path) -> dict[str, Any]:
    _require_toolchain()
    try:
        root = repo_root.resolve(strict=True)
    except (OSError, RuntimeError) as error:
        raise ExtractorError("repository root is unavailable") from error
    _verify_tag(root)
    manifest, raw_rows = _read_manifest(root)
    del manifest
    content_rows = _verify_tree_bindings(root, raw_rows)
    anchors = _anchor_records(root)
    all_records: list[Mapping[str, Any]] = [*content_rows, *anchors]
    all_blobs = _cat_file_blobs(root, all_records)
    content_blobs = {row["path"]: all_blobs[str(row["path"])] for row in content_rows}
    anchor_blobs = {anchor["path"]: all_blobs[str(anchor["path"])] for anchor in anchors}
    _verify_executed_reference_safety(anchor_blobs)
    resource_roots, resource_kinds = _resource_catalog(content_rows)

    with tempfile.TemporaryDirectory(prefix="qiongli-ctr201d-") as directory:
        worker_root = Path(directory).resolve()
        snapshot = worker_root / "accepted"
        snapshot.mkdir()
        _materialize_snapshot(snapshot, content_rows, content_blobs, anchor_blobs)
        forbidden_roots = (str(root), str(worker_root), str(Path.home()))

        portable_output = _run_materializer_worker(snapshot, worker_root, "portable-core")
        portable_tree = _inventory_materialized_tree(
            portable_output,
            content_blobs,
            forbidden_roots=forbidden_roots,
        )
        _assert_tree_facts(portable_tree, PORTABLE_CORE_FACTS)

        profiles: list[dict[str, Any]] = []
        for facts in PROFILE_FACTS:
            output = _run_materializer_worker(snapshot, worker_root, str(facts["worker_plan"]))
            tree = _inventory_materialized_tree(
                output,
                content_blobs,
                forbidden_roots=forbidden_roots,
            )
            _assert_tree_facts(tree, facts)
            profiles.append(
                {
                    "profile_id": facts["profile_id"],
                    "aliases": list(facts["aliases"]),
                    "variant_id": facts["variant_id"],
                    "subject": facts["subject"],
                    "flavor": facts["flavor"],
                    "coverage": facts["coverage"],
                    "skill_name": facts["skill_name"],
                    "source_closure": _source_closure(content_rows, facts),
                    "source_closure_semantics": "resource-kind-policy-projection;not-an-exact-materializer-read-set",
                    "pipeline": list(facts["pipeline"]),
                    "materialized_tree": tree,
                    "evidence_scope": "authenticated-accepted-source-skill-subtree",
                    "published_archive_member_parity": "not-captured",
                }
            )

    artifact: dict[str, Any] = {
        "$schema": ARTIFACT_SCHEMA,
        "schema_version": SCHEMA_VERSION,
        "record_type": RECORD_TYPE,
        "task_id": "CTR-201D",
        "status": "content-materialization-captured",
        "source": {
            "accepted_tag": ACCEPTED_TAG,
            "accepted_commit": ACCEPTED_COMMIT,
            "accepted_tag_object": ACCEPTED_TAG_OBJECT,
            "manifest_path": MANIFEST_RELATIVE,
            "manifest_sha256": MANIFEST_SHA256,
            "content_tree": {
                "root": "content/",
                "file_count": EXPECTED_CONTENT_FILE_COUNT,
                "total_bytes": EXPECTED_CONTENT_TOTAL_BYTES,
                "tree_sha256": EXPECTED_CONTENT_TREE_SHA256,
                "files": content_rows,
            },
            "materializer_anchors": anchors,
        },
        "resource_catalog": {
            "resource_root_count": len(resource_roots),
            "resource_kind_count": len(resource_kinds),
            "roots": resource_roots,
            "kinds": resource_kinds,
        },
        "materialization_contract": {
            "python_requirement": ">=3.12",
            "pyyaml_version": "6.0.3",
            "execution_model": "authenticated-accepted-materializer-in-ephemeral-isolated-subprocess",
            "input_policy": "A8-authenticated-regular-blobs-only",
            "path_policy": "relative-posix-utf8-nfc-portable-casefold-unique",
            "mode_policy": "regular-skill-resources-normalized-to-0644",
            "entry_order": "ascending-utf8-path-bytes",
            "tree_canonicalization": TREE_CANONICALIZATION,
            "worker_network": "accepted-reference-code-static-allowlist-plus-python-audit-denial;os-network-sandbox-not-proven",
            "worker_write_scope": "ephemeral-temporary-root-only",
            "host_cache_writes": "forbidden",
        },
        "portable_core": {
            "variant_id": PORTABLE_CORE_FACTS["variant_id"],
            "pipeline": ["stage-plugin-source-excluding-templates/CLAUDE.project.md"],
            "materialized_tree": portable_tree,
            "evidence_scope": "authenticated-accepted-source-intermediate-tree",
        },
        "profiles": profiles,
        "coverage": {
            "accepted_content_file_count": EXPECTED_CONTENT_FILE_COUNT,
            "accepted_content_total_bytes": EXPECTED_CONTENT_TOTAL_BYTES,
            "resource_root_count": 12,
            "resource_kind_count": 11,
            "portable_core_file_count": 263,
            "materialized_profile_count": 3,
            "materialized_output_file_count": 863,
            "identity_output_count": 850,
            "transformed_output_count": 6,
            "generated_output_count": 7,
            "capture_ready": True,
        },
        "compatibility_boundary": {
            "a8_generated_tree_evidence": False,
            "published_archive_member_parity": "not-captured",
            "complete_plugin_wrapper_parity": "not-captured",
            "complete_native_binary_parity": "not-captured",
            "complete_subject_matrix_parity": "not-captured",
            "extraction_network_sandbox": "not-proven",
            "extraction_filesystem_sandbox": "python-audit-write-confined;host-read-isolation-not-proven;os-sandbox-not-proven",
            "captured_outputs": [
                "legacy-portable-core-intermediate",
                "accepted-prerelease-skill-only-core-subtree",
                "accepted-prerelease-marketplace-lite-core-skill-subtree",
                "accepted-full-local-core-skill-subtree",
            ],
            "excluded_outputs": [
                "published-archive-container-metadata",
                "plugin-manifests-and-command-wrappers",
                "target-specific-native-binaries",
                "npm-and-python-runtime-payload-closures",
                "non-core-subject-and-custom-overlay-matrix",
            ],
            "fnd_202_implemented": False,
        },
    }
    artifact["integrity"] = {
        "algorithm": "sha256",
        "canonicalization": CANONICALIZATION,
        "payload_sha256": canonical_payload_sha256(artifact),
    }
    return artifact


def _closed_object(properties: Mapping[str, Any], *, required: Sequence[str] | None = None) -> dict[str, Any]:
    keys = list(properties)
    return {
        "type": "object",
        "required": list(required) if required is not None else keys,
        "properties": dict(properties),
        "additionalProperties": False,
    }


def build_content_schema(_artifact: Mapping[str, Any]) -> dict[str, Any]:
    sha = {"type": "string", "pattern": "^[0-9a-f]{64}$"}
    oid = {"type": "string", "pattern": "^[0-9a-f]{40}$"}
    nonnegative = {"type": "integer", "minimum": 0}
    positive = {"type": "integer", "minimum": 1}
    path = {"type": "string", "minLength": 1, "maxLength": MAX_PATH_BYTES}
    source_file = _closed_object(
        {
            "git_blob_oid": oid,
            "mode": {"const": "100644"},
            "path": path,
            "sha256": sha,
            "size_bytes": nonnegative,
        }
    )
    anchor = _closed_object(
        {
            "role": {"type": "string", "minLength": 1},
            "path": path,
            "mode": {"enum": ["100644", "100755"]},
            "git_blob_oid": oid,
            "sha256": sha,
            "size_bytes": nonnegative,
        }
    )
    root_record = _closed_object(
        {
            "source": path,
            "match": {"enum": ["exact", "prefix"]},
            "resource_kind": {"enum": list(RESOURCE_KIND_ORDER)},
            "file_count": positive,
            "total_bytes": positive,
            "entries_sha256": sha,
        }
    )
    kind_record = _closed_object(
        {
            "resource_kind": {"enum": list(RESOURCE_KIND_ORDER)},
            "source_roots": {"type": "array", "minItems": 1, "items": path},
            "file_count": positive,
            "total_bytes": positive,
            "entries_sha256": sha,
        }
    )
    origin = _closed_object(
        {
            "origin_class": {
                "enum": ["identity-copy", "content-transform", "generated-metadata"]
            },
            "source_paths": {"type": "array", "minItems": 1, "items": path},
        }
    )
    materialized_entry = _closed_object(
        {
            "path": path,
            "mode": {"const": "0644"},
            "size_bytes": nonnegative,
            "sha256": sha,
            "origin": origin,
        }
    )
    origin_counts = _closed_object(
        {
            "identity-copy": nonnegative,
            "content-transform": nonnegative,
            "generated-metadata": nonnegative,
        }
    )
    top_level = _closed_object({"name": path, "file_count": positive})
    materialized_tree = _closed_object(
        {
            "root": {"const": "normalized-skill-root"},
            "file_count": positive,
            "total_bytes": positive,
            "tree_sha256": sha,
            "entries": {"type": "array", "minItems": 1, "items": materialized_entry},
            "top_level_counts": {"type": "array", "minItems": 1, "items": top_level},
            "origin_counts": origin_counts,
        }
    )
    source_closure = _closed_object(
        {
            "included_resource_kinds": {
                "type": "array",
                "minItems": 8,
                "maxItems": 11,
                "items": {"enum": list(RESOURCE_KIND_ORDER)},
            },
            "file_count": positive,
            "total_bytes": positive,
            "tree_sha256": sha,
        }
    )
    profile = _closed_object(
        {
            "profile_id": {"enum": ["skill-only", "marketplace-lite", "full"]},
            "aliases": {"type": "array", "maxItems": 1, "items": {"const": "lite"}},
            "variant_id": {"type": "string", "minLength": 1},
            "subject": {"const": "core"},
            "flavor": {"enum": ["desktop", "full"]},
            "coverage": {"enum": ["focused", "complete"]},
            "skill_name": {"enum": ["qiongli", "qiongli-next"]},
            "source_closure": source_closure,
            "source_closure_semantics": {"const": "resource-kind-policy-projection;not-an-exact-materializer-read-set"},
            "pipeline": {"type": "array", "minItems": 1, "items": {"type": "string"}},
            "materialized_tree": materialized_tree,
            "evidence_scope": {"const": "authenticated-accepted-source-skill-subtree"},
            "published_archive_member_parity": {"const": "not-captured"},
        }
    )
    source = _closed_object(
        {
            "accepted_tag": {"const": ACCEPTED_TAG},
            "accepted_commit": {"const": ACCEPTED_COMMIT},
            "accepted_tag_object": {"const": ACCEPTED_TAG_OBJECT},
            "manifest_path": {"const": MANIFEST_RELATIVE},
            "manifest_sha256": {"const": MANIFEST_SHA256},
            "content_tree": _closed_object(
                {
                    "root": {"const": "content/"},
                    "file_count": {"const": EXPECTED_CONTENT_FILE_COUNT},
                    "total_bytes": {"const": EXPECTED_CONTENT_TOTAL_BYTES},
                    "tree_sha256": {"const": EXPECTED_CONTENT_TREE_SHA256},
                    "files": {
                        "type": "array",
                        "minItems": EXPECTED_CONTENT_FILE_COUNT,
                        "maxItems": EXPECTED_CONTENT_FILE_COUNT,
                        "items": source_file,
                    },
                }
            ),
            "materializer_anchors": {
                "type": "array",
                "minItems": len(SOURCE_ANCHORS),
                "maxItems": len(SOURCE_ANCHORS),
                "items": anchor,
            },
        }
    )
    resource_catalog = _closed_object(
        {
            "resource_root_count": {"const": 12},
            "resource_kind_count": {"const": 11},
            "roots": {"type": "array", "minItems": 12, "maxItems": 12, "items": root_record},
            "kinds": {"type": "array", "minItems": 11, "maxItems": 11, "items": kind_record},
        }
    )
    materialization_contract = _closed_object(
        {
            "python_requirement": {"const": ">=3.12"},
            "pyyaml_version": {"const": "6.0.3"},
            "execution_model": {"const": "authenticated-accepted-materializer-in-ephemeral-isolated-subprocess"},
            "input_policy": {"const": "A8-authenticated-regular-blobs-only"},
            "path_policy": {"const": "relative-posix-utf8-nfc-portable-casefold-unique"},
            "mode_policy": {"const": "regular-skill-resources-normalized-to-0644"},
            "entry_order": {"const": "ascending-utf8-path-bytes"},
            "tree_canonicalization": {"const": TREE_CANONICALIZATION},
            "worker_network": {"const": "accepted-reference-code-static-allowlist-plus-python-audit-denial;os-network-sandbox-not-proven"},
            "worker_write_scope": {"const": "ephemeral-temporary-root-only"},
            "host_cache_writes": {"const": "forbidden"},
        }
    )
    portable = _closed_object(
        {
            "variant_id": {"const": "legacy-portable-core"},
            "pipeline": {
                "type": "array",
                "minItems": 1,
                "maxItems": 1,
                "items": {"const": "stage-plugin-source-excluding-templates/CLAUDE.project.md"},
            },
            "materialized_tree": materialized_tree,
            "evidence_scope": {"const": "authenticated-accepted-source-intermediate-tree"},
        }
    )
    coverage = _closed_object(
        {
            "accepted_content_file_count": {"const": 377},
            "accepted_content_total_bytes": {"const": 1_761_400},
            "resource_root_count": {"const": 12},
            "resource_kind_count": {"const": 11},
            "portable_core_file_count": {"const": 263},
            "materialized_profile_count": {"const": 3},
            "materialized_output_file_count": {"const": 863},
            "identity_output_count": {"const": 850},
            "transformed_output_count": {"const": 6},
            "generated_output_count": {"const": 7},
            "capture_ready": {"const": True},
        }
    )
    compatibility = _closed_object(
        {
            "a8_generated_tree_evidence": {"const": False},
            "published_archive_member_parity": {"const": "not-captured"},
            "complete_plugin_wrapper_parity": {"const": "not-captured"},
            "complete_native_binary_parity": {"const": "not-captured"},
            "complete_subject_matrix_parity": {"const": "not-captured"},
            "extraction_network_sandbox": {"const": "not-proven"},
            "extraction_filesystem_sandbox": {
                "const": "python-audit-write-confined;host-read-isolation-not-proven;os-sandbox-not-proven"
            },
            "captured_outputs": {"type": "array", "minItems": 4, "maxItems": 4, "items": {"type": "string"}},
            "excluded_outputs": {"type": "array", "minItems": 5, "maxItems": 5, "items": {"type": "string"}},
            "fnd_202_implemented": {"const": False},
        }
    )
    integrity = _closed_object(
        {
            "algorithm": {"const": "sha256"},
            "canonicalization": {"const": CANONICALIZATION},
            "payload_sha256": sha,
        }
    )
    properties = {
        "$schema": {"const": ARTIFACT_SCHEMA},
        "schema_version": {"const": SCHEMA_VERSION},
        "record_type": {"const": RECORD_TYPE},
        "task_id": {"const": "CTR-201D"},
        "status": {"const": "content-materialization-captured"},
        "source": source,
        "resource_catalog": resource_catalog,
        "materialization_contract": materialization_contract,
        "portable_core": portable,
        "profiles": {"type": "array", "minItems": 3, "maxItems": 3, "items": profile},
        "coverage": coverage,
        "compatibility_boundary": compatibility,
        "integrity": integrity,
    }
    schema = {
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": "https://qiongli.dev/schemas/ctr-201-content.schema.json",
        "title": "Qiongli CTR-201D content and materialized-tree inventory",
        **_closed_object(properties),
    }
    return schema


def _write_json(path: Path, value: Mapping[str, Any]) -> None:
    if path.is_symlink() or (path.exists() and not path.is_file()):
        raise UsageError("output path is unsafe")
    try:
        parent = path.parent.resolve()
    except (OSError, RuntimeError) as error:
        raise UsageError("output parent is unsafe") from error
    parent.mkdir(parents=True, exist_ok=True)
    if path.parent.is_symlink():
        raise UsageError("output parent is unsafe")
    data = (json.dumps(value, ensure_ascii=False, indent=2, sort_keys=True, allow_nan=False) + "\n").encode("utf-8")
    descriptor, temporary_name = tempfile.mkstemp(prefix=f".{path.name}.", dir=parent)
    temporary = Path(temporary_name)
    try:
        with os.fdopen(descriptor, "wb") as handle:
            handle.write(data)
            handle.flush()
            os.fsync(handle.fileno())
        temporary.chmod(0o644)
        os.replace(temporary, path)
    finally:
        if temporary.exists():
            temporary.unlink()


def _load_checked(path: Path) -> Mapping[str, Any]:
    try:
        if path.is_symlink() or not path.is_file():
            raise ExtractorError("checked inventory path is unsafe")
        return _load_json_bytes(path.read_bytes())
    except OSError as error:
        raise ExtractorError("checked inventory is unavailable") from error


def _output_paths_may_alias(left: Path, right: Path) -> bool:
    try:
        left_resolved = left.resolve(strict=False)
        right_resolved = right.resolve(strict=False)
    except (OSError, RuntimeError):
        left_resolved = left.absolute()
        right_resolved = right.absolute()
    if _path_collision_key(str(left_resolved)) == _path_collision_key(str(right_resolved)):
        return True
    try:
        return left.samefile(right)
    except (OSError, RuntimeError):
        return False


def _build_parser() -> argparse.ArgumentParser:
    parser = _RedactedArgumentParser(
        description="Extract the authenticated CTR-201D content/materialization inventory."
    )
    parser.add_argument("--root", default=str(REPO_ROOT))
    parser.add_argument("--json", action="store_true")
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--output")
    parser.add_argument("--schema-output")
    return parser


def _emit_result(
    *,
    json_mode: bool,
    status: str,
    exit_code: int,
    code: str,
    artifact: Mapping[str, Any] | None = None,
    schema: Mapping[str, Any] | None = None,
) -> None:
    payload: dict[str, Any] = {
        "status": status,
        "exit_code": exit_code,
        "code": code,
        "ctr_201": "in-progress",
        "fnd_202": "not-implemented",
    }
    if artifact is not None:
        payload["payload_sha256"] = artifact["integrity"]["payload_sha256"]
        payload["coverage"] = artifact["coverage"]
    if schema is not None:
        payload["schema_canonical_sha256"] = canonical_schema_sha256(schema)
    if json_mode:
        sys.stdout.buffer.write(_canonical_json_bytes(payload) + b"\n")
    elif exit_code == 0:
        print(code)
    else:
        print(code, file=sys.stderr)


def main(argv: Sequence[str] | None = None) -> int:
    arguments = list(sys.argv[1:] if argv is None else argv)
    if arguments and arguments[0] == "--_worker":
        try:
            return _worker_main(arguments)
        except Exception:  # worker output is intentionally redacted
            return 1
    json_mode = "--json" in arguments
    try:
        args = _build_parser().parse_args(arguments)
        json_mode = bool(args.json)
        if args.check and args.schema_output:
            raise UsageError("invalid command usage")
        if not args.check and not args.output:
            raise UsageError("invalid command usage")
        artifact = extract_content_inventory(Path(args.root))
        schema = build_content_schema(artifact)
        if args.check:
            output_path = Path(args.output) if args.output else Path(args.root) / DEFAULT_OUTPUT_RELATIVE
            schema_path = output_path.parent / Path(DEFAULT_SCHEMA_RELATIVE).name
            checked_artifact = _load_checked(output_path)
            checked_schema = _load_checked(schema_path)
            if _canonical_json_bytes(checked_artifact) != _canonical_json_bytes(artifact):
                raise InventoryMismatch("checked content artifact differs")
            if _canonical_json_bytes(checked_schema) != _canonical_json_bytes(schema):
                raise InventoryMismatch("checked content schema differs")
            _emit_result(
                json_mode=json_mode,
                status="pass",
                exit_code=0,
                code="accepted-content-inventory-matches",
                artifact=artifact,
                schema=schema,
            )
            return 0
        output_path = Path(args.output)
        schema_path = Path(args.schema_output) if args.schema_output else output_path.parent / Path(DEFAULT_SCHEMA_RELATIVE).name
        if _output_paths_may_alias(output_path, schema_path):
            raise UsageError("artifact and schema outputs must differ")
        _write_json(output_path, artifact)
        _write_json(schema_path, schema)
        _emit_result(
            json_mode=json_mode,
            status="pass",
            exit_code=0,
            code="accepted-content-inventory-written",
            artifact=artifact,
            schema=schema,
        )
        return 0
    except UsageError:
        _emit_result(
            json_mode=json_mode,
            status="error",
            exit_code=2,
            code="invalid-command-usage",
        )
        return 2
    except InventoryMismatch:
        _emit_result(
            json_mode=json_mode,
            status="fail",
            exit_code=1,
            code="accepted-content-inventory-mismatch",
        )
        return 1
    except (
        ExtractorError,
        OSError,
        RuntimeError,
        ValueError,
        subprocess.SubprocessError,
    ):
        _emit_result(
            json_mode=json_mode,
            status="error",
            exit_code=2,
            code="content-inventory-extraction-failed",
        )
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
