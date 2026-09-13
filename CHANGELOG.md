# Changelog

本文件汇总自 `v0.3.0`（2026-03-25）以来到当前 `HEAD`（2026-09-01）的主要更新，重点记录用户可感知的新能力、安装体验变化与重要修复。正式版条目采用 summary 写法，将对应 beta 演进合并整理，不再按小 beta 分段展开。

## [Unreleased]

暂无其他未发布变更。

## [2.0.0-alpha.5] - Internal candidate (unpublished)

该版本是绑定精确源码
`842f6bb7136fc03551b7a1acf3b612daa3dc6953` 的内部测试候选。Native CI run
`33525293258` 与非发布 promotion run `33527363262` 已通过；未创建 tag、GitHub
Release、更新通道或公开公告。

### Changed

- 将原生 Cargo workspace、Cargo.lock、Codex/Claude Plugin、Full MCPB、Skill
  registry、workflow 与嵌入内容版本统一推进到 `2.0.0-alpha.5`。
- 纳入 Alpha 4 私有候选之后完成的授权异常生命周期、自授权负面矩阵与 Trellis
  scoped standing implementation authorization 改进。
- 使用现有 Native CI 与 Community Alpha promotion 生成三平台内部候选，不新增
  发布流水线。

### Verification boundary

- 三个平台候选来自同一精确 `2.x` 源码，并通过 Native CI、目标原生启动和候选安装
  生命周期检查。
- 发布授权、离线签名、公开上传与公开回读被有意跳过；这些内部证据不能授权未来版本
  发布。

### Internal-candidate limits

- macOS 使用 ad-hoc 签名且未公证；Windows 未做 Authenticode 签名；Linux 依赖
  AppImage/portable package 声明的运行条件。候选仅用于内部验证。
- 未来公开预发布使用新版本并重新运行完整资格链，不复用 Alpha 5 的内部候选或收据。

## [2.0.0-alpha.4] - Private test candidate

该版本是绑定单一 `2.x` 合并源码的 macOS arm64、Windows x86_64 与 Linux x86_64
私有测试候选，仅通过保留三天且需要认证的 GitHub Actions artifact 分发。
`publication_allowed=false`；不创建 tag、GitHub Release、更新通道或公开公告，当前
公开的 Qiongli 2 预发布版本仍为 `v2.0.0-alpha.1`。

### Changed

- 将原生 Cargo workspace、Cargo.lock、Codex/Claude Plugin、Full MCPB、Skill
  registry、workflow 与嵌入内容版本统一推进到 `2.0.0-alpha.4`。
- 候选版本纳入当前 `2.x` 已完成的公共契约冻结、N-2 migration/rollback、forward-
  version fail-closed、灾难恢复、Graph v1、Host 集成、三平台 provenance 与安装生命周期
  改进，但不会据此推断未完成里程碑或公开发布资格。
- 使用现有 Native CI 与 Community Alpha promotion 生成精确源码三平台候选；安装和
  替换保持手动，不发布自动更新 metadata。

### Verification boundary

- 必须在合并后的精确 `2.x` 源码上通过本地 release readiness、显式完整 Native CI
  和 publication authorization 保持 false 的三平台聚合。
- 下载后的封闭文件集合、字节数、SHA-256、源码、版本、Native CI run、promotion
  attempt 与 candidate-set digest 必须独立核验并记录在 path-redacted receipt 中。
- 绿色 CI、合并或候选聚合都不授权 tag、Release、公开上传、更新通道或公告；历史
  Alpha 3 receipt 不能用于本候选版本。

### Community Alpha limits

- macOS 产物使用 ad-hoc 签名且未公证；Windows 产物未做 Authenticode 签名；Linux
  依赖 AppImage/portable package 声明的运行条件。它们都不具备生产级发布者信任。
- `GOV-413`、`GOV-417`–`GOV-418`、`PLT-401`–`PLT-408` 与
  `SEC-401`–`SEC-405` 保持现有未完成状态；本候选版本不构成 M1 或 Stable 退出证据。

## [2.0.0-alpha.3] - Unpublished candidate

该 exact first-usable 内部候选版本尚未发布，`publication_allowed=false`，
公开 Release 仍为 `v2.0.0-alpha.1`。

### Added

- 新增原生 Academic Graph v1，将受支持的项目、artifact、capture、claim、evidence、analysis、output 与 provenance 汇总为确定性的只读研究图，并在 App、CLI 与 Full MCP 中共享同一份覆盖率和 stale-state 语义。
- 新增 receipt-backed 原生 CLI 安装、校验、修复、删除和 shell PATH 配置生命周期；删除时会验证所有权和 digest，并在安全时恢复被替换的前序文件。
- 新增 Codex 与 Claude Code 的结构化 Host 安装指示、版本探测、激活/MCP 观察和真实客户端兼容测试；未观察到的 Host 状态不再显示为 Ready。

