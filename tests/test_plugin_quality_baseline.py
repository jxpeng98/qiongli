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


ANSWER = "The supplied ten-person study reports an association; causality and wider applicability remain uncertain."


def trace(*, native=True, usage=True):
    events = [{"type": "thread.started"}, {"type": "turn.started"}]
    if native:
        for index, tool in enumerate(("qiongli_project_read", "qiongli_project_document_list", "qiongli_project_document_read")):
            item = {"type": "mcp_tool_call", "id": str(index), "server": "qiongli", "tool": tool,
                    "arguments": {"project_id": "synthetic"}}
            events.extend([
                {"type": "item.started", "item": {**item, "status": "in_progress"}},
                {"type": "item.completed", "item": {**item, "status": "completed", "error": None,
                                                     "result": {"content": []}}},
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


if __name__ == "__main__":
    unittest.main()
