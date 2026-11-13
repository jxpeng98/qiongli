import copy
import json
from pathlib import Path
import shutil
import tempfile
import unittest

from evals.research_journey import comparison
from evals.research_journey.writing_observation import bind_files, recover_packet


class ResearchComparisonTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.materials = self.root / "inputs"
        shutil.copytree(comparison.MATERIALS, self.materials)

    def change_protocol(self, mutate):
        path = self.materials / "protocol.json"
        protocol = json.loads(path.read_text())
        mutate(protocol)
        path.write_text(json.dumps(protocol))
        manifest_path = self.materials / "fixture-manifest.json"
        manifest = json.loads(manifest_path.read_text())
        manifest["files"]["protocol.json"] = bind_files({"protocol.json": path})["protocol.json"]
        manifest_path.write_text(json.dumps(manifest))

    def test_prepare_binds_equal_arm_inputs_and_dependent_sessions_without_running(self):
        receipt = comparison.prepare(self.root / "prepared", 20261113, self.materials)
        recovered = recover_packet(receipt["packet"], receipt["sha256"])
        self.assertEqual(recovered["status"], "bindings_current")
        order = json.loads((self.root / "prepared/schedule.json").read_text())
        readiness = json.loads((self.root / "prepared/preparation.json").read_text())
        self.assertEqual((order["planned_units"], order["planned_sessions"]), (6, 9))
        self.assertEqual(receipt["model_calls_executed"], 0)
        self.assertIsNone(readiness["runtime_identity"])
        self.assertFalse(readiness["runtime_policy_verified"])
        self.assertFalse(readiness["main_cohort_ready"])
        self.assertEqual({(u["task_id"], u["arm"]) for u in order["units"]},
                         {(task, arm) for task in ["reading", "continuity"] for arm in "ABC"})
        for task, checkpoints in [("reading", ["initial"]), ("continuity", ["initial", "followup"])]:
            units = [u for u in order["units"] if u["task_id"] == task]
            self.assertEqual(units[0]["checkpoints"], units[1]["checkpoints"])
            self.assertEqual(units[1]["checkpoints"], units[2]["checkpoints"])
            for unit in units:
                self.assertEqual(unit["status"], "not-started")
                self.assertEqual([c["id"] for c in unit["checkpoints"]], checkpoints)
                for checkpoint in unit["checkpoints"]:
                    self.assertEqual(checkpoint["status"], "not-started")
                    for name in checkpoint["material_files"]:
                        self.assertEqual((self.materials / name).read_bytes(),
                                         (self.root / "prepared/materials" / name).read_bytes())
        protocol, _ = comparison.check_materials(self.materials)
        self.assertEqual(order, comparison.schedule(protocol, 20261113))

    def test_prepared_packet_detects_changed_source_schedule_and_missing_review(self):
        receipt = comparison.prepare(self.root / "prepared", 1, self.materials)
        for name in ["materials/calibration/reading/initial/sources.md", "schedule.json",
                     "materials/review/continuity.md"]:
            path = self.root / "prepared" / name
            original = path.read_bytes()
            with self.subTest(name=name):
                path.write_bytes(original + b" changed")
                with self.assertRaises(ValueError):
                    recover_packet(receipt["packet"], receipt["sha256"])
                path.unlink()
                with self.assertRaises(ValueError):
                    recover_packet(receipt["packet"], receipt["sha256"])
                path.write_bytes(original)

    def test_changed_or_missing_input_fails_before_creating_output(self):
        path = self.materials / "calibration/reading/initial/sources.md"
        path.write_text("changed source")
        for kind in ["changed", "missing"]:
            with self.subTest(kind=kind):
                if kind == "missing":
                    path.unlink()
                with self.assertRaises(ValueError):
                    comparison.prepare(self.root / "prepared", 1, self.materials)
                self.assertFalse((self.root / "prepared").exists())

    def test_review_and_future_checkpoint_cannot_become_initial_inputs(self):
        initial = json.loads((self.materials / "protocol.json").read_text())
        for name in ["review/reading.md", "coordinator/continuity-save.md",
                     "calibration/continuity/followup/correction.md"]:
            with self.subTest(name=name):
                protocol = copy.deepcopy(initial)
                protocol["calibration_cases"][0]["checkpoints"][0]["files"].append(name)
                self.change_protocol(lambda p: p.update(protocol))
                with self.assertRaisesRegex(ValueError, "own checkpoint"):
                    comparison.check_materials(self.materials)

    def test_duplicate_missing_or_reordered_selection_is_rejected(self):
        initial = json.loads((self.materials / "protocol.json").read_text())
        mutations = [
            lambda p: p["shared_files"].append("README.md"),
            lambda p: p["shared_files"].remove("README.md"),
            lambda p: p["calibration_cases"][1]["checkpoints"].reverse(),
        ]
        for mutate in mutations:
            protocol = copy.deepcopy(initial)
            mutate(protocol)
            self.change_protocol(lambda p: p.update(protocol))
            with self.assertRaises(ValueError):
                comparison.check_materials(self.materials)

    def test_paths_and_source_symlinks_cannot_escape_selection(self):
        for name in ["../outside", "/absolute", "a/../b", "a//b", "a\\b", "./file", ""]:
            with self.subTest(name=name), self.assertRaises(ValueError):
                comparison.selected_path(self.materials, name)
        path = self.materials / "calibration/reading/initial/sources.md"
        path.unlink()
        path.symlink_to(self.materials / "README.md")
        with self.assertRaises(ValueError):
            comparison.check_materials(self.materials)

    def test_destination_and_invalid_seed_never_overwrite_or_start_preparation(self):
        existing = self.root / "existing"
        existing.mkdir()
        sentinel = existing / "keep"
        sentinel.write_text("unrelated")
        link = self.root / "link"
        link.symlink_to(existing, target_is_directory=True)
        for destination in [existing, link, link / "child", Path("relative"),
                            comparison.ROOT / "unwanted-preparation"]:
            with self.subTest(destination=destination), self.assertRaises(ValueError):
                comparison.prepare(destination, 1, self.materials)
        self.assertEqual(sentinel.read_text(), "unrelated")
        for seed in [-1, 2**32, True, "1"]:
            with self.subTest(seed=seed), self.assertRaises(ValueError):
                comparison.prepare(self.root / "prepared", seed, self.materials)
        self.assertFalse((self.root / "prepared").exists())


if __name__ == "__main__":
    unittest.main()
