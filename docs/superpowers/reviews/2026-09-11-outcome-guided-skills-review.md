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
