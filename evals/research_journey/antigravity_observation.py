"""Bound one AGY status observation to raw events; never grant Host permissions.

Print-mode events are Host evidence, not a reconstruction of the MCP wire.
This adapter records full output when provided and refuses summaries as proof.
"""
from __future__ import annotations

import argparse
from contextlib import redirect_stdout
import csv
import io
import json
import os
from pathlib import Path
import re
import signal
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT))
from evals.research_journey import observe, plugin_baseline as baseline  # noqa: E402
from evals.runner.run_suite import run_evals  # noqa: E402
from tooling.scripts.validate_capability_contract import validate_instance  # noqa: E402

KIND = "qiongli-agy-status-observation/v1"
TOOL = "qiongli_config_status"
STATUS_SCHEMA = ROOT / "content/mcp-contracts/v2/schemas/qiongli_config_status.output.schema.json"
ANSI = re.compile(r"\x1b\[[0-?]*[ -/]*[@-~]|\x1b\][^\x07\x1b]*(?:\x07|\x1b\\)")


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError("Duplicate JSON field")
        result[key] = value
    return result


def once_permission(snapshot: str, server: str, cwd: str) -> bool:
    """Check a current, expanded permission screen; never send an approval key.

    Labels may be case-insensitive. Tool identities and JSON arguments may not.
    A caller must obtain a fresh screen and separately authorize the action.
    """
    text = ANSI.sub("", snapshot).replace("\r", "")
    question = text.lower().rfind("allow calling this tool?")
    if question < 0:
        return False
    tool = server + "/" + TOOL
    before, menu = text[:question], text[question:].lower()
    invocations = list(re.finditer(r"([\w./-]+)\(\s*\{", before))
    if not invocations or invocations[-1][1] != tool:
        return False
    invocation = before[invocations[-1].start() + len(tool) + 1:].replace("\n", "")
    try:
        args, end = json.JSONDecoder(object_pairs_hook=unique_object).raw_decode(invocation.lstrip())
    except ValueError:
        return False
    if args != {"cwd": cwd} or not invocation.lstrip()[end:].startswith(") (ctrl+o to collapse)"):
        return False
    lines = [line.strip() for line in menu.splitlines() if line.strip()]
    expected = ["> 1. yes, allow tool call",
                f"2. yes, and always allow tool '{tool.lower()}' in this conversation",
                f"3. yes, and always allow tool '{tool.lower()}' (persist to settings.json)"]
    return all(line in lines for line in expected) and sum(line.startswith(">") for line in lines) == 1


def status_output(value):
    """Validate full native status data and, separately, its complete MCP envelope."""
    try:
        if isinstance(value, str):
            value = json.loads(value, object_pairs_hook=unique_object)
        if not isinstance(value, dict):
            raise ValueError("No structured status output")
        schema = json.loads(STATUS_SCHEMA.read_text())
        if "structuredContent" not in value:
            return {"status_data_valid": not validate_instance(value, schema), "mcp_envelope_complete": False}
        body = value["structuredContent"]
        valid = not validate_instance(body, schema)
        blocks = value.get("content")
        complete = (valid and isinstance(blocks, list) and len(blocks) == 1
                    and isinstance(blocks[0], dict) and blocks[0].get("type") == "text"
                    and isinstance(blocks[0].get("text"), str)
                    and json.loads(blocks[0]["text"], object_pairs_hook=unique_object) == body
                    and all(value.get(k, False) is False for k in ("isError", "is_error")))
        return {"status_data_valid": valid, "mcp_envelope_complete": complete}
    except (ValueError, TypeError, KeyError):
        return {"status_data_valid": False, "mcp_envelope_complete": False}


