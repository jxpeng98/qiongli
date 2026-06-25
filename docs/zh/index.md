---
layout: home

hero:
  name: Qiongli
  text: "让 AI 辅助研究留下可复查证据链。"
  tagline: "把宽泛的学术请求拆成 Task ID、质量门、文献诊断、角色交接和后续能检查的标准产物。"
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
  - title: "先选刚好够用的安装"
    details: "按任务选择原生 plugin、bootstrap partial/full、npm 或 pipx，不需要一开始就装完整运行时。"
  - title: "从主题走到论文路线"
    details: "支持 systematic review、empirical、qualitative、RCT、theory、code-first methods 等研究路线。"
  - title: "证据链可以追溯"
    details: "把 claim、citation、search log、diagnostics、method、code 和 review status 放到稳定的产物路径里。"
  - title: "多 Agent 审阅可控"
    details: "使用 solo、duo、triad 模式，并保留 handoff、disagreement record 与 verification outcome。"
---

## 先选入口

| 你想做什么 | 从这里开始 | 原因 |
|---|---|---|
| 只想先在一个客户端试用 | [安装](/zh/guide/install) | 原生 plugin / extension 路径最轻。 |
| 安装后不知道怎么调用 | [使用 Agent Skills](/zh/guide/using-agent-skills) | Codex、Claude Code、Antigravity、Hermes 和 shell 的入口不一样。 |
| 给多个客户端安装全局 workflow | [快速开始](/zh/quickstart) | Bootstrap `partial` 不要求 Python，只安装 workflow assets。 |
| 使用 validator、`doctor` 或 orchestrated task | [多 Agent 运行](/zh/guide/multi-agent) | `full` runtime 会说明 Python、模型 CLI、认证和验证边界。 |
| 选择论文工作流 | [任务场景](/zh/guide/task-recipes) | 将真实研究目标映射到 paper type、stage、Task ID 和产物。 |
| 自动化安装或升级 | [CLI 参考](/zh/reference/cli) | 覆盖 `qiongli`、`ql`、npm/npx、pipx、兼容别名和 JSON 检查。 |

## 最新稳定版下载

当前稳定版是 [v1.9.0](https://github.com/jxpeng98/qiongli/releases/tag/v1.9.0)。下面这些直达链接覆盖常见安装路径；需要 subject 专精 Desktop ZIP 或维护者 artifacts 时，再打开下载指南。

| 需求 | 链接或命令 |
|---|---|
| npm CLI | [`qiongli@1.9.0`](https://www.npmjs.com/package/qiongli/v/1.9.0)：`npm install -g qiongli@latest` |
| PyPI CLI | [`qiongli 1.9.0`](https://pypi.org/project/qiongli/1.9.0/)：`pipx install qiongli` |
| Claude Desktop/Web core skill | [`qiongli-claude-desktop-skill-core-v1.9.0.zip`](https://github.com/jxpeng98/qiongli/releases/download/v1.9.0/qiongli-claude-desktop-skill-core-v1.9.0.zip) |
| Claude Desktop literature MCPB | [`qiongli-literature-provider-0.1.5.mcpb`](https://github.com/jxpeng98/qiongli/releases/download/v1.9.0/qiongli-literature-provider-0.1.5.mcpb) |
| Zotero Desktop companion | [`qiongli-zotero-companion-0.2.2.xpi`](https://github.com/jxpeng98/qiongli/releases/download/v1.9.0/qiongli-zotero-companion-0.2.2.xpi) |
| 全部 release assets | [下载指南](https://github.com/jxpeng98/qiongli/releases/download/v1.9.0/qiongli-downloads-v1.9.0.md) 和 [GitHub Release](https://github.com/jxpeng98/qiongli/releases/tag/v1.9.0) |

## 当前系统覆盖什么

Qiongli 提供便携的 `qiongli-workflow` 包，也提供可选的本地 literature search 和 orchestration 运行时。文档按研究者和项目维护者真正要完成的动作来组织：

- **定义研究问题：** 精炼问题、找 gap、画 theory map、判断 venue，并明确 contribution claim。
- **构建文献基础：** 规划 provider-aware search，生成 search bundle，记录 diagnostics，处理 dedup、screening、extraction 和 citation snowballing。
- **设计并执行研究：** 明确变量、数据集、robustness、preregistration、ethics artifact 和 data management。
- **写作并审计论文：** 组织章节，把 claim 绑回 evidence，规划图表，审阅 limitations，并准备 submission 或 rebuttal。
- **处理研究代码：** 用 Stage-I specification -> planning -> execution -> review 路线处理 code-first 或 methods-heavy 工作。
- **协调多模型：** 在 Codex、Claude Code 和 Antigravity 之间分配 controller、primary、reviewer、verifier，并保存交接与验证状态。

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

安装 workflow assets 比运行完整 orchestrator 轻得多。你可以在没有 Python 的情况下使用 `qiongli-workflow`；只有在运行 `doctor`、validator、tests 或模型协同 task execution 时，才需要 Python 3.12+、相关模型 CLI 和对应认证。

## 社区致谢

感谢 [linux.do](https://linux.do/) 社区提供开放、务实的中文技术讨论场域。Qiongli 后续会在 linux.do 做项目曝光、收集反馈，也欢迎从 linux.do 看到这个项目的朋友提出使用建议和真实批评。
