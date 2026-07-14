# Qiongli Config Windows Persistence Design

Status: approved for execution

Date: July 14, 2026

Task: `CFG-201B`

Branch: `feat/2x-native-alpha1`

PR: rolling Draft PR #63

## Decision

CFG-201B makes the existing `qiongli-config` global settings transaction
writable on supported Windows desktops without changing its schema, public
settings model, secret-reference model, or redaction contract.

The Windows adapter must provide the same externally visible revision,
rollback, conflict, and cleanup semantics as the accepted Unix adapter. It may
use different target-native primitives only where Windows requires them.

## Scope

CFG-201B includes:

- a protected current-user-only DACL for the managed `v2` directory, lock,
  settings, staging, and recovery files;
- no-follow final opens and reparse-point rejection for every existing path
  component inspected by the store;
- handle identity and hard-link-count validation for managed files;
- the existing bounded two-second exclusive lock using Rust's native Windows
  `LockFileEx` implementation;
- synchronized staging and recovery files;
- same-volume `MoveFileExW` activation and rollback with
  `MOVEFILE_WRITE_THROUGH` and replacement where required;
- byte, revision, identity, DACL, and absence verification after commit or
  rollback; and
- Windows-only tests for DACLs, reparse points, hard links, conflicts,
  concurrency, failure injection, cleanup, and redaction.

CFG-201B does not include:

- an OS keychain or any credential-value API;
- config CLI, UI, MCP, installer, or updater wiring;
- project state, 1.x import, vault fallback, or interrupted-process automated
  recovery;
- content-materializer Windows ACL changes; or
- a guarantee against an already-compromised same-user process that can change
  ACLs or race filesystem namespace operations with the Qiongli process.

## Unsafe-Code Boundary

The native workspace currently forbids unsafe code in product crates. Direct
Win32 security-descriptor creation and inspection requires FFI, so CFG-201B
adds one narrowly scoped crate:

```text
packages/qiongli-native/crates/qiongli-windows-security/
```

The crate:

- contains all Win32 FFI and exposes only safe, path-bounded operations;
- has no Qiongli schema, credential, provider, network, process-launch, UI, or
  logging knowledge;
- creates and verifies security descriptors instead of trusting inheritance;
- returns `io::Error` without embedding caller paths or private values; and
- is a target-specific dependency of `qiongli-config` on Windows.

`qiongli-config` retains its workspace `unsafe_code = "forbid"` policy. The new
crate is the demonstrated security boundary that justifies a physical split;
the workspace-wide lint is not weakened.

The DACL implementation is adapted from the accepted Rust Lite provider-config
implementation in
`packages/qiongli-lite-mcp/src/config/provider_config.rs`. Its Windows behavior
already passed the v1.19 acceptance gate; CFG-201B adds directory, identity,
hard-link, lock, recovery, and strict global-settings integration around that
primitive.

## Windows Security Descriptor

Every newly created managed directory or file receives a descriptor with:

- owner equal to the current process token user SID;
- a present, protected DACL;
- exactly one non-inherited `ACCESS_ALLOWED_ACE`;
- that ACE SID equal to the current user SID; and
- `FILE_ALL_ACCESS` as the ACE mask.

After creation and after every activation or rollback, the descriptor is read
back from the open handle using `GetSecurityInfo`. Any missing owner/DACL,
unprotected DACL, additional or inherited ACE, wrong mask, or SID mismatch is
`insecure-permissions`. No operation widens or repairs an existing insecure
managed object.

## Path And Handle Contract

Lexical root resolution remains unchanged. Windows opens existing files and
directories with `FILE_FLAG_OPEN_REPARSE_POINT`; directories additionally use
`FILE_FLAG_BACKUP_SEMANTICS`. Rust open options remove
`FILE_SHARE_DELETE` while a managed object is inspected or locked.

For every existing normal component from the selected root to `v2`, the store
checks that the opened object is a directory and not a reparse point. Managed
files must be regular, non-reparse, current-user-only objects with exactly one
hard link.

`GetFileInformationByHandle` supplies volume serial number, file index, link
count, and attributes. A path observation and its opened handle must identify
the same object before bytes are accepted. The lock identity is checked before
and after lock acquisition.

