# ADR 0225: Discoverable CLI content upserts

- Status: Accepted
- Date: 2026-09-11
- Task ID: `CLI-402`
- Owners: Qiongli maintainers
- Authority: the maintainer requested discoverable installation, simpler upgrades
  and using install to update existing content.
- Supersedes ADR 0224 only for optional interactive destination selection and
  the bare terminal install/upgrade entry; all write/trust boundaries remain.

Bare install and upgrade open the existing guide in a terminal without explicit
output flags. Redirected install and explicit --json/--text retain the inventory;
redirected upgrade retains help. Bare update keeps managed status. No script gains
implicit prompting, approval or mutation.

Interactive install/upgrade/update plugin select the same upsert operation. A
missing Host is selected in the guide. The selected Host's official local
marketplace inventory supplies the prior source directory; duplicate, non-local,
invalid or conflicting observations refuse. Its path only selects existing
source/receipt validation and grants no write authority. An explicit destination
remains available. A new export requires the same absolute path, fixed basename,
reserved-root exclusion and existing secure parent. No search of research folders,
new source registry or automatic directory cleanup is added.

--target all processes Codex and Claude with distinct destinations and separate
file/Host confirmations. Cancellation or failure stops subsequent Hosts; completed
steps are retained and reported. There is no cross-Host atomicity claim. A single
explicit destination for all Hosts refuses. Scripted --dry-run still requires one
explicit Host and destination and emits the unchanged plan schema.

Foreign enabled Qiongli Plugins are named before export, with manual Host disable
guidance. They are never disabled, uninstalled or archived by this flow. Codex's
current CLI has no standalone Plugin-disable subcommand; remove is not a substitute
because it deletes the cache. Post-export registration still revalidates all Host
observations, approvals and verified replacement-cache ownership under ADR 0224.
A skipped registration is incomplete; a successful one reports source/cache paths
and explains that the Host loads Skills there, without copying into .agents.
Doctor/inventory establish registered bytes, not a live research session.
