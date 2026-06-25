# CLI 命令参考（qiongli）

本文件整理本仓库所有“可执行入口”（pipx CLI / Python module / Bash scripts），用于本地与 GitHub CI 保持一致的调用方式。

## 0) 命令名约定

- `qiongli`：主 CLI（pipx/venv 安装后可用，或通过 shell bootstrap 安装）
- `ql`：短主别名。`research-skills`、`rsk`、`rsw`：旧兼容别名，与 `qiongli` 等价

下文统一用 `qiongli` 作为示例。

---

## 1) Upstream（上游仓库）如何确定（如何省略 `--repo`）

很多命令需要知道“去哪个 GitHub 仓库查询/下载 release”。`qiongli` 的上游解析优先级如下（从高到低）：

1. CLI 参数：`--repo <owner/repo|Git URL>`
2. 环境变量：`QIONGLI_REPO=<owner/repo|Git URL>`
3. 旧环境变量 fallback：`RESEARCH_SKILLS_REPO=<owner/repo|Git URL>`
4. 项目配置文件（从当前目录或 `--project-dir` 向上搜索）：
   - `qiongli.toml`
   - `.qiongli.toml`
5. 打包默认（pipx 安装的包内）：`qiongli/project.toml`（由 CI 注入）
6. 如果你正在 `qiongli` 仓库 clone 内运行：从 git remote 推断（优先 `upstream`，其次 `origin`）

支持的 repo 形式：

- `owner/repo`
- `https://github.com/owner/repo.git`
- `git@github.com:owner/repo.git`

推荐把上游提交到你的项目仓库（适合 CI）：

```toml
# qiongli.toml
[upstream]
repo = "owner/repo"   # 或 url = "https://github.com/owner/repo.git"
```

---

## 2) `qiongli`（安装/升级器 CLI）

这个 CLI 现在有两种分发方式：
- Python CLI：通过 `pip`/`pipx` 安装
- Shell CLI：由 `bootstrap_qiongli.sh` 默认安装到 `${QIONGLI_BIN_DIR:-${RESEARCH_SKILLS_BIN_DIR:-~/.local/bin}}`

### 2.0 默认安装模型

当前 CLI 默认是 plugin-first。最短命令：

```bash
qiongli install
qiongli upgrade
```

会展开为完整本地 plugin surface：

| 接口 | 默认值 |
|---|---|
| Target | `--target all` |
| Subject package | `--subject core --coverage complete` |
| Runtime profile | `--profile full`；如果显式写了 `--surface skills` 且没有指定 profile，则 CLI 使用 `partial` |
| Output surface | `install` 和 `upgrade` 默认 `--surface plugin` |
| Install mode | `--mode copy` |
| Project directory | 当前工作目录；只有请求 project-facing parts 时才会写项目文件 |
| Shell CLI wrapper | effective `full` profile 会启用；用 `--no-cli` 跳过 wrapper 刷新 |
| MCP registration | effective `full` profile 会启用；Codex/Claude 的 plugin-owned MCP entry 会跳过，因为本地 plugin 自己持有 MCP 启动配置 |
| Doctor | 直接运行 `install` / `upgrade` 时默认不跑；显式传 `--doctor` 或在 `--parts` 里包含 `doctor` 才运行 |
| Overwrite | `install` 默认不覆盖；`upgrade` 默认覆盖，并支持 `--no-overwrite` |

`--surface` 是高层输出形态选择：

| Surface | 安装内容 |
|---|---|
| `plugin` | Codex / Claude Code 的 CLI-managed local plugin；target 包含 Antigravity/Hermes 时写入受管理 MCP config |
| `skills` | 旧版全局 `qiongli-workflow` skill 目录，以及客户端支持的 workflow discovery |
| `both` | 同时安装旧版全局 skills 和本地 plugin surface |

`--parts` 是精确覆盖。设置后，它会替代由 surface/profile 推导出的安装集合，只执行你列出的逗号分隔 parts：`globals`、`plugin`、`project`、`cli`、`mcp`、`doctor`。`all` 和 `*` 会展开成全部 parts。自动化脚本如果只想触碰一个面，应该用 `--parts mcp` 或 `--parts plugin,cli` 这类写法。

### 2.1 `qiongli check`（检查版本/是否有更新）

