<div align="center">
  <h1>穷理（Qiongli）</h1>
  <p><strong>面向 Codex、Claude Code 与 Gemini 的契约驱动学术工作流系统。</strong></p>
  <p>用同一套 Task ID、质量门、角色交接和标准产物路径，串联文献、写作、研究代码、审稿与可复查证据链。</p>
  <p>
    <a href="docs/zh/quickstart.md">快速开始</a> ·
    <a href="docs/zh/guide/install.md">安装</a> ·
    <a href="docs/zh/guide/task-recipes.md">任务场景</a> ·
    <a href="docs/zh/reference/cli.md">CLI</a> ·
    <a href="docs/zh/architecture.md">架构</a>
  </p>
</div>

## 穷理现在能做什么

穷理把学术工作拆成可执行、可审计的任务链。它不是让模型一次性即兴生成整篇论文，而是把每一步绑定到 Task ID、质量门、角色交接和 `RESEARCH/[topic]/` 下的标准产物。

适合用于：

- **研究工作流：** systematic review、empirical study、qualitative study、RCT preregistration、theory paper、code-first methods paper。
- **文献严谨性：** provider-aware search planning、search diagnostics、search bundle、dedup log、screening readiness、snowball readiness。
- **写作完整性：** claim-evidence map、citation risk、图表规划、limitations review、proofreading、rebuttal。
- **研究代码纪律：** 严格 Stage-I `I5 -> I6 -> I7 -> I8`，覆盖 specification、planning、execution、review。
- **多 agent 审阅：** Codex / Claude / Gemini 的 solo、duo、triad 模式，显式 handoff、disagreement record 和 verification status。

## 从哪里开始

按你的目标选择最小入口：

| 目标 | 推荐入口 | 说明 |
|---|---|---|
| 只在一个 AI 客户端里使用 | 原生 plugin / extension | [安装指南](docs/zh/guide/install.md) |
| 给多个客户端安装 workflow assets | Bootstrap `partial` profile | [快速开始](docs/zh/quickstart.md) |
| 使用 `qiongli doctor`、validator 或 orchestrator | Bootstrap `full` profile，要求 Python 3.12+ | [多 Agent 指南](docs/zh/guide/multi-agent.md) |
| 通过 npm 做脚本化安装 | `npm install -g qiongli` 或 `npx qiongli@latest` | [CLI 参考](docs/zh/reference/cli.md) |
| 更新 Python CLI 分发 | `pipx install qiongli` 或 `pipx upgrade qiongli` | [升级指南](docs/zh/guide/upgrade.md) |
| 选择论文路线 | 从任务场景开始 | [任务场景](docs/zh/guide/task-recipes.md) |

## 当前能力地图

| 领域 | 覆盖内容 |
|---|---|
| Framing | 问题精炼、贡献陈述、假设、理论图、gap 分析、venue fit |
| Literature | 学术搜索、概念扩展、筛选、抽取、citation snowballing、全文检索、reference bridge |
| Design | 研究设计、变量、robustness、dataset、preregistration、data management |
| Ethics and compliance | IRB、deidentification、ethics statement、PRISMA、reporting checks |
| Writing and synthesis | 证据综合、manuscript architecture、analysis interpretation、表格、图、discussion、limitations |
| Submission and rebuttal | peer-review simulation、fatal-flaw detection、cover materials、response matrix |
| Code and reproducibility | data cleaning、merge、statistics、code build/review、release packaging、reproducibility audit |
| Presentation | 报告规划、slide architecture、Slidev、Beamer、PPTX-oriented outputs |

## 运行时边界

> [!WARNING]
> 如果你要使用“完整功能集”，需要真实安装并配置：
> `python3`、`codex`、`claude`、`gemini` 四个运行时入口，以及对应的 `OPENAI_API_KEY`、`ANTHROPIC_API_KEY`、`GOOGLE_API_KEY`。
> 如果缺少这些依赖，你仍然可以完成 shell 安装和 `qiongli check|upgrade|align`，但 `doctor`、validator、tests 和完整 orchestrator 多模型执行链会受限或不可用。

## 为什么叫“穷理”

**穷理**是这个项目的对外主名。它取“追究其理”之意：面对一个研究问题，不止生成一段文本，而是继续向下追到概念、文献、方法、证据、代码与反驳边界。对学术工作流来说，这个名字强调的是把研究判断放回可检查的证据链里。

完整体系名是 **穷理证澈**。其中 **证澈** 是方法论与核心模块名：让证据、引用风险、假设、claim 边界和推理链条清澈可审。落实到仓库里，就是所有 workflow 都围绕 Task ID、质量门和 `RESEARCH/[topic]/` 下的标准产物运行，而不是依赖一次性的 prompt 即兴发挥。

技术命名统一跟随对外主名：plugin ID 是 `qiongli`，便携 skill 包是 `qiongli-workflow`，Python 升级器分发名是 `qiongli`。`research-skills`、`rsk`、`rsw` 等旧入口只作为迁移期兼容别名继续保留。

## 设计借鉴与相关项目

这个仓库不是凭空长出来的，两个外部项目对它的设计方向尤其重要：

