# Multi-Agent Runtime

Use this guide when you run `parallel`, `task-run`, `team-run`, or any workflow that coordinates Codex and Claude under the orchestrator.

## Supported Runtime Agents

Current runtime agents:

- `codex`
- `claude`

Gemini CLI is no longer a supported runtime target. Antigravity and Hermes remain install surfaces for the portable skill package, but the full orchestrator launches only Codex and Claude local agent processes.

## Safety Boundary

`qiongli_task_run` defaults to preview mode. It launches local agents only when all of these are true:

- the MCP caller sends JSON boolean `run_agents: true`
- `doctor` passes for the local runtime
- the task packet has a concrete `task_id`, `paper_type`, `topic`, and artifact root

Preview mode should be used first for normal planning, review, or routing decisions.

## Required Local Runtime

Full local execution needs:

```bash
python3
codex
claude
```

Authentication:

- Codex: `OPENAI_API_KEY` or an existing supported Codex/ChatGPT login
- Claude: `ANTHROPIC_API_KEY` or an existing supported Claude Code login

Run a health check before launching agents:

```bash
python3 -m bridges.orchestrator doctor --cwd .
```

## Preview-First Flow

Use the MCP route tool, then inspect the generated task plan before execution:

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

Only add execution flags after the preview is acceptable:

```bash
python3 -m bridges.orchestrator task-run \
  --task-id F3 \
  --paper-type empirical \
  --topic ai-in-education \
  --cwd . \
  --run-agents
```

## Runtime Routing

Use these fields when you need explicit ownership:

```bash
--execution-mode solo|duo|triad
--controller codex|claude
--primary codex|claude
--reviewer codex|claude
--verifier codex|claude
--solo-role-gates strict|standard|off
```

`triad` is retained as an execution-mode label for compatibility with existing task-run metadata, but it no longer implies a third Gemini runtime. When no distinct runtime remains available, the orchestrator records a routing note and reuses an available Codex or Claude runtime.

## Parallel And Team Runs

Use `parallel` when you want independent Codex/Claude analysis followed by synthesis:

```bash
python3 -m bridges.orchestrator parallel \
  --prompt "Review this methods section for causal overclaiming and missing robustness checks." \
  --cwd . \
  --summarizer claude
```

Use `team-run` for fanout/fanin task packets:

```bash
python3 -m bridges.orchestrator team-run \
  --task-id H3 \
  --paper-type empirical \
  --topic acceptance-probe \
  --cwd .
```

Team runs should record skipped or failed workers explicitly and should not silently treat a missing runtime as a completed review.

## Worker Adapter Routing

When `task-run` includes a `worker_plan`, adapter names describe dispatch mechanics, not task quality:

- `generic_prompt`: portable worker packet for any supported runtime or manual dispatch
- `codex_subagent`: Codex-native subagent dispatch when available
- `claude_cowork`: Claude-native coworker dispatch when available

Adapter fallback must be recorded in routing notes so reviewer handoff and merge decisions remain auditable.

## Troubleshooting

If execution is blocked:

- run `doctor --cwd .`
- confirm `codex` and `claude` are on `PATH`
- confirm the relevant auth environment or logged-in session exists
- rerun `task-run` without `--run-agents` to inspect the preview packet
- check `.qiongli/trace/` for local guidance and routing notes
