from __future__ import annotations

import importlib.util
import shutil
import sys
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
AUDIT_PATH = REPO_ROOT / "scripts" / "audit_distribution_payloads.py"


def _load_audit_module():
    spec = importlib.util.spec_from_file_location("audit_distribution_payloads", AUDIT_PATH)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"Unable to load {AUDIT_PATH}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


class DistributionPayloadTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.audit_module = _load_audit_module()

    def test_current_distribution_payloads_match_sources(self) -> None:
        issues = self.audit_module.audit(REPO_ROOT)
        self.assertEqual([], issues)

    def test_audit_detects_stale_npm_payload(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            for rel in (
                "qiongli-workflow",
                "plugins/qiongli/skills/qiongli-workflow",
                "packages/npm-qiongli/payload/qiongli-workflow",
                "packages/npm-qiongli/python-runtime",
                "skills",
                "templates",
                "standards",
                "roles",
                "venue-profiles",
            ):
                src = REPO_ROOT / rel
                dest = root / rel
                if src.is_dir():
                    shutil.copytree(src, dest, symlinks=False)
            for rel in (
                "skills-core.md",
                "skills-summary.md",
                "LICENSE",
                "packages/npm-qiongli/package.json",
            ):
                src = REPO_ROOT / rel
                dest = root / rel
                dest.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(src, dest)

            stale_file = root / "packages/npm-qiongli/payload/qiongli-workflow/skills/registry.yaml"
            stale_file.write_text(stale_file.read_text(encoding="utf-8") + "\n# stale payload marker\n", encoding="utf-8")

            issues = self.audit_module.audit(root)
            joined = "\n".join(f"{issue.label}: {issue.detail}" for issue in issues)
            self.assertIn("npm payload vs portable package", joined)
            self.assertIn("skills/registry.yaml", joined)

    def test_audit_detects_generated_symlink(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            for rel in (
                "qiongli-workflow",
                "plugins/qiongli/skills/qiongli-workflow",
                "packages/npm-qiongli/payload/qiongli-workflow",
                "packages/npm-qiongli/python-runtime",
                "skills",
                "templates",
                "standards",
                "roles",
                "venue-profiles",
            ):
                src = REPO_ROOT / rel
                dest = root / rel
                if src.is_dir():
                    shutil.copytree(src, dest, symlinks=False)
            for rel in (
                "skills-core.md",
                "skills-summary.md",
                "LICENSE",
                "packages/npm-qiongli/package.json",
            ):
                src = REPO_ROOT / rel
                dest = root / rel
                dest.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(src, dest)

            target = root / "qiongli-workflow/SKILL.md"
            link = root / "packages/npm-qiongli/payload/qiongli-workflow/SKILL-link.md"
            link.symlink_to(target)

            issues = self.audit_module.audit(root)
            joined = "\n".join(f"{issue.label}: {issue.detail}" for issue in issues)
            self.assertIn("symlink", joined)
            self.assertIn("SKILL-link.md", joined)


if __name__ == "__main__":
    unittest.main()
