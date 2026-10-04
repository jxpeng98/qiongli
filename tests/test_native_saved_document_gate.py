import unittest

from tooling.scripts.native_registry_install_check import (
    validate_tool_counts,
    validate_tool_inventory,
)


class SavedDocumentGateTests(unittest.TestCase):
    def test_named_full_extension_requires_fulltext_and_full_profile(self):
        base = [f"base_{index}" for index in range(32)]
        fulltext = "qiongli_literature_read_fulltext"
        document = "qiongli_project_document_read"
        self.assertEqual(
            validate_tool_inventory("full", base + [fulltext, document]), 34
        )
        invalid_inventories = [
            ("lite", base[:14] + [document]),
            ("full", base + [document]),
            ("full", base + [fulltext, "unknown"]),
            ("full", base + [fulltext, document, document]),
        ]
        for profile, names in invalid_inventories:
            with self.subTest(profile=profile, names=names):
                with self.assertRaises(ValueError):
                    validate_tool_inventory(profile, names)

    def test_only_coherent_historical_or_current_counts(self):
        valid_counts = [
            {"lite": 14, "full": 32},
            {"lite": 15, "full": 33},
            {"lite": 15, "full": 34},
            {"lite": 15, "full": 35},
        ]
        for counts in valid_counts:
            self.assertEqual(validate_tool_counts(counts), counts)
        invalid_counts = [
            {"lite": 14, "full": 34},
            {"lite": 16, "full": 34},
            {"lite": 15, "full": 32},
            {"lite": True, "full": 34},
        ]
        for counts in invalid_counts:
            with self.subTest(counts=counts):
                with self.assertRaises(ValueError):
                    validate_tool_counts(counts)

    def test_listing_extension_requires_paired_full_reader(self):
        base = [f"base_{index}" for index in range(32)]
        fulltext = "qiongli_literature_read_fulltext"
        reader = "qiongli_project_document_read"
        listing = "qiongli_project_document_list"
        self.assertEqual(
            validate_tool_inventory("full", base + [fulltext, reader, listing]), 35
        )
        for profile, names in [
            ("lite", base[:14] + [fulltext, reader, listing]),
            ("full", base + [fulltext, listing]),
            ("full", base + [reader, listing]),
            ("full", base + [fulltext, reader, "unknown"]),
        ]:
            with self.subTest(profile=profile, names=names):
                with self.assertRaises(ValueError):
                    validate_tool_inventory(profile, names)
        for counts in [{"lite": 14, "full": 35}, {"lite": 16, "full": 35}]:
            with self.subTest(counts=counts):
                with self.assertRaises(ValueError):
                    validate_tool_counts(counts)
