# ADR 0205: Deterministic Embedded Resource Pack

- Status: Accepted
- Date: 2026-07-11
- Task ID: `ARC-201E`
- Owners: Qiongli maintainers
- Decision scope: Canonical content compilation, verification, embedding, and materialization

## Context

Qiongli's workflows, skills, agents, roles, templates, standards, schemas,
subjects, MCP contracts, and distribution metadata remain canonical files under
`content/`. The Rust migration must not translate those files into parallel
hand-maintained constants or require a source checkout on the user's machine.
The installed native binary therefore needs a self-contained resource pack
that can list, load, and materialize the canonical tree without Python,
Node.js, Cargo, or network access.

The same source commit must yield the same content identity on macOS, Windows,
and Linux despite filesystem ordering, timestamps, owners, permissions, line
ending settings, or build paths. Externally supplied packs add a supply-chain
and extraction boundary: a valid archive is not automatically trusted, and a
host application's private cache is not a supported installation API.

## Decision drivers

- retain `content/` as the single academic and product-contract source of
  truth;
- produce byte-stable content identity from the same canonical commit on every
  Tier 1 build host;
- ship resources inside the native artifact so runtime language tools and a
  repository checkout are unnecessary;
- authenticate every production or externally loaded pack before use;
- reject traversal, links, collisions, oversized input, and archive metadata
  ambiguity before materialization;
- materialize host-readable skills, agents, and related resources without
  writing a host-managed cache directly;
- make source drift, generated drift, and rollback observable through hashes
  and managed receipts.

## Decision

### Canonical input

`qiongli-content` will compile an explicit allowlist of canonical roots under
`content/`. Generated plugins, installer payloads, host caches, local config,
build output, VCS metadata, and files outside those roots are never pack input.
Every input must be a regular file. Absolute paths, `..`, empty components,
symlinks, hard-link aliases, reparse points, device files, and normalized path
collisions are rejected.

Pack paths are relative UTF-8 paths normalized to `/` separators and Unicode
NFC. The compiler rejects two paths that collide after Unicode normalization or
portable case folding. Entry order is the ascending byte order of normalized
paths. File content is the canonical repository byte sequence; text contracts
must already satisfy their declared UTF-8 and LF policy, and the compiler fails
instead of silently changing semantic bytes.

Filesystem timestamps, creation times, UID, GID, ACLs, extended attributes,
source absolute paths, directory iteration order, and host tool versions are
not recorded. Logical file mode is normalized to `0644` or `0755` from the
committed executable declaration; no other source permission bits enter the
pack.

### Pack format and identity

Format version 1 is an uncompressed, seekable `.qlpack` core with:

1. fixed magic bytes and a fixed-width little-endian format version;
2. the byte length of a canonical UTF-8 JSON manifest;
3. the manifest encoded with RFC 8785 JSON Canonicalization Scheme (JCS); and
4. raw file payloads concatenated in manifest path order.

The version 1 header is exactly 20 bytes: the eight-byte magic `QLPACK\0\0`
(`QLPACK` followed by two NUL bytes), a four-byte unsigned format version, and
an eight-byte unsigned manifest length. Both integers use little-endian
encoding. Entry
`payload_offset` values are relative to the first byte after the canonical
manifest, not to the start of the core.

The manifest contains at least:

- `format_version`, `pack_id`, `content_version`, and canonical source commit;
- compatible product range and declared `skill-only`, `lite`, or `full`
  profiles;
- normalized compiler-contract version, with no wall-clock build timestamp;
- for every entry, path, logical resource kind, mode, byte length, payload
  offset, and SHA-256 digest; and
- a domain-separated `content_root_sha256` calculated over the ordered entry
  metadata and digests.

For compiler-contract version 1, the content-root preimage is the ASCII domain
`qiongli:resource-pack:content-root:v1` followed by one NUL byte, then the
eight-byte little-endian length of the RFC 8785 canonical entry-array JSON,
then those canonical entry-array bytes. Release metadata such as source commit
and compatibility range is intentionally outside the content root but remains
bound by `pack_sha256`. Manifest integers must remain within the RFC 8785 /
I-JSON safe range (`0..=9007199254740991`).

