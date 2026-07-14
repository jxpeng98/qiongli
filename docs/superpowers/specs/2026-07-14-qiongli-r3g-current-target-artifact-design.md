# Qiongli 2.0 R3G Current-target Native Artifact Design

Date: 2026-07-14

Status: frozen for implementation

Roadmap slice: `R3G / PKG-201A`

Scope: one unpublished current-target portable-archive staging tree

## Outcome

R3G assembles the canonical Qiongli executable into one deterministic,
target-specific artifact staging tree. The tree binds the complete artifact
identity, the executable digest and mode, and the verified embedded-content
identity in a canonical manifest. The copied executable must start as a CLI and
as the Marketplace Lite MCP server with an empty runtime `PATH`.

This is the smallest artifact boundary needed by later production launch
grants, signing, compression, installers, and service-backed install actions.
It is deliberately an unpublished staging tree rather than a public release
asset. R3G does not create a compressed archive, platform signature, launch
grant, checksum sidecar, SBOM, provenance statement, installer, updater record,
or release receipt.

## Canonical Identity

The artifact uses the ADR 0207 tuple:

```text
(qiongli, 2.0.0-alpha.1, alpha, lite, <current-os>, <current-arch>, portable-archive)
```

The composer accepts a typed `ArtifactIdentityV1` and rejects every identity
that is not:

- product `qiongli`;
- a valid version/channel pair;
- profile `lite`;
- installer kind `portable-archive`; and
- the operating system and architecture of the running composer.

No `current`, `host`, `any`, or implicit target token enters the manifest. The
current target is discovered through the compiled Rust target and materialized
as a concrete enum value before composition.

The target directory leaf is the deterministic artifact ID:

```text
qiongli-<version>-<channel>-<profile>-<os>-<arch>-portable-archive
```

The manifest remains authoritative; consumers never infer trust from the leaf
name alone. The leaf-name check prevents a caller from accidentally staging a
concrete native payload under a generic or wrong-target name.

## Staging-tree Contract

The committed tree is:

```text
qiongli-<identity>/
  .qiongli-native-artifact.json
  bin/
    qiongli              # macOS and Linux
    qiongli.exe          # Windows
```

`.qiongli-native-artifact.json` is canonical RFC 8785 JSON with
`schema_version: 1`, record type `qiongli-native-artifact`, and status
`assembled-unpublished`. It binds:

- the complete `ArtifactIdentityV1` tuple;
- the exact binary path, logical executable mode, byte length, and SHA-256;
- the embedded pack ID, content version, content source commit, pack SHA-256,
  and content-root SHA-256 supplied by a verified `LoadedResourcePack`;
- the complete managed-entry list; and
- a domain-separated artifact content-root SHA-256 over the managed entries.

The manifest excludes itself from the content root so canonical bytes can be
computed without a recursive digest. Its own SHA-256 is returned by the
verifier and can become an input to later launch-grant and release work.

The `assembled-unpublished` status is a closed enum. It makes the missing
release gates machine-readable and prevents this receipt from being mistaken
for publication, signing, or target-native clean-machine acceptance evidence.

## Composition And Verification

The public platform API consists of:

- current-target identity construction;
- deterministic artifact-ID and binary-relative-path helpers;
- explicit target approval at a trusted CLI, UI, release, or test boundary;
- staging-tree composition; and
- complete-tree verification.

Composition requires a verified embedded resource pack, a regular executable,
the exact current-target identity, and an approved absolute target with the
canonical leaf. The source executable must be non-empty, bounded to 128 MiB,
owned by the current user where the platform exposes ownership, executable on
Unix, and neither a symbolic link, Windows reparse point, nor hard link.

The composer writes a private sibling stage, uses create-new files, fixes
logical modes, syncs files and directories, verifies the complete staged tree,
and promotes it with no-replace semantics. It never overwrites or adopts an
existing target. A target lock prevents two local composers from racing for
the same artifact path. Failure before promotion removes only the
transaction-owned stage.

Verification is independent of composition. It rejects:

- a missing, oversized, non-canonical, unknown-field, or invalid manifest;
- a non-current or otherwise unsupported identity;
- a root whose leaf disagrees with the canonical artifact ID;
- missing or extra files and directories;
- links, reparse points, hard links, non-regular entries, or mode drift;
- binary size, digest, path, or content-root drift; and
- pack identifiers, source commit, versions, or digests outside their bounded
  canonical forms.

Public errors expose fixed reason codes and I/O kinds only. Debug and Display
output do not reveal approved paths, home directories, or source locations.

## Runtime Acceptance

The application integration test composes from `CARGO_BIN_EXE_qiongli` into an
isolated private parent and then launches only the committed artifact binary.
It clears `PATH`, relocates HOME and Qiongli config, and uses a working
directory outside the checkout.

The copied artifact must:

1. return the exact package version from `qiongli --version`;
2. return the embedded pack identity from `qiongli content list`;
3. complete MCP initialize and `tools/list` over stdio;
4. expose the exact 12-tool Marketplace Lite public contract; and
5. complete one bounded read-only MCP call without a language runtime.

The existing copied-binary MCP suite continues to exercise all 12 Lite tools.
R3G adds the missing proof that those same bytes came through the artifact
composer and its manifest verifier.

## Non-claims

R3G does not provide or prove:

- `.zip`, `.tar.gz`, DMG, PKG, MSI, MSIX, AppImage, Flatpak, or another final
  delivery container;
- a production launch grant or any signature;
- notarization, Windows signing, checksum sidecars, SBOM, or provenance;
- publication to GitHub, Codex, Claude, ChatGPT, or another marketplace;
- installation, upgrade, repair, removal, rollback, or A/B update;
- packaged desktop-window or clean-machine acceptance;
- cross-target assembly or cross-compilation acceptance;
- Claude Desktop, cloud/web execution, Full MCP, agents, ToolHost, or
  orchestrator execution.

## Exit Gate

R3G closes when:

1. the platform crate exposes the typed current-target artifact contract;
2. two equivalent inputs produce byte-identical manifests and managed trees;
3. target, identity, source-binary, existing-target, link, hard-link, mode,
   extra-entry, receipt, and digest drift are rejected;
4. the committed artifact verifies independently;
5. the artifact binary passes CLI and Lite MCP startup with empty `PATH`;
6. format, locked workspace check, strict Clippy, all native tests, focused Lite
   compatibility, and Windows MSVC check/Clippy pass; and
7. the accelerated roadmap, native README, rolling Draft PR, and exact-head CI
   report only the accepted facts and explicit non-claims.
