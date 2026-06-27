from __future__ import annotations

import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]


DOC_PATHS = (
    Path("README.md"),
    Path("README_CN.md"),
    Path("docs/guide/install.md"),
    Path("docs/zh/guide/install.md"),
    Path("docs/reference/cli.md"),
    Path("docs/zh/reference/cli.md"),
)


class CLISetupDocsTests(unittest.TestCase):
    def test_cli_setup_docs_cover_wizard_flags_and_choices(self) -> None:
        for path in DOC_PATHS:
            content = (REPO_ROOT / path).read_text(encoding="utf-8")
            with self.subTest(path=str(path)):
                for token in (
                    "qiongli setup",
                    "--dry-run",
                    "--no-doctor",
                    "subject",
                    "coverage",
                    "--mode",
                    "--overwrite",
                ):
                    self.assertIn(token, content)
                self.assertRegex(content, re.compile(r"install.*upgrade|安装.*升级", re.S))
                self.assertRegex(content, re.compile(r"CLI (directory|目录)|shell CLI"))
                self.assertRegex(content, re.compile(r"provider (config|配置)|provider config"))

    def test_cli_setup_docs_keep_scriptable_npm_install_examples(self) -> None:
        readme_default_install = 'qiongli install --target all --project-dir "$PWD"'
        guide_core_install = 'qiongli install --subject core --target all --project-dir "$PWD"'

        for path in (Path("README.md"), Path("README_CN.md")):
            content = (REPO_ROOT / path).read_text(encoding="utf-8")
            with self.subTest(path=str(path)):
                self.assertIn("qiongli setup", content)
                self.assertIn(readme_default_install, content)

        for path in (Path("docs/guide/install.md"), Path("docs/zh/guide/install.md")):
            content = (REPO_ROOT / path).read_text(encoding="utf-8")
            with self.subTest(path=str(path)):
                self.assertIn("qiongli setup", content)
                self.assertIn(guide_core_install, content)

    def test_cli_setup_docs_disclose_npm_full_runtime_boundary(self) -> None:
        for path in (
            Path("README.md"),
            Path("README_CN.md"),
            Path("docs/guide/install.md"),
            Path("docs/zh/guide/install.md"),
            Path("docs/reference/cli.md"),
            Path("docs/zh/reference/cli.md"),
            Path("packages/npm-qiongli/README.md"),
        ):
            content = (REPO_ROOT / path).read_text(encoding="utf-8")
            setup_snippet = self._extract_setup_snippet(content)
            with self.subTest(path=str(path)):
                self.assertRegex(
                    setup_snippet,
                    re.compile(r"Python-free asset manager|免 Python 资产管理器|无 Python 资产管理器"),
                )
                self.assertIn("pipx install qiongli", setup_snippet)
                self.assertRegex(
                    setup_snippet,
                    re.compile(r"full runtime commands|完整运行时命令|full runtime"),
                )
                self.assertIn("qiongli install", setup_snippet)
                self.assertNotIn("Python 3.12+", setup_snippet)
                self.assertNotIn("PyYAML", setup_snippet)
                self.assertNotRegex(setup_snippet, re.compile(r"Python bridge|Python 桥"))

    def test_cli_setup_reference_documents_provider_boundaries(self) -> None:
        expectations = {
            Path("docs/reference/cli.md"): (
                (r"doctor/capability checks", r"\b[Ss]ecrets\b", r"generated research artifacts"),
                r"provider config",
            ),
            Path("docs/zh/reference/cli.md"): (
                (r"doctor|验证|检查", r"capability|能力", r"secrets|密钥|凭据", r"generated research artifacts|生成的研究"),
                r"provider (config|配置)|provider config",
            ),
        }
        for path, (boundary_patterns, provider_pattern) in expectations.items():
            content = (REPO_ROOT / path).read_text(encoding="utf-8")
            setup_section = self._extract_setup_section(content)
            with self.subTest(path=str(path)):
                for token in ("qiongli provider setup", "qiongli provider doctor"):
                    self.assertIn(token, setup_section)
                for pattern in boundary_patterns:
                    self.assertRegex(setup_section, re.compile(pattern))
                self.assertRegex(setup_section, re.compile(provider_pattern))

    def test_cli_setup_sections_do_not_frame_desktop_or_mcpb_installation(self) -> None:
        for path in DOC_PATHS:
            content = (REPO_ROOT / path).read_text(encoding="utf-8")
            setup_snippet = self._extract_setup_snippet(content)
            with self.subTest(path=str(path)):
                self.assertNotRegex(setup_snippet, re.compile(r"\bDesktop\b|MCPB|provider companion"))

    def _extract_setup_section(self, content: str) -> str:
        match = re.search(
            r"### 2\.2 `qiongli setup`.*?(?=\n### 2\.3 `qiongli install`)",
            content,
            flags=re.S,
        )
        self.assertIsNotNone(match)
        return match.group(0)

    def _extract_setup_snippet(self, content: str) -> str:
        for heading in ("Recommended CLI Setup Wizard", "推荐的 CLI Setup Wizard"):
            marker = content.find(heading)
            if marker != -1:
                end = content.find("\n##", marker + len(heading))
                return content[marker:] if end == -1 else content[marker:end]

        marker = content.find("qiongli setup")
        self.assertNotEqual(marker, -1)
        start = max(0, marker - 500)
        end = min(len(content), marker + 1500)
        return content[start:end]


if __name__ == "__main__":
    unittest.main()
