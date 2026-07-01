# Adaptive Subject Runtime Roadmap

## Purpose

Qiongli now installs as an adaptive core package and can infer economics or
finance refinements at runtime. The next product goal is to make this adaptive
subject layer measurable, user-confirmable, and safe to expand across more
disciplines.

This roadmap covers the work after the initial runtime subject refinement and
real preview smoke harness. It is organized as staged increments so each stage
can ship with tests, local verification, and clear rollback behavior.

## Current Baseline

Completed on `dev` as of July 1, 2026:

- Runtime subject refinement packet with `decision`, `signals`,
  `resource_activation_plan`, `borrowed_lenses`, and backward-compatible fields.
- Cross-run `.qiongli/trace/subject_evidence.json` memory for repeated subject
  suggestions.
- Confirmation recommendations in guidance proposals without automatic manifest
  writes.
- Preview-first real smoke harness for subject runtime behavior.
- Isolated smoke environment including `HOME`, `CODEX_HOME`,
  `QIONGLI_CONFIG_HOME`, `QIONGLI_GUIDANCE_HOME`, `XDG_CONFIG_HOME`, and
  `RESEARCH_CLI_LANG=en`.

Remaining product gaps:

- Router quality is not yet measured with a curated corpus.
- Users cannot directly confirm, dismiss, reset, lock, or inspect subject state
  through CLI or MCP tools.
- Confirmed subject state does not yet create lightweight local guidance
  materialization.
- Expansion to additional subjects lacks a quality gate that prevents
  accidental over-activation.
- Real local-agent smoke remains opt-in roadmap work rather than a standard
  release gate.

## Roadmap Principles

- Install remains core-first. Users should not choose a subject during normal
  installation.
- Runtime inference proposes and explains. User action confirms or locks.
- Method-only evidence borrows lenses. It does not switch the project subject.
- Subject expansion requires evaluation fixtures before new rules are trusted.
- Local guidance writes must be explicit, reversible, and project-scoped.
- Every stage must preserve preview-first safety: local agents do not launch
  unless explicitly requested.

## Stage 1: Router Evaluation And Lifecycle Controls

Status: next recommended implementation.

Primary outcome:

- Qiongli can evaluate subject router quality on a curated fixture corpus.
- Users and clients can inspect and control subject state through stable CLI and
  MCP operations.

Scope:

- Add curated subject evaluation fixtures for clear finance, clear economics,
  method-only borrowed-lens, mixed subject, weak-signal core-only, and near-miss
  cases.
- Add a local evaluation runner that produces accuracy, false-positive,
  false-negative, and confusion summaries.
- Add lifecycle operations: `status`, `confirm`, `dismiss`, `reset`, `lock`,
  and `unlock`.
- Connect lifecycle operations to `.qiongli/guidance_manifest.yaml` and
  `.qiongli/trace/subject_evidence.json`.
- Keep `promotion_recommendation` as a recommendation until a user or client
  calls a lifecycle operation.

Success criteria:

- Evaluation report exits non-zero when thresholds fail.
- `suggest_subject` precision is measured separately from `borrow_lens`.
- Dismissed subject candidates stop producing confirmation prompts until new
  evidence appears or the user resets the dismissal.
- Confirm and lock operations update only project-local manifest state.
- Existing smoke harness and full test suite remain green.

First formal spec:

- `docs/superpowers/specs/2026-07-01-adaptive-subject-lifecycle-and-eval-design.md`

## Stage 2: Lightweight Local Guidance Materialization

Primary outcome:

- Confirmed subject state influences future agent runs through project-local
  guidance text, not only runtime packets.

Scope:

- Generate or update a concise `.qiongli/guidance.d/subject-runtime.md` file
  after explicit confirmation or lock.
- Include active subject, confirmation source, method lenses, resource
  activation summary, and date of last update.
- Keep generated guidance managed and clearly marked so user edits can be
  preserved or merged.
- Add lint checks that generated subject guidance cannot override core safety,
  evidence, or quality-gate contracts.

Success criteria:

- Confirming finance writes a stable local guidance fragment that future
  `effective_guidance()` calls include.
