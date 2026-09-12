# ADR 0226: Confirmed Codex Plugin source migration

- Status: Accepted
- Date: 2026-09-12
- Task ID: `CLI-402`
- Owners: Qiongli maintainers
- Authority: the maintainer requested install-as-upgrade and reported that an
  enabled `qiongli-next@personal` prevents this flow in beta.6.
- Supersedes ADR 0225's manual-only Codex conflict handling and ADR 0224's
  rejection of other enabled Qiongli entries / no-stdin restriction only for the
  bounded official configuration calls described here. Claude stays unchanged.

The interactive guide may migrate recognized Qiongli Plugin identities to
`qiongli-next@qiongli-cli-local`. It names the prior entries before export. The
separate Host plan includes their exact enabled-flag edits and the observed user
configuration version. Declining that confirmation preserves their enabled state;
previously approved source exports may remain available.

Reuse the official Host executable/environment owner. A bounded stdio app-server
client sends only initialization, `config/read` and `config/batchWrite`; it starts
no model or research session. Read a regular, bounded user configuration and
require the selected entries to be enabled in that user layer. The batch changes
only their enabled flags to false with `expectedVersion`. Rebuild the complete
Host plan under the existing guard after confirmation. Changed configuration,
unexpected paths/identities, non-user entries and unsupported APIs refuse.

Keep previous source/cache files, including modified files. Do not uninstall an
older Marketplace Plugin or edit its catalog, cache, models or unrelated flags.
Then execute the existing verified local registration plan. Success still needs
a fresh inventory and matching cache receipt with no previous Qiongli entry
enabled. Later install/upgrade/update aliases share this same flow.

If registration fails after the old entry was disabled, report that intermediate
state and retain the files. The user can retry installation or re-enable the old
entry in Codex. There is no guessed rollback or cross-process atomicity claim.

Checks cover exact-edit scope, malformed/duplicate identities, stale configuration,
failed RPC responses and existing cache ownership. An isolated real Codex trial
must reproduce beta.6's conflict, exercise cancellation and a concurrent edit,
migrate successfully, preserve the old modified cache and unrelated configuration,
then repeat installation without further Host changes. Local checks do not prove
current user-session tool visibility or other-platform behavior.
