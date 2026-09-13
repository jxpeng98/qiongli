# Technical Design — Qiongli 2.0.0-alpha.5 release

> Superseded on 2026-09-03. Alpha 5 remains an internal candidate; the public
> publication design below is retained only as historical planning context.

## Existing owners

Reuse the current release spine:

```text
sync_versions + content lock
  -> reviewed 2.x source S
  -> local release readiness + full Native CI N
  -> three-target promotion P
  -> protected exact-set authorization A
  -> offline release signing R
  -> draft GitHub prerelease
  -> public download/startup verification
  -> evidence-only closeout
```

The owners are `scripts/sync_versions.py`, `update_qiongli_core_lock`,
`scripts/release_ready.sh`, `.github/workflows/native-ci.yml`,
`.github/workflows/native-community-alpha-promotion.yml`, and
`native_community_alpha_release`. No parallel implementation is needed.

## Identity contracts

- Preparation base `B` is the clean `2.x` commit used to regenerate the embedded
  content lock.
- Product source `S` is the merged Alpha 5 freeze commit and the Git tag target.
- Native CI run `N`, promotion run/attempt `P`, candidate set `C`, protected
  authorization `A`, signed release set `R`, and public release `G` must all bind
  `S` and `2.0.0-alpha.5`.
- The checked-in content lock keeps its existing preparation-source semantics;
  it is not substituted for `S` in package or publication receipts.
- Any changed candidate byte changes the release-set digest and invalidates `A`.

## Release inventory

The five platform assets come from the verified aggregate candidate:

- macOS arm64 ZIP and DMG;
- Windows x86_64 portable ZIP;
- Linux x86_64 AppImage and portable ZIP.

The offline owner adds the candidate-set receipt, release authority, SHA-256
inventory, CycloneDX SBOM, SLSA provenance, bilingual release notes, signed
integrity manifest, and protected publication-authorization receipt. Its
directory verifier is the canonical closed-set check before upload.

## Security and authority

- GitHub Actions receives only the public authority and emits an exact-set
  authorization after the protected-environment review.
- The `community-alpha-release-1` private key enters only the maintainer process
  through `QIONGLI_ALPHA_RELEASE_PRIVATE_KEY_HEX`, is held by the existing
  zeroizing Rust path, and is unset immediately after signing.
- Do not log the environment, shell-trace the command, store the key in a file,
  or move signing into CI.
- Protected authorization expires after 86,400 seconds; candidate download,
  signing, verification, and draft creation must finish within that window.

## Publication order

1. Confirm no Alpha 5 tag or release exists and remote `2.x` still equals `S`.
2. Qualify `S` locally and through full target-native CI.
3. Produce `C`, inspect its exact inventory, and request `A` only when the
   offline key is locally available.
4. Generate and verify `R` offline.
5. Create a draft prerelease targeting `S`; compare its asset inventory and
   GitHub-reported digests with `R`.
6. Publish that draft without changing its bytes.
7. Download from public URLs and verify again before recording success.

## Failure and rollback

- Before the tag exists, abandon invalid output and rerun from the last valid
  owner. Never repair receipts by hand.
- If `2.x` moves before promotion authorization, restart with the new head or
  restore the intended release source through a reviewed change.
- If the private key is unavailable, retain no active protected authorization;
  stop after the qualified candidate and report the credential blocker.
- If draft inspection fails, delete only the unpublished draft/tag when it was
  created by this attempt, then rerun with a new version if any public byte was
  exposed.
- After publication, do not clobber assets or retarget the tag. Mark the release
  withdrawn or publish a superseding prerelease when correction is required.

## Evidence closeout

One path-redacted receipt records `S`, `N`, `P`, authorization identity and
expiry, exact public asset names/sizes/digests, signed release-set digest, public
release URL, startup results, and rollback disposition. It contains no local
paths, credentials, Host transcripts, or private material. The evidence commit
is distinct from `S` and never masquerades as the released source.
