from contextlib import redirect_stdout
import csv
import io
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

from evals.research_journey import observe
from evals.skill_routing import probe, resource_reader


FINDING = "In this synthetic survey of 120 students, study time correlated with scores (r = .32)."
LIMIT = "Causality cannot be inferred from this abstract; no CI, p-value or adjusted analysis was supplied."


def continuity_answer(stage):
    finding = FINDING if stage == "C" else FINDING.replace("120", "118").replace(".32", ".23")
    return "\n".join([
        f"# STG-{stage}-001 — synthetic {'R1' if stage == 'C' else 'R2'} candidate",
        "## Stage Outcome", "Partial exercise only; D/E collection, ethics and analysis remain unperformed.",
        "## Retained Research Content", "Finding: C1 " + finding + " SyntheticStudy2026:abstract-result",
        "Limits: C2 " + LIMIT + " SyntheticStudy2026:abstract-limit",
        "## Source Coverage", "source.md and the supplied context files only; no full text.",
        "## Changes Since Previous Summary",
        ("STG-B-001 precedes STG-C-001; retain originals. R1 source remains current." if stage == "C" else
         "STG-B-001 precedes STG-C-001; retain originals. R2 corrects 120/.32 to 118/.23, not a new study."),
        "## Decisions and Open Questions", "DEC-001 and DEC-002 unchanged; C3 pending and unused.",
        "## Next-Stage Handoff", "Revisit on changed source bytes; prior summary is not new evidence or approval.",
    ])


