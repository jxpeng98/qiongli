# Local Desktop Development and Packaging

This guide is the maintainer fast path for the Qiongli 2 Svelte/Tauri desktop
application. It covers source development, native execution, local package
assembly, and the difference between a source-built package and a releasable
product.

## Know Which Build You Are Running

| Build | Native services | Writes real Qiongli state | Packaged-product authority | Intended use |
|---|---:|---:|---:|---|
| Browser fixture | No | No | No | Fast layout, responsive, and localisation work |
| `cargo run` source App | Yes | After explicit preview/confirmation | No | Native UI, CLI, project, and service development |
| Local source package | Yes | After explicit preview/confirmation | No | Package layout and target-native launch testing |
| Product-controlled acceptance App | Yes | Only inside its isolated test home | Ephemeral test authority | Automated install/update/integration acceptance |
| Promoted release | Yes | After explicit preview/confirmation | Production or Community Alpha authority | Distribution to testers or users |

A normal source build intentionally has no packaged-product authority. Messages
such as `Confirmation is unavailable because this source build has no
packaged-product authority`, unavailable Apply operations, and unavailable
client installation or updates are therefore expected. Do not add a bypass to
make source builds look like released products.

## One-command macOS Build

When you only need to test the complete App on the current Mac—not Windows or
Linux, release acceptance, notarisation, or an installer—run this from the
repository root:

```bash
pnpm desktop:macos
```

Run `pnpm install --frozen-lockfile` once before the first build. The command
builds the Svelte static assets and the current Mac's Rust executable, then
creates a locally ad-hoc-signed App at:

```text
dist/macos/Qiongli.app
```

It does not run cross-platform gates, security scans, the release composer,
notarisation, or product-control acceptance. Build and open it in one command
with `pnpm desktop:macos:open`.

The command deliberately uses Cargo's release profile and enables Qiongli's
`custom-protocol` feature so Tauri serves the embedded Svelte assets instead of
connecting to `http://127.0.0.1:1420`. This is still a source App for local
Tauri/Svelte and native-service testing. It has no packaged-product authority
and is not a distributable release. The longer composer and signing flow later
in this guide remains the release-structure, update-chain, and
distribution-acceptance path.

### Test the model backend from the source App

Model-backend settings and credentials are normal user configuration, not a
packaged-product installation action. They therefore work in the App produced
by `pnpm desktop:macos`. The source App and packaged App both use macOS
Keychain; the JSON settings document contains only an opaque reference and
never the API key.

Use this local acceptance sequence:

1. Run `pnpm desktop:macos:open` and open **Model Backend**.
2. Enable OpenAI Responses and confirm the settings preview.
3. Enter the API key, choose **Save key**, inspect the preview, and confirm it.
4. Quit and reopen `dist/macos/Qiongli.app`. The page must report the key as
   stored and readiness as **Ready**; this proves restart resolution through
   Keychain rather than an in-memory UI value.
5. Choose **Test connection** only when you intentionally want one real OpenAI
   Responses request. The test uses the fixed model, disables provider storage
   and hosted tools, and discards response text.
6. Use **Remove key** and confirm the preview if the credential was created only
   for local testing.

`qiongli config backend status` is the non-network CLI view of the same
readiness state. `qiongli config backend test` intentionally fails as a usage
error unless `--confirm-network-request` is present. Do not pass API keys on a
command line, store them in shell history, or put them in a `.env` file.

This does not grant the source App authority to install Skills, client plugins,
or updates. Use the isolated acceptance command below for those product-owned
writes.

## One-command macOS Install Acceptance

The ordinary source App cannot install Qiongli Skills or client plugins because
it deliberately has no signed product authority. To test those actions without
touching the real `~/.codex` or `~/.claude` directories, run:

```bash
pnpm desktop:macos:acceptance:open
```

This command builds the same embedded Svelte application with an ephemeral
development authority, composes and ad-hoc-signs a non-publishing package, and
then completes automated Skills materialize/verify/refresh plus Codex and Claude
Code install/verify/repair/remove acceptance. Only after all checks pass does it
publish the test App under:

```text
dist/macos-acceptance/current/extracted/Qiongli.app
```

The opened process receives an isolated `HOME` at
`dist/macos-acceptance/current/isolated-home`; it therefore discovers test-only
Codex and Claude directories and cannot write integration state to the real
user home. Use the App's normal preview and confirmation UI to test installation
again interactively. Evidence is recorded in
`qiongli-packaged-product-acceptance.receipt.json` beside that test home.

The package is labelled by its acceptance output location, uses ad-hoc signing,
sets `publication_allowed` to `false`, and has install grants that expire after
one hour. Re-run the command after expiry. Do not copy it into `/Applications`,
open it through Finder, or distribute it: either route would discard the
isolated launch environment or misrepresent non-publishing test evidence.

## Prerequisites

Use the versions exercised by native CI:

