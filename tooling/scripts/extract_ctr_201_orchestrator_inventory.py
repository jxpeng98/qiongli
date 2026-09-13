#!/usr/bin/env python3
from __future__ import annotations

import argparse
import ast
import hashlib
import io
import json
import math
import os
from pathlib import Path, PurePosixPath
import re
import subprocess
import sys
from typing import Any, Mapping, Sequence

try:
    import yaml
except ImportError:  # Keep the public CLI inside its redacted exit contract.
    yaml = None  # type: ignore[assignment]


REPO_ROOT = Path(__file__).resolve().parents[2]
MANIFEST_RELATIVE = "tooling/migration/baselines/v1.19.0-beta.1/manifest.json"
MANIFEST_SHA256 = "77bb7628d43a496c995e4b0a8daf6a624847b62e96948c0461affe89002da131"
PYTHON_ORACLE_RELATIVE = (
    "tooling/migration/baselines/v1.19.0-beta.1/oracles/python-full.json"
)
PYTHON_ORACLE_SHA256 = "26d247c9268c3166c98080aef420acfdb8248f62b11cc69420250f6e493a23e3"
DEFAULT_OUTPUT_RELATIVE = "tooling/migration/ctr-201-orchestrator.json"
DEFAULT_SCHEMA_RELATIVE = "tooling/migration/ctr-201-orchestrator.schema.json"
ACCEPTED_TAG = "v1.19.0-beta.1"
ACCEPTED_COMMIT = "8d2e99866ce4c4efb8b3b5e0265c0c1f89a36b0f"
ARTIFACT_SCHEMA = "./ctr-201-orchestrator.schema.json"
ARTIFACT_SCHEMA_VERSION = "1.0"
ARTIFACT_RECORD_TYPE = "qiongli-ctr-201-orchestrator-static-semantics"
SCHEMA_ID = (
    "https://qiongli.dev/schemas/ctr-201-orchestrator-static-semantics-v1.json"
)
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

SOURCE_BINDINGS: tuple[Mapping[str, Any], ...] = (
    {
        "role": "orchestrator-module",
        "consumption_class": "source-constant-and-runtime-module",
        "path": "packages/python-qiongli/src/qiongli/bridges/orchestrator.py",
        "mode": "100644",
        "git_blob_oid": "219743fcae7674dc709c311b90df3684ff98435d",
        "sha256": "e9e5899f69ee1853c2105724100a52096c8f0f5353863ef4c1c794424212cf6e",
        "size_bytes": 355170,
    },
    {
        "role": "orchestrator-mcp-boundary",
        "consumption_class": "runtime-loaded-mcp-boundary",
        "path": "packages/python-qiongli/src/qiongli/bridges/mcp_tool_handlers.py",
        "mode": "100644",
        "git_blob_oid": "173da40211ba04527423f474807f8a773f98d9d1",
        "sha256": "f1aacdc2d00667497416271f20a5121fed04d7198c0bddd88ce4fa4076aeb3fd",
        "size_bytes": 53455,
    },
    {
        "role": "workflow-contract",
        "consumption_class": "runtime-loaded-declarative-contract",
        "path": "content/standards/research-workflow-contract.yaml",
        "mode": "100644",
        "git_blob_oid": "609cdd9e6616c838b7ec959c7402d37f3f118345",
        "sha256": "b77b901b552bb235edde21476763cab6149b63d329d124cf00c92c2fece6cf63",
        "size_bytes": 41041,
    },
    {
        "role": "agent-capability-map",
        "consumption_class": "runtime-loaded-declarative-contract",
        "path": "content/standards/mcp-agent-capability-map.yaml",
        "mode": "100644",
        "git_blob_oid": "d925febb940964b95fcb9b082295f91efcdb7435",
        "sha256": "b1ff20d2edcec9cfc02669f635ee77c2ec17d5bc23531ed5ebd481c9865a661a",
        "size_bytes": 62173,
    },
)

EXPECTED_COUNTS = {
    "stage_count": 13,
    "task_count": 76,
    "prerequisites_all_edge_count": 54,
    "prerequisites_any_edge_count": 50,
    "required_dependency_edge_count": 104,
    "recommended_prerequisite_edge_count": 44,
    "recommended_next_edge_count": 154,
    "task_output_assignment_count": 136,
    "unique_task_output_count": 118,
    "runtime_agent_count": 3,
    "functional_agent_count": 9,
    "functional_stage_default_count": 13,
    "functional_task_override_count": 16,
    "routing_skill_id_count": 82,
    "task_skill_assignment_count": 207,
    "logical_mcp_capability_count": 11,
    "task_mcp_assignment_count": 139,
    "quality_gate_count": 4,
    "task_quality_gate_assignment_count": 136,
    "declared_profile_count": 5,
    "team_run_task_count": 2,
    "worker_orchestration_task_count": 2,
    "source_anchor_count": 4,
}

CHOICE_CONSTANTS = (
    "RUNTIME_AGENT_CHOICES",
    "CONTROLLER_EXECUTION_MODE_CHOICES",
    "SOLO_ROLE_GATE_CHOICES",
    "WORKER_MODE_CHOICES",
    "WORKER_ADAPTER_CHOICES",
)
SUMMARY_CONSTANTS = (
    "DEFAULT_AGENT_PROFILES",
    "DOMAIN_PROFILE_ALIASES",
    "CODE_BUILD_FOCUS_TO_TASK",
    "STAGE_I_TEMPLATE_TYPE_BY_TASK",
    "STAGE_I_CONTRACT_HEADING_BY_TASK",
    "STAGE_I_CONTRACT_KEYS_BY_TASK",
    "ACADEMIC_CONTEXT_TASK_IDS",
    "WRITING_HARNESS_SKILLS",
)
PUBLIC_METHODS = ("execute", "task_plan", "doctor", "team_run", "task_run", "code_build")
SOURCE_SYMBOLS = {
    "packages/python-qiongli/src/qiongli/bridges/orchestrator.py": (
        "ModelOrchestrator._load_task_agent_plan",
        "ModelOrchestrator._load_task_functional_plan",
        "ModelOrchestrator._build_functional_handoff_trace",
        "ModelOrchestrator._resolve_runtime_agent",
        "ModelOrchestrator._runtime_preflight_error",
        "ModelOrchestrator._execute_runtime_agent",
        "ModelOrchestrator._build_controller_metadata",
        "ModelOrchestrator._controller_runtime_overrides",
        "ModelOrchestrator._load_worker_orchestration_config",
        "ModelOrchestrator._resolve_worker_orchestration_adapter",
        "ModelOrchestrator._build_worker_orchestration_plan",
        "ModelOrchestrator._apply_worker_barrier",
        "ModelOrchestrator._collect_skill_context",
        "ModelOrchestrator.task_plan",
        "ModelOrchestrator.task_run",
        "ModelOrchestrator.team_run",
    ),
    "packages/python-qiongli/src/qiongli/bridges/mcp_tool_handlers.py": (
        "_tool_orchestrator_route",
        "_orchestrator_route_signals",
        "_normalize_platform",
        "_tool_task_run",
        "_run_agents_enabled",
        "_task_run_preview",
    ),
}

HEX_40 = re.compile(r"^[0-9a-f]{40}$")
HEX_64 = re.compile(r"^[0-9a-f]{64}$")
TASK_ID = re.compile(r"^[A-Z][0-9]+(?:_[0-9]+)?$")
MAX_AST_NODES = 50_000
MAX_AST_DEPTH = 32
MAX_YAML_NODES = 50_000
MAX_YAML_DEPTH = 32


class ExtractorError(RuntimeError):
    """The frozen source or extraction toolchain cannot be evaluated safely."""


class InventoryMismatch(RuntimeError):
    """The frozen source differs from an accepted identity or checked artifact."""


class UsageError(RuntimeError):
    """Public extractor usage is invalid and must be reported without echoing input."""


class _RedactedArgumentParser(argparse.ArgumentParser):
    def error(self, _message: str) -> None:  # pragma: no cover - exercised via main
        raise UsageError("invalid command usage")


def _require_toolchain() -> None:
    if sys.version_info[:2] != (3, 12):
        raise ExtractorError("CTR-201C extraction requires Python 3.12")
    if yaml is None or getattr(yaml, "__version__", "") != "6.0.3":
        raise ExtractorError("CTR-201C extraction requires PyYAML 6.0.3")


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
    return _normalize_json(value)