### Changed

- 桌面端统一采用 compact Nova/default-radius 设计系统，减少页面空白和说明文字，统一 block、tab、sidebar、暗色模式、交互过渡与溢出约束。
- Community Alpha 发布链改为从成功的同一 source commit `Native CI` 结果触发，版本、候选产物名、macOS 签名/验收和 release notes 均由当前 Cargo 版本派生。
- 原生 2.x 安装文档与旧版 npm、Python、shell 1.x 安装路径明确分离，避免将退休的 1.x CLI 误认为原生 2.x 产品。

### Fixed

- 修复 Host probe 不可观察或失败时仍可能显示 Ready 的问题，并保留 unavailable、timeout、malformed、version mismatch 和 positive observation 的独立状态。
- 修复 Academic Graph stale 原因、canonical coverage 与不同入口之间可能漂移的问题。
- 修复 `Cargo.lock` 多个工作区包、插件清单、嵌入内容和发布标签之间可能出现混合版本的问题。

### Community Alpha limits

- macOS 为 ad-hoc 签名且未公证；Windows 产物未做 Authenticode 签名；Linux 依赖所声明的 AppImage 运行条件。它们都不具备生产级操作系统发布者信任。
- 不承诺任意目录或自由文本的启发式图推断、不受限的 Full MCP mutation、云端执行、Codex/Claude Desktop Marketplace 绕过或 Stable 资格；唯一的 Full MCP 项目写入 `qiongli_project_capture_apply` 仍要求 preview、匹配 digest 与明确批准。
- 自动更新仅在另行发布并通过目标平台验收的签名 update metadata 可用时启用；否则使用已记录的手动替换与回滚流程。

## [1.17.0] - 2026-07-08

### Added

- 新增 registry-backed 平台 target governance：release、marketplace validator、本地 installer、npm plugin-lite、local install acceptance 与 download guide 现在共享同一套 platform target metadata、recommended keys、adapter/materializer metadata 和 smoke policy，减少 Codex、Claude Code、Claude Desktop、Antigravity、npm 与 PyPI 安装面的分叉。
- 新增 release download index 与 artifact manifest：稳定版和 beta release page 现在生成按安装面分组的人类下载指南、机器可读下载索引和 per-target artifact policy，帮助用户区分 marketplace install、direct Desktop plugin、fallback skill ZIP、MCPB、Zotero Companion 与 maintainer-only artifacts。
- 新增 political economy、geoeconomics 与 economics-accounting bridge runtime activation：这些 subject 现在有 dedicated runtime signals、fixture packs、near-miss guards、method-lens borrowing 和 runtime-enabled gate，保持 adaptive subject router 的可测扩展。
- 新增 Zotero collection 与 reference note workflow：Zotero Companion 与 literature MCPB 支持 collection path mapping、reference note 写入、dry-run 计划和测试覆盖，帮助文献资料从 search/export 进入本地 Zotero 项目组织。
- 新增 Rust Marketplace Lite MCP roadmap、design spec 与 implementation plan，明确后续 marketplace-lite provider MCP 将迁移为 plugin-bundled Rust local executable，同时 Full CLI 继续保留 Python 完整运行时。

### Changed

- installer 现在通过 registry recommended keys 自动选择 Codex、Claude Code、Claude Desktop direct plugin、Antigravity、本地 plugin 和 npm plugin-lite 的推荐 target，避免 release guide、installer 和 validator 使用各自独立的 target ID。
- npm plugin-lite lifecycle messaging 现在显式记录 registry-derived target metadata，并区分 npm-managed plugin root、link-mode marker、full runtime plugin 与 stale plugin source。
- release automation 现在在 preflight、local install acceptance、artifact upload、release notes、release receipts 和 postflight upload list 中复用 registry-backed artifact/download metadata。
- Claude Desktop direct plugin 继续保持与 Codex marketplace plugin 分离：direct plugin ZIP 只包含 Claude manifest、commands、bundled literature MCP 和 canonical workflow skill，不包含 Codex metadata 或 Codex workflow wrapper skills。

### Fixed

- 修复 release staging 中 local Qiongli state、target metadata、artifact manifests、recommended target labels 和 release companion target registry 的若干漂移问题，使发布前验证可以更早阻断错误 artifact。
- 修复 npm beta alias 与 self-update 路径，使 prerelease channel 可以正确解析并刷新对应 payload。
- 修复 release guide/download notes 中 end-user install path 与 maintainer artifact path 混杂的问题，让 Codex/Claude marketplace 用户优先走 marketplace command，而不是误下 plugin tarball。
- 修复多个 subject runtime near-miss 与 method-only borrowing 边界，降低 subject auto-routing 在课程、运营、政策简报或非研究任务中的误触发风险。

