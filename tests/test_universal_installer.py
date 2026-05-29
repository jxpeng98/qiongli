from __future__ import annotations

import contextlib
import io
import json
import os
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from qiongli.universal_installer import InstallOptions, clean, clean_global_legacy_skills, install


REPO_ROOT = Path(__file__).resolve().parents[1]


class UniversalInstallerTests(unittest.TestCase):
    def test_same_version_install_reports_current_and_source_versions(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            temp_root = Path(tmp_dir)
            project_dir = temp_root / "project"
            project_dir.mkdir(parents=True)
            codex_home = temp_root / "codex-home"
            existing_skill = codex_home / "skills" / "qiongli-workflow"
            existing_skill.mkdir(parents=True)
            source_version = (REPO_ROOT / "qiongli-workflow" / "VERSION").read_text(encoding="utf-8").strip()
            (existing_skill / "SKILL.md").write_text(
                "---\nname: qiongli-workflow\ndescription: current\n---\n",
                encoding="utf-8",
            )
            (existing_skill / "VERSION").write_text(f"{source_version}\n", encoding="utf-8")

            env = os.environ.copy()
            env["CODEX_HOME"] = str(codex_home)
            env["PATH"] = ""

            stdout = io.StringIO()
            with mock.patch.dict(os.environ, env, clear=True):
                with contextlib.redirect_stdout(stdout):
                    result = install(
                        InstallOptions(
                            repo_root=REPO_ROOT,
                            project_dir=project_dir,
                            target="codex",
                            profile="partial",
                        )
                    )

            self.assertEqual(result, 0)
            rendered = stdout.getvalue()
            self.assertIn("== Detected Versions ==", rendered)
            self.assertIn(f"source:      {source_version}", rendered)
            self.assertIn(f"codex       {source_version}", rendered)
            self.assertIn(f"current {source_version}; source {source_version}; already installed", rendered)

    def test_existing_managed_skill_auto_updates_without_overwrite(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            temp_root = Path(tmp_dir)
            project_dir = temp_root / "project"
            project_dir.mkdir(parents=True)
            codex_home = temp_root / "codex-home"
            existing_skill = codex_home / "skills" / "qiongli-workflow"
            existing_skill.mkdir(parents=True)
            (existing_skill / "SKILL.md").write_text(
                "---\nname: qiongli-workflow\ndescription: legacy\n---\n",
                encoding="utf-8",
            )
            (existing_skill / "VERSION").write_text("v0.4.0-beta.14\n", encoding="utf-8")
            (existing_skill / "legacy.txt").write_text("old", encoding="utf-8")

            env = os.environ.copy()
            env["CODEX_HOME"] = str(codex_home)
            env["PATH"] = ""

            with mock.patch.dict(os.environ, env, clear=True):
                result = install(
                    InstallOptions(
                        repo_root=REPO_ROOT,
                        project_dir=project_dir,
                        target="codex",
                        profile="partial",
                    )
                )

            self.assertEqual(result, 0)
            self.assertEqual(
                (existing_skill / "VERSION").read_text(encoding="utf-8").strip(),
                (REPO_ROOT / "qiongli-workflow" / "VERSION").read_text(encoding="utf-8").strip(),
            )
            self.assertFalse((existing_skill / "legacy.txt").exists())
            self.assertTrue((existing_skill / "skills-core.md").exists())

    def test_existing_versionless_managed_skill_auto_updates_without_overwrite(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            temp_root = Path(tmp_dir)
            project_dir = temp_root / "project"
            project_dir.mkdir(parents=True)
            codex_home = temp_root / "codex-home"
            existing_skill = codex_home / "skills" / "qiongli-workflow"
            existing_skill.mkdir(parents=True)
            source_version = (REPO_ROOT / "qiongli-workflow" / "VERSION").read_text(encoding="utf-8").strip()
            (existing_skill / "SKILL.md").write_text(
                "---\nname: qiongli-workflow\ndescription: legacy without version\n---\n",
                encoding="utf-8",
            )
            (existing_skill / "legacy.txt").write_text("old", encoding="utf-8")

            env = os.environ.copy()
            env["CODEX_HOME"] = str(codex_home)
            env["PATH"] = ""

            stdout = io.StringIO()
            with mock.patch.dict(os.environ, env, clear=True):
                with contextlib.redirect_stdout(stdout):
                    result = install(
                        InstallOptions(
                            repo_root=REPO_ROOT,
                            project_dir=project_dir,
                            target="codex",
                            profile="partial",
                        )
                    )

            self.assertEqual(result, 0)
            self.assertEqual(
                (existing_skill / "VERSION").read_text(encoding="utf-8").strip(),
                source_version,
            )
            self.assertFalse((existing_skill / "legacy.txt").exists())
            self.assertTrue((existing_skill / "skills-core.md").exists())
            self.assertIn(
                f"current unknown; source {source_version}; updated unknown -> {source_version}",
                stdout.getvalue(),
            )

    def test_install_removes_legacy_global_skill_residues(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            temp_root = Path(tmp_dir)
            project_dir = temp_root / "project"
            project_dir.mkdir(parents=True)
            codex_home = temp_root / "codex-home"
            legacy_skill = codex_home / "skills" / "research-paper-workflow"
            legacy_skill.mkdir(parents=True)
            (legacy_skill / "SKILL.md").write_text(
                "---\nname: research-paper-workflow\ndescription: legacy\n---\n",
                encoding="utf-8",
            )

            env = os.environ.copy()
            env["CODEX_HOME"] = str(codex_home)
            env["PATH"] = ""

            stdout = io.StringIO()
            with mock.patch.dict(os.environ, env, clear=True):
                with contextlib.redirect_stdout(stdout):
                    result = install(
                        InstallOptions(
                            repo_root=REPO_ROOT,
                            project_dir=project_dir,
                            target="codex",
                            profile="partial",
                        )
                    )

            self.assertEqual(result, 0)
            rendered = stdout.getvalue()
            self.assertIn("Legacy Install Cleanup", rendered)
            self.assertIn("codex: research-paper-workflow", rendered)
            self.assertIn(str(legacy_skill), rendered)
            self.assertFalse(legacy_skill.exists())

    def test_install_keeps_unmanaged_legacy_named_global_dir(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            temp_root = Path(tmp_dir)
            project_dir = temp_root / "project"
            project_dir.mkdir(parents=True)
            codex_home = temp_root / "codex-home"
            legacy_named_dir = codex_home / "skills" / "research-paper-workflow"
            legacy_named_dir.mkdir(parents=True)
            (legacy_named_dir / "notes.txt").write_text("user data", encoding="utf-8")

            env = os.environ.copy()
            env["CODEX_HOME"] = str(codex_home)
            env["PATH"] = ""

            stdout = io.StringIO()
            with mock.patch.dict(os.environ, env, clear=True):
                with contextlib.redirect_stdout(stdout):
                    result = install(
                        InstallOptions(
                            repo_root=REPO_ROOT,
                            project_dir=project_dir,
                            target="codex",
                            profile="partial",
                        )
                    )

            self.assertEqual(result, 0)
            self.assertIn("unmanaged legacy-named path", stdout.getvalue())
            self.assertTrue(legacy_named_dir.exists())

    def test_existing_unmanaged_cli_requires_overwrite(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            temp_root = Path(tmp_dir)
            project_dir = temp_root / "project"
            project_dir.mkdir(parents=True)
            cli_dir = temp_root / "bin"
            cli_dir.mkdir(parents=True)
            existing_cli = cli_dir / "qiongli"
            existing_cli.write_text("#!/usr/bin/env bash\necho custom\n", encoding="utf-8")
            codex_home = temp_root / "codex-home"
            claude_home = temp_root / "claude-home"
            gemini_home = temp_root / "gemini-home"
            antigravity_home = temp_root / "antigravity-home"

            env = os.environ.copy()
            env["CODEX_HOME"] = str(codex_home)
            env["CLAUDE_CODE_HOME"] = str(claude_home)
            env["GEMINI_HOME"] = str(gemini_home)
            env["ANTIGRAVITY_HOME"] = str(antigravity_home)
            env["PATH"] = ""

            with mock.patch.dict(os.environ, env, clear=True):
                result = install(
                    InstallOptions(
                        repo_root=REPO_ROOT,
                        project_dir=project_dir,
                        target="codex",
                        profile="full",
                        cli_dir=cli_dir,
                        doctor=False,
                    )
                )

            self.assertEqual(result, 0)
            self.assertEqual(existing_cli.read_text(encoding="utf-8"), "#!/usr/bin/env bash\necho custom\n")

    def test_project_only_parts_skip_global_skill_install(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            temp_root = Path(tmp_dir)
            project_dir = temp_root / "project"
            project_dir.mkdir(parents=True)
            codex_home = temp_root / "codex-home"
            claude_home = temp_root / "claude-home"
            gemini_home = temp_root / "gemini-home"
            antigravity_home = temp_root / "antigravity-home"

            env = os.environ.copy()
            env["CODEX_HOME"] = str(codex_home)
            env["CLAUDE_CODE_HOME"] = str(claude_home)
            env["GEMINI_HOME"] = str(gemini_home)
            env["ANTIGRAVITY_HOME"] = str(antigravity_home)
            env["PATH"] = ""

            with mock.patch.dict(os.environ, env, clear=True):
                result = install(
                    InstallOptions(
                        repo_root=REPO_ROOT,
                        project_dir=project_dir,
                        target="all",
                        parts=("project",),
                    )
                )

            self.assertEqual(result, 0)
            self.assertFalse((codex_home / "skills" / "qiongli-workflow" / "SKILL.md").exists())
            self.assertFalse((claude_home / "skills" / "qiongli-workflow" / "SKILL.md").exists())
            self.assertFalse((gemini_home / "skills" / "qiongli-workflow" / "SKILL.md").exists())
            # Project parts now only installs .env
            self.assertTrue((project_dir / ".env").exists())
            # Workflows and CLAUDE.md are no longer installed project-locally
            self.assertFalse((project_dir / ".agent" / "workflows" / "proofread.md").exists())
            self.assertFalse((project_dir / "CLAUDE.md").exists())
            self.assertFalse((project_dir / ".gemini" / "qiongli.md").exists())

    def test_partial_profile_installs_global_skills_with_bundled_workflows(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            temp_root = Path(tmp_dir)
            project_dir = temp_root / "project"
            project_dir.mkdir(parents=True)
            codex_home = temp_root / "codex-home"
            claude_home = temp_root / "claude-home"
            gemini_home = temp_root / "gemini-home"
            antigravity_home = temp_root / "antigravity-home"

            env = os.environ.copy()
            env["CODEX_HOME"] = str(codex_home)
            env["CLAUDE_CODE_HOME"] = str(claude_home)
            env["GEMINI_HOME"] = str(gemini_home)
            env["ANTIGRAVITY_HOME"] = str(antigravity_home)
            env["PATH"] = ""

            with mock.patch.dict(os.environ, env, clear=True):
                result = install(
                    InstallOptions(
                        repo_root=REPO_ROOT,
                        project_dir=project_dir,
                        profile="partial",
                    )
                )

            self.assertEqual(result, 0)
            # Global skills installed for all clients
            self.assertTrue((codex_home / "skills" / "qiongli-workflow" / "SKILL.md").exists())
            self.assertTrue((claude_home / "skills" / "qiongli-workflow" / "SKILL.md").exists())
            self.assertTrue((gemini_home / "skills" / "qiongli-workflow" / "SKILL.md").exists())
            self.assertTrue((antigravity_home / "skills" / "qiongli-workflow" / "SKILL.md").exists())
            # Workflows bundled inside each global skill directory
            self.assertTrue((claude_home / "skills" / "qiongli-workflow" / "workflows" / "paper.md").exists())
            self.assertTrue((gemini_home / "skills" / "qiongli-workflow" / "workflows" / "lit-review.md").exists())
            self.assertTrue((antigravity_home / "skills" / "qiongli-workflow" / "workflows" / "paper.md").exists())
            # Synced bundled assets present in global skill directories
            self.assertTrue((claude_home / "skills" / "qiongli-workflow" / "skills-core.md").exists())
            self.assertTrue((claude_home / "skills" / "qiongli-workflow" / "skills" / "A_framing").is_dir())
            self.assertTrue((claude_home / "skills" / "qiongli-workflow" / "templates" / "manuscript-outline.md").exists())
            self.assertTrue((claude_home / "skills" / "qiongli-workflow" / "standards" / "research-workflow-contract.yaml").exists())
            self.assertTrue((claude_home / "skills" / "qiongli-workflow" / "roles" / "pi.yaml").exists())
            # No project-local files
            self.assertFalse((project_dir / ".agent" / "workflows" / "proofread.md").exists())
            self.assertFalse((project_dir / ".gemini" / "qiongli.md").exists())
            self.assertFalse((project_dir / ".agents" / "skills" / "qiongli-workflow" / "SKILL.md").exists())
            self.assertFalse((project_dir / ".env").exists())
            self.assertFalse((temp_root / ".local" / "bin" / "qiongli").exists())

    def test_install_materializes_requested_subject(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            temp_root = Path(tmp_dir)
            project_dir = temp_root / "project"
            project_dir.mkdir(parents=True)
            codex_home = temp_root / "codex-home"
            env = os.environ.copy()
            env["CODEX_HOME"] = str(codex_home)
            env["PATH"] = ""

            with mock.patch.dict(os.environ, env, clear=True):
                result = install(
                    InstallOptions(
                        repo_root=REPO_ROOT,
                        project_dir=project_dir,
                        target="codex",
                        profile="partial",
                        subject="economics",
                    )
                )

            self.assertEqual(result, 0)
            skill_dir = codex_home / "skills" / "qiongli-workflow"
            self.assertEqual((skill_dir / "SUBJECT").read_text(encoding="utf-8").strip(), "economics")
            manifest = json.loads((skill_dir / "SUBJECT_MANIFEST.json").read_text(encoding="utf-8"))
            self.assertEqual(manifest["coverage"], "complete")
            self.assertTrue((skill_dir / "skills" / "domain-profiles" / "economics.yaml").exists())
            self.assertTrue((skill_dir / "skills" / "domain-profiles" / "cs-ai.yaml").exists())
            self.assertTrue((skill_dir / "skills" / "F_writing" / "manuscript-architect.md").exists())
            self.assertIn(
                "Economics Overlay",
                (skill_dir / "skills" / "F_writing" / "manuscript-architect.md").read_text(encoding="utf-8"),
            )

    def test_install_can_materialize_focused_subject_coverage(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            temp_root = Path(tmp_dir)
            project_dir = temp_root / "project"
            project_dir.mkdir(parents=True)
            codex_home = temp_root / "codex-home"
            env = os.environ.copy()
            env["CODEX_HOME"] = str(codex_home)
            env["PATH"] = ""

            with mock.patch.dict(os.environ, env, clear=True):
                result = install(
                    InstallOptions(
                        repo_root=REPO_ROOT,
                        project_dir=project_dir,
                        target="codex",
                        profile="partial",
                        subject="economics",
                        coverage="focused",
                    )
                )

            self.assertEqual(result, 0)
            skill_dir = codex_home / "skills" / "qiongli-workflow"
            manifest = json.loads((skill_dir / "SUBJECT_MANIFEST.json").read_text(encoding="utf-8"))
            self.assertEqual(manifest["coverage"], "focused")
            self.assertTrue((skill_dir / "skills" / "domain-profiles" / "economics.yaml").exists())
            self.assertFalse((skill_dir / "skills" / "domain-profiles" / "cs-ai.yaml").exists())

    def test_install_warns_when_switching_from_subject_to_core(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            temp_root = Path(tmp_dir)
            project_dir = temp_root / "project"
            project_dir.mkdir(parents=True)
            codex_home = temp_root / "codex-home"
            existing_skill = codex_home / "skills" / "qiongli-workflow"
            existing_skill.mkdir(parents=True)
            source_version = (REPO_ROOT / "qiongli-workflow" / "VERSION").read_text(encoding="utf-8").strip()
            (existing_skill / "SKILL.md").write_text(
                "---\nname: qiongli\ndescription: old\n---\n",
                encoding="utf-8",
            )
            (existing_skill / "VERSION").write_text(f"{source_version}\n", encoding="utf-8")
            (existing_skill / "SUBJECT").write_text("economics\n", encoding="utf-8")
            (existing_skill / "SUBJECT_MANIFEST.json").write_text(
                json.dumps({"subject": "economics", "coverage": "focused", "flavor": "full", "layers": ["core", "economics"]}),
                encoding="utf-8",
            )
            env = os.environ.copy()
            env["CODEX_HOME"] = str(codex_home)
            env["PATH"] = ""
            stdout = io.StringIO()

            with mock.patch.dict(os.environ, env, clear=True), contextlib.redirect_stdout(stdout):
                result = install(
                    InstallOptions(
                        repo_root=REPO_ROOT,
                        project_dir=project_dir,
                        target="codex",
                        profile="partial",
                        subject="core",
                    )
                )

            self.assertEqual(result, 0)
            self.assertIn("Changing active subject from economics to core", stdout.getvalue())

    def test_full_profile_allows_explicit_no_cli(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            temp_root = Path(tmp_dir)
            project_dir = temp_root / "project"
            project_dir.mkdir(parents=True)
            codex_home = temp_root / "codex-home"
            claude_home = temp_root / "claude-home"
            gemini_home = temp_root / "gemini-home"
            antigravity_home = temp_root / "antigravity-home"
            env = os.environ.copy()
            env["CODEX_HOME"] = str(codex_home)
            env["CLAUDE_CODE_HOME"] = str(claude_home)
            env["GEMINI_HOME"] = str(gemini_home)
            env["ANTIGRAVITY_HOME"] = str(antigravity_home)
            env["PATH"] = ""

            cli_dir = temp_root / "bin"
            with mock.patch.dict(os.environ, env, clear=True):
                result = install(
                    InstallOptions(
                        repo_root=REPO_ROOT,
                        project_dir=project_dir,
                        profile="full",
                        install_cli=False,
                        doctor=False,
                        cli_dir=cli_dir,
                    )
                )

            self.assertEqual(result, 0)
            self.assertFalse((cli_dir / "qiongli").exists())


class CleanTests(unittest.TestCase):
    def test_clean_removes_stale_project_assets(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            project_dir = Path(tmp_dir) / "project"
            project_dir.mkdir(parents=True)
            # Create stale assets
            workflows_dir = project_dir / ".agent" / "workflows"
            workflows_dir.mkdir(parents=True)
            (workflows_dir / "paper.md").write_text("stale workflow")
            (workflows_dir / "lit-review.md").write_text("stale workflow")
            skill_dir = project_dir / ".agents" / "skills" / "qiongli-workflow"
            skill_dir.mkdir(parents=True)
            (skill_dir / "SKILL.md").write_text("stale skill")
            legacy_named_skill = project_dir / ".agents" / "skills" / "research-paper-workflow"
            legacy_named_skill.mkdir(parents=True)
            (legacy_named_skill / "SKILL.md").write_text("stale legacy skill")
            legacy_skill = project_dir / ".agent" / "skills" / "qiongli-workflow"
            legacy_skill.mkdir(parents=True)
            (legacy_skill / "SKILL.md").write_text("stale skill")
            legacy_agent_skill = project_dir / ".agent" / "skills" / "research-paper-workflow"
            legacy_agent_skill.mkdir(parents=True)
            (legacy_agent_skill / "SKILL.md").write_text("stale legacy skill")
            gemini_dir = project_dir / ".gemini"
            gemini_dir.mkdir(parents=True)
            (gemini_dir / "qiongli.md").write_text("stale quickstart")
            (gemini_dir / "agent-profiles.example.json").write_text("{}")
            (project_dir / "CLAUDE.qiongli.md").write_text("stale")
            # Write a template-looking CLAUDE.md
            (project_dir / "CLAUDE.md").write_text("# Qiongli\nQiongli Zhengche\nqiongli-workflow")

            result = clean(project_dir)
            self.assertEqual(result, 0)
            # All stale files removed
            self.assertFalse((workflows_dir / "paper.md").exists())
            self.assertFalse((workflows_dir / "lit-review.md").exists())
            self.assertFalse(skill_dir.exists())
            self.assertFalse(legacy_skill.exists())
            self.assertFalse(legacy_named_skill.exists())
            self.assertFalse(legacy_agent_skill.exists())
            self.assertFalse((gemini_dir / "qiongli.md").exists())
            self.assertFalse((gemini_dir / "agent-profiles.example.json").exists())
            self.assertFalse((project_dir / "CLAUDE.qiongli.md").exists())
            self.assertFalse((project_dir / "CLAUDE.md").exists())

    def test_clean_keeps_user_customized_claude_md(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            project_dir = Path(tmp_dir) / "project"
            project_dir.mkdir(parents=True)
            (project_dir / "CLAUDE.md").write_text("# My Custom Project Instructions\nCustom content")

            result = clean(project_dir)
            self.assertEqual(result, 0)
            # User-customized CLAUDE.md should be preserved
            self.assertTrue((project_dir / "CLAUDE.md").exists())

    def test_clean_dry_run_does_not_delete(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            project_dir = Path(tmp_dir) / "project"
            workflows_dir = project_dir / ".agent" / "workflows"
            workflows_dir.mkdir(parents=True)
            (workflows_dir / "paper.md").write_text("stale")

            result = clean(project_dir, dry_run=True)
            self.assertEqual(result, 0)
            # File should still exist after dry-run
            self.assertTrue((workflows_dir / "paper.md").exists())

    def test_clean_workflow_symlinks_removes_only_ours(self) -> None:
        """clean_workflow_symlinks removes only symlinks pointing to qiongli-workflow."""
        with tempfile.TemporaryDirectory() as tmp_dir:
            temp_root = Path(tmp_dir)
            claude_home = temp_root / "claude-home"
            commands_dir = claude_home / "commands"
            commands_dir.mkdir(parents=True)

            # Create a symlink pointing to qiongli-workflow (ours)
            skill_wf = claude_home / "skills" / "qiongli-workflow" / "workflows"
            skill_wf.mkdir(parents=True)
            (skill_wf / "paper.md").write_text("workflow content")
            our_link = commands_dir / "paper.md"
            our_link.symlink_to(skill_wf / "paper.md")

            # Create a user's own command (not a symlink)
            (commands_dir / "my-custom.md").write_text("user command")
            # Create a symlink pointing elsewhere (not ours)
            other_target = temp_root / "other-skill" / "workflows" / "deploy.md"
            other_target.parent.mkdir(parents=True)
            other_target.write_text("other workflow")
            other_link = commands_dir / "deploy.md"
            other_link.symlink_to(other_target)

            env = os.environ.copy()
            env["CLAUDE_CODE_HOME"] = str(claude_home)
            env["GEMINI_HOME"] = str(temp_root / "gemini-home")
            with mock.patch.dict(os.environ, env, clear=True):
                from qiongli.universal_installer import clean_workflow_symlinks
                result = clean_workflow_symlinks()

            self.assertEqual(result, 0)
            # Our symlink removed
            self.assertFalse(our_link.exists())
            # User's own command preserved
            self.assertTrue((commands_dir / "my-custom.md").exists())
            # Other symlink preserved
            self.assertTrue(other_link.is_symlink())

    def test_clean_workflow_symlinks_removes_legacy_skill_links(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            temp_root = Path(tmp_dir)
            claude_home = temp_root / "claude-home"
            commands_dir = claude_home / "commands"
            commands_dir.mkdir(parents=True)
            legacy_wf = claude_home / "skills" / "research-paper-workflow" / "workflows"
            legacy_wf.mkdir(parents=True)
            (legacy_wf / "paper.md").write_text("legacy workflow")
            legacy_link = commands_dir / "paper.md"
            legacy_link.symlink_to(legacy_wf / "paper.md")

            env = os.environ.copy()
            env["CLAUDE_CODE_HOME"] = str(claude_home)
            env["GEMINI_HOME"] = str(temp_root / "gemini-home")
            with mock.patch.dict(os.environ, env, clear=True):
                from qiongli.universal_installer import clean_workflow_symlinks

                result = clean_workflow_symlinks()

            self.assertEqual(result, 0)
            self.assertFalse(legacy_link.exists())

    def test_clean_globals_removes_legacy_global_skill_dirs(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            temp_root = Path(tmp_dir)
            codex_home = temp_root / "codex-home"
            legacy_skill = codex_home / "skills" / "research-paper-workflow"
            legacy_skill.mkdir(parents=True)
            (legacy_skill / "SKILL.md").write_text(
                "---\nname: research-paper-workflow\ndescription: legacy\n---\n",
                encoding="utf-8",
            )

            env = os.environ.copy()
            env["CODEX_HOME"] = str(codex_home)
            env["CLAUDE_CODE_HOME"] = str(temp_root / "claude-home")
            env["GEMINI_HOME"] = str(temp_root / "gemini-home")
            env["ANTIGRAVITY_HOME"] = str(temp_root / "antigravity-home")
            with mock.patch.dict(os.environ, env, clear=True):
                result = clean_global_legacy_skills()

            self.assertEqual(result, 0)
            self.assertFalse(legacy_skill.exists())


class SymlinkAndSummaryTests(unittest.TestCase):
    """Test workflow symlinks and skills-summary bundling."""

    def test_install_creates_workflow_symlinks(self) -> None:
        """After install with target=all, Claude commands/ and Gemini workflows/ have symlinks."""
        with tempfile.TemporaryDirectory() as tmp_dir:
            temp_root = Path(tmp_dir)
            project_dir = temp_root / "project"
            project_dir.mkdir(parents=True)
            claude_home = temp_root / "claude-home"
            gemini_home = temp_root / "gemini-home"
            codex_home = temp_root / "codex-home"
            antigravity_home = temp_root / "antigravity-home"
            env = os.environ.copy()
            env["CODEX_HOME"] = str(codex_home)
            env["CLAUDE_CODE_HOME"] = str(claude_home)
            env["GEMINI_HOME"] = str(gemini_home)
            env["ANTIGRAVITY_HOME"] = str(antigravity_home)
            env["PATH"] = ""

            with mock.patch.dict(os.environ, env, clear=True):
                result = install(
                    InstallOptions(
                        repo_root=REPO_ROOT,
                        project_dir=project_dir,
                        profile="partial",
                        install_cli=False,
                        doctor=False,
                    )
                )

            self.assertEqual(result, 0)

            # Claude: symlinks in commands/
            claude_commands = claude_home / "commands"
            self.assertTrue(claude_commands.is_dir(), "Claude commands/ dir should exist")
            paper_link = claude_commands / "paper.md"
            self.assertTrue(paper_link.is_symlink(), "paper.md should be a symlink")
            self.assertTrue(paper_link.resolve().exists(), "symlink target should exist")
            self.assertIn("qiongli-workflow", str(paper_link.resolve()))

            # Gemini: symlinks in workflows/
            gemini_workflows = gemini_home / "workflows"
            self.assertTrue(gemini_workflows.is_dir(), "Gemini workflows/ dir should exist")
            lit_link = gemini_workflows / "lit-review.md"
            self.assertTrue(lit_link.is_symlink(), "lit-review.md should be a symlink")
            self.assertTrue(lit_link.resolve().exists(), "symlink target should exist")

            # All 16 workflows should have symlinks
            expected_count = len(list((claude_home / "skills" / "qiongli-workflow" / "workflows").glob("*.md")))
            self.assertEqual(len(list(claude_commands.glob("*.md"))), expected_count)
            self.assertEqual(len(list(gemini_workflows.glob("*.md"))), expected_count)

    def test_skills_summary_bundled_in_package(self) -> None:
        """skills-summary.md should be synced into the skill package."""
        with tempfile.TemporaryDirectory() as tmp_dir:
            temp_root = Path(tmp_dir)
            project_dir = temp_root / "project"
            project_dir.mkdir(parents=True)
            claude_home = temp_root / "claude-home"
            codex_home = temp_root / "codex-home"
            gemini_home = temp_root / "gemini-home"
            antigravity_home = temp_root / "antigravity-home"
            env = os.environ.copy()
            env["CODEX_HOME"] = str(codex_home)
            env["CLAUDE_CODE_HOME"] = str(claude_home)
            env["GEMINI_HOME"] = str(gemini_home)
            env["ANTIGRAVITY_HOME"] = str(antigravity_home)
            env["PATH"] = ""

            with mock.patch.dict(os.environ, env, clear=True):
                result = install(
                    InstallOptions(
                        repo_root=REPO_ROOT,
                        project_dir=project_dir,
                        profile="partial",
                        install_cli=False,
                        doctor=False,
                    )
                )

            self.assertEqual(result, 0)
            # skills-summary.md present in global skill dir
            summary_path = claude_home / "skills" / "qiongli-workflow" / "skills-summary.md"
            self.assertTrue(summary_path.exists(), "skills-summary.md should be bundled")
            content = summary_path.read_text()
            self.assertIn("Skills Summary", content)
            self.assertIn("question-refiner", content)
            # Should be smaller than skills-core.md
            core_path = claude_home / "skills" / "qiongli-workflow" / "skills-core.md"
            self.assertTrue(core_path.exists())
            self.assertLess(summary_path.stat().st_size, core_path.stat().st_size,
                            "skills-summary.md should be smaller than skills-core.md")


if __name__ == "__main__":
    unittest.main()
