# Native Desktop Alpha Packages

Qiongli 2 desktop packages are pre-release artifacts. Until the Alpha.1
readiness receipt explicitly permits publication, CI packages are
`assembled-unpublished` test evidence and must not be redistributed as signed
releases.

## Target Matrix

| Target | Desktop artifact | CLI access |
|---|---|---|
| macOS | `Qiongli.app` inside `.app.zip` | `Qiongli.app/Contents/MacOS/qiongli-cli` |
| Windows | portable application ZIP | `Qiongli/qiongli-cli.exe` |
| Linux | Type 2 `Qiongli-<version>-x86_64.AppImage` | use the companion portable CLI artifact |

The artifact filename and receipt are authoritative for architecture. Alpha.1
does not claim macOS Intel, Windows Arm64, Linux Arm64, 32-bit systems, mobile,
or browser/cloud execution unless a release publishes a separately accepted
target receipt.

## Install And Launch

On macOS, extract the archive, move `Qiongli.app` to a user-controlled
Applications directory if desired, and launch it from Finder. On Windows,
extract the entire `Qiongli` directory to a user-controlled location and launch
`Qiongli.exe`; do not separate it from `qiongli-cli.exe`. On Linux, make the
AppImage executable and launch it as one file:

```text
chmod +x Qiongli-<version>-x86_64.AppImage
./Qiongli-<version>-x86_64.AppImage
```

No package requires Rust, Python, Node.js, Cargo, npm, or pip at runtime. Linux
still depends on the operating-system facilities required by a Type 2 AppImage
and the native window stack. Clean-machine compatibility is not claimed until
the final readiness receipt records it.

## Repeatable macOS Preflight

Maintainers and clean-machine testers can verify one exact downloaded macOS
artifact without using Python, Node.js, or Rust:

```text
tooling/scripts/macos_alpha1_acceptance.sh \
  --artifact-dir <absolute-artifact-directory> \
  --expected-source-commit <exact-package-source> \
  --expected-package-sha256 <digest-from-the-trusted-run-record> \
  --output <absolute-new-receipt.json> \
  --launchservices-preflight
```

Take the expected digest and source identity from the trusted CI or release
record, not only from files beside the downloaded archive. The command verifies
the package and bundle, runs the packaged launcher with an isolated home and
empty `PATH`, and optionally exercises LaunchServices with the fixed
auto-exiting startup check. The LaunchServices result records request
acceptance only; it is not process or displayed-window observation. The
receipt contains no machine paths.

This is automated engineering evidence. A successful result does not assert
that the host is clean, that a normal window was observed, or that scale,
VoiceOver, contrast, signing, notarization, and publication gates passed.

## Maintainer macOS Signing Boundary

The signing entry point consumes the same externally bound unsigned package.
With no mode flag it performs source verification only. Native CI uses the
explicit `--test-only-ad-hoc` mode to exercise nested signing, hardened-runtime
flags, bundle verification, and signed-archive receipt generation without a
production certificate or network submission. An ad-hoc result is test
evidence only and is never distributable release trust.

Production signing is an explicit maintainer operation:

```text
QIONGLI_MACOS_SIGNING_IDENTITY=<Developer-ID-identity-in-Keychain> \
QIONGLI_MACOS_NOTARY_PROFILE=<existing-notarytool-Keychain-profile> \
QIONGLI_MACOS_EXPECTED_TEAM_ID=<10-character-Team-ID> \
tooling/scripts/macos_alpha1_sign_notarize.sh \
  --artifact-dir <absolute-artifact-directory> \
  --expected-source-commit <exact-package-source> \
  --expected-package-sha256 <trusted-unsigned-package-digest> \
  --output-dir <absolute-new-output-directory> \
  --production
```

Configure the Developer ID identity and `notarytool` profile in the macOS
Keychain before running the command. The entry point does not accept private
key paths, Apple ID passwords, API-key files, or credential-creation options.
It signs the three nested executables before the application, requires hardened
runtime plus the expected Team ID, waits for an `Accepted` notarization result,
staples and validates the ticket, and requires a successful Gatekeeper
assessment.

The unsigned manifest inside the application remains the immutable pre-signing
source descriptor; it must not be interpreted as a post-signing executable
hash manifest. The sidecar signing receipt binds that descriptor and unsigned
archive to the final signed archive and post-signing executable hashes. Even a
successful production run records `publication_allowed: false`: final-source
regeneration, clean-machine and human acceptance, release-ledger assembly, and
explicit maintainer publication authorization remain separate gates. The
script never creates a tag or GitHub Release.

Production mode emits two distinct receipts. The Alpha.1 signing-boundary
receipt records the wider publication ledger and its open gates. The strict
`*.signing.receipt.json` update receipt contains only the bounded fields
accepted by the native updater and binds the final archive plus the launcher,
canonical runtime, and update-helper post-signing hashes. Ad-hoc mode never
emits the update receipt because an ad-hoc signature is not an update trust
anchor.

The desktop launcher intentionally opens only UI mode. Use the explicit
`qiongli-cli` executable on macOS or Windows, or the companion portable CLI on
Linux, for terminal commands. The AppImage is not an arbitrary CLI argument
forwarder.

## Removal And Managed State

Before deleting the application, use Qiongli's Skills and integration lifecycle
actions to verify and remove receipt-owned content that you no longer want.
Then remove the `.app`, extracted Windows directory, or AppImage file. Deleting
only the application does not silently delete Qiongli-managed Skills, client
registrations, receipts, configuration, or unrelated user files.

## Trust Prompts

Development CI packages are unsigned and non-publishing. Do not bypass
Gatekeeper, SmartScreen, antivirus, enterprise policy, or Linux signature
checks for an artifact presented as a public release. A publishable Alpha.1
must provide matching source/artifact receipts plus maintainer-controlled
macOS signing/notarization, Windows Authenticode, or signed Linux release
metadata as applicable.

The rolling Alpha.1 source contains the macOS CLI update engine and bundled
native replacement helper. Its transition fault matrix, receipt-owned
Skills/Codex/Claude Code reconciliation, reverse compensation, and packaged
ad-hoc-signed update/rollback journey are automated. The Overview Update card
now exposes Stable/Beta selection, signed checks, non-blocking preparation
progress, cancellation, typed install confirmation, restart persistence, and
fixed path-free recovery guidance. A source or ordinary CI package without
embedded production release authority reports update as unavailable. Public
automatic-update readiness is not claimed until the final exact-head
production signed/notarized fixture journey and publication ledger pass.
There is no Marketplace bypass or Desktop/cloud plugin injection in Alpha.1;
those capabilities must not be inferred from the existence of a desktop
package.
