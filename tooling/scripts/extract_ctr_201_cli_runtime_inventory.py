#!/usr/bin/env python3
from __future__ import annotations

import argparse
import copy
import hashlib
import io
import json
import os
from pathlib import Path, PurePosixPath
import re
import shutil
import subprocess
import sys
import tempfile
import traceback
from contextlib import redirect_stderr, redirect_stdout
from typing import Any, Callable, Mapping, Sequence

try:
    from tooling.scripts import extract_ctr_201_cli_inventory as static_cli
except ModuleNotFoundError:  # Isolated worker execution from tooling/scripts.
    _SCRIPT_DIRECTORY = str(Path(__file__).resolve().parent)
    if _SCRIPT_DIRECTORY not in sys.path:
        sys.path.insert(0, _SCRIPT_DIRECTORY)
    import extract_ctr_201_cli_inventory as static_cli  # type: ignore[no-redef]


REPO_ROOT = Path(__file__).resolve().parents[2]
DEFAULT_OUTPUT_RELATIVE = "tooling/migration/ctr-201-cli-runtime.json"
SCHEMA_RELATIVE = "tooling/migration/ctr-201-cli-runtime.schema.json"
STATIC_ARTIFACT_RELATIVE = "tooling/migration/ctr-201-cli.json"
STATIC_SCHEMA_RELATIVE = "tooling/migration/ctr-201-cli.schema.json"
PYTHON_ORACLE_RELATIVE = (
    "tooling/migration/baselines/v1.19.0-beta.1/oracles/python-full.json"
)
ACCEPTED_TAG = static_cli.ACCEPTED_TAG
ACCEPTED_TAG_OBJECT = "e68e3af4c879d8e9053124d1aed625bfcddfdbb4"
ACCEPTED_COMMIT = static_cli.ACCEPTED_COMMIT
MANIFEST_RELATIVE = static_cli.MANIFEST_RELATIVE
MANIFEST_SHA256 = static_cli.MANIFEST_SHA256
ACCEPTANCE_RECEIPT = {
    "path": "tooling/release/acceptance/v1.19.0-beta.1-receipt.md",
    "git_blob_oid": "605ab151b1621838a85f9909d6877f0f69857fc3",
    "sha256": "a462dc24d94debfb678038e9ed437bdf04dc75476237cc74a9bf06ac366444e9",
    "size_bytes": 6641,
    "source_commit": "ba4517c8dfd5ce8b551c83b129213e689d32cac4",
}
PYTHON_ORACLE_SHA256 = static_cli.PYTHON_ORACLE_BINDING["sha256"]
PYTHON_PACKAGE_TREE_SHA256 = static_cli.EXPECTED_PACKAGE_TREE_SHA256
NPM_PACKAGE_ROOT = "packages/npm-qiongli/"
NPM_PACKAGE_FILE_COUNT = 16
NPM_PACKAGE_TREE_SHA256 = (
    "b3090cbc64b89ec794963d4c679c5e7eb334d95cf565b9484cc68ef052505513"
)
EXPECTED_COUNTS = {
    "canonical_commands": 46,
    "public_commands": 49,
    "console_entrypoints": 5,
    "argument_actions": 164,
    "cwd_defaults": 27,
    "json_canonical_commands": 13,
    "dry_run_canonical_commands": 8,
    "dry_run_public_commands": 11,
    "group_or_delegate_commands": 6,
    "executable_canonical_commands": 40,
    "executable_public_commands": 43,
    "help_observations": 245,
    "invalid_usage_observations": 49,
    "zero_argument_observations": 5,
    "npm_aliases": 5,
}
EXPECTED_ENTRYPOINTS = tuple(name for name, _ in static_cli.EXPECTED_ENTRYPOINTS)
EXPECTED_NPM_DISPATCH = {
    "help": ("help", False),
    "install": ("install", False),
    "setup": ("setup", False),
    "update": ("install", True),
    "refresh": ("install", True),
    "upgrade": ("install", True),
    "remove": ("remove", False),
    "uninstall": ("remove", False),
    "delete": ("remove", False),
    "check": ("check", False),
    "clean": ("clean", False),
    "runtime": ("runtime", False),
    "project": ("project", False),
    "doctor": ("doctor", False),
    "task-run": ("task-run", False),
    "mcp": ("mcp", False),
    "provider": ("provider", False),
    "guidance": ("guidance", False),
    "customize": ("customize", False),
    "init": ("init", False),
    "align": ("align", False),
    "unknown-command": ("unknown-command", False),
}
DISPOSITION_DECISIONS = [
    {
        "id": "CTR-201E-D001",
        "status": "approved-for-ctr-201-inventory-only",
        "owner_task": "LEG-201",
        "scope": "bounded-handler-runtime-fixtures",
        "rationale": (
            "CTR-201 freezes the complete migration inventory; successful handler "
            "parity requires stateful fixtures owned by LEG-201 and is not inferred "
            "from parser evidence"
        ),
    },
    {
        "id": "CTR-201E-D002",
        "status": "approved-for-ctr-201-inventory-only",
        "owner_task": "LEG-201",
        "scope": "network-browser-listener-secret-update-runtime",
        "rationale": (
            "network, browser, listener, secret-bearing, download, and self-update "
            "handlers remain explicitly unexecuted until a purpose-built bounded "
            "LEG-201 fixture exists"
        ),
    },
    {
        "id": "CTR-201E-D003",
        "status": "approved-for-ctr-201-inventory-only",
        "owner_task": "LEG-201",
        "scope": "legacy-npm-handler-runtime-parity",
        "rationale": (
            "CTR-201E authenticates and freezes parseArgv dispatch only; npm handler "
            "stdout, exit, and side-effect parity remains LEG-201 work"
        ),
    },
]
DISPOSITION_DECISION_IDS = frozenset(row["id"] for row in DISPOSITION_DECISIONS)
CANARY_SECRET = "CTR201E_CANARY_SECRET_MUST_NOT_APPEAR"
ARTIFACT_SCHEMA = "./ctr-201-cli-runtime.schema.json"
ARTIFACT_SCHEMA_VERSION = "1.0"
ARTIFACT_RECORD_TYPE = "qiongli-ctr-201-cli-runtime-inventory-freeze"
CANONICALIZATION = "utf-8-json-sorted-keys-compact-excluding-integrity"
HEX_64 = re.compile(r"^[0-9a-f]{64}$")
EXPECTED_PAYLOAD_SHA256 = (
    "07d0a8b97a117e137351ac18f3cbcabc2078b6031dcd1da2893a9cda11d7c8f4"
)
EXPECTED_CASE_MANIFEST_SHA256 = (
    "6f0236562749c56adc94318e49b098b7392dc99bf193e1afd9bda1deb66a7e2e"
)
MACHINE_PATH_PATTERN = re.compile(
    r"(?:file://|(?<![A-Za-z0-9/])/(?:Users|home|root|Volumes|tmp|var/tmp|"
    r"var/folders|private/tmp|private/var/folders)/|"
    r"(?<![A-Za-z0-9+.-])[A-Za-z]:[\\/]|\\\\[^\\/\s]+[\\/][^\\/\s]+)",
    re.IGNORECASE,
)
SECRET_PATTERN = re.compile(
    r"(?:QIONGLI_CANARY_DO_NOT_ECHO|-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----|"
    r"\b(?:sk[-_]|ghp_|github_pat_)[A-Za-z0-9_-]{12,}\b)"
)
CALLABLE_REPR_PATTERN = re.compile(
    r"(?:<(?:(?:bound )?method|function|class)\b|\bat 0x[0-9a-f]+>)",
    re.IGNORECASE,
)
EXPECTED_CAPTURE_CONTRACT = {
    "source_mode": "accepted-tag-git-blobs",
    "python_version": "python3.12",
    "python_isolation": "-I-S-B-dual-environment-audit-hook",
    "npm_capture": "accepted-args-mjs-parse-only-dual-environment",
    "npm_engine_requirement": ">=18",
    "terminal": "non-tty-80-columns-no-color",
    "locale": "C.UTF-8",
    "timezone": "UTC",
    "path_normalization": [
        "<RUNTIME_SOURCE>",
        "<ACCEPTED_ROOT>",
        "<CONFIG_ROOT>",
        "<HOME>",
        "<CWD>",
        "<SANDBOX>",
    ],
    "network_policy": (
        "python-audit-denied;node-authenticated-parse-only-no-capability-imports"
    ),
    "write_policy": (
        "python-mutable-roots-measured-and-audit-confined;"
        "node-accepted-tree-before-after-identical"
    ),
    "process_policy": (
        "accepted-runtime-cases-launch-no-child-processes;"
        "parent-launches-git-python-node-capture-processes"
    ),
    "secret_policy": "sanitized-environment-and-canary-rejection",
    "determinism": "two-distinct-temp-roots-and-hash-seeds-byte-equivalent",
}
OBSERVED_ERROR_TAXONOMY = [
    {"id": "none", "meaning": "successful observable outcome"},
    {
        "id": "usage-error",
        "meaning": "argparse rejected command usage with exit code 2",
    },
    {
        "id": "input-error",
        "meaning": "accepted handler rejected an input at the console boundary",
    },
]


class RuntimeExtractorError(RuntimeError):
    """The accepted runtime capture could not be evaluated safely."""


class RuntimeInventoryMismatch(RuntimeError):
    """The accepted runtime observation differs from the checked contract."""


class CliUsageError(RuntimeError):
    """Public extractor usage is invalid and must not echo user input."""


class _RedactedArgumentParser(argparse.ArgumentParser):
    def error(self, message: str) -> None:  # pragma: no cover - exercised via main
        del message
        raise CliUsageError("invalid command usage")


def _sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def _canonical_json_bytes(value: Any) -> bytes:
    try:
        rendered = json.dumps(
            value,
            ensure_ascii=False,
            sort_keys=True,
            separators=(",", ":"),
            allow_nan=False,
        )
        return rendered.encode("utf-8")
    except (TypeError, ValueError, UnicodeEncodeError) as error:
        raise RuntimeExtractorError("runtime artifact is not canonically serializable") from error


def canonical_payload_sha256(record: Mapping[str, Any]) -> str:
    payload = {key: value for key, value in record.items() if key != "integrity"}
    return _sha256(_canonical_json_bytes(payload))


def _canonical_schema_sha256(path: Path) -> str:
    document = static_cli._load_json_bytes(path.read_bytes())
    return _sha256(_canonical_json_bytes(document))


def _canonical_case_sha256(value: Mapping[str, Any]) -> str:
    return _sha256(_canonical_json_bytes(value))


def case_manifest_sha256(cases: Sequence[Mapping[str, Any]]) -> str:
    manifest = [
        {"id": case.get("id"), "sha256": _canonical_case_sha256(case)}
        for case in cases
    ]
    return _sha256(_canonical_json_bytes(manifest))


def _iter_strings(value: Any) -> list[str]:
    strings: list[str] = []
    if isinstance(value, str):
        strings.append(value)
    elif isinstance(value, Mapping):
        for key, item in value.items():
            if isinstance(key, str):
                strings.append(key)
            strings.extend(_iter_strings(item))
    elif isinstance(value, list):
        for item in value:
            strings.extend(_iter_strings(item))
    return strings


