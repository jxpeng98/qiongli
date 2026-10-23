"""Small, explicit measurements for public writing observations.

These checks bind inputs and count delivered text; they do not judge prose,
source support, installed-Host behavior or research acceptance. Historical
plugin_baseline length contracts keep their existing semantics.
"""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import re


def _bytes(path):
    path = Path(path)
    if path.is_symlink() or any(p.is_symlink() for p in path.parents):
        raise ValueError("Selected evidence must not traverse a symlink")
    if not path.is_file():
        raise ValueError("Selected evidence file is unavailable")
    return path.read_bytes()


def bind_files(files):
    """Bind only caller-selected files, never discover private/config roots."""
    if not isinstance(files, dict) or not files:
        raise ValueError("An explicit nonempty file selection is required")
    result = {}
    for name, path in files.items():
        if not isinstance(name, str) or not name:
            raise ValueError("File identities must be nonempty strings")
        data = _bytes(path)
        result[name] = {"sha256": hashlib.sha256(data).hexdigest(), "bytes": len(data)}
    return result


def verify_files(files, binding):
    """A different original, candidate, task or guidance invalidates its review."""
    if bind_files(files) != binding:
        raise ValueError("Selected evidence differs from its frozen binding")


def freeze_packet(files, destination):
    """Save one explicit review selection; never overwrite a draft or packet.

    This local evaluation record confers no project-write approval. The caller
    supplies authorized files and retains the returned digest for a new session.
    """
    binding = bind_files(files)
    destination = Path(destination)
    if destination.is_symlink() or any(p.is_symlink() for p in destination.parents):
        raise ValueError("Packet destination must not traverse a symlink")
    packet = {"kind": "qiongli-writing-packet/v1",
              "paths": {name: str(Path(path).absolute()) for name, path in files.items()},
              "binding": binding}
    data = (json.dumps(packet, ensure_ascii=False, indent=2) + "\n").encode("utf-8")
    with destination.open("xb") as stream:
        stream.write(data)
    return {"packet": str(destination.absolute()), "sha256": hashlib.sha256(data).hexdigest()}


def recover_packet(path, expected_sha256):
    """Recover exact review inputs, refusing stale originals, returns or sources."""
    data = _bytes(path)
    if hashlib.sha256(data).hexdigest() != expected_sha256:
        raise ValueError("Review packet changed since handoff")
    packet = json.loads(data)
    if (not isinstance(packet, dict) or set(packet) != {"kind", "paths", "binding"}
            or packet["kind"] != "qiongli-writing-packet/v1"
            or not isinstance(packet["paths"], dict)
            or any(not isinstance(p, str) or not Path(p).is_absolute()
                   for p in packet["paths"].values())):
        raise ValueError("Invalid writing packet")
    verify_files(packet["paths"], packet["binding"])
    return {"packet_sha256": expected_sha256, "status": "bindings_current",
            "files": {name: {"path": p, **packet["binding"][name]}
                      for name, p in packet["paths"].items()},
            "scope": "Input recovery only; no semantic verdict or approval to save a manuscript."}


def antigravity_answer(raw, cwd):
    """Reuse Host stream validation without claiming an unrequested MCP result.

    A completed text edit needs no native status call. Preserve failed/pending
    tools and unknown model identity separately; never extract an answer from
    partial response deltas or a model's claim about its own identity.
    """
    from evals.research_journey.antigravity_observation import inspect_stream

    observation = inspect_stream(raw, "not-requested", str(cwd))
    checks = {key: observation["checks"][key] for key in
              ("complete_stream", "host_success", "single_completed_turn", "no_permission_denial")}
    reported = []
    terminal = None
    for line in raw.splitlines():
        try:
            event = json.loads(line)
        except ValueError:
            continue
        if not isinstance(event, dict):
            continue
        kind = event.get("event")
        if kind not in ("init", "result"):
            continue
        body = event.get(kind)
        if not isinstance(body, dict):
            continue
        for key in ("model", "model_id", "model_name"):
            if isinstance(body.get(key), str) and body[key].strip():
                reported.append({"event": kind, "field": key, "value": body[key]})
        if kind == "result":
            terminal = body
    answer = terminal["response"] if all(checks.values()) else None
    return answer, {"checks": checks, "calls": observation["calls"],
                    "structured_finishes": observation["structured_finishes"],
                    "pending_steps": observation["pending_steps"], "problems": observation["problems"],
                    "usage": observation["usage"], "reported_model_fields": reported,
                    "scope": "Text-turn completion only; no native status, semantic fidelity or model inference."}


def count_text(text, unit, anchor_pattern, anchor_exclusion):
    """Count a body with explicit units and citation exclusions; keep unknown text."""
    if unit not in ("codepoints", "words"):
        raise ValueError("Unknown counting unit")
    if anchor_exclusion not in ("none", "standalone", "all"):
        raise ValueError("Unknown source-anchor exclusion")
    anchor = re.compile(anchor_pattern)
    if anchor.match(""):
        raise ValueError("An anchor pattern must not match empty text")
    if anchor_exclusion == "all":
        text = anchor.sub("", text)
    elif anchor_exclusion == "standalone":
        text = "\n".join(
            line for line in text.splitlines()
            if not (anchor.search(line) and not anchor.sub("", line).strip())
        )
    return sum(not c.isspace() for c in text) if unit == "codepoints" else len(text.split())


