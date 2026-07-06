# Platform Routing

Use this mapping to keep behavior consistent across tools. Explicit workflow
commands are helpful shortcuts, but Qiongli routing does not require explicit
`$qiongli`, `/qiongli`, `/paper`, `/lit-review`, or slash-command invocation when the user is
doing academic research lifecycle work.

## Cross-Platform Trigger Contract

Route to Qiongli when the user request involves academic research artifacts,
judgment, or outputs:

- topic framing, research question narrowing, hypothesis, contribution, theory, or
  venue fit
- paper/PDF/note reading, citation handling, bibliography work, literature search,
  literature screening, extraction, synthesis, or gap mapping
- study design, variable construction, instruments, data management, robustness
  checks, preregistration, ethics, or IRB text
- manuscript/proposal/abstract/table/figure/discussion/rebuttal/submission text
- statistical interpretation, effect sizes, diagnostics, model output, robustness,
  meta-analysis, or evidence synthesis
- academic analysis code, notebooks, R/Stata/Python/Julia/MATLAB scripts, Quarto,
  or replication packages when they affect data, models, tables, figures, results,
  or reproducibility
- proofread, de-AI rewriting, citation-risk checking, reviewer response,
  peer-review simulation, fatal-flaw analysis, or academic presentations
- coursework, assignment briefs, marking rubrics, learning outcomes, capstone
  coursework, dissertations, theses, dissertation handbooks, supervisor
  feedback, viva preparation, or defense preparation when they require academic
  claim, evidence, source, method, or integrity judgment

Do not route to Qiongli for generic software feature work, generic file cleanup,
format conversion without scholarly interpretation, or prose edits with no claim,
evidence, citation, venue, method, or reviewer-risk consequence.

For timed exams, quizzes, or assessed problem sets, use concept explanation and
study support rather than drafting an answer for submission.

## Ambiguity Trigger

When an academic request is vague or the user asks for judgment, run a light
boundary/grill pass before producing a final artifact. Trigger phrases include:

- English: "I don't know how to start", "not sure", "help me decide",
  "which direction", "is this reasonable", "what should I do next"
- Chinese: "帮我判断", "不知道怎么做", "不确定", "方向不清楚", "帮我想想",
  "这样是否合理"

The light pass must inspect available artifacts first, ask one blocking academic
question, and include a recommended answer with rationale. Escalate to a deep
stage-aware grill loop when the user explicitly asks to be grilled, stress-tested,
challenged like Reviewer 2, or checked for fatal flaws.

## Natural Request Routing Examples

| User intent | Route |
|---|---|
| "Read this paper / PDF / DOI" | `B2` or `/paper-read` |
| "Find gaps / I don't know where to start" | Stage A + ambiguity grill, then `A4` or `/find-gap` |
| "Run a literature review" | `B1` or `/lit-review` |
| "Improve related work" | `B4` or `/academic-write related-work` |
| "Design the study / variables / robustness" | `C1`, `C3`, `C3_5`, or `/study-design` |
| "Interpret these results" | `F3`, `F4`, `F5`, or `stats-engine` depending on artifact |
| "Modify this analysis script / notebook" | Stage I, usually `I5 -> I6 -> I7 -> I8` for claim-supporting code |
| "Proofread / make it less AI-like" | Stage J or `/proofread` |
| "Prepare submission / cover letter" | `H1` or `/submission-prep` |
| "Reply to reviewer comments" | `H2`, `H2_5`, or `/rebuttal` |
| "Make slides" | Stage K or `/academic-present` |
| "Analyze this assignment brief / coursework rubric" | Stage L or `/coursework` |
| "Plan or revise my coursework essay/report/case analysis" | `L1-L7` or `/coursework` |
| "Plan my dissertation / thesis / capstone" | Stage M or `/dissertation` |
| "Integrate supervisor feedback / prepare viva questions" | `M4`, `M7`, or `/dissertation` |

## Claude Desktop / Claude.ai

- Use the installed `qiongli` skill as the user-visible main entry. Natural
  academic requests should route through this contract even when the user does
  not type a command.
- When a direct Desktop plugin exposes workflow command wrappers, `/qiongli` is
  the unified entry router and should delegate to the same canonical workflow
  files listed below.
- Treat the direct Desktop plugin as the preferred install when available. Treat
  focused Desktop/Web skill ZIPs as fallback skill-only packages for manual skill
  upload.
- Literature provider tools require the Qiongli Literature Provider MCPB, the
  direct plugin bundled literature MCP when visible, or another configured
  provider MCP. Do not claim `provider_connected` from skill instructions alone.
- Full orchestration tools such as `qiongli_orchestrator_route`,
  `qiongli_task_plan`, and `qiongli_task_run` require the Python-backed runtime
  MCP; Desktop skill ZIPs and the literature MCPB do not provide those tools.

## Claude Code

