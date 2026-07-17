# Qiongli 2.0 R3H Deterministic Portable Archive Design

Date: 2026-07-14

Status: accepted for implementation

Roadmap slice: `R3H / PKG-201B`

Scope: one unsigned, unpublished, current-target portable archive built from
the accepted R3G native artifact staging tree

## Decision

R3H turns a verified R3G staging tree into one deterministic store-only ZIP
file and can safely materialize that file back into the same verified artifact
tree. It does not create another artifact identity or weaken the R3G resource
pack anchor. The archive filename is:

```text
<canonical-artifact-id>.zip
```

The archive contains exactly four entries in fixed order:

```text
<canonical-artifact-id>/
<canonical-artifact-id>/.qiongli-native-artifact.json
<canonical-artifact-id>/bin/
<canonical-artifact-id>/bin/qiongli[.exe]
```

The inner manifest remains the identity authority. A verified archive value
adds only observed container size, SHA-256, inner manifest SHA-256, and the
already verified artifact identity. No checksum sidecar or release receipt is
created in R3H.

## Canonical ZIP Profile

R3H writes a deliberately narrow ZIP profile:

- stored entries only; compression method is zero;
- UTF-8 names and no encryption;
- fixed DOS timestamp `1980-01-01 00:00:00`;
- fixed creator, extractor-version, file-type, and logical-mode fields;
- no extra fields, comments, data descriptors, ZIP64, spanning, or multiple
  disks;
- one local header and one central-directory record per fixed entry;
- exact local/central name, CRC-32, size, offset, and attribute agreement;
- one end-of-central-directory record with no trailing bytes; and
- total size bounded to the R3G binary and manifest limits plus fixed envelope
  overhead.

The directory entries use logical mode `0755`; the manifest uses `0644`; the
binary uses `0755`. The Windows archive still records those portable logical
modes even though NTFS execution does not use POSIX execute bits.

This profile remains a conventional ZIP that standard tools can inspect, but
Qiongli accepts only the exact canonical subset. Re-archiving the same files
with a generic tool is not equivalent and must fail canonical verification.

## Composition Boundary

Composition requires:

1. a caller-supplied verified resource pack;
2. an explicitly approved R3G source target;
3. an explicitly approved canonical `.zip` output target; and
4. matching current-target artifact identities.

The source tree is verified before payload bytes are captured. The captured
manifest and binary bytes are verified again as one payload so a path race
cannot change the accepted identity. The archive is written to an owner-private
create-new sibling file under a target lock, synced, assigned its fixed logical
mode, verified byte-for-byte, and promoted without replacement. Existing
caller data is never adopted, replaced, or removed.

## Read-only Verification

Archive verification is read-only. It validates the source file type,
ownership, link count, mode, size bound, complete ZIP structure, canonical
entry sequence and metadata, CRC-32 values, and absence of trailing bytes. It
then validates the inner canonical manifest and binary bytes against:

- the canonical artifact ID and current target;
- the external verified resource-pack identity;
- the R3G binary path, size, SHA-256, and artifact content root; and
- the complete R3G manifest shape and `assembled-unpublished` status.

Errors expose fixed reason codes only. Paths, raw archive names from rejected
input, OS messages, content bytes, and configuration values are not rendered.

## Safe Extraction

Extraction never joins arbitrary archive names. The parser first accepts the
complete archive in memory and yields only the two fixed verified file payloads.
Those payloads are committed through the R3G private staging, fixed-mode,
no-replace, persistence, and committed-verification path into an explicitly
approved canonical artifact target.

The extractor therefore rejects traversal, absolute paths, duplicate or case-
colliding names, symlinks, hard-link encodings, devices, reparse metadata,
unknown entries, oversized entries, decompression bombs, and partial output by
construction. A failed extraction leaves no destination tree and preserves an
existing destination unchanged.

## Acceptance

R3H acceptance proves:

- two archives from equivalent verified trees are byte-identical;
- archive filename, root, inner identity, resource pack, digest, and target
  agree;
- local-header, central-directory, CRC, mode, offset, truncation, trailing-byte,
  duplicate-entry, and source-file tampering fail closed;
- extraction produces a tree accepted by `verify_native_artifact`;
- only the extracted binary, outside the checkout and with an empty runtime
  `PATH`, runs `--version`, content inspection, MCP initialization, the exact
  12-tool Lite surface, and one bounded read-only tool call; and
- target-native Linux, macOS, and Windows CI accept the exact implementation
  head.

## Explicit Non-claims

R3H does not add compression, encryption, signatures, notarization, checksum
sidecars, SBOM, provenance, a launch grant, an installer, host integration
mutation, UI mutation, updater metadata, publication, cross-target assembly,
packaged-window startup, or clean-machine release acceptance. The output stays
unsigned and unpublished even though it is a final deterministic container for
this current-target slice.
