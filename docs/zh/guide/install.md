# 多端客户端安装指南 (Codex / Claude Code / Gemini)

::: warning 完整功能依赖
`partial` 轻量模式不要求 Python。`full` 模式和完整运行时仍然依赖：

- `python3` 3.12+
- `codex`
- `claude`
- `gemini`
- `OPENAI_API_KEY`、`ANTHROPIC_API_KEY`、`GOOGLE_API_KEY`

如果缺少这些依赖，资产安装和 shell `rsk` 维护命令仍可使用，但 orchestrator 执行、`doctor`、validator 和完整多模型流程会受限。
:::

## 1. 原生插件与扩展安装

如果你只想在单个客户端里使用，推荐通过客户端自己的原生扩展入口安装 **Research Skills**。这条路径会直接把 `research-paper-workflow` skill 安装到对应客户端，不需要用户先安装 `pip`、`pipx` 或 `rsk` CLI。

如果你需要下面这些能力，再使用 bootstrap / CLI 路径：

- 跨客户端全局安装
- 由 `rsk` 管理全局 slash command 软链接
- 使用 `rsk upgrade`、`rsk check` 或 `doctor`
- 执行多模型 orchestrator

本地开发安装命令：

```text
# Codex
从官方 Codex plugin marketplace 安装 Research Skills。

# Claude Code
/plugin marketplace add ./path/to/research-skills
/plugin install research-skills@research-skills

# Gemini CLI
gemini extensions install ./path/to/research-skills/plugins/research-skills
```

这个仓库包含用于打包和本地校验的元数据：

- `.agents/plugins/marketplace.json`
- `.claude-plugin/marketplace.json`
- `plugins/research-skills/.codex-plugin/plugin.json`
- `plugins/research-skills/.claude-plugin/plugin.json`
- `plugins/research-skills/gemini-extension.json`
- `plugins/research-skills/commands/*.md`
- `plugins/research-skills/skills/research-paper-workflow`

在这套结构里，plugin 负责安装、发现和平台入口；`research-paper-workflow` 是 plugin 内真正被加载的 portable skill package。`commands/*.md` 只做薄转发，真实 workflow 逻辑仍在 `skills/research-paper-workflow/workflows/*.md`。

Codex 和 Claude Code 使用 marketplace catalog。Gemini CLI 使用官方 extension 系统（`gemini-extension.json`），不是 marketplace JSON。

## 2. 与旧版全局 Skills 的兼容

新的原生 plugin / extension 安装可以和旧版 `partial` / `full` 安装共存，但不会自动迁移或删除旧安装。

它们是不同的安装面：

| 安装面 | 典型路径 | 管理方 |
|---|---|---|
| 原生 plugin bundle | 客户端 plugin / extension store，内部包含 `plugins/research-skills/skills/research-paper-workflow` | Codex / Claude Code / Gemini 的 plugin 系统 |
| 旧版全局 skill 安装 | `~/.codex/skills/research-paper-workflow`、`~/.claude/skills/research-paper-workflow`、`~/.gemini/skills/research-paper-workflow` | `rsk`、bootstrap 或本地 installer |
| 旧版 slash command discovery | `~/.claude/commands/*.md`、`~/.gemini/workflows/*.md` | `rsk` 管理的软链接 |

建议按下面规则选择：

- 新用户如果只需要客户端原生 skills 和 `/paper` 这类 workflow，直接安装官方 plugin / extension。
- 已经装过 `partial` / `full` 的用户，可以在旧全局 skills 旁边再安装 plugin。
- 如果还需要 `research-skills`、`rsk`、`rsw`、`doctor`、validator 或 `bridges.orchestrator`，继续保留 `full` runtime，并执行 `rsk upgrade --target all --doctor`，让全局 skill package 与 plugin 版本保持一致。
- 如果准备完全切到官方 plugin，不再需要旧版全局 slash discovery，先用 `rsk clean --globals --dry-run` 查看会清理哪些旧入口。

如果旧版全局 skills 和新 plugin 同时存在，建议保持版本一致，避免不同客户端或不同命令路径加载到不同版本的 `research-paper-workflow`。

## 3. 先选 `partial` 还是 `full`

如果你不传 `--profile`，bootstrap 会先解释这两种模式，再提示你选择。建议选最小可用模式：只用全局 skills 就选 `partial`，需要本地 CLI 和 orchestrator 检查再选 `full`。

