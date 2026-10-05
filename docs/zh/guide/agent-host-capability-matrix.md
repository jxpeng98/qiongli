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
| Codex CLI | 隔离安装、收据与缓存校验，以及新一轮三个科研任务记录 | 最新完整通过为 0/3；需在可用沙箱中补齐指导加载、有效工具策略及成功恢复证据。 |
| Antigravity CLI | 原生安装更新、Skill 入口读取、Full 35 工具发现及一次原生结果返回 | 完整原始返回与正常退出仍未闭合，科研续接尚未验证。 |
| Claude Code / DeepSeek | 保留现有共享 Plugin 与软件包适配器 | 需要最终候选版的兼容和软件包检查；本轮没有新的模型任务结论。 |

Antigravity 使用 CLI Plugin 格式及 Host 正常信任审批流程；注册期间保留源目录，
更新后新开会话。参见[安装说明](../advanced/plugin-installation.md#antigravity)。

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
