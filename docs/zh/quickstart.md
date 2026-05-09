# 快速开始

这一页是 `docs/quickstart.md` 的中文整理版，面向“先跑起来，再决定是否看维护者文档”的使用者。

如果你确实想看清 `skills/` 每一部分都包含什么内容，请配合 [Skills 指南](/zh/reference/skills) 一起使用。
如果你更关心“系统综述怎么走、qualitative paper 怎么走、methods paper 怎么走、审稿回复怎么走”，请直接看 [任务场景](/zh/guide/task-recipes)。

::: warning 完整功能依赖
如果你要使用完整功能集，请确保已经安装并配置：

- `python3` 3.12+
- `codex`
- `claude`
- `gemini`
- `OPENAI_API_KEY`、`ANTHROPIC_API_KEY`、`GOOGLE_API_KEY`

如果缺少这些依赖，你仍然可以安装 workflow 资产并使用 shell `qiongli check|upgrade|align`，但 `doctor`、validator、tests 与完整 orchestrator 执行链会受限。
:::

## 1. 原生插件与扩展安装

如果你只想在单个客户端里使用，推荐通过客户端自己的原生扩展入口安装 **Qiongli**。这会直接安装 `qiongli-workflow` skill，不需要用户先安装 `pip`、`pipx` 或 `rsk` CLI。

```text
# Codex
从官方 Codex plugin marketplace 安装 Qiongli。

# Claude Code
/plugin marketplace add ./path/to/qiongli
/plugin install qiongli@qiongli

# Gemini CLI
gemini extensions install ./path/to/qiongli/plugins/qiongli
```

如果你还需要跨客户端全局安装、shell 维护命令或 orchestrator 运行时，再使用下面的 bootstrap 路径。

## 2. 全局一键安装

目前推荐的首装路径是一键 bootstrap。已明确知道自己需要什么时，可以直接指定安装模式：

- `partial`：安装全局 skill 资产和 slash workflow discovery，不要求 Python。
- `full`：包含 `partial` 的全部内容，并安装 shell CLI（`qiongli`、`ql`、`research-skills`、`rsk`、`rsw`）和可选 `doctor` 校验；要求机器上已经有 Python 3.12+。

安装器不会自动安装 Python 或 `mise`。如果要使用 `full`，先通过 python.org、Homebrew、winget、Microsoft Store、pyenv、mise 或系统包管理器安装 Python。

Linux / macOS：

```bash
# 交互式选择 partial 或 full
curl -fsSL https://raw.githubusercontent.com/jxpeng98/qiongli/main/scripts/bootstrap_qiongli.sh | bash -s -- --project-dir "$PWD" --target all

# 强制轻量模式
curl -fsSL https://raw.githubusercontent.com/jxpeng98/qiongli/main/scripts/bootstrap_qiongli.sh | bash -s -- --profile partial --project-dir "$PWD" --target all

# 强制完整运行时模式
curl -fsSL https://raw.githubusercontent.com/jxpeng98/qiongli/main/scripts/bootstrap_qiongli.sh | bash -s -- --profile full --project-dir "$PWD" --target all
```

Windows PowerShell 7+：

```powershell
winget install --id Microsoft.PowerShell --source winget
Invoke-WebRequest https://raw.githubusercontent.com/jxpeng98/qiongli/main/scripts/bootstrap_qiongli.ps1 -OutFile .\bootstrap_qiongli.ps1

# 交互式选择 partial 或 full
pwsh -ExecutionPolicy Bypass -File .\bootstrap_qiongli.ps1 -ProjectDir "$PWD" -Target all

# 强制轻量模式
pwsh -ExecutionPolicy Bypass -File .\bootstrap_qiongli.ps1 -Profile partial -ProjectDir "$PWD" -Target all

# 强制完整运行时模式
pwsh -ExecutionPolicy Bypass -File .\bootstrap_qiongli.ps1 -Profile full -ProjectDir "$PWD" -Target all
```

Bootstrap 会把 `qiongli-workflow` 安装到 Codex、Claude Code、Gemini、Antigravity 等客户端的全局配置目录，并自动创建 Slash Command 发现链接。项目内文件只有在你显式运行 `qiongli init --project-dir .` 时才会写入。

## 3. 极简开局（零配置）

有了全局化命令注册，现在的开启流程完全可以做到肌肉记忆：

1. **新建一个空白文件夹：** `mkdir my-new-paper && cd my-new-paper`
2. **唤出你惯用的 AI：** 敲击 `claude` 或 `gemini`
3. **直接下发指令：** `输入 /paper` 或 `/lit-review` 等命令

模型会自动寻址并调用全局后台存放的技能体系。

## 4. 进阶调用方式

| 入口 | 适用场景 | 说明 |
|---|---|---|
| 原生插件 / 扩展 | 你只想在单个客户端里最省事地安装 | Codex marketplace、Claude marketplace 或 Gemini extension |
| Slash 命令 | 你想直接用 `/paper`、`/lit-review` 等命令 | 基于全局软链接，开箱即可在任何目录触发 |
| Orchestrator CLI | 你想结合自己的自动化脚本，或执行环境预检 | `python3 -m bridges.orchestrator task-plan|task-run|doctor` |
| 安装 / 升级 CLI | 你想安装、刷新全局 skill 或卸载软链接 | `qiongli`、`ql`、`research-skills`、`rsk`、`rsw` |

## 5. 先确定 paper type

典型 paper type 与 pipeline 对应关系：

| paper type | pipeline | 场景 |
|---|---|---|
| `systematic-review` | `systematic-review-prisma` | PRISMA 风格系统综述 |
| `empirical` | `empirical-study` | 标准实证研究 |
| `qualitative` | `qualitative-study` | 访谈、案例、民族志或过程导向 qualitative paper |
| `empirical` | `rct-prereg` | 含预注册的 RCT |
| `theory` | `theory-paper` | 理论或概念型论文 |
| `methods` | `code-first-methods` | 代码与方法并重的 methods paper |

## 6. 先 plan 再 run

推荐先看任务的依赖和路由：

```bash
python3 -m bridges.orchestrator task-plan \
  --task-id F3 \
  --paper-type empirical \
  --topic ai-in-education \
  --cwd .
```

你会看到：

- contract 产物
- 前置任务
- functional owner
- handoff 轨迹
- runtime plan（draft / review / fallback）

## 7. 再执行 canonical task

```bash
python3 -m bridges.orchestrator task-run \
  --task-id F3 \
  --paper-type empirical \
  --topic ai-in-education \
  --cwd . \
  --triad
```

常用参数：

- `--mcp-strict`
- `--skills-strict`
- `--triad`
- `--profile`
- `--draft-profile`
- `--review-profile`
- `--triad-profile`
- `--focus-output` 与 `--output-budget`：把本次运行收敛到更小的 active outputs，减少辅助文件扩散
- `--research-depth deep` 配合 `--max-rounds`：强制更窄、更有对抗性的证据扩展与修订流程

## 8. 什么时候切到维护者文档

你只是“使用系统”时，看这一页和 [入门](/zh/guide/) 就够了。

只有在下面这些场景才需要切换：

- 想理解系统分层：看 [系统架构](/zh/architecture)
- 想判断某个改动该落哪层：看 [规范约定](/zh/conventions)
- 想改具体行为：看 [扩展 Qiongli](/zh/advanced/extend-qiongli)