| Profile | 安装内容 | 安装前是否要求 Python | 安装后结果 |
|---|---|---|---|
| `partial` | Codex / Claude Code / Gemini 的全局 `research-paper-workflow` skill 资产和 workflow discovery 链接 | 否 | `/paper`、`/lit-review` 等 slash workflow 可直接使用 |
| `full` | `partial` 的全部内容，加 shell CLI：`research-skills` / `rsk` / `rsw`，以及可选 `doctor` 校验 | 是，需要 Python 3.12+ | 完整 orchestrator 运行时可用 |

适合选 `partial` 的情况：

- 你只需要 AI 客户端里的 skills 和 slash workflow
- 机器上还没有 Python
- 你希望在 Windows 或受限机器上先用最低摩擦方式安装

适合选 `full` 的情况：

- 你要使用 `rsk upgrade`、`rsk init`、`rsk doctor` 或 `research-skills`
- 你要运行 `python3 -m bridges.orchestrator task-plan|task-run|doctor`
- 你要跑本地 validator、unit tests 或多模型 orchestrator

`full` 的真实行为：

- 如果系统里已经有 `python3 >= 3.12`，bootstrap 会直接复用。
- 如果 Python 缺失或版本过低，bootstrap 会立即失败，并打印可选安装方式。安装器不会自动安装 Python 或 `mise`。
- Windows 上由 PowerShell 直接安装，只有在 shell CLI 包装器需要 Bash 时才会通过 `winget` 安装 Git for Windows。

### `full` 模式的 Python 前提

`full` 模式要求机器上已经有 Python 3.12+，并且能在 PATH 中找到。安装器不会再自动安装 Python 或 `mise`。你可以用任何方式安装 Python：

- macOS：python.org 安装包、`brew install python`、`pyenv` 或 `mise`
- Windows：python.org 安装包、`winget install -e --id Python.Python.3.12 --source winget`、Microsoft Store 或 pyenv-win
- Linux：系统包管理器、`pyenv` 或 `mise`

运行 `full` 前先确认：

```bash
python3 --version
```

## 4. 运行推荐的一键 bootstrap

### Linux / macOS

交互式选择 `partial` 或 `full`：

```bash
curl -fsSL https://raw.githubusercontent.com/jxpeng98/research-skills/main/scripts/bootstrap_research_skill.sh | bash -s -- --project-dir "$PWD" --target all
```

强制 `partial`：

```bash
curl -fsSL https://raw.githubusercontent.com/jxpeng98/research-skills/main/scripts/bootstrap_research_skill.sh | bash -s -- --profile partial --project-dir "$PWD" --target all
```

强制 `full`：

```bash
curl -fsSL https://raw.githubusercontent.com/jxpeng98/research-skills/main/scripts/bootstrap_research_skill.sh | bash -s -- --profile full --project-dir "$PWD" --target all
```

安装最新 beta / prerelease：

```bash
curl -fsSL https://raw.githubusercontent.com/jxpeng98/research-skills/main/scripts/bootstrap_research_skill.sh | bash -s -- --beta --profile full --project-dir "$PWD" --target all
```

### Windows PowerShell 7+

如果机器上还没有 `pwsh`，先安装：

```powershell
winget install --id Microsoft.PowerShell --source winget
```

下载后交互式选择 `partial` 或 `full`：

```powershell
Invoke-WebRequest https://raw.githubusercontent.com/jxpeng98/research-skills/main/scripts/bootstrap_research_skill.ps1 -OutFile .\bootstrap_research_skill.ps1
pwsh -ExecutionPolicy Bypass -File .\bootstrap_research_skill.ps1 -ProjectDir "$PWD" -Target all
```

强制 `partial`：

```powershell
pwsh -ExecutionPolicy Bypass -File .\bootstrap_research_skill.ps1 -Profile partial -ProjectDir "$PWD" -Target all
```

强制 `full`：

```powershell
pwsh -ExecutionPolicy Bypass -File .\bootstrap_research_skill.ps1 -Profile full -ProjectDir "$PWD" -Target all
```

安装最新 beta / prerelease：

```powershell
pwsh -ExecutionPolicy Bypass -File .\bootstrap_research_skill.ps1 -Beta -Profile full -ProjectDir "$PWD" -Target all
```

Bootstrap 会安装：

- Codex / Claude Code / Gemini 的 workflow 资产
- `.agent/workflows/`、`CLAUDE.md`、`.gemini/` 等项目集成文件，仅在执行 `rsk init` 或 `--parts project` 时写入
- `full` 模式下的 shell CLI：`research-skills`、`rsk`、`rsw`

