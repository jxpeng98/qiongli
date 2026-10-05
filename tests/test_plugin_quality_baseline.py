"""Synthetic trace/receipt negatives; these tests do not execute a live Host."""
from contextlib import redirect_stdout
import csv
import io
import json
from pathlib import Path
import tempfile
import unittest

from evals.research_journey import observe, plugin_baseline as baseline
from evals.skill_routing import probe


ANSWER = "The supplied ten-person study reports an association; causality and wider applicability remain uncertain." + "证" * 160


def trace(*, native=True, usage=True):
    events = [{"type": "thread.started"}, {"type": "turn.started"}]
    if native:
        for index, tool in enumerate(("qiongli_project_read", "qiongli_project_document_list", "qiongli_project_document_read")):
            item = {"type": "mcp_tool_call", "id": str(index), "server": "qiongli", "tool": tool,
                    "arguments": {"project_id": "synthetic"}}
            events.extend([
                {"type": "item.started", "item": {**item, "status": "in_progress"}},
                {"type": "item.completed", "item": {**item, "status": "completed", "error": None,
                                                     "result": {"content": [{"type": "text", "text": "{\"synthetic\":true}"}]}}},
            ])
    events.append({"type": "item.completed", "item": {"type": "agent_message", "text": ANSWER}})
    events.append({"type": "turn.completed", **({"usage": {"input_tokens": 100, "cached_input_tokens": 60,
                                                           "output_tokens": 20}} if usage else {})})
    return events


class PluginBaselineTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.capture = self.root / "capture"
        self.capture.mkdir()
        catalog = Path(baseline.__file__).with_name("plugin-baseline.json")
        (self.capture / "catalog.json").write_bytes(catalog.read_bytes())
        probe.write_json(self.capture / "installation.json", {
            "source_commit": "a" * 40, "cli_sha256": "b" * 64, "content_pack_sha256": "c" * 64,
            "plugin_receipt_sha256": "d" * 64, "codex_version": "synthetic-test", "model": "test-model",
            "reasoning_effort": "low"})
        (self.capture / "inputs").mkdir()
        (self.capture / "inputs/source.md").write_text("# Study:result\n" + ANSWER)
        with (self.capture / "sources.csv").open("w", newline="") as stream:
            writer = csv.writer(stream)
            writer.writerow(observe.SOURCE_FIELDS)
            writer.writerow(["Study", "Study:result", "inputs/source.md", "synthetic excerpt only"])
        for case_id in baseline.CASES:
            target = self.capture / case_id
            target.mkdir()
            (target / "prompt.txt").write_text("Bounded synthetic task; no private data.")
            (target / "answer.md").write_text(ANSWER)
            self.set_events(case_id, trace())
            probe.write_json(target / "preservation.json", {"before": {"project/note.md": "e" * 64},
                                                           "after": {"project/note.md": "e" * 64}})
        self.report_count = 0
        catalog_value = json.loads(catalog.read_text())
        guidance = {path for spec in catalog_value["cases"].values() for path in spec["guidance_paths"]}
        identity_path = self.capture / "installation.json"
        identity = json.loads(identity_path.read_text())
        identity["guidance_files"] = {}
        for path in sorted(guidance):
            body = "Complete synthetic installed guidance for " + path + "\n"
            artifact = "inputs/guidance/" + path
            target = self.capture / artifact
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text(body)
            identity["guidance_files"][path] = probe.sha(body)
        probe.write_json(identity_path, identity)
        for case_id, spec in catalog_value["cases"].items():
            context = case_id + "/host-context.txt"
            (self.capture / context).write_text("\n".join(
                (self.capture / ("inputs/guidance/" + path)).read_text() for path in spec["guidance_paths"]))
            probe.write_json(self.capture / case_id / "guidance.json", {
                path: {"mechanism": "host_injection", "artifact_path": "inputs/guidance/" + path,
                       "context_artifact": context} for path in spec["guidance_paths"]})
        self.seal()

    def set_events(self, case_id, events, exit_code=0):
        target = self.capture / case_id
        raw = "\n".join(map(json.dumps, events))
        (target / "events.jsonl").write_text(raw)
        probe.write_json(target / "capture.json", {"exit_code": exit_code, "elapsed_seconds": 2.5,
            "timeout_seconds": 180, "prompt_sha256": probe.digest(target / "prompt.txt"),
            "events_sha256": probe.sha(raw), "answer_sha256": probe.digest(target / "answer.md")})

    def seal(self):
        probe.write_json(self.capture / "manifest.json", {"kind": baseline.KIND, "cases": list(baseline.CASES),
            "files": {p.relative_to(self.capture).as_posix(): probe.digest(p) for p in self.capture.rglob("*")
                      if p.is_file() and p.name != "manifest.json"}})

    def review(self):
        path = self.root / f"review-{self.report_count}.json"
        baseline.prepare(self.capture, path)
        review = json.loads(path.read_text())
        review["reviewer"] = {"kind": "model", "id": "synthetic-test-reviewer"}
        for case in review["cases"].values():
            if case is None:
                continue
            span = case["segments"][0]
            span.update(role="manuscript", verdict="pass", reason="Synthetic judgment against fixture.",
                links=[{"claim_id": claim, "source_location": "Study:result", "status": "supported",
                        "claim_type": "limitation"} for claim in ("Q-C1", "Q-C2", "Q-C3")])
            case["checks"] = {key: {"status": "pass", "segments": [0], "reason": "Synthetic judgment."}
                              for key in case["checks"]}
        probe.write_json(path, review)
        return path

    def score(self, review=None):
        review = review or self.review()
        target = self.root / f"report-{self.report_count}"
        self.report_count += 1
        with redirect_stdout(io.StringIO()):
            passed = baseline.score(self.capture, review, target)
        result = json.loads((target / "summary.json").read_text())
        self.assertEqual(result["case_count"], 3)
        self.assertEqual(passed, result["reviewed_passed"] == 3)
        return result

    def test_valid_whole_answers_use_existing_runner_and_report_actual_usage(self):
        result = self.score()
        self.assertEqual(result["structural_passed"], 3)
        self.assertEqual(result["reviewed_passed"], 3)
        metrics = result["cases"]["saved-continuation"]["metrics"]
        self.assertEqual(metrics["tool_call_count"], 3)
        self.assertEqual(metrics["failed_tool_calls"], 0)
        self.assertEqual(metrics["usage"]["cached_input_tokens"], 60)
        self.assertEqual(metrics["elapsed_seconds"], 2.5)
        receipt = json.loads((self.root / "report-0/receipts/saved-continuation.json").read_text())
        self.assertEqual(receipt["summary"]["executed_assertions"], 2)

    def test_unreviewed_and_semantic_failure_do_not_become_quality_success(self):
        unreviewed = self.root / "unreviewed.json"
        baseline.prepare(self.capture, unreviewed)
        self.assertEqual(self.score(unreviewed)["reviewed_passed"], 0)
        path = self.review()
        review = json.loads(path.read_text())
        case = review["cases"]["paper-explanation"]
        case["segments"][0].update(verdict="fail", reason="A claimed causal effect exceeds the evidence.")
        case["checks"]["claim_strength"].update(status="fail", reason="Overclaim.")
        probe.write_json(path, review)
        result = self.score(path)
        self.assertEqual(result["structural_passed"], 3)
        self.assertEqual(result["reviewed_passed"], 2)

    def test_partial_answer_review_and_forged_locator_fail_closed(self):
        for mutation in ("partial", "locator"):
            with self.subTest(mutation=mutation):
                path = self.review()
                review = json.loads(path.read_text())
                span = review["cases"]["source-paragraph"]["segments"][0]
                if mutation == "partial":
                    span.update(end=len(ANSWER) - 10, quote=ANSWER[:-10])
                else:
                    span["links"][0]["source_location"] = "Study:invented"
                probe.write_json(path, review)
                self.assertEqual(self.score(path)["reviewed_passed"], 2)

    def test_tool_names_in_prose_cannot_satisfy_native_recovery_calls(self):
        self.set_events("saved-continuation", trace(native=False))
        self.seal()
        result = self.score()
        self.assertEqual(result["structural_passed"], 2)
        self.assertFalse(result["cases"]["saved-continuation"]["behavior"]["required_native_calls"])

    def test_failed_and_unattempted_cases_remain_in_denominator(self):
        self.set_events("source-paragraph", trace(), exit_code=124)
        (self.capture / "saved-continuation/events.jsonl").unlink()
        self.seal()
        result = self.score()
        self.assertEqual(result["structural_passed"], 1)
        self.assertEqual(result["reviewed_passed"], 1)
        manifest = json.loads((self.capture / "manifest.json").read_text())
        manifest["cases"].pop()
        probe.write_json(self.capture / "manifest.json", manifest)
        with self.assertRaises(ValueError):
            baseline.load_capture(self.capture)

    def test_changed_project_bytes_block_behavior_pass(self):
        probe.write_json(self.capture / "paper-explanation/preservation.json", {
            "before": {"project/note.md": "e" * 64}, "after": {"project/note.md": "f" * 64}})
        self.seal()
        self.assertEqual(self.score()["structural_passed"], 2)

    def test_unbound_changed_source_and_escaping_paths_are_rejected(self):
        source = self.capture / "inputs/source.md"
        original = source.read_text()
        source.write_text(original + "\nchanged")
        with self.assertRaises(ValueError):
            baseline.load_capture(self.capture)
        source.write_text(original)
        source.unlink()
        outside = self.root / "outside.md"
        outside.write_text(original)
        source.symlink_to(outside)
        with self.assertRaises(ValueError):
            baseline.load_capture(self.capture)
        for name in ("../outside.md", "/tmp/outside.md", "inputs/../outside.md", "inputs\\outside.md"):
            with self.subTest(name=name), self.assertRaises(ValueError):
                baseline.safe_name(name)

    def test_unmatched_repeated_changed_and_incomplete_calls_refuse(self):
        for mutation in ("unmatched", "repeated", "arguments", "pending", "after-answer", "unknown"):
            events = trace()
            if mutation == "unmatched":
                events.pop(2)
            elif mutation == "repeated":
                events.insert(3, events[2])
            elif mutation == "arguments":
                events[3]["item"]["arguments"] = {"project_id": "another"}
            elif mutation == "pending":
                events.pop(3)
            elif mutation == "after-answer":
                events.insert(2, events.pop(-2))
            else:
                events.insert(2, {"type": "item.completed", "item": {"type": "unsupported_tool"}})
            with self.subTest(mutation=mutation), self.assertRaises(ValueError):
                baseline.trace_summary("\n".join(map(json.dumps, events)), 0)

    def test_failed_tool_is_counted_and_missing_usage_remains_unknown(self):
        events = trace(usage=False)
        events[3]["item"]["result"]["isError"] = True
        _, metrics = baseline.trace_summary("\n".join(map(json.dumps, events)), 0)
        self.assertEqual(metrics["failed_tool_calls"], 1)
        self.assertIsNone(metrics["usage"])
        events[-1]["usage"] = {"input_tokens": -1}
        with self.assertRaises(ValueError):
            baseline.trace_summary("\n".join(map(json.dumps, events)), 0)



    def test_empty_and_conflicting_mcp_results_do_not_satisfy_required_calls(self):
        for result in ({}, {"content": []}, {"content": [{}]}, {"content": [{"type": "unknown", "text": "x"}]},
                       {"content": [{"type": "text", "text": ""}]},
                       {"content": [{"type": "text", "text": "ok"}],
                                          "isError": False, "is_error": True}):
            with self.subTest(result=result):
                events = trace()
                events[3]["item"]["result"] = result
                self.set_events("saved-continuation", events)
                self.seal()
                summary = self.score()
                self.assertFalse(summary["cases"]["saved-continuation"]["behavior"]["required_native_calls"])
                self.assertEqual(summary["structural_passed"], 2)

    def test_call_value_digests_bind_exact_raw_event_values(self):
        events = trace()
        _, metrics = baseline.inspect_trace("\n".join(map(json.dumps, events)), 0)
        call = metrics["tool_calls"][0]
        completed = events[call["completed_event"]]["item"]
        for field in ("arguments", "result", "error"):
            expected = probe.sha(json.dumps(completed.get(field), ensure_ascii=False,
                                           sort_keys=True, separators=(",", ":")))
            self.assertEqual(call[field + "_sha256"], expected)
        self.assertEqual(events[call["started_event"]]["item"]["id"], call["id"])

    def test_incomplete_runs_retain_completed_calls_without_passing(self):
        for code, reason, expected in ((-9, None, "terminated"), (124, "timeout", "timeout"),
                                       (0, "permission-denied", "permission-denied"),
                                       (0, None, "incomplete-or-failed-turn")):
            with self.subTest(code=code, reason=reason):
                events = trace()
                if expected == "incomplete-or-failed-turn":
                    events.pop()
                self.set_events("saved-continuation", events, exit_code=code)
                receipt_path = self.capture / "saved-continuation/capture.json"
                receipt = json.loads(receipt_path.read_text())
                if reason:
                    receipt["termination_reason"] = reason
                probe.write_json(receipt_path, receipt)
                self.seal()
                result = self.score()
                case = result["cases"]["saved-continuation"]
                self.assertNotEqual(case["structural"], "pass")
                self.assertEqual(case["metrics"]["run_outcome"], expected)
                self.assertEqual(case["metrics"]["tool_call_count"], 3)
                self.assertTrue(all(call["success"] for call in case["metrics"]["tool_calls"]))
                self.assertEqual(case["metrics"]["elapsed_seconds"], 2.5)

    def test_pending_call_and_failed_turn_preserve_completed_results(self):
        for failed_turn in (False, True):
            with self.subTest(failed_turn=failed_turn):
                events = trace()
                if failed_turn:
                    events[-1] = {"type": "turn.failed", "error": {"message": "synthetic failure"}}
                else:
                    events.insert(-2, {"type": "item.started", "item": {
                        "id": "pending", "type": "mcp_tool_call", "server": "qiongli",
                        "tool": "qiongli_project_document_read", "arguments": {"project_id": "synthetic"},
                        "status": "in_progress"}})
                answer, metrics = baseline.inspect_trace("\n".join(map(json.dumps, events)), 0)
                self.assertIsNone(answer)
                self.assertEqual(metrics["run_outcome"], "incomplete-or-failed-turn")
                self.assertEqual(len(metrics["tool_calls"]), 3)
                self.assertTrue(all(call["success"] for call in metrics["tool_calls"]))
                self.assertEqual(len(metrics["pending_tool_calls"]), 0 if failed_turn else 1)
                self.assertEqual(metrics["tool_call_count"], 3 if failed_turn else 4)

    def test_truncated_tail_retains_calls_but_never_completes(self):
        raw = "\n".join(map(json.dumps, trace())) + '\n{"type":"item.completed"'
        answer, metrics = baseline.inspect_trace(raw, 0)
        self.assertIsNone(answer)
        self.assertTrue(metrics["truncated_event_tail"])
        self.assertEqual(metrics["run_outcome"], "incomplete-or-failed-turn")
        self.assertEqual(len(metrics["tool_calls"]), 3)
        self.assertTrue(all(call["success"] for call in metrics["tool_calls"]))
        with self.assertRaises(ValueError):
            baseline.trace_summary(raw, 0)
        # Corruption before a valid later event is not a recoverable truncated tail.
        malformed_middle = raw.splitlines()
        malformed_middle.insert(3, '{"invalid":')
        with self.assertRaises(ValueError):
            baseline.inspect_trace("\n".join(malformed_middle), 0)

    def test_installed_server_identity_is_exact(self):
        identity_path = self.capture / "installation.json"
        identity = json.loads(identity_path.read_text())
        identity["mcp_server"] = "installed_qiongli"
        probe.write_json(identity_path, identity)
        for server, expected in (("installed_qiongli", True), ("qiongli", False), ("other", False)):
            with self.subTest(server=server):
                events = trace()
                for event in events:
                    if event.get("item", {}).get("type") == "mcp_tool_call":
                        event["item"]["server"] = server
                self.set_events("saved-continuation", events)
                self.seal()
                result = self.score()
                self.assertEqual(result["cases"]["saved-continuation"]["behavior"]["required_native_calls"], expected)

    def test_guidance_requires_complete_installed_bytes_not_prose(self):
        path = self.capture / "paper-explanation/guidance.json"
        original = path.read_bytes()
        identity_path = self.capture / "installation.json"
        identity_original = identity_path.read_bytes()
        context = self.capture / "paper-explanation/host-context.txt"
        context_original = context.read_bytes()
        for mutation in ("missing", "wrong-version", "prose", "answer-as-context"):
            with self.subTest(mutation=mutation):
                path.write_bytes(original)
                identity_path.write_bytes(identity_original)
                context.write_bytes(context_original)
                if mutation == "missing":
                    path.unlink()
                elif mutation == "wrong-version":
                    identity = json.loads(identity_original)
                    identity["guidance_files"]["SKILL.md"] = "f" * 64
                    probe.write_json(identity_path, identity)
                elif mutation == "prose":
                    context.write_text("I read all Skills successfully.")
                else:
                    record = json.loads(original)
                    record["SKILL.md"]["context_artifact"] = "paper-explanation/answer.md"
                    probe.write_json(path, record)
                self.seal()
                result = self.score()
                self.assertFalse(result["cases"]["paper-explanation"]["behavior"]["guidance_evidence"])
                self.assertEqual(result["reviewed_passed"], 0 if mutation == "wrong-version" else 2)

    def test_guidance_command_output_requires_success_and_whole_body(self):
        record_path = self.capture / "paper-explanation/guidance.json"
        record = json.loads(record_path.read_text())
        body = (self.capture / record["SKILL.md"]["artifact_path"]).read_text()
        record["SKILL.md"] = {"mechanism": "command_output", "artifact_path": record["SKILL.md"]["artifact_path"],
                              "call_id": "guidance-read"}
        probe.write_json(record_path, record)
        for code, output, expected in ((0, body, True), (1, body, False), (0, body[:-5], False)):
            with self.subTest(code=code, output=output):
                events = trace()
                item = {"id": "guidance-read", "type": "command_execution", "command": "cat synthetic-SKILL.md"}
                events[2:2] = [{"type": "item.started", "item": {**item, "status": "in_progress"}},
                               {"type": "item.completed", "item": {**item, "status": "completed",
                                "exit_code": code, "aggregated_output": output}}]
                self.set_events("paper-explanation", events)
                self.seal()
                result = self.score()
                self.assertEqual(result["cases"]["paper-explanation"]["behavior"]["guidance_evidence"], expected)

    def test_full_unicode_length_bounds_and_registered_citation_exclusion(self):
        limit = json.loads((self.capture / "catalog.json").read_text())["cases"]["source-paragraph"]["length"]
        sources = (self.capture / "sources.csv").read_text()
        for count in (249, 250, 350, 351):
            with self.subTest(count=count):
                result = baseline.output_length("文" * count + " \n(Study:result)", limit, sources)
                self.assertEqual(result["count"], count)
                self.assertEqual(result["pass"], 250 <= count <= 350)
        result = baseline.output_length("文" * 250 + "(Study:unknown)", limit, sources)
        self.assertEqual(result["count"], 250 + len("(Study:unknown)"))
        for citation in ("[Study:result]", "（Study:result）"):
            self.assertEqual(baseline.output_length("文" * 250 + citation, limit, sources)["count"], 250)
        self.set_events("source-paragraph", trace())
        (self.capture / "source-paragraph/answer.md").write_text("短")
        events = trace()
        events[-2]["item"]["text"] = "短"
        self.set_events("source-paragraph", events)
        self.seal()
        result = self.score()
        self.assertFalse(result["cases"]["source-paragraph"]["behavior"]["output_length"])

    def test_freeze_seal_are_exclusive_and_reject_input_drift(self):
        fresh = self.root / "fresh"
        fresh.mkdir()
        names = {"catalog.json", "installation.json", "sources.csv", "inputs/source.md",
                 *(case + "/prompt.txt" for case in baseline.CASES)}
        for name in names:
            target = fresh / name
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes((self.capture / name).read_bytes())
        baseline.freeze_inputs(fresh)
        frozen = (fresh / "frozen-inputs.json").read_bytes()
        with self.assertRaises((ValueError, FileExistsError)):
            baseline.freeze_inputs(fresh)
        (fresh / "inputs/source.md").write_text("changed input")
        with self.assertRaises(ValueError):
            baseline.seal_capture(fresh)
        self.assertFalse((fresh / "manifest.json").exists())
        self.assertEqual((fresh / "frozen-inputs.json").read_bytes(), frozen)
        (fresh / "inputs/source.md").write_bytes((self.capture / "inputs/source.md").read_bytes())
        baseline.seal_capture(fresh)
        manifest = (fresh / "manifest.json").read_bytes()
        with self.assertRaises(ValueError):
            baseline.seal_capture(fresh)
        self.assertEqual((fresh / "manifest.json").read_bytes(), manifest)

    def test_legacy_catalog_remains_readable_without_rewriting_capture(self):
        path = self.capture / "catalog.json"
        catalog = json.loads(path.read_text())
        for spec in catalog["cases"].values():
            spec.pop("guidance_paths", None)
            spec.pop("length", None)
        probe.write_json(path, catalog)
        self.seal()
        before = {p: p.read_bytes() for p in (path, self.capture / "manifest.json")}
        baseline.load_capture(self.capture)
        self.assertEqual(self.score()["reviewed_passed"], 3)
        self.assertEqual(before, {p: p.read_bytes() for p in before})


if __name__ == "__main__":
    unittest.main()