def _expected_source(
    repo_root: Path, checked_static: Mapping[str, Any]
) -> Mapping[str, Any]:
    return {
        "trust": "exact-tag-object-receipt-and-digest-bound-not-signature-verified",
        "accepted_tag": ACCEPTED_TAG,
        "tag_object_oid": ACCEPTED_TAG_OBJECT,
        "accepted_commit": ACCEPTED_COMMIT,
        "a8_manifest": {"path": MANIFEST_RELATIVE, "sha256": MANIFEST_SHA256},
        "acceptance_receipt": dict(ACCEPTANCE_RECEIPT),
        "python_package_tree": {
            "root": static_cli.PYTHON_PACKAGE_ROOT,
            "file_count": static_cli.EXPECTED_PACKAGE_FILE_COUNT,
            "tree_sha256": PYTHON_PACKAGE_TREE_SHA256,
        },
        "npm_package_tree": {
            "root": NPM_PACKAGE_ROOT,
            "file_count": NPM_PACKAGE_FILE_COUNT,
            "tree_sha256": NPM_PACKAGE_TREE_SHA256,
        },
        "python_full_oracle": {
            "path": PYTHON_ORACLE_RELATIVE,
            "sha256": PYTHON_ORACLE_SHA256,
            "case_ids": ["python.cli-align", "python.installer-dry-run"],
        },
        "ctr_201b": {
            "artifact_path": STATIC_ARTIFACT_RELATIVE,
            "schema_path": STATIC_SCHEMA_RELATIVE,
            "schema_canonical_sha256": _canonical_schema_sha256(
                repo_root / STATIC_SCHEMA_RELATIVE
            ),
            "payload_sha256": checked_static["integrity"]["payload_sha256"],
        },
    }


def _expected_coverage() -> Mapping[str, Any]:
    return {
        **EXPECTED_COUNTS,
        "formatted_help_output": "captured",
        "stdout_stderr": (
            "parser-and-bounded-handler-cases-plus-approved-dispositions"
        ),
        "json_output": "classified-with-safe-cases-and-explicit-dispositions",
        "runtime_behavior_matrix": "classified-with-no-omissions",
        "exit_code_matrix": (
            "parser-and-bounded-handler-cases-plus-approved-dispositions"
        ),
        "dry_run_semantics": "accepted-case-plus-explicit-dispositions",
        "error_matrix": (
            "parser-and-representative-domain-cases-plus-approved-dispositions"
        ),
        "side_effect_matrix": "measured-safe-cases-plus-approved-dispositions",
        "legacy_npm_compatibility_surface": (
            "dispatch-captured-handler-parity-pending-LEG-201"
        ),
        "full_handler_runtime_parity": "not-claimed",
        "cli_inventory_completion_ready": True,
        "ctr_201": "in-progress",
        "ctr_202": "not-complete",
        "fnd_202": "not-implemented",
        "rust_cli": "not-implemented",
        "cross_platform_runtime_parity": "not-claimed",
    }


def _expected_compatibility_boundary() -> Mapping[str, Any]:
    return {
        "closes": "accepted-source-cli-runtime-inventory-slice",
        "does_not_prove": [
            "published-wheel-or-sdist-runtime-parity",
            "full-handler-runtime-parity",
            "rust-full-cli-implementation",
            "cross-platform-runtime-parity",
            "orchestrator-agent-runtime-parity",
            "plugin-or-marketplace-installation",
            "zero-dependency-native-distribution",
            "ctr-201-parent-completion",
            "ctr-202-completion",
            "fnd-202-implementation",
        ],
        "remaining_ctr_201_blocker": (
            "accepted-source-orchestrator-runtime-closure"
        ),
        "archive_parity": (
            "unassigned-downstream-governance-boundary-not-ctr-201e-evidence"
        ),
    }


def _verify_tag_binding(repo_root: Path) -> None:
    commands = (
        (["git", "rev-parse", f"{ACCEPTED_TAG}^{{tag}}"], ACCEPTED_TAG_OBJECT),
        (["git", "rev-parse", f"{ACCEPTED_TAG}^{{commit}}"], ACCEPTED_COMMIT),
    )
    for command, expected in commands:
        try:
            completed = subprocess.run(
                command,
                cwd=repo_root,
                text=True,
                encoding="utf-8",
                errors="strict",
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                check=False,
            )
        except OSError as error:
            raise RuntimeExtractorError("accepted tag reader is unavailable") from error
        if completed.returncode != 0 or completed.stderr or completed.stdout.strip() != expected:
            raise RuntimeInventoryMismatch("accepted tag identity drifted")


def _read_bound_sources(
    repo_root: Path,
) -> tuple[
    Mapping[str, Any],
    list[Mapping[str, Any]],
    list[Mapping[str, Any]],
    Mapping[str, Any],
    Mapping[str, Any],
]:
    manifest, python_entries = static_cli._read_manifest(repo_root)
    _verify_tag_binding(repo_root)
    receipt = manifest.get("acceptance_receipt")
    if not isinstance(receipt, Mapping) or any(
        receipt.get(key) != value for key, value in ACCEPTANCE_RECEIPT.items()
    ) or receipt.get("status") != "finalized":
        raise RuntimeInventoryMismatch("accepted receipt binding drifted")
    package_trees = manifest.get("package_trees")
    if not isinstance(package_trees, list):
        raise RuntimeExtractorError("accepted package trees are unavailable")
    npm_matches = [
        item
        for item in package_trees
        if isinstance(item, Mapping) and item.get("root") == NPM_PACKAGE_ROOT
    ]
    if len(npm_matches) != 1:
        raise RuntimeExtractorError("accepted npm package tree is not unique")
    npm_tree = npm_matches[0]
    if (
        npm_tree.get("file_count") != NPM_PACKAGE_FILE_COUNT
        or npm_tree.get("tree_sha256") != NPM_PACKAGE_TREE_SHA256
        or not isinstance(npm_tree.get("files"), list)
    ):
        raise RuntimeInventoryMismatch("accepted npm package tree drifted")
    npm_entries: list[Mapping[str, Any]] = []
    for raw in npm_tree["files"]:
        if not isinstance(raw, Mapping):
            raise RuntimeExtractorError("accepted npm package entry is invalid")
        path = static_cli._canonical_repository_path(raw.get("path"), prefix=NPM_PACKAGE_ROOT)
        entry = {
            "path": path,
            "git_blob_oid": raw.get("git_blob_oid"),
            "sha256": raw.get("sha256"),
            "size_bytes": raw.get("size_bytes"),
            "mode": raw.get("mode"),
        }
        if (
            not isinstance(entry["git_blob_oid"], str)
            or not re.fullmatch(r"[0-9a-f]{40}", entry["git_blob_oid"])
            or not isinstance(entry["sha256"], str)
            or not HEX_64.fullmatch(entry["sha256"])
            or not isinstance(entry["size_bytes"], int)
            or isinstance(entry["size_bytes"], bool)
            or entry["size_bytes"] < 0
            or entry["mode"] not in {"100644", "100755"}
        ):
            raise RuntimeExtractorError("accepted npm package entry is invalid")
        npm_entries.append(entry)

    static_artifact = static_cli.extract_cli_inventory(repo_root)
    checked_static = static_cli._load_checked_output(repo_root / STATIC_ARTIFACT_RELATIVE)
    if _canonical_json_bytes(static_artifact) != _canonical_json_bytes(checked_static):
        raise RuntimeInventoryMismatch("CTR-201B checked artifact drifted")

    oracle_path = repo_root / PYTHON_ORACLE_RELATIVE
    try:
        oracle_bytes = oracle_path.read_bytes()
    except OSError as error:
        raise RuntimeExtractorError("accepted Python oracle is unavailable") from error
    if _sha256(oracle_bytes) != PYTHON_ORACLE_SHA256:
        raise RuntimeInventoryMismatch("accepted Python oracle digest drifted")
    oracle = static_cli._load_json_bytes(oracle_bytes)
    return manifest, python_entries, npm_entries, static_artifact, oracle


def _tree_digest(root: Path) -> str:
    records: list[Mapping[str, Any]] = []
    for path in sorted(root.rglob("*"), key=lambda item: item.as_posix()):
        relative = path.relative_to(root).as_posix()
        if path.is_symlink():
            raise RuntimeExtractorError("runtime materialization contains a symbolic link")
        if path.is_dir():
            continue
        if not path.is_file():
            raise RuntimeExtractorError("runtime materialization contains a special file")
        payload = path.read_bytes()
        records.append(
            {"path": relative, "size_bytes": len(payload), "sha256": _sha256(payload)}
        )
    return _sha256(_canonical_json_bytes(records))


def _runtime_environment(temp_root: Path, variant: str) -> dict[str, str]:
    environment = static_cli._worker_environment(temp_root, variant)
    environment.update(
        {
            "COLUMNS": "80",
            "LINES": "24",
            "LANG": "C.UTF-8",
            "LC_ALL": "C.UTF-8",
            "TZ": "UTC",
            "NO_COLOR": "1",
            "PYTHONHASHSEED": "1" if variant == "a" else "731",
            "QIONGLI_CONFIG_HOME": str(temp_root / f"state-{variant}" / "config"),
            "CTR201E_CANARY_SECRET": CANARY_SECRET,
        }
    )
    if os.environ.get("CTR201E_WORKER_DEBUG") == "1":
        environment["CTR201E_WORKER_DEBUG"] = "1"
    return environment


def _normalization_replacements(control: Mapping[str, Any]) -> list[tuple[str, str]]:
    candidates = [
        (str(control["source_root"]), "<RUNTIME_SOURCE>"),
        (str(control["accepted_root"]), "<ACCEPTED_ROOT>"),
        (str(control["config_root"]), "<CONFIG_ROOT>"),
        (str(control["home"]), "<HOME>"),
        (str(control["cwd"]), "<CWD>"),
        (str(control["write_root"]), "<SANDBOX>"),
    ]
    return sorted(candidates, key=lambda item: len(item[0]), reverse=True)


def _normalize_text(value: str, replacements: Sequence[tuple[str, str]]) -> str:
    normalized = value.replace("\r\n", "\n").replace("\r", "\n")
    for source, replacement in replacements:
        normalized = normalized.replace(source, replacement)
        normalized = normalized.replace(source.replace("\\", "/"), replacement)
        normalized = normalized.replace(source.replace("/", "\\"), replacement)
    if CANARY_SECRET in normalized:
        raise RuntimeExtractorError("runtime output contains the canary secret")
    return normalized


def _outcome_sha256(outcome: Mapping[str, Any]) -> str:
    return _sha256(_canonical_json_bytes(outcome))


def _classify_error(
    *, exit_code: int, termination: str, exception_type: str | None
) -> str:
    if exit_code == 0 and termination in {"return", "system-exit"}:
        return "none"
    if termination == "system-exit" and exit_code == 2:
        return "usage-error"
    if termination == "exception" and exception_type is not None:
        return "input-error"
    raise RuntimeInventoryMismatch("runtime outcome has no frozen error classification")