## 5. 使用已安装的 skills

`partial` 和 `full` 都是 global-first。安装完成后，日常使用流程是：

1. 新建或打开一个研究目录：`mkdir my-paper && cd my-paper`
2. 启动 Claude Code 或 Gemini CLI 等支持的客户端
3. 直接运行 slash workflow，例如 `/paper`、`/lit-review`、`/paper-write` 或 `/code-build`

模型会读取全局安装的 `research-paper-workflow`。默认不会往你的项目目录写文件；只有明确初始化项目时才会写入 `.env` 或本地 workflow 资产：

```bash
rsk init --project-dir .
```

## 6. 可选：本地安装器

如果机器上已经有 Python，可以用跨平台本地安装器：

```bash
python3 scripts/bootstrap_research_skill.py --profile partial --project-dir .
python3 scripts/bootstrap_research_skill.py --profile full --project-dir .
```

如果你在 Linux 或 macOS 上已经有本地仓库副本，也可以直接用本地 shell 安装器：

```bash
./scripts/install_research_skill.sh --profile partial --target all --project-dir /path/to/project
./scripts/install_research_skill.sh --profile full --target all --project-dir /path/to/project
```

`pip` / `pipx` 路径仍然保留给升级器 CLI，但不再是推荐的首次安装入口：

```bash
pipx install research-skills-installer
rsk upgrade --target all --doctor
rsk init --project-dir /path/to/project
```

如果你先装了 `partial`，之后又安装了 Python 3.12+，可以重新运行 bootstrap 并指定 `--profile full`；如果 shell CLI 已经可用，也可以执行 `rsk upgrade --target all --doctor`。

## 7. 全局优先安装与修改产物

目前系统所有的安装与升级**默认全部是全局操作（Global-first）**，你的项目目录会被保持绝对干净。

安装器主要执行两步：
1. **安装核心技能包：** 把 `research-paper-workflow` 下载存进你本地 AI 客户端所在的专属配置目录（例如 `~/.claude/skills/` 和 `~/.gemini/skills/`）。
2. **注册快捷指令 (Slash Commands)：** 自动在客户端的发现路径里打下轻量级的软链接（例如 `~/.claude/commands/paper.md`）。

这意味着像 `/paper`、`/study-design` 这样的命令，**无论你当前在电脑的哪个文件夹下工作，AI 都能原生识别**。

_注：涉及你具体项目内文件的写入（比如需要注入 API Key 的 `.env`），只有在你显式运行 `rsk init --project-dir .` 时才会发生。_

## 8. 常用参数

- `--profile partial|full`：显式指定安装模式，跳过交互提示。
- `--target codex|claude|gemini|antigravity|all`：限制安装范围。
- `--beta`：在未传 `--ref` 时安装最新 beta / prerelease tag。
- `--install-cli`：即使不是 `full` 也安装 shell CLI。
- `--no-cli`：即使是 `full` 也跳过 shell CLI。
- `--cli-dir <path>`：指定 shell CLI 安装目录。默认：`${RESEARCH_SKILLS_BIN_DIR:-~/.local/bin}`。
- `--overwrite`：覆盖已有安装目标。
- `--dry-run`：只预览安装动作。
- `--doctor`：安装后运行 `python3 -m bridges.orchestrator doctor --cwd <project>`。

## 9. 极简使用指南（零配置）

有了全局化命令注册，现在开启一篇新论文的研究步骤非常简单：

1. 新建一个纯空目录：`mkdir my-new-paper && cd my-new-paper`
2. 唤出终端里的 AI：输入 `claude` 或 `gemini`
3. 直接调用命令：敲击 `/paper` 或 `/lit-review`

模型会自动调用全局安装的技能体系，不会默认往你的工作区写入模板文件。

## 10. 日常升级与故障排查

要将你电脑上所有 AI 客户端的学术引擎统一升级到远端最新版本（你不用再去分别挨个项目升级了）：

```bash
rsk upgrade --target all --doctor
```

如果已经采用官方 plugin，并想预览清理旧版全局 slash command 软链接：

```bash
rsk clean --globals --dry-run
```

_注：终端工具（如 `rsk check`, `rsk upgrade`, `rsk clean`）现在自身不再依赖 Python 即可运行。但底层的核心验证器（`--doctor`，单元测试 以及 `bridges.orchestrator`）仍需合法的 Python 3 解释器来工作。_