用途：
- 输出 CLI 版本、本地 repo 版本（若在仓库内运行）、受支持客户端的已安装版本
- 可选：查询上游最新 release tag，并判断是否需要升级

```bash
qiongli check [--repo <owner/repo|url>] [--json] [--strict-network]
```

关键参数：
- `--repo`：指定上游（可省略，见“上游解析”）
- `--json`：只输出 JSON（便于 CI/脚本）
- `--strict-network`：如果上游查询失败则返回失败（默认仅提示并继续）

JSON 输出会包含每个 target 当前安装的 active subject 和 coverage。旧 managed install 如果没有 `SUBJECT_MANIFEST.json` 或 `SUBJECT` marker，会按 legacy `core` / `complete` 处理。

退出码约定：
- `0`：无更新/或跳过上游检查
- `1`：检测到更新可用
- `2`：参数错误

### 2.2 `qiongli setup`（交互式 CLI setup wizard）

用途：
- npm、pipx、pip 或 bootstrap 安装 CLI 后的推荐第一个命令。
- 引导 CLI、Codex、Claude Code、Antigravity 和 Hermes 用户选择 install/upgrade、runtime surface、subject、coverage、install mode、install scope、overwrite 策略、upgrade source、可选 provider key setup，并执行 doctor verification。

```bash
qiongli setup [--project-dir <path>] [--dry-run] [--no-doctor]
```

示例：

```bash
qiongli setup
qiongli setup --dry-run
qiongli setup --project-dir "$PWD" --no-doctor
```

通过 npm launcher 调用时，`qiongli setup` 使用 npm 包内置的 Python bridge，因此要求 Python 3.12+ 和 `PyYAML`。显式 `qiongli install ...` npm 命令仍可用于 Node-only asset installation。

wizard 选项：
- Setup path：`install` 或 `upgrade`。
- Runtime surface：`cli`、`codex`、`claude-code`、`antigravity`、`hermes` 或 `multi-platform`。
- Subject：`core`、`economics`、`accounting`、`business`、`finance`、`political-economy`、`geoeconomics` 或 `economics-accounting`。
- Coverage：`complete` 或 `focused`。
- Install mode：普通用户用 `--mode copy`，本地开发 checkout 用 `--mode link`。
- Install scope：`all`、`globals`、`project` 或 `cli`。
- 所选 scope 包含 CLI wrapper 时的 CLI 目录。
- Overwrite 策略：install refresh 使用 `--overwrite`；升级但不替换 managed files 时使用 `--no-overwrite`。
- Upgrade source：latest stable、latest beta、可选 `--repo`、显式 `--ref`，以及 `--ref-type tag|branch`。
- 可选 literature provider credentials。
- Doctor verification，除非设置 `--no-doctor`。

每一步 prompt 都包含简短的 `Tip:` 注释，解释这个选择为什么重要，以及会改变哪种安装或升级行为。

通过 setup 输入的 provider 密钥使用与 `qiongli provider setup` 和 `qiongli provider doctor` 相同的 provider 配置。密钥保存在生成的研究 artifacts 之外。setup 会配置凭据并执行 doctor/capability 检查；它不承诺一定会运行外部 literature search。

### 2.2.1 `qiongli mcp`（跨平台 MCP server）

用途：
- 为支持 MCP 的桌面或 agent 客户端启动本地 Qiongli MCP server。
- 让 CLI 用户和 Desktop-only 用户配置同一套 provider key。
- 生成不包含 secret 的客户端配置示例。

```bash
qiongli mcp serve --transport stdio
qiongli mcp serve --transport http --host 127.0.0.1 --port 8765
qiongli mcp configure --provider openalex --field email --value you@example.com
qiongli mcp doctor --json
qiongli mcp config example --target codex --json
qiongli mcp config example --target claude-code --json
qiongli mcp config example --target antigravity --json
qiongli mcp config example --target hermes --json
qiongli mcp wizard
```

server 暴露的 MCP tools 包括 `qiongli_config_status`、`qiongli_save_provider_config`、`qiongli_collect_evidence`、`qiongli_list_provider_env`、`qiongli_test_provider`、`qiongli_configure_provider`、`qiongli_orchestrator_route`、`qiongli_orchestrator_doctor`、`qiongli_task_plan` 和 `qiongli_task_run`。