def _filesystem_manifest(
    mutable_roots: Mapping[str, Path],
) -> tuple[dict[str, str], str]:
    manifest: dict[str, str] = {}
    for label, root in sorted(mutable_roots.items()):
        if not re.fullmatch(r"[a-z][a-z0-9-]*", label):
            raise RuntimeExtractorError("runtime mutable-root label is invalid")
        if root.is_symlink() or not root.is_dir():
            raise RuntimeExtractorError("runtime mutable root is invalid")
        for path in sorted(root.rglob("*"), key=lambda item: item.as_posix()):
            if path.is_symlink():
                raise RuntimeExtractorError("runtime mutable root contains a symbolic link")
            if path.is_dir():
                continue
            if not path.is_file():
                raise RuntimeExtractorError("runtime mutable root contains a special file")
            relative = path.relative_to(root).as_posix()
            key = f"{label}/{relative}"
            if key in manifest:
                raise RuntimeExtractorError("runtime mutable-root manifest collides")
            manifest[key] = _sha256(path.read_bytes())
    records = [{"path": path, "sha256": digest} for path, digest in manifest.items()]
    return manifest, _sha256(_canonical_json_bytes(records))


def _observe_runtime_call(
    callback: Callable[[], Any],
    *,
    replacements: Sequence[tuple[str, str]],
    mutable_roots: Mapping[str, Path],
    json_expected: bool = False,
) -> tuple[Mapping[str, Any], Mapping[str, Any]]:
    before, before_digest = _filesystem_manifest(mutable_roots)
    outcome = _capture_call(
        callback,
        replacements=replacements,
        json_expected=json_expected,
    )
    after, after_digest = _filesystem_manifest(mutable_roots)
    created = sorted(set(after) - set(before))
    deleted = sorted(set(before) - set(after))
    modified = sorted(
        path for path in set(before) & set(after) if before[path] != after[path]
    )
    if created or deleted or modified or before_digest != after_digest:
        raise RuntimeInventoryMismatch("safe runtime case changed a measured mutable root")
    effects = {
        "filesystem": {
            "class": "none",
            "before_tree_sha256": before_digest,
            "after_tree_sha256": after_digest,
            "created": created,
            "modified": modified,
            "deleted": deleted,
            "writes_outside_sandbox": False,
        },
        "network": "denied-by-python-audit-hook",
        "process": "denied-by-python-audit-hook",
        "browser": "denied-by-python-audit-hook",
    }
    return outcome, effects


def _capture_call(
    callback: Callable[[], Any],
    *,
    replacements: Sequence[tuple[str, str]],
    json_expected: bool = False,
) -> Mapping[str, Any]:
    stdout = io.StringIO()
    stderr = io.StringIO()
    termination = "return"
    exception_type: str | None = None
    exception_message: str | None = None
    try:
        with redirect_stdout(stdout), redirect_stderr(stderr):
            result = callback()
        exit_code = 0 if result is None else int(result)
    except SystemExit as error:
        termination = "system-exit"
        code = error.code
        exit_code = int(code) if isinstance(code, int) and not isinstance(code, bool) else 1
    except Exception as error:  # Captures the actual console-wrapper failure boundary.
        termination = "exception"
        exit_code = 1
        exception_type = f"{type(error).__module__}.{type(error).__name__}"
        exception_message = _normalize_text(str(error), replacements)
    stdout_text = _normalize_text(stdout.getvalue(), replacements)
    stderr_text = _normalize_text(stderr.getvalue(), replacements)
    if "Traceback (most recent call last)" in stdout_text + stderr_text:
        raise RuntimeExtractorError("runtime outcome contains an unnormalized traceback")
    if len(stdout_text) > 200_000 or len(stderr_text) > 200_000:
        raise RuntimeExtractorError("runtime outcome exceeds the output limit")
    outcome: dict[str, Any] = {
        "exit_code": exit_code,
        "termination": termination,
        "error_class": _classify_error(
            exit_code=exit_code,
            termination=termination,
            exception_type=exception_type,
        ),
        "exception_type": exception_type,
        "exception_message": exception_message,
        "stdout_lines": stdout_text.splitlines(),
        "stdout_terminated": stdout_text.endswith("\n"),
        "stderr_lines": stderr_text.splitlines(),
        "stderr_terminated": stderr_text.endswith("\n"),
        "json": {
            "disposition": "not-applicable",
            "canonical_json": None,
            "canonical_sha256": None,
        },
    }
    if json_expected:
        if exit_code != 0 or stderr_text or termination not in {"return", "system-exit"}:
            raise RuntimeInventoryMismatch("JSON runtime case did not complete successfully")
        try:
            json_value = json.loads(stdout_text)
        except json.JSONDecodeError as error:
            raise RuntimeInventoryMismatch("JSON runtime case emitted invalid JSON") from error
        canonical_json = _canonical_json_bytes(json_value).decode("utf-8")
        outcome["json"] = {
            "disposition": "captured",
            "canonical_json": canonical_json,
            "canonical_sha256": _sha256(canonical_json.encode("utf-8")),
        }
    return outcome


def _python_route(
    qiongli_cli: Any,
    mcp_cli: Any,
    arguments: Sequence[str],
) -> None:
    main_parser = qiongli_cli.build_parser()
    parsed = main_parser.parse_args(list(arguments))
    if getattr(parsed, "cmd", None) == "mcp" and getattr(parsed, "mcp_args", None):
        mcp_cli.build_parser().parse_args(list(parsed.mcp_args))


def _runtime_case(
    *,
    case_id: str,
    layer: str,
    invocation: Mapping[str, Any],
    outcome: Mapping[str, Any],
    effects: Mapping[str, Any],
    entrypoint_observations: Mapping[str, str] | None = None,
) -> Mapping[str, Any]:
    case: dict[str, Any] = {
        "id": case_id,
        "layer": layer,
        "invocation": dict(invocation),
        "outcome": dict(outcome),
        "effects": dict(effects),
        "entrypoint_observations": (
            {
                entrypoint: (
                    entrypoint_observations.get(entrypoint)
                    if entrypoint_observations is not None
                    else None
                )
                for entrypoint in EXPECTED_ENTRYPOINTS
            }
        ),
        "source_case_id": None,
        "source_case_sha256": None,
    }
    return case


def _public_command_rows(static_artifact: Mapping[str, Any]) -> list[Mapping[str, Any]]:
    commands = static_artifact.get("commands")
    if not isinstance(commands, list):
        raise RuntimeExtractorError("CTR-201B command inventory is unavailable")
    rows: list[Mapping[str, Any]] = []
    for command in commands:
        if not isinstance(command, Mapping) or not isinstance(command.get("path"), list):
            raise RuntimeExtractorError("CTR-201B command inventory is invalid")
        canonical = list(command["path"])
        aliases = command.get("aliases")
        if not isinstance(aliases, list) or any(not isinstance(alias, str) for alias in aliases):
            raise RuntimeExtractorError("CTR-201B command aliases are invalid")
        json_capable = any(
            isinstance(argument, Mapping) and argument.get("destination") == "json"
            for argument in command.get("arguments", [])
        )
        dry_run_capable = any(
            isinstance(argument, Mapping) and argument.get("destination") == "dry_run"
            for argument in command.get("arguments", [])
        )
        executable = command.get("subcommand_metadata") is None and command.get("delegate") is None
        rows.append(
            {
                "public_path": canonical,
                "canonical_path": canonical,
                "path_kind": "canonical",
                "json_capable": json_capable,
                "dry_run_capable": dry_run_capable,
                "executable": executable,
            }
        )
        for alias in aliases:
            rows.append(
                {
                    "public_path": [*canonical[:-1], alias],
                    "canonical_path": canonical,
                    "path_kind": "alias",
                    "json_capable": json_capable,
                    "dry_run_capable": dry_run_capable,
                    "executable": executable,
                }
            )
    return rows


def _case_slug(path: Sequence[str]) -> str:
    return ".".join(path)


