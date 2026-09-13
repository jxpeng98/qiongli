from __future__ import annotations

import unittest

from scripts.check_frozen_2x_architecture_baseline import (
    FROZEN_ARCHITECTURE_ANCHOR,
    FROZEN_PATHS,
    frozen_changes,
)


class Frozen2xArchitectureBaselineTests(unittest.TestCase):
    def test_expected_decisions_and_handoff_are_frozen(self) -> None:
        self.assertIn(FROZEN_ARCHITECTURE_ANCHOR, FROZEN_PATHS)
        self.assertIn("tooling/migration/2x-branch-point.json", FROZEN_PATHS)
        for number in range(201, 208):
            prefix = f"docs/architecture/decisions/0{number}"
            self.assertTrue(any(path.startswith(prefix) for path in FROZEN_PATHS))

    def test_bootstrap_base_allows_initial_decision_set(self) -> None:
        changes = frozen_changes(
            sorted(FROZEN_PATHS), base_has_architecture_anchor=False
        )
        self.assertEqual(changes, [])

    def test_later_base_rejects_frozen_evidence_change(self) -> None:
        changed = [
            "docs/architecture/decisions/0202-rust-native-ui-and-accessibility.md",
            "docs/superpowers/roadmaps/future.md",
            "tooling/migration/2x-branch-point.json",
        ]
        changes = frozen_changes(changed, base_has_architecture_anchor=True)
        self.assertEqual(
            changes,
            [
                "docs/architecture/decisions/0202-rust-native-ui-and-accessibility.md",
                "tooling/migration/2x-branch-point.json",
            ],
        )

    def test_later_base_allows_new_superseding_adr(self) -> None:
        changes = frozen_changes(
            ["docs/architecture/decisions/0208-supersede-ui-toolkit.md"],
            base_has_architecture_anchor=True,
        )
        self.assertEqual(changes, [])


if __name__ == "__main__":
    unittest.main()
