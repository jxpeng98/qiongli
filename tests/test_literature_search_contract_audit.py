import re
import unittest
from pathlib import Path

from bridges.providers.literature_schema import QUERY_PLAN_REQUIRED_KEYS


REPO_ROOT = Path(__file__).resolve().parents[1]
SEARCH_STRATEGY_TEMPLATE_PATHS = (
    Path("templates/search-strategy.md"),
    Path("plugins/qiongli/skills/qiongli-workflow/templates/search-strategy.md"),
)


def fenced_yaml_blocks(markdown: str) -> list[str]:
    return re.findall(r"```yaml\s*\n(.*?)\n```", markdown, flags=re.DOTALL)


def machine_readable_search_plan_blocks(markdown: str) -> list[str]:
    heading = re.search(r"^## Machine-Readable Search Plan\s*$", markdown, flags=re.MULTILINE)
    if not heading:
        return []

    following_section = re.split(
        r"^##\s+",
        markdown[heading.end() :],
        maxsplit=1,
        flags=re.MULTILINE,
    )[0]
    return fenced_yaml_blocks(following_section)


def top_level_keys(block: str) -> set[str]:
    return {
        match.group(1)
        for match in re.finditer(r"^([A-Za-z_][A-Za-z0-9_]*):", block, flags=re.MULTILINE)
    }


class LiteratureSearchContractAuditTests(unittest.TestCase):
    def test_search_strategy_templates_include_machine_readable_query_plan(self):
        required_keys = set(QUERY_PLAN_REQUIRED_KEYS)

        for relative_path in SEARCH_STRATEGY_TEMPLATE_PATHS:
            with self.subTest(template=str(relative_path)):
                template = (REPO_ROOT / relative_path).read_text(encoding="utf-8")
                search_plan_blocks = [
                    block
                    for block in machine_readable_search_plan_blocks(template)
                    if required_keys.issubset(top_level_keys(block))
                ]

                self.assertTrue(
                    search_plan_blocks,
                    msg=(
                        f"{relative_path} must include a fenced yaml query-plan block under "
                        "`## Machine-Readable Search Plan` with top-level keys: "
                        f"{sorted(required_keys)}"
                    ),
                )

                search_plan = search_plan_blocks[0]
                expected_nested_lines = (
                    r"(?m)^\s*-\s+id:\s+c1_population\s*$",
                    r"(?m)^\s+label:\s+Population or corpus\s*$",
                    r"(?m)^\s*-\s+provider:\s+semantic_scholar\s*$",
                    r"(?m)^\s+query_id:\s+q1\s*$",
                    r"(?m)^\s+translated_query:\s+\"[^\"]+\"\s*$",
                    r"(?m)^\s+filters:\s+\{\}\s*$",
                    r"(?m)^\s+rationale:\s+.+\s*$",
                    r"(?m)^\s+max_rounds:\s+2\s*$",
                    r"(?m)^\s+stop_when_new_included_below:\s+3\s*$",
                )
                for pattern in expected_nested_lines:
                    self.assertRegex(search_plan, pattern)


if __name__ == "__main__":
    unittest.main()
