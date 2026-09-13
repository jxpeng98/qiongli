# 内嵌 Agent 提案 v3：EMB-Q01 复核与修订建议

日期：2026-09-05。状态：Proposed / 本地审查记录；不是功能验收或发布批准。
本轮仅处理 Q01。Master 仍拥有排序，program ledger 仍拥有状态，accepted ADR
仍拥有架构。本文件把用户提供的 v3 映射到现有执行计划，不建立第二份任务台账。

## 结论

首个“两篇可读文献 → 连续比较 → 来源可定位的候选 → 审阅保存 → 重开可见”
旅程在现有架构上可行，值得继续。v3 的单仓库、单 Agent、受控资料和业务写入边界
应保留；其历史基线、传输选择、已有能力判断和任务依赖需要修正后再实施。

建议先完成已有 Stage 3c/3d 的离线研究闭环，再资格化一个 Codex 入口。
候选保存与基础恢复不依赖真实模型。当前按 ADR 0217 继续使用 ACP；官方 App Server
直连是有价值的替代方案，但不能在 Q01 中静默替换已接受的传输决策。
首轮只需一个 Codex 后端；Claude 和多 Agent 是后续增量，但既有双适配器验收条件
不会因此自动获得通过或被删除。

## 1. 本轮实际基线

| 项目 | 观察 |
| --- | --- |
| 仓库 | `/Users/pengjiaxin/Work/utility/cli-tools/research-skills` |
| 分支 / HEAD | `codex/app-acp-all-chat-realignment` / `4012ee1329620ae34bcfb59cadd0d40db6768214` |
| 目标分支 | `2.x`；本地 `2.x` 与 HEAD 的 merge-base 为 `accafa7477da9d55dd22e161b2e65e0765671b13`；HEAD 多 4 个提交 |
| 与 v3 的差异 | v3 的快照恰为本地分叉点，不能代表当前 HEAD 加工作树；本轮未 fetch，不宣称远端最新 |
| 原有工作树 | `git status --short` 115 项：56 删除、43 修改、16 未跟踪项（目录会折叠）；暂存区为空 |
| 未提交工作 | 包含 Trellis 流程移除、roadmap/ledger/spec 修订，以及 ACP 控制、Tauri/前端、schema、私有历史和恢复实现；全部保留 |
| 本机运行时探测 | `codex --version` 返回 `codex-cli 0.152.0`；仅版本探测，不证明登录、App Server 兼容性或内嵌资格 |
| 本轮权限 | 仓库文档修订、静态追踪、离线检查与官方文档读取；未读取其他产品仓库、账号或真实研究资料 |

已读根 `AGENTS.md`、`CONTRIBUTING.md`、Master 当前 horizon、现有 PRD/design/
implement、ADR 0211/0216/0217、program ledger，以及 Product Control、Native Runtime、
Program Ledger 和 All Chat App 契约。代码观察来自当前工作树，不能仅用 HEAD 重现。

当前台账有 237 项，没有 `EMB-Q` 或 `EMB-S` 项。相关状态为：

| 既有条目 | 当前状态 | 对本计划的约束 |
| --- | --- | --- |
| `PLT-401`—`PLT-403` | accepted | 已接受的容量/边界基线，不重新排为待实现 |
| `PLT-404` | active，依赖 `PLT-408` | ACP 生命周期尚未整体接受 |
| `PLT-408` | proposed | 窄会话/取消/锁契约，不扩成通用 job 系统 |
| `PLT-405` | proposed，依赖 `PLT-404`、`SEC-403` | 当前单 Agent 研究闭环的正式归属 |
| `SEC-401`—`SEC-405` | proposed | 前三项按依赖顺序约束真实工具启用；离线合成负例可先开发 |
| `PLT-406`、`PLT-407` | proposed | 依次位于单 Agent 接受之后；本轮不实施 |
| `REL-902`—`REL-904` | accepted | 复用迁移、未来版本保护和恢复原语；不借此声称新聊天恢复已接受 |
| `PILOT-905`、`GOV-408` | accepted | 分别是历史 Host 能力证据与 Rust schema 权威；不是新内嵌入口的资格 |

ADR 0217 的文本状态为 Accepted，且已登记在架构决策目录；它与上述功能条目的
状态是不同事实。此次不改写 ADR 历史、ledger、生成 index 或 release 状态。

## 2. 需要修正的审查发现

