# PILOT-903 representative migrated real-project pilot

## Goal

Close `PILOT-903` with one real Codex Host pilot over the repository-owned
asset-pricing project. The pilot must use the supported copy migration,
Qiongli Skill guidance, current-source Full MCP project/Graph/orchestration
contracts, and a host-produced evidence-grounded result while Qiongli stores
no Host conversation content.

## Background and Confirmed Facts

- `PLT-322` accepted deterministic migration and Graph v1 on
  `RESEARCH/asset-pricing-capm-ff3`, but explicitly excluded a live Host pilot.
- The project is a genuine empirical CAPM-versus-FF3 study with pinned public
  inputs, deterministic derived results, canonical research artifacts, and a
  source-anchored Graph. The source remains intentionally 1.19-compatible.
- Codex and Claude Code Plugin, embedded Skill, Lite MCP, and Full MCP
  compatibility were already accepted in isolated clients. Repeating that
  matrix would not close the remaining pilot gap.
- The current Codex CLI supports an ephemeral, non-persisted execution and can
  load project-local Skills from the standard `.agents/skills` location.
- Full MCP already owns project reads, Graph snapshot/query, host doctor,
  revision-bound handoff, evidence reads, candidate submission, recovery, and
  redacted checkpoint inspection. No new workflow or project API is needed.
- The installed Alpha.3 runtime predates the accepted Full-route repair and can
  still return the Lite upgrade response. The pilot must therefore use the
  exact current source binary and fail closed if Full routing is not truthful.

## Requirements

### R1 — Use one representative project without changing its scholarship

- Use `RESEARCH/asset-pricing-capm-ff3` as the sole source project.
- Inventory the committed source before the run, migrate it through the
  existing preview/apply contract into a private temporary destination, and
  verify the source inventory is unchanged after the pilot and rollback.
- Do not revise the empirical analysis, literature claims, Graph semantics, or
  canonical research artifacts merely to make the pilot pass.

### R2 — Run through a real Codex Host and current Qiongli contracts

- Build the current-source native `qiongli` binary once and bind the pilot
  receipt to the clean Git commit and binary digest.
- Run a fresh Codex CLI execution with `--ephemeral`, `--ignore-user-config`, a
  read-only workspace sandbox, an isolated `QIONGLI_CONFIG_HOME`, and an
  explicit Full MCP stdio registration for that binary.
- Materialize the existing Qiongli Codex Skill into the temporary conventional
  `.agents/skills` root and record its digest. Do not invent a second Skill or
  write into a Host cache.
- Use a truthful `single-agent` Codex descriptor. Do not claim native subagents
  merely because the Host or repository can support them.

### R3 — Complete a meaningful Full MCP project journey

- Require Full routing to return `orchestrator_mcp` with no Lite upgrade or
  preview-only fields, and require the project, Graph, and host-orchestration
  tools needed by the pilot.
- Read the migrated project and its non-empty source-anchored Graph, exercise at
  least one bounded stable query, and preserve the project/projection revision
  bindings.
- Run `doctor -> start -> read -> submit -> next` until the solo checkpoint is
  terminal or Qiongli reports a stable blocker. Candidate claims must cite only
  authenticated evidence references returned for the active handoff.
- Treat candidate text as untrusted input. A successful submission is a
  checkpoint transition, not a project mutation or research-quality approval.

### R4 — Preserve privacy, authority, and rollback boundaries

- Qiongli may persist only its existing redacted checkpoint identities and
  candidate/evidence digests. It must not persist prompts, responses, candidate
  bodies, Host conversation/session IDs, credentials, or absolute Host paths.
- Keep raw Host stdout and temporary output-schema material outside the
  repository and remove them after deriving bounded evidence.
- Do not mutate the normal Qiongli project registry, normal Codex configuration,
  Plugin cache, Host authentication, or the source project.
- Finish with the existing migration rollback contract so only the exact
  receipt-owned temporary destination and registration are removed.

### R5 — Record exact, bounded acceptance evidence

- Emit one redacted pilot receipt containing only schema/status, exact product
  commit, binary/Skill/source inventory digests, Host family/version, MCP and
  adapter versions, bounded Graph counts/digests, checkpoint transition counts,
  terminal state/reason, required check IDs, and the receipt digest.
- Scan the receipt and acceptance note for credentials, prompts/responses,
  conversations, candidate/tool bodies, temporary paths, and absolute paths.
- Mark `PILOT-903` accepted only after the real Host run, source-retention check,
  rollback check, focused verification, and exact-head Slice CI pass.
- If the pilot exposes a product defect, fix the shared owner once and add one
  focused regression. Do not create a parallel Host adapter or umbrella suite.

## Acceptance Criteria

- [ ] The exact clean product commit, native binary, Qiongli Skill, source
      inventory, Codex version, and Full MCP protocol are digest/version bound.
- [ ] The supported migration preview/apply succeeds in isolated state, the
      migrated project is visible through Full MCP, and the source inventory is
      unchanged.
- [ ] A fresh ephemeral Codex run discovers the conventional Qiongli Skill and
      current-source Full MCP without reading normal user configuration.
- [ ] Full routing is truthful, required project/Graph/orchestration tools are
      present, and host doctor reports the exact project revision runnable.
- [ ] The Host obtains authenticated project and Graph evidence, completes the
      solo handoff sequence to a terminal state, and never converts submission
      into an unapproved project mutation.
- [ ] A bounded Graph query is non-empty and internally consistent with the
      accepted projection identity.
- [ ] Qiongli state and committed evidence contain no prompt, response,
      candidate/tool body, Host conversation/session identifier, credential,
      temporary path, or absolute path.
- [ ] Supported migration rollback removes only the pilot destination and
      registration; the original project and normal Host/Qiongli state remain
      unchanged.
- [ ] One redacted receipt and acceptance note bind the observed pilot, focused
      checks pass, and `PILOT-903` is accepted only after exact-head CI passes.

## Out of Scope

- A second Claude Code run, cross-Host equivalence, or a Host capability matrix;
  those remain `REL-307` / `PILOT-905` work.
- Candidate packaging, three-target artifacts, signing, publication, update,
  permanent system installation, or release authorization.
- App UI changes, Graph v2, new MCP tools/schemas, new agent abstractions, or a
  second Plugin/Skill installer.
- Editing the asset-pricing analysis, making new scholarly claims, writing a
  manuscript, or applying any research-project capture.
- Persisting or exporting the Codex conversation, model response, or raw tool
  transcript as acceptance evidence.

## Resolved Decisions

- One Codex pilot is sufficient for `PILOT-903`; the already accepted two-client
  compatibility matrix is reused instead of repeated.
- The run is ephemeral and isolated rather than modifying the user's normal
  Codex or Qiongli installation.
- The existing migrated asset-pricing project is representative; creating a
  second research project would add no evidence for this gate.
- The default implementation is evidence-only. Product code changes occur only
  for a reproducible essential-path defect exposed by the live run.
