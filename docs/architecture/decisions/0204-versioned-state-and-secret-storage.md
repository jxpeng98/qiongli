# ADR 0204: Versioned State And Secret Storage

- Status: Accepted
- Date: 2026-07-11
- Task ID: `ARC-201D`
- Owners: Qiongli maintainers
- Decision scope: Qiongli 2 global config, project state, secrets, migration, and recovery

## Context

Qiongli 2 must coexist with the accepted 1.x installation while the native
runtime is still proving parity. The native CLI, desktop application, MCP
profiles, installer, and orchestrator all need the same state semantics. A
partial migration, concurrent writer, or failed update must not corrupt a
research project or make the 1.x recovery path unusable.

The current 1.x product has global files under `QIONGLI_CONFIG_HOME` and
project files under `<project>/.qiongli/`. Some legacy configuration can contain
credential values. Qiongli 2 needs explicit schemas and safe imports without
rewriting those sources in place or placing secrets in ordinary configuration,
logs, diagnostics, fixtures, or backups.

The `v2` directory name is a product-state namespace, not a document schema
number. Documents within that namespace will evolve independently and need an
explicit `schema_version` so a newer document is never interpreted as an older
shape.

## Decision drivers

- preserve a byte-equivalent 1.x recovery path throughout the alpha period;
- isolate global machine/user state from project-owned and potentially shared
  state;
- make every mutation durable, atomic, version-aware, and testable under fault
  injection;
- make preview, import, retry, rollback, and downgrade behavior unambiguous;
- keep credential values out of config and project files by construction;
- prefer native operating-system credential protection without making a
  keychain-less host silently fall back to plaintext;
- support the same contract on macOS, Windows, and Linux without a Python,
  Node.js, or Rust installation on the user's machine.

## Decision

### State roots and ownership

When `QIONGLI_CONFIG_HOME` is set, its existing absolute or home-relative path
remains the compatibility root and Qiongli 2 global state lives only under
`$QIONGLI_CONFIG_HOME/v2/`. When it is not set, the compatibility root remains
`<platform-user-home>/.config/qiongli/` and Qiongli 2 appends `v2/`. Platform
APIs resolve the user home without invoking a shell. Moving the default to a
different platform directory requires a later migration ADR; an implementation
must not silently choose a second root. The resolved path is reported by
`qiongli doctor` in redacted form and must be stable across CLI, desktop, MCP,
installer, and updater processes.

Global state and project state are separate stores:

- user- or machine-scoped config, provider definitions, integration markers,
  migration receipts, and recovery metadata live in the resolved global `v2/`
  root;
- project-owned state lives under `<project>/.qiongli/v2/` and is never copied
  into the global store as an implementation shortcut;
- project settings may refer to a global profile or credential by an opaque
  identifier, but they do not contain a credential, keychain locator with user
  identity, or unnecessary absolute path;
- merge precedence is calculated in memory. A project override does not rewrite
  its global source, and a global update does not rewrite a project file.

All Qiongli-owned mutable documents are UTF-8 JSON objects with a stable
`document_kind` and positive integer `schema_version`. The `v2` namespace
starts document schemas at the version appropriate to each kind; it does not
force every schema to be version 2. Unknown document kinds, malformed data,
and schema versions newer than the running binary fail closed and remain
unmodified.

### Atomic mutation and concurrency

`qiongli-config` is the only crate allowed to persist Qiongli state. A write
must:

1. acquire a bounded per-store lock and verify the expected document revision;
2. serialize and validate the complete replacement in memory;
3. create an owner-only temporary file in the destination directory;
4. write all bytes, flush them, and call `fsync` or the platform durability
   equivalent on the temporary file;
5. atomically replace the destination with same-filesystem rename or the native
   atomic-replace API; and
6. synchronize the containing directory where the operating system supports
   that operation before reporting success.

No caller observes or adopts a partially written document. A revision mismatch
returns a conflict instead of losing another writer's update. Startup ignores
uncommitted staging files and reports them through recovery diagnostics.

### Copy-on-migrate

Migrations are ordered, forward-only functions between known schema versions.
Import from 1.x and every later state migration use the same transaction model:

1. inventory and validate source paths without following symlinks;
2. produce a read-only preview of creates, replacements, secret imports, and
   unsupported or deferred records;
3. after explicit approval where required, create a checksummed snapshot of
   every pre-existing target that could change and record source hashes;
4. transform into a transaction-specific staging directory under the target
   `v2` namespace;
