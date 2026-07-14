# Qiongli R3K Native CLI And Release Authority Design

Status: frozen for implementation

Date: July 14, 2026

Scope: production public-key injection and explicit current-target native
payload CLI lifecycle

Branch: `feat/2x-native-alpha1`

Rolling PR: `#63`

## Goal

Turn the accepted R3J release token and R3I transaction service into one
explicit local CLI installation boundary for the current Lite target. A
release-built binary may preview, apply, verify, and remove a signed native
payload without Python, Node, Rust, a source checkout, or a caller-selected
trust root.

R3K embeds public trust policy only. Private signing material, client
activation, desktop mutation, public artifacts, and Alpha.1 publication remain
outside this batch.

## Embedded Release Authority

`qiongli-platform` owns one strict canonical `NativeReleaseAuthorityV1`
document containing:

- schema version and expected release channel;
- minimum accepted release and launch-grant generations;
- at most sixteen release public keys, each with its R3J generation window;
- at most sixteen independent launch-grant public keys; and
- lowercase hexadecimal Ed25519 public-key bytes.

Key IDs in each role are sorted and unique. Both role lists are non-empty.
Generation floors are positive bounded integers. Unknown fields, duplicate or
unsorted IDs, invalid Ed25519 keys, invalid windows, oversized input, and
noncanonical JSON fail closed with fixed reason codes.

Release and launch-grant keys remain different Rust types and lists. A key in
one list never authorizes the other role. The authority document contains no
signature, secret, seed, command, URL, path, or runtime override; its authority
comes from being embedded into the reviewed release binary.

## Build Injection

The canonical app build script accepts one optional build-time input:

```text
QIONGLI_NATIVE_RELEASE_AUTHORITY_FILE=<canonical-public-policy.json>
```

When present, the build script performs bounded parsing and full policy
validation, requires byte-exact canonical JSON, and embeds the validated bytes
in the executable. Invalid input fails the build. Only the file contents are
embedded; the source path is not a runtime input.

When absent, the generated embedded-authority resource is empty. This keeps
ordinary source and test builds truthful and non-installable. The executable
does not read a runtime environment variable, command-line public key, home
file, network key, or fallback development key.

## CLI Contract

R3K adds the closed command family:

```text
qiongli install native preview \
  --release <canonical-release.json> \
  --archive <current-target-archive> \
  --managed-root <existing-private-absolute-directory> \
  --target <codex|claude>

qiongli install native apply <same source options> \
  --expected-plan-digest <preview digest> \
  --approve-filesystem-write

qiongli install native verify \
  --managed-root <existing-private-absolute-directory> \
  --install-id <native-payload-id>

qiongli install native remove <same receipt options> \
  --approve-filesystem-write
```

Options are order-independent, unique, and complete. Control values are strict
UTF-8 closed vocabulary or lowercase identifiers/digests. Paths remain native
`OsString` values and are never rendered in success, error, debug, plan, or
receipt output.

`preview` and `apply` require an embedded authority. They derive the expected
artifact from the compiled product version, compiled OS/architecture, Lite
profile, portable-archive installer kind, and authority channel. The caller
cannot select a version, channel, profile, architecture, key, generation floor,
mode, or integration scope outside the target choice.

The release JSON is read with a fixed byte limit, the archive path is approved
through R3H, and the complete R3J pipeline runs before a plan exists. The
managed root must already exist, be absolute, owner-private, unlinked, and
approved as `QiongliManagedData`; R3K does not create or discover it.

## Preview And Approval

Preview builds one short-lived R3I plan using the selected local family,
`cli-local`, user scope, Lite profile, the current target, and exactly
`filesystem-write` approval. Output is a redacted JSON summary containing the
artifact, symbolic target, install ID, release/archive digests, plan semantic
digest, approval requirement, and `mutation: none`.

Apply repeats the complete verification and preview pipeline. It executes only
when `--expected-plan-digest` exactly matches the newly derived semantic digest
and `--approve-filesystem-write` is present. This binds the trusted local
confirmation to the previewed semantics while allowing display timestamps and
plan IDs to vary. There is no generic `--force`, implicit yes, stdin prompt,
MCP/model mutation route, or unsigned fallback.

## Verify And Remove

Verify and remove use the R3I canonical receipt and embedded resource pack, so
they remain available after the release/archive source expires or disappears.
Both require an explicit validated install ID and approved managed root.
Verify is read-only. Remove additionally requires the explicit filesystem-write
confirmation flag and preserves R3I ownership, quarantine, rollback, journal,
and drift guarantees.

R3K intentionally does not expose repair. Reintroducing executable bytes
requires the signed release source and will be composed with recovery and UI
intent in a later bounded batch.

## Output And Failure Contract

Success output is versioned JSON and contains no absolute path, home directory,
environment value, public-key bytes, signature bytes, or release JSON.
Failures use static reason codes only. Usage failures never echo rejected
arguments. The ordinary source build reports release authority and native
preview/apply as unavailable while receipt-backed verify/remove remain
supported when supplied an existing managed root and install ID.

## Acceptance

- strict authority parsing, canonicalization, role separation, key bounds,
  generation floors, and redaction tests pass;
- the source build contains no trust roots and remains unable to preview/apply;
- a test-injected public authority accepts a matching separately signed release
  and rejects wrong roles, stale generations, tampering, and mismatched targets;
- preview performs no mutation, apply requires the exact preview digest and
  explicit approval, and replay/verify/remove preserve R3I semantics;
- CLI output never exposes release, archive, managed-root, or test canary paths;
- installed payload execution remains runtime-independent through the accepted
  R3I/R3J current-target vertical; and
- local Native, focused Lite, Windows MSVC, frozen-boundary, and exact-head CI
  gates pass.

## Explicit Non-Claims

R3K does not generate or store a production private key, select a production
key value, sign a release, create/discover the managed root, download an
archive, activate Codex or Claude, mutate client configuration, expose desktop
apply, start a packaged window, publish a Marketplace entry, provide an
updater, sign/notarize an OS package, produce checksums/SBOM/provenance, build
cross-target artifacts, pass a clean-machine release gate, or publish
`v2.0.0-alpha.1`.
