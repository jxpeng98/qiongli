# 使用 Agent Skills

Qiongli 安装的是一套 agent-facing skill 系统，但不同客户端暴露入口的方式不一样。安装之后，如果你不知道在 Codex、Claude Code、Gemini CLI 或 shell 里该输入什么，先看这一页。

## 名称模型

| 名称 | 含义 | 出现位置 |
|---|---|---|
| `qiongli` | 公开 plugin、CLI 和用户可见 skill 名称 | Skillsplace、npm、PyPI、Codex `/skills`、shell 命令 |
| `qiongli-workflow` | 便携 skill package 目录名 | `~/.codex/skills/`、`~/.claude/skills/`、`~/.gemini/skills/`、`~/.gemini/antigravity/skills/`、`~/.hermes/skills/`、plugin payload |
| `skills/*/*.md` | 内部学术能力卡片 | 仓库源码和 orchestrator 自动注入 |

大多数使用者应该找 `qiongli`，不要再找 `research-paper-workflow`。目录名 `qiongli-workflow` 仍然保留，是为了兼容已有 installer 和 release artifacts。

## 客户端入口

| 客户端 | 发现方式 | 调用方式 | 说明 |
|---|---|---|---|
| Codex | `/skills` | `$qiongli` | Codex 不暴露自定义 `/qiongli` slash command。安装或升级后需要重启 Codex。 |
| Claude Code | Plugin UI 或 `/plugin` 命令 | `/paper`、`/lit-review`、`/paper-write`、`/code-build`，或自然语言要求使用 Qiongli | Plugin 会安装 command wrappers 和便携 skill package。 |
| Gemini CLI | Extension 安装或全局 workflow discovery | `/paper`、`/lit-review`、`/paper-write`、`/code-build` | 全局安装会在 Gemini home 下创建 workflow discovery entries。 |
| Shell | `qiongli check` | `qiongli doctor`、`qiongli upgrade`、`qiongli task-run`、`python3 -m bridges.orchestrator ...` | 需要 npm 或 Python CLI 入口。高级命令需要 Python 3.12+。 |

## Codex 用法

通过 Skillsplace、npm、PyPI 或 `qiongli upgrade --target codex` 安装后，重启 Codex，然后检查：

```text
/skills
```

你应该能看到 `qiongli`。调用时使用 `$qiongli`，并带上具体研究任务：

```text
$qiongli plan a systematic review on retrieval augmented generation in education
$qiongli design an empirical study about ai writing support in universities
$qiongli prepare a submission checklist for my CHI paper
```

不要期待 `/qiongli` 在 Codex 里可用。Codex 的 slash commands 是客户端内建或客户端自己暴露的入口；Qiongli 在 Codex 中的入口是 skill invocation，也就是 `$qiongli`。

如果 `/skills` 只看到 `research-paper-workflow`，说明当前机器上还有旧的全局安装。先运行当前升级路径，重启 Codex，再检查：

```bash
qiongli upgrade --target codex --overwrite
```

当前 qiongli 安装器会在升级时删除确认过的 `research-paper-workflow` 旧全局 skill 目录。如果你想单独预览全局清理，先运行 `qiongli clean --globals --dry-run`。

## Claude Code 和 Gemini 用法

Claude Code 和 Gemini 可以通过 workflow entry markdown 暴露 Qiongli。常用入口是：

| 命令 | 适合场景 |
|---|---|
| `/paper` | 需要 guided paper workflow 和 paper-type routing。 |
| `/lit-review` | 需要文献检索、筛选、提取或综合。 |
| `/paper-read` | 需要深度分析单篇论文。 |
| `/find-gap` | 需要识别和排序 research gaps。 |
| `/study-design` | 需要实证、质性或混合方法研究设计。 |
| `/paper-write` | 需要基于已有研究工作区写 manuscript。 |
| `/code-build` | 需要严格的学术代码 specification、planning、execution、review 和 reproducibility checks。 |
| `/submission-prep` | 需要期刊或会议投稿包。 |

这些 slash workflows 是便捷入口。它们最终都会路由到同一套 Qiongli task contract 和 skill package。

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
4. 在 Claude Code 或 Gemini 中，用 `/paper`、`/lit-review`、`/paper-write` 或 `/code-build`。
5. 需要可重复 task execution 时，用 `qiongli doctor` 和 `python3 -m bridges.orchestrator task-plan|task-run`。

当 workflow 或 orchestrator task 产生持久产物时，Qiongli 会把研究产物写到 `RESEARCH/[topic]/` 下。只有在你明确运行 `qiongli init` 或选择 project install parts 时，才会写入项目本地集成文件。
