# Changelog

本文件汇总自 `v0.3.0`（2026-03-25）以来到当前 `HEAD`（2026-06-27）的主要更新，重点记录用户可感知的新能力、安装体验变化与重要修复。正式版条目采用 summary 写法，将对应 beta 演进合并整理，不再按小 beta 分段展开。

## [Unreleased]

暂无未发布变更。

## [1.12.0] - 2026-06-27

### Added

- `qiongli update` / `qiongli self-update` 现在会先检查已安装 package 与最新 release 的版本状态；发现可升级或无法确认状态时，会交互式询问是否升级 CLI/package。
- npm launcher 会把自身 package version 传给 Python self-update bridge，避免 npm 安装路径误用 Python package metadata 判断当前版本。

### Changed

- `qiongli update` 成为默认交互式升级入口：package 升级成功后再询问是否刷新本地 plugins/assets；`--yes` 会自动确认 package update 和本地 assets refresh，`--no-refresh` 只升级 package。
- `qiongli upgrade` 明确收敛为 content/assets refresh 命令，不再被文档描述为 CLI package 升级入口。
- README、中文 README 和 VitePress 首页/Guide 入口大幅精简，保留安装、更新、runtime 边界和文档地图，把长篇细节交给对应 guide/reference 页面。

### Fixed

- Python 与 npm distribution payload 现在会携带 `payload/scripts/qiongli_cli.sh` 和 `bootstrap_qiongli.sh`，修复 release archive refresh 时 shell CLI payload 缺失导致的 `FileNotFoundError`。

## [1.11.0] - 2026-06-27

### Added

- 新增项目级 subject guidance runtime：项目可以通过 `.qiongli/guidance_manifest.yaml` 保存 `active_subject`、venue profiles、method lenses、strictness 和结构化 subject 状态；缺失 manifest 时仍可按 `active_subject: auto` 使用。
- `qiongli project` 命令新增项目 guidance/manifest 管理入口，让用户可以在项目层显式设置或检查 subject、venue、method 和 strictness，而不需要重新安装 subject-specific CLI package。
- Task packet、guidance state、trace bundle 和 MCP `qiongli_task_run` preview 现在会暴露 `project_manifest` 与 `project_subject`，让客户端在不启动 agent 的情况下预览本次项目 subject 路由。
- Task evidence 现在可以生成结构化 manifest proposal，并在 `guidance_mode=apply` 时把已接受的 project guidance 更新写回 manifest 和 local guidance。

### Changed

- Orchestrator subject routing 改为优先读取本次显式 `--domain`，再读取项目 manifest，之后才使用保守临时推断或 core/auto fallback，避免用户必须通过安装包选择学科。
- README、中文 README 和中文架构文档更新项目使用流程图，直接从使用态说明 project manifest、local guidance、MCP preview、agent run 和 guidance learning loop。
- 中文 README 与英文 README 对齐，移除已删除的 Gemini runtime 支持表述，并保留 Antigravity legacy config path 的迁移说明。

### Fixed

- 修复 project manifest packet update、trace manifest snapshot 和 guidance-mode subject routing 的一致性问题，避免预览、执行和 trace 中的项目状态不一致。
- 收紧 subject 推断触发条件，减少弱证据导致的 finance/economics 等 subject 误切换。

## [1.10.0] - 2026-06-26

### Added

- 新增 `qiongli self-update` / `qiongli update`：可根据安装渠道委托 npm、pipx 或 pip 更新 CLI package，并在更新后刷新 full local plugin / MCP 安装面。
- npm launcher 新增 self-update 转发路径，并显式标记 npm 安装渠道，避免更新计划误判为 pip/source checkout。
- `qiongli setup` 和 `qiongli provider setup` 新增本地 provider 配置页面，一次配置 OpenAlex、Semantic Scholar、Crossref、PubMed，并展示 arXiv 无需 API key 的使用说明。

### Changed

- 文档更新 `self-update` 与 `upgrade` 的边界：`self-update` 先更新 package 再从本地 payload 执行 `install --overwrite`，`upgrade` 保持为 GitHub release archive refresh 路径。
- 安装和检查测试进一步隔离 Codex、Claude Code、Antigravity、Hermes 的本地配置路径，降低 release/local install 检查污染用户现有环境的风险。

### Fixed

- Provider setup 不再逐项阻塞式读取终端输入，避免一次配置多个 key 时反复启动或误写密钥。
- npm runtime contract 增加 `qiongli.self_update` 覆盖，确保发布 staging 会携带自更新模块。

## [1.9.2] - 2026-06-26

