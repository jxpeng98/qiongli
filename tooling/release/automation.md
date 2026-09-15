# Release Automation Runbook

## Current native CLI release: 2.0.0

Use the native lane for this release. The retained instructions below describe
the early native dry-run and legacy Python automation, not the current CLI publisher.
`packages/qiongli-native/Cargo.toml` owns the version; synchronize it with
`python3 scripts/sync_versions.py 2.0.0`, regenerate the Skills docs, and review
`tooling/release/v2.0.0.md`. Commit the versioned canonical content first, then
regenerate its embedded lock with the existing owner:

```sh
QIONGLI_NATIVE_SOURCE_COMMIT="$(git rev-parse HEAD)" \
  cargo run --manifest-path packages/qiongli-native/Cargo.toml \
  -p qiongli-content --example update_qiongli_core_lock --locked --offline
```

Review and commit the lock, notes and remaining preparation. Qualify clean main with:

```sh
bash scripts/release_ready.sh --version 2.0.0 --cli-github \
  --staging-dir /tmp/qiongli-2.0.0-qualified
```

The staging directory must be new and outside the checkout. After the authorized
source/tag push, keep remote main frozen at that immutable tag and dispatch:

```sh
gh workflow run release-automation.yml --ref v2.0.0 \
  -f mode=post -f tag=v2.0.0 -f create_release=true
```

The publisher qualifies the tag on macOS ARM64, Windows x64 and Linux x64,
verifies the combined assets, creates the stable GitHub Release and verifies
public downloads before dispatching npm, PyPI and Cargo. Stable uses GitHub
latest and npm latest; npm next can remain on the Beta. Dispatch acceptance does
not establish publication success. No legacy Python, App promotion, local Cargo
upload or external Marketplace catalog change is part of this lane. See
[ADR 0227](../../docs/architecture/decisions/0227-native-main-cutover-and-stable-release-routing.md).

2.0.1 retains actual `plugin-source-status` v1 and the existing local
`qiongli-next@qiongli-cli-local` / public `qiongli-next-<platform>` identities.
This is a stable compatibility patch on npm `latest`. Version 2.1 is the planned
v1-output removal and stable Qiongli identity boundary; consumer migration and
stable/Next target-native upgrade, retry, cancellation and recovery evidence
remain prerequisites. Historical receipt readers and published assets stay intact.

The release owner probes both Hosts through the extracted CLI, installed npm/wheel
and bundled Plugin executables. Probes use disposable secure checkout directories
and isolated configuration, because shared `/tmp` parents are deliberately refused
for Plugin exports. No real Host Plugin is registered. The target receipt retains
these observations; the final packet requires all three targets, v1/Next evidence,
six matching Plugin archives, and the existing source/hash/channel checks.
Historical packets are not required to contain the new 2.0.1 observations.

