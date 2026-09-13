# Qiongli 2.x 一致性审查

日期：2026-09-11。代码基线：`bb668f08`；本轮只增加审查记录。
用户已澄清审查对象为 **2.x**，不存在额外的“1.2x”版本范围。
本记录服务于[当前计划](../plans/2026-09-06-cli-first-closeout-and-extraction.md)，
任务状态仍由[程序台账](../roadmaps/qiongli-program-ledger-v1.json)管理。

## 结论与范围

2.x 已经实现了共享原生服务、受控写入和有来源的研究记录；恢复 1.x 的便捷入口
没有消除这些架构收益。但“安装、发现、诊断、更新”尚未形成一致的用户体验，
新 Skills 入口的实际 Host 会话表现也没有完成验证。不能据此宣布完整替代 1.x，
或声称维护成本、研究质量已经有可量化改善。

本轮追踪当前 CLI、MCP、Plugin 生成与发现路径，重点检查 `v2.0.0-beta.3`
（`9b436bd3`）之后的本地整合，包括 Zotero 修复、直接安装升级和 Skills 薄入口。
历史对照复用冻结的 `v1.19.0-beta.1` 产品分类，以及 REL-902 的
`v1.19.0-beta.1` / `v1.18.0-beta.3` 两个前序迁移样例。没有执行整个旧运行时，
也没有把两个样例的通过推广为所有 1.x 项目的兼容证明。

## 四项价值是否兑现

| 方面 | 当前证据 | 判断与下一步 |
|---|---|---|
| 统一运行时与职责 | CLI `project_cli.rs`、Capture CLI 和 MCP `full_project.rs` 复用 `ProjectStateService`；Plugin 携带同一原生 CLI；薄入口读取同一主 Skill 和工作流 | **核心已实现，外围未统一。** 安装后的诊断仍走旧路径，见 R1。修正在现有观察入口完成，不增加安装状态数据库 |
| 更安全的写入 | 文件计划绑定摘要、当前状态和授权；Host 注册另行确认、执行前重查；迁移保留原始数据并支持回滚 | **有实现和本地反例测试。** 本轮迁移与 9 项 managed-operation 检查通过；不能推广为新版所有平台和 Host 的安全验收 |
| 可追踪的研究 | Graph 从规范记录重建，稳定 claim ID 可绑定多来源与位置；阶段总结保留前序、来源和增量；清理只供用户逐文件审阅并亲自删除 | **结构能力已实现，语义效果部分验证。** Graph 不是任意目录/PDF 的原生语义扫描器。下一步用固定合成材料观察新 Codex 会话中的提取、规范化、重建和总结保真 |
| 更低的维护与扩展成本 | 内容、工作流和模板有唯一来源；Codex 入口没有复制 82 张技能卡；新增入口沿用现有收据，无新依赖 | **有减少重复的设计证据，尚无成本改善的测量。** Rust/Python 两处投影仍需保持一致；CLI 的 Host 工具仍从 `desktop.rs` 复用。遇到真实修改再抽取共享函数，不按文件大小重构 |

代码入口：`packages/qiongli-native/apps/qiongli/src/project_cli.rs`、
`packages/qiongli-native/crates/qiongli-runtime/src/full_project.rs`、
`packages/qiongli-native/apps/qiongli/src/managed_operation.rs`、
`packages/qiongli-native/crates/qiongli-project/src/academic_graph_extract.rs`、
`content/workflow/references/stage-consolidation.md`。
历史语义试验、真实 Host 安装和 Zotero 10.0.2 观察保留在当前计划中；本轮没有重做或升级其验收范围。

## 统一入口与升级口径

| 入口 | 当前实际行为 | 审查时必须区分 |
|---|---|---|
| GitHub 原生 CLI 包 | 支持的平台上解压运行，携带资源，不要求 Python/Node/Rust 运行时 | 操作系统/架构匹配与外部 Host 的要求仍存在 |
| npm / PyPI / Cargo | 发布同一原生实现；npm/PyPI 有启动包装；Cargo 从源码构建 | 渠道所需的 Node/Python/构建工具，不能算作原生二进制运行依赖；包管理器不会清理其他渠道的副本 |
| CLI `install/upgrade plugin` | 导出资源和原生程序，分别确认文件变更与官方 Host 注册；本地 Plugin 启动 Full MCP | 导出成功、Host 已注册/启用、缓存一致、会话已加载、工具可用是不同状态 |
| 原生 Marketplace 平台包 | 捆绑原生程序，当前生成器启动 Lite MCP，14 个工具 | 与本地 Full Plugin 的 32 个工具不同；有相同 Skills 名称不代表工具权限相同 |
| CLI `install/upgrade skills` | 写入 home 或当前项目下的 `.qiongli-skills`，默认 Full 内容 profile | 内容 profile 不等于 MCP 运行 profile；文件已导出不等于 Host 会自动发现，当前提示用户通过 Plugin 加载 |
| CLI `upgrade cli` | 输出原渠道升级方法 | 不会执行包管理器；先升级 CLI，再刷新其随包内容，不能把刷新旧 CLI 的资源当作下载新版 |