### Fixed

- CLI-managed local plugins now package the full `qiongli mcp serve --transport stdio` entrypoint consistently across Codex, Claude Code, and Antigravity, including Antigravity's plugin-root `mcp_config.json` layout.
- `qiongli check` / install discovery now reports plugin-managed MCP sources for Codex, Claude Code, and Antigravity, while Hermes remains a managed client-level MCP target.
- Release readiness now includes an isolated local install acceptance check so plugin/MCP discovery regressions are caught before publishing.

## [1.9.1] - 2026-06-25

### Fixed

- `release_automation.sh publish` 新增 `--resume-after-ready` 恢复路径：当 `release_ready.sh` 和 release-prep commit 已完成，但后续 branch push、tag push、postflight 或 acceptance receipt 阶段失败时，可以跳过重复 preflight 并由 publish 模式继续接管。
- 发布 tag 处理改为幂等校验：本地或远端 tag 已存在且指向同一个 release-prep commit 时会继续发布；只有 tag 指向不同 commit 时才阻断，避免认证或网络中断后被迫手动拼接发布命令。

## [1.9.0] - 2026-06-25

### Changed

- `qiongli install` 和 `qiongli upgrade` 默认切换为 full local plugin surface：等价于 `--profile full --surface plugin`，让 Codex / Claude Code 通过 CLI-managed local plugin 接入完整 Python MCP，同时继续给 Antigravity / Hermes 写入受管理 MCP config。
- `qiongli upgrade` 新增 plugin-first 迁移清理：只有在新 plugin 安装成功后，才清理旧版 global skills、Claude Code workflow discovery links、以及 Codex / Claude standalone MCP config。
- CLI reference、安装指南、升级指南、README 和 plugin-first 架构文档同步更新默认行为、`--surface` / `--parts` 交互、显式 legacy skills-only 路径，以及 `install` 与 `upgrade` 的清理边界。

### Fixed

- 保留 Antigravity / Hermes MCP config 作为 plugin-first 架构下的 canonical 接入方式，避免升级迁移误删非 plugin-managed MCP 配置。
- 明确 `qiongli remove --surface plugin` 只删除 CLI-managed local plugin，不会隐式删除 MCP client config；需要删除 MCP config 时必须显式使用 `--parts mcp`。

## [1.8.0] - 2026-06-25

### Added

- 新增本地 full plugin surface：`qiongli install --profile full --target all --surface plugin` 现在可以生成客户端原生 plugin，并让 Codex / Claude Code 的 plugin-owned MCP 启动完整 Python-backed `qiongli mcp serve --transport stdio`。
- Codex 本地 full plugin 会写入受管理的 personal marketplace entry、`.codex-plugin/plugin.json`、plugin `.mcp.json`、workflow skill、commands 和 `.qiongli-managed.json`；Claude Code 本地 full plugin 使用同一套 full MCP server。
- `--target all --surface plugin` 现在把 Codex / Claude Code 接到本地 full plugin，同时继续给 Antigravity / Hermes 写入受管理的 full MCP client config。
- `qiongli remove --surface plugin` / `--parts plugin` 新增受管 marker 语义，只删除 CLI 创建的本地 full plugin root 和 CLI-managed Codex marketplace entry。

### Changed

- 安装架构文档更新为 marketplace lite 与 CLI full plugin 的分层模型：marketplace plugin 继续作为 no CLI / no Python fallback，完整本地 Qiongli 由 CLI 生成的本地 plugin 包装 full MCP。
- README、中文 README、install guide、quickstart、CLI reference 和 cross-platform MCP 文档同步说明 Codex / Claude Code plugin-owned MCP、Antigravity / Hermes client config，以及本地 plugin 删除边界。

### Fixed

- 修复 CLI-managed 本地 plugin 删除逻辑，避免误删 marketplace 安装的 lite plugin 或用户自建 plugin root。
- 修复 stale Codex marketplace entry 清理边界，只清理带 `metadata.managedBy = "qiongli-cli"` 的本地 full plugin entry。

## [1.7.0] - 2026-06-25

### Added