def _normalize_json(value: Any, *, depth: int = 0, seen: set[int] | None = None) -> Any:
    if depth > MAX_YAML_DEPTH:
        raise ExtractorError("declarative data exceeds the nesting limit")
    if value is None or isinstance(value, (str, bool, int)):
        if isinstance(value, str) and any(0xD800 <= ord(char) <= 0xDFFF for char in value):
            raise ExtractorError("declarative data contains invalid Unicode")
        return value
    if isinstance(value, float):
        if not math.isfinite(value):
            raise ExtractorError("declarative data contains a non-finite number")
        return value
    if seen is None:
        seen = set()
    if isinstance(value, Mapping):
        identity = id(value)
        if identity in seen:
            raise ExtractorError("declarative data contains an alias or cycle")
        seen.add(identity)
        result: dict[str, Any] = {}
        for key, item in value.items():
            if not isinstance(key, str):
                raise ExtractorError("declarative mapping keys must be strings")
            if key in result:
                raise ExtractorError("declarative data contains a duplicate key")
            result[key] = _normalize_json(item, depth=depth + 1, seen=seen)
        seen.remove(identity)
        return result
    if isinstance(value, (list, tuple)):
        identity = id(value)
        if identity in seen:
            raise ExtractorError("declarative data contains an alias or cycle")
        seen.add(identity)
        result = [_normalize_json(item, depth=depth + 1, seen=seen) for item in value]
        seen.remove(identity)
        return result
    if isinstance(value, set):
        normalized = [_normalize_json(item, depth=depth + 1, seen=seen) for item in value]
        return sorted(normalized, key=lambda item: _canonical_json_bytes(item))
    raise ExtractorError("declarative data contains a non-JSON value")


_YamlSafeLoaderBase: Any = yaml.SafeLoader if yaml is not None else object


class _StrictSafeLoader(_YamlSafeLoaderBase):
    def __init__(self, stream: str) -> None:
        if yaml is None:
            raise ExtractorError("CTR-201C extraction requires PyYAML 6.0.3")
        super().__init__(stream)
        self._qiongli_node_count = 0
        self._qiongli_depth = 0
        self._qiongli_allowed_identical_duplicates: dict[str, int] = {}
        self._qiongli_observed_identical_duplicates: dict[str, int] = {}

    def compose_node(self, parent: Any, index: Any) -> yaml.Node:
        if self.check_event(yaml.AliasEvent):
            raise ExtractorError("YAML aliases are not allowed")
        event = self.peek_event()
        if getattr(event, "anchor", None) is not None:
            raise ExtractorError("YAML anchors are not allowed")
        self._qiongli_depth += 1
        if self._qiongli_depth > MAX_YAML_DEPTH:
            raise ExtractorError("YAML exceeds the nesting limit")
        try:
            node = super().compose_node(parent, index)
        finally:
            self._qiongli_depth -= 1
        self._qiongli_node_count += 1
        if self._qiongli_node_count > MAX_YAML_NODES:
            raise ExtractorError("YAML exceeds the node limit")
        allowed_tags = {
            "tag:yaml.org,2002:null",
            "tag:yaml.org,2002:bool",
            "tag:yaml.org,2002:int",
            "tag:yaml.org,2002:float",
            "tag:yaml.org,2002:str",
            "tag:yaml.org,2002:seq",
            "tag:yaml.org,2002:map",
        }
        if node.tag not in allowed_tags:
            raise ExtractorError("YAML contains an unsupported tag")
        return node

    def construct_mapping(self, node: yaml.MappingNode, deep: bool = False) -> dict[str, Any]:
        if not isinstance(node, yaml.MappingNode):
            raise ExtractorError("YAML mapping node is invalid")
        result: dict[str, Any] = {}
        value_nodes: dict[str, yaml.Node] = {}
        for key_node, value_node in node.value:
            if key_node.tag == "tag:yaml.org,2002:merge":
                raise ExtractorError("YAML merge keys are not allowed")
            key = self.construct_object(key_node, deep=deep)
            if not isinstance(key, str):
                raise ExtractorError("YAML mapping keys must be strings")
            if key in result:
                allowed = self._qiongli_allowed_identical_duplicates.get(key, 0)
                observed = self._qiongli_observed_identical_duplicates.get(key, 0)
                if observed >= allowed or _yaml_node_fingerprint(value_nodes[key]) != _yaml_node_fingerprint(
                    value_node
                ):
                    raise ExtractorError("YAML contains a duplicate key")
                self._qiongli_observed_identical_duplicates[key] = observed + 1
            result[key] = self.construct_object(value_node, deep=deep)
            value_nodes[key] = value_node
        return result


if yaml is not None:
    _StrictSafeLoader.add_constructor(
        yaml.resolver.BaseResolver.DEFAULT_MAPPING_TAG,
        _StrictSafeLoader.construct_mapping,
    )


def _yaml_node_fingerprint(node: yaml.Node) -> Any:
    if isinstance(node, yaml.ScalarNode):
        return ("scalar", node.tag, node.value)
    if isinstance(node, yaml.SequenceNode):
        return ("sequence", node.tag, tuple(_yaml_node_fingerprint(item) for item in node.value))
    if isinstance(node, yaml.MappingNode):
        return (
            "mapping",
            node.tag,
            tuple(
                (_yaml_node_fingerprint(key), _yaml_node_fingerprint(value))
                for key, value in node.value
            ),
        )
    raise ExtractorError("YAML contains an unsupported node")


def _load_yaml_bytes(
    data: bytes, *, allowed_identical_duplicates: Mapping[str, int] | None = None
) -> Mapping[str, Any]:
    try:
        text = data.decode("utf-8")
        loader = _StrictSafeLoader(text)
        loader._qiongli_allowed_identical_duplicates = dict(allowed_identical_duplicates or {})
        try:
            value = loader.get_single_data()
        finally:
            observed_duplicates = dict(loader._qiongli_observed_identical_duplicates)
            loader.dispose()
    except (UnicodeDecodeError, yaml.YAMLError) as error:
        raise ExtractorError("YAML document is invalid") from error
    if observed_duplicates != dict(allowed_identical_duplicates or {}):
        raise ExtractorError("YAML known duplicate declaration count drifted")
    if not isinstance(value, Mapping):
        raise ExtractorError("YAML document must contain an object")
    return _normalize_json(value)


def _canonical_repository_path(raw: Any, *, allow_trailing_slash: bool = False) -> str:
    if not isinstance(raw, str) or not raw or "\\" in raw or ":" in raw or "\x00" in raw:
        raise ExtractorError("source contains a non-canonical path")
    trailing = raw.endswith("/")
    candidate = raw[:-1] if trailing else raw
    path = PurePosixPath(candidate)
    if path.is_absolute() or not path.parts or any(part in {"", ".", ".."} for part in path.parts):
        raise ExtractorError("source contains a non-canonical path")
    normalized = path.as_posix() + ("/" if trailing else "")
    if normalized != raw or (trailing and not allow_trailing_slash):
        raise ExtractorError("source contains a non-canonical path")
    return normalized


def _read_manifest(repo_root: Path) -> tuple[Mapping[str, Any], dict[str, Mapping[str, Any]]]:
    path = repo_root / MANIFEST_RELATIVE
    try:
        data = path.read_bytes()
    except OSError as error:
        raise ExtractorError("accepted A8 manifest is unavailable") from error
    if _sha256(data) != MANIFEST_SHA256:
        raise InventoryMismatch("accepted A8 manifest digest drifted")
    manifest = _load_json_bytes(data)
    source = manifest.get("source")
    if not isinstance(source, Mapping) or source.get("tag") != ACCEPTED_TAG or source.get(
        "peeled_commit"
    ) != ACCEPTED_COMMIT:
        raise InventoryMismatch("accepted A8 source identity drifted")
    package_trees = manifest.get("package_trees")
    if not isinstance(package_trees, list):
        raise ExtractorError("accepted package-tree inventory is unavailable")
    all_files: dict[str, Mapping[str, Any]] = {}
    for expected_tree in (PYTHON_TREE, CONTENT_TREE):
        matches = [
            tree
            for tree in package_trees
            if isinstance(tree, Mapping) and tree.get("root") == expected_tree["root"]
        ]
        if len(matches) != 1:
            raise ExtractorError("accepted package-tree identity is not unique")
        tree = matches[0]
        if any(tree.get(key) != value for key, value in expected_tree.items()):
            raise InventoryMismatch("accepted package-tree identity drifted")
        files = tree.get("files")
        if not isinstance(files, list) or len(files) != expected_tree["file_count"]:
            raise ExtractorError("accepted package-tree file inventory is incomplete")
        for item in files:
            if not isinstance(item, Mapping):
                raise ExtractorError("accepted package-tree entry is invalid")
            item_path = _canonical_repository_path(item.get("path"))
            oid = item.get("git_blob_oid")
            digest = item.get("sha256")
            size = item.get("size_bytes")
            mode = item.get("mode")
            if (
                item_path in all_files
                or not isinstance(oid, str)
                or not HEX_40.fullmatch(oid)
                or not isinstance(digest, str)
                or not HEX_64.fullmatch(digest)
                or not isinstance(size, int)
                or isinstance(size, bool)
                or size < 0
                or mode != "100644"
            ):
                raise ExtractorError("accepted package-tree metadata is invalid")
            all_files[item_path] = {
                "path": item_path,
                "git_blob_oid": oid,
                "sha256": digest,
                "size_bytes": size,
                "mode": mode,
            }
    for binding in SOURCE_BINDINGS:
        expected = {key: binding[key] for key in ("path", "mode", "git_blob_oid", "sha256", "size_bytes")}
        if all_files.get(str(binding["path"])) != expected:
            raise InventoryMismatch("accepted orchestrator source anchor drifted")
    return manifest, all_files


