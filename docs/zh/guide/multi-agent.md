# 多 Agent 运行

当你运行 `parallel`、`task-run`、`team-run`，或任何由 orchestrator 协调 Codex 和 Claude 的流程时，先看这一页。

## 支持的 Runtime Agent

当前 runtime agent：

- `codex`
- `claude`

Gemini CLI 不再是受支持的 runtime target。Antigravity 和 Hermes 仍然是便携 skill package 的安装面，但完整 orchestrator 只会启动本地 Codex 和 Claude agent 进程。

## 安全边界

`qiongli_task_run` 默认是 preview mode。只有同时满足以下条件时，才会启动本地 agent：

- MCP caller 发送 JSON boolean `run_agents: true`
- 本地 runtime 的 `doctor` 通过
- task packet 有明确的 `task_id`、`paper_type`、`topic` 和 artifact root

普通 planning、review 或 routing 决策应先使用 preview mode。

## 必需的本地 Runtime

完整本地执行需要：

```bash
python3
codex
claude
```

认证：

- Codex：`OPENAI_API_KEY` 或已支持的 Codex/ChatGPT 登录态
- Claude：`ANTHROPIC_API_KEY` 或已支持的 Claude Code 登录态

启动 agent 前先运行健康检查：

```bash
python3 -m bridges.orchestrator doctor --cwd .
```

## Preview-First 流程

先用 MCP route tool 或 task plan 检查任务包，再决定是否执行：

```bash
python3 -m bridges.orchestrator task-plan \
  --task-id F3 \
  --paper-type empirical \
  --topic ai-in-education \
  --cwd .

python3 -m bridges.orchestrator task-run \
  --task-id F3 \
  --paper-type empirical \
  --topic ai-in-education \
  --cwd .
```

只有 preview 可接受后，再加执行参数：

```bash
python3 -m bridges.orchestrator task-run \
  --task-id F3 \
  --paper-type empirical \
  --topic ai-in-education \
  --cwd . \
  --run-agents
```

## Runtime Routing

需要明确分工时使用这些字段：

```bash
--execution-mode solo|duo|triad
--controller codex|claude
--primary codex|claude
--reviewer codex|claude
--verifier codex|claude
--solo-role-gates strict|standard|off
```

`triad` 作为 execution-mode label 会保留，以兼容已有 task-run metadata，但它不再意味着第三个 Gemini runtime。没有可区分的第三 runtime 时，orchestrator 会记录 routing note，并复用可用的 Codex 或 Claude runtime。

## Parallel And Team Runs

需要 Codex/Claude 独立分析后再综合时，使用 `parallel`：

```bash
python3 -m bridges.orchestrator parallel \
  --prompt "Review this methods section for causal overclaiming and missing robustness checks." \
  --cwd . \
  --summarizer claude
```

fanout/fanin task packet 用 `team-run`：

```bash
python3 -m bridges.orchestrator team-run \
  --task-id H3 \
  --paper-type empirical \
  --topic acceptance-probe \
  --cwd .
```

Team run 必须明确记录 skipped 或 failed workers，不能把缺失 runtime 静默当成已完成 review。

## Worker Adapter Routing

当 `task-run` 包含 `worker_plan` 时，adapter 名称描述的是 dispatch 机制，不代表任务质量：

- `generic_prompt`：适合任意受支持 runtime 或人工分发的便携 worker packet
- `codex_subagent`：可用时走 Codex-native subagent dispatch
- `claude_cowork`：可用时走 Claude-native coworker dispatch

adapter fallback 必须写入 routing notes，保证 reviewer handoff 和 merge decision 可审计。

## 故障排除

如果 execution 被阻断：

- 运行 `doctor --cwd .`
- 确认 `codex` 和 `claude` 在 `PATH` 上
- 确认对应 auth env 或登录态存在
- 去掉 `--run-agents` 重新运行 `task-run`，先检查 preview packet
- 查看 `.qiongli/trace/` 中的 local guidance 和 routing notes