- 新增统一 full CLI MCP server 路径：`qiongli mcp serve --transport stdio` 现在可以在同一个 Python-backed MCP 进程里暴露 literature tools、provider/config tools、orchestrator doctor/route、task plan 和 task run preview。
- `qiongli install --profile full` 新增受管理的 MCP client config 写入能力，覆盖 Codex、Claude Code、Antigravity 和 Hermes；`--target all` 会同时注册四个本地客户端的 full MCP 配置。
- Python full MCP literature stack 新增 OpenAlex、Crossref、PubMed provider clients，并通过共享 provider config 与 MCP tool handlers 统一路由，避免 full CLI 和 lite MCPB 文献能力割裂。
- Literature MCP 新增 arXiv provider。arXiv 在 Python full MCP 和 Claude Desktop MCPB 中均默认可用，不需要 API key，并支持 preprint/topic search、arXiv ID metadata、year/category metadata 和 provider diagnostics。
- Claude Desktop literature MCPB 升级到 `0.1.5`，打包 arXiv provider runtime，并继续保留零依赖 Node stdio server、provider setup wizard、query variants、deep-search routing 和 Zotero bridge tools。

### Changed

- 安装和跨平台 MCP 文档明确区分 marketplace/plugin bundled lite MCP 与 CLI full MCP：marketplace 仍适合无 CLI 环境的 literature-provider fallback，full CLI 是完整 Qiongli/orchestrator MCP 的 canonical 路径。
- npm 安装文档更新为 CLI bridge 定位，说明 npm 可以作为入口委托到 Python-backed full runtime，但完整 MCP/orchestrator 能力最终由 full CLI server 承载。
- Claude Desktop/Web skill ZIP、Codex plugin、Claude Code plugin、Antigravity、Hermes 和 CLI 安装说明同步更新 provider 边界，强调 secrets 留在共享 provider config，不写入 plugin manifests 或 release ZIP。

## [1.6.0] - 2026-06-24

### Added

- 新增 project-local guidance stack：除 `.qiongli/local_guidance.md` 外，项目可以通过 `.qiongli/guidance.d/*.md` 维护多个本地约束或扩展片段；运行时会按稳定顺序合成 guidance，并保留 source order metadata。
- `qiongli guidance` CLI 新增 `add`、`list`、`lint` 子命令，用于创建本地 guidance fragment、列出实际生效来源、检查削弱 canonical contract / evidence gate / quality gate 的高风险表述。
- Task-run trace、routing note、MCP preview 和 proposal 输出新增 guidance source metadata、fragment count、source order、conflict notes 与 proposal target/conflict check，让本地 guidance 的影响路径可审计。

### Changed

- Skill-only Qiongli workflow 现在会在项目存在 `.qiongli/local_guidance.md` 或 `.qiongli/guidance.d/*.md` 时读取本地 guidance，但本地规则仍只能作为 advisory layer，不能覆盖 canonical workflow contract、必需产物、证据门、质量门或安全边界。
- Guidance proposal 默认定位为 project-local；`qiongli guidance apply` 只写入 `.qiongli/local_guidance.md`，提升到 user-global preferences 或 canonical source 需要显式后续命令或常规 repository PR。
- README、文档首页和安装/发布文档更新为更完整的入口说明，覆盖 skill/plugin/CLI/MCP/Zotero 路由、稳定版下载方式和本地运行边界。

## [1.5.0] - 2026-06-23

### Added

- 新增 worker orchestration 执行模型：`task-run` 现在可以生成 worker plan、执行通用 worker、合并 worker outputs，并通过 barrier / final review gate 阻断未验证或冲突的主输出。
- 新增 controller-aware 多智能体协作契约，覆盖 Codex primary、Claude primary、duo review、solo fallback、agent handoff、worker merge report、worker review packet 与 run-scoped artifact path，提升 multiagent 执行的可审计性。
- 新增 project-local guidance layer 和 `qiongli guidance` CLI 入口，可在本地项目中自动 bootstrap guidance、读取项目边界，并把本地 trace boundary 纳入 orchestrator task run。
- Stage B literature skills 完成精细化升级：academic search、screening、snowballing、mapping、fulltext、paper extraction、citation formatting 与 reference manager bridge 现在都有更明确的输入、输出、证据限制、失败条件和 artifact handoff。
- Qiongli Literature Provider MCP 扩展为更完整的 scholarly provider stack，新增 Crossref、PubMed、query-plan variants、paginated search、finance/economics routing、provider diagnostics 和更高默认检索上限。
- 新增 Zotero local bridge 与 Zotero Companion extension：支持本地 Zotero source search、record export、Crossref verification、verified import candidate tagging，以及作为 release asset 发布的 companion XPI。
- 新增 B literature precision audit 和对应 release validator 覆盖，保证 Stage B skill 文案不再停留在 descriptive 层，而是包含可检查的 artifact、tooling、routing 和 evidence contract。
- 新增 Codex distribution refs 发布能力，正式 release / prerelease 可以同步生成并发布 `codex/<tag>` 分发引用，便于 marketplace 和 plugin 安装使用稳定 payload。

