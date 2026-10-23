import hashlib
import json
from pathlib import Path
import tempfile
import unittest

from evals.research_journey.writing_observation import (
    antigravity_answer, bind_files, count_text, freeze_packet, measure_document,
    read_chunk, recover_packet, verify_files,
)


ANCHOR = r"〔Source2024 pp?\.[^〕]+〕"


class WritingObservationTests(unittest.TestCase):
    def test_packet_recovers_exact_inputs_and_rejects_stale_versions(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            files = {name: root/name for name in ['original', 'candidate', 'source', 'task']}
            for name, path in files.items():path.write_text(name)
            receipt = freeze_packet(files, root/'packet.json')
            result = recover_packet(receipt['packet'], receipt['sha256'])
            self.assertEqual(result['status'], 'bindings_current')
            self.assertEqual(set(result['files']), set(files))
            for name, path in files.items():
                path.write_text('new version')
                with self.subTest(name=name), self.assertRaises(ValueError):
                    recover_packet(receipt['packet'], receipt['sha256'])
                path.write_text(name)
            files['candidate'].unlink()
            with self.assertRaises(ValueError):recover_packet(receipt['packet'], receipt['sha256'])

    def test_packet_refuses_overwrite_tampering_and_symlink_redirection(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp);original = root/'original';original.write_text('valid')
            packet = root/'packet.json';receipt = freeze_packet({'original': original}, packet)
            with self.assertRaises(FileExistsError):freeze_packet({'original': original}, packet)
            with self.assertRaises(FileExistsError):freeze_packet({'original': original}, original)
            packet.write_text(packet.read_text()+' ')
            with self.assertRaises(ValueError):recover_packet(packet, receipt['sha256'])
            original.unlink();original.symlink_to(packet)
            with self.assertRaises(ValueError):freeze_packet({'original': original}, root/'other.json')
            (root/'link').symlink_to(root, target_is_directory=True)
            with self.assertRaises(ValueError):freeze_packet({'source': packet}, root/'link/new.json')

    def test_packet_refuses_invalid_schema_and_relative_paths(self):
        with tempfile.TemporaryDirectory() as temp:
            path = Path(temp)/'packet.json'
            for value in [[], {}, {'kind': 'other', 'paths': {}, 'binding': {}},
                          {'kind': 'qiongli-writing-packet/v1', 'paths': {'original': 'relative'}, 'binding': {}},
                          {'kind': 'qiongli-writing-packet/v1', 'paths': {}, 'binding': {}}]:
                path.write_text(json.dumps(value))
                with self.subTest(value=value), self.assertRaises(ValueError):
                    recover_packet(path, hashlib.sha256(path.read_bytes()).hexdigest())

    def agy_events(self):
        return [{"event": "init", "conversation_id": "turn-1", "init": {"cwd": "/public/eval"}},
                {"event": "result", "result": {"conversation_id": "turn-1", "status": "SUCCESS",
                 "response": "Polished paragraph.", "num_turns": 1}}]

    def test_text_handoff_needs_no_mcp_call_and_keeps_unknown_model(self):
        answer, metrics = antigravity_answer("\n".join(map(json.dumps, self.agy_events())), "/public/eval")
        self.assertEqual(answer, "Polished paragraph.")
        self.assertTrue(all(metrics["checks"].values()))
        self.assertEqual(metrics["calls"], [])
        self.assertEqual(metrics["reported_model_fields"], [])

    def test_text_handoff_rejects_missing_foreign_failed_and_denied_results(self):
        for kind in ["missing", "foreign", "failure", "denied", "wrong-workspace", "tail"]:
            events = self.agy_events()
            if kind == "missing":events.pop()
            if kind == "foreign":events[-1]["result"]["conversation_id"] = "other"
            if kind == "failure":events[-1]["result"]["status"] = "ERROR"
            if kind == "denied":events[-1]["result"]["denied_actions"] = ["tool"]
            if kind == "wrong-workspace":events[0]["init"]["cwd"] = "/elsewhere"
            raw = "\n".join(map(json.dumps, events)) + ('\n{"event":' if kind == "tail" else '')
            with self.subTest(kind=kind):
                self.assertIsNone(antigravity_answer(raw, "/public/eval")[0])

    def test_model_identity_comes_from_host_fields_only(self):
        events = self.agy_events();events[-1]["result"]["response"] = "My model is imagined."
        events[0]["init"]["model_id"] = "host-reported-model"
        _, metrics = antigravity_answer("\n".join(map(json.dumps, events)), "/public/eval")
        self.assertEqual(metrics["reported_model_fields"], [{"event": "init", "field": "model_id", "value": "host-reported-model"}])

    def structured_events(self):
        events = self.agy_events()
        schema = {"type": "object", "properties": {"text": {"type": "string"}}, "required": ["text"]}
        payload = {"text": "Polished paragraph."}
        events[0]["init"]["json_schema"] = schema
        events[-1]["result"].update(json_schema=schema, structured_output=payload, response=json.dumps(payload))
        events[1:1] = [
            {"event": "step_update", "step_update": {"conversation_id": "turn-1", "step_index": 2,
             "state": "ACTIVE", "step_type": "tool", "tool_name": "finish",
             "tool_info": {"name": "finish", "parameters": payload}}},
            {"event": "step_update", "step_update": {"conversation_id": "turn-1", "step_index": 2,
             "state": "DONE", "step_type": "finish"}}]
        return events

    def test_structured_finish_is_bound_without_fabricating_an_mcp_call(self):
        raw = "\n".join(map(json.dumps, self.structured_events()))
        answer, metrics = antigravity_answer(raw, "/public/eval")
        self.assertEqual(json.loads(answer), {"text": "Polished paragraph."})
        self.assertEqual(metrics["calls"], [])
        self.assertEqual(len(metrics["structured_finishes"]), 1)
        self.assertEqual(metrics["pending_steps"], [])

    def test_structured_finish_refuses_missing_or_mismatched_witnesses(self):
        for kind in ["no-start", "no-finish", "foreign", "schema", "payload", "response", "no-result"]:
            events = self.structured_events()
            if kind == "no-start":events.pop(1)
            if kind == "no-finish":events.pop(2)
            if kind == "foreign":events[2]["step_update"]["conversation_id"] = "other"
            if kind == "schema":events[-1]["result"]["json_schema"] = {}
            if kind == "payload":events[-1]["result"]["structured_output"] = {"text": "different"}
            if kind == "response":events[-1]["result"]["response"] = '{"text":"different"}'
            if kind == "no-result":events.pop()
            with self.subTest(kind=kind):
                answer, metrics = antigravity_answer("\n".join(map(json.dumps, events)), "/public/eval")
                self.assertIsNone(answer)

    def test_structured_finish_accepts_bound_clipped_preview_but_uses_full_terminal(self):
        events = self.structured_events()
        payload = {"text": "中" * 600}
        events[1]["step_update"]["tool_info"]["parameters"] = {"text": "中" * 512 + "…"}
        events[-1]["result"].update(structured_output=payload, response=json.dumps(payload))
        answer, metrics = antigravity_answer("\n".join(map(json.dumps, events)), "/public/eval")
        self.assertEqual(json.loads(answer), payload)
        self.assertFalse(metrics["structured_finishes"][0]["parameters_are_complete"])
        for preview in ["中" * 511 + "…", "a" + "中" * 511 + "…", "中" * 512 + "x"]:
            events[1]["step_update"]["tool_info"]["parameters"] = {"text": preview}
            with self.subTest(preview=preview):
                self.assertIsNone(antigravity_answer("\n".join(map(json.dumps, events)), "/public/eval")[0])

    def test_standalone_and_inline_are_different_metrics(self):
        text = "A claim. 〔Source2024 p.3〕\n\n〔Source2024 p.4〕 〔Source2024 p.5〕"
        self.assertEqual(count_text(text, "words", ANCHOR, "standalone"), 4)
        self.assertEqual(count_text(text, "words", ANCHOR, "all"), 2)
        self.assertEqual(count_text(text, "words", ANCHOR, "none"), 8)

    def test_codepoints_keep_punctuation_digits_and_unknown_anchors(self):
        text = "甲乙，12。\n〔Other p.2〕"
        self.assertEqual(count_text(text, "codepoints", ANCHOR, "standalone"), 16)
        self.assertEqual(count_text("甲🙂 e\u0301", "codepoints", ANCHOR, "none"), 4)

    def test_invalid_counter_contract_fails(self):
        for args in [("x", "tokens", ANCHOR, "all"), ("x", "words", ANCHOR, "guess"),
                     ("x", "words", ".*", "all")]:
            with self.subTest(args=args), self.assertRaises(ValueError):
                count_text(*args)

    def contract(self):
        return {"anchor_pattern": ANCHOR, "anchor_exclusion": "standalone", "languages": [
            {"heading": "## 中文", "sections": ["### 摘要", "### 讨论"], "unit": "codepoints", "min": 4, "max": 4},
            {"heading": "## English", "sections": ["### Abstract", "### Discussion"], "unit": "words", "min": 4, "max": 4}]}

    def answer(self):
        return "## 中文\n### 摘要\n甲乙\n### 讨论\n丙丁\n## English\n### Abstract\nA claim.\n### Discussion\nIts limit.\n"

    def test_section_counts_exclude_headings_and_keep_both_boundaries(self):
        result = measure_document(self.answer(), self.contract())
        self.assertTrue(result["pass"])
        self.assertEqual(result["languages"]["## 中文"]["count"], 4)
        self.assertEqual(result["languages"]["## English"]["sections"]["### Abstract"], 2)

    def test_out_of_range_is_not_a_pass(self):
        for addition in [" Extra.", ""]:
            text = self.answer().replace("Its limit.", "Its" + addition)
            result = measure_document(text, self.contract())
            self.assertEqual(result["pass"], bool(addition))
        self.assertFalse(measure_document(self.answer()+" Extra words.", self.contract())["pass"])

    def test_missing_duplicate_extra_empty_or_unassigned_sections_fail(self):
        answer = self.answer()
        changes = [answer.replace("### 讨论", "### 摘要"), answer.replace("丙丁", ""),
                   answer.replace("### 讨论\n丙丁\n", ""), answer+"### Other\nx",
                   "Preface\n"+answer, answer.replace("## 中文\n", "## 中文\nPreface\n")]
        for text in changes:
            with self.subTest(text=text), self.assertRaises(ValueError):
                measure_document(text, self.contract())

    def test_contract_rejects_duplicate_headings_and_boolean_bounds(self):
        for mutation in ["duplicate", "bool", "extra"]:
            contract = self.contract()
            if mutation == "duplicate":contract["languages"][1]["sections"][0] = "### 摘要"
            if mutation == "bool":contract["languages"][0]["min"] = True
            if mutation == "extra":contract["silent_default"] = True
            with self.subTest(mutation=mutation), self.assertRaises(ValueError):
                measure_document(self.answer(), contract)

    def test_handoff_rejects_changed_original_candidate_or_selection(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            files = {name: root/name for name in ["original", "candidate", "task"]}
            for name, path in files.items():path.write_text(name)
            frozen = bind_files(files)
            verify_files(files, frozen)
            for name in files:
                files[name].write_text("changed")
                with self.subTest(name=name), self.assertRaises(ValueError):verify_files(files, frozen)
                files[name].write_text(name)
            with self.assertRaises(ValueError):verify_files({"original": files["original"]}, frozen)
            files["candidate"].unlink()
            with self.assertRaises(ValueError):verify_files(files, frozen)

    def test_selected_symlinks_and_empty_snapshots_fail(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp);(root/"real").write_text("data");(root/"link").symlink_to(root/"real")
            with self.assertRaises(ValueError):bind_files({"candidate": root/"link"})
        with self.assertRaises(ValueError):bind_files({})

    def test_bounded_chunks_reassemble_unicode_and_long_lines_exactly(self):
        text = '中文🙂"\\\n' * 400 + "a" * 3000
        with tempfile.TemporaryDirectory() as temp:
            path = Path(temp)/"source";path.write_text(text);digest = hashlib.sha256(path.read_bytes()).hexdigest()
            offset, pieces = 0, []
            while offset is not None:
                chunk = read_chunk(path, digest, offset, 700)
                self.assertLessEqual(len(json.dumps(chunk, ensure_ascii=False).encode()), 700)
                self.assertEqual(chunk["start_offset"], offset)
                self.assertGreater(chunk["end_offset"], offset)
                pieces.append(chunk["text"]);offset = chunk["next_offset"]
            self.assertEqual("".join(pieces), text)
            self.assertTrue(chunk["complete"])
            self.assertEqual(read_chunk(path, digest, len(text))["text"], "")

    def test_chunk_refuses_stale_hash_bad_offset_and_tiny_budget(self):
        with tempfile.TemporaryDirectory() as temp:
            path = Path(temp)/"source";path.write_text("abc");digest = hashlib.sha256(path.read_bytes()).hexdigest()
            for kwargs in [{"expected_sha256": "0"*64}, {"offset": -1}, {"offset": 4},
                           {"offset": True}, {"max_bytes": 5}]:
                args = {"expected_sha256": digest, **kwargs}
                with self.subTest(kwargs=kwargs), self.assertRaises(ValueError):read_chunk(path, **args)


if __name__ == "__main__":
    unittest.main()