## [1.16.1] - 2026-07-06

### Fixed

- 修复 Claude Desktop direct plugin archive 混入 Codex plugin metadata 与 expanded
  Qiongli wrapper skills 的问题，使 direct desktop plugin 只包含 Claude plugin
  manifest、commands、MCP server 与必要 runtime payload。
- 更新跨平台 packaging regression checks，明确禁止 `.codex-plugin/` 和
  `skills/qiongli-*` 出现在 Claude Desktop direct plugin ZIP 中，同时继续验证
  Codex、Claude marketplace、Desktop/Web skill ZIP 与 Antigravity 安装面。

## [1.16.0] - 2026-07-06

### Added

- 新增 coursework / assignment 支持：`/coursework` 现在覆盖 assignment brief intake、rubric 与 learning outcome mapping、coursework outline、claim-evidence plan、draft、revision 和 final readiness，对应 canonical Task IDs `L1`–`L7`。
- 新增 dissertation / thesis / major project 支持：`/dissertation` 现在覆盖 dissertation planning、chapter architecture、chapter drafting、supervisor feedback integration、milestone risk plan、final readiness 和 viva/defense preparation，对应 canonical Task IDs `M1`–`M7`。
- 新增 Stage L 与 Stage M 的 skill cards、workflow playbooks、artifact templates、artifact types、routing references 和 skill registry/docs，使本科 coursework、capstone、taught master dissertation 与 supervisor feedback 场景可以进入同一套 Qiongli contract。

### Changed

- `research-workflow-contract.yaml`、capability map、platform routing、Claude/Codex quick commands 和 generated workflow reference 现在统一覆盖 `A1`–`M7`，并保留 coursework/dissertation 的 project-mode 输出路径。
- `qiongli_task_plan` / MCP preview 现在可以规划 `L`/`M` task IDs，返回 functional owner、runtime route、required outputs 和 dissertation prerequisite chain。
- coursework/dissertation templates 默认保留 missing-information 和 do-not-invent 边界，避免伪造课程规则、rubric、导师反馈、数据、个人经历、引用或成绩承诺。

### Fixed

- 修复 paper router、validator、skill docs 和 workflow contract reference 只覆盖论文生命周期 `A`–`K` 的问题，使 project-mode academic writing tasks 也能被标准验证和跨平台路由发现。

## [1.15.0] - 2026-07-06

### Added

- 新增完整论文生命周期 workflow：`/paper-lifecycle` 现在可以从选题、文献、大纲、数据/方法、写作、审查、强 judge、期刊匹配到反馈修订串联成可审计的全流程，并通过 lifecycle harness 检查研究问题漂移、claim-evidence 缺口、judge block 和 submission readiness。
- 新增 manuscript-first 期刊推荐能力：`journal-fit-recommender` 与 MCP preview 可以基于已成稿内容、claim map、方法证据和 venue profiles 给出 primary/stretch/do-not-submit 排名，而不仅是从目标期刊反推写作。
- 新增 adaptive subject runtime：项目可以在 `active_subject: auto` 下动态建议、确认或锁定 subject，并通过 materialized guidance、subject lifecycle state、router eval fixtures 和 runtime smoke 验证 economics、finance、accounting、business 等 subject 的资源加载。
- 新增 accounting 与 business 的 subject runtime 支持，包括 eval-ready、promotion-ready、runtime-enabled gates、near-miss fixtures、venue profiles、method lenses 和 subject-specific guidance。
- 新增 literature search 覆盖诊断与 full-text 边界记录：搜索计划现在区分 provider search、native search、strategy-only、Zotero/user corpus 和 full-text access candidates，避免把 OpenAlex/Semantic Scholar metadata 误判为正文覆盖。

### Changed

- 文献 provider 路由明确以 `qiongli_literature_status`、`qiongli_search_plan`、`qiongli_literature_search` 等工具判断 OpenAlex、Semantic Scholar、Crossref、PubMed 和 arXiv 可用性；`qiongli_collect_evidence` 只代表外部 evidence adapter 命令边界。
- 文献搜索默认范围扩大，review-mode 默认每个 provider 搜索 50 条结果，并保留 raw/normalized/deduped hit counts、provider diagnostics、known-item recall 和 coverage gaps。
- Zotero companion 增强附件核验、路径清洗、full-text status 和 Crossref/DOI metadata 校验，使 Zotero 本地资料可以参与正文/附件可得性判断。
- Claude Desktop direct plugin、Desktop/Web skill ZIP、Codex/Claude plugin artifacts、npm/PyPI package preflight 和 release download index 进一步对齐，稳定版 release page 会给出按安装面分组的下载指南。
- 稳定版发布流程继承 beta train 的 validator、unit tests、controller evals、release smoke、local plugin install acceptance、PyPI/npm preflight 和 tag publish workflow gates。