### Changed

- 插件分发改为 single-source materialization：`qiongli`、`qiongli-next`、Codex plugin、Claude plugin、Desktop skill ZIP 与 npm/PyPI payload 都从 canonical source 生成，减少手工镜像和路径漂移。
- Orchestrator runtime 移除旧 research-collab implementation，收敛到 controller-aware orchestrator、agent routing policy、worker contract 和 source layout resolution。
- 写作 workflow 新增 staged writing harness，`academic-write` 与 `paper-write` 更明确地区分 plan、claim map、draft、verification 和 manuscript-facing outputs。
- Stage B 文献工作流默认强调 provider-backed search diagnostics、dedup / screening readiness、known-item recall、coverage gap 和 retrieval manifest，而不是只输出泛化的搜索描述。
- 发布自动化进一步强化：release prep 会同步 generated distribution payloads，tag 前等待 branch CI / checkout install，tag 后等待 PyPI/npm publish workflows，并在 acceptance receipt 中记录结果。
- 文档和安装指南更新为 plugin-first / marketplace-first 分发模型，并同步 Codex、Claude Code、Claude Desktop、Antigravity、Hermes 与 CLI 安装说明。

### Fixed

- 修复 Zotero companion 新版本安装、metadata 展示、release asset 打包和 source verification 相关问题，确保本地 Zotero 桥接可以作为 opt-in source 正常交付。
- 修复 release gates 对 Antigravity、Codex dist refs、Zotero companion assets、generated payloads 和 npm/PyPI package metadata 的识别与校验缺口。
- 修复 Stage B artifact catalog 缺失 `DedupLog`、`SearchDiagnostics`、`FullTextScreening`、`RetrievalManifest`、`ZoteroImportReport` 等输出类型导致 strict validator 阻断的问题。
- 修复 literature provider contract wording 与 MCP/provider stack 路由不一致的问题，避免文献搜索 skill 回退到含糊的非 provider 执行路径。

### Removed

- 移除 Gemini CLI / Gemini plugin lane，官方跨平台目标改为 Codex、Claude、Antigravity、Hermes、CLI/npm/PyPI 与本地 MCP provider 组合。

## [1.3.0] - 2026-06-12

### Added

- Qiongli workflow 新增跨平台触发契约，覆盖 Codex、Claude、Gemini、CLI 与 portable install 场景；自然语言中的 academic research、literature review、manuscript、rebuttal、reproducible analysis 等请求现在更容易路由到 Qiongli。
- 新增全阶段 stage-aware grill 机制：边界访谈、自我批判与 stage handoff 现在可以记录 resolved grill decisions、open grill issues 和 revisit triggers，并在用户模糊、不确定或不知道如何推进时触发轻量 grill。
- Stage I code workflow 新增 academic analysis code 约束，要求围绕 estimand、dataset lineage、diagnostics、robustness checks、manuscript-facing outputs 与 rerun evidence 组织分析代码。

### Changed

- Codex、Claude、Gemini 与 qiongli-next plugin metadata 扩展学术发现关键词和默认提示，提升多平台自动触发概率。
- source checkout 下的 orchestrator standards resolution 现在优先使用仓库 `content/standards`，再回退到 packaged payload，避免开发仓库中的 generated payload 干扰本地测试。

## [1.2.2] - 2026-06-12

### Added

- Qiongli Literature Provider MCP 新增 `review` / `systematic_review` 检索模式，综述检索默认返回 50 条结果，并允许显式 `limit` 提高到 100。

### Changed

- DOI 与精确题名检索现在走 known-item 查询路径，并在 provider 合并后按题名相似度重排，提升 OpenAlex 与 Semantic Scholar 精确文献查找稳定性。

## [1.2.1] - 2026-06-11

### Added

- `qiongli mcp upgrade` 新增 CLI 入口，可在 MCP 命令空间内升级 Qiongli runtime 与已安装资产，并支持 Hermes 等安装目标。

### Changed

- npm CLI help 增加 `qiongli mcp upgrade --target all [--dry-run]` 示例，避免用户配置 MCP 后无法从帮助中发现升级路径。

## [1.2.0] - 2026-06-10

### Added

- 新增 Hermes Agent 作为一等安装目标，覆盖 Python CLI、npm/npx、shell installer、Windows bootstrap 与 `--target all` 安装路径，默认写入 `${HERMES_HOME:-~/.hermes}/skills/qiongli-workflow`。
- `qiongli mcp config example` 新增 `--target hermes --json`，可生成 Hermes MCP 配置示例。
- OpenAlex provider 配置引导新增 API key 说明与保存后的完成反馈，避免 CLI/桌面版引导页保存后继续阻塞 session。