默认 `stdio` 模式是本地进程，不需要远端 server。Codex、Claude Code、Antigravity、Hermes 或其他本地 MCP client 可以先调用 `qiongli_orchestrator_route`，决定是否从 skill-only routing 升级到 full orchestrator tools。`qiongli_task_run` 默认是 preview mode；只有 MCP caller 显式传入 JSON boolean `run_agents: true` 时，才会启动本地模型 CLI。

### 2.3 `qiongli install`（安装包内 subject payload）

用途：
- 把 PyPI/npm/source checkout 内携带的 subject payload 安装成当前本地 Qiongli surface。
- 默认是完整 plugin surface：Codex / Claude Code 本地 plugin、Antigravity / Hermes 受管理 MCP config，并刷新 shell CLI wrapper，除非显式传 `--no-cli`。
- 默认不会迁移或删除旧版 global skills；自动迁移走 `qiongli upgrade`，显式清理走 `qiongli remove`。

```bash
qiongli install \
  [--profile partial|full] \
  [--subject core|economics|accounting|business|finance|political-economy|geoeconomics|economics-accounting] \
  [--coverage complete|focused] \
  [--target codex|claude|antigravity|hermes|all] \
  [--surface skills|plugin|both] \
  [--mode copy|link] \
  [--project-dir <path>] \
  [--install-cli | --no-cli] \
  [--cli-dir <path>] \
  [--overwrite] \
  [--doctor] \
  [--parts globals,plugin,project,cli,mcp,doctor] \
  [--dry-run]
```

示例：

```bash
qiongli install --target all
qiongli install --subject economics --target all
qiongli install --subject accounting --target all
qiongli install --subject political-economy --target all
qiongli install --subject geoeconomics --target all
qiongli install --subject economics-accounting --target all
qiongli install --subject economics --coverage focused --target all
qiongli install --surface skills --profile partial --target all
qiongli install --profile full --target codex --surface plugin
qiongli install --profile full --target all --surface plugin
qiongli install --profile full --target all --surface both
qiongli install --parts mcp --target hermes
```

Subject package 是专精安装包，不是降质删减版。默认安装是 `core/complete`。`--subject economics`、`--subject business`、`--subject finance`、`--subject political-economy` 和 `--subject geoeconomics` 表示 complete 专精安装，不是缩水包；`--subject accounting` 表示 `accounting/complete`，即全量框架加 accounting 专精。`focused` coverage 只选择该 subject 的 profiles 和 active effective skills，用于有意选择的精简安装和 Desktop/Web ZIP。当前官方 subjects 是 `core`、`economics`、`accounting`、`business`、`finance`、`political-economy`、`geoeconomics` 和命名 composite subject `economics-accounting`；`political-economy` 和 `geoeconomics` 是两个独立 subject 选择，不是一个 composite。官方 composite subjects 不是任意逗号分隔叠加。本阶段公开 Desktop ZIP subjects 是 `core`、`economics`、`business`、`finance`、`political-economy`、`geoeconomics` 和 `economics-accounting`，还没有 standalone accounting Desktop ZIP。切换 subject 或 coverage 时，重新运行 `install` 或 `upgrade` 并指定新参数。

从 v1.9.0 开始，`qiongli install` 默认等价于 `--profile full --surface plugin`。Codex 会写入 personal marketplace entry 和启动 `qiongli mcp serve --transport stdio` 的 plugin `.mcp.json`；Claude Code 会写入本地 plugin manifest 并启动同一个 full MCP server。`--target all` 会让 Codex/Claude Code 使用本地 plugin，同时给 Antigravity/Hermes 写入受管理的 full MCP client 配置。Marketplace 安装的 plugin 仍然保持 lite/no-Python 路径，使用内置 Node literature provider。需要旧版 skills-only 布局时，使用 `--surface skills --profile partial`。