Codex 本地导出与 Marketplace 投影生成 **20 个快捷入口 + 1 个 `$qiongli` 主入口**。
排除重复的 `qiongli-qiongli`；82 张技能卡保持内部资源。
Claude 保留一个主 Skill；本次并未为其生成同名的 20 个 Skills。
共享模板要求先读主 Skill，再读工作流，按需加载资源，保留缺少 MCP 时的明确提示，
并且不扩大任务范围、模型选择或写入权限。历史无模板包仍按旧投影验证。

生成来源：`content/workflow/references/codex-workflow-wrapper.md`、
`packages/qiongli-native/crates/qiongli-platform/src/codex_bundle.rs`、
`tooling/scripts/native_marketplace_plugins.py`。
既有字节对比和收据测试证明结构一致；尚不能证明新会话中快捷入口被正确发现和执行。

冻结的 16 项产品分类（`tooling/migration/qiongli-1x-product-parity.json`）
是迁移对照，`classification_status: complete` 表示分类完成。
其中仍有 6 项延期分类，不能当作当前“已完成 16 项”的证明；反过来，新增加的
安装、编排和发布路径也不能继续只用早期 R3Q 的文字概括。比较用户结果时，须同时读取
当前计划和对应证据，不改写 GOV-407 的历史接受结论。1.x 的 copy/link、subject/coverage、
自动目标选择也不是恢复旧命令名字就获得了等价实现。

## 已确认的修正项

### R1 · P2：注册成功后，doctor 仍提示安装集成

位置：`client_inventory.rs:507`、`product_diagnostics.rs:853`；新注册入口在
`plugin_host.rs:21`。Codex 发现逻辑仍调用旧 `discover_codex_user`，读取固定的
`.qiongli/plugins/codex/` 和旧注册收据；Claude 也使用旧托管观察路径。
新入口注册的是 `qiongli-next@qiongli-cli-local`，可使用用户选择的导出位置。

复现：对之前成功注册的两个隔离 home，官方 `plugin list --json` 均报告该 Plugin
已启用。用本轮构建的 CLI 执行 `doctor --json`，对应检查仍返回
`state: attention`、`remediation: install-client-integration`。这是诊断覆盖遗漏，
并不证明 Plugin 的实际研究会话失败；但它会诱导重复安装并使用户无法判断升级结果。

最小修正：让现有诊断入口复用官方只读库存和本地收据校验，保留“未检查会话”的独立状态。
无法观察时明确报告未知，不靠版本号或目录存在推断可用。增加一个“安装后诊断”回归检查，
同时保留缓存漂移、冲突和无法读取的反例。不要建立第二套注册表。

### R2 · P2：新增架构决定未进入当前 ADR 索引

位置：`tooling/architecture/current-decisions.json` 最后一项为 0218，
实际已有 0219–0224。运行 `python3 scripts/validate_arc_201_adrs.py` 返回 1，
报错 `current decision paths must exactly match every numbered ADR Markdown file in filename order`。
直接安装注册对应的 ADR 0224 因此也未被当前索引覆盖。

最小修正：按现有索引契约登记缺失决定，处理新增条目的元数据兼容；复跑同一校验器。
不篡改冻结的 ARC-201 集或历史接受状态，也不跳过文件全集检查。
这一缺口早于本次审查，不能归咎于薄入口模板。

### R3 · P2：旧 Skills 检查仍把延迟加载的指导固定在主入口

位置：`tests/test_skill_structure_lint.py:382`。该检查要求主 `SKILL.md` 包含
`subject-installed domain profile`、`canonical_references`、`diagnostic_artifacts`
和 `failure_triggers`；主入口已经简化，这些内容实际保留在
`content/workflow/references/platform-routing.md:237`。本轮单独执行该检查失败，
与当前计划已记录的旧失败一致。

最小修正：让既有检查验证主入口能到达对应指导、且领域契约仍完整。
保留“丢失引用/丢失契约就失败”的约束，不为通过措辞断言把整段内容重新塞回薄入口。

## 可复用审查顺序

每次跨版本整合，在现有计划中记录基线、改动路径和以下结果即可，不增加审查服务或新台账。

1. **结果与入口**：列出实际改变的安装/命令/Skills/MCP 路径，检查帮助和 README 是否把
   导出、注册、加载、工具就绪说清楚。包升级与内容刷新分别检查；旧命令别名不能改变授权语义。