Offsets must be contiguous, in bounds, non-overlapping, and consistent with
the declared lengths. The SHA-256 digest of the complete unsigned core is its
`pack_sha256`. The core is deliberately uncompressed in format version 1 so
there is no compressor-version nondeterminism or decompression-bomb surface.
A future deterministic compression scheme requires a new format version and a
superseding compatibility decision.

Release signing wraps the core with a canonical signature descriptor containing
the algorithm, trusted key ID, `pack_sha256`, compatibility claims, and an
Ed25519 signature over a domain-separated message. Descriptor compatibility
claims must equal the signed manifest claims. Signature metadata does not
change the `content_root_sha256` or `pack_sha256`. Signing timestamps belong in
release provenance, not in the deterministic pack core. The release-channel
decision owns key custody, rotation, and revocation without changing this
format contract.

### Embedding and verification

Production binaries embed the core and its signature descriptor as immutable
bytes and carry the trusted public-key set, never a signing key. Startup verifies
the descriptor, whole-pack digest, manifest limits, and entry digests before
exposing content. Verification failure disables the affected content surface
and reports a stable diagnostic; it never falls back to loose checkout files.

A development-only build flag may embed an unsigned fixture, but that binary is
marked non-release and cannot import unsigned external packs. Release tooling
must reject an artifact built with that flag.

Every external pack requires a valid signature from an already trusted Qiongli
resource key, an allowed compatibility range, and complete digest validation
before any payload is parsed as a schema or written to disk. There is no
trust-on-first-use prompt, user-added key beside the pack, or “continue anyway”
path. Key rotation and revocation follow signed release metadata defined by the
release-channel decision.

### Materialization boundary

The content service can list or read resources directly from verified embedded
bytes and can materialize a manifest-declared subset such as skills, agents,
workflows, roles, templates, standards, schemas, or target metadata. Profile and
resource-kind selection is explicit; callers cannot request an arbitrary pack
path by string and bypass policy.

Materialization writes regular files only into a Qiongli-owned staging directory
selected by a validated `InstallPlan`. It revalidates every joined path, applies
normalized modes, verifies bytes after writing, and atomically activates a
complete tree with a managed manifest. It refuses to replace an unmanaged file
or tree. No resource is executed during verification or extraction.

`qiongli-content` and `qiongli-installer` never write a Codex, Claude, ChatGPT,
or other host-managed cache directly. They may materialize a documented local
source/plugin/skills directory or a Qiongli-owned payload and then use the
host's supported registration or import boundary. Host activation and config
mutation belong to the `InstallPlan` ADR, not to the resource-pack parser.

## Alternatives considered

### Ship loose `content/` files beside the executable

Loose files are easy to inspect, but installation can omit or partially update
them, path identity varies by package, and users can unknowingly combine a new
binary with old contracts. Rejected as the production source. An explicit
materialization command remains available after verification.

### Translate content into Rust constants

Generated or hand-written constants can embed data, but they create another
representation of academic truth and obscure source/materialization drift.
Rejected; Rust embeds one reproducible pack built from canonical files.

### Use a conventional ZIP or tar archive

Both formats can be normalized, but their optional timestamps, owner fields,
path types, compression settings, and extractor behavior create unnecessary
cross-platform variability and attack surface. Rejected for format version 1
in favor of a small uncompressed, bounded, seekable envelope.

### Accept unsigned local packs after confirmation

A confirmation dialog does not establish publisher identity and is unsuitable
for unattended CLI, MCP, repair, or update flows. Rejected. Development
fixtures require an explicit non-release build mode.

### Install by extracting directly into host caches

Host caches are private implementation details and may be replaced, indexed,
or permissioned by their owner at any time. Direct writes would be brittle and
could bypass host trust prompts. Rejected.

## Consequences

Positive consequences:

- the binary contains one verifiable representation of canonical content and
  works without a source checkout or language runtime;
- two builds can compare one content root and one whole-pack digest instead of
  relying on archive listings;
- uncompressed bounded parsing is straightforward to audit and supports direct
  reads without extraction;
- signed external packs cannot silently replace trusted academic or execution
  contracts;
- materialization is target-aware, reversible, and separated from undocumented
  host storage.

Costs and limitations:

