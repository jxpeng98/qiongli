# Qiongli R3A Install Plan And Lite Launch Grant Design

Status: implemented and accepted

Date: July 14, 2026

Scope: `PLT-201` typed install-plan contract and signed Lite launch-grant
verification boundary

Branch: `feat/2x-native-alpha1`

PR: rolling Draft PR #63

## Decision

Create the first `qiongli-platform` crate and give it two responsibilities:

1. validate an exact native artifact identity plus an Ed25519-signed Lite
   launch grant; and
2. build and validate a bounded, versioned, deterministic `InstallPlan`
   preview that a later transactional executor can consume.

R3A is deliberately read-only. It defines no filesystem executor, does not
discover or write a client path, does not register a plugin or MCP entry, and
does not claim installation or activation. `PLT-202` will add mutation only
after this contract is stable.

The canonical source-built binary exposes `qiongli install status`. It reports
the current compiled target, the supported local target vocabulary, and the
truth that this unpackaged build has no verified launch grant. It must not
manufacture a development grant or report plan/apply readiness.

## Artifact Identity

`ArtifactIdentityV1` is the exact ADR 0207 tuple:

```text
(product, version, channel, profile, os, arch, installer_kind)
```

The vocabulary is closed and serialized in kebab-case:

- product: `qiongli` only;
- channel: `alpha`, `beta`, or `stable`;
- profile: `skill-only`, `lite`, or `full`;
- OS: `macos`, `windows`, or `linux`;
- architecture: `aarch64` or `x86-64`; and
- installer kind: `native-installer`, `portable-archive`, `plugin-bundle`, or
  `mcpb`.

SemVer and channel must agree. Alpha and beta require a positive numeric
sequence and no extra prerelease identifiers; stable rejects every prerelease.
The identity validator rejects unknown fields, unsupported values, empty
identifiers, and mismatched target/profile data before signature verification.

R3A launch grants accept `profile=lite` only. Supporting `skill-only` or
`full` requires a later explicit contract update; a command argument can never
raise this ceiling.

## Signed Lite Launch Grant

`SignedLaunchGrantV1` contains a strict `LaunchGrantV1` plus one signature:

- schema version and monotonically increasing generation;
- exact artifact identity;
- lowercase SHA-256 identities for the native executable and embedded resource
  pack;
- allowed modes from the closed set `cli` and `lite-mcp`;
- allowed local integration scopes from `codex-local` and
  `claude-code-local`;
- not-before and expiry Unix seconds; and
- an Ed25519 signature carrying a bounded key ID and lowercase hexadecimal
  signature bytes.

The signed bytes are domain-separated canonical JSON:

```text
QIONGLI-LAUNCH-GRANT-V1\0 || JCS(grant)
```

Verification requires caller-supplied, pretrusted public keys. It checks the
closed schema and resource bounds, key ID, Ed25519 signature, time window,
minimum accepted generation, exact artifact tuple, exact binary/resource-pack
digests, requested mode, and requested integration scope. A successful call
returns `VerifiedLaunchGrant`, whose fields are private and which cannot be
constructed without verification.

The repository contains no production signing private key. The product binary
does not accept an arbitrary public key from a user or plan as a trust root.
Packaging will inject the publisher trust roots and signed grant into the
target-specific artifact. Tests use a clearly test-only key and cannot make a
source build installable.

## Install Plan Contract

`InstallPlanV1` records:

- schema version, opaque plan ID, creation and expiry Unix seconds;
- the verified signed grant and exact artifact identity;
- target host, local surface, scope, profile, OS, architecture, and adapter
  version;
- a bounded allowlist of adapter-defined root IDs and symbolic roots;
- bounded typed operations, each with a precondition, observed-state digest,
  postcondition, and inverse operation;
- required approval kinds and an optional outstanding host action; and
- the lowercase SHA-256 semantic preview digest.

The first operation vocabulary is declarative only:

- materialize Qiongli-owned resources below an allowlisted root;
- register a documented plugin source;
- register a bounded native Lite MCP command; and
- remove a matching Qiongli-managed entry as an inverse action.

