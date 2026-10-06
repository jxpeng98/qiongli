"""Offline, whole-answer review of three captured installed-Plugin journeys.

Capture stays with the isolated Host/native transaction owners. This adapter
never launches a model, installs a Plugin or grants project-write approval.
"""
from __future__ import annotations

import argparse
from contextlib import redirect_stdout
import csv
import io
import json
import math
from pathlib import Path
import re
import sys

import yaml

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT))
from evals.research_journey import observe  # noqa: E402
from evals.runner.run_suite import run_evals  # noqa: E402
from evals.skill_routing import probe  # noqa: E402

KIND = "qiongli-plugin-baseline/v1"
CASES = ("paper-explanation", "source-paragraph", "saved-continuation")
HASH = re.compile(r"[a-f0-9]{64}")


class IncompleteObservation(ValueError):
    """Keep valid observed calls even when the Host/driver did not finish."""

    def __init__(self, metrics):
        super().__init__("Incomplete Host observation")
        self.metrics = metrics


def value_digest(value):
    return probe.sha(json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")))


def safe_name(name):
    if (not isinstance(name, str) or not name or Path(name).is_absolute()
            or "\\" in name or any(p in ("", ".", "..") for p in name.split("/"))):
        raise ValueError("Invalid capture-relative path")
    return name


def bound_read(root, manifest, name):
    safe_name(name)
    expected = manifest["files"].get(name)
    value = observe.read_file(root, name)
    if not isinstance(expected, str) or not HASH.fullmatch(expected) or probe.sha(value) != expected:
        raise ValueError("Missing or changed capture binding")
    return value


def load_capture(root, manifest=None):
    if manifest is None:
        manifest = json.loads(observe.read_file(root, "manifest.json"))
    observe.fields(manifest, {"kind", "cases", "files"})
    if manifest["kind"] != KIND or manifest["cases"] != list(CASES) or not isinstance(manifest["files"], dict):
        raise ValueError("Changed baseline kind or fixed case selection")
    for name in manifest["files"]:
        bound_read(root, manifest, name)
    frozen = None
    if "frozen-inputs.json" in manifest["files"]:
        frozen = json.loads(bound_read(root, manifest, "frozen-inputs.json"))
        observe.fields(frozen, {"kind", "files"})
        if frozen["kind"] != "qiongli-plugin-inputs/v1" or not isinstance(frozen["files"], dict):
            raise ValueError("Invalid input freeze")
        for name, digest in frozen["files"].items():
            if manifest["files"].get(name) != digest:
                raise ValueError("Inputs changed after freeze")
        required = {"catalog.json", "installation.json", "sources.csv", *(c + "/prompt.txt" for c in CASES)}
        if not required <= frozen["files"].keys():
            raise ValueError("Incomplete input freeze")
    catalog = json.loads(bound_read(root, manifest, "catalog.json"))
    observe.fields(catalog, {"kind", "cases"})
    observe.fields(catalog["cases"], CASES)
    if catalog["kind"] != KIND:
        raise ValueError("Invalid baseline catalog")
    for spec in catalog["cases"].values():
        if not isinstance(spec, dict):
            raise ValueError("Invalid baseline case")
        optional = {"guidance_paths", "length"} & spec.keys()
        observe.fields(spec, {"description", "required_claims", "checks", "required_tools"} | optional)
        if not observe.text(spec["description"]):
            raise ValueError("Missing task description")
        for key in ("required_claims", "checks", "required_tools"):
            values = spec[key]
            if (not isinstance(values, list) or any(not observe.text(v) for v in values)
                    or len(values) != len(set(values)) or (key != "required_tools" and not values)):
                raise ValueError("Invalid fixed baseline expectations")
        if "guidance_paths" in spec:
            paths = spec["guidance_paths"]
            if (not isinstance(paths, list) or not paths or any(not isinstance(p, str) for p in paths)
                    or len(paths) != len(set(paths))):
                raise ValueError("Invalid required guidance paths")
            for path in paths:
                safe_name(path)
        if "length" in spec:
            limit = spec["length"]
            observe.fields(limit, {"unit", "min", "max", "exclude"})
            if (limit["unit"] != "non_whitespace_codepoints"
                    or limit["exclude"] != "bracketed_source_anchors"
                    or type(limit["min"]) is not int or type(limit["max"]) is not int
                    or not 0 < limit["min"] <= limit["max"]):
                raise ValueError("Invalid frozen output length contract")
    identity = json.loads(bound_read(root, manifest, "installation.json"))
    if not isinstance(identity, dict) or not re.fullmatch(r"[a-f0-9]{40}", identity.get("source_commit", "")):
        raise ValueError("Missing exact candidate identity")
    for key in ("cli_sha256", "content_pack_sha256", "plugin_receipt_sha256"):
        if not isinstance(identity.get(key), str) or not HASH.fullmatch(identity[key]):
            raise ValueError("Missing installed byte identity")
    for key in ("codex_version", "model", "reasoning_effort"):
        if not observe.text(identity.get(key)):
            raise ValueError("Missing Host invocation identity")
    if "mcp_server" in identity and not observe.text(identity["mcp_server"]):
        raise ValueError("Invalid installed MCP server identity")
    if "guidance_files" in identity:
        if not isinstance(identity["guidance_files"], dict):
            raise ValueError("Invalid installed guidance identity")
        for name, digest in identity["guidance_files"].items():
            safe_name(name)
            if not isinstance(digest, str) or not HASH.fullmatch(digest):
                raise ValueError("Invalid installed guidance digest")
    source_text = bound_read(root, manifest, "sources.csv")
    reader = csv.DictReader(io.StringIO(source_text))
    sources = list(reader)
    if (len(reader.fieldnames or ()) != len(observe.SOURCE_FIELDS)
            or set(reader.fieldnames or ()) != set(observe.SOURCE_FIELDS) or not sources):
        raise ValueError("Missing source registry")
    locators = set()
    for source in sources:
        observe.fields(source, observe.SOURCE_FIELDS)
        if any(not observe.text(source.get(k)) for k in observe.SOURCE_FIELDS):
            raise ValueError("Incomplete source registry")
        if source["source_location"] in locators:
            raise ValueError("Duplicate source locator")
        locators.add(source["source_location"])
        body = bound_read(root, manifest, source["artifact_path"])
        if frozen is not None and source["artifact_path"] not in frozen["files"]:
            raise ValueError("Source was not frozen before observation")
        if source["source_location"] not in body:
            raise ValueError("Source locator absent from supplied packet")
    return manifest, catalog["cases"], identity, source_text


def snapshot_files(root):
    files = {}
    for path in sorted(root.rglob("*")):
        if path.is_symlink():
            raise ValueError("Capture symlinks are unsupported")
        if path.is_file():
            name = safe_name(path.relative_to(root).as_posix())
            files[name] = probe.sha(observe.read_file(root, name))
    return files


def exclusive_json(path, value):
    with path.open("x", encoding="utf-8") as stream:
        json.dump(value, stream, ensure_ascii=False, indent=2)
        stream.write("\n")


def freeze_inputs(root):
    files = snapshot_files(root)
    required = {"catalog.json", "installation.json", "sources.csv", *(c + "/prompt.txt" for c in CASES)}
    forbidden = {"manifest.json", "frozen-inputs.json", *(c + "/" + name for c in CASES
                  for name in ("events.jsonl", "capture.json", "answer.md", "guidance.json", "preservation.json"))}
    if not required <= files.keys() or forbidden & files.keys():
        raise ValueError("Freeze prepared inputs once, before any observations")
    for source in csv.DictReader(io.StringIO(observe.read_file(root, "sources.csv"))):
        if source.get("artifact_path") not in files:
            raise ValueError("Source absent from frozen inputs")
    exclusive_json(root / "frozen-inputs.json", {"kind": "qiongli-plugin-inputs/v1", "files": files})


def seal_capture(root):
    files = snapshot_files(root)
    if "manifest.json" in files or "frozen-inputs.json" not in files:
        raise ValueError("Seal a new, previously frozen capture only")
    manifest = {"kind": KIND, "cases": list(CASES), "files": files}
    load_capture(root, manifest)
    exclusive_json(root / "manifest.json", manifest)


def result_payload(result):
    """Require visible payload, not just an empty transport envelope."""
    structured = any(isinstance(result.get(k), dict) and bool(result[k])
                     for k in ("structuredContent", "structured_content"))
    blocks = result.get("content")
    if not isinstance(blocks, list) or not blocks:
        return structured
    for block in blocks:
        if not isinstance(block, dict):
            return False
        kind = block.get("type")
        if kind == "text":
            if not observe.text(block.get("text")):
                return False
        elif kind in ("image", "audio"):
            if not observe.text(block.get("data")) or not observe.text(block.get("mimeType")):
                return False
        elif kind == "resource":
            if not isinstance(block.get("resource"), dict) or not block["resource"]:
                return False
        elif kind == "resource_link":
            if not observe.text(block.get("uri")):
                return False
        else:
            return False
    return True


def inspect_trace(raw, exit_code, termination_reason=None):
    """Observe calls separately from completion; raw events remain the evidence."""
    if type(exit_code) is not int:
        raise ValueError("Missing process exit code")
    if termination_reason not in (None, "timeout", "permission-denied", "cancelled"):
        raise ValueError("Unknown driver termination reason")
    lines = [line for line in raw.splitlines() if line.strip()]
    events, truncated = [], False
    for index, line in enumerate(lines):
        try:
            events.append(json.loads(line))
        except ValueError:
            if index != len(lines) - 1:
                raise
            truncated = True
    remaining, pending, done, calls = [], {}, set(), []
    last_tool, last_answer = -1, -1
    for index, event in enumerate(events):
        if not isinstance(event, dict):
            raise ValueError("Malformed trace event")
        kind, item = event.get("type"), event.get("item", {})
        if not isinstance(item, dict):
            raise ValueError("Malformed trace item")
        tool_kind = item.get("type")
        if tool_kind not in ("mcp_tool_call", "command_execution"):
            remaining.append(event)
            if kind == "item.completed" and tool_kind == "agent_message":
                last_answer = index
            continue
        if index < 2:
            raise ValueError("Tool outside a turn")
        call_id = item.get("id")
        if not observe.text(call_id):
            raise ValueError("Missing call ID")
        keys = ("type", "server", "tool", "arguments") if tool_kind == "mcp_tool_call" else ("type", "command")
        identity = {k: item.get(k) for k in keys}
        if tool_kind == "mcp_tool_call":
            if (not observe.text(item.get("server")) or not observe.text(item.get("tool"))
                    or not isinstance(item.get("arguments"), dict)):
                raise ValueError("Invalid MCP identity")
        elif not observe.text(item.get("command")):
            raise ValueError("Invalid command identity")
        if kind == "item.started":
            if call_id in pending or call_id in done or item.get("status") != "in_progress":
                raise ValueError("Repeated or invalid tool start")
            if tool_kind == "mcp_tool_call" and (item.get("error") is not None or item.get("result") is not None):
                raise ValueError("Invalid MCP start result")
            pending[call_id] = (identity, index)
        elif kind == "item.completed":
            start = pending.pop(call_id, None)
            if start is None or start[0] != identity or item.get("status") not in ("completed", "failed"):
                raise ValueError("Unmatched tool completion")
            success = item["status"] == "completed"
            if tool_kind == "mcp_tool_call":
                result = item.get("result")
                success = success and item.get("error") is None and isinstance(result, dict)
                if isinstance(result, dict):
                    success = success and all(result.get(k, False) is False for k in ("isError", "is_error"))
                    # An empty envelope proves no usable result for these read journeys.
                    success = success and result_payload(result)
            else:
                success = success and type(item.get("exit_code")) is int and item["exit_code"] == 0
            calls.append({"id": call_id, "type": tool_kind, "server": item.get("server"),
                          "tool": item.get("tool"), "success": bool(success),
                          "status": item["status"], "started_event": start[1], "completed_event": index,
                          "arguments_sha256": value_digest(item.get("arguments") if tool_kind == "mcp_tool_call"
                                                           else item["command"]),
                          "result_sha256": value_digest(item.get("result") if tool_kind == "mcp_tool_call"
                                                        else item.get("aggregated_output")),
                          "error_sha256": value_digest(item.get("error"))})
            done.add(call_id)
            last_tool = index
        else:
            raise ValueError("Unsupported tool event")
    outcome = termination_reason or ("terminated" if exit_code < 0 else "nonzero-exit" if exit_code else "completed")
    answer = None
    if outcome == "completed":
        try:
            if truncated or pending or last_tool > last_answer:
                raise ValueError("Incomplete calls or no final answer after tools")
            answer = probe.final_message("\n".join(map(json.dumps, remaining)), exit_code)
        except ValueError:
            outcome = "incomplete-or-failed-turn"
    usage = events[-1].get("usage") if events and events[-1].get("type") == "turn.completed" else None
    if usage is not None and (not isinstance(usage, dict) or not usage or any(
        not isinstance(k, str) or type(v) is not int or v < 0 for k, v in usage.items()
    )):
        raise ValueError("Invalid reported token usage")
    return answer, {"tool_calls": calls, "tool_call_count": len(calls) + len(pending),
                    "pending_tool_calls": [{"id": key, "identity_sha256": value_digest(value[0]),
                                            "started_event": value[1]}
                                           for key, value in pending.items()],
                    "failed_tool_calls": sum(not c["success"] for c in calls),
                    "usage": usage, "exit_code": exit_code, "run_outcome": outcome,
                    "truncated_event_tail": truncated}


def trace_summary(raw, exit_code):
    """Compatibility API for callers requiring a completed final answer."""
    answer, metrics = inspect_trace(raw, exit_code)
    if metrics["run_outcome"] != "completed":
        raise IncompleteObservation(metrics)
    return answer, metrics


def observed_case(root, manifest, case_id):
    def read(name):
        return bound_read(root, manifest, case_id + "/" + name)
    receipt = json.loads(read("capture.json"))
    raw, prompt = read("events.jsonl"), read("prompt.txt")
    if receipt["prompt_sha256"] != probe.sha(prompt) or receipt["events_sha256"] != probe.sha(raw):
        raise ValueError("Prompt or events changed")
    answer, metrics = inspect_trace(raw, receipt["exit_code"], receipt.get("termination_reason"))
    for field in ("elapsed_seconds", "timeout_seconds"):
        value = receipt.get(field)
        if type(value) not in (int, float) or not math.isfinite(value) or value < 0:
            raise ValueError("Missing or invalid timing")
        metrics[field] = value
    preservation = json.loads(read("preservation.json"))
    observe.fields(preservation, {"before", "after"})
    for state in preservation.values():
        if not isinstance(state, dict) or not state:
            raise ValueError("Missing preservation observation")
        for name, value in state.items():
            safe_name(name)
            if not isinstance(value, str) or not HASH.fullmatch(value):
                raise ValueError("Invalid preservation digest")
    metrics["project_config_unchanged"] = preservation["before"] == preservation["after"]
    if metrics["run_outcome"] != "completed":
        raise IncompleteObservation(metrics)
    if read("answer.md") != answer or receipt["answer_sha256"] != probe.sha(answer):
        raise ValueError("Answer differs from actual trace")
    return answer, receipt, metrics


def output_length(answer, limit, sources):
    """Count the whole delivered unit, excluding only registered anchor citations."""
    anchors = [re.escape(row["source_location"]) for row in csv.DictReader(io.StringIO(sources))]
    anchor = "(?:" + "|".join(anchors) + ")"
    citation = r"\s*" + anchor + r"(?:\s*[;,；，]\s*" + anchor + r")*\s*"
    for left, right in (("[", "]"), ("(", ")"), ("（", "）")):
        answer = re.sub(re.escape(left) + citation + re.escape(right), "", answer)
    count = sum(not char.isspace() for char in answer)
    return {**limit, "count": count, "pass": limit["min"] <= count <= limit["max"]}


def guidance_observation(root, manifest, case_id, paths, identity):
    """Bind complete guidance bytes to a read result or retained Host context.

    Host context provenance still requires review; a model's claim is not an
    injection record. Neither mechanism is inferred from a command or tool name.
    """
    record = json.loads(bound_read(root, manifest, case_id + "/guidance.json"))
    observe.fields(record, paths)
    events = [json.loads(line) for line in bound_read(root, manifest, case_id + "/events.jsonl").splitlines()
              if line.strip()]
    bound = {}
    for path in paths:
        evidence = record[path]
        if not isinstance(evidence, dict):
            raise ValueError("Invalid guidance evidence")
        mechanism = evidence.get("mechanism")
        key = "call_id" if mechanism == "command_output" else "context_artifact"
        observe.fields(evidence, {"mechanism", "artifact_path", key})
        body = bound_read(root, manifest, evidence["artifact_path"])
        digest = probe.sha(body)
        if not observe.text(body) or identity.get("guidance_files", {}).get(path) != digest:
            raise ValueError("Guidance does not match installed content")
        if mechanism == "command_output":
            matches = [e["item"] for e in events if e.get("type") == "item.completed"
                       and e.get("item", {}).get("id") == evidence["call_id"]
                       and e.get("item", {}).get("type") == "command_execution"]
            if (len(matches) != 1 or matches[0].get("status") != "completed"
                    or type(matches[0].get("exit_code")) is not int or matches[0]["exit_code"] != 0):
                raise ValueError("No successful guidance read")
            observed = matches[0].get("aggregated_output")
        elif mechanism == "host_injection":
            observed = bound_read(root, manifest, evidence["context_artifact"])
            if evidence["context_artifact"] in (evidence["artifact_path"], case_id + "/answer.md"):
                raise ValueError("Guidance snapshot or answer is not Host context evidence")
        else:
            raise ValueError("Unsupported guidance mechanism")
        if not isinstance(observed, str) or body not in observed:
            raise ValueError("Complete guidance bytes absent from observed context/output")
        bound[path] = {"sha256": digest, "mechanism": mechanism, key: evidence[key]}
    return bound


def prepare(root, destination):
    observe.separate_destination(destination, root)
    manifest, specs, _, _ = load_capture(root)
    cases = {}
    for case_id in CASES:
        try:
            answer, receipt, _ = observed_case(root, manifest, case_id)
            cases[case_id] = {"answer_sha256": receipt["answer_sha256"], "events_sha256": receipt["events_sha256"],
                "segments": [{"start": 0, "end": len(answer), "quote": answer, "role": "unmapped",
                              "links": [], "verdict": "unreviewed", "reason": ""}],
                "checks": {c: {"status": "unreviewed", "segments": [], "reason": ""} for c in specs[case_id]["checks"]}}
        except (OSError, ValueError, KeyError, TypeError):
            cases[case_id] = None
    with destination.open("x", encoding="utf-8") as stream:
        json.dump({"kind": KIND, "capture_sha256": probe.digest(root / "manifest.json"),
                   "reviewer": {"kind": "unassigned", "id": ""}, "cases": cases}, stream, ensure_ascii=False, indent=2)
        stream.write("\n")


def score(root, review_path, report):
    observe.separate_destination(report, root)
    if review_path.resolve().is_relative_to(report.resolve()):
        raise ValueError("Review must be outside report")
    manifest, specs, identity, sources = load_capture(root)
    review_text = review_path.read_bytes().decode("utf-8")
    review = json.loads(review_text)
    observe.fields(review, {"kind", "capture_sha256", "reviewer", "cases"})
    observe.fields(review["cases"], CASES)
    observe.fields(review["reviewer"], {"kind", "id"})
    if (review["kind"] != KIND or review["capture_sha256"] != probe.digest(root / "manifest.json")
            or review["reviewer"]["kind"] not in ("human", "model", "unassigned")):
        raise ValueError("Unbound review")
    report.mkdir(parents=True, exist_ok=False)
    (report / "review.json").write_text(review_text, encoding="utf-8")
    case_dir, outputs = report / "cases", report / "outputs"
    case_dir.mkdir()
    observations = {}
    for case_id in CASES:
        target = outputs / case_id
        target.mkdir(parents=True)
        binding = json.dumps({"capture": review["capture_sha256"], "review": probe.sha(review_text), "case": case_id}, sort_keys=True)
        case = {"schema_version": "1.0", "case_id": case_id, "pipeline": KIND,
                "input": {"topic": specs[case_id]["description"]}, "expected_outputs": {
                    "binding": {"artifact": "binding.json", "required": True,
                                "assertions": [{"type": "file_digest", "sha256": probe.sha(binding)}]},
                    "behavior": {"artifact": "behavior.csv", "required": True,
                                 "assertions": [{"type": "field_constraint", "field": "status", "allowed_values": ["pass"]}]}}}
        try:
            answer, receipt, metrics = observed_case(root, manifest, case_id)
            annotation = review["cases"][case_id]
            if any(annotation[k] != receipt[k] for k in ("answer_sha256", "events_sha256")):
                raise ValueError("Review belongs to another answer")
            _, semantic = observe.project(answer, annotation, sources, rubric=tuple(specs[case_id]["checks"]))
            if review["reviewer"]["kind"] == "unassigned" or not observe.text(review["reviewer"]["id"]):
                semantic["status"] = "unreviewed"
            claims = {link["claim_id"] for span in annotation["segments"] for link in span["links"]}
            tools = {c["tool"] for c in metrics["tool_calls"]
                     if c["server"] == identity.get("mcp_server", "qiongli") and c["success"]}
            # These observations are checked by the existing V1 field validator;
            # semantic correctness remains the named reviewer's separate judgment.
            behavior = {"requested_claim_coverage": set(specs[case_id]["required_claims"]) <= claims,
                        "required_native_calls": set(specs[case_id]["required_tools"]) <= tools,
                        "project_config_preservation": metrics["project_config_unchanged"]}
            if "length" in specs[case_id]:
                metrics["output_length"] = output_length(answer, specs[case_id]["length"], sources)
                behavior["output_length"] = metrics["output_length"]["pass"]
            if "guidance_paths" in specs[case_id]:
                try:
                    metrics["guidance"] = guidance_observation(root, manifest, case_id,
                                                               specs[case_id]["guidance_paths"], identity)
                    behavior["guidance_evidence"] = True
                except (OSError, ValueError, KeyError, TypeError):
                    metrics["guidance"] = {"status": "missing-or-invalid"}
                    behavior["guidance_evidence"] = False
            with (target / "behavior.csv").open("w", newline="", encoding="utf-8") as stream:
                writer = csv.writer(stream)
                writer.writerow(["check", "status"])
                writer.writerows((k, "pass" if v else "fail") for k, v in behavior.items())
            (target / "binding.json").write_text(binding, encoding="utf-8")
            observations[case_id] = {"semantic": semantic, "metrics": metrics, "behavior": behavior,
                                     "answer_sha256": receipt["answer_sha256"], "events_sha256": receipt["events_sha256"]}
        except IncompleteObservation as error:
            observations[case_id] = {"semantic": {"status": "unreviewed"},
                                     "binding": "incomplete-observation", "metrics": error.metrics}
        except (OSError, ValueError, KeyError, TypeError):
            observations[case_id] = {"semantic": {"status": "unreviewed"}, "binding": "missing-or-invalid"}
        (case_dir / (case_id + ".yaml")).write_text(yaml.safe_dump(case), encoding="utf-8")
    with redirect_stdout(io.StringIO()) as log:
        result = run_evals(case_dir, outputs, report / "receipts")
    (report / "score.log").write_text(log.getvalue(), encoding="utf-8")
    for case_id in CASES:
        receipt = json.loads((report / "receipts" / (case_id + ".json")).read_text())
        observations[case_id]["structural"] = receipt["case"]["status"]
    passed = sum(o["structural"] == "pass" and o["semantic"]["status"] == "pass" for o in observations.values())
    probe.write_json(report / "summary.json", {"kind": KIND, "capture_sha256": review["capture_sha256"],
        "review_sha256": probe.sha(review_text), "reviewer": review["reviewer"], "installation": identity,
        "case_count": len(CASES), "structural_passed": result.passed_cases, "reviewed_passed": passed,
        "cases": observations, "scorer_sha256": probe.digest(Path(__file__)),
        "limitations": "Local hashes bind supplied observations, not Host authentication. Installation and preservation "
                        "snapshots need independent verification. Whole-answer judgments remain reviewer assertions; "
                        "guidance loading is not inferred from tool names. Missing token usage stays null. "
                        "This bounded baseline does not establish general academic or cross-Host acceptance."})
    print(f"Structural {result.passed_cases}/3; reviewed {passed}/3. {report / 'summary.json'}")
    return passed == len(CASES)


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=("freeze", "seal", "prepare", "score"))
    parser.add_argument("capture", type=Path)
    parser.add_argument("--review", type=Path)
    parser.add_argument("--report", type=Path)
    args = parser.parse_args(argv)
    try:
        if ((args.mode == "score") != (args.report is not None)
                or (args.mode in ("prepare", "score")) != (args.review is not None)):
            raise ValueError("Review belongs to prepare/score; report belongs to score only")
        if args.mode in ("freeze", "seal"):
            (freeze_inputs if args.mode == "freeze" else seal_capture)(args.capture)
            return 0
        if args.mode == "prepare":
            prepare(args.capture, args.review)
            return 0
        return 0 if score(args.capture, args.review, args.report) else 1
    except (OSError, ValueError, KeyError, TypeError) as error:
        print(f"Plugin baseline unavailable: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
