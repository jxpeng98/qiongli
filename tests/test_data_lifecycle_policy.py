from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[1]


class DataLifecyclePolicyTests(unittest.TestCase):
    def test_rel_905_policy_is_complete_discoverable_and_source_bound(self) -> None:
        english = (ROOT / "docs/guide/data-lifecycle.md").read_text()
        chinese = (ROOT / "docs/zh/guide/data-lifecycle.md").read_text()
        release_policy = (ROOT / "docs/maintainer/release-branch-policy.md").read_text()
        english_index = (ROOT / "docs/guide/index.md").read_text()
        chinese_index = (ROOT / "docs/zh/guide/index.md").read_text()
        vitepress = (ROOT / "docs/.vitepress/config.mjs").read_text()
        workflow = (ROOT / ".github/workflows/evaluation-truth.yml").read_text()

        for heading in (
            "## Ownership Boundary",
            "## Backup and Restore",
            "## Portable Project Export",
            "## Uninstall and Deletion",
            "## Qiongli 1.x End of Support",
        ):
            self.assertIn(heading, english)

        for heading in (
            "## 所有权边界",
            "## 备份与恢复",
            "## Portable 项目导出",
            "## 卸载与删除",
            "## 1.x 支持终止",
        ):
            self.assertIn(heading, chinese)

        for contract in (
            "<project>/.qiongli/v2",
            "<user-home>/.config/qiongli/v2",
            "$QIONGLI_CONFIG_HOME/v2",
            "qiongli project export preview",
            "not a complete backup",
            "These operations do not delete project directories",
            "v1.19.0-beta.1",
            "90 days after Qiongli 2 Stable is published",
            "there is no calendar end date yet",
        ):
            self.assertIn(contract, english.replace("\n", " "))

        self.assertIn(
            "The planned 1.x support window ends **90 days after Qiongli 2 stable**",
            release_policy,
        )
        self.assertIn("[Data Ownership and Lifecycle](/guide/data-lifecycle)", english_index)
        self.assertIn("[数据所有权与生命周期](/zh/guide/data-lifecycle)", chinese_index)
        self.assertIn("/guide/data-lifecycle", vitepress)
        self.assertIn("/zh/guide/data-lifecycle", vitepress)
        self.assertIn("tests.test_data_lifecycle_policy", workflow)


if __name__ == "__main__":
    unittest.main()