def inspect_stream(raw: str, server: str, cwd: str):
    """Retain raw event indexes/value hashes, not tool-shaped response prose."""
    expected = {"ServerName": server, "ToolName": TOOL, "Arguments": {"cwd": cwd}}
    conversation, init, result = None, None, None
    steps, calls, problems = {}, [], []
    lines = [line for line in raw.splitlines() if line.strip()]
    for index, line in enumerate(lines):
        try:
            event = json.loads(line, object_pairs_hook=unique_object)
            if not isinstance(event, dict):
                raise ValueError("Invalid event")
            kind = event.get("event")
            if result is not None:
                raise ValueError("Event after terminal result")
            if kind == "init":
                if index != 0 or init is not None or not isinstance(event.get("init"), dict):
                    raise ValueError("Missing or repeated init")
                conversation, init = event.get("conversation_id"), event["init"]
                if not observe.text(conversation) or init.get("cwd") != cwd:
                    raise ValueError("Wrong conversation/workspace")
            elif kind == "step_update":
                step = event["step_update"]
                if not isinstance(step, dict) or not conversation or step.get("conversation_id") != conversation:
                    raise ValueError("Foreign or uninitialized step")
                step_id, state = step.get("step_index"), step.get("state")
                if type(step_id) is not int or step_id < 0 or state not in ("ACTIVE", "DONE"):
                    raise ValueError("Invalid step identity/state")
                if step.get("step_type") != "tool":
                    if step.get("step_type") not in ("user_input", "agent_response", "checkpoint"):
                        raise ValueError("Unknown step type")
                    continue
                info = step.get("tool_info")
                if (not isinstance(info, dict) or info.get("name") != step.get("tool_name")
                        or not isinstance(info.get("parameters"), dict)):
                    raise ValueError("Missing tool invocation")
                identity = {"name": info["name"], "parameters": info["parameters"]}
                previous = steps.get(step_id)
                if previous is None:
                    if state != "ACTIVE":
                        raise ValueError("Completion without start")
                    steps[step_id] = {"identity": identity, "started_event": index, "state": state}
                else:
                    if previous["state"] == "DONE" or previous["identity"] != identity:
                        raise ValueError("Repeated or changed invocation")
                    if state == "ACTIVE":  # A stream may repeat an unchanged active update.
                        continue
                    previous["state"] = "DONE"
                    matched = info["name"] == "call_mcp_tool" and info["parameters"] == expected
                    payload = status_output(info.get("output")) if matched else {
                        "status_data_valid": False, "mcp_envelope_complete": False}
                    calls.append({"step_index": step_id, "started_event": previous["started_event"],
                                  "completed_event": index, "expected_call": matched,
                                  "parameters_sha256": baseline.value_digest(info["parameters"]),
                                  "output_present": "output" in info,
                                  "output_sha256": baseline.value_digest(info.get("output")),
                                  "error_sha256": baseline.value_digest(info.get("error")),
                                  "tool_error": info.get("error") is not None, **payload})
            elif kind == "result":
                value = event.get("result")
                if (not isinstance(value, dict) or not conversation
                        or value.get("conversation_id") != conversation):
                    raise ValueError("Foreign or missing terminal result")
                result = value
            else:
                raise ValueError("Unsupported event")
        except (ValueError, KeyError, TypeError):
            problems.append({"event_index": index, "reason": "invalid-or-truncated-event"})
    pending = [key for key, value in steps.items() if value["state"] != "DONE"]
    usage = result.get("usage") if result else None
    if usage is not None and (not isinstance(usage, dict) or any(
        type(value) is not int or value < 0 for value in usage.values())):
        problems.append({"reason": "invalid-usage"})
        usage = None
    denied = result.get("denied_actions", []) if result else []
    if not isinstance(denied, list):
        problems.append({"reason": "invalid-denied-actions"})
        denied = ["invalid"]
    exact = [call for call in calls if call["expected_call"]]
    checks = {
        "complete_stream": init is not None and result is not None and not problems and not pending,
        "host_success": result is not None and result.get("status") == "SUCCESS",
        "single_completed_turn": result is not None and type(result.get("num_turns")) is int
                                 and result["num_turns"] == 1 and observe.text(result.get("response")),
        "no_permission_denial": not denied,
        "exactly_one_expected_call": len(calls) == 1 and len(exact) == 1,
        "full_native_result": len(exact) == 1 and exact[0]["mcp_envelope_complete"] and not exact[0]["tool_error"],
    }
    return {"checks": checks, "calls": calls, "pending_steps": pending, "problems": problems,
            "host_status": result.get("status") if result else None, "denied_action_count": len(denied),
            "usage": usage, "schema_sha256": baseline.probe.digest(STATUS_SCHEMA)}


