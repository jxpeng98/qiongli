# Release Rollback Plan

This runbook defines the rollback procedure for Qiongli releases. It is
parameterized for later releases and defaults to the `v1.19.0-beta.1`
prerelease from `dev`.

A rollback must preserve published provenance. Once a tag or package version
may have become public, do not rewrite, move, reuse, or republish that tag or
version. Use a revert commit, channel withdrawal, and a new fixed version.

## Release Variables

Run these commands from a clean Qiongli checkout. Override the values when
handling another release.

```bash
set -euo pipefail

export REPO_SLUG="${REPO_SLUG:-jxpeng98/qiongli}"
export RELEASE_BRANCH="${RELEASE_BRANCH:-dev}"
export RELEASE_TAG="${RELEASE_TAG:-v1.19.0-beta.1}"
export PYPI_VERSION="${PYPI_VERSION:-1.19.0b1}"
export NPM_VERSION="${NPM_VERSION:-1.19.0-beta.1}"
export PYPI_FALLBACK_VERSION="${PYPI_FALLBACK_VERSION:-1.18.0b3}"
export NPM_FALLBACK_VERSION="${NPM_FALLBACK_VERSION:-1.18.0-beta.3}"
```

Fetch the release branch and tags before resolving provenance. If the release
tag exists after the fetch, derive the release commit from that tag. Otherwise,
use the branch tip only while checked out at the exact reviewed
release-preparation commit.

```bash
set -euo pipefail

test -z "$(git status --porcelain)" || {
  echo "rollback requires a clean worktree" >&2
  exit 1
}

git fetch origin "$RELEASE_BRANCH" --tags

if git rev-parse "${RELEASE_TAG}^{commit}" >/dev/null 2>&1; then
  export RELEASE_COMMIT
  RELEASE_COMMIT="$(git rev-parse "${RELEASE_TAG}^{commit}")"
else
  test "$(git branch --show-current)" = "$RELEASE_BRANCH"
  export RELEASE_COMMIT
  RELEASE_COMMIT="$(git rev-parse HEAD)"
fi

git show --no-patch --decorate "$RELEASE_COMMIT"
```

Stop if the displayed commit is not the intended release snapshot.

## Trigger Conditions

Start rollback triage when any of the following is confirmed:

- Validator, unit, install, or acceptance tests fail on the released snapshot.
- The CLI, orchestrator, agent, MCP, skill, or plugin path has a material
  regression.
- A release artifact is incomplete, corrupt, mis-versioned, or built from the
  wrong commit.
- A security, privacy, data-integrity, or academic-reproducibility issue is
  found.
- The release cannot meet its documented compatibility or dependency claims.

## Classify The Release State

Capture all output in the incident record before changing anything.

```bash
set -euo pipefail

git status --short
git fetch origin "$RELEASE_BRANCH" --tags

git ls-remote --tags origin \
  "refs/tags/${RELEASE_TAG}" \
  "refs/tags/${RELEASE_TAG}^{}"

curl -sS -o /dev/null -w 'PyPI HTTP %{http_code}\n' \
  "https://pypi.org/pypi/qiongli/${PYPI_VERSION}/json"

npm view "qiongli@${NPM_VERSION}" version dist-tags --json || true

gh release view "$RELEASE_TAG" \
  --repo "$REPO_SLUG" \
  --json tagName,isDraft,isPrerelease,title,url || true
```

Use these states:

1. **Tag not published:** the remote tag is absent, PyPI returns `404`, npm
   does not contain the version, and no tag-triggered workflow can still
   publish it.
2. **Public or ambiguous:** the remote tag exists, either registry contains the
   version, a tag workflow is running, or registry state cannot be confirmed.

Treat every public or ambiguous state as immutable. Do not delete or move the
remote tag while a publish workflow could still complete.

## Immediate Mitigation

1. Pause release announcements and automatic promotion.
2. Stop new release tags and marketplace updates.
3. Record UTC time, owner, reason, release commit, workflow runs, artifact
   checksums, registry state, and selected fallback versions.
4. Preserve logs, receipts, packages, and artifacts for diagnosis.
5. Validate the fallback in isolation before changing public channels.
6. For credential or supply-chain incidents, follow the security incident
   process in addition to this runbook.

## Isolated Downgrade Validation

Validate the known-good fallback without changing normal installation or
plugin directories.

