# Skills：按结果约束的改造记录

日期：2026-09-11。基线：`142d4f8f`。工作分支：
`codex/skills-outcome-boundaries`。
本记录对应[当前执行计划](../plans/2026-09-06-cli-first-closeout-and-extraction.md)，
任务状态由[程序台账](../roadmaps/qiongli-program-ledger-v1.json)管理。

## 本轮目标与边界

让模型根据请求、材料和风险选择执行方法，同时明确它需要交付什么、依据是什么、
何时完成，以及哪些操作需要真实授权。首轮覆盖阅读、写作和自查，追踪它们的共享
参考、角色提示、简要索引和分发产物。没有增加编排服务、模型分级开关或新依赖。

保留 CLI／Plugin／MCP 的职责、82 张内部技能卡、任务编号、正式产物路径、
稳定 claim／decision ID、引用和证据边界。用户已有的模型设置、协议、运行最低
复核轮数和独立审查要求继续有效。项目写入仍经过 preview／approval／CAS；
旧文件始终由用户逐个选择并自行删除。

这次修改针对本地开发内容，版本仍为 `2.0.0-beta.5`。公开 beta.5 包和此前的
发布记录保持原样，本轮没有提交发布，也没有跟踪此前的 Release Actions。

## 取舍与实现

| 发现 | 本轮处理 | 保留的约束 |
|---|---|---|
| 写作规则在入口、角色和核心参考中重复 | 统一引用现有 `references/stage-F-writing.md` | 文稿主线、主张强度、来源与完成标准 |
| 每段至少包含两个分析动作，整篇不得一次起草 | 根据篇幅、依赖和风险决定结构与分段；方法和结果允许准确描述 | 不为增加深度而补造机制、结果或反例 |
| standard／deep 名称自动引入两／三轮复核 | 检查当前结果，修复具体缺陷；有新证据、缺陷或明确协议时继续 | 已配置的最低轮数、连续通过和独立审查要求 |
| 一段写作默认附带提纲、引用表和改进清单 | 交付请求的文本，只补充必要缺口 | 正式 F2/F3/F4/F5 的产物和 Q1/Q2 要求 |
| 阅读入口重复字段解释、输出模板和检索步骤 | 复用已有模板及检索参考，先读所给材料 | 实际访问范围、来源标签、推断强度与不完整证据 |
| 完整论文请求仍再次询问是否继续写全文 | 沿用已授权的范围和已有提纲 | 实质范围变化、未决主张和实际写入审批 |
| 归档试验改写了历史表列名 | 原有归档参考直接指向 `templates/research-state.md` 的固定表头 | `Previous summary`、历史行与稳定 ID |
| Stage F 的示例表与规范 claim map 列名不同 | 删除重复示例，引用已有 claim map 模板 | 精确列名、证据指针与 citation key 的区别 |

主要修改在 `content/workflow/workflows/`、`content/workflow/references/`、
`content/skills/F_writing/manuscript-architect.md`、
`content/skills/Z_cross_cutting/self-critique.md`、三个写作相关角色及
`content/skills-core.md`。没有直接修改安装目录或生成的 Plugin 副本。
分发契约在 `.trellis/spec/content/distribution/index.md` 记录共享规则的归属。

四个主要阅读／写作文件合计由 1,056 行减至 362 行，UTF-8 字节由 42,185
减至 18,170。这是源码规模变化，不是实际上下文 token、运行时间或质量改善的测量。
公共 Stage F 参考有所增加，因为它承接了原来散落的边界说明。

没有采用按模型品牌切换强弱约束、增加多层规划器、默认多代理审查或重写全部技能库
的方案。它们会增加维护点，也缺少本轮证据支持。现有的按需加载、共享契约和原生
写入服务足以承接这次调整。

## 对照试验及其限制

原始请求、完整回复、读取资源的摘要和观察保存在
[试验记录](./2026-09-11-outcome-guided-skills-trial.json)。主试验用五个相同请求，
分别交给无额外研究 Skill、基线 Skill、修订 Skill 的独立代理。没有提供预期答案。
所有材料为合成内容；没有读取私人项目或连接外部模型账户。

| 请求 | 观察 |
|---|---|
| 两句话解释横断面相关摘要 | 三组都保留样本量、相关系数与因果／显著性边界，没有要求启动完整项目 |
| 只输出英文 Results 段落 | 三组都返回一段正文，没有补造测量方式、抽样过程、p 值或置信区间 |
| 无另行最低轮数的 formal deep 自查 | 基线回复宣称默认三轮；修订回复在说明检查结果后结束。回复中的轮数不是独立执行轨迹，不能据此计算节省量 |
| 已配置两轮且需要独立审查，但审查人不可用 | 三组都报告未完成，未修改配置或模拟独立审查 |
| 无写入工具时整理阶段档案并建议旧文件处理 | 基线与修订稿保留来源、CLM-001、DEC-001、原件及候选状态；无 Skill 稿使用了非规范归档路径，也缺少阶段历史行 |

初版修订回复把历史表中的 `Previous summary` 写成了 `Predecessor`。这说明
简化流程时仍需明确机器读取的结构。已在共享归档参考补上模板引用，并增加引用
可达性检查；独立复测保留了精确表头、来源和用户自行删除的边界，记录在同一 JSON 中。

第一次复测误用了旧 subject 生成器的根入口，字节核对发现它与当前根 Skill 不同。
该回复单独归档并排除出当前版本的验证结论。随后复用已有源码组装函数，核对根入口
和参考字节，再以新的独立代理复测。这个检查过程不构成旧 subject 分发入口的修复
或验收结论。

每个主试验分组由一个代理执行五个请求，单组内没有五个完全隔离的模型会话。
使用继承的会话模型，未固定模型构建、采样参数或种子，也未测量 token、延迟或成本。
因此这些结果只用于发现局部行为问题；不能证明跨模型改善、实际 Host 安装成功，
或 2.x 的研究质量已经优于 1.x。

## 验证与源码身份

- 第一轮内容、结构、工作流、Graph、连续性和学术质量检查：39 项通过。
- 链接、阶段交接、能力与路由检查：56 项中 55 项通过，1 项既有跳过。
- 最终角色引用检查：2 项通过；固定表头修正后的相关检查：14 项通过。
- Capability Contract v2 校验通过；12 个既有学术质量样例全部通过。
- 原生内容 crate：46 项通过，包含内容漂移、无效包、链接路径、未知文件、原子写入
  和收据归属反例。CLI 内容相关集成检查：5 项通过，包含空 PATH 和复制到仓库外运行。