def _python_worker_artifact(control: Mapping[str, Any]) -> Mapping[str, Any]:
    static_cli._require_python_312()
    required = {
        "accepted_root",
        "source_root",
        "cwd",
        "write_root",
        "config_root",
        "home",
        "mutable_roots",
        "public_commands",
        "entrypoints",
    }
    if set(control) != required:
        raise RuntimeExtractorError("runtime worker control shape is invalid")
    for key in required - {"public_commands", "entrypoints", "mutable_roots"}:
        if not isinstance(control[key], str):
            raise RuntimeExtractorError("runtime worker path is invalid")
    if not isinstance(control["public_commands"], list) or not isinstance(control["entrypoints"], list):
        raise RuntimeExtractorError("runtime worker matrix is invalid")
    accepted_root = Path(control["accepted_root"]).resolve()
    source_root = Path(control["source_root"]).resolve()
    cwd = Path(control["cwd"]).resolve()
    write_root = Path(control["write_root"]).resolve()
    if (
        not static_cli._path_is_within(accepted_root, write_root)
        or not static_cli._path_is_within(source_root, accepted_root)
        or not static_cli._path_is_within(cwd, write_root)
    ):
        raise RuntimeExtractorError("runtime worker paths escape the sandbox")
    raw_mutable_roots = control["mutable_roots"]
    if not isinstance(raw_mutable_roots, Mapping) or set(raw_mutable_roots) != {
        "cwd",
        "state",
    }:
        raise RuntimeExtractorError("runtime mutable-root control is invalid")
    mutable_roots: dict[str, Path] = {}
    for label, raw_path in raw_mutable_roots.items():
        if not isinstance(raw_path, str):
            raise RuntimeExtractorError("runtime mutable-root path is invalid")
        path = Path(raw_path).resolve()
        if (
            not static_cli._path_is_within(path, write_root)
            or static_cli._path_is_within(path, accepted_root)
        ):
            raise RuntimeExtractorError("runtime mutable root escapes its boundary")
        mutable_roots[label] = path
    os.chdir(cwd)
    sys.dont_write_bytecode = True
    static_cli._install_worker_audit_hook(write_root)
    base_sys_path = tuple(sys.path)
    modules_before = set(sys.modules)
    if "yaml" in sys.modules:
        raise RuntimeExtractorError("runtime worker preloaded ambient YAML")
    yaml_stub = static_cli._make_yaml_deny_use_stub()
    sys.modules["yaml"] = yaml_stub
    sys.path.insert(0, str(source_root))
    try:
        from qiongli import cli as qiongli_cli
        from bridges import mcp_cli
    except Exception as error:
        raise RuntimeExtractorError("accepted runtime modules could not be imported") from error
    static_cli._verify_module_provenance(source_root)
    static_cli._verify_dependency_isolation(
        source_root,
        base_sys_path,
        modules_before,
        yaml_stub,
    )
    replacements = _normalization_replacements(control)
    source_tree_sha256 = _tree_digest(accepted_root)
    cases: list[Mapping[str, Any]] = []

    entrypoints = list(control["entrypoints"])
    if entrypoints != list(EXPECTED_ENTRYPOINTS):
        raise RuntimeInventoryMismatch("runtime worker entrypoints differ from CTR-201B")
    entrypoint_rows: list[Mapping[str, Any]] = []
    for entrypoint in entrypoints:
        sys.argv[:] = [entrypoint, "--help"]
        root_help, root_effects = _observe_runtime_call(
            qiongli_cli.main,
            replacements=replacements,
            mutable_roots=mutable_roots,
        )
        if root_help["exit_code"] != 0 or root_help["error_class"] != "none":
            raise RuntimeInventoryMismatch("console entrypoint root help failed")
        root_case_id = f"python.entrypoint.{entrypoint}.root-help"
        cases.append(
            _runtime_case(
                case_id=root_case_id,
                layer="python-console-entrypoint",
                invocation={"entrypoint": entrypoint, "arguments": ["--help"]},
                outcome=root_help,
                effects=root_effects,
            )
        )
        sys.argv[:] = [entrypoint, "align"]
        align, align_effects = _observe_runtime_call(
            qiongli_cli.main,
            replacements=replacements,
            mutable_roots=mutable_roots,
        )
        if align["exit_code"] != 0 or align["error_class"] != "none":
            raise RuntimeInventoryMismatch("console entrypoint align failed")
        align_case_id = f"python.entrypoint.{entrypoint}.align"
        cases.append(
            _runtime_case(
                case_id=align_case_id,
                layer="python-console-entrypoint",
                invocation={"entrypoint": entrypoint, "arguments": ["align"]},
                outcome=align,
                effects=align_effects,
            )
        )
        entrypoint_rows.append(
            {
                "name": entrypoint,
                "target": "qiongli.cli:main",
                "root_help_case_id": root_case_id,
                "align_case_id": align_case_id,
                "root_help_sha256": _outcome_sha256(root_help),
                "align_sha256": _outcome_sha256(align),
            }
        )

    public_rows: list[Mapping[str, Any]] = []
    for row in control["public_commands"]:
        if not isinstance(row, Mapping):
            raise RuntimeExtractorError("runtime public command row is invalid")
        public_path = row.get("public_path")
        canonical_path = row.get("canonical_path")
        if (
            not isinstance(public_path, list)
            or not isinstance(canonical_path, list)
            or any(not isinstance(item, str) for item in [*public_path, *canonical_path])
            or not public_path
            or public_path[0] != "qiongli"
        ):
            raise RuntimeExtractorError("runtime public command path is invalid")
        arguments = public_path[1:]
        help_observations: dict[str, str] = {}
        qiongli_help: Mapping[str, Any] | None = None
        qiongli_help_effects: Mapping[str, Any] | None = None
        for entrypoint in entrypoints:
            sys.argv[:] = [entrypoint]
            help_outcome, help_effects = _observe_runtime_call(
                lambda args=[*arguments, "--help"]: _python_route(
                    qiongli_cli, mcp_cli, args
                ),
                replacements=replacements,
                mutable_roots=mutable_roots,
            )
            if help_outcome["exit_code"] != 0 or help_outcome["error_class"] != "none":
                raise RuntimeInventoryMismatch("public command help did not exit successfully")
            help_observations[entrypoint] = _outcome_sha256(help_outcome)
            if entrypoint == "qiongli":
                qiongli_help = help_outcome
                qiongli_help_effects = help_effects
        if qiongli_help is None or qiongli_help_effects is None:
            raise RuntimeExtractorError("qiongli help observation is missing")
        slug = _case_slug(public_path)
        help_case_id = f"python.help.{slug}"
        cases.append(
            _runtime_case(
                case_id=help_case_id,
                layer="python-parser-route",
                invocation={"entrypoint": "qiongli", "arguments": [*arguments, "--help"]},
                outcome=qiongli_help,
                effects=qiongli_help_effects,
                entrypoint_observations=help_observations,
            )
        )
        sys.argv[:] = ["qiongli"]
        invalid, invalid_effects = _observe_runtime_call(
            lambda args=[*arguments, "--ctr201e-invalid-option"]: _python_route(
                qiongli_cli, mcp_cli, args
            ),
            replacements=replacements,
            mutable_roots=mutable_roots,
        )
        if invalid["exit_code"] != 2 or invalid["error_class"] != "usage-error":
            raise RuntimeInventoryMismatch("public command invalid usage did not exit 2")
        invalid_case_id = f"python.invalid-usage.{slug}"
        cases.append(
            _runtime_case(
                case_id=invalid_case_id,
                layer="python-parser-route",
                invocation={
                    "entrypoint": "qiongli",
                    "arguments": [*arguments, "--ctr201e-invalid-option"],
                },
                outcome=invalid,
                effects=invalid_effects,
            )
        )
        zero_argument_case_id: str | None = None
        if tuple(public_path) in static_cli.EXPECTED_ZERO_ARGUMENT_COMMANDS:
            sys.argv[:] = ["qiongli"]
            zero_argument, zero_argument_effects = _observe_runtime_call(
                lambda args=list(arguments): _python_route(qiongli_cli, mcp_cli, args),
                replacements=replacements,
                mutable_roots=mutable_roots,
            )
            if zero_argument["exit_code"] != 2 or zero_argument["error_class"] != "usage-error":
                raise RuntimeInventoryMismatch("zero-argument group boundary did not exit 2")
            zero_argument_case_id = f"python.zero-argument.{slug}"
            cases.append(
                _runtime_case(
                    case_id=zero_argument_case_id,
                    layer="python-parser-route",
                    invocation={"entrypoint": "qiongli", "arguments": list(arguments)},
                    outcome=zero_argument,
                    effects=zero_argument_effects,
                )
            )
        public_rows.append(
            {
                **dict(row),
                "help": {
                    "disposition": "captured",
                    "case_ids": [help_case_id],
                    "reason_code": "accepted-parser-route",
                    "decision_id": None,
                },
                "behavior": _behavior_disposition(public_path, bool(row.get("executable"))),
                "stdout_stderr": _runtime_boundary_disposition(
                    public_path,
                    bool(row.get("executable")),
                    "stdout-stderr",
                ),
                "exit_code": _runtime_boundary_disposition(
                    public_path,
                    bool(row.get("executable")),
                    "exit-code",
                ),
                "json": _json_disposition(public_path, bool(row.get("json_capable"))),
                "dry_run": _dry_run_disposition(public_path, bool(row.get("dry_run_capable"))),
                "zero_argument": _zero_argument_disposition(
                    public_path, zero_argument_case_id
                ),
                "error": _runtime_boundary_disposition(
                    public_path,
                    bool(row.get("executable")),
                    "error",
                ),
                "side_effects": _runtime_boundary_disposition(
                    public_path,
                    bool(row.get("executable")),
                    "side-effects",
                ),
            }
        )

    safe_cases = (
        (
            "python.handler.provider-list-json",
            ["provider", "list", "--json"],
            True,
        ),
        (
            "python.handler.mcp-config-example-json",
            ["mcp", "config", "example", "--target", "codex", "--json"],
            True,
        ),
    )
    for case_id, arguments, json_expected in safe_cases:
        sys.argv[:] = ["qiongli", *arguments]
        outcome, handler_effects = _observe_runtime_call(
            qiongli_cli.main,
            replacements=replacements,
            mutable_roots=mutable_roots,
            json_expected=json_expected,
        )
        cases.append(
            _runtime_case(
                case_id=case_id,
                layer="python-safe-handler",
                invocation={"entrypoint": "qiongli", "arguments": arguments},
                outcome=outcome,
                effects=handler_effects,
            )
        )

    sys.argv[:] = [
        "qiongli",
        "provider",
        "set",
        "arxiv",
        "api-key",
        "invalid-value",
    ]
    domain_error, domain_error_effects = _observe_runtime_call(
        qiongli_cli.main,
        replacements=replacements,
        mutable_roots=mutable_roots,
    )
    if domain_error["exit_code"] != 1 or domain_error["error_class"] != "input-error":
        raise RuntimeInventoryMismatch("representative domain error boundary drifted")
    cases.append(
        _runtime_case(
            case_id="python.handler.provider-invalid-field",
            layer="python-safe-handler",
            invocation={
                "entrypoint": "qiongli",
                "arguments": [
                    "provider",
                    "set",
                    "arxiv",
                    "api-key",
                    "invalid-value",
                ],
            },
            outcome=domain_error,
            effects=domain_error_effects,
        )
    )
    return {
        "console_entrypoints": entrypoint_rows,
        "public_commands": public_rows,
        "cases": cases,
        "source_tree_sha256": source_tree_sha256,
    }


def _dimension(
    disposition: str,
    case_ids: Sequence[str],
    reason_code: str,
    decision_id: str | None = None,
) -> Mapping[str, Any]:
    if (disposition == "explicit-disposition") != (decision_id is not None):
        raise RuntimeInventoryMismatch("runtime disposition decision binding is invalid")
    if decision_id is not None and decision_id not in DISPOSITION_DECISION_IDS:
        raise RuntimeInventoryMismatch("runtime disposition decision is unknown")
    return {
        "disposition": disposition,
        "case_ids": list(case_ids),
        "reason_code": reason_code,
        "decision_id": decision_id,
    }


_UNSAFE_HANDLER_PATHS = frozenset(
    {
        ("qiongli", "upgrade"),
        ("qiongli", "self-update"),
        ("qiongli", "update"),
        ("qiongli", "setup"),
        ("qiongli", "provider", "setup"),
        ("qiongli", "mcp", "serve"),
        ("qiongli", "mcp", "upgrade"),
        ("qiongli", "mcp", "configure"),
        ("qiongli", "mcp", "wizard"),
    }
)


def _handler_decision_id(path: Sequence[str]) -> str:
    return "CTR-201E-D002" if tuple(path) in _UNSAFE_HANDLER_PATHS else "CTR-201E-D001"


def _captured_handler_case_id(path: Sequence[str]) -> str | None:
    key = tuple(path)
    return {
        ("qiongli", "align"): "python.entrypoint.qiongli.align",
        ("qiongli", "install"): "a8.python.installer-dry-run",
        ("qiongli", "provider", "list"): "python.handler.provider-list-json",
        (
            "qiongli",
            "mcp",
            "config",
            "example",
        ): "python.handler.mcp-config-example-json",
    }.get(key)


def _runtime_boundary_disposition(
    path: Sequence[str], executable: bool, field: str
) -> Mapping[str, Any]:
    slug = _case_slug(path)
    parser_cases = [f"python.help.{slug}", f"python.invalid-usage.{slug}"]
    if not executable:
        return _dimension(
            "captured",
            parser_cases,
            f"parser-{field}-boundary",
        )
    handler_case_id = _captured_handler_case_id(path)
    if handler_case_id is not None:
        disposition = (
            "accepted-oracle"
            if handler_case_id.startswith("a8.")
            else "captured"
        )
        return _dimension(
            disposition,
            [*parser_cases, handler_case_id],
            f"captured-handler-{field}-boundary",
        )
    extra_cases = (
        ["python.handler.provider-invalid-field"]
        if tuple(path) == ("qiongli", "provider", "set")
        else []
    )
    return _dimension(
        "explicit-disposition",
        [*parser_cases, *extra_cases],
        f"handler-{field}-requires-leg-201-fixture",
        _handler_decision_id(path),
    )


