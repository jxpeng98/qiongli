# Observed Agent Host capability matrix

This page reports what accepted Qiongli receipts directly observed as of
August 30, 2026. It is not a vendor comparison, a model ranking, or a promise
that one Host behaves like another.

Status vocabulary:

- **Observed present** — the cited receipt directly demonstrated the capability.
- **Observed absent** — the cited receipt directly demonstrated its absence.
- **Not observed** — no accepted receipt proves presence or absence; this does
  not mean unsupported.

## Installation and runtime surfaces

| Host | Plugin lifecycle | Skill discovery | Lite MCP | Full MCP | Cleanup |
|---|---|---|---|---|---|
| Codex CLI | Observed present | Observed present | Observed present | Observed present | Observed present |
| Claude Code | Observed present | Observed present | Observed present | Observed present | Observed present |
| Codex Desktop | Not observed | Not observed | Not observed | Not observed | Not observed |
| Claude Desktop | Not observed | Not observed | Not observed | Not observed | Not observed |
| Antigravity | Not observed | Not observed | Not observed | Not observed | Not observed |
| Generic local MCP Host | Not observed | Not observed | Not observed | Not observed | Not observed |

## Authenticated model journey

| Host | Model run | Project read | Graph read | Structured output | Native subagents | No conversation retained |
|---|---|---|---|---|---|---|
| Codex CLI | Observed present | Observed present | Observed present | Observed present | Observed absent | Observed present |
| Claude Code | Not observed | Not observed | Not observed | Not observed | Not observed | Not observed |
| Codex Desktop | Not observed | Not observed | Not observed | Not observed | Not observed | Not observed |
| Claude Desktop | Not observed | Not observed | Not observed | Not observed | Not observed | Not observed |
| Antigravity | Not observed | Not observed | Not observed | Not observed | Not observed | Not observed |
| Generic local MCP Host | Not observed | Not observed | Not observed | Not observed | Not observed | Not observed |

## Evidence boundary

| Receipt | Exact observation |
|---|---|
| [Codex and Claude MCP compatibility](../superpowers/acceptance/2026-08-24-qiongli-codex-claude-mcp-compatibility.md) | Product source `192ad24fb175f1eaa7c289dfa916f2b5543bfa70`; Codex CLI `0.147.0` and Claude Code `2.1.237`; isolated Plugin, Skill, Lite/Full MCP, and cleanup compatibility |
| [PILOT-903 real-project receipt](../superpowers/acceptance/2026-08-30-qiongli-pilot903-real-project-receipt.json) | Product source `d0b4113364452d6ff8ff7cb2a3735e7c8d40d3f8`; Codex CLI `0.147.0`; authenticated Skill + Full MCP project/Graph journey, structured output, privacy, and rollback |

Neither receipt records the exact model identifier, so the model identity is
**not recorded**. The Claude compatibility receipt does not prove an
authenticated Claude model journey. Codex CLI evidence does not qualify Codex
Desktop, and Claude Code evidence does not qualify Claude Desktop. Historical
receipt results remain valid only for their named source and scope; they do not
qualify a changed release candidate.

The canonical machine-readable projection is the
[PILOT-905 matrix receipt](../superpowers/acceptance/2026-08-30-qiongli-pilot905-host-capability-matrix.json).
`publicationAllowed` remains `false`.

