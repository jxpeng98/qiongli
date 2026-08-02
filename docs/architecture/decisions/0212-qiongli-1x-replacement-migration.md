# ADR 0212: Qiongli 1.x Replacement Migration And 2.x Cutover

- Status: Accepted
- Date: 2026-07-24
- Task ID: `ARC-212`
- Owners: Qiongli maintainers
- Decision scope: Qiongli 1.x detection, user-data migration, integration
  replacement, cleanup, rollback, and Qiongli 2 release identity
- Supersedes in part: ADR 0209's refusal to discover, read, migrate, or remove
  Qiongli 1.x; the R3Q and R4-0 steady-state `qiongli`/`qiongli-next`
  coexistence policy
- Retains: ADR 0209's complete-application update, major-version boundary, and
  receipt-owned Qiongli 2 reconciliation

## Context

The first Qiongli 2 alpha deliberately treated every Qiongli 1.x installation
as unmanaged, read-only legacy content. A new `qiongli-next` plugin could be
installed beside the old `qiongli` plugin, and acceptance preserved legacy
canaries to prove that Qiongli 2 did not take ownership of them.

This policy is safe for an early engineering preview but produces a confusing
product:

- a current Qiongli 2 installation is reported as mixed ownership whenever an
  old Qiongli path remains;
- removing Qiongli 2 can leave a visible `qiongli` source behind;
- the UI cannot tell a transitional upgrade from an intentionally supported
  dual installation;
- users must understand internal plugin names, standalone Skills, MCP
  registrations, and old package managers to finish an upgrade; and
- an installed application built from the latest source can still look old
  when its release version was not advanced.

The product decision is now that Qiongli 2 replaces Qiongli 1.x. Coexistence is
allowed only inside a bounded migration transaction. It is not a supported
steady state.

## Decision

### One-way replacement

Qiongli 2 provides one local, one-way 1.x-to-2.x migration workflow. The
workflow may inspect only documented Qiongli 1.x locations and recognized
Qiongli-owned records. It does not execute a 1.x binary, import a Python or
Node runtime, load 1.x plugin code, or use 1.x as an operational fallback.

The migration sequence is:

1. discover and classify supported 1.x surfaces without mutation;
2. produce a redacted preview and content-addressed migration plan;
3. snapshot only the exact owned records needed for transaction compensation;
4. convert supported global settings into native 2.x schemas and direct users
   to the separate, explicit source/destination project migration for each
   research project they choose;
5. materialize fresh 2.x plugins, Skills, MCP declarations, and receipts from
   the verified embedded resource pack;
6. require supported-client activation evidence or an explicit client action;
7. verify the Qiongli 2 application, converted provider state, integrations,
   and restart;
8. remove recognized 1.x registrations and generated installation surfaces;
9. commit a migration receipt proving that no active recognized 1.x surface
   remains; and
10. discard transaction-only legacy snapshots after the health window commits.

No normal Qiongli 2 read path consults 1.x after the migration receipt commits.
There is no supported downgrade to 1.x.

The installation migration never searches the home directory for projects.
Project migration has its own copy-based preview, digest, and receipt because a
project can live anywhere and path discovery alone cannot prove ownership. An
installation migration can therefore complete independently; each selected
project becomes 2.x-only when its separate project migration commits.

### Migration classes

Every discovered item belongs to exactly one class:

| Class | Examples | Treatment |
|---|---|---|
| User academic data | research projects, portable artifacts, notes, sources, manuscripts | Convert or register through the native project migration service, verify counts and digests, then keep the 2.x result |
| Supported settings | non-secret provider settings, language and workflow preferences with an explicit 2.x mapping | Normalize into the current schema; unknown fields are reported, never guessed |
| Secrets | recognized literature-provider credentials | Import into the OS credential service only with explicit approval and verify by redacted lookup before removing plaintext legacy storage |
| Obsolete credentials | direct model-provider credentials no longer used by the host-driven product | Do not import; offer explicit redacted removal |
| Generated installation bytes | plugins, Skills, MCP runtimes, wrappers, manifests, generated workflow files | Never copy into 2.x; regenerate from the verified Qiongli 2 application, then remove the recognized 1.x source |
| Host registrations | Codex and Claude Code plugin, marketplace, Skills, and standalone MCP entries | Register and activate Qiongli 2 first, then remove only the exact recognized 1.x entry |
| Ephemeral state | caches, downloads, logs, transient sessions, client caches | Do not migrate; remove only when ownership is proven |
| Unrecognized or modified content | custom files inside a legacy-named path, ambiguous marketplace entries, unsupported schemas | Stop cleanup for that item and require review; never silently delete or reinterpret it |

"Migrate all 1.x content" therefore means preserving all supported user-owned
meaning while replacing all Qiongli-owned executable and generated content
with verified 2.x content. It does not mean copying old runtime bytes into the
new product.

### Transitional coexistence

Temporary coexistence is permitted only while the migration receipt is in one
of these non-terminal states:

- `detected`;
- `preview-ready`;
- `staged`;
- `awaiting-client-activation`;
- `verification-required`;
- `cleanup-ready`; or
- `recovery-required`.

The only successful terminal state is `complete`. A completed migration has a
current Qiongli 2 installation and no active recognized Qiongli 1.x plugin,
standalone Skill, MCP registration, package registration, or generated source.

