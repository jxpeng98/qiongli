# Qiongli Config Global Settings Design

Status: approved for implementation planning on July 14, 2026

Task: `CFG-201A`, the first vertical slice of `CFG-201`

Design baseline: `be46217465fb0b82564d42a0d128cff0e8414d0e`

Rolling branch and review surface: `feat/2x-native-alpha1`, Draft PR #63

## Goal

Create the first Qiongli 2 state service as a small Rust-native boundary that:

- resolves the accepted Qiongli 2 global config root;
- owns one versioned global settings document;
- stores typed profile and provider settings without storing credential values;
- detects concurrent updates through an expected revision;
- persists owner-only state atomically on macOS and Linux;
- exposes diagnostics that cannot disclose private settings, secret references,
  credential values, or concrete user paths; and
- compiles and behaves deterministically without Python, Node.js, Rust, a
  source checkout, a network service, or an available credential backend on
  the user's machine.

This slice establishes a trustworthy state contract for the later native CLI,
desktop UI, Lite MCP runtime, installer, and orchestrator. It does not connect
those surfaces yet.

## Accepted Scope Boundary

`CFG-201A` includes:

- a new `packages/qiongli-native/crates/qiongli-config` workspace crate;
- `QIONGLI_CONFIG_HOME` compatibility-root resolution with the required `v2/`
  namespace;
- a single `settings.json` document with a stable kind, schema version, and
  revision;
- typed selection of the `skill-only`, `marketplace-lite`, or `full` profile;
- typed public settings for OpenAlex, Semantic Scholar, Crossref, PubMed, and
  arXiv;
- a strict opaque `SecretRef` type for API-key slots;
- a read-only secret-store facade with an explicit unavailable implementation;
- bounded reads, validation, optimistic concurrency, atomic replacement,
  rollback, and redacted status;
- secure persistent writes on macOS and Linux; and
- read/schema/path behavior plus fail-closed write behavior on Windows.

The following remain outside this slice:

- real Keychain, Credential Manager, Secret Service, or encrypted-vault
  backends;
- accepting, importing, returning, rotating, or deleting credential values;
- 1.x `providers.json` reads, environment-secret fallback, or migration;
- project state, snapshots, migration receipts, and recovery commands;
- `config show`, `config set`, `status`, or `doctor` command wiring;
- UI, MCP, provider-network, installer, updater, or orchestrator integration;
- Windows persistent state mutation; and
- automatic repair of malformed, future-schema, insecure, or ambiguous state.

No code in this slice modifies a 1.x file or treats a 1.x file as Qiongli 2
state.

## Chosen Architecture

### One document, one writer

The global settings store owns exactly one mutable document:

```text
<compatibility-root>/v2/settings.json
```

`qiongli-config` is the only crate allowed to create or replace this document.
Callers submit a complete typed replacement with the revision they previously
read. The store validates and serializes the complete next document and then
commits it as one transaction.

One document is preferred over separate profile and provider documents because
the first product flows update both concepts together. Splitting them now would
require a multi-document transaction or permit internally inconsistent reads.
An embedded database is also unnecessary for this bounded, reviewable state
and would add a file format, dependency, and migration boundary before they are
needed.

### Crate boundary and dependencies

The native workspace adds:

```text
packages/qiongli-native/crates/qiongli-config/
  Cargo.toml
  src/
    lib.rs
    document.rs
    error.rs
    path.rs
    redaction.rs
    secret.rs
    store.rs
```

The module names describe responsibilities, not mandatory public modules. The
implementation may keep them private and re-export only the supported service
types.

`qiongli-config` may depend on `qiongli-content` for the canonical `ProfileId`
contract. It must not depend on the native application, UI, runtime, platform,
or execution crates. The application does not gain a dependency on
`qiongli-config` until command composition is implemented in a later slice.

Local crate code continues to forbid `unsafe`. Target-native operations may use
maintained safe Rust dependencies whose scope and features are pinned in the
workspace lockfile.