def _behavior_disposition(path: Sequence[str], executable: bool) -> Mapping[str, Any]:
    key = tuple(path)
    if not executable:
        return _dimension(
            "captured",
            [f"python.invalid-usage.{_case_slug(path)}"],
            "required-subcommand-or-delegate-boundary",
        )
    captured_case_id = _captured_handler_case_id(path)
    if captured_case_id is not None and not captured_case_id.startswith("a8."):
        return _dimension(
            "captured",
            [captured_case_id],
            "safe-bounded-handler",
        )
    if captured_case_id is not None:
        return _dimension(
            "accepted-oracle",
            [captured_case_id],
            "accepted-a8-installer-dry-run",
        )
    return _dimension(
        "explicit-disposition",
        [],
        "handler-runtime-requires-separate-bounded-fixture",
        _handler_decision_id(path),
    )


def _json_disposition(path: Sequence[str], capable: bool) -> Mapping[str, Any]:
    if not capable:
        return _dimension("not-applicable", [], "no-json-argument")
    key = tuple(path)
    if key == ("qiongli", "provider", "list"):
        return _dimension(
            "captured",
            ["python.handler.provider-list-json"],
            "safe-empty-provider-state",
        )
    if key == ("qiongli", "mcp", "config", "example"):
        return _dimension(
            "captured",
            ["python.handler.mcp-config-example-json"],
            "safe-static-config-example",
        )
    if key in {
        ("qiongli", "check"),
        ("qiongli", "mcp", "doctor"),
    }:
        reason = "requires-authenticated-offline-read-fixture"
    elif key in {
        ("qiongli", "mcp", "wizard"),
    }:
        reason = "listener-and-browser-runtime-excluded"
    elif key in {
        ("qiongli", "mcp", "configure"),
    }:
        reason = "secret-bearing-bounded-write-excluded"
    else:
        reason = "project-state-bounded-write-fixture-required"
    decision_id = "CTR-201E-D002" if key in _UNSAFE_HANDLER_PATHS else "CTR-201E-D001"
    return _dimension("explicit-disposition", [], reason, decision_id)


def _dry_run_disposition(path: Sequence[str], capable: bool) -> Mapping[str, Any]:
    if not capable:
        return _dimension("not-applicable", [], "no-dry-run-argument")
    key = tuple(path)
    if key == ("qiongli", "install"):
        return _dimension(
            "accepted-oracle",
            ["a8.python.installer-dry-run"],
            "accepted-a8-no-filesystem-delta",
        )
    if key in {
        ("qiongli", "upgrade"),
        ("qiongli", "mcp", "upgrade"),
    }:
        reason = "downloads-before-install-even-in-dry-run"
    elif key == ("qiongli", "setup"):
        reason = "fixed-interactive-transcript-required"
    elif key in {
        ("qiongli", "self-update"),
        ("qiongli", "update"),
    }:
        reason = "self-update-process-plan-fixture-required"
    else:
        reason = "bounded-filesystem-fixture-required"
    return _dimension(
        "explicit-disposition",
        [],
        reason,
        _handler_decision_id(path),
    )


def _zero_argument_disposition(
    path: Sequence[str], case_id: str | None
) -> Mapping[str, Any]:
    if tuple(path) in static_cli.EXPECTED_ZERO_ARGUMENT_COMMANDS:
        if case_id is None:
            raise RuntimeInventoryMismatch("zero-argument case binding is missing")
        return _dimension(
            "captured", [case_id], "required-subcommand-zero-argument-boundary"
        )
    return _dimension("not-applicable", [], "not-static-zero-argument-command")


def _capture_python_once(
    repo_root: Path,
    python_entries: Sequence[Mapping[str, Any]],
    public_commands: Sequence[Mapping[str, Any]],
    variant: str,
) -> Mapping[str, Any]:
    blobs = static_cli._cat_file_blobs(repo_root, python_entries)
    with tempfile.TemporaryDirectory(prefix=f"qiongli-ctr201e-python-{variant}-") as raw:
        temp_root = Path(raw)
        accepted_root = temp_root / "accepted"
        accepted_root.mkdir()
        source_root = static_cli._write_materialized_tree(accepted_root, blobs)
        cwd = temp_root / f"cwd-{variant}"
        cwd.mkdir()
        environment = _runtime_environment(temp_root, variant)
        for key in static_cli.WORKER_DIRECTORY_ENV_KEYS:
            Path(environment[key]).mkdir(parents=True, exist_ok=True)
        Path(environment["QIONGLI_CONFIG_HOME"]).mkdir(parents=True, exist_ok=True)
        control = {
            "accepted_root": str(accepted_root),
            "source_root": str(source_root),
            "cwd": str(cwd),
            "write_root": str(temp_root),
            "config_root": environment["QIONGLI_CONFIG_HOME"],
            "home": environment["HOME"],
            "mutable_roots": {
                "cwd": str(cwd),
                "state": str(Path(environment["HOME"]).parent),
            },
            "public_commands": [dict(row) for row in public_commands],
            "entrypoints": list(EXPECTED_ENTRYPOINTS),
        }
        control_path = temp_root / "control.json"
        control_path.write_bytes(_canonical_json_bytes(control))
        command = [
            sys.executable,
            "-I",
            "-S",
            "-B",
            str(Path(__file__).resolve()),
            "--_python-worker",
            str(control_path),
        ]
        try:
            completed = subprocess.run(
                command,
                cwd=cwd,
                env=environment,
                text=True,
                encoding="utf-8",
                errors="strict",
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                timeout=45,
                check=False,
            )
        except (OSError, subprocess.TimeoutExpired) as error:
            raise RuntimeExtractorError("isolated Python runtime worker failed to run") from error
        if completed.stderr:
            if os.environ.get("CTR201E_WORKER_DEBUG") == "1" and completed.stderr:
                sys.stderr.write(completed.stderr)
            raise RuntimeExtractorError("isolated Python runtime worker failed")
        payload = static_cli._load_json_bytes(completed.stdout.encode("utf-8"))
        if completed.returncode == 1 and payload.get("status") == "fail":
            raise RuntimeInventoryMismatch("isolated Python runtime capture drifted")
        if completed.returncode != 0:
            raise RuntimeExtractorError("isolated Python runtime worker failed")
        if payload.get("status") != "pass" or not isinstance(payload.get("artifact"), Mapping):
            raise RuntimeExtractorError("isolated Python runtime worker returned no artifact")
        artifact = payload["artifact"]
        rendered = _canonical_json_bytes(artifact).decode("utf-8")
        if CANARY_SECRET in rendered or str(temp_root) in rendered:
            raise RuntimeExtractorError("isolated Python artifact leaked sandbox state")
        return artifact


NODE_RUNNER = r"""
import { pathToFileURL } from 'node:url';
import { join } from 'node:path';

const root = process.argv[2];
const { parseArgv } = await import(pathToFileURL(join(root, 'lib', 'args.mjs')).href);
const rawCommands = [
  'help', 'install', 'setup', 'update', 'refresh', 'upgrade', 'remove',
  'uninstall', 'delete', 'check', 'clean', 'runtime', 'project', 'doctor',
  'task-run', 'mcp', 'provider', 'guidance', 'customize', 'init', 'align',
  'unknown-command',
];
const dispatch = rawCommands.map((raw) => {
  const parsed = parseArgv([raw]);
  return {
    raw_command: raw,
    normalized_command: parsed.command,
    overwrite: parsed.options.overwrite,
    rest: parsed.rest,
  };
});
process.stdout.write(JSON.stringify({status: 'pass', dispatch}) + '\n');
""".strip()


def _capture_npm_once(
    repo_root: Path,
    npm_entries: Sequence[Mapping[str, Any]],
    variant: str,
) -> Mapping[str, Any]:
    node_candidate = shutil.which("node")
    if node_candidate is None:
        raise RuntimeExtractorError("Node is required only for the maintainer npm dispatch oracle")
    version_environment = {
        "PATH": "",
        "LANG": "C.UTF-8",
        "LC_ALL": "C.UTF-8",
        "TZ": "UTC",
        "NO_COLOR": "1",
    }
    if os.name == "nt" and os.environ.get("SystemRoot"):
        version_environment["SystemRoot"] = os.environ["SystemRoot"]
    try:
        resolution = subprocess.run(
            [
                node_candidate,
                "-p",
                "JSON.stringify({execPath:process.execPath,version:process.versions.node})",
            ],
            cwd=repo_root,
            env=version_environment,
            text=True,
            encoding="utf-8",
            errors="strict",
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            timeout=10,
            check=False,
        )
    except (OSError, subprocess.TimeoutExpired) as error:
        raise RuntimeExtractorError("Node engine requirement could not be checked") from error
    if resolution.returncode != 0 or resolution.stderr:
        raise RuntimeExtractorError("Node executable could not be resolved safely")
    resolved = static_cli._load_json_bytes(resolution.stdout.encode("utf-8"))
    raw_node = resolved.get("execPath")
    raw_version = resolved.get("version")
    if not isinstance(raw_node, str) or not isinstance(raw_version, str):
        raise RuntimeExtractorError("Node executable identity is invalid")
    node = Path(raw_node)
    try:
        node = node.resolve(strict=True)
    except OSError as error:
        raise RuntimeExtractorError("Node executable identity is unavailable") from error
    match = re.fullmatch(r"([0-9]+)(?:\.[0-9]+){2}", raw_version)
    if (
        not node.is_file()
        or match is None
        or int(match.group(1)) < 18
    ):
        raise RuntimeExtractorError("Node engine does not satisfy the npm parser boundary")
    blobs = static_cli._cat_file_blobs(repo_root, npm_entries)
    args_source = blobs.get("packages/npm-qiongli/lib/args.mjs")
    if args_source is None:
        raise RuntimeExtractorError("accepted npm argument parser is unavailable")
    forbidden = (b"node:http", b"node:https", b"node:net", b"child_process", b"fetch(")
    if any(token in args_source for token in forbidden) or b"import " in args_source:
        raise RuntimeInventoryMismatch("accepted npm parse-only source gained a capability")
    with tempfile.TemporaryDirectory(prefix=f"qiongli-ctr201e-node-{variant}-") as raw:
        temp_root = Path(raw)
        accepted_root = temp_root / "accepted"
        accepted_root.mkdir()
        static_cli._write_materialized_tree(accepted_root, blobs)
        npm_root = accepted_root / "packages" / "npm-qiongli"
        runner = temp_root / "runner.mjs"
        runner.write_text(NODE_RUNNER + "\n", encoding="utf-8", newline="\n")
        before = _tree_digest(accepted_root)
        environment = {
            "HOME": str(temp_root / f"home-{variant}"),
            "USERPROFILE": str(temp_root / f"home-{variant}"),
            "TMP": str(temp_root / f"tmp-{variant}"),
            "TEMP": str(temp_root / f"tmp-{variant}"),
            "TMPDIR": str(temp_root / f"tmp-{variant}"),
            "PATH": "",
            "LANG": "C.UTF-8",
            "LC_ALL": "C.UTF-8",
            "TZ": "UTC",
            "NO_COLOR": "1",
            "CTR201E_CANARY_SECRET": CANARY_SECRET,
        }
        for key in ("HOME", "TMP"):
            Path(environment[key]).mkdir(parents=True, exist_ok=True)
        try:
            completed = subprocess.run(
                [str(node), str(runner), str(npm_root)],
                cwd=temp_root,
                env=environment,
                text=True,
                encoding="utf-8",
                errors="strict",
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                timeout=15,
                check=False,
            )
        except (OSError, subprocess.TimeoutExpired) as error:
            raise RuntimeExtractorError("isolated npm dispatch worker failed to run") from error
        after = _tree_digest(accepted_root)
        if completed.returncode != 0 or completed.stderr or before != after:
            raise RuntimeExtractorError("isolated npm dispatch worker failed")
        payload = static_cli._load_json_bytes(completed.stdout.encode("utf-8"))
        dispatch = payload.get("dispatch")
        if payload.get("status") != "pass" or not isinstance(dispatch, list):
            raise RuntimeExtractorError("isolated npm dispatch worker returned invalid data")
        normalized: list[Mapping[str, Any]] = []
        for row in dispatch:
            if not isinstance(row, Mapping):
                raise RuntimeExtractorError("npm dispatch row is invalid")
            raw_command = row.get("raw_command")
            expected = EXPECTED_NPM_DISPATCH.get(str(raw_command))
            if (
                expected is None
                or row.get("normalized_command") != expected[0]
                or row.get("overwrite") is not expected[1]
                or row.get("rest") != []
            ):
                raise RuntimeInventoryMismatch("accepted npm dispatch mapping drifted")
            normalized.append(dict(row))
        if len(normalized) != len(EXPECTED_NPM_DISPATCH):
            raise RuntimeInventoryMismatch("accepted npm dispatch coverage drifted")
        rendered = _canonical_json_bytes(normalized).decode("utf-8")
        if CANARY_SECRET in rendered or str(temp_root) in rendered:
            raise RuntimeExtractorError("npm dispatch artifact leaked sandbox state")
        return {
            "dispatch": normalized,
            "source_tree_sha256": before,
            "filesystem_unchanged": True,
        }