def fingerprint(path: Path):
    """Hash explicitly selected public fixture roots; never follow links."""
    if path.is_symlink() or any(parent.is_symlink() for parent in path.parents):
        raise ValueError("Snapshot links are unsupported")
    if path.is_file():
        return {"file": baseline.probe.digest(path)}
    if not path.is_dir():
        raise ValueError("Snapshot root unavailable")
    values = {}
    for entry in sorted(path.rglob("*")):
        if entry.is_symlink():
            raise ValueError("Snapshot links are unsupported")
        if entry.is_file():
            values[entry.relative_to(path).as_posix()] = baseline.probe.digest(entry)
        elif not entry.is_dir():
            raise ValueError("Snapshot special files are unsupported")
    return values


def valid_snapshots(value):
    if (not isinstance(value, dict) or not {"workspace", "qiongli_config"} <= value.keys()
            or value.keys() - {"workspace", "qiongli_config", "host_settings"}):
        return False
    for files in value.values():
        if not isinstance(files, dict):
            return False
        for name, digest in files.items():
            try:
                baseline.safe_name(name)
            except ValueError:
                return False
            if not isinstance(digest, str) or not baseline.HASH.fullmatch(digest):
                return False
    if "host_settings" in value and set(value["host_settings"]) != {"file"}:
        return False
    return True


def stop_process(process, grace_seconds=3):
    """Terminate this owned POSIX process group before waiting; always reap it."""
    def group_present():
        try:
            os.killpg(process.pid, 0)
            return True
        except ProcessLookupError:
            return False

    def send(sig):
        try:
            os.killpg(process.pid, sig)
        except ProcessLookupError:
            pass

    code = process.poll()
    method = "already-exited"
    deadline = time.monotonic() + grace_seconds
    if code is None or group_present():
        method = "sigterm" if code is None else "sigterm-descendants"
        send(signal.SIGTERM)
        try:
            code = process.wait(timeout=grace_seconds)
        except subprocess.TimeoutExpired:
            method = "sigkill"
            send(signal.SIGKILL)
            code = process.wait()
        while group_present() and time.monotonic() < deadline:
            time.sleep(min(0.02, max(0, deadline - time.monotonic())))
        if group_present():
            method = "sigkill"
            send(signal.SIGKILL)
    return {"exit_code": code, "method": method, "reaped": process.poll() is not None,
            "group_gone": not group_present()}


def run_process(command, workspace, output, timeout_seconds, env=None):
    """Spool complete stdout/stderr to exclusive files, including failed attempts."""
    process, reason, failure, cleanup = None, "completed", None, None
    started = time.monotonic()
    with (output / "events.jsonl").open("xb") as stdout, (output / "stderr.log").open("xb") as stderr:
        try:
            process = subprocess.Popen(command, cwd=workspace, stdin=subprocess.DEVNULL,
                                       stdout=stdout, stderr=stderr, start_new_session=True, env=env)
            try:
                process.wait(timeout=timeout_seconds)
            except subprocess.TimeoutExpired:
                reason = "timeout"
        except BaseException as error:
            reason, failure = "driver-error", type(error).__name__
        finally:
            if process is not None:
                cleanup = stop_process(process)
    if reason == "completed" and cleanup is not None and cleanup["exit_code"] != 0:
        reason = "terminated" if cleanup["exit_code"] < 0 else "nonzero-exit"
    return {"termination_reason": reason, "driver_error": failure, "process_cleanup": cleanup,
            "elapsed_seconds": time.monotonic() - started, "timeout_seconds": timeout_seconds,
            "events_sha256": baseline.probe.digest(output / "events.jsonl"),
            "stderr_sha256": baseline.probe.digest(output / "stderr.log")}


def request(server, workspace):
    return (f"Call only {server}/{TOOL} once with the exact arguments "
            + json.dumps({"cwd": str(workspace)}, ensure_ascii=False)
            + ". Use its actual result to say whether it succeeded. Do not use other tools, read papers, "
              "write files or change permissions. If permission is denied, report it and stop; do not retry.")


def capture_command(agy, prompt, timeout_seconds):
    return [str(agy), "--mode", "plan", "--sandbox", "--effort", "low", "--output-format", "stream-json",
            "--print-timeout", f"{timeout_seconds}s", "--print", prompt]


def validate_installation(identity):
    if not isinstance(identity, dict) or not observe.text(identity.get("agy_version")):
        raise ValueError("Missing installed AGY identity")
    for key in ("cli_sha256", "content_pack_sha256", "plugin_receipt_sha256"):
        if not isinstance(identity.get(key), str) or not baseline.HASH.fullmatch(identity[key]):
            raise ValueError("Missing installed Qiongli identity")
    if not re.fullmatch(r"[a-f0-9]{40}", str(identity.get("source_commit", ""))):
        raise ValueError("Missing native source commit")
    server = identity.get("mcp_server")
    if not isinstance(server, str) or not re.fullmatch(r"[A-Za-z0-9_-]+", server):
        raise ValueError("Missing exact server identity")
    return server