## Config Root Contract

The resolver accepts two injected inputs so tests never mutate process-global
environment or depend on a developer's home directory:

- the raw optional `QIONGLI_CONFIG_HOME` value as an `OsStr`; and
- the platform user-home path supplied by the composition layer.

The production composition layer reads the environment once and obtains the
home using a platform API or maintained platform-path adapter. It never invokes
a shell or external process.

Resolution follows ADR 0204 exactly:

1. If the override is absent, the compatibility root is
   `<platform-user-home>/.config/qiongli/`.
2. If the override is absolute, it is the compatibility root.
3. If the override is exactly `~`, it resolves to the platform user home.
4. If it starts with the platform form of `~/`, its suffix is joined to the
   platform user home.
5. Any other relative value, an empty override, a missing or relative user
   home, a root/prefix in the home-relative suffix, or `.`/`..` traversal is an
   error.
6. The resolver always appends one literal `v2` component. It does not guess
   that an override ending in `v2` already contains the namespace.

Absolute non-UTF-8 paths remain usable on platforms that support them. Paths
are kept as `PathBuf` values and never round-tripped through JSON or lossily
converted for diagnostics. Home-relative syntax must be valid Unicode because
the `~` marker itself is a textual compatibility syntax.

Resolution is lexical and has no filesystem side effects. Before any read or
write, the store rejects a symlink or reparse point from the selected
compatibility root through the managed `v2` path, and for `settings.json`, the
lock, staging, or recovery artifact. Persistent writes are anchored to the
validated store directory and cannot select an arbitrary caller-provided
destination.

## Global Settings Schema V1

The UTF-8 JSON document has this exact logical shape:

```json
{
  "document_kind": "qiongli-global-settings",
  "schema_version": 1,
  "revision": 1,
  "default_profile": "marketplace-lite",
  "providers": {
    "openalex": {
      "enabled": false,
      "email": null,
      "api_key_ref": null
    },
    "semantic_scholar": {
      "enabled": false,
      "api_key_ref": null
    },
    "crossref": {
      "enabled": false,
      "email": null
    },
    "pubmed": {
      "enabled": false,
      "api_key_ref": null
    },
    "arxiv": {
      "enabled": true
    }
  }
}
```

The persisted document rules are:

- `document_kind` is exactly `qiongli-global-settings`;
- `schema_version` is exactly integer `1` for this binary;
- `revision` is an integer from `1` through `9,007,199,254,740,991`;
- `default_profile` uses the existing `qiongli-content::ProfileId` serialized
  values;
- all five named provider objects are present exactly once;
- all fields shown for each provider are present, including explicit `null`
  optional values;
- unknown root fields, unknown providers, unknown provider fields, duplicate
  JSON keys, non-finite or fractional numbers, and type mismatches fail closed;
  and
- serialization is deterministic pretty JSON with a single trailing newline.

The store applies a 64 KiB maximum to the complete settings document before
parsing. This is far above the valid typed representation but prevents an
unbounded local read. A persisted document that exceeds the bound is invalid
and remains unchanged.

When `settings.json` does not exist, `load` returns an in-memory default at
virtual revision `0`; it does not create directories or a file. The default is
`marketplace-lite`, with arXiv enabled and every provider that needs setup
disabled. The first accepted replacement of that virtual state is revision
`1`.

### Provider semantics

The schema deliberately separates user intent from readiness:

- `enabled` records whether the user wants the provider considered;
- OpenAlex is ready when enabled and `api_key_ref` is present; its email is
  optional;
- Semantic Scholar is ready when enabled and `api_key_ref` is present;
- Crossref is ready when enabled and its email is present;
- PubMed is ready when enabled and `api_key_ref` is present;
- arXiv is ready whenever enabled; and
- an enabled but incomplete provider remains valid config and reports a
  redacted `needs-secret` or `needs-public-setting` status.