安装行为细节：
- `--surface plugin --target all` 会给 Codex / Claude Code 安装 plugin-owned MCP，并给 Antigravity / Hermes 写入 client-level MCP config。
- `--surface skills --profile partial` 只安装旧版 skill 目录和 workflow discovery；除非使用 `--parts cli` 或 `--parts mcp`，否则不会安装 shell wrapper 或 MCP config。
- `--surface both` 会保留旧版 global skills，同时安装本地 plugin；只有明确需要两个发现路径同时存在时才使用。
- `--parts` 优先于 `--surface`。例如 `--parts mcp --target antigravity` 只写 Antigravity MCP config，`--parts project` 只写项目侧文件。
- 直接运行 `qiongli install` 时，`--doctor` 必须显式传入。交互式 setup wizard 可能默认建议 doctor verification，但 `install` 命令本身不会自动跑 doctor。

### 2.4 `qiongli upgrade`（下载 release 并运行安装器）

用途：
- 下载上游 release（默认 latest tag 的 tar.gz）
- 解压后运行包内 Python installer
- 默认使用完整本地 plugin surface，并在新安装成功后迁移清理旧 global skills 和 Codex/Claude 独立 MCP config

```bash
qiongli upgrade \
  [--repo <owner/repo|url>] \
  [--ref <tag-or-branch>] \
  [--ref-type tag|branch] \
  [--profile partial|full] \
  [--subject core|economics|accounting|business|finance|political-economy|geoeconomics|economics-accounting] \
  [--coverage complete|focused] \
  [--target codex|claude|antigravity|hermes|all] \
  [--surface skills|plugin|both] \
  [--project-dir <path>] \
  [--install-cli | --no-cli] \
  [--cli-dir <path>] \
  [--overwrite | --no-overwrite] \
  [--doctor] \
  [--parts globals,plugin,project,cli,mcp,doctor] \
  [--dry-run]
```

说明：
- `--project-dir` 主要在你显式请求项目侧安装面时生效，例如 `--parts project`。
- 现在默认的 `upgrade` 等价于 `--profile full --surface plugin`：刷新 Codex / Claude Code 本地 plugin，给 Antigravity / Hermes 写入受管理 MCP config，并在安装成功后清理旧 global skills 和 Codex/Claude 独立 MCP config。
- 如果要保留旧版 skills-only 升级路径，使用 `qiongli upgrade --surface skills --profile partial ...`。
- 如果明确想让旧版 global skills 和 plugin surface 并存，使用 `qiongli upgrade --surface both ...`；这个路径不会执行 plugin migration cleanup。
- migration cleanup 只会在 effective `--surface plugin` 升级成功后运行，并且 selected parts 为空或包含 `plugin`。安装失败时绝不会删除旧资产。
- migration cleanup 会删除旧版全局 `qiongli-workflow` skill 目录、Claude Code workflow discovery links，以及 Codex/Claude 独立 MCP config。Antigravity/Hermes MCP config 会保留，因为 plugin-first 架构下它们仍然通过受管理 MCP config 接入。
- 项目接线建议走 `qiongli init --project-dir .`；如果确实要在升级时重写项目文件，再显式加 `--parts project`。
- `--subject` 默认是 `core`，`--coverage` 默认是 `complete`；使用 `--subject economics` 会安装全量 Qiongli 加 economics 专精，使用 `--subject accounting` 会安装全量 Qiongli 加 accounting 专精，显式加 `--coverage focused` 时才安装精简 selected 包。
- 示例：`qiongli upgrade --subject accounting --target all`。
- 示例：`qiongli upgrade --target all` 会刷新本地 full plugin surface，不会切换到 marketplace lite plugin。
- 只有 legacy skills-only 升级路径会创建 Claude Code 工作流发现 symlink：`~/.claude/commands/*.md`，可直接使用 `/paper`、`/lit-review` 等 slash 命令。
- Shell CLI 会通过随附的 bootstrap helper 执行升级。完整 plugin/MCP 路径要求可用的 Python-backed `qiongli` runtime，因为 plugin 会启动 `qiongli mcp serve --transport stdio`。
- 退出码为底层安装器返回码（若安装失败，沿用其错误码）。

### 2.5 `qiongli align`（快速参考）

用途：打印“pipx 安装了什么 / upgrade 会修改哪些路径 / 常见用法”。

```bash
qiongli align [--repo <owner/repo|url>]
```

### 2.6 `qiongli init`（项目初始化）

用途：在项目目录中创建 `.env` 等项目配置。