- [fengshao1227/ccg-workflow](https://github.com/fengshao1227/ccg-workflow)
  - 本项目借鉴了它“强阶段隔离”的流程思想：spec -> plan -> execute -> review。
  - 也借鉴了“通过流程约束减少模型即兴发挥”的思路，而不是把整个任务塞进一个大 prompt。
  - 但两者目标不同：CCG 更偏工程开发协作；本仓库把这些思想本地化成学术场景里的 `I5 -> I6 -> I7 -> I8` Stage-I 任务，以及 `RESEARCH/[topic]/` 下的合同化产物。
- [GuDaStudio/skills](https://github.com/GuDaStudio/skills)
  - 这个项目对 Claude-oriented skill 打包方式，以及 Codex / Gemini 协作能力的可安装化，提供了很好的参考。
  - 但本仓库的重点不同：`GuDaStudio/skills` 更像通用协作 skill 集合，而 `qiongli` 更强调“单一研究合同 + 单一任务目录 + 单一产物树”的学术工作流。
- [Matt Pocock 的 `grill-me` skill](https://github.com/mattpocock/skills/blob/main/skills/productivity/grill-me/SKILL.md)
  - 本项目 credit 它“一次只追问一个问题、同时给推荐答案”的交互模式。
  - Qiongli 将它改造成 academic idea-discovery loop：不追问通用软件方案，而是检验一个 topic 是否能成为有 claim 强度、证据阈值、rival explanations、可行性和审稿风险边界的学术 idea。

---

## 🚀 快速开始 (0 → 1)

这是从“还没装”到“开始跑 canonical task”的最短稳定路径。

需要细节时，优先看已经整理好的文档入口：

- [快速开始](docs/zh/quickstart.md)
- [安装指南](docs/zh/guide/install.md)
- [CLI 参考](docs/zh/reference/cli.md)
- [系统架构](docs/zh/architecture.md)
- [Controller Modes](docs/advanced/controller-modes.md)

### 0. 先选安装路径

如果你只想在单个客户端里安装，推荐走各客户端自己的原生扩展入口：

- **Codex：** 添加统一的 [Skillsplace](https://github.com/jxpeng98/skillsplace) marketplace，然后在 Codex plugin UI 中安装或启用默认 `qiongli`，或选择 `qiongli-economics` 等 subject entry。Codex plugin 也内置本地 Node literature-provider MCP runtime，这部分 MCP 工具不需要 `qiongli` CLI。
- **Claude Code：** 添加统一的 [Skillsplace](https://github.com/jxpeng98/skillsplace) marketplace，然后安装 `qiongli@skillsplace`；经济学专精可安装 `qiongli-economics@skillsplace`。
- **Claude Desktop / Claude.ai：** 如果不想处理 code / CLI 环境，从 GitHub Release assets 下载 focused subject ZIP。默认通用包用 `qiongli-claude-desktop-skill-core-<tag>.zip`，经济学专精包用 `qiongli-claude-desktop-skill-economics-<tag>.zip`，political economy 专精包用 `qiongli-claude-desktop-skill-political-economy-<tag>.zip`，geoeconomics 专精包用 `qiongli-claude-desktop-skill-geoeconomics-<tag>.zip`，business 专精包用 `qiongli-claude-desktop-skill-business-<tag>.zip`，finance 专精包用 `qiongli-claude-desktop-skill-finance-<tag>.zip`，官方 economics/accounting 交叉学科包用 `qiongli-claude-desktop-skill-economics-accounting-<tag>.zip`。然后拖拽到 Claude Desktop 的 Skills 上传/安装流程中，或在 `Customize > Skills > + > Create skill > Upload a skill` 中上传。旧名 `qiongli-claude-desktop-skill-<tag>.zip` 暂时保留为 core alias。
- **Gemini CLI：** 从 `packages/qiongli-plugin` 本地安装 Gemini extension；发布为独立 extension 仓库或 gallery 条目后，也可以从远端安装。

公开的 Codex / Claude marketplace catalog 现在由 `jxpeng98/skillsplace` 统一维护。Release 构建会为 `core`、`economics`、`accounting`、`business`、`finance`、`political-economy`、`geoeconomics`、`economics-accounting` 生成独立 Codex / Claude Code plugin artifacts，让 marketplace 可以展示多个 subject 安装选项。本仓库保留这些生成 artifacts 的源 plugin payload 和平台 manifest：

Beta / prerelease 测试使用单独的 `qiongli-next` entry。它只发布 core 版本，不生成 subject plugin variants；Codex 和 Claude Code 的 `qiongli-next` artifacts 仍然内置 Node literature-provider MCP runtime。Claude Desktop 测试使用 `qiongli-next-claude-desktop-skill-core-<tag>.zip`，并与单独的 `qiongli-literature-provider-<version>.mcpb` 配合使用。CLI prerelease 测试使用 `npx qiongli@next install --target all --project-dir "$PWD"`。

- `packages/qiongli-plugin/.codex-plugin/plugin.json`
- `packages/qiongli-plugin/.mcp.json`
- `packages/qiongli-plugin/mcp/qiongli-literature-provider/`
- `packages/qiongli-plugin/.claude-plugin/plugin.json`
- `packages/qiongli-plugin/gemini-extension.json`
- `packages/qiongli-plugin/platforms/`
- `plugins/qiongli/skills/qiongli-workflow`（materialized artifact）

Claude Desktop 不走 Claude Code 的第三方 plugin marketplace 路径。Desktop 使用上面的 GitHub Release ZIP 手动上传；ZIP 内部顶层目录是 `qiongli/`，与 `SKILL.md` 里的 skill 名称一致。

Desktop/Web ZIP 使用 `coverage=focused`，用于保持当前 180 文件上传预算。它是 subject 专精安装包，不是降质删减版：保留统一 workflows、prompts、templates、standards、所选 profiles、`skills-summary.md` 和 `skills-core.md`；专精 ZIP 还包含经过 layered overlays 生成的 selected effective skill markdown。这个 Desktop skill ZIP 是 skill-only asset：只包含 workflows/prompts/templates，不保存 secrets，也不执行 provider calls。需要全量 canonical 源码细节时，使用默认 `coverage=complete` 的 CLI/npm 安装、Codex / Claude Code / Gemini plugin 包或源码仓库。

独立的 Qiongli Literature Provider `.mcpb`（`qiongli-literature-provider.mcpb`）才是 Claude Desktop 本地 provider asset。它在本地运行 Desktop literature search，支持 OpenAlex 和 Semantic Scholar，并通过 Desktop 配置 UI 填写 OpenAlex email 和 Semantic Scholar API key；敏感 key 交给 Claude Desktop sensitive-field handling，不写入 Desktop skill ZIP。这个 MCPB 自带零依赖 Node stdio server，所以 Desktop 用户不需要安装 `qiongli` CLI 或运行 npm install。CLI、Codex 和 Claude Code 用户仍然可以运行 `qiongli provider setup`，再用 `qiongli provider doctor` 检查当前是 `provider_connected` 还是 `strategy_only`。Desktop 用户需要 `qiongli-literature-provider` MCPB 或平台原生搜索能力，才能声称 `provider_connected`；如果没有 MCPB 或平台原生搜索能力，就把运行记录为 `strategy_only`，并把平台搜索或用户提供的 corpus 作为证据来源。

### Subject 专精安装

Subject packaging 需要同时区分两个视角：用户选择安装形态，开发者判断专精内容应该放在哪一层。完整说明见 [Subject Packaging Model](docs/zh/advanced/subject-packaging-model.md)。

对用户来说：

| 需求 | 安装形态 | 命令 |
|---|---|---|
| 不知道选什么 | `core / complete` | `qiongli install --target all` |
| 全量框架 + economics 专精 | `economics / complete` | `qiongli install --subject economics --target all` |
| 全量框架 + accounting 专精 | `accounting / complete` | `qiongli install --subject accounting --target all` |
| 全量框架 + business 专精 | `business / complete` | `qiongli install --subject business --target all` |
| 全量框架 + finance 专精 | `finance / complete` | `qiongli install --subject finance --target all` |
| 全量框架 + political economy 专精 | `political-economy / complete` | `qiongli install --subject political-economy --target all` |
| 全量框架 + geoeconomics 专精 | `geoeconomics / complete` | `qiongli install --subject geoeconomics --target all` |
| 轻量 economics 包 | `economics / focused` | `qiongli install --subject economics --coverage focused --target all` |
| 官方 economics/accounting 交叉学科包 | `economics-accounting / complete` | `qiongli install --subject economics-accounting --target all` |
| 更新 CLI 后刷新 accounting | `accounting / complete` | `qiongli upgrade --subject accounting --target all` |

对开发者来说，`core` 负责共享 workflow contracts、generic skills、templates、standards 和 quality gates。specialized subject 通过 selected profiles、append overlays、声明式 section replacements 和少量 subject-specific skills 增加学科深度。generic skills 源文件不会复制成学科版本；effective package 由 `skill_refs`、subject overlays、分层 section overrides 和可选本地 custom overlays 生成。

当前官方 subjects 包括 `core`、`economics`、`accounting`、`business`、`finance`、`political-economy`、`geoeconomics` 和官方组合 subject `economics-accounting`。默认安装是 `core/complete`。`--subject economics`、`--subject business`、`--subject finance`、`--subject political-economy` 和 `--subject geoeconomics` 表示 complete 专精安装，不是缩水包；`--subject accounting` 表示 `accounting/complete`，即全量框架加 accounting 专精。`--coverage focused` 是有意选择的精简路径，也是 Desktop/Web ZIP 路径。本阶段公开 Desktop ZIP subjects 是 `core`、`economics`、`business`、`finance`、`political-economy`、`geoeconomics` 和 `economics-accounting`；还没有 standalone accounting Desktop ZIP。`political-economy` 和 `geoeconomics` 是两个独立 subject，不是彼此叠加的交叉学科包。官方 composite subjects 是命名 subject，不是任意逗号分隔叠加。切换 subject 或 coverage 时，重新运行 install 或 upgrade；同一客户端一次只有一个 active `qiongli-workflow` package。

开发或加深一个 subject 时，需要同步更新：`content/subjects/catalog.yaml`、subject overlays、subject-specific registry and markdown、选定的 domain and venue profiles、subject eval fixtures、specialization audit expected terms、materializer tests、该 subject 可通过 npm 安装时的 npm payload tests，以及该 subject 有 Desktop/Web artifact 时的 release validation。

### 本地自定义

当个人、课题组或项目需要本地 overlays、profiles 或 custom skills，但不想修改 canonical Qiongli source 时，可以创建本地 custom subject layer。这个 scaffold / materialization 路径面向 Python/source checkout 工作流；custom overlays 只影响 materialize 后的 generated output，不会改写 canonical source files。

```bash
qiongli customize --subject economics --name my-econ-lab --out ./qiongli-custom/econ-lab
python3 scripts/materialize_subject_package.py --subject economics --custom-dir ./qiongli-custom/econ-lab --source . --out /tmp/qiongli-workflow
```

npm runtime installs 只使用预生成 payloads；在这个阶段不接受 runtime `--custom-dir`。

如果你需要跨客户端全局安装、多端 slash command、`qiongli upgrade`、`doctor` 或多模型 orchestrator，再使用下面的 bootstrap / CLI 路径。

### 1. 先选 `partial` 还是 `full`

一键 bootstrap 提供两个 profile。`partial` 只安装跨客户端 skill 包和 workflow 发现入口；`full` 还会安装本地 shell CLI，并在已有 Python 运行时的前提下执行 orchestrator 预检。

| Profile | 你会得到什么 | 安装前是否要求 Python | 安装后结果 |
|---|---|---|---|
| `partial` | 仅全局 skills | 否 | 资产可用，但 orchestrator 还没准备好 |
| `full` | `partial` + `qiongli` / `ql` shell CLI 和兼容别名 + `doctor` | 是，需要 Python 3.12+ | orchestrator 运行时可直接使用 |

`full` 模式的真实行为：

- 如果系统里已经有 `python3 >= 3.12`，bootstrap 会直接复用。
- 如果 Python 缺失或版本过低，bootstrap 会快速失败并打印安装建议。它不会自动安装 Python 或 `mise`。
- Windows 上由 PowerShell 直接安装，只有在 shell CLI 包装器需要 Bash 时才会通过 `winget` 安装 Git for Windows。

如果你不传 `--profile`，脚本会先解释两种模式，再提示你选择。

### 2. 运行一键安装

Linux / macOS：

```bash
curl -fsSL https://raw.githubusercontent.com/jxpeng98/qiongli/main/scripts/bootstrap_qiongli.sh | bash -s -- --project-dir "$PWD" --target all
```

Windows PowerShell 7+：

```powershell
winget install --id Microsoft.PowerShell --source winget
Invoke-WebRequest https://raw.githubusercontent.com/jxpeng98/qiongli/main/scripts/bootstrap_qiongli.ps1 -OutFile .\bootstrap_qiongli.ps1
pwsh -ExecutionPolicy Bypass -File .\bootstrap_qiongli.ps1 -ProjectDir "$PWD" -Target all
```

如果你想跳过交互，直接指定 profile：

```bash
# Linux / macOS
curl -fsSL https://raw.githubusercontent.com/jxpeng98/qiongli/main/scripts/bootstrap_qiongli.sh | bash -s -- --profile partial --project-dir "$PWD" --target all
curl -fsSL https://raw.githubusercontent.com/jxpeng98/qiongli/main/scripts/bootstrap_qiongli.sh | bash -s -- --profile full --project-dir "$PWD" --target all
```

```powershell
# Windows PowerShell 7+
# skills + workflow 资产安装
pwsh -ExecutionPolicy Bypass -File .\bootstrap_qiongli.ps1 -Profile partial -ProjectDir "$PWD" -Target all
# 完整安装（含 shell CLI 和 doctor；要求已有 Python 3.12+）
pwsh -ExecutionPolicy Bypass -File .\bootstrap_qiongli.ps1 -Profile full -ProjectDir "$PWD" -Target all
# 测试版本
pwsh -ExecutionPolicy Bypass -File .\bootstrap_qiongli.ps1 -Beta -Profile full -ProjectDir "$PWD" -Target all
```

这一步会安装：

- Codex / Claude Code / Gemini 的 workflow 资产
- 项目集成文件，例如 `.agent/workflows/`、`CLAUDE.md`、`.gemini/`，仅在执行 `qiongli init` 或 `--parts project` 时写入
- `full` 模式下的 shell CLI：`qiongli`、`ql`，以及兼容别名 `research-skills`、`rsk`、`rsw`

### npm / npx 替代入口

如果你更偏向 Node 生态，可以直接用 npm 包。这个入口是独立安装器，不依赖 PyPI：

```bash
npm install -g qiongli
qiongli install --target all --project-dir "$PWD"
qiongli install --subject economics --target all --project-dir "$PWD"
qiongli install --subject accounting --target all --project-dir "$PWD"
qiongli install --subject political-economy --target all --project-dir "$PWD"
qiongli install --subject geoeconomics --target all --project-dir "$PWD"
qiongli install --subject economics-accounting --target all --project-dir "$PWD"
```

如果只是测试 prerelease，不想全局安装：

```bash
npx qiongli@next install --subject economics --target all --project-dir "$PWD"
npx qiongli@next check --json
```

npm 包内携带预生成的 `core`、`economics`、`accounting`、`business`、`finance`、`political-economy`、`geoeconomics` 与 `economics-accounting` payload，并同时提供 `complete` / `focused` coverage。`--subject` 默认是 `core`，`--coverage` 默认是 `complete`；只有在明确需要精简包时才使用 `--coverage focused`。`qiongli check --json` 会显示 bundled payload subject/coverage 和各 target 已安装 subject/coverage。`qiongli doctor`、`qiongli task-run`、`qiongli team-run` 等高级命令会委托到 npm 包内置的 Python bridge 源码执行，因此仍要求本机已有 Python 3.12+ 和 `PyYAML`。

npm、pipx、pip 或 bootstrap 安装 CLI 后，推荐第一个运行：

```bash
qiongli setup
qiongli setup --dry-run
qiongli setup --project-dir "$PWD" --no-doctor
```

setup wizard 面向 CLI、Codex 和 Claude Code 用户，会交互式引导选择 install 或 upgrade、runtime surface（`cli`、`codex`、`claude-code` 或 `multi-platform`）、subject、coverage（`complete` 或 `focused`）、`--mode copy|link`、install scope（`all`、`globals`、`project` 或 `cli`）、CLI 目录、`--overwrite` / `--no-overwrite`、upgrade source（`--repo`、`--ref`、`--ref-type` 或 beta）、可选 literature provider keys，并在最后执行 doctor verification，除非使用 `--no-doctor`。每一步 prompt 都会显示简短的 `Tip:` 注释，解释这个选择会改变什么。

在 npm 安装中，`qiongli setup` 会委托到 npm 包内置的 Python bridge，因此要求本机已有 Python 3.12+ 和 `PyYAML`。如果只需要 Node-based asset installation，继续使用显式 `qiongli install ...` 命令。

通过 `qiongli setup` 输入的 provider 密钥使用与 `qiongli provider setup` 和 `qiongli provider doctor` 相同的 provider 配置。密钥会保存在生成的研究 artifacts 之外。setup 只负责配置凭据并执行 doctor/capability 检查，不承诺一定能运行外部检索。

### 3. 为 `full` 准备 Python

只有在你要使用 `full`、`doctor`、orchestrator、validator 或 tests 时，才需要提前准备 Python 3.12+。

推荐用 `mise`：

```bash
# Linux / macOS
curl https://mise.run | sh
```

```bash
# bash
echo 'eval "$(mise activate bash)"' >> ~/.bashrc
source ~/.bashrc
```

```bash
# zsh
echo 'eval "$(mise activate zsh)"' >> "${ZDOTDIR-$HOME}/.zshrc"
source "${ZDOTDIR-$HOME}/.zshrc"
```

```powershell
# Windows
scoop install mise
```

```powershell
# Windows 备用方式
winget install jdx.mise
```

```bash
mise install python@3.12
mise use -g python@3.12
python3 --version
```

### 4. 先选入口

稳定入口现在有三类：

- `.agent/workflows/*.md` 里的 workflow 命令，例如 `/paper`、`/lit-review`、`/paper-write`、`/code-build`
- 安装 / 升级 CLI：`qiongli`、`ql`，以及兼容别名 `research-skills`、`rsk`、`rsw`
- Orchestrator CLI：`python3 -m bridges.orchestrator ...`

### 5. 可选：本地安装器与刷新路径

如果机器已经有 Python，也可以改用跨平台本地安装器：

```bash
python3 scripts/bootstrap_qiongli.py --profile partial --project-dir .
python3 scripts/bootstrap_qiongli.py --profile full --project-dir .
```

如果机器已经有 Python，且你只想继续使用 Python 分发的升级器 CLI，这条路径依然保留：

```bash
pipx install qiongli
```

但 `pip` / `pipx` 现在只是兼容性的 CLI 分发方式，不再是推荐的首次安装入口。

从项目目录刷新已有安装时：

```bash
qiongli upgrade --target all --project-dir . --doctor
```

如果你已经跑过上面的 shell bootstrap，后续刷新时重新执行 bootstrap 或 `qiongli upgrade --overwrite` 即可。

*Python 边界：shell 版 `qiongli check|upgrade|align` 不需要 Python；`--doctor`、`python3 -m bridges.orchestrator ...`、validator 和 tests 仍然需要 `python3`。*

### 6. 先做环境检查

如果机器有 Python，建议在跑大任务前先做稳定预检：

```bash
python3 -m bridges.orchestrator doctor --cwd .
python3 scripts/validate_research_standard.py --strict
```

其中：

- `doctor` 负责检查运行时 CLI、API key、MCP wiring
- validator 负责检查仓库级 contract / schema 一致性

### 6. 先 plan 再 run

先看任务依赖、产物路径和路由：

```bash
python3 -m bridges.orchestrator task-plan \
  --task-id F3 \
  --paper-type empirical \
  --topic ai-in-education \
  --cwd .
```

`task-plan` 会展示：

- contract outputs
- 前置任务
- functional owner 与 handoff trace
- runtime plan（`draft` / `review` / `fallback`）

### 7. 运行 canonical research task

```bash
python3 -m bridges.orchestrator task-run \
  --task-id F3 \
  --paper-type empirical \
  --topic ai-in-education \
  --cwd . \
  --triad
```

常用控制项：

- `--focus-output` 与 `--output-budget`：缩小 active outputs，减少辅助文件扩散
- `--research-depth deep` 配合 `--max-rounds`：强制更窄、更有对抗性的证据扩展与修订流程
- `--only-target <id>`：对结构化 Stage-I 任务 `I4`-`I8`，回读现有 artifact，并且只重跑指定 actionable target
- Controller-aware flags：`--execution-mode solo|duo|triad`、`--controller`、`--primary`、`--reviewer`、`--verifier`、`--solo-role-gates strict|standard|off`
- 严格 controller-mode validation：非法 controller 参数会被 CLI 拒绝；需要缺失 provider 或 skill spec 直接阻断时，配合 `--mcp-strict` 与 `--skills-strict`

示例：只重跑一个 planning step

```bash
python3 -m bridges.orchestrator task-run \
  --task-id I6 \
  --paper-type methods \
  --topic llm-bias \
  --cwd . \
  --only-target S1
```

示例：Claude-primary duo 写作执行，并由 Codex 复核

```bash
python3 -m bridges.orchestrator task-run \
  --task-id F3 \
  --paper-type empirical \
  --topic ai-in-education \
  --cwd . \
  --execution-mode duo \
  --controller claude \
  --primary claude \
  --reviewer codex \
  --mcp-strict \
  --skills-strict
```

更多 controller-aware task-run 约定、solo gates 和 disagreement 处理见 [Controller Modes](docs/advanced/controller-modes.md)、[Solo Mode](docs/advanced/solo-mode.md) 与 [Codex-Claude Duo](docs/advanced/codex-claude-duo.md)。

### 8. 运行严格学术代码流

当代码本身是研究产物，而不是泛工程实现时，用 `code-build`：

```bash
python3 -m bridges.orchestrator code-build \
  --method "Staggered DID" \
  --topic policy-effects \
  --domain econ \
  --focus full \
  --cwd .
```

带上 `--topic` 后，`code-build` 会进入严格 Stage-I 路径：

- `I5` code specification
- `I6` zero-decision planning
- `I7` execution + performance packaging
- `I8` review

并且支持 targeted follow-up：

```bash
python3 -m bridges.orchestrator code-build \
  --method "Transformer Fine-Tuning" \
  --topic llm-bias \
  --domain cs \
  --focus full \
  --only-target I5:decision-1 \
  --only-target I8:P1-01 \
  --cwd .
```

### 9. 需要 slash-command UX 时再用 workflow 命令

如果你的客户端已经挂载了 workflow 入口 markdown，可以直接用这些命令：

| 命令 | 用途 | 示例 |
|------|------|------|
| `/paper` | 论文写作工作流入口（基于对话选择） | `/paper ai-in-education CHI` |
| `/lit-review` | 系统性文献综述 | `/lit-review transformer architecture 2020-2024` |
| `/paper-read` | 深度阅读单篇论文 | `/paper-read https://arxiv.org/abs/2303.08774` |
| `/find-gap` | 识别研究空白（5种 Gap） | `/find-gap LLM in education` |
| `/build-framework` | 构建理论框架与概念图谱 | `/build-framework technology acceptance` |
| `/academic-write` | 学术段落/章节写作辅助 | `/academic-write introduction AI ethics` |
| `/paper-write` | 完整论文（草稿端到端） | `/paper-write ai-in-education empirical CHI` |
| `/synthesize` | 证据综合 / Meta 分析规划 | `/synthesize ai-in-education` |
| `/study-design` | 实证研究设计 | `/study-design ai-in-education` |
| `/ethics-check` | 伦理评估与 IRB 审查材料 | `/ethics-check ai-in-education` |
| `/submission-prep` | 投稿材料打包生成 | `/submission-prep ai-in-education CHI` |
| `/rebuttal` | 审稿意见回复与矩阵生成 | `/rebuttal ai-in-education` |
| `/code-build` | 严格 Stage-I 学术代码流 | `/code-build "Staggered DID" --topic policy-effects --domain econ --focus full` |
| `/proofread` | AI 去痕与终审校对 | `/proofread ai-in-education` |
| `/academic-present` | 学术报告制作 | `/academic-present ai-in-education --format slidev` |

---

## CLI 安装与参数说明

这一节只说明“安装器/升级器 CLI”本身，不展开 orchestrator 的研究执行参数。

### 1. CLI 有哪几种安装方式

#### 方案 A：Shell bootstrap 安装 CLI（推荐）

适用场景：
- 你想使用 shell CLI，但不想先安装 PyPI 包
- 你想快速使用 `qiongli` / `ql`，同时保留 `rsk` / `rsw` 兼容入口
- 你希望同时把 workflow 资产也装好

命令：

```bash
cd /path/to/your/project
curl -fsSL https://raw.githubusercontent.com/jxpeng98/qiongli/main/scripts/bootstrap_qiongli.sh | bash -s -- \
  --project-dir "$PWD" \
  --target all
```

效果：
- 安装 shell CLI：`qiongli`、`ql`，以及兼容别名 `research-skills`、`rsk`、`rsw`
- 安装 `qiongli-workflow` skill 到对应客户端目录
- 项目集成文件只在执行 `qiongli init` 或 `--parts project` 时写入，例如 `.agent/workflows/`、`CLAUDE.md`、`.gemini/`

默认 CLI 目录：
- `${QIONGLI_BIN_DIR:-${RESEARCH_SKILLS_BIN_DIR:-~/.local/bin}}`

如果装完后命令不可用，通常是因为这个目录不在 `PATH` 中。可在 shell 配置里加入：

```bash
export PATH="$HOME/.local/bin:$PATH"
```

#### 方案 B：通过 npm / npx 安装 npm CLI

适用场景：
- 机器上已经有 Node.js
- 你想使用 npm 原生安装入口，并由 npm 包直接携带 skills payload
- 你不想为了安装 skills 先安装 PyPI 包

命令：

```bash
npm install -g qiongli
qiongli install --target all --project-dir "$PWD"
qiongli install --subject economics --target all --project-dir "$PWD"
qiongli install --subject accounting --target all --project-dir "$PWD"
qiongli install --subject economics --coverage focused --target all --project-dir "$PWD"
```

prerelease 测试：

```bash
npx qiongli@next install --subject economics --target all --project-dir "$PWD"
```

效果：
- 安装 npm 版 `qiongli`
- 安装 `qiongli-workflow` skill 到对应客户端目录
- npm 包内置 Python bridge 源码，供 `doctor`、`task-run`、`team-run` 等高级命令委托使用

npm 包没有 `postinstall` hook。安装 npm 包本身不会修改用户 skill 目录；只有执行 `qiongli install` 或 `qiongli upgrade` 时才会写入资产。

#### 推荐的 CLI Setup Wizard

通过 bootstrap、npm、pipx 或 pip 安装 shell CLI 后，先运行 setup wizard：

```bash
qiongli setup
qiongli setup --dry-run
qiongli setup --project-dir "$PWD" --no-doctor
```

wizard 会引导 CLI、Codex 和 Claude Code 安装选择 install 或 upgrade、runtime surface（`cli`、`codex`、`claude-code` 或 `multi-platform`）、subject、coverage（`complete` 或 `focused`）、`--mode copy|link`、install scope（`all`、`globals`、`project` 或 `cli`）、CLI 目录、`--overwrite` / `--no-overwrite`、可选 upgrade source、literature provider key setup，以及 doctor verification。每一步 prompt 都会显示简短的 `Tip:` 注释，解释这个选择会改变什么。

在 npm 安装中，`qiongli setup` 会委托到 npm 包内置的 Python bridge，因此要求本机已有 Python 3.12+ 和 `PyYAML`。如果只需要 Node-only installer path，继续使用显式 `qiongli install ...` 命令。

通过 setup 输入的 provider 密钥使用与 `qiongli provider setup` 和 `qiongli provider doctor` 相同的 provider 配置。密钥保存在生成的研究 artifacts 之外。provider 步骤会配置凭据并检查 capability，不应被描述成一定会执行外部 literature search。

#### 方案 C：通过 `pipx` 安装 Python CLI

适用场景：
- 机器上已经有 Python
- 你想继续使用现有 PyPI 分发方式

命令：

```bash
pipx install qiongli
qiongli setup
```

`qiongli setup` 会交互式引导同一组选项。脚本化安装仍可继续使用上文记录的 `qiongli upgrade` 或显式 `qiongli install ...` 命令。

效果：
- 安装 Python 版 `qiongli` / `ql`，以及兼容别名 `research-skills` / `rsk` / `rsw`
- CLI 本身进入 PATH
- 不会自动把 workflow 资产写入你的项目，仍需手动执行 `qiongli upgrade`

#### 方案 D：从本地仓库安装 shell CLI

适用场景：
- 你已经 clone 了这个仓库
- 你希望控制安装目录，或用 `link` 模式维护本地副本

命令：

```bash
./scripts/install_qiongli.sh \
  --target all \
  --project-dir /path/to/project \
  --install-cli \
  --overwrite
```

### 2. Shell bootstrap 参数说明

入口脚本：
- `scripts/bootstrap_qiongli.sh`

常用参数：

| 参数 | 作用 | 默认值 / 说明 |
|------|------|---------------|
| `--repo <owner/repo|git-url>` | 指定上游 GitHub 仓库 | 默认取 `QIONGLI_REPO`，再回退到旧 `RESEARCH_SKILLS_REPO`，否则 `jxpeng98/qiongli` |
| `--ref <tag-or-branch>` | 指定安装的版本或分支 | 默认自动解析 latest release |
| `--ref-type <tag|branch>` | 指定 `--ref` 是 tag 还是 branch | 默认 `tag` |
| `--beta` | 在未传 `--ref` 时安装最新 beta / prerelease tag | 默认关闭，默认仍解析稳定版 latest release |
| `--target <codex|claude|gemini|antigravity|all>` | 指定写入哪些客户端目录 | 默认 `all` |
| `--project-dir <path>` | 在启用项目侧安装面时，指定项目集成文件的写入目录 | 默认当前目录 |
| `--install-cli` | 安装 shell CLI | 默认开启 |
| `--no-cli` | 跳过 shell CLI 安装，只装 workflow 资产 | 与 `--install-cli` 相反 |
| `--cli-dir <path>` | 指定 shell CLI 安装目录 | 默认 `${QIONGLI_BIN_DIR:-${RESEARCH_SKILLS_BIN_DIR:-~/.local/bin}}` |
| `--overwrite` | 覆盖已存在的 skill / CLI / 项目文件 | 默认关闭 |
| `--doctor` | 安装后运行环境预检 | 仅在存在 `python3` 时执行 |
| `--dry-run` | 只打印将要执行的动作 | 不实际下载和写文件 |

示例：

```bash
# 安装指定 tag
curl -fsSL https://raw.githubusercontent.com/jxpeng98/qiongli/main/scripts/bootstrap_qiongli.sh | bash -s -- \
  --repo jxpeng98/qiongli \
  --ref v0.1.0 \
  --ref-type tag \
  --project-dir "$PWD" \
  --target all \
  --overwrite

# 安装最新 beta / prerelease
curl -fsSL https://raw.githubusercontent.com/jxpeng98/qiongli/main/scripts/bootstrap_qiongli.sh | bash -s -- \
  --profile full \
  --beta \
  --project-dir "$PWD" \
  --target all

# 只装 workflow，不装 CLI
curl -fsSL https://raw.githubusercontent.com/jxpeng98/qiongli/main/scripts/bootstrap_qiongli.sh | bash -s -- \
  --project-dir "$PWD" \
  --target claude \
  --no-cli

# 预演安装动作
curl -fsSL https://raw.githubusercontent.com/jxpeng98/qiongli/main/scripts/bootstrap_qiongli.sh | bash -s -- \
  --project-dir "$PWD" \
  --target codex \
  --dry-run
```

### 3. 本地安装脚本参数说明

入口脚本：
- `scripts/install_qiongli.sh`

常用参数：

| 参数 | 作用 | 默认值 / 说明 |
|------|------|---------------|
| `--target <codex|claude|gemini|antigravity|all>` | 指定写入哪些客户端目录 | 默认 `all` |
| `--mode <copy|link>` | 复制文件或创建软链接 | 默认 `copy` |
| `--project-dir <path>` | 在启用项目侧安装面时，指定项目集成文件写入目录 | 默认当前目录 |
| `--install-cli` | 安装 shell CLI | 默认关闭 |
| `--no-cli` | 跳过 shell CLI 安装 | 默认行为 |
| `--cli-dir <path>` | 指定 shell CLI 安装目录 | 默认 `${QIONGLI_BIN_DIR:-${RESEARCH_SKILLS_BIN_DIR:-~/.local/bin}}` |
| `--overwrite` | 覆盖已有目标 | 默认关闭 |
| `--doctor` | 安装后运行 `python3 -m bridges.orchestrator doctor` | 仅在存在 `python3` 时执行 |
| `--dry-run` | 只打印将要执行的动作 | 不实际写文件 |

说明：
- 如果你想长期维护一个本地 clone，推荐 `--mode link`
- 如果你只想一次性安装，推荐 `--mode copy`
- `--mode link` 适合本地仓库安装，不适合远程 bootstrap

### 4. `qiongli` CLI 子命令与别名

shell CLI 和 Python CLI 都有这些入口名：
- `qiongli`
- `ql`
- `research-skills`（旧兼容入口）
- `rsk`
- `rsw`

共同支持的命令：
- `check`
- `upgrade`
- `align`

仅 Python CLI 提供：
- `doctor`
- `init`

#### `qiongli check`

用途：
- 查看本地已安装 skill 版本
- 查看上游最新 release
- 判断是否有可升级版本

参数：

| 参数 | 作用 |
|------|------|
| `--repo <owner/repo|url>` | 指定上游仓库 |
| `--json` | 输出 JSON，便于脚本或 CI 使用 |
| `--strict-network` | 若上游查询失败则返回失败 |

示例：

```bash
qiongli check
qiongli check --repo jxpeng98/qiongli
qiongli check --json
```

#### `qiongli upgrade`

用途：
- 下载上游 release/branch 压缩包
- 默认刷新全局 skill 安装，必要时再刷新 shell CLI
- 项目集成文件改为通过 `qiongli init` 或 `--parts project` 显式更新

常用参数：

| 参数 | 作用 |
|------|------|
| `--repo <owner/repo|url>` | 指定上游仓库 |
| `--ref <tag-or-branch>` | 指定版本或分支 |
| `--ref-type <tag|branch>` | 指定 ref 类型 |
| `--target <codex|claude|gemini|antigravity|all>` | 指定安装目标 |
| `--project-dir <path>` | 指定项目路径 |
| `--install-cli` | 安装或刷新 shell CLI 包装命令 |
| `--no-cli` | 升级时不刷新 shell CLI |
| `--cli-dir <path>` | 指定 shell CLI 目录 |
| `--parts <globals,project,cli,doctor>` | 只执行指定安装面 |
| `--overwrite` | 覆盖已有目标 |
| `--doctor` | 升级后执行 doctor |
| `--dry-run` | 预演升级动作 |

示例：

```bash
qiongli upgrade --target all --overwrite
qiongli upgrade --project-dir . --parts project,doctor
qiongli upgrade --repo jxpeng98/qiongli --ref main --ref-type branch --project-dir . --target claude
qiongli upgrade --project-dir . --target codex --dry-run
```

#### `qiongli doctor`（仅 Python CLI）

用途：
- 用更短的命令运行 `bridges.orchestrator doctor`

示例：

```bash
qiongli doctor --cwd .
```

#### `qiongli init`（仅 Python CLI）

用途：
- 直接从已安装的包初始化项目侧 workflow 资产，不需要重新下载 release 压缩包
- 这是全局安装/升级之后给项目接线的默认入口

常用参数：

| 参数 | 作用 |
|------|------|
| `--project-dir <path>` | 指定项目路径 |
| `--target <codex|claude|gemini|antigravity|all>` | 指定客户端/项目侧表面 |
| `--parts <globals,project,cli,doctor>` | 选择安装面（默认 `project`） |
| `--overwrite` | 覆盖已有项目资产 |
| `--doctor` | init 后执行 doctor |
| `--dry-run` | 预演 init 动作 |

示例：

```bash
qiongli init --project-dir .
qiongli init --project-dir . --target claude --overwrite
```

#### `qiongli align`

用途：
- 打印一个简短说明，告诉你 CLI 装了什么、`upgrade` 会改哪些路径

参数：

| 参数 | 作用 |
|------|------|
| `--repo <owner/repo|url>` | 仅用于在示例输出中替换 repo 提示 |

示例：

```bash
qiongli align
qiongli align --repo jxpeng98/qiongli
```

### 5. 常用环境变量

| 环境变量 | 作用 |
|----------|------|
| `QIONGLI_REPO` | 默认上游仓库，省去每次传 `--repo` |
| `QIONGLI_BIN_DIR` | shell CLI 默认安装目录 |
| `RESEARCH_SKILLS_REPO` | `QIONGLI_REPO` 的旧兼容 fallback |
| `RESEARCH_SKILLS_BIN_DIR` | `QIONGLI_BIN_DIR` 的旧兼容 fallback |
| `CODEX_HOME` | Codex skill 安装根目录 |
| `CLAUDE_CODE_HOME` | Claude Code skill 安装根目录 |
| `GEMINI_HOME` | Gemini skill 安装根目录 |
| `ANTIGRAVITY_HOME` | Antigravity 全局 skill 安装根目录 |
| `GITHUB_TOKEN` / `GH_TOKEN` | 私有仓库或 GitHub API 限流时的认证令牌 |

### 6. 什么时候需要 Python

不需要 Python 的部分：
- shell bootstrap 安装
- shell CLI 的 `check` / `upgrade` / `align`
- 本地安装脚本的 `copy/link` 资产安装

仍然需要 Python 的部分：
- `--doctor`
- `python3 -m bridges.orchestrator ...`
- 仓库内其他 validator / orchestrator / test 命令

---

## 🧬 Subject 专精安装与动态领域挂载

**Subject package 和 runtime domain 是什么关系？**

Qiongli 现在支持 subject-specialized installs。`core` 是默认通用包；`economics`、`accounting`、`business`、`finance`、`political-economy`、`geoeconomics` 和命名 composite subject `economics-accounting` 会安装同一套 canonical workflow，并叠加学科 overlays 与 subject-specific skills。CLI/npm 默认是 `coverage=complete`，所以专精安装会保留全量框架；`coverage=focused` 主要用于有意选择精简包和 Desktop/Web ZIP。

`--domain econ` 这类 runtime flag 仍然用于单次 task packet 的临时强调，但不再替代 subject packaging。当你希望某个客户端默认就是经济学专精工作流时，使用 `qiongli install --subject economics --target all`。如果只是当前任务需要临时领域约束，再使用 runtime domain。

---

## 🏗 标准化层与跨模型契约
为了让 Codex、Claude、Gemini 输出可相互继承的中间件，系统使用严苛的“契约”驱动运转。

- **工作流契约**: `content/standards/research-workflow-contract.yaml` (所有 Task ID，必需前置条件与质量门规范)
- **能力映射路由**: `content/standards/mcp-agent-capability-map.yaml` (所有 MCP 工具代理，自动 fallback 以及检查清单）
- **落盘规范**: 所有代理人生成的学术内容必须严格落进 `RESEARCH/[topic]/` 对应目录下。

### Skills + Agents 协同流程（ASCII）

```text
用户目标 / Prompt
        |
        v
Skill 路由层（Task ID + paper_type）
        |
        +--------------------------+
        |                          |
        v                          v
MCP 证据采集                  Agent 运行时路由
        |                          |
        +------------+-------------+
                     v
                  Draft 生成
                     |
                     v
                  Review 复核
                     |
         +-----------+-----------+
         |                       |
         v                       v
   Triad 三端审查（可选）   双端/单端自动降级
                     \       /
                      v     v
                 Summarizer 综合
                     |
                     v
           质量门 + 产物落盘输出
              -> RESEARCH/[topic]/...
```
*(详情请参考 [docs/zh/advanced/agent-skill-collaboration.md](docs/zh/advanced/agent-skill-collaboration.md)。)*

---

## 多模型并发审查 (`orchestrator`)

支持通过 Orchestrator 网桥，联动本地不同接口服务执行复合流。
*(需预先在环境变量配置了 `OPENAI_API_KEY`, `ANTHROPIC_API_KEY`, `GOOGLE_API_KEY`)*

```bash
# 先看任务前置、产物和路由
python3 -m bridges.orchestrator task-plan --task-id F3 --paper-type empirical --topic my-topic --cwd .

# 并发分析：三端平行背靠背审查，并由 Claude 做 Summary
python3 -m bridges.orchestrator parallel --prompt "分析数据的可靠性约束" --cwd . --summarizer claude

# 契约执行：强制按照 F3 的要求调度
python3 -m bridges.orchestrator task-run --task-id F3 --paper-type empirical --topic my-topic --cwd .

# 严格 Stage-I 学术代码流
python3 -m bridges.orchestrator code-build --method "Staggered DID" --topic my-topic --domain econ --focus full --cwd .

# 步进交互模式 (Interactive Mode)：在调用任何 Agent 前暂停并提示输入 Y/n 确认
python3 -m bridges.orchestrator task-run --task-id F3 --paper-type empirical --topic my-topic --cwd . -i

# MCP环境严格测试：如果没有相关搜素工具环境则强制阻挡
python3 -m bridges.orchestrator task-run --task-id B1 --paper-type systematic-review --topic my-topic --cwd . --mcp-strict

# 收敛辅助文件，并强化证据深度/修订深度
python3 -m bridges.orchestrator task-run --task-id F3 --paper-type empirical --topic my-topic --cwd . \
  --focus-output manuscript/manuscript.md \
  --research-depth deep \
  --draft-profile deep-research \
  --review-profile strict-review \
  --triad-profile deep-research \
  --triad \
  --max-rounds 4

# 只重开指定 Stage-I target
python3 -m bridges.orchestrator code-build --method "Transformer Fine-Tuning" --topic llm-bias --domain cs --focus full \
  --only-target I5:decision-1 \
  --only-target I8:P1-01 \
  --cwd .
```

`task-run` 的几个关键控制项：

- `--focus-output <path>`：可重复；只激活本次运行需要的 contract output。
- `--output-budget <n>`：限制本次运行最多处理多少个 contract outputs。
- `--research-depth deep`：显式要求证据扩展、反例搜索、边界条件检查与更窄结论。
- `--max-rounds <n>`：提高 review 阻断后的修订轮数。
- `--only-target <id>`：对 Stage-I 结构化产物，回读已有 artifact，并且只重跑指定 actionable target。
- `--execution-mode`、`--controller`、`--primary`、`--reviewer`、`--verifier`、`--solo-role-gates`：记录 solo / duo / triad 的严格 controller-mode ownership metadata。
- 内置 profiles：`focused-delivery`、`deep-research`、`strict-review`、`rapid-draft`、`default`。

---

## 支持接入的学术数据库映射

| API 来源 | 用途 | 覆盖范围 |
|--------|---------|----------|
| Semantic Scholar | 第一搜索源文献检索 | 200M+ 论文 |
| arXiv | 理工科预印本读取 | 全集 |
| OpenAlex | 文献计量与本体网络 | 250M+ 作品 |
| Crossref | DOI 源数据核对验证 | 140M+ DOIs |

---

## 开发者与贡献者指引

由于该项目为高度结构化的学术框架，禁止直接魔改导致 Schema 失效报错。

### CI 流水线与本地验证
如果你修改了 yaml 合同、修改了路由链路，或者修改了 `.md` 的依赖产物节点，请必须使用以下命令校验通过：

```bash
# 验证框架格式合同 (无 warning 方可合并)
python3 scripts/validate_research_standard.py --strict
# 运行单元测试
python3 -m unittest tests.test_orchestrator_workflows -v

# 验证你在项目里最新跑出来的数据结果结构是否与合同相符
python3 scripts/validate_project_artifacts.py --cwd ./project  --topic <topic> --task-id H1 --strict
```

如果你希望测试传统的底层安装脚本能力，请使用: `scripts/install_qiongli.sh`


### 发版自动化 (Release Automation)
由 CI 接管或手动拉草稿：
```bash
# 从 main/master 一条命令走完整稳定版发版
./scripts/release_automation.sh publish --tag v0.1.0 --from-tag v0.1.0-beta.6

# 从 dev 一条命令走完整 beta 发版
git switch dev
./scripts/release_automation.sh publish --tag v0.8.0-beta.1 --skip-bump --from-tag v0.7.0-beta.2

# 需要拆阶段时再手动执行
./scripts/release_automation.sh pre --tag v0.1.0 --from-tag v0.1.0-beta.6
./scripts/release_automation.sh post --tag v0.1.0 --create-release
```

---

## 目录结构介绍

```
qiongli/
├── content/                  # 学术内容 canonical source
│   ├── workflow/             # 生成 qiongli-workflow package 的源
│   ├── standards/            # workflow contract + capability map
│   ├── skills/               # internal skill specs
│   ├── templates/            # 可复用 artifact templates
│   ├── roles/                # functional-agent role configs
│   ├── subjects/             # subject catalog 与 overlays
│   └── schemas/              # Validator 数据验证
├── packages/                 # Python、npm、plugin、MCPB package sources
├── tooling/                  # scripts、pipelines、install、release assets
├── evals/                    # 性能、覆盖率与质量评测资产
├── docs/                     # VitePress 文档站
├── scripts/                  # 指向 tooling/scripts 的稳定 wrapper
└── tests/                    # 单元测试验证
```

许可协议: MIT