Email values are non-secret but private settings. They are trimmed on typed
construction, must contain between 1 and 320 Unicode scalar values, and must
not contain control characters. This slice does not claim complete RFC mailbox
validation. Raw emails never appear in status, `Debug`, error text, logs, or
test failure snapshots.

The document model is private to the crate. Public settings/view types do not
offer an unrestricted serializer. Types containing emails or secret references
either omit `Debug` or implement a redacted `Debug` representation.

## Secret Boundary

### Opaque references only

An API-key slot stores `Option<SecretRef>`, never `String` and never secret
bytes. The serialized reference format is:

```text
qsr1_<32 lowercase hexadecimal characters>
```

References created by a future secure backend must fill the 128-bit identifier
with cryptographically secure random bytes. `CFG-201A` validates syntax but
does not claim that it can prove the entropy of an externally parsed fixture.
The value carries only a Qiongli reference format version; it does not encode a
provider, backend, account, username, filesystem path, keychain locator, or
credential fingerprint. The typed settings API accepts `SecretRef`, not an
unvalidated credential string, and reference parsing rejects every value
outside this narrow grammar.

`SecretRef` supports equality and serialization for the private document
adapter, but its `Debug` and `Display` representations are always redacted.
The first slice parses and holds references but does not generate one because
no secure backend can yet prove a successful secret write and read-back.

### Facade without a fake secure backend

The crate defines the read-side facade needed by later provider resolution:

```text
SecretStore
  status()  -> SecretStoreStatus
  resolve(secret_ref) -> Result<SecretValue, SecretStoreError>
```

`SecretValue` is a non-serializable, non-cloneable, non-debuggable byte
container with a 16 KiB maximum. A real implementation must zeroize its storage
on drop. No real `SecretValue` is constructed in this slice.

The only implementation in `CFG-201A` is `UnavailableSecretStore`. It reports
the stable remediation code `secure-store-not-implemented`; every resolution
returns a typed unavailable error without filesystem, process, prompt, or
network side effects. It is not a memory store, plaintext store, environment
fallback, or security claim.

Write, rotate, and delete capabilities will be introduced as a separate
additive facade when a real target-native backend is implemented. Config
mutation will only accept the resulting `SecretRef` after backend write and
read-back verification.

## Service API And Data Flow

The supported service concepts are:

```text
ConfigRootResolver::resolve(inputs) -> ConfigRoot
GlobalSettingsStore::load() -> LoadedGlobalSettings
GlobalSettingsStore::replace(expected_revision, replacement) -> CommitOutcome
GlobalSettingsStore::status() -> RedactedConfigStatus
```

`LoadedGlobalSettings` contains the typed settings and observed revision.
`replace` never accepts a caller-selected output path or revision for the new
document. The store compares `expected_revision` to the current persisted or
virtual revision while holding the lock, increments it exactly once, and
returns the committed revision. Revision exhaustion fails before mutation.

The mutation flow is:

```text
typed replacement
-> validate fields and references in memory
-> resolve and validate the owned store
-> acquire bounded per-store lock
-> bounded read and validate current document
-> compare expected revision
-> serialize and parse-check complete next document
-> write and synchronize owner-only staging state
-> atomically replace settings.json
-> synchronize the store directory
-> return the committed revision
```

Callers cannot patch arbitrary JSON fields. A future CLI or UI may offer
field-level intents, but the service converts those intents into a complete
typed replacement before reaching the persistence boundary.

## Atomicity, Permissions, And Recovery

### Lock and revision

The store uses one stable owner-only lock artifact inside the `v2` directory
and a target-native advisory exclusive lock. Lock acquisition is bounded; the
production default is two seconds. Timeout returns `lock-busy` and performs no
document mutation. Tests use an injected lock/clock boundary rather than
sleeping for two seconds.

The current document is read only after the lock is held. A mismatch between
its revision and `expected_revision` returns `revision-conflict` with the
observed numeric revision but no document values. It never retries by silently
overwriting another writer.

### Unix persistence

On macOS and Linux:

