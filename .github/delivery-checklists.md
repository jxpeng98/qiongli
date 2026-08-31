# Repository delivery checklists

Use the smallest verification tier that matches the boundary: **Focused** while
editing, one exact-head **Slice** when a pull request is ready, and
**Acceptance** only for an explicit release candidate. `Machine` items produce
checkable evidence; `Human / authority` items require judgment or a separate
decision. A green check is evidence, not authorization to commit, push, merge,
tag, or publish.

## Pre-commit checklist

- [ ] **Machine** — `git status --short --branch` shows the intended working
      branch and preserves unrelated user changes.
- [ ] **Machine** — `git diff --cached --name-only` contains only explicitly
      authorized paths.
- [ ] **Machine** — `git diff --cached --check` passes, then
      `git diff --cached` has been reviewed in full.
- [ ] **Human / authority** — the staged diff contains no credential, private
      machine path, prompt/response, restricted data, or accidental generated
      output.
- [ ] **Machine** — the smallest task-focused check that can falsify the change
      passes and its exact command/result is recorded. Do not run Slice or
      Acceptance work merely because a commit is being created.
- [ ] **Human / authority** — the commit message names one bounded change and
      follows the repository's Conventional Commit policy.

Completing this section records commit evidence only; it does not authorize a
push.

## Pre-push checklist

- [ ] **Machine** — `git branch --show-current` names the intended working branch,
      not `2.x`, a release branch, or a tag.
- [ ] **Machine** — after `git fetch origin`, review
      `git diff --name-status origin/2.x...HEAD` and
      `git diff --check origin/2.x...HEAD`.
- [ ] **Machine** — `git status --short` is empty and `git rev-parse HEAD`
      identifies the clean checkpoint being pushed.
- [ ] **Machine** — all boundary-appropriate Focused checks pass on that
      checkpoint. Broader package or cross-platform checks are required here
      only when this push deliberately freezes a Slice.
- [ ] **Human / authority** — compatibility, migration, rollback, data-loss,
      claims, non-claims, and follow-up notes are current where the diff affects
      them.
- [ ] **Human / authority** — push intent, upstream, and PR target are explicit;
      no protected branch or accepted-evidence head will be force-pushed.

Completing this section records push evidence only; it does not authorize merge
or release.

## Pull request checklist

- [ ] **Human / authority** — the PR body states the problem/outcome, in-scope
      paths, non-goals, boundary impacts, tests, compatibility, migration,
      rollback, risks, follow-ups, and required reviewers.
- [ ] **Machine** — local `git rev-parse HEAD` equals
      `gh pr view --json headRefOid --jq .headRefOid` before exact-head evidence
      is recorded.
- [ ] **Human / authority** — keep the PR Draft while evidence is moving. Mark a
      source-affecting PR ready only after the Slice is frozen so the required
      Linux, macOS, and Windows matrix runs once deliberately.
- [ ] **Machine** — `gh pr checks --required --watch` passes for the current head,
      including `Evaluation Truth V1` and all protected Native CI contexts.
- [ ] **Human / authority** — every new push invalidates stale exact-head CI,
      package, review, and release evidence; affected evidence has been replaced.
- [ ] **Human / authority** — reviewer/CODEOWNER status is reported truthfully.
      The current independent-reviewer blocker is not represented as approval.
- [ ] **Human / authority** — merge occurs only through the protected PR path.

Completing this section establishes integration evidence only; merge and release
remain separate decisions.

## Release checklist

- [ ] **Human / authority** — freeze the version, claims, non-claims, channels,
      rollback path, and exact merged `2.x` integration commit. A feature-branch
      commit or green PR Slice is not release qualification.
- [ ] **Machine** — `git status --short --branch` is clean on the intended source
      and `git rev-parse HEAD` records that integration commit.
- [ ] **Machine** — run the existing local owner, not copied release logic:
      `./scripts/release_ready.sh --version <version> --staging-dir <external-dir>`.
      For native 2.x this remains a non-publishing dry-run while release blockers
      are present.
- [ ] **Machine** — explicitly dispatch candidate Acceptance with
      `gh workflow run native-ci.yml --ref 2.x`; verify the resulting source with
      `gh run view <run-id> --json headSha,status,conclusion,url`.
- [ ] **Machine** — target packages, checksums, SBOM, provenance, signatures,
      receipts, and advertised channel assets all bind the same frozen source and
      exact bytes. Missing evidence blocks publication.
- [ ] **Human / authority** — obtain a named release decision bound to the exact
      commit, asset digests, channels, claims, and rollback plan. CI cannot grant
      this decision.
- [ ] **Machine** — after any authorized publication, independently download and
      verify every advertised target/channel before announcement.
- [ ] **Human / authority** — announce only verified public bytes with truthful
      capabilities, non-claims, upgrade/rollback guidance, known issues, and
      evidence links.

Tags and published assets are immutable. Changed source, digest, destination,
channel, or claim requires new qualification and authorization.
