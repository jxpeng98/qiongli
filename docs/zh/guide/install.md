# 安装 Qiongli

Qiongli 有多个安装入口，是因为不同用户需要的运行时能力不同。先选能满足目标的最小入口。

## 安装入口

| 入口 | 适合场景 | 安装内容 | 是否要求 Python |
|---|---|---|---|
| 原生 plugin / extension | 单个客户端，最少配置 | 客户端 plugin 和 `qiongli-workflow` | 否 |
| Claude Desktop Skill ZIP | Claude Desktop 或 Claude.ai，尤其适合不熟悉 code / CLI 环境的用户 | 个人上传的 `qiongli` Skill | 否 |
| Bootstrap `partial` | 多客户端全局 workflow assets | skills 和客户端支持的 workflow discovery | 否 |
| Bootstrap `full` | runtime check 和 orchestrator | `partial` 加 shell CLI 与 `doctor` 支持 | 是，Python 3.12+ |
| npm / npx | Node 自动化安装 | npm CLI 和内置 workflow payload | 只有高级 bridge 命令需要 |
| pipx / pip | Python updater CLI | Python CLI 分发 | 是 |

用户可见的 skill 名称是 `qiongli`。安装目录仍然是 `qiongli-workflow`，这是为了兼容已有客户端和 release artifacts。`core` 是默认 subject，所以默认安装是 `core/complete`。CLI/npm 专精安装默认使用 `coverage=complete`，也就是全量 Qiongli 框架加指定 subject 专精。

## 原生 Plugin 和 Extension

只需要在一个客户端里使用 Qiongli 时，优先选这个入口。

