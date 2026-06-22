# Controller Modes

Controller mode records who owns orchestration, review, and verification during `task-run`. Under the resolved execution mode, `--primary` and `--reviewer` also route the draft and review runtimes while the capability map continues to provide task requirements and fallback routes.

## Flags

Use these flags with `python3 -m bridges.orchestrator task-run`:

| Flag | Values | Meaning |
| --- | --- | --- |
| `--execution-mode` | `solo`, `duo`, `triad` | Declares the collaboration shape. Defaults to `duo`, or `triad` when `--triad` is set. |
| `--controller` | `codex`, `claude`, `antigravity` | Declares the runtime agent accountable for orchestration metadata. Defaults to `codex`. |
| `--primary` | `codex`, `claude`, `antigravity` | Declares the primary runtime agent for the task packet. |
| `--reviewer` | `codex`, `claude`, `antigravity` | Declares the review runtime agent for the task packet. |
| `--verifier` | `codex`, `claude`, `antigravity` | Records the verification agent for audit metadata. |
| `--solo-role-gates` | `strict`, `standard`, `off` | Sets solo-mode gate strictness. Defaults to `standard`. |

Invalid values are rejected by the CLI parser. Use `--mcp-strict` and `--skills-strict` when controller declarations must also fail fast on missing providers or skill specs.

## Runtime Override Semantics

`--primary` and `--reviewer` override the draft and review runtime agents for `task-run` under the resolved execution mode; `duo` is the default when no mode is supplied. The capability map still supplies required skills, MCP requirements, output contracts, quality gates, and fallback routes.

- `solo`: `--primary` controls the draft runtime; reviewer is not forced unless the task still requires an independent review gate.
- `duo`: `--primary` controls draft and `--reviewer` controls review.
- `triad`: `--primary` controls draft, `--reviewer` controls review, and `--verifier` records the preferred verification runtime. The third audit chooses a distinct available runtime outside draft/review when possible; with the default Codex/Claude duo, that distinct runtime is Antigravity.

Fallback behavior is explicit: if a declared runtime fails preflight, the result includes `Runtime agent '<agent>' unavailable` and `Runtime routed agent '<from>' to '<to>'`.

## Worker Delegation Semantics

Controller mode and worker orchestration are separate layers. Controller mode
chooses runtime accountability for the task-run; worker orchestration decides
whether that controller creates a limited `worker_plan` before merge and final
review.

Use `--worker-mode`, `--worker-adapter`, and `--max-workers` to create and limit
the `worker_plan`. The canonical adapter names are `generic_prompt`,
`codex_subagent`, and `claude_cowork`.

Native Codex subagent or Claude cowork execution is an adapter detail. Task IDs,
skills, MCP evidence, artifacts, and quality gates remain unchanged when the
adapter changes or degrades.

## Codex-Primary Mode

Use Codex-primary mode when implementation control, reproducibility, or diff inspection is the main risk.

```bash
python3 -m bridges.orchestrator task-run \
  --task-id I7 \
  --paper-type methods \
  --topic llm-bias \
  --cwd . \
  --execution-mode duo \
  --controller codex \
  --primary codex \
  --reviewer claude \
  --mcp-strict \
  --skills-strict
```

Expected gates:

- Codex owns command evidence, tests, and write-set discipline.
- Claude reviews research framing, hidden assumptions, prose quality, and reviewer-facing risk.
- Blocking disagreements are recorded before downstream use.

## Claude-Primary Mode

Use Claude-primary mode when scholarly argument, manuscript structure, rebuttal strategy, or peer-review simulation is the main risk.

```bash
python3 -m bridges.orchestrator task-run \
  --task-id F3 \
  --paper-type empirical \
  --topic ai-in-education \
  --cwd . \
  --execution-mode duo \
  --controller claude \
  --primary claude \
  --reviewer codex \
  --mcp-strict \
  --skills-strict
```

Expected gates:

- Claude owns argument continuity, citation risk, and reviewer self-critique.
- Codex reviews artifact paths, contract conformance, reproducibility notes, and implementation assumptions.
- Claims without evidence become gap notes, not invented support.

## Strict Validation

For controller-aware runs, strict validation means:

- `--execution-mode`, `--controller`, `--primary`, `--reviewer`, `--verifier`, and `--solo-role-gates` must use supported values.
- `--mcp-strict` blocks when required MCP providers are unavailable.
- `--skills-strict` blocks when required skill specs are missing.
- `--skip-validation` is only for rapid iteration; it disables strict MCP/skill checks and skips the artifact validator gate.

Controller metadata is written into the task packet so later review can see who controlled, drafted, reviewed, and verified the run.
