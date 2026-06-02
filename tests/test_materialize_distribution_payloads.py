from __future__ import annotations

import importlib.util
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
MATERIALIZER_PATH = REPO_ROOT / "scripts" / "materialize_distribution_payloads.py"
SYNC_SKILL_PACKAGE = REPO_ROOT / "scripts" / "sync_skill_package.sh"
SYNC_NPM_PAYLOAD = REPO_ROOT / "scripts" / "sync_npm_package_payload.py"


def _load_materializer_module():
    spec = importlib.util.spec_from_file_location("materialize_distribution_payloads", MATERIALIZER_PATH)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"Unable to load {MATERIALIZER_PATH}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


class DistributionMaterializerTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.materializer = _load_materializer_module()

    def test_rejects_default_in_place_materialization(self) -> None:
        result = subprocess.run(
            [sys.executable, str(MATERIALIZER_PATH), "--target", "plugin"],
            cwd=REPO_ROOT,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )

        self.assertEqual(2, result.returncode)
        self.assertIn("requires either --out or --in-place", result.stderr)

    def test_rejects_existing_non_empty_staging_without_force(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            out = Path(tmp) / "staging"
            out.mkdir()
            (out / "marker.txt").write_text("existing\n", encoding="utf-8")

            result = subprocess.run(
                [sys.executable, str(MATERIALIZER_PATH), "--target", "plugin", "--out", str(out)],
                cwd=REPO_ROOT,
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                check=False,
            )

        self.assertEqual(2, result.returncode)
        self.assertIn("already exists and is not empty", result.stderr)
        self.assertIn("--force", result.stderr)

    def test_source_tree_copy_excludes_materialized_outputs(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            source = Path(tmp) / "source"
            dest = Path(tmp) / "dest"
            (source / "skills").mkdir(parents=True)
            (source / "skills" / "registry.yaml").write_text("skills: []\n", encoding="utf-8")
            generated = source / "packages/npm-qiongli/payload/qiongli-workflow/SKILL.md"
            generated.parent.mkdir(parents=True)
            generated.write_text("generated\n", encoding="utf-8")
            python_payload = source / "qiongli/payload/qiongli-workflow/SKILL.md"
            python_payload.parent.mkdir(parents=True)
            python_payload.write_text("generated\n", encoding="utf-8")
            plugin_payload = source / "plugins/qiongli/skills/qiongli-workflow/SKILL.md"
            plugin_payload.parent.mkdir(parents=True)
            plugin_payload.write_text("generated\n", encoding="utf-8")

            self.materializer.copy_source_tree(source, dest)

            self.assertTrue((dest / "skills" / "registry.yaml").is_file())
            self.assertFalse((dest / "packages/npm-qiongli/payload").exists())
            self.assertFalse((dest / "qiongli/payload").exists())
            self.assertFalse((dest / "plugins/qiongli/skills/qiongli-workflow").exists())

    def test_plugin_target_materializes_to_staging_without_touching_source(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            out = Path(tmp) / "staging"

            result = subprocess.run(
                [
                    sys.executable,
                    str(MATERIALIZER_PATH),
                    "--target",
                    "plugin",
                    "--out",
                    str(out),
                    "--force",
                ],
                cwd=REPO_ROOT,
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                check=False,
            )

            self.assertEqual(0, result.returncode, result.stderr)
            self.assertTrue((out / "plugins/qiongli/skills/qiongli-workflow/SKILL.md").is_file())
            self.assertTrue((out / "plugins/qiongli/skills/qiongli-workflow/skills/registry.yaml").is_file())
            self.assertTrue((out / "qiongli-workflow/skills/registry.yaml").is_file())

    def test_plugin_materializer_does_not_depend_on_bash(self) -> None:
        source = MATERIALIZER_PATH.read_text(encoding="utf-8")

        self.assertNotIn("scripts/sync_skill_package.sh", source)
        self.assertNotIn('["bash"', source)
        self.assertIn("def materialize_plugin_payload", source)

    def test_legacy_sync_helpers_are_marked_internal(self) -> None:
        sync_skill = SYNC_SKILL_PACKAGE.read_text(encoding="utf-8")
        sync_npm = SYNC_NPM_PAYLOAD.read_text(encoding="utf-8")

        for content in (sync_skill, sync_npm):
            with self.subTest(script=content.splitlines()[0]):
                self.assertIn("Internal compatibility helper", content)
                self.assertIn("scripts/materialize_distribution_payloads.py", content)
                self.assertIn("Do not use this as the normal feature-development entrypoint", content)


if __name__ == "__main__":
    unittest.main()
