from pathlib import Path
import json
import subprocess
import unittest


ROOT = Path(__file__).resolve().parents[1]


class DataLifecyclePolicyTests(unittest.TestCase):
    def test_rel_905_policy_is_complete_discoverable_and_source_bound(self) -> None:
        english = (ROOT / "docs/guide/data-lifecycle.md").read_text()
        chinese = (ROOT / "docs/zh/guide/data-lifecycle.md").read_text()
        release_policy = (ROOT / "docs/maintainer/release-branch-policy.md").read_text()
        english_index = (ROOT / "docs/guide/index.md").read_text()
        chinese_index = (ROOT / "docs/zh/guide/index.md").read_text()
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
            "## 可迁移的项目导出",
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
            "first stable release publication date",
        ):
            self.assertIn(contract, english.replace("\n", " "))

        self.assertIn(
            "The planned 1.x support window ends **90 days after Qiongli 2 stable**",
            release_policy,
        )
        self.assertIn("[Data Ownership and Lifecycle](/guide/data-lifecycle)", english_index)
        self.assertIn("[数据所有权与生命周期](/zh/guide/data-lifecycle)", chinese_index)
        navigation = subprocess.run(
            ["node", "--input-type=module", "-e",
             "import config from './docs/.vitepress/config.mjs'; "
             "console.log(JSON.stringify(Object.values(config.locales).flatMap(locale => "
             "Object.values(locale.themeConfig.sidebar).flatMap(groups => "
             "groups.flatMap(group => group.items.map(item => item.link))))));"],
            cwd=ROOT, text=True, capture_output=True, check=True,
        )
        links = json.loads(navigation.stdout)
        self.assertIn("/guide/data-lifecycle", links)
        self.assertIn("/zh/guide/data-lifecycle", links)
        self.assertIn("tests.test_data_lifecycle_policy", workflow)

    def test_private_chat_retention_and_recovery_policy_is_bilingual(self) -> None:
        for path in ("docs/guide/data-lifecycle.md", "docs/zh/guide/data-lifecycle.md"):
            text = (ROOT / path).read_text()
            for term in ("<project>/.qiongli/all-chat/run_*.json", ".all-chat-session.lock", ".all-chat.lock", "32", "64", "2,048", "2,304", "8 MiB", "load/resume"):
                self.assertIn(term.lower(), text.lower())
        native = (ROOT / "packages/qiongli-native/apps/qiongli/src/all_chat_history.rs").read_text()
        self.assertIn("all_chat_history_recovers_committed_intent_without_replay_and_excludes_export", native)
        diagnostics = (ROOT / "packages/qiongli-native/apps/qiongli/src/product_diagnostics.rs").read_text()
        self.assertIn("PRIVATE_CHAT_DIAGNOSTIC_CANARY", diagnostics)


if __name__ == "__main__":
    unittest.main()
