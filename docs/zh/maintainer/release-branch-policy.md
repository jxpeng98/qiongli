# 发布分支策略

本仓库使用 `dev` 承接日常开发，`main` 只保留稳定发布版本。

## 分支职责

| 分支 | 职责 | 允许的变更 |
|------|------|------------|
| `dev` | 活跃开发与集成 | 功能、修复、预发布 plugin 打包、文档、测试、CI 加固。 |
| `main` | 稳定发布源 | release-prep commit、稳定 tag、postflight acceptance receipt，以及已经可发布的紧急修复。 |

普通 PR 应该合入 `dev`。只有当下一版 release candidate 已经通过发布门禁、plugin 包也准备好进入稳定发布时，才把 `dev` 合入 `main`。

## 官方 Plugin 接入

公开的官方 marketplace 条目现在由 `jxpeng98/skillsplace` 统一维护，并指向稳定的 Qiongli plugin payload：

- Marketplace repository: `https://github.com/jxpeng98/skillsplace`
- Qiongli repository: `https://github.com/jxpeng98/qiongli`
- Plugin subdirectory: `packages/qiongli-plugin`
- Codex manifest: `packages/qiongli-plugin/.codex-plugin/plugin.json`
- Claude Code manifest: `packages/qiongli-plugin/.claude-plugin/plugin.json`
- Gemini extension manifest: `packages/qiongli-plugin/gemini-extension.json`

Skillsplace catalog 应跟踪 `main` 和 release tag，而不是 `dev`。`dev` 用于本地 plugin packaging 测试和预发布验证，验证完成后再更新统一 marketplace 入口。本仓库不再携带 Codex 或 Claude marketplace catalog 文件，只负责 plugin manifest，并从 canonical source materialize release payload。

## 开发流程

1. 功能和 packaging 工作从 `dev` 开始。
2. 验证 artifact 前，把 portable skill package materialize 到 staging 目录：

```bash
python3 scripts/materialize_distribution_payloads.py --target all --out /tmp/qiongli-dist --force
```

3. 在 `dev` 上运行常规验证：

```bash
python3 scripts/validate_research_standard.py --strict
python3 -m unittest discover -s tests -v
```

4. 如需检查预发布打包，用目标 tag 构建 plugin artifacts：

```bash
python3 scripts/build_plugin_artifacts.py --tag v0.7.0-beta.2 --dist-dir dist
```

5. 只有当 CI、install checks 和 release preflight 都通过后，才把变更合入 `main`。

## 稳定发布规则

只有 `main` 应创建稳定 release tag 和公开 plugin artifacts；统一的 Skillsplace 条目也应在 release gates 通过后再推进。release automation 已要求 publish mode 必须从 primary branch 运行。beta 和 release-candidate 工作保留在 `dev`，直到它可以成为稳定发布。

beta 不是每个 stable release 的必经步骤。只有当 release 改动发布自动化、package payload、installer、package metadata、CI 或 publish workflows 这类高风险面时，才需要先用 beta 验证。低风险文档、小修复和维护改动可以直接从 `main` 发 stable。若 stable 没有对应的新 beta，npm `latest` 会前进，npm `next` 会有意停在上一个 beta；`next` 表示最新预发布验证版，不是必须始终比 stable 更新的通道。
