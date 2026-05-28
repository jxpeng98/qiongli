from __future__ import annotations

import contextlib
import io
import tempfile
import unittest
from pathlib import Path
from unittest import mock

import yaml

from qiongli import cli as cli_module
from qiongli.custom_subject import scaffold_custom_subject


class CustomSubjectScaffoldTests(unittest.TestCase):
    def test_scaffold_custom_subject_dir(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            out = Path(tmp_dir) / "econ-lab"
            scaffold_custom_subject(out, base_subject="economics", name="econ-lab")

            self.assertTrue((out / "subject.yaml").exists())
            self.assertTrue((out / "overlays" / "skills" / "README.md").exists())
            self.assertTrue((out / "skills" / "registry.yaml").exists())
            subject_yaml = (out / "subject.yaml").read_text(encoding="utf-8")
            self.assertIn("base_subject: economics", subject_yaml)
            self.assertIn("skill_overrides:", subject_yaml)

    def test_scaffold_quotes_yaml_significant_values(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            out = Path(tmp_dir) / "custom"
            scaffold_custom_subject(out, base_subject="econ: lab", name="custom\nlab")

            payload = yaml.safe_load((out / "subject.yaml").read_text(encoding="utf-8"))

            self.assertEqual(payload["base_subject"], "econ: lab")
            self.assertEqual(payload["name"], "custom\nlab")

    def test_scaffold_refuses_non_empty_directory_without_force(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            out = Path(tmp_dir) / "custom"
            out.mkdir()
            (out / "existing.txt").write_text("keep", encoding="utf-8")

            with self.assertRaisesRegex(FileExistsError, "not empty"):
                scaffold_custom_subject(out, base_subject="economics", name="custom")

    def test_customize_command_scaffolds_custom_subject(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            out = Path(tmp_dir) / "econ-lab"
            stdout = io.StringIO()
            with mock.patch.object(
                cli_module.sys,
                "argv",
                [
                    "qiongli",
                    "customize",
                    "--subject",
                    "economics",
                    "--name",
                    "econ-lab",
                    "--out",
                    str(out),
                ],
            ), contextlib.redirect_stdout(stdout):
                exit_code = cli_module.main()

            self.assertEqual(exit_code, 0)
            self.assertTrue((out / "subject.yaml").exists())
            self.assertIn(f"Created custom subject overlay at {out}", stdout.getvalue())

    def test_customize_command_reports_non_empty_directory_without_traceback(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            out = Path(tmp_dir) / "custom"
            out.mkdir()
            (out / "existing.txt").write_text("keep", encoding="utf-8")
            stderr = io.StringIO()

            with mock.patch.object(
                cli_module.sys,
                "argv",
                [
                    "qiongli",
                    "customize",
                    "--subject",
                    "economics",
                    "--name",
                    "custom",
                    "--out",
                    str(out),
                ],
            ), contextlib.redirect_stderr(stderr):
                exit_code = cli_module.main()

            self.assertNotEqual(exit_code, 0)
            self.assertIn("[error] custom subject directory is not empty:", stderr.getvalue())
            self.assertNotIn("Traceback", stderr.getvalue())
