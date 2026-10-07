# ADR 0235: Install Plugins For Detected Hosts

- Status: Accepted
- Date: 2026-10-07
- Task: CLI-402
- Authority: the maintainer requests a single all-install entry and confirms
  that it should cover every already-installed supported client, with Skills
  and MCP supplied by each Plugin.
- Supersedes only ADR 0225's unconditional all-Host selection. Its source,
  approval, receipt, conflict and cancellation boundaries remain unchanged.

Add `qiongli install all`, its existing update/upgrade equivalents, main guide
choice 5 and Plugin Host choice 6/all. These entries and `--target all` select
the supported Host CLIs found through the existing read-only executable lookup:
Codex, Claude Code, DeepSeek Harness and Antigravity, in that order. Report the
selected set and skipped missing CLIs before starting installation. If no client
is found, return an actionable error and perform no install. Detection does not
execute the clients or establish that their versions and configuration are valid.

Explicit Host lists remain strict, including 3/both for Codex and Claude. For the
all preset, validate options against the full supported set before filtering;
detecting only one client does not authorize a shared destination or Host-specific
Hook flags. All is terminal-only and supplies no new scripted batch plan.

Pass the detected set to the existing sequential Plugin installers. Each Plugin
includes Skills and Full MCP, so no extra standalone export or MCP registration
is added. Keep the existing per-Host source and registration approvals and stop on a selected
Host's cancellation, conflict or failure, retaining completed steps. Missing
clients are skipped only during selection; subsequent failures are not ignored.
The guide continues to choose or reuse each Host's source through its existing
owner. It does not install client applications or choose a model.

Checks cover detection order and missing-client reporting, empty selection,
strict explicit lists, incompatible/duplicate options, terminal-only behavior
and cancellation without writes. Rollback removes the new entries and restores
the earlier all-selection behavior without changing installed Plugins. No
receipt, public App schema or per-Host registration command changes are required.