def _a8_cases(oracle: Mapping[str, Any]) -> list[Mapping[str, Any]]:
    cases = oracle.get("cases")
    if not isinstance(cases, list):
        raise RuntimeExtractorError("accepted Python oracle cases are unavailable")
    required = {
        "python.cli-align": "a8.python.cli-align",
        "python.installer-dry-run": "a8.python.installer-dry-run",
    }
    selected: list[Mapping[str, Any]] = []
    for source_id, target_id in required.items():
        matches = [case for case in cases if isinstance(case, Mapping) and case.get("id") == source_id]
        if len(matches) != 1:
            raise RuntimeInventoryMismatch("accepted Python CLI oracle case is not unique")
        source_case = matches[0]
        outcome = source_case.get("outcome")
        side_effects = source_case.get("side_effects")
        invocation = source_case.get("invocation")
        if not isinstance(outcome, Mapping) or not isinstance(side_effects, Mapping) or not isinstance(invocation, Mapping):
            raise RuntimeExtractorError("accepted Python CLI oracle case is invalid")
        value = outcome.get("value")
        stdout_lines = value.get("stdout_lines", []) if isinstance(value, Mapping) else []
        stderr_lines = value.get("stderr_lines", []) if isinstance(value, Mapping) else []
        if not isinstance(stdout_lines, list) or not isinstance(stderr_lines, list):
            raise RuntimeExtractorError("accepted Python CLI oracle streams are invalid")
        if outcome.get("error") is not None or outcome.get("exit_code") != 0:
            raise RuntimeInventoryMismatch("accepted Python CLI oracle outcome drifted")
        filesystem = side_effects.get("filesystem_delta")
        if not isinstance(filesystem, Mapping):
            raise RuntimeExtractorError("accepted Python CLI oracle effects are invalid")
        selected.append(
            {
                "id": target_id,
                "layer": "accepted-a8-oracle",
                "invocation": {
                    "entrypoint": "python-module" if source_id == "python.cli-align" else "qiongli",
                    "arguments": [str(invocation.get("operation", source_id))],
                },
                "outcome": {
                    "exit_code": outcome.get("exit_code"),
                    "termination": "accepted-oracle",
                    "error_class": "none",
                    "exception_type": None,
                    "exception_message": None,
                    "stdout_lines": stdout_lines,
                    "stdout_terminated": None,
                    "stderr_lines": stderr_lines,
                    "stderr_terminated": None,
                    "json": {
                        "disposition": "not-applicable",
                        "canonical_json": None,
                        "canonical_sha256": None,
                    },
                },
                "effects": {
                    "filesystem": {
                        "class": side_effects.get("class"),
                        "before_tree_sha256": filesystem.get("before_tree_sha256"),
                        "after_tree_sha256": filesystem.get("after_tree_sha256"),
                        "created": filesystem.get("created"),
                        "modified": filesystem.get("modified"),
                        "deleted": filesystem.get("deleted"),
                        "writes_outside_sandbox": side_effects.get("writes_outside_sandbox"),
                    },
                    "network": "not-assessed-by-a8-runtime-fixture",
                    "process": "not-assessed-by-a8-runtime-fixture",
                    "browser": "not-assessed-by-a8-runtime-fixture",
                },
                "entrypoint_observations": {
                    entrypoint: None for entrypoint in EXPECTED_ENTRYPOINTS
                },
                "source_case_id": source_id,
                "source_case_sha256": _canonical_case_sha256(source_case),
            }
        )
    return selected


