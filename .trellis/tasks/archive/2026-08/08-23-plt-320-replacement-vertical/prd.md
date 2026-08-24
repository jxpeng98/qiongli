# Close GOV-320 and prove PLT-320 replacement vertical

## Goal

Close the merged replacement-policy dependency and establish current, exact-head
Slice evidence for the native 1.19 replacement path:

`CLI -> Codex/Claude Plugin and Skills -> Lite/Full MCP -> Zotero`

The task fixes only gaps exposed in the shared native owners. It does not add a
second acceptance framework or move App and Graph work ahead of `PLT-320`.

## Background

- PRs #138 and #139 are merged into `2.x`; merge head `41accaf0407510c971c596fa174f6f3527e03b30`
  contains the archived `GOV-320` Trellis evidence.
- Program Ledger v1 still records `GOV-320` as `active` and `PLT-320` as
  `proposed`, with `PLT-320` depending on `GOV-320`.
- The repository already owns the required mechanisms: managed CLI lifecycle,
  receipt-bound Codex and Claude bundles, fixed official Host CLI plans, native
  Lite/Full stdio servers, and loopback-only Zotero operations.
- Existing copied-binary Zotero coverage proves search and dry-run upsert, but
  does not complete the receipt-bound approved write in the same public MCP
  path.
- Packaged-product, live authenticated Host, signing, promotion, and public
  release checks are Acceptance-tier work and do not run for an ordinary Slice.

## Requirements

### R1 — Close the merged governance dependency

- After the exact `41accaf0` merge-head Native CI succeeds, record `GOV-320` as
  `accepted` with its archived evidence, exact commit, run, and update date.
- Regenerate the current program index with the existing script.
- Move `PLT-320` to `active` before changing product behavior.

### R2 — Prove the existing shared replacement owners

- Reuse the current CLI lifecycle, Plugin/Skill bundle, official Host CLI,
  Lite/Full MCP, and Zotero implementations and tests.
- Run the ignored isolated real-client tests for current Codex and Claude Code;
  they may use only disposable homes and must remove the installed test Plugin.
- Preserve one canonical content source and the current receipt/digest chain.
- Do not add App-owned product logic, another registry, another test runner, or
  another acceptance harness.

### R3 — Close the Zotero public-path gap

- Extend the existing copied-binary stdio test to perform Zotero discovery,
  search, dry-run upsert, and receipt-bound approved upsert through the public
  MCP tool.
- Assert the exact loopback request order and returned write result.
- Keep default writes dry-run, require `write_intent=apply` plus the immediately
  preceding receipt, and retain loopback-only networking and bounded output.
- Exercise the shared behavior without adding a new production API or schema.

### R4 — Preserve Lite/Full truth

- Keep the existing Full profile orchestration regression green so Full does not
  return a Lite upgrade/profile result.
- Verify the installed Host bundles launch the same native binary and expose
  their declared Plugin/Skill/MCP identities without profile drift.

### R5 — Record Slice evidence truthfully

- Run focused checks during implementation and the affected Rust Slice before
  closeout.
- Push a task branch and open a PR targeting `2.x` so exact-head Native CI can
  supply the required Slice evidence.
- Record a path-redacted repository acceptance note bound to the exact product
  commit and CI run, then set `PLT-320` to `accepted` and regenerate the index.
- State explicitly that no release candidate, live authenticated model session,
  real user Zotero library, signing, publication, App (`PLT-321`), or Graph
  (`PLT-322`) claim is made.

## Acceptance Criteria

- [x] `GOV-320` is `accepted` with archived evidence, commit `41accaf0...`, and
      its successful exact-head Native CI run; `PLT-320` becomes `active` before
      implementation.
- [x] The existing copied-binary stdio test proves Zotero search, dry-run, and a
      receipt-bound approved write against an isolated loopback fixture.
- [x] Missing, malformed, or changed approval data remains rejected before an
      approved write; the existing negative coverage remains green.
- [x] Full profile routing still returns the Full orchestration contract, not a
      Lite profile/upgrade result.
- [x] Isolated current Codex and Claude Code tests install, inspect, launch, and
      remove the receipt-owned Plugin/Skill/MCP bundle without touching normal
      Host profiles.
- [x] Focused tests, formatting, affected Clippy/checks, roadmap generation, and
      exact-head Native CI pass at Slice tier.
- [x] A path-redacted evidence note binds the result to one exact product commit
      and run; Program Ledger v1 records `PLT-320` as `accepted` and the generated
      current index is byte-current.
- [x] The evidence note makes no App, Graph, release-candidate, authenticated
      model execution, real Zotero-library, signing, or publication claim.

## Out of Scope

- App setup/status/recovery work (`PLT-321`).
- Graph v1 migrated-project acceptance (`PLT-322`) or any Graph v2/Kernel work.
- New MCP tools, public schemas, Host integrations, provider backends, or UI.
- Real user-profile mutation, authenticated model handoffs, public packages,
  signing/notarization, promotion, or release authorization.