def _git_environment() -> dict[str, str]:
    environment = {
        key: value
        for key, value in os.environ.items()
        if not key.upper().startswith("GIT_")
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


def _verify_tag_and_tree(repo_root: Path) -> None:
    try:
        resolved = subprocess.run(
            ["git", "rev-parse", f"{ACCEPTED_TAG}^{{}}"],
            cwd=repo_root,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
            env=_git_environment(),
        )
        tree = subprocess.run(
            ["git", "ls-tree", "-z", ACCEPTED_COMMIT, "--", *[str(item["path"]) for item in SOURCE_BINDINGS]],
            cwd=repo_root,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
            env=_git_environment(),
        )
    except OSError as error:
        raise ExtractorError("accepted Git tree reader is unavailable") from error
    if resolved.returncode != 0:
        raise ExtractorError("accepted tag is unavailable locally")
    if resolved.stdout.strip() != ACCEPTED_COMMIT.encode("ascii"):
        raise InventoryMismatch("accepted tag does not resolve to the frozen commit")
    if tree.returncode != 0:
        raise ExtractorError("accepted commit tree could not be inspected")
    expected = b"".join(
        (
            f"{item['mode']} blob {item['git_blob_oid']}\t{item['path']}\0".encode("utf-8")
            for item in sorted(SOURCE_BINDINGS, key=lambda value: str(value["path"]))
        )
    )
    if tree.stdout != expected:
        raise InventoryMismatch("accepted path-to-blob bindings drifted")


def _cat_file_blobs(repo_root: Path) -> dict[str, bytes]:
    entries = sorted(SOURCE_BINDINGS, key=lambda value: str(value["path"]))
    request = b"".join(f"{entry['git_blob_oid']}\n".encode("ascii") for entry in entries)
    try:
        completed = subprocess.run(
            ["git", "cat-file", "--batch"],
            cwd=repo_root,
            input=request,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
            env=_git_environment(),
        )
    except OSError as error:
        raise ExtractorError("accepted Git object reader is unavailable") from error
    if completed.returncode != 0:
        raise ExtractorError("accepted Git object reader failed")
    stream = io.BytesIO(completed.stdout)
    blobs: dict[str, bytes] = {}
    for expected in entries:
        header = stream.readline()
        tokens = header[:-1].split() if header.endswith(b"\n") else []
        if len(tokens) == 2 and tokens[1] == b"missing":
            raise ExtractorError("accepted Git blob is unavailable locally")
        if len(tokens) != 3:
            raise ExtractorError("accepted Git object response is invalid")
        try:
            oid = tokens[0].decode("ascii")
            object_type = tokens[1].decode("ascii")
            size = int(tokens[2].decode("ascii"))
        except (UnicodeDecodeError, ValueError) as error:
            raise ExtractorError("accepted Git object response is invalid") from error
        if oid != expected["git_blob_oid"] or object_type != "blob" or size != expected["size_bytes"]:
            raise InventoryMismatch("accepted Git blob identity drifted")
        payload = stream.read(size)
        if len(payload) != size or stream.read(1) != b"\n":
            raise ExtractorError("accepted Git blob payload is truncated")
        if _sha256(payload) != expected["sha256"]:
            raise InventoryMismatch("accepted Git blob digest drifted")
        blobs[str(expected["path"])] = payload
    if stream.read(1):
        raise ExtractorError("accepted Git object response contains trailing data")
    return blobs


def _ast_shape(tree: ast.AST) -> tuple[int, int]:
    count = 0
    maximum = 0
    stack: list[tuple[ast.AST, int]] = [(tree, 1)]
    while stack:
        node, depth = stack.pop()
        count += 1
        maximum = max(maximum, depth)
        if count > MAX_AST_NODES or maximum > MAX_AST_DEPTH:
            raise ExtractorError("accepted Python AST exceeds the safety limit")
        stack.extend((child, depth + 1) for child in ast.iter_child_nodes(node))
    return count, maximum


def _parse_python(data: bytes, *, label: str) -> tuple[str, ast.Module, int, int]:
    try:
        source = data.decode("utf-8")
        tree = ast.parse(source, filename=label, type_comments=False)
    except (UnicodeDecodeError, SyntaxError) as error:
        raise ExtractorError("accepted Python source is invalid") from error
    count, depth = _ast_shape(tree)
    return source, tree, count, depth


def _module_class(tree: ast.Module, name: str) -> ast.ClassDef:
    matches = [node for node in tree.body if isinstance(node, ast.ClassDef) and node.name == name]
    if len(matches) != 1:
        raise ExtractorError("accepted Python class is missing or duplicated")
    return matches[0]


def _class_method(class_node: ast.ClassDef, name: str) -> ast.FunctionDef | ast.AsyncFunctionDef:
    matches = [
        node
        for node in class_node.body
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)) and node.name == name
    ]
    if len(matches) != 1:
        raise ExtractorError("accepted Python method is missing or duplicated")
    return matches[0]


def _module_function(tree: ast.Module, name: str) -> ast.FunctionDef | ast.AsyncFunctionDef:
    matches = [
        node
        for node in tree.body
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)) and node.name == name
    ]
    if len(matches) != 1:
        raise ExtractorError("accepted Python function is missing or duplicated")
    return matches[0]


def _assignment_value(nodes: Sequence[ast.stmt], name: str) -> ast.AST:
    matches: list[ast.AST] = []
    for node in nodes:
        if isinstance(node, ast.Assign):
            if any(isinstance(target, ast.Name) and target.id == name for target in node.targets):
                matches.append(node.value)
        elif isinstance(node, ast.AnnAssign) and isinstance(node.target, ast.Name) and node.target.id == name:
            if node.value is not None:
                matches.append(node.value)
    if len(matches) != 1:
        raise ExtractorError("accepted Python literal is missing or duplicated")
    return matches[0]


def _local_assignment(function: ast.FunctionDef | ast.AsyncFunctionDef, name: str) -> ast.AST:
    matches: list[ast.AST] = []
    for node in ast.walk(function):
        if isinstance(node, ast.Assign) and any(
            isinstance(target, ast.Name) and target.id == name for target in node.targets
        ):
            matches.append(node.value)
        elif isinstance(node, ast.AnnAssign) and isinstance(node.target, ast.Name) and node.target.id == name:
            if node.value is not None:
                matches.append(node.value)
    if len(matches) != 1:
        raise ExtractorError("accepted Python local literal is missing or duplicated")
    return matches[0]


def _literal(node: ast.AST, known: Mapping[str, Any] | None = None) -> Any:
    known = known or {}
    if isinstance(node, ast.Constant):
        if node.value is None or isinstance(node.value, (str, bool, int, float)):
            return _normalize_json(node.value)
        raise ExtractorError("accepted Python constant has an unsupported type")
    if isinstance(node, (ast.List, ast.Tuple, ast.Set)):
        values = [_literal(item, known) for item in node.elts]
        if isinstance(node, ast.Set):
            values = sorted(values, key=lambda item: _canonical_json_bytes(item))
        return values
    if isinstance(node, ast.Dict):
        result: dict[str, Any] = {}
        for key_node, value_node in zip(node.keys, node.values, strict=True):
            if key_node is None:
                raise ExtractorError("accepted Python literal uses dictionary expansion")
            key = _literal(key_node, known)
            if not isinstance(key, str) or key in result:
                raise ExtractorError("accepted Python dictionary keys are invalid")
            result[key] = _literal(value_node, known)
        return result
    if isinstance(node, ast.Name) and node.id in known:
        return _normalize_json(known[node.id])
    raise ExtractorError("accepted Python value is not an allowlisted literal")


def _enum_record(tree: ast.Module, name: str, ordinal: int) -> Mapping[str, Any]:
    class_node = _module_class(tree, name)
    members: list[Mapping[str, Any]] = []
    for node in class_node.body:
        if isinstance(node, ast.Assign) and len(node.targets) == 1 and isinstance(node.targets[0], ast.Name):
            member_name = node.targets[0].id
            if not member_name.startswith("_"):
                value = _literal(node.value)
                if not isinstance(value, str):
                    raise ExtractorError("accepted enum value must be a string")
                members.append(
                    {
                        "name": member_name,
                        "value": value,
                        "declaration_ordinal": len(members),
                    }
                )
    if not members:
        raise ExtractorError("accepted enum has no members")
    return {"name": name, "declaration_ordinal": ordinal, "members": members}


def _parameters(node: ast.FunctionDef | ast.AsyncFunctionDef) -> list[Mapping[str, Any]]:
    positional = [*node.args.posonlyargs, *node.args.args]
    default_offset = len(positional) - len(node.args.defaults)
    records: list[Mapping[str, Any]] = []
    for index, argument in enumerate(positional):
        kind = "positional-only" if index < len(node.args.posonlyargs) else "positional-or-keyword"
        default_node = node.args.defaults[index - default_offset] if index >= default_offset else None
        records.append(
            {
                "name": argument.arg,
                "kind": kind,
                "required": default_node is None,
                "default_json": (
                    None
                    if default_node is None
                    else json.dumps(_literal(default_node), ensure_ascii=False, sort_keys=True, separators=(",", ":"))
                ),
            }
        )
    if node.args.vararg is not None:
        records.append(
            {"name": node.args.vararg.arg, "kind": "var-positional", "required": False, "default_json": None}
        )
    for argument, default_node in zip(node.args.kwonlyargs, node.args.kw_defaults, strict=True):
        records.append(
            {
                "name": argument.arg,
                "kind": "keyword-only",
                "required": default_node is None,
                "default_json": (
                    None
                    if default_node is None
                    else json.dumps(_literal(default_node), ensure_ascii=False, sort_keys=True, separators=(",", ":"))
                ),
            }
        )
    if node.args.kwarg is not None:
        records.append(
            {"name": node.args.kwarg.arg, "kind": "var-keyword", "required": False, "default_json": None}
        )
    return records