There is no arbitrary absolute-path write, shell snippet, process launch,
environment expansion, host-cache operation, or generic force flag. Relative
paths and entry keys are portable, bounded, and traversal-free. Every forward
operation requires a typed inverse and postcondition.

The semantic digest covers schema, artifact identity, complete signed grant,
target, allowed roots, normalized operations, preconditions, observed-state
digests, postconditions, inverse operations, approvals, and host action. It
excludes plan ID and creation/display time. Expiry is separately bound by the
future approval record as required by ADR 0206.

Canonical order is part of validation: root IDs, operation IDs, approval
kinds, grant modes, and grant scopes are unique and sorted. Equivalent
semantics therefore produce the same digest even when plan ID and timestamps
differ. Any semantic change changes the digest.

## Local Target Vocabulary

R3A recognizes only documented local families:

- `codex-local`; and
- `claude-code-local`.

Each target descriptor still distinguishes CLI-local and desktop-local host
surfaces plus user or repository scope. This vocabulary does not imply that an
adapter is implemented. ChatGPT web, Codex cloud, Claude cloud, and generic
ChatGPT desktop are not local install targets. Claude Desktop direct-plugin
support remains a later adapter with real activation evidence.

## Bounds And Errors

- signed grant JSON: at most 64 KiB;
- install-plan JSON: at most 1 MiB;
- allowed roots: at most 16;
- operations: at most 128;
- approvals: at most 16;
- command arguments: at most 16 bounded static arguments; and
- identifiers, symbolic roots, relative paths, and entry keys have explicit
  byte limits.

All public errors expose static reason codes. They never echo JSON input, key
IDs, signature bytes, paths, entry keys, command arguments, or digests.
Unknown fields and future schema versions fail closed.

## Canonical CLI Status

`qiongli install status` is the only R3A product command. It emits
schema-version-1 JSON containing:

- contract schema versions;
- current compiled OS and architecture;
- `launch_grant: unavailable` for the ordinary source build;
- `preview: unavailable` and `apply: unavailable`; and
- the two recognized local target families marked `contract-only`.

It performs no home/config lookup, file read, network request, process launch,
or client discovery and must work with an empty `PATH` and no home directory.

## Verification

Unit and integration tests cover:

- strict identity/channel validation;
- canonical grant bytes and Ed25519 success;
- unknown key, invalid signature, tampered field, wrong digest/identity,
  unavailable mode/scope, expiry, not-before, and replay generation;
- strict bounded JSON and unknown-field rejection;
- plan schema, sorted uniqueness, root references, traversal, operation
  inverse/postcondition requirements, and target/grant matching;
- deterministic semantic digests across different plan IDs/timestamps and
  digest changes for every semantic family; and
- copied-binary `install status` with empty `PATH` and no home.

Required gates are the native change boundary, format, locked workspace check,
strict Clippy, all native Rust tests, and Windows MSVC cross-target
check/Clippy. Python, Node, live clients, packaging, signing infrastructure,
and filesystem mutation tests remain outside R3A.

## Nonclaims

R3A does not implement or claim:

- a production publisher key, signed release artifact, or injected launch
  grant;
- install-plan apply, filesystem transactions, receipts, repair, remove, or
  rollback;
- Codex or Claude discovery, registration, enablement, trust, or activation;
- Marketplace/Desktop installation, private-cache mutation, or cloud install;
- native provider credential storage or UI; or
- `v2.0.0-alpha.1` package or release readiness.

## Acceptance Criteria

R3A is complete only when:

1. `qiongli-platform` owns strict artifact, grant, and plan contracts;
2. only a correctly signed, current, matching Lite grant can construct a
   verified capability token;
3. a plan requires that token, is bounded and typed, and has a reproducible
   semantic digest;
4. plan parsing rejects unknown, stale, unsorted, traversal-bearing,
   non-invertible, or target-mismatched input without mutation;
5. the canonical source binary reports the truthful unavailable install state
   on an empty-runtime machine; and
6. all local native and exact-head CI gates pass before roadmap or PR claims
   are updated.

## Approval Record

The user instructed continuation into the next roadmap stage on July 14, 2026.
This authorizes R3A on the existing rolling branch and Draft PR without a new
branch or PR.
