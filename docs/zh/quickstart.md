# 快速开始

这一页面向“先用起来”的用户，不面向维护者。最短路径是：

1. 选择安装入口。
2. 创建或打开研究工作区。
3. 运行 workflow，或先查看 task plan。
4. 用质量门和标准产物保证结果可审计。

## 1. 选择最小安装入口

| 场景 | 使用 | 安装前是否需要 Python |
|---|---|---|
| Claude Desktop/Web 且不想用 CLI | focused subject Desktop ZIP，例如 `qiongli-claude-desktop-skill-economics-<tag>.zip` | 否 |
| 只在一个客户端里用 | 原生 plugin / extension | 否 |
| 多个客户端需要全局 workflow assets | Bootstrap `partial` | 否 |
| 需要 `doctor`、validator 或 orchestrator task execution | Bootstrap `full` | 是，Python 3.12+ |
| 偏向 npm 自动化 | `npm install -g qiongli` 或 `npx qiongli@latest` | 只有高级 bridge 命令需要 |
| 只需要 Python updater CLI | `pipx install qiongli` | 是 |

完整细节看 [安装](/zh/guide/install)。

## 2. 安装 workflow assets

如果你使用 Claude Desktop 或 Claude.ai 网页版，并且不想处理 code / CLI 环境，从 GitHub Release assets 下载需要的 focused subject ZIP。本阶段公开 Desktop ZIP subjects 是 `core`、`economics`、`business`、`finance`、`political-economy`、`geoeconomics` 和 `economics-accounting`，还没有 standalone accounting Desktop ZIP。默认通用 workflow 用 `qiongli-claude-desktop-skill-core-<tag>.zip`；经济学专精 workflow 用 `qiongli-claude-desktop-skill-economics-<tag>.zip`；political economy 专精 workflow 用 `qiongli-claude-desktop-skill-political-economy-<tag>.zip`；geoeconomics 专精 workflow 用 `qiongli-claude-desktop-skill-geoeconomics-<tag>.zip`；business 专精 workflow 用 `qiongli-claude-desktop-skill-business-<tag>.zip`；finance 专精 workflow 用 `qiongli-claude-desktop-skill-finance-<tag>.zip`；官方 economics/accounting 交叉学科包用 `qiongli-claude-desktop-skill-economics-accounting-<tag>.zip`。在 Claude Desktop 中把 ZIP 拖拽到 Skills 上传/安装流程中，或使用 `Customize > Skills > + > Create skill > Upload a skill`。Claude.ai 网页版也使用同一个 ZIP 上传流程。

Desktop/Web ZIP 使用 `coverage=focused`，用于保持上传文件数预算。它是 subject 专精包，不是降质删减版：保留 workflows、templates、standards、所选 profiles、`skills-summary.md` 和 `skills-core.md`；专精 ZIP 还包含通过 layered overlays 生成的 selected effective skill markdown。

如果你选择 Codex 或 Claude Code 的原生 plugin 路径，从统一的 Skillsplace marketplace 安装 Qiongli：

```bash
codex plugin marketplace add jxpeng98/skillsplace --ref main
```

```bash
claude plugin marketplace add jxpeng98/skillsplace@main
claude plugin install qiongli@skillsplace
```

内置 literature MCP 不通过客户端 MCP 设置页填写 provider key。让 Codex、Claude Desktop、Claude Code 或其他本地 MCP client 先运行 `qiongli_config_status`，再用 `qiongli_configure_provider` 打开本地设置页配置 `openalex.api_key`、可选 `openalex.email` 或 `semantic_scholar.api_key`。这样密钥不会进入 plugin manifest、release artifact 或对话上下文。

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

npm 或 pipx 安装中，`--subject` 默认是 `core`，`--coverage` 默认是 `complete`：

```bash
qiongli install --subject economics --target all
qiongli install --subject accounting --target all
npx qiongli@latest install --subject economics --target all
qiongli install --subject political-economy --target all
qiongli install --subject geoeconomics --target all
qiongli install --subject economics-accounting --target all
qiongli install --subject economics --coverage focused --target all
qiongli upgrade --subject accounting --target all
qiongli remove --target all --dry-run
qiongli customize --subject economics --name my-econ-lab --out ./qiongli-custom/econ-lab
qiongli check --json
```

不确定怎么选时使用默认 complete：`qiongli install --target all` 表示 `core/complete`，`--subject economics`、`--subject business`、`--subject finance`、`--subject political-economy` 和 `--subject geoeconomics` 表示 complete 专精安装，`--subject accounting` 表示 `accounting/complete`，即全量框架加 accounting 专精。只有明确想要精简包或 Desktop/Web ZIP 形态时才使用 `--coverage focused`。`political-economy` 和 `geoeconomics` 是两个独立 subject，不是一个 composite。官方 composite subjects（例如 `economics-accounting`）是命名 subject，不是任意逗号分隔叠加。切换 subject 或 coverage 时，重新运行 `install` 或 `upgrade` 并指定新的参数。Custom overlays 只影响 generated output，不会改写 canonical source files；`qiongli customize` 加 `--custom-dir` materialization 面向 Python/source checkout 工作流，npm runtime installs 在这个阶段使用预生成 payloads。

需要只保留 marketplace plugin 时，用 `qiongli remove` 移除 CLI 安装产生的全局资产。

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

Claude Code：

```text
/paper
/lit-review
/paper-write
/code-build
```

Codex 用 `/skills` 发现 skill，用 `$qiongli` 执行；它不会注册自定义 `/qiongli` slash command。Claude Code 在安装 command/workflow discovery 后可以使用 slash workflow 入口。这些入口只是 UX wrapper；真正的任务定义、预期产物、质量门和角色边界都在 Qiongli contracts 里。

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
