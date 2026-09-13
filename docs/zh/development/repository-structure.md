# 仓库结构

`main` 保存整合后的原生产品，`2.x` 继续用于开发和预发布。
旧 Python 运行时保留用于兼容工作，新的 CLI 与 MCP 行为在 Rust 工作区实现。

| 路径 | 负责内容 |
|---|---|
| `content/workflow/` | 主 Skill、快捷入口和阶段参考 |
| `content/skills/`、`roles/`、`templates/`、`subjects/` | 均位于 `content/` 下，保存研究指导和可复用产物结构 |
| `content/standards/` | 研究任务与产物契约 |
| `content/mcp-contracts/` | MCP 工具、参数契约和测试样例 |
| `content/distribution/` | 共享 Plugin 元数据 |
| `packages/qiongli-native/apps/qiongli/` | 原生产品入口、CLI 和 Host 适配 |
| `packages/qiongli-native/crates/` | 共享领域、项目、运行时及安装服务 |
| `packages/qiongli-zotero-companion/` | 安装到 Zotero 内部的扩展 |
| `tooling/scripts/native_*.py` | 原生发布、包管理器和 Marketplace 打包 |
| `tooling/release/` | 发布契约与证据 |
| `docs/`、`docs/zh/` | 英文与中文文档 |
| `docs/architecture/decisions/` | 已接受和已被替代的架构决策 |
| `docs/superpowers/` | 计划、路线图和证据账本 |
| `tests/`、`evals/` | 回归与行为检查 |

## 保留的旧版与桌面源码

`packages/qiongli-desktop/` 和 `packages/qiongli-app-api/` 支持保留的 Svelte/Tauri
桌面端。桌面维护与当前 CLI 交付分别处理。
`packages/python-qiongli/`、`packages/npm-qiongli/`、`packages/qiongli-lite-mcp/`
及旧 MCPB 源码保留用于兼容，不负责生成原生渠道包。
这些目录里的旧 README 也不是当前原生 npm 或 Python 包随附的说明。

## 生成结果

Plugin 目录、`qiongli-workflow/`、`.agent/`、包内资源和已安装缓存都由源文件生成，
不应直接编辑。原生包在临时目录构建，共用 CLI 元数据，但各渠道保留自己的启动包装和安装说明。

根目录 `scripts/` 提供稳定命令入口，真正的实现位于 `tooling/scripts/`。
旧版内容生成方式保留在已标注版本的[兼容指南](../../development/distribution-materialization.md)。
新增实现前，请先看[编辑约定](../conventions.md)。