def _validate_expected_artifact(
    artifact: Mapping[str, Any], repo_root: Path = REPO_ROOT
) -> None:
    try:
        from tooling.scripts.validate_capability_contract import validate_instance
    except ModuleNotFoundError as error:  # Parent-only validation, never the worker.
        raise RuntimeExtractorError("schema validator is unavailable") from error
    try:
        schema = static_cli._load_json_bytes((repo_root / SCHEMA_RELATIVE).read_bytes())
    except OSError as error:
        raise RuntimeExtractorError("CTR-201E schema is unavailable") from error
    schema_errors = validate_instance(artifact, schema)
    if schema_errors:
        raise RuntimeInventoryMismatch("CTR-201E artifact does not match its closed schema")
    if (
        artifact.get("task_id") != "CTR-201E"
        or artifact.get("status") != "runtime-inventory-freeze-captured"
    ):
        raise RuntimeInventoryMismatch("CTR-201E identity is invalid")
    public = artifact.get("public_commands")
    entrypoints = artifact.get("console_entrypoints")
    cases = artifact.get("cases")
    coverage = artifact.get("coverage")
    npm = artifact.get("npm_compatibility")
    if not isinstance(public, list) or len(public) != EXPECTED_COUNTS["public_commands"]:
        raise RuntimeInventoryMismatch("CTR-201E public command coverage is incomplete")
    if not isinstance(entrypoints, list) or len(entrypoints) != EXPECTED_COUNTS["console_entrypoints"]:
        raise RuntimeInventoryMismatch("CTR-201E console entrypoint coverage is incomplete")
    if not isinstance(cases, list) or any(not isinstance(case, Mapping) for case in cases):
        raise RuntimeInventoryMismatch("CTR-201E runtime cases are invalid")
    case_ids = [case.get("id") for case in cases]
    if any(not isinstance(case_id, str) or not case_id for case_id in case_ids) or len(case_ids) != len(set(case_ids)):
        raise RuntimeInventoryMismatch("CTR-201E runtime case IDs are not unique")
    public_paths = [tuple(row.get("public_path", [])) for row in public if isinstance(row, Mapping)]
    if len(public_paths) != len(set(public_paths)):
        raise RuntimeInventoryMismatch("CTR-201E public command paths are not unique")
    checked_static = static_cli._load_checked_output(repo_root / STATIC_ARTIFACT_RELATIVE)
    if artifact.get("source") != _expected_source(repo_root, checked_static):
        raise RuntimeInventoryMismatch("CTR-201E source binding drifted")
    if artifact.get("capture_contract") != EXPECTED_CAPTURE_CONTRACT:
        raise RuntimeInventoryMismatch("CTR-201E capture contract drifted")
    if artifact.get("error_taxonomy") != OBSERVED_ERROR_TAXONOMY:
        raise RuntimeInventoryMismatch("CTR-201E observed error taxonomy drifted")
    if artifact.get("disposition_decisions") != DISPOSITION_DECISIONS:
        raise RuntimeInventoryMismatch("CTR-201E disposition decisions drifted")
    expected_public = _public_command_rows(checked_static)
    projection_keys = (
        "public_path",
        "canonical_path",
        "path_kind",
        "json_capable",
        "dry_run_capable",
        "executable",
    )
    actual_projection = [
        {key: row.get(key) for key in projection_keys}
        for row in public
        if isinstance(row, Mapping)
    ]
    if actual_projection != expected_public:
        raise RuntimeInventoryMismatch("CTR-201E public command projection drifted from CTR-201B")
    case_by_id = {str(case["id"]): case for case in cases}
    expected_case_ids: list[str] = []
    for entrypoint in EXPECTED_ENTRYPOINTS:
        expected_case_ids.extend(
            [
                f"python.entrypoint.{entrypoint}.root-help",
                f"python.entrypoint.{entrypoint}.align",
            ]
        )
    for row in expected_public:
        slug = _case_slug(row["public_path"])
        expected_case_ids.extend(
            [f"python.help.{slug}", f"python.invalid-usage.{slug}"]
        )
        if tuple(row["public_path"]) in static_cli.EXPECTED_ZERO_ARGUMENT_COMMANDS:
            expected_case_ids.append(f"python.zero-argument.{slug}")
    expected_case_ids.extend(
        [
            "python.handler.provider-list-json",
            "python.handler.mcp-config-example-json",
            "python.handler.provider-invalid-field",
            "a8.python.cli-align",
            "a8.python.installer-dry-run",
        ]
    )
    if case_ids != expected_case_ids:
        raise RuntimeInventoryMismatch("CTR-201E runtime case identity or order drifted")
    if [row.get("name") for row in entrypoints] != list(EXPECTED_ENTRYPOINTS):
        raise RuntimeInventoryMismatch("CTR-201E console entrypoint order drifted")
    for row in entrypoints:
        if not isinstance(row, Mapping):
            raise RuntimeInventoryMismatch("CTR-201E console entrypoint row is invalid")
        name = str(row["name"])
        root_case_id = f"python.entrypoint.{name}.root-help"
        align_case_id = f"python.entrypoint.{name}.align"
        if (
            row.get("target") != "qiongli.cli:main"
            or row.get("root_help_case_id") != root_case_id
            or row.get("align_case_id") != align_case_id
            or row.get("root_help_sha256")
            != _outcome_sha256(case_by_id[root_case_id]["outcome"])
            or row.get("align_sha256")
            != _outcome_sha256(case_by_id[align_case_id]["outcome"])
        ):
            raise RuntimeInventoryMismatch(
                "CTR-201E console entrypoint digest binding drifted"
            )
    for row in public:
        if not isinstance(row, Mapping):
            raise RuntimeInventoryMismatch("CTR-201E public command row is invalid")
        for field in (
            "help",
            "behavior",
            "stdout_stderr",
            "exit_code",
            "json",
            "dry_run",
            "zero_argument",
            "error",
            "side_effects",
        ):
            dimension = row.get(field)
            if not isinstance(dimension, Mapping):
                raise RuntimeInventoryMismatch("CTR-201E behavior dimension is missing")
            references = dimension.get("case_ids")
            if not isinstance(references, list) or any(reference not in case_ids for reference in references):
                raise RuntimeInventoryMismatch("CTR-201E case reference is invalid")
        slug = _case_slug(row["public_path"])
        if row["help"] != _dimension(
            "captured", [f"python.help.{slug}"], "accepted-parser-route"
        ):
            raise RuntimeInventoryMismatch("CTR-201E help disposition drifted")
        if row["error"] != _runtime_boundary_disposition(
            row["public_path"], bool(row["executable"]), "error"
        ):
            raise RuntimeInventoryMismatch("CTR-201E error disposition drifted")
        if row["stdout_stderr"] != _runtime_boundary_disposition(
            row["public_path"], bool(row["executable"]), "stdout-stderr"
        ):
            raise RuntimeInventoryMismatch("CTR-201E stream disposition drifted")
        if row["exit_code"] != _runtime_boundary_disposition(
            row["public_path"], bool(row["executable"]), "exit-code"
        ):
            raise RuntimeInventoryMismatch("CTR-201E exit-code disposition drifted")
        if row["behavior"] != _behavior_disposition(
            row["public_path"], bool(row["executable"])
        ):
            raise RuntimeInventoryMismatch("CTR-201E behavior disposition drifted")
        if row["json"] != _json_disposition(
            row["public_path"], bool(row["json_capable"])
        ):
            raise RuntimeInventoryMismatch("CTR-201E JSON disposition drifted")
        if row["dry_run"] != _dry_run_disposition(
            row["public_path"], bool(row["dry_run_capable"])
        ):
            raise RuntimeInventoryMismatch("CTR-201E dry-run disposition drifted")
        zero_case_id = (
            f"python.zero-argument.{slug}"
            if tuple(row["public_path"]) in static_cli.EXPECTED_ZERO_ARGUMENT_COMMANDS
            else None
        )
        if row["zero_argument"] != _zero_argument_disposition(
            row["public_path"], zero_case_id
        ):
            raise RuntimeInventoryMismatch("CTR-201E zero-argument disposition drifted")
        if row["side_effects"] != _runtime_boundary_disposition(
            row["public_path"], bool(row["executable"]), "side-effects"
        ):
            raise RuntimeInventoryMismatch("CTR-201E side-effect disposition drifted")
        for field in (
            "help",
            "behavior",
            "stdout_stderr",
            "exit_code",
            "json",
            "dry_run",
            "zero_argument",
            "error",
            "side_effects",
        ):
            dimension = row[field]
            decision_id = dimension.get("decision_id")
            if (
                dimension.get("disposition") == "explicit-disposition"
                and decision_id not in DISPOSITION_DECISION_IDS
            ) or (
                dimension.get("disposition") != "explicit-disposition"
                and decision_id is not None
            ):
                raise RuntimeInventoryMismatch(
                    "CTR-201E disposition decision linkage drifted"
                )
    expected_case_prefix_counts = {
        "python.entrypoint.": 10,
        "python.help.": EXPECTED_COUNTS["public_commands"],
        "python.invalid-usage.": EXPECTED_COUNTS["public_commands"],
        "python.zero-argument.": EXPECTED_COUNTS["zero_argument_observations"],
        "python.handler.": 3,
        "a8.python.": 2,
    }
    for prefix, expected_count in expected_case_prefix_counts.items():
        if sum(str(case_id).startswith(prefix) for case_id in case_ids) != expected_count:
            raise RuntimeInventoryMismatch("CTR-201E runtime case coverage drifted")
    try:
        oracle = static_cli._load_json_bytes(
            (repo_root / PYTHON_ORACLE_RELATIVE).read_bytes()
        )
    except OSError as error:
        raise RuntimeExtractorError("accepted Python oracle is unavailable") from error
    expected_a8_cases = _a8_cases(oracle)
    if cases[-2:] != expected_a8_cases:
        raise RuntimeInventoryMismatch("CTR-201E accepted A8 case projection drifted")
    observed_error_ids = {row["id"] for row in OBSERVED_ERROR_TAXONOMY}
    if {case["outcome"]["error_class"] for case in cases} != observed_error_ids:
        raise RuntimeInventoryMismatch("CTR-201E observed error evidence drifted")
    for case in cases:
        outcome = case.get("outcome")
        effects = case.get("effects")
        observations = case.get("entrypoint_observations")
        if not isinstance(outcome, Mapping) or not isinstance(effects, Mapping):
            raise RuntimeInventoryMismatch("CTR-201E runtime outcome is invalid")
        error_class = outcome.get("error_class")
        exit_code = outcome.get("exit_code")
        expected_exit = {"none": 0, "usage-error": 2, "input-error": 1}
        if error_class not in expected_exit or exit_code != expected_exit[error_class]:
            raise RuntimeInventoryMismatch("CTR-201E observable error class and exit code disagree")
        json_outcome = outcome.get("json")
        if not isinstance(json_outcome, Mapping):
            raise RuntimeInventoryMismatch("CTR-201E JSON outcome is invalid")
        if json_outcome.get("disposition") == "captured":
            canonical_json = json_outcome.get("canonical_json")
            if not isinstance(canonical_json, str):
                raise RuntimeInventoryMismatch("CTR-201E captured JSON is unavailable")
            try:
                parsed_json = json.loads(canonical_json)
            except json.JSONDecodeError as error:
                raise RuntimeInventoryMismatch("CTR-201E captured JSON is invalid") from error
            if (
                _canonical_json_bytes(parsed_json).decode("utf-8") != canonical_json
                or _sha256(canonical_json.encode("utf-8"))
                != json_outcome.get("canonical_sha256")
            ):
                raise RuntimeInventoryMismatch("CTR-201E captured JSON digest is invalid")
        filesystem = effects.get("filesystem")
        if (
            not isinstance(filesystem, Mapping)
            or filesystem.get("before_tree_sha256") != filesystem.get("after_tree_sha256")
            or filesystem.get("created") != []
            or filesystem.get("modified") != []
            or filesystem.get("deleted") != []
            or filesystem.get("writes_outside_sandbox") is not False
        ):
            raise RuntimeInventoryMismatch("CTR-201E case has an unclassified filesystem delta")
        case_id = str(case["id"])
        if case_id.startswith("python.entrypoint."):
            expected_layer = "python-console-entrypoint"
        elif case_id.startswith(("python.help.", "python.invalid-usage.", "python.zero-argument.")):
            expected_layer = "python-parser-route"
        elif case_id.startswith("python.handler."):
            expected_layer = "python-safe-handler"
        else:
            expected_layer = "accepted-a8-oracle"
        if case.get("layer") != expected_layer:
            raise RuntimeInventoryMismatch("CTR-201E runtime case layer drifted")
        if case_id.startswith("python."):
            if (
                effects.get("network") != "denied-by-python-audit-hook"
                or effects.get("process") != "denied-by-python-audit-hook"
                or effects.get("browser") != "denied-by-python-audit-hook"
                or case.get("source_case_id") is not None
                or case.get("source_case_sha256") is not None
            ):
                raise RuntimeInventoryMismatch("CTR-201E Python effect provenance drifted")
        elif (
            effects.get("network") != "not-assessed-by-a8-runtime-fixture"
            or effects.get("process") != "not-assessed-by-a8-runtime-fixture"
            or effects.get("browser") != "not-assessed-by-a8-runtime-fixture"
            or outcome.get("stdout_terminated") is not None
            or outcome.get("stderr_terminated") is not None
        ):
            raise RuntimeInventoryMismatch("CTR-201E A8 effect provenance drifted")
        if str(case["id"]).startswith("python.help."):
            if not isinstance(observations, Mapping) or set(observations) != set(EXPECTED_ENTRYPOINTS):
                raise RuntimeInventoryMismatch("CTR-201E help entrypoint observations are incomplete")
            if observations.get("qiongli") != _outcome_sha256(outcome):
                raise RuntimeInventoryMismatch(
                    "CTR-201E help observation digest binding drifted"
                )
        elif not isinstance(observations, Mapping) or (
            set(observations) != set(EXPECTED_ENTRYPOINTS)
            or any(value is not None for value in observations.values())
        ):
            raise RuntimeInventoryMismatch("CTR-201E non-help case has unexpected entrypoint observations")
    for path in public_paths:
        help_case = case_by_id[f"python.help.{_case_slug(path)}"]
        invalid_case = case_by_id[f"python.invalid-usage.{_case_slug(path)}"]
        if (
            help_case["outcome"]["exit_code"] != 0
            or help_case["outcome"]["stderr_lines"] != []
            or invalid_case["outcome"]["exit_code"] != 2
            or invalid_case["outcome"]["stdout_lines"] != []
            or not invalid_case["outcome"]["stderr_lines"]
        ):
            raise RuntimeInventoryMismatch("CTR-201E parser stream contract drifted")
    mcp_serve_help = case_by_id["python.help.qiongli.mcp.serve"]["outcome"]["stdout_lines"]
    if not mcp_serve_help or not str(mcp_serve_help[0]).startswith("usage: qiongli serve"):
        raise RuntimeInventoryMismatch("CTR-201E nested MCP prog compatibility drifted")
    for entrypoint in EXPECTED_ENTRYPOINTS:
        align_lines = case_by_id[f"python.entrypoint.{entrypoint}.align"]["outcome"]["stdout_lines"]
        if not align_lines or align_lines[0] != f"{entrypoint} — Quick Reference":
            raise RuntimeInventoryMismatch("CTR-201E console entrypoint argv[0] behavior drifted")
    if (
        case_by_id["python.handler.provider-list-json"]["outcome"]["json"]["disposition"]
        != "captured"
        or case_by_id["python.handler.mcp-config-example-json"]["outcome"]["json"]["disposition"]
        != "captured"
        or case_by_id["python.handler.provider-invalid-field"]["outcome"]["error_class"]
        != "input-error"
    ):
        raise RuntimeInventoryMismatch("CTR-201E representative handler boundaries drifted")
    if coverage != _expected_coverage():
        raise RuntimeInventoryMismatch("CTR-201E coverage counts differ from CTR-201B")
    if artifact.get("compatibility_boundary") != _expected_compatibility_boundary():
        raise RuntimeInventoryMismatch("CTR-201E compatibility boundary drifted")
    if (
        not isinstance(npm, Mapping)
        or npm.get("status") != "dispatch-frozen"
        or npm.get("capture_layer") != "accepted-node-parseArgv"
        or npm.get("source_tree_sha256")
        != "469ca19818fbc6cdefba364d4b9cbf4f954d56d6217de66f465d5305482367fc"
        or npm.get("filesystem_unchanged") is not True
        or npm.get("alias_count") != EXPECTED_COUNTS["npm_aliases"]
        or npm.get("handler_runtime_parity") != "pending-LEG-201"
        or npm.get("disposition_decision_id") != "CTR-201E-D003"
        or npm.get("python_npm_divergences")
        != [
            {
                "public_command": "update",
                "python_target": "self-update",
                "npm_target": "install",
                "npm_overwrite": True,
                "disposition": "frozen-divergence-pending-LEG-201",
            }
        ]
    ):
        raise RuntimeInventoryMismatch("CTR-201E npm compatibility boundary is invalid")
    dispatch = npm.get("dispatch")
    expected_dispatch = [
        {
            "raw_command": raw,
            "normalized_command": normalized,
            "overwrite": overwrite,
            "rest": [],
        }
        for raw, (normalized, overwrite) in EXPECTED_NPM_DISPATCH.items()
    ]
    if dispatch != expected_dispatch:
        raise RuntimeInventoryMismatch("CTR-201E npm dispatch mapping drifted")
    integrity = artifact.get("integrity")
    if (
        not isinstance(integrity, Mapping)
        or integrity.get("algorithm") != "sha256"
        or integrity.get("canonicalization") != CANONICALIZATION
        or integrity.get("case_manifest_sha256") != case_manifest_sha256(cases)
        or integrity.get("case_manifest_sha256")
        != EXPECTED_CASE_MANIFEST_SHA256
        or integrity.get("payload_sha256") != canonical_payload_sha256(artifact)
        or integrity.get("payload_sha256") != EXPECTED_PAYLOAD_SHA256
    ):
        raise RuntimeInventoryMismatch("CTR-201E artifact integrity is invalid")
    strings = _iter_strings(artifact)
    if any(MACHINE_PATH_PATTERN.search(value) for value in strings):
        raise RuntimeInventoryMismatch("CTR-201E artifact contains a machine-local path")
    if any(SECRET_PATTERN.search(value) for value in strings):
        raise RuntimeInventoryMismatch("CTR-201E artifact contains secret-shaped data")
    if any(CALLABLE_REPR_PATTERN.search(value) for value in strings):
        raise RuntimeInventoryMismatch("CTR-201E artifact contains an unstable representation")
    rendered = _canonical_json_bytes(artifact).decode("utf-8")
    for forbidden in (
        CANARY_SECRET,
        str(repo_root),
        "Traceback (most recent call last)",
    ):
        if forbidden in rendered:
            raise RuntimeInventoryMismatch(
                "CTR-201E artifact contains non-portable or secret data"
            )


