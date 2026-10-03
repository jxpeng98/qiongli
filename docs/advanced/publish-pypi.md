# Publish native Qiongli packages

The 2.x PyPI package carries a Python launcher and the native executable. The npm
package carries its Node launcher and native binaries; Cargo distributes the Rust
source crates. GitHub Release archives contain the executable, README and licence.
Each channel uses the same product version and CLI description while retaining
its own installation instructions. None publishes the old Python application as
the 2.x runtime.

## Version and source

The version source is `packages/qiongli-native/Cargo.toml`.
For example, `2.0.0-beta.6` maps to Python `2.0.0b6`; a stable `2.0.0` keeps that
version in each registry. Prereleases use `2.x`, npm `next` and crates.io prerelease
versions. Stable releases use reviewed `main` and npm `latest`. Cargo has no
`next` tag. The current source version is `2.2.0`; publication requires the
explicit tag-bound workflow below, not just a main merge.

## Beta channel policy

Beta releases are optional validation releases. A stable release advances npm
`latest`; `next` may stay on the earlier beta. Decide the intended channel before
tag creation rather than issuing another beta only to move a label.

## Qualify before publishing

Follow the [release branch policy](../maintainer/release-branch-policy.md) and
`tooling/release/automation.md`. Native CLI distribution builds and installs on
macOS ARM64, Windows x64 and Linux x64/ARM64 (glibc 2.35+). Both Linux targets
build on Ubuntu 22.04 and require audited wheels. New packets require all four
target receipts. Linux ARM64 is included from 2.2.0.
The assembly step verifies platform
artifacts, version identity, hashes and generated packages. Cargo has its own
source-package checks. A successful source build is not the entire release gate.

For a local native CLI release check, the existing entry is:

```sh
bash scripts/release_ready.sh --version 2.2.0 --cli-github \
  --staging-dir /tmp/qiongli-2.2.0-qualified
```

Use the intended version and a new staging directory outside the checkout. Create the immutable tag
only after the required preparation and explicit publication authority.
Pushing a tag alone does not publish the native packages. Dispatch **Release
Automation** (`release-automation.yml`) with `mode=post`, the exact `v2.*` tag and
`create_release=true`; its native publisher qualifies the requested source and
uses verified assets for the release and registry workflows.

## Registry credentials

PyPI uses the configured Trusted Publisher for `publish-pypi.yml`.
npm uses its configured publication credentials. Cargo currently uses the
repository's `CARGO_REGISTRY_TOKEN` in Actions; there is no fallback to a local
upload. Keep secrets out of release notes, package contents and logs.

The legacy Python builder and TestPyPI route belong to `release/1.x-python`.
Main's workflow edits do not update that frozen branch. Legacy preflight scripts
are compatibility checks, not native publication evidence. See the
[retained 1.x publishing guide](https://github.com/jxpeng98/qiongli/blob/5a3ab87fcba67dfbe700f895c0321e455bbbc914/docs/advanced/publish-pypi.md)
only when maintaining that line.

Release assets and published versions are immutable. Correct a bad package with
a new version through the same checks; do not replace the bytes behind a tag.