def _symbol_record(path: str, qualified_name: str, tree: ast.Module) -> Mapping[str, Any]:
    if "." in qualified_name:
        class_name, method_name = qualified_name.split(".", 1)
        node = _class_method(_module_class(tree, class_name), method_name)
        kind = "method"
    else:
        node = _module_function(tree, qualified_name)
        kind = "function"
    return {
        "source_path": path,
        "qualified_name": qualified_name,
        "kind": kind,
        "line": node.lineno,
        "parameters": [item["name"] for item in _parameters(node)],
    }


def _constant_summary(name: str, value: Any, ordinal: int) -> Mapping[str, Any]:
    normalized = _normalize_json(value)
    if isinstance(normalized, dict):
        value_type = "mapping"
        item_count = len(normalized)
        keys = sorted(normalized)
    elif isinstance(normalized, list):
        value_type = "sequence"
        item_count = len(normalized)
        keys = []
    else:
        value_type = "scalar"
        item_count = 1
        keys = []
    return {
        "name": name,
        "declaration_ordinal": ordinal,
        "value_type": value_type,
        "item_count": item_count,
        "top_level_keys": keys,
        "canonical_sha256": _sha256(_canonical_json_bytes(normalized)),
    }


def _module_surface(blobs: Mapping[str, bytes]) -> Mapping[str, Any]:
    orchestrator_path = str(SOURCE_BINDINGS[0]["path"])
    mcp_path = str(SOURCE_BINDINGS[1]["path"])
    _source, tree, node_count, depth = _parse_python(blobs[orchestrator_path], label=orchestrator_path)
    _mcp_source, mcp_tree, mcp_node_count, mcp_depth = _parse_python(blobs[mcp_path], label=mcp_path)
    model_class = _module_class(tree, "ModelOrchestrator")

    known: dict[str, Any] = {}
    choice_sets: list[Mapping[str, Any]] = []
    for ordinal, name in enumerate(CHOICE_CONSTANTS):
        value = _literal(_assignment_value(tree.body, name), known)
        if not isinstance(value, list) or not value or not all(isinstance(item, str) for item in value):
            raise ExtractorError("accepted choice set is invalid")
        known[name] = value
        choice_sets.append(
            {
                "name": name,
                "declaration_ordinal": ordinal,
                "external_values": value,
                "normalized_values": [item.lower().replace("-", "_") for item in value],
            }
        )

    profile_value = _literal(_assignment_value(model_class.body, "DEFAULT_AGENT_PROFILES"), known)
    if not isinstance(profile_value, Mapping):
        raise ExtractorError("accepted default profile catalog is invalid")
    profiles: list[Mapping[str, Any]] = []
    for ordinal, (profile_id, profile) in enumerate(profile_value.items()):
        if not isinstance(profile, Mapping):
            raise ExtractorError("accepted default profile is invalid")
        runtime_options = profile.get("runtime_options", {})
        if not isinstance(runtime_options, Mapping):
            raise ExtractorError("accepted profile runtime options are invalid")
        profiles.append(
            {
                "profile_id": profile_id,
                "declaration_ordinal": ordinal,
                "keys": sorted(profile),
                "runtime_agents": sorted(runtime_options),
                "canonical_sha256": _sha256(_canonical_json_bytes(profile)),
            }
        )

    summaries: list[Mapping[str, Any]] = []
    for ordinal, name in enumerate(SUMMARY_CONSTANTS):
        value = _literal(_assignment_value(model_class.body, name), known)
        summaries.append(_constant_summary(name, value, ordinal))

    public_methods = []
    for ordinal, name in enumerate(PUBLIC_METHODS):
        node = _class_method(model_class, name)
        public_methods.append(
            {
                "name": name,
                "declaration_ordinal": ordinal,
                "line": node.lineno,
                "parameters": _parameters(node),
            }
        )

    symbols = [
        _symbol_record(path, qualified_name, tree if path == orchestrator_path else mcp_tree)
        for path, names in SOURCE_SYMBOLS.items()
        for qualified_name in names
    ]

    route_signals = _module_function(mcp_tree, "_orchestrator_route_signals")
    strong_terms = _literal(_local_assignment(route_signals, "strong_terms"))
    normalize_platform = _module_function(mcp_tree, "_normalize_platform")
    platform_aliases = _literal(_local_assignment(normalize_platform, "aliases"))
    if not isinstance(strong_terms, Mapping) or not isinstance(platform_aliases, Mapping):
        raise ExtractorError("accepted MCP routing literals are invalid")

    normalize_worker = _class_method(model_class, "_normalize_worker_choice")
    replace_calls = [
        node
        for node in ast.walk(normalize_worker)
        if isinstance(node, ast.Call)
        and isinstance(node.func, ast.Attribute)
        and node.func.attr == "replace"
        and len(node.args) == 2
        and isinstance(node.args[0], ast.Constant)
        and isinstance(node.args[1], ast.Constant)
        and node.args[0].value == "-"
        and node.args[1].value == "_"
    ]
    if len(replace_calls) < 2:
        raise ExtractorError("accepted worker normalization rule is unavailable")

    if len(profiles) != EXPECTED_COUNTS["declared_profile_count"]:
        raise InventoryMismatch("accepted default profile count drifted")
    return {
        "ast_limits": {
            "max_nodes": MAX_AST_NODES,
            "max_depth": MAX_AST_DEPTH,
            "orchestrator_node_count": node_count,
            "orchestrator_depth": depth,
            "mcp_boundary_node_count": mcp_node_count,
            "mcp_boundary_depth": mcp_depth,
        },
        "enums": [
            _enum_record(tree, "CollaborationMode", 0),
            _enum_record(tree, "AcademicTaskType", 1),
        ],
        "choice_sets": choice_sets,
        "default_profiles": profiles,
        "literal_constants": summaries,
        "public_methods": public_methods,
        "source_symbols": symbols,
        "mcp_escalation": {
            "strong_terms": [
                {"term": term, "reason": reason, "declaration_ordinal": ordinal}
                for ordinal, (term, reason) in enumerate(strong_terms.items())
            ],
            "platform_aliases": [
                {"input": source, "normalized": target}
                for source, target in sorted(platform_aliases.items())
            ],
            "valid_platforms": ["codex", "claude_code", "antigravity", "cli", "unknown"],
            "required_canonical_fields": ["task_id", "paper_type", "topic"],
            "orchestrator_route_sequence": [
                "qiongli_orchestrator_doctor",
                "qiongli_task_plan",
                "qiongli_task_run",
            ],
            "skill_route_sequence": ["qiongli_task_plan"],
            "recommendation_rule": "duo-or-triad-execution-mode-or-at-least-two-routing-reasons",
            "run_agents_default": False,
            "run_agents_input_policy": "json-boolean-only",
            "doctor_enforcement": "advisory-sequence-not-enforced-by-task-run",
        },
    }


def _string(value: Any, *, label: str, allow_empty: bool = False) -> str:
    if not isinstance(value, str):
        raise ExtractorError(f"{label} must be a string")
    result = value.strip()
    if not result and not allow_empty:
        raise ExtractorError(f"{label} must not be empty")
    return result


def _string_list(value: Any, *, label: str, allow_empty: bool = True) -> list[str]:
    if not isinstance(value, list) or not all(isinstance(item, str) for item in value):
        raise ExtractorError(f"{label} must be a string list")
    result = [_string(item, label=label) for item in value]
    if not allow_empty and not result:
        raise ExtractorError(f"{label} must not be empty")
    if len(result) != len(set(result)):
        raise ExtractorError(f"{label} contains duplicates")
    return result


def _mapping(value: Any, *, label: str) -> Mapping[str, Any]:
    if not isinstance(value, Mapping):
        raise ExtractorError(f"{label} must be an object")
    return value


def _portable_artifact_path(value: Any, *, label: str) -> str:
    path = _string(value, label=label)
    return _canonical_repository_path(path, allow_trailing_slash=True)


def _functional_owner(
    task_id: str,
    stage_id: str,
    defaults: Mapping[str, Any],
    overrides: Mapping[str, Any],
    agents: Mapping[str, Any],
) -> Mapping[str, Any]:
    default_owner = _string(defaults.get(stage_id), label="functional stage default")
    owner = _string(overrides.get(task_id, default_owner), label="functional owner")
    if owner not in agents:
        raise ExtractorError("functional task routing references an unknown owner")
    block = _mapping(agents[owner], label="functional agent")
    raw_role_file = _string(block.get("file"), label="functional role file")
    role_file = _canonical_repository_path(f"content/{raw_role_file}")
    return {
        "owner": owner,
        "source": "task-override" if task_id in overrides else "stage-default",
        "stage_default_owner": default_owner,
        "role_id": _string(block.get("mapped_role", owner), label="functional role id"),
        "role_file": role_file,
    }


