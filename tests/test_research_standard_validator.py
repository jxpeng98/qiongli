from __future__ import annotations

import tempfile
import shutil
import unittest
from pathlib import Path
from unittest.mock import patch

from tooling.scripts.validate_research_standard import (
    ValidationReport,
    validate_boundary_review,
    validate_controller_mode_contracts,
    validate_domain_method_pack_contracts,
    validate_generated_skill_docs,
    validate_docs,
    validate_guides,
    validate_skill_package_resource_links,
    validate_workflow_entrypoints,
    validate_literature_first_contracts,
    validate_profile_bundle_template,
    validate_quality_gate_contracts,
)


class ResearchStandardValidatorTests(unittest.TestCase):
    def native_fixture(self, root: Path) -> None:
        repo = Path(__file__).resolve().parents[1]
        manifest = root / 'packages/qiongli-native/Cargo.toml'
        manifest.parent.mkdir(parents=True)
        manifest.touch()
        shutil.copytree(repo / 'content', root / 'content')

    def test_native_workflows_require_both_shared_skill_and_matching_route(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            self.native_fixture(root)
            report = ValidationReport()
            validate_workflow_entrypoints(root, report)
            self.assertEqual(report.errors, [])
            template = root / 'content/workflow/references/codex-workflow-wrapper.md'
            original = template.read_text()
            for link in ('../qiongli-workflow/SKILL.md',
                         '../qiongli-workflow/workflows/{{workflow}}.md'):
                template.write_text(original.replace(link, 'missing-reference'))
                report = ValidationReport()
                validate_workflow_entrypoints(root, report)
                self.assertTrue(any('missing global skill' in error for error in report.errors))
            template.write_text(original)
            workflow = root / 'content/workflow/workflows/paper-read.md'
            workflow.write_text(workflow.read_text().replace('B2', 'unknown-task'))
            report = ValidationReport()
            validate_workflow_entrypoints(root, report)
            self.assertTrue(any('missing task ID B2' in error for error in report.errors))

    def test_native_resource_audit_materializes_without_requiring_generated_outputs(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            self.native_fixture(root)
            report = ValidationReport()
            validate_skill_package_resource_links(root, report)
            self.assertEqual(report.errors, [])
            self.assertFalse((root / 'qiongli-workflow').exists())
            (root / 'content/workflow/references/coverage-matrix.md').unlink()
            report = ValidationReport()
            validate_skill_package_resource_links(root, report)
            self.assertTrue(any('coverage-matrix.md' in error for error in report.errors))

    def test_native_readmes_and_guides_reject_missing_current_contracts(self) -> None:
        repo = Path(__file__).resolve().parents[1]
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            self.native_fixture(root)
            paths = ('README.md', 'README_CN.md', 'CLAUDE.md',
                     'docs/advanced/agent-skill-collaboration.md', 'docs/guide/cli-2x.md')
            for relative in paths:
                destination = root / relative
                destination.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(repo / relative, destination)
            report = ValidationReport()
            validate_docs(root, report)
            validate_guides(root, report)
            self.assertEqual(report.errors, [])
            for relative, token, validator in (
                ('README_CN.md', 'docs/zh/guide/cli-2x.md', validate_docs),
                ('docs/guide/cli-2x.md', 'qiongli mcp serve --profile full', validate_guides),
                ('docs/advanced/agent-skill-collaboration.md', 'templates/agent-handoff.md', validate_guides),
            ):
                path = root / relative
                path.write_text(path.read_text().replace(token, 'missing-contract'))
                report = ValidationReport()
                validator(root, report)
                self.assertTrue(any(token in error for error in report.errors))

    def test_legacy_workflow_and_resource_requirements_are_preserved(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            workflow = root / 'qiongli-workflow/workflows/paper-read.md'
            workflow.parent.mkdir(parents=True)
            workflow.write_text('B2')
            with patch('tooling.scripts.validate_research_standard.WORKFLOW_TASK_EXPECTATIONS',
                       {'.agent/workflows/paper-read.md': {'B2'}}):
                report = ValidationReport()
                validate_workflow_entrypoints(root, report)
                self.assertTrue(any('missing global skill' in error for error in report.errors))
                workflow.write_text('qiongli-workflow B2')
                report = ValidationReport()
                validate_workflow_entrypoints(root, report)
                self.assertEqual(report.errors, [])
            shutil.rmtree(root / 'qiongli-workflow')
            report = ValidationReport()
            validate_skill_package_resource_links(root, report)
            self.assertTrue(any('package missing' in error for error in report.errors))

    def test_native_docs_use_the_canonical_generator_and_reject_drift(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            manifest = root / 'packages/qiongli-native/Cargo.toml'
            manifest.parent.mkdir(parents=True)
            manifest.touch()
            relative = 'docs/reference/skills.md'
            page = root / relative
            page.parent.mkdir(parents=True)
            page.write_text('Auto-generated native guide\n')
            with patch('tooling.scripts.validate_research_standard.generate_native_skill_docs',
                       return_value={relative: page.read_text()}) as native, \
                    patch('tooling.scripts.validate_research_standard.generate_skill_reference_docs') as legacy:
                report = ValidationReport()
                validate_generated_skill_docs(root, report)
                self.assertEqual(report.errors, [])
                native.assert_called_once_with(root)
                legacy.assert_not_called()
                page.write_text('Auto-generated stale guide\n')
                report = ValidationReport()
                validate_generated_skill_docs(root, report)
                self.assertTrue(any('out of sync' in error for error in report.errors))

    def test_legacy_docs_keep_the_legacy_generator(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            with patch('tooling.scripts.validate_research_standard.generate_native_skill_docs') as native, \
                    patch('tooling.scripts.validate_research_standard.generate_skill_reference_docs',
                          return_value={}) as legacy:
                validate_generated_skill_docs(root, ValidationReport())
                legacy.assert_called_once_with(root)
                native.assert_not_called()

    def test_strict_controller_mode_contract_reports_missing_files(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            report = ValidationReport()

            validate_controller_mode_contracts(root, report, strict=True)

        joined = "\n".join(report.errors)
        self.assertIn("standards/agent-run-contract.yaml", joined)
        self.assertIn("standards/worker-orchestration-contract.yaml", joined)
        self.assertIn("templates/worker-run-packet.json", joined)
        self.assertIn("templates/worker-review-packet.md", joined)
        self.assertIn("templates/worker-merge-report.md", joined)
        self.assertIn("tests/test_worker_orchestration_contract.py", joined)
        self.assertIn("scripts/audit_solo_role_gates.py", joined)

    def test_non_strict_controller_mode_contract_does_not_require_files(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            report = ValidationReport()

            validate_controller_mode_contracts(root, report, strict=False)

        self.assertEqual([], report.errors)

    def test_non_strict_literature_first_contract_does_not_require_files(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            report = ValidationReport()

            validate_literature_first_contracts(root, report, strict=False)

        self.assertEqual([], report.errors)

    def test_strict_literature_first_contract_reports_missing_files(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            report = ValidationReport()

            validate_literature_first_contracts(root, report, strict=True)

        joined = "\n".join(report.errors)
        self.assertIn("scripts/audit_literature_search_quality.py", joined)
        self.assertIn("scripts/materialize_literature_search_bundle.py", joined)
        self.assertIn("templates/search-diagnostics.md", joined)
        self.assertIn("qiongli-workflow/references/literature-search-quality-contract.md", joined)

    def test_non_strict_quality_gate_contract_does_not_require_files(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            report = ValidationReport()

            validate_quality_gate_contracts(root, report, strict=False)

        self.assertEqual([], report.errors)

    def test_strict_quality_gate_contract_reports_missing_files(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            report = ValidationReport()

            validate_quality_gate_contracts(root, report, strict=True)

        joined = "\n".join(report.errors)
        self.assertIn("standards/quality-gate-contract.yaml", joined)
        self.assertIn("scripts/audit_quality_gates.py", joined)
        self.assertIn("templates/quality-gate-report.md", joined)
        self.assertIn("tests/test_quality_gate_contract.py", joined)

    def test_non_strict_domain_method_pack_contract_does_not_require_files(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            report = ValidationReport()

            validate_domain_method_pack_contracts(root, report, strict=False)

        self.assertEqual([], report.errors)

    def test_strict_domain_method_pack_contract_reports_missing_files(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            report = ValidationReport()

            validate_domain_method_pack_contracts(root, report, strict=True)

        joined = "\n".join(report.errors)
        self.assertIn("scripts/audit_domain_method_packs.py", joined)
        self.assertIn("tests/test_domain_method_packs.py", joined)

    def test_boundary_review_gate_blocks_missing_required_sections(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            project = Path(tmp_dir)
            manuscript = project / "manuscript" / "main.md"
            boundary = project / "context" / "boundary_review.md"
            manuscript.parent.mkdir(parents=True, exist_ok=True)
            boundary.parent.mkdir(parents=True, exist_ok=True)
            manuscript.write_text("The intervention proves a causal effect.", encoding="utf-8")
            boundary.write_text(
                "# Boundary Review\n\n- locked_decision: Claims are associative only.\n",
                encoding="utf-8",
            )

            issues = validate_boundary_review(project)

        joined = "\n".join(issues)
        self.assertIn("boundary_review missing required marker: claim_strength_boundary", joined)
        self.assertIn("boundary_review missing required marker: evidence_threshold_boundary", joined)

    def test_boundary_review_gate_passes_when_claims_are_within_boundary(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            project = Path(tmp_dir)
            manuscript = project / "manuscript" / "main.md"
            boundary = project / "context" / "boundary_review.md"
            manuscript.parent.mkdir(parents=True, exist_ok=True)
            boundary.parent.mkdir(parents=True, exist_ok=True)
            manuscript.write_text(
                "The evidence suggests an associative relationship.",
                encoding="utf-8",
            )
            boundary.write_text(
                "\n".join(
                    [
                        "# Boundary Review",
                        "## Claim Strength And Evidence Threshold",
                        "- claim_strength_boundary: associative, not causal",
                        "- evidence_threshold_boundary: triangulated observational evidence",
                        "## Locked Decisions And Revisit Triggers",
                        "- locked_decision: Do not use causal language.",
                        "- revisit_trigger: new identification strategy or randomized evidence",
                    ]
                ),
                encoding="utf-8",
            )

            issues = validate_boundary_review(project)

        self.assertEqual([], issues)

    def test_strict_quality_gate_contract_reports_malformed_contract(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            required_files = {
                "standards/quality-gate-contract.yaml": "gates:\n  Q1: [unterminated\n",
                "scripts/audit_quality_gates.py": "placeholder\n",
                "templates/quality-gate-report.md": (
                    "# Quality Gate Report\n\n```yaml\n"
                    "gates: {}\n"
                    "```\n"
                ),
                "tests/test_quality_gate_contract.py": "placeholder\n",
            }
            for relative_path, content in required_files.items():
                path = root / relative_path
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(content, encoding="utf-8")
            report = ValidationReport()

            validate_quality_gate_contracts(root, report, strict=True)

        joined = "\n".join(report.errors)
        self.assertIn(
            "Quality gate contract failed to load: standards/quality-gate-contract.yaml",
            joined,
        )

    def test_strict_controller_mode_contract_runs_solo_gate_audit(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            for relative_path in (
                "standards/agent-run-contract.yaml",
                "standards/solo-role-policy.yaml",
                "templates/agent-run-packet.json",
                "templates/agent-review-packet.md",
                "templates/agent-handoff.md",
                "templates/solo-task-packet.md",
                "templates/solo-self-review.md",
                "templates/implementation-intent.md",
                "templates/writing-claim-map.md",
                "templates/quality-gate-report.md",
                "templates/worker-run-packet.json",
                "templates/worker-review-packet.md",
                "templates/worker-merge-report.md",
                "standards/worker-orchestration-contract.yaml",
                "tests/test_worker_orchestration_contract.py",
                "scripts/audit_solo_role_gates.py",
                "scripts/audit_agent_handoffs.py",
            ):
                path = root / relative_path
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text("placeholder\n", encoding="utf-8")
            run_path = root / "runs" / "bad-run.json"
            run_path.parent.mkdir(parents=True, exist_ok=True)
            run_path.write_text(
                '{"run_id": "bad-run", "execution_mode": "solo_codex"}',
                encoding="utf-8",
            )
            report = ValidationReport()

            validate_controller_mode_contracts(root, report, strict=True)

        joined = "\n".join(report.errors)
        self.assertIn("bad-run", joined)
        self.assertIn("verification_status", joined)

    def test_profile_bundle_accepts_antigravity_runtime_options(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            profile_path = root / "standards" / "agent-profiles.example.json"
            profile_path.parent.mkdir(parents=True, exist_ok=True)
            profile_path.write_text(
                """
{
  "profiles": {
    "default": {
      "runtime_options": {
        "codex": {"timeout_seconds": 300},
        "claude": {"timeout_seconds": 300},
        "antigravity": {"timeout_seconds": 300}
      }
    }
  }
}
""".strip(),
                encoding="utf-8",
            )
            report = ValidationReport()

            validate_profile_bundle_template(root, report)

        self.assertEqual([], report.errors)


if __name__ == "__main__":
    unittest.main()
