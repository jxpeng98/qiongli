# 快速开始

这一页面向“先用起来”的用户，不面向维护者。最短路径是：

1. 选择安装入口。
2. 创建或打开研究工作区。
3. 运行 workflow，或先查看 task plan。
4. 用质量门和标准产物保证结果可审计。

## 1. 选择最小安装入口

| 场景 | 使用 | 安装前是否需要 Python |
|---|---|---|
| 只在一个客户端里用 | 原生 plugin / extension | 否 |
| 多个客户端需要全局 workflow assets | Bootstrap `partial` | 否 |
| 需要 `doctor`、validator 或 orchestrator task execution | Bootstrap `full` | 是，Python 3.12+ |
| 偏向 npm 自动化 | `npm install -g qiongli` 或 `npx qiongli@latest` | 只有高级 bridge 命令需要 |
| 只需要 Python updater CLI | `pipx install qiongli` | 是 |

完整细节看 [安装](/zh/guide/install)。

## 2. 安装 workflow assets

如果你使用 Claude Desktop 或 Claude.ai 网页版，并且不想处理 code / CLI 环境，从 GitHub Release assets 下载 `qiongli-claude-desktop-skill-<tag>.zip`。在 Claude Desktop 中把 ZIP 拖拽到 Skills 上传/安装流程中，或使用 `Customize > Skills > + > Create skill > Upload a skill`。Claude.ai 网页版也使用同一个 ZIP 上传流程。

Desktop/Web ZIP 是为了满足 Claude 上传文件数限制而生成的 slim 包。它保留 workflows、templates、standards、venue profiles、`skills-summary.md` 和 `skills-core.md`，但省略细分 per-skill markdown specs。需要完整细分 skill 语料时，使用 plugin/source 发行。

如果你选择 Codex 或 Claude Code 的原生 plugin 路径，从统一的 Skillsplace marketplace 安装 Qiongli：

```bash
codex plugin marketplace add jxpeng98/skillsplace --ref main
```

```bash
claude plugin marketplace add jxpeng98/skillsplace@main
claude plugin install qiongli@skillsplace
```

如果你需要跨客户端全局 workflow assets，使用 bootstrap 安装器。

Linux / macOS：

```bash
curl -fsSL https://raw.githubusercontent.com/jxpeng98/qiongli/main/scripts/bootstrap_qiongli.sh | bash -s -- --profile partial --project-dir "$PWD" --target all
```

Windows PowerShell 7+：

```powershell
winget install --id Microsoft.PowerShell --source winget
Invoke-WebRequest https://raw.githubusercontent.com/jxpeng98/qiongli/main/scripts/bootstrap_qiongli.ps1 -OutFile .\bootstrap_qiongli.ps1
pwsh -ExecutionPolicy Bypass -File .\bootstrap_qiongli.ps1 -Profile partial -ProjectDir "$PWD" -Target all
```

如果机器已经有 Python 3.12+，并且你需要 runtime check、validator 或 orchestrated task，把 `--profile partial` 改成 `--profile full`。

## 3. 创建研究工作区

```bash
mkdir my-paper
cd my-paper
```

然后打开你使用的客户端，并使用该客户端支持的入口。

Codex：

```text
/skills
$qiongli plan an empirical paper on ai-in-education
```

Claude Code 或 Gemini CLI：

```text
/paper
/lit-review
/paper-write
/code-build
```

Codex 用 `/skills` 发现 skill，用 `$qiongli` 执行；它不会注册自定义 `/qiongli` slash command。Claude Code 和 Gemini 在安装 command/workflow discovery 后可以使用 slash workflow 入口。这些入口只是 UX wrapper；真正的任务定义、预期产物、质量门和角色边界都在 Qiongli contracts 里。

完整客户端用法看 [使用 Agent Skills](/zh/guide/using-agent-skills)。

## 4. 选择研究路线

| paper type | pipeline | 适合场景 |
|---|---|---|
| `systematic-review` | `systematic-review-prisma` | 需要 PRISMA 风格 search、screening、extraction、synthesis。 |
| `empirical` | `empirical-study` | 标准实证研究论文。 |
| `qualitative` | `qualitative-study` | 访谈、案例、民族志或 process-oriented 输出。 |
| `empirical` | `rct-prereg` | 需要 RCT preregistration 和 reporting checks。 |
| `theory` | `theory-paper` | 需要概念发展、机制和 contribution framing。 |
| `methods` | `code-first-methods` | 研究代码和方法 artifact 是一等产物。 |

按真实场景选路，看 [任务场景](/zh/guide/task-recipes)。

## 5. 先 inspect 再 run

如果你安装了 `full` runtime，建议先看任务路线：

```bash
python3 -m bridges.orchestrator task-plan \
  --task-id F3 \
  --paper-type empirical \
  --topic ai-in-education \
  --cwd .
```

`task-plan` 会显示：

- 必需和可选产物
- 前置任务
- functional owner 与 handoff trace
- draft、review、fallback、verification 的 runtime plan

## 6. 执行 canonical task

```bash
python3 -m bridges.orchestrator task-run \
  --task-id F3 \
  --paper-type empirical \
  --topic ai-in-education \
  --cwd . \
  --triad
```

常用参数：

- `--mcp-strict`：外部 evidence provider 不可用时阻断。
- `--skills-strict`：内部 skill spec 缺失时阻断。
- `--triad`：请求第三方独立 audit。
- `--focus-output` 和 `--output-budget`：减少单次运行的辅助产物扩散。
- `--research-depth deep` 配合 `--max-rounds`：执行更深的证据扩展与修订。

## 7. 理解质量门

Qiongli 的价值在于留下可审计痕迹：

- literature search diagnostics 与 materialized search bundle
- claim-evidence map 与 citation risk artifact
- method diagnostics 与 reporting checks
- code spec、plan、execution、review、reproducibility artifact
- 多 agent 工作中的 handoff、disagreement record 与 verification status

需要理解这些 contract 怎么组合时，看 [系统架构](/zh/architecture)。