```bash
qiongli init \
  [--project-dir <path>] \
  [--target all|codex|claude|antigravity|hermes] \
  [--mode copy|link] \
  [--overwrite] \
  [--doctor] \
  [--parts project,doctor] \
  [--dry-run]
```

说明：
- 默认等价于 `--parts project`，只创建 project-facing assets（`.env`）。除非显式传 parts，否则不会触碰 global skill 目录、本地 plugin 或 MCP config。
- 可重复运行；除非显式传 `--overwrite`，否则不会覆盖已有文件。

### 2.7 `qiongli remove`（移除 CLI 安装的资产）

用途：移除 CLI 安装产生的资产，方便在 npm/PyPI/bootstrap 安装和原生 marketplace plugin 之间切换。

```bash
qiongli remove \
  [--target codex|claude|antigravity|hermes|all] \
  [--surface skills|plugin|both] \
  [--parts globals|project|cli|mcp|plugin] \
  [--project-dir <path>] \
  [--cli-dir <path>] \
  [--dry-run]
```

示例：

```bash
qiongli remove --target all --dry-run
qiongli remove --target codex
qiongli remove --target codex --surface plugin
qiongli remove --parts plugin --target codex
qiongli remove --parts globals,project --project-dir "$PWD"
qiongli remove --parts cli --cli-dir ~/.local/bin
qiongli uninstall --target all
qiongli delete --target claude
```

说明：
- `remove` 默认等价于 `--parts globals`，会移除 CLI 安装的 `qiongli-workflow` skill 目录和生成的 workflow discovery links。
- 如果某个 `qiongli-workflow` 目录不像 Qiongli package payload，会跳过，避免删除用户自建内容。
- Plugin removal 只删除带 `.qiongli-managed.json` 的 CLI-managed 本地 full plugin root，以及带 `metadata.managedBy = "qiongli-cli"` 的 Codex marketplace entry。
- `--surface plugin` 只删除 CLI-managed local plugin surface，不会删除 MCP client config；需要删 MCP config 时使用 `--parts mcp`。
- `--surface both` 会删除旧版 global skills 和 CLI-managed local plugin，但仍不会删除 MCP config，除非同时包含 `--parts mcp`。
- 它不会卸载 `qiongli` 或 `qiongli-next` 这类 marketplace plugin；这些需要在 Codex、Claude Code 或 Claude Desktop 的 plugin manager 中移除。
- 需要同时清理旧项目本地文件时，使用 `--parts project`。
- 只有通过 full CLI/bootstrap 安装过 shell wrapper 时，才需要使用 `--parts cli`。

### 2.8 `qiongli clean`（清理过期资产）

用途：移除旧版本安装留下的项目本地资产。

```bash
qiongli clean [--project-dir <path>] [--dry-run] [--globals]
```

参数说明：
- `--project-dir`：要清理的目录（默认当前目录）。
- `--globals`：同时移除全局工作流发现 symlink（例如 `~/.claude/commands/`）。也会清理旧版本遗留的 Gemini workflow symlink；只移除指向 `qiongli-workflow` 的 symlink，用户自建的命令不受影响。
- `--dry-run`：只显示将要移除的内容，不实际删除。

### 2.9 `qiongli doctor`（环境预检）

```bash
qiongli doctor [--cwd <path>]
```

### 2.10 `qiongli customize`（创建 custom subject overlay）

用途：
- 为 Python/source checkout materialization 工作流创建本地 custom overlay scaffold。
- Custom overlays 只影响 generated output，不会改写 canonical source files。
- npm runtime installs 在这个阶段使用预生成 payloads，不会在 install 时 materialize `--custom-dir` overlays。

```bash
qiongli customize --subject economics --name my-econ-lab --out ./qiongli-custom/econ-lab
python3 scripts/materialize_subject_package.py --subject economics --custom-dir ./qiongli-custom/econ-lab --source . --out /tmp/qiongli-workflow
```

开发或加深一个 subject 时，需要同步更新 `subjects/catalog.yaml`、subject overlays、subject-specific registry and markdown、选定的 domain and venue profiles、subject eval fixtures、specialization audit expected terms、materializer tests、基于 staged materialization 的 npm package contract tests（当该 subject 可通过 npm 安装时），以及该 subject 有 Desktop/Web artifact 时的 release validation。

---