The adapter does not claim protection from a malicious process running as the
same user. It does prevent ordinary symlink/junction traversal, replacement
through a held managed handle, inherited broad ACLs, and hard-link aliases.

## Transaction Contract

The high-level transaction remains shared with Unix:

1. validate or create the compatibility chain and protected `v2` directory;
2. open/verify the protected lock and acquire the bounded exclusive lock;
3. re-read the current document under the lock and check revision;
4. encode the complete next document;
5. create, write, `sync_all`, and verify a protected staging file;
6. create, write, `sync_all`, and verify a protected recovery file containing
   prior bytes or the accepted prior-absence marker;
7. activate staging with `MoveFileExW` using
   `MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH`;
8. reopen and verify exact bytes, revision, DACL, identity, and link count;
9. remove the obsolete recovery artifact; and
10. report committed cleanup state separately if cleanup fails.

Microsoft documents `MOVEFILE_WRITE_THROUGH` as not returning until the file
is actually moved on disk. Windows therefore uses synchronized files plus the
write-through move as its activation durability boundary; it does not invent
an undocumented directory-`fsync` guarantee.

For an existing document, rollback moves the synchronized recovery file over
the live document with the same write-through replacement and verifies exact
prior bytes. For a failed first write, rollback moves the live document back
to its transaction name with write-through semantics, verifies live absence,
then removes owner-only artifacts. Any unproven rollback returns
`recovery-required` and never reports ordinary failure or success.

## Error And Privacy Contract

Win32 failures are mapped to the existing allowlisted `ConfigError` reason and
stage codes. Raw Win32 text, resolved paths, SIDs, email addresses, secret
references, and document bytes never enter public `Display`, `Debug`, status,
or receipts.

Windows status changes from `write-unsupported` to the same `missing`, `ready`,
`invalid`, `insecure`, `busy`, or `recovery-required` states used by the shared
service. No new public status field is required.

## Acceptance Criteria

CFG-201B is complete only when:

1. `qiongli-config` remains free of unsafe code;
2. the isolated adapter creates and verifies protected current-user-only
   directories and files on a real Windows runner;
3. first and replacement writes advance revisions and preserve exact bytes;
4. stale and concurrent writes produce one winner without lost updates;
5. reparse points, linked files, broad DACLs, and replaced lock identities fail
   closed without changing prior live bytes;
6. failure injection before activation preserves prior state and after
   activation restores prior bytes or prior absence;
7. rollback ambiguity produces `recovery-required` without false success;
8. every surviving transaction artifact is owner-only and only its existence,
   never its name or path, reaches status;
9. local native gates pass without Python or Node suites; and
10. exact-head Linux, macOS, and Windows native CI passes before CFG-201B is
    recorded as complete.

## Source Authority

- Microsoft `CreateFileW` documents `FILE_FLAG_OPEN_REPARSE_POINT` and
  directory opening with `FILE_FLAG_BACKUP_SEMANTICS`:
  <https://learn.microsoft.com/windows/win32/api/fileapi/nf-fileapi-createfilew>
- Microsoft `MoveFileExW` documents replacement and write-through semantics:
  <https://learn.microsoft.com/windows/win32/api/winbase/nf-winbase-movefileexw>
- Microsoft `GetSecurityInfo` documents handle-based owner and DACL reads:
  <https://learn.microsoft.com/windows/win32/api/aclapi/nf-aclapi-getsecurityinfo>
- Microsoft file-information documentation defines volume/file identity and
  link count:
  <https://learn.microsoft.com/windows/win32/api/fileapi/ns-fileapi-by_handle_file_information>
- Rust 1.97 documents that `File::try_lock` maps to
  `LockFileEx(LOCKFILE_EXCLUSIVE_LOCK | LOCKFILE_FAIL_IMMEDIATELY)` on Windows:
  <https://doc.rust-lang.org/std/fs/struct.File.html#method.try_lock>

## Approval Record

The user instructed execution of the next roadmap step on July 14, 2026. That
instruction accepts the already-roadmapped CFG-201B capability boundary. This
design does not expand the accepted scope to credentials, commands, UI, MCP,
project state, migration, or release claims.
