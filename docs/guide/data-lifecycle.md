# Data Ownership and Lifecycle

This policy covers the current native Qiongli 2 product. Qiongli keeps its
project and product state local. Agent Hosts and remote providers keep their
own records under their own policies.

## Ownership Boundary

| Data | Owner and location |
| --- | --- |
| Project files and private Qiongli state | The user owns the project directory, including `<project>/.qiongli/v2`. |
| Global Qiongli 2 state | The user owns the resolved v2 root: `<user-home>/.config/qiongli/v2` by default, or `$QIONGLI_CONFIG_HOME/v2` when configured. |
| Provider credentials | The user owns the credential. On supported systems Qiongli stores the secret in the operating-system credential store and keeps only an opaque reference in native configuration. If no supported store is available, Qiongli fails closed instead of writing the secret to ordinary configuration. |
| Plugins, Skills, CLI files, and client entries | The relevant Agent Host or Qiongli install receipt owns the installed integration state. |
| Agent Host chats and transcripts | The Agent Host owns these records; they are not Qiongli project data. |
| Native App All Chat history (development preview) | The user owns `<project>/.qiongli/all-chat/run_*.json`. These private logs are separate from research artifacts and accepted evidence. |
| Remote provider records | The provider owns retention and deletion behavior for its service. |

## Backup and Restore

For a complete recoverable checkpoint:

1. Stop the App, CLI, MCP servers, and agents that may write Qiongli state.
2. Back up every full project directory, including hidden files and
   the whole `<project>/.qiongli` directory, including `<project>/.qiongli/v2` and All Chat history.
3. Back up the full resolved global Qiongli 2 root: either
   `<user-home>/.config/qiongli/v2` or `$QIONGLI_CONFIG_HOME/v2`.
4. Back up provider credentials separately using the operating system's secure
   credential export or recovery process.
5. Keep project and global copies from the same stopped checkpoint and verify
   that the backup can be read.

Restore the project directories and global root while Qiongli is stopped. Start
Qiongli, register any project missing from the Research Library through the
normal project workflow, and allow rebuildable derived indexes to be rebuilt.
Qiongli does not currently provide a one-command whole-product backup, restore,
or purge operation.

### Private App conversation history

The native development App saves explicit prompts, supplied context and source
labels, bounded public Agent activity, permission outcomes and lifecycle events
in version 1 logs under `<project>/.qiongli/all-chat/run_*.json`. User-supplied
text can itself contain private information; protect these logs as research data.
The App does not collect credentials, hidden reasoning or raw tool payloads into
the log. Agent Host history remains owned by that Host. Browser fixtures are
in-memory previews and do not provide restart recovery.

The newest session is displayed after restart. Unfinished work becomes
interrupted; saved prompts, permission choices and project writes are never
automatically replayed. Load/resume is currently unavailable. Starting a new
session preserves earlier files; browsing archives is not yet exposed in the App.
Retention is explicit: no automatic expiry or eviction, at most **32 sessions per
project**, **64 turns**, **2,048 public updates**, **2,304 log records** and **8 MiB
per log**. At capacity, writing stops without deleting old history. Corrupt,
truncated or unsupported logs block recovery/new sessions and preserve original
bytes. Stop the App and verify a full backup before inspecting those files.

The existing private directory/file modes, atomic replacement and digest CAS
protect writes. `.qiongli/.all-chat-session.lock` holds the OS writer lease;
`.qiongli/.all-chat.lock` serializes file operations. Leftover lock files are not
live sessions; do not delete locks to bypass a running owner. Copy and restore
history with all writers stopped. For deliberate deletion, remove only the
chosen `run_*.json` files from the backed-up project while the App is stopped;
restart then loads the newest remaining valid log. Uninstall does not purge them.
History content is excluded from portable project export and product diagnostics;
private snapshot/debug formatting also omits conversation content.

## Portable Project Export

Use portable export to move a privacy-filtered project snapshot between
machines:

```bash
qiongli project export preview --project-id PROJECT_ID --destination DESTINATION
qiongli project export apply --project-id PROJECT_ID --destination DESTINATION \
  --expected-plan-digest DIGEST --approve-filesystem-write
```

The result contains `qiongli-portable-project.json` and a `project/` directory.
It is **not a complete backup**. It excludes Qiongli private state, absolute
paths, client configuration, credentials, sessions, chats, conversations,
transcripts, Git metadata, dependency/build/cache directories, `.env` files,
and recognized secret or private-key files. Use the complete checkpoint above
when recovery, rather than exchange, is the goal.

## Uninstall and Deletion

- App **Remove selected** removes only the selected Qiongli-owned client
  integration state.
- App **Remove CLI** removes receipt-owned CLI files or restores the exact
  predecessor recorded by the receipt.
- Agent Host marketplace managers remove the plugin or Skill state they own.
- Legacy `qiongli remove` removes only the selected CLI-managed assets.
- Unregistering a project removes its Research Library registration, not its
  directory.

These operations do not delete project directories, the global Qiongli 2 data
root, Agent Host chats, operating-system credentials, or remote provider
records. Retaining data and uninstalling software are separate decisions.

After making and verifying a backup, deliberate deletion means removing only
the exact project directories, resolved global v2 root, secure credentials,
Host-owned records, and provider records that the user has chosen to erase.
Follow the relevant Host, operating-system, and provider deletion controls. Do
not use broad recursive cleanup commands. Preserve required 1.x sources and
state while migration or rollback is still needed.

## Qiongli 1.x End of Support

`v1.19.0-beta.1` is the accepted final feature-bearing 1.x release. The planned
1.x support window ends **90 days after Qiongli 2 Stable is published**. Alpha,
Beta, this policy, and ordinary source merges do not start that clock, so there
is no calendar end date yet.

During that window, 1.x remains limited to approved critical security or
release-breakage fixes; it does not resume normal feature development. The
separate REL-906 migration and rollback runbook will cover operational transfer
between 1.x and 2.x. End of support does not automatically delete user data.
See the [release branch policy](/maintainer/release-branch-policy) for the
maintenance authority.

### Selected research comparison preview

The development excerpt comparison stores the native source manifest (selected
text, line ranges, source/method digests) in the existing private prompt context.
Scripted candidate output is visible in public Agent activity. These contain
research content and follow the conversation retention/deletion policy above.
No source is sent to a model by the offline fixture. After App restart, history
can be read, but unsaved candidate authority and source authorization are not
restored; start a new comparison before submitting another candidate.
Capture intake and subsequent academic consolidation use existing project storage.
Deleting a chat log does not delete those saved project records.