| 优先级 | 发现与证据 | 对 v3 的修正 |
| --- | --- | --- |
| 高 | [ADR 0217](../../../../docs/architecture/decisions/0217-app-owned-acp-and-all-chat-state.md) 已取代 ADR 0211 的 external-only 限制，并将 ACP 规定为新默认传输 | Q01 必须以 0217 为起点。直连 App Server 需要新的 superseding decision；不能仅宣称“扩展 0211” |
| 高 | `all_chat_control.rs` 已有 `DesktopChat`、`ChatRequest`、`ChatSnapshot`；控制请求和更新仍包含 `AcpV1ControlRequest` / `AcpV1Update` | Q02 复用这些所有者。换传输并非零成本插拔；若改变公开 wire/history 形状，按 ADR 0216 生成 schema、golden 和兼容证据 |
| 高 | `ChatPrompt.source_refs` 明确仅是标签；`acp.rs` 创建 session 时尚未装载研究 MCP；App 的 Codex/Claude 分支仍返回 unavailable | “能聊天”不等于已读两篇文献。先把对象选择解析为获准字节和来源清单，证明实际工具集合，再开放真实入口 |
| 高 | `storage.rs::read_academic_graph_artifact` 是封闭路径读取；`FullProjectService::dispatch` 的现有工具表不含任意文献正文读取 | 复用安全读取及 `ProjectArtifactViewV1`，但不能把 Library/Graph 存在写成任意 PDF、附件或正文已可读。首例只用已解析内容；缺失的对象到正文映射属于 Q02 |
| 高 | `capture.rs` 提供受控入库；`consolidation.rs::ConsolidationArtifact` 仅有 ResearchState / DecisionLog。`artifact_review.rs` 的计划类型不是已接通的通用笔记存储服务 | 明确首版保存为带来源的 ResearchCapture，并经既有 consolidation 审阅合并。若要独立 Markdown 笔记，另补有边界的业务服务切片，不能直接落盘冒充复用 |
| 高 | v3 把 Q05 放在 Q04 后、Q06 放在 Q05 后，会让业务与恢复验证等待真实模型 | Q05 的 Fake 保存和 Q06 的基础恢复在 Q02/Q03 后即可开展；只有真实 provider resume、原生隔离及 live 集成依赖 Q04 |
| 中 | `ChatHistory::load_latest` 只恢复最近一份本地观察；`session_info` 明确 `load_enabled=false`、`resume_enabled=false` | 重新显示历史、启动新会话、续接原后端会话必须分别报告。新增精确 run 选择时扩展现有 owner，不以 latest 替代身份 |
| 中 | 当前历史保存逐条更新；私有日志已有容量/删除规则，但 canonical 检查点、聊天观察、模型运行时历史不是同一数据类 | Q02/Q06 保留这些区别：部分输出标为部分，永不变成候选或业务真相；补运行时自身历史、暂存资料和删除边界的验证 |
| 中 | 本轮默认并行 All Chat 检查出现 1 次 fixture 项目创建失败；单项与串行复跑通过，根因尚未确认 | 把首次失败保留为当前基线问题；后续定位创建/隔离路径，不能以串行通过代替默认测试稳定性。详见第 6 节 |

没有发现跨仓库依赖是实现所必需的。上表描述的未接通路径不是当前已开放功能的
安全漏洞声明；它们是启用真实研究入口之前必须闭合的差距。

## 3. 官方 Codex 接入结论