### Changed

- Skills 与 CLI 状态输出现在展示 Qiongli workflow 版本信息，便于确认 Codex、Claude、Gemini、Antigravity 与 Hermes 使用的安装版本。
- 安装文档、本地打包说明和 provider setup 文档补充 Hermes target、MCP 配置和本地 MCPB 打包/tag 流程。

## [1.1.0] - 2026-06-10

### Added

- 新增跨平台 Qiongli MCP runtime 和 CLI-free literature provider runtime，支持通过 MCPB、Codex plugin bundled MCP、Claude Code plugin bundled MCP 以及其他 MCP host 调用文献工具。
- 新增本地 provider 配置向导：`qiongli_configure_provider` 可打开仅监听 localhost 的配置页，引导用户配置 Semantic Scholar、OpenAlex、Crossref 和 PubMed/NCBI，并避免 API key 进入对话上下文。
- 新增 `qiongli-next` 预发布插件通道、Git-backed 插件源、Claude plugin ZIP 产物，以及按安装场景分组的 release download guide assets。
- CLI 新增 installed assets remove 能力，并扩展 orchestrator runtime tools，支持 controller routing、task-run preview 与 runtime closure flow。

### Changed

- 统一已发布插件 package metadata，包括 category、description、author 和 platform-specific manifest 信息；Codex、Claude、Gemini 与 Claude Desktop/Web 的分发说明保持一致。
- 发布流程现在要求 branch `CI` 和 `Checkout Install Check` 通过后才允许创建 release tag，并在 postflight 中等待 PyPI/npm tag publish workflows，降低 release 与 publish 脱节风险。
- Subject overlays 与 packaged workflow payload 进一步加深，literature workflow 默认通过 provider route 执行，并增加路径漂移和 plugin mirror 回归测试。

### Fixed

- 修复 Claude Desktop MCPB / plugin-bundled MCP 的 stdio framing 兼容问题，解决 Desktop host 无法 attach 到 Qiongli Literature Provider 的连接失败。
- 修复 `qiongli-next` MCP server id 隔离、plugin workflow source 同步、task-run preview / triad routing 以及 controller fallback routing 的一致性问题。
- 修复重构后 release、CI、checkout install、npm bridge 和 packaged runtime standards 的路径解析问题。

## [0.13.0] - 2026-05-29

### Added

- 新增官方 `business` 与 `finance` subject packages，面向本科及以上研究使用，同时以博士及以上可投稿学术期刊论文为质量门槛，覆盖 catalog、overlays、subject-specific skills、venue profiles、eval cases 和 generated payloads。
- GitHub Release 现在为 `business` 和 `finance` 生成 Codex / Claude Code marketplace artifacts，以及 `qiongli-claude-desktop-skill-business-<tag>.zip` 和 `qiongli-claude-desktop-skill-finance-<tag>.zip` Claude Desktop focused ZIPs。
- CLI、npm/npx 与 installer subject 参数支持 `--subject business` 和 `--subject finance`；Skillsplace 可暴露 `qiongli-business` 与 `qiongli-finance` 独立安装条目。

### Changed

- Marketplace 和 Desktop ZIP 文档更新为多 subject 分发模型，明确当前公开 Desktop ZIP subjects 为 `core`、`economics`、`business`、`finance` 和 `economics-accounting`，仍不发布 standalone accounting Desktop ZIP。
- Release postflight、artifact contract tests 与 marketplace install validator 扩展到 business / finance 的 Codex、Claude Code 和 Claude Desktop 产物。

## [0.12.1] - 2026-05-28

### Changed

- 发布入口统一收敛到 `scripts/release_automation.sh publish`：该流程现在负责 release-ready 检查、generated payload 同步、release-prep commit、tag push、branch CI 等待、PyPI/npm tag publish 等待、GitHub Release 创建和 acceptance receipt 记录。
- PyPI/npm publish workflows 改为只响应 `v*` tag push，移除手动 `workflow_dispatch` 发布入口；`.github/workflows/release-automation.yml` 保留为诊断/恢复 wrapper，不再作为发布入口。
- release postflight 现在同时等待分支检查 `CI`、`Checkout Install Check` 和 tag 发布 workflows `Publish to PyPI`、`Publish to npm`，避免只检查 branch CI 却漏掉实际发布失败。

### Fixed

