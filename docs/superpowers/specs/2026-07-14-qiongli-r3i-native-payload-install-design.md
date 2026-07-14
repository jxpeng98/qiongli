# Qiongli R3I Verified Native Payload Installation Design

Status: frozen for implementation

Date: July 14, 2026

Scope: remaining `PLT-202` current-target native-payload vertical

Branch: `feat/2x-native-alpha1`

Rolling PR: `#63`

## Goal

Install one verified R3H current-target portable archive into a caller-approved,
Qiongli-owned local data root through shared Rust platform services. The service
must preview, approve, apply, verify, repair, remove, and roll back the payload
without depending on Python, Node, Rust, the source checkout, or a client-owned
plugin directory at runtime.

R3I closes the gap between a verified transport archive and a receipt-owned
installed executable. It does not make an unsigned archive publicly installable:
apply and repair still require the R3A verified signed launch grant and install
plan boundary. Tests use an explicit test-only signing key. The source-built
product has no production grant and cannot manufacture one.

## Authority And Compatibility

- ADR 0207 remains the artifact-identity authority.
- R3A remains the signed launch-grant, `InstallPlanV1`, and explicit approval
  authority.
- R3B remains the private managed-root and fail-closed transaction precedent.
- R3G remains the canonical unpacked native artifact and payload verifier.
- R3H remains the strict archive parser and fixed-path extraction authority.

R3I adds one `InstallNativePayload` action to the existing plan vocabulary.
Existing plan JSON and semantic digests are unchanged. The additive action is
accepted only by the new native-payload executor; the R3B resource executor and
client adapters continue to reject action shapes they do not execute.

## Trusted Inputs

An apply or repair accepts all of the following:

1. a `VerifiedInstallPlan` whose signed launch grant authorizes Lite MCP for
   the selected local integration scope;
2. an `ApprovedInstallPlan` created only at a trusted CLI or UI confirmation
   boundary with exactly `filesystem-write` approval;
3. an `ApprovedManagedRoot` for the symbolic `QiongliManagedData` root;
4. an explicitly approved canonical R3H archive target;
5. the verified embedded resource pack referenced by the archive and grant;
6. the current time used to recheck plan and approval expiry.

The service never discovers a home directory, client cache, marketplace,
plugin directory, config file, or arbitrary destination. MCP and model-generated
requests must not invoke the path or approval constructors directly.

## Plan Contract

One R3I plan contains exactly one `InstallNativePayload` operation:

```text
root                = QiongliManagedData
entry key           = native-payload
relative path       = <canonical artifact id>
archive sha256      = verified R3H archive digest
manifest sha256     = canonical R3G manifest digest
pack sha256         = signed and embedded resource-pack digest
content sha256      = R3G artifact content root
binary sha256       = signed executable digest
precondition        = missing
postcondition       = managed(content sha256)
inverse             = RemoveManagedEntry(content sha256)
approval            = filesystem-write only
outstanding action  = none
```

The ownership marker uses `native-payload-<archive-sha256>` as its portable
install ID and binds the signed launch-grant payload digest. This prevents two
versions with identical executable bytes from sharing state. The installed
directory leaf remains the canonical artifact ID. Preview rejects any mismatch
among the archive, R3G manifest, signed grant, target descriptor, pack, or
canonical artifact ID before producing a plan.

The executor reparses the action instead of trusting a planner convention. It
requires the exact single-operation shape, rechecks the approval, verifies the
archive again, and compares every digest before persistent mutation.

## Managed Layout

The approved private root contains only caller-independent portable leaves:

```text
<managed-root>/
  <artifact-id>/
    .qiongli-native-artifact.json
    bin/qiongli[.exe]
  .qiongli-native-payload-<archive-sha256>.json
```

During a transaction it may additionally contain:

```text
.qiongli-native-payload-transaction.json
.qiongli-native-payload-quarantine-<transaction-id>/<artifact-id>/
```

No persisted record contains the absolute managed root, archive source path,
home directory, environment value, secret, credential, or client-owned path.
The payload leaf is always the canonical artifact ID and cannot be selected by
untrusted archive metadata.

