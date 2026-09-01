# Release Qiongli 2.0.0-alpha.5

## Goal

Publish `v2.0.0-alpha.5` as a limited Community Alpha GitHub prerelease from one
exact merged `2.x` source, with fresh macOS arm64, Windows x86_64, and Linux
x86_64 artifacts and public-download verification. Stop the current development
chain after the release and its evidence closeout are complete.

## Background

- Local and remote `2.x` are clean and aligned at
  `4454b57b7e78a31d34de1d261dedc8234087e26d`.
- Source identities currently use `2.0.0-alpha.4`; Alpha 4 is a private,
  non-publishing candidate. The only public Qiongli 2 release is
  `v2.0.0-alpha.1`.
- Native CI already owns the exact-source Linux, macOS, Windows, Lite,
  packaged-product, package-startup, and candidate-installation checks.
- Community Alpha promotion already rebuilds all three targets, aggregates an
  exact five-binary candidate set, and can emit a short-lived authorization
  through the protected `community-alpha-publication` environment.
- `native_community_alpha_release` already owns offline Ed25519 signing,
  checksums, SBOM, provenance, authorization consumption, and final verification.
- The corresponding release private key is intentionally absent from the
  repository and GitHub Actions. It is not exported in the current process.

## Requirements

### R1 — Freeze one truthful Alpha 5 identity

- Advance the existing native, lockfile, Plugin/Skill, workflow, MCPB, embedded
  content, changelog, and release-note identities to `2.0.0-alpha.5` through the
  existing version synchronizer and content-lock generator.
- Describe Alpha 5 as a public test prerelease with manual installation and
  replacement. Do not claim Stable eligibility, automatic updates, production
  OS signing, notarization, Authenticode, package-manager publication, or a
  roadmap milestone exit.
- Do not add another builder, workflow, receipt schema, signing service, or
  dependency.

### R2 — Qualify the exact merged source

- Merge the reviewed version-freeze PR to `2.x` and bind all later evidence to
  that exact product source `S`.
- Run the existing local native release readiness checks against `S`.
- Explicitly run full Native CI for `S`; every required Linux, macOS, Windows,
  Lite, packaged-product, package, and candidate-lifecycle job must pass.
- Build a fresh three-target candidate from `S`. Historical Alpha 3/4 artifacts
  or receipts cannot qualify Alpha 5.

### R3 — Authorize and assemble the exact public release

- Before opening the 24-hour authorization window, require secure local access
  to the existing `community-alpha-release-1` private key. Never print, commit,
  upload, persist in receipts, or send it to GitHub.
- Request protected-environment publication authorization for the exact
  candidate set and consume only the matching short-lived authorization.
- Use the existing offline release example to create and verify the closed
  release directory, including the five platform binaries, candidate receipt,
  release authority, checksums, CycloneDX SBOM, SLSA provenance, bilingual
  notes, signed integrity manifest, and publication-authorization receipt.
- Create a draft GitHub prerelease targeting `S`, verify its exact asset
  inventory and digests, then make that same immutable release public. Never
  replace an existing tag or asset in place.

### R4 — Verify the public tester path

- Download the published assets from the public release into a fresh temporary
  directory and verify the inventory, GitHub digests, SHA-256 document,
  integrity signature, source, version, and authorization binding.
- Re-run a macOS startup check from the freshly downloaded ZIP. Accept the
  Windows and Linux startup results only from the same exact-source target-native
  promotion jobs; do not infer one platform from another.
- Confirm the public release is non-draft, marked prerelease, and targets `S`.
- Record a path-redacted acceptance receipt and update Program Ledger states
  only where this Alpha 5 evidence actually satisfies the owning criteria.

### R5 — Preserve rollback and evidence boundaries

- Any source or package-input change after `S` invalidates downstream candidate,
  authorization, signing, and publication evidence and restarts qualification.
- Before publication, discard a failed candidate and rerun. After publication,
  keep the immutable release and supersede or withdraw it explicitly; never
  clobber public bytes.
- Publication does not authorize a separate announcement, update-channel
  mutation, package-manager submission, or Stable promotion.

## Acceptance Criteria

- [ ] All owned source identities and release notes agree on `2.0.0-alpha.5`.
- [ ] The version-freeze PR is merged and a clean exact product source `S` is
      recorded.
- [ ] Local release readiness and one explicit full Native CI run pass for `S`.
- [ ] One fresh promotion run produces and verifies exactly the macOS arm64,
      Windows x86_64, and Linux x86_64 candidate set for `S`.
- [ ] Protected authorization, offline signing, and authorization consumption
      bind the same candidate set without exposing private key material.
- [ ] `v2.0.0-alpha.5` is a public GitHub prerelease targeting `S` with the exact
      verified release inventory and no in-place replacement.
- [ ] Fresh public download, checksum/integrity verification, macOS startup, and
      target-native Windows/Linux startup evidence pass.
- [ ] A path-redacted closeout receipt is merged; unsupported roadmap claims
      remain open; the task is archived with clean synchronized local `2.x`.

## Out of Scope

- Stable, M0/M1 exit, production Developer ID/notarization, Windows
  Authenticode, automatic-update publication, Homebrew/Scoop/WinGet, official
  marketplace submission, or a public announcement campaign.
- New release infrastructure or tests for behavior already covered by existing
  exact-source owners.
- Rotating or regenerating release keys. If the existing private key cannot be
  supplied securely, stop before protected authorization and publication.

## Key Decisions

- Release class: limited public Community Alpha tester prerelease.
- Target set: macOS arm64, Windows x86_64, and Linux x86_64.
- Update model: manual replacement; no automatic-update claim.
- Evidence model: one exact-source chain; rerun existing owners instead of
  writing another pipeline.
- Ledger model: close only criteria proved by Alpha 5; preserve all other gaps.