## 3) 编排器 CLI：`python3 -m bridges.orchestrator`

这是“三端并发/降级 + task-run 标准合同落盘”的执行入口。

```bash
python3 -m bridges.orchestrator <mode> [args...]
```

mode 列表：

- `doctor`：环境预检
  ```bash
  python3 -m bridges.orchestrator doctor --cwd .
  ```
- `parallel`：三端并发分析 + 总结端综合（自动降级为双端/单端）
  ```bash
  python3 -m bridges.orchestrator parallel \
    --prompt "Analyze this study design" \
    --cwd . \
    --summarizer claude \
    --profile-file standards/agent-profiles.example.json \
    --profile default
  ```
- `task-run`：按 Task ID 跑标准链（plan -> evidence -> draft -> review -> gates -> 写入 RESEARCH/）
  ```bash
  python3 -m bridges.orchestrator task-run \
    --task-id F3 \
    --paper-type empirical \
    --topic your-topic \
    --cwd . \
    --triad
  ```
  常用可选参数：
  - `--domain <name>`：把运行时领域 profile（例如 `econ`、`cs`、`psychology`）注入 task packet 和 prompts
  - `--venue <name>` / `--context <text>`
  - `--mcp-strict` / `--skills-strict`
  - `--profile-file <path>` + `--profile <name>`（以及 `--draft-profile` / `--review-profile` / `--triad-profile`）
  - `--focus-output <path>`（可重复）+ `--output-budget <n>`：把本次运行收敛到更小的 active outputs，其余 contract outputs 明确标记为 deferred，而不是继续扩写
  - `--research-depth standard|deep` + `--max-rounds <n>`：提高证据扩展强度，并把 review/revision loop 拉深
  - `--only-target <id>`（可重复）：针对结构化 Stage-I 任务 `I4`-`I8`，回读 `RESEARCH/[topic]/code/` 下的现有 artifact，并且只重跑指定 actionable target
  - `--skip-validation`：关闭严格的 MCP/skill 可用性校验，并跳过 artifact validator gate；运行结果会明确给出 warning，同时把 `validator_gate.skipped=true` 写进结果数据
  - `--guidance-mode off|read|propose|apply`：控制项目本地 `.qiongli/` 指导层；默认 `propose` 会在存在指导文件时读取它，写入 trace bundle，并生成保守的 guidance update proposal
  - `--update-academic-context`：对支持的阶段收口任务（`A5`、`B6`、`C5`、`D3`、`E5`、`F6`、`H4`），把 `context/research_state.md` 和 `context/decision_log.md` 追加进本次 active outputs，并向 draft prompt 注入阶段化的 academic continuity 更新约束
  - 内置 profile 新增 `focused-delivery`、`deep-research`；原有 `default`、`rapid-draft`、`strict-review` 仍可用

  正式研究产物仍然属于 `RESEARCH/[topic]/...`。第一次非 `off` 的 task-run 会在缺失时自动初始化 `.qiongli/local_guidance.md` 和 `.qiongli/trace/`。orchestrator 会要求运行时 agent 创建这些 required files；如果 agent 只返回文本而没有真正写入文件，validator 会把它们标为 missing。`.qiongli/trace/runs/<run_id>/` 是独立追溯目录，即使正式产物不完整，也会记录 task packet、draft、review、validator gate 和 guidance proposal。

  示例：减少辅助文件，但保持更强的深度审查
  ```bash
  python3 -m bridges.orchestrator task-run \
    --task-id F3 \
    --paper-type empirical \
    --topic your-topic \
    --cwd . \
    --focus-output manuscript/manuscript.md \
    --research-depth deep \
    --draft-profile deep-research \
    --review-profile strict-review \
    --triad-profile deep-research \
    --triad \
    --max-rounds 4
  ```
  示例：只重跑某个 Stage-I planning step
  ```bash
  python3 -m bridges.orchestrator task-run \
    --task-id I6 \
    --paper-type methods \
    --topic llm-bias \
    --cwd . \
    --only-target S1
  ```
  示例：在阶段收口任务中强制刷新项目级学术上下文连续性产物
  ```bash
  python3 -m bridges.orchestrator task-run \
    --task-id F6 \
    --paper-type empirical \
    --topic your-topic \
    --cwd . \
    --update-academic-context
  ```