- Node.js 24;
- pnpm 11.13.1, as pinned by the root `packageManager` field;
- Rust 1.97.0 with `rustfmt` and `clippy`, pinned by
  `packages/qiongli-native/rust-toolchain.toml`;
- the platform WebView and native build prerequisites required by Tauri 2.

The repository does not require a globally installed Tauri CLI for the
supported commands below. On macOS, install Xcode Command Line Tools. On
Windows, install the MSVC C++ build tools, Windows SDK, and WebView2. Debian or
Ubuntu developers can match CI with:

```bash
sudo apt-get install --no-install-recommends \
  libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev \
  libssl-dev libayatana-appindicator3-dev librsvg2-dev patchelf
```

From the repository root, install the locked frontend dependencies and verify
the selected tools:

```bash
pnpm install --frozen-lockfile
node --version
pnpm --version
(cd packages/qiongli-native && rustc --version)
```

## Fast Svelte UI Loop

Run the Svelte application with its read-only development transport:

```bash
pnpm --dir packages/qiongli-desktop dev
```

Open
`http://127.0.0.1:1420/?fixture=source-read-only`. The fixture provides typed,
deterministic sample data and never invokes native commands or writes project
state. A plain browser URL without the fixture is not a substitute for the
Tauri host because the native IPC bridge is absent.

Use this loop for styling, compact layouts, responsive behaviour, localisation,
and component states. Confirm native actions in the full App before considering
the work complete.

## Run the Full Source App

Build the static Svelte assets, then run the canonical Rust executable:

```bash
pnpm desktop:build
cargo run \
  --manifest-path packages/qiongli-native/Cargo.toml \
  --package qiongli \
  --features custom-protocol \
  --locked
```

`cargo run` opens the desktop window because `qiongli` is the package's default
binary and no CLI arguments were supplied. After changing Svelte code, run
`pnpm desktop:build` again before restarting the native App. Rust changes are
rebuilt by Cargo automatically.

The source App uses native services and can discover actual local clients and
projects. Mutating actions still require the application's preview and
confirmation flow. To inspect the same native state without opening a window:

```bash
cargo run \
  --manifest-path packages/qiongli-native/Cargo.toml \
  --package qiongli \
  --locked \
  -- doctor
```

After a successful build, the direct development executable is at
`packages/qiongli-native/target/debug/qiongli` (or `qiongli.exe` on Windows).
It still depends on neither Node nor pnpm at runtime; those tools are only
needed to rebuild the embedded frontend.

## Validate a Desktop Change

Run the smallest relevant checks while iterating:

```bash
pnpm desktop:check
pnpm desktop:test
pnpm desktop:build
cargo test \
  --manifest-path packages/qiongli-native/Cargo.toml \
  --package qiongli \
  --locked
```

Before submitting a native change, run the complete native gates documented in
`CONTRIBUTING.md`. Before submitting documentation changes, also run
`pnpm docs:build`.

## Assemble a Local Source Package

Qiongli does not use the generic Tauri bundler as its authoritative packaging
boundary: `packages/qiongli-native/apps/qiongli/tauri.conf.json` keeps
`bundle.active` disabled. The repository's native composer binds the canonical
runtime, thin desktop launcher, update helper, embedded resource pack,
application metadata, target, and source commit into one verified archive and
receipt.

Use a clean committed checkout for a trustworthy source binding. From the
repository root on macOS or Linux:

```bash
set -euo pipefail

REPO_ROOT="$(pwd -P)"
SOURCE_COMMIT="$(git rev-parse HEAD)"
PACKAGE_PARENT="$(mktemp -d "${TMPDIR:-/tmp}/qiongli-local-package.XXXXXX")"
PACKAGE_ROOT="$PACKAGE_PARENT/artifact"
TARGET_DIR="$REPO_ROOT/packages/qiongli-native/target/release"

pnpm desktop:build
QIONGLI_NATIVE_SOURCE_COMMIT="$SOURCE_COMMIT" cargo build \
  --manifest-path packages/qiongli-native/Cargo.toml \
  --package qiongli \
  --release \
  --bins \
  --features custom-protocol \
  --locked

QIONGLI_NATIVE_SOURCE_COMMIT="$SOURCE_COMMIT" cargo run \
  --manifest-path packages/qiongli-native/Cargo.toml \
  --package qiongli \
  --example native_desktop_package \
  --release \
  --locked \
  -- \
  --canonical "$TARGET_DIR/qiongli" \
  --launcher "$TARGET_DIR/qiongli-desktop" \
  --update-helper "$TARGET_DIR/qiongli-update-helper" \
  --output "$PACKAGE_ROOT" \
  --source-commit "$SOURCE_COMMIT"

printf 'Local package: %s\n' "$PACKAGE_ROOT"
```

The output must be a new absolute path outside the checkout. The composer
rejects an existing output directory, so create only its private parent in
advance.

