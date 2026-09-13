# Qiongli PILOT-903 Representative Real-Project Pilot Acceptance

Status: accepted at Slice tier

Date: August 30, 2026

Target branch: `2.x`

Pull request: `#149`

Publication allowed: false

## Exact identities

| Identity | Accepted value |
|---|---|
| Product source | `d0b4113364452d6ff8ff7cb2a3735e7c8d40d3f8` |
| Native CI | run `33280296497`: success |
| Native binary | `7f4dc44396ecd71de5b1cc831ebe460054435a20d3e60eb57008b8f94816b693` |
| Qiongli Skill tree | `5f51c2207d561c9ff88d7c7ab89c0c7361a964ed6df5b5577aee640de970f8c0` |
| Migration source inventory | `f1122f1f7a5679bc8d626e8790aebf63665876f5ab20b21d150f92a58fd90369` |
| Receipt payload | `f81eb7aa501f905a453e3e026cb8d3dd966e7617578cc65fd70682f231d561db` |
| Receipt file | `cf21bcaba8faafd4208e22cda3fd90177a23e5b947dd2de3173bb7caaf487067` |

The machine-readable receipt is
`docs/superpowers/acceptance/2026-08-30-qiongli-pilot903-real-project-receipt.json`.
Its payload digest covers canonical sorted JSON with the
`receiptPayloadSha256` field omitted.

## Accepted real Host journey

One Codex CLI `0.147.0` execution used the current-source Qiongli
`2.0.0-alpha.3` binary, an explicit `qiongli-full-mcp/1` stdio registration,
and the existing Qiongli Skill at the conventional project-local Skill root.
The execution was ephemeral, ignored normal user configuration, used a
read-only workspace sandbox, and reported `single-agent` plus the active
structured-output capability. It did not report native subagents.

The current-source Full MCP routed to Host orchestration without Lite upgrade
or preview-only fields. Host doctor was runnable at the exact project revision.
The solo checkpoint completed all 76 tasks through 76 handoffs, 77 authenticated
evidence reads, and 76 submissions. The Host observed both the project summary
and Graph and performed no research-project write.

## Graph, privacy, and rollback evidence

The migrated project produced a visualizable Graph with 60 nodes, 52 semantic
nodes, 110 relations, and zero diagnostics. A bounded stable query matched one
node without truncation. The projection, manifest, semantic, and source digests
are recorded in the machine-readable receipt without research prose or rows.

Independent checkpoint inspection found a maximum persisted string length of
64, zero absolute-path strings, zero multiline strings, and zero
conversation-like, prompt-like, response-like, candidate-body, or content-body
keys. Only existing redacted checkpoint identities and digests remained. Raw
Host output and the temporary output schema were deleted after receipt
derivation and were not committed.

Supported migration copied 58 files and retained the source. Rollback preview
reported no drift, then rollback removed 61 migration-owned artifacts, the
temporary destination, and its isolated registration. The original source
still matched the clean Git product source after rollback.

## Focused verification

- Current-source native build passed.
- Existing Full-route regression: 1 passed.
- Existing Host-handoff regression: 1 passed.
- Full MCP initialize, registry, route, project, Graph, doctor, handoff,
  evidence-read, submission, and terminal-checkpoint probes passed.
- Receipt canonical-digest and forbidden-value scan passed.
- Program Ledger generation, Trellis task validation, and `git diff --check`
  passed.
- Exact-source Native CI run `33280296497` passed its change boundary, Lite
  compatibility, and Linux, macOS, and Windows native jobs. Candidate,
  packaged-product, promotion, and publication jobs were outside this Slice.

## Nonclaims

This Slice does not accept a second Claude Code pilot, cross-Host equivalence,
Graph v2, App UI changes, candidate or packaged artifacts, signing, promotion,
publication, or release authorization. It does not treat model-produced
candidate text as a project mutation or research-quality approval.