- a newly created `v2` directory is mode `0700`;
- the lock, staging, recovery, and final document are mode `0600`;
- an existing managed directory or file with group/other access fails with
  `insecure-permissions` rather than being silently weakened or adopted;
- managed paths are no-follow and must be owned by the effective user;
- staging and recovery names are transaction-unique and created with
  create-new semantics in the destination directory;
- complete bytes are written, flushed, and synchronized before activation;
- replacement uses same-filesystem atomic rename; and
- the containing directory is synchronized before a normal success result.

If a current document exists, the transaction preserves an owner-only,
synchronized recovery copy before activation. Failure before activation leaves
the current bytes untouched. A failure after activation attempts an atomic
rollback and verifies the restored revision and bytes before returning the
original stage error. For a first write, rollback restores the prior absence of
`settings.json` and synchronizes that removal.

If rollback itself cannot be proven, the store returns the distinct
`recovery-required` state, retains only owner-only transaction artifacts, and
does not claim either success or ordinary rollback. Readers do not adopt a
staging or recovery artifact as live state. Automated recovery of that rare
double-failure state belongs to the later recovery slice.

After a durable commit, cleanup failure for a no-longer-needed recovery artifact
does not roll back the committed document. The commit outcome carries a
redacted `cleanup-required` flag, and status can report that flag without an
artifact name or path. This makes the commit point explicit instead of
returning a generic error after state has already changed.

### Windows boundary for `CFG-201A`

Windows builds support pure path resolution, typed defaults, document
serialization/validation, bounded reading of an existing regular
`settings.json`, secret-reference parsing, and redacted status.

Persistent mutation is intentionally unavailable until a safe adapter can
prove owner-restricted DACLs, no-follow/reparse safety, exclusive locking,
native atomic replacement, and directory durability. On Windows, `replace`
returns `unsupported-platform-security` before creating a directory, lock,
staging file, recovery file, or settings document.

This is a temporary first-slice boundary, not a Qiongli 2 release non-support
decision. `CFG-201B` will add and separately accept the Windows persistence
adapter. No alpha capability may claim Windows config writes before that gate
passes.

## Read And Error Semantics

Every read selects only `settings.json`; transaction artifacts are ignored as
live documents. These conditions fail closed and never trigger repair or
replacement:

- wrong or unknown `document_kind`;
- zero, negative, malformed, or future `schema_version`;
- zero, malformed, duplicate, or exhausted persisted revision;
- malformed UTF-8 or JSON, duplicate keys, unknown fields, or invalid values;
- oversized input;
- non-regular, symlinked, reparse-point, or insecure managed paths; and
- I/O or permission failure.

Public errors and their `Display`/`Debug` output contain only allowlisted stage
and reason codes, plus a revision where relevant. They do not wrap or print a
raw `io::Error` whose message may contain a concrete path. Internal test-only
fault identifiers also contain no user data.

The stable error/status reason set for this slice includes:

- `invalid-config-home`;
- `home-unavailable`;
- `state-missing`;
- `invalid-document-kind`;
- `unsupported-schema`;
- `invalid-document`;
- `document-too-large`;
- `unsafe-managed-path`;
- `insecure-permissions`;
- `lock-busy`;
- `revision-conflict`;
- `revision-exhausted`;
- `persistence-failed`;
- `recovery-required`;
- `unsupported-platform-security`; and
- `secure-store-not-implemented`.

`state-missing` is a status, not a load failure: load returns the virtual
revision-zero default. The implementation may use richer internal enums, but
it must preserve these redacted external meanings.

## Redacted Diagnostics

`RedactedConfigStatus` may report:

- whether root selection came from the default or override policy;
- a symbolic root such as `<user-home>/.config/qiongli/v2` or
  `<configured-root>/v2`, never the resolved path;
- state as missing, ready, invalid, future-schema, busy, insecure,
  recovery-required, or write-unsupported;
- the numeric revision and selected profile when safely readable;
- each provider's enabled flag and readiness category;
- whether a secret reference is present, never its value;
- secret-store availability and a stable remediation code; and
- whether owner action is required to clean a transaction artifact.

