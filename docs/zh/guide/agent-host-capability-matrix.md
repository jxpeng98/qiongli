# Agent Host 实测能力矩阵

本页只报告截至 2026 年 8 月 30 日已接受 Qiongli receipt 直接观察到的能力。
它不是厂商比较、模型排名，也不承诺一个 Host 与另一个 Host 等价。

状态含义：

- **已观察存在** — 引用的 receipt 直接证明该能力存在。
- **已观察不存在** — 引用的 receipt 直接证明该能力不存在。
- **未观察** — 没有已接受 receipt 证明存在或不存在；这不等于“不支持”。

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
| [PILOT-903 真实项目 receipt](../../superpowers/acceptance/2026-08-30-qiongli-pilot903-real-project-receipt.json) | 产品源码 `d0b4113364452d6ff8ff7cb2a3735e7c8d40d3f8`；Codex CLI `0.147.0`；已认证 Skill + Full MCP 项目/Graph 旅程、结构化输出、隐私与回滚 |

两个 receipt 都没有记录精确模型标识，因此模型身份是**未记录**。Claude
兼容性 receipt 不证明已认证 Claude 模型旅程；Codex CLI 证据不能用于认定
Codex Desktop，Claude Code 证据也不能用于认定 Claude Desktop。历史 receipt
只对其命名的源码与范围有效，不能认定发生变更后的 release candidate。

规范的机器可读投影是
[PILOT-905 矩阵 receipt](../../superpowers/acceptance/2026-08-30-qiongli-pilot905-host-capability-matrix.json)。
`publicationAllowed` 仍为 `false`。
