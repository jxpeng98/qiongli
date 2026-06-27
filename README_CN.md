<div align="center">
  <h1>穷理（Qiongli）</h1>
  <p><strong>让 AI 辅助研究留下可复查证据链的学术工作流系统。</strong></p>
  <p>把宽泛研究请求拆成 Task ID、质量门、证据链、角色交接和 <code>RESEARCH/[topic]/</code> 下的稳定产物。</p>
  <p>
    <a href="https://www.npmjs.com/package/qiongli"><img alt="npm latest version" src="https://img.shields.io/npm/v/qiongli/latest?style=flat-square&amp;logo=npm&amp;label=npm%20latest"></a>
    <a href="https://www.npmjs.com/package/qiongli?activeTab=versions"><img alt="npm next version" src="https://img.shields.io/npm/v/qiongli/next?style=flat-square&amp;logo=npm&amp;label=npm%20next&amp;color=cb3837"></a>
    <a href="https://pypi.org/project/qiongli/"><img alt="PyPI latest version" src="https://img.shields.io/pypi/v/qiongli?style=flat-square&amp;logo=pypi&amp;label=PyPI%20latest"></a>
  </p>
  <p>
    <a href="README.md">English README</a> ·
    <a href="docs/zh/index.md">中文文档</a> ·
    <a href="docs/index.md">Docs</a> ·
    <a href="docs/zh/quickstart.md">快速开始</a> ·
    <a href="docs/zh/guide/install.md">安装</a> ·
    <a href="docs/zh/reference/cli.md">CLI</a>
  </p>
</div>

## 它是什么

穷理是一套便携的学术 workflow package，也可以按需接入本地运行时。它适合不能只靠一次 prompt 完成、后续还需要复查证据和过程的研究任务。

你可以用它来：

- 为 empirical、qualitative、systematic review、RCT、theory、code-first methods 等任务选择论文路线；
- 把 literature search、citation risk、methods、writing 和 review 绑定到明确证据；
- 用 solo、duo、triad 模式组织多 agent 协作，并保留 handoff 与 verification status；
- 把轻量 skill/plugin 使用和完整本地 orchestrator 运行分开管理。

“穷理”表示持续追问一个 claim 背后的 principle、evidence 和 limit。

## 从哪里开始

| 目标 | 推荐入口 |
|---|---|
| 浏览完整文档站 | [中文文档](docs/zh/index.md)，或本地运行 `npm run docs:dev` |
| 阅读英文说明 | [English README](README.md) 或 [Docs](docs/index.md) |
| 先在一个客户端里安装 | [安装指南](docs/zh/guide/install.md) |
| 从零跑到第一个 workspace | [快速开始](docs/zh/quickstart.md) |
| 选择论文 workflow | [任务场景](docs/zh/guide/task-recipes.md) |
| 使用 CLI、别名、JSON check 或自动化 | [CLI 参考](docs/zh/reference/cli.md) |
| 理解运行时和 package 模型 | [系统架构](docs/zh/architecture.md) |

## 最新稳定版下载

