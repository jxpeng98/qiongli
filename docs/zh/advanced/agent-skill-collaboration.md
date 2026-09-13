# Agent 协作与可选 Hook

当前 2.x 开发版让模型留在自己的 Host 中运行。穷理提供研究规范、可核对来源的
交接材料和项目工具。Plugin 包含 Skills 与 Full MCP，但安装 Plugin 本身不会
创建其他 Agent，也不会更换你选择的模型。

## 仅回复，不执行操作 {#reply-only}

beta.6 包含独立的 **`no-qiongli`** Skill。在 Codex 中可以使用
`$no-qiongli`；其他 Host 可从已安装的 Skills 中选择 `no-qiongli`。也可以自然地说
**“NoQ问理”**、**“仅回复”**、**“不处理”** 或 **“no 处理”**，例如：

> 接下来仅回复：请解释下面这段文字，不调用工具，不读取或保存文件。

这个入口直接回答，不先加载穷理主 Skill 或研究工作流。Codex 和 Claude Plugin 都将
它放在 `skills/no-qiongli/SKILL.md`，与研究入口并列；更新 Plugin 并重新加载 Skills
后才能发现新入口。独立 Skills 导出包含 `workflow/no-qiongli/SKILL.md`，可以把这个
自包含目录作为一个 Skill 交给 Host 安装。导出本身不注册 Host 或连接 MCP，CLI 中
也没有 `qiongli no-qiongli` 子命令。

模型只根据对话中已经可见的内容回答，不读取额外的 Skill 资源，不检索、不调用
MCP 或其他代理，也不检查项目、更新 Graph。材料不足时，说明缺口或请你贴出相关
内容。这个选择持续到你明确要求恢复执行；如果只想限制一次，可以说“本次仅回复”。
使用工作流快捷入口时也遵循这个选择。普通请求沿用原有路由，引文里的“no”不会切换模式。

这是一条 Skills 行为约束，不是 CLI 参数或 Host 的工具权限锁。它不会改变更高优先级
的 Host 规则、取消已经运行的任务，也不会关闭 Host 自动触发的 Hook。如需移除这份
Plugin 的上下文 Hook，请另外运行 `qiongli install plugin --hooks off`，并完成安装确认。

## 按任务选择协作方式

| 任务需要 | 建议方式 | 怎样判断完成 |
|---|---|---|
| 独立的方法或证据判断 | 让新的原生子代理阅读限定来源，先独立给出意见 | 收到真实审查结果、执行身份和阅读范围 |
| 写作后复核 | 先写作，再审查这一份具体候选稿 | 候选稿身份、审查意见和主代理的处理记录一致 |
| 可拆开的工作 | 子代理分别负责不同产物或文件 | 收齐结果，再检查整合后的内容 |
| 请另一个 Host 审查或修改 | 交接任务包，收回审查报告或修改提案 | 任务、来源版本和候选稿对应；分歧有明确记录 |

可以直接说：“请用一个独立 Agent 检查这段结果是否符合研究设计，先返回意见，
不要直接改正文。”普通小修改仍可由当前会话完成，不必固定安排几个 Agent 或几轮讨论。

`model-collaborator` 会使用当前 Host 实际提供的代理工具。主代理说明目标、允许
阅读的材料、文件分工和返回要求，再收集真实结果。角色名称或已排队任务不能证明
独立审查完成。没有可用审查者时，这项要求保持未完成；同一会话切换角色只能算自查。

