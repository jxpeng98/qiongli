# Qiongli R3J Signed Native Release Envelope Design

Status: frozen for implementation

Date: July 14, 2026

Scope: `PKG-202A` current-target release identity and trust boundary

Branch: `feat/2x-native-alpha1`

Rolling PR: `#63`

## Goal

Authorize one exact R3H current-target portable archive for the Lite Alpha.1
installation path through canonical, detached Ed25519-signed release metadata.
R3J must bind the complete artifact identity, archive, R3G payload, embedded
resource pack, and R3A launch grant before R3I can preview or execute a native
payload installation.

The product consumes only a verified release token. It never receives a
release private key, never manufactures a production grant, and never treats a
filename, checksum string, or caller-selected public key as authority.

## Authority And Scope

- ADR 0207 remains authoritative for channel and artifact identity.
- R3A remains authoritative for signed launch grants and integration scope.
- R3G remains authoritative for unpacked payload identity.
- R3H remains authoritative for strict portable archives.
- R3I remains authoritative for managed installation transactions.

R3J adds the release-authorization layer between R3H and R3I. It does not add
automatic managed-root discovery, CLI or desktop mutation, updater metadata,
OS code signing/notarization, SBOM, provenance, public publication, or
clean-machine release acceptance.

## Separate Signing Roles

Release-envelope keys and launch-grant keys are distinct trust roles:

- the release key authorizes an exact archive, release channel, generation,
  validity interval, and attached launch grant;
- the launch-grant key authorizes executable capabilities and supported local
  integration scopes; and
- verification requires both roles independently.

A key accepted for one role is not automatically accepted for the other. The
repository stores no private key. Release automation may request signing of the
domain-separated canonical preimage from an external signer or CI secret, then
attach only the key ID and signature bytes.

## Canonical Envelope

`NativeReleaseEnvelopeV1` contains exactly:

```text
schema version
release generation
complete ArtifactIdentityV1
canonical archive filename
archive size and SHA-256
R3G artifact-manifest SHA-256
resource-pack SHA-256
artifact content-root SHA-256
binary SHA-256
complete SignedLaunchGrantV1
not-before and expiry timestamps
```

`SignedNativeReleaseEnvelopeV1` adds one detached Ed25519 signature with a
bounded key ID and lowercase hexadecimal signature. Its signing preimage is:

```text
"QIONGLI-NATIVE-RELEASE-ENVELOPE-V1\0" || RFC8785(envelope)
```

Input is bounded to 256 KiB, uses `deny_unknown_fields`, and must be the exact
canonical JSON representation. Whitespace variants, duplicate or unknown
fields, unsupported schemas, invalid identifiers, unsafe numeric ranges,
noncanonical digests, or mismatched validity intervals fail closed.

The canonical archive filename is derived from the artifact identity. The
envelope validity interval must be contained by the attached launch grant.
The attached grant must bind the same artifact, resource pack, and executable.

## Trusted Release Keys And Rotation

`TrustedReleasePublicKey` contains:

- a bounded stable key ID;
- one Ed25519 public key;
- an inclusive minimum release generation; and
- an optional exclusive maximum release generation.

At most sixteen unique release keys may be supplied. Verification accepts only
the signature key whose configured generation window contains the envelope
generation. Rotation is performed by overlapping or adjacent generation
windows. Removing a key from the trusted set revokes it; there is no unsigned
fallback, caller-provided ad-hoc trust, or fallback to a launch-grant key.

The verification context additionally fixes the expected artifact, expected
channel, minimum accepted release generation, minimum accepted launch-grant
generation, requested mode, requested integration scope, and current time.

## Verification Pipeline

Verification proceeds in this order:

1. Parse and validate the strict canonical signed envelope.
2. Validate the trusted release-key set and select the exact key ID.
3. Enforce the key generation window and verify the detached signature.
4. Enforce time, minimum generation, expected channel, and expected artifact.
5. Re-verify the caller-approved R3H archive against the supplied verified
   resource pack.
6. Compare filename, size, archive digest, manifest digest, pack digest,
   content root, binary digest, and complete artifact identity.
7. Verify the attached launch grant through the independent launch-grant trust
   set and requested Lite integration scope.
8. Return `VerifiedNativeReleaseEnvelope`, which privately retains the approved
   archive target plus verified archive and launch-grant tokens.

Errors expose fixed reason codes only. Archive paths, managed roots, home
directories, environment values, secrets, signature material, and peer input
are not rendered.

## R3I Binding

The public native-payload preview and apply/repair path must consume
`VerifiedNativeReleaseEnvelope` rather than accepting a launch grant and
archive independently.

`InstallNativePayload` records the release-envelope signed-payload SHA-256 in
addition to the R3I archive and payload digests. The plan semantic digest and
install receipt therefore bind the release authorization. Before extraction,
the executor:

- requires the plan's signed launch grant to equal the release token's grant;
- requires the planned release-envelope digest to equal the verified token;
- re-verifies the retained approved archive target; and
- compares the new archive token with the release token before mutation.

Remove and rollback remain possible from the canonical managed receipt without
the release source. Repair requires the same verified release token because it
reintroduces executable bytes.

## Release-CI Boundary

R3J exposes deterministic envelope construction and signing-preimage APIs. A
future release job may:

1. build and verify the target-native R3H archive;
2. build the canonical envelope;
3. send only the domain-separated preimage to an external signing boundary;
4. attach the returned key ID and signature; and
5. verify the complete signed envelope before publishing or installing it.

No API accepts a private key, seed, mnemonic, credential, or arbitrary signing
command. Concrete production public keys and CI secret provisioning require a
separate release-authority decision before Alpha.1 publication.

## Acceptance

- canonical construction and byte-identical serialization pass;
- signature tampering, unknown keys, wrong signing role, invalid key windows,
  stale generations, invalid time, wrong channel, and wrong artifact fail;
- archive, manifest, pack, content-root, binary, and launch-grant mismatches
  fail before an install plan or managed write exists;
- R3I preview, apply, replay, repair, verify, remove, and rollback remain green
  only through a verified release token;
- the current-target installed binary still runs CLI and exact Lite MCP with an
  empty runtime `PATH`; and
- Linux, macOS, and Windows target-native CI pass on the exact implementation
  head.

## Explicit Non-Claims

R3J does not claim a production key has been generated or provisioned. It does
not provide threshold signatures, signed channel/update metadata, emergency
rollback records, automatic key download, OS signing, notarization, checksum
sidecars, SBOM, provenance, automatic root discovery, user-facing install
commands, desktop mutation, updater behavior, public Marketplace publication,
or clean-machine Alpha.1 acceptance.
