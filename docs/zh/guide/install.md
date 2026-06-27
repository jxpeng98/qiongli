# 安装 Qiongli

Qiongli 有多个安装入口，是因为不同用户需要的运行时能力不同。先选能满足目标的最小入口。

## 最新稳定版下载

当前稳定版是 [v1.10.0](https://github.com/jxpeng98/qiongli/releases/tag/v1.10.0)。下面这些直达链接覆盖常见安装路径；需要 subject 专精 Desktop ZIP 或维护者 artifacts 时，再打开下载指南。

| 需求 | 链接或命令 |
|---|---|
| npm CLI | [`qiongli@1.10.0`](https://www.npmjs.com/package/qiongli/v/1.10.0)：`npm install -g qiongli@latest` |
| PyPI CLI | [`qiongli 1.10.0`](https://pypi.org/project/qiongli/1.10.0/)：`pipx install qiongli` |
| Claude Desktop/Web core skill | [`qiongli-claude-desktop-skill-core-v1.10.0.zip`](https://github.com/jxpeng98/qiongli/releases/download/v1.10.0/qiongli-claude-desktop-skill-core-v1.10.0.zip) |
| Claude Desktop literature MCPB | [`qiongli-literature-provider-0.1.5.mcpb`](https://github.com/jxpeng98/qiongli/releases/download/v1.10.0/qiongli-literature-provider-0.1.5.mcpb) |
| Zotero Desktop companion | [`qiongli-zotero-companion-0.2.2.xpi`](https://github.com/jxpeng98/qiongli/releases/download/v1.10.0/qiongli-zotero-companion-0.2.2.xpi) |
| 全部 release assets | [下载指南](https://github.com/jxpeng98/qiongli/releases/download/v1.10.0/qiongli-downloads-v1.10.0.md) 和 [GitHub Release](https://github.com/jxpeng98/qiongli/releases/tag/v1.10.0) |

## 安装入口

| 入口 | 适合场景 | 安装内容 | 是否要求 Python |
|---|---|---|---|
| Marketplace plugin / extension | 单个客户端、最少配置，或没有本地 CLI 环境 | 客户端 plugin 和 `qiongli-workflow`；Codex 和 Claude Code 内置 Node literature MCP，作为 lite/no-CLI fallback | skill 使用和内置 literature MCP 不要求；完整本地 Qiongli 需要 Python/CLI |
| Claude Desktop Skill ZIP | Claude Desktop 或 Claude.ai，尤其适合不熟悉 code / CLI 环境的用户 | 个人上传的 `qiongli` Skill | 否 |
| 本地 full plugin：`qiongli install --target all` | 在客户端原生 plugin 容器中使用完整本地 Qiongli | 让 CLI 生成客户端 plugin，并接入统一 full MCP | 是，Python 3.12+ |
| 旧版 skills-only：`qiongli install --surface skills --profile partial` | 明确需要全局 skill 目录的兼容路径 | skills 和客户端支持的 workflow discovery | 否 |
| Bootstrap `partial` | 多客户端全局 workflow assets | skills 和客户端支持的 workflow discovery | 否 |
| Bootstrap `full` | 通过 release scripts 安装 runtime check 和 orchestrator | `partial` 加 shell CLI、MCP registration part 和 `doctor` 支持 | 是，Python 3.12+ |
| npm / npx | Node 自动化安装 | npm CLI 和内置 workflow payload | 只有高级 bridge 命令需要 |
| pipx / pip | Python updater CLI | Python CLI 分发 | 是 |

用户可见的 skill 名称是 `qiongli`。安装目录仍然是 `qiongli-workflow`，这是为了兼容已有客户端和 release artifacts。普通 CLI / local plugin 使用中，先安装稳定的 `core/complete` runtime，并把日常 subject 行为保存在 `.qiongli/guidance_manifest.yaml`。CLI/npm 专精安装默认使用 `coverage=complete`，但它们是 Desktop ZIP、focused package、release payload 和 install-surface testing 等 advanced compatibility 路径。

从 v1.9.0 开始，`qiongli install` 默认等价于 `--profile full --surface plugin`。Codex / Claude Code / Antigravity 会得到 CLI 管理的本地 plugin，并由 `qiongli mcp serve --transport stdio` 支撑；Antigravity 会按官方 plugin 结构在 plugin 根目录写入 `mcp_config.json`，Hermes 会写入受管理的 full MCP client config。只有明确想保留旧版 skills-only 布局时，才使用 `--surface skills --profile partial`。

## 原生 Plugin 和 Extension

只需要在一个客户端里使用 Qiongli 时，优先选这个入口。

Codex 通过统一的 [Skillsplace](https://github.com/jxpeng98/skillsplace) marketplace 安装：

```bash
codex plugin marketplace add jxpeng98/skillsplace --ref main
codex plugin marketplace list
```

然后在 Codex plugin UI 中安装或启用 `qiongli`，这是默认 core package。也可以选择 `qiongli-economics`、`qiongli-accounting`、`qiongli-business`、`qiongli-finance`、`qiongli-political-economy`、`qiongli-geoeconomics`、`qiongli-economics-accounting` 这类 subject entry，它们会安装对应的 `subject/complete` package。

Codex plugin 自带 `.mcp.json` 和 `mcp/qiongli-literature-provider/` 下的零依赖 Node literature-provider MCP runtime。只使用这些内置文献 provider 工具时，桌面用户不需要安装 `qiongli` CLI，也不需要手写 MCP config。Provider key 不写入 plugin manifest；可以通过平台无关的本地设置工具 `qiongli_configure_provider` 配置，也可以用 `qiongli_save_provider_config` 保存，或者在已安装 CLI 时用 `qiongli mcp configure` / `qiongli provider setup` 配置。完整本地 Qiongli 使用 CLI 生成的本地 plugin：`qiongli install --profile full --target codex --surface plugin`。这个本地 plugin 保留 Codex 原生 plugin 容器，但它的 `.mcp.json` 会启动完整 Python-backed `qiongli mcp serve --transport stdio` server。

Codex 目前会把 plugin-bundled MCP server 当作 plugin asset：设置页可以启用 server 和管理 tool policy，但不适合作为这个内置 server 的 provider key 注入入口。Claude Desktop MCPB、Claude Code、Cursor 类客户端和其他本地 stdio MCP client 也应使用同一个 Qiongli provider setup contract。请改用 Qiongli provider config：

1. 让 Codex 运行 `qiongli_config_status`，查看 redacted status 和 `config_path`。
2. 让当前客户端运行 `qiongli_configure_provider`，然后打开返回的 `127.0.0.1` URL。
3. 在本地浏览器页面里填写 OpenAlex API key、可选 OpenAlex email 和 Semantic Scholar API key。页面会写入共享 provider config，避免把密钥放进对话上下文。
4. 再运行 `qiongli_config_status` 或 `qiongli_literature_status`；结果只应显示 `configured` / `missing`，不应打印完整密钥。

不要把 provider key 写进 `.mcp.json`、`.codex-plugin/plugin.json`、release ZIP 或研究产物。Codex plugin MCP、Claude Code plugin MCP、Claude Desktop MCPB 和完整 CLI MCP server 读取的是同一个共享 provider config。

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

Claude Code marketplace plugin 也内置 `mcp/qiongli-literature-provider/` 下的零依赖 Node literature-provider MCP runtime，提供与 Codex plugin 相同的文献 provider、search 和 status 工具。只使用这些内置 literature/provider 工具时，不需要安装 `qiongli` CLI。完整 Python-backed full MCP 仍然是 CLI runtime：如果需要 `qiongli_literature_search`、`qiongli_task_plan`、`qiongli_task_run` 或 `qiongli_orchestrator_doctor` 等完整工具，需要 npm、pipx/pip 或 `full` bootstrap。运行 `qiongli install --profile full --target claude --surface plugin` 会生成本地 Claude Code plugin，并由这个 plugin 启动统一的 `qiongli mcp serve --transport stdio` server。`--target antigravity` 会生成带 root `mcp_config.json` 的 Antigravity plugin，`--target hermes` 写入 Hermes MCP config；`--target all --surface plugin` 会让 Codex / Claude Code / Antigravity 使用本地 plugin，同时给 Hermes 写入受管理的 full MCP client 配置。

Claude Desktop 和 Claude.ai 不安装第三方 Claude Code plugin marketplace。如果你使用 Desktop 或网页版，并且不熟悉 code / CLI 环境，优先使用 release ZIP 路径，不需要任何终端命令：

1. 从 GitHub Release assets 下载 `qiongli-claude-desktop-skill-core-<tag>.zip`、`qiongli-claude-desktop-skill-economics-<tag>.zip`、`qiongli-claude-desktop-skill-business-<tag>.zip`、`qiongli-claude-desktop-skill-finance-<tag>.zip`、`qiongli-claude-desktop-skill-political-economy-<tag>.zip`、`qiongli-claude-desktop-skill-geoeconomics-<tag>.zip` 或 `qiongli-claude-desktop-skill-economics-accounting-<tag>.zip`。本阶段公开 Desktop ZIP subjects 是 `core`、`economics`、`business`、`finance`、`political-economy`、`geoeconomics` 和 `economics-accounting`；还没有 standalone accounting Desktop ZIP。
2. 在 Claude Desktop 中，把 ZIP 拖拽到 Skills 上传/安装流程中；也可以打开 `Customize > Skills`，点击 `+`，选择 `Create skill`，再选择 `Upload a skill`。
3. 在 Claude.ai 网页版中，使用同样的 `Customize > Skills` 上传流程，选择同一个 ZIP。
4. 启用上传后的 `qiongli` skill。

Release ZIP 使用 `coverage=focused`，用于保持当前 180 文件上传预算。它是 subject 专精 Desktop/Web 包，不是降质删减版：保留可执行 workflows、prompts、templates、standards、所选 profiles、`skills-summary.md` 和 `skills-core.md`；专精 ZIP 还包含通过 layered overlays 生成的 selected effective skill markdown。这个 Desktop skill ZIP 是 skill-only asset：只包含 workflows/prompts/templates，不保存 secrets，也不执行 provider calls。完整 canonical source 可通过默认 `coverage=complete` 的 CLI/npm 安装、Codex / Claude Code plugin 包和源码仓库获得。

独立的 Qiongli Literature Provider `.mcpb`（`qiongli-literature-provider.mcpb`）才是 Claude Desktop 本地 provider asset。它在本地运行 Desktop literature search，支持 OpenAlex、Semantic Scholar、Crossref、PubMed 和 arXiv，并通过 Desktop 配置 UI 填写需要凭据的 provider；arXiv 默认可用，不需要凭据。敏感 key 交给 Claude Desktop sensitive-field handling，不写入 Desktop skill ZIP。这个 MCPB 自带零依赖 Node stdio server，所以 Desktop 用户不需要安装 `qiongli` CLI 或运行 npm install。CLI、Codex、Claude Code、Antigravity 和 Hermes 用户仍然可以运行 `qiongli provider setup`，再用 `qiongli provider doctor` 检查当前是 `provider_connected` 还是 `strategy_only`。Desktop 用户需要 `qiongli-literature-provider` MCPB 或平台原生搜索能力，才能声称 `provider_connected`；如果没有 MCPB 或平台原生搜索能力，就把运行记录为 `strategy_only`，并把平台搜索或用户提供的 corpus 作为证据来源。

## 安装后如何使用

安装或升级后，先重启目标客户端。然后使用该客户端暴露的入口：

| 客户端 | 发现方式 | 调用方式 |
|---|---|---|
| Codex | `/skills` 应该能列出 `qiongli` | `$qiongli <research task>` |
| Claude Code | Plugin UI、`/plugin` 或全局 command discovery | `/paper`、`/lit-review`、`/paper-write`、`/code-build` |
| Shell | `qiongli check` | `qiongli doctor`、`qiongli upgrade`、`python3 -m bridges.orchestrator ...` |

Codex 不暴露自定义 `/qiongli` slash command。先用 `/skills` 确认 skill 存在，再用 `$qiongli` 调用。

当客户端 UI 和文件系统状态看起来不一致时，先运行 `qiongli check --json`。它会为每个客户端报告当前安装 surface：`plugin`、`mcp`、`legacy_skill` 或 `none`。运行时/orchestrator 健康检查使用 `qiongli doctor --cwd .`；它也会追加一个非致命的 `Client Integration` 摘要。

如果 Codex 在 Personal plugins 里显示 `qiongli`，但打开详情时显示 `Plugin detail unavailable`，说明 Codex 找到了 marketplace entry，但无法读取本地 plugin payload。常见原因是 `.codex-plugin/plugin.json` 不符合 schema、`skills/qiongli-workflow/SKILL.md` frontmatter 不是合法 YAML，或 plugin 路径缺失。刷新 CLI 管理的本地 plugin：

```bash
qiongli install --target codex --surface plugin --overwrite
```

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

`full` 要求 Python 3.12+ 已经在 `PATH` 上。安装器不会安装 Python 或 `mise`。Codex 的完整本地安装入口也可以直接使用：

```bash
qiongli install --profile full --target codex --surface plugin
qiongli mcp doctor --json
```

安装后检查工作区：

```bash
qiongli doctor --cwd .
python3 -m bridges.orchestrator doctor --cwd .
qiongli mcp doctor --json
```

## npm / npx

如果你想用 Node 分发的安装器，并让 workflow payload 跟随 npm 包：

```bash
npm install -g qiongli
qiongli install --target all
qiongli project init --project-dir .
qiongli project set-subject finance --project-dir .
qiongli project status --project-dir .
qiongli install --profile full --target all --surface plugin
```

脚本兼容写法 `qiongli install --target all --project-dir "$PWD"` 对安装面等价；普通项目 subject 行为仍然交给 `qiongli project ...`。

日常 CLI / local plugin 使用中，保持这个 installed runtime 稳定，把 subject 行为保存在 `.qiongli/guidance_manifest.yaml`。如果 manifest 不存在，Qiongli 使用隐式 `active_subject: auto`：先使用 core guidance，再根据当前任务临时推断 subject 或 method lens，并在改变项目本地状态前写出可审计 proposal。

更新全局 package 并刷新 full local plugin/MCP 安装面：

```bash
qiongli self-update --dry-run
qiongli self-update --yes
```

在 npm 安装中，`self-update` 会委托给 `npm install -g qiongli@latest`，然后用刷新后的 package 执行 `qiongli install --target all --surface plugin --profile full --overwrite`。原生 marketplace plugin 仍由客户端 plugin manager 管理。

一次性运行：

```bash
npx qiongli@latest install --target all
npx qiongli@latest project status --project-dir .
npx qiongli@latest check --json
```

测试 prerelease 时仍可使用 `next` dist-tag：

```bash
npx qiongli@next install --target all
```

Advanced compatibility、Desktop ZIP、focused package、release payload 和 install-surface testing 示例：

```bash
qiongli install --subject core --target all --project-dir "$PWD"
qiongli install --subject economics --target all --project-dir "$PWD"
qiongli install --subject accounting --target all --project-dir "$PWD"
qiongli install --subject economics-accounting --target all --project-dir "$PWD"
qiongli install --subject economics --coverage focused --target all --project-dir "$PWD"
npx qiongli@latest install --subject economics --target all --project-dir "$PWD"
npx qiongli@next install --subject economics --target all --project-dir "$PWD"
```

如果要完全切换到 marketplace plugin，先移除 CLI 安装产生的资产：

```bash
qiongli remove --target all --dry-run
qiongli remove --target all
```

`qiongli remove` 默认只移除 CLI 安装的全局 workflow assets 和 discovery links。原生 marketplace plugin 仍由安装它的客户端/plugin manager 管理。

对于本地 full plugin surface，只移除 CLI 管理的 plugin root：

```bash
qiongli remove --parts plugin --target codex
qiongli remove --surface plugin --target claude
```

安装器会用 `.qiongli-managed.json` 标记自己创建的 plugin root，并给 Codex marketplace entry 写入 `metadata.managedBy = "qiongli-cli"`。`remove` 会保留非受管 plugin 目录和 marketplace lite plugin。

## 推荐的 CLI Setup Wizard

通过 npm、pipx、pip 或 bootstrap script 安装 CLI 后，先运行交互式 setup wizard，再手写安装参数：

```bash
qiongli setup
qiongli setup --dry-run
qiongli setup --project-dir "$PWD" --no-doctor
qiongli setup --provider-mode prompt --no-browser
```

wizard 会引导 CLI、Codex、Claude Code 和 Antigravity 用户完成：

- setup path：`install` 用于首次安装内置 assets，`upgrade` 用于从上游刷新
- runtime surface：CLI、Codex、Claude Code、Antigravity 或 multi-platform
- subject 选择
- coverage 选择：`complete` 或 `focused`
- install mode：普通用户使用 `--mode copy`，本地 checkout 开发使用 `--mode link`
- install scope：`all`、`globals`、`project` 或 `cli`
- 启用 CLI wrapper 时的 CLI 目录
- overwrite 策略：需要替换已管理安装时使用 `--overwrite`；升级但保留现有 managed files 时使用 `--no-overwrite`
- upgrade source：latest stable、latest beta、显式 `--ref` tag、显式 `--ref-type branch`，以及可选 `--repo`
- 可选 literature provider setup。默认会打开一个本地浏览器页面，一次配置 OpenAlex、Semantic Scholar、Crossref、PubMed，并展示 arXiv 不需要 API key；远程终端可用 `--no-browser` 只打印 URL，也可以用 `--provider-mode prompt` 回到终端逐项录入，或用 `--provider-mode skip` 跳过 provider setup
- doctor verification，除非设置 `--no-doctor`

每一步 prompt 都会打印简短的 `Tip:` 注释，解释这个选择会改变什么；不熟悉完整 CLI 参数的用户也可以按引导完成安装或升级。

通过 setup 保存的 provider 密钥使用与 `qiongli provider setup` 和 `qiongli provider doctor` 相同的 provider 配置。本地页面包含各 provider 获取 key 的链接和步骤；arXiv 标注为无需 API key。密钥会保存在生成的研究 artifacts 之外。provider 步骤用于配置凭据并执行 doctor/capability 检查；它不保证一定产生外部检索结果。

在 npm 安装中，`qiongli setup` 会委托到 npm 包内置的 Python bridge，因此要求本机已有 Python 3.12+ 和 `PyYAML`。如果只需要 Node-only asset installer，继续使用显式 `qiongli install ...` 命令。

## pipx / pip

如果你明确需要 Python 分发的 updater CLI，用 pipx：

```bash
pipx install qiongli
qiongli setup
qiongli install --target all
qiongli project init --project-dir .
qiongli project set-subject finance --project-dir .
qiongli project status --project-dir .
```

`qiongli setup` 可以交互式引导安装。脚本化安装使用 `qiongli install --target all`；项目 subject 行为用 `qiongli project set-subject` 修改。

升级：

```bash
qiongli self-update --dry-run
qiongli self-update --yes
```

手动 package-manager 更新仍然可用：运行 `pipx upgrade qiongli` 或 `python -m pip install --upgrade qiongli`，然后用 `qiongli install --target all --surface plugin --profile full --overwrite` 刷新客户端安装面。`qiongli upgrade` 仍是你明确想下载上游 GitHub release 时使用的 release archive refresh 路径。

`--subject` 默认是 `core`，`--coverage` 默认是 `complete`，但 subject 安装参数是 advanced compatibility 控制项。它们用于有意选择的精简安装、Desktop/Web 等价包、release payload validation 和 install-surface testing。当前官方 subjects 是 `core`、`economics`、`accounting`、`business`、`finance`、`political-economy`、`geoeconomics` 和命名 composite subject `economics-accounting`；`political-economy` 和 `geoeconomics` 是两个独立 subject 选择，不是一个 composite。官方 composite subjects 不是任意逗号分隔叠加。普通项目 subject 行为用 `qiongli project set-subject` 修改；installed subject 或 coverage 只在有意刷新专精安装包时改变。`qiongli check --json` 会输出每个 target 当前安装的 subject 和 coverage；旧安装缺少 `SUBJECT_MANIFEST.json` 或 `SUBJECT` 文件时按 legacy `core` / `complete` 处理。

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
qiongli self-update --dry-run
qiongli self-update --yes
```

如果你已经完全转向原生 plugin，不再需要旧的全局 skill 目录或 slash discovery，先 dry-run 清理：

```bash
qiongli remove --target all --dry-run
```