5. validate all schemas, references, permissions, and secret-store writes;
6. atomically activate the staged documents; and
7. write a committed migration receipt last.

Every secret-store mutation is journaled before it is attempted. Imports create
a new transaction-owned secret entry instead of overwriting an existing entry.
If later validation or activation fails, rollback deletes only entries whose
backend-owned entry version and transaction marker still match the journal. The
journal never persists a stable unkeyed digest or other reusable fingerprint of
the credential value. A backend without a safe ownership/version check cannot
automatically delete the entry; it quarantines the reference and records a
redacted recovery action instead. Existing and legacy credentials remain
unchanged until a separately approved cleanup transaction.

Legacy global and project sources are never modified by preview or import.
During alpha, readers prefer committed v2 state and may read legacy data only
through the explicit compatibility/import layer; all writes go to v2. A v2
document from an incomplete transaction is not eligible for fallback reads.

A migration identity contains the migration ID, source digest, target document
kind, and target schema version. Repeating an accepted migration with the same
identity is a no-op that verifies the existing receipt and hashes. A changed
source produces a new preview; it is never silently folded into an old receipt.

Snapshots have a manifest, content hashes, state, and retention policy. A
snapshot that contains legacy secret-bearing material must use the same
keychain or encrypted-vault protection as live secrets; plaintext duplication
of a legacy credential file is prohibited. Failure before the committed
receipt restores all prior targets from the verified snapshot and removes the
uncommitted staging tree. Initial 1.x import can always recover by discarding
v2 output because its legacy source remains untouched.

### Secret references and fallback

Ordinary config and project state store only an opaque `secret_ref`. They never
store a secret value, an encryption key, or reversible credential material.
`qiongli-config` resolves the reference through a secret-store facade and
returns secret bytes only to the bounded operation that requested them.

The default backend is the operating-system credential service: Keychain on
macOS, Credential Manager on Windows, and an available Secret Service provider
on Linux. Importing a legacy credential requires user approval, a successful
write, and a read-back verification before its `secret_ref` can be committed.
The legacy source is retained until a separate previewable cleanup is approved.

If no usable operating-system service exists, Qiongli fails closed until the
user explicitly enables the passphrase-vault fallback. The fallback uses a
reviewed memory-hard password KDF and authenticated encryption from maintained
Rust cryptography crates. Its random salt, KDF parameters, nonce, and encrypted
payload may be stored in an owner-only vault, but the derived encryption key
and passphrase are never persisted. They exist only in locked process memory
for the shortest practical lifetime. The user must unlock the vault through a
separate interactive CLI or desktop action; an MCP stdio request cannot prompt
or silently downgrade storage.

No fallback may place an encryption key beside its ciphertext, hide a plaintext
secret in encoded config, or use a machine identifier as a key. If the secure
backend cannot be initialized or unlocked, the secret-dependent operation is
unavailable and status returns a redacted remediation code.

## Alternatives considered

### Rewrite 1.x files in place

This minimizes duplicate state, but a failed alpha migration would also damage
the downgrade path and could expose partially upgraded documents to 1.x.
Rejected.

### Use one unversioned platform directory

An unversioned directory makes new and old writers indistinguishable and turns
rollback into a document-by-document downgrade. Rejected in favor of a
version-scoped product namespace plus per-document schema versions.

### Keep global and project state in one database

A single database simplifies transactions but moves project portability,
ownership, backup, and sharing into a machine-scoped opaque store. It also
widens the blast radius of corruption. Rejected; implementations may use an
internal database only after a superseding ADR preserves the two-store
boundary and migration guarantees.

### Store credentials directly in config

Plaintext, reversible encoding, and environment-expanded config are easy to
implement but leak through source control, diagnostics, backups, and support
bundles. Rejected.

### Encrypt secrets with a key stored in the same directory

This protects against accidental text search but not against theft of the
config directory. It creates the appearance of security while packaging the
key and ciphertext together. Rejected.

### Require an OS keychain with no fallback

This is secure on supported interactive desktops but excludes legitimate
headless Linux and recovery environments. Rejected in favor of an explicit,
passphrase-derived fallback that never persists its decryption key.

## Consequences

Positive consequences:

- 1.x and 2.x can coexist without either runtime interpreting the other's
  partially migrated state;
- the same write, migration, and recovery code serves CLI, desktop, MCP,
  installer, and updater callers;