- Skill Creator 基础校验、最终差异空白检查和原生改动边界检查通过。
- 由当前 CLI 导出的 Marketplace 内容 profile 中，433 项资源逐项与规范源码字节
  及清单摘要一致，使用下述最终内容包。没有据此声明正式发布或现场 Host 就绪。
- 程序台账 7 项检查通过，生成索引已同步；Docs 构建通过，保留既有高亮与包体积提示。

计数包含不同阶段的相关复查，不应相加当作独立测试总量。结构检查证明引用与产物
形状；离线学术样例证明既有断言检测，不代表现场模型质量。

主要内容提交为 `fd151283`，表结构修正提交为 `a0af80cd`。最终资源锁由现有
`update_qiongli_core_lock` 生成，指向
`a0af80cd9d89cee57aecf1816e44bb34c282fabe`，共 433 项资源：

- Content root SHA-256：`bc9b8753acfe70cae941ffb4c05cf99cd399c3ad0da3bdc92d49d1d61d2cb104`
- Pack SHA-256：`a9bcbc9447ecf17dcd21f5a3b31f93ea762adc97b9b558d6da5759998c22c35e`

原始本地日志以 `/private/tmp/qiongli-outcome-` 为前缀，分别为
`content-tests.log`、`links.log`、`roles-final.log`、`continuity-checks.log`、
`capability.log`、`evals.log`、`native-content.log`、`cli-content.log`、
`pack-final.log` 和 `embedded-export.log`。临时日志只便于本机复查；本记录及
归档 JSON 保留可跨机器阅读的结果。精确变更可由 Git 提交恢复，不另复制整套技能库。
Docs 使用已安装的 `./node_modules/.bin/vitepress build docs`，日志为
`docs-direct.log`；最初的 `pnpm` shim 未输出，已取消，没有安装新依赖。

主要可复跑检查：

```sh
python3 -m unittest tests.test_cross_platform_routing_grill_contract tests.test_skill_structure_lint tests.test_command_workflow_alignment tests.test_academic_graph_content_contracts tests.test_academic_context_continuity tests.test_academic_quality_evals
python3 -m unittest tests.test_skill_resource_links tests.test_stage_handoff_contract tests.test_capability_contract_v2 tests.test_skill_routing_probe tests.test_program_roadmap
python3 scripts/validate_capability_contract.py
python3 evals/runner/run_suite.py
cargo +1.97.0 test --manifest-path packages/qiongli-native/Cargo.toml --offline --locked -p qiongli-content --tests
cargo +1.97.0 test --manifest-path packages/qiongli-native/Cargo.toml --offline --locked -p qiongli --test cli content
./node_modules/.bin/vitepress build docs
```

## 下一步

1. 在真实支持的 Host／模型组合中重复这组请求，并补充长论文、证据冲突、正式 B2、
   F3/F4 和恢复已有运行的任务。先固定可比较的输入、权限和预算，再评价质量和耗时。
2. 优先复查研究设计、系统综述和其他角色的默认要求。区分研究协议／报告规范与
   任意数量要求；例如 PI 角色仍有历史文献数和稳健性数量门槛，本轮只调整其写作部分。
   `tooling/scripts/add_missing_sections.py` 的历史补写文案也不应作为新规则来源。
3. 用现有 Graph 与阶段总结链验证长任务中的来源、主张、历史版本和人工清理边界。
   缺少新证据时不重开已经完成的检查，也不建立第二套研究记录存储。