def _validate_required_dag(tasks: Sequence[Mapping[str, Any]]) -> None:
    dependencies = {
        str(task["task_id"]): list(task["dependencies"]["prerequisites_all"])
        for task in tasks
    }
    visited: set[str] = set()
    visiting: set[str] = set()

    def visit(task_id: str) -> None:
        if task_id in visited:
            return
        if task_id in visiting:
            raise InventoryMismatch("accepted prerequisites_all graph contains a cycle")
        visiting.add(task_id)
        for dependency in dependencies[task_id]:
            visit(dependency)
        visiting.remove(task_id)
        visited.add(task_id)

    for task_id in dependencies:
        visit(task_id)


def _team_run_records(value: Any, runtime_agents: set[str]) -> list[Mapping[str, Any]]:
    config = _mapping(value, label="team-run config")
    if set(config) != {"B1", "H3"}:
        raise InventoryMismatch("accepted team-run task set drifted")
    records: list[Mapping[str, Any]] = []
    for ordinal, (task_id, raw) in enumerate(config.items()):
        block = _mapping(raw, label="team-run block")
        worker_pool = _string_list(block.get("worker_pool"), label="team-run worker pool", allow_empty=False)
        review_pool = _string_list(block.get("review_pool"), label="team-run review pool", allow_empty=False)
        planner = _string(block.get("planner_agent"), label="team-run planner")
        merge_agent = _string(block.get("merge_agent"), label="team-run merge agent")
        if any(agent not in runtime_agents for agent in [planner, merge_agent, *worker_pool, *review_pool]):
            raise ExtractorError("team-run config references an unknown runtime agent")
        barrier = _mapping(block.get("barrier_rules"), label="team-run barrier")
        ratio = barrier.get("min_success_ratio")
        if not isinstance(ratio, (int, float)) or isinstance(ratio, bool) or not 0 <= ratio <= 1:
            raise ExtractorError("team-run success ratio is invalid")
        personas_raw = block.get("personas", [])
        if not isinstance(personas_raw, list):
            raise ExtractorError("team-run personas must be a list")
        personas = []
        for persona in personas_raw:
            item = _mapping(persona, label="team-run persona")
            personas.append(
                {
                    "id": _string(item.get("id"), label="persona id"),
                    "focus": _string(item.get("focus"), label="persona focus"),
                }
            )
        records.append(
            {
                "task_id": task_id,
                "declaration_ordinal": ordinal,
                "execution_mode": _string(block.get("execution_mode"), label="team-run execution mode"),
                "partition_strategy": _string(block.get("partition_strategy"), label="team-run partition strategy"),
                "max_parallel_units": block.get("max_parallel_units"),
                "planner_agent": planner,
                "worker_pool": worker_pool,
                "merge_agent": merge_agent,
                "review_pool": review_pool,
                "consensus_policy": _string(block.get("consensus_policy"), label="team-run consensus policy"),
                "barrier": {
                    "min_success_ratio": float(ratio),
                    "on_failure": _string(barrier.get("on_failure"), label="team-run failure policy"),
                },
                "shard_outputs": [
                    _portable_artifact_path(item, label="team-run shard output")
                    for item in _string_list(block.get("shard_outputs"), label="team-run shard outputs")
                ],
                "canonical_outputs": [
                    _portable_artifact_path(item, label="team-run canonical output")
                    for item in _string_list(block.get("canonical_outputs"), label="team-run canonical outputs")
                ],
                "personas": personas,
            }
        )
    return records


def _worker_config_records(value: Any, runtime_agents: set[str]) -> list[Mapping[str, Any]]:
    config = _mapping(value, label="worker orchestration config")
    if set(config) != {"B1", "H3"}:
        raise InventoryMismatch("accepted worker-orchestration task set drifted")
    records: list[Mapping[str, Any]] = []
    for ordinal, (task_id, raw) in enumerate(config.items()):
        block = _mapping(raw, label="worker orchestration block")
        preference = _mapping(block.get("adapter_preference"), label="worker adapter preference")
        if set(preference) != runtime_agents:
            raise ExtractorError("worker adapter preference does not cover runtime agents")
        barrier = _mapping(block.get("barrier_rules"), label="worker barrier")
        ratio = barrier.get("min_success_ratio")
        max_workers = block.get("max_workers")
        if (
            not isinstance(max_workers, int)
            or isinstance(max_workers, bool)
            or max_workers <= 0
            or not isinstance(ratio, (int, float))
            or isinstance(ratio, bool)
            or not 0 <= ratio <= 1
        ):
            raise ExtractorError("worker orchestration numeric policy is invalid")
        records.append(
            {
                "task_id": task_id,
                "declaration_ordinal": ordinal,
                "default_mode": _string(block.get("default_mode"), label="worker default mode"),
                "adapter_preference": [
                    {"runtime_agent": agent, "adapter": _string(preference[agent], label="worker adapter")}
                    for agent in sorted(runtime_agents)
                ],
                "partition_strategy": _string(block.get("partition_strategy"), label="worker partition strategy"),
                "max_workers": max_workers,
                "worker_pool": _string_list(block.get("worker_pool"), label="worker pool", allow_empty=False),
                "merge_policy": _string(block.get("merge_policy"), label="worker merge policy"),
                "barrier": {
                    "min_success_ratio": float(ratio),
                    "on_failure": _string(barrier.get("on_failure"), label="worker failure policy"),
                },
            }
        )
    return records