### Fixed

- 修复 OpenAlex provider configured 与 external OpenAlex MCP command not_configured 的语义混淆，避免把 `RESEARCH_MCP_OPENALEX_CMD` 缺失误报为内置 provider 不可用。
- 修复 Python materialized payload 缺少 `content/subjects/catalog.yaml` 导致 local plugin install acceptance 失败的问题，并增加 subject catalog 与 runtime-enabled subject payload 回归测试。
- 修复 DOI-only、OpenAlex PDF/access candidates 和 full-text retrieval candidates 混杂的问题，使全文获取计划可解释、可复核。
- 修复 subject guidance 写入、managed marker、unsafe path、malformed lifecycle state 和 project manifest update 的若干边界问题，降低项目级 guidance 与全局 workflow contract 漂移风险。

## [1.14.0] - 2026-06-29

### Added

- Codex full plugin 现在会生成轻量 wrapper skills，对齐 Claude Code `/lit-review`、`/academic-write`、`/paper-read`、`/find-gap`、`/study-design`、`/synthesize` 等工作流入口，让自然语言触发和 slash-command 风格需求都能路由到同一套 canonical workflow。
- Python full MCP 与 Claude Desktop literature MCPB 新增混合搜索规划工具 `qiongli_search_plan`，用于区分 provider search、agent-native search、user corpus 和 strategy-only review，并在计划中保留 provenance。
- Literature workflow 增加 provider/native 协作说明，明确 OpenAlex、Semantic Scholar、Crossref、PubMed、arXiv、客户端联网搜索和用户文献库各自的使用边界。

### Changed

- Codex plugin 安装诊断现在区分 plugin、skill 和 MCP 状态，避免用户看到 plugin 已激活但误以为 standalone MCP 必须写入 Codex config。
- `lit-review` 和 `paper-read` workflow 改为优先用搜索计划说明路由策略，再决定是否调用 provider MCP、客户端原生联网搜索或用户提供材料。
- 跨平台文档更新 Codex、Claude Code、Antigravity 的 wrapper skill、plugin-owned MCP 与 standalone MCP 边界，保持多平台入口一致。

### Fixed

- 修复 Codex 本地 full plugin 没有独立 workflow wrapper 时只能激活总入口、难以被动触发细分学术需求的问题。
- 修复 provider 可用性提示过于绝对的问题：没有 provider-connected MCP 时不再把任务锁死为 strategy-only，而是显式允许与客户端原生搜索协作。

## [1.13.0] - 2026-06-28

### Added

- npm/npx `qiongli` 现在定位为免 Python 的资产管理器：`install`、`setup`、`update`、`refresh`、`upgrade`、`remove`、`check` 和 `project ...` 默认留在 Node 路径，不再依赖 Python bridge。
- npm CLI 新增 Node-only `qiongli project init/status/set-subject`，可在没有 Python runtime 的情况下创建和读取 `.qiongli/guidance_manifest.yaml`，并在修改 subject 时保留未知顶层 manifest blocks。
- npm package 新增 plugin-lite payload 安装面：`--surface plugin|both` 可显式安装 bundled plugin-lite assets，并通过 `.qiongli-npm-lite.json` 或 link-mode sidecar marker 区分 npm 管理的 plugin root。
- README、中文 README 和使用指南重新加入 Mermaid 运行架构图，展示安装入口、project guidance、task routing、preview、agent execution、trace 和正式产物的关系。

### Changed

- npm `update`、`refresh` 和 `upgrade` 统一为“从当前 npm package 重新应用 assets”的刷新语义；它们不会升级 npm package，也不会升级完整 Python CLI。
- `qiongli self-update`、`doctor`、`mcp serve`、provider setup、`task-run`、`customize` 等完整运行时命令在 npm 路径下改为明确提示安装 full runtime（例如 `pipx install qiongli`）。
- README、VitePress 中英文文档和 npm README 增加安装入口对比表，明确 marketplace plugin、Desktop ZIP/MCPB、npm/npx、pipx/pip full runtime 和 bootstrap 的包含内容与边界。

### Fixed

- npm plugin-lite 安装和删除现在只覆盖/移除带 npm ownership marker 的 plugin root，避免误删 full-runtime local plugin 或用户自建同名 plugin。
- `qiongli check --json` 现在能识别 npm plugin-only installs，并保留旧的顶层兼容字段，同时增加 nested `skill` / `plugin` diagnostics。
- 修复 link-mode plugin-lite source 丢失后的断链删除和 overwrite reinstall 行为，避免断链被误判为未安装或 reinstall 时触发 `EEXIST`。

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