For 2.0.0 upgrades and recovery, follow the review/update sequence in
[v2.0.1 notes](v2.0.1.md#upgrade-and-recovery--升级与恢复). Retain prior executable,
source receipt and user settings. A failed probe is a failed qualification;
do not change the runtime security policy, skip the probe, or infer Host trust
from source-current. A local macOS ARM64 receipt qualifies only that target;
Windows x64, Linux x64, combined installs and public downloads remain external
gates until their exact-candidate evidence exists.

## Retained earlier workflows

This repository standardizes release with four scripts:

- `scripts/release_ready.sh`
- `scripts/release_preflight.sh`
- `scripts/release_postflight.sh`
- `scripts/release_automation.sh`

Prerelease note draft generator:

- `scripts/generate_release_notes.sh`

Native release identity and non-publishing plan tools:

- `scripts/release_version.py`
- `scripts/native_release_dry_run.py`

## Native 2.x alpha dry-run (`REL-201`)

The authoritative native product version is
`packages/qiongli-native/Cargo.toml` under `[workspace.package]`; its explicit
`[workspace.metadata.qiongli].channel` must agree with the SemVer suffix. Run
native release diagnostics from `2.x` into an explicit directory outside the
checkout:

```bash
OUT_DIR="$(mktemp -d "${TMPDIR:-/tmp}/qiongli-native-release.XXXXXX")"
./scripts/release_automation.sh pre \
  --tag v2.0.0-alpha.1 \
  --materialize-out "$OUT_DIR"
```

The dry-run bundle contains a deterministic JSON release plan, alpha notes, a
target-specific planned-only artifact identity, canonical channel-isolation
metadata, and a rollback/promotion plan. It explicitly records
`publication_performed=false` and `publication_allowed=false`. It does not
materialize 1.x Python/npm/plugin payloads, mutate Git refs, create a GitHub
Release, publish registry or Marketplace records, or claim that a native
artifact was built, signed, or accepted.

The publication-network prohibition is scoped to native publication. CI may
download the pinned Rust toolchain or locked dependencies and upload the
planned-only evidence bundle; that diagnostic traffic does not publish a
native product artifact or release record.

`qiongli-next` distribution metadata records native alpha/beta only as planned
targets. The active artifact allowlist remains legacy 1.x beta, so the current
builder cannot turn frozen 1.x workflow content into a native plugin or claim
that such a plugin is installable.

`release_automation.sh publish` and native postflight remain fail-closed until
the later package, signing, target-native acceptance, updater, and public
release gates are implemented. A `v2.*` tag is excluded from the frozen 1.x
PyPI and npm jobs. Do not bypass these guards by manually creating a tag.

Alpha, beta, and stable are independent canonical channels. `next` remains a
legacy installation alias, not a native channel name. Promotion creates a new
SemVer version and immutable identity; it never relabels an alpha asset or
moves a mutable alias across channels.

## 1) Legacy 1.x one-command publish

If you want the whole path chained together, use:

```bash
./scripts/release_automation.sh publish --tag v0.1.0 --from-tag v0.1.0-beta.6
```

`publish` is the routine legacy 1.x release entrypoint. Native 2.x publishing
is deliberately blocked at `REL-201`; do not manually create release tags or
dispatch production publish workflows to bypass that boundary.

This mode runs:

- `scripts/release_ready.sh`
- release-prep commit creation
- push of the release branch
- hard waiting for branch checks (`CI`, `Checkout Install Check`) on the release-prep commit
- annotated tag creation and tag push only after branch checks pass
- hard waiting for tag publish workflows (`Publish to PyPI`, `Publish to npm`)
- `scripts/release_postflight.sh --create-release`
- marketplace artifact generation for Codex and Claude Code

Stable tags publish from the primary branch (`main` or `master`) and become normal GitHub Releases. Beta tags may publish from `dev` or the primary branch and become GitHub prereleases, so stable and beta releases can coexist without breaking `releases/latest`.

Beta releases are optional validation releases, not a required step before every stable release. Use beta when the release changes high-risk surfaces such as release automation, package payload layout, installer behavior, package metadata, CI, or publish workflows. Routine docs, small fixes, and low-risk maintenance may publish directly as stable.

Native Lite artifacts currently use a `current-host-only` target policy. The
release page may publish those artifacts with an embedded target identity and a
machine-readable artifact record, but postflight must not advance the generic
Codex or Claude marketplace dist refs while that policy is active. Users may
install a matching target-identified beta asset for validation; the generic
marketplace entries remain on the last multi-target-compatible release until
the native matrix is available.

If a stable release ships without a matching beta, the prerelease channel remains on the previous beta. For npm, `latest` advances with stable releases while `next` continues to point at the most recent beta. This is intentional: `next` means "latest prerelease validation build", not "newer than latest stable". Do not publish a mechanical beta only to move `next`.

The release page receives these installable distribution artifacts:

- `qiongli-zotero-companion-<companion-version>.xpi`
- `qiongli-zotero-companion-updates.json` (Zotero-consumed stable update
  metadata, bound to the release tag and XPI SHA-256)
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
Each Claude plugin tarball also has a parallel `.zip` sibling, for example
`qiongli-claude-plugin-<tag>.zip` and
`qiongli-economics-claude-plugin-<tag>.zip`, so Claude upload flows that reject
`.tar.gz` can install the same bundled MCP plugin payload directly.

The release page also receives focused Claude Desktop ZIPs for `core`, `economics`, `business`, `finance`, `political-economy`, `geoeconomics`, and `economics-accounting`, plus the legacy core alias `qiongli-claude-desktop-skill-<tag>.zip`.

The unqualified `qiongli-*plugin` artifacts remain the default core-compatible entries. The subject-qualified Codex and Claude Code artifacts let the shared Skillsplace marketplace expose separate install choices such as `qiongli-economics`, `qiongli-business`, `qiongli-finance`, `qiongli-political-economy`, `qiongli-geoeconomics`, or `qiongli-economics-accounting`, each with its own plugin manifest and materialized `subject/complete` skill package. These artifacts make the release consumable by the supported client-native install surfaces. They do not bypass official directory review: Codex marketplace listing and Claude official plugin directory submission still follow each platform's external submission process when applicable.

## 2) Prepare a publish-ready local state

```bash
./scripts/release_ready.sh --version 0.1.0 --from-tag v0.1.0-beta.6
```

This is the recommended local entrypoint. It chains:

- `scripts/bump-version.sh`
- `scripts/release_automation.sh pre`
- `scripts/verify_release_tag_version.sh`
- `scripts/release_local_install_check.py`
- `scripts/pypi_preflight.sh`
- `scripts/npm_preflight.sh`

When it succeeds, the repository is in a publish-ready state with synchronized version files, validated release docs, an isolated local plugin install acceptance run, and built package artifacts.

The synchronized version files include package metadata, the portable workflow version, and skill registry metadata. Client-native plugin manifests are generated from `content/distribution/plugins.yaml` during staged materialization instead of being edited in the source checkout.
The local install acceptance runs `qiongli install --surface plugin --parts plugin,mcp` against the staged release root inside a temporary HOME, then validates Codex/Claude local plugin payloads, Antigravity/Hermes MCP configs, and `qiongli check --offline --json` discovery.

## 3) Manual pre-release gates (optional)

```bash
./scripts/release_automation.sh pre --tag v0.1.0 --from-tag v0.1.0-beta.6
```

Runs validator + repository unit tests + release-tier smoke checks, verifies the tag is not already used, and then:

- beta / prerelease tags: auto-generate `tooling/release/<tag>.md` draft if missing
- stable tags: verify the matching version section already exists in `CHANGELOG.md`

After checks pass, preflight auto-fills validation evidence lines in prerelease notes.

Manual prerelease draft generation (optional):

```bash
./scripts/generate_release_notes.sh --tag v0.1.0 --from-tag v0.1.0-beta.6
```

The draft generator remains available, but the default policy is now:

- stable tags publish from `CHANGELOG.md`
- prerelease tags publish from `tooling/release/<tag>.md`

## 4) Publish from the release branch

Stable releases run from the primary branch. Beta releases run from `dev`:

```bash
git switch dev
./scripts/release_automation.sh publish --tag v0.8.0-beta.1 --skip-bump --from-tag v0.7.0-beta.2
```

The command above owns the release-prep commit, branch push, branch CI/check gate, tag creation,
tag push, registry publish wait, GitHub Release, and acceptance receipt. It does not create the
tag or publish package registries until the release-prep commit has passed the required branch
checks.

## 4.1) Resume after release-ready failures

If `publish` has already completed `release_ready.sh` and created the release-prep commit, but
then fails during branch push, branch CI wait, local tag creation, tag push, tag workflow wait, or
postflight, resume with:

```bash
./scripts/release_automation.sh publish --tag v0.1.0 --resume-after-ready
```

This recovery path skips `release_ready.sh` and skips the release-prep commit step. It requires a
clean working tree, treats local and remote tags as resumable when they already point at the
release-prep commit, still waits for the required branch checks before tag publication, and then
continues through postflight and the acceptance receipt. Do not manually assemble the remaining
`git push`, `git tag`, and `post` commands for this state.

## 5) Post-release checks

```bash
./scripts/release_automation.sh post --tag v0.1.0 --create-release
```

Runs local/remote consistency checks, checks branch CI + tag publish status, checks release docs + rollback docs, and generates:

- `tooling/release/acceptance/v0.1.0-receipt.md`

It also runs:

```bash
python3 scripts/build_marketplace_artifacts.py --tag <tag> --dist-dir dist
```

When `--create-release` is used, the generated Codex and Claude Code artifacts are attached to the GitHub Release alongside the Python package artifacts. Stable release pages are created from a generated notes file that combines the matching `CHANGELOG.md` section with a release-category summary and a download guide. If the GitHub Release already exists, postflight uploads those marketplace artifacts with `--clobber`.

## Optional flags

- `--tag <tag>`: accepts stable (`v1.19.0`), legacy beta (`v1.19.0-beta.1`), and native alpha (`v2.0.0-alpha.1`) identities. Native alpha is accepted by `pre`, not by `publish` or production `post`.
- `--version <version>`: accepts the corresponding canonical or compact input forms; output is always normalized to SemVer/tag form.
- `--skip-smoke`: skip smoke stage during preflight.
- `--maintainer-smoke`: upgrade preflight smoke from the default release tier to the maintainer tier (`parallel` + `task-run` profile checks).
- `--skip-note-gen`: skip prerelease draft generation of `tooling/release/<tag>.md`.
- `--note-overwrite`: overwrite existing `tooling/release/<tag>.md` when generating prerelease draft.
- `--from-tag <tag>`: choose baseline tag used for prerelease draft highlights.
- `--skip-bump`: start `release_ready.sh` from preflight/package checks only.
- `--allow-dirty`: let `release_ready.sh` continue on a dirty worktree.
- `--resume-after-ready`: resume `publish` after a completed `release_ready.sh` and release-prep
  commit; this skips repeated preflight/package checks and continues branch push, branch CI gate,
  tag publication, postflight, and acceptance receipt generation.
- `--commit-message <msg>`: override the release-prep commit message used by `publish`.
- `--push-remote <name>` / `--push-branch <name>`: override the remote/branch used by `publish`.
- `--wait-ci`: accepted for compatibility; publish mode always waits for required branch checks
  before tag creation and tag publish workflows before release creation.
- `--ci-timeout-seconds <n>` / `--ci-poll-interval-seconds <n>`: control publish wait behavior.
- `--ci-timeout-mode hard`: publish mode requires hard CI gates. `soft` is available only for
  manual `post` diagnostics or recovery, not for routine publishing.
- `--skip-remote`: skip remote ref checks in postflight.
- `--skip-ci-status`: skip GitHub Actions status checks in postflight.
- `--create-release`: if `gh auth` is available, create the GitHub release page from the prerelease note file, or from generated stable notes that combine the matching `CHANGELOG.md` section with release-category and download guidance.

## GitHub workflow entrypoint

- Workflow: `.github/workflows/release-automation.yml`
- Trigger: `workflow_dispatch`
- Inputs: `mode` and `tag`
- Purpose: diagnostic `pre` / recovery `post` only. Production publishing is intentionally kept in `scripts/release_automation.sh publish` so tag creation and downstream registry workflows have one owner.
