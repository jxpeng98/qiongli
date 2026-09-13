#!/usr/bin/env python3
from __future__ import annotations

import argparse
from dataclasses import asdict, is_dataclass
import hashlib
import io
import json
import math
import os
from pathlib import Path, PurePosixPath
import re
import shutil
import subprocess
import sys
import tempfile
import threading
from typing import Any, Callable, Mapping, Sequence


REPO_ROOT = Path(__file__).resolve().parents[2]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

ACCEPTED_TAG = "v1.19.0-beta.1"
ACCEPTED_COMMIT = "8d2e99866ce4c4efb8b3b5e0265c0c1f89a36b0f"
MANIFEST_RELATIVE = "tooling/migration/baselines/v1.19.0-beta.1/manifest.json"
MANIFEST_SHA256 = "77bb7628d43a496c995e4b0a8daf6a624847b62e96948c0461affe89002da131"
PYTHON_ORACLE_RELATIVE = (
    "tooling/migration/baselines/v1.19.0-beta.1/oracles/python-full.json"
)
PYTHON_ORACLE_SHA256 = "26d247c9268c3166c98080aef420acfdb8248f62b11cc69420250f6e493a23e3"
STATIC_ARTIFACT_RELATIVE = "tooling/migration/ctr-201-orchestrator.json"
STATIC_SCHEMA_RELATIVE = "tooling/migration/ctr-201-orchestrator.schema.json"
STATIC_ARTIFACT_PAYLOAD_SHA256 = (
    "508ed0f92a511a0a9a6daa33598ce891222540b15e5aa207984db97319fe2c5e"
)
STATIC_SCHEMA_CANONICAL_SHA256 = (
    "0473158288cf35d4a10e39cfc741fd5b4cb38a49c68209aaea48337d52782510"
)
CONTENT_ARTIFACT_RELATIVE = "tooling/migration/ctr-201-content.json"
CONTENT_ARTIFACT_PAYLOAD_SHA256 = (
    "d17f37aa96d1896d047b27d197d63f773ae1d644a875722f5262be39593ff304"
)
ACCEPTED_MANIFEST_CORPUS_SHA256 = (
    "7fdd92894d88b221180e77ad73677cc158147cc861b17ba0245ea54f0127fbe2"
)

DEFAULT_OUTPUT_RELATIVE = "tooling/migration/ctr-201-orchestrator-runtime.json"
DEFAULT_SCHEMA_RELATIVE = "tooling/migration/ctr-201-orchestrator-runtime.schema.json"
ARTIFACT_SCHEMA = "./ctr-201-orchestrator-runtime.schema.json"
SCHEMA_ID = "https://qiongli.dev/schemas/ctr-201-orchestrator-runtime.schema.json"
SCHEMA_VERSION = "1.0"
RECORD_TYPE = "qiongli-ctr-201-orchestrator-runtime-inventory-freeze"
CANONICALIZATION = "utf-8-json-sorted-keys-compact-excluding-integrity"

PYTHON_TREE = {
    "root": "packages/python-qiongli/",
    "file_count": 76,
    "tree_sha256": "3a91a6dde9a78116fed73358275b2797c3ce7bf3d9a54894e7dbd11d2f0f9781",
}
CONTENT_TREE = {
    "root": "content/",
    "file_count": 377,
    "tree_sha256": "4659cbcd839c3f8eb3798a64981b7ec2180cf766566fcf439ac892eb32a8a5a8",
}
PYYAML_VERSION = "6.0.3"
PYYAML_PURE_FILE_COUNT = 17
PYYAML_PURE_TOTAL_BYTES = 217506
PYYAML_PURE_TREE_SHA256 = (
    "06f74e5c27433e236e83428ba11cc911d29217f68bc778b0396885699afb8992"
)

EXPECTED_PAYLOAD_SHA256 = (
    "29bbb1c0cd042d469f55e93078a4d3b4494148f47a2bd66e568d097f83e6b5da"
)
EXPECTED_CASE_MANIFEST_SHA256 = (
    "6a930dd355eb57b0b6b1759f73dba9c7af4b115e1b0bdc576112e49c14cc20ee"
)

HEX_40 = re.compile(r"^[0-9a-f]{40}$")
HEX_64 = re.compile(r"^[0-9a-f]{64}$")
UUID_RE = re.compile(
    r"\b[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}\b",
    re.IGNORECASE,
)
MACHINE_PATH_RE = re.compile(
    r"(?:file://|(?<![A-Za-z0-9/])/(?:Users|home|root|Volumes|tmp|var/tmp|"
    r"var/folders|private/tmp|private/var/folders)/|"
    r"(?<![A-Za-z0-9+.-])[A-Za-z]:[\\/]|\\\\[^\\/\s]+[\\/][^\\/\s]+)",
    re.IGNORECASE,
)
SECRET_RE = re.compile(
    r"(?:CTR201F_CANARY_SECRET|-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----|"
    r"\b(?:sk[-_]|ghp_|github_pat_)[A-Za-z0-9_-]{12,}\b)"
)
CALLABLE_REPR_RE = re.compile(
    r"(?:<(?:(?:bound )?method|function|class)\b|\bat 0x[0-9a-f]+>)",
    re.IGNORECASE,
)
CANARY_SECRET = "CTR201F_CANARY_SECRET_MUST_NOT_APPEAR"

DIMENSION_IDS = (
    "complete-runtime-behavior-matrix",
    "complete-state-and-resume",
    "complete-agent-launch-behavior",
    "complete-solo-duo-triad-runtime-parity",
    "complete-failure-and-cancellation",
    "complete-quality-gate-semantic-execution",
)

CASE_IDS = (
    "a8.orchestration-preview",
    "mcp.preview-controller-matrix",
    "mcp.run-agents-boolean-contract",
    "mcp.doctor-advisory-run-agents",
    "mcp.orchestrator-route-matrix",
    "task-plan.prerequisite-filesystem-matrix",
    "quality.artifact-existence-gate",
    "profile.builtin-and-custom-resolution",
    "profile.unknown-rejected",
    "execute.single-chain-role",
    "execute.parallel-triad",
    "execute.parallel-dual-degrade",
    "code-build.focus-routing",
    "code-build.legacy-standard",
    "code-build.legacy-advanced",
    "code-build.legacy-advanced-failure",
    "task-run.solo-observed-review",
    "task-run.duo-pass",
    "task-run.direct-triad-metadata-only",
    "task-run.triad-enabled",
    "task-run.primary-fallback",
    "task-run.block-revision-pass",
    "task-run.final-block",
    "task-run.draft-failure",
    "failure.policy-boundary-matrix",
    "worker.adapter-fallback",
    "worker.b1-success",
    "worker.b1-degrade",
    "worker.b1-merge-failure",
    "worker.b1-final-review-failure",
    "worker.b1-final-review-block",
    "worker.h3-block",
    "team-run.b1-planner-success",
    "team-run.b1-degrade",
    "team-run.b1-planner-fallback",
    "team-run.b1-all-workers-block",
    "team-run.b1-merge-failure",
    "team-run.b1-review-failure",
    "team-run.b1-review-block-observed",
    "team-run.h3-static-personas",
    "team-run.h3-block",
    "experience.replay-plan-advisory",
    "bridge.session-command-passthrough",
    "doctor.sanitized-environment",
)

EXPECTED_EXCEPTION_CASES = {
    "profile.unknown-rejected": (
        "ConfigError",
        "196f9ad28e6de781a4741ce4b63feb2780cb4d743895dd217beff00c8307f4b7",
    )
}

DISPOSITIONS: tuple[Mapping[str, Any], ...] = (
    {
        "id": "CTR-201F-D001",
        "scope": "real-agent-provider-and-network-execution",
        "accepted_fact": (
            "The accepted control plane can be exercised with deterministic bridge fakes, "
            "but real Codex, Claude, Antigravity, credentials, network, model output, and tool "
            "loops are not part of this offline inventory."
        ),
        "downstream_tasks": ["AGT-201", "AGT-202", "AGT-204", "AGT-205", "ORC-202"],
        "prohibited_claims": [
            "real-agent-runtime-parity",
            "provider-or-model-parity",
            "network-tool-loop-parity",
        ],
    },
    {
        "id": "CTR-201F-D002",
        "scope": "real-concurrency-timeout-signal-and-cancellation",
        "accepted_fact": (
            "Deterministic fakes capture fanout, fallback, and failure-policy decisions only; "
            "wall-clock ordering, operating-system signals, kill behavior, and user cancellation "
            "are not frozen."
        ),
        "downstream_tasks": ["AGT-204", "ORC-202"],
        "prohibited_claims": [
            "real-concurrency-timing-parity",
            "timeout-signal-parity",
            "public-cancellation-support",
        ],
    },
    {
        "id": "CTR-201F-D003",
        "scope": "durable-task-team-resume-and-checkpoint",
        "accepted_fact": (
            "The accepted source forwards a legacy single-mode session_id and can generate an "
            "experience replay recommendation, but task_run and team_run expose no durable "
            "checkpoint, resume, rollback, or cancel API."
        ),
        "downstream_tasks": ["CFG-202", "ORC-201"],
        "prohibited_claims": [
            "task-run-resume-parity",
            "team-run-resume-parity",
            "durable-checkpoint-support",
        ],
    },
    {
        "id": "CTR-201F-D004",
        "scope": "semantic-academic-quality-gate-execution",
        "accepted_fact": (
            "The accepted runtime propagates declared Q1-Q4 identifiers and checks artifact "
            "existence; it does not execute their academic semantic policies."
        ),
        "downstream_tasks": ["ORC-203", "GOV-202"],
        "prohibited_claims": [
            "semantic-quality-gate-parity",
            "academic-quality-verdict-parity",
        ],
    },
    {
        "id": "CTR-201F-D005",
        "scope": "native-worker-dispatch-and-mcp-worker-controls",
        "accepted_fact": (
            "codex-subagent and claude-cowork are recognized but route through generic_prompt; "
            "the accepted MCP task-run schema does not expose worker mode, adapter, or max-workers."
        ),
        "downstream_tasks": ["AGT-203", "ORC-202", "MCP-204", "LEG-201"],
        "prohibited_claims": [
            "native-worker-adapter-parity",
            "mcp-worker-control-parity",
        ],
    },
    {
        "id": "CTR-201F-D006",
        "scope": "strict-topic-code-build-state-and-stage-integration",
        "accepted_fact": (
            "Topic-less standard and advanced code-build routes plus focus and target mapping are "
            "captured offline. The strict topic route forces project guidance or trace state writes "
            "and Stage-I task-run integration, so it is excluded from the no-write fixture matrix."
        ),
        "downstream_tasks": ["CFG-202", "DOM-202", "ORC-203", "GOV-202", "LEG-201"],
        "prohibited_claims": [
            "strict-topic-code-build-runtime-parity",
            "stage-i-code-build-state-parity",
        ],
    },
)


class ExtractorError(RuntimeError):
    """The authenticated runtime inventory could not be evaluated safely."""


class InventoryMismatch(RuntimeError):
    """The accepted source or checked inventory differs from the fixed contract."""


class UsageError(RuntimeError):
    """The public extractor invocation is invalid."""


class _RedactedArgumentParser(argparse.ArgumentParser):
    def error(self, _message: str) -> None:  # pragma: no cover - exercised via main
        raise UsageError("invalid command usage")


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
    except (TypeError, ValueError, UnicodeEncodeError) as error:
        raise ExtractorError("value cannot be serialized canonically") from error
    return rendered.encode("utf-8")


def canonical_payload_sha256(record: Mapping[str, Any]) -> str:
    payload = {key: value for key, value in record.items() if key != "integrity"}
    return _sha256(_canonical_json_bytes(payload))


def canonical_schema_sha256(schema: Mapping[str, Any]) -> str:
    return _sha256(_canonical_json_bytes(schema))


def canonical_case_sha256(case: Mapping[str, Any]) -> str:
    payload = {key: value for key, value in case.items() if key != "case_sha256"}
    return _sha256(_canonical_json_bytes(payload))


def case_manifest_sha256(cases: Sequence[Mapping[str, Any]]) -> str:
    rows = [{"id": case["id"], "case_sha256": case["case_sha256"]} for case in cases]
    return _sha256(_canonical_json_bytes(rows))