If the host cannot expose activation state programmatically, the transaction
stops at `awaiting-client-activation`. The UI gives the exact host-owned action
and does not remove the working 1.x surface until the user completes and
confirms the Qiongli 2 activation.

### Ownership and cleanup

Automatic cleanup requires proof from at least one of:

- a valid Qiongli 1.x managed marker or receipt;
- an exact recognized marketplace or MCP entry generated by the accepted 1.x
  installer contract;
- a deterministic content identity from the frozen
  `v1.19.0-beta.1` migration baseline; or
- an exact legacy path whose bounded manifest validates as a Qiongli package
  and contains no unknown files.

Path name alone never grants deletion authority. A symlink, malformed record,
unknown child, ownership mismatch, or content drift produces
`review-required`. Cleanup edits shared client configuration structurally and
removes only the exact Qiongli entry; it never replaces the entire file.

### Product and plugin identity

The application displays both:

- the released product version, such as `2.0.0-alpha.2`; and
- an immutable build identity derived from the packaged source commit.

Development builds do not pretend to be a newer release merely because the
source commit changed. A release claiming Alpha.2 must advance every
application, Cargo, embedded control, update, package, receipt, fixture, and
acceptance version binding together.

The internal Alpha plugin slug may remain `qiongli-next` while the migration
engine is implemented, but the product UI labels it "Qiongli 2 plugin".
Internal slugs appear only in diagnostics and exact client actions. The stable
plugin slug decision is made before Beta.1 and must not create another
coexistence mode.

## Alternatives considered

### Continue preserving 1.x indefinitely

Rejected because it makes mixed ownership a normal state, keeps duplicate
Skills and MCP surfaces discoverable, and leaves users to finish the migration
manually.

### Delete every legacy-named path during installation

Rejected because a name does not prove ownership and could destroy user
modifications or unrelated marketplace content.

### Copy the complete 1.x tree into the 2.x namespace

Rejected because it would import old executable content, stale generated
assets, credentials, caches, and unsupported schemas into the native product.

### Remove 1.x before Qiongli 2 activation

Rejected because a client-owned activation or restart can fail after files are
installed. Cleanup occurs only after Qiongli 2 verification or remains in an
explicit waiting state.

## Consequences

- Qiongli 2 gains a bounded migration subsystem instead of a permanent legacy
  compatibility subsystem.
- The current inventory model changes from `LegacyOnly` observation to
  actionable migration evidence.
- `Mixed ownership` is no longer a successful or current state.
- Existing project migration primitives remain the explicit data-migration
  boundary; generated 1.x integration bytes are replaced rather than imported.
- The packaged-product acceptance fixture no longer leaves legacy canaries in
  the manual test home.
- Migration adds a temporary disk and transaction cost, but no 1.x runtime
  dependency is added to the shipped application.

## Security and privacy

- Discovery is local, bounded to documented paths, path-redacted by default,
  and never launches a 1.x executable.
- Shared configuration files are parsed and edited structurally with
  compare-and-swap protection.
- Secrets are never copied into receipts, logs, previews, plugin manifests, or
  project artifacts.
- Every destructive step follows a fresh verification of the preview digest,
  filesystem identity, ownership evidence, and current 2.x health.
- Unknown or drifted content fails closed at the item boundary without
  blocking migration of unrelated proven items.
- This decision adds focused migration and transaction tests; it does not add
  a separate formal cybersecurity scan.

## Rollback

Before cleanup commits, the transaction retains exact private compensation
records for files and shared-config entries it will change. A failure restores
the pre-migration registrations and records and removes newly materialized
2.x bytes only when their new receipt proves ownership.

After `complete`, Qiongli does not roll back to or operate Qiongli 1.x. User
academic data remains in its verified 2.x form. Transaction-only copies of
generated 1.x installation bytes are deleted after the bounded health window.
An unrecoverable compensation failure enters `recovery-required` and performs
no further cleanup.

## Acceptance tests

1. Fresh 2.x installation with no 1.x content remains a normal install and
   creates no migration receipt.
2. Supported 1.x Skills-only, plugin-only, standalone-MCP, full-plugin, and
   mixed installation fixtures each produce deterministic previews.
3. Qiongli 2 is installed and verified before any active 1.x integration is
   removed.
4. Project files, supported preferences, and approved provider credentials
   retain their accepted meaning after migration and restart.
5. Generated plugins, Skills, MCP runtime bytes, wrappers, and caches are
   regenerated or retired, never copied into the 2.x product.
6. Unknown children, symlinks, malformed config, unmanaged entries, and
   concurrent edits stop only the affected cleanup and preserve the bytes.
7. Fault injection before and after every write restores one working
   pre-migration state or one complete 2.x state, never a partial silent
   cleanup.
8. A completed installation receipt proves current Qiongli 2 source,
   registration, Skills, MCP declaration, restart health, converted provider
   state, and absence of every recognized active 1.x integration surface.
   Separately selected projects carry their own migration receipts.
9. The manual acceptance home is clean; legacy migration fixtures use separate
   isolated homes and cannot affect the visible integration status.
10. Alpha.2 UI and receipts display the coordinated product version and exact
    packaged source commit.
