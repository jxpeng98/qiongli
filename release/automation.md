# Release Automation Runbook

This repository standardizes release with four scripts:

- `scripts/release_ready.sh`
- `scripts/release_preflight.sh`
- `scripts/release_postflight.sh`
- `scripts/release_automation.sh`

Prerelease note draft generator:

- `scripts/generate_release_notes.sh`

## 1) One-command publish

If you want the whole path chained together, use:

```bash
./scripts/release_automation.sh publish --tag v0.1.0 --from-tag v0.1.0-beta.6
```

`publish` is the only routine release entrypoint. Do not manually create release tags and do not manually dispatch production publish workflows for normal releases.

This mode runs:

- `scripts/release_ready.sh`
- release-prep commit creation
- annotated tag creation
- push of the release branch + tag
- waiting for branch checks (`CI`, `Checkout Install Check`) and tag publish workflows (`Publish to PyPI`, `Publish to npm`)
- `scripts/release_postflight.sh --create-release`
- marketplace / extension artifact generation for Codex, Claude Code, and Gemini CLI

Stable tags publish from the primary branch (`main` or `master`) and become normal GitHub Releases. Beta tags may publish from `dev` or the primary branch and become GitHub prereleases, so stable and beta releases can coexist without breaking `releases/latest`.

Beta releases are optional validation releases, not a required step before every stable release. Use beta when the release changes high-risk surfaces such as release automation, package payload layout, installer behavior, package metadata, CI, or publish workflows. Routine docs, small fixes, and low-risk maintenance may publish directly as stable.

If a stable release ships without a matching beta, the prerelease channel remains on the previous beta. For npm, `latest` advances with stable releases while `next` continues to point at the most recent beta. This is intentional: `next` means "latest prerelease validation build", not "newer than latest stable". Do not publish a mechanical beta only to move `next`.

The release page receives these installable distribution artifacts:

- `qiongli-codex-plugin-<tag>.tar.gz`
- `qiongli-claude-plugin-<tag>.tar.gz`
- `qiongli-core-codex-plugin-<tag>.tar.gz`
- `qiongli-core-claude-plugin-<tag>.tar.gz`
- `qiongli-economics-codex-plugin-<tag>.tar.gz`
- `qiongli-economics-claude-plugin-<tag>.tar.gz`
- `qiongli-accounting-codex-plugin-<tag>.tar.gz`
- `qiongli-accounting-claude-plugin-<tag>.tar.gz`
- `qiongli-business-codex-plugin-<tag>.tar.gz`
- `qiongli-business-claude-plugin-<tag>.tar.gz`
- `qiongli-finance-codex-plugin-<tag>.tar.gz`
- `qiongli-finance-claude-plugin-<tag>.tar.gz`
- `qiongli-political-economy-codex-plugin-<tag>.tar.gz`
- `qiongli-political-economy-claude-plugin-<tag>.tar.gz`
- `qiongli-geoeconomics-codex-plugin-<tag>.tar.gz`
- `qiongli-geoeconomics-claude-plugin-<tag>.tar.gz`
- `qiongli-economics-accounting-codex-plugin-<tag>.tar.gz`
- `qiongli-economics-accounting-claude-plugin-<tag>.tar.gz`
- `qiongli-gemini-extension-<tag>.tar.gz`

The release page also receives focused Claude Desktop ZIPs for `core`, `economics`, `business`, `finance`, `political-economy`, `geoeconomics`, and `economics-accounting`, plus the legacy core alias `qiongli-claude-desktop-skill-<tag>.zip`.

The unqualified `qiongli-*plugin` artifacts remain the default core-compatible entries. The subject-qualified Codex and Claude Code artifacts let the shared Skillsplace marketplace expose separate install choices such as `qiongli-economics`, `qiongli-business`, `qiongli-finance`, `qiongli-political-economy`, `qiongli-geoeconomics`, or `qiongli-economics-accounting`, each with its own plugin manifest and materialized `subject/complete` skill package. These artifacts make the release consumable by the three client-native install surfaces. They do not bypass official directory review: Codex marketplace listing, Claude official plugin directory submission, and Gemini gallery publication still follow each platform's external submission process when applicable.

## 2) Prepare a publish-ready local state

```bash
./scripts/release_ready.sh --version 0.1.0 --from-tag v0.1.0-beta.6
```