- 修复 release-prep 未 stage Python payload 更新的问题，确保 `qiongli/payload`、npm payload、plugin mirror 和版本校验使用同一套发布版本。
- 修复 tag 版本校验未覆盖 Python runtime payload registry 的问题，避免 PyPI 包和 npm 包内置 payload 出现隐性版本漂移。
- 修复 postflight 生成 acceptance receipt 后留下未跟踪文件的问题；`publish` 成功后会自动提交并推送 `release/acceptance/<tag>-receipt.md`。

## [0.12.0] - 2026-05-28

### Added

- 新增 subject-specialized package 体系：`core` 作为默认兼容 subject，`economics` 与 `accounting` 作为学科专精安装包。subject catalog 使用 ordered groups，并通过 `skill_refs`、overlays 和 layered section overrides 生成 effective package。
- 加深 economics v2 内容，并新增 accounting subject 的 registry、overlays、profiles、eval fixtures 与 specialization audit expected terms，用于提升领域术语、方法和 venue 适配深度。
- 新增 coverage-aware subject 安装：CLI/npm 默认 `coverage=complete`，即全量 core 框架加指定 subject 专精；显式 `--coverage focused` 时生成精简 selected subject package。
- 新增官方 composite subject `economics-accounting` 和 composite metadata，用于经济学/会计交叉场景；official composite subjects 是命名 subject，不是任意逗号分隔叠加，仍 materialize 为单一 active `qiongli-workflow`。
- 新增 `qiongli install --subject core|economics|accounting|economics-accounting --coverage complete|focused`、`qiongli upgrade --subject accounting`、npm/npx 同名参数，以及 `check --json` 中的 installed subject/coverage 输出。旧安装缺少 `SUBJECT_MANIFEST.json` 或 `SUBJECT` marker 时按 legacy `core` / `complete` 处理。
- 新增 subject eval 与 specialization audits，并扩展 materializer / npm payload tests，覆盖 accounting、economics v2 depth 与 economics-accounting composite payload。
- Python materializer 新增 `--custom-dir`，并新增 `qiongli customize` scaffold，支持本地 overlays、profiles、registry entries 和 custom skill markdown，只影响本次 materialized output，不回写 canonical source。
- GitHub Release 现在生成 `qiongli-claude-desktop-skill-core-<tag>.zip`、`qiongli-claude-desktop-skill-economics-<tag>.zip` 和 `qiongli-claude-desktop-skill-economics-accounting-<tag>.zip`；旧名 `qiongli-claude-desktop-skill-<tag>.zip` 暂时作为 core alias 保留。

### Changed

- Desktop/Web 文档改为引导用户选择 focused subject ZIP。subject package 是专精安装包，不是降质删减版；统一 workflow、contract、templates、standards 与 quality gates 保持一致，学科深度通过 overlays、selected profiles、official composite subjects 和 local customization 增强。本阶段公开 Desktop ZIP subjects 是 `core`、`economics` 和 `economics-accounting`，没有 standalone accounting Desktop ZIP。

## [0.11.1] - 2026-05-26

### Changed

- Claude Desktop / Claude.ai ZIP 现在改为 slim skill package：保留可执行 workflows、templates、contracts、standards、roles、venue profiles、`skills-core.md`、`skills-summary.md` 与 `skills/registry.yaml`，但省略细分 per-skill markdown specs，避免触发 Claude 上传文件数上限。
- README、README_CN、quickstart 与 install guides 现在明确说明 Desktop/Web ZIP 是轻量安装包；需要完整细分 skill 语料时，应使用 Codex / Claude Code / Gemini plugin 包或源码仓库。
- Release validator 现在会校验 Claude Desktop ZIP 的文件数预算，并阻止误把细分 skill specs 打进 Desktop/Web 上传包。

### Fixed

- 修复 GitHub Release 中 `qiongli-claude-desktop-skill-<tag>.zip` 因包含过多文件而无法拖拽或上传安装到 Claude Desktop / Claude.ai 的问题。实际发布 ZIP 现在控制在 Claude 的 200 文件限制以内。

## [0.11.0] - 2026-05-26

### Added