- `task-plan`：从合同渲染依赖任务顺序（用于“从哪一步开始做”）
  ```bash
  python3 -m bridges.orchestrator task-plan --task-id F3 --paper-type empirical --topic your-topic --cwd .
  ```
- `guidance`：管理项目本地 guidance 和 trace
  ```bash
  qiongli guidance init --project-dir .
  qiongli guidance show --project-dir .
  qiongli guidance add --project-dir . --name writing-style
  qiongli guidance list --project-dir .
  qiongli guidance lint --project-dir .
  qiongli guidance trace --project-dir .
  qiongli guidance apply \
    --project-dir . \
    --proposal .qiongli/trace/runs/<run_id>/guidance_update_proposal.md
  ```
  项目本地定制写在 `.qiongli/local_guidance.md`；运行追溯写在 `.qiongli/trace/index.jsonl` 和 `.qiongli/trace/runs/<run_id>/`。这些文件不会修改 canonical workflow contract、内置 skills 或 release payload。
  Guidance proposal 默认是 project-local。proposal 可以建议 `user-global` 或 `canonical-candidate`，但 `qiongli guidance apply` 只会写 `.qiongli/local_guidance.md`。将规则提升到 `~/.qiongli/preferences.md` 或 canonical source，需要显式的后续命令或正常 repository PR。
- `code-build`：学术代码工作流入口
  ```bash
  python3 -m bridges.orchestrator code-build \
    --method "Staggered DID" \
    --topic policy-effects \
    --domain econ \
    --focus full \
    --cwd .
  ```
  关键参数：
  - `--topic <slug>`：提供后会进入严格 Stage I 工作流；不提供时才回落到 legacy prompt-only 模式
  - `--focus <name>`：映射到 `I1`/`I2`/`I3`/`I4`/`I5`/`I6`/`I7`/`I8`，或用 `full` 跑 `I5 -> I6 -> I7 -> I8`
  - `--domain <name>`：注入对应的 `skills/domain-profiles/*.yaml`
  - `--paper-type <type>`：严格 Stage-I 路由使用的论文类型
  - `--triad`：在最终严格 review 阶段追加第三个独立审计
  - `--paper <path-or-url>`：可选论文引用，会带入任务上下文
  - `--only-target <selector>`（可重复）：定向 follow-up 模式
    - 单阶段 focus：直接用 `S1`、`P1-01` 这类 target ID
    - `--focus full`：必须写成 `STAGE_ID:TARGET`，例如 `I5:decision-1`、`I8:P1-01`

  示例：只跑高级 CS 方法的 spec 阶段
  ```bash
  python3 -m bridges.orchestrator code-build \
    --method "Transformer Fine-Tuning" \
    --topic llm-bias \
    --domain cs \
    --tier advanced \
    --focus code_specification \
    --paper-type methods \
    --cwd .
  ```
  示例：在 full 流程里只重跑特定 target
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
- `single`：单模型执行（调试/快速跑）
  ```bash
  python3 -m bridges.orchestrator single --prompt "..." --cwd . --model codex
  ```
- `chain`：一端生成、另一端验证
  ```bash
  python3 -m bridges.orchestrator chain --prompt "..." --cwd . --generator codex
  ```
- `role`：按专长拆分任务
  ```bash
  python3 -m bridges.orchestrator role --cwd . --codex-task "..." --claude-task "..."
  ```

---

## 4) Bash 脚本入口（不依赖 pipx）

### 4.1 远程 bootstrap 安装器：`./scripts/bootstrap_qiongli.sh`

用途：
- 在没有 Python 的机器上完成安装或刷新。
- 下载 GitHub release/branch 压缩包，解压后转调其中的 `scripts/install_qiongli.sh`。

```bash
./scripts/bootstrap_qiongli.sh \
  --repo owner/repo \
  --target all \
  --project-dir /path/to/project \
  --overwrite
```

说明：
- 依赖 `bash` 和 `curl` 或 `wget`，以及 `tar`。
- 支持 `--ref <tag-or-branch>` 配合 `--ref-type tag|branch`。
- 默认会安装 shell CLI 命令：`qiongli`、`ql`、`research-skills`、`rsk`、`rsw`。
- 如果你不想安装 shell CLI，可加 `--no-cli`；如需改目录，可用 `--cli-dir <path>`。
- 远程 bootstrap 只支持 `--mode copy`。
- `--doctor` 在没有 `python3` 时会自动跳过。

