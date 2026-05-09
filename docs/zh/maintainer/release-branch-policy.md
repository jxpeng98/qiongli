# 发布分支策略

本仓库使用 `dev` 承接日常开发，`main` 只保留稳定发布版本。

## 分支职责

| 分支 | 职责 | 允许的变更 |
|------|------|------------|
| `dev` | 活跃开发与集成 | 功能、修复、预发布 plugin 打包、文档、测试、CI 加固。 |
| `main` | 稳定发布源 | release-prep commit、稳定 tag、postflight acceptance receipt，以及已经可发布的紧急修复。 |

普通 PR 应该合入 `dev`。只有当下一版 release candidate 已经通过发布门禁、plugin 包也准备好进入稳定发布时，才把 `dev` 合入 `main`。

## 官方 Plugin 接入

官方 plugin marketplace 条目应指向稳定仓库身份：

- Repository: `https://github.com/jxpeng98/qiongli`
- Codex plugin catalog: `.agents/plugins/marketplace.json`
- Codex manifest: `plugins/qiongli/.codex-plugin/plugin.json`
- Claude Code marketplace: `.claude-plugin/marketplace.json`
- Claude Code manifest: `plugins/qiongli/.claude-plugin/plugin.json`
- Gemini extension manifest: `plugins/qiongli/gemini-extension.json`

官方 plugin marketplace 应跟踪 `main` 和 release tag，而不是 `dev`。`dev` 用于本地 marketplace 测试和预发布验证，验证完成后再更新官方入口。

## 开发流程

1. 功能和 packaging 工作从 `dev` 开始。
2. 验证前同步 portable skill package：

```bash
bash scripts/sync_skill_package.sh --target all
```

3. 在 `dev` 上运行常规验证：

```bash
python3 scripts/validate_research_standard.py --strict
python3 -m unittest discover -s tests -v
```

4. 如需检查预发布打包，用目标 tag 构建 marketplace artifacts：

```bash
python3 scripts/build_marketplace_artifacts.py --tag v0.7.0-beta.2 --dist-dir dist
```

5. 只有当 CI、install checks 和 release preflight 都通过后，才把变更合入 `main`。

## 稳定发布规则

只有 `main` 应创建稳定 release tag 和官方 marketplace artifacts。release automation 已要求 publish mode 必须从 primary branch 运行。beta 和 release-candidate 工作保留在 `dev`，直到它可以成为稳定发布。