This is the recommended local entrypoint. It chains:

- `scripts/bump-version.sh`
- `scripts/release_automation.sh pre`
- `scripts/pypi_preflight.sh`

When it succeeds, the repository is in a publish-ready state with synchronized version files, validated release docs, and built package artifacts.

The synchronized version files include package metadata, the portable workflow version, skill registry metadata, and client-native distribution manifests under `.agents/`, `.claude-plugin/`, and `plugins/qiongli/`.

## 3) Manual pre-release gates (optional)

```bash
./scripts/release_automation.sh pre --tag v0.1.0 --from-tag v0.1.0-beta.6
```

Runs validator + repository unit tests + release-tier smoke checks, verifies the tag is not already used, and then:

- beta / prerelease tags: auto-generate `release/<tag>.md` draft if missing
- stable tags: verify the matching version section already exists in `CHANGELOG.md`

After checks pass, preflight auto-fills validation evidence lines in prerelease notes.

Manual prerelease draft generation (optional):

```bash
./scripts/generate_release_notes.sh --tag v0.1.0 --from-tag v0.1.0-beta.6
```

The draft generator remains available, but the default policy is now:

- stable tags publish from `CHANGELOG.md`
- prerelease tags publish from `release/<tag>.md`

## 4) Publish from the release branch

Stable releases run from the primary branch. Beta releases run from `dev`:

```bash
git switch dev
./scripts/release_automation.sh publish --tag v0.8.0-beta.1 --skip-bump --from-tag v0.7.0-beta.2
```

The command above owns the release-prep commit, tag creation, branch/tag push, registry publish wait, GitHub Release, and acceptance receipt.

## 5) Post-release checks

```bash
./scripts/release_automation.sh post --tag v0.1.0 --create-release
```

Runs local/remote consistency checks, checks branch CI + tag publish status, checks release docs + rollback docs, and generates:

- `release/acceptance/v0.1.0-receipt.md`

It also runs:

```bash
python3 scripts/build_marketplace_artifacts.py --tag <tag> --dist-dir dist
```

When `--create-release` is used, the generated Codex, Claude Code, and Gemini CLI artifacts are attached to the GitHub Release alongside the Python package artifacts. If the GitHub Release already exists, postflight uploads those marketplace artifacts with `--clobber`.

## Optional flags

- `--tag <tag>`: preferred by `publish`, accepts stable (`v0.2.0`) and beta (`v0.2.0-beta.1`) tag forms.
- `--version <version>`: compatibility input for `publish`, accepts stable (`0.2.0`) and beta (`0.2.0b1`) forms.
- `--skip-smoke`: skip smoke stage during preflight.
- `--maintainer-smoke`: upgrade preflight smoke from the default release tier to the maintainer tier (`parallel` + `task-run` profile checks).
- `--skip-note-gen`: skip prerelease draft generation of `release/<tag>.md`.
- `--note-overwrite`: overwrite existing `release/<tag>.md` when generating prerelease draft.
- `--from-tag <tag>`: choose baseline tag used for prerelease draft highlights.
- `--skip-bump`: start `release_ready.sh` from preflight/package checks only.
- `--allow-dirty`: let `release_ready.sh` continue on a dirty worktree.
- `--commit-message <msg>`: override the release-prep commit message used by `publish`.
- `--push-remote <name>` / `--push-branch <name>`: override the remote/branch used by `publish`.
- `--wait-ci`: wait for required branch checks and tag publish workflows to succeed before release creation.
- `--ci-timeout-seconds <n>` / `--ci-poll-interval-seconds <n>`: control publish wait behavior.
- `--skip-remote`: skip remote ref checks in postflight.
- `--skip-ci-status`: skip GitHub Actions status checks in postflight.
- `--create-release`: if `gh auth` is available, create GitHub release page from the prerelease note file or the matching `CHANGELOG.md` section for stable tags.

## GitHub workflow entrypoint

- Workflow: `.github/workflows/release-automation.yml`
- Trigger: `workflow_dispatch`
- Inputs: `mode` and `tag`
- Purpose: diagnostic `pre` / recovery `post` only. Production publishing is intentionally kept in `scripts/release_automation.sh publish` so tag creation and downstream registry workflows have one owner.