def _workflow_and_routing(blobs: Mapping[str, bytes]) -> tuple[Mapping[str, Any], Mapping[str, Any], Mapping[str, int]]:
    workflow = _load_yaml_bytes(blobs[str(SOURCE_BINDINGS[2]["path"])])
    capability = _load_yaml_bytes(
        blobs[str(SOURCE_BINDINGS[3]["path"])],
        allowed_identical_duplicates={"academic-context-maintainer": 1},
    )
    task_catalog = _mapping(workflow.get("task_catalog"), label="task catalog")
    dependency_catalog = _mapping(workflow.get("dependency_catalog"), label="dependency catalog")
    task_skill_mapping = _mapping(capability.get("task_skill_mapping"), label="task skill mapping")
    task_execution = _mapping(capability.get("task_execution"), label="task execution mapping")
    task_ids = set(task_catalog)
    if (
        len(task_ids) != EXPECTED_COUNTS["task_count"]
        or task_ids != set(dependency_catalog)
        or task_ids != set(task_skill_mapping)
        or task_ids != set(task_execution)
        or not all(isinstance(task_id, str) and TASK_ID.fullmatch(task_id) for task_id in task_ids)
    ):
        raise InventoryMismatch("accepted task-key closure drifted")

    stages_raw = workflow.get("stages")
    if not isinstance(stages_raw, list):
        raise ExtractorError("workflow stages must be a list")
    stages: list[Mapping[str, Any]] = []
    stage_ids: set[str] = set()
    for ordinal, raw in enumerate(stages_raw):
        block = _mapping(raw, label="workflow stage")
        stage_id = _string(block.get("id"), label="stage id")
        sequence_index = block.get("sequence_index")
        if stage_id in stage_ids or not isinstance(sequence_index, int) or isinstance(sequence_index, bool):
            raise ExtractorError("workflow stage identity is invalid")
        stage_ids.add(stage_id)
        stages.append(
            {
                "stage_id": stage_id,
                "declaration_ordinal": ordinal,
                "sequence_index": sequence_index,
                "name": _string(block.get("name"), label="stage name"),
                "phase_type": _string(block.get("phase_type"), label="stage phase type"),
                "outputs": [
                    _portable_artifact_path(item, label="stage output")
                    for item in _string_list(block.get("outputs"), label="stage outputs")
                ],
            }
        )
    if len(stages) != EXPECTED_COUNTS["stage_count"] or len({item["sequence_index"] for item in stages}) != len(stages):
        raise InventoryMismatch("accepted workflow stage inventory drifted")

    runtime_agents = set(_string_list(capability.get("agent_registry"), label="runtime agent registry", allow_empty=False))
    mcp_capabilities = _string_list(capability.get("mcp_registry"), label="MCP registry", allow_empty=False)
    mcp_set = set(mcp_capabilities)
    skill_registry = _string_list(capability.get("skill_registry"), label="skill registry", allow_empty=False)
    skill_set = set(skill_registry)
    skill_catalog = _mapping(capability.get("skill_catalog"), label="skill catalog")
    if set(skill_catalog) != skill_set:
        raise InventoryMismatch("accepted skill registry and catalog differ")

    functional_registry = _string_list(
        capability.get("functional_agent_registry"),
        label="functional agent registry",
        allow_empty=False,
    )
    functional_agents_raw = _mapping(capability.get("functional_agents"), label="functional agent catalog")
    if set(functional_registry) != set(functional_agents_raw):
        raise InventoryMismatch("accepted functional agent registry and catalog differ")
    functional_routing = _mapping(capability.get("task_functional_routing"), label="functional routing")
    defaults = _mapping(functional_routing.get("defaults_by_stage"), label="functional stage defaults")
    overrides = _mapping(functional_routing.get("overrides"), label="functional task overrides")
    if set(defaults) != stage_ids or not set(overrides).issubset(task_ids):
        raise InventoryMismatch("accepted functional routing closure drifted")

    quality_raw = workflow.get("quality_gates")
    if not isinstance(quality_raw, list):
        raise ExtractorError("quality gate catalog must be a list")
    quality_gates: list[Mapping[str, Any]] = []
    for ordinal, raw in enumerate(quality_raw):
        block = _mapping(raw, label="quality gate")
        quality_gates.append(
            {
                "gate_id": _string(block.get("id"), label="quality gate id"),
                "declaration_ordinal": ordinal,
                "name": _string(block.get("name"), label="quality gate name"),
                "rule": _string(block.get("rule"), label="quality gate rule"),
                "contract_ref": _canonical_repository_path(
                    _string(block.get("contract_ref"), label="quality gate contract ref").split("#", 1)[0]
                )
                + "#"
                + _string(block.get("contract_ref"), label="quality gate contract ref").split("#", 1)[1],
            }
        )
    quality_ids = {item["gate_id"] for item in quality_gates}
    if len(quality_ids) != len(quality_gates):
        raise ExtractorError("quality gate IDs are duplicated")

    tasks: list[Mapping[str, Any]] = []
    all_edges = any_edges = recommended_edges = next_edges = 0
    output_assignments = skill_assignments = mcp_assignments = gate_assignments = 0
    unique_outputs: set[str] = set()
    for ordinal, (task_id, raw_task) in enumerate(task_catalog.items()):
        catalog = _mapping(raw_task, label="task catalog row")
        stage_id = _string(catalog.get("stage"), label="task stage")
        if stage_id not in stage_ids:
            raise ExtractorError("task references an unknown stage")
        outputs = [
            _portable_artifact_path(item, label="task output")
            for item in _string_list(catalog.get("outputs"), label="task outputs", allow_empty=False)
        ]
        dependencies_raw = _mapping(dependency_catalog[task_id], label="task dependencies")
        dependencies = {
            key: _string_list(dependencies_raw.get(key, []), label=f"task {key}")
            for key in (
                "prerequisites_all",
                "prerequisites_any",
                "recommended_prerequisites",
                "recommended_next",
            )
        }
        if any(reference not in task_ids for values in dependencies.values() for reference in values):
            raise ExtractorError("task dependency references an unknown task")
        skill_row = _mapping(task_skill_mapping[task_id], label="task skill row")
        skills = _string_list(skill_row.get("required_skills"), label="task required skills", allow_empty=False)
        if not set(skills).issubset(skill_set):
            raise ExtractorError("task references an unknown routing skill")
        execution = _mapping(task_execution[task_id], label="task execution row")
        required_mcp = _string_list(execution.get("required_mcp"), label="task required MCP", allow_empty=False)
        gates = _string_list(execution.get("quality_gates"), label="task quality gates", allow_empty=False)
        primary = _string(execution.get("primary_agent"), label="task primary agent")
        reviewer = _string(execution.get("review_agent"), label="task review agent")
        fallback = _string(execution.get("fallback_agent"), label="task fallback agent")
        if not set(required_mcp).issubset(mcp_set) or not set(gates).issubset(quality_ids):
            raise ExtractorError("task execution row contains an unknown reference")
        if any(agent not in runtime_agents for agent in (primary, reviewer, fallback)):
            raise ExtractorError("task execution row references an unknown runtime agent")
        tasks.append(
            {
                "task_id": task_id,
                "declaration_ordinal": ordinal,
                "stage_id": stage_id,
                "title": _string(catalog.get("title"), label="task title"),
                "purpose": (
                    _string(catalog.get("purpose"), label="task purpose")
                    if catalog.get("purpose") is not None
                    else None
                ),
                "outputs": outputs,
                "dependencies": dependencies,
                "required_skills": skills,
                "required_mcp": required_mcp,
                "quality_gates": gates,
                "runtime_plan": {
                    "primary_agent": primary,
                    "review_agent": reviewer,
                    "fallback_agent": fallback,
                },
                "functional_plan": _functional_owner(
                    task_id, stage_id, defaults, overrides, functional_agents_raw
                ),
            }
        )
        all_edges += len(dependencies["prerequisites_all"])
        any_edges += len(dependencies["prerequisites_any"])
        recommended_edges += len(dependencies["recommended_prerequisites"])
        next_edges += len(dependencies["recommended_next"])
        output_assignments += len(outputs)
        unique_outputs.update(outputs)
        skill_assignments += len(skills)
        mcp_assignments += len(required_mcp)
        gate_assignments += len(gates)
    _validate_required_dag(tasks)

    skills: list[Mapping[str, Any]] = []
    for ordinal, skill_id in enumerate(skill_registry):
        block = _mapping(skill_catalog[skill_id], label="skill catalog row")
        skills.append(
            {
                "skill_id": skill_id,
                "declaration_ordinal": ordinal,
                "file": _canonical_repository_path(f"content/{_string(block.get('file'), label='skill file')}"),
                "category": _string(block.get("category"), label="skill category"),
                "default_outputs": [
                    _portable_artifact_path(item, label="skill default output")
                    for item in _string_list(block.get("default_outputs", []), label="skill default outputs")
                ],
                "focus_sha256": _sha256(_string(block.get("focus"), label="skill focus").encode("utf-8")),
            }
        )

    functional_agents: list[Mapping[str, Any]] = []
    for ordinal, agent_id in enumerate(functional_registry):
        block = _mapping(functional_agents_raw[agent_id], label="functional agent row")
        role_file = _canonical_repository_path(f"content/{_string(block.get('file'), label='functional role file')}")
        functional_agents.append(
            {
                "agent_id": agent_id,
                "declaration_ordinal": ordinal,
                "mapped_role": _string(block.get("mapped_role", agent_id), label="mapped role"),
                "role_file": role_file,
                "owns_stages": _string_list(block.get("owns_stages"), label="owned stages", allow_empty=False),
                "focus_sha256": _sha256(_string(block.get("focus"), label="functional focus").encode("utf-8")),
            }
        )

    derived = {
        "stage_count": len(stages),
        "task_count": len(tasks),
        "prerequisites_all_edge_count": all_edges,
        "prerequisites_any_edge_count": any_edges,
        "required_dependency_edge_count": all_edges + any_edges,
        "recommended_prerequisite_edge_count": recommended_edges,
        "recommended_next_edge_count": next_edges,
        "task_output_assignment_count": output_assignments,
        "unique_task_output_count": len(unique_outputs),
        "runtime_agent_count": len(runtime_agents),
        "functional_agent_count": len(functional_agents),
        "functional_stage_default_count": len(defaults),
        "functional_task_override_count": len(overrides),
        "routing_skill_id_count": len(skills),
        "task_skill_assignment_count": skill_assignments,
        "logical_mcp_capability_count": len(mcp_capabilities),
        "task_mcp_assignment_count": mcp_assignments,
        "quality_gate_count": len(quality_gates),
        "task_quality_gate_assignment_count": gate_assignments,
        "team_run_task_count": len(_mapping(capability.get("team_run_config"), label="team-run config")),
        "worker_orchestration_task_count": len(
            _mapping(capability.get("worker_orchestration_config"), label="worker config")
        ),
    }
    for key, expected in EXPECTED_COUNTS.items():
        if key in derived and derived[key] != expected:
            raise InventoryMismatch(f"accepted orchestrator count drifted: {key}")

    workflow_record = {
        "contract_version": _string(workflow.get("contract_version"), label="workflow version"),
        "artifact_root": _portable_artifact_path(
            _mapping(workflow.get("artifacts"), label="workflow artifacts").get("root"),
            label="artifact root",
        ),
        "stages": stages,
        "tasks": tasks,
    }
    routing_record = {
        "map_version": _string(capability.get("map_version"), label="capability map version"),
        "known_source_anomalies": [
            {
                "kind": "identical-duplicate-yaml-key",
                "mapping_path": "skill_catalog",
                "key": "academic-context-maintainer",
                "duplicate_occurrence_count": 1,
                "production_semantics": "last-declaration-wins-with-identical-value",
            }
        ],
        "runtime_agents": [
            {"agent_id": agent, "declaration_ordinal": ordinal}
            for ordinal, agent in enumerate(_string_list(capability.get("agent_registry"), label="runtime agents"))
        ],
        "logical_mcp_capabilities": [
            {"capability_id": item, "declaration_ordinal": ordinal}
            for ordinal, item in enumerate(mcp_capabilities)
        ],
        "functional_agents": functional_agents,
        "functional_stage_defaults": [
            {"stage_id": stage_id, "agent_id": _string(owner, label="functional default"), "declaration_ordinal": ordinal}
            for ordinal, (stage_id, owner) in enumerate(defaults.items())
        ],
        "functional_task_overrides": [
            {"task_id": task_id, "agent_id": _string(owner, label="functional override"), "declaration_ordinal": ordinal}
            for ordinal, (task_id, owner) in enumerate(overrides.items())
        ],
        "skills": skills,
        "quality_gates": quality_gates,
        "team_runs": _team_run_records(capability.get("team_run_config"), runtime_agents),
        "worker_configs": _worker_config_records(
            capability.get("worker_orchestration_config"), runtime_agents
        ),
    }
    return workflow_record, routing_record, derived


