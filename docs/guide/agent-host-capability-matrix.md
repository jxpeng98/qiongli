# Observed Agent Host capability matrix

Current development guidance for native subagents, cross-Host proposals and optional hooks is in [Agent collaboration](/advanced/agent-skill-collaboration). This historical matrix does not qualify those newer changes.

This page reports what accepted Qiongli receipts directly observed as of
August 30, 2026. It is not a vendor comparison, a model ranking, or a promise
that one Host behaves like another.

Status vocabulary:

- **Observed present** — the cited receipt directly demonstrated the capability.
- **Observed absent** — the cited receipt directly demonstrated its absence.
- **Not observed** — no accepted receipt proves presence or absence; this does
  not mean unsupported.

## 2.4 development observations

The [2.4 candidate plan](../superpowers/plans/2026-10-05-v2-4-plugin-quality-release-plan.md)
keeps newer observations separate from the accepted August receipts below.
These results concern the named development sources in that plan and do not
qualify a changed final release candidate.

| Host | Development evidence | Remaining qualification |
| --- | --- | --- |
| Codex CLI | On `85758cac`, installed regression passes structure/review 3/3, actual new-note save/restart 1/1, and one frozen fresh synthetic cohort 3/3; guidance, read-tool policy and cleanup are verified | Bounded tasks with supplied guidance locations; no automatic discovery, full-paper or broad academic acceptance. Earlier failures remain in the plan. |
| Antigravity CLI | Earlier 1.2.17 install/update, Skill reads and native return remain scoped observations; isolated 1.3.0 install/enable and source/cache verification also pass on `85758cac` | The fresh 1.3.0 profile requires login and makes no MCP call. Complete raw result, successful Host completion and research continuation remain open. |
| Claude Code / DeepSeek | `c2cae736` Linux ARM64 qualification passes the Claude archive and npm DeepSeek projection | Other target qualification remains open; no new model journey is claimed. |

Antigravity uses its CLI Plugin format and the normal Host trust/approval flow;
keep its registered source directory and start a fresh session after updates.
See [installation instructions](../advanced/plugin-installation.md#antigravity).

For native source `c2cae736`, local Linux ARM64 qualification passes the archived
Codex/Claude Plugins and the npm DeepSeek projection. The archived CLI also
exports an Antigravity Plugin with 22 Skill entries and verified file bytes and
modes; its bundled executable directly lists 35 Full MCP tools and returns a
complete `qiongli_config_status` result over stdio. These are package/native
transport checks. This archived-binary observation does not register a Plugin or
run an AGY session; the separate 1.3.0 installation/authentication outcome is
listed above. Final-source four-target qualification remains open.

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
| [PILOT-903 real-project receipt](https://github.com/jxpeng98/qiongli/blob/5a3ab87fcba67dfbe700f895c0321e455bbbc914/docs/superpowers/acceptance/2026-08-30-qiongli-pilot903-real-project-receipt.json) | Product source `d0b4113364452d6ff8ff7cb2a3735e7c8d40d3f8`; Codex CLI `0.147.0`; authenticated Skill + Full MCP project/Graph journey, structured output, privacy, and rollback |

Neither receipt records the exact model identifier, so the model identity is
**not recorded**. The Claude compatibility receipt does not prove an
authenticated Claude model journey. Codex CLI evidence does not qualify Codex
Desktop, and Claude Code evidence does not qualify Claude Desktop. Historical
receipt results remain valid only for their named source and scope; they do not
qualify a changed release candidate.

The canonical machine-readable projection is the
[PILOT-905 matrix receipt](https://github.com/jxpeng98/qiongli/blob/5a3ab87fcba67dfbe700f895c0321e455bbbc914/docs/superpowers/acceptance/2026-08-30-qiongli-pilot905-host-capability-matrix.json).
`publicationAllowed` remains `false`.
