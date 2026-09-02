# Repository delivery checklists

Use **Focused** checks for daily work, one exact-head **Slice** for a ready pull
request, and **Acceptance** only for an explicit release candidate.
A green check is evidence, not authorization to commit, push, merge, publish,
or announce.

Denied, expired, or revoked authorization blocks execution; obtain a new receipt
for the current action. Emergency hotfix authorization is repository-only and
still requires a named incident, minimum scope, exact-head checks, rollback, and
the protected PR path.
Post-incident reconciliation is evidence, not retroactive authorization.

## Pre-commit checklist

- [ ] **Machine** — `git status --short --branch` and the staged path list match the intended bounded change.
- [ ] **Machine** — `git diff --cached --check` passes and the full staged diff has been reviewed.
- [ ] **Machine** — the smallest affected Focused check passes and its exact command/result is recorded.
- [ ] **Human / authority** — no credential, private path, prompt/response, restricted data, or accidental generated output is staged; the Conventional Commit message describes one change.

This records commit evidence only; it does not authorize a push.

## Pre-push checklist

- [ ] **Machine** — the current branch is a working branch, not `2.x`, `release/*`, or a tag.
- [ ] **Machine** — after fetching, review `git diff --name-status origin/2.x...HEAD` and run `git diff --check origin/2.x...HEAD`.
- [ ] **Machine** — the tree is clean, `git rev-parse HEAD` records the checkpoint, and its affected Focused checks pass.
- [ ] **Human / authority** — compatibility, migration, rollback, data-loss, claims, non-claims, upstream, and PR target are explicit where affected. Plain `--force` is forbidden.
- [ ] **Human / authority** — an exceptional rewrite is limited to an unprotected, unpublished feature branch with owner approval and reviewer notice; use `--force-with-lease` only and invalidate evidence for replaced commits.

This records push evidence only; it does not authorize merge or release.

## Pull request checklist

- [ ] **Human / authority** — the PR template records the bounded outcome, paths, non-goals, boundary impact, tests, compatibility, rollback, risks, follow-ups, and reviewers.
- [ ] **Machine** — local `git rev-parse HEAD` equals the PR head before exact-head evidence is recorded.
- [ ] **Human / authority** — keep the PR draft while its head moves; freeze a source-affecting Slice before the required Linux, macOS, and Windows matrix.
- [ ] **Machine** — `gh pr checks --required --watch` passes for the current head, including all protected contexts.
- [ ] **Human / authority** — Every head change invalidates stale exact-head CI and review evidence; report CODEOWNER state truthfully, retain the current independent-reviewer blocker, and merge only through the protected PR path.

This records integration evidence only; merge, publication, and announcement
remain separate decisions.

## Release checklist

- [ ] **Human / authority** — freeze the version, claims, non-claims, channels, rollback, and exact merged `2.x` commit; a PR Slice is not release qualification.
- [ ] **Machine** — record a clean source with `git rev-parse HEAD`, then run `./scripts/release_ready.sh --version <version> --staging-dir <external-dir>`.
- [ ] **Machine** — dispatch exact-source Acceptance with `gh workflow run native-ci.yml --ref 2.x` and verify the run's head SHA and conclusion.
- [ ] **Machine** — packages, checksums, SBOM, provenance, signatures, receipts, and advertised assets bind the same source and bytes; missing evidence blocks publication.
- [ ] **Human / authority** — obtain a named release decision bound to the commit, asset digests, channels, claims, and rollback plan; CI cannot grant it.
- [ ] **Machine** — after authorized publication, independently download and verify every advertised target and channel.
- [ ] **Human / authority** — obtain a distinct announcement decision and receipt bound to verified public bytes and exact claims. Publication authorization does not authorize announcement.

Tags and published assets are immutable. A changed source, digest, destination,
channel, or claim requires new qualification and authorization.
