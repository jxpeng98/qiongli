# Controller-Agnostic Orchestration Audit

## Current Runtime Surfaces

- `bridges/base_bridge.py` defines `ModelType` for `codex`, `claude`, and `gemini`, plus a shared `BridgeResponse` with `success`, `model`, `session_id`, `content`, `error`, and `raw_messages`. It also centralizes non-interactive execution, timeout handling, CLI availability checks, and optional auth env checks.
- `bridges/codex_bridge.py` wraps `codex exec --json`, supports sandbox selection, `--cd`, optional model, image inputs, and session resume via `resume <session_id>`.
- `bridges/claude_bridge.py` wraps `claude -p --output-format stream-json`, supports optional model, permission mode, and session resume via `--resume`.
- `bridges/orchestrator.py` exposes legacy runtime modes `parallel`, `chain`, `role`, and `single`, plus contract-aware `task-run`, `team-run`, `task-plan`, `doctor`, and strict Stage-I `code-build` routing.
- `bridges/orchestrator.py` can dispatch any runtime agent through `_execute_runtime_agent()` and preflight Codex, Claude, and Gemini through `_runtime_preflight_error()`.

## Current Contract Coverage

- `standards/mcp-agent-capability-map.yaml` has a task-level `coordination_contract` requiring task packet fields such as `task_id`, `paper_type`, `topic`, `required_outputs`, `required_skills`, `required_skill_cards`, and `quality_gates`.
- The same map has `agent_registry` entries for `codex`, `claude`, and `gemini`, functional role ownership through `functional_agent_registry`, and per-task `primary_agent`, `review_agent`, `fallback_agent`, `required_mcp`, and `quality_gates`.
- `standards/agent-profiles.example.json` defines runtime options per agent, including Codex sandbox, Claude permission mode, Gemini transport, non-interactive settings, and timeout values.
- `bridges/orchestrator.py` builds task packets, injects skill/MCP context, resolves draft/review/fallback runtimes, runs revision loops, optionally runs triad audit, and checks required output files with `_validator_gate()`.
- This branch now has an initial `standards/agent-run-contract.yaml` plus run/review/handoff templates, but the orchestrator and strict validators do not yet consume that contract during real task execution.

## Codex Bridge Gaps

- `bridges/codex_bridge.py` returns a generic `BridgeResponse`, but it does not emit controller-agnostic run metadata, artifact read/write lists, verification status, or declared write-set evidence.
- Codex sandbox configuration exists, but there is no bridge-level mapping from orchestration mode to sandbox/write permissions for `solo_codex`, duo, or triad runs.
- Codex parsing requires a `thread_id` and agent messages, but it does not normalize Codex events into a shared agent run packet or handoff/review packet.
- There is no Codex-specific solo role gate for writing tasks, citation risk, evidence ledger checks, or claim calibration.

## Claude Bridge Gaps

- `bridges/claude_bridge.py` normalizes several Claude JSON output shapes into `BridgeResponse`, but it does not emit shared run metadata, artifacts touched, blocking issues, warnings, or verification status.
- Claude permission mode is profile-driven, but there is no mode-aware contract that distinguishes Claude as controller, primary draft agent, reviewer, fallback, or solo runtime.
- Claude output parsing accepts plain text fallback, which helps robustness, but there is no schema validation for review verdicts, required revisions, or handoff fields.
- There is no Claude-specific solo role gate for code tasks, implementation intent, declared write set, failing-test-first evidence, command evidence, or rollback notes.

## Orchestrator Mode Gaps

- `bridges/orchestrator.py` has runtime modes and a task-run pipeline, but current CLI flags do not expose `--execution-mode`, `--controller`, `--primary-agent`, `--review-agent`, or `--solo-role-gates`.
- `task-run` chooses draft/review/fallback agents from `standards/mcp-agent-capability-map.yaml`; it cannot currently force `solo_codex`, `solo_claude`, `duo`, or `triad` through a canonical controller-agnostic enum.
- `single` mode executes one model directly, but it is not connected to the Task-ID contract, MCP/skill collection, validator gate, or solo self-review policy.
- `parallel`, `chain`, and `role` modes operate as collaboration helpers and do not produce structured handoff, review, disagreement, or adjudication artifacts.
- `team-run` has `fanout_merge` settings in `standards/mcp-agent-capability-map.yaml`, but the execution mode is scoped to team fanout rather than the planned controller-agnostic solo/duo/triad contract.

## Solo Mode Gaps

- There is no `standards/solo-role-policy.yaml` or equivalent policy in the inspected files.
- Solo execution today means legacy `single` mode or degraded task-run fallback, not a first-class `solo_codex`, `solo_claude`, or `solo_gemini` mode.
- Solo runs do not require role-specific self-review templates, implementation intent, writing claim maps, quality gate reports, or handoff summaries.
- Current `task-run` excludes the draft runtime from review routing, but intentional single-runtime execution is still not modeled as a first-class solo mode with explicit controller, reviewer, and self-review semantics.

## Validator Coverage Gaps

- `_validator_gate()` in `bridges/orchestrator.py` checks only whether required output paths exist under `RESEARCH/[topic]/`; it does not validate run packet fields, review packet fields, handoff metadata, controller identity, or execution mode enums.
- `--mcp-strict` and `--skills-strict` validate provider/skill availability, but not solo role gates or controller/mode compatibility.
- `scripts/validate_research_standard.py` is referenced by the broader standards workflow, but the inspected runtime files do not wire it into controller-agnostic run/review/handoff contract validation.
- The initial agent-run contract test enforces packet fields, but no runtime validator yet checks real Codex-only and Claude-only run artifacts for equivalent evidence, blocking issue reporting, confidence, and verification status.

## P0 Implementation Recommendations

- Integrate `standards/agent-run-contract.yaml` as the source of truth for execution modes, runtime agents, required run/review/handoff fields, and verification statuses in orchestrator runtime output and strict validation.
- Add the remaining machine-readable and human-readable templates for disagreement/adjudication, solo task packets, solo self-review, implementation intent, writing claim maps, and quality gate reports.
- Extend `bridges/orchestrator.py` CLI and task packet construction to carry `execution_mode`, `controller`, `primary_agent`, selected reviewer, session IDs, artifact read/write declarations, warnings, blockers, confidence, and validator status.
- Promote solo execution to first-class task-run behavior: `solo_codex`, `solo_claude`, and `solo_gemini` should use the normal Task-ID contract, MCP/skill context, output validator, and role-specific self-review gates.
- Add offline validators and tests before behavior changes so bridge outputs can be exercised with fake responses without invoking real Codex, Claude, or Gemini CLIs.
