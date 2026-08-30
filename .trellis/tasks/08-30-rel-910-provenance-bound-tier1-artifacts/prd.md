# REL-910 provenance-bound Tier 1 artifacts

## Goal

Produce one non-publishing Qiongli 2 candidate whose macOS arm64, Windows
x86_64, and Linux x86_64 artifacts are all built from one accepted `2.x`
source and are bound by exact-byte digests to the GitHub Actions run attempt
that actually built them.

## Background

- Program Ledger v1 defines REL-910 as producing reproducible or fully
  provenance-bound macOS, Windows, and Linux artifacts from one accepted
  source. This task chooses the fully provenance-bound route; byte-identical
  cross-run reproducibility is not required.
- `.github/workflows/native-community-alpha-promotion.yml` already checks the
  current remote `2.x` head, requires a successful exact-source Native CI run,
  rebuilds all three targets, verifies embedded source identity, records every
  public asset and evidence-file SHA-256, and aggregates a digest-bound
  non-publishing candidate set.
- The workflow currently assigns `build_run_url` to the qualifying Native CI
  run even though the public artifacts are rebuilt later in the promotion run.
  The generated SLSA statement therefore names the qualification run as the
  builder/invocation instead of the build run that created the assets.
- Candidate aggregation is also coupled to an unconditional protected
  `community-alpha-publication` Environment job. A non-publishing candidate
  cannot finish green without requesting publication authorization, although
  the roadmap explicitly keeps release authorization separate from green CI.
- Production macOS notarization, Windows Authenticode, package-manager
  projections, lifecycle acceptance, independent public-download verification,
  and revocation are owned by REL-911 through REL-915.

## Requirements

### R1 — One accepted source and closed target set

- Accept only the exact current remote `2.x` commit with a completed successful
  Native CI run for that same commit.
- Reuse the existing target-native rebuild and aggregation path for exactly
  macOS arm64, Windows x86_64, and Linux x86_64.
- Preserve embedded source verification, clean checkout checks, locked Rust
  inputs, target-native startup checks, closed asset/evidence roles, and the
  canonical candidate-set digest.

### R2 — Truthful build provenance

- Set every target receipt and the aggregate candidate `build_run_url` to the
  exact promotion workflow run attempt that created the artifacts:
  `https://github.com/<repo>/actions/runs/<run>/attempts/<attempt>`.
- Continue validating the qualifying Native CI run separately. Do not present
  it as the builder invocation for artifacts produced by another run.
- Keep one identical source commit, product version, build run URL, and ordered
  target set across the candidate.

### R3 — Candidate generation is not publication authorization

- Add one boolean workflow input controlling whether the protected publication
  authorization job runs; default it to `false`.
- The normal exact-source acceptance dispatch must produce a complete
  non-publishing candidate and finish without entering the protected
  Environment.
- When the input is explicitly `true`, preserve the existing read-only,
  protected exact-set authorization path. Never tag, publish, sign with a
  private release key, or grant `contents: write` in this task.

### R4 — Source-bound acceptance evidence

- Run explicit exact-head Native CI after the implementation is merged, then
  let it dispatch the three-target candidate build with publication
  authorization disabled.
- Download the resulting candidate artifact and verify the canonical receipt,
  three ordered target records, five public assets, evidence files, source
  commit, build-attempt URL, candidate-set SHA-256, file sizes, and SHA-256
  digests.
- Record the implementation source, Native CI run, promotion run/attempt,
  candidate-set digest, public asset identities, and `publication_allowed:
  false` in a repository acceptance receipt.

### R5 — Minimum checks only

- Extend the existing branch-policy owner to fail if build provenance points at
  the qualifying CI run or if non-publishing generation again requires
  protected publication authorization.
- Run that focused Python test and the closest existing Rust Community Alpha
  contract tests; do not run the full workspace during iteration.
- Use exact-head Slice/Acceptance CI as the cross-platform proof instead of
  adding another package workflow or umbrella test.

## Acceptance Criteria

- [ ] The workflow records the actual promotion run attempt as
      `build_run_url`, while separately validating the exact-source Native CI
      run.
- [ ] Candidate generation defaults to non-publishing and completes without a
      protected Environment approval; explicit authorization remains available
      only through a true boolean input.
- [ ] One exact merged `2.x` source produces macOS arm64, Windows x86_64, and
      Linux x86_64 target receipts in one successful promotion run.
- [ ] The downloaded canonical candidate has exactly three ordered targets and
      five public assets, and every recorded size and SHA-256 matches the
      downloaded bytes.
- [ ] Candidate source, version, build-attempt URL, target policy, evidence,
      and candidate-set digest verify without drift; publication remains false.
- [ ] Focused policy/Rust checks, exact-head Native CI, promotion aggregation,
      Program Ledger freshness, task validation, and `git diff --check` pass.
- [ ] REL-910 is accepted in Program Ledger v1 with exact evidence, product
      source, and successful Actions run identity.

## Out of Scope

- Claiming or requiring byte-identical rebuilds on independently repeated
  runners.
- Developer ID/notarization, Authenticode/timestamping, or any production trust
  claim (REL-911).
- Homebrew, Scoop, or WinGet publication (REL-912).
- Install, upgrade, repair, rollback, or uninstall lifecycle proof (REL-913).
- Independent verification of public downloads, SBOM, signatures, and
  provenance (REL-914), or revocation/withdrawal policy (REL-915).
- Git tags, GitHub Releases, update metadata mutation, private signing keys,
  protected-Environment approval, or public release authorization.
