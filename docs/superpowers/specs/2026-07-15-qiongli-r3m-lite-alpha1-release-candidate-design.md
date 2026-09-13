# Qiongli R3M Lite Alpha.1 Release Candidate Design

Status: frozen for implementation

Date: July 15, 2026

Scope: current-target Lite `v2.0.0-alpha.1` release candidate, local-client
materialization, and clean-machine acceptance

Branch: `feat/2x-native-alpha1`

Rolling PR: `#63`

Rebaseline notice: the July 15 displayed-window review found five product gaps
which are now owned by R3N. This document remains the accepted signed-candidate
core design, but R3M alone no longer closes R3 or permits Alpha.1 publication.
See `2026-07-15-qiongli-r3n-alpha1-desktop-app-closure-design.md`.

## Outcome

R3M closes the bounded Lite Alpha.1 release-candidate core. It joins the
accepted R3H portable
archive, R3J signed release envelope, R3D/R3E target-specific PluginBundle
grants, R3K embedded release authority, and R3L activation coordinator behind
one signed release-candidate contract.

The public candidate is a small authenticated set rather than another runtime:

```text
<artifact-id>.zip
<artifact-id>.candidate.json
<artifact-id>.release-notes.md
```

The candidate JSON embeds the signed portable release envelope and exactly one
signed PluginBundle grant for Codex local and Claude Code local. The notes are
detached but their exact name, size, and SHA-256 are covered by the candidate
signature. The portable archive remains the only transport containing the
canonical product binary; verified local plugin sources are deterministically
materialized from that binary and its embedded pack.

Users of the accepted candidate need no Rust, Python, Node.js, Cargo, npm, pip,
or checkout at runtime. Build and release jobs may use the pinned Rust
toolchain; that is not a user runtime dependency.

## Authority And Private-Key Boundary

R3M reuses the two accepted public-key roles:

- a release key authorizes the portable release envelope and the complete
  release candidate;
- a launch-grant key authorizes the portable and target-specific executable
  capabilities; and
- the build embeds only the strict R3K public authority document.

The product, platform crate, repository, release candidate, logs, and receipts
never receive or store a private key. Public construction APIs return only the
domain-separated canonical bytes that an external signer must sign. An
external signer or protected CI boundary returns a key ID and detached
signature. R3M adds no seed, mnemonic, secret environment fallback, arbitrary
signing command, or development-key path.

Concrete production key generation, custody, and the final signing operation
require maintainer-controlled external authority. Test fixtures use explicit
ephemeral keys and can never be published.

## Canonical Candidate Contract

`NativeReleaseCandidateV1` contains exactly:

```text
schema version
assembled-unpublished status
release generation
lowercase source commit
complete current-target portable ArtifactIdentityV1
complete SignedNativeReleaseEnvelopeV1
exact ordered Codex and Claude Code PluginBundle grants
release-notes filename, size, and SHA-256
not-before and expiry timestamps
```

`SignedNativeReleaseCandidateV1` adds one detached Ed25519 release-key
signature. Its signing preimage is:

```text
"QIONGLI-NATIVE-RELEASE-CANDIDATE-V1\0" || RFC8785(candidate)
```

The input is bounded, uses `deny_unknown_fields`, requires byte-canonical JSON,
and keeps every integer within the RFC 8785 safe range. The source commit is a
40- or 64-character lowercase hexadecimal object ID. Candidate, archive, and
notes filenames are derived from the complete artifact identity.

The candidate generation equals the nested release generation. Its validity
window is contained by the portable release envelope and all three launch
grants. The nested portable grant authorizes both supported local scopes. The
two plugin descriptors are ordered exactly as Codex then Claude Code and each
grant authorizes only its named scope.

## Identity And Digest Closure

The portable identity is Lite and `portable-archive`. Each plugin identity is
derived from it by changing only `installer_kind` to `plugin-bundle`.

All three grants must bind the same:

- product, version, release channel, profile, OS, and architecture;
- target-native executable SHA-256;
- embedded resource-pack SHA-256;
- `lite-mcp` mode; and
- accepted validity interval.

The Codex grant must contain only `codex-local`; the Claude Code grant must
contain only `claude-code-local`. Swapped, missing, duplicated, multi-scope,
portable-kind, stale, or otherwise mismatched plugin grants fail before a
target path, plan, or mutation is available.

## Verification Pipeline

Verification is target-specific and proceeds in this order:

1. Parse and validate the strict canonical candidate.
2. Select the exact embedded-authority release key, enforce its generation
   window, and verify the candidate signature.
