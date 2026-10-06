#!/usr/bin/env python3
"""Offline corpus byte/anchor checks; never scores a research answer."""
import argparse
import csv
import hashlib
import io
import json
from pathlib import Path
import re
import shutil
import sys
import tempfile
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parent


def digest(data):
    return hashlib.sha256(data).hexdigest()


def check(root, raw_dir=None):
    manifest = json.loads((root / "manifest.json").read_text())
    assert manifest["schema"] == "qiongli-public-paper-trials/v1"
    assert len(manifest["cases"]) == 3
    ids, all_locators = set(), set()
    for case in manifest["cases"]:
        assert case["id"] not in ids
        ids.add(case["id"])
        assert re.fullmatch(r"[a-f0-9]{64}", case["raw_source_sha256"])
        article = None
        if raw_dir is not None:
            name = {"quantitative-education": "quant.xml",
                    "qualitative-education": "qual.xml",
                    "systematic-review-audit": "review.xml"}[case["id"]]
            raw = (raw_dir / name).read_bytes()
            assert digest(raw) == case["raw_source_sha256"], name
            article = ET.fromstring(raw)
            if article.tag != "article":
                article = article.find("article")
            assert article is not None
            assert case["doi"] in ["".join(e.itertext()) for e in
                                  article.findall("./front/article-meta/article-id")]
            license_node = article.find("./front/article-meta/permissions/license")
            assert license_node is not None
            assert " ".join("".join(license_node.itertext()).split()) == case["license_statement"]
        for packet in case["checkpoints"]:
            path = root / packet["file"]
            assert path.resolve().is_relative_to(root.resolve())
            data = path.read_bytes()
            assert digest(data) == packet["sha256"], packet["file"]
            text = data.decode("utf-8")
            assert f"DOI: https://doi.org/{case['doi']}\n" in text
            assert f"Citekey/source ID: `{case['citekey']}`." in text
            assert f"Raw source SHA-256: `{case['raw_source_sha256']}`." in text
            assert text.startswith(f"# {case['title']}\n")
            chunks = re.split(r"\n## ", text)[1:]
            assert len(chunks) == len(packet["segments"])
            for chunk, segment in zip(chunks, packet["segments"]):
                match = re.fullmatch(r"([^\n]+)\n\nXML selector \(relative to article\): `([^`]+)`\n\n(.*)\n", chunk, re.S)
                assert match is not None, packet["file"]
                locator, selector, paragraph = match.groups()
                paragraph = paragraph.rstrip("\n")
                assert locator == segment["locator"] and locator not in all_locators
                all_locators.add(locator)
                assert locator.startswith(case["citekey"] + ":")
                assert selector == segment["xml_selector"]
                assert digest(paragraph.encode()) == segment["normalized_text_sha256"]
                if article is not None:
                    node = article.find(selector)
                    assert node is not None, selector
                    assert " ".join("".join(node.itertext()).split()) == paragraph
        for record in case["tasks"]:
            path = root / record["file"]
            assert path.resolve().is_relative_to(root.resolve())
            assert digest(path.read_bytes()) == record["sha256"], record["file"]
    rubric = manifest["review_criteria"]
    assert rubric["supply_to_task_runner"] is False
    assert digest((root / rubric["file"]).read_bytes()) == rubric["sha256"]


def self_test():
    """Fail closed for a changed paragraph, swapped paper and forged anchor."""
    for mutation in ("changed", "swapped", "anchor"):
        with tempfile.TemporaryDirectory(prefix="qiongli-paper-integrity-") as tmp:
            root = Path(tmp) / "corpus"
            shutil.copytree(ROOT, root)
            source = root / "quantitative-education/source.md"
            if mutation == "changed":
                source.write_text(source.read_text().replace("109 pupils", "999 pupils", 1))
            elif mutation == "swapped":
                source.write_bytes((root / "qualitative-education/source.md").read_bytes())
            else:
                source.write_text(source.read_text().replace(":participants", ":invented", 1))
                path = root / "manifest.json"
                manifest = json.loads(path.read_text())
                manifest["cases"][0]["checkpoints"][0]["sha256"] = digest(source.read_bytes())
                path.write_text(json.dumps(manifest))
            try:
                check(root)
            except AssertionError:
                continue
            raise AssertionError(f"Negative mutation passed: {mutation}")