2026-09-05 读取官方 [App Server 文档](https://learn.chatgpt.com/docs/app-server)
及 [CLI 命令文档](https://learn.chatgpt.com/docs/developer-commands#codex-app-server)。
文档支持深度集成、默认 stdio、托管 ChatGPT 登录、thread/turn、事件和中断；CLI
仍将 App Server 用于开发/调试并提示可能变化。稳定 API 子集不等于生产支持承诺。

`readOnly` 默认可广泛读取；暂存 cwd 本身不足以隔离资料。动态工具仍有实验边界。
因此 Q04 必须按目标版本验证受限读取、工具装载、权限、终态、恢复和平台隔离。
采用官方托管登录，不接管 token；按精确 thread/turn 续接和取消。

这是当前文档观察，不能永久固化为“不支持”或“已支持”的产品结论。本机 0.152.0
未做上述资格验证。文档查询不等于账号授权，也不验证第三方适配器条件。

| 路线 | 可以复用 | 新成本 / 必须证明 | 建议 |
| --- | --- | --- | --- |
| 已有 ACP → 固定 Codex adapter | 当前会话/控制/fixture、App 接线和 ADR 0217 | adapter 实际安全能力、认证、工具、load/resume、sidecar；这些尚未通过 | 当前离线开发默认路线，先完成业务闭环 |
| 本仓库直接连接官方 App Server | 项目业务、受控候选、UI 组织和私有历史的所有者 | 官方协议驱动、事件映射、现有 ACP wire/history 兼容、隔离与包装；需正式扩展 0217 | 保留为有边界的替代方案，不同时建设两套会话服务 |
| 直接模型 API / 新通用 Agent 平台 | 对首个旅程没有必要增益 | 第二套认证、执行语义及维护面 | 本轮不做 |

选择标准应是同一两篇合成资料旅程上的：可执行的资料隔离、准确终态/取消、来源
和续接能力、平台自包含性、维护成本。若 ACP adapter 无法满足隔离或精确续接，而
官方直连能以更小维护面满足，则提出 superseding ADR，替换连接层并保留业务层。
不能只因已经写过 ACP 就继续投入，也不能只因“官方”就免除接入验证。

若后续选择直连，最小架构变更只允许 App 显式选择经过资格化的本地 Codex stdio
连接器；保留 External Host、官方认证所有权、候选/preview/approval/CAS、无静默
fallback、旧 ACP 记录可读及各平台独立支持声明。Q01 不给该变更分配 Accepted 状态。

## 4. 已存在的调用链与准确复用位置

以下路径均相对仓库根目录。新增文件名仅可在实现时按现有分层决定。

| 层 / 入口 | 实际所有者及边界 |
| --- | --- |
| App | `packages/qiongli-desktop/src/routes/orchestrator/+page.svelte` → `src/lib/features/all-chat/AllChat.svelte` → `packages/qiongli-app-api/src/all-chat.ts::QiongliChatClient` |
| 原生命令 / 会话 | `packages/qiongli-native/apps/qiongli/src/desktop/tauri_adapter.rs` 注册 `qiongli_all_chat` → `all_chat_control.rs::DesktopChat::execute` → `AcpV1Client::with_session` / `AcpV1Session::run_turn` |
| 本地历史 | `apps/qiongli/src/all_chat_history.rs::ChatHistory` → `crates/qiongli-project/src/runtime_state.rs` 的 lease/CAS 私有存储；已存在 intent-before-send 和中断恢复，未接通后端 resume |
| 上下文与来源 | `crates/qiongli-project/src/service.rs::ProjectStateService`；`academic_graph.rs::AcademicGraphService::{resolve_artifact,read_graph_artifact,read_registered_artifact}` 返回带 digest、行范围和截断标志的 `ProjectArtifactViewV1`；Capture 使用 `read_capture` |
| 工作流 / 方法 | `apps/qiongli/src/orchestration_control.rs::FullOrchestrationService::from_embedded_content` 复用 `EmbeddedWorkflowHostHandoffBuilder` / role / worker builders；内容权威仍为 `content/`，使用 embedded/variant 所有者投影 |
| 工具 | `apps/qiongli/src/agent_run.rs::project_scoped_read_tools`；`crates/qiongli-runtime/src/full_project.rs::FullProjectService`；`crates/qiongli-execution/src/tool_host.rs`。复用只读筛选与校验，不启用该文件中的实验直连 provider runner |
| 任务候选 | `apps/qiongli/src/orchestration_control.rs::submit_host` / `crates/qiongli-execution/src/host_handoff.rs` 已绑定 packet、角色、attempt、revision、digest 和 checkpoint CAS；自由聊天不能伪造 packet 或推进任务 |
| 业务保存 | `crates/qiongli-project/src/capture.rs::{preview_capture,apply_capture,read_capture}` → `consolidation.rs::{preview_capture_consolidation,apply_capture_consolidation}`；App 在 `desktop.rs` 复用其 preview/confirm 路由，Full MCP 经 `mcp.rs` / `FullProjectService` 入库，CLI 经 `capture_cli.rs` 读取 |
| Graph 刷新 | `desktop.rs::academic_graph` → `AcademicGraphService::rebuild_projection`，按真实项目结果重建；无语义就保持空/稀疏，不改派生缓存 |
| Fake / 契约 | `crates/qiongli-execution/src/acp.rs::fixture_agent` 和 `for_development_demo`；`crates/qiongli-execution/tests/fixtures/acp-control-stream-v1.json`；`apps/qiongli/src/all_chat_control.rs` 的真实 Tauri MockRuntime 测试；`apps/qiongli/tests/fixtures/all-chat-*.json` 和 App API/AllChat 消费者测试 |

生产路径尚未贯通上述全部箭头。建议的最小新增适配是：原生解析获准资料 →
绑定 manifest 的比较候选 → ResearchCapture 草稿 → 既有两阶段保存。无需新 DAG、
图谱数据库、跨项目包或通用 Connector registry。

### Q02 要先明确的四个契约

1. **资料清单。** 原生从选择的对象解析字节，记录 project/revision、对象 ID、
   source digest、片段/行范围、读取/截断状态，以及方法资源版本/digest。初始 fixture
   是两篇合成文献的已解析文本和可核验锚点；不宣称已有任意附件读取。模型给出的路径
   和来源标签不能变成读取授权。新公开字段按 ADR 0216 生成并记录兼容类别。
2. **授权与实际装载。** manifest 是选择和快照的记录，不是自授权文书。原生把
   project、manifest digest、工具集合、目的地和 run/turn 绑定到有效授权，每次调用
   再校验。复用 Full 服务但仅暴露本轮允许读工具及受控候选提交；不把整个 Full MCP
   （含 `qiongli_project_capture_apply`）直接装入会话。未知权限、越界项目/对象、
   symlink/路径替换、Shell/hooks/MCP/网络旁路均须有合成负例；无法隔离则保持不可用。
3. **候选与保存。** 比较内容至少分出两篇文献的方法、结论、共同点/差异、限制和
   逐项来源；结构与锚点检查不声称结论科学正确。原生绑定 run/turn、manifest digest、
   project revision 与候选 digest，映射至 `ResearchCaptureDraftV1` 的既有字段。
   首版无决定变更时不制造 decision。入库只是 pending history；consolidation 的
   filesystem-write / academic-review 才授权正式合并。候选编辑重新验证和 preview，
   稳定 capture ID / 原始时间保持重试身份；AlreadyApplied 后读回收据，不重造候选。
   现有 Capture evidence 只有 locator 等字段，不能假装已有正文 digest 校验；新适配
   必须在送审和提交时通过 manifest 重新验证来源，必要时只扩展现有 owner。
4. **恢复与删除。** 沿用现有会话/turn/日志身份；聊天序号、任务 checkpoint generation
   和项目 revision 分开。来源或授权变化后显式重建上下文；批准保存导致的 revision
   更新也要以收据重新绑定后才能追问。重开只观察、不重放。原会话 resume 与新会话
   带有限摘要必须分开显示。删除 Qiongli 对话不删除 Capture/产物，也不冒充已删除
   Codex 自身历史；运行时历史、暂存资料与本地日志的保留/删除分别说明。

## 5. 八项本地职责映射及执行顺序

`EMB-Q01`—`EMB-Q08` 仅作为用户提案的工作包别名，正式状态归下表既有条目。
“旧分类”只保留用户提供的迁移关系，不是依赖、另一项目任务或接受证据。

| 别名 / 旧分类 | 现有归属 | 修订后最小交付 / 依赖 |
| --- | --- | --- |
| Q01 / S01 | 当前计划、ADR 0217、Master | 本审查与差异映射；不建立新 ledger 行，不实施后续代码 |
| Q02 / S02、S05 上下文 | Stage 3c，`PLT-408`、`SEC-401`—`SEC-403`、`GOV-408` 规则 | 复用现有 session/control/Fake；补两篇合成资料的 native manifest、来源校验及候选映射。依赖 Q01 |
| Q03 / S04 会话事件 | Stage 3b 增量 + 3d，`PLT-405` | 扩展已有 AllChat 展示实际资料、两轮回答、候选/拒绝/错误。依赖 Q02；不重建聊天页 |
| Q04 / S03、S06、S05 工具 | Stage 2b 剩余 + 3c，`PLT-404`、`PLT-408`、`SEC-403` | 资格化一个 Codex 路径；协议适配可在 Q02 后开展，真实数据须有资料授权与隔离证明。直连先解决 ADR 0217 变更 |
| Q05 / S05 候选保存 | Stage 3c/3d，`PLT-405`；既有 Capture/consolidation | Fake 即可证明候选修改、预览、两阶段保存、幂等、过期拒绝、收据刷新。依赖 Q02/Q03 和所触及的安全规则，不依赖 Q04；真实集成再汇合 Q04 |
| Q06 / S04 恢复 | Stage 3a 增量 + 3d，`PLT-405`；复用 `REL-902`—`REL-904` | 本地观察/授权变化可随 Q02/Q03 完善，业务崩溃窗口随 Q05，真实 runtime resume 随 Q04。外部变更先聚焦/发送/提交时校验，不提前实施整个 `PLT-701`—`PLT-718` |
| Q07 / S07 | 3d、`PLT-404`/`PLT-405` 安全和各目标原生验收 | 汇合 Q04/Q05/Q06 的协议、业务与恢复证据；无账号 CI 使用 Fake。保留原生自包含要求，开发时安装运行时不等于独立打包通过 |
| Q08 / S08 | 当前单 Agent 旅程的用户观察，反馈现有计划 | 同资料、方法、Agent 的入口对照；先做小型形成性观察，有结果再决定规模。正式交付依赖 Q07，本轮不新增统计承诺或改发布门槛 |

推荐顺序：Q01 → Q02 → Q03 + Q05 的离线保存，并接入 Q06 的本地恢复 →
Q04 的真实 Codex 集成 → Q07 → Q08。Q04 的协议适配与离线闭环可独立推进；
权限/资料边界始终在真实执行之前。Q06 的后端恢复与 Q04 汇合。

这一顺序沿用 Master 的 `PLT-408 → PLT-404 / SEC-401—403 → PLT-405` 方向，
不改变正式依赖或 accepted 标记。第二运行时和多 Agent 不阻塞此工作包的离线开发，
但单 Codex 证据不能关闭既有计划要求的双适配器产品验收。

## 6. 检查入口与本轮记录

文档检查命令从仓库根运行：

```bash
python3 tooling/scripts/update_program_roadmap.py --check
python3 -m unittest tests.test_program_roadmap tests.test_frozen_2x_architecture_baseline tests.test_public_schema_policy
git diff --check
```

现有离线功能检查入口（不是已经完成 Q02—Q08 的声明）：

```bash
cargo test --manifest-path packages/qiongli-native/Cargo.toml -p qiongli-execution --locked --offline acp::tests
cargo test --manifest-path packages/qiongli-native/Cargo.toml -p qiongli --lib all_chat --locked --offline
cargo test --manifest-path packages/qiongli-native/Cargo.toml -p qiongli-project --locked --offline consolidation::tests
pnpm --dir packages/qiongli-app-api test
```

真实入口盘点：`AcpV1Client::for_development_npx` 有固定 Codex/Claude development
presets，但 App 真实后端仍不可用，仓库尚无可据以声称“真实内嵌旅程通过”的验收入口。
`apps/qiongli/examples/native_host_acceptance_contract.rs` 是已有 External Host
fixture/receipt 验证入口；`native_packaged_product_acceptance.rs` 属于原生包验证。
两者不能改名充当新连接器的 live 证据。Q04 补最小显式启用的合成 live harness，
Q07 再收目标包收据，不在常规测试隐式登录或发送资料。

本轮实际检查结果：

| 检查 | 结果 |
| --- | --- |
| `update_program_roadmap.py --check` | PASS：237 项，生成 index 当前有效 |
| 上述三组 Python unittest | PASS：23 tests |
| `python3 tooling/scripts/validate_public_schema_policy.py` | PASS：3 public boundaries、3 compatibility classes、N-2 persisted-state support |
| `git diff --check`、三个修改文档的相对链接解析 | PASS |
| 原有工作树文件 SHA-256 对照 | 除本轮两处计划文档外保持原字节；ledger/index 与 46 条 accepted 记录不变；仅新增本报告 |
| `cargo test ... -p qiongli --lib all_chat --locked --offline` | **FAIL**：4 passed / 1 failed；失败发生在 `all_chat_history.rs:581` 的测试 fixture 项目创建，`PersistenceFailed(AlreadyExists)`；实际 Tauri IPC 等其余 4 项通过 |
| 同一失败项 `-- --exact` 单独复跑 | PASS：1 test，39.42 s |
| 同一 All Chat 组加 `-- --test-threads=1` | PASS：5 tests，40.08 s；不消除前述默认并行失败记录 |
| execution ACP、consolidation、pnpm 全组 | 本轮未运行；上方列出实际可用入口 |
| 真实模型/登录、原 provider 会话续接、真实资料、跨平台原生包 | 未运行 / 未验证 |

单项复跑的精确命令：

```bash
cargo test --manifest-path packages/qiongli-native/Cargo.toml -p qiongli --lib all_chat_history::tests::all_chat_history_deduplicates_bound_updates_and_freezes_schema_and_fixture --locked --offline -- --exact
cargo test --manifest-path packages/qiongli-native/Cargo.toml -p qiongli --lib all_chat --locked --offline -- --test-threads=1
```

失败 fixture 使用进程 ID 与系统时间构造临时目录；本轮没有证明是目录碰撞、创建
竞态还是其他持久化原因，不推断已找到根因，也不修改测试来隐藏失败。
下一项为 Q02 / Stage 3c：先查清这一基线失败，再补资料与候选适配。
本轮 HEAD 未改变，未提交；文档完成不代表 Q02—Q08 或 `PLT-405` 接受。
