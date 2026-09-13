---
layout: home
hero:
  name: 穷理 Qiongli
  text: "做研究，也保留结论背后的证据。"
  tagline: "Rust 原生 CLI、Skills 与 MCP，共用研究记录和来源。继续使用你熟悉的 Host 和模型。"
  actions:
    - theme: brand
      text: 下载 2.x CLI
      link: /zh/guide/cli-2x#standalone-binary-download
    - theme: alt
      text: 快速开始
      link: /zh/quickstart
    - theme: alt
      text: 选择研究工作流
      link: /zh/guide/task-recipes
features:
  - title: 解压就能运行
    details: "独立二进制包含研究资源，无需另装 Python、Node.js 或 Rust 运行时。"
  - title: 按当前任务开始
    details: "阅读、综述、设计、写作和润色共用 Skills，按需读取指导，不强制启动整个研究流程。"
  - title: 找得到来源
    details: "Graph 连接规范记录中的论点、来源和位置；阶段总结保留主要内容与变化。"
  - title: 修改前可审阅
    details: "项目写入需要预览、明确授权和版本校验。总结不会自动删除源文件。"
---

## 从这里开始

本站面向已合入 `main` 的原生 2.x。下载示例固定为 **2.0.0-beta.6**，
后续合入的安装修复在指南中另行说明，尚不代表正式版发布。
可以先试用离线 Research Graph、仅回复 Skill、可选 Hook 和交互安装。

| 你想做什么 | 入口 |
|---|---|
| 下载、解压后直接使用 | [独立二进制下载](guide/cli-2x.md#standalone-binary-download) |
| 通过 npm、PyPI 或 Cargo 安装 | [安装和命令指南](guide/cli-2x.md) |
| 安装、更新随包 Plugin 与 Skills | [Plugin 安装整合](guide/cli-2x.md#install-and-upgrade-bundled-content) |
| 查看 CLI 副本和集成状态 | `qiongli setup`、`qiongli install`、`qiongli doctor` |
| 理解 Graph 如何使用证据 | [Graph 使用与检查](guide/cli-2x.md#research-graph) |
| 查看旧 Python/npm 命令 | [1.x 历史参考](reference/cli.md) |

## 运行边界

2.x 的 CLI、Lite/Full MCP 和导出的原生 Plugin 都不需要额外的语言运行时。
npm 安装入口需要 Node，PyPI 入口需要 Python，Cargo 安装需要 Rust 构建工具；
想省去这些安装前提，可以直接下载二进制包。Host 应用和在线服务的账号按需配置。

包升级由原渠道完成；`upgrade plugin` 刷新当前 CLI 随附的内容，`upgrade cli`
显示升级方法。Plugin 注册成功后，还需新开 Host 会话检查实际工具。
原生 Marketplace 平台包使用 Lite MCP，本地 CLI 导出的 Plugin 使用 Full MCP。

[研究工作流](guide/task-recipes.md) · [架构](architecture.md) · [已观察的 Host 能力](guide/agent-host-capability-matrix.md)