- 新增 Claude Desktop / Claude.ai 可直接安装的 `qiongli-claude-desktop-skill-<tag>.zip` 发布产物。该 ZIP 以顶层 `qiongli/` skill 目录打包，适合不熟悉命令行或无法接入第三方 marketplace 的 Desktop / Web 用户通过拖拽或上传安装。
- 新增 Claude Desktop ZIP 的构建、结构校验与回归测试，`scripts/build_plugin_artifacts.py` 现在会同时生成 Codex、Claude Code、Gemini 与 Claude Desktop skill 发行产物，并由 release postflight 上传到 GitHub Release。
- 新增 academic boundary interviewer 能力，覆盖研究问题、数据、方法、证据边界与写作主张边界，并提供 `boundary-review` artifact、contract、template、question engine 和严格校验。
- 新增跨阶段 boundary review 支持：`task-run`、context package、academic context continuity 与各阶段 workflow 现在可记录、传播并消费学术边界决策，降低后续写作越界或 claim 过度扩张风险。
- 新增 research proposal writer skill 与 proposal template，将 proposal 写作纳入 Stage F writing contract 与发行包。

### Changed

- README、README_CN、安装指南与 quickstart 现在突出 Desktop / Web 用户的 ZIP 下载安装路径：从 GitHub Release 下载 Desktop skill ZIP，然后在 Claude Skills 中拖拽或上传，无需本地 Code 环境。
- Qiongli 插件安装说明改为 plugin-first / Skillsplace 路径，并移除仓库内旧的 marketplace catalog，减少用户误用过期入口。
- 文档站与中英文指南完成品牌与导航刷新，安装、troubleshooting、agent skills 使用说明更聚焦当前发行模型。
- 发布自动化改进了 publish mode 的本地/CI 行为，避免 release-prep 前污染工作区，并确保 generated distribution payload 在 tag 校验前完成同步。

### Fixed

- 修复 release workflow 在 publish mode 中传递无效 `--create-release` 参数导致发布失败的问题。
- 修复 publish workflow 预构建步骤改写版本文件后导致 `release_ready` 因工作区不干净而失败的问题。
- 修复 marketplace/plugin 分发校验命名与 invocation 覆盖不足的问题，确保公开插件身份与发行产物保持一致。

## [0.10.1] - 2026-05-22

### Added

- 新增 literature-first Stage B 搜索质量诊断合同，覆盖 `targeted_search`、`review_grade`、`systematic_review` 三种模式，并输出 concept coverage、known-item recall、provider coverage、query health、dedup health、screening readiness、snowball readiness 和 recommended actions。
- 新增 deterministic literature search bundle materializer，可将 provider 输出物化为 `search_strategy.md`、`search_log.md`、`search_results.csv`、`dedup_log.csv` 和 `search_diagnostics.md`。
- 新增 `scripts/audit_literature_search_quality.py` 与 `scripts/materialize_literature_search_bundle.py`，为 B1/B3/B6 的离线验收提供可回归执行入口。
- 新增 `qiongli-workflow/references/literature-search-quality-contract.md` 与 `templates/search-diagnostics.md`，明确 Stage B 搜索闭环的 artifact contract。
- 新增 controller-agnostic execution 基础能力，支持在 `task-run` 中记录 `solo`、`duo`、`triad` 执行模式，以及 `controller`、`primary`、`reviewer`、`verifier` 和 `solo-role-gates` ownership metadata。
- 新增 controller-mode contracts、solo role policy、agent handoff、disagreement matrix、duo review report、solo self-review、implementation intent、writing claim map 和 quality gate report 模板。
- 新增 `scripts/audit_solo_role_gates.py` 与 `scripts/audit_agent_handoffs.py`，用于离线审计 Codex-only 写作、Claude-only 工程、duo handoff 和 blocking disagreement artifacts。
- 新增 controller-mode 使用指南：`guides/advanced/controller-modes.md`、`guides/advanced/solo-mode.md` 与 `guides/advanced/codex-claude-duo.md`。
- 新增 controller-mode offline eval corpus 与 runner：`evals/controller_modes/*.json` 和 `scripts/run_controller_mode_evals.py`，覆盖 Codex-only、Claude-only、Claude-primary + Codex review、Codex-primary + Claude review、duo disagreement 与 expected verification blocked 场景。
- 为 `scripts/bootstrap_qiongli.ps1` 增加用于生成用户 `PATH` 更新命令的辅助函数，进一步完善 Windows 安装后的环境变量刷新体验。

### Changed