It never reports an email address, secret reference, secret value, account
identifier, artifact name, raw OS error, or absolute path. Redaction applies to
normal output, errors, `Debug`, tests, and future support bundles. Status is
read-only and does not create, repair, chmod, delete, or lock state.

## Verification Strategy

### Platform-independent unit tests

- all accepted and rejected root forms, including empty, relative, traversal,
  home-relative, absolute, namespace-appending, and non-UTF-8 absolute cases on
  platforms that support them;
- exact defaults and round-trip serialization for all three profiles and five
  providers;
- strict document kind, schema, revision, provider, field, duplicate-key,
  email, secret-reference, size, and unknown-field rejection;
- virtual revision `0`, first commit `1`, monotonic increment, conflict, and
  exhaustion behavior;
- provider enabled/readiness state combinations;
- unavailable secret-store behavior with no side effects; and
- canary assertions proving private values and concrete paths never enter
  error, `Debug`, status, or serialized diagnostic output.

### macOS and Linux persistence tests

- initial create and replacement produce `0700`/`0600` ownership and modes;
- load and replace use only the selected `v2` root;
- lock contention times out without mutation;
- concurrent writers with the same expected revision produce one success and
  one conflict;
- malformed, future-schema, oversized, symlinked, and insecure state remains
  byte-equivalent;
- every injected single failure before and after activation restores the prior
  bytes and revision;
- a simulated rollback failure returns `recovery-required` without adopting a
  transaction artifact;
- successful commit cleanup failure returns the committed revision plus the
  redacted cleanup flag; and
- directory synchronization and no-follow boundaries are exercised through
  the persistence adapter rather than asserted only through mocks.

### Windows tests

- path, default, schema, reference, and bounded-read fixtures behave the same
  as the other targets; and
- every persistent replacement returns `unsupported-platform-security` with a
  byte-for-byte and entry-for-entry unchanged test root.

### Workspace gate

The implementation must pass native formatting, check, Clippy with warnings as
errors, workspace tests, the native boundary workflow, and Linux/macOS/Windows
GitHub jobs on the same exact implementation head. No Python or Node suite is a
required gate for this Rust-native slice.

## Acceptance Criteria

`CFG-201A` is complete only when:

1. the new crate is a member of the canonical native workspace;
2. absent state resolves to the exact revision-zero typed default without a
   filesystem write;
3. valid macOS/Linux replacements are owner-only, atomic, durable, and
   revision-checked;
4. every tested failed mutation preserves the previous live bytes, except an
   explicitly simulated rollback failure which must enter `recovery-required`;
5. future, malformed, ambiguous, oversized, symlinked, and insecure documents
   fail closed and remain unchanged;
6. no API in this slice accepts or persists a credential value;
7. document bytes contain only a syntactically valid opaque reference for an
   API-key slot;
8. status and all error surfaces pass private-value and path canary tests;
9. Windows writes fail before any filesystem mutation and are not advertised as
   supported; and
10. the exact implementation head passes the Rust-native cross-platform gate.

## Follow-On Sequence

After this design is implemented and accepted, the dependency-contiguous work
is:

1. `CFG-201B`: Windows owner-only DACL, no-follow/reparse, lock, replace, and
   durability adapter;
2. the first real OS credential backend and write/read-back `SecretRef`
   creation flow;
3. native `config show`, `config set`, `status`, and `doctor` service/command
   wiring; and
4. R2 provider resolution through the shared config and secret facades.

Project state, 1.x import, vault fallback, and migration recovery remain later
roadmap work and do not expand `CFG-201A`.

## Approval Record

The user approved the single-document design, public-settings-plus-opaque-ref
scope, and temporary Windows fail-closed write boundary on July 14, 2026. This
document freezes that decision for implementation planning; material changes
to the schema, secret boundary, persistence commit semantics, or Windows write
claim require a design amendment before code changes.
