# Changelog

本文件汇总自 `v0.3.0`（2026-03-25）以来到当前 `HEAD`（2026-05-22）的主要更新，重点记录用户可感知的新能力、安装体验变化与重要修复。正式版条目采用 summary 写法，将对应 beta 演进合并整理，不再按小 beta 分段展开。

## [Unreleased]

暂无。

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
