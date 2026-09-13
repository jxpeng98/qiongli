#!/usr/bin/env python3
"""Run the real CLI on synthetic records and retain a checked, offline example."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess

REPO = Path(__file__).resolve().parents[2]
FIXTURE = REPO / "tests/fixtures/research_graph_example"


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def run_example(cli, destination):
    if not __debug__:
        raise RuntimeError("Run without -O so example checks remain enabled")
    cli = Path(cli).resolve(strict=True)
    destination = Path(destination).absolute()
    # A user-selected fresh directory is the entire write scope. Never replace one.
    destination.mkdir(mode=0o700)
    destination = destination.resolve()
    project = destination / "project"
    shutil.copytree(FIXTURE, project)
    inputs = {p.relative_to(project).as_posix(): sha256(p)
              for p in project.rglob("*") if p.is_file()}
    env = {**os.environ, "QIONGLI_CONFIG_HOME": str(destination / "config"), "PATH": ""}
    calls = []

    def call(*args, expect_failure=False):
        result = subprocess.run([str(cli), *args], env=env, cwd=destination,
                                capture_output=True, text=True, encoding="utf-8",
                                timeout=60, check=False)
        calls.append({"args": list(args), "exitCode": result.returncode})
        if expect_failure:
            if result.returncode == 0 or "project-revision-conflict" not in result.stderr:
                raise RuntimeError("mismatched source revision was not rejected")
            return result.stderr.strip()
        if result.returncode:
            raise RuntimeError(result.stderr.strip() or "CLI command failed")
        return result.stdout

    version = call("--version").strip()
    registration = ["--root", str(project), "--name", "Synthetic reading study"]
    preview = json.loads(call("project", "register", "preview", *registration))
    project_id = preview["preview"]["projectId"]
    call("project", "register", "apply", *registration, "--project-id", project_id,
         "--expected-plan-digest", preview["preview"]["planDigest"], "--approve-filesystem-write")
    baseline = {p.relative_to(project).as_posix(): sha256(p)
                for p in project.rglob("*") if p.is_file()}
    snapshot_text = call("project", "graph", "snapshot", "--project-id", project_id, "--json")
    result = json.loads(snapshot_text)
    graph = result["snapshot"]
    claims = {n["canonicalId"]: n for n in graph["nodes"] if n["nodeType"] == "claim"}
    support = [e for e in graph["edges"] if e["relation"] == "supports"]
    informs = [e for e in graph["edges"] if e["relation"] == "informs"]
    assert len(support) == 2 and all(e["targetNodeId"] == claims["CLM-1"]["nodeId"] for e in support)
    assert all(e["status"] == "reviewed" and "Synthetic pilot" in e["evidenceLimit"] for e in support)
    assert {e["status"] for e in informs} == {"reviewed", "proposed"}
    assert all("not supporting evidence" in e["evidenceLimit"] for e in informs)
    assert any(d.get("relatedId") == "CLM-2" for d in graph["diagnostics"])
    assert not any(e["targetNodeId"] == claims["CLM-2"]["nodeId"] for e in support)

    selected = next(e for e in support if "no random assignment" in e["evidenceLimit"])
    source_args = ["project", "graph", "source", "--project-id", project_id,
                   "--expected-project-revision", str(graph["projectRevision"]),
                   "--expected-projection-id", graph["projectionId"], "--edge-id", selected["edgeId"]]
    source = json.loads(call(*source_args, "--json"))
    assert source["artifact"]["anchorMatched"]
    assert "DemoA,Table 1,notes/synthetic-reading-notes.md" in source["artifact"]["content"]
    mismatch_args = source_args.copy()
    mismatch_args[mismatch_args.index("--expected-project-revision") + 1] = str(graph["projectRevision"] + 1)
    mismatch_reason = call(*mismatch_args, expect_failure=True)
    assert json.loads(call("project", "graph", "snapshot", "--project-id", project_id, "--json")) == result

    html = call("project", "graph", "view", "--project-id", project_id)
    embedded = html.split('<script id="graph-data" type="application/json">', 1)[1].split("</script>", 1)[0]
    assert json.loads(embedded) == {"snapshot": graph, "readiness": result["readiness"]}
    assert "__QIONGLI_GRAPH_DATA__" not in html and "connect-src 'none'" in html
    assert str(destination) not in html and str(destination) not in snapshot_text
    after = {p.relative_to(project).as_posix(): sha256(p)
             for p in project.rglob("*") if p.is_file()}
    assert baseline == after
    assert all(after[path] == digest for path, digest in inputs.items())

    for name, text in [("research-graph.html", html), ("snapshot.json", snapshot_text),
                       ("source-excerpt.json", json.dumps(source, ensure_ascii=False, indent=2) + "\n")]:
        with (destination / name).open("x", encoding="utf-8", newline="\n") as output:
            output.write(text)
    proof = {
        "synthetic": True, "cliVersion": version, "cliSha256": sha256(cli),
        "projectId": project_id, "projectRevision": graph["projectRevision"],
        "projectionId": graph["projectionId"],
        "inputSha256": inputs, "htmlSha256": sha256(destination / "research-graph.html"),
        "semanticRecords": sum(n["nodeType"] not in ("project", "artifact") for n in graph["nodes"]),
        "semanticRelations": sum(e["relation"] != "contains" for e in graph["edges"]),
        "supportEdges": len(support), "decisionStatuses": sorted(e["status"] for e in informs),
        "diagnostics": graph["diagnostics"], "revisionMismatch": mismatch_reason,
        "checks": ["real-cli-registration-preview-apply", "evidence-limits-preserved",
                   "unsupported-claim-not-promoted", "decisions-are-not-support",
                   "source-anchor-resolved", "revision-mismatch-rejected",
                   "repeatable-projection", "html-matches-snapshot",
                   "input-and-project-files-unchanged", "empty-path-cli-execution"]
    }
    (destination / "proof.json").write_text(json.dumps(proof, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    (destination / "commands.json").write_text(json.dumps(calls, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print("Example:", destination / "research-graph.html")
    print("Evidence:", destination / "proof.json")
    print("For this example's source commands, set QIONGLI_CONFIG_HOME to:", env["QIONGLI_CONFIG_HOME"])
    print("Synthetic normalization is supplied, not inferred by the CLI. Files are retained; no browser is launched.")
    return destination


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cli", required=True, help="path to the built native Qiongli executable")
    parser.add_argument("--destination", required=True, help="new directory; its parent must already exist")
    args = parser.parse_args()
    try:
        run_example(args.cli, args.destination)
    except (OSError, RuntimeError, AssertionError, KeyError, subprocess.TimeoutExpired) as error:
        parser.exit(1, "Example did not finish: " + str(error) + "\nExisting files are retained.\n")


if __name__ == "__main__":
    main()