- `/qiongli` -> unified entry router that delegates by user intent
- `A1–A5` -> `/paper` (master router picks framing tasks)
- `A3` -> `/build-framework`
- `A4` -> `/find-gap`
- `B1` -> `/lit-review`
- `B2` -> `/paper-read`
- `B4` -> `/academic-write related-work [topic]`
- `C1–C5` -> `/study-design`
- `D1–D3` -> `/ethics-check`
- `E1–E5` -> `/synthesize`
- `F2` -> `/academic-write [section] [topic]`
- `F3` -> `/paper-write`
- `G1–G4` -> `/submission-prep` (reporting checks)
- `H1` -> `/submission-prep`
- `H2` -> `/rebuttal`
- `H3–H4` -> `/paper` (peer-review simulation, fatal-flaw)
- `I1–I8` -> `/code-build`
- `J1–J4` -> `/proofread`
- `K1–K4` -> `/academic-present`
- `L1–L7` -> `/coursework`
- `M1–M7` -> `/dissertation`
- Natural academic requests should route to the same task IDs even when the user
  does not type the command wrapper.
- If the request is ambiguous, use the Ambiguity Trigger before drafting.
- If full Qiongli MCP tools are installed and the request involves multi-agent
  coordination, independent review, handoff, strict gates, or task-run artifacts,
  call `qiongli_orchestrator_route` before running a skill-only workflow. Follow
  its returned `doctor -> task_plan -> task_run` sequence.

## Codex

- Use `$qiongli` when explicit invocation is available, but natural academic
  requests should still route to Qiongli.
- Provide: `paper_type`, `task_id`, `topic`, and optional `venue`
- Follow artifact paths from workflow contract
- For multi-agent execution, apply `standards/mcp-agent-capability-map.yaml`:
  - use `primary_agent` for draft
  - use `review_agent` for independent check
  - use `fallback_agent` when primary fails
- For proofread tasks (`J1`–`J4`), recommend `--triad` mode for iterative de-AI
- For presentation tasks (`K1`–`K4`), specify backend: `slidev`, `beamer`, or `pptx`
- For coursework tasks (`L1`–`L7`), preserve assignment brief, rubric, learning
  outcomes, word count, source rules, and AI-policy status before drafting.
- For dissertation tasks (`M1`–`M7`), preserve degree level, chapter status,
  supervisor feedback, ethics dependencies, and milestone risks.
- For academic code, prioritize estimand, data lineage, diagnostics, manuscript
  tables/figures, and reproducibility over generic software scaffolding.
- If full Qiongli MCP tools are installed and the request involves multi-agent
  coordination, independent review, handoff, strict gates, or task-run artifacts,
  call `qiongli_orchestrator_route` before running a skill-only workflow. Follow
  its returned `doctor -> task_plan -> task_run` sequence.

## CLI / npm / Python

- Slash-style commands and `qiongli task-run` remain the stable entry points.
- Generic task prompt pattern: `Task {ID} on RESEARCH/[topic] using outputs defined in the active contract.`
- Task packets from other platforms should preserve `paper_type`, `stage`,
  `task_id`, `topic`, `academic_project_type`, artifact paths, and open grill
  issues.
- Orchestrator runs should carry boundary decisions and stage handoff risks into
  downstream agents rather than resetting context.

## Orchestrator MCP Escalation

Use skill-only execution for small single-agent drafting, reading, or local
editing tasks. Escalate through full MCP when the task needs runtime
coordination:

- call `qiongli_orchestrator_route` with the user's request, platform, and any
  known `task_id`, `paper_type`, `topic`, controller, primary, reviewer, or
  verifier choices
- run `qiongli_orchestrator_doctor` before launching agents
- run `qiongli_task_plan` to inspect the task packet, quality gates, runtime
  plan, writing harness, and required artifacts
- call `qiongli_task_run` first in preview mode; only set JSON boolean
  `run_agents: true` after the doctor passes and the caller explicitly wants
  local runtime agents launched

## Worker Adapter Routing

When `task-run` includes worker orchestration, use the canonical `worker_plan`.
Adapters only change dispatch mechanics:

- `generic_prompt`: portable packet for any runtime or manual dispatch.
- `codex_subagent`: Codex native subagent dispatch when available.
- `claude_cowork`: Claude native cowork dispatch when available.

If native dispatch is unavailable, record the degradation and run the same packet
through `generic_prompt`. Do not change Task IDs, outputs, quality gates,
required skills, or MCP evidence when switching adapters.

## Portable Skill Installs

- `qiongli-workflow` portable packages must include this routing contract.
- Desktop or web skill-only installs can route and write artifacts, but they
  should not claim `provider_connected` literature search unless a provider MCP
  or MCPB is available. Platform-native search alone is `native_only`; if no
  provider MCP/MCPB and no platform-native search is available, record
  `strategy_only`.