是否能调用子代理，取决于运行中的 Host、配置和模型接入能力。沿用用户的模型设置。
平台能力可参考 [Codex 子代理文档](https://learn.chatgpt.com/docs/agent-configuration/subagents)
和 [Claude Code 子代理文档](https://code.claude.com/docs/en/sub-agents)。

## 与另一个 Host 协作

使用随包的 `templates/agent-handoff.md`，交接目标、允许共享的材料、实际观察到的
来源哈希或修订号、候选稿身份和允许操作。对方用 `templates/agent-review-packet.md`
返回意见，或提交候选修改。已有并获授权的通信工具可以传递任务包；也可以手动转交。
仅准备好材料时，应记录为“等待外部审查”。

收到结果后，主代理先核对它是否针对当前来源和候选稿，再根据证据处理分歧、预览修改。
过期审查需要重新核对，必要时重审。正式研究文件由一个协调者整合；其他代理使用
各自的候选文件或工作树，避免覆盖彼此的工作。交接要保留 claim/decision ID、citekey、
来源位置、方法限制及前一份阶段总结。审查意见不能直接成为 Graph 的原始支持证据。

不要转交私人聊天记录、凭据或审批令牌。当前 Full MCP 运行绑定启动它的 Host 和
已认证读取，任务包不能接管这个检查点或继承审批权限。自动领取跨 Host 任务、并发
修改正式文件及崩溃恢复仍按 ADR 0218 后续实现；目前可以先进行审查和修改提案的交接。

## 可选的上下文 Hook

Hook 适合在压缩上下文、恢复会话或启动子代理时，提醒模型重新核对研究状态。
开发版新增 `qiongli hooks context`：从标准输入接收事件 JSON，返回简短提示。
它是原生命令，不依赖 Python、Node、MCP 连接或额外模型调用，不读取项目文件和聊天
记录，也不保存总结、批准写入或强迫已经结束的任务继续运行。不启用 Hook 也可正常使用 Skills。

beta.6 已将 Hook 加入安装向导：运行 `qiongli install` 并选择 Plugin，
就可以选择是否加入上下文提醒。首次安装默认关闭，重新安装和升级时保留已有选择。
也可以直接指定：

```sh
qiongli install plugin --hooks context
qiongli install plugin --hooks off
```

确认文件修改前，CLI 会显示 Hook 的事件和命令。配置保存在 Plugin 清单中，调用同包
的原生程序，不需要另外安装脚本运行环境或启动 MCP。每个 Host 分别选择、分别安装。
`--hooks off` 只移除这份 Plugin 中的提醒配置，保留手动添加的 Host 配置。
如果之前已手动配置同一提醒，请在 Host 中检查重复条目，避免收到两次提醒。

注册完成后，Codex 仍需在 `/hooks` 或 Hook 设置中审阅并信任命令；定义变化后可能
需要重新信任。Claude Code 需要 2.1.139 或更新版本以支持 command Hook 的 `args`
字段，安装向导会在写入前检查这一点。可以在 `/hooks` 中核对 Plugin 条目。重新加载 Plugin 或开启新会话后，再通过恢复会话、
压缩上下文或启动子代理确认提醒确实送达。文件导出和注册检查不代表 Hook 已经触发。
查看导出状态可运行 `qiongli app plugin-source-status --target codex --destination /absolute/path/qiongli-next`：
`source.context_hooks` 表示经过收据校验的配置选择，`host_state` 仍为 `not-verified`。
Claude Code 改用 `--target claude`。

如果希望手动配置 Host，可以使用下面的示例。把路径替换成实际原生二进制的绝对路径，保留命令引号。Windows 使用 `qiongli.exe` 的绝对路径，并按 Host 的命令 shell
和 JSON 规则处理引号及转义。

```json
{
  "hooks": {
    "SessionStart": [
      {
        "matcher": "resume|compact",
        "hooks": [
          {
            "type": "command",
            "command": "\"/absolute/path/qiongli\" hooks context",
            "timeout": 5
          }
        ]
      }
    ],
    "SubagentStart": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "\"/absolute/path/qiongli\" hooks context",
            "timeout": 5
          }
        ]
      }
    ]
  }
}
```

Codex 将这些条目合入相应的 `.codex/hooks.json`，然后在 `/hooks` 中审阅并信任；
项目配置还需要项目受信任。Claude Code 将 `hooks` 条目合入相应的
`.claude/settings.json`。保留已有条目，避免在用户级和项目级重复添加同一提示。
具体配置和事件支持以
[Codex Hook 文档](https://learn.chatgpt.com/docs/hooks)和
[Claude Code Hook 文档](https://code.claude.com/docs/en/hooks)为准。

先运行 `qiongli hooks context --help`，旧的已发布 CLI 可能还没有这个命令。
本地检查时，向命令传入 `source: "resume"` 的 SessionStart JSON 并关闭标准输入，
应收到 `hookSpecificOutput.additionalContext`。不支持的事件返回 `{}`；无效或超过
64 KiB 的输入以退出码 1 结束，不回显输入内容。返回提示不代表审查通过或研究已保存。

这里只使用 command Hook，不能假定各 Host 的 prompt、agent、MCP Hook 行为一致。
还需在实际 Host 中确认提示是否送达；协议测试不能证明真实安装已通过验证。
[历史实测矩阵](/zh/guide/agent-host-capability-matrix)继续保留原有证据范围。

<details>
<summary>历史 1.x 增强指南</summary>

以下命令和固定角色建议属于保留的 Python 实现，不是原生 2.x 的安装或执行说明。

# Agent + Skill 协同增强指南

本指南用于在 `qiongli` 中系统增强某一能力（不仅限代码），并保持跨模型一致性。

## 1) 先定目标：增强哪一类能力

先绑定到标准任务 ID（`A1`~`I8`）：

- **选题与定位**：`A1`~`A4`
- **文献与综述**：`B1`~`B5`
- **研究设计/伦理**：`C1`~`D2`
- **证据综合**：`E1`~`E5`
- **写作**：`F1`~`F6`
- **合规与校对**：`G1`~`J4` (含去AI化改写)
- **投稿与返修**：`H1`~`H4`
- **代码与复现**：`I1`~`I8` (包含 CCG 强约束代码引擎)

只要确定目标任务，就能复用统一编排链：`plan -> mcp-evidence -> primary-agent-draft -> review-agent-check -> validator-gate`。

## 2) 协同分工原则（固定）

- **Skill**：方法与产物标准（做什么、产出什么）。
- **MCP**：证据与工具层（从哪里取证、怎么落盘）。
- **Agent**：推理与写作执行层（如何完成草稿与复核）。

建议始终保留“双 agent”结构：主执行 + 独立复核。

## 3) 如何增强某个能力（标准流程）

1. 选定目标任务（例如 `E3` 或 `I2`）。
2. 在 `standards/mcp-agent-capability-map.yaml` 中更新：
   - `required_mcp`
   - `required_skills`
   - `required_skill_cards`（由 `skill_catalog` 自动解析）
   - `primary_agent/review_agent/fallback_agent`
3. 若新增 skill：
   - 新建 `skills/<A-I_stage>/<skill-name>.md`
   - 加入 `skill_registry`、`skill_catalog` 与 `task_skill_mapping`
4. 若新增 agent runtime：
   - 在 `bridges/` 增加 bridge
   - 在 `bridges/orchestrator.py` 的 runtime 路由中接入
   - 参考现有实现：`bridges/claude_bridge.py`
5. 运行校验：
   - `python3 scripts/validate_research_standard.py --strict`

## 3.1) 外部 MCP 接入约定（命令模式）

对 `filesystem` 以外的 MCP，`task-run` 使用环境变量注入外部命令：

- 变量命名：`RESEARCH_MCP_<PROVIDER>_CMD`
- 例子：`RESEARCH_MCP_SCHOLARLY_SEARCH_CMD`

执行约定：

1. 编排器向命令 `stdin` 传入 JSON：
   - `provider`
   - `task_packet`
2. 外部命令从 `stdout` 返回 JSON：
   - `status`: `ok|warning|error|not_configured`
   - `summary`: 简要结果
   - `provenance`: 来源列表（可选）
   - `data`: 结构化附加信息（可选）

未配置变量时，状态为 `not_configured`；可用 `task-run --mcp-strict` 强制阻断执行。

## 3.2) Skills 注入约定（标准化 skill cards）

`task-run` 会从 `skill_catalog` 自动注入 `required_skill_cards`，每张 card 至少包含：

- `skill`：技能名
- `category`：技能类别（如 `evidence-synthesis`、`research-code`）
- `focus`：执行重点
- `file`：技能规范路径（`skills/*/*.md`）
- `default_outputs`：建议产物路径

可用 `task-run --skills-strict` 在技能规范文件缺失时阻断执行。

## 3.3) Profile 注入约定（人格 / 审稿风格 / 工具权限）

避免全局固定配置，使用“按运行注入”的 profile 机制：

- profile 文件：`standards/agent-profiles.example.json`
- 并发模式：
  - `parallel --profile-file ... --profile ... --summarizer-profile ...`
- 任务模式：
  - `task-run --profile-file ... --profile ...`
  - `task-run --draft-profile ... --review-profile ... --triad-profile ...`

优先级（高 -> 低）：

1. 命令行显式传参（如 `--review-profile strict-review`）
2. `task_overrides`（按 Task ID 覆盖）
3. `--profile`（本次运行默认 profile）
4. 内置 `default` profile

profile 可定义：

- `persona`
- `analysis_style` / `draft_style` / `review_style` / `summary_style` / `triad_style`
- `runtime_options`（按 agent 注入工具权限，如 Codex sandbox、Claude permission mode）
  - 推荐设置：`non_interactive: true`、`timeout_seconds`
  - 可选严格认证：`require_api_key: true`（缺失 key 时直接快速失败，避免卡在登录流程）

## 4) 按能力类型给出推荐协同模板

### A. 代码能力（`I1`~`I8`）

- **CCG 强约束执行 (I5-I8)**：借鉴 `ccg-workflow`，将代码阶段严格拆分为约束集提取(I5)->无决策规划(I6)->主端执行(I7)->侧端验收(I8)。
- 推荐 skills：`code-specification`, `code-planning`, `code-execution`, `code-review`
- 推荐 MCP：`code-runtime`, `filesystem`
- agent 组合：主执行 `codex` (执行I7)，复核 `claude` (验收I8)

### B. 系统综述能力（`B1`）

- 推荐 skills：`academic-searcher`, `paper-screener`, `paper-extractor`, `prisma-checker`, `evidence-synthesizer`, `model-collaborator`
- 推荐 MCP：`scholarly-search`, `screening-tracker`, `extraction-store`, `fulltext-retrieval`
- agent 组合：主执行 `claude`，复核 `codex`

### C. 证据综合与 Meta（`E1/E2/E3`）

- 推荐 skills：`evidence-synthesizer`, `quality-assessor`, `code-builder`
- 推荐 MCP：`stats-engine`, `extraction-store`
- agent 组合：主执行 `codex`，复核 `claude`

### D. 写作与一致性（`F3/G3`）

- 推荐 skills：`manuscript-architect`, `citation-formatter`, `reporting-checker`, `quality-assessor`
- 推荐 MCP：`metadata-registry`, `reporting-guidelines`
- agent 组合：主执行 `claude`，复核 `codex`

### E. 去痕与终审校对（`J1`~`J4`）

- **多 AI 协作迭代**：使用 `--triad` 模式进行循环去 AI 化。主执行负责重写，复核负责检查 AI 痕迹，三端负责查验科学准确性。
- 推荐 skills：`proofread-editor`, `ai-detector`, `similarity-checker`
- agent 组合：主执行 `claude`，复核 `codex`；开启 `task-run --triad` 时优先由 Antigravity 执行第三路独立审计

### F. 投稿与返修（`H1`~`H4`）

- **多角色专家互审 (H3-H4)**：在正式投稿前，通过平行调用模拟 Methodologist、Domain Expert 等苛刻审稿人进行交叉审查（H3），并执行 Desktop-reject 致命缺陷排查（H4）。
- 推荐 skills：`submission-packager`, `rebuttal-assistant`, `peer-review-simulation`, `fatal-flaw-detector`, `model-collaborator`
- 推荐 MCP：`submission-kit`, `metadata-registry`, `reporting-guidelines`
- agent 组合：主执行 `claude`，复核 `codex`

## 4.1) `team-run` 验收流程（`B1`, `H3`）

`team-run` 是当前 MVP 的 fanout/fanin 执行模式：

- `B1`：用于系统综述分片，先走 planner，失败时退回单 shard fallback
- `H3`：固定 reviewer persona（`methodologist`、`domain_expert`、`reviewer_2`）

不要只看 mock 单测，建议用 receipt 脚本记录至少一次真实运行：

```bash
python3 scripts/capture_team_run_acceptance.py \
  --task-id B1 \
  --paper-type systematic-review \
  --topic acceptance-probe \
  --cwd . \
  --max-units 2 \
  --receipt tooling/release/acceptance/team-run-b1-local-receipt.md

python3 scripts/capture_team_run_acceptance.py \
  --task-id H3 \
  --paper-type empirical \
  --topic acceptance-probe \
  --cwd . \
  --receipt tooling/release/acceptance/team-run-h3-local-receipt.md
```

判读规则：

- `Barrier Status: ok`：所有 shard 都进入 merge/review。
- `Barrier Status: degraded`：成功 shard 足以继续 merge，但必须保留 receipt 并人工检查缺失 shard 的说明。
- `Barrier Status: blocked`：只能算环境证据，不能算功能已绿；必须保留精确的阻塞观察。

当前仓库内的本地 receipt 已记录两类真实阻塞：

- `B1`：scholarly-search 的外网解析失败，且外部 MCP overlay 未配置。
- `H3`：review runtime 不在 `PATH` 中，同时 Codex worker 没有产出可消费的 agent message。

## 5) 运行入口（统一）

建议先做预检：

```bash
python -m bridges.orchestrator doctor --cwd ./project
```

使用 `task-run` 按任务执行并自动注入 `required_skills + required_skill_cards`：

```bash
python -m bridges.orchestrator task-run \
  --task-id F3 \
  --paper-type empirical \
  --topic ai-in-education \
  --cwd ./project \
  --context "Target venue style and strict claim-evidence alignment" \
  --mcp-strict \
  --skills-strict \
  --triad
```

`--triad` 会保留主执行 + 复核之后的独立审计合同。Antigravity 会替代之前的 Gemini 通道，作为优先的第三路 runtime；如果它不可用，orchestrator 会记录 routing fallback，并复用可用 runtime 完成审计。

并发分析模式（不限定 Task ID）：

```bash
python -m bridges.orchestrator parallel \
  --prompt "审查当前研究方案的风险、证据缺口与改进顺序" \
  --cwd ./project \
  --summarizer claude
```

该模式默认 Codex/Claude/Antigravity 并发，并在并发后执行总结分析；不可用 worker 会被记录为 skipped 或 failed，不能静默算作已完成 review。

## 6) 引入外部 agent 还是自建 agent？

推荐混合策略：

- **外部 agent/runtime**：负责通用能力上限（代码、推理、长文本）。
- **本地映射与约束**：负责研究场景一致性与可控性（Task ID、质量门、产物路径、技能约束）。

也就是：把“能力”交给外部，把“标准”留在本地。

</details>
