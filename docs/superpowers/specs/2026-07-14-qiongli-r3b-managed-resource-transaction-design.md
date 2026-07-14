# Qiongli R3B Managed Resource Transaction Design

Status: approved for execution

Date: July 14, 2026

Scope: first `PLT-202` vertical behind the R3A verified-plan boundary

## Goal

Implement one safe current-user transaction lifecycle for a signed,
approved Marketplace Lite resource materialization without adding a second
planner or prematurely writing client configuration.

R3B consumes an already constructed `VerifiedInstallPlan`, an explicit local
approval bound to that plan digest and expiry, an explicitly approved
`QiongliManagedData` root, and a verified embedded resource pack. It never
discovers a home directory, host client, plugin cache, or marketplace path.

## Scope Decision

The first executable transaction vertical accepts exactly one
`MaterializeResources` operation:

- target profile is Lite;
- symbolic root is `QiongliManagedData`;
- destination is one portable directory leaf below the approved root;
- precondition and observed state are `Missing`;
- postcondition and inverse match the selected resource-pack content root and
  ownership marker;
- no host action remains outstanding; and
- only `FilesystemWrite` approval is present.

This covers a fresh materialization and receipt-backed restoration of a
missing previously managed target. R3B rejects, before persistent mutation:

- multiple operations;
- `RegisterPluginSource`, `RegisterLiteMcp`, and action-driven
  `RemoveManagedEntry` apply;
- managed replacement, upgrade, or overwrite;
- nested or reserved transaction paths;
- client-config, host-trust, or extra approval scope; and
- drifted, unmanaged, linked, hard-linked, or foreign-owned destinations.

The restriction is intentional. Host adapters remain `INT-201`/`INT-202`, and
managed upgrade with retained prior-version backups remains updater work. R3B
does not weaken R3A's broader plan schema; it defines the first executor
capability subset.

## Trusted Inputs

### Verified plan

Only `VerifiedInstallPlan` is accepted. The executor rechecks plan expiry,
semantic digest binding, action shape, exact ownership, pack root, observed
missing-state digest, and postcondition immediately before mutation.

### Explicit approval

`approve_install_plan` is a trusted CLI/UI composition boundary. It creates a
private `ApprovedInstallPlan` token only when:

- the plan is currently valid;
- supplied approvals are sorted, unique, and exactly equal the plan's
  requirements; and
- the token binds the semantic digest and plan expiry.

The source-built canonical binary has no signed grant or verified plan, so it
cannot reach this boundary in R3B.

### Approved managed root

`approve_managed_root` accepts one caller-selected absolute root already owned
by the current user. It rejects traversal, symlinks/reparse points, non-directory
objects, insecure Unix modes, non-owner-only Windows DACLs, and roots other
than `QiongliManagedData`. Debug and errors never print the absolute root.

The executor does not create or infer this root. A later installer composition
must resolve and create it through an approved product boundary.

## Persistent Contract

R3B adds bounded, strict, canonical JSON:

- `InstallReceiptV1`: active plan, artifact, target, ownership, operation,
  content-root, materialization-receipt digest, transaction time, and any
  prior transaction link;
- `InstallLifecycleReceiptV1`: receipt-backed `removed` or `rolled-back`
  terminal event; and
- `ManagedInstallStateV1`: monotonic generation, optional active receipt, and
  optional latest lifecycle receipt.

Receipts contain symbolic root IDs and portable relative leaves, never an
absolute path, secret, environment value, API key, or copied credential.

For install ID `<id>`, the approved root contains only these platform metadata
siblings:

```text
.qiongli-install-<id>.json
.<id>.qiongli-transaction.json
.<id>.qiongli-quarantine-<transaction-id>
```

The state document is the durable source of truth. The journal is created and
synced before the first managed-target mutation and doubles as the exclusive
per-install transaction claim. A surviving journal means `recovery-required`;
normal apply never silently deletes an unknown or stale journal.

State files and journals are owner-only, bounded, unknown-field-denying,
canonical JSON. Unix uses `0600`; Windows uses the isolated owner-only security
adapter. State replacement is atomic and directory-synced where supported.

## Lifecycle

### Apply

1. Validate the verified plan, approval token, approved root, pack, action,
   missing-state digest, and absent destination without writing.
2. Write and sync the immutable transaction journal.
3. Revalidate the destination and state after journal acquisition.
4. Delegate the resource tree write to the existing atomic
   `qiongli-content` materializer using the `lite` projection.
