# ADR 0224: CLI local Plugin registration

- Status: Accepted
- Date: 2026-09-11
- Task ID: `CLI-402`
- Owners: Qiongli maintainers
- Authority: the maintainer explicitly selected CLI export followed by official
  Codex/Claude Plugin registration.
- Supersedes ADR 0222's automatic Host registration deferral for this terminal
  flow only. Signed App installation under ADR 0213 remains separate.

`install plugin|skills`, `upgrade plugin|skills` and `update plugin|skills` expose
the existing local source and managed Skills owners. Plugin destinations remain
explicit, absolute, outside reserved roots and below an existing secure parent.
Skills retain the home/current-project presets. No new distribution downloader,
package-manager launcher or filesystem replacement owner is introduced.

The terminal displays the canonical file plan and asks for confirmation. Enter,
EOF and incomplete input decline. The exact in-memory plan passes the same
version, digest, expiry, content, observation, lock and receipt checks as `app
apply`. `--dry-run` emits the existing file-plan schema; scripted apply retains
its explicit digest/approval flags and does not gain Host execution authority.

After successful Plugin export, a separate terminal confirmation binds one fixed
Host plan. It covers the discovered absolute Host executable and hash, home and
configuration root, exact source receipt, verified cache receipt, official
marketplace/Plugin observations, and ordered argument vectors. The preview has
a canonical digest and a ten-minute confirmation limit. The plan is regenerated
under the existing managed-write guard immediately before execution; drift
refuses. The Host plan is local to this interaction, not a new persisted public
plan schema or publisher attestation.

Only `qiongli-next@qiongli-cli-local` can be registered. An existing marketplace
must point at the selected local source. An enabled Qiongli Plugin from another
marketplace, duplicate identity, non-user Claude scope or malformed inventory
refuses. Replacing a cache requires the existing local-bundle verifier to prove
all files and the binary match its receipt; signed, unknown or modified caches
refuse. The fixed replacement sequence removes this verified Plugin through its
Host, then installs the refreshed source. Qiongli never edits Host cache files
directly or removes an unrelated Plugin.

Execution reuses the existing bounded native Host process runner: no shell or
stdin, fixed arguments, scoped environment, timeout and bounded output. The first
failed command stops execution. Exported source files remain available, and a
fresh retry must inspect the resulting Host state. There is no guessed rollback
or restoration of unrelated configuration.

Success requires a fresh official inventory showing the exact local Plugin
enabled, its matching source and scope, and a fully verified cache receipt equal
to the current export. This proves registration and cached content, not live
tool visibility or a research session. Users start a new Host session for Skills
and Full MCP. Configured models remain owned by the Host.

CLI self-upgrades still use the original package manager or a newly downloaded
GitHub binary. `upgrade cli` explains those commands and performs no upgrade.
Bare `install` and `update` retain their previous read-only query contracts.

Rollback removes the new terminal shortcuts; existing source exports and Host
registrations stay in place. ADR 0222's lower-level source commands remain valid.
Focused parser/confirmation/ownership checks and isolated real Host installation
and upgrade evidence are recorded in the current bounded implementation plan.
