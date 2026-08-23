from __future__ import annotations

import copy
import json
from pathlib import Path
import tempfile
import unittest

from tooling.scripts.update_program_roadmap import (
    INDEX_RELATIVE,
    LEDGER_RELATIVE,
    ROADMAP_RELATIVE,
    ProgramLedgerError,
    render_index,
    require_current_index,
    validate_program,
)


REPO_ROOT = Path(__file__).resolve().parents[1]


def _row(task_id: str, **overrides: object) -> dict[str, object]:
    row: dict[str, object] = {
        "id": task_id,
        "state": "proposed",
        "owner": task_id.split("-", 1)[0],
        "dependencies": [],
        "evidence": [],
        "commit": "",
        "run": "",
        "updated_at": "2026-08-18",
        "blocker": "",
    }
    row.update(overrides)
    return row


class ProgramRoadmapTests(unittest.TestCase):
    def _fixture(self) -> tuple[Path, Path, Path, dict[str, object], tempfile.TemporaryDirectory[str]]:
        temporary = tempfile.TemporaryDirectory()
        root = Path(temporary.name)
        roadmap = root / "roadmap.md"
        ledger = root / "ledger.json"
        evidence = root / "evidence.md"
        roadmap.write_text(
            "# Test roadmap\n\n"
            "## 9. Milestone M0 — Test\n\n"
            "- [ ] `GOV-401` Create the ledger.\n"
            "- [x] `GOV-402` Validate the ledger.\n",
            encoding="utf-8",
        )
        evidence.write_text("accepted\n", encoding="utf-8")
        document: dict[str, object] = {
            "schema_version": "qiongli-program-ledger/v1",
            "roadmap": "roadmap.md",
            "tasks": [_row("GOV-401"), _row("GOV-402")],
        }
        ledger.write_text(json.dumps(document), encoding="utf-8")
        return root, roadmap, ledger, document, temporary

    def test_repository_ledger_is_complete_and_fresh(self) -> None:
        tasks, rows = validate_program(
            REPO_ROOT,
            REPO_ROOT / ROADMAP_RELATIVE,
            REPO_ROOT / LEDGER_RELATIVE,
        )

        self.assertEqual(len(tasks), 237)
        self.assertEqual([task.id for task in tasks], [row["id"] for row in rows])
        self.assertEqual(
            render_index(tasks, rows),
            (REPO_ROOT / INDEX_RELATIVE).read_text(encoding="utf-8"),
        )

    def test_rejects_invalid_duplicate_and_missing_ids(self) -> None:
        root, roadmap, ledger, document, temporary = self._fixture()
        self.addCleanup(temporary.cleanup)
        mutations = {
            "invalid state": lambda rows: rows[0].update(state="done"),
            "duplicate task ID": lambda rows: rows.append(copy.deepcopy(rows[0])),
            "ledger IDs do not match roadmap": lambda rows: rows.pop(),
        }
        for message, mutate in mutations.items():
            with self.subTest(message=message):
                changed = copy.deepcopy(document)
                rows = changed["tasks"]
                self.assertIsInstance(rows, list)
                mutate(rows)
                ledger.write_text(json.dumps(changed), encoding="utf-8")
                with self.assertRaisesRegex(ProgramLedgerError, message):
                    validate_program(root, roadmap, ledger)

    def test_rejects_unknown_and_cyclic_dependencies(self) -> None:
        root, roadmap, ledger, document, temporary = self._fixture()
        self.addCleanup(temporary.cleanup)
        changed = copy.deepcopy(document)
        changed["tasks"][0]["dependencies"] = ["GOV-999"]
        ledger.write_text(json.dumps(changed), encoding="utf-8")
        with self.assertRaisesRegex(ProgramLedgerError, "unknown dependency"):
            validate_program(root, roadmap, ledger)

        changed["tasks"][0]["dependencies"] = ["GOV-402"]
        changed["tasks"][1]["dependencies"] = ["GOV-401"]
        ledger.write_text(json.dumps(changed), encoding="utf-8")
        with self.assertRaisesRegex(ProgramLedgerError, "dependency cycle"):
            validate_program(root, roadmap, ledger)

    def test_accepted_requires_repository_evidence_commit_and_run(self) -> None:
        root, roadmap, ledger, document, temporary = self._fixture()
        self.addCleanup(temporary.cleanup)
        required = {
            "repository evidence": {},
            "exact commit": {"evidence": ["evidence.md"]},
            "Actions run": {
                "evidence": ["evidence.md"],
                "commit": "a" * 40,
            },
        }
        for message, values in required.items():
            with self.subTest(message=message):
                changed = copy.deepcopy(document)
                changed["tasks"][0].update(state="accepted", **values)
                ledger.write_text(json.dumps(changed), encoding="utf-8")
                with self.assertRaisesRegex(ProgramLedgerError, message):
                    validate_program(root, roadmap, ledger)

    def test_render_is_deterministic_and_ignores_checkbox_state(self) -> None:
        root, roadmap, ledger, document, temporary = self._fixture()
        self.addCleanup(temporary.cleanup)
        document["tasks"][0].update(state="active")
        document["tasks"][1].update(state="accepted", evidence=["evidence.md"], commit="a" * 40, run="42")
        ledger.write_text(json.dumps(document), encoding="utf-8")
        tasks, rows = validate_program(root, roadmap, ledger)

        first = render_index(tasks, rows)
        second = render_index(tasks, rows)

        self.assertEqual(first, second)
        self.assertIn("| `GOV-401` | `active` |", first)
        self.assertIn("| `GOV-402` | `accepted` |", first)

    def test_stale_generated_index_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            index = Path(directory) / "index.md"
            index.write_text("stale\n", encoding="utf-8")

            with self.assertRaisesRegex(ProgramLedgerError, "index is stale"):
                require_current_index(index, "current\n")

    def test_evaluation_truth_ci_owns_the_program_check(self) -> None:
        workflow = (
            REPO_ROOT / ".github/workflows/evaluation-truth.yml"
        ).read_text(
            encoding="utf-8"
        )

        self.assertIn(
            "python tooling/scripts/update_program_roadmap.py --check", workflow
        )


if __name__ == "__main__":
    unittest.main()
