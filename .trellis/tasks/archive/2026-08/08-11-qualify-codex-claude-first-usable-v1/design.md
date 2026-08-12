# Design

## Boundary

Reuse the existing product and acceptance spine:

`frozen candidate -> isolated real clients -> system registration -> fresh Host -> Skill + Full MCP -> Zotero -> redacted receipt`

There is one candidate and two Host observations. Codex and Claude Code are not
separate implementations. The native registry, embedded Plugin/Skills, and MCP
server remain the shared owner.

## Candidate And Isolation

- Treat promotion artifact `9082833758` as input and its aggregate receipt as
  identity authority. Extract it into a fresh private temporary root.
- Resolve Codex and Claude Code to their absolute installations, not version
  manager shims.
- Reuse `native_candidate_acceptance` and the two ignored real-client tests.
  Run them against an exact-source worktree or the downloaded candidate path as
  supported by the existing acceptance tooling; do not make current evidence-
  only `2.x` head masquerade as source `cced6082...`.

## System Profile Migration

- Before mutation, record bounded digests and private rollback copies of only
  Qiongli-owned registrations, source bundles, and cached bundle references.
- Use the existing candidate/App preview-and-apply authority and its receipts.
  Direct file copying is permitted only when the existing command explicitly
  defines it as the installation mechanism.
- Confirm the installed executable and Plugin/content digests, not only
  `2.0.0-alpha.3`. Preserve all unrelated marketplace, plugin, MCP, and account
  state.
- Restart each Host after apply. On failure, use receipt-owned removal/rollback
  and verify the preflight digests are restored.

## Host Observation

- Codex and Claude Code each run once in a fresh, bounded session using their
  normal authenticated profile. The prompt names the existing Qiongli Skill and
  restricts tools to the Qiongli MCP/read-only journey.
- Record client version, adapter version, Plugin state, Skill discovery, MCP
  protocol, candidate/content digests, tool result digests, and a bounded final
  disposition. Do not record prompts, responses, paths, project IDs, or secrets.
- Where its prepared fixture is available, use
  `r5c-c5-live-host-runbook.md` and its existing schema/composer. Otherwise the
  basic first-use observation remains separate and must not be mislabeled as an
  A7 R5C C5 receipt.

## Zotero Observation

- Use the bundled Companion XPI with the installed Zotero application in an
  isolated Zotero profile and disposable data root. Do not open or mutate the
  user's normal Zotero library.
- Full MCP calls the existing loopback endpoint-2 contract. Search is bounded;
  upsert stays dry-run until the exact one-shot receipt is explicitly applied.
- Verify replay rejection and cleanup, then stop the isolated Zotero instance
  and remove its disposable profile. Separately verify export/import fallback
  while the Companion is unavailable.

## Failure And Source-Change Rule

Classification is enough unless a product defect is reproduced:

- environment/configuration failure: repair only the disposable/profile setup;
- evidence/tooling failure: correct the existing acceptance owner;
- shared product P0: fix once at the native/content owner and add one focused
  check.

Any product or package-input change creates a new candidate identity and
invalidates downstream Host evidence. Documentation-only evidence updates do
not replace the frozen product candidate.

The authenticated Codex receipt for `cced6082` is retained as diagnostic
evidence because it reproduced the route defect and required a bypass. The
native route fix is a new product input; it receives one focused local
regression now and requires a new exact candidate only if Alpha 3 release
qualification resumes. Claude live execution is deferred rather than simulated.

## Security And Publication Boundary

Host authentication remains Host-owned. Secrets and private state never enter
Qiongli receipts. All candidate and Host actions remain local and
`publication_allowed=false`; this design contains no publication action.