Codex 通过统一的 [Skillsplace](https://github.com/jxpeng98/skillsplace) marketplace 安装：

```bash
codex plugin marketplace add jxpeng98/skillsplace --ref main
codex plugin marketplace list
```

然后在 Codex plugin UI 中安装或启用 `qiongli`，这是默认 core package。也可以选择 `qiongli-economics`、`qiongli-accounting`、`qiongli-business`、`qiongli-finance`、`qiongli-political-economy`、`qiongli-geoeconomics`、`qiongli-economics-accounting` 这类 subject entry，它们会安装对应的 `subject/complete` package。

Claude Code 使用同一个 Skillsplace catalog：

```bash
claude plugin marketplace add jxpeng98/skillsplace@main
claude plugin install qiongli@skillsplace
# Subject 专精安装：
claude plugin install qiongli-economics@skillsplace
```

在 Claude Code 交互会话中，也可以使用 slash commands：

```text
/plugin marketplace add jxpeng98/skillsplace@main
/plugin install qiongli@skillsplace
/plugin install qiongli-economics@skillsplace
```

Claude Desktop 和 Claude.ai 不安装第三方 Claude Code plugin marketplace。如果你使用 Desktop 或网页版，并且不熟悉 code / CLI 环境，优先使用 release ZIP 路径，不需要任何终端命令：

1. 从 GitHub Release assets 下载 `qiongli-claude-desktop-skill-core-<tag>.zip`、`qiongli-claude-desktop-skill-economics-<tag>.zip`、`qiongli-claude-desktop-skill-business-<tag>.zip`、`qiongli-claude-desktop-skill-finance-<tag>.zip`、`qiongli-claude-desktop-skill-political-economy-<tag>.zip`、`qiongli-claude-desktop-skill-geoeconomics-<tag>.zip` 或 `qiongli-claude-desktop-skill-economics-accounting-<tag>.zip`。本阶段公开 Desktop ZIP subjects 是 `core`、`economics`、`business`、`finance`、`political-economy`、`geoeconomics` 和 `economics-accounting`；还没有 standalone accounting Desktop ZIP。
2. 在 Claude Desktop 中，把 ZIP 拖拽到 Skills 上传/安装流程中；也可以打开 `Customize > Skills`，点击 `+`，选择 `Create skill`，再选择 `Upload a skill`。
3. 在 Claude.ai 网页版中，使用同样的 `Customize > Skills` 上传流程，选择同一个 ZIP。
4. 启用上传后的 `qiongli` skill。

Release ZIP 使用 `coverage=focused`，用于保持当前 180 文件上传预算。它是 subject 专精 Desktop/Web 包，不是降质删减版：保留可执行 workflows、prompts、templates、standards、所选 profiles、`skills-summary.md` 和 `skills-core.md`；专精 ZIP 还包含通过 layered overlays 生成的 selected effective skill markdown。这个 Desktop skill ZIP 是 skill-only asset：只包含 workflows/prompts/templates，不保存 secrets，也不执行 provider calls。完整 canonical source 可通过默认 `coverage=complete` 的 CLI/npm 安装、Codex / Claude Code / Gemini plugin 包和源码仓库获得。

独立的 Qiongli Literature Provider `.mcpb`（`qiongli-literature-provider.mcpb`）才是 Claude Desktop 本地 provider asset。它在本地运行 Desktop literature search，支持 OpenAlex 和 Semantic Scholar，并通过 Desktop 配置 UI 填写 OpenAlex email 和 Semantic Scholar API key；敏感 key 交给 Claude Desktop sensitive-field handling，不写入 Desktop skill ZIP。CLI、Codex 和 Claude Code 用户可以运行 `qiongli provider setup`，再用 `qiongli provider doctor` 检查当前是 `provider_connected` 还是 `strategy_only`。Desktop 用户需要 `qiongli-literature-provider` MCPB 或平台原生搜索能力，才能声称 `provider_connected`；如果没有 MCPB 或平台原生搜索能力，就把运行记录为 `strategy_only`，并把平台搜索或用户提供的 corpus 作为证据来源。

Gemini CLI 仍然直接安装本地 extension payload：

```bash
gemini extensions install ./path/to/qiongli/plugins/qiongli
```

这条路径不会安装 shell CLI、Python bridge 或全局 slash-command symlinks。需要这些能力时，用 bootstrap 或 npm。

## 安装后如何使用

安装或升级后，先重启目标客户端。然后使用该客户端暴露的入口：

| 客户端 | 发现方式 | 调用方式 |
|---|---|---|
| Codex | `/skills` 应该能列出 `qiongli` | `$qiongli <research task>` |
| Claude Code | Plugin UI、`/plugin` 或全局 command discovery | `/paper`、`/lit-review`、`/paper-write`、`/code-build` |
| Gemini CLI | Extension list 或全局 workflow discovery | `/paper`、`/lit-review`、`/paper-write`、`/code-build` |
| Shell | `qiongli check` | `qiongli doctor`、`qiongli upgrade`、`python3 -m bridges.orchestrator ...` |

Codex 不暴露自定义 `/qiongli` slash command。先用 `/skills` 确认 skill 存在，再用 `$qiongli` 调用。

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
qiongli install --subject core --target all --project-dir "$PWD"
qiongli install --subject economics --target all --project-dir "$PWD"
qiongli install --subject accounting --target all --project-dir "$PWD"
qiongli install --subject economics-accounting --target all --project-dir "$PWD"
qiongli install --subject economics --coverage focused --target all --project-dir "$PWD"
```

一次性运行：

```bash
npx qiongli@latest install --subject economics --target all --project-dir "$PWD"
npx qiongli@latest install --subject economics --coverage focused --target all --project-dir "$PWD"
npx qiongli@latest check --json
```

测试 prerelease 时仍可使用 `next` dist-tag：

```bash
npx qiongli@next install --subject economics --target all --project-dir "$PWD"
```

## 推荐的 CLI Setup Wizard

通过 npm、pipx、pip 或 bootstrap script 安装 CLI 后，先运行交互式 setup wizard，再手写安装参数：

```bash
qiongli setup
qiongli setup --dry-run
qiongli setup --project-dir "$PWD" --no-doctor
```

wizard 会引导 CLI、Codex 和 Claude Code 用户完成：

- runtime surface：CLI、Codex、Claude Code 或 multi-platform
- subject 选择
- coverage 选择：`complete` 或 `focused`
- install scope：`all`、`globals`、`project` 或 `cli`
- 可选 literature provider key setup
- doctor verification，除非设置 `--no-doctor`

通过 setup 输入的 provider 密钥使用与 `qiongli provider setup` 和 `qiongli provider doctor` 相同的 provider 配置。密钥会保存在生成的研究 artifacts 之外。provider 步骤用于配置凭据并执行 doctor/capability 检查；它不保证一定产生外部检索结果。

在 npm 安装中，`qiongli setup` 会委托到 npm 包内置的 Python bridge，因此要求本机已有 Python 3.12+ 和 `PyYAML`。如果只需要 Node-only asset installer，继续使用显式 `qiongli install ...` 命令。

## pipx / pip

如果你明确需要 Python 分发的 updater CLI，用 pipx：

```bash
pipx install qiongli
qiongli setup
qiongli install --subject economics --target all
qiongli install --subject accounting --target all
qiongli install --subject political-economy --target all
qiongli install --subject geoeconomics --target all
qiongli install --subject economics-accounting --target all
```

`qiongli setup` 可以交互式引导同一组选项。脚本化安装仍可使用这里展示的 `qiongli upgrade` 或显式 `qiongli install ...` 命令。

升级：

```bash
pipx upgrade qiongli
qiongli upgrade --subject accounting --target all --doctor --project-dir /path/to/project
```

`--subject` 默认是 `core`，`--coverage` 默认是 `complete`。不确定怎么选时使用 complete：`--subject economics`、`--subject business`、`--subject finance`、`--subject political-economy` 和 `--subject geoeconomics` 表示 complete 专精安装，不是缩水包；`--subject accounting` 表示 `accounting/complete`，即全量框架加 accounting 专精。只有明确需要精简包或 Desktop/Web 等价包时才使用 `--coverage focused`。当前官方 subjects 是 `core`、`economics`、`accounting`、`business`、`finance`、`political-economy`、`geoeconomics` 和命名 composite subject `economics-accounting`；`political-economy` 和 `geoeconomics` 是两个独立 subject 选择，不是一个 composite。官方 composite subjects 不是任意逗号分隔叠加。切换 subject 或 coverage 时，重新运行 `install` 或 `upgrade` 并指定新参数。`qiongli check --json` 会输出每个 target 当前安装的 subject 和 coverage；旧安装缺少 `SUBJECT_MANIFEST.json` 或 `SUBJECT` 文件时按 legacy `core` / `complete` 处理。

先创建 custom scaffold，再 materialize 本地 overlays：

```bash
qiongli customize --subject economics --name my-econ-lab --out ./qiongli-custom/econ-lab
```

本地自定义 overlays 通过源码 materializer 支持：

```bash
python3 scripts/materialize_subject_package.py \
  --subject economics \
  --coverage complete \
  --source . \
  --custom-dir /path/to/custom-qiongli \
  --out /tmp/qiongli-workflow
```

这个路径适合添加本地 overlays、profiles、registry entries 或 custom skill markdown。Custom overlays 只影响 generated output，不会改写 canonical source files。`qiongli customize` 加 `--custom-dir` materialization 面向 Python/source checkout 工作流；npm runtime installs 在这个阶段使用预生成 payloads，不支持 `--custom-dir`。

## 实际会安装什么

根据入口不同，Qiongli 可能安装：

- 客户端 home 目录下的 `qiongli-workflow` skill assets，用户侧显示为 `qiongli`
- 在支持该 discovery 模型的客户端中，提供 `/paper`、`/lit-review`、`/paper-write`、`/code-build` 等 workflow command discovery links
- shell 命令 `qiongli`、`ql` 和兼容别名 `research-skills`、`rsk`、`rsw`
- 只有显式运行 `qiongli init --project-dir .` 时才写入的项目集成文件

默认不会写入项目本地文件。全局 workflow package 可在任意研究工作区使用。

完整调用细节见 [使用 Agent Skills](/zh/guide/using-agent-skills)。

## 保持版本一致

如果你同时使用多个安装面，保持 plugin、global skill assets、npm payload 和 Python CLI 一致：

```bash
qiongli check
qiongli upgrade --subject core --target all
```

如果你已经完全转向原生 plugin，不再需要旧的全局 skill 目录或 slash discovery，先 dry-run 清理：

```bash
qiongli clean --globals --dry-run
```
