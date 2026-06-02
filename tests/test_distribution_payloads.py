from __future__ import annotations

import importlib.util
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from qiongli.source_layout import RepoLayout


REPO_ROOT = Path(__file__).resolve().parents[1]
LAYOUT = RepoLayout(REPO_ROOT)
AUDIT_PATH = REPO_ROOT / "scripts" / "audit_distribution_payloads.py"
MATERIALIZER_PATH = REPO_ROOT / "scripts" / "materialize_distribution_payloads.py"


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
        cls._materialized_tmp = tempfile.TemporaryDirectory()
        cls.materialized_root = Path(cls._materialized_tmp.name) / "qiongli-dist"
        subprocess.run(
            [
                sys.executable,
                str(MATERIALIZER_PATH),
                "--target",
                "all",
                "--out",
                str(cls.materialized_root),
                "--force",
            ],
            cwd=REPO_ROOT,
            check=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )

    @classmethod
    def tearDownClass(cls) -> None:
        cls._materialized_tmp.cleanup()

    def test_current_distribution_payloads_match_sources(self) -> None:
        issues = self.audit_module.audit(self.materialized_root)
        self.assertEqual([], issues)

    def test_distribution_includes_specialized_subject_payloads(self) -> None:
        for payload_root in (
            self.materialized_root / "qiongli" / "payload" / "subjects",
            self.materialized_root / "packages" / "npm-qiongli" / "payload" / "subjects",
        ):
            for subject in ("accounting", "business", "finance", "political-economy", "geoeconomics"):
                with self.subTest(payload_root=payload_root, subject=subject):
                    self.assertTrue(
                        (
                            payload_root
                            / subject
                            / "complete"
                            / "qiongli-workflow"
                            / "SUBJECT_MANIFEST.json"
                        ).exists()
                    )
                    self.assertTrue(
                        (
                            payload_root
                            / subject
                            / "focused"
                            / "qiongli-workflow"
                            / "SUBJECT_MANIFEST.json"
                        ).exists()
                    )

    def test_audit_detects_stale_npm_payload(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self._copy_distribution_tree(root)

            stale_file = root / "packages/npm-qiongli/payload/qiongli-workflow/skills/registry.yaml"
            stale_file.write_text(stale_file.read_text(encoding="utf-8") + "\n# stale payload marker\n", encoding="utf-8")

            issues = self.audit_module.audit(root)
            joined = "\n".join(f"{issue.label}: {issue.detail}" for issue in issues)
            self.assertIn("npm payload skills/ vs source skills/", joined)
            self.assertIn("registry.yaml", joined)

    def test_audit_detects_generated_symlink(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self._copy_distribution_tree(root)

            target = root / LAYOUT.workflow.relative_to(REPO_ROOT) / "SKILL.md"
            link = root / "packages/npm-qiongli/payload/qiongli-workflow/SKILL-link.md"
            link.symlink_to(target)

            issues = self.audit_module.audit(root)
            joined = "\n".join(f"{issue.label}: {issue.detail}" for issue in issues)
            self.assertIn("symlink", joined)
            self.assertIn("SKILL-link.md", joined)

    def test_audit_detects_stale_generated_subject_payload(self) -> None:
        for payload_root in (
            "packages/npm-qiongli/payload/subjects",
            "qiongli/payload/subjects",
        ):
            with self.subTest(payload_root=payload_root), tempfile.TemporaryDirectory() as tmp:
                root = Path(tmp)
                self._copy_distribution_tree(root)

                stale_file = (
                    root
                    / payload_root
                    / "economics-accounting"
                    / "focused"
                    / "qiongli-workflow"
                    / "skills"
                    / "registry.yaml"
                )
                stale_file.write_text(
                    stale_file.read_text(encoding="utf-8") + "\n# stale subject payload marker\n",
                    encoding="utf-8",
                )

                issues = self.audit_module.audit(root)
                joined = "\n".join(f"{issue.label}: {issue.detail}" for issue in issues)
                self.assertIn("subject payload", joined)
                self.assertIn("economics-accounting/focused", joined)
                self.assertIn("registry.yaml", joined)

    def _copy_distribution_tree(self, root: Path) -> None:
        shutil.copytree(self.materialized_root, root, symlinks=False, dirs_exist_ok=True)


if __name__ == "__main__":
    unittest.main()
