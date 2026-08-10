# Close Alpha 3 First-Usable Product Spine

## Goal

Deliver one internally usable Qiongli `v2.0.0-alpha.3` candidate in which the
App, native CLI, Codex/Claude Plugin and Skills, Lite/Full MCP, and Zotero work
as one self-contained product without a user-installed Python or Node runtime.

Public release authorization is not part of this task. It resumes only after
the exact candidate satisfies the separate A6-A9 claim gates.

## Confirmed Facts

- `origin/2.x` and the current branch start at
  `19b549424cc417dd70140dbc5b3ce080848544af`.
- The current `fix/alpha3-app-usability` worktree contains uncommitted fixes for
  the three reported App defects; no candidate run contains those changes.
- The native Zotero runtime currently probes status and exports import files,
  while the bundled reference-manager Skill calls
  `qiongli_zotero_upsert_references`. The Companion already implements endpoint
  contract 2 search and receipt-bound dry-run/apply endpoints.
- Full MCP publicly exposes `qiongli_project_capture_apply`; release text that
  claims “no Full MCP mutation” is inaccurate.
- The public GitHub release remains Alpha 1. Alpha 3 publication is not
  authorized.

Evidence is recorded in `research/current-state.md`.

## Requirements

### R1 — App first-use defects

- Literature provider configuration must render fields from the native provider
  contract: OpenAlex email/key, Semantic Scholar key, Crossref email, PubMed
  key, and no credential field for arXiv.
- Research Library artifact preview and the owning workspace must not compete
  for the same vertical scroll gesture.
- Installed/bundled Plugin and Skill resources must be previewable. For a
  registered project, users may edit advisory local guidance and preview an
  exact write to `<project>/.qiongli/local_guidance.md` before approval.
- Canonical embedded Plugin/Skill content remains immutable; customization does
  not overwrite verified source files or unrelated user content.

### R2 — Native Zotero vertical

- Native MCP must report the real loopback Companion status and reject endpoint
  versions other than `2`.
- Native MCP must expose bounded Zotero search and
  `qiongli_zotero_upsert_references` using the Companion's existing endpoint-2
  request/response contract.
- Upsert defaults to dry-run. Apply requires the unchanged request, the
  Companion-issued one-shot receipt, and explicit write intent within the
  Companion's expiry window.
- When the Companion is unavailable, status must truthfully advertise the
  import-file fallback; it must not fabricate direct Zotero success.
- The bundled Skill, MCP registry/profile, schemas, native dispatch, and docs
  must name the same available operations.

### R3 — Truthful public contracts

- Alpha 3 notes must name only CLI commands present in the native parser.
- Full MCP documentation must distinguish the one approval-bound capture write
  from unrestricted mutation and from read-only in-process ToolHost dispatch.
- App, CLI, Skills, MCP and Zotero capability claims must match the exact
  packaged artifact.

### R4 — Minimal evidence path

- Each slice uses one focused local regression while editing.
- After the source set is frozen, exact-head Native CI runs once.
- When package inputs changed, the existing packaged macOS vertical acceptance
  command runs once; no new umbrella suite or duplicate receipt is added.
- Any later commit invalidates the exact-head and packaged candidate evidence.

## Acceptance Criteria

- [x] Provider, scrolling, and Plugin/Skill guidance behavior pass the App API,
      Desktop, and affected native/project focused checks.
- [ ] A fresh native MCP session can list and exercise Zotero status, bounded
      search, dry-run upsert, approved apply, receipt replay rejection, and the
      unavailable/fallback path.
- [ ] Every tool named by the reference-manager Skill exists in its advertised
      MCP profile or the Skill routes to the documented safe fallback.
- [ ] `qiongli --version`, native help, `qiongli app snapshot`, Lite MCP, and
      Full MCP run from packaged bytes with an empty runtime `PATH`.
- [ ] Codex and Claude integration payloads contain the expected Plugin, Skills,
      and MCP registrations and fail closed until a Host positively observes
      activation/attachment.
- [ ] Release notes, architecture, roadmap, and ledger make no capability or
      publication claim beyond the exact product contract.
- [ ] Focused checks, exact-head CI, and one package vertical receipt identify
      the same final source/package set with `publication_allowed=false`.

## Out Of Scope

- Public A8 authorization, tags, releases, announcements, or marketplace writes.
- Manual visual/Zotero acceptance, real system-profile Host receipts, and
  update/rollback acceptance except when a focused defect is reproduced.
- Graph v2, typed research kernel, Q1-Q4 evaluation redesign, more providers,
  more agents/pages, remote collaboration, or legacy 1.x feature work.
- Arbitrary editing of canonical Plugin or Skill source from the App.

## Constraints

- Preserve preview/digest/revision approval, path containment, secret redaction,
  loopback-only Zotero networking, and receipt-owned removal.
- Do not edit generated payloads directly or introduce a second project,
  provider, MCP, or release authority.
- Keep user-owned dirty changes intact and commit review slices separately.

## Blocking Open Questions

None. Product scope and risk boundaries follow the user's stated first-version
goal and the existing native/Companion contracts.