- an uncompressed core increases binary and installer size;
- a custom envelope needs maintained parsers, schema compatibility tests, and
  inspection tooling;
- content changes require rebuilding and signing the product or a compatible
  external pack;
- externally authored community packs are not supported until a separate trust,
  namespace, and governance model is accepted;
- pack signature verification does not by itself make materialized scripts safe
  to execute; execution remains subject to profile and ToolHost policy.

## Security and privacy

- Input uses an allowlist and build-time secret/private-data scan. A pack must
  not contain user config, credentials, telemetry, local project data, machine
  paths, or signing material.
- Parsers enforce maximum pack size, manifest size, entry count, individual
  entry size, path depth, and cumulative materialized bytes before allocation
  or writing.
- Signature and whole-pack verification precede schema parsing and
  materialization for external input. Unknown keys, invalid signatures,
  incompatible profiles, and digest mismatches fail closed.
- Build and runtime validation reject traversal, absolute paths, links, device
  names, Unicode/case collisions, overlapping offsets, duplicate IDs, and
  trailing undeclared payload bytes.
- Materialization uses no-follow operations, owner-appropriate permissions,
  bounded staging, post-write hashes, and an atomic managed-tree activation.
- A valid signature establishes Qiongli publisher integrity, not permission to
  execute. Scripts and agents remain inert resources until an allowed profile,
  installation plan, and execution policy approve their use.

## Rollback

Resource packs are immutable. An update retains the last verified product
artifact and pack identity until the new binary, embedded pack, signature, and
materialization acceptance complete. Failure keeps or restores the prior
artifact; it never edits an old pack in place.

For an external pack, activation is an atomic reference to a verified pack in
Qiongli-owned state. Rollback selects the previous verified identity. If content
was materialized, the installer restores its pre-operation snapshot or prior
managed tree using the recorded manifest. Unmanaged host files are never
deleted during pack rollback.

## Acceptance tests

1. Two clean builds from the same commit on macOS, Windows, and Linux produce
   byte-identical core packs with identical `content_root_sha256` and
   `pack_sha256`.
2. Randomized input enumeration, source mtimes, owners, non-semantic permission
   bits, and build paths do not change the pack; one canonical content-byte or
   declared-mode change does change the relevant entry, root, and pack digest.
3. Manifest tests verify canonical encoding, sorted paths, contiguous offsets,
   resource kinds, profiles, entry hashes, root calculation, and complete
   payload coverage.
4. The release binary starts, lists, reads, and materializes its embedded core
   profile on a clean machine with no repository checkout, Python, Node.js, or
   Rust toolchain process.
5. Tampered core, manifest, payload, signature, compatibility range, unknown
   key, revoked key, and unsigned external-pack fixtures all fail closed before
   materialization.
6. Traversal, absolute path, symlink, hard-link alias, reparse point, device
   name, duplicate ID, Unicode collision, case collision, oversized input,
   offset overlap, and trailing-byte fixtures are rejected on every Tier 1
   target.
7. Materializing the accepted core profile produces the expected normalized
   skills, agents, workflows, roles, templates, standards, and schema tree with
   a matching managed manifest and post-write hashes.
8. Fault injection during materialization leaves either the previous complete
   managed tree or the new complete tree and never overwrites an unmanaged
   destination.
9. A filesystem guard proves pack loading and materialization do not write any
   host-managed plugin cache or undeclared path.
10. Release gates reject development unsigned mode, content drift, embedded
    pack/hash mismatch, secrets, private data, absolute build paths, and missing
    signature evidence.

## Follow-up tasks

- `FND-201`: define the shared Rust workspace and bounded I/O primitives used by
  the compiler and loader.
- `FND-202`: implement the canonical compiler, embedded loader, drift gate,
  inspection command, and cross-platform reproducibility fixtures.
- `CTR-201`: inventory the exact canonical roots, logical resource kinds,
  profiles, and expected materialized tree.
- `PLT-201`: ensure `InstallPlan` owns target selection, documented host
  boundaries, managed markers, preview, activation, and rollback.
- `PKG-202`: sign production packs and record checksums, SBOM, provenance, key
  identity, and verification receipts.
- `QAT-201`: add clean-machine embedded startup, malicious-pack,
  materialization, no-host-cache-write, and zero-runtime audits.