def _reject_duplicate_object(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise ExtractorError("JSON document contains a duplicate key")
        result[key] = value
    return result


def _reject_nonfinite_constant(_value: str) -> None:
    raise ExtractorError("JSON document contains a non-finite number")


def _validate_loaded_json(value: Any, *, depth: int = 0) -> Any:
    if depth > 64:
        raise ExtractorError("JSON document exceeds the nesting limit")
    if value is None or isinstance(value, (bool, int)):
        return value
    if isinstance(value, float):
        if not math.isfinite(value):
            raise ExtractorError("JSON document contains a non-finite number")
        return value
    if isinstance(value, str):
        if any(0xD800 <= ord(char) <= 0xDFFF for char in value):
            raise ExtractorError("JSON document contains invalid Unicode")
        return value
    if isinstance(value, list):
        return [_validate_loaded_json(item, depth=depth + 1) for item in value]
    if isinstance(value, Mapping):
        normalized: dict[str, Any] = {}
        for key, item in value.items():
            if not isinstance(key, str):
                raise ExtractorError("JSON document contains a non-string key")
            normalized_key = _validate_loaded_json(key, depth=depth + 1)
            if not isinstance(normalized_key, str):
                raise ExtractorError("JSON document contains an invalid key")
            normalized[normalized_key] = _validate_loaded_json(item, depth=depth + 1)
        return normalized
    raise ExtractorError("JSON document contains an unsupported value")


def _load_json(path: Path, *, label: str) -> Mapping[str, Any]:
    try:
        value = json.loads(
            path.read_bytes().decode("utf-8"),
            object_pairs_hook=_reject_duplicate_object,
            parse_constant=_reject_nonfinite_constant,
        )
    except ExtractorError:
        raise
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        raise ExtractorError(f"{label} is unavailable or invalid") from error
    if not isinstance(value, Mapping):
        raise ExtractorError(f"{label} must contain an object")
    normalized = _validate_loaded_json(value)
    if not isinstance(normalized, Mapping):  # defensive type narrowing
        raise ExtractorError(f"{label} must contain an object")
    return normalized


def _fact(key: str, value: Any) -> Mapping[str, str]:
    if value is None:
        rendered = "null"
    elif isinstance(value, bool):
        rendered = "true" if value else "false"
    elif isinstance(value, (str, int, float)):
        rendered = str(value)
    else:
        rendered = _canonical_json_bytes(value).decode("utf-8")
    return {"key": key, "value": rendered}


def _iter_strings(value: Any) -> list[str]:
    values: list[str] = []
    if isinstance(value, str):
        values.append(value)
    elif isinstance(value, Mapping):
        for key, item in value.items():
            values.extend(_iter_strings(key))
            values.extend(_iter_strings(item))
    elif isinstance(value, (list, tuple, set)):
        for item in value:
            values.extend(_iter_strings(item))
    return values


def _runtime_replacements(control: Mapping[str, Any]) -> list[tuple[str, str]]:
    sources = [
        (str(control["accepted_root"]), "<ACCEPTED_ROOT>"),
        (str(control["source_root"]), "<RUNTIME_SOURCE>"),
        (str(control["content_root"]), "<CONTENT_ROOT>"),
        (str(control["capsule_root"]), "<PYYAML_CAPSULE>"),
    ]
    for key, value in sorted(control["project_roots"].items()):
        sources.append((str(value), f"<PROJECT_{str(key).upper()}>"))
    for env_name, token in (
        ("HOME", "<HOME>"),
        ("USERPROFILE", "<HOME>"),
        ("XDG_CONFIG_HOME", "<XDG_CONFIG_HOME>"),
        ("XDG_CACHE_HOME", "<XDG_CACHE_HOME>"),
        ("XDG_DATA_HOME", "<XDG_DATA_HOME>"),
        ("CODEX_HOME", "<CODEX_HOME>"),
        ("CLAUDE_CODE_HOME", "<CLAUDE_CODE_HOME>"),
        ("ANTIGRAVITY_HOME", "<ANTIGRAVITY_HOME>"),
        ("HERMES_HOME", "<HERMES_HOME>"),
        ("TMP", "<TMP>"),
        ("TEMP", "<TMP>"),
        ("TMPDIR", "<TMP>"),
    ):
        value = os.environ.get(env_name, "")
        if value:
            sources.append((value, token))

    aliases: dict[str, str] = {}
    for source, token in sources:
        for alias in (source, os.path.realpath(source)):
            if not alias:
                continue
            existing = aliases.get(alias)
            if existing is not None and existing != token:
                raise ExtractorError("runtime replacement alias maps to conflicting tokens")
            aliases[alias] = token
    return sorted(
        aliases.items(),
        key=lambda item: (-len(item[0]), item[0], item[1]),
    )


def _normalize_runtime_value(
    value: Any,
    replacements: Sequence[tuple[str, str]],
) -> Any:
    if is_dataclass(value) and not isinstance(value, type):
        value = asdict(value)
    if isinstance(value, Path):
        value = str(value)
    if value is None or isinstance(value, (bool, int)):
        return value
    if isinstance(value, float):
        if not math.isfinite(value):
            raise ExtractorError("runtime value contains a non-finite number")
        return value
    if isinstance(value, str):
        normalized = value.replace("\r\n", "\n").replace("\r", "\n")
        for source, replacement in replacements:
            normalized = normalized.replace(source, replacement)
            normalized = normalized.replace(source.replace("\\", "/"), replacement)
            normalized = normalized.replace(source.replace("/", "\\"), replacement)
        normalized = UUID_RE.sub("<UUID>", normalized)
        if CANARY_SECRET in normalized:
            raise ExtractorError("runtime value contains the canary secret")
        return normalized
    if isinstance(value, Mapping):
        return {
            str(key): _normalize_runtime_value(item, replacements)
            for key, item in sorted(value.items(), key=lambda pair: str(pair[0]))
        }
    if isinstance(value, (list, tuple)):
        return [_normalize_runtime_value(item, replacements) for item in value]
    if isinstance(value, set):
        normalized = [_normalize_runtime_value(item, replacements) for item in value]
        return sorted(normalized, key=_canonical_json_bytes)
    raise ExtractorError("runtime value contains a non-JSON type")


_FORBIDDEN_PROCESS_EVENTS = frozenset(
    {
        "os.exec",
        "os.fork",
        "os.forkpty",
        "os.posix_spawn",
        "os.spawn",
        "os.startfile",
        "os.startfile/2",
        "os.system",
        "pty.spawn",
        "subprocess.Popen",
    }
)
_MUTATION_EVENTS = frozenset(
    {
        "os.chflags",
        "os.chmod",
        "os.chown",
        "os.link",
        "os.mkdir",
        "os.mkfifo",
        "os.mknod",
        "os.remove",
        "os.removexattr",
        "os.rename",
        "os.replace",
        "os.rmdir",
        "os.setxattr",
        "os.symlink",
        "os.truncate",
        "os.unlink",
        "os.utime",
        "shutil.copyfile",
        "shutil.copytree",
        "shutil.move",
        "shutil.rmtree",
    }
)


def _install_worker_audit_hook() -> list[str]:
    violations: list[str] = []

    def audit(event: str, args: tuple[Any, ...]) -> None:
        if event in _FORBIDDEN_PROCESS_EVENTS or event.startswith(
            ("os.exec", "os.spawn", "pty.spawn", "socket.")
        ):
            violations.append(event)
            raise PermissionError("accepted orchestrator attempted a forbidden capability")
        if event == "open" and args:
            mode = args[1] if len(args) > 1 else "r"
            flags = args[2] if len(args) > 2 else 0
            write_mode = isinstance(mode, str) and any(token in mode for token in "wax+")
            write_flags = isinstance(flags, int) and bool(
                flags
                & (
                    getattr(os, "O_WRONLY", 0)
                    | getattr(os, "O_RDWR", 0)
                    | getattr(os, "O_CREAT", 0)
                    | getattr(os, "O_TRUNC", 0)
                    | getattr(os, "O_APPEND", 0)
                )
            )
            if write_mode or write_flags:
                violations.append(event)
                raise PermissionError("accepted orchestrator attempted a write")
        if event in _MUTATION_EVENTS:
            violations.append(event)
            raise PermissionError("accepted orchestrator attempted a filesystem mutation")

    sys.addaudithook(audit)
    return violations


def _tree_manifest(roots: Mapping[str, Path]) -> list[Mapping[str, Any]]:
    rows: list[Mapping[str, Any]] = []
    for root_id, root in sorted(roots.items()):
        for path in sorted(root.rglob("*"), key=lambda item: item.as_posix()):
            relative = path.relative_to(root).as_posix()
            if path.is_symlink():
                raise ExtractorError("fixture tree contains a symbolic link")
            if path.is_dir():
                rows.append({"root": root_id, "path": relative + "/", "kind": "directory", "sha256": ""})
            elif path.is_file():
                rows.append(
                    {
                        "root": root_id,
                        "path": relative,
                        "kind": "file",
                        "sha256": _sha256(path.read_bytes()),
                    }
                )
            else:
                raise ExtractorError("fixture tree contains a special file")
    return rows


def _stage_from_prompt(prompt: str) -> str:
    if prompt.startswith("Draft the task outputs"):
        return "draft"
    if prompt.startswith("Review the draft"):
        return "review"
    if prompt.startswith("Revise the task outputs") or prompt.startswith("You are revising"):
        return "revision"
    if prompt.startswith("Perform a third independent audit"):
        return "triad"
    if prompt.startswith("You are a research task planner"):
        return "team-planner"
    if prompt.startswith("Synthesize multi-agent parallel analysis"):
        return "parallel-synthesis"
    if prompt.startswith("Final-review"):
        return "worker-final-review"
    if prompt.startswith("Merge worker results for this Qiongli task"):
        return "worker-merge"
    if prompt.startswith("You are the merge agent for a team-run research task"):
        return "team-merge"
    if "Worker packet (JSON):" in prompt or "shard root" in prompt:
        return "worker"
    if "Tier 2 Advanced Mode" in prompt:
        return "code-build-advanced"
    if "Tier 1 Standard Mode" in prompt:
        return "code-build-standard"
    if "verify" in prompt[:160].lower():
        return "verify"
    return "analysis"


def _canonicalize_team_merge_prompt(prompt: str) -> str:
    shard_marker = "Worker shard outputs:\n"
    output_marker = "\n\nCanonical output files to produce:\n"
    if shard_marker not in prompt or output_marker not in prompt:
        raise ExtractorError("team merge prompt shape drifted")
    prefix, remainder = prompt.split(shard_marker, 1)
    shard_text, suffix = remainder.split(output_marker, 1)
    shard_blocks = shard_text.split("\n\n---\n\n") if shard_text else []
    return (
        prefix
        + shard_marker
        + "\n\n---\n\n".join(sorted(shard_blocks))
        + output_marker
        + suffix
    )


def _trace_rows(states: Sequence[Any], replacements: Sequence[tuple[str, str]]) -> list[Mapping[str, Any]]:
    normalized: list[Mapping[str, Any]] = []

    def normalize_call(call: Mapping[str, Any]) -> Mapping[str, Any]:
        prompt = str(_normalize_runtime_value(call["prompt"], replacements))
        if call["stage"] == "team-merge":
            prompt = _canonicalize_team_merge_prompt(prompt)
        options = _normalize_runtime_value(call["runtime_options"], replacements)
        return {
            "agent": call["agent"],
            "stage": call["stage"],
            "prompt_sha256": _sha256(prompt.encode("utf-8")),
            "runtime_options_sha256": _sha256(_canonical_json_bytes(options)),
            "success": bool(call["success"]),
        }

    def canonical_key(item: Mapping[str, Any]) -> tuple[Any, ...]:
        return (
            str(item["stage"]),
            str(item["agent"]),
            str(item["prompt_sha256"]),
            str(item["runtime_options_sha256"]),
            bool(item["success"]),
        )

    logical_cohort_ordinal = 0
    for state_ordinal, state in enumerate(states):
        calls = sorted(
            state.calls,
            key=lambda item: int(item["invocation_ordinal"]),
        )
        cursor = 0
        while cursor < len(calls):
            call = calls[cursor]
            if not bool(call["concurrent"]):
                normalized.append(
                    {
                        "state_ordinal": state_ordinal,
                        "logical_cohort_ordinal": logical_cohort_ordinal,
                        "cohort_member_ordinal": 0,
                        "ordering": "sequential",
                        **normalize_call(call),
                    }
                )
                logical_cohort_ordinal += 1
                cursor += 1
                continue

            end = cursor + 1
            while end < len(calls) and bool(calls[end]["concurrent"]):
                end += 1
            cohort = [normalize_call(item) for item in calls[cursor:end]]
            for member_ordinal, item in enumerate(sorted(cohort, key=canonical_key)):
                normalized.append(
                    {
                        "state_ordinal": state_ordinal,
                        "logical_cohort_ordinal": logical_cohort_ordinal,
                        "cohort_member_ordinal": member_ordinal,
                        "ordering": "concurrent",
                        **item,
                    }
                )
            logical_cohort_ordinal += 1
            cursor = end

    return [{"ordinal": index, **item} for index, item in enumerate(normalized)]


def _case(
    *,
    case_id: str,
    group: str,
    operation: str,
    provenance: str,
    dimension_ids: Sequence[str],
    input_facts: Sequence[Mapping[str, str]],
    result_facts: Sequence[Mapping[str, str]],
    result: Any = None,
    exception: BaseException | None = None,
    states: Sequence[Any],
    before: Sequence[Mapping[str, Any]],
    after: Sequence[Mapping[str, Any]],
    replacements: Sequence[tuple[str, str]],
) -> Mapping[str, Any]:
    normalized_result = _normalize_runtime_value(result, replacements) if exception is None else None
    normalized_message = (
        str(_normalize_runtime_value(str(exception), replacements)) if exception is not None else ""
    )
    normalized_strings = _iter_strings([normalized_result, normalized_message])
    if any(MACHINE_PATH_RE.search(value) for value in normalized_strings):
        raise ExtractorError("normalized runtime result contains a machine-local path")
    if any(SECRET_RE.search(value) for value in normalized_strings):
        raise ExtractorError("normalized runtime result contains secret-shaped data")
    if any(CALLABLE_REPR_RE.search(value) for value in normalized_strings):
        raise ExtractorError("normalized runtime result contains an unstable callable representation")
    before_map = {(row["root"], row["path"]): row for row in before}
    after_map = {(row["root"], row["path"]): row for row in after}
    changed = sorted(
        f"{root}:{path}"
        for root, path in set(before_map) | set(after_map)
        if before_map.get((root, path)) != after_map.get((root, path))
    )
    case: dict[str, Any] = {
        "id": case_id,
        "declaration_ordinal": -1,
        "group": group,
        "operation": operation,
        "provenance": provenance,
        "dimension_ids": list(dimension_ids),
        "input_facts": [dict(item) for item in input_facts],
        "result_facts": [dict(item) for item in result_facts],
        "outcome": {
            "kind": "exception" if exception is not None else "result",
            "result_sha256": _sha256(_canonical_json_bytes(normalized_result)),
            "exception_type": type(exception).__name__ if exception is not None else "",
            "exception_message_sha256": _sha256(normalized_message.encode("utf-8")),
        },
        "trace": _trace_rows(states, replacements),
        "effects": {
            "before_tree_sha256": _sha256(_canonical_json_bytes(before)),
            "after_tree_sha256": _sha256(_canonical_json_bytes(after)),
            "changed_path_count": len(changed),
            "changed_paths_sha256": _sha256(_canonical_json_bytes(changed)),
        },
        "case_sha256": "",
    }
    case["case_sha256"] = canonical_case_sha256(case)
    return case


def _capture_worker(control: Mapping[str, Any]) -> list[Mapping[str, Any]]:
    required = {
        "accepted_root",
        "source_root",
        "content_root",
        "capsule_root",
        "project_roots",
    }
    if set(control) != required or not isinstance(control["project_roots"], Mapping):
        raise ExtractorError("worker control shape is invalid")
    accepted_root = Path(str(control["accepted_root"])).resolve()
    source_root = Path(str(control["source_root"])).resolve()
    content_root = Path(str(control["content_root"])).resolve()
    capsule_root = Path(str(control["capsule_root"])).resolve()
    project_roots = {
        str(key): Path(str(value)).resolve() for key, value in control["project_roots"].items()
    }
    for path in (source_root, content_root, capsule_root, *project_roots.values()):
        try:
            path.relative_to(accepted_root.parent)
        except ValueError as error:
            raise ExtractorError("worker path escapes the temporary boundary") from error

    os.chdir(project_roots["empty"])
    sys.dont_write_bytecode = True
    base_sys_path = [
        entry
        for entry in sys.path
        if entry
        and Path(entry).resolve() != REPO_ROOT
        and "site-packages" not in entry.lower()
        and "dist-packages" not in entry.lower()
    ]
    sys.path[:] = [str(capsule_root), str(source_root), *base_sys_path]
    audit_violations = _install_worker_audit_hook()

    try:
        import yaml
        from bridges.base_bridge import BridgeResponse
        from bridges.experience_runtime import replay_experience_plan
        from bridges import mcp_tool_handlers as handlers
        from bridges import orchestrator as orchestrator_module
        from bridges.orchestrator import CollaborationMode, ModelOrchestrator
        from bridges.codex_bridge import CodexBridge
        from bridges.claude_bridge import ClaudeBridge
        from bridges.antigravity_bridge import AntigravityBridge
        from bridges.mcp_connectors import MCPEvidence, MCPConnector
    except Exception as error:
        raise ExtractorError("accepted orchestrator modules could not be imported") from error
    if getattr(yaml, "__version__", "") != PYYAML_VERSION or bool(
        getattr(yaml, "__with_libyaml__", False)
    ):
        raise ExtractorError("isolated PyYAML capsule identity is invalid")
    if not Path(str(yaml.__file__)).resolve().is_relative_to(capsule_root):
        raise ExtractorError("accepted runtime imported YAML outside the capsule")
    for module in (orchestrator_module, handlers):
        if not Path(str(module.__file__)).resolve().is_relative_to(source_root):
            raise ExtractorError("accepted runtime imported a module outside the source tree")

    replacements = _runtime_replacements(control)
    fixed_uuid = __import__("uuid").UUID("00000000-0000-4000-8000-000000000001")
    uuid_module = __import__("uuid")
    uuid_module.uuid4 = lambda: fixed_uuid

    class FakeState:
        def __init__(self, scenario: str = "pass", availability: set[str] | None = None) -> None:
            self.scenario = scenario
            self.availability = set(
                {"codex", "claude", "antigravity"}
                if availability is None
                else availability
            )
            self.calls: list[dict[str, Any]] = []
            self.lock = threading.Lock()
            self.controller_thread_id = threading.get_ident()
            self.call_count = 0
            self.review_count = 0

        def execute(
            self,
            agent: str,
            prompt: str,
            runtime_options: Mapping[str, Any],
        ) -> Any:
            stage = _stage_from_prompt(prompt)
            with self.lock:
                invocation_ordinal = self.call_count
                self.call_count += 1
                if stage == "review":
                    self.review_count += 1
                    review_ordinal = self.review_count
                else:
                    review_ordinal = 0

            success = True
            error: str | None = None
            if stage == "team-planner":
                if self.scenario == "b1-team-planner-failure":
                    success = False
                    error = "fixture planner failure"
                    content = ""
                else:
                    units = [
                        {"unit_id": "batch_1", "description": "fixture one", "scope": "scope one"},
                        {"unit_id": "batch_2", "description": "fixture two", "scope": "scope two"},
                    ]
                    if self.scenario == "b1-team-degrade":
                        units.append(
                            {
                                "unit_id": "batch_3",
                                "description": "fixture three",
                                "scope": "scope three",
                            }
                        )
                    content = json.dumps(units, sort_keys=True)
            elif stage in {"review", "triad", "worker-final-review"}:
                if (
                    self.scenario == "worker-final-review-failure"
                    and stage == "worker-final-review"
                ):
                    success = False
                    error = "fixture final review failure"
                    content = ""
                elif (
                    self.scenario == "worker-final-review-block"
                    and stage == "worker-final-review"
                ):
                    content = "Verdict: BLOCK\nCritical Issues: fixture\nConfidence: 0.2"
                elif self.scenario == "b1-team-review-failure" and stage == "review":
                    success = False
                    error = "fixture team review failure"
                    content = ""
                elif self.scenario == "b1-team-review-block" and stage == "review":
                    content = "Verdict: BLOCK\nCritical Issues: fixture\nConfidence: 0.2"
                elif self.scenario == "block-then-pass" and stage == "review" and review_ordinal == 1:
                    content = "Verdict: BLOCK\nCritical Issues: fixture\nConfidence: 0.4"
                elif self.scenario == "final-block" and stage == "review":
                    content = "Verdict: BLOCK\nCritical Issues: fixture\nConfidence: 0.4"
                elif self.scenario == "review-failure" and stage == "review":
                    success = False
                    error = "fixture review failure"
                    content = ""
                else:
                    content = "Verdict: PASS\nCritical Issues: none\nConfidence: 0.9"
            elif self.scenario == "draft-failure" and stage in {"draft", "revision"}:
                success = False
                error = "fixture draft failure"
                content = ""
            elif (
                self.scenario == "code-build-advanced-failure"
                and stage == "code-build-advanced"
            ):
                success = False
                error = "fixture code-build failure"
                content = ""
            elif self.scenario == "worker-merge-failure" and stage == "worker-merge":
                success = False
                error = "fixture worker merge failure"
                content = ""
            elif self.scenario == "b1-team-merge-failure" and stage == "team-merge":
                success = False
                error = "fixture team merge failure"
                content = ""
            elif (
                self.scenario in {"h3-worker-block", "h3-team-block"}
                and stage == "worker"
                and "methodologist" in prompt
            ):
                success = False
                error = "fixture worker failure"
                content = ""
            elif (
                self.scenario == "b1-worker-degrade"
                and stage == "worker"
                and "screening_worker" in prompt
            ):
                success = False
                error = "fixture worker failure"
                content = ""
            elif (
                self.scenario == "b1-team-degrade"
                and stage == "worker"
                and "Work Unit ID: batch_2" in prompt
            ):
                success = False
                error = "fixture worker failure"
                content = ""
            elif self.scenario == "b1-team-all-block" and stage == "worker":
                success = False
                error = "fixture worker failure"
                content = ""
            else:
                content = f"fixture {stage} output from {agent}"
            with self.lock:
                self.calls.append(
                    {
                        "invocation_ordinal": invocation_ordinal,
                        "concurrent": threading.get_ident() != self.controller_thread_id,
                        "agent": agent,
                        "stage": stage,
                        "prompt": prompt,
                        "runtime_options": dict(runtime_options),
                        "success": success,
                    }
                )
            return BridgeResponse(
                success=success,
                model=agent,
                content=content,
                error=error,
                session_id=str(runtime_options.get("session_id") or "") or None,
            )

    class FakeBridge:
        def __init__(self, agent: str, state: FakeState) -> None:
            self.agent = agent
            self.state = state

        def execute(self, prompt: str, _cwd: Path, **options: Any) -> Any:
            return self.state.execute(self.agent, prompt, options)

    class FakeMCP:
        def collect(self, provider: str, _packet: Mapping[str, Any], _cwd: Path) -> Any:
            return MCPEvidence(
                provider=provider,
                status="ok",
                summary=f"fixture evidence for {provider}",
                provenance=[f"fixture:{provider}"],
            )

    def new_orchestrator(
        scenario: str = "pass", availability: set[str] | None = None
    ) -> tuple[Any, FakeState]:
        state = FakeState(scenario=scenario, availability=availability)
        orchestrator_module.shutil.which = (
            lambda name: f"/fixture/bin/{name}" if name in state.availability else None
        )
        instance = ModelOrchestrator(standards_dir=content_root / "standards")
        instance.codex = FakeBridge("codex", state)
        instance.claude = FakeBridge("claude", state)
        instance.antigravity = FakeBridge("antigravity", state)
        instance.mcp_connector = FakeMCP()
        return instance, state

    def capture(
        case_id: str,
        group: str,
        operation: str,
        action: Callable[[], tuple[Any, Sequence[FakeState], Sequence[Mapping[str, str]]]],
        input_facts: Sequence[Mapping[str, str]],
    ) -> Mapping[str, Any]:
        before = _tree_manifest(project_roots)
        result: Any = None
        states: Sequence[FakeState] = []
        result_facts: Sequence[Mapping[str, str]] = []
        exception: BaseException | None = None
        violation_count = len(audit_violations)
        try:
            result, states, result_facts = action()
        except (PermissionError, ExtractorError, InventoryMismatch):
            raise
        except Exception as error:  # deterministic public error capture
            exception = error
        if len(audit_violations) != violation_count:
            raise PermissionError("accepted orchestrator attempted a forbidden capability")
        after = _tree_manifest(project_roots)
        return _case(
            case_id=case_id,
            group=group,
            operation=operation,
            provenance="accepted-source-bounded-runtime",
            dimension_ids=_case_dimension_ids(case_id),
            input_facts=input_facts,
            result_facts=result_facts,
            result=result,
            exception=exception,
            states=states,
            before=before,
            after=after,
            replacements=replacements,
        )

    cases: list[Mapping[str, Any]] = []

    def mcp_preview_matrix() -> tuple[Any, Sequence[FakeState], Sequence[Mapping[str, str]]]:
        outputs: dict[str, Any] = {}
        states: list[FakeState] = []
        original = handlers.ModelOrchestrator
        try:
            for mode in ("solo", "duo", "triad"):
                instance, state = new_orchestrator()
                states.append(state)
                handlers.ModelOrchestrator = lambda instance=instance: instance
                outputs[mode] = handlers._tool_task_run(
                    {
                        "task_id": "F3",
                        "paper_type": "empirical",
                        "topic": "runtime-fixture",
                        "cwd": str(project_roots["empty"]),
                        "guidance_mode": "off",
                        "execution_mode": mode,
                        "run_agents": False,
                    }
                )
        finally:
            handlers.ModelOrchestrator = original
        projection = {
            mode: {
                "mode": output["mode"],
                "run_agents": output["run_agents"],
                "execution_mode": output["data"]["task_run_preview"][
                    "controller_metadata"
                ]["execution_mode"],
                "will_launch_agents": output["data"]["task_run_preview"][
                    "will_launch_agents"
                ],
            }
            for mode, output in outputs.items()
        }
        return projection, states, [_fact("preview_matrix", projection)]

    cases.append(
        capture(
            "mcp.preview-controller-matrix",
            "mcp",
            "_tool_task_run preview matrix",
            mcp_preview_matrix,
            [_fact("run_agents", False)],
        )
    )

    def run_agents_contract() -> tuple[Any, Sequence[FakeState], Sequence[Mapping[str, str]]]:
        result: dict[str, Any] = {
            "missing": handlers._run_agents_enabled({}),
            "false": handlers._run_agents_enabled({"run_agents": False}),
            "true": handlers._run_agents_enabled({"run_agents": True}),
        }
        try:
            handlers._run_agents_enabled({"run_agents": "true"})
        except ValueError as error:
            result["string_true_error"] = str(error)
        return result, [], [
            _fact("missing", result["missing"]),
            _fact("false", result["false"]),
            _fact("true", result["true"]),
            _fact("string_true_rejected", "string_true_error" in result),
        ]

    cases.append(
        capture(
            "mcp.run-agents-boolean-contract",
            "mcp",
            "_run_agents_enabled",
            run_agents_contract,
            [_fact("accepted_type", "JSON boolean")],
        )
    )

    def doctor_advisory() -> tuple[Any, Sequence[FakeState], Sequence[Mapping[str, str]]]:
        instance, state = new_orchestrator()
        original = handlers.ModelOrchestrator
        doctor_calls = {"count": 0}
        real_doctor = instance.doctor

        def counted_doctor(cwd: Path) -> Any:
            doctor_calls["count"] += 1
            return real_doctor(cwd)

        instance.doctor = counted_doctor
        try:
            handlers.ModelOrchestrator = lambda: instance
            result = handlers._tool_task_run(
                {
                    "task_id": "F3",
                    "paper_type": "empirical",
                    "topic": "runtime-fixture",
                    "cwd": str(project_roots["empty"]),
                    "guidance_mode": "off",
                    "execution_mode": "duo",
                    "run_agents": True,
                    "skip_validation": True,
                    "max_revision_rounds": 0,
                }
            )
        finally:
            handlers.ModelOrchestrator = original
        return (
            {"task_run": result, "doctor_calls": doctor_calls["count"]},
            [state],
            [_fact("doctor_calls_during_task_run", doctor_calls["count"])],
        )

    cases.append(
        capture(
            "mcp.doctor-advisory-run-agents",
            "mcp",
            "_tool_task_run run_agents=true",
            doctor_advisory,
            [_fact("doctor_requirement", "advisory-not-enforced")],
        )
    )

    def orchestrator_route_matrix() -> tuple[Any, Sequence[FakeState], Sequence[Mapping[str, str]]]:
        skill_route = handlers._tool_orchestrator_route(
            {
                "request": "draft a short research note",
                "platform": "codex",
                "cwd": str(project_roots["empty"]),
            }
        )
        orchestrator_route = handlers._tool_orchestrator_route(
            {
                "request": "run multi-agent independent review",
                "platform": "claude-code",
                "task_id": "F3",
                "paper_type": "empirical",
                "topic": "runtime-fixture",
                "execution_mode": "duo",
                "cwd": str(project_roots["empty"]),
            }
        )
        projection = {
            "skill_route": skill_route["route"],
            "skill_tools": [item["tool"] for item in skill_route["sequence"]],
            "skill_missing": skill_route["missing"],
            "orchestrator_route": orchestrator_route["route"],
            "orchestrator_tools": [
                item["tool"] for item in orchestrator_route["sequence"]
            ],
            "orchestrator_run_agents": orchestrator_route["sequence"][-1]["args"][
                "run_agents"
            ],
            "platform": orchestrator_route["platform"],
            "requires_full_runtime": orchestrator_route["requires_full_runtime"],
            "route_safety_claims_doctor_gate": (
                "doctor passes" in orchestrator_route["safety"]
            ),
        }
        return projection, [], [
            _fact("skill_route", projection["skill_route"]),
            _fact("skill_tools", projection["skill_tools"]),
            _fact("skill_missing", projection["skill_missing"]),
            _fact("orchestrator_route", projection["orchestrator_route"]),
            _fact("orchestrator_tools", projection["orchestrator_tools"]),
            _fact("orchestrator_run_agents", projection["orchestrator_run_agents"]),
            _fact("platform", projection["platform"]),
            _fact("requires_full_runtime", projection["requires_full_runtime"]),
            _fact(
                "route_safety_claims_doctor_gate",
                projection["route_safety_claims_doctor_gate"],
            ),
        ]

    cases.append(
        capture(
            "mcp.orchestrator-route-matrix",
            "mcp",
            "_tool_orchestrator_route",
            orchestrator_route_matrix,
            [_fact("routes", ["skill_workflow", "orchestrator_mcp"])],
        )
    )

    def task_plan_matrix() -> tuple[Any, Sequence[FakeState], Sequence[Mapping[str, str]]]:
        instance, state = new_orchestrator()
        outputs = {
            "f3_empty": instance.task_plan("F3", "empirical", "fixture", project_roots["empty"]),
            "f3_satisfied": instance.task_plan(
                "F3", "empirical", "fixture", project_roots["f3_satisfied"]
            ),
            "b3_any_satisfied": instance.task_plan(
                "B3", "systematic-review", "fixture", project_roots["b3_any"]
            ),
        }
        projection = {
            name: {
                "confidence": result.confidence,
                "missing_prerequisites_all": result.data.get(
                    "missing_prerequisites_all", []
                ),
                "any_of": [
                    {
                        "task": item.get("task"),
                        "satisfied": item.get("satisfied"),
                        "satisfied_by": item.get("satisfied_by"),
                    }
                    for item in result.data.get("any_of_requirements", [])
                ],
            }
            for name, result in outputs.items()
        }
        return projection, [state], [_fact("plan_matrix", projection)]

    cases.append(
        capture(
            "task-plan.prerequisite-filesystem-matrix",
            "task-plan",
            "ModelOrchestrator.task_plan",
            task_plan_matrix,
            [_fact("tasks", ["F3", "B3"])],
        )
    )

    def validator_matrix() -> tuple[Any, Sequence[FakeState], Sequence[Mapping[str, str]]]:
        instance, state = new_orchestrator()
        root = project_roots["validator"]
        outputs = {
            "empty_file": instance._validator_gate(
                "fixture", root, "RESEARCH/[topic]/", ["empty.txt"]
            ),
            "empty_directory": instance._validator_gate(
                "fixture", root, "RESEARCH/[topic]/", ["empty-dir/"]
            ),
            "nonempty_directory": instance._validator_gate(
                "fixture", root, "RESEARCH/[topic]/", ["nonempty-dir/"]
            ),
        }
        projection = {
            name: {
                "passed": output["passed"],
                "found": output["found"],
                "missing": output["missing"],
                "checked": output["checked"],
            }
            for name, output in outputs.items()
        }
        return projection, [state], [
            _fact("gate_results", projection),
            _fact("semantic_execution", False),
        ]

    cases.append(
        capture(
            "quality.artifact-existence-gate",
            "quality",
            "ModelOrchestrator._validator_gate",
            validator_matrix,
            [_fact("gate_kind", "artifact-existence-only")],
        )
    )

    def profile_matrix() -> tuple[Any, Sequence[FakeState], Sequence[Mapping[str, str]]]:
        instance, state = new_orchestrator()
        profile_file = project_roots["profile"] / "profile.json"
        registry, overrides = instance._load_profile_bundle(profile_file)
        parallel_result = instance.execute(
            CollaborationMode.PARALLEL,
            project_roots["empty"],
            prompt="fixture profiled analysis",
            profile_file=profile_file,
            profile="fixture-custom",
            summarizer_profile="fixture-custom",
        )
        task_result = instance.task_run(
            task_id="F3",
            paper_type="empirical",
            topic="runtime-fixture",
            cwd=project_roots["empty"],
            profile_file=profile_file,
            profile="default",
            primary_agent="codex",
            review_agent="claude",
            guidance_mode="off",
            skip_validation=True,
            max_revision_rounds=0,
        )
        analysis_call = next(
            call
            for call in state.calls
            if call["stage"] == "analysis" and call["agent"] == "codex"
        )
        draft_call = next(call for call in state.calls if call["stage"] == "draft")
        review_call = next(call for call in state.calls if call["stage"] == "review")
        parallel_profile_applied = (
            "Agent Profile: fixture-custom (stage: analysis)" in analysis_call["prompt"]
        )
        draft_profile_applied = (
            "Agent Profile: fixture-custom (stage: draft)" in draft_call["prompt"]
        )
        review_profile_applied = (
            "Agent Profile: strict-review (stage: review)" in review_call["prompt"]
        )
        outputs = {
            "builtin_directives": {
                name: instance._build_profile_directive(
                    name, instance._resolve_profile_config(name, registry), "review"
                )
                for name in sorted(instance.DEFAULT_AGENT_PROFILES)
            },
            "custom": instance._resolve_profile_config("fixture-custom", registry),
            "override": instance._resolve_task_profile_names(
                "F3", overrides, "default", None, None, None
            ),
            "runtime": {
                "parallel_mode": parallel_result.mode,
                "task_mode": task_result.mode,
                "parallel_codex_timeout_seconds": analysis_call["runtime_options"].get(
                    "timeout_seconds"
                ),
                "draft_timeout_seconds": draft_call["runtime_options"].get(
                    "timeout_seconds"
                ),
                "parallel_profile_applied": parallel_profile_applied,
                "draft_profile_applied": draft_profile_applied,
                "review_profile_applied": review_profile_applied,
            },
        }
        return outputs, [state], [
            _fact("builtin_profile_count", len(instance.DEFAULT_AGENT_PROFILES)),
            _fact(
                "parallel_codex_timeout_seconds",
                analysis_call["runtime_options"].get("timeout_seconds"),
            ),
            _fact("draft_timeout_seconds", draft_call["runtime_options"].get("timeout_seconds")),
            _fact("parallel_profile_applied", parallel_profile_applied),
            _fact("draft_profile_applied", draft_profile_applied),
            _fact("review_profile_applied", review_profile_applied),
        ]

    cases.append(
        capture(
            "profile.builtin-and-custom-resolution",
            "profile",
            "profile bundle resolution",
            profile_matrix,
            [_fact("profile_file", "fixture profile bundle")],
        )
    )

    def unknown_profile() -> tuple[Any, Sequence[FakeState], Sequence[Mapping[str, str]]]:
        instance, state = new_orchestrator()
        registry, _ = instance._load_profile_bundle(None)
        return instance._resolve_profile_config("missing-profile", registry), [state], []

    cases.append(
        capture(
            "profile.unknown-rejected",
            "profile",
            "_resolve_profile_config unknown",
            unknown_profile,
            [_fact("profile", "missing-profile")],
        )
    )

    def execute_modes() -> tuple[Any, Sequence[FakeState], Sequence[Mapping[str, str]]]:
        outputs: dict[str, Any] = {}
        states: list[FakeState] = []
        for mode in ("single", "chain", "role"):
            instance, state = new_orchestrator()
            states.append(state)
            if mode == "single":
                outputs[mode] = instance.execute(
                    CollaborationMode.SINGLE,
                    project_roots["empty"],
                    prompt="fixture prompt",
                    single_model="codex",
                    session_id="fixture-session",
                )
            elif mode == "chain":
                outputs[mode] = instance.execute(
                    CollaborationMode.CHAIN,
                    project_roots["empty"],
                    prompt="fixture prompt",
                    generator="codex",
                )
            else:
                outputs[mode] = instance.execute(
                    CollaborationMode.ROLE_BASED,
                    project_roots["empty"],
                    codex_task="fixture code",
                    claude_task="fixture prose",
                    antigravity_task="fixture audit",
                )
        projection = {
            mode: {
                "result_mode": result.mode,
                "confidence": result.confidence,
                "codex_success": bool(
                    result.codex_response and result.codex_response.success
                ),
                "claude_success": bool(
                    result.claude_response and result.claude_response.success
                ),
                "antigravity_success": bool(
                    result.antigravity_response and result.antigravity_response.success
                ),
            }
            for mode, result in outputs.items()
        }
        return projection, states, [_fact("mode_results", projection)]

    cases.append(
        capture(
            "execute.single-chain-role",
            "execute",
            "ModelOrchestrator.execute",
            execute_modes,
            [_fact("session_id", "fixture-session")],
        )
    )

    def parallel_case(availability: set[str]) -> tuple[Any, Sequence[FakeState], Sequence[Mapping[str, str]]]:
        instance, state = new_orchestrator(availability=availability)
        result = instance.execute(
            CollaborationMode.PARALLEL,
            project_roots["empty"],
            prompt="fixture parallel prompt",
        )
        response_success = {
            "codex": bool(result.codex_response and result.codex_response.success),
            "claude": bool(result.claude_response and result.claude_response.success),
            "antigravity": bool(
                result.antigravity_response and result.antigravity_response.success
            ),
        }
        projection = {
            "mode": result.mode,
            "confidence": result.confidence,
            "response_success": response_success,
        }
        return projection, [state], [
            _fact("available_agents", sorted(availability)),
            _fact("mode", result.mode),
            _fact("confidence", result.confidence),
            _fact("response_success", response_success),
        ]

    cases.append(
        capture(
            "execute.parallel-triad",
            "execute",
            "ModelOrchestrator.execute parallel",
            lambda: parallel_case({"codex", "claude", "antigravity"}),
            [_fact("expected_level", "triad")],
        )
    )
    cases.append(
        capture(
            "execute.parallel-dual-degrade",
            "execute",
            "ModelOrchestrator.execute parallel degrade",
            lambda: parallel_case({"claude", "antigravity"}),
            [_fact("unavailable_agent", "codex")],
        )
    )

    def code_build_focus_routing() -> tuple[Any, Sequence[FakeState], Sequence[Mapping[str, str]]]:
        instance, state = new_orchestrator()
        focuses = (None, "implementation", "planning", "execution", "review", "full")
        mappings = {
            "default" if focus is None else focus: {
                "normalized": instance._normalize_code_build_focus(focus),
                "task_id": instance.CODE_BUILD_FOCUS_TO_TASK.get(
                    instance._normalize_code_build_focus(focus),
                    "I1",
                ),
            }
            for focus in focuses
        }
        target_map = instance._resolve_code_build_target_map(
            "FULL",
            ["I6:S1", "I8:P1-01", "I6:S2", "I6:S1"],
        )
        invalid_error = ""
        try:
            instance._resolve_code_build_target_map("FULL", ["S1"])
        except ValueError as error:
            invalid_error = str(error)
        projection = {
            "mappings": mappings,
            "target_map": target_map,
            "invalid_selector_rejected": bool(invalid_error),
            "invalid_selector_error_sha256": _sha256(invalid_error.encode("utf-8")),
        }
        return projection, [state], [
            _fact("mappings", mappings),
            _fact("target_map", target_map),
            _fact("invalid_selector_rejected", bool(invalid_error)),
        ]

    cases.append(
        capture(
            "code-build.focus-routing",
            "code-build",
            "code_build focus and target routing",
            code_build_focus_routing,
            [_fact("topic_route", "not-executed-no-write-boundary")],
        )
    )

    def legacy_code_build(
        tier: str,
        scenario: str = "pass",
    ) -> tuple[Any, Sequence[FakeState], Sequence[Mapping[str, str]]]:
        instance, state = new_orchestrator(scenario=scenario)
        result = instance.code_build(
            method="difference-in-differences",
            cwd=project_roots["empty"],
            domain="economics",
            tier=tier,
        )
        response_success = {
            "codex": bool(result.codex_response and result.codex_response.success),
            "claude": bool(result.claude_response and result.claude_response.success),
            "antigravity": bool(
                result.antigravity_response and result.antigravity_response.success
            ),
        }
        projection = {
            "mode": result.mode,
            "confidence": result.confidence,
            "response_success": response_success,
        }
        return projection, [state], [
            _fact("mode", result.mode),
            _fact("confidence", result.confidence),
            _fact("response_success", response_success),
        ]

    for case_id, tier, scenario in (
        ("code-build.legacy-standard", "standard", "pass"),
        ("code-build.legacy-advanced", "advanced", "pass"),
        (
            "code-build.legacy-advanced-failure",
            "advanced",
            "code-build-advanced-failure",
        ),
    ):
        cases.append(
            capture(
                case_id,
                "code-build",
                "ModelOrchestrator.code_build topic-less route",
                lambda tier=tier, scenario=scenario: legacy_code_build(tier, scenario),
                [_fact("tier", tier), _fact("scenario", scenario)],
            )
        )

    def task_run_case(
        *,
        mode: str,
        triad: bool,
        scenario: str = "pass",
        availability: set[str] | None = None,
        worker_mode: str = "none",
        worker_adapter: str = "auto",
        max_workers: int | None = None,
        task_id: str = "F3",
    ) -> tuple[Any, Sequence[FakeState], Sequence[Mapping[str, str]]]:
        instance, state = new_orchestrator(scenario=scenario, availability=availability)
        result = instance.task_run(
            task_id=task_id,
            paper_type="systematic-review" if task_id == "B1" else "empirical",
            topic="runtime-fixture",
            cwd=project_roots["empty"],
            execution_mode=mode,
            controller="codex",
            primary_agent="codex",
            review_agent="claude",
            verifier_agent="antigravity",
            triad=triad,
            guidance_mode="off",
            skip_validation=True,
            max_revision_rounds=1 if scenario == "block-then-pass" else 0,
            worker_mode=worker_mode,
            worker_adapter=worker_adapter,
            max_workers=max_workers,
        )
        review_loop = dict(result.data.get("review_loop_state", {}))
        validator_gate = dict(result.data.get("validator_gate", {}))
        facts = [
            _fact("trace_count", len(state.calls)),
            _fact("mode", result.mode),
            _fact("confidence", result.confidence),
            _fact("review_loop_status", review_loop.get("status", "missing")),
            _fact("final_verdict", review_loop.get("final_verdict", "missing")),
            _fact("reviews_completed", review_loop.get("reviews_completed", -1)),
            _fact("revisions_attempted", review_loop.get("revisions_attempted", -1)),
            _fact("validator_gate_passed", validator_gate.get("passed")),
        ]
        if worker_mode != "none":
            worker_calls = [call for call in state.calls if call["stage"] == "worker"]
            worker_state = dict(
                result.data.get("task_packet", {}).get("worker_orchestration", {})
            )
            facts.extend(
                [
                    _fact("worker_count", len(worker_calls)),
                    _fact(
                        "worker_failure_count",
                        sum(not bool(call["success"]) for call in worker_calls),
                    ),
                    _fact("worker_status", worker_state.get("status", "missing")),
                    _fact(
                        "worker_barrier_status",
                        worker_state.get("barrier_status", "missing"),
                    ),
                    _fact(
                        "worker_merge_status",
                        worker_state.get("merge_status", "missing"),
                    ),
                    _fact(
                        "worker_final_review_status",
                        worker_state.get("merge_review_status", "missing"),
                    ),
                    _fact(
                        "worker_final_review_verdict",
                        worker_state.get("merge_review_verdict", ""),
                    ),
                    _fact(
                        "worker_final_review_confidence",
                        worker_state.get("merge_review_confidence"),
                    ),
                    _fact(
                        "review_loop_stopped_reason",
                        review_loop.get("stopped_reason", "missing"),
                    ),
                    _fact("validator_gate_skipped", validator_gate.get("skipped")),
                    _fact("validator_gate_reason", validator_gate.get("reason", "")),
                ]
            )
        return result, [state], facts

    task_run_specs = (
        ("task-run.solo-observed-review", "solo", False, "pass", None),
        ("task-run.duo-pass", "duo", False, "pass", None),
        ("task-run.direct-triad-metadata-only", "triad", False, "pass", None),
        ("task-run.triad-enabled", "triad", True, "pass", None),
        ("task-run.primary-fallback", "duo", False, "pass", {"claude", "antigravity"}),
        ("task-run.block-revision-pass", "duo", False, "block-then-pass", None),
        ("task-run.final-block", "duo", False, "final-block", None),
        ("task-run.draft-failure", "duo", False, "draft-failure", None),
    )
    for case_id, mode, triad, scenario, availability in task_run_specs:
        cases.append(
            capture(
                case_id,
                "task-run",
                "ModelOrchestrator.task_run",
                lambda mode=mode, triad=triad, scenario=scenario, availability=availability: task_run_case(
                    mode=mode,
                    triad=triad,
                    scenario=scenario,
                    availability=availability,
                ),
                [_fact("execution_mode", mode), _fact("triad_flag", triad), _fact("scenario", scenario)],
            )
        )

    def failure_policy_matrix() -> tuple[Any, Sequence[FakeState], Sequence[Mapping[str, str]]]:
        instance, state = new_orchestrator()

        def worker_rows(successes: Sequence[bool]) -> list[dict[str, Any]]:
            return [
                {
                    "worker_id": f"worker_{index + 1}",
                    "agent": "codex",
                    "success": success,
                    "error": None if success else "fixture failure",
                }
                for index, success in enumerate(successes)
            ]

        def shard_rows(successes: Sequence[bool]) -> list[dict[str, Any]]:
            return [
                {
                    "unit_id": f"unit_{index + 1}",
                    "agent": "codex",
                    "success": success,
                    "error": None if success else "fixture failure",
                }
                for index, success in enumerate(successes)
            ]

        worker_degraded, _ = instance._apply_worker_barrier(
            worker_rows([True, True, False]),
            {"min_success_ratio": 0.6, "on_failure": "degrade"},
        )
        worker_below_threshold, _ = instance._apply_worker_barrier(
            worker_rows([True, False, False]),
            {"min_success_ratio": 0.6, "on_failure": "degrade"},
        )
        worker_block, _ = instance._apply_worker_barrier(
            worker_rows([True, True, False]),
            {"min_success_ratio": 1.0, "on_failure": "block"},
        )
        team_degraded_rows, team_degraded, _ = instance._apply_failure_policy(
            shard_rows([True, True, False]),
            {"min_success_ratio": 0.6, "on_failure": "degrade"},
        )
        team_below_rows, team_below_threshold, _ = instance._apply_failure_policy(
            shard_rows([True, False, False]),
            {"min_success_ratio": 0.6, "on_failure": "degrade"},
        )
        team_block_rows, team_block, _ = instance._apply_failure_policy(
            shard_rows([True, True, False]),
            {"min_success_ratio": 1.0, "on_failure": "block"},
        )
        projection = {
            "worker_degraded": worker_degraded,
            "worker_below_threshold": worker_below_threshold,
            "worker_block": worker_block,
            "team_degraded": team_degraded,
            "team_degraded_success_count": len(team_degraded_rows),
            "team_below_threshold": team_below_threshold,
            "team_below_threshold_success_count": len(team_below_rows),
            "team_block": team_block,
            "team_block_success_count": len(team_block_rows),
        }
        return projection, [state], [_fact("policy_matrix", projection)]

    cases.append(
        capture(
            "failure.policy-boundary-matrix",
            "failure",
            "worker and team failure policy thresholds",
            failure_policy_matrix,
            [_fact("policies", ["degrade", "block"])],
        )
    )

    def worker_adapter_matrix() -> tuple[Any, Sequence[FakeState], Sequence[Mapping[str, str]]]:
        instance, state = new_orchestrator()
        config = {"adapter_preference": {"codex": "codex_subagent"}}
        task_run_tool = next(
            item for item in handlers.MCP_TOOL_DEFINITIONS if item.get("name") == "qiongli_task_run"
        )
        task_run_properties = sorted(task_run_tool["inputSchema"]["properties"])
        outputs = {
            "adapters": {
                requested: instance._resolve_worker_orchestration_adapter(
                    requested_adapter=requested,
                    controller_runtime="codex",
                    worker_config=config,
                )
                for requested in ("auto", "codex_subagent", "claude_cowork")
            },
            "mcp_task_run_properties": task_run_properties,
            "mcp_worker_controls_present": any(
                key in task_run_properties for key in ("worker_mode", "worker_adapter", "max_workers")
            ),
        }
        effective_adapters = {
            requested: value[0] for requested, value in outputs["adapters"].items()
        }
        return outputs, [state], [
            _fact("effective_adapters", effective_adapters),
            _fact(
                "mcp_worker_controls_present",
                outputs["mcp_worker_controls_present"],
            ),
        ]

    cases.append(
        capture(
            "worker.adapter-fallback",
            "worker",
            "_resolve_worker_orchestration_adapter",
            worker_adapter_matrix,
            [_fact("native_dispatch", False)],
        )
    )
    cases.append(
        capture(
            "worker.b1-success",
            "worker",
            "task_run delegated workers B1",
            lambda: task_run_case(
                mode="solo",
                triad=False,
                worker_mode="delegated-workers",
                worker_adapter="generic-prompt",
                max_workers=2,
                task_id="B1",
            ),
            [_fact("task_id", "B1"), _fact("barrier_policy", "degrade")],
        )
    )
    cases.append(
        capture(
            "worker.b1-degrade",
            "worker",
            "task_run delegated workers B1 degrade",
            lambda: task_run_case(
                mode="solo",
                triad=False,
                scenario="b1-worker-degrade",
                worker_mode="delegated-workers",
                worker_adapter="generic-prompt",
                max_workers=3,
                task_id="B1",
            ),
            [_fact("task_id", "B1"), _fact("barrier_policy", "degrade")],
        )
    )
    for case_id, scenario in (
        ("worker.b1-merge-failure", "worker-merge-failure"),
        ("worker.b1-final-review-failure", "worker-final-review-failure"),
        ("worker.b1-final-review-block", "worker-final-review-block"),
    ):
        cases.append(
            capture(
                case_id,
                "worker",
                "task_run delegated workers B1 post-barrier failure",
                lambda scenario=scenario: task_run_case(
                    mode="solo",
                    triad=False,
                    scenario=scenario,
                    worker_mode="delegated-workers",
                    worker_adapter="generic-prompt",
                    max_workers=2,
                    task_id="B1",
                ),
                [_fact("task_id", "B1"), _fact("scenario", scenario)],
            )
        )
    cases.append(
        capture(
            "worker.h3-block",
            "worker",
            "task_run delegated workers H3",
            lambda: task_run_case(
                mode="solo",
                triad=False,
                scenario="h3-worker-block",
                worker_mode="delegated-workers",
                worker_adapter="generic-prompt",
                max_workers=3,
                task_id="H3",
            ),
            [_fact("task_id", "H3"), _fact("barrier_policy", "block")],
        )
    )

    def team_run_case(
        task_id: str,
        scenario: str = "pass",
    ) -> tuple[Any, Sequence[FakeState], Sequence[Mapping[str, str]]]:
        instance, state = new_orchestrator(scenario=scenario)
        result = instance.team_run(
            task_id=task_id,
            paper_type="systematic-review" if task_id == "B1" else "empirical",
            topic="runtime-fixture",
            cwd=project_roots["empty"],
            max_parallel_units=(
                3 if scenario == "b1-team-degrade" else (2 if task_id == "B1" else 3)
            ),
            skip_validation=True,
        )
        barrier_match = re.search(
            r"^- Barrier status: ([^\n]+)$",
            result.merged_analysis,
            re.MULTILINE,
        )
        barrier_status = barrier_match.group(1) if barrier_match else "missing"
        worker_calls = [call for call in state.calls if call["stage"] == "worker"]
        merge_executed = any(call["stage"] == "team-merge" for call in state.calls)
        review_executed = any(call["stage"] == "review" for call in state.calls)
        review_block_observed = "Verdict: BLOCK" in result.merged_analysis
        projection = {
            "mode": result.mode,
            "confidence": result.confidence,
            "barrier_status": barrier_status,
            "data_keys": sorted(result.data),
            "contains_team_header": f"Team-Run: {task_id}" in result.merged_analysis,
            "contains_batch_1": "batch_1" in result.merged_analysis,
            "contains_batch_2": "batch_2" in result.merged_analysis,
            "contains_methodologist": "methodologist" in result.merged_analysis,
            "contains_domain_expert": "domain_expert" in result.merged_analysis,
            "contains_reviewer_2": "reviewer_2" in result.merged_analysis,
            "worker_count": len(worker_calls),
            "worker_failure_count": sum(
                not bool(call["success"]) for call in worker_calls
            ),
            "merge_executed": merge_executed,
            "review_executed": review_executed,
            "review_block_observed": review_block_observed,
        }
        return projection, [state], [
            _fact("task_id", task_id),
            _fact("trace_count", len(state.calls)),
            _fact("barrier_status", barrier_status),
            _fact("confidence", result.confidence),
            _fact("worker_count", len(worker_calls)),
            _fact(
                "worker_failure_count",
                sum(not bool(call["success"]) for call in worker_calls),
            ),
            _fact("merge_executed", merge_executed),
            _fact("review_executed", review_executed),
            _fact("review_block_observed", review_block_observed),
        ]

    cases.append(
        capture(
            "team-run.b1-planner-success",
            "team-run",
            "ModelOrchestrator.team_run B1",
            lambda: team_run_case("B1"),
            [_fact("partition", "dynamic-planner")],
        )
    )
    cases.append(
        capture(
            "team-run.b1-degrade",
            "team-run",
            "ModelOrchestrator.team_run B1 degrade",
            lambda: team_run_case("B1", scenario="b1-team-degrade"),
            [_fact("partition", "dynamic-planner"), _fact("barrier_policy", "degrade")],
        )
    )
    for case_id, scenario in (
        ("team-run.b1-planner-fallback", "b1-team-planner-failure"),
        ("team-run.b1-all-workers-block", "b1-team-all-block"),
        ("team-run.b1-merge-failure", "b1-team-merge-failure"),
        ("team-run.b1-review-failure", "b1-team-review-failure"),
        ("team-run.b1-review-block-observed", "b1-team-review-block"),
    ):
        cases.append(
            capture(
                case_id,
                "team-run",
                "ModelOrchestrator.team_run B1 failure branch",
                lambda scenario=scenario: team_run_case("B1", scenario=scenario),
                [_fact("partition", "dynamic-planner"), _fact("scenario", scenario)],
            )
        )
    cases.append(
        capture(
            "team-run.h3-static-personas",
            "team-run",
            "ModelOrchestrator.team_run H3",
            lambda: team_run_case("H3"),
            [_fact("partition", "static-personas")],
        )
    )
    cases.append(
        capture(
            "team-run.h3-block",
            "team-run",
            "ModelOrchestrator.team_run H3 block",
            lambda: team_run_case("H3", scenario="h3-team-block"),
            [_fact("partition", "static-personas"), _fact("barrier_policy", "block")],
        )
    )

    def experience_replay() -> tuple[Any, Sequence[FakeState], Sequence[Mapping[str, str]]]:
        outputs = {
            "failed": replay_experience_plan(project_roots["experience"], "failed-run"),
            "passed": replay_experience_plan(project_roots["experience"], "passed-run"),
        }
        return outputs, [], [
            _fact("execution_performed", False),
            _fact("failed_validator_status", outputs["failed"]["validator_status"]),
            _fact("failed_failure_modes", outputs["failed"]["failure_modes"]),
            _fact("failed_next_action", outputs["failed"]["next_action"]),
            _fact("passed_validator_status", outputs["passed"]["validator_status"]),
            _fact("passed_failure_modes", outputs["passed"]["failure_modes"]),
            _fact("passed_next_action", outputs["passed"]["next_action"]),
        ]

    cases.append(
        capture(
            "experience.replay-plan-advisory",
            "state",
            "replay_experience_plan",
            experience_replay,
            [_fact("run_ids", ["failed-run", "passed-run"])],
        )
    )

    def bridge_commands() -> tuple[Any, Sequence[FakeState], Sequence[Mapping[str, str]]]:
        cwd = project_roots["empty"]
        inspect = __import__("inspect")
        task_run_parameters = set(inspect.signature(ModelOrchestrator.task_run).parameters)
        team_run_parameters = set(inspect.signature(ModelOrchestrator.team_run).parameters)
        outputs = {
            "codex": CodexBridge().build_command("fixture", cwd, session_id="fixture-session"),
            "claude": ClaudeBridge().build_command("fixture", cwd, session_id="fixture-session"),
            "antigravity": AntigravityBridge().build_command(
                "fixture", cwd, session_id="fixture-session"
            ),
            "task_run_has_resume": "resume" in task_run_parameters,
            "team_run_has_resume": "resume" in team_run_parameters,
            "task_run_has_cancel": "cancel" in task_run_parameters,
            "team_run_has_cancel": "cancel" in team_run_parameters,
        }
        return outputs, [], [
            _fact("session_id", "fixture-session"),
            _fact("task_run_has_resume", outputs["task_run_has_resume"]),
            _fact("team_run_has_resume", outputs["team_run_has_resume"]),
            _fact("task_run_has_cancel", outputs["task_run_has_cancel"]),
            _fact("team_run_has_cancel", outputs["team_run_has_cancel"]),
            _fact(
                "codex_session_forwarded",
                "fixture-session" in outputs["codex"],
            ),
            _fact(
                "claude_session_forwarded",
                "fixture-session" in outputs["claude"],
            ),
            _fact(
                "antigravity_session_forwarded",
                "fixture-session" in outputs["antigravity"],
            ),
        ]

    cases.append(
        capture(
            "bridge.session-command-passthrough",
            "state",
            "bridge build_command session_id",
            bridge_commands,
            [_fact("bridges", ["codex", "claude", "antigravity"])],
        )
    )

    def doctor_case() -> tuple[Any, Sequence[FakeState], Sequence[Mapping[str, str]]]:
        instance, state = new_orchestrator(availability=set())
        instance.mcp_connector = MCPConnector()
        result = instance.doctor(project_roots["empty"])
        cli_statuses: dict[str, str] = {}
        for cli_name in ("codex", "claude", "antigravity"):
            match = re.search(
                rf"^- \[([A-Z]+)\] CLI {re.escape(cli_name)}:",
                result.merged_analysis,
                re.MULTILINE,
            )
            cli_statuses[cli_name] = match.group(1).lower() if match else "missing"
        return result, [state], [
            _fact("executes_commands", False),
            _fact("cli_statuses", cli_statuses),
        ]

    cases.append(
        capture(
            "doctor.sanitized-environment",
            "doctor",
            "ModelOrchestrator.doctor",
            doctor_case,
            [_fact("availability", "none")],
        )
    )

    return cases


def _git_environment() -> dict[str, str]:
    environment = {key: value for key, value in os.environ.items() if not key.upper().startswith("GIT_")}
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


def _verify_tag(repo_root: Path) -> None:
    try:
        completed = subprocess.run(
            ["git", "rev-parse", f"{ACCEPTED_TAG}^{{}}"],
            cwd=repo_root,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            env=_git_environment(),
            check=False,
        )
    except OSError as error:
        raise ExtractorError("accepted Git tag reader is unavailable") from error
    if completed.returncode != 0:
        raise ExtractorError("accepted tag is unavailable locally")
    if completed.stdout.strip() != ACCEPTED_COMMIT.encode("ascii"):
        raise InventoryMismatch("accepted tag does not resolve to the fixed commit")


def _materialize_tree(root: Path, blobs: Mapping[str, bytes]) -> None:
    for relative, payload in sorted(blobs.items()):
        path = root.joinpath(*PurePosixPath(relative).parts)
        try:
            path.relative_to(root)
        except ValueError as error:
            raise ExtractorError("accepted path escapes the materialized tree") from error
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(payload)
        path.chmod(0o444)
    for directory in sorted(
        (item for item in root.rglob("*") if item.is_dir()),
        key=lambda item: len(item.parts),
        reverse=True,
    ):
        directory.chmod(0o555)
    root.chmod(0o555)


def _pyyaml_capsule(root: Path) -> Mapping[str, Any]:
    try:
        import yaml
    except ImportError as error:
        raise ExtractorError("PyYAML is unavailable") from error
    if getattr(yaml, "__version__", "") != PYYAML_VERSION:
        raise ExtractorError("CTR-201F extraction requires PyYAML 6.0.3")
    package = Path(str(yaml.__file__)).resolve().parent
    rows: list[Mapping[str, Any]] = []
    target = root / "yaml"
    target.mkdir(parents=True)
    for path in sorted(package.glob("*.py"), key=lambda item: item.name):
        payload = path.read_bytes()
        rows.append({"path": path.name, "size": len(payload), "sha256": _sha256(payload)})
        (target / path.name).write_bytes(payload)
    identity = {
        "version": PYYAML_VERSION,
        "pure_python_file_count": len(rows),
        "pure_python_total_bytes": sum(int(row["size"]) for row in rows),
        "pure_python_tree_sha256": _sha256(_canonical_json_bytes(rows)),
    }
    expected = {
        "version": PYYAML_VERSION,
        "pure_python_file_count": PYYAML_PURE_FILE_COUNT,
        "pure_python_total_bytes": PYYAML_PURE_TOTAL_BYTES,
        "pure_python_tree_sha256": PYYAML_PURE_TREE_SHA256,
    }
    if identity != expected:
        raise InventoryMismatch("PyYAML pure-Python capsule identity drifted")
    return identity


def _write_fixture_output(root: Path, relative: str) -> None:
    path = root.joinpath(*PurePosixPath(relative.rstrip("/")).parts)
    if relative.endswith("/"):
        path.mkdir(parents=True, exist_ok=True)
    else:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text("fixture\n", encoding="utf-8")


def _prepare_project_fixtures(temp_root: Path, static_artifact: Mapping[str, Any]) -> Mapping[str, str]:
    roots = {
        name: temp_root / f"project-{name}"
        for name in ("empty", "f3_satisfied", "b3_any", "validator", "profile", "experience")
    }
    for root in roots.values():
        root.mkdir(parents=True)
    tasks = {
        str(task["task_id"]): task
        for task in static_artifact["workflow"]["tasks"]
        if isinstance(task, Mapping)
    }
    for task_id in ("A1", "A2", "F1"):
        for output in tasks[task_id]["outputs"]:
            _write_fixture_output(roots["f3_satisfied"] / "RESEARCH" / "fixture", str(output))
    for output in tasks["B1"]["outputs"]:
        _write_fixture_output(roots["b3_any"] / "RESEARCH" / "fixture", str(output))
    validator_root = roots["validator"] / "RESEARCH" / "fixture"
    validator_root.mkdir(parents=True)
    (validator_root / "empty.txt").write_bytes(b"")
    (validator_root / "empty-dir").mkdir()
    (validator_root / "nonempty-dir").mkdir()
    (validator_root / "nonempty-dir" / "item.txt").write_text("fixture\n", encoding="utf-8")
    profile = {
        "profiles": {
            "fixture-custom": {
                "persona": "Fixture profile",
                "runtime_options": {"codex": {"timeout_seconds": 123}},
            }
        },
        "task_overrides": {"F3": {"profile": "fixture-custom", "review_profile": "strict-review"}},
    }
    (roots["profile"] / "profile.json").write_text(
        json.dumps(profile, sort_keys=True), encoding="utf-8"
    )
    records = {
        "failed-run": {
            "schema_version": "1.0",
            "run_id": "failed-run",
            "task": {"task_id": "F3", "paper_type": "empirical", "topic": "fixture"},
            "inputs": {"guidance_sources": []},
            "outputs": {"missing_outputs": ["manuscript/manuscript.md"]},
            "quality": {"validator_status": "failed"},
            "experience": {"failure_modes": ["missing-output"]},
        },
        "passed-run": {
            "schema_version": "1.0",
            "run_id": "passed-run",
            "task": {"task_id": "F3", "paper_type": "empirical", "topic": "fixture"},
            "inputs": {"guidance_sources": []},
            "outputs": {"missing_outputs": []},
            "quality": {"validator_status": "passed"},
            "experience": {"failure_modes": []},
        },
    }
    for run_id, record in records.items():
        path = roots["experience"] / ".qiongli" / "trace" / "runs" / run_id / "experience_record.json"
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(json.dumps(record, sort_keys=True), encoding="utf-8")
    return {key: str(path) for key, path in roots.items()}


def _worker_environment(temp_root: Path, variant: str) -> dict[str, str]:
    state = temp_root / f"state-{variant}"
    environment = {
        "HOME": str(state / "home"),
        "USERPROFILE": str(state / "home"),
        "XDG_CONFIG_HOME": str(state / "xdg-config"),
        "XDG_CACHE_HOME": str(state / "xdg-cache"),
        "XDG_DATA_HOME": str(state / "xdg-data"),
        "CODEX_HOME": str(state / "codex"),
        "CLAUDE_CODE_HOME": str(state / "claude"),
        "ANTIGRAVITY_HOME": str(state / "antigravity"),
        "HERMES_HOME": str(state / "hermes"),
        "TMP": str(state / "tmp"),
        "TEMP": str(state / "tmp"),
        "TMPDIR": str(state / "tmp"),
        "PATH": "" if variant == "a" else str(state / "unused-bin"),
        "LANG": "C.UTF-8",
        "LC_ALL": "C.UTF-8",
        "TZ": "UTC",
        "NO_COLOR": "1",
        "CTR201F_CANARY_SECRET": CANARY_SECRET,
    }
    if os.name == "nt" and os.environ.get("SystemRoot"):
        environment["SystemRoot"] = os.environ["SystemRoot"]
    if os.environ.get("CTR201F_WORKER_DEBUG") == "1":
        environment["CTR201F_WORKER_DEBUG"] = "1"
    for key in (
        "HOME",
        "USERPROFILE",
        "XDG_CONFIG_HOME",
        "XDG_CACHE_HOME",
        "XDG_DATA_HOME",
        "CODEX_HOME",
        "CLAUDE_CODE_HOME",
        "ANTIGRAVITY_HOME",
        "HERMES_HOME",
        "TMP",
        "TEMP",
        "TMPDIR",
    ):
        Path(environment[key]).mkdir(parents=True, exist_ok=True)
    return environment


def _run_capture_once(
    repo_root: Path,
    all_files: Mapping[str, Mapping[str, Any]],
    static_artifact: Mapping[str, Any],
    variant: str,
) -> list[Mapping[str, Any]]:
    from tooling.scripts import extract_ctr_201_content_inventory as content_extractor

    records = list(all_files.values())
    blobs = content_extractor._cat_file_blobs(repo_root, records)
    with tempfile.TemporaryDirectory(prefix=f"qiongli-ctr201f-{variant}-") as raw:
        temp_root = Path(raw)
        accepted_root = temp_root / "accepted"
        accepted_root.mkdir()
        _materialize_tree(accepted_root, blobs)
        capsule_root = temp_root / "capsule"
        capsule_root.mkdir()
        _pyyaml_capsule(capsule_root)
        project_roots = _prepare_project_fixtures(temp_root, static_artifact)
        control = {
            "accepted_root": str(accepted_root),
            "source_root": str(accepted_root / "packages" / "python-qiongli" / "src"),
            "content_root": str(accepted_root / "content"),
            "capsule_root": str(capsule_root),
            "project_roots": project_roots,
        }
        control_path = temp_root / "control.json"
        control_path.write_bytes(_canonical_json_bytes(control))
        environment = _worker_environment(temp_root, variant)
        command = [
            sys.executable,
            "-I",
            "-S",
            "-B",
            str(Path(__file__).resolve()),
            "--_worker",
            str(control_path),
        ]
        try:
            completed = subprocess.run(
                command,
                cwd=Path(project_roots["empty"]),
                env=environment,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
                encoding="utf-8",
                errors="strict",
                timeout=180,
                check=False,
            )
        except (OSError, subprocess.TimeoutExpired) as error:
            raise ExtractorError("isolated orchestrator runtime worker failed to run") from error
        if os.environ.get("CTR201F_WORKER_DEBUG") == "1":
            if completed.stdout:
                sys.stderr.write("[ctr-201f-worker-stdout] " + completed.stdout + "\n")
            if completed.stderr:
                sys.stderr.write("[ctr-201f-worker-stderr] " + completed.stderr + "\n")
        if completed.returncode != 0 or completed.stderr:
            raise ExtractorError("isolated orchestrator runtime worker failed")
        try:
            payload = json.loads(completed.stdout)
        except json.JSONDecodeError as error:
            raise ExtractorError("isolated orchestrator runtime worker returned invalid JSON") from error
        if not isinstance(payload, Mapping) or payload.get("status") != "pass":
            raise ExtractorError("isolated orchestrator runtime worker returned no artifact")
        cases = payload.get("cases")
        if not isinstance(cases, list):
            raise ExtractorError("isolated orchestrator runtime worker returned invalid cases")
        return cases


def _a8_case(static_artifact: Mapping[str, Any]) -> Mapping[str, Any]:
    oracle = static_artifact["oracle"]
    source = {"task": oracle["task"], "outcome": oracle["outcome"]}
    case: dict[str, Any] = {
        "id": "a8.orchestration-preview",
        "declaration_ordinal": -1,
        "group": "mcp",
        "operation": "tools/call qiongli_task_run",
        "provenance": "accepted-a8-oracle",
        "dimension_ids": _case_dimension_ids("a8.orchestration-preview"),
        "input_facts": [
            _fact("task_id", oracle["task"]["task_id"]),
            _fact("run_agents", oracle["task"]["run_agents"]),
        ],
        "result_facts": [
            _fact("mode", oracle["outcome"]["mode"]),
            _fact("will_launch_agents", oracle["outcome"]["will_launch_agents"]),
        ],
        "outcome": {
            "kind": "result",
            "result_sha256": _sha256(_canonical_json_bytes(source)),
            "exception_type": "",
            "exception_message_sha256": _sha256(b""),
        },
        "trace": [],
        "effects": {
            "before_tree_sha256": oracle["filesystem_delta"]["before_tree_sha256"],
            "after_tree_sha256": oracle["filesystem_delta"]["after_tree_sha256"],
            "changed_path_count": 0,
            "changed_paths_sha256": _sha256(_canonical_json_bytes([])),
        },
        "case_sha256": "",
    }
    case["case_sha256"] = canonical_case_sha256(case)
    return case


def _read_bound_inputs(repo_root: Path) -> tuple[Mapping[str, Any], Mapping[str, Mapping[str, Any]], Mapping[str, Any]]:
    from tooling.scripts import extract_ctr_201_orchestrator_inventory as static_extractor
    from tooling.scripts import extract_ctr_201_content_inventory as content_extractor

    _verify_tag(repo_root)
    manifest_path = repo_root / MANIFEST_RELATIVE
    if _sha256(manifest_path.read_bytes()) != MANIFEST_SHA256:
        raise InventoryMismatch("accepted A8 manifest digest drifted")
    if _sha256((repo_root / PYTHON_ORACLE_RELATIVE).read_bytes()) != PYTHON_ORACLE_SHA256:
        raise InventoryMismatch("accepted Python Full oracle digest drifted")
    manifest, all_files = static_extractor._read_manifest(repo_root)
    static_artifact = _load_json(repo_root / STATIC_ARTIFACT_RELATIVE, label="CTR-201C artifact")
    static_schema = _load_json(repo_root / STATIC_SCHEMA_RELATIVE, label="CTR-201C schema")
    if static_extractor.canonical_payload_sha256(static_artifact) != STATIC_ARTIFACT_PAYLOAD_SHA256:
        raise InventoryMismatch("CTR-201C artifact payload drifted")
    if static_extractor.canonical_schema_sha256(static_schema) != STATIC_SCHEMA_CANONICAL_SHA256:
        raise InventoryMismatch("CTR-201C schema digest drifted")
    content_artifact = _load_json(repo_root / CONTENT_ARTIFACT_RELATIVE, label="CTR-201D artifact")
    if (
        content_artifact.get("integrity", {}).get("payload_sha256")
        != CONTENT_ARTIFACT_PAYLOAD_SHA256
        or content_extractor.canonical_payload_sha256(content_artifact)
        != CONTENT_ARTIFACT_PAYLOAD_SHA256
    ):
        raise InventoryMismatch("CTR-201D artifact payload drifted")
    return manifest, all_files, static_artifact


def _behavior_dimensions() -> list[Mapping[str, Any]]:
    return [
        {
            "id": "complete-runtime-behavior-matrix",
            "resolution": "captured-with-explicit-disposition",
            "case_ids": [case_id for case_id in CASE_IDS],
            "decision_ids": [
                "CTR-201F-D001",
                "CTR-201F-D002",
                "CTR-201F-D005",
                "CTR-201F-D006",
            ],
            "accepted_behavior": "Bounded offline control-flow matrix captured; live provider behavior remains downstream.",
        },
        {
            "id": "complete-state-and-resume",
            "resolution": "captured-accepted-absence",
            "case_ids": [
                "execute.single-chain-role",
                "experience.replay-plan-advisory",
                "bridge.session-command-passthrough",
            ],
            "decision_ids": ["CTR-201F-D003"],
            "accepted_behavior": "Single session passthrough and replay advice captured; durable task/team resume is absent.",
        },
        {
            "id": "complete-agent-launch-behavior",
            "resolution": "captured-with-explicit-disposition",
            "case_ids": [
                "mcp.run-agents-boolean-contract",
                "mcp.doctor-advisory-run-agents",
                "execute.single-chain-role",
                "execute.parallel-triad",
                "code-build.legacy-standard",
                "code-build.legacy-advanced",
                "task-run.duo-pass",
                "task-run.triad-enabled",
                "team-run.b1-planner-success",
            ],
            "decision_ids": ["CTR-201F-D001", "CTR-201F-D005"],
            "accepted_behavior": "Dispatch and fallback decisions captured through deterministic bridge fakes; live agents are excluded.",
        },
        {
            "id": "complete-solo-duo-triad-runtime-parity",
            "resolution": "captured-with-explicit-disposition",
            "case_ids": [
                "mcp.preview-controller-matrix",
                "task-run.solo-observed-review",
                "task-run.duo-pass",
                "task-run.direct-triad-metadata-only",
                "task-run.triad-enabled",
            ],
            "decision_ids": ["CTR-201F-D001"],
            "accepted_behavior": "Accepted controller semantics captured, including solo review and direct triad flag behavior.",
        },
        {
            "id": "complete-failure-and-cancellation",
            "resolution": "captured-with-explicit-disposition",
            "case_ids": [
                "execute.parallel-dual-degrade",
                "code-build.legacy-advanced-failure",
                "task-run.primary-fallback",
                "task-run.block-revision-pass",
                "task-run.final-block",
                "task-run.draft-failure",
                "failure.policy-boundary-matrix",
                "worker.b1-degrade",
                "worker.b1-merge-failure",
                "worker.b1-final-review-failure",
                "worker.b1-final-review-block",
                "worker.h3-block",
                "team-run.b1-degrade",
                "team-run.b1-planner-fallback",
                "team-run.b1-all-workers-block",
                "team-run.b1-merge-failure",
                "team-run.b1-review-failure",
                "team-run.b1-review-block-observed",
                "team-run.h3-block",
            ],
            "decision_ids": ["CTR-201F-D002", "CTR-201F-D003"],
            "accepted_behavior": "Deterministic failure policies captured; public cancellation and real process interruption are absent or downstream.",
        },
        {
            "id": "complete-quality-gate-semantic-execution",
            "resolution": "captured-accepted-absence",
            "case_ids": ["quality.artifact-existence-gate"],
            "decision_ids": ["CTR-201F-D004"],
            "accepted_behavior": "Artifact existence behavior captured; semantic Q1-Q4 execution is absent from accepted runtime.",
        },
    ]


def _case_dimension_ids(case_id: str) -> list[str]:
    if case_id not in CASE_IDS:
        raise ExtractorError("CTR-201F case is not part of the frozen dimension matrix")
    dimension_case_ids = {
        str(dimension["id"]): set(dimension["case_ids"])
        for dimension in _behavior_dimensions()
    }
    dimensions = [
        dimension_id
        for dimension_id in DIMENSION_IDS
        if case_id in dimension_case_ids[dimension_id]
    ]
    if not dimensions:
        raise ExtractorError("CTR-201F case has no behavior dimension")
    return dimensions


def extract_orchestrator_runtime_inventory(repo_root: Path = REPO_ROOT) -> Mapping[str, Any]:
    if sys.version_info[:2] != (3, 12):
        raise ExtractorError("CTR-201F extraction requires Python 3.12")
    root = repo_root.resolve()
    manifest, all_files, static_artifact = _read_bound_inputs(root)
    captures = [
        _run_capture_once(root, all_files, static_artifact, variant) for variant in ("a", "b")
    ]
    if _canonical_json_bytes(captures[0]) != _canonical_json_bytes(captures[1]):
        raise InventoryMismatch("isolated orchestrator runtime captures are not deterministic")
    cases = [_a8_case(static_artifact), *captures[0]]
    if [case.get("id") for case in cases] != list(CASE_IDS):
        raise InventoryMismatch("CTR-201F case identity or order drifted")
    for ordinal, case in enumerate(cases):
        case["declaration_ordinal"] = ordinal
        case["case_sha256"] = canonical_case_sha256(case)

    source_blob_anchors = [dict(item) for item in static_artifact["source"]["blob_anchors"]]
    artifact: dict[str, Any] = {
        "$schema": ARTIFACT_SCHEMA,
        "schema_version": SCHEMA_VERSION,
        "record_type": RECORD_TYPE,
        "task_id": "CTR-201F",
        "status": "runtime-inventory-freeze-captured",
        "source": {
            "accepted_tag": ACCEPTED_TAG,
            "accepted_commit": ACCEPTED_COMMIT,
            "a8_manifest": {"path": MANIFEST_RELATIVE, "sha256": MANIFEST_SHA256},
            "python_full_oracle": {
                "path": PYTHON_ORACLE_RELATIVE,
                "sha256": PYTHON_ORACLE_SHA256,
                "case_id": "python.orchestration-preview",
            },
            "package_trees": [dict(PYTHON_TREE), dict(CONTENT_TREE)],
            "blob_anchors": source_blob_anchors,
            "ctr_201c": {
                "artifact_path": STATIC_ARTIFACT_RELATIVE,
                "schema_path": STATIC_SCHEMA_RELATIVE,
                "schema_canonical_sha256": STATIC_SCHEMA_CANONICAL_SHA256,
                "payload_sha256": STATIC_ARTIFACT_PAYLOAD_SHA256,
            },
            "ctr_201d": {
                "artifact_path": CONTENT_ARTIFACT_RELATIVE,
                "payload_sha256": CONTENT_ARTIFACT_PAYLOAD_SHA256,
            },
            "accepted_manifest_corpus_sha256": manifest["integrity"]["corpus_sha256"],
        },
        "capture_contract": {
            "python_version": "python3.12",
            "python_isolation": "-I-S-B-dual-environment-audit-hook",
            "dependency": "pure-python-pyyaml-6.0.3-capsule",
            "dependency_tree_sha256": PYYAML_PURE_TREE_SHA256,
            "source_mode": "accepted-tag-git-blobs",
            "fake_boundary": "bridge-mcp-availability-and-deterministic-identity-injection",
            "network_policy": "python-audit-denied",
            "process_policy": "worker-denies-child-processes;parent-launches-git-and-python-capture-processes",
            "write_policy": "worker-denies-sut-writes;fixture-roots-before-after-measured",
            "secret_policy": "sanitized-environment-and-canary-rejection",
            "trace_order": "logical-order-preserved-only-concurrent-cohorts-canonicalized",
            "determinism": "two-distinct-temp-roots-and-environments-byte-equivalent",
            "os_sandbox_claim": "authenticated-source-capability-guard-not-hostile-code-os-sandbox",
        },
        "behavior_dimensions": _behavior_dimensions(),
        "disposition_decisions": [dict(item) for item in DISPOSITIONS],
        "cases": cases,
        "coverage": {
            "case_count": len(cases),
            "bounded_runtime_case_count": len(cases) - 1,
            "accepted_a8_case_count": 1,
            "resolved_dimension_count": len(DIMENSION_IDS),
            "disposition_decision_count": len(DISPOSITIONS),
            "required_not_fully_captured_count": 0,
            "ctr_201": "complete",
            "ctr_202": "not-complete",
            "fnd_202": "not-implemented",
            "rust_orchestrator": "not-implemented",
            "cross_platform_runtime_parity": "not-claimed",
            "real_agent_runtime_parity": "not-claimed",
            "completion_ready": True,
        },
        "compatibility_boundary": {
            "real_agent_execution": "not-executed",
            "real_provider_network": "not-executed",
            "real_timeout_signal_cancel": "not-captured",
            "task_team_checkpoint_resume": "accepted-absent",
            "semantic_quality_gate_execution": "accepted-absent",
            "native_worker_adapter_dispatch": "accepted-generic-fallback",
            "strict_topic_code_build": "not-captured-no-write-disposition",
            "plugin_marketplace_behavior": "not-captured",
            "rust_implementation": "not-implemented",
            "cross_platform_runtime_parity": "not-claimed",
        },
        "integrity": {
            "algorithm": "sha256",
            "canonicalization": CANONICALIZATION,
            "case_manifest_sha256": case_manifest_sha256(cases),
            "payload_sha256": "",
        },
    }
    artifact["integrity"]["payload_sha256"] = canonical_payload_sha256(artifact)
    validate_runtime_artifact(artifact, require_fixed_digests=False)
    return artifact


def validate_runtime_artifact(
    artifact: Mapping[str, Any], *, require_fixed_digests: bool = True
) -> None:
    from tooling.scripts import extract_ctr_201_orchestrator_inventory as static_extractor

    if artifact.get("$schema") != ARTIFACT_SCHEMA or artifact.get("record_type") != RECORD_TYPE:
        raise InventoryMismatch("CTR-201F artifact identity is invalid")
    if artifact.get("task_id") != "CTR-201F" or artifact.get("status") != "runtime-inventory-freeze-captured":
        raise InventoryMismatch("CTR-201F status boundary is invalid")
    cases = artifact.get("cases")
    if not isinstance(cases, list) or [case.get("id") for case in cases] != list(CASE_IDS):
        raise InventoryMismatch("CTR-201F case closure is invalid")
    if any(case.get("declaration_ordinal") != index for index, case in enumerate(cases)):
        raise InventoryMismatch("CTR-201F case ordinals are invalid")
    if any(case.get("case_sha256") != canonical_case_sha256(case) for case in cases):
        raise InventoryMismatch("CTR-201F case digest is invalid")
    dimensions = artifact.get("behavior_dimensions")
    decisions = artifact.get("disposition_decisions")
    if not isinstance(dimensions, list) or [item.get("id") for item in dimensions] != list(DIMENSION_IDS):
        raise InventoryMismatch("CTR-201F dimension closure is invalid")
    if not isinstance(decisions, list) or [item.get("id") for item in decisions] != [
        item["id"] for item in DISPOSITIONS
    ]:
        raise InventoryMismatch("CTR-201F disposition closure is invalid")
    if dimensions != _behavior_dimensions() or decisions != [dict(item) for item in DISPOSITIONS]:
        raise InventoryMismatch("CTR-201F dimension or disposition contract drifted")
    case_ids = set(CASE_IDS)
    decision_ids = {str(item["id"]) for item in DISPOSITIONS}
    dimension_links: set[tuple[str, str]] = set()
    for dimension in dimensions:
        if not set(dimension.get("case_ids", [])).issubset(case_ids) or not set(
            dimension.get("decision_ids", [])
        ).issubset(decision_ids):
            raise InventoryMismatch("CTR-201F dimension references are invalid")
        for case_id in dimension["case_ids"]:
            dimension_links.add((str(dimension["id"]), str(case_id)))
    case_links: set[tuple[str, str]] = set()
    for case in cases:
        if not isinstance(case, Mapping):
            raise InventoryMismatch("CTR-201F case dimension references are invalid")
        case_dimensions = case.get("dimension_ids")
        if (
            not isinstance(case_dimensions, list)
            or not case_dimensions
            or case_dimensions != _case_dimension_ids(str(case["id"]))
        ):
            raise InventoryMismatch("CTR-201F case/dimension links are not bidirectionally exact")
        case_links.update((dimension_id, str(case["id"])) for dimension_id in case_dimensions)
        if case.get("provenance") not in {
            "accepted-a8-oracle",
            "accepted-source-bounded-runtime",
        }:
            raise InventoryMismatch("CTR-201F case provenance is invalid")
        for facts_name in ("input_facts", "result_facts"):
            facts = case.get(facts_name)
            if not isinstance(facts, list) or any(
                not isinstance(row, Mapping)
                or set(row) != {"key", "value"}
                or not isinstance(row.get("key"), str)
                or not isinstance(row.get("value"), str)
                for row in facts
            ):
                raise InventoryMismatch("CTR-201F case facts are invalid")
            keys = [str(row["key"]) for row in facts]
            if len(keys) != len(set(keys)) or any(
                row["value"] in {"missing", "-1"} for row in facts
            ):
                raise InventoryMismatch("CTR-201F case facts contain a sentinel or duplicate")
        outcome = case.get("outcome")
        expected_exception = EXPECTED_EXCEPTION_CASES.get(str(case.get("id")))
        if not isinstance(outcome, Mapping) or (
            expected_exception is None
            and (
                outcome.get("kind") != "result"
                or outcome.get("exception_type") != ""
            )
        ) or (
            expected_exception is not None
            and (
                outcome.get("kind") != "exception"
                or outcome.get("exception_type") != expected_exception[0]
                or outcome.get("exception_message_sha256") != expected_exception[1]
            )
        ):
            raise InventoryMismatch("CTR-201F case outcome boundary is invalid")
        effects = case.get("effects")
        if (
            not isinstance(effects, Mapping)
            or effects.get("before_tree_sha256") != effects.get("after_tree_sha256")
            or effects.get("changed_path_count") != 0
            or effects.get("changed_paths_sha256")
            != _sha256(_canonical_json_bytes([]))
        ):
            raise InventoryMismatch("CTR-201F bounded case contains an unexpected filesystem effect")
    if case_links != dimension_links:
        raise InventoryMismatch("CTR-201F case/dimension relation is not bidirectionally exact")
    source = artifact.get("source")
    expected_source = {
        "accepted_tag": ACCEPTED_TAG,
        "accepted_commit": ACCEPTED_COMMIT,
        "a8_manifest": {"path": MANIFEST_RELATIVE, "sha256": MANIFEST_SHA256},
        "python_full_oracle": {
            "path": PYTHON_ORACLE_RELATIVE,
            "sha256": PYTHON_ORACLE_SHA256,
            "case_id": "python.orchestration-preview",
        },
        "package_trees": [dict(PYTHON_TREE), dict(CONTENT_TREE)],
        "blob_anchors": [dict(item) for item in static_extractor.SOURCE_BINDINGS],
        "ctr_201c": {
            "artifact_path": STATIC_ARTIFACT_RELATIVE,
            "schema_path": STATIC_SCHEMA_RELATIVE,
            "schema_canonical_sha256": STATIC_SCHEMA_CANONICAL_SHA256,
            "payload_sha256": STATIC_ARTIFACT_PAYLOAD_SHA256,
        },
        "ctr_201d": {
            "artifact_path": CONTENT_ARTIFACT_RELATIVE,
            "payload_sha256": CONTENT_ARTIFACT_PAYLOAD_SHA256,
        },
        "accepted_manifest_corpus_sha256": ACCEPTED_MANIFEST_CORPUS_SHA256,
    }
    if source != expected_source:
        raise InventoryMismatch("CTR-201F accepted-source binding is invalid")
    expected_capture_contract = {
        "python_version": "python3.12",
        "python_isolation": "-I-S-B-dual-environment-audit-hook",
        "dependency": "pure-python-pyyaml-6.0.3-capsule",
        "dependency_tree_sha256": PYYAML_PURE_TREE_SHA256,
        "source_mode": "accepted-tag-git-blobs",
        "fake_boundary": "bridge-mcp-availability-and-deterministic-identity-injection",
        "network_policy": "python-audit-denied",
        "process_policy": "worker-denies-child-processes;parent-launches-git-and-python-capture-processes",
        "write_policy": "worker-denies-sut-writes;fixture-roots-before-after-measured",
        "secret_policy": "sanitized-environment-and-canary-rejection",
        "trace_order": "logical-order-preserved-only-concurrent-cohorts-canonicalized",
        "determinism": "two-distinct-temp-roots-and-environments-byte-equivalent",
        "os_sandbox_claim": "authenticated-source-capability-guard-not-hostile-code-os-sandbox",
    }
    if artifact.get("capture_contract") != expected_capture_contract:
        raise InventoryMismatch("CTR-201F capture contract is invalid")
    integrity = artifact.get("integrity")
    if (
        not isinstance(integrity, Mapping)
        or integrity.get("case_manifest_sha256") != case_manifest_sha256(cases)
        or integrity.get("payload_sha256") != canonical_payload_sha256(artifact)
    ):
        raise InventoryMismatch("CTR-201F integrity is invalid")
    if require_fixed_digests:
        if EXPECTED_PAYLOAD_SHA256 == "__GENERATE__" or EXPECTED_CASE_MANIFEST_SHA256 == "__GENERATE__":
            raise InventoryMismatch("CTR-201F fixed digests are not configured")
        if integrity.get("payload_sha256") != EXPECTED_PAYLOAD_SHA256:
            raise InventoryMismatch("CTR-201F payload differs from the fixed digest")
        if integrity.get("case_manifest_sha256") != EXPECTED_CASE_MANIFEST_SHA256:
            raise InventoryMismatch("CTR-201F cases differ from the fixed manifest root")
    strings = _iter_strings(artifact)
    if any(MACHINE_PATH_RE.search(value) for value in strings):
        raise InventoryMismatch("CTR-201F artifact contains a machine-local path")
    if any(SECRET_RE.search(value) for value in strings):
        raise InventoryMismatch("CTR-201F artifact contains secret-shaped data")
    if any(CALLABLE_REPR_RE.search(value) for value in strings):
        raise InventoryMismatch("CTR-201F artifact contains an unstable callable representation")
    coverage = artifact.get("coverage")
    if not isinstance(coverage, Mapping) or coverage != {
        "case_count": len(CASE_IDS),
        "bounded_runtime_case_count": len(CASE_IDS) - 1,
        "accepted_a8_case_count": 1,
        "resolved_dimension_count": len(DIMENSION_IDS),
        "disposition_decision_count": len(DISPOSITIONS),
        "required_not_fully_captured_count": 0,
        "ctr_201": "complete",
        "ctr_202": "not-complete",
        "fnd_202": "not-implemented",
        "rust_orchestrator": "not-implemented",
        "cross_platform_runtime_parity": "not-claimed",
        "real_agent_runtime_parity": "not-claimed",
        "completion_ready": True,
    }:
        raise InventoryMismatch("CTR-201F coverage boundary is invalid")
    if artifact.get("compatibility_boundary") != {
        "real_agent_execution": "not-executed",
        "real_provider_network": "not-executed",
        "real_timeout_signal_cancel": "not-captured",
        "task_team_checkpoint_resume": "accepted-absent",
        "semantic_quality_gate_execution": "accepted-absent",
        "native_worker_adapter_dispatch": "accepted-generic-fallback",
        "strict_topic_code_build": "not-captured-no-write-disposition",
        "plugin_marketplace_behavior": "not-captured",
        "rust_implementation": "not-implemented",
        "cross_platform_runtime_parity": "not-claimed",
    }:
        raise InventoryMismatch("CTR-201F compatibility boundary is invalid")
    case_by_id = {str(case["id"]): case for case in cases}

    for case_id, case in case_by_id.items():
        trace = case.get("trace")
        if not isinstance(trace, list) or [row.get("ordinal") for row in trace] != list(
            range(len(trace))
        ):
            raise InventoryMismatch("CTR-201F trace ordinals are invalid")
        cohort_rows: dict[tuple[int, int], list[Mapping[str, Any]]] = {}
        for row in trace:
            if not isinstance(row, Mapping):
                raise InventoryMismatch("CTR-201F trace row is invalid")
            key = (int(row.get("state_ordinal", -1)), int(row.get("logical_cohort_ordinal", -1)))
            cohort_rows.setdefault(key, []).append(row)
        if sorted({key[1] for key in cohort_rows}) != list(range(len(cohort_rows))):
            raise InventoryMismatch("CTR-201F logical trace cohorts are invalid")
        for cohort in cohort_rows.values():
            ordering = {row.get("ordering") for row in cohort}
            if ordering == {"sequential"}:
                valid = len(cohort) == 1 and cohort[0].get("cohort_member_ordinal") == 0
            elif ordering == {"concurrent"}:
                valid = [row.get("cohort_member_ordinal") for row in cohort] == list(
                    range(len(cohort))
                )
            else:
                valid = False
            if not valid:
                raise InventoryMismatch("CTR-201F trace cohort boundary is invalid")

    def fact_values(case_id: str) -> Mapping[str, str]:
        rows = case_by_id[case_id].get("result_facts")
        if not isinstance(rows, list):
            raise InventoryMismatch("CTR-201F result facts are invalid")
        values = {
            str(row.get("key")): str(row.get("value"))
            for row in rows
            if isinstance(row, Mapping)
        }
        if len(values) != len(rows):
            raise InventoryMismatch("CTR-201F result facts are duplicated or invalid")
        return values

    def require_facts(case_id: str, expected: Mapping[str, Any]) -> None:
        rendered = {key: _fact(key, value)["value"] for key, value in expected.items()}
        if fact_values(case_id) != rendered:
            raise InventoryMismatch(f"CTR-201F {case_id} result projection is invalid")

    def sequential_trace(
        calls: Sequence[tuple[str, str, bool]],
        *,
        state: int = 0,
        start_cohort: int = 0,
    ) -> list[tuple[Any, ...]]:
        return [
            (state, start_cohort + index, 0, "sequential", stage, agent, success)
            for index, (stage, agent, success) in enumerate(calls)
        ]

    def concurrent_trace(
        calls: Sequence[tuple[str, str, bool]],
        *,
        state: int = 0,
        cohort: int = 0,
    ) -> list[tuple[Any, ...]]:
        return [
            (state, cohort, index, "concurrent", stage, agent, success)
            for index, (stage, agent, success) in enumerate(calls)
        ]

    expected_trace_signatures: Mapping[str, list[tuple[Any, ...]]] = {
        "mcp.doctor-advisory-run-agents": sequential_trace(
            [("draft", "claude", True), ("review", "codex", True)]
        ),
        "profile.builtin-and-custom-resolution": (
            concurrent_trace(
                [
                    ("analysis", "antigravity", True),
                    ("analysis", "claude", True),
                    ("analysis", "codex", True),
                ]
            )
            + sequential_trace(
                [
                    ("parallel-synthesis", "claude", True),
                    ("draft", "codex", True),
                    ("review", "claude", True),
                ],
                start_cohort=1,
            )
        ),
        "execute.single-chain-role": (
            sequential_trace([("analysis", "codex", True)])
            + sequential_trace(
                [("analysis", "codex", True), ("verify", "claude", True)],
                state=1,
                start_cohort=1,
            )
            + sequential_trace(
                [
                    ("analysis", "codex", True),
                    ("analysis", "claude", True),
                    ("analysis", "antigravity", True),
                ],
                state=2,
                start_cohort=3,
            )
        ),
        "execute.parallel-triad": (
            concurrent_trace(
                [
                    ("analysis", "antigravity", True),
                    ("analysis", "claude", True),
                    ("analysis", "codex", True),
                ]
            )
            + sequential_trace(
                [("parallel-synthesis", "claude", True)], start_cohort=1
            )
        ),
        "execute.parallel-dual-degrade": (
            concurrent_trace(
                [("analysis", "antigravity", True), ("analysis", "claude", True)]
            )
            + sequential_trace(
                [("parallel-synthesis", "claude", True)], start_cohort=1
            )
        ),
        "code-build.legacy-standard": sequential_trace(
            [
                ("code-build-standard", "codex", True),
                ("code-build-standard", "claude", True),
            ]
        ),
        "code-build.legacy-advanced": sequential_trace(
            [("code-build-advanced", "codex", True), ("verify", "claude", True)]
        ),
        "code-build.legacy-advanced-failure": sequential_trace(
            [("code-build-advanced", "codex", False)]
        ),
        "task-run.solo-observed-review": sequential_trace(
            [("draft", "codex", True), ("review", "antigravity", True)]
        ),
        "task-run.duo-pass": sequential_trace(
            [("draft", "codex", True), ("review", "claude", True)]
        ),
        "task-run.direct-triad-metadata-only": sequential_trace(
            [("draft", "codex", True), ("review", "claude", True)]
        ),
        "task-run.triad-enabled": sequential_trace(
            [
                ("draft", "codex", True),
                ("review", "claude", True),
                ("triad", "antigravity", True),
            ]
        ),
        "task-run.primary-fallback": sequential_trace(
            [("draft", "antigravity", True), ("review", "claude", True)]
        ),
        "task-run.block-revision-pass": sequential_trace(
            [
                ("draft", "codex", True),
                ("review", "claude", True),
                ("revision", "codex", True),
                ("review", "claude", True),
            ]
        ),
        "task-run.final-block": sequential_trace(
            [("draft", "codex", True), ("review", "claude", True)]
        ),
        "task-run.draft-failure": sequential_trace(
            [("draft", "codex", False), ("draft", "antigravity", False)]
        ),
        "worker.b1-success": sequential_trace(
            [
                ("worker", "codex", True),
                ("worker", "codex", True),
                ("worker-merge", "codex", True),
                ("worker-final-review", "codex", True),
                ("draft", "codex", True),
                ("review", "antigravity", True),
            ]
        ),
        "worker.b1-degrade": sequential_trace(
            [
                ("worker", "codex", True),
                ("worker", "codex", False),
                ("worker", "codex", True),
                ("worker-merge", "codex", True),
                ("worker-final-review", "codex", True),
                ("draft", "codex", True),
                ("review", "antigravity", True),
            ]
        ),
        "worker.b1-merge-failure": sequential_trace(
            [
                ("worker", "codex", True),
                ("worker", "codex", True),
                ("worker-merge", "codex", False),
            ]
        ),
        "worker.b1-final-review-failure": sequential_trace(
            [
                ("worker", "codex", True),
                ("worker", "codex", True),
                ("worker-merge", "codex", True),
                ("worker-final-review", "codex", False),
            ]
        ),
        "worker.b1-final-review-block": sequential_trace(
            [
                ("worker", "codex", True),
                ("worker", "codex", True),
                ("worker-merge", "codex", True),
                ("worker-final-review", "codex", True),
            ]
        ),
        "worker.h3-block": sequential_trace(
            [
                ("worker", "codex", False),
                ("worker", "codex", True),
                ("worker", "codex", True),
            ]
        ),
        "team-run.b1-planner-success": (
            sequential_trace([("team-planner", "claude", True)])
            + concurrent_trace(
                [("worker", "claude", True), ("worker", "codex", True)],
                cohort=1,
            )
            + sequential_trace(
                [("team-merge", "claude", True), ("review", "codex", True)],
                start_cohort=2,
            )
        ),
        "team-run.b1-degrade": (
            sequential_trace([("team-planner", "claude", True)])
            + concurrent_trace(
                [
                    ("worker", "antigravity", True),
                    ("worker", "claude", False),
                    ("worker", "codex", True),
                ],
                cohort=1,
            )
            + sequential_trace(
                [("team-merge", "claude", True), ("review", "codex", True)],
                start_cohort=2,
            )
        ),
        "team-run.b1-planner-fallback": (
            sequential_trace([("team-planner", "claude", False)])
            + concurrent_trace([("worker", "codex", True)], cohort=1)
            + sequential_trace(
                [("team-merge", "claude", True), ("review", "codex", True)],
                start_cohort=2,
            )
        ),
        "team-run.b1-all-workers-block": (
            sequential_trace([("team-planner", "claude", True)])
            + concurrent_trace(
                [("worker", "claude", False), ("worker", "codex", False)],
                cohort=1,
            )
        ),
        "team-run.b1-merge-failure": (
            sequential_trace([("team-planner", "claude", True)])
            + concurrent_trace(
                [("worker", "claude", True), ("worker", "codex", True)],
                cohort=1,
            )
            + sequential_trace(
                [("team-merge", "claude", False)], start_cohort=2
            )
        ),
        "team-run.b1-review-failure": (
            sequential_trace([("team-planner", "claude", True)])
            + concurrent_trace(
                [("worker", "claude", True), ("worker", "codex", True)],
                cohort=1,
            )
            + sequential_trace(
                [("team-merge", "claude", True), ("review", "codex", False)],
                start_cohort=2,
            )
        ),
        "team-run.b1-review-block-observed": (
            sequential_trace([("team-planner", "claude", True)])
            + concurrent_trace(
                [("worker", "claude", True), ("worker", "codex", True)],
                cohort=1,
            )
            + sequential_trace(
                [("team-merge", "claude", True), ("review", "codex", True)],
                start_cohort=2,
            )
        ),
        "team-run.h3-static-personas": (
            concurrent_trace(
                [
                    ("worker", "antigravity", True),
                    ("worker", "claude", True),
                    ("worker", "codex", True),
                ]
            )
            + sequential_trace(
                [("team-merge", "claude", True), ("review", "codex", True)],
                start_cohort=1,
            )
        ),
        "team-run.h3-block": concurrent_trace(
            [
                ("worker", "antigravity", True),
                ("worker", "claude", True),
                ("worker", "codex", False),
            ]
        ),
    }

    require_facts(
        "mcp.preview-controller-matrix",
        {
            "preview_matrix": {
                mode: {
                    "mode": "task-run-preview",
                    "run_agents": False,
                    "execution_mode": mode,
                    "will_launch_agents": False,
                }
                for mode in ("solo", "duo", "triad")
            }
        },
    )
    require_facts(
        "mcp.run-agents-boolean-contract",
        {"missing": False, "false": False, "true": True, "string_true_rejected": True},
    )
    require_facts(
        "mcp.doctor-advisory-run-agents",
        {"doctor_calls_during_task_run": 0},
    )
    require_facts(
        "mcp.orchestrator-route-matrix",
        {
            "skill_route": "skill_workflow",
            "skill_tools": ["qiongli_task_plan"],
            "skill_missing": ["task_id", "paper_type", "topic"],
            "orchestrator_route": "orchestrator_mcp",
            "orchestrator_tools": [
                "qiongli_orchestrator_doctor",
                "qiongli_task_plan",
                "qiongli_task_run",
            ],
            "orchestrator_run_agents": False,
            "platform": "claude_code",
            "requires_full_runtime": True,
            "route_safety_claims_doctor_gate": True,
        },
    )
    require_facts(
        "task-plan.prerequisite-filesystem-matrix",
        {
            "plan_matrix": {
                "f3_empty": {
                    "confidence": 0.6,
                    "missing_prerequisites_all": ["A1", "A2", "F1"],
                    "any_of": [],
                },
                "f3_satisfied": {
                    "confidence": 1.0,
                    "missing_prerequisites_all": [],
                    "any_of": [],
                },
                "b3_any_satisfied": {
                    "confidence": 1.0,
                    "missing_prerequisites_all": [],
                    "any_of": [
                        {"task": "B3", "satisfied": True, "satisfied_by": "B1"}
                    ],
                },
            }
        },
    )
    require_facts(
        "quality.artifact-existence-gate",
        {
            "gate_results": {
                "empty_file": {
                    "passed": True,
                    "found": ["empty.txt"],
                    "missing": [],
                    "checked": 1,
                },
                "empty_directory": {
                    "passed": False,
                    "found": [],
                    "missing": ["empty-dir/"],
                    "checked": 1,
                },
                "nonempty_directory": {
                    "passed": True,
                    "found": ["nonempty-dir/"],
                    "missing": [],
                    "checked": 1,
                },
            },
            "semantic_execution": False,
        },
    )
    require_facts(
        "profile.builtin-and-custom-resolution",
        {
            "builtin_profile_count": 5,
            "parallel_codex_timeout_seconds": 123,
            "draft_timeout_seconds": 123,
            "parallel_profile_applied": True,
            "draft_profile_applied": True,
            "review_profile_applied": True,
        },
    )
    require_facts(
        "execute.single-chain-role",
        {
            "mode_results": {
                "single": {
                    "result_mode": "single",
                    "confidence": 1.0,
                    "codex_success": True,
                    "claude_success": False,
                    "antigravity_success": False,
                },
                "chain": {
                    "result_mode": "chain",
                    "confidence": 0.85,
                    "codex_success": True,
                    "claude_success": True,
                    "antigravity_success": False,
                },
                "role": {
                    "result_mode": "role_based",
                    "confidence": 1.0,
                    "codex_success": True,
                    "claude_success": True,
                    "antigravity_success": True,
                },
            }
        },
    )
    require_facts(
        "execute.parallel-triad",
        {
            "available_agents": ["antigravity", "claude", "codex"],
            "mode": "parallel",
            "confidence": 0.93,
            "response_success": {"codex": True, "claude": True, "antigravity": True},
        },
    )
    require_facts(
        "execute.parallel-dual-degrade",
        {
            "available_agents": ["antigravity", "claude"],
            "mode": "parallel",
            "confidence": 0.8,
            "response_success": {"codex": False, "claude": True, "antigravity": True},
        },
    )
    require_facts(
        "code-build.focus-routing",
        {
            "mappings": {
                "default": {"normalized": "implementation", "task_id": "I1"},
                "implementation": {"normalized": "implementation", "task_id": "I1"},
                "planning": {"normalized": "planning", "task_id": "I6"},
                "execution": {"normalized": "execution", "task_id": "I7"},
                "review": {"normalized": "review", "task_id": "I8"},
                "full": {"normalized": "full", "task_id": "FULL"},
            },
            "target_map": {"I6": ["S1", "S2"], "I8": ["P1-01"]},
            "invalid_selector_rejected": True,
        },
    )
    for case_id, expected in {
        "code-build.legacy-standard": {
            "mode": "role_based",
            "confidence": 1.0,
            "response_success": {"codex": True, "claude": True, "antigravity": False},
        },
        "code-build.legacy-advanced": {
            "mode": "chain",
            "confidence": 0.85,
            "response_success": {"codex": True, "claude": True, "antigravity": False},
        },
        "code-build.legacy-advanced-failure": {
            "mode": "chain",
            "confidence": 0.0,
            "response_success": {"codex": False, "claude": False, "antigravity": False},
        },
    }.items():
        require_facts(case_id, expected)

    task_expectations = {
        "task-run.solo-observed-review": (0.9, "passed", "PASS", 1, 0, True),
        "task-run.duo-pass": (0.9, "passed", "PASS", 1, 0, True),
        "task-run.direct-triad-metadata-only": (0.9, "passed", "PASS", 1, 0, True),
        "task-run.triad-enabled": (0.95, "passed", "PASS", 1, 0, True),
        "task-run.primary-fallback": (0.9, "passed", "PASS", 1, 0, True),
        "task-run.block-revision-pass": (0.9, "passed", "PASS", 2, 1, True),
        "task-run.final-block": (0.55, "blocked", "BLOCK", 1, 0, True),
        "task-run.draft-failure": (0.0, "skipped", "NOT_RUN", 0, 0, True),
    }
    for case_id, (
        confidence,
        loop_status,
        verdict,
        reviews,
        revisions,
        validator_passed,
    ) in task_expectations.items():
        require_facts(
            case_id,
            {
                "trace_count": len(expected_trace_signatures[case_id]),
                "mode": "task-run",
                "confidence": confidence,
                "review_loop_status": loop_status,
                "final_verdict": verdict,
                "reviews_completed": reviews,
                "revisions_attempted": revisions,
                "validator_gate_passed": validator_passed,
            },
        )

    require_facts(
        "failure.policy-boundary-matrix",
        {
            "policy_matrix": {
                "worker_degraded": "degraded",
                "worker_below_threshold": "blocked",
                "worker_block": "blocked",
                "team_degraded": "degraded",
                "team_degraded_success_count": 2,
                "team_below_threshold": "blocked",
                "team_below_threshold_success_count": 0,
                "team_block": "blocked",
                "team_block_success_count": 0,
            }
        },
    )
    require_facts(
        "worker.adapter-fallback",
        {
            "effective_adapters": {
                "auto": "generic_prompt",
                "codex_subagent": "generic_prompt",
                "claude_cowork": "generic_prompt",
            },
            "mcp_worker_controls_present": False,
        },
    )
    worker_expectations: Mapping[str, Mapping[str, Any]] = {
        "worker.b1-success": {
            "confidence": 0.9,
            "review_loop_status": "passed",
            "final_verdict": "PASS",
            "reviews_completed": 1,
            "validator_gate_passed": True,
            "worker_count": 2,
            "worker_failure_count": 0,
            "worker_status": "completed",
            "worker_barrier_status": "ok",
            "worker_merge_status": "passed",
            "worker_final_review_status": "passed",
            "worker_final_review_verdict": "PASS",
            "worker_final_review_confidence": 0.9,
            "review_loop_stopped_reason": (
                "Review PASSED at round 0 (confidence=0.90); loop converged."
            ),
            "validator_gate_skipped": True,
            "validator_gate_reason": "--skip-validation requested",
        },
        "worker.b1-degrade": {
            "confidence": 0.9,
            "review_loop_status": "passed",
            "final_verdict": "PASS",
            "reviews_completed": 1,
            "validator_gate_passed": True,
            "worker_count": 3,
            "worker_failure_count": 1,
            "worker_status": "completed",
            "worker_barrier_status": "degraded",
            "worker_merge_status": "passed",
            "worker_final_review_status": "passed",
            "worker_final_review_verdict": "PASS",
            "worker_final_review_confidence": 0.9,
            "review_loop_stopped_reason": (
                "Review PASSED at round 0 (confidence=0.90); loop converged."
            ),
            "validator_gate_skipped": True,
            "validator_gate_reason": "--skip-validation requested",
        },
        "worker.b1-merge-failure": {
            "confidence": 0.0,
            "review_loop_status": "skipped",
            "final_verdict": "NOT_RUN",
            "reviews_completed": 0,
            "validator_gate_passed": False,
            "worker_count": 2,
            "worker_failure_count": 0,
            "worker_status": "blocked",
            "worker_barrier_status": "ok",
            "worker_merge_status": "failed",
            "worker_final_review_status": "merge_failed",
            "worker_final_review_verdict": "",
            "worker_final_review_confidence": None,
            "review_loop_stopped_reason": "worker merge failed",
            "validator_gate_skipped": True,
            "validator_gate_reason": "worker merge failed",
        },
        "worker.b1-final-review-failure": {
            "confidence": 0.0,
            "review_loop_status": "skipped",
            "final_verdict": "NOT_RUN",
            "reviews_completed": 0,
            "validator_gate_passed": False,
            "worker_count": 2,
            "worker_failure_count": 0,
            "worker_status": "blocked",
            "worker_barrier_status": "ok",
            "worker_merge_status": "passed",
            "worker_final_review_status": "failed",
            "worker_final_review_verdict": "",
            "worker_final_review_confidence": None,
            "review_loop_stopped_reason": "worker final review failed",
            "validator_gate_skipped": True,
            "validator_gate_reason": "worker final review failed",
        },
        "worker.b1-final-review-block": {
            "confidence": 0.0,
            "review_loop_status": "skipped",
            "final_verdict": "NOT_RUN",
            "reviews_completed": 0,
            "validator_gate_passed": False,
            "worker_count": 2,
            "worker_failure_count": 0,
            "worker_status": "blocked",
            "worker_barrier_status": "ok",
            "worker_merge_status": "passed",
            "worker_final_review_status": "blocked",
            "worker_final_review_verdict": "BLOCK",
            "worker_final_review_confidence": 0.2,
            "review_loop_stopped_reason": "worker final review blocked",
            "validator_gate_skipped": True,
            "validator_gate_reason": "worker final review blocked",
        },
        "worker.h3-block": {
            "confidence": 0.0,
            "review_loop_status": "skipped",
            "final_verdict": "NOT_RUN",
            "reviews_completed": 0,
            "validator_gate_passed": False,
            "worker_count": 3,
            "worker_failure_count": 1,
            "worker_status": "blocked",
            "worker_barrier_status": "blocked",
            "worker_merge_status": "skipped",
            "worker_final_review_status": "skipped",
            "worker_final_review_verdict": "",
            "worker_final_review_confidence": None,
            "review_loop_stopped_reason": "worker barrier blocked",
            "validator_gate_skipped": True,
            "validator_gate_reason": "worker barrier blocked",
        },
    }
    for case_id, expected in worker_expectations.items():
        require_facts(
            case_id,
            {
                "trace_count": len(expected_trace_signatures[case_id]),
                "mode": "task-run",
                "revisions_attempted": 0,
                **expected,
            },
        )

    team_expectations: Mapping[str, tuple[Any, ...]] = {
        "team-run.b1-planner-success": ("B1", "ok", 2, 0, True, True, False, 0.92),
        "team-run.b1-degrade": ("B1", "degraded", 3, 1, True, True, False, 0.92),
        "team-run.b1-planner-fallback": ("B1", "ok", 1, 0, True, True, False, 0.92),
        "team-run.b1-all-workers-block": ("B1", "blocked", 2, 2, False, False, False, 0.0),
        "team-run.b1-merge-failure": ("B1", "ok", 2, 0, True, False, False, 0.3),
        "team-run.b1-review-failure": ("B1", "ok", 2, 0, True, True, False, 0.78),
        "team-run.b1-review-block-observed": ("B1", "ok", 2, 0, True, True, True, 0.92),
        "team-run.h3-static-personas": ("H3", "ok", 3, 0, True, True, False, 0.92),
        "team-run.h3-block": ("H3", "blocked", 3, 1, False, False, False, 0.0),
    }
    for case_id, (
        task_id,
        barrier_status,
        worker_count,
        failure_count,
        merge_executed,
        review_executed,
        review_block_observed,
        confidence,
    ) in team_expectations.items():
        require_facts(
            case_id,
            {
                "task_id": task_id,
                "trace_count": len(expected_trace_signatures[case_id]),
                "barrier_status": barrier_status,
                "confidence": confidence,
                "worker_count": worker_count,
                "worker_failure_count": failure_count,
                "merge_executed": merge_executed,
                "review_executed": review_executed,
                "review_block_observed": review_block_observed,
            },
        )

    require_facts(
        "experience.replay-plan-advisory",
        {
            "execution_performed": False,
            "failed_validator_status": "failed",
            "failed_failure_modes": ["missing-output"],
            "failed_next_action": "rerun_after_addressing_failures",
            "passed_validator_status": "passed",
            "passed_failure_modes": [],
            "passed_next_action": "no_rerun_needed",
        },
    )
    require_facts(
        "bridge.session-command-passthrough",
        {
            "session_id": "fixture-session",
            "task_run_has_resume": False,
            "team_run_has_resume": False,
            "task_run_has_cancel": False,
            "team_run_has_cancel": False,
            "codex_session_forwarded": True,
            "claude_session_forwarded": True,
            "antigravity_session_forwarded": True,
        },
    )
    require_facts(
        "doctor.sanitized-environment",
        {
            "executes_commands": False,
            "cli_statuses": {
                "codex": "warning",
                "claude": "warning",
                "antigravity": "warning",
            },
        },
    )

    traced_case_ids = {
        case_id for case_id, case in case_by_id.items() if case.get("trace")
    }
    if traced_case_ids != set(expected_trace_signatures):
        raise InventoryMismatch("CTR-201F traced case closure is invalid")
    for case_id, expected in expected_trace_signatures.items():
        actual = [
            (
                row["state_ordinal"],
                row["logical_cohort_ordinal"],
                row["cohort_member_ordinal"],
                row["ordering"],
                row["stage"],
                row["agent"],
                row["success"],
            )
            for row in case_by_id[case_id]["trace"]
        ]
        if actual != expected:
            raise InventoryMismatch(f"CTR-201F {case_id} logical trace is invalid")


def _schema_for_values(values: Sequence[Any]) -> Mapping[str, Any]:
    from tooling.scripts.extract_ctr_201_orchestrator_inventory import _schema_for_values

    return _schema_for_values(values)


def _set_scalar_consts(schema: Mapping[str, Any], value: Mapping[str, Any]) -> None:
    from tooling.scripts.extract_ctr_201_orchestrator_inventory import _set_scalar_consts

    _set_scalar_consts(schema, value)


def build_runtime_schema(artifact: Mapping[str, Any]) -> Mapping[str, Any]:
    inferred = dict(_schema_for_values([artifact]))
    properties = inferred.get("properties")
    if inferred.get("type") != "object" or not isinstance(properties, dict):
        raise ExtractorError("generated CTR-201F schema is invalid")
    for key in ("$schema", "schema_version", "record_type", "task_id", "status"):
        properties[key] = {"const": artifact[key]}
    for section in ("capture_contract", "coverage", "compatibility_boundary"):
        section_schema = properties.get(section)
        if not isinstance(section_schema, Mapping):
            raise ExtractorError("generated CTR-201F schema section is invalid")
        _set_scalar_consts(section_schema, artifact[section])
    integrity = properties.get("integrity")
    if not isinstance(integrity, Mapping) or not isinstance(integrity.get("properties"), dict):
        raise ExtractorError("generated CTR-201F integrity schema is invalid")
    integrity["properties"]["algorithm"] = {"const": "sha256"}
    integrity["properties"]["canonicalization"] = {"const": CANONICALIZATION}
    for key in ("case_manifest_sha256", "payload_sha256"):
        integrity["properties"][key] = {"type": "string", "pattern": "^[0-9a-f]{64}$"}
    return {
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": SCHEMA_ID,
        **inferred,
    }


def _write_json(path: Path, value: Mapping[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    rendered = json.dumps(value, ensure_ascii=False, sort_keys=True, indent=2, allow_nan=False) + "\n"
    path.write_text(rendered, encoding="utf-8", newline="\n")


def _worker_main(control_path: str) -> int:
    try:
        control = _load_json(Path(control_path), label="worker control")
        cases = _capture_worker(control)
        sys.stdout.write(json.dumps({"status": "pass", "cases": cases}, ensure_ascii=False, sort_keys=True))
        return 0
    except (ExtractorError, InventoryMismatch, OSError, ValueError, PermissionError) as error:
        if os.environ.get("CTR201F_WORKER_DEBUG") == "1":
            import traceback

            traceback.print_exception(error, file=sys.stderr)
        sys.stdout.write(json.dumps({"status": "error", "code": "worker-failed"}, sort_keys=True))
        return 2


def _build_parser() -> argparse.ArgumentParser:
    parser = _RedactedArgumentParser(
        description="Extract the accepted CTR-201F orchestrator runtime inventory."
    )
    parser.add_argument("--root", default=str(REPO_ROOT))
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--json", action="store_true")
    parser.add_argument("--output")
    parser.add_argument("--schema-output")
    parser.add_argument("--_worker", help=argparse.SUPPRESS)
    return parser


def _emit(json_mode: bool, status: str, exit_code: int, code: str, artifact: Mapping[str, Any] | None = None) -> None:
    payload: dict[str, Any] = {"status": status, "exit_code": exit_code, "code": code}
    if artifact is not None:
        payload.update(
            {
                "case_count": len(artifact["cases"]),
                "payload_sha256": artifact["integrity"]["payload_sha256"],
                "case_manifest_sha256": artifact["integrity"]["case_manifest_sha256"],
                "ctr_201": artifact["coverage"]["ctr_201"],
                "ctr_202": artifact["coverage"]["ctr_202"],
                "fnd_202": artifact["coverage"]["fnd_202"],
            }
        )
    if json_mode:
        print(json.dumps(payload, ensure_ascii=False, sort_keys=True))
    elif status == "pass":
        print(f"[ctr-201f] {code}")
    else:
        print(f"[ctr-201f] {code}", file=sys.stderr)


def main(argv: Sequence[str] | None = None) -> int:
    parser = _build_parser()
    args_list = list(argv) if argv is not None else sys.argv[1:]
    try:
        args = parser.parse_args(args_list)
        if args._worker:
            return _worker_main(args._worker)
        if args.check and (args.output or args.schema_output):
            raise UsageError("check and output modes are mutually exclusive")
        if not args.check and not (args.output and args.schema_output):
            raise UsageError("generation requires --output and --schema-output")
        artifact = extract_orchestrator_runtime_inventory(Path(args.root))
        schema = build_runtime_schema(artifact)
        if args.check:
            validate_runtime_artifact(artifact, require_fixed_digests=True)
            checked_artifact = _load_json(
                Path(args.root) / DEFAULT_OUTPUT_RELATIVE, label="checked CTR-201F artifact"
            )
            checked_schema = _load_json(
                Path(args.root) / DEFAULT_SCHEMA_RELATIVE, label="checked CTR-201F schema"
            )
            if _canonical_json_bytes(artifact) != _canonical_json_bytes(checked_artifact):
                raise InventoryMismatch("checked CTR-201F artifact differs from extraction")
            if _canonical_json_bytes(schema) != _canonical_json_bytes(checked_schema):
                raise InventoryMismatch("checked CTR-201F schema differs from extraction")
            code = "accepted-orchestrator-runtime-inventory-matches"
        else:
            _write_json(Path(args.output), artifact)
            _write_json(Path(args.schema_output), schema)
            code = "accepted-orchestrator-runtime-inventory-written"
        _emit(args.json, "pass", 0, code, artifact)
        return 0
    except UsageError:
        _emit("--json" in args_list, "error", 2, "accepted-orchestrator-runtime-inventory-unavailable")
        return 2
    except InventoryMismatch:
        _emit("--json" in args_list, "fail", 1, "accepted-orchestrator-runtime-inventory-mismatch")
        return 1
    except (ExtractorError, OSError, ValueError):
        _emit("--json" in args_list, "error", 2, "accepted-orchestrator-runtime-inventory-unavailable")
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
