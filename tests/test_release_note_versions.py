from __future__ import annotations

import unittest

from scripts.validate_research_standard import parse_release_note_version


class ReleaseNoteVersionTests(unittest.TestCase):
    def test_orders_alpha_before_beta_before_stable(self) -> None:
        alpha = parse_release_note_version("v2.0.0-alpha.1.md")
        beta = parse_release_note_version("v2.0.0-beta.1.md")
        stable = parse_release_note_version("v2.0.0.md")

        self.assertIsNotNone(alpha)
        self.assertIsNotNone(beta)
        self.assertIsNotNone(stable)
        self.assertLess(alpha, beta)
        self.assertLess(beta, stable)

    def test_rejects_unsupported_or_ambiguous_prerelease_names(self) -> None:
        for name in (
            "v2.0.0-rc.1.md",
            "v2.0.0-alpha.md",
            "v2.0.0-alpha.01.md",
            "v02.0.0-alpha.1.md",
        ):
            with self.subTest(name=name):
                self.assertIsNone(parse_release_note_version(name))


if __name__ == "__main__":
    unittest.main()
