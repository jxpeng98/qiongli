# 安装 Qiongli

Qiongli 有多个安装入口，是因为不同用户需要的运行时能力不同。先选能满足目标的最小入口。

## 安装入口

| 入口 | 适合场景 | 安装内容 | 是否要求 Python |
|---|---|---|---|
| 原生 plugin / extension | 单个客户端，最少配置 | 客户端 plugin 和 `qiongli-workflow` | 否 |
| Bootstrap `partial` | 多客户端全局 workflow assets | skills 和 slash workflow discovery | 否 |
| Bootstrap `full` | runtime check 和 orchestrator | `partial` 加 shell CLI 与 `doctor` 支持 | 是，Python 3.12+ |
| npm / npx | Node 自动化安装 | npm CLI 和内置 workflow payload | 只有高级 bridge 命令需要 |
| pipx / pip | Python updater CLI | Python CLI 分发 | 是 |

## 原生 Plugin 和 Extension

只需要在一个客户端里使用 Qiongli 时，优先选这个入口。

Codex 通过统一的 [Skillsplace](https://github.com/jxpeng98/skillsplace) marketplace 安装：

```bash
codex plugin marketplace add jxpeng98/skillsplace --ref main
codex plugin marketplace list
```

然后在 Codex plugin UI 中安装或启用 `qiongli`。

Claude Code 使用同一个 Skillsplace catalog：

```bash
claude plugin marketplace add jxpeng98/skillsplace@main
claude plugin install qiongli@skillsplace
```

在 Claude Code 交互会话中，也可以使用 slash commands：

```text
/plugin marketplace add jxpeng98/skillsplace@main
/plugin install qiongli@skillsplace
```

Gemini CLI 仍然直接安装本地 extension payload：

```bash
gemini extensions install ./path/to/qiongli/plugins/qiongli
```

这条路径不会安装 shell CLI、Python bridge 或全局 slash-command symlinks。需要这些能力时，用 bootstrap 或 npm。

## Bootstrap Partial

`partial` 用于安装跨客户端 workflow package，不要求 Python：

```bash
curl -fsSL https://raw.githubusercontent.com/jxpeng98/qiongli/main/scripts/bootstrap_qiongli.sh | bash -s -- --profile partial --project-dir "$PWD" --target all
```

Windows PowerShell 7+：

```powershell
winget install --id Microsoft.PowerShell --source winget
Invoke-WebRequest https://raw.githubusercontent.com/jxpeng98/qiongli/main/scripts/bootstrap_qiongli.ps1 -OutFile .\bootstrap_qiongli.ps1
pwsh -ExecutionPolicy Bypass -File .\bootstrap_qiongli.ps1 -Profile partial -ProjectDir "$PWD" -Target all
```

`partial` 会安装 workflow assets 和 discovery links，但不会运行完整 runtime validation。

## Bootstrap Full

需要本地验证或 orchestrated task execution 时，用 `full`：

```bash
curl -fsSL https://raw.githubusercontent.com/jxpeng98/qiongli/main/scripts/bootstrap_qiongli.sh | bash -s -- --profile full --project-dir "$PWD" --target all
```

Windows PowerShell 7+：

```powershell
pwsh -ExecutionPolicy Bypass -File .\bootstrap_qiongli.ps1 -Profile full -ProjectDir "$PWD" -Target all
```

`full` 要求 Python 3.12+ 已经在 `PATH` 上。安装器不会安装 Python 或 `mise`。

安装后检查工作区：

```bash
qiongli doctor --project-dir .
python3 -m bridges.orchestrator doctor --cwd .
```

## npm / npx

如果你想用 Node 分发的安装器，并让 workflow payload 跟随 npm 包：

```bash
npm install -g qiongli
qiongli install --target all --project-dir "$PWD"
```

一次性运行：

```bash
npx qiongli@latest install --target all --project-dir "$PWD"
npx qiongli@latest check --json
```

测试 prerelease 时仍可使用 `next` dist-tag：

```bash
npx qiongli@next install --target all --project-dir "$PWD"
```

## pipx / pip

如果你明确需要 Python 分发的 updater CLI，用 pipx：

```bash
pipx install qiongli
qiongli upgrade --target all
```

升级：

```bash
pipx upgrade qiongli
qiongli upgrade --target all --doctor --project-dir /path/to/project
```

## 实际会安装什么

根据入口不同，Qiongli 可能安装：

- 客户端 home 目录下的 `qiongli-workflow` skill assets
- `/paper`、`/lit-review`、`/paper-write`、`/code-build` 等 workflow command discovery links
- shell 命令 `qiongli`、`ql` 和兼容别名 `research-skills`、`rsk`、`rsw`
- 只有显式运行 `qiongli init --project-dir .` 时才写入的项目集成文件

默认不会写入项目本地文件。全局 workflow package 可在任意研究工作区使用。

## 保持版本一致

如果你同时使用多个安装面，保持 plugin、global skill assets、npm payload 和 Python CLI 一致：

```bash
qiongli check
qiongli upgrade --target all
```

如果你已经完全转向原生 plugin，不再需要旧的全局 slash commands，先 dry-run 清理：

```bash
qiongli clean --globals --dry-run
```
