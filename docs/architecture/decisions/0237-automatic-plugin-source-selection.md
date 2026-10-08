# ADR 0237: Automatic Plugin Source Selection

- Status: Accepted
- Date: 2026-10-08
- Task: CLI-402
- Authority: the maintainer requests detected clients and automatic installation
  paths instead of repeated platform/path selection.
- Supersedes ADR 0235's manual fresh-source choice, ADR 0222's two-basename
  restriction and ADR 0233's fixed default on AGY updates only. Existing receipt,
  approval, conflict and sequential failure boundaries remain unchanged.

Default the main installation guide to detected-all on Enter. Unqualified
`install/upgrade/update plugin` also selects detected clients, unless an explicit
destination or Hook option requires the existing Host menu. Explicit targets and
the advanced menu remain available. Detection uses the existing read-only
executable lookup for Codex, Claude, DeepSeek, AGY and Pi; missing CLI commands
are skipped. An application/configuration directory alone is not sufficient.

Codex/Claude reuse registered source paths before selecting defaults. Without a
registration, reuse a verified legacy home/qiongli (or qiongli-next) export for
the selected Host. Otherwise use home/qiongli-codex or home/qiongli-claude;
prereleases use qiongli-next-codex/claude. These matching Host-specific basenames
extend the existing source validator; historical names stay valid. Unknown,
changed or unsafe new default paths refuse without overwriting or inventing
numbered fallback directories. Other files at legacy paths are left untouched.
No parent is created during discovery, and normal source approval owns writes.

AGY recovers a custom source from its completely verified cache's MCP command,
rechecking the manifest digest against that receipt. The existing source and
cache validation and pre-apply comparison still run. Pi's registered package
source discovery and DeepSeek's profile manager retain their existing owners.
All-detected mode preserves existing Hook choices (off for new sources); explicit
Host selection retains the Hook choice. Language, profile and trust controls
remain available. This is automatic selection, not unattended approval.

No receipt or public JSON schema changes. Older CLIs cannot manage the new
Host-scoped source names; keep older verified sources if reverting. This change
does not move/delete prior installations, change models, install client apps,
update the CLI binary, or establish live Host readiness. Focused checks cover
separate defaults, verified reuse, invalid/foreign data, cancellation and CAS.