方向参考：OpenAI 关于[强模型下 Skills 和提示设计](https://developers.openai.com/blog/rethinking-skills-and-prompts-for-gpt-6-astra)
的建议，以及 Anthropic 的[Skills 编写实践](https://platform.claude.com/docs/en/agents-and-tools/agent-skills/best-practices)。
本轮把这些建议转化为局部实现与可观察试验，没有用型号名称代替可靠性证据。

## 后续调整：研究设计与角色边界

继续已记录的下一步，基线 `37386173`，分支 `codex/design-role-boundaries`。
源码提交 `d21cd570`。这次把同样的原则用于研究设计、角色默认要求和综述入口，
没有改动原生执行逻辑、模型配置、冻结的 1.x 运行时或既有研究记录。

### 发现与处理

| 发现 | 调整 | 保留的要求 |
|---|---|---|
| PI、Methods Lead 和 Statistician 给所有研究设置文献或检验数量、功效分析等默认门槛 | 复用 Stage C 的 Design judgment contract，让要求取决于问题、方法和已批准协议 | 指定的检查数量、功效依据、独立审查与方法诊断仍须满足 |
| 定性研究统一套用饱和、双人编码和一致率 | 按分析传统说明材料充分性及解释过程，不替 reflexive thematic analysis 增加不相容的默认检验 | 材料依据、反思、研究范围、协议冲突及未解决问题 |
| 稳健性卡按符号和显著性“投票”，并提供缺少条件的因果补救与参数示例 | 删除通用检验配方，要求说明威胁、假设、能检验什么及什么结果会改变主张 | 必须检查的项目、失败和未运行项目、预先承诺及偏离记录 |
| 竞争解释卡至少三项，无法区分就被视为不合格 | 保留有依据的重要解释；无法区分时明确限制，不补造解释或决定性检验 | 稳定 rival ID、原有表头、实际证据与下游联系 |
| 预注册卡误标 C4，并可能把看过的数据说成未看过 | 改回 C5；复用已有模板，补充收集、访问、分析、注册状态和真实时间线 | 既有注册规则、实际权限、原始协议和修订历史；草稿不等于注册 |
| 综述不足 20 条就建议扩大检索，角色按不一致的全文缺失比例升级 | 依据覆盖诊断及具体缺口决定是否修订；保留未获取全文与待裁决状态 | 正式检索的两个有效提供方门槛、协议边界、独立筛选与 PRISMA 计数 |
| 工作流重复询问已知项目、伦理和协议信息 | 复用已知上下文，仅询问改变决定的缺口；窄请求可留在聊天中 | 正式 C/B 产物、Q1/Q4、实际工具与 preview/approval/CAS |

四张修改的设计卡合计从 1,022 行、50,501 字节降至 619 行、29,395 字节。
共享 Stage C 参考承接规则，角色、工作流、核心摘要和模板指向该参考。包内容仍由
既有生成器处理；没有新配置层、依赖或研究存储。中英文使用说明明确这些内容属于
beta.5 之后的开发构建，未宣称已进入公开包。文字按 Humanizer 检查了语气与事实边界。

方法依据使用一手来源，而非模型品牌或通用模板：

- [Lakens：Sample Size Justification](https://online.ucpress.edu/collabra/article/8/1/33267/120491/Sample-Size-Justification)：样本量论证应对应推断目的和实际约束。
- [Braun 与 Clarke：To saturate or not to saturate?](https://uwe-repository.worktribe.com/output/4820803/to-saturate-or-not-to-saturate-questioning-data-saturation-as-a-useful-concept-for-thematic-analysis-and-sample-size-rationales)及[One size fits all?](https://www.tandfonline.com/doi/abs/10.1080/14780887.2020.1769238)：主题分析有不同方法立场，不宜统一套用饱和与编码一致性。
- [Center for Open Science：Preregistration](https://www.cos.io/initiatives/prereg)：既有数据需要披露先前接触和知识，注册不能消除已知结果的影响。
- [PRISMA 2020](https://www.prisma-statement.org/prisma-2020)：用于透明报告综述的方法和结果，不单凭清单证明研究质量。

### 行为观察

同一组六个合成请求分别交给基线和修订快照下的独立代理执行，不提供预期答案。
包括 reflexive thematic analysis、固定数据描述、不可改动的明确协议、两种竞争解释、
小规模系统综述及已看过结局的预注册状态。输入、完整回答和读取资源的 SHA-256
追加在[原试验档案](./2026-09-11-outcome-guided-skills-trial.json)的
`design_role_followup` 中。试验不访问私人研究、不联网、不执行分析或注册。

两组都给出了可用的回答，均拒绝把未知或未执行工作标为通过。基线模型已能根据
明确请求避开旧配额和不合适的方法要求，因此这次不能据此宣称回答质量、时延或
token 使用已经改善。确定的变化是源码中的矛盾要求已移除，不再依赖模型自行纠正。
修订回复保留五项指定检查、功效依据、真实注册和独立审查的阻塞状态；两种有依据
的竞争解释没有被扩成三种，9 条综述记录没有触发扩大协议范围。

每个代理连续处理其六个独立项目，使用同一会话环境，没有固定随机种子，也未覆盖
其他实际 Host 或外部模型。记录的是回复及资源读取证据，不是实际研究执行或平台
性能基准。后续应在代表性的长任务中观察 C→F 的证据交接和阶段归档，并检查真实
问题出现在哪个方法入口，再决定是否继续修改该入口。

### 验证与集成

先完成的六例修订快照与最终源码只有两个工作流的边界说明不同：研究设计补回
明确的 claim strength／evidence threshold 提示，综述补回
`context/boundary_review.md` 路径及已锁定边界。随后用最终快照对这两个入口独立
复测，回答仍保留材料、协议与正式完成的界限。基线、修订和最终复测的所有读取
资源均核对 SHA-256；两例最终快照与源码字节完全一致。

| 检查 | 实际结果 |
|---|---|
| Skill 结构、入口投影、引用链接、综述检索质量 | 首组 28 项通过 |
| 连续性、Graph 内容、阶段交接、学术质量与能力契约 | 62 项通过，1 项既有跳过 |
| 最终边界、共享参考、C5 路由与检索质量检查 | 21 项通过（包括重复核验的检索用例） |
| 离线学术质量案例 | 12 项通过；这是断言检查，不是模型表现测量 |
| 原生内容包 | 46 项通过 |
| CLI content 集成检查 | 5 项通过，含无 PATH 运行与写入边界负例 |
| Codex workflow wrapper 元数据与不安全入口拒绝 | 1 项通过 |
| CLI 实际导出的 profile | 433 项与规范内容逐字节相同；尺寸和 SHA-256 一致 |
| Skill quick validation、能力契约、冻结源码检查 | 通过 |
| 台账与生成索引 | 7 项通过，249 项任务的索引已更新，接受状态未改变 |
| Docs 构建 | 通过；保留既有高亮语言与大 chunk 提示 |

初次边界检查发现两个入口遗漏明确的契约提示，已补回。修正后检查继续发现上一轮
写作入口已改用共享参考，而测试仍要求旧标题；测试改为验证共享参考可达，同时保留
边界工具、主张强度和证据门槛断言。这三处均已复测，没有通过删除实际要求来放行。

内容提交：`d21cd570d467b17ac585d1440675689d10a50ada`。
内嵌 pack SHA-256：
`5d626cdc8e25c3cc357b2c8f24e1d498c8f5e8c4320e4229ffeb8e0032776958`；
content root SHA-256：
`9b811dd6a586035a952143d0985a062f52e3531c6a04a2767adddb7cb92d2e97`。
公开版本号保持 `2.0.0-beta.5`，这是新的本地开发内容身份；未覆盖公开包。

复现使用现有测试与生成器：Python 检查见 `tests/test_skill_structure_lint.py`、
`test_cross_platform_routing_grill_contract.py`、`test_command_workflow_alignment.py`、
`test_skill_resource_links.py`、`test_literature_search_quality_audit.py`、
`test_academic_context_continuity.py`、`test_academic_graph_content_contracts.py`、
`test_boundary_interviewer_contract.py`、`test_stage_handoff_contract.py`、
`test_academic_quality_evals.py`、`test_capability_contract_v2.py` 与 `test_program_roadmap.py`。
原生检查用 `cargo +1.97.0`、`--offline --locked` 和上述源码提交设置
`QIONGLI_NATIVE_SOURCE_COMMIT`，运行 `qiongli-content --tests`、`qiongli --test cli content`、
`qiongli-platform codex_bundle::tests::workflow_wrappers_preserve_metadata_and_reject_unsafe_entries`，
再运行 CLI 的 `export_marketplace_content` 示例。先用
`qiongli-content --example update_qiongli_core_lock` 生成锁文件。

本次本地日志统一位于 `/private/tmp/qiongli-design-` 前缀下：`content.log`、
`safety.log`（保留最初失败）、`boundary-final.log`、`contracts-final.log`、
`capability.log`、`evals.log`、`native-content.log`、`native-cli.log`、
`native-wrapper.log`、`native-export.log`、`export-check.json`、`ledger.log`、
`native-boundary.log`、`docs.log` 与 `docs-final.log`。临时文件不是长期存储承诺；
本记录、完整试验 JSON、源码提交和资源锁是仓库内的可追踪依据。

本轮只进行本地提交与合并，不推送、不发布、不跟踪此前的发布任务。下一步是用
代表性的长研究任务验证 C→F 的证据联系、Graph 和阶段总结的连续性，再据实际
失败修改其他方法入口。跨 Host／模型及非 macOS 的验收与性能结论仍未建立。

## 协作与连续性：9 月 11 日后续增量

本轮从 `a1a5861d` 继续，源码提交为 `d4818ab5` 和 `baa892dc`。目标是在
C→F、Graph 与阶段归档的衔接中使用真实子代理，并明确跨 Host 交接和 Hook
能承担的工作。沿用 ADR 0218、现有内容卡、交接模板、原生 CLI 和项目写入机制。

### 调整依据与实现

| 检查结果 | 本轮处理 | 保留的边界 |
|---|---|---|
| Host 已有子代理工具；角色名称不能证明独立执行 | `model-collaborator` 明确有界委派、真实任务身份、结果回收、独立首轮与写作后审查的区别 | 不固定模型品牌、代理数或讨论轮数；没有独立审查者时不把自查记为通过 |
| 两种原生 Plugin 的指令都允许无子代理时顺序执行全部角色 | 修正两个适配入口，共享独立审查和协调者规则 | 当前服务端没有新增参与者身份证明；指令不能替代实际执行证据 |
| Full MCP 运行绑定 Host，认证读取属于执行它的 MCP 进程 | 主协调者负责 start/read/submit/next，子代理返回提案；复用交接和审查模板记录任务、来源、候选稿与状态 | 其他会话不能冒用检查点或把文件哈希当作已认证工具证据 |
| 跨 Host 可以交换材料，但尚无当前 CLI 自动领取任务的协调器 | 提供限定范围的审查／修改提案交接，核对返回版本，记录冲突和处理结果 | 真实通信需要已获授权的可用工具或用户转交；CLI-406–408 的并发领取、写入和恢复未实现 |
| 会话恢复可能遗失研究限制和交接状态 | 新增可选的 `qiongli hooks context`，由 Host 的 SessionStart resume/compact 或 SubagentStart 调用 | 不读取研究或聊天，不调用模型，不自动保存／审批／删除；安装 Plugin 不会启用 Hook |

Hook 使用已有 Rust 标准库与 `serde_json`，没有新增依赖、常驻服务、聊天存储或
调度框架。它在 Host 发现和配置加载之前处理事件，接受至多 64 KiB 的 JSON，
仅返回固定、有限的上下文提示；未知事件返回 `{}`，错误以退出码 1 结束且不回显输入。
用户在 Host 中手动配置绝对二进制路径并完成信任审查，普通 Skills 使用不依赖它。
中英文协作页提供配置，旧 Python 指令保留在历史折叠区，历史能力矩阵不改验收结论。

Hook 的事件、配置和信任规则核对了 [Codex Hook 文档](https://learn.chatgpt.com/docs/hooks)
及 [Claude Code Hook 文档](https://code.claude.com/docs/en/hooks)；子代理入口核对了
[Codex 子代理文档](https://learn.chatgpt.com/docs/agent-configuration/subagents)。
这些是平台说明，不是本项目的安装验收。当前实现只使用 command Hook，不假定不同
Host 的 prompt、agent 或 MCP Hook 可直接互换。

### C→F 与阶段归档观察

三个真实 Codex 子任务在隔离的合成项目中执行。写作与方法审查分别收到相同的十份
原始文件，互不读取对方结果。主协调者读完返回结果后逐项核对；第三个子任务使用
原始材料、两份报告和协调记录生成阶段归档预览。没有打开私人研究、启动其他模型
CLI、接入真实 Zotero 或更改个人 Host 配置。

两份独立结果都发现了原稿把相关写成因果、把 120 名招募者当成 90 个完整观测值，
以及忽略差异失访的问题。当前调整后结果为 +0.18，95% CI [-0.12, 0.48]；
过期外部审查的 +0.42 [0.10, 0.74] 和因果建议被明确拒绝。修订保留了
CLM-01/CLM-02、DEC-01/DEC-02、study2026 和 STG-C-001。

审查还发现了合成旧记录中不符合契约的 ledger 词汇及缺失的 Graph 字段。代理提出
规范化记录和 claim map，没有编造 Graph 节点或宣称已经重建。这一观察支持继续用
Host 阅读和规范化、原生服务确定性提取的现有分工，不需要为本例新增 Graph 引擎。

阶段预览保留主要研究内容、完整候选正文、数量结果、方法限制、审查分歧与来源覆盖；
旧历史行保持原样，新 STG-F-001 行以实际保存成功为前提。十四项保留建议均为 keep，
用户选择为空。十个原始文件的字节均未改变，正式项目中没有新增总结或执行清理。
预览区分了“原稿已独立审查”“主协调者已核对新稿”与“新稿尚未独立审查”；
另一个 Host 无法参与时，没有模拟返回意见。

完整输入、四份产物（含协调记录）、观察和 24 个已核对的 Skill 资源哈希写入同目录
`2026-09-11-outcome-guided-skills-trial.json` 的 `collaboration_continuity_followup`。
试用资源对应 `d4818ab5`；`baa892dc` 随后补充协调者独占检查点推进的文字边界，
由内容检查和最终原生投影检查覆盖，不把旧试用追认为新增 MCP 行为的实测。

### 本地验证与剩余工作

- 内容、路由、链接、交接及实际工作流投影：43 项通过。
- 学术质量与能力契约：49 项中 48 项通过，1 项原有跳过；离线质量断言 12 项通过。
- CLI 回归：40 项通过；加强后的 Hook 输入边界补查通过，涵盖两个入口、有效的
  64 KiB／超限 JSON、敏感输入、无效输入、未知事件、空 PATH 和无研究写入。
- 两种原生 Plugin 回归：9 项通过，2 项真实 Host 测试按原有标记跳过。最终协调者
  文本与内容锁更新后，单独重查两种实际完整包投影，保留篡改／冲突／无运行时验证。
- MCP：6 项直接通过；Zotero 模拟服务在 `127.0.0.1:0` 监听被沙盒拒绝后，
  使用获准的本地测试权限重跑剩余一项通过。首次失败日志保留，没有访问真实 Zotero。
- 最终原生内容检查：46 项通过。协调者文字补充后的链接／连续性检查 13 项通过。
  Clippy 全目标、Desktop 特性编译和格式检查通过；文字补充不改变已验证的 Rust 控制流。
- 程序账本检查 7 项通过；文档构建与冻结源码边界检查通过。文档构建保留原有的
  代码高亮语言与 bundle 大小提示，不影响构建完成。
- 最终 CLI 导出的 433 个资源逐一与 `content/` 比较字节、长度和 SHA-256，全部一致。
  内容及应用源码均绑定 `baa892dc099762f6beefbab0fb3d3f17a01ff1bf`，pack SHA-256 为
  `ad484826415de40435e9dbd961d9ddcf35c30a83cf978202366b39bf800e2119`。

临时验证日志使用 `/private/tmp/qiongli-collab-` 前缀。仓库中的源码、内容锁、本记录
和试用 JSON 是长期追踪依据；临时文件不作长期保存承诺。没有增加测试框架或模型评测服务。

这些结果是本地源码与合成内容验证，不建立跨 Host 真实传输、参与者身份认证、Hook
真实会话送达或非 macOS 的新验收，也不能证明比 1.x 更快或质量更高。下一步先在获准
的真实 Host 中验证 Hook 恢复与一次来源绑定的跨 Host 审查交接；自动领取、独立审查
绑定和并发恢复继续按 CLI-406–408 的现有边界逐步实现。本轮本地集成，不推送或发布。


## 后续调整：安装时选择上下文 Hook（2026-09-12）

基线 `cb6685c3`，分支 `codex/install-context-hooks`。用户确认将可选 Hook 加入
`install`。本轮复用 Plugin 的预览、批准、收据和原子更新流程，没有增加全局配置
写入器、脚本运行环境或新的研究记录存储。

### 选择与边界

- `qiongli install` 的 Plugin 向导增加上下文提醒选择；首次默认关闭，更新时保留
  当前收据中的选择。`install/upgrade/update plugin --hooks context|off` 使用同一流程，
  `app plan plugin-source-install|plugin-source-update` 也支持这个参数。
- 选择写入安装计划及收据，受摘要、过期时间、文件批准和 CAS 检查约束。关闭时省略
  新字段，保持旧计划及收据的规范化表示；旧的本地导出 API 在更新时保留已有选择。
- 配置只包含恢复／压缩后的 SessionStart 和 SubagentStart，调用同包原生程序，
  超时为 5 秒。安装前展示具体配置；`--hooks off` 只移除这份 Plugin 的提醒条目，
  不更改其他 Host 或用户／项目层的手动 Hook。独立 Skills 导出不安装 Hook。
- Codex 的信任和实际事件送达仍由 Host 确认。Claude Code 的原生参数启动方式要求
  2.1.139 或更新版本，向导在文件写入前检查；未知或更旧的版本仍可选择关闭 Hook。
  纯导出命令保留不依赖 Host 的用途。

### Host 格式验证发现的问题

首次真实 Claude Code 校验虽然退出码为 0，却报告 `hooks.hooks` 是未知事件，会在
运行时忽略。这一结果没有被计为成功。修正后，Codex 内联一个 HooksFile，清单路径为
`hooks.hooks.SessionStart`；Claude Code 直接内联事件表，路径为 `hooks.SessionStart`。
同一个原生生成函数按 Host 返回这两种结构，预览、投影和验证共用它；回归检查分别
断言结构，防止再次把两个 Host 的格式混用。

规则来源是 [Codex Plugin 指南](https://developers.openai.com/plugins/build/plugins)、
[Codex 内联 Hook 解析器](https://github.com/openai/codex/blob/main/codex-rs/core-plugins/src/manifest.rs)、
[Claude Code Plugin 文档](https://code.claude.com/docs/en/plugins-reference)及
[Claude Code 2.1.139 变更记录](https://github.com/anthropics/claude-code/blob/main/CHANGELOG.md#21139)。
Hook 信任与事件协议按 [Codex Hooks](https://learn.chatgpt.com/docs/hooks)和
[Claude Code Hooks](https://code.claude.com/docs/en/hooks)处理。

### 验证结果

- 初轮原生应用单元测试：225 项通过，1 项既有容量测试跳过。最终版本的 Hook
  选择／版本检查 2 项、交互预览／取消检查 2 项通过；这些数量包含重叠检查。
- CLI 集成测试 40 项通过。修正 Host 配置后，重新运行覆盖两个 Host 的本地 Plugin
  生命周期检查，通过旧批准失效、安装／升级别名、保留选择、清单篡改拒绝和用户文件
  保留检查；生成的命令在带空格的路径与空 PATH 下返回预期上下文。
- Codex bundle 4 项通过、1 项真实 Host 测试既有跳过；Claude bundle 5 项通过、
  1 项真实 Host 测试既有跳过。结构修正后的 Claude 本地来源检查另行复跑通过，
  包含开启、重复更新、关闭、收据绑定和有签名／本地来源隔离。
- 在独立 HOME／配置目录中，Claude Code 2.1.263 对最终导出的
  `.claude-plugin/plugin.json` 校验通过，没有警告。Codex 0.153.4 的本地 marketplace
  添加、Plugin 添加和官方 JSON 清单检查通过，缓存包含两个预期 Hook 事件。
  这些操作没有修改个人 Host 配置，也没有启动模型会话或授予 Hook 信任。
- 最终 Clippy（全部 targets、警告视为错误）、Desktop feature 编译检查通过。
  派生 schema 已由 `plugin_source_contract` 示例更新，默认关闭的旧计划 fixture 未变。
  公共 schema／程序台账检查共 19 项通过。Docs 构建通过，保留既有高亮和包体积提示。

检查使用 `cargo +1.97.0`、`--offline --locked`。临时日志前缀为
`/private/tmp/qiongli-install-hooks-`，关键后缀为 `unit.log`、`integration.log`、
`cli-final.log`、`claude-final.log`、`hook-unit-final.log`、`preview-final.log`、
`host-validation-final.log`、`codex-validation.log`、`clippy-final.log`、
`desktop-final.log`、`policy.log` 和 `docs.log`。本节保留可跨机器阅读的结果，
临时日志不作为唯一证据，也不把多轮检查数量相加当成独立测试数。

### 保留的后续工作

本轮只做本地集成，不发布、不提升台账验收状态。公共 Marketplace 和有签名包保持
原来的默认配置。Skills 规范内容没有变化，因此沿用上一轮资源锁。
实际会话中的 Hook 信任、事件触发和重复提醒检查，以及 Windows／Linux 现场执行
仍需单独验证；安装清单与本地协议检查不能替代这些证据。跨 Host 审查及 CLI-406
的领取／恢复工作仍按原有计划推进。

## 后续调整：仅回复，不执行操作（2026-09-12）

来源提交：`8ed5b39baa69e46d310b6eac54745f9b7fb933a9`。
用户希望在保留现有能力的同时，明确选择只在对话中回答。原来的“小任务可以在聊天中
完成”仍要求先读取工作流，连接和连续性规则也可能触发工具，因此将选择放在根 Skill、
统一路由和 Codex 快捷入口的资源读取之前，并让原生 Hook 提醒尊重该选择。

“仅回复／不处理／no 处理／reply only／no tools”使用已经可见的对话材料；缺少原文时
说明缺口，不检索、不调用 MCP 或代理、不读写文件，也不声称搜索、审查或保存完成。
默认持续到用户明确恢复执行，也可只限定一次回复。引用文本和普通的“no”不切换模式。
原来的工作流、正式任务要求和写入审批在恢复执行后继续适用；暂停执行不代表质量门通过。

没有增加 Skill、CLI 参数、依赖或持久配置。它是行为指引，不是 Host 工具权限锁，
不能取消已运行的任务或关闭 Host 自动触发的 Hook。文档说明了这一界限，并指向既有
`install plugin --hooks off` 安装选择。没有更改个人 Host 设置或已发布版本。

本轮检查：

- `python3 -m unittest tests.test_skill_routing_probe tests.test_cross_platform_routing_grill_contract tests.test_skill_structure_lint tests.test_command_workflow_alignment tests.test_skill_resource_links`：35 项通过。
  复用既有评测器，新增 10 个中英文样例；合成轨迹检查确认“仅回复”中即使成功读取
  指引也不能计为通过。样例覆盖缺少原文、继续会话、引文不触发和明确恢复执行。
  这是离线评测约束检查，没有运行新的模型会话，不能当作模型遵循率或真实 Host 验收。
- `python3 scripts/validate_capability_contract.py` 与 skill-creator 的
  `quick_validate.py content/workflow` 通过。
- `cargo +1.97.0 fmt --all --check`、原生快捷入口元数据／路径检查（1 项）和
  `cargo +1.97.0 test -p qiongli-content --tests`（46 项）通过。
  Cargo 检查均使用 native workspace、`--offline --locked`。
- CLI 的 `context_hook_preserves_protocol_without_path_or_project_access`（1 项）
  和 `content`（5 项）通过，覆盖两个命令入口、空 PATH、错误／超限输入、无副作用
  事件以及仓库外独立二进制。Hook 输出包含仅回复边界，仍不回显输入或写入状态。
  程序台账检查 7 项通过，249 项任务的状态未改变。
- 通过 `export_marketplace_content` 在仓库外、空 PATH 下导出实际内嵌资源，433 项
  全部与 `content/` 源文件逐字节一致，收据哈希也一致。Docs 构建通过；保留既有
  语法高亮和包体积提示。临时检查记录使用 `/private/tmp/qiongli-reply-only-` 前缀，
  包括 `content.log`、`pack.log`、`hook.log`、`cli-content.log`、`wrapper.log`、
  `ledger.log`、`docs.log` 和 `export-check.json`；本节保留可独立阅读的结论。

通过既有 `update_qiongli_core_lock` 更新资源锁，仍含 433 项资源；版本保持
`2.0.0-beta.5`，来源绑定上面的提交。Content root SHA-256 为
`6119b241821413afb3d3974d8715c35cce3dbb7d14e2b7bb84255b9a918fc490`，pack SHA-256 为
`8995d4d502c9f7e1829a70f3b15a42dc88c37085ec9a63e18d6f1c192533688a`。

CLI-402 保持 active。下一步在获授权的真实 Host 会话中观察仅回复、恢复执行和 Hook
共存，分别记录模型实际调用与 Host 自动事件；继续保留跨 Host、非 macOS 和研究质量
验收缺口。本轮只做本地集成，不发布，也不把静态指引称为工具执行拦截。

## 后续调整：独立 no-qiongli 入口（2026-09-12）

用户进一步要求可单独选择的入口。来源提交为
`f3e0dbc85ae5c067950af052a0b04da18a6f3005`；新增
`content/workflow/no-qiongli/SKILL.md`，保留 `$no-qiongli`、NoQ问理和自然语言的
仅回复选择。入口本身包含完整边界，不读取主 Skill、工作流或其他指引。
讨论、开发或安装这个入口不等于选择仅回复模式。

原生 Codex／Claude Plugin 和 Marketplace 打包把同一份内容放到
`skills/no-qiongli/SKILL.md`，使 Host 可单独发现它；独立 Skills 的三个内容 profile
保留 `workflow/no-qiongli/SKILL.md`。没有新增 CLI 子命令、模式配置或工具权限锁。
路径白名单只增加该文件，旧包没有这份资源时仍保留原结构，现有收据和文件校验继续生效。

本轮本地检查：

- `python3 -m unittest tests.test_native_marketplace_plugins tests.test_skill_routing_probe tests.test_skill_resource_links`：28 项通过。
  包括两个 Host 的独立入口、旧包结构、入口缺失／篡改（连同收据一起修改）、
  中英文选择与零资源读取轨迹。离线样例约束不代表真实模型会话的遵循率。
- 新 Skill 的 `quick_validate.py`、Capability Contract 校验及 Rust 格式检查通过。
- 原生 Codex 快捷入口与 Claude 独立入口路径检查各 1 项通过，拒绝额外脚本、越界路径
  和相近目录名；`qiongli-content --tests` 46 项、程序台账检查 7 项通过。
- Codex 和 Claude 的 `complete_bundle_is_deterministic_tamper_evident_and_runtime_independent`
  各 1 项通过：完整原生包中包含独立入口，入口字节与三个 profile 一致，并保留确定性、
  篡改拒绝和空 PATH 下的运行检查。Docs 构建通过，保留既有包体积和高亮提示。

既有资源锁生成器将 434 项资源绑定到上述来源提交，版本仍为 `2.0.0-beta.5`。
Content root SHA-256：`50caa475bf0fc98175ed3076d6e6af049f6738ab474c341a9fadaa5f68c7c856`；
pack SHA-256：`dd0312d1fea8d584e8f7aed418fa74332240c9939857d1c0bba433db121bb09a`。
原生检查使用 `cargo +1.97.0`、native workspace 和 `--offline --locked`。
临时日志前缀为 `/private/tmp/qiongli-no-entry-`，包括 `python.log`、`platform.log`、
`content.log`、`bundles.log`、`ledger.log` 和 `docs.log`。

未更改个人 Host 设置或发布新包。CLI-402 保持 active；后续需要在实际安装后的 Host
中观察独立入口发现、自然语言选择、恢复执行以及自动 Hook 共存，不能把文件导出等同于
会话行为验收。

## 后续调整：期刊匹配与审稿判断（2026-09-12）

基线为 `f6487a4370ada92d8b59e69c193626605ffa5e11`；主体实现提交为
`04305045322ccc65eeae9b93f2c7e3eee91aebf7`，最终内容来源为
`2615afbe867fcbeec7a71874bcbe7bc211a2c36e`。用户要求加强两条路径：选定期刊后
调整稿件，以及根据已有稿件选择期刊，并让模型在明确边界内自行选择分析方式。

### 修改依据与取舍

沿用 A5、H5 和 Stage H，不增加期刊数据库服务、评分器或新的调度层。Stage H 统一
来源核验、适用条件、匹配、修改与审稿边界，五张任务卡保留各自职责。A5 负责目标导向
分析及调整方案，H5 先检查稿件再推荐目标；修正 `paper.md` 中 H5 错指 A5 的入口。
H1、H3、H4、Stage A、地缘经济学覆盖层、核心摘要、投稿模板及中英文指南同步。

删除“至少两／三本期刊”“3–10 个致命问题”、固定引文年限、页码、缺失值比例和
一律要求稳健性检验等规则。五张卡从 1,150 行缩至 638 行。精简的是重复流程和无依据
阈值，保留任务路径、正式质量门、独立复核要求、证据记录及 preview/approval/CAS。
模型按文章类型、方法和实际问题选择审查角度；未见资料仍是未知，不等于研究没有做。

现行官方资料核查支持以下设计（核查日期均为 2026-09-12）：

- [Nature 初投稿说明](https://www.nature.com/nature/for-authors/initial-submission)：初稿格式有弹性，返修时再给详细格式说明。因此不能把最终排版要求直接当成初稿拒收条件。
- [PLOS One 接收标准](https://journals.plos.org/plosone/s/criteria-for-publication)：一般综述与系统综述、注册报告、质性研究、阴性结果和协议稿存在类型区别。因此必须读取具体条款和例外，不能按一个宽泛标签筛除稿件。
- [Elsevier 期刊 AI 使用规定](https://www.elsevier.com/about/policies-and-standards/generative-ai-policies-for-journals)：受邀审稿的保密稿件不得上传 AI 工具，允许的辅助用途另有条件。因此作者投稿前自审与受委托保密审稿需要区分，并核查实际适用规则；不能由模型能力强或收到邀请推导出处理权限。

上述来源用于解释本轮设计，不是固化到每个期刊上的永久规则。Nature 的另一篇编辑
标准页面访问失败，没有把它当成已核查证据。运行时要求记录官方来源、核查日期、
文章类型和初投／返修阶段；本地 profile 只提供线索。相冲突、无法访问和未核实的费用、
索引、时限保持未知，不猜测录用概率。`safe` 保留兼容分类，但明确不保证录用。

修改方案需要对应稿件位置，区分表达／报告补充和需要新分析、数据或作者决定的工作。
保留结果与限制，不为迎合目标夸大创新或因果，也不把期刊不匹配等同于科学无效。
保密资料不会自动传给外部选刊工具或其他代理；仅回复模式继续优先。

### 本地检查与未通过项

- Capability Contract v2 校验通过。
- `python3 -m unittest tests.test_cross_platform_routing_grill_contract tests.test_skill_routing_probe tests.test_native_marketplace_plugins`：37 项通过。新增检查覆盖五张卡的共享引用、既有任务输出及实际生成 Plugin 中对应资源逐字节一致。
- `python3 -m unittest tests.test_skill_contract_alignment tests.test_skill_structure_lint tests.test_skill_resource_links tests.test_command_workflow_alignment tests.test_workflow_contract_doc tests.test_subject_materializer`：52 项中 49 项通过、3 项失败。
  在 `/private/tmp/qiongli-venue-baseline.Cy2eS6` 从基线 `git archive` 导出的独立副本中，
  单独重跑这三项，均复现相同失败：`test_proposal_writer_contract_is_registered`
  仍断言旧的大小写文案；两个 core Desktop materializer 预算检查均为 187 个文件超过
  180 上限。它们是本轮前已有的问题，未改测试阈值或把失败计为通过；保留为后续维护项。
- 既有路由语料增加六组中英文请求，共 36 组／72 请求。覆盖文章类型例外和投稿阶段、
  无本地档案的人文学科匹配、无法核实的费用、质性研究的报告缺口、保密审稿权限，以及
  仅回复优先。语料校验和离线评测器测试通过；这些案例没有运行新的模型会话，不能称为
  推荐准确率、审稿质量或跨模型性能验证。
- 程序台账检查 7 项通过；249 项任务状态保持不变，CLI-402 仍为 active。

- 最终内容的共享引用／任务路径及实际 Plugin 投影复查 2 项通过。
- `cargo +1.97.0 test --manifest-path packages/qiongli-native/Cargo.toml --offline --locked -p qiongli-content --tests`：46 项通过。
- 同一工具链和 workspace 的 `-p qiongli --test cli content`：5 项通过，覆盖资源查看、
  脚本中的安装预览、退役直接导出路径拒绝写入，以及仓库外、空 PATH 下的独立程序。最终复查
  改动后重新生成资源锁，并重跑这两组检查，使用最终资源字节。

- `./node_modules/.bin/vitepress build docs` 最终构建通过（15.60 秒），保留已有语法
  高亮回退和大 chunk 提示。临时日志前缀为 `/private/tmp/qiongli-venue-`，包括
  `final-projection.log`、`native-content-final.log`、`cli-content-final.log`、
  `lock.log` 和 `docs-final.log`；本节保留可独立阅读的结果和基线失败说明。

最终复查显式保留了“验证中心主张”时的证据台账义务，不只在修改主张时才记录。
经既有生成器更新的内嵌资源锁仍为 434 项、`2.0.0-beta.5`，绑定最终内容来源提交。
Content root SHA-256：`98d2c362766a60336b2034f71d0d46352c46169dbd8233dab4ed9e5034b817d1`；
pack SHA-256：`58af00b3ac474ce57b94013ceb52e153f63433265f0f6316ce63edeb54706331`。

后续需要在获授权的实际 Host 中运行这些选刊／审稿案例，记录所读来源、建议依据、
未知信息和越界行为，再评估真实效果。此前独立 no-qiongli、Hook、跨 Host 和 Graph
的现场证据缺口仍保留。本轮不更新个人安装，不发布，不提升程序验收状态。

## September 12 — Research Graph evidence, views and review decisions

来源提交：`76cdc7f333eab2c10ee6073b9d6a40ab353b87c2`，从 `39c25b81` 在
`codex/research-graph-views` 继续维护者批准的顺序。沿用同一原生投影和规范记录，
不增加图数据库、服务、运行时依赖或写入权限。

- 证据链：支持关系显示台账中的实际限制；过长或多行限制保留在来源记录，并明确提示
  读取完整记录，避免丢掉原有支持边。修复 `decision:<id>` 无法定位表格行的问题；
  重复 ID 不会得到一个貌似精确的来源定位。
- 呈现：`graph snapshot --text` 给出记录、关系、来源缺口和论点摘要；`graph view`
  向 stdout 输出自包含 HTML。搜索最多显示 100 条结果，局部图最多显示 12 个邻居和
  40 条关系，并显示截断数量。来源、限制、状态和诊断使用 DOM 文本；内嵌 JSON 转义
  script 结束标记，CSP 禁止联网。新 `graph source` 复用原来的版本绑定读取接口。
- 审稿与选刊：决策日志新增可选 `Related Claims`，兼容已有 Markdown/CSV。明确的
  `locked` 决策生成已审阅 `informs`，暂定、受阻或待重审的决定仍是 proposed。
  决策不是支持证据；缺失论点、无效状态或引用、重复决策不会生成已确认关系。
  原报告与稿件位置保留在决策记录中；没有另建审稿数据库。
- 阶段记录：用户可保存 HTML 或从页面下载同一份 JSON 快照。它们不自动刷新、不
  代替原始材料，也不执行删除或回滚。Skills、模板和中英文 CLI 文档已同步这些边界。

最终源码的检查使用 `cargo +1.97.0`、native workspace、`--offline --locked`：

| 检查 | 本地结果 |
|---|---|
| `-p qiongli-project academic_graph` | 52 通过；覆盖来源定位、稳定身份、重复/冲突、过期读取、限制文本及旧记录兼容 |
| `-p qiongli --lib` | 227 通过，1 项原有三平台容量手动观察被忽略 |
| `-p qiongli --test cli project_graph_cli` | 1 通过；HTML、摘要、来源别名、错误输出参数、旧 revision 拒绝及无索引写入 |
| `-p qiongli --test mcp_stdio full_profile_reuses_redacted_project_state_and_accepts_connected_capture` | 1 通过；保留 Full MCP 的绑定、捕获和 Graph 查询路径 |
| `-p qiongli-content --tests` | 46 通过 |
| Python：Academic Graph、证据台账、阶段交接、原生 Marketplace 投影、资源链接 | 27 通过 |
| VitePress 文档构建 | 最终构建通过，15.60 秒；保留原有高亮回退和 chunk 提示 |
| 内联脚本 `node --check` | 通过；只是开发期语法检查，运行 Graph 不依赖 Node |
| 程序台账与格式检查 | 7 项台账检查、`cargo fmt --check` 和 `git diff --check` 通过；249 项任务状态不变 |

另用合成项目复制二进制到仓库外，在空 PATH、独立 HOME/config 下执行 snapshot、view、
source。HTML 数据与快照/readiness 完全一致；支持边能回到含原文件和 Table 2 的台账行；
前后所有项目文件哈希不变。该项目有 7 个语义节点、4 条非结构关系、1 项证据缺口诊断，
状态保持 sparse，没有因为连了边而声称完整。它是已提供规范记录的合成用例，不能证明
模型能够自动、准确地从任意 PDF 提取事实，也不是与 1.x 的性能比较。

复制二进制 SHA-256：`1c9520ca2ad392109037fe3ee3ea917b86dd806b756c5454040a9a599655951d`。
资源版本保持 `2.0.0-beta.5`，434 项；资源锁绑定上述来源提交。
Content root SHA-256：`30615b153f55c0084de0089957793e5c71dc8b7dca8d957331fd6fe8c05b79af`；
pack SHA-256：`2bf43ed9ac70a98470aa62082f68714ea3ca16c1fcee59be5f56dc1941dda1d0`。
临时日志为 `/private/tmp/qiongli-graph-views-*.log`；合成页面在
`/private/tmp/qiongli-graph-view-ndg0fy0v/graph.html`，未放入发布资源。

浏览器安全策略拒绝打开本地 `file://` 页面，未尝试绕过。真实渲染、键盘交互和 JSON
下载因此尚未验收，语法/序列化检查不能替代它们。另一次包含旧版
`test_plugin_distribution_contract` 的扩大检查在 materializer 中出现错误，随后被中止；
没有得到完整结果，不计为通过。本轮改用当前 2.x 的原生 Marketplace 投影检查；此前
已记录的三项基线失败仍保留，并未在本轮重跑或放宽标准。

下一步先在允许的浏览器中验收该离线视图，再用获授权材料在真实 Host 中观察规范化、
证据追踪和审稿/选刊决策质量。阶段快照沿用既有归档规则；自动历史合并、自动删除和
新的协调服务没有必要。CLI-402 继续 active，其他现场验证与性能缺口不变。
