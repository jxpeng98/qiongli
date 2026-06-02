# 系统架构

Qiongli 现在采用 hybrid 仓库布局：学术内容、运行时代码、包壳、维护工具分别有清晰 source 边界；发布和安装所需的 payload 只在 staging/materialization 阶段生成。

## Source 边界

| 边界 | 可编辑源 | 职责 |
|---|---|---|
| 学术内容 | `content/` | workflow package source、internal skills、templates、standards、roles、subjects、schemas、venue profiles |
| Python runtime | `packages/python-qiongli/src/` | `qiongli`、弃用兼容的 `research_skills` shim、bridge adapters、CLI/runtime code |
| 包壳 | `packages/npm-qiongli/`、`packages/qiongli-plugin/`、`packages/qiongli-literature-mcpb/` | npm、plugin、MCPB 发布源 |
| 维护工具 | `tooling/scripts/`、`tooling/pipelines/`、`tooling/install/`、`tooling/release/` | 自动化、pipeline 描述、安装 manifest、release 资产 |
| 质量资产 | `evals/`、`tests/` | eval cases/runners 与跨包回归测试 |
| 文档 | `docs/` | VitePress 文档和维护者说明 |

根目录 `scripts/` 是兼容 wrapper。用户命令和 CI 可以继续使用 `scripts/...`，但维护者应编辑 `tooling/scripts/`。

根目录 `qiongli-workflow/`、`plugins/qiongli/`、`.agent/`、`.gemini/` 是生成后的 artifact 形状。要改源文件，请到 `content/workflow/` 或 `packages/qiongli-plugin/`。

## 分层模型

| 层 | 主要可编辑源 | 职责 |
|---|---|---|
| Contract | `content/standards/research-workflow-contract.yaml` | Task ID、产物路径、质量门 |
| Capability Map | `content/standards/mcp-agent-capability-map.yaml` | 运行时路由、MCP 与 skill 要求 |
| Functional Agents | `content/roles/` | 责任归属、质量阈值、语气 |
| Internal Skill Specs | `content/skills/` | 可复用执行行为 |
| Pipelines | `tooling/pipelines/` | 步骤编排与 handoff |
| Client entry UX | `content/workflow/workflows/`、`packages/qiongli-plugin/platforms/` | portable workflows 与平台命令入口 |
| Runtime | `packages/python-qiongli/src/qiongli/` | CLI、installer、orchestration、providers |
| Distribution | materialized staging tree | `qiongli-workflow/`、plugin payload、npm payload、Python payload |

## 稳定入口

| 入口方式 | 适用场景 | 稳定入口 |
|---|---|---|
| CLI install/upgrade | 安装与升级 assets | `qiongli`、`ql`、`research-skills`、`rsk`、`rsw` |
| Script entrypoints | CI、release、本地维护 | `scripts/*.py`、`scripts/*.sh` wrappers |
| Orchestrator CLI | 任务规划、执行、校验 | `python3 -m qiongli.bridges.orchestrator ...` |
| Portable skill package | 跨客户端分发 | 生成后的 `qiongli-workflow/` |
| Plugin package | Codex/Claude/Gemini plugin 分发 | 生成后的 `plugins/qiongli/` |

## 依赖方向

默认把系统看成单向依赖图：

1. `content/standards/`
2. `content/roles/` 与 `content/skills/`
3. `content/templates/`
4. `tooling/pipelines/` 与 platform command source
5. `packages/python-qiongli/src/qiongli/`
6. materialized distribution payloads

生成 payload 不能反过来成为隐藏真源。如果生成目录与 `content/` 或 `packages/` 不一致，应修源文件后重新 materialize。

精确目录职责见英文维护页 [Repository Structure](/development/repository-structure)。