def _oracle(repo_root: Path) -> Mapping[str, Any]:
    path = repo_root / PYTHON_ORACLE_RELATIVE
    try:
        data = path.read_bytes()
    except OSError as error:
        raise ExtractorError("accepted Python Full oracle is unavailable") from error
    if _sha256(data) != PYTHON_ORACLE_SHA256:
        raise InventoryMismatch("accepted Python Full oracle digest drifted")
    document = _load_json_bytes(data)
    cases = document.get("cases")
    matches = [
        case
        for case in cases
        if isinstance(case, Mapping) and case.get("id") == "python.orchestration-preview"
    ] if isinstance(cases, list) else []
    if len(matches) != 1:
        raise InventoryMismatch("accepted orchestration oracle case is not unique")
    case = matches[0]
    invocation = _mapping(case.get("invocation"), label="orchestration oracle invocation")
    arguments = _mapping(invocation.get("arguments"), label="orchestration oracle arguments")
    outcome = _mapping(case.get("outcome"), label="orchestration oracle outcome")
    value = _mapping(outcome.get("value"), label="orchestration oracle value")
    controller = _mapping(value.get("controller_metadata"), label="oracle controller metadata")
    task = _mapping(value.get("task"), label="oracle task")
    side_effects = _mapping(case.get("side_effects"), label="orchestration oracle side effects")
    delta = _mapping(side_effects.get("filesystem_delta"), label="oracle filesystem delta")
    source_paths = _string_list(case.get("source_paths"), label="oracle source paths", allow_empty=False)
    expected_source_paths = [str(item["path"]) for item in SOURCE_BINDINGS]
    if source_paths != expected_source_paths:
        raise InventoryMismatch("accepted orchestration oracle source paths drifted")
    exact = {
        "operation": "tools/call qiongli_task_run",
        "transport": "jsonrpc-stdio",
        "task_id": "F3",
        "paper_type": "empirical",
        "topic": "runtime-baseline",
        "guidance_mode": "off",
        "run_agents": False,
        "status": "success",
        "exit_code": 0,
        "mode": "task-run-preview",
        "will_launch_agents": False,
        "controller": "codex",
        "execution_mode": "duo",
        "primary_agent": "",
        "review_agent": "",
        "verifier_agent": "",
        "solo_role_gates": "standard",
        "effective_domain": "auto",
        "task_description": "F3 empirical runtime-baseline",
    }
    actual = {
        "operation": invocation.get("operation"),
        "transport": invocation.get("transport"),
        "task_id": arguments.get("task_id"),
        "paper_type": arguments.get("paper_type"),
        "topic": arguments.get("topic"),
        "guidance_mode": arguments.get("guidance_mode"),
        "run_agents": arguments.get("run_agents"),
        "status": outcome.get("status"),
        "exit_code": outcome.get("exit_code"),
        "mode": value.get("mode"),
        "will_launch_agents": value.get("will_launch_agents"),
        "controller": controller.get("controller"),
        "execution_mode": controller.get("execution_mode"),
        "primary_agent": controller.get("primary_agent"),
        "review_agent": controller.get("review_agent"),
        "verifier_agent": controller.get("verifier_agent"),
        "solo_role_gates": controller.get("solo_role_gates"),
        "effective_domain": value.get("effective_domain"),
        "task_description": value.get("task_description"),
    }
    if actual != exact or task != {
        "task_id": "F3",
        "paper_type": "empirical",
        "topic": "runtime-baseline",
    }:
        raise InventoryMismatch("accepted orchestration oracle outcome drifted")
    if (
        outcome.get("error") is not None
        or value.get("run_agents") is not False
        or value.get("stderr_lines") != []
        or side_effects.get("class") != "none"
        or side_effects.get("writes_outside_sandbox") is not False
        or delta.get("before_tree_sha256") != delta.get("after_tree_sha256")
        or any(delta.get(key) != [] for key in ("created", "modified", "deleted"))
    ):
        raise InventoryMismatch("accepted orchestration oracle safety boundary drifted")
    return {
        "oracle_id": "python-full",
        "case_id": "python.orchestration-preview",
        "source_paths": source_paths,
        "operation": exact["operation"],
        "transport": exact["transport"],
        "task": {
            "task_id": exact["task_id"],
            "paper_type": exact["paper_type"],
            "topic": exact["topic"],
            "guidance_mode": exact["guidance_mode"],
            "run_agents": exact["run_agents"],
        },
        "outcome": {
            "status": exact["status"],
            "exit_code": exact["exit_code"],
            "mode": exact["mode"],
            "will_launch_agents": exact["will_launch_agents"],
            "controller": exact["controller"],
            "execution_mode": exact["execution_mode"],
            "primary_agent": exact["primary_agent"],
            "review_agent": exact["review_agent"],
            "verifier_agent": exact["verifier_agent"],
            "solo_role_gates": exact["solo_role_gates"],
            "effective_domain": exact["effective_domain"],
            "task_description": exact["task_description"],
            "stderr_lines": [],
        },
        "filesystem_delta": {
            "before_tree_sha256": _string(delta.get("before_tree_sha256"), label="oracle before hash"),
            "after_tree_sha256": _string(delta.get("after_tree_sha256"), label="oracle after hash"),
            "created": [],
            "modified": [],
            "deleted": [],
            "writes_outside_sandbox": False,
        },
    }


def _compatibility(module_surface: Mapping[str, Any], routing: Mapping[str, Any]) -> Mapping[str, Any]:
    choices = {
        item["name"]: item
        for item in module_surface["choice_sets"]
        if isinstance(item, Mapping)
    }
    worker_modes = choices["WORKER_MODE_CHOICES"]
    worker_adapters = choices["WORKER_ADAPTER_CHOICES"]
    mode_pairs = [
        {
            "external": external,
            "normalized": normalized,
            "requires_runtime_resolution": normalized == "auto",
        }
        for external, normalized in zip(
            worker_modes["external_values"], worker_modes["normalized_values"], strict=True
        )
    ]
    adapter_pairs = []
    for external, normalized in zip(
        worker_adapters["external_values"], worker_adapters["normalized_values"], strict=True
    ):
        adapter_pairs.append(
            {
                "external": external,
                "normalized": normalized,
                "dispatch_status": (
                    "direct-generic-prompt"
                    if normalized in {"auto", "generic_prompt"}
                    else "recognized-fallback-to-generic-prompt"
                ),
            }
        )
    collaboration_enum = next(
        item for item in module_surface["enums"] if item["name"] == "CollaborationMode"
    )
    controller_choices = choices["CONTROLLER_EXECUTION_MODE_CHOICES"]
    return {
        "collaboration_modes": [item["value"] for item in collaboration_enum["members"]],
        "controller_modes": list(controller_choices["external_values"]),
        "worker_mode_pairs": mode_pairs,
        "worker_adapter_pairs": adapter_pairs,
        "worker_default_status": "disabled-unless-explicitly-requested",
        "worker_native_dispatch": "recognized-native-names-fall-back-to-generic-prompt",
        "doctor_gate": "route-sequence-advisory-not-enforced-by-task-run",
        "quality_gate_runtime": "task-declared-and-artifact-existence-only-not-semantic-policy-execution",
        "functional_owns_stages_policy": "descriptive-not-runtime-enforced",
        "dependency_dag_policy": "prerequisites-all-only",
        "indirect_content_dependencies": {
            "skill_registry_path": "content/skills/registry.yaml",
            "functional_role_paths": sorted(
                item["role_file"] for item in routing["functional_agents"]
            ),
            "binding": "covered-by-frozen-content-tree-full-closure-deferred-to-CTR-201D",
        },
    }


def extract_orchestrator_inventory(repo_root: Path = REPO_ROOT) -> Mapping[str, Any]:
    _require_toolchain()
    root = Path(repo_root)
    _manifest, _files = _read_manifest(root)
    _verify_tag_and_tree(root)
    blobs = _cat_file_blobs(root)
    module_surface = _module_surface(blobs)
    workflow, routing, derived = _workflow_and_routing(blobs)
    oracle = _oracle(root)
    coverage = {
        **derived,
        "declared_profile_count": len(module_surface["default_profiles"]),
        "source_anchor_count": len(SOURCE_BINDINGS),
        "static_contract": "captured",
        "runtime_behavior_matrix": "incomplete",
        "state_resume_behavior": "incomplete",
        "agent_launch_behavior": "incomplete",
        "solo_duo_triad_runtime_parity": "incomplete",
        "concurrency_timeout_cancellation": "incomplete",
        "failure_replay_behavior": "incomplete",
        "quality_gate_semantic_execution": "incomplete",
        "profile_override_runtime_behavior": "incomplete",
        "plugin_marketplace_behavior": "not-implemented",
        "materialized_content_closure": "incomplete",
        "rust_orchestrator": "not-implemented",
        "ctr_201": "in-progress",
        "fnd_202": "not-implemented",
        "completion_ready": False,
    }
    for key, expected in EXPECTED_COUNTS.items():
        if coverage.get(key) != expected:
            raise InventoryMismatch(f"accepted orchestrator coverage drifted: {key}")
    artifact: dict[str, Any] = {
        "$schema": ARTIFACT_SCHEMA,
        "schema_version": ARTIFACT_SCHEMA_VERSION,
        "record_type": ARTIFACT_RECORD_TYPE,
        "task_id": "CTR-201C",
        "status": "static-contract-captured",
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
            "blob_anchors": [
                {
                    "role": item["role"],
                    "consumption_class": item["consumption_class"],
                    "path": item["path"],
                    "mode": item["mode"],
                    "git_blob_oid": item["git_blob_oid"],
                    "sha256": item["sha256"],
                    "size_bytes": item["size_bytes"],
                }
                for item in SOURCE_BINDINGS
            ],
        },
        "capture_contract": {
            "source_mode": "accepted-tag-git-blobs",
            "python_version": "python3.12",
            "yaml_toolchain": "pyyaml-6.0.3-strict-safe-loader",
            "python_analysis": "stdlib-ast-no-import-no-compile-no-eval",
            "environment_mode": "environment-independent-static-analysis",
            "side_effect_policy": "git-read-only-until-explicit-artifact-write",
            "ordering_policy": "source-order-for-primary-catalogs;canonical-sort-for-map-summaries;explicit-ordinals-where-order-preserved",
            "yaml_policy": "strict-safe-loader-with-one-authenticated-identical-duplicate;reject-other-duplicates-merge-anchor-alias-custom-tag-timestamp-binary-or-nonfinite",
            "compatibility_policy": "preserve-external-and-normalized-spellings",
        },
        "module_surface": module_surface,
        "workflow": workflow,
        "routing": routing,
        "compatibility": _compatibility(module_surface, routing),
        "oracle": oracle,
        "coverage": coverage,
    }
    artifact["integrity"] = {
        "algorithm": "sha256",
        "canonicalization": CANONICALIZATION,
        "payload_sha256": canonical_payload_sha256(artifact),
    }
    return artifact


