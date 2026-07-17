# ADR 0208: Community Alpha Distribution Boundary

- Status: Accepted
- Date: 2026-07-17
- Task ID: `PKG-202C`
- Owners: Qiongli maintainers
- Decision scope: Qiongli 2 zero-cost pre-release distribution and promotion
- Extends: ADR 0207 release channels and native artifact identity

## Decision drivers

- Alpha.1 needs a practical macOS, Windows, and Linux testing distribution
  before paid Apple Developer ID and Windows Authenticode credentials exist.
- Removing paid operating-system publisher trust must not remove Qiongli's own
  release signatures, target identity, integrity evidence, or authorization.
- Ordinary short-lived CI artifacts are useful diagnostics but are not a safe
  source for a public release.
- The free lane must describe operating-system warnings truthfully and must not
  tell users to disable global security controls.
- The production-signing lane must remain available without silently relabelling
  already published Community Alpha bytes.

## Context

ADR 0207 defines Alpha, Beta, and Stable channels and the canonical native
artifact identity. It assumes each release advertises and verifies the platform
trust appropriate to its delivery. Qiongli 2.0.0-alpha.1 needs an additional,
orthogonal distinction between a zero-cost test distribution and a later
platform-trusted production distribution.

macOS can apply an ad-hoc code signature without Developer ID or notarization.
Windows can distribute a complete portable directory without Authenticode, but
SmartScreen, Smart App Control, antivirus, or enterprise policy may warn or
block it. Linux can distribute a Type 2 AppImage without a central publisher
identity. Those mechanisms permit testing; they do not establish platform
publisher reputation.

## Decision

### Distribution class is separate from release channel

Every public native release set declares one closed `distribution_class`:

- `community-alpha` is allowed only for an Alpha prerelease and is never Stable
  eligible;
- `production` is reserved for releases that satisfy the advertised production
  platform-trust policy.

The class does not change the ADR 0207 identity tuple. It is release-set policy
that binds the same artifact identities, platform-trust claims, release notes,
update metadata, and publication ledger.

Promotion from Community Alpha to production creates a new SemVer release and
new release set. Previously published bytes are never silently relabelled.

### Closed Community Alpha target matrix

The first Community Alpha has exactly these target policies:

| Target | Platform trust | Public application assets |
|---|---|---|
| macOS arm64 | ad-hoc signed; not Developer ID signed or notarized | first-install DMG and update `.app.zip` |
| Windows x86_64 | unsigned; no Authenticode publisher identity | complete portable application ZIP |
| Linux x86_64 | AppImage without a central platform publisher identity | Type 2 AppImage and portable AppDir ZIP containing `qiongli-cli` |

The Linux portable directory is required while the AppImage launcher is a
desktop-only forwarding surface. A future AppImage that truthfully forwards the
complete CLI may replace that companion only through a versioned schema change.

### Fresh exact-source promotion

Public candidates are not selected from ordinary Native CI artifacts. A
read-only exact-head promotion workflow must:

1. require an explicit source commit equal to the current remote `2.x` HEAD;
2. check out that immutable commit with a clean worktree on every target runner;
3. rebuild and verify each package on its advertised operating system and
   architecture;
4. generate target promotion records that set
   `raw_ci_artifact_reused: false` and `publication_allowed: false`;
5. bind public asset bytes and package/acceptance receipt digests; and
6. aggregate only the three target records produced by that same workflow run.

The R3P-B candidate is still non-publishing. It cannot create a tag, upload a
GitHub Release asset, or mutate an update stream.

### Common Qiongli trust remains mandatory

Community Alpha waives only paid platform publisher trust. Publication still
requires all of the following over the final release set:

- detached Qiongli Ed25519 release and update metadata signatures;
- target, version, channel, profile, installer-kind, size, and digest binding;
- a sorted SHA-256 inventory;
- CycloneDX SBOM and SLSA provenance;
- target-native packaged startup and acceptance evidence;
- English and Chinese installation warnings; and
- explicit authorization for the exact source and release-set digest.

The authorization input is trusted only when a protected workflow compares it
with its own repository, Environment, run, actor, and time context. Copying an
authorization JSON file to another run is not authorization.

### User-visible warnings

Every Community Alpha download surface and release note must use the label
`community-alpha — not platform-trusted` and state the target limitation before
download. Documentation may describe the bounded macOS **Open Anyway** control
or the normal Windows continuation when available. It must not instruct users
to disable Gatekeeper, Smart App Control, antivirus, enterprise policy, or Linux
integrity controls, and it must not install a self-signed Windows root.

A device whose policy blocks the unsigned Windows build is outside the free
Community Alpha support boundary.

## Security consequences

- Platform publisher trust and Qiongli release trust are independent and
  machine-verifiable.
- Unknown distribution classes, targets, asset roles, warnings, receipt fields,
  digests, or authorization contexts fail closed.
- Private release keys remain outside the repository and shipped binaries.
- The promotion workflow has read-only repository permissions and no publishing
  Environment, tag, Release, or update-endpoint authority.
- The protected authorization job also remains read-only and receives no
  private key. The maintainer machine performs final Ed25519 signing and the
  GitHub Release mutation after consuming that short-lived authorization.
- Raw CI and R3P-B candidate artifacts remain non-publishing even when their
  target-native startup check passes.

## Alternatives rejected

- Publishing ordinary seven-day CI artifacts: rejected because they are not a
  fresh, closed, exact-set promotion boundary.
- Self-signing Windows with a locally generated root: rejected because it asks
  users to trust an unestablished certificate authority and can conflict with
  managed-device policy.
- Disabling Gatekeeper or Smart App Control: rejected because it weakens the
  whole device rather than applying a bounded application decision.
- Blocking all testing until paid credentials exist: rejected because Qiongli's
  own cryptographic trust can support a truthful, bounded pre-release lane.
- Treating Community Alpha as production: rejected because it would overstate
  platform trust and make later promotion ambiguous.

## Acceptance evidence

- R3P-A tests close distribution class, platform trust, warnings, target set,
  Stable rejection, and exact authorization context.
- R3P-B tests close fresh-build provenance, public asset roles, evidence roles,
  target order, candidate digest, and non-publishing status.
- The promotion workflow must pass on the exact merged `2.x` head before R3P-B
  has target-native execution evidence.
- R3P-C integrity and R3P-D protected-authorization contracts are implemented;
  their exact-head execution and the public release remain required.

## References

- `docs/architecture/decisions/0207-release-channel-and-artifact-identity.md`
- `docs/superpowers/specs/2026-07-17-qiongli-community-alpha-distribution-note.md`
- `docs/superpowers/roadmaps/2026-07-13-qiongli-2-accelerated-rust-migration-roadmap.md`
- `tooling/release/v2.0.0-alpha.1.md`