- schema and transaction receipts make import repeatable and supportable;
- project files remain portable and reviewable without containing secrets;
- keychain failure is visible and actionable rather than causing a silent
  plaintext downgrade.

Costs and limitations:

- atomic replacement, locking, directory synchronization, permissions, and
  keychain behavior require target-native implementations and receipts;
- copy-on-migrate temporarily consumes additional disk space;
- the passphrase fallback requires an explicit unlock and cannot provide
  unattended startup unless a separately approved secure unlock mechanism is
  available;
- forward-only schemas mean rollback restores a snapshot or selects an older
  namespace; an older binary does not edit a newer document.

## Security and privacy

- Global directories and snapshots use owner-only access; Unix acceptance
  requires restrictive modes and Windows acceptance requires an equivalent
  owner-restricted DACL.
- Path resolution rejects relative roots, traversal, symlink substitution,
  reparse-point escape, and writes outside the selected global or project
  store.
- Logs, status, diagnostics, crash reports, receipts, and fixtures use an
  allowlisted redacted representation and never serialize resolved secrets.
- Snapshot manifests avoid unnecessary absolute project paths and account
  identifiers. Secret references are opaque and non-derivable.
- Secret bytes are not cached in view models or persisted task state and are
  zeroized where supported by the selected secret and cryptography crates.
- A migration never deletes legacy credentials automatically. Cleanup is a
  separate transaction after the new reference has been verified.

## Rollback

Before activation, the migration engine verifies that every changed target has
a restorable snapshot. On any validation, secret-store, write, synchronization,
or receipt failure, it restores the previous v2 bytes and permissions, removes
matching transaction-created secret entries, verifies the restored hashes and
secret journal, and leaves the legacy source untouched. If a secret entry
cannot be safely removed, it remains quarantined and unreferenced with a
redacted recovery receipt. A recovery command can complete the same
compensation after an interrupted process.

Rolling back the product switches host registrations to the accepted 1.x
runtime and uses the original 1.x global and project state. It does not attempt
to downgrade v2 documents in place. A later 2.x run may resume only from a
verified committed receipt or repeat the idempotent import after removing an
uncommitted transaction.

## Acceptance tests

1. Config-home fixtures resolve the same global `v2` root from CLI, desktop,
   MCP, installer, and updater code on macOS, Windows, and Linux, including the
   `QIONGLI_CONFIG_HOME` override.
2. Global and project fixtures prove that writes stay in their own stores,
   precedence is computed without source rewrites, and 1.x paths remain
   byte-identical.
3. Every mutable document carries `document_kind` and `schema_version`;
   malformed, unknown, and future-version documents fail closed without a
   write.
4. Fault injection at every write, flush, replace, directory-sync, and receipt
   boundary leaves either the complete previous state or complete new state,
   never a parseable partial document.
5. Concurrent writers exercise the lock and revision check and cannot produce
   a lost update.
6. Import preview has no writes. Accepted import creates a verified snapshot,
   commits the receipt last, is a no-op on identical replay, and produces a new
   preview when a source digest changes.
7. Failure at every migration step restores byte-equivalent target state,
   removes uncommitted output, and leaves all legacy sources untouched.
8. Target-native keychain tests write, read back, rotate, and delete a test
   secret while config, output, logs, snapshots, and fixtures contain only its
   opaque `secret_ref`.
9. Fallback tests require explicit enablement and unlock, verify authenticated
   encryption, prove that neither the passphrase nor derived key is persisted,
   and fail closed for a corrupt or unavailable vault.
10. Failure after each keychain/vault write removes the matching
    transaction-created entry or quarantines it unreferenced; pre-existing and
    legacy credentials remain byte- and identity-equivalent.
11. Unix mode and Windows DACL receipts prove owner-only access; path-safety
    tests reject symlink, reparse-point, traversal, and out-of-root attacks.
12. Secret and private-path canaries do not appear in status, diagnostics,
    crash fixtures, migration receipts, or repository output.

## Follow-up tasks

- `FND-201`: provide the shared platform path, durability, lock, and permission
  abstractions.
- `CFG-201`: implement global schemas, the secret-store facade, redaction, and
  atomic persistence.
- `CFG-202`: implement project schemas, copy-on-migrate, receipts, snapshots,
  idempotency, and recovery fixtures.
- `PLT-201`: make installer markers reference versioned state without taking
  ownership of config writes.
- `QAT-201`: run migration, keychain, permissions, fault-injection, and
  zero-runtime process audits on every Tier 1 target.
