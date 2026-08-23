# Design: PLT-320 native replacement vertical

## Boundary

This task is an evidence-and-gap-closing slice over existing owners. No new
runtime layer is introduced.

| Concern | Existing owner | Planned change |
|---|---|---|
| Program state | Program Ledger v1 and generator | Close `GOV-320`, activate then accept `PLT-320`. |
| CLI and Host setup | `managed_operation.rs`, `desktop.rs`, existing bundle tests | Reuse; fix only an observed shared-root defect. |
| Canonical Plugin/Skills | embedded `content/` pack and receipt-owned bundle composers | Reuse unchanged. |
| Lite/Full MCP | `apps/qiongli/src/mcp.rs`, runtime contracts, `mcp_stdio.rs` | Extend one existing stdio journey; preserve profile routing. |
| Zotero | `qiongli-runtime::zotero` loopback client | Reuse approval contract; add no API. |
| Evidence | acceptance Markdown plus Program Ledger v1 | Add one redacted exact-source Slice record. |

## Data flow

```text
canonical content + native binary
  -> receipt-owned Codex/Claude bundle
  -> isolated official Host CLI install/cache observation
  -> bundled native stdio server
  -> Lite/Full profile dispatch
  -> loopback Zotero search
  -> dry-run receipt
  -> explicit apply using the same receipt
```

The test fixture owns only loopback responses. Production validation continues
to require `dry_run=false`, `write_intent=apply`, and a valid `zwr1_` receipt.
Normal Host homes, credentials, prompts, responses, and Zotero libraries are not
inputs.

## Implementation shape

1. Use the successful `41accaf0` Native CI run to accept `GOV-320`; activate
   `PLT-320` and regenerate the index.
2. Run the existing Host, bundle, Full-profile, and Zotero focused tests.
3. Extend the existing copied-binary Zotero stdio test with one approved apply
   after its dry-run. Keep the exact request sequence assertion.
4. If the test exposes a defect, fix it once in the shared runtime MCP/Zotero
   owner after tracing every caller. Otherwise make no production-code change.
5. Freeze the Slice commit, push it, and require exact-head Native CI.
6. Add the redacted evidence record and accept `PLT-320` in the ledger using the
   tested product commit/run; the evidence-only closeout does not widen claims.

## Compatibility and security

- No public schema or tool name changes.
- No fallback to Python/Node, shell execution, arbitrary Host commands, or
  direct Host cache writes.
- Existing official CLI command allowlists, digest revalidation, output bounds,
  target order, and fail-closed probes stay authoritative.
- Zotero traffic remains loopback-only and writes remain preview/receipt bound.
- Installed bundle outputs remain generated projections, never canonical input.

## Verification tiers

- **Focused:** the exact stdio Zotero test, Full profile regression, official
  Host plan tests, and isolated Codex/Claude bundle tests.
- **Slice:** affected Rust formatting/check/Clippy/tests plus exact-head Native
  CI for Linux, macOS, and Windows.
- **Acceptance:** candidate packages, real authenticated sessions, real user
  Zotero, signing, and publication remain deferred.

## Rollback

Revert the focused test or any shared-owner fix and restore the two ledger rows.
The implementation writes only disposable fixture state; no Host or Zotero user
state requires cleanup.