当前稳定版是 [v1.11.0](https://github.com/jxpeng98/qiongli/releases/tag/v1.11.0)。下面这些直达链接覆盖常见安装路径；需要 subject 专精 Desktop ZIP 或维护者 artifacts 时，再打开下载指南。

| 需求 | 链接或命令 |
|---|---|
| npm CLI | [`qiongli@1.11.0`](https://www.npmjs.com/package/qiongli/v/1.11.0)：`npm install -g qiongli@latest` |
| PyPI CLI | [`qiongli 1.11.0`](https://pypi.org/project/qiongli/1.11.0/)：`pipx install qiongli` |
| Claude Desktop/Web core skill | [`qiongli-claude-desktop-skill-core-v1.11.0.zip`](https://github.com/jxpeng98/qiongli/releases/download/v1.11.0/qiongli-claude-desktop-skill-core-v1.11.0.zip) |
| Claude Desktop literature MCPB | [`qiongli-literature-provider-0.1.5.mcpb`](https://github.com/jxpeng98/qiongli/releases/download/v1.11.0/qiongli-literature-provider-0.1.5.mcpb) |
| Zotero Desktop companion | [`qiongli-zotero-companion-0.2.2.xpi`](https://github.com/jxpeng98/qiongli/releases/download/v1.11.0/qiongli-zotero-companion-0.2.2.xpi) |
| 全部 release assets | [下载指南](https://github.com/jxpeng98/qiongli/releases/download/v1.11.0/qiongli-downloads-v1.11.0.md) 和 [GitHub Release](https://github.com/jxpeng98/qiongli/releases/tag/v1.11.0) |

## 快速安装

默认 CLI 安装会准备支持客户端的 full local plugin surface：

```bash
npm install -g qiongli
qiongli install --target all
qiongli check --offline
```

脚本化安装时建议显式传入项目目录：

```bash
qiongli install --target all --project-dir "$PWD"
```

日常切换项目领域时，不需要反复重装 package；用项目级 subject guidance：

```bash
qiongli project init --project-dir "$PWD"
qiongli project set-subject finance --project-dir "$PWD"
qiongli project status --project-dir "$PWD"
```

如果只需要 skill-only 或无 Python 安装路径，请看安装指南。里面分别说明 Codex / Claude Code marketplace plugin、Claude Desktop Skill ZIP、literature MCPB、bootstrap partial/full、npm/npx、pipx 和 pip。

## 推荐的 CLI Setup Wizard

当你希望 CLI 帮你选择安装和升级路径时，使用 setup wizard：

```bash
qiongli setup
qiongli setup --dry-run
qiongli setup --project-dir "$PWD" --no-doctor
```

它会覆盖 runtime surface、subject、coverage、`--mode copy|link`、shell CLI / CLI 目录、`--overwrite` / `--no-overwrite`、可选 provider config，以及 doctor 验证。npm 安装下，`qiongli setup` 会通过内置 Python bridge 委托执行，因此需要 Python 3.12+ 和 `PyYAML`。如果只需要脚本化安装 assets，直接运行 `qiongli install ...` 即可。

## 更新还是刷新

`qiongli update` 会先更新已安装的 CLI package，然后询问是否用新 package 的 payload 刷新本地 plugins/assets：

```bash
qiongli update
qiongli update --yes
qiongli update --no-refresh
```

`qiongli upgrade` 是另一件事：它只刷新本地 content/assets，可以来自当前 package，也可以来自指定 release archive。它不会升级 npm、pipx 或 pip 中安装的 qiongli package。

```bash
qiongli upgrade --ref v1.11.0 --target all
```

## 运行时边界

安装 Qiongli assets 比运行完整 orchestrator 轻得多。

| Surface | 用途 | 是否需要 Python / 模型 CLI |
|---|---|---|
| Skill 或 plugin package | prompts、task routes、templates、standards、subject overlays | 否 |
| Literature MCPB / bundled literature MCP | provider status、本地检索、evidence export | 不需要 Python |
| Full local plugin 或 CLI MCP | `doctor`、provider config、`task-plan`、`task-run`、orchestrator tools | 需要 |
| Shell/Python CLI | validators、release checks、本地编排、package 维护 | 需要 |

只有在运行时配置完成并且执行命令明确启用时，才会启动真实 agent execution。Preview 和 check 设计上都应先可检查，再产生副作用。

## 文档地图

- [入门](docs/zh/guide/index.md)：安装、使用、升级、故障排除和 runtime 选择。
- [快速开始](docs/zh/quickstart.md)：最小安装面和第一个研究路线。
- [使用 Agent Skills](docs/zh/guide/using-agent-skills.md)：Codex、Claude Code、Antigravity、Hermes 和 shell 里该输入什么。
- [任务场景](docs/zh/guide/task-recipes.md)：按真实研究目标选择 paper route。
- [参考](docs/zh/reference/index.md)：CLI 行为和 skill catalog。
- [高级](docs/zh/advanced/index.md)：MCP providers、Zotero、subject packaging、plugin-first distribution。
- [维护者](docs/zh/maintainer/index.md)：release policy、naming policy 和贡献者指南。

## 开发

常用检查：

```bash
python3 -m unittest tests.test_self_update tests.test_cli tests.test_cli_setup_docs
python3 -m unittest tests.test_materialize_distribution_payloads tests.test_npm_package_contract
npm --prefix packages/npm-qiongli test
npm run docs:build
git diff --check
```

常规发版入口：

```bash
./scripts/release_automation.sh publish --version <version> --from-tag <previous-tag>
```

## 致谢

Qiongli 借鉴了严格 agent planning/review、Claude skill packaging 和学术审阅实践中的有效做法，并把它们收敛到可复查的研究产物链路里。感谢 [linux.do](https://linux.do/) 社区提供务实的中文 AI tooling 讨论和反馈。