2. **迁移与写入**：运行受影响的前序样例，验证配置/模型保留、旧数据不变、回滚、重复操作、
   拒绝过期计划和未知文件。多渠道副本只检测并提供明确处置选择，不自动删除或归档用户文件。
3. **Skills 与研究链**：复用投影、链接、收据、Graph 和总结测试；实际会话资格验证依次观察
   一个薄入口、一项缺失工具场景、一轮“来源→规范记录→Graph→阶段总结”。人工选文件也不授权助手删除。
4. **收益与复杂度**：同一缺陷是否只需修改一个现有 owner？新增入口是否复制了工作流？
   对同一合成任务记录加载资源量、操作步骤、失败恢复和来源保真；没有前后测量就不宣称更快或更省。

从仓库根目录选择受影响的命令运行，复用输入未变的结果：

```sh
python3 -m unittest tests.test_native_marketplace_plugins tests.test_skill_resource_links tests.test_academic_graph_content_contracts tests.test_academic_context_continuity tests.test_stage_handoff_contract tests.test_capability_contract_v2 tests.test_command_workflow_alignment
cargo +1.97.0 test --manifest-path packages/qiongli-native/Cargo.toml --locked --offline -p qiongli-platform --test product_parity_ledger
cargo +1.97.0 test --manifest-path packages/qiongli-native/Cargo.toml --locked --offline -p qiongli --lib rel_902_migrates_and_rolls_back_both_supported_predecessors
cargo +1.97.0 test --manifest-path packages/qiongli-native/Cargo.toml --locked --offline -p qiongli --lib managed_operation::tests
cargo +1.97.0 test --manifest-path packages/qiongli-native/Cargo.toml --locked --offline -p qiongli-project --lib academic_graph_
cargo +1.97.0 test --manifest-path packages/qiongli-native/Cargo.toml --locked --offline -p qiongli --test mcp_stdio full_profile_reuses_redacted_project_state_and_accepts_connected_capture
```

两个已知失败必须单独保留，修正后转为通过，不能在汇总中掩盖：

```sh
python3 scripts/validate_arc_201_adrs.py
python3 -m unittest tests.test_skill_structure_lint.SkillStructureLintTests.test_workflow_skill_documents_subject_installed_domain_pack_contract
```

## 本轮证据与边界

| 检查 | 本轮结果 |
|---|---|
| 内容、链接、Graph/连续性契约、能力契约、工作流投影 | 77 项中 76 通过、1 跳过；跳过未计入通过 |
| 原生 Graph 提取、索引和组合等选定模块 | 23 通过；不是全部项目测试 |
| REL-902 前序迁移与回滚 | 1 通过，覆盖两个保留前序样例 |
| 产品分类台账结构与符号 | 4 通过；符号存在不等于真实用户路径通过 |
| managed-operation 授权、状态/漂移检查 | 9 通过 |
| Full MCP 项目、Capture、Graph 与过期 revision | 1 通过 |
| 官方已启用状态与 doctor 对比 | Codex、Claude 均复现 R1；隔离 home，无个人研究数据 |
| ADR 索引、旧领域指导断言 | 各失败 1 项，见 R2/R3 |
| 审查记录与集成检查 | 台账 7 项通过，生成索引一致，原生边界检查通过；所有任务状态保留 |
| 文档构建 | 通过，保留既有高亮/包体积提示 |

原始本地日志为 `/private/tmp/qiongli-2x-review-{content,graph,migration,parity,approval,mcp,domain-lint}.log`。
R1 的源码身份、CLI 摘要和响应在 `/private/tmp/qiongli-2x-review-doctor-receipt.json`；
官方只读库存为同目录 `qiongli-2x-review-host-list-{codex,claude}.json`。
这些临时日志帮助本机复查，不是可跨机器获取的接受证据；本记录保留命令和关键结果。
文档首次构建发现本报告的 8 个仓库源码链接不能作为网站页面解析；已改为源码路径，
复跑通过。两次日志分别为 `qiongli-2x-review-docs.log` 和 `qiongli-2x-review-docs-final.log`。

没有运行新版 Plugin 的模型会话、真实研究项目、Windows/Linux Host 安装或新的发布资格验证。
既有检查通过、已记录的本地 Host 试验、候选接受和发布状态各自独立。

最小后续顺序：先修 R1 的用户可见状态，再修 R2/R3 的现有审查入口，随后复用固定合成项目
做一轮新版 Codex 会话观察。每项在当前计划下单独形成可验证的小改动。
本轮没有增加依赖、运行时模块、Skills 副本或清理功能，也未执行这些后续修复。
