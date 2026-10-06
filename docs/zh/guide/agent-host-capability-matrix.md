# Agent Host 实测能力矩阵

当前开发版的子代理、跨 Host 提案和可选 Hook 说明见 [Agent 协作](/zh/advanced/agent-skill-collaboration)。本页历史矩阵不为这些新变更提供验收结论。

本页只报告截至 2026 年 8 月 30 日已接受 Qiongli receipt 直接观察到的能力。
它不是厂商比较、模型排名，也不承诺一个 Host 与另一个 Host 等价。

状态含义：

- **已观察存在** — 引用的 receipt 直接证明该能力存在。
- **已观察不存在** — 引用的 receipt 直接证明该能力不存在。
- **未观察** — 没有已接受 receipt 证明存在或不存在；这不等于“不支持”。

## 2.4 开发观察

[2.4 候选计划](../../superpowers/plans/2026-10-05-v2-4-plugin-quality-release-plan.md)
单独记录后续开发观察，不改写下方已接受的八月收据。这些结果只适用于计划中列明的
开发源版本，不能直接为变化后的最终候选版提供验收结论。

| Host | 开发证据 | 尚需验证 |
| --- | --- | --- |
| Codex CLI | `85758cac` 的安装后回归在结构与内容审查上均为 3/3，新笔记保存与重启恢复为 1/1，一组冻结的新合成材料为 3/3；指导加载、读取策略与清理均已核对 | 仅覆盖指定任务和已提供的指导路径，不代表自动发现、全文阅读或广泛科研验收；旧失败保留在计划中。 |
| Antigravity CLI | 保留此前 1.2.17 安装更新、Skill 读取及原生返回的限定观察；`85758cac` 在隔离 1.3.0 中的安装启用和源文件/缓存校验也已通过 | 新的 1.3.0 配置需要登录，尚未调用 MCP；完整原始返回、成功结束及科研续接仍待验证。 |
| Claude Code / DeepSeek | `c2cae736` 的 Linux ARM64 检查已通过 Claude 归档及 npm DeepSeek 内容投影 | 其他目标平台仍待验证；本轮没有新的模型任务结论。 |

Antigravity 使用 CLI Plugin 格式及 Host 正常信任审批流程；注册期间保留源目录，
更新后新开会话。参见[安装说明](../advanced/plugin-installation.md#antigravity)。

原生源码 `c2cae736` 的本地 Linux ARM64 检查已通过 Codex/Claude Plugin 归档及
npm DeepSeek 内容投影。归档 CLI 导出的 Antigravity Plugin 包含 22 个 Skill 入口，
文件字节和权限均通过核对；其内置程序通过原生 stdio 列出 35 个 Full MCP 工具，
并返回完整的 `qiongli_config_status` 结果。这些证据仅覆盖软件包与原生传输，
此归档程序观察未注册 Plugin 或运行 AGY 会话；另行执行的 1.3.0 安装和登录结果
见上表。最终源码的四平台验收仍待完成。

## 安装与运行时入口

| Host | Plugin 生命周期 | Skill 发现 | Lite MCP | Full MCP | 清理 |
|---|---|---|---|---|---|
| Codex CLI | 已观察存在 | 已观察存在 | 已观察存在 | 已观察存在 | 已观察存在 |
| Claude Code | 已观察存在 | 已观察存在 | 已观察存在 | 已观察存在 | 已观察存在 |
| Codex Desktop | 未观察 | 未观察 | 未观察 | 未观察 | 未观察 |
| Claude Desktop | 未观察 | 未观察 | 未观察 | 未观察 | 未观察 |
| Antigravity | 未观察 | 未观察 | 未观察 | 未观察 | 未观察 |
| Generic local MCP Host（通用本地 MCP Host） | 未观察 | 未观察 | 未观察 | 未观察 | 未观察 |

## 已认证模型旅程

| Host | 模型运行 | 项目读取 | Graph 读取 | 结构化输出 | 原生子 Agent | 不保留对话 |
|---|---|---|---|---|---|---|
| Codex CLI | 已观察存在 | 已观察存在 | 已观察存在 | 已观察存在 | 已观察不存在 | 已观察存在 |
| Claude Code | 未观察 | 未观察 | 未观察 | 未观察 | 未观察 | 未观察 |
| Codex Desktop | 未观察 | 未观察 | 未观察 | 未观察 | 未观察 | 未观察 |
| Claude Desktop | 未观察 | 未观察 | 未观察 | 未观察 | 未观察 | 未观察 |
| Antigravity | 未观察 | 未观察 | 未观察 | 未观察 | 未观察 | 未观察 |
| Generic local MCP Host（通用本地 MCP Host） | 未观察 | 未观察 | 未观察 | 未观察 | 未观察 | 未观察 |

## 证据边界

| Receipt | 精确观察范围 |
|---|---|
| [Codex 与 Claude MCP 兼容性](../../superpowers/acceptance/2026-08-24-qiongli-codex-claude-mcp-compatibility.md) | 产品源码 `192ad24fb175f1eaa7c289dfa916f2b5543bfa70`；Codex CLI `0.147.0` 与 Claude Code `2.1.237`；隔离 Plugin、Skill、Lite/Full MCP 与清理兼容性 |
| [PILOT-903 真实项目 receipt](https://github.com/jxpeng98/qiongli/blob/5a3ab87fcba67dfbe700f895c0321e455bbbc914/docs/superpowers/acceptance/2026-08-30-qiongli-pilot903-real-project-receipt.json) | 产品源码 `d0b4113364452d6ff8ff7cb2a3735e7c8d40d3f8`；Codex CLI `0.147.0`；已认证 Skill + Full MCP 项目/Graph 旅程、结构化输出、隐私与回滚 |

两个 receipt 都没有记录精确模型标识，因此模型身份是**未记录**。Claude
兼容性 receipt 不证明已认证 Claude 模型旅程；Codex CLI 证据不能用于认定
Codex Desktop，Claude Code 证据也不能用于认定 Claude Desktop。历史 receipt
只对其命名的源码与范围有效，不能认定发生变更后的 release candidate。

规范的机器可读投影是
[PILOT-905 矩阵 receipt](https://github.com/jxpeng98/qiongli/blob/5a3ab87fcba67dfbe700f895c0321e455bbbc914/docs/superpowers/acceptance/2026-08-30-qiongli-pilot905-host-capability-matrix.json)。
`publicationAllowed` 仍为 `false`。
