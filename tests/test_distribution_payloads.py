from __future__ import annotations

import importlib.util
import json
import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from qiongli.source_layout import RepoLayout


REPO_ROOT = Path(__file__).resolve().parents[1]
LAYOUT = RepoLayout(REPO_ROOT)
AUDIT_PATH = LAYOUT.scripts / "audit_distribution_payloads.py"
EVALUATE_ROUTER_PATH = LAYOUT.scripts / "evaluate_subject_router.py"
MATERIALIZER_PATH = LAYOUT.scripts / "materialize_distribution_payloads.py"


def _load_audit_module():
    spec = importlib.util.spec_from_file_location("audit_distribution_payloads", AUDIT_PATH)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"Unable to load {AUDIT_PATH}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def _load_evaluate_router_module():
    spec = importlib.util.spec_from_file_location("evaluate_subject_router", EVALUATE_ROUTER_PATH)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"Unable to load {EVALUATE_ROUTER_PATH}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


class DistributionFileMapTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.audit_module = _load_audit_module()

    def test_excluded_directories_are_not_scanned(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            for name in (*self.audit_module.EXCLUDED_NAMES, "payload"):
                (root / "kept" / name / "nested").mkdir(parents=True)
            (root / "kept/source.md").write_bytes(b"source")
            # Suffix exclusions historically apply to the entry, not descendants.
            (root / "cache.pyc").mkdir()
            (root / "cache.pyc/source.md").write_bytes(b"source")
            (root / "ignored.pyc").write_bytes(b"ignored")
            original = os.scandir
            scanned = []

            def scan(path):
                scanned.append(Path(path).relative_to(root).as_posix())
                return original(path)

            with patch("os.scandir", side_effect=scan):
                files, issues = self.audit_module._file_map(root, extra_excluded_names={"payload"})
            self.assertEqual([], issues)
            self.assertEqual(["cache.pyc/source.md", "kept/source.md"], list(files))
            self.assertEqual([".", "cache.pyc", "kept"], sorted(scanned))

    def test_comparison_retains_missing_extra_and_changed_bytes(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            left, right = Path(tmp) / "source", Path(tmp) / "payload"
            left.mkdir()
            right.mkdir()
            for root in (left, right):
                (root / "nested").mkdir()
                (root / "nested/same.md").write_bytes(b"same\r\n")
            (left / "missing.md").write_bytes(b"missing")
            (right / "extra.md").write_bytes(b"extra")
            (left / "nested/changed.md").write_bytes(b"source\r\n")
            (right / "nested/changed.md").write_bytes(b"source\n")
            issues = self.audit_module._compare_trees(left, right, "payload")
            self.assertEqual([
                f"missing in {right}: missing.md",
                f"extra in {right}: extra.md",
                "content mismatch: nested/changed.md",
            ], [issue.detail for issue in issues])
            files, issues = self.audit_module._file_map(left / "missing.md")
            self.assertEqual({"missing.md": self.audit_module._hash_file(left / "missing.md")}, files)
            self.assertEqual([], issues)
            self.assertEqual("missing", self.audit_module._file_map(left / "absent")[1][0].label)

    def test_symlinks_are_reported_without_following_them(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp) / "payload"
            root.mkdir()
            (root / "real.md").write_bytes(b"source")
            (root / "file-link").symlink_to(root / "real.md")
            (root / "directory-link").symlink_to(root, target_is_directory=True)
            (root / "broken-link").symlink_to(root / "absent")
            (root / "node_modules").mkdir()
            (root / "node_modules/hidden-link").symlink_to(root / "real.md")
            files, issues = self.audit_module._file_map(root)
            self.assertEqual(["real.md"], list(files))
            self.assertEqual(["broken-link", "directory-link", "file-link"],
                             [Path(issue.detail).name for issue in issues])
            self.assertTrue(all(issue.label == "symlink" for issue in issues))
            self.assertEqual("symlink", self.audit_module._file_map(root / "directory-link")[1][0].label)
            # The independent generated-tree guard still checks excluded subtrees.
            guarded = self.audit_module._assert_no_symlinks(root, "generated")
            self.assertEqual(4, len(guarded))
            self.assertTrue(any("hidden-link" in issue.detail for issue in guarded))

    def test_unreadable_included_content_cannot_pass(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            denied = root / "included"
            denied.mkdir()
            original = os.scandir

            def scan(path):
                if Path(path) == denied:
                    raise PermissionError(13, "denied", str(denied))
                return original(path)

            with patch("os.scandir", side_effect=scan):
                issues = self.audit_module._compare_trees(root, root, "payload")
            self.assertEqual(2, len(issues))
            self.assertTrue(all("scan:" in issue.detail and "denied" in issue.detail for issue in issues))
            (root / "source.md").write_bytes(b"source")
            with patch.object(self.audit_module, "_hash_file", side_effect=PermissionError("denied")):
                with self.assertRaises(PermissionError):
                    self.audit_module._file_map(root)


class DistributionPayloadTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.audit_module = _load_audit_module()
        cls.evaluate_router_module = _load_evaluate_router_module()
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
            RepoLayout(self.materialized_root).python_package / "payload" / "subjects",
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

    def test_runtime_enabled_subject_payload_resources_validate(self) -> None:
        from qiongli.bridges.subject_contracts import load_runtime_subject_contracts

        payload_roots = (
            RepoLayout(self.materialized_root).python_package / "payload" / "subjects",
            self.materialized_root / "packages" / "npm-qiongli" / "python-runtime" / "subjects",
        )
        for payload_root in payload_roots:
            contracts = load_runtime_subject_contracts(payload_root, recursive=False)
            for subject in ("accounting", "business", "economics", "finance"):
                with self.subTest(payload_root=payload_root, subject=subject):
                    failures = self.evaluate_router_module._missing_resource_failures(
                        contracts[subject]
                    )
                    self.assertEqual([], failures)

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

    def test_audit_detects_stale_npm_platform_target_registry(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self._copy_distribution_tree(root)

            registry = root / "packages/npm-qiongli/payload/content/distribution/platform-targets.json"
            payload = json.loads(registry.read_text(encoding="utf-8"))
            payload["targets"]["npm-plugin-lite"]["validator"] = "stale-validator"
            registry.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")

            issues = self.audit_module.audit(root)
            joined = "\n".join(f"{issue.label}: {issue.detail}" for issue in issues)
            self.assertIn("npm platform target registry", joined)
            self.assertIn("platform-targets.json differs", joined)

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
            "packages/python-qiongli/src/qiongli/payload/subjects",
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

    def test_audit_detects_stale_runtime_subject_resource(self) -> None:
        for resource_path in (
            "packages/python-qiongli/src/qiongli/payload/overlays/finance.yaml",
            "packages/npm-qiongli/python-runtime/method-packs/finance/event-study.yaml",
        ):
            with self.subTest(resource_path=resource_path), tempfile.TemporaryDirectory() as tmp:
                root = Path(tmp)
                self._copy_distribution_tree(root)

                stale_file = root / resource_path
                stale_file.write_text(
                    stale_file.read_text(encoding="utf-8")
                    + "\n# stale runtime subject resource marker\n",
                    encoding="utf-8",
                )

                issues = self.audit_module.audit(root)
                joined = "\n".join(f"{issue.label}: {issue.detail}" for issue in issues)
                self.assertIn("runtime subject resource", joined)
                self.assertIn(stale_file.name, joined)

    def _copy_distribution_tree(self, root: Path) -> None:
        shutil.copytree(self.materialized_root, root, symlinks=False, dirs_exist_ok=True)


if __name__ == "__main__":
    unittest.main()
