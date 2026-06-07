---
layout: home

hero:
  name: Qiongli
  text: 面向 AI 编程代理的契约化学术工作流。
  tagline: 一次安装后，在 Codex、Claude Code 或 Gemini 中运行带 Task ID、质量门、文献诊断、角色交接和可审计产物的研究流程。
  actions:
    - theme: brand
      text: 快速开始
      link: /zh/quickstart
    - theme: alt
      text: 选择工作流
      link: /zh/guide/task-recipes
    - theme: alt
      text: 安装
      link: /zh/guide/install

features:
  - title: 安装路径清楚
    details: 根据实际需要选择原生 plugin、bootstrap partial/full、npm 或 pipx，而不是先装一堆不需要的运行时。
  - title: 研究路线明确
    details: 从 systematic review、empirical、qualitative、RCT、theory、code-first methods 等路线开始。
  - title: 证据合同可审
    details: 把 claim、citation、search log、diagnostics、method、code 和 review status 固定到标准产物路径里。
  - title: 多 Agent 可控
    details: 支持 solo、duo、triad 模式，并记录 handoff、disagreement 与 verification outcome。
---

## 先选入口

| 你想做什么 | 从这里开始 | 原因 |
|---|---|---|
| 只在一个客户端试用 Qiongli | [安装](/zh/guide/install) | 原生 plugin / extension 路径最轻。 |
| 安装后不知道输入什么 | [使用 Agent Skills](/zh/guide/using-agent-skills) | Codex、Claude Code、Gemini 和 shell 暴露 Qiongli 的方式不同。 |
| 给多个客户端安装全局 workflow | [快速开始](/zh/quickstart) | Bootstrap `partial` 不要求 Python，只安装 workflow assets。 |
| 使用 validator、`doctor` 或 orchestrated task | [多 Agent 运行](/zh/guide/multi-agent) | `full` runtime 说明 Python、模型 CLI、认证、Gemini broker/direct 和验证边界。 |
| 选择论文工作流 | [任务场景](/zh/guide/task-recipes) | 将真实研究目标映射到 paper type、stage、Task ID 和产物。 |
| 自动化安装或升级 | [CLI 参考](/zh/reference/cli) | 覆盖 `qiongli`、`ql`、npm/npx、pipx、兼容别名和 JSON 检查。 |

## 当前系统覆盖什么

Qiongli 发布的是一个统一的便携 workflow 包：`qiongli-workflow`。文档按研究者和项目维护者真正要完成的动作组织：

- **定义研究问题：** refine question、找 gap、画 theory map、判断 venue、定义 contribution claim。
- **构建文献基础：** provider-aware search、search bundle、diagnostics、dedup、screening、extraction、citation snowballing。
- **设计并执行研究：** 变量、数据集、robustness、preregistration、ethics artifact、data management。
- **写作并审计论文：** 章节结构、claim-evidence integrity、图表、limitations、submission、rebuttal。
- **处理研究代码：** 用 Stage-I specification -> planning -> execution -> review 路线处理 code-first 或 methods-heavy 工作。
- **协调多模型：** 在 Codex、Claude Code、Gemini 之间分配 controller、primary、reviewer、verifier，并保存交接与验证状态。

## 文档地图

- [入门](/zh/guide/): 安装、使用、升级、故障排除和 runtime 选择。
- [使用 Agent Skills](/zh/guide/using-agent-skills): 按客户端说明调用方式，包括 Codex 的 `/skills` 和 `$qiongli`。
- [任务场景](/zh/guide/task-recipes): 按 paper type 和常见研究目标选择路线。
- [示例](/zh/examples/): systematic review、empirical、qualitative、methods、theory 的 playbook。
- [参考](/zh/reference/): CLI 行为、skills catalog 和使用者约定。
- [架构](/zh/architecture): contracts、skills、roles、pipelines、bridges 和 package surfaces 如何组合。
- [高级](/zh/advanced/): subject packaging、扩展、MCP providers、Zotero、严格 literature search 和 plugin-first distribution。
- [维护者](/zh/maintainer/): release policy、naming policy 和贡献者实现指南。

## 运行时边界

安装 assets 和使用 workflow discovery 比完整 orchestrator 轻得多。你可以在没有 Python 的情况下安装 `qiongli-workflow`；但 `doctor`、validator、tests 和模型协同 task execution 需要 Python 3.12+、相关模型 CLI 和对应认证。
