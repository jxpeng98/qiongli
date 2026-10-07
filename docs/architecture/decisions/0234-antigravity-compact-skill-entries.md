# ADR 0234: Compact Antigravity Skill Entries

- Status: Accepted
- Date: 2026-10-07
- Task: CLI-409
- Authority: the maintainer reports an overcrowded Antigravity Skill list and
  requests an improvement to the main and secondary Qiongli entries.
- Supersedes only ADR 0233's projection of separate workflow wrapper Skills.
  Its receipt, transaction, Host registration and experimental support boundaries
  remain in force; the historical 22-entry observations remain unchanged.

The AGY adapter currently inherits Codex's generated workflow wrappers. AGY
discovers each `skills/<entry>/SKILL.md` as a public Skill, so the same Plugin
appears as a main entry and twenty research shortcuts. The official Skill
contract documents name and description metadata, not a portable visibility
flag for hiding these wrappers.

Project one research entry, `qiongli`, and retain the self-contained `no-qiongli`
reply-only entry. Remove only the generated top-level `qiongli-<workflow>` wrapper
manifests from the AGY projection. Preserve the canonical workflow library,
cards, references, translations and user variant bytes. The existing main Skill
route table selects the needed internal document. Its AGY guidance treats a
legacy shortcut name as task intent; it must not search for an uninstalled Skill
or execute that name as a program. Reply-only still takes precedence over reads.

Codex, Claude, DeepSeek and standalone Skill projections keep their existing
entry policy. The shared pack and MCP inventory do not change. AGY continues to
register one Qiongli MCP server with the retained absolute binary path. Do not
duplicate workflow instructions in the adapter or load all resources at startup.

Keep the existing receipt schema and strict tree verification. A verified older
22-entry AGY source/cache remains readable and removable. A confirmed update
stages the compact projection and replaces the old source through the existing
receipt/CAS owner, then uses the official AGY manager to replace its cache.
Unknown files, modified wrappers and stale receipts still refuse replacement.
Do not manually delete wrappers from installed or cached directories: that
would invalidate their receipts. Reopen the Host session after an update.

Regression checks cover the compact public entry set, unchanged internal
workflow bytes, the separate reply-only entry, unchanged Codex shortcuts, and
legacy-source update with drift/stale-receipt refusal. An isolated official
manager observation may establish registration and cache replacement; it does
not establish GUI rendering, model routing or a complete research session.

Rollback reverts only the AGY projection and entry guidance; the same verified
update owner can restore the older wrapper layout. No profile cleanup, credential
access or user-model change is part of this decision.

References: [AGY Skills](https://antigravity.google/docs/skills) and
[Plugin format](https://www.antigravity.google/docs/plugins?tab=cli).