## Receipt And State Contract

R3I uses a dedicated canonical JSON schema rather than reinterpreting the R3B
resource materialization receipt:

- `NativePayloadOperationReceiptV1` records the root ID, fixed entry key,
  canonical leaf, ownership, archive, manifest, pack, content-root, and binary
  digests;
- `NativePayloadInstallReceiptV1` records the verified plan, artifact, target,
  operation, transaction time, and optional repaired transaction link;
- `NativePayloadLifecycleReceiptV1` records a removed or rolled-back terminal
  event; and
- `NativePayloadInstallStateV1` records a monotonic generation, optional active
  receipt, and optional latest lifecycle receipt.

State is bounded, strict `deny_unknown_fields` canonical JSON in an owner-only
file. Non-canonical, linked, insecure, oversized, future-schema, or identity-
drifted state fails closed and is never repaired automatically.

## Transaction State Machine

### Apply

1. Revalidate root, plan, approval, archive, pack, missing observed state, and
   absent destination.
2. Persist and sync an owner-only journal before payload mutation.
3. Recheck root identity, archive identity, state digest, and destination.
4. Extract through the accepted R3H parser and R3G no-replace commit path.
5. Verify the committed artifact against all planned and signed digests.
6. Atomically persist and verify the active state receipt.
7. Remove the journal and report `applied`.

An identical replay verifies the active receipt and returns `already-applied`.
A foreign destination, a different active plan, or drift is never adopted or
overwritten.

### Verify

Load canonical state, require one active receipt, approve the fixed artifact
target, run the R3G verifier with the supplied pack, and compare every receipt
field. Verification is read-only.

### Repair

Repair requires an existing matching active receipt and an absent payload leaf.
It repeats apply from the verified archive, writes a new active receipt linked
to the prior transaction, and returns `repaired`. A present healthy payload
returns `already-healthy`; a present drifted or foreign payload fails closed.

### Remove And Rollback

Removal and rollback share safe mechanics but emit distinct lifecycle kinds.
The service verifies the active tree, moves it without replacement into a new
owner-private quarantine container, verifies it again, commits inactive state,
and only then deletes the verified quarantine. If state commit fails before the
durable point, the service restores the verified payload to its original leaf.
An unprovable restore retains the journal/quarantine and returns
`recovery-required`.

R3I has no upgrade or prior-version replacement. Rollback therefore means
reversing the accepted fresh/repair installation, not selecting an older
version. Signed updater and retained-version rollback remain `UPD-201`.

## Public Service Surface

`qiongli-platform` exposes:

- `preview_native_payload_install`;
- `ManagedNativePayloadExecutor::apply`;
- `ManagedNativePayloadExecutor::verify`;
- `ManagedNativePayloadExecutor::repair`;
- `ManagedNativePayloadExecutor::remove`; and
- `ManagedNativePayloadExecutor::rollback`.

These are shared Rust services for future CLI and desktop intents. R3I does not
add direct filesystem work to UI callbacks and does not expose mutation through
Lite MCP.

## Acceptance

- plan and receipt JSON are canonical, bounded, path-redacted, and tamper-
  rejecting;
- apply/replay/verify/repair/remove/rollback and idempotent terminal replays pass;
- destination conflict, archive/grant/pack mismatch, expired approval, linked
  state, payload drift, journal conflict, and commit rollback fail closed;
- existing R3B resource plans and executors remain green;
- the installed binary, not a source or extraction-tree binary, runs
  `--version`, content inspection, Lite MCP initialize, exact `tools/list`, and
  one bounded read-only call with an empty runtime `PATH`; and
- Linux, macOS, and Windows target-native CI pass on the exact implementation
  head.

## Explicit Non-Claims

R3I does not claim production signing keys, archive signing, notarization,
checksums, SBOM, provenance, update-channel metadata, upgrade/downgrade,
automatic managed-root discovery/creation, client config mutation, public
Marketplace installation, packaged-window startup, cross-target packaging, or
clean-machine release acceptance.