def _json_type(value: Any) -> str:
    if value is None:
        return "null"
    if isinstance(value, bool):
        return "boolean"
    if isinstance(value, int):
        return "integer"
    if isinstance(value, float):
        return "number"
    if isinstance(value, str):
        return "string"
    if isinstance(value, Mapping):
        return "object"
    if isinstance(value, list):
        return "array"
    raise ExtractorError("artifact contains an unsupported schema value")


def _schema_for_values(values: Sequence[Any]) -> Mapping[str, Any]:
    if not values:
        return {}
    types = {_json_type(value) for value in values}
    if types <= {"integer", "number"}:
        return {"type": "number" if "number" in types else "integer"}
    if len(types) > 1:
        non_null = [value for value in values if value is not None]
        if types - {"null"} and len(types - {"null"}) == 1:
            inferred = dict(_schema_for_values(non_null))
            inferred_type = inferred.get("type")
            if isinstance(inferred_type, str):
                inferred["type"] = [inferred_type, "null"]
                return inferred
        return {"anyOf": [_schema_for_values([value]) for value in values]}
    value_type = next(iter(types))
    if value_type in {"null", "boolean", "integer", "number", "string"}:
        return {"type": value_type}
    if value_type == "array":
        flattened = [item for value in values for item in value]
        schema: dict[str, Any] = {"type": "array"}
        if flattened:
            schema["items"] = _schema_for_values(flattened)
        else:
            schema["maxItems"] = 0
        return schema
    mappings = [value for value in values if isinstance(value, Mapping)]
    key_sets = [set(value) for value in mappings]
    if not key_sets or any(keys != key_sets[0] for keys in key_sets[1:]):
        raise ExtractorError("artifact object arrays do not share a closed shape")
    keys = sorted(key_sets[0])
    return {
        "type": "object",
        "required": keys,
        "properties": {
            key: _schema_for_values([value[key] for value in mappings]) for key in keys
        },
        "additionalProperties": False,
    }


def _set_scalar_consts(schema: Mapping[str, Any], value: Mapping[str, Any]) -> None:
    properties = schema.get("properties")
    if not isinstance(properties, dict):
        raise ExtractorError("generated schema object is invalid")
    for key, item in value.items():
        item_schema = properties.get(key)
        if not isinstance(item_schema, dict):
            raise ExtractorError("generated schema property is invalid")
        if item is None or isinstance(item, (str, bool, int, float)):
            item_schema.clear()
            item_schema["const"] = item


def build_orchestrator_schema(artifact: Mapping[str, Any]) -> Mapping[str, Any]:
    inferred = dict(_schema_for_values([artifact]))
    if inferred.get("type") != "object":
        raise ExtractorError("generated artifact schema is invalid")
    properties = inferred.get("properties")
    if not isinstance(properties, dict):
        raise ExtractorError("generated artifact schema properties are invalid")
    for key in ("$schema", "schema_version", "record_type", "task_id", "status"):
        properties[key] = {"const": artifact[key]}
    capture_schema = properties.get("capture_contract")
    coverage_schema = properties.get("coverage")
    source_schema = properties.get("source")
    integrity_schema = properties.get("integrity")
    if not all(
        isinstance(item, Mapping)
        for item in (capture_schema, coverage_schema, source_schema, integrity_schema)
    ):
        raise ExtractorError("generated child schema sections are invalid")
    _set_scalar_consts(capture_schema, artifact["capture_contract"])
    _set_scalar_consts(coverage_schema, artifact["coverage"])
    source_properties = source_schema.get("properties")
    if not isinstance(source_properties, dict):
        raise ExtractorError("generated source schema is invalid")
    for key in ("accepted_tag", "accepted_commit"):
        source_properties[key] = {"const": artifact["source"][key]}
    integrity_properties = integrity_schema.get("properties")
    if not isinstance(integrity_properties, dict):
        raise ExtractorError("generated integrity schema is invalid")
    integrity_properties["algorithm"] = {"const": "sha256"}
    integrity_properties["canonicalization"] = {"const": CANONICALIZATION}
    integrity_properties["payload_sha256"] = {
        "type": "string",
        "pattern": "^[0-9a-f]{64}$",
    }
    return {
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": SCHEMA_ID,
        **inferred,
    }


def _write_json(path: Path, value: Mapping[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    rendered = json.dumps(value, ensure_ascii=False, sort_keys=True, indent=2, allow_nan=False) + "\n"
    path.write_text(rendered, encoding="utf-8", newline="\n")


def _load_checked(path: Path, *, label: str) -> Mapping[str, Any]:
    try:
        return _load_json_bytes(path.read_bytes())
    except OSError as error:
        raise InventoryMismatch(f"checked {label} is unavailable") from error


def _output_paths_may_alias(left: Path, right: Path) -> bool:
    try:
        left_identity = left.resolve(strict=False)
        right_identity = right.resolve(strict=False)
    except (OSError, RuntimeError):
        left_identity = left.absolute()
        right_identity = right.absolute()
    if str(left_identity).casefold() == str(right_identity).casefold():
        return True
    try:
        return left.samefile(right)
    except (OSError, RuntimeError):
        return False


def _build_parser() -> argparse.ArgumentParser:
    parser = _RedactedArgumentParser(
        description="Extract the accepted CTR-201C declared orchestrator inventory."
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
    json_mode = "--json" in arguments
    try:
        args = _build_parser().parse_args(arguments)
        json_mode = bool(args.json)
        if args.check and args.schema_output:
            raise UsageError("invalid command usage")
        if not args.check and not args.output:
            raise UsageError("invalid command usage")
        artifact = extract_orchestrator_inventory(Path(args.root))
        schema = build_orchestrator_schema(artifact)
        if args.check:
            output_path = Path(args.output) if args.output else Path(args.root) / DEFAULT_OUTPUT_RELATIVE
            schema_path = output_path.parent / Path(DEFAULT_SCHEMA_RELATIVE).name
            checked_artifact = _load_checked(output_path, label="orchestrator artifact")
            checked_schema = _load_checked(schema_path, label="orchestrator schema")
            if _canonical_json_bytes(checked_artifact) != _canonical_json_bytes(artifact):
                raise InventoryMismatch("checked orchestrator artifact differs")
            if _canonical_json_bytes(checked_schema) != _canonical_json_bytes(schema):
                raise InventoryMismatch("checked orchestrator schema differs")
            _emit_result(
                json_mode=json_mode,
                status="pass",
                exit_code=0,
                code="accepted-orchestrator-inventory-matches",
                artifact=artifact,
                schema=schema,
            )
            return 0
        output_path = Path(args.output)
        schema_path = (
            Path(args.schema_output)
            if args.schema_output
            else output_path.parent / Path(DEFAULT_SCHEMA_RELATIVE).name
        )
        if _output_paths_may_alias(output_path, schema_path):
            raise UsageError("artifact and schema outputs must differ")
        _write_json(output_path, artifact)
        _write_json(schema_path, schema)
        _emit_result(
            json_mode=json_mode,
            status="pass",
            exit_code=0,
            code="accepted-orchestrator-inventory-written",
            artifact=artifact,
            schema=schema,
        )
        return 0
    except InventoryMismatch:
        _emit_result(
            json_mode=json_mode,
            status="fail",
            exit_code=1,
            code="accepted-orchestrator-inventory-mismatch",
        )
        return 1
    except (ExtractorError, UsageError, OSError, ValueError):
        _emit_result(
            json_mode=json_mode,
            status="error",
            exit_code=2,
            code="accepted-orchestrator-inventory-unavailable",
        )
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