```bash
set -euo pipefail
(

export ISOLATION_ROOT
ISOLATION_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/qiongli-rollback.XXXXXX")"

mkdir -p \
  "$ISOLATION_ROOT/home" \
  "$ISOLATION_ROOT/codex" \
  "$ISOLATION_ROOT/claude" \
  "$ISOLATION_ROOT/antigravity" \
  "$ISOLATION_ROOT/hermes" \
  "$ISOLATION_ROOT/project" \
  "$ISOLATION_ROOT/npm-cache"

python3 -m venv "$ISOLATION_ROOT/venv"

export CODEX_HOME="$ISOLATION_ROOT/codex"
export CLAUDE_CODE_HOME="$ISOLATION_ROOT/claude"
export ANTIGRAVITY_HOME="$ISOLATION_ROOT/antigravity"
export HERMES_HOME="$ISOLATION_ROOT/hermes"
export npm_config_cache="$ISOLATION_ROOT/npm-cache"

env -u PYTHONPATH "$ISOLATION_ROOT/venv/bin/python" -m pip install \
  "qiongli==${PYPI_FALLBACK_VERSION}"

cd "$ISOLATION_ROOT"

env -u PYTHONPATH "$ISOLATION_ROOT/venv/bin/python" -I -c '
import os
import qiongli

expected = os.environ["PYPI_FALLBACK_VERSION"]
assert qiongli.__version__ == expected, (qiongli.__version__, expected)
print(f"PyPI fallback verified: {expected}")
'

env -u PYTHONPATH \
  HOME="$ISOLATION_ROOT/home" \
  "$ISOLATION_ROOT/venv/bin/qiongli" install \
  --target all \
  --surface plugin \
  --parts plugin,mcp \
  --project-dir "$ISOLATION_ROOT/project" \
  --overwrite

env -u PYTHONPATH \
  HOME="$ISOLATION_ROOT/home" \
  "$ISOLATION_ROOT/venv/bin/qiongli" check \
  --offline \
  --json

export NPM_FALLBACK_CHECK
NPM_FALLBACK_CHECK="$(npm exec --yes \
  --package "qiongli@${NPM_FALLBACK_VERSION}" \
  -- qiongli check --json)"

node --input-type=module -e '
const payload = JSON.parse(process.env.NPM_FALLBACK_CHECK);
const expectedVersion = process.env.NPM_FALLBACK_VERSION;
if (payload?.npm_package?.version !== expectedVersion) {
  throw new Error(`unexpected npm fallback version: ${payload?.npm_package?.version}`);
}
if (payload?.npm_cli?.role !== "asset-manager") {
  throw new Error(`unexpected npm CLI role: ${payload?.npm_cli?.role}`);
}
if (payload?.npm_cli?.python_free !== true) {
  throw new Error("npm fallback asset manager is not Python-free");
}
console.log(`npm fallback verified: ${expectedVersion}`);
'

npm exec --yes \
  --package "qiongli@${NPM_FALLBACK_VERSION}" \
  -- qiongli runtime doctor
)
```

Do not direct users to downgrade until fallback CLI, plugin payload, MCP
configuration, and offline discovery checks pass. Back up user configuration
before replacing an installed plugin or MCP configuration.

Moving a registry channel does not replace already installed skills, plugins,
or MCP files. Affected installations must explicitly reinstall the fallback or
upgrade to the later fixed release.

## Git Tag Rollback

### Tag Not Published

Only delete a local tag after proving that the remote tag is absent and no tag
workflow can still publish it.

```bash
set -euo pipefail

if git ls-remote --exit-code --tags origin \
  "refs/tags/${RELEASE_TAG}" >/dev/null 2>&1; then
  echo "Remote tag exists; preserve it and use the public rollback path." >&2
  exit 1
else
  remote_tag_status="$?"
  test "$remote_tag_status" -eq 2 || {
    echo "Could not prove that the remote tag is absent." >&2
    exit "$remote_tag_status"
  }
fi

if git show-ref --verify --quiet "refs/tags/${RELEASE_TAG}"; then
  git tag -d "$RELEASE_TAG"
fi
```

Do not push the tag after rollback begins.

### Public Or Ambiguous Tag

If the remote tag exists or may have triggered publication:

- keep the local and remote tag unchanged;
- do not force-push, retarget, delete, or recreate it;
- do not publish different files under the same PyPI or npm version;
- withdraw affected channels and publish the fix under a new version.

The retained tag is the immutable provenance record for withdrawn artifacts.

## Published Registry Rollback

### PyPI Yank

PyPI supports a non-destructive release yank through the official project page:

- <https://pypi.org/manage/project/qiongli/releases/>
- <https://docs.pypi.org/project-management/yanking/>