def extract_cli_runtime_inventory(repo_root: Path = REPO_ROOT) -> Mapping[str, Any]:
    static_cli._require_python_312()
    root = repo_root.resolve()
    manifest, python_entries, npm_entries, static_artifact, oracle = _read_bound_sources(root)
    public_commands = _public_command_rows(static_artifact)
    python_captures = [
        _capture_python_once(root, python_entries, public_commands, variant)
        for variant in ("a", "b")
    ]
    if _canonical_json_bytes(python_captures[0]) != _canonical_json_bytes(python_captures[1]):
        raise RuntimeInventoryMismatch("isolated Python runtime captures are not deterministic")
    npm_captures = [
        _capture_npm_once(root, npm_entries, variant) for variant in ("a", "b")
    ]
    if _canonical_json_bytes(npm_captures[0]) != _canonical_json_bytes(npm_captures[1]):
        raise RuntimeInventoryMismatch("isolated npm dispatch captures are not deterministic")
    python_capture = python_captures[0]
    npm_capture = npm_captures[0]
    cases = [*python_capture["cases"], *_a8_cases(oracle)]
    json_canonical = sum(
        row["path_kind"] == "canonical" and row["json_capable"]
        for row in python_capture["public_commands"]
    )
    dry_run_canonical = sum(
        row["path_kind"] == "canonical" and row["dry_run_capable"]
        for row in python_capture["public_commands"]
    )
    dry_run_public = sum(row["dry_run_capable"] for row in python_capture["public_commands"])
    group_or_delegate = sum(not row["executable"] for row in python_capture["public_commands"] if row["path_kind"] == "canonical")
    executable_canonical = sum(row["executable"] for row in python_capture["public_commands"] if row["path_kind"] == "canonical")
    executable_public = sum(row["executable"] for row in python_capture["public_commands"])
    artifact: dict[str, Any] = {
        "$schema": ARTIFACT_SCHEMA,
        "schema_version": ARTIFACT_SCHEMA_VERSION,
        "record_type": ARTIFACT_RECORD_TYPE,
        "task_id": "CTR-201E",
        "status": "runtime-inventory-freeze-captured",
        "source": _expected_source(root, static_artifact),
        "capture_contract": dict(EXPECTED_CAPTURE_CONTRACT),
        "error_taxonomy": [dict(row) for row in OBSERVED_ERROR_TAXONOMY],
        "disposition_decisions": [dict(row) for row in DISPOSITION_DECISIONS],
        "console_entrypoints": python_capture["console_entrypoints"],
        "public_commands": python_capture["public_commands"],
        "cases": cases,
        "npm_compatibility": {
            "status": "dispatch-frozen",
            "capture_layer": "accepted-node-parseArgv",
            "source_tree_sha256": npm_capture["source_tree_sha256"],
            "filesystem_unchanged": npm_capture["filesystem_unchanged"],
            "dispatch": npm_capture["dispatch"],
            "alias_count": EXPECTED_COUNTS["npm_aliases"],
            "python_npm_divergences": [
                {
                    "public_command": "update",
                    "python_target": "self-update",
                    "npm_target": "install",
                    "npm_overwrite": True,
                    "disposition": "frozen-divergence-pending-LEG-201",
                }
            ],
            "handler_runtime_parity": "pending-LEG-201",
            "disposition_decision_id": "CTR-201E-D003",
        },
        "coverage": _expected_coverage(),
        "compatibility_boundary": _expected_compatibility_boundary(),
    }
    artifact["integrity"] = {
        "algorithm": "sha256",
        "canonicalization": CANONICALIZATION,
        "case_manifest_sha256": case_manifest_sha256(cases),
        "payload_sha256": canonical_payload_sha256(artifact),
    }
    _validate_expected_artifact(artifact, root)
    del manifest  # Keep the checked artifact bound to digests rather than duplicating A8.
    return artifact


def _python_worker_main(control_path: str) -> int:
    try:
        control = static_cli._load_json_bytes(Path(control_path).read_bytes())
        artifact = _python_worker_artifact(control)
        sys.stdout.buffer.write(_canonical_json_bytes({"status": "pass", "artifact": artifact}) + b"\n")
        return 0
    except (RuntimeInventoryMismatch, static_cli.InventoryMismatch):
        sys.stdout.buffer.write(
            _canonical_json_bytes({"status": "fail", "code": "worker-mismatch"})
            + b"\n"
        )
        return 1
    except Exception:
        if os.environ.get("CTR201E_WORKER_DEBUG") == "1":
            traceback.print_exc(file=sys.stderr)
        sys.stdout.buffer.write(_canonical_json_bytes({"status": "error", "code": "worker-failed"}) + b"\n")
        return 2


def _write_output(path: Path, artifact: Mapping[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(json.dumps(artifact, ensure_ascii=False, indent=2).encode("utf-8") + b"\n")


def _load_checked_output(path: Path) -> Mapping[str, Any]:
    try:
        return static_cli._load_json_bytes(path.read_bytes())
    except OSError as error:
        raise RuntimeExtractorError("checked CTR-201E artifact is unavailable") from error


def _emit_result(
    *, json_mode: bool, status: str, exit_code: int, code: str, artifact: Mapping[str, Any] | None = None
) -> None:
    payload: dict[str, Any] = {
        "status": status,
        "exit_code": exit_code,
        "code": code,
        "ctr_201": "in-progress",
        "ctr_202": "not-complete",
        "fnd_202": "not-implemented",
    }
    if artifact is not None:
        payload["payload_sha256"] = artifact["integrity"]["payload_sha256"]
        payload["public_command_count"] = artifact["coverage"]["public_commands"]
        payload["case_count"] = len(artifact["cases"])
    if json_mode:
        print(json.dumps(payload, ensure_ascii=False, sort_keys=True))
    elif status == "pass":
        print(f"[ctr-201e] {code}")
    else:
        print(f"[ctr-201e] {code}", file=sys.stderr)


def _build_parser() -> argparse.ArgumentParser:
    parser = _RedactedArgumentParser(
        description="Extract the accepted CTR-201E Full CLI runtime-freeze inventory."
    )
    parser.add_argument("--root", default=str(REPO_ROOT))
    parser.add_argument("--json", action="store_true")
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--output")
    parser.add_argument("--_python-worker", help=argparse.SUPPRESS)
    return parser


def main(argv: Sequence[str] | None = None) -> int:
    parser = _build_parser()
    try:
        args = parser.parse_args(argv)
        if args._python_worker:
            return _python_worker_main(args._python_worker)
        if args.check and args.output:
            raise CliUsageError("--check and --output are mutually exclusive")
        if not args.check and not args.output:
            raise CliUsageError("generation requires an explicit --output")
        root = Path(args.root).resolve()
        artifact = extract_cli_runtime_inventory(root)
        if args.check:
            checked = _load_checked_output(root / DEFAULT_OUTPUT_RELATIVE)
            if _canonical_json_bytes(artifact) != _canonical_json_bytes(checked):
                raise RuntimeInventoryMismatch("checked CTR-201E artifact does not match extraction")
            code = "accepted-cli-runtime-inventory-matches"
        else:
            output = Path(args.output)
            _write_output(output, artifact)
            code = "accepted-cli-runtime-inventory-written"
        _emit_result(json_mode=args.json, status="pass", exit_code=0, code=code, artifact=artifact)
        return 0
    except CliUsageError:
        _emit_result(
            json_mode="--json" in (argv or sys.argv[1:]),
            status="error",
            exit_code=2,
            code="accepted-cli-runtime-inventory-unavailable",
        )
        return 2
    except (RuntimeInventoryMismatch, static_cli.InventoryMismatch):
        _emit_result(
            json_mode="--json" in (argv or sys.argv[1:]),
            status="fail",
            exit_code=1,
            code="accepted-cli-runtime-inventory-mismatch",
        )
        return 1
    except (RuntimeExtractorError, static_cli.ExtractorError, OSError, ValueError):
        _emit_result(
            json_mode="--json" in (argv or sys.argv[1:]),
            status="error",
            exit_code=2,
            code="accepted-cli-runtime-inventory-unavailable",
        )
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