def capture(output, workspace, config_root, identity_file, agy, timeout_seconds=180, host_settings=None):
    """One explicitly invoked live observation. No configuration or permission writes."""
    if os.name != "posix" or type(timeout_seconds) is not int or not 0 < timeout_seconds <= 180:
        raise ValueError("This bounded driver requires POSIX and at most 180 seconds")
    roots = {"workspace": workspace, "qiongli_config": config_root}
    if host_settings is not None:
        roots["host_settings"] = host_settings
    if not workspace.is_absolute() or not config_root.is_absolute() or not agy.is_absolute():
        raise ValueError("Use absolute fixture/config/executable paths")
    for root in roots.values():
        if output.resolve().is_relative_to(root.resolve()) or root.resolve().is_relative_to(output.resolve()):
            raise ValueError("Capture directory must be separate from fixture/config roots")
    identity = json.loads(identity_file.read_text())
    server = validate_installation(identity)
    before = {name: fingerprint(path) for name, path in roots.items()}
    output.mkdir(mode=0o700, parents=True, exist_ok=False)
    prompt = request(server, workspace)
    command = capture_command(agy, prompt, timeout_seconds)
    selected_env = {"QIONGLI_CONFIG_HOME": str(config_root), "QIONGLI_PROJECT_ROOT": str(workspace)}
    # Inherited Host credentials/model remain Host-owned; never serialize them.
    # A Plugin's explicit env can override these. Inspect its actual registration
    # before claiming the native server used this public fixture configuration.
    env = {**os.environ, **selected_env}
    (output / "prompt.txt").write_text(prompt, encoding="utf-8")
    baseline.exclusive_json(output / "installation.json", identity)
    baseline.exclusive_json(output / "inputs.json", {"command": command, "before": before,
        "workspace": str(workspace), "config_root": str(config_root), "environment_selection": selected_env,
        "agy_sha256": baseline.probe.digest(agy), "schema_sha256": baseline.probe.digest(STATUS_SCHEMA),
        "driver_sha256": baseline.probe.digest(Path(__file__)), "prompt_sha256": baseline.probe.sha(prompt)})
    receipt = run_process(command, workspace, output, timeout_seconds, env=env)
    after, errors = {}, []
    for name, path in roots.items():
        try:
            after[name] = fingerprint(path)
        except (OSError, ValueError):
            errors.append(name)
    receipt.update(before=before, after=after, preservation_errors=errors,
                   environment_selection_sha256=baseline.value_digest(selected_env),
                   snapshots_unchanged=not errors and before == after)
    baseline.exclusive_json(output / "capture.json", receipt)
    files = {p.name: baseline.probe.digest(p) for p in output.iterdir() if p.is_file()}
    baseline.exclusive_json(output / "manifest.json", {"kind": KIND, "files": files})
    return receipt