Select the exact `${PYPI_VERSION}` release, choose **Options**, then
**Yank release**. Record a reason that names the known-good fallback or later
fixed release. Do not delete the PyPI release. A yank removes it from normal
resolver selection while preserving files, hashes, provenance, and explicit
exact-version installation.

### npm Deprecation And `next` Rollback

Confirm the fallback, restore `next`, and deprecate only the affected version:

```bash
set -euo pipefail

test "$(npm view "qiongli@${NPM_FALLBACK_VERSION}" version)" \
  = "$NPM_FALLBACK_VERSION"

npm dist-tag add "qiongli@${NPM_FALLBACK_VERSION}" next

npm deprecate "qiongli@${NPM_VERSION}" \
  "Withdrawn after a confirmed regression. Use qiongli@${NPM_FALLBACK_VERSION} or a later fixed release."

npm dist-tag ls qiongli
npm view "qiongli@${NPM_VERSION}" version deprecated --json
```

For a beta rollback, do not change npm `latest`. Do not use `npm unpublish`,
and do not attempt to republish the same version. See the official npm guidance:

- <https://docs.npmjs.com/deprecating-and-undeprecating-packages-or-package-versions/>
- <https://docs.npmjs.com/adding-dist-tags-to-packages/>

### GitHub Prerelease Withdrawal

Preserve the Git tag and withdraw the GitHub prerelease by converting the
release page back to a draft:

```bash
set -euo pipefail

gh release view "$RELEASE_TAG" \
  --repo "$REPO_SLUG" >/dev/null

gh release edit "$RELEASE_TAG" \
  --repo "$REPO_SLUG" \
  --draft \
  --prerelease \
  --title "${RELEASE_TAG} (withdrawn)"
```

This hides the public release page and install links while preserving the tag
and assets for authorized diagnosis. Do not use `--cleanup-tag`.

If GitHub refuses the draft transition and a hard withdrawal is required,
archive checksums and evidence first. Deleting the GitHub release page and
assets requires explicit release-owner approval; retain the Git tag.

### Marketplace And Plugin Channels

- Do not advance Codex, Claude, or other marketplace references to a withdrawn
  release.
- If a reference already advanced, restore it through a reviewed catalog
  commit that points to the previous immutable release.
- Do not overwrite an artifact at an existing versioned URL.
- Withdraw or annotate external marketplace submissions separately when the
  platform does not follow GitHub release state automatically.
- Keep user recovery instructions pinned to an explicit known-good version.

## Commit-Level Rollback

Use a new revert commit on the release branch. Never reset or force-push `dev`.
Set `ROLLBACK_COMMIT` explicitly to the exact offending or
release-preparation commit; do not assume that either the release tag or the
current tip is the correct target after later commits. When more than one
commit caused the problem, review and revert the minimal set one at a time,
newest first.

```bash
set -euo pipefail

: "${ROLLBACK_COMMIT:?Set ROLLBACK_COMMIT to the reviewed offending commit}"
git cat-file -e "${ROLLBACK_COMMIT}^{commit}"

test -z "$(git status --porcelain)" || {
  echo "rollback requires a clean worktree before branch update" >&2
  exit 1
}

git switch "$RELEASE_BRANCH"
git pull --ff-only origin "$RELEASE_BRANCH"

test -z "$(git status --porcelain)" || {
  echo "rollback requires a clean worktree before revert" >&2
  exit 1
}

git show --stat "$ROLLBACK_COMMIT"
git diff "${ROLLBACK_COMMIT}^" "$ROLLBACK_COMMIT"
```

Stop if the displayed diff contains changes that must remain on `dev`. For a
normal non-merge commit:

```bash
set -euo pipefail

git revert --no-edit "$ROLLBACK_COMMIT"
export REVERT_COMMIT
REVERT_COMMIT="$(git rev-parse HEAD)"
```

For multiple independent causes, revert only the minimal confirmed set, newest
first. For a merge commit, review its parents and use an explicitly approved
mainline; do not guess a `-m` value.

## Recovery Validation

Run from the revert commit:

```bash
set -euo pipefail

git diff --check
python3 scripts/validate_research_standard.py --strict

python3 -m unittest \
  tests.test_orchestrator_workflows \
  tests.test_release_automation \
  -v

./scripts/run_beta_smoke.sh --tier maintainer
./scripts/release_preflight.sh --quick
```

After local checks pass, push through the protected review flow and wait for
`CI` and `Checkout Install Check` on the exact revert commit.

Confirm npm channel recovery:

