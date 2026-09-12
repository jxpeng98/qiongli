import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

from tooling.scripts.build_research_graph_example import FIXTURE, REPO, run_example, sha256


class ResearchGraphExampleTests(unittest.TestCase):
    def test_existing_destination_and_disabled_checks_refuse_before_writing(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            sentinel = root / "keep.txt"
            sentinel.write_text("keep this file")
            with patch("subprocess.run") as run:
                with self.assertRaises(FileExistsError):
                    run_example(sys.executable, root)
                run.assert_not_called()
            self.assertEqual([p.name for p in root.iterdir()], ["keep.txt"])
            self.assertEqual(sentinel.read_text(), "keep this file")
            destination = root / "disabled"
            result = subprocess.run([sys.executable, "-O", str(REPO / "tooling/scripts/build_research_graph_example.py"),
                                     "--cli", sys.executable, "--destination", str(destination)],
                                    capture_output=True, text=True, check=False)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("without -O", result.stderr)
            self.assertFalse(destination.exists())

    def test_public_example_matches_inputs_template_snapshot_and_source_response(self):
        public = REPO / "docs/public/demos"
        html_path = public / "research-graph.html"
        html = html_path.read_text(encoding="utf-8")
        embedded = html.split('<script id="graph-data" type="application/json">', 1)[1].split("</script>", 1)[0]
        template = (REPO / "packages/qiongli-native/apps/qiongli/src/graph_view.html").read_text(encoding="utf-8")
        self.assertEqual(html, template.replace("__QIONGLI_GRAPH_DATA__", embedded))
        snapshot = json.loads((public / "research-graph.snapshot.json").read_text())
        proof = json.loads((public / "research-graph.proof.json").read_text())
        source = json.loads((public / "research-graph.source.json").read_text())["artifact"]
        self.assertEqual(json.loads(embedded), {key: snapshot[key] for key in ("snapshot", "readiness")})
        self.assertEqual(proof["htmlSha256"], sha256(html_path))
        self.assertTrue(proof["synthetic"])
        self.assertEqual(proof["inputSha256"], {p.relative_to(FIXTURE).as_posix(): sha256(p)
                                               for p in FIXTURE.rglob("*") if p.is_file()})
        self.assertEqual(proof["semanticRecords"], 9)
        self.assertEqual(proof["semanticRelations"], 8)
        self.assertEqual(source["projectionId"], snapshot["snapshot"]["projectionId"])
        self.assertEqual(source["anchorLine"], 2)
        self.assertEqual(source["contentDigest"], hashlib.sha256(source["content"].encode()).hexdigest())
        self.assertEqual(source["content"], (FIXTURE / source["artifactPath"]).read_text())
        self.assertIn(source["entityId"], {e["edgeId"] for e in snapshot["snapshot"]["edges"]})


if __name__ == "__main__":
    unittest.main()