def score(root, report):
    observe.separate_destination(report, root)
    manifest = json.loads(observe.read_file(root, "manifest.json"))
    observe.fields(manifest, {"kind", "files"})
    if manifest["kind"] != KIND or not isinstance(manifest["files"], dict):
        raise ValueError("Invalid AGY observation manifest")
    for name in manifest["files"]:
        baseline.bound_read(root, manifest, name)
    read = lambda name: baseline.bound_read(root, manifest, name)
    identity, inputs, receipt = (json.loads(read(name)) for name in ("installation.json", "inputs.json", "capture.json"))
    validate_installation(identity)
    raw = read("events.jsonl")
    if (receipt["events_sha256"] != baseline.probe.sha(raw)
            or receipt["stderr_sha256"] != baseline.probe.sha(read("stderr.log"))
            or inputs["prompt_sha256"] != baseline.probe.sha(read("prompt.txt"))
            or inputs["schema_sha256"] != baseline.probe.digest(STATUS_SCHEMA)):
        raise ValueError("Changed capture/schema binding")
    command = inputs["command"]
    workspace = inputs["workspace"]
    expected_prompt = request(identity["mcp_server"], workspace)
    expected_env = {"QIONGLI_CONFIG_HOME": inputs["config_root"], "QIONGLI_PROJECT_ROOT": workspace}
    if (not isinstance(command, list) or not command
            or command != capture_command(command[0], expected_prompt, receipt["timeout_seconds"])
            or read("prompt.txt") != expected_prompt
            or type(receipt["timeout_seconds"]) is not int or not 0 < receipt["timeout_seconds"] <= 180):
        raise ValueError("Changed bounded capture invocation")
    if (inputs["environment_selection"] != expected_env
            or receipt.get("environment_selection_sha256") != baseline.value_digest(expected_env)):
        raise ValueError("Changed fixture environment selection")
    observation = inspect_stream(raw, identity["mcp_server"], workspace)
    cleanup = receipt.get("process_cleanup") or {}
    observation["checks"].update(
        process_completed=receipt.get("termination_reason") == "completed" and receipt.get("driver_error") is None
                          and type(cleanup.get("exit_code")) is int and cleanup["exit_code"] == 0
                          and cleanup.get("reaped") is True and cleanup.get("group_gone") is True
                          and cleanup.get("method") == "already-exited",
        snapshots_unchanged=all(valid_snapshots(v) for v in (receipt.get("before"), inputs.get("before"), receipt.get("after")))
                            and receipt.get("snapshots_unchanged") is True and receipt.get("preservation_errors") == []
                            and receipt.get("before") == inputs.get("before") == receipt.get("after"),
        configured_model_preserved="--model" not in command and "--dangerously-skip-permissions" not in command,
    )
    report.mkdir(parents=True, exist_ok=False)
    case_dir, fixture = report / "cases", report / "outputs/status"
    case_dir.mkdir()
    fixture.mkdir(parents=True)
    with (fixture / "behavior.csv").open("x", newline="") as stream:
        writer = csv.writer(stream)
        writer.writerow(["check", "status"])
        writer.writerows((key, "pass" if value else "fail") for key, value in observation["checks"].items())
    binding = {"manifest_sha256": baseline.probe.digest(root / "manifest.json"),
               "scorer_sha256": baseline.probe.digest(Path(__file__))}
    baseline.exclusive_json(fixture / "binding.json", binding)
    case = {"schema_version": "1.0", "case_id": "status", "pipeline": KIND,
            "input": {"topic": "One installed AGY native status call"}, "expected_outputs": {
                "behavior": {"artifact": "behavior.csv", "required": True, "assertions": [
                    {"type": "field_constraint", "field": "status", "allowed_values": ["pass"]}]},
                "binding": {"artifact": "binding.json", "required": True, "assertions": [
                    {"type": "file_digest", "sha256": baseline.probe.digest(fixture / "binding.json")}]}}}
    (case_dir / "status.yaml").write_text(json.dumps(case))
    with redirect_stdout(io.StringIO()) as log:
        result = run_evals(case_dir, report / "outputs", report / "receipts")
    (report / "score.log").write_text(log.getvalue())
    baseline.exclusive_json(report / "summary.json", {"kind": KIND, **binding, **observation,
        "capture": receipt, "structural_passed": result.success,
        "limitations": "Host event provenance and installation identity require independent inspection. "
                        "A result summary is not a full MCP envelope; model prose never fills missing bytes. "
                        "This single status observation does not qualify Skills or academic tasks."})
    return result.success


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="mode", required=True)
    run = sub.add_parser("capture")
    run.add_argument("output", type=Path)
    for name in ("workspace", "config-root", "identity-file", "agy"):
        run.add_argument("--" + name, type=Path, required=True)
    run.add_argument("--host-settings", type=Path)
    run.add_argument("--timeout-seconds", type=int, default=180)
    review = sub.add_parser("score")
    review.add_argument("capture", type=Path)
    review.add_argument("--report", type=Path, required=True)
    args = parser.parse_args(argv)
    try:
        if args.mode == "capture":
            receipt = capture(args.output, args.workspace, args.config_root, args.identity_file, args.agy,
                              args.timeout_seconds, args.host_settings)
            return 0 if (receipt["termination_reason"] == "completed" and receipt["snapshots_unchanged"]
                         and (receipt["process_cleanup"] or {}).get("exit_code") == 0) else 1
        return 0 if score(args.capture, args.report) else 1
    except (OSError, ValueError, KeyError, TypeError) as error:
        print(f"AGY observation unavailable: {type(error).__name__}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