5. Read-only verify the materialized tree and hash its canonical content
   receipt.
6. Atomically commit `ManagedInstallStateV1` with the active platform receipt.
7. Remove the journal and report committed cleanup separately if removal or
   directory sync fails.

Identical active-plan replay verifies and returns `already-applied` without
rewriting the tree.

### Verify

Verification is read-only. It rejects a surviving journal, loads and validates
the canonical state document, re-approves the recorded relative target below
the same root, verifies the complete materialized tree, and compares the
materialization receipt hash, pack root, profile, ownership, and active plan
receipt.

Verification reports only `healthy`; missing, drifted, linked, foreign,
corrupt, or recovery-required state is an error with a static reason code.

### Repair

Repair requires an existing active platform receipt and a new verified plan
with the same install ID and target leaf. If the target is healthy it returns
`already-healthy`. If the target is absent, repair performs the same journal,
materialization, verification, and state-commit transaction as apply and links
the new receipt to the prior transaction.

A present but drifted target is a conflict and is never overwritten. Managed
replacement and upgrade are not R3B repair behavior.

### Remove And Rollback

Both operations load and verify the active receipt, write the journal, pin and
reverify the target identity, and atomically rename it to a private quarantine
sibling. They then atomically commit inactive state with a distinct lifecycle
receipt:

- `remove` records `removed`;
- `rollback` records `rolled-back`.

If state commit fails, the quarantine is renamed back only when identity and
expected content still match. After state commit, quarantine deletion is
cleanup: failure is reported as committed cleanup required rather than a false
rollback. Repeated matching lifecycle calls are idempotent.

Removal and rollback delete only the exact Qiongli materialization whose
active platform receipt and content receipt both match. They never remove an
unmanaged, foreign, or user-modified target.

## Failure And Recovery

Fault tests cover journal persistence, post-materialization failure,
pre-state-commit failure, rollback failure, state-commit failure after
quarantine, and post-commit cleanup failure.

Before state commit, an apply failure removes only the just-created verified
target. A remove/rollback commit failure restores the verified quarantine. If
that safe recovery cannot be proven, the journal and data remain and the
executor returns `recovery-required` instead of attempting another destructive
action.

R3B does not add an automatic crash-recovery command. A surviving journal is
preserved evidence and blocks later mutation until the next recovery slice.

## CLI Status

`qiongli install status` adds the receipt schema and reports the transaction
engine as `grant-and-approval-gated`. Because the ordinary source binary has
no production launch grant, root approval, or install plan, its `launch_grant`,
`preview`, and `apply` fields remain `unavailable`.

## Security Properties

- No arbitrary path or bytes enter the executor from the plan.
- Resource bytes come only from a verified `LoadedResourcePack`.
- All destination, state, journal, and quarantine paths stay below one
  explicitly approved private root.
- Target verification rejects symlink/reparse substitution, hard links,
  unexpected files, digest/mode drift, and materialization-receipt drift.
- Errors and debug output use static reason codes and symbolic identifiers,
  not local absolute paths or supplied secret values.
- Production code contains no signing private key and cannot select its own
  launch-grant trust root.

## Nonclaims

R3B does not implement or claim:

- Codex or Claude discovery, plugin source registration, MCP config writing,
  enablement, trust, or runtime activation;
- Marketplace/Desktop/private-cache/cloud installation;
- multi-operation atomicity, managed in-place upgrade, automatic crash
  recovery, or cross-version backup restoration;
- a native UI or production provider credential store;
- a production signed artifact, installable package, updater, or
  `v2.0.0-alpha.1` readiness.

## Acceptance Criteria

R3B is complete when:

1. only an exact verified plan plus exact local approval and approved private
   root can enter the executor;
2. fresh apply commits an exact Lite resource tree and canonical active state;
3. verify detects every receipt, ownership, path, mode, and content drift;
4. missing-target repair is safe and present drift is never overwritten;
5. remove and rollback affect only the exact active managed target and are
   idempotent;
6. injected pre-commit failures restore absence or the prior active target,
   while ambiguous recovery fails closed with the journal retained;
7. no absolute path or secret appears in receipts, errors, or status; and
8. local and exact-head Tier 1 Rust gates pass with the nonclaims above intact.

## Approval Record

The user instructed continuation into the next roadmap batch on July 14, 2026.
This authorizes R3B on the existing rolling branch and Draft PR #63.
