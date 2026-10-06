# 扩展 Qiongli

从真实的研究或安装问题出发，先找到负责这一行为的源文件，再用现有构建流程更新各入口。
已有工作流能完成的任务，不必另加一套工作流或运行时。

## 选择修改位置

| 需要修改什么 | 源文件位置 |
|---|---|
| 任务产物与研究要求 | `content/standards/` |
| 可复用的研究指导 | `content/skills/`、`content/roles/`、`content/templates/` |
| 主 Skill、快捷入口与阶段参考 | `content/workflow/` |
| Plugin 元数据 | `content/distribution/plugins.yaml` |
| MCP 工具名称和参数契约 | `content/mcp-contracts/` |
| 原生 CLI、MCP、项目、Graph 或安装行为 | `packages/qiongli-native/` |
| 原生包组装与发布检查 | `tooling/scripts/native_*.py` 及对应工作流 |
| 用户文档 | `docs/` 与 `docs/zh/` 下的中文对应页 |

Python 和旧 npm 产品目录保留用于兼容，不是修复 2.x 运行时的入口。
Plugin、已安装缓存和嵌入资源包都是生成结果，应从源文件重新构建，不能直接修改。

## 让指导与任务相称

Skill 应说明目标、证据要求和边界，再让模型选择合适的方法。
只回答一个小问题，不必执行整篇论文的生命周期；协议要求、来源归属、必要的独立审查，
以及项目写入前的批准仍须保留。

快捷入口复用主 Skill 和共享阶段参考。新增内部 Skill 时，同步注册信息和受影响的任务映射，
只补充解释这项独立任务所需的模板与示例。学科差异可以先看是否适合放进
[学科指导](subject-packaging-model.md)。

## 检查并整合

按仓库的 `AGENTS.md`、`CONTRIBUTING.md` 和当前计划推进。
运行最贴近变更的内容、CLI 或打包检查，保留拒绝操作和防止数据丢失的测试。
文档修改后运行 `npm run docs:build`，用当前 CLI 核对命令，中英文同步更新。

审阅差异，将检查结果和剩余缺口集中记录到计划与进度账本，再进行本地提交和整合。
测试通过不等于 Host 实际使用或发布验收通过；发布遵循单独的
[发布策略](../maintainer/release-branch-policy.md)。
