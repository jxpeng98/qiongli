# PILOT-905 observed model and Host capability matrix

## Goal

Publish a truthful, bilingual model/Host capability matrix that lets users and
release reviewers distinguish directly observed Qiongli behavior from absent
evidence. Every positive cell must resolve to an accepted, source-bound receipt;
one Host's result must never be projected onto another Host or model.

## Background

- The Program Ledger defines PILOT-905 as publishing a matrix from observed
  receipts rather than marketing equivalence.
- The accepted August 24 compatibility receipt binds Codex CLI `0.147.0` and
  Claude Code `2.1.237` to product source
  `192ad24fb175f1eaa7c289dfa916f2b5543bfa70`. It directly observes Plugin
  lifecycle, Skill discovery, Lite MCP, Full MCP, and clean removal in isolated
  client roots; it does not observe an authenticated model journey.
- The accepted PILOT-903 receipt binds Codex CLI `0.147.0` to product source
  `d0b4113364452d6ff8ff7cb2a3735e7c8d40d3f8`. It directly observes a
  project-local Skill, Full MCP, authenticated model execution, project and
  Graph reads, structured output, single-agent execution, privacy-safe
  checkpointing, and rollback.
- Neither accepted receipt records an exact model identifier. Host identity is
  not model identity.
- No accepted receipt independently proves Codex Desktop, Claude Desktop,
  Antigravity, or a generic MCP Host. Existing packaging or installation design
  does not qualify those Hosts.

## Requirements

### R1 — Closed, evidence-shaped matrix

- Publish exactly these capability columns: Plugin lifecycle, Skill discovery,
  Lite MCP, Full MCP, authenticated model execution, project read, Graph read,
  structured output, native subagents, conversation non-retention, and cleanup.
- Publish rows for Codex CLI, Claude Code, Codex Desktop, Claude Desktop,
  Antigravity, and generic local MCP Hosts so missing evidence is visible rather
  than omitted.
- Use only `observed-present`, `observed-absent`, and `not-observed`. An
  `observed-*` cell must cite at least one accepted evidence record.

### R2 — Exact provenance and non-equivalence

- Each evidence record must carry its repository path, file SHA-256, product
  source commit, Host family/version, observation class, and publication
  boundary.
- Record the model identifier as `not-recorded` unless an accepted receipt names
  it. Do not infer it from the Host version, current login, or vendor branding.
- Preserve each receipt's exact source. Do not combine historical Claude
  compatibility and current Codex model evidence into a same-candidate or
  cross-Host equivalence claim.

### R3 — Public and machine-readable projections

- Add one canonical machine-readable acceptance receipt and one concise
  English/Chinese user-facing matrix projection.
- Link both user-facing pages from the corresponding documentation landing page.
- The pages must explain the status vocabulary, evidence age/source boundary,
  and the difference between client compatibility and authenticated model use.

### R4 — Privacy and release boundary

- Store only public repository-relative evidence paths, versions, digests,
  booleans/statuses, and capability names. Store no Host conversations,
  prompts, responses, credentials, usernames, or machine-local paths.
- This task synthesizes already accepted receipts. It does not launch another
  external model, mutate a normal Host profile, build a candidate, or authorize
  publication.

### R5 — Minimal executable guard

- Add one dependency-free focused test that validates the closed Host/capability
  inventory, allowed statuses, evidence references and SHA-256 bindings, and the
  invariant that every observed cell has evidence.
- Reuse the existing Program Ledger generator and docs build; add no schema
  framework, generator, or full-suite gate.

## Acceptance Criteria

- [ ] The machine receipt contains all six Host rows and all eleven capability
      columns with only the three declared statuses.
- [ ] Every observed cell points to an accepted evidence record whose path,
      SHA-256, product source and Host version match the source document.
- [ ] Codex CLI reports the directly observed compatibility and PILOT-903 model
      journey; Claude Code reports compatibility only; the other four Hosts are
      explicitly `not-observed`.
- [ ] Exact model identity is `not-recorded`, and no text claims Codex/Claude,
      CLI/Desktop, Lite/Full, or Host/model equivalence.
- [ ] English and Chinese pages expose the same matrix and are linked from both
      documentation landing pages.
- [ ] The focused matrix test, docs build, Program Ledger freshness check,
      Trellis task validation, `git diff --check`, and required Slice CI pass.
- [ ] PILOT-905 is accepted in Program Ledger v1 with exact evidence, product
      source, and CI identity; publication remains false.

## Out of Scope

- A new live Codex or Claude model run, model-quality ranking, benchmark score,
  or claim that one model/Host is better than another.
- New Codex Desktop, Claude Desktop, Antigravity, Hermes, or generic-agent
  installation/runtime support.
- Changes to Plugin, Skill, Lite/Full MCP, Graph, CLI, App, package, or provider
  implementation.
- Candidate assembly, signing, notarization, release promotion, marketplace
  publication, or Stable authorization.
