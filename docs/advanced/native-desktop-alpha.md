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
auto-exiting startup check. It writes no machine paths to the receipt.

This is automated engineering evidence. A successful result does not assert
that the host is clean, that a normal window was observed, or that scale,
VoiceOver, contrast, signing, notarization, and publication gates passed.

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

There is no installer, automatic updater, managed upgrade, Marketplace bypass,
or Desktop/cloud plugin injection in Alpha.1. Those capabilities must not be
inferred from the existence of a desktop package.