- Reset removes or disables managed subject guidance without touching unrelated
  user guidance fragments.
- Manual user edits are detected and not overwritten silently.

## Stage 3: Subject Expansion With Evaluation Gates

Primary outcome:

- New subjects can be added without weakening router precision.

Candidate subjects:

- Accounting.
- Business and management.
- Political economy.
- Geoeconomics.
- Economics-accounting bridge.

Scope:

- Require an evaluation fixture pack before enabling each new subject.
- Add subject-specific method, venue, data, and outcome signal groups.
- Add near-miss cases for adjacent disciplines to prevent broad over-activation.
- Extend subject resource activation plans only after the evaluation pack passes.

Success criteria:

- Each new subject has clear positive, borrowed-lens, mixed, and near-miss
  fixtures.
- Existing economics and finance metrics do not regress.
- Subject expansion does not increase false positives in core-only cases.

## Stage 4: Real Local-Agent Smoke

Primary outcome:

- Qiongli can verify a minimal end-to-end local-agent run in an isolated
  environment.

Scope:

- Add a separate opt-in smoke path that requires both a command flag and an
  environment variable.
- Use temporary `HOME`, client config roots, project root, and trace root.
- Run one small task through the local runtime with bounded output and no
  external network requirement.
- Verify trace bundle completeness, subject refinement packet persistence, and
  no writes outside the isolated root.

Success criteria:

- Preview smoke remains the default release gate.
- Local-agent smoke is available for maintainers and release candidates.
- Failed local-agent smoke reports the exact command, isolated root, and trace
  path for diagnosis.

## Stage 5: Feedback-Aware Subject Refinement

Primary outcome:

- User confirmations, dismissals, and resets improve later recommendations
  without turning the router into hidden automation.

Scope:

- Track lifecycle actions as explicit evidence events.
- Add per-project cooldowns for dismissed subjects.
- Use confirmation history as one signal dimension in routing.
- Add explainability output showing which evidence came from task text,
  manifest state, user action, or trace memory.

Success criteria:

- Users can see why a subject was suggested.
- Repeated user dismissals reduce repeated prompts.
- Explicit user confirmation remains stronger than inferred text evidence.

## Stage 6: Release And Marketplace Readiness

Primary outcome:

- Adaptive subject lifecycle works consistently across CLI, plugin, marketplace,
  and client-native installs.

Scope:

- Ensure lifecycle MCP tools are included in full runtime installs.
- Ensure npm-lite surfaces explain that lifecycle controls require the Python
  runtime when applicable.
- Document marketplace behavior for clients that cannot write `.qiongli`
  project files.
- Add release-readiness checks for lifecycle controls, evaluation reports, and
  smoke isolation.

Success criteria:

- CLI and MCP behavior match for status, confirm, dismiss, reset, lock, and
  unlock.
- Read-only clients can still show recommendations and export proposed actions.
- Release receipts include subject eval and smoke report summaries.

## Cross-Stage Risks

- Over-activation risk: mitigated by near-miss fixtures and precision
  thresholds.
- User trust risk: mitigated by explicit confirm, dismiss, reset, lock, and
  unlock controls.
- Client parity risk: mitigated by CLI and MCP contracts that share one runtime
  implementation.
- Local file safety risk: mitigated by project-local writes, managed fragments,
  and isolated smoke tests.
- Scope creep risk: mitigated by delaying new subject expansion until lifecycle
  and evaluation gates are in place.

## Recommended Immediate Plan

Implement Stage 1 first:

1. Build the evaluation fixture schema and runner.
2. Add lifecycle state helpers and tests.
3. Add CLI and MCP lifecycle operations.
4. Connect dismissal and confirmation to evidence memory.
5. Run targeted tests, preview smoke, full Python tests, npm tests, and
   whitespace checks.

After Stage 1 ships, decide whether Stage 2 or Stage 4 should come next:

- Choose Stage 2 if the priority is better day-to-day agent behavior after
  subject confirmation.
- Choose Stage 4 if the priority is release confidence in the full local-agent
  runtime.
