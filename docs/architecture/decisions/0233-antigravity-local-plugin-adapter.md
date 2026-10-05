# ADR 0233: Antigravity local Plugin adapter

- Status: Accepted
- Date: 2026-10-05
- Task: CLI-409
- Authority: the maintainer explicitly requests Antigravity/agy installation and
  actual Qiongli Skill/MCP verification in the current native architecture.
- Extends ADRs 0222/0224 for this terminal flow. Their signed-product and public
  App target boundaries remain unchanged; historical decisions are retained.

Add `install/update/upgrade plugin --target antigravity` (`agy`, menu 5). Keep
`3`/`both` as Codex+Claude; `all` includes the new Host. The official AGY manager
owns profile writes. Two terminal confirmations separately authorize source
export and install/enable, with ten-minute expiry, exact executable/profile/
receipt digests and revalidation under the existing managed-write guard.

The existing bundle projection and filesystem transaction owner supports a
distinct `user-local-antigravity-full-mcp` receipt kind and
`.qiongli-antigravity-plugin-bundle.json`. Reuse canonical workflow projection,
variant and metadata language, wrapper Skills, full tree verification, staging,
locking, no-replace moves and CAS. Codex receipts keep their existing serialized
bytes; signed Codex/Claude and foreign local kinds cannot be adopted. This local
kind has no publisher grant and does not authorize signed AGY activation.

Project native `plugin.json`, `mcp_config.json`, 22 Skill entries and a bundled
binary. AGY 1.2.17's observed native import preserves these bytes and copies to
`.gemini/config/plugins`. Source and cache must both verify. Inspect registration
and enabled state independently because `plugin list` does not expose all state.
Unknown or changed caches and conflicting standalone MCP entries refuse; no
automatic uninstall, credential migration or model selection is introduced.
AGY can create cache directories with owner-only `0700` permissions while
preserving canonical file modes and bytes. A separate read-only cache verifier
accepts `0700` or `0755` directories through the existing ownership/link checks;
source and signed-package verification retain their canonical directory modes.
File modes, receipt identity and whole-tree content checks are unchanged, and
Qiongli does not rewrite permissions in the Host-owned cache.

MCP uses the absolute path of the retained source binary. Relative commands and
Claude-specific variables were not demonstrated to resolve against the Plugin
cache. The source directory must stay available; unregister through AGY before
removing it. This is a user-local installation, not a portable release archive.
The adapter defaults to `~/qiongli-antigravity`; custom destinations are explicit
and must be supplied again on update. It does not advertise automatic discovery
of a previous custom source.

Full orchestration uses the existing `other-local` descriptor and defaults to
`single-agent`; only actual Host capabilities may expand it. Models and sessions
remain Host-owned. Canonical content and shared MCP schemas are unchanged.
Context hooks, scripted App plans, signed installation, IDE and other-platform
qualification are separate increments.

Tests retain cancellation, ownership, unsafe paths, drift, stale receipts and
Codex compatibility. A live AGY observation must distinguish Plugin installation,
Skill loading, MCP discovery and successful calls. Rollback removes the new
terminal entry; previously installed Plugins remain managed by AGY, and older
Codex owners refuse the separate local receipt kind.
