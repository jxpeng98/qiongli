# Alpha 4 scope and candidate freeze

## Goal

Prepare `2.0.0-alpha.4` and freeze one private, exact-source macOS, Windows, and
Linux candidate for rapid tester feedback. The candidate remains an expiring
GitHub Actions artifact with `publication_allowed=false`; it is not a tag,
GitHub Release, update-channel publication, or public announcement.

## Background

- The only public Qiongli 2 prerelease is `v2.0.0-alpha.1`.
- The source, Cargo lock, embedded Plugin/Skill content, and release notes are
  currently aligned to `2.0.0-alpha.3`; `CHANGELOG.md` identifies it as an
  unpublished candidate with `publication_allowed=false`.
- Current remote and local `2.x` identify
  `658a376b4abb2664603eb6ff88011087ecb35204`. Evaluation Truth run
  `33447527437` passed that exact commit, but no explicit full Native Acceptance
  or three-target candidate run exists for it.
- The latest accepted lifecycle candidate binds source
  `ca0a4a5d530cf53c14d51968387a2aefe19dc630`, Native CI run `33310992152`,
  and promotion run `33311931096`. Current `2.x` is 30 commits later, including
  product and workflow changes, so those exact-candidate bytes cannot qualify a
  new release.
- The M1 Alpha 4 ledger has 27 accepted items, 15 proposed items, and one
  blocked item. The remaining groups are `GOV-417`—`GOV-418`, `PLT-401`—
  `PLT-408`, `SEC-401`—`SEC-405`, plus the single-maintainer independent-review
  blocker in `GOV-413`.
- REL-910 and REL-913 already own the provenance-bound candidate and
  installation-lifecycle mechanisms. This task must reuse those workflows and
  owners instead of adding another builder, installer, receipt format, or
  release pipeline.

## Requirements

### R1 — Freeze one private Alpha 4 identity

- Update the native workspace version, lockfile, embedded Plugin/Skill
  identities, embedded content lock, release notes, and `CHANGELOG.md` to
  `2.0.0-alpha.4` as one reviewed release-preparation change.
- State explicitly that this is a private three-target tester candidate with no
  public download, automatic-update, production-signing, or milestone-exit
  claim.
- Do not publish current development under the historical Alpha 3 candidate
  receipt or reuse any source-, package-, Host-, or authorization-bound evidence
  after its owning input changes.

### R2 — Keep unfinished milestone work truthful and separate

- Do not change the live states of `GOV-413`, `GOV-417`—`GOV-418`,
  `PLT-401`—`PLT-408`, or `SEC-401`—`SEC-405` merely to produce a private
  candidate. This task is not M1 exit evidence.
- Preserve the truthful solo-maintainer limitation for `GOV-413`; self-review
  must never be recorded as independent approval.
- Do not pull Stable-only production signing, package-manager, soak, Kernel,
  Graph v2, institutional, or collaboration work into the Community Alpha
  critical path.

### R3 — Freeze and qualify the exact candidate source

- Freeze a clean, merged `2.x` integration commit after all selected source and
  release-note inputs land.
- Run the existing local native release dry-run and version checks against that
  exact source.
- Explicitly dispatch full Native CI on `2.x`; Linux, macOS, Windows, Lite,
  package, packaged-product, and candidate jobs must bind the same exact source.
- Build a fresh three-target candidate through the existing Community Alpha
  promotion workflow. Candidate generation defaults to
  `publication_allowed=false` and must not enter the protected publication
  environment.

### R4 — Preserve evidence and authority boundaries

- Verify downloaded candidate files, closed target inventory, sizes, SHA-256
  values, source commit, version, Native CI run, promotion attempt, and
  candidate-set digest from a temporary directory outside the source tree.
- Any accepted target, package, installation, Host, update, trust, publication,
  or announcement claim must bind the exact frozen source and candidate bytes.
- Release authorization and public-announcement authorization remain separate
  human decisions. This task must not request, infer, fabricate, or consume
  either decision.

### R5 — Prefer reruns over new release code

- Reuse REL-910 provenance aggregation, REL-913 lifecycle acceptance, the
  current Native CI workflow, and the existing release checklist.
- Add code only for a reproduced blocker in the selected Alpha 4 claim. A
  missing run or expired receipt is solved by a new exact-source run, not a new
  abstraction.

## Acceptance Criteria

- [ ] Cargo, lockfile, Plugin/Skill identities, embedded content, release notes,
      and `CHANGELOG.md` agree on `2.0.0-alpha.4`.
- [ ] Claims and non-claims identify a private tester candidate, the closed
      macOS arm64/Windows x86_64/Linux x86_64 target set, manual replacement,
      short artifact retention, and no M1 or Stable exit claim.
- [ ] The live M1 states remain unchanged unless separately accepted through
      their own evidence; `GOV-413` remains truthfully blocked.
- [ ] The embedded-content lock records its preparation source, while native
      build and candidate receipts bind the exact merged product source; neither
      identity reuses historical Alpha 3 candidate evidence.
- [ ] The exact frozen source passes local release preparation and explicit full
      Native CI, then produces one fresh macOS arm64, Windows x86_64, and Linux
      x86_64 candidate with verified byte digests.
- [ ] The private-candidate path creates no tag, GitHub Release, update-channel
      mutation, publication authorization, or public announcement.
- [ ] The downloaded aggregate candidate is independently checked from a private
      temporary directory, and a path-redacted receipt records the product
      source, Native CI run, promotion run/attempt, target files, byte sizes,
      SHA-256 values, candidate-set digest, and `publication_allowed=false`.
- [ ] Program Ledger/index freshness, task validation, exact-head CI evidence,
      and `git diff --check` pass; the repository ends clean on synchronized
      `2.x` after evidence closeout.

## Out of Scope

- Stable eligibility, 1.19 retirement, production Developer ID/notarization,
  Windows Authenticode/timestamping, Homebrew/Scoop/WinGet publication, and the
  90-day post-Stable maintenance transition.
- New installers, builders, candidate schemas, release workflows, package
  managers, public commands, runtime dependencies, or authorization roles
  without a reproduced requirement.
- Post-2.0 Kernel, Graph v2, Evidence/Reproducibility v2, institutional research
  modes, remote collaboration, or additional Host/provider expansion.
- Publishing, tagging, uploading public assets, or announcing from the same
  implementation batch that changes candidate inputs.
- Closing the remaining M1 governance, platform-baseline, or external-content
  task IDs. They remain follow-up work before a public Alpha 4 milestone exit.

## Key Decisions

- End state: private exact-source `2.0.0-alpha.4` tester candidate.
- Distribution: short-lived authenticated GitHub Actions artifact only.
- Publication state: `publication_allowed=false`; no tag or GitHub Release.
- Scope shape: one sequential task because version preparation, exact-head
  qualification, three-target build, and receipt verification produce one
  inseparable candidate identity; no child tasks are needed.
- Implementation strategy: reuse the current REL-910/REL-913 and Native CI
  owners; rerun exact-source evidence instead of adding release infrastructure.
