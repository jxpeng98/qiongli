# Qiongli Codex And Claude MCP Compatibility Acceptance

Status: accepted at Slice tier

Date: August 24, 2026

Target branch: `2.x`

Pull request: `#142`, stacked on `#141`

Publication allowed: false

## Exact identities

| Identity | Accepted value |
|---|---|
| Product source | `192ad24fb175f1eaa7c289dfa916f2b5543bfa70` |
| Evaluation Truth | run `32674594754`: success |
| Native CI | run `32674596106`: success |
| App Full MCP dependency | PR `#141` branch head `9a869da6523a68158e964d1512ebe2c8d7e7f8f4`; accepted product source `670478ccf7c5ed73b264128e1c790c58564117c4` |

The Native CI run passed the change-boundary gate, R2 Lite compatibility, and
the Linux, macOS, and Windows native foundation jobs. Candidate, packaged
product, non-publishing package, and promotion jobs were skipped by the
ordinary Slice boundary.

## Accepted compatibility path

The accepted source proves this path with the existing native owners:

`Codex or Claude Plugin + qiongli-workflow Skill -> bundled native executable -> Lite and Full MCP`

Codex `0.147.0` installed, listed, cached, and removed the receipt-owned Plugin
through its personal marketplace. Its isolated MCP configuration reported the
exact cached executable and Lite arguments. Claude Code `2.1.237` validated the
direct Skills form, installed and listed the user-scope marketplace Plugin,
reported its Skill and MCP components, and returned `Connected` from both
`mcp get` and `mcp list` for the isolated Lite entry.

Both clients launched the same cache-verified executable with an empty runtime
`PATH`. Lite exposed the exact ordered 14-tool registry. Full exposed the exact
ordered 32-tool union and returned the Full `qiongli_orchestrator_route` result
without Lite-only `preview_only`, `runtime_profile`, `recommended_runtime`, or
`upgrade` fields. Each fixture removed its Lite entry and receipt-owned Plugin
state before completion.

The documented path contract uses `~/.agents/skills` and
`<project>/.agents/skills` for Codex, and the configured Claude root plus
`<project>/.claude/skills` for Claude Code. `.agents` is plural. Host caches
remain Host-owned and are verified rather than written directly.

## Focused evidence

| Gate | Result |
|---|---|
| Codex real client | Plugin, customized Skill bytes, cache receipt, Lite client registration, Lite/Full protocol, Full route, and removal passed |
| Claude Code real client | direct Skills discovery, strict validation, Plugin details, cache receipt, connected Lite health, Lite/Full protocol, Full route, and removal passed |
| Profile registries | Lite 14 tools; Full 32 tools; exact ordered equality passed for both clients |
| Client inventory | exact Codex and Claude user/project Skill, marketplace, and managed Plugin source paths passed |
| Native MCP and activation | `mcp_stdio` 7 passed; `client_activation` 3 passed |
| Native workspace | Rust 1.97 format, check, Clippy with warnings denied, and all workspace targets/tests passed |
| Public capability contract | Capability Contract v2 validation passed |

The real-client tests used disposable homes and config roots. Normal Host
profiles, credentials, prompts, responses, user projects, and provider secrets
were not inputs or mutation targets. Emitted evidence contains versions,
digests, counts, and boolean outcomes but no fixture paths.

## Nonclaims

This Slice does not install Lite and Full side by side in a normal user profile,
change Full MCP as the production Ready boundary, prove an authenticated model
session, or support another Agent. It does not accept candidate or packaged
artifacts, signing, notarization, promotion, publication, or release
authorization. PR `#142` remains stacked on unmerged PR `#141`; merging and
release decisions remain separate maintainer actions.
