# Agents, collaboration and optional hooks

Current 2.x development keeps the model in its Host. Qiongli supplies research
contracts, source-bound handoffs and project tools. Installing the Plugin gives
you Skills and Full MCP; it does not create another agent or change your model.

## Choose the smallest useful collaboration

| Need | Use | Completion evidence |
|---|---|---|
| Independent methodological judgment | A fresh native subagent reviewing the same bounded sources | Actual reviewer result, execution identity and sources read |
| Draft plus review | A writer, then a reviewer of the exact candidate | Candidate identity, findings and coordinator's reconciliation |
| Independent components | Native subagents with separate output/file scopes | Returned components and an integration check |
| Another Host's review or edits | A portable handoff and returned review/candidate | Matching task, source version and candidate; unresolved conflicts remain visible |

Ask naturally, for example: “Use an independent agent to check whether this
results section respects our study design. Return findings before changing the
manuscript.” Small edits can stay in one conversation. There is no required
agent count or discussion round count.

Qiongli's `model-collaborator` guidance uses the Host's actual delegation tools.
The coordinator gives each participant an outcome, source scope, file ownership
and return format, then collects real results. A role label or a queued task
is not an independent completed review. If no reviewer can run, that requirement
stays open; same-conversation self-review is labelled accordingly.

Native tool availability depends on the running Host, configuration and model
integration. Preserve the user's model settings. See the official
[Codex subagent guide](https://learn.chatgpt.com/docs/agent-configuration/subagents)
and [Claude Code subagent guide](https://code.claude.com/docs/en/sub-agents).

## Work with another Host

Use the bundled `templates/agent-handoff.md` to send the requested outcome,
permitted source excerpts/files, observed source hashes or revision, exact
candidate and allowed actions. The other Host returns
`templates/agent-review-packet.md` or candidate edits. Use an available,
authorized communication tool, or transfer the packet manually. Prepared packets
remain **awaiting external review** until a result returns.

The coordinator compares the returned source/candidate identity with the current
project, resolves disagreements against evidence and previews any accepted
changes. Stale reviews need reconciliation or a new review. One coordinator owns
canonical research writes; separate candidate files or worktrees avoid concurrent
overwrites. Keep claim/decision IDs, citekeys, source anchors, design limitations
and previous stage-summary links across the exchange. A review is not new
primary evidence for the Graph.

Do not transfer private conversations, credentials or approval tokens. Existing
Full MCP runs are bound to their starting Host and authenticated reads; a packet
does not transfer that checkpoint or its authority. Automatic cross-Host task
claiming, concurrent canonical writes and crash recovery remain later work under
ADR 0218. Portable review/edit exchange is available without those features.

## Optional context hooks

A hook can remind the model what to recheck after compaction, resume or child-agent
startup. The development CLI provides `qiongli hooks context`: a native command
that reads an event JSON object from stdin and returns a brief context reminder.
It needs no Python, Node, MCP connection or model call. It does not read project
files or transcripts, save summaries, approve writes or keep a completed task
running. Hooks are optional; ordinary Skill routing remains sufficient.

In the development build after beta.5, `qiongli install` offers context hooks
when you choose Plugin. They default to off on first install; later installs and
upgrades preserve your choice. You can also choose explicitly:

```sh
qiongli install plugin --hooks context
qiongli install plugin --hooks off
```

The preview shows the events and command before you confirm file changes. The
configuration lives in the Plugin manifest and calls its bundled native binary;
no separate script runtime or MCP service is needed. Each Host has its own
choice and installation. `--hooks off` removes this Plugin's reminder entries.
It leaves manual Host settings intact. Avoid enabling the same reminder through
both the Plugin and a manual configuration.

After registration, Codex still requires you to review and trust the commands in
`/hooks` or its Hook settings. A changed definition can require trust again.
Claude Code needs 2.1.139 or newer for command hooks with `args`; the guide
checks this before writing files. Check the Plugin entries in `/hooks`. Reload the Plugin or start a new session, then
verify that resume/compaction or child startup delivers a reminder. Export and
registration checks do not prove event delivery. To inspect exported configuration,
use `qiongli app plugin-source-status --target codex --destination /absolute/path/qiongli-next`:
`source.context_hooks` reports the receipt-verified choice; `host_state` remains
`not-verified`. Use `--target claude` for Claude Code.

If you prefer manual Host configuration, the following example uses an absolute
native binary path; replace it with yours, preserving
shell quoting. On Windows use the absolute `qiongli.exe` path and quote/escape it
for the Host's command shell and JSON.

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

For Codex, merge these entries into the appropriate `.codex/hooks.json` and
review/trust them in `/hooks`; project-local configuration also needs project
trust. For Claude Code, merge the `hooks` entries into the appropriate
`.claude/settings.json`. Keep other entries intact and avoid registering this
same reminder at multiple scopes. Configuration and event support follow the Host's own
[Codex hooks documentation](https://learn.chatgpt.com/docs/hooks) or
[Claude Code hooks documentation](https://code.claude.com/docs/en/hooks).

Check `qiongli hooks context --help` first: a previously released CLI may not
contain this command. To inspect the protocol locally, pipe a SessionStart
object with `source: "resume"` into the command and close stdin. It returns
`hookSpecificOutput.additionalContext`; unsupported events return `{}`. Invalid
or over-64-KiB input exits 1 without printing its contents. The Hook result is
only a reminder, never evidence of an independent review or saved research.

This implementation uses command hooks only. Do not assume all Hosts implement
the same prompt/agent/MCP hook handlers. Confirm delivery in the actual Host;
protocol tests alone do not qualify a live installation. The historical
[observed capability matrix](/guide/agent-host-capability-matrix) keeps its
original evidence scope.

<details>
<summary>Historical 1.x enhancement guide</summary>

The following commands and fixed-role recommendations describe the retained
Python implementation. They are not native 2.x setup or execution instructions.

# Agent + Skill Collaboration Enhancement Guide

This guide is designed to systematically enhance a specific capability (not limited to code) within `qiongli` while maintaining cross-model consistency.

For distribution boundaries, see [Plugin-First Architecture](/advanced/plugin-first-architecture). For skill structure requirements, see [Skill Quality Contract](/maintainer/skill-quality-contract).

## 1) Define the Goal: Which Capability to Enhance

First, bind the capability to a standard Task ID (`A1`~`I8`):

- **Topic Selection & Positioning**: `A1`~`A4`
- **Literature & Review**: `B1`~`B5`
- **Study Design / Ethics**: `C1`~`D2`
- **Evidence Synthesis**: `E1`~`E5`
- **Drafting**: `F1`~`F6`
- **Compliance & Proofread**: `G1`~`J4` (Including de-AI and humanization)
- **Submission & Rebuttal**: `H1`~`H4`
- **Code & Replication**: `I1`~`I8` (Includes CCG strict-constraint code engine)

Once the target task is determined, use the matching orchestration chain:

- Standard: `plan -> mcp-evidence -> primary-agent-draft -> review-agent-check -> validator-gate`
- Worker-enabled: `plan -> mcp-evidence -> worker_plan -> worker-execute -> merge -> final-review -> validator-gate`

`worker_plan` is platform-neutral: Codex maps it to `codex_subagent`, Claude
maps it to `claude_cowork`, and any runtime can fall back to `generic_prompt`.

## 2) Division of Labor Principles (Fixed)

- **Skill**: Methodology and artifact standards (What to do, what to produce).
- **MCP**: Evidence and tooling layer (Where to fetch evidence, how to save).
- **Agent**: Reasoning and execution layer (How to complete the draft and review).

It is recommended to always retain the "dual agent" structure: Primary execution + Independent review.

## 3) How to Enhance a Capability (Standard Workflow)

1. Select the target task (e.g., `E3` or `I2`).
2. Update the following in `standards/mcp-agent-capability-map.yaml`:
   - `required_mcp`
   - `required_skills`
   - `required_skill_cards` (Automatically parsed by `skill_catalog`)
   - `primary_agent/review_agent/fallback_agent`
3. If adding a new skill:
   - Create `skills/<A-I_stage>/<skill-name>.md`
   - Add it to `skill_registry`, `skill_catalog`, and `task_skill_mapping`
4. If adding a new agent runtime:
   - Add a bridge in `bridges/`
   - Integrate it into the runtime router in `bridges/orchestrator.py`
   - Refer to existing implementations: `bridges/claude_bridge.py`
5. Run validations:
   - `python3 scripts/validate_research_standard.py --strict`

## 3.1) External MCP Integration Conventions (Command Mode)

For MCPs other than `filesystem`, `task-run` uses environment variables to inject external commands:

- Variable naming convention: `RESEARCH_MCP_<PROVIDER>_CMD`
- Example: `RESEARCH_MCP_SCHOLARLY_SEARCH_CMD`

Execution Protocol:

1. The orchestrator passes JSON to the command's `stdin`:
   - `provider`
   - `task_packet`
2. The external command returns JSON via `stdout`:
   - `status`: `ok|warning|error|not_configured`
   - `summary`: Brief summary string
   - `provenance`: List of sources (optional)
   - `data`: Structured additional info (optional)

If the variable is not configured, the status is `not_configured`; you can use `task-run --mcp-strict` to forcefully block execution.

## 3.2) Skill Injection Conventions (Standardized Skill Cards)

`task-run` will automatically inject `required_skill_cards` from `skill_catalog`. Each card contains at least:

- `skill`: Skill name
- `category`: Skill category (e.g., `evidence-synthesis`, `research-code`)
- `focus`: Primary execution focus
- `file`: Path to the skill specification (`skills/*/*.md`)
- `default_outputs`: Recommended artifact output paths

Use `task-run --skills-strict` to block execution if the skill specification files are missing.

## 3.3) Profile Injection Conventions (Persona / Style / Tool Permissions)

Avoid global fixed configurations; use the "per-run injected" profile mechanism:

- Profile file: `standards/agent-profiles.example.json`
- Parallel mode:
  - `parallel --profile-file ... --profile ... --summarizer-profile ...`
- Task mode:
  - `task-run --profile-file ... --profile ...`
  - `task-run --draft-profile ... --review-profile ... --triad-profile ...`

Priority (High -> Low):

1. Command-line explicit parameters (e.g., `--review-profile strict-review`)
2. `task_overrides` (Override by Task ID)
3. `--profile` (Default profile for this run)
4. Built-in `default` profile

A profile can define:

- `persona`
- `analysis_style` / `draft_style` / `review_style` / `summary_style` / `triad_style`
- `runtime_options` (Agent-specific tool permissions, e.g., Codex sandbox and Claude permission mode)
  - Recommended settings: `non_interactive: true`, `timeout_seconds`
  - Optional strict auth: `require_api_key: true` (Fails fast if key is missing, avoiding getting stuck in login flows)

## 4) Recommended Collaboration Templates by Capability Type

### A. Code Capabilities (`I1`~`I8`)

- **CCG Strict Execution Constraint (I5-I8)**: Drawing from `ccg-workflow`, the code phase is strictly split into Constraint Extraction (I5) -> Decision-free Planning (I6) -> Primary Execution (I7) -> Side-channel Validation (I8).
- Recommended skills: `code-specification`, `code-planning`, `code-execution`, `code-review`
- Recommended MCPs: `code-runtime`, `filesystem`
- Agent combination: Primary `codex` (executes I7), Review `claude` (validates I8)

### B. Systematic Review Capabilities (`B1`)

- Recommended skills: `academic-searcher`, `paper-screener`, `paper-extractor`, `prisma-checker`, `evidence-synthesizer`, `model-collaborator`
- Recommended MCPs: `scholarly-search`, `screening-tracker`, `extraction-store`, `fulltext-retrieval`
- Agent combination: Primary `claude`, Review `codex`

### C. Evidence Synthesis and Meta-Analysis (`E1/E2/E3`)

- Recommended skills: `evidence-synthesizer`, `quality-assessor`, `code-builder`
- Recommended MCPs: `stats-engine`, `extraction-store`
- Agent combination: Primary `codex`, Review `claude`

### D. Drafting and Consistency (`F3/G3`)

- Recommended skills: `manuscript-architect`, `citation-formatter`, `reporting-checker`, `quality-assessor`
- Recommended MCPs: `metadata-registry`, `reporting-guidelines`
- Agent combination: Primary `claude`, Review `codex`

### E. Proofread & De-AI (`J1`~`J4`)

- **Multi-AI Triad Iteration**: Use triad mode to perform iterative de-AI. Drafter rewrites text, Reviewer checks for AI fingerprints, and Auditor ensures scientific accuracy.
- Recommended skills: `proofread-editor`, `ai-detector`, `similarity-checker`
- Agent combination: Primary `claude`, Review `codex`, Antigravity triad audit when `task-run --triad` is enabled

### F. Submission and Rebuttal (`H1`~`H4`)

- **Multi-Role Expert Cross-Review (H3-H4)**: Before final submission, use parallel invocations to simulate harsh reviewers (Methodologist, Domain Expert) across a cross-review (H3) and execute a Fatal Flaw Desktop-reject scan (H4).
- Recommended skills: `submission-packager`, `rebuttal-assistant`, `peer-review-simulation`, `fatal-flaw-detector`, `model-collaborator`
- Recommended MCPs: `submission-kit`, `metadata-registry`, `reporting-guidelines`
- Agent combination: Primary `claude`, Review `codex`

## 4.1) `team-run` Acceptance Workflow (`B1`, `H3`)

`team-run` is the fanout/fanin execution mode for the current MVP tasks:

- `B1`: planner or fallback partitioning for systematic-review shards
- `H3`: fixed reviewer personas (`methodologist`, `domain_expert`, `reviewer_2`)

Use the receipt helper to capture one real run instead of relying only on mock tests:

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

Interpretation rules:

- `Barrier Status: ok`: all shards reached merge/review.
- `Barrier Status: degraded`: enough shards succeeded to merge; keep the receipt and inspect missing shard notes before trusting the merged output.
- `Barrier Status: blocked`: treat the receipt as environment evidence, not product acceptance. Keep the exact blocking observations.

Current local receipts in this repo show two concrete block classes:

- `B1`: outbound scholarly-search resolution failed and optional external MCP overlays were not configured.
- `H3`: the review runtime was absent from `PATH`, and the Codex worker produced no consumable agent message.

## 5) Execution Entry Points (Unified)

It is recommended to run a pre-flight check first:

```bash
python -m bridges.orchestrator doctor --cwd ./project
```

### Auto vs Interactive Mode
By default, the orchestrator runs in **Auto Mode**, executing all agents and synthesis steps seamlessly.
To enable **Interactive Step-by-Step Mode** (pauses for human `Y/n/p/q` confirmation before invoking any agent), append `-i` or `--interactive` to any command. Note: inside an AI chat terminal (like Claude Code), simply ask the AI to do it "step by step" via natural language instead of using `-i`.

Use `task-run` to execute by task and automatically inject `required_skills + required_skill_cards`:

```bash
python -m bridges.orchestrator task-run \
  --task-id F3 \
  --paper-type empirical \
  --topic ai-in-education \
  --cwd ./project \
  --context "Target venue style and strict claim-evidence alignment" \
  --mcp-strict \
  --skills-strict \
  --triad \
  -i  # (Optional) Interactive step-by-step
```

`--triad` preserves the independent-audit contract after the primary draft and review. Antigravity replaces the previous Gemini lane as the preferred distinct third runtime; if it is unavailable, the orchestrator records the routing fallback and reuses an available runtime for the audit.

Parallel Analysis Mode (Not restricted by Task ID):

```bash
python -m bridges.orchestrator parallel \
  --prompt "Review the risks, evidence gaps, and improvement priorities for the current study design" \
  --cwd ./project \
  --summarizer claude \
  -i
```

This mode defaults to concurrent Codex/Claude/Antigravity execution followed by a synthesis analysis; it records skipped or unavailable workers instead of treating them as completed reviews.

## 6) External Agents vs. Custom Agents?

A hybrid strategy is recommended:

- **External agent/runtime**: Handles the ceiling of general capabilities (Code generation, reasoning, long-context text).
- **Local mappings and constraints**: Ensures consistency and controllability for the specific research scenario (Task ID, Quality Gates, Artifact paths, Skill constraints).

In other words: Outsource the "capability" to external agents, but keep the "standards" locally.

</details>