def measure_document(text, contract):
    """Measure exactly the requested ordered language/section headings.

    Fail on missing, duplicate or extra headings and unassigned introductory text.
    Section bodies cannot borrow their headings' characters to satisfy a limit.
    """
    if set(contract) != {"anchor_pattern", "anchor_exclusion", "languages"}:
        raise ValueError("Unexpected length-contract fields")
    languages = contract["languages"]
    if not isinstance(languages, list) or not languages:
        raise ValueError("At least one language is required")
    expected = []
    for language in languages:
        if set(language) != {"heading", "sections", "unit", "min", "max"}:
            raise ValueError("Unexpected language-contract fields")
        if (type(language["min"]) is not int or type(language["max"]) is not int
                or not 0 < language["min"] <= language["max"]):
            raise ValueError("Invalid length range")
        if not isinstance(language["sections"], list) or not language["sections"]:
            raise ValueError("At least one section is required")
        expected.extend([language["heading"], *language["sections"]])
    if (any(not isinstance(x, str) or not re.fullmatch(r"#{1,6} +\S.*", x) for x in expected)
            or len(set(expected)) != len(expected)):
        raise ValueError("Contract headings must be unique Markdown headings")
    matches = list(re.finditer(r"^#{1,6} +[^\n]+", text, re.M))
    if [m[0].rstrip() for m in matches] != expected:
        raise ValueError("Delivered headings differ from the frozen layout")
    if text[:matches[0].start()].strip():
        raise ValueError("Unassigned text precedes the requested sections")
    bodies = {m[0].rstrip(): text[m.end():matches[i+1].start() if i+1 < len(matches) else len(text)].strip()
              for i, m in enumerate(matches)}
    result = {}
    for language in languages:
        if bodies[language["heading"]]:
            raise ValueError("Unassigned text follows a language heading")
        sections = {}
        for name in language["sections"]:
            value = count_text(bodies[name], language["unit"], contract["anchor_pattern"],
                               contract["anchor_exclusion"])
            if value == 0:
                raise ValueError("A requested section is empty")
            sections[name] = value
        total = sum(sections.values())
        result[language["heading"]] = {"unit": language["unit"], "count": total,
            "min": language["min"], "max": language["max"], "sections": sections,
            "pass": language["min"] <= total <= language["max"]}
    return {"anchor_exclusion": contract["anchor_exclusion"], "languages": result,
            "pass": all(x["pass"] for x in result.values())}


def read_chunk(path, expected_sha256, offset=0, max_bytes=6000):
    """Return a hash-bound text span whose complete JSON fits a UTF-8 byte budget.

    Offsets are Unicode code points, never PDF page locators. The caller records
    actual delivery separately, follows next_offset to fill missing ranges and
    must budget the outer tool envelope too. No optional PDF utility is invoked.
    """
    data = _bytes(path)
    digest = hashlib.sha256(data).hexdigest()
    if digest != expected_sha256:
        raise ValueError("Source changed since the requested binding")
    text = data.decode("utf-8")
    if type(offset) is not int or not 0 <= offset <= len(text):
        raise ValueError("Invalid code-point offset")
    if type(max_bytes) is not int or max_bytes <= 0:
        raise ValueError("Invalid serialized-output budget")

    def packet(end):
        return {"sha256": digest, "start_offset": offset, "end_offset": end,
                "next_offset": end if end < len(text) else None,
                "total_codepoints": len(text), "text": text[offset:end],
                "complete": end == len(text)}

    def fits(end):
        return len(json.dumps(packet(end), ensure_ascii=False).encode("utf-8")) <= max_bytes

    # Check EOF separately: its null cursor can be shorter than a numeric cursor.
    if fits(len(text)):
        return packet(len(text))
    lo, hi = offset, len(text) - 1
    while lo < hi:
        mid = (lo + hi + 1) // 2
        if fits(mid):
            lo = mid
        else:
            hi = mid - 1
    if lo == offset or not fits(lo):
        raise ValueError("Budget cannot deliver a nonempty text span")
    return packet(lo)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)
    read = sub.add_parser("read", help="Read a bound UTF-8 extraction or guidance chunk")
    read.add_argument("path", type=Path)
    read.add_argument("--sha256", required=True)
    read.add_argument("--offset", type=int, default=0)
    read.add_argument("--max-bytes", type=int, default=6000)
    measure = sub.add_parser("measure", help="Measure a frozen section contract")
    measure.add_argument("answer", type=Path)
    measure.add_argument("contract", type=Path)
    freeze = sub.add_parser("freeze", help="Bind explicitly selected files for a later review")
    freeze.add_argument("destination", type=Path)
    freeze.add_argument("--file", action="append", required=True, metavar="ROLE=PATH")
    recover = sub.add_parser("recover", help="Verify a packet before continuing in a new session")
    recover.add_argument("packet", type=Path)
    recover.add_argument("--sha256", required=True)
    args = parser.parse_args()
    if args.command == "read":
        result = read_chunk(args.path, args.sha256, args.offset, args.max_bytes)
    elif args.command == "measure":
        result = measure_document(_bytes(args.answer).decode("utf-8"),
                                  json.loads(_bytes(args.contract)))
    elif args.command == "recover":
        result = recover_packet(args.packet, args.sha256)
    else:
        files = {}
        for selection in args.file:
            name, separator, path = selection.partition("=")
            if not separator or not name or not path or name in files:
                parser.error("Each --file must have a unique nonempty ROLE=PATH")
            files[name] = Path(path)
        result = freeze_packet(files, args.destination)
    print(json.dumps(result, ensure_ascii=False))
    if args.command == "measure" and not result["pass"]:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