On Windows PowerShell, use the same composer with `.exe` inputs:

```powershell
$RepoRoot = (Get-Location).Path
$SourceCommit = (git rev-parse HEAD).Trim()
$PackageParent = Join-Path $env:TEMP ("qiongli-local-package-" + [guid]::NewGuid())
New-Item -ItemType Directory -Path $PackageParent | Out-Null
$PackageRoot = Join-Path $PackageParent "artifact"
$TargetDir = Join-Path $RepoRoot "packages\qiongli-native\target\release"
$env:QIONGLI_NATIVE_SOURCE_COMMIT = $SourceCommit

pnpm desktop:build
cargo build `
  --manifest-path packages/qiongli-native/Cargo.toml `
  --package qiongli --release --bins --features custom-protocol --locked
cargo run `
  --manifest-path packages/qiongli-native/Cargo.toml `
  --package qiongli --example native_desktop_package --release --locked -- `
  --canonical "$TargetDir\qiongli.exe" `
  --launcher "$TargetDir\qiongli-desktop.exe" `
  --update-helper "$TargetDir\qiongli-update-helper.exe" `
  --output "$PackageRoot" `
  --source-commit "$SourceCommit"

Remove-Item Env:QIONGLI_NATIVE_SOURCE_COMMIT
Write-Output "Local package: $PackageRoot"
```

The package directory contains exactly:

- the target archive;
- `qiongli-desktop-package.manifest.json`;
- `qiongli-desktop-package.receipt.json` with
  `status: assembled-unpublished`.

The native output is target-specific:

| Host | Composer output | Local launch form |
|---|---|---|
| macOS | `Qiongli-<version>-macOS-<arch>.source.zip` | Convert the accepted arm64 source package to an ad-hoc test ZIP/DMG as shown below |
| Windows | `Qiongli-<version>-Windows-x64.zip` | Extract the whole `Qiongli` directory and run `Qiongli.exe` |
| Linux | `Qiongli-<version>-Linux-x64.zip` | Extract and run `Qiongli.AppDir/AppRun`; CI performs the additional pinned AppImage conversion |

### Create a macOS ad-hoc test DMG

On a supported macOS arm64 host, turn the verified source package into a local,
ad-hoc-signed test ZIP and DMG:

```bash
PACKAGE_SHA256="$(plutil -extract package_sha256 raw \
  "$PACKAGE_ROOT/qiongli-desktop-package.receipt.json")"
SIGNED_ROOT="$PACKAGE_PARENT/ad-hoc"

tooling/scripts/macos_alpha1_sign_notarize.sh \
  --artifact-dir "$PACKAGE_ROOT" \
  --expected-source-commit "$SOURCE_COMMIT" \
  --expected-package-sha256 "$PACKAGE_SHA256" \
  --output-dir "$SIGNED_ROOT" \
  --test-only-ad-hoc
```

The resulting ZIP and DMG are non-publishing engineering evidence. Do not send
them to users or attach them to a release. `--community-alpha` and
`--production` belong to the controlled promotion/signing workflow, not normal
local development.

## What a Local Package Does Not Prove

A local source package is self-contained and does not need the checkout, Rust,
Node.js, pnpm, Cargo, Python, or another Qiongli installation to start on the
target machine. The target operating system still supplies its native WebView
and window facilities.

However, self-contained does not mean release-authorised. Without embedded
release authority and product control, integration Apply operations, client
plugin installation, and automatic updates remain unavailable. The macOS-only
`native_packaged_product_acceptance` example exercises those paths with
ephemeral keys and an isolated home; it is an automated acceptance harness,
not a distributable App.

For distribution classes, target support, platform trust prompts, signing,
notarisation, and release acceptance, continue with
[Native Desktop Alpha Packages](/advanced/native-desktop-alpha).

## Common Failures

| Symptom | Cause and action |
|---|---|
| The browser page cannot load native state | Use `?fixture=source-read-only`, or run the full Tauri App |
| Svelte changes do not appear in `cargo run` | Run `pnpm desktop:build`, then restart the App |
| A release App opens as an empty frame | Build `qiongli` with `--features custom-protocol`; `pnpm desktop:macos` already does this |
| `frontendDist` or asset files are missing | Run `pnpm install --frozen-lockfile` and `pnpm desktop:build` from the repository root |
| Rust tries to use the wrong compiler | Run Cargo through the native manifest/workspace so the pinned `rust-toolchain.toml` is selected |
| `desktop-package-source-commit-unbound` | Set the same `QIONGLI_NATIVE_SOURCE_COMMIT` for the release build and composer run |
| `desktop-package-output-invalid` | Use a new absolute output path outside the checkout under an existing private parent |
| Apply or update is unavailable | Expected for a normal source build or source package without product authority |
| Codex or Claude Code is Missing/Unavailable | Run `qiongli doctor` and distinguish client discovery from plugin source, registration, activation, and MCP attachment; they are separate states |