### 4.2 安装脚本：`./scripts/install_qiongli.sh`

```bash
./scripts/install_qiongli.sh \
  --target all \
  --mode copy \
  --project-dir /path/to/project \
  --install-cli \
  --overwrite \
  --doctor
```

说明：
- 这是本地仓库安装器。
- `copy/link` 安装路径本身不再依赖 Python。
- 如果需要同时安装 shell CLI，可加 `--install-cli`；默认目录为 `${QIONGLI_BIN_DIR:-${RESEARCH_SKILLS_BIN_DIR:-~/.local/bin}}`，也可用 `--cli-dir <path>` 覆盖。
- `--doctor` 仅在系统存在 `python3` 时运行 `python3 -m bridges.orchestrator doctor --cwd <project>`。

### 4.3 Release 自动化：`./scripts/release_automation.sh`

```bash
./scripts/release_automation.sh publish --version 0.1.0 --from-tag v0.1.0-beta.X
./scripts/release_automation.sh pre  --tag v0.1.0-beta.X
./scripts/release_automation.sh post --tag v0.1.0-beta.X --create-release
```

推荐方式：

- 日常发布只用 `publish`
- 只有诊断或恢复时才用 `pre` / `post`
- 让 `publish` 统一负责 commit、推送 branch、branch CI/check 门禁、推送 tag、等待 tag publish、GitHub Release 和 acceptance receipt
- release-prep commit 通过 `CI` 和 `Checkout Install Check` 之前，不创建也不推送 release tag
- stable 正式版从 `CHANGELOG.md` 对应章节发布
- beta / prerelease 继续从 `tooling/release/<tag>.md` 发布

也可单独运行：

```bash
./scripts/release_preflight.sh [--tag v0.1.0-beta.X] [--quick] [--skip-smoke] [--maintainer-smoke] [--no-strict]
./scripts/release_postflight.sh --tag v0.1.0-beta.X [--skip-remote] [--skip-ci-status] [--wait-ci] [--ci-timeout-seconds 900] [--ci-timeout-mode soft] [--create-release]
```

`publish` 始终使用 hard CI 门禁：tag 创建前必须通过 branch checks，GitHub Release 创建前也必须确认 tag publish workflows 通过。`--ci-timeout-mode soft` 只用于手动 `post` 诊断或恢复，可把未完成 CI 记录为 acceptance receipt 里的 `pending`，不再用于日常 publish。

### 4.4 Beta smoke：`./scripts/run_beta_smoke.sh`

```bash
./scripts/run_beta_smoke.sh
./scripts/run_beta_smoke.sh --tier release
./scripts/run_beta_smoke.sh --tier maintainer
```

这个主 smoke 入口现在支持两档：

- `release`：内置 literature pipeline smoke + `doctor`
- `maintainer`：包含 `release` 全部内容，并额外执行 `parallel` 和 `task-run` 的 profile 路径检查

release preflight 默认只跑 `release` 档。只有在你明确想补跑维护者级别检查时，才加 `--maintainer-smoke`。

### 4.5 Literature smoke：`./scripts/run_literature_smoke.sh`

```bash
./scripts/run_literature_smoke.sh
```

### 4.6 CI 注入打包默认上游：`./scripts/inject_project_toml.sh`

GitHub Actions 构建时会运行它，把当前仓库 slug 写入 `qiongli/project.toml`，让 pipx 安装后的 CLI 默认指向正确上游。

```bash
bash scripts/inject_project_toml.sh

# 或覆盖（用于构建时切换到别的 upstream repo）
QIONGLI_REPO_SLUG="other-owner/other-repo" bash scripts/inject_project_toml.sh
```

---

## 5) 校验器（推荐在 CI/发布前运行）

```bash
python3 scripts/validate_research_standard.py --strict
python3 -m unittest tests.test_orchestrator_workflows -v
```

项目产物校验（在你的项目里跑）：

```bash
python3 scripts/validate_project_artifacts.py \
  --cwd /path/to/project \
  --topic your-topic \
  --task-id H1 \
  --strict
```