- `run_scholarly_search()` 保持兼容旧调用，同时追加 v2 search diagnostics；`paper_type=systematic-review` 默认进入 `systematic_review` gate，其他任务默认 `targeted_search`，也可显式声明 `review_grade`。
- citation snowballing 与 screening tracker 现在会消费 search diagnostics，用于 seed rationale、coverage gap、saturation、screening readiness 和后续轮次建议。
- `scripts/validate_project_artifacts.py` 现在对 B1/B3/B6 执行 mode-aware literature quality gate；strict 模式会阻断系统综述级搜索缺失 diagnostics、单 provider、required concept 零覆盖、known item 未召回、snowball 缺少双向记录等问题。
- `scripts/validate_research_standard.py --strict` 现在会检查 literature-first contracts、templates、scripts、tests 以及 portable package / plugin mirror 中的资源一致性。
- Stage B skills 更新为围绕 search diagnostics、materialized artifacts、snowball readiness 和 screening readiness 执行，减少“只写搜索记录但不可验收”的宽松路径。
- `scripts/validate_research_standard.py --strict` 现在会检查 controller-mode contract 文件，并运行 solo role gate audit，防止缺失 claim map、implementation intent 或 verification status 的运行记录通过严格验证。
- `scripts/release_preflight.sh` 新增 controller-mode eval warning stage；当前作为发布前风险提示，不阻断 beta 发版。
- README、README_CN、CLAUDE.md 和 `qiongli-workflow/SKILL.md` 补充 controller-aware task-run flags、solo gates、duo disagreement handling 和 strict validation 建议。
- 正式版发布流程改为以 `CHANGELOG.md` 作为 GitHub Release 的说明来源；beta / prerelease 继续使用 `release/<tag>.md`。
- 发布脚本和版本同步现在会跟踪 `package-lock.json` 的 workspace 版本，确保 npm workspace metadata 与 release tag 保持一致。

### Fixed

- 修复 release tag 校验早于 generated payload 同步执行时可能出现的版本不一致；publish 模式现在会先同步 skill package 与 npm payload，再执行 payload audit 和 tag 版本校验。
- 修复 npm prerelease 发布后的 `latest` dist-tag 清理失败会导致 workflow 失败的问题，改为 best-effort 清理并保持 beta 版本通过 `next` 分发。
- 修复 quality gate contract 路径诊断在 CI 中的稳定性问题。

## [0.4.0] - 2026-04-01

### Added

- 新增严格文献工作流能力，包括 `literature_search`、`metadata_registry`、`fulltext_retrieval`、`citation_graph` 与 `overlay_runtime`，并配套 smoke tests、fixtures 与集成测试。
- 新增学术演示阶段 `K_presentation`，覆盖 presentation planning、slide architecture、Slidev scholarly builder 与 Beamer builder。
- 引入一键 bootstrap 安装流程，支持 `partial` / `full` 两种安装档位。
- 扩展 Codex / Claude Code / Gemini 的集成资产，补充 agent profiles、路由文档与多客户端安装说明。
- CLI 新增 command runtime utilities，为命令解析、执行与安装流程打基础。
- 新增 `install-check` GitHub Actions 工作流，并增强发布自动化的 publish mode、CI 检查、push 事件支持和标签校验。
- Universal installer 增加 `install_manifest.tsv` 打包支持，改善通过 Python 包安装时的资产分发一致性。
- 安装脚本新增缺失客户端 CLI 的提示文案，并加入 `antigravity` 目标支持。
- bootstrap 脚本支持安装最新 beta / prerelease 标签，便于验证预发布版本。
- 为 bootstrap 增加 PowerShell 7+ 版本检查和更稳健的命令执行流程。
- 支持从本地源码仓库安装，便于本地开发、离线测试和验证未发布改动。
- `full` 模式增强了 `PATH` 管理和用户环境持久化，安装 `mise` 后会同步更新当前会话与后续 shell 环境。

### Changed

- 大幅增强 skills 语料与工作流文档，覆盖 literature、compliance、design、writing、code、submission 和 presentation 阶段。
- MCP provider 文档补充 resolver handoff、source-aware merge policy 和环境变量说明。
- Validator 简化冗余兼容性检查，降低维护复杂度。
- 安装与 quickstart 文档重写，推荐使用 `mise` 管理 Python，并明确 Windows 侧需要 PowerShell 7+。
- README 与安装文档补充 prerelease 安装方式以及 `full` 模式行为说明。

### Fixed

- 修复早期 Windows 兼容性问题，并提升跨平台校验一致性。
- 改进 bootstrap 的 dry-run 输出、清理逻辑以及从源码仓库安装 `partial` profile 的处理。
- 改善 Python 环境处理和跨平台脚本执行细节，显式使用 Bash 运行 postflight 与测试脚本。
- 修复 release automation 和 postflight 脚本中的 `git fetch` 分支引用处理问题。

## [0.3.0] - 2026-03-25

### Baseline

- 这是 `0.4.0` beta 系列之前的稳定基线版本，完成了 skill 版本元数据收敛：以 `skills/registry.yaml` 作为单一版本源，并更新了对应的校验与发布流程。