```bash
set -euo pipefail

test "$(npm view qiongli dist-tags.next)" \
  = "$NPM_FALLBACK_VERSION"

test -n "$(npm view "qiongli@${NPM_VERSION}" deprecated)"
```

Confirm that every PyPI file for the affected release is yanked:

```bash
set -euo pipefail

curl -fsSL \
  "https://pypi.org/pypi/qiongli/${PYPI_VERSION}/json" |
python3 -c '
import json
import sys

payload = json.load(sys.stdin)
files = payload.get("urls", [])
assert files, "PyPI release has no files"
assert all(item.get("yanked") is True for item in files), \
    "one or more PyPI files are not yanked"
print("PyPI yank verified")
'
```

Confirm that the remote tag still dereferences to the originally recorded
release commit and that the GitHub release is draft:

```bash
set -euo pipefail

git fetch origin \
  "refs/tags/${RELEASE_TAG}:refs/tags/${RELEASE_TAG}"

export VERIFIED_TAG_COMMIT
VERIFIED_TAG_COMMIT="$(git rev-parse "${RELEASE_TAG}^{commit}")"
test "$VERIFIED_TAG_COMMIT" = "$RELEASE_COMMIT"

gh release view "$RELEASE_TAG" \
  --repo "$REPO_SLUG" \
  --json tagName,isDraft,isPrerelease,title,url

test "$(gh release view "$RELEASE_TAG" \
  --repo "$REPO_SLUG" \
  --json isDraft \
  --jq '.isDraft')" = "true"
```

Rollback is complete only when:

- the fallback passes isolated CLI, plugin, MCP, and discovery validation;
- the revert commit is green on required branch workflows;
- PyPI files are yanked;
- the npm version is deprecated and `next` points to the fallback;
- the GitHub prerelease is withdrawn while its Git tag remains immutable;
- marketplace references no longer direct new users to the affected release;
- the incident record contains commands, outputs, workflow URLs, owners, and
  remaining limitations.

## Forward Recovery

Fix forward with a new version, for example `v1.19.0-beta.2`, `1.19.0b2`, and
`1.19.0-beta.2`. Run the normal release automation and acceptance process. Do
not reuse an existing tag or registry version, even when replacement artifacts
would contain the intended fix.

Unyanking or removing deprecation from the original immutable release requires
a separate release-owner decision and evidence that those exact artifacts are
safe.

## Native 2.x Alpha Dry-Run And Future Promotion

`REL-201` supports a non-publishing native release plan. Its output records
`publication_performed=false`, `publication_allowed=false`, and writes only to
the explicitly selected staging directory outside the checkout. Rolling back
that dry-run means deleting only its three generated
`qiongli-native-release-*` bundle files. Preserve the containing directory and
every unrelated file. Do not yank PyPI, change npm dist-tags, edit Marketplace
records, or modify Git refs because the dry-run does not touch any of those
systems.

The frozen 1.x PyPI, npm, plugin, MCPB, and GitHub feeds remain independent
from native 2.x `alpha`, `beta`, and `stable` channels. A native rollback must
not direct users to a 1.x registry version through update metadata; returning
to 1.x is a separate, previewed migration rollback transaction.

Once a later task authorizes native publication, use these rules:

1. Never move, delete, reuse, or relabel an existing native tag or artifact
   identity.
2. Withdraw a bad channel entry with signed revocation or replacement metadata
   and retain the verified last-known-good installation.
3. Remove or pause only the affected target/profile/installer identity; do not
   advertise another target as a generic fallback.
4. Preserve checksums, signatures, SBOM, provenance, startup receipts, and the
   incident record for the withdrawn identity.
5. Promote alpha to beta, or beta to stable, by creating a new SemVer version,
   new immutable identity, new signatures, and a new acceptance receipt after
   the destination-channel gates pass. Promotion never moves a mutable alias.

Until `PKG-201`, `PKG-202`, `UPD-201`, and the matching release acceptance
gates are complete, native `publish` and production `post` remain fail-closed.

## Known Rollback Risks

- A PyPI yank does not block an explicit exact-version installation.
- npm deprecation warns users but does not prevent exact-version installation.
- Previously downloaded assets, packages, plugins, or MCP binaries cannot be
  remotely revoked.
- Registry, CDN, marketplace, and client caches may delay channel changes.
- A source revert does not downgrade existing user installations.
- Older fallback code may not accept configuration written by a newer release;
  preserve configuration and validate downgrade compatibility.
- A remotely pushed tag can race with registry workflows; preserve the tag and
  use the public rollback path whenever state is uncertain.