class ResearchJourneyObservationTests(unittest.TestCase):
    def capture(self, root, *, bad_answer=False, timeout=False, answers=None):
        output = root / "capture"
        (root / "config.toml").write_text('model="configured-test-model"\nmodel_reasoning_effort="high"\n')
        calls = []

        def codex(cmd, **kwargs):
            if cmd == ["codex", "--version"]:
                return subprocess.CompletedProcess(cmd, 0, "codex-cli synthetic", "")
            calls.append((cmd, kwargs))
            if timeout:
                raise subprocess.TimeoutExpired(cmd, kwargs["timeout"], output=b'{"type":"thread.started"}\n', stderr=b"partial error")
            finding = "Study time causes higher scores." if bad_answer else FINDING
            answer = "\n".join([finding, LIMIT] * (2 if len(calls) == 1 else 1))
            if answers is not None:
                answer = answers[len(calls) - 1]
            resources = json.loads((output / "resources.json").read_text())
            item = {"id": "read-1", "type": "mcp_tool_call", "server": resource_reader.SERVER,
                    "tool": resource_reader.TOOL, "arguments": {"path": "workflows/paper-read.md"}}
            events = [{"type": "thread.started"}, {"type": "turn.started"},
                {"type": "item.started", "item": {**item, "status": "in_progress"}},
                {"type": "item.completed", "item": {**item, "status": "completed", "error": None,
                    "result": resource_reader.read_result(resources, item["arguments"])}},
                {"type": "item.completed", "item": {"type": "agent_message", "text": answer}},
                {"type": "turn.completed"}]
            return subprocess.CompletedProcess(cmd, 0, "\n".join(map(json.dumps, events)), "")

        with patch.dict(os.environ, {"CODEX_HOME": str(root)}), patch.object(
            probe.subprocess, "run", side_effect=codex
        ), redirect_stdout(io.StringIO()):
            self.assertEqual(not timeout, observe.capture(output, continuity=answers is not None))
        review_path = root / "review.json"
        observe.prepare(output, review_path)
        return output, review_path, calls

    def review(self, review_path, *, bad_answer=False):
        review = json.loads(review_path.read_text())
        review["reviewer"] = {"kind": "model", "id": "synthetic-test-reviewer"}
        for case_id, case in review["cases"].items():
            answer = case["segments"][0]["quote"]
            segments, cursor = [], 0
            for i, quote in enumerate(answer.splitlines()):
                role = "reading" if case_id == observe.CASES[0] and i < 2 else "manuscript"
                fails = bad_answer and i % 2 == 0
                segments.append({"start": cursor, "end": cursor + len(quote), "quote": quote,
                    "role": role, "verdict": "fail" if fails else "pass",
                    "reason": "Causal overclaim." if fails else "Matches supplied abstract.",
                    "links": [{"claim_id": "C1" if i % 2 == 0 else "C2",
                               "source_location": "SyntheticStudy2026:abstract-" + ("result" if i % 2 == 0 else "limit"),
                               "status": "supported", "claim_type": "finding" if i % 2 == 0 else "limitation"}]})
                cursor += len(quote) + 1
            case["segments"] = segments
            case["checks"] = {key: {"status": "fail" if bad_answer and key == "causality" else "pass",
                "segments": list(range(len(segments))), "reason": "Synthetic reviewer judgment against supplied abstract."}
                for key in observe.CHECKS}
        probe.write_json(review_path, review)
        return review

    def score(self, output, review_path, report):
        with redirect_stdout(io.StringIO()):
            passed = observe.score(output, review_path, report)
        summary = json.loads((report / "summary.json").read_text())
        self.assertEqual(passed, summary["reviewed_passed"] == 2)
        self.assertEqual(2, summary["case_count"])
        return summary

    def continuity_review(self, review_path):
        review = json.loads(review_path.read_text())
        review["reviewer"] = {"kind": "model", "id": "synthetic-test-reviewer"}
        for case_id, case in review["cases"].items():
            answer, cursor = case["segments"][0]["quote"], 0
            case["segments"] = []
            for line in answer.splitlines():
                finding, limitation = line.startswith("Finding:"), line.startswith("Limits:")
                linked = finding or limitation
                case["segments"].append({"start": cursor, "end": cursor + len(line), "quote": line,
                    "role": ("summary" if case_id == observe.CONTINUITY_CASES[0] else "manuscript") if linked else "context",
                    "verdict": "pass" if linked else "not-evidence", "reason": "Synthetic test annotation.",
                    "links": [{"claim_id": "C1" if finding else "C2",
                        "source_location": "SyntheticStudy2026:abstract-" + ("result" if finding else "limit"),
                        "status": "supported", "claim_type": "finding" if finding else "limitation"}] if linked else []})
                cursor += len(line) + 1
            case["checks"] = {key: {"status": "pass", "segments": list(range(len(case["segments"]))),
                "reason": "Synthetic judgment against current source/state/handoff."} for key in observe.CONTINUITY_CHECKS}
        probe.write_json(review_path, review)
        return review

    def test_continuity_supplies_actual_predecessor_and_current_revision(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            answers = [continuity_answer("C") + "\nCaptured-only marker.", continuity_answer("F")]
            output, review_path, calls = self.capture(root, answers=answers)
            manifest, files = observe.captured_files(output)
            self.assertEqual(observe.CONTINUITY_KIND, manifest["kind"])
            self.assertEqual(list(observe.CONTINUITY_CASES), manifest["cases"])
            self.assertNotIn("synthetic continuity fixture R2", calls[0][1]["input"])
            self.assertNotIn("Captured-only marker.", calls[0][1]["input"])
            self.assertIn(answers[0], calls[1][1]["input"])
            self.assertIn("118", observe.research_inputs(files, observe.CONTINUITY_CASES[1])["source.md"])
            for _, args in calls:
                self.assertNotIn("expected_outputs", args["input"])
                self.assertNotIn("manuscript.csv", args["input"])
                self.assertEqual(observe.CONTINUITY_TIMEOUT_SECONDS, args["timeout"])
            self.continuity_review(review_path)
            summary = self.score(output, review_path, root / "reviewed")
            self.assertEqual(2, summary["reviewed_passed"])
            parent = summary["cases"][observe.CONTINUITY_CASES[0]]
            self.assertEqual(360, parent["timeout_seconds"])
            self.assertEqual({key: parent[key] for key in ("answer_sha256", "events_sha256")},
                             summary["cases"][observe.CONTINUITY_CASES[1]]["predecessor"])
            self.assertEqual(answers[0], (root / "reviewed/outputs/f-stage-continuation" / observe.PREDECESSOR).read_text())
            for case_id in observe.CONTINUITY_CASES:
                target = root / "reviewed/outputs" / case_id
                self.assertFalse((target / "reading.csv").exists())
                receipt = json.loads((root / "reviewed/receipts" / f"{case_id}.json").read_text())
                self.assertEqual(19, receipt["summary"]["executed_assertions"])

    def test_continuation_cannot_reuse_a_changed_or_missing_predecessor(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            output, review_path, _ = self.capture(root, answers=[continuity_answer("C"), continuity_answer("F")])
            self.continuity_review(review_path)
            _, files = observe.captured_files(output)
            parent = output / observe.CONTINUITY_CASES[0]
            original = (parent / "answer.md").read_text()
            (parent / "answer.md").unlink()
            self.assertEqual(0, self.score(output, review_path, root / "missing-parent")["reviewed_passed"])
            (parent / "answer.md").write_text(original)
            # Even a locally rehashed valid replacement of C cannot match F's captured input.
            events = list(map(json.loads, (parent / "events.jsonl").read_text().splitlines()))
            replacement = original + "\nDifferent prior candidate."
            events[-2]["item"]["text"] = replacement
            (parent / "events.jsonl").write_text("\n".join(map(json.dumps, events)))
            (parent / "answer.md").write_text(replacement)
            receipt = json.loads((parent / "capture.json").read_text())
            probe.write_json(parent / "capture.json", {**receipt, "answer_sha256": probe.sha(replacement),
                "events_sha256": probe.digest(parent / "events.jsonl")})
            self.assertEqual(replacement, observe.observed_answer(output, files, observe.CONTINUITY_CASES[0])[0])
            with self.assertRaisesRegex(ValueError, "Changed prompt"):
                observe.observed_answer(output, files, observe.CONTINUITY_CASES[1])

    def test_continuity_rejects_changed_state_sources_selection_and_old_rubric(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            output, review_path, _ = self.capture(root, answers=[continuity_answer("C"), continuity_answer("F")])
            review = self.continuity_review(review_path)
            for name in ("inputs/revision-2/source.md", "inputs/context/decision_log.md",
                         "inputs/revision-2/context/stage_handoff.md"):
                path = output / name
                original = path.read_text()
                path.write_text(original + "\nChanged revision.")
                with self.assertRaisesRegex(ValueError, "snapshot changed"):
                    observe.score(output, review_path, root / "tampered")
                self.assertFalse((root / "tampered").exists())
                path.write_text(original)
            review["cases"][observe.CONTINUITY_CASES[1]]["checks"].pop("source_revision")
            probe.write_json(review_path, review)
            self.assertEqual(1, self.score(output, review_path, root / "old-rubric")["reviewed_passed"])
            manifest = json.loads((output / "manifest.json").read_text())
            manifest["cases"].reverse()
            probe.write_json(output / "manifest.json", manifest)
            with self.assertRaisesRegex(ValueError, "selection"):
                observe.captured_files(output)

    def test_stale_numbers_and_renamed_decisions_remain_visible_failures(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            stale = continuity_answer("F").replace("Finding: C1 " + FINDING.replace("120", "118").replace(".32", ".23"),
                                                   "Finding: C1 " + FINDING)
            output, review_path, _ = self.capture(root, answers=[continuity_answer("C"), stale])
            review = self.continuity_review(review_path)
            case = review["cases"][observe.CONTINUITY_CASES[1]]
            case["segments"][4].update(verdict="fail", reason="R1 numbers were reused as current despite the R2 correction.")
            case["checks"]["source_revision"] = {"status": "fail", "segments": [4], "reason": "Current source.md supplies 118/.23."}
            probe.write_json(review_path, review)
            summary = self.score(output, review_path, root / "stale-values")
            self.assertEqual((2, 1), (summary["structural_passed"], summary["reviewed_passed"]))
            nested = root / "renamed"
            nested.mkdir()
            output, review_path, _ = self.capture(nested, answers=[continuity_answer("C"), continuity_answer("F").replace("DEC-001", "DEC-999")])
            self.continuity_review(review_path)
            self.assertEqual(1, self.score(output, review_path, nested / "lost-id")["structural_passed"])

    def test_continuity_timeout_keeps_both_checkpoints_in_denominator(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            output, review_path, calls = self.capture(root, answers=[continuity_answer("C"), continuity_answer("F")], timeout=True)
            self.assertEqual(1, len(calls))
            self.assertFalse((output / observe.CONTINUITY_CASES[1]).exists())
            self.assertEqual(0, self.score(output, review_path, root / "failed")["reviewed_passed"])

    def test_capture_review_and_canonical_projection_use_actual_answer(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            output, review_path, calls = self.capture(root)
            template = self.score(output, review_path, root / "unreviewed")
            self.assertEqual(0, template["reviewed_passed"])
            review = self.review(review_path)
            before = {p: probe.digest(p) for p in output.rglob("*") if p.is_file()}
            # Regrade from frozen inputs without a new model call or current package reads.
            with patch.object(probe, "snapshot_resources", side_effect=AssertionError), patch.object(
                probe.subprocess, "run", side_effect=AssertionError
            ):
                summary = self.score(output, review_path, root / "reviewed")
            self.assertEqual((2, 2), (summary["structural_passed"], summary["reviewed_passed"]))
            self.assertEqual(before, {p: probe.digest(p) for p in before})
            self.assertEqual({"model": "configured-test-model", "model_reasoning_effort": "high"},
                             summary["configured_settings"])
            self.assertEqual(2, len(calls))
            for cmd, args in calls:
                self.assertNotIn("--output-schema", cmd)
                self.assertIn("--ignore-user-config", cmd)
                self.assertIn('model="configured-test-model"', cmd)
                self.assertNotIn("expected_outputs", args["input"])
                self.assertNotIn("manuscript.csv", args["input"])
                self.assertEqual(180, args["timeout"])
            for case_id, expected_assertions in zip(observe.CASES, (17, 14)):
                case = summary["cases"][case_id]
                self.assertEqual(["workflows/paper-read.md"], case["resource_reads"])
                self.assertEqual(["C1", "C2"], case["covered_claims"])
                projected = root / "reviewed/outputs" / case_id
                with (projected / "manuscript.csv").open() as handle:
                    rows = list(csv.DictReader(handle))
                self.assertEqual([FINDING, LIMIT], [row["passage"] for row in rows])
                self.assertEqual(case_id == observe.CASES[0], (projected / "reading.csv").exists())
                receipt = json.loads((root / "reviewed/receipts" / f"{case_id}.json").read_text())
                self.assertEqual(expected_assertions, receipt["summary"]["executed_assertions"])
            for destination in (output, output / "nested", root):
                with self.assertRaises(ValueError):
                    observe.prepare(output, destination)
            with self.assertRaises(ValueError):
                observe.score(output, review_path, root / "reviewed")
            review["reviewer"] = {"kind": "unassigned", "id": ""}
            probe.write_json(review_path, review)
            self.assertEqual(0, self.score(output, review_path, root / "no-reviewer")["reviewed_passed"])

    def test_valid_tuples_do_not_hide_causal_overclaim(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            output, review_path, _ = self.capture(root, bad_answer=True)
            review = self.review(review_path, bad_answer=True)
            # An absent reviewer cannot turn an explicit failure into an unreviewed case.
            review["reviewer"] = {"kind": "unassigned", "id": ""}
            probe.write_json(review_path, review)
            summary = self.score(output, review_path, root / "failed")
            self.assertEqual((2, 0), (summary["structural_passed"], summary["reviewed_passed"]))
            self.assertTrue(all(case["semantic"]["status"] == "fail" for case in summary["cases"].values()))
            self.assertIn("causes", (output / observe.CASES[0] / "answer.md").read_text())

    def test_splitting_a_bound_note_does_not_multiply_source_records(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            output, review_path, _ = self.capture(root)
            review = self.review(review_path)
            case = review["cases"][observe.CASES[0]]
            first = case["segments"][0]
            split = first["quote"].index(",") + 1
            case["segments"][:1] = [
                {**first, "end": split, "quote": first["quote"][:split]},
                {**first, "start": split, "quote": first["quote"][split:]},
            ]
            for check in case["checks"].values():
                check["segments"] = list(range(len(case["segments"])))
            probe.write_json(review_path, review)
            summary = self.score(output, review_path, root / "split-note")
            self.assertEqual(2, summary["reviewed_passed"])
            with (root / "split-note/outputs/reading-to-manuscript/reading.csv").open() as handle:
                rows = list(csv.DictReader(handle))
            self.assertEqual(2, len(rows))
            self.assertEqual(first["quote"][:split] + "\n" + first["quote"][split:], rows[0]["reading_note"])

    def test_span_coverage_and_stale_reviews_fail_closed(self):
        mutations = [
            lambda c: c.update(answer_sha256="stale"),
            lambda c: c.update(events_sha256="stale"),
            lambda c: c["segments"][0].update(quote="Passing fixture text"),
            lambda c: c["segments"][0].update(start=True),
            lambda c: c["segments"][0].update(start=1),
            lambda c: c["segments"][1].update(start=0),
            lambda c: c["segments"].pop(),
            lambda c: c["segments"][0]["links"][0].update(evidence_limit="full_text"),
            lambda c: c["segments"][0]["links"][0].update(source_location="Unknown:source"),
            lambda c: c["segments"][0]["links"].append(dict(c["segments"][0]["links"][0])),
            lambda c: c["checks"]["access"].update(segments=[]),
        ]
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            output, review_path, _ = self.capture(root)
            original = self.review(review_path)
            for i, mutate in enumerate(mutations):
                with self.subTest(mutation=i):
                    changed = json.loads(json.dumps(original))
                    mutate(changed["cases"][observe.CASES[0]])
                    probe.write_json(review_path, changed)
                    summary = self.score(output, review_path, root / f"invalid-{i}")
                    self.assertEqual(1, summary["reviewed_passed"])
                    self.assertNotEqual("pass", summary["cases"][observe.CASES[0]]["structural"])
            # Correct spans, but missing C2, must fail the original V1 requested-claim check.
            original["cases"][observe.CASES[1]]["segments"][1]["links"][0]["claim_id"] = "C99"
            probe.write_json(review_path, original)
            self.assertEqual(1, self.score(output, review_path, root / "missing-claim")["reviewed_passed"])

    def test_changed_capture_bytes_and_scope_never_pass(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            output, review_path, _ = self.capture(root)
            self.review(review_path)
            for name in ("answer.md", "events.jsonl", "capture.json"):
                path = output / observe.CASES[0] / name
                original = path.read_text()
                path.write_text("substituted")
                summary = self.score(output, review_path, root / name)
                self.assertEqual(1, summary["reviewed_passed"])
                path.write_text(original)
            # Text-mode universal-newline conversion must not hide changed bytes.
            for name in ("answer.md", "events.jsonl"):
                path = output / observe.CASES[0] / name
                original = path.read_bytes()
                path.write_bytes(original.replace(b"\n", b"\r\n"))
                self.assertEqual(1, self.score(output, review_path, root / f"crlf-{name}")["reviewed_passed"])
                path.write_bytes(original)
            for name in ("inputs/source.md", "resources.json", "manifest.json"):
                path = output / name
                original = path.read_text()
                if name == "manifest.json":
                    manifest = json.loads(original)
                    manifest["cases"].pop()
                    probe.write_json(path, manifest)
                else:
                    path.write_text("substituted")
                with self.assertRaises(ValueError):
                    observe.score(output, review_path, root / "invalid-snapshot")
                self.assertFalse((root / "invalid-snapshot").exists())
                path.write_text(original)

    def test_timeout_preserves_partial_trace_and_unattempted_denominator(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            output, review_path, calls = self.capture(root, timeout=True)
            self.assertEqual(1, len(calls))
            self.assertFalse((output / observe.CASES[1]).exists())
            self.assertIn("thread.started", (output / observe.CASES[0] / "events.jsonl").read_text())
            self.assertIn("partial error", (output / observe.CASES[0] / "stderr.log").read_text())
            summary = self.score(output, review_path, root / "failed")
            self.assertEqual((0, 0), (summary["structural_passed"], summary["reviewed_passed"]))


if __name__ == "__main__":
    unittest.main()
