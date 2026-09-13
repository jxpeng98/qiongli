# Qiongli PILOT-905 Observed Model And Host Capability Matrix Acceptance

Status: accepted at Slice tier

Date: August 30, 2026

Target branch: `2.x`

Pull request: `#151`

Publication allowed: false

## Exact identities

| Identity | Accepted value |
|---|---|
| Matrix implementation source | `73d33932f8bef962fc13fbce8ce6be4e844f258d` |
| Native CI | run `33283608007`: success |
| Evaluation Truth | run `33283608009`: success |
| Latest observed product source | `d0b4113364452d6ff8ff7cb2a3735e7c8d40d3f8` |
| Compatibility evidence source | `192ad24fb175f1eaa7c289dfa916f2b5543bfa70` |
| Matrix payload SHA-256 | `2beea266096a0acf77785a234379f4b9dc0814be71e8b3c886617a873a35132d` |
| Matrix file SHA-256 | `e83f94fec3569239895ce9ce9d0c105570294c917395554811e4cbc61b627ea7` |

The canonical machine-readable matrix is
`docs/superpowers/acceptance/2026-08-30-qiongli-pilot905-host-capability-matrix.json`.
Its payload digest covers canonical sorted JSON with the
`receiptPayloadSha256` field omitted.

## Accepted matrix

The matrix publishes six explicit Host rows and eleven closed capability
columns. Every cell is one of `observed-present`, `observed-absent`, or
`not-observed`; every observed cell resolves to an accepted evidence record with
its repository path, SHA-256, exact product source, Host version, observation
class, and non-publication boundary.

Codex CLI `0.147.0` has directly observed Plugin lifecycle, Skill discovery,
Lite and Full MCP, cleanup, authenticated model execution, project and Graph
reads, structured output, and conversation non-retention. PILOT-903 directly
observed single-agent execution without native subagents, so that one capability
is `observed-absent` for the named journey.

Claude Code `2.1.237` has directly observed isolated Plugin lifecycle, Skill
discovery, Lite and Full MCP, and cleanup. Its authenticated model journey,
project/Graph reads, structured output, native subagents, and conversation
retention behavior remain `not-observed`.

Codex Desktop, Claude Desktop, Antigravity, and a generic local MCP Host each
remain independently `not-observed`. This is an evidence gap, not an unsupported
classification. Neither accepted input records an exact model identifier, so
all model identities remain `not-recorded`.

## Evidence binding

| Evidence | SHA-256 | Exact scope |
|---|---|---|
| `docs/superpowers/acceptance/2026-08-24-qiongli-codex-claude-mcp-compatibility.md` | `fe51b6a82e0c6a0cd42bf68dcd6b0f53d4c009e71b8d981fa5be7d3c4de89a19` | Source `192ad24f...`; isolated Codex CLI and Claude Code compatibility |
| `docs/superpowers/acceptance/2026-08-30-qiongli-pilot903-real-project-receipt.json` | `cf21bcaba8faafd4208e22cda3fd90177a23e5b947dd2de3173bb7caaf487067` | Source `d0b4113...`; authenticated Codex real-project journey |

The bilingual public projections are
`docs/guide/agent-host-capability-matrix.md` and
`docs/zh/guide/agent-host-capability-matrix.md`.

## Verification

- The dependency-free focused matrix test passed: 1 test.
- The VitePress documentation build passed.
- Program Ledger generation/check reported 237 current tasks.
- Trellis task validation and `git diff --check` passed.
- Exact implementation-source Native CI run `33283608007` passed its change
  boundary, Lite compatibility, and Linux, macOS, and Windows native jobs.
- Evaluation Truth run `33283608009` passed.
- Candidate, packaged-product, promotion, and publication jobs remained outside
  this docs/evidence-only Slice.

## Nonclaims

This acceptance does not establish current-source equivalence between Codex and
Claude, CLI and Desktop, Lite and Full, Host and model, or one model and another.
It does not add or qualify another Host, run a new external model, change any
product/package input, build a candidate, authorize publication, or advance
Stable release eligibility by itself.

