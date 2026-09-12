# 使用 Agent Skills

本页当前入口面向 2.x。先按[安装指南](cli-2x.md#install-and-upgrade-bundled-content)
更新并注册 Plugin，再新开 Host 会话检查实际工具。

## 直接提出研究请求

你可以说“阅读这篇论文，列出主要发现和证据局限”“把这些来源整理成文献综述”，
或“总结已经完成的阶段，保留来源和变化”。窄任务只读取相关指导，不会因为项目存在
就启动全部研究阶段。稿件润色沿用 J2，保留数字、引用、术语和因果限制。

当前开发源码也让研究设计指导随方法和已批准协议而定。角色不再统一规定文献、
竞争解释或稳健性检验的数量。提出请求时，说明需要作出的决定、现有材料，以及
必须保留的要求即可。正式交付仍需相应证据和检查；预注册草稿也不等于已经注册。
这些调整位于 beta.5 之后，需要使用包含新内容的构建。

新构建的 Codex Plugin 提供 `$qiongli` 和 20 个 workflow 快捷入口，例如
`$qiongli-paper-read`、`$qiongli-lit-review`、`$qiongli-stage-close`。
这些入口先读取共享 Skill，再读取对应工作流；82 张内部技能卡不单独包装。
Claude 保留一个主 Skill，也可以直接用自然语言调用。旧缓存需要更新才会出现新入口。

## 工具与保存

| 使用方式 | 边界 |
|---|---|
| 独立 Skills | 使用 Host 现有工具和授权材料；不假定已经连接 MCP |
| 原生 Marketplace Plugin | 启动随包 Lite MCP，提供 14 个工具 |
| CLI 导出的本地 Plugin | 启动随包 Full MCP，提供 32 个工具；Host 和模型由用户选择 |
| CLI | `qiongli doctor`、`qiongli project` 和 `qiongli help`；不需要 Python 运行时 |

注册和缓存校验通过不等于会话工具已加载。需要的 MCP 工具缺失时，先检查连接；
可继续独立完成有材料支持的工作，但不能伪造工具调用或直接修改已登记项目来绕过审批。
项目写入沿用预览、明确授权和当前 revision 校验。阶段总结保留原文件，清理由用户亲自操作。

[Graph 与研究连续性](cli-2x.md#research-graph) · [任务场景](task-recipes.md)

<details>
<summary>1.x 的旧客户端与运行时说明</summary>

以下说明仅适用于旧版本，保留用于迁移对照；其中的 Python、bootstrap 和旧 CLI
命令不适用于原生 2.x。


Qiongli 安装的是一套 agent-facing skill 系统，但不同客户端暴露入口的方式不一样。安装之后，如果你不知道在 Codex、Claude Code、Antigravity、Hermes 或 shell 里该输入什么，先看这一页。

## 名称模型

| 名称 | 含义 | 出现位置 |
|---|---|---|
| `qiongli` | 公开 plugin、CLI 和用户可见 skill 名称 | Skillsplace、npm、PyPI、Codex `/skills`、shell 命令 |
| `qiongli-workflow` | 便携 skill package 目录名 | `~/.codex/skills/`、`~/.claude/skills/`、`~/.gemini/antigravity/skills/`、`~/.hermes/skills/`、plugin payload |
| `skills/*/*.md` | 内部学术能力卡片 | 仓库源码和 orchestrator 自动注入 |

大多数使用者应该找 `qiongli`，不要再找 `research-paper-workflow`。目录名 `qiongli-workflow` 仍然保留，是为了兼容已有 installer 和 release artifacts。

## 客户端入口

| 客户端 | 发现方式 | 调用方式 | 说明 |
|---|---|---|---|
| Codex | `/skills` | `$qiongli`、`$qiongli-lit-review`、`$qiongli-academic-write`，或其他生成的 Qiongli workflow wrapper | Codex 不会把 Qiongli workflows 暴露成自定义 slash command。Plugin 安装会生成很薄的 wrapper skills，路由到和 Claude slash commands 相同的 canonical workflows。安装或升级后需要重启 Codex。 |
| Claude Code | Plugin UI 或 `/plugin` 命令 | `/paper`、`/lit-review`、`/paper-write`、`/code-build`，或自然语言要求使用 Qiongli | Plugin 会安装 command wrappers 和便携 skill package。 |
| Shell | `qiongli check` | npm：`qiongli install`、`qiongli update`、`qiongli project ...`；完整运行时：`qiongli doctor`、`qiongli task-run`、`python3 -m bridges.orchestrator ...` | npm/npx 是免 Python 资产管理器。完整运行时命令需要先 `pipx install qiongli`，并使用 Python 3.12+。 |

## 运行架构流程

这张图展示从用户请求到运行时选择、preview、执行和持久产物的完整路径。

```mermaid
flowchart TB
    subgraph Entrypoints["入口面"]
        Request["学术请求<br/>topic, paper type, constraints"]
        Client["客户端 skill/plugin<br/>Codex, Claude Code,<br/>Claude Desktop/Web"]
        Npm["npm/npx 资产管理器<br/>install, setup, update,<br/>refresh, upgrade, check"]
        RuntimeCli["完整运行时 CLI/MCP<br/>pipx, pip, bootstrap full"]
        Request --> Client
        Request --> Npm
        Request --> RuntimeCli
    end

    subgraph ProjectState["项目使用状态"]
        Manifest{"项目 manifest 存在?<br/>.qiongli/guidance_manifest.yaml"}
        Auto["隐式项目状态<br/>active_subject: auto"]
        Configured["已配置项目状态<br/>subject, venue profiles,<br/>method lenses, strictness"]
        LocalGuidance["人工 guidance<br/>.qiongli/local_guidance.md<br/>.qiongli/guidance.d/*.md"]
        Npm --> Manifest
        RuntimeCli --> Manifest
        Manifest -->|no| Auto
        Manifest -->|yes| Configured
        Auto --> LocalGuidance
        Configured --> LocalGuidance
    end

    subgraph Routing["任务路由"]
        Contract["任务合同<br/>Task ID, stage, outputs,<br/>evidence rules, quality gates"]
        Subject["解析 subject context<br/>explicit domain > project state<br/>> temporary inference > core"]
        Runtime{"选择能完成任务的<br/>最小运行时"}
        Client --> Contract
        RuntimeCli --> Contract
        LocalGuidance --> Subject
        Contract --> Subject
        Subject --> Runtime
    end

    subgraph RuntimeChoice["运行时选择"]
        SkillOnly["Skill/plugin only<br/>读取 guidance,<br/>draft 或 review artifacts"]
        Provider["Literature provider<br/>MCPB 或内置 Node MCP<br/>status, search, evidence export"]
        Preview["完整运行时 preview<br/>doctor, task-plan,<br/>不启动 agents 的 task-run"]
        Execute{"run_agents == true<br/>且 doctor 通过?"}
        Agents["受控 agent run<br/>controller, primary,<br/>reviewer, verifier"]
        Runtime --> SkillOnly
        Runtime --> Provider
        Runtime --> Preview
        Preview --> Execute
        Execute -->|no| PreviewResult["Preview result<br/>safe plan 和 runtime notes"]
        Execute -->|yes| Agents
    end

    subgraph Outputs["产物与学习回路"]
        Formal["正式产物<br/>RESEARCH/[topic]/..."]
        Trace["Trace bundle<br/>.qiongli/trace/runs/&lt;run_id&gt;/"]
        Proposal["Guidance proposal<br/>manifest patch 和 local notes"]
        Apply{"guidance_mode"}
        SkillOnly --> Formal
        Provider --> Formal
        PreviewResult --> Trace
        Agents --> Formal
        Agents --> Trace
        Trace --> Proposal
        Proposal --> Apply
        Apply -->|propose| LocalGuidance
        Apply -->|apply| Manifest
    end
```

关键边界是：安装不等于执行。npm/npx 只安装和刷新客户端资产；只有显式安装并调用完整运行时后，`doctor`、MCP 编排、provider setup 或 agent execution 才会运行。

## Codex 用法

通过 Skillsplace、npm、PyPI 或 `qiongli upgrade --target codex` 安装后，重启 Codex，然后检查：

```text
/skills
```

你应该能看到 `qiongli`。plugin-first 安装还会生成很薄的 workflow wrapper skills，例如 `qiongli-lit-review`、`qiongli-academic-write`、`qiongli-paper-read` 和 `qiongli-proofread`。调用主 skill 或 wrapper 时，都带上具体研究任务：

```text
$qiongli plan a systematic review on retrieval augmented generation in education
$qiongli-lit-review retrieval augmented generation in education
$qiongli-academic-write related work for my CHI paper
$qiongli design an empirical study about ai writing support in universities
$qiongli prepare a submission checklist for my CHI paper
```

不要期待 `/qiongli` 或 `/lit-review` 在 Codex 里可用。Codex 的 slash commands 是客户端内建或客户端自己暴露的入口；Qiongli 在 Codex 中的入口是 skill invocation。生成的 wrappers 是刻意保持很薄的适配层：`$qiongli-lit-review` 会路由到 Claude Code `/lit-review` 使用的同一个 `workflows/lit-review.md` source。

如果 `/skills` 只看到 `research-paper-workflow`，说明当前机器上还有旧的全局安装。先运行当前升级路径，重启 Codex，再检查：

```bash
qiongli upgrade --target codex --overwrite
```

当前 qiongli 安装器会在升级时删除确认过的 `research-paper-workflow` 旧全局 skill 目录。如果你想单独预览全局清理，先运行 `qiongli clean --globals --dry-run`。

## Claude Desktop / Claude.ai 用法

Claude Desktop 应该把 Qiongli 暴露为已安装的 `qiongli` skill 或 direct plugin entry。可以直接用自然语言开始：

```text
Use Qiongli to plan a literature review on retrieval augmented generation in education.
Use Qiongli to read this DOI and extract the claim, method, evidence, and limits.
Use Qiongli to prepare a rebuttal matrix for these reviewer comments.
```

当 direct plugin 的 workflow command wrappers 可见时，`/qiongli` 是统一入口路由器。它会委派到与更窄阶段命令相同的 workflow 文件。Qiongli Literature Provider MCPB 或 bundled literature MCP 会提供 provider search tools；只有 skill 指令本身时，不应声称 `provider_connected` literature search。

## Claude Code 用法

Claude Code 可以通过 workflow entry markdown 暴露 Qiongli。常用入口是：

| 命令 | 适合场景 |
|---|---|
| `/qiongli` | 需要统一 Qiongli 入口自动选择正确 workflow。 |
| `/paper` | 需要 guided paper workflow 和 paper-type routing。 |
| `/lit-review` | 需要文献检索、筛选、提取或综合。 |
| `/paper-read` | 需要深度分析单篇论文。 |
| `/find-gap` | 需要识别和排序 research gaps。 |
| `/study-design` | 需要实证、质性或混合方法研究设计。 |
| `/paper-write` | 需要基于已有研究工作区写 manuscript。 |
| `/code-build` | 需要严格的学术代码 specification、planning、execution、review 和 reproducibility checks。 |
| `/submission-prep` | 需要期刊或会议投稿包。 |

这些 slash workflows 是便捷入口。它们最终都会路由到同一套 Qiongli task contract 和 skill package。

## 选刊、按期刊调整稿件与投稿前审稿

直接说清你现在要做的判断即可：

- “我已经选定这本期刊。请核查研究论文初投稿的要求，并对这份稿件提出修改建议。”
  A5 会把适用要求与稿件位置对应起来，说明需要改什么、依据是什么。
- “请阅读这份稿件，推荐合适的期刊。”H5 先看论文的贡献、方法和证据，再比较
  接收范围、读者、费用，以及你关心的其他限制。
- “请审查这篇论文的方法和结论。”H3 提供审稿意见，H4 着重检查有依据的投稿障碍；
  两者都不会自动转入修稿或投稿。

影响判断的期刊规定会附上来源、核查日期和适用的文章类型、投稿阶段。本地期刊档案
用于寻找线索，不能替代现行规定。只有摘要或无法核查来源时，会给出暂定建议并说明
缺少什么，不会凑够固定数量、承诺录用，也不会把期刊不匹配直接说成研究有问题。

修改建议会区分表达与报告补充，以及需要重新分析、补充数据或由作者确认的事项。
简单咨询可以只在聊天里回答；正式任务继续沿用现有文档和写入确认。同一模型的多个
审稿视角仍然属于自审，不能算独立复核。处理期刊委托的保密审稿，还要遵守该刊对
AI 使用和保密的规定。

## Shell 与 Orchestrator 用法

当你需要检查、升级、验证或运行显式 Task ID 时，使用 shell CLI：

```bash
qiongli check
qiongli upgrade --target all
qiongli doctor --project-dir .
```

当你需要明确的 task planning 或多 agent 执行时，使用 orchestrator：

```bash
python3 -m bridges.orchestrator task-plan \
  --task-id F3 \
  --paper-type empirical \
  --topic ai-in-education \
  --cwd .

python3 -m bridges.orchestrator task-run \
  --task-id F3 \
  --paper-type empirical \
  --topic ai-in-education \
  --cwd . \
  --triad
```

## 推荐使用流程

1. 只在单个客户端里用时，通过 Skillsplace 安装；需要跨客户端全局使用时，运行 `qiongli upgrade --target all`。
2. 重启目标客户端，让 skill registry 和 workflow discovery 刷新。
3. 在 Codex 中，用 `/skills` 确认出现 `qiongli`，然后用 `$qiongli` 调用。
4. 在 Claude Code 中，用 `/paper`、`/lit-review`、`/paper-write` 或 `/code-build`。
5. 需要可重复 task execution 时，用 `qiongli doctor` 和 `python3 -m bridges.orchestrator task-plan|task-run`。

当 workflow 或 orchestrator task 产生持久产物时，Qiongli 会把研究产物写到 `RESEARCH/[topic]/` 下。只有在你明确运行 `qiongli init` 或选择 project install parts 时，才会写入项目本地集成文件。
</details>
