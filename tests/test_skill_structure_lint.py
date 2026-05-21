from __future__ import annotations

import tempfile
import textwrap
import unittest
from pathlib import Path

from scripts.validate_research_standard import (
    ValidationReport,
    validate_skill_quality_contract,
    validate_skill_structure,
)


class SkillStructureLintTests(unittest.TestCase):
    def _skill_doc(self, body: str) -> str:
        return textwrap.dedent(body).strip() + "\n"

    def _write_registry(self, root: Path, items: list[dict[str, object]]) -> None:
        registry_lines = ["skills:"]
        for item in items:
            registry_lines.extend(
                [
                    f"  - id: {item['id']}",
                    f"    stage: {item['stage']}",
                    f"    file: {item['file']}",
                    f"    canonical: {str(item.get('canonical', True)).lower()}",
                    f"    deprecated: {str(item.get('deprecated', False)).lower()}",
                    f"    alias_of: \"{item.get('alias_of', '')}\"",
                    f"    summary: \"{item.get('summary', '')}\"",
                    f"    display_name: \"{item.get('display_name', item['id'])}\"",
                    f"    when_to_use: \"{item.get('when_to_use', 'Use when needed.')}\"",
                    f"    summary_zh: \"{item.get('summary_zh', '摘要')}\"",
                    f"    display_name_zh: \"{item.get('display_name_zh', '显示名')}\"",
                    f"    when_to_use_zh: \"{item.get('when_to_use_zh', '需要时使用。')}\"",
                    "    inputs: []",
                    "    outputs: []",
                ]
            )
        (root / "skills").mkdir(parents=True, exist_ok=True)
        (root / "skills" / "registry.yaml").write_text("\n".join(registry_lines) + "\n", encoding="utf-8")

    def test_canonical_skill_lint_warns_for_size_section_sprawl_and_summary_duplication(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            self._write_registry(
                root,
                [
                    {
                        "id": "demo-skill",
                        "stage": "Z_cross_cutting",
                        "file": "skills/Z_cross_cutting/demo-skill.md",
                        "summary": "Canonical summary sentence.",
                    }
                ],
            )
            skill_path = root / "skills" / "Z_cross_cutting" / "demo-skill.md"
            skill_path.parent.mkdir(parents=True, exist_ok=True)
            sections = "\n".join(
                f"## Section {index}\n\nCanonical summary sentence.\n" for index in range(1, 35)
            )
            filler = "\n".join(f"Line {index}" for index in range(600))
            skill_path.write_text(
                self._skill_doc(
                    f"""\
                    ---
                    id: demo-skill
                    stage: Z_cross_cutting
                    description: "Canonical summary sentence."
                    ---

                    # Demo Skill

                    ## Purpose

                    Canonical summary sentence.

                    ## Process

                    {filler}

                    {sections}

                    ## When to Use

                    Use when needed.

                    ## Quality Bar

                    - [ ] Clear

                    ## Common Pitfalls

                    - Avoid drift
                    """
                ),
                encoding="utf-8",
            )

            report = ValidationReport()
            validate_skill_structure(root, report)

        warning_blob = "\n".join(report.warnings)
        self.assertIn("budget: 520", warning_blob)
        self.assertIn("budget: 32", warning_blob)
        self.assertIn("repeats the registry summary", warning_blob)

    def test_alias_skill_lint_rejects_fat_alias_without_canonical_pointer(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            self._write_registry(
                root,
                [
                    {
                        "id": "alias-skill",
                        "stage": "Z_cross_cutting",
                        "file": "skills/Z_cross_cutting/alias-skill.md",
                        "canonical": False,
                        "deprecated": True,
                        "alias_of": "canonical-skill",
                        "summary": "Alias summary.",
                    }
                ],
            )
            skill_path = root / "skills" / "Z_cross_cutting" / "alias-skill.md"
            skill_path.parent.mkdir(parents=True, exist_ok=True)
            body = "\n".join(f"## Extra {index}\n\nDetails.\n" for index in range(1, 8))
            skill_path.write_text(
                self._skill_doc(
                    f"""\
                    ---
                    id: alias-skill
                    stage: Z_cross_cutting
                    description: "Alias summary."
                    ---

                    # Alias Skill

                    ## Purpose

                    Short purpose.

                    ## Process

                    Some process.

                    {body}
                    """
                )
                + ("\nextra-line" * 120),
                encoding="utf-8",
            )

            report = ValidationReport()
            validate_skill_structure(root, report)

        error_blob = "\n".join(report.errors)
        self.assertIn("thin stubs", error_blob)
        self.assertIn("does not mention the canonical skill id", error_blob)

    def test_alias_skill_lint_accepts_thin_stub_with_canonical_pointer(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            self._write_registry(
                root,
                [
                    {
                        "id": "alias-skill",
                        "stage": "Z_cross_cutting",
                        "file": "skills/Z_cross_cutting/alias-skill.md",
                        "canonical": False,
                        "deprecated": True,
                        "alias_of": "canonical-skill",
                        "summary": "Alias summary.",
                    }
                ],
            )
            skill_path = root / "skills" / "Z_cross_cutting" / "alias-skill.md"
            skill_path.parent.mkdir(parents=True, exist_ok=True)
            skill_path.write_text(
                self._skill_doc(
                    """\
                    ---
                    id: alias-skill
                    stage: Z_cross_cutting
                    description: "Alias summary."
                    ---

                    # Alias Skill

                    ## Purpose

                    This is a thin alias stub. Use `canonical-skill` for the canonical implementation.

                    ## Process

                    Redirect to `canonical-skill` and avoid maintaining duplicate logic here.

                    ## When to Use

                    Only when following older references.

                    ## Common Pitfalls

                    - Do not expand this alias
                    """
                ),
                encoding="utf-8",
            )

            report = ValidationReport()
            validate_skill_structure(root, report)

        self.assertEqual(report.errors, [])
        self.assertEqual(report.warnings, [])

    def test_skill_quality_contract_warns_for_missing_required_sections(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            skill_path = root / "skills" / "F_writing" / "weak-skill.md"
            skill_path.parent.mkdir(parents=True, exist_ok=True)
            skill_path.write_text(
                self._skill_doc(
                    """\
                    ---
                    id: weak-skill
                    stage: F_writing
                    ---

                    # Weak Skill

                    ## Purpose

                    Improve writing.
                    """
                ),
                encoding="utf-8",
            )

            report = ValidationReport()
            validate_skill_quality_contract(root, report)

        warning_blob = "\n".join(report.warnings)
        self.assertIn("skill quality contract gaps", warning_blob)
        self.assertIn("weak-skill.md", warning_blob)

    def test_gate_and_method_pack_consumers_reference_required_contract_fields(self) -> None:
        root = Path(__file__).resolve().parents[1]
        required_tokens = {
            "skills/C_design/study-designer.md": [
                "quality-gate-contract.yaml",
                "Q1",
                "design/validity-threat-matrix.md",
            ],
            "skills/C_design/robustness-planner.md": [
                "quality-gate-contract.yaml",
                "method_templates",
                "required_diagnostics",
            ],
            "skills/I_code/stats-engine.md": [
                "method_templates",
                "required_diagnostics",
                "minimum_report_fields",
            ],
            "skills/I_code/code-builder.md": [
                "method_templates",
                "required_artifacts",
                "failure_modes",
            ],
            "skills/I_code/code-review.md": [
                "quality-gate-contract.yaml",
                "failure_modes",
                "minimum_report_fields",
            ],
        }

        missing_tokens: list[str] = []
        for relative_path, tokens in required_tokens.items():
            text = (root / relative_path).read_text(encoding="utf-8")
            missing_tokens.extend(
                f"{relative_path}: {token}" for token in tokens if token not in text
            )

        self.assertEqual(missing_tokens, [])

    def test_method_pack_consumers_do_not_duplicate_domain_specific_rules(self) -> None:
        root = Path(__file__).resolve().parents[1]
        forbidden_tokens = {
            "skills/I_code/stats-engine.md": [
                "Domain-Specific Diagnostic Quick Reference",
            ],
            "skills/I_code/code-review.md": [
                "Domain-Specific Review Rules",
            ],
        }
        required_tokens = {
            "skills/I_code/stats-engine.md": [
                "skills/domain-profiles/[domain].yaml",
                "method_templates[*].required_diagnostics",
                "minimum_report_fields",
            ],
            "skills/I_code/code-review.md": [
                "skills/domain-profiles/[domain].yaml",
                "method_templates[*].required_diagnostics",
                "failure_modes",
                "minimum_report_fields",
            ],
        }

        failures: list[str] = []
        for relative_path, tokens in forbidden_tokens.items():
            text = (root / relative_path).read_text(encoding="utf-8")
            failures.extend(
                f"{relative_path}: remove {token}" for token in tokens if token in text
            )
        for relative_path, tokens in required_tokens.items():
            text = (root / relative_path).read_text(encoding="utf-8")
            failures.extend(
                f"{relative_path}: add {token}" for token in tokens if token not in text
            )

        self.assertEqual(failures, [])


if __name__ == "__main__":
    unittest.main()
