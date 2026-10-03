# 使用 Skills

先[安装 Plugin](install.md)，新开 Host 会话并检查穷理工具。

## 直接描述任务

说明你想得到什么、已有材料和必须遵守的要求。例如：

> 阅读这篇论文，说明主要发现、证据位置和局限。

> 根据这份分析写结果部分，保留估计值、引用和不确定性。

窄任务只加载相关指导。设计遵循所用方法与已批准协议；润色保留数字、术语、引用和因果限制。正式交付仍需相应证据与检查。

## 选择入口

Codex 和 DeepSeek 提供 `qiongli`、`no-qiongli` 与 20 个工作流入口，例如 `qiongli-paper-read`、`qiongli-lit-review`、`qiongli-stage-close`。Codex 用 `$` 调用；Claude 使用研究主 Skill，也接受自然语言请求。

想只讨论、不处理文件时，用 `$no-qiongli` 或说“仅回复”，详见[仅回复](../advanced/agent-skill-collaboration.md#reply-only)。

| 安装方式 | 可用内容 |
|---|---|
| 独立 Skills | 指导文件，使用 Host 现有工具 |
| Marketplace Plugin | Skills 与 Lite MCP，15 个工具 |
| CLI 安装的 Plugin | Skills 与 Full MCP，33 个工具 |

工具缺失时检查连接，不伪造结果或绕过审批。保存前预览、授权并核对当前版本；阶段总结保留原文件，删除由你亲自执行。

## 选刊与投稿前审阅 {#journal-fit-and-pre-submission-review}

- 已选期刊：“核查这本期刊的初投稿要求，并对稿件提出建议。”
- 寻找期刊：“先阅读稿件，再比较适合的期刊、读者与费用。”
- 核查研究：“审查方法和结论，给出有来源的意见。”

期刊要求须核对当前来源、文章类型和投稿阶段。只有摘要或材料不全时，建议保持暂定。表达调整与补充分析分别说明；同一模型的多个视角属于自审。保密审稿须遵守期刊的 AI 使用与保密规定。

[研究任务](task-recipes.md) · [Skills 完整参考](../reference/skills.md) · [Graph 示例](../examples/research-graph.md)