3. Enforce candidate time, generation floor, channel, current-target artifact,
   and expected source commit.
4. Verify the detached release notes against their signed name, size, digest,
   UTF-8, and bound.
5. Re-run the complete R3J portable release, archive, pack, and portable-grant
   verification for the requested local target.
6. Select exactly the requested plugin descriptor and independently verify its
   launch-grant signature, generation, target scope, mode, binary, and pack.
7. Return `VerifiedNativeReleaseCandidate`, privately retaining the verified
   portable release and target-specific PluginBundle grant.

The verified token is the only input accepted by R3M materialization and
activation. Candidate parsing or a checksum string alone grants no capability.

## Candidate Materialization

The candidate-backed installer uses existing implementations rather than
copying them:

- R3I installs or verifies the portable native payload;
- R3D composes and verifies the fixed Codex source package;
- R3E composes and verifies the fixed Claude Code source package;
- R3L previews and applies the exact target registration plan; and
- existing receipt-backed lifecycle operations diagnose and remove only
  Qiongli-managed state.

Fresh multi-step apply records which steps committed. A later-step failure
rolls back only fresh earlier mutations in reverse order. Existing healthy
state is replayed and never deleted as rollback compensation. Ambiguous
ownership or rollback results retain recovery evidence and fail closed.

Qiongli writes only its documented local source and marketplace-registration
boundaries. Codex and Claude Code continue to own cache, enablement, trust,
reload, and plugin-install actions. The CLI and UI report those actions rather
than modifying client-owned state or driving client UI.

## Release Notes And Claims

The detached release notes must state:

- the exact supported current target;
- CLI, native UI, embedded skills, and Lite MCP availability;
- Codex local and Claude Code local source-registration behavior;
- the required host-owned install/enable/reload action;
- that Full MCP, executing agents, ToolHost, and full orchestrator execution
  target Alpha.2;
- that Claude Desktop, Codex Desktop Marketplace bypass, ChatGPT/Desktop,
  cloud/web execution, and public Marketplace publication are unavailable;
- that updater, managed upgrade, cross-target packages, OS signing/notarizing,
  SBOM, and provenance are not Alpha.1 claims; and
- the exact diagnose, remove, and rollback limitations.

The signature authenticates the notes bytes. Semantic review of release prose
remains a human release gate and is not inferred from a digest.

## Clean-Machine Acceptance

The release candidate is accepted only after a freshly extracted artifact runs
outside the checkout with an empty runtime `PATH` and isolated user/config
roots. The target-native journey covers:

1. CLI version, embedded-skill inspection, and `ui --startup-check`;
2. Lite MCP initialize, exact tools/list, one bounded tool call, and clean EOF;
3. candidate preview with no mutation;
4. native payload and one Codex local integration apply, diagnose, and remove;
5. native payload and one Claude Code local integration apply, diagnose, and
   remove;
6. rollback and unrelated-state preservation; and
7. output checks for paths, credentials, signature bytes, environment values,
   and private fixture canaries.

Real-client acceptance runs only in an explicitly isolated environment when
the supported Codex or Claude Code executable is available. A missing external
client may block publication of that client claim; it does not permit a test
double to be reported as real-client evidence.

An actual displayed desktop window and screen-reader pass require a suitable
interactive target host. `ui --startup-check` is necessary but is not described
as displayed-window evidence.

## Publication Gate

R3M implementation may produce an `assembled-unpublished` test-signed or
production-signed candidate. Publication is a separate irreversible action and
requires all of the following:

- maintainer-approved production public authority and external signatures;
- clean `2.x` merge-candidate source binding;
- exact-target candidate and notes verification;
- current-head native CI and Cloudflare success;
- applicable real-client and interactive UI evidence;
- review of release notes and unsupported surfaces; and
- explicit maintainer authorization to create the tag and GitHub prerelease.

Until every gate is recorded, PR #63 remains Draft and no tag or public release
is created. The R3N global-settings, Skills destination, MCP self-test,
source-session discovery, desktop packaging, and packaged-window acceptance
gates are additional mandatory publication gates after the July 15 rebaseline.

## Non-Claims

R3M itself does not complete the R3N desktop application closure, Full MCP,
direct model backends, executing agents, ToolHost, full orchestration, updater,
in-place upgrade, state import, Tier 1 cross-target matrix, Claude Desktop,
Codex Desktop/ChatGPT Marketplace bypass, cloud/web execution, or public
Marketplace distribution. OS package signing/notarization is promoted to R3N
for advertised Alpha.1 desktop targets; SBOM, provenance, and stable update
metadata remain R5 unless separately implemented and evidenced before
publication.