def check_frozen_spans(root, record, expected):
    """Replay frozen coordinator judgments; do not judge new research prose."""
    sys.path.insert(0, str(ROOT.parents[2]))
    from evals.research_journey.observe import project

    assert record["schema"] == "qiongli-public-paper-regressions/v1"
    assert record["reviewer"]["kind"] == "model"
    assert [c["id"] for c in record["cases"]] == list(expected)
    for name, sha in record["files"].items():
        path = root / name
        assert path.resolve().is_relative_to(root.resolve()) and not path.is_symlink()
        assert digest(path.read_bytes()) == sha, name
    for case in record["cases"]:
        assert case["answer_file"] in record["files"]
        assert case["source"]["artifact_path"] in record["files"]
        raw = (root / case["answer_file"]).read_bytes()
        assert digest(raw) == case["answer_sha256"]
        answer = raw.decode()[case["start"]:case["end"]]
        assert digest(answer.encode()) == case["review"]["answer_sha256"]
        registry = io.StringIO()
        writer = csv.DictWriter(registry, fieldnames=list(case["source"]))
        writer.writeheader()
        writer.writerow(case["source"])
        _, judgment = project(answer, case["review"], registry.getvalue(), rubric=(case["id"],))
        assert judgment["status"] == case["expected"] == expected[case["id"]]


def check_observations(root):
    record = json.loads((root / "regressions.json").read_text())
    check_frozen_spans(root, record, {
        "units": "fail", "denominators": "pass", "abstract_body": "pass",
        "source_revision": "pass", "review_attribution": "fail"})
    addendum = json.loads((root / "quantitative-education/tables-manifest.json").read_text())
    assert addendum["base_manifest_sha256"] == digest((root / "manifest.json").read_bytes())
    for item in addendum["files"]:
        assert digest((root / item["file"]).read_bytes()) == item["sha256"]
    receipt = json.loads((root / "quantitative-education/observations/claude-execution.json").read_text())
    assert receipt["actor_kind"] == "model"
    assert digest((root / receipt["review_file"]).read_bytes()) == receipt["review_sha256"]
    transition = json.loads((root / "discipline-transition/observations.json").read_text())
    assert transition["reviewer"]["relationship"] == "self-review"
    check_frozen_spans(root, transition, {"education_transition": "pass", "computing_transition": "pass"})


def observation_self_test():
    for mutation in ("answer", "denominator", "revision", "anchor", "attribution", "promoted",
                     "transition-source", "transition-answer", "transition-anchor", "transition-attribution"):
        with tempfile.TemporaryDirectory(prefix="qiongli-observation-integrity-") as tmp:
            root = Path(tmp) / "corpus"
            shutil.copytree(ROOT, root)
            path = root / ("discipline-transition/observations.json"
                           if mutation.startswith("transition-") else "regressions.json")
            record = json.loads(path.read_text())
            if mutation in ("answer", "denominator", "revision"):
                name = {"answer": record["cases"][0]["answer_file"],
                        "denominator": "quantitative-education/tables-source.md",
                        "revision": "quantitative-education/continuation-source.md"}[mutation]
                with (root / name).open("a") as file:
                    file.write("\nchanged bytes\n")
            elif mutation in ("transition-source", "transition-answer"):
                name = ("discipline-transition/computing-source.md" if mutation == "transition-source"
                        else "discipline-transition/answers.md")
                with (root / name).open("a") as file:
                    file.write("\nchanged bytes\n")
            elif mutation in ("anchor", "transition-anchor"):
                record["cases"][0]["review"]["segments"][0]["links"][0]["source_location"] = "forged:anchor"
            elif mutation == "attribution":
                record["reviewer"]["kind"] = "human"
            elif mutation == "transition-attribution":
                record["reviewer"]["relationship"] = "independent"
            else:
                case = record["cases"][0]
                case["review"]["segments"][0]["verdict"] = "pass"
                case["review"]["checks"]["units"]["status"] = "pass"
            path.write_text(json.dumps(record))
            try:
                check_observations(root)
            except (AssertionError, ValueError):
                continue
            raise AssertionError(f"Negative observation passed: {mutation}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--raw-dir", type=Path, help="Optional retrieved quant.xml, qual.xml, review.xml directory")
    parser.add_argument("--self-test", action="store_true", help="Also reject three isolated negative mutations")
    parser.add_argument("--observations", action="store_true", help="Replay frozen selected-span judgments, not new answers")
    args = parser.parse_args()
    check(ROOT, args.raw_dir)
    if args.self_test:
        self_test()
    if args.observations:
        check_observations(ROOT)
        if args.self_test:
            observation_self_test()
        print("Five frozen span judgments reproduced (3 supported, 2 failed); no whole-answer or Host acceptance.")
        print("Two additional discipline spans reproduced; self-review only, not independent evaluation.")
    print("Corpus integrity passed; no new answer was automatically graded or Host certified.")
