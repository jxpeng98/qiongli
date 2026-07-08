from __future__ import annotations

import importlib.util
import sys
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
SCRIPT_PATH = REPO_ROOT / "tooling" / "scripts" / "release_upload_assets.py"


def load_release_upload_assets():
    spec = importlib.util.spec_from_file_location("release_upload_assets", SCRIPT_PATH)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"Unable to load {SCRIPT_PATH}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


class ReleaseUploadAssetsTests(unittest.TestCase):
    def test_stable_upload_assets_are_derived_from_registry_targets(self) -> None:
        module = load_release_upload_assets()

        names = module.release_upload_asset_names(
            "v1.6.0",
            root=REPO_ROOT,
            require_existing=False,
        )

        self.assertFalse(hasattr(module, "EXTRA_UPLOAD_ASSET_KEYS"))
        self.assertIn("qiongli-codex-plugin-v1.6.0.tar.gz", names)
        self.assertIn("qiongli-core-codex-plugin-v1.6.0.tar.gz", names)
        self.assertIn("qiongli-claude-plugin-v1.6.0.tar.gz", names)
        self.assertIn("qiongli-claude-plugin-v1.6.0.zip", names)
        self.assertIn("qiongli-claude-desktop-plugin-v1.6.0.zip", names)
        self.assertIn("qiongli-claude-desktop-skill-core-v1.6.0.zip", names)
        self.assertIn("qiongli-literature-provider-0.1.5.mcpb", names)
        self.assertIn("qiongli-zotero-companion-0.2.2.xpi", names)
        self.assertIn("qiongli-downloads-v1.6.0.md", names)
        self.assertIn("qiongli-downloads-v1.6.0.json", names)
        self.assertIn("qiongli-artifacts-v1.6.0.json", names)
        self.assertEqual(len(names), len(set(names)))
        self.assertLess(
            names.index("qiongli-codex-plugin-v1.6.0.tar.gz"),
            names.index("qiongli-claude-plugin-v1.6.0.tar.gz"),
        )

    def test_prerelease_upload_assets_are_next_channel_only(self) -> None:
        module = load_release_upload_assets()

        names = module.release_upload_asset_names(
            "v1.6.0-beta.1",
            root=REPO_ROOT,
            require_existing=False,
        )

        self.assertIn("qiongli-next-codex-plugin-v1.6.0-beta.1.tar.gz", names)
        self.assertIn("qiongli-next-claude-plugin-v1.6.0-beta.1.tar.gz", names)
        self.assertIn("qiongli-next-claude-plugin-v1.6.0-beta.1.zip", names)
        self.assertIn("qiongli-next-claude-desktop-plugin-v1.6.0-beta.1.zip", names)
        self.assertIn("qiongli-next-claude-desktop-skill-core-v1.6.0-beta.1.zip", names)
        self.assertIn("qiongli-artifacts-v1.6.0-beta.1.json", names)
        self.assertNotIn("qiongli-finance-codex-plugin-v1.6.0-beta.1.tar.gz", names)

    def test_missing_upload_asset_paths_fail_when_required(self) -> None:
        module = load_release_upload_assets()
        with tempfile.TemporaryDirectory() as tmp_dir:
            dist_dir = Path(tmp_dir)
            with self.assertRaisesRegex(module.ReleaseUploadAssetError, "missing upload assets"):
                module.release_upload_asset_paths(
                    "v1.6.0-beta.1",
                    dist_dir=dist_dir,
                    root=REPO_ROOT,
                    require_existing=True,
                )


if __name__ == "__main__":
    unittest.main()
