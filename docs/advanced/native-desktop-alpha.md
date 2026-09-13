# Native Desktop Alpha Packages

Qiongli 2 desktop packages are pre-release artifacts. Raw CI packages remain
`assembled-unpublished` test evidence and must not be redistributed. The first
planned public set uses the explicitly labelled, zero-cost `community-alpha`
distribution class; it still requires exact-head promotion, Qiongli release
signatures, target-native acceptance, and explicit publication authorization.

## Distribution Classes

Community Alpha deliberately does not claim paid operating-system publisher
trust. macOS is ad-hoc signed but not Developer ID signed or notarized, Windows
is an unsigned portable application, and Linux is an AppImage with signed
Qiongli release metadata and an optional embedded GPG signature. This lane is
for prerelease testing only and never enters Stable.

The later production lane retains Developer ID/notarization for macOS and
trusted, timestamped Authenticode for Windows. Both lanes require Qiongli's
detached Ed25519 release/update signatures, checksums, SBOM, provenance, and
truthful target receipts. The full policy is recorded in
`docs/superpowers/specs/2026-07-17-qiongli-community-alpha-distribution-note.md`.

## Target Matrix

| Target | Desktop artifact | CLI access |
|---|---|---|
| macOS | `Qiongli-<version>-macOS-arm64.dmg` plus update `.zip` | `Qiongli.app/Contents/MacOS/qiongli-cli` |
| Windows | `Qiongli-<version>-Windows-x64.zip` | `Qiongli/qiongli-cli.exe` |
| Linux | `Qiongli-<version>-Linux-x64.AppImage` | use `Qiongli-<version>-Linux-x64.zip` for portable CLI access |

The artifact filename and receipt are authoritative for architecture. Alpha.1
does not claim macOS Intel, Windows Arm64, Linux Arm64, 32-bit systems, mobile,
or browser/cloud execution unless a release publishes a separately accepted
target receipt.

## Install And Launch

On macOS, open the DMG, drag `Qiongli.app` to Applications, and launch it from
Finder. For a Community Alpha, first try the app once and then use the bounded
**System Settings > Privacy & Security > Open Anyway** control. Do not disable
Gatekeeper globally. The companion update ZIP is retained for Qiongli's atomic
self-update and rollback flow rather than normal first installation.

On Windows, extract the entire `Qiongli` directory to a user-controlled
location and launch `Qiongli.exe`; do not separate it from `qiongli-cli.exe`.
SmartScreen may offer **More info > Run anyway**, but Smart App Control,
antivirus, or enterprise policy may block an unsigned Community Alpha without
an override. Do not disable those controls or install a self-signed root for
Qiongli; use a supported test machine instead.

On Linux, make the AppImage executable and launch it as one file:

```text
chmod +x Qiongli-<version>-Linux-x64.AppImage
./Qiongli-<version>-Linux-x64.AppImage
```

No package requires Rust, Python, Node.js, Cargo, npm, or pip at runtime. Linux
still depends on the operating-system facilities required by a Type 2 AppImage
and the native window stack. Clean-machine compatibility is not claimed until
the final readiness receipt records it.

## R3Q Control Plane

The current R3Q source line organizes the App by outcome:

- **Overview** is a read-only product dashboard and recommends the next action.
- **Skills** is the advanced standalone/custom content manager. For normal
  Codex or Claude Code setup, **Integrations → Install recommended** installs
  the Qiongli plugin as one unit containing Skills and the dependency-free Lite
  MCP adapter. All mutations operate only on receipt-owned content.
- **Lite MCP** tests protocol initialize, the exact tool registry, an offline
  representative call, provider readiness, and timeout/cancellation separately
  from client attachment and registration.
- **Literature Providers** owns provider enablement, public contact fields, and
  masked OpenAlex/Semantic Scholar credentials. On macOS, raw credentials are
  stored in Keychain while native configuration contains only opaque references.
- **Integrations** reports read-only installation-metadata client versions,
  Client, Source, Skills, Registration, Activation, MCP attachment, and Overall
  independently, then offers the relevant recovery action for supported Codex
  and Claude Code targets. Discovery never launches a client runtime.
- **Global Settings** always previews the current product-wide defaults and
  previews changed or unchanged edits explicitly. **About** owns product
  identity, the Qiongli project link, and Stable/Beta Software Update controls.
- **Diagnostics** runs the native Product Doctor for content, configuration,
  secure storage, managed receipts, Codex/Claude Code, Lite MCP, literature
  providers, and update/recovery state. Exact paths remain hidden until the
  user selects **Show exact paths**; the explicit view can copy or reveal one
  inspected location. Full runtime remains labelled as deferred to R4.

The bundled CLI exposes the same inspection service:

```text
qiongli doctor                 # redacted Product Doctor JSON
qiongli paths                  # explicit human-readable exact paths
qiongli paths --json           # versioned exact-path JSON
qiongli doctor --paths exact   # Doctor plus the same exact-path snapshot
```

Path entries identify their adapter source, scope, selection, existence, type,
owner, writability, safety, and symlink/reparse resolution. Ordinary status,
logs, errors, receipts, and copied diagnostic summaries remain path-redacted.

R3Q remains the Lite control plane. Full orchestration, native agent execution,
and external worker coordination remain R4 work and are not implied by a Ready
Lite MCP or client registration state.

Before any R3Q field-test publication, perform the following on the exact App:

1. Save, replace, restart with, test, and remove one OpenAlex or Semantic
   Scholar key while confirming no raw value appears in configuration or UI.
2. Exercise managed and conflicting Codex/Claude installations, then verify,
   repair, and remove only receipt-owned state.
3. Check path labels, keyboard traversal, 100% and 200% scale, light/dark
   contrast, and VoiceOver names for every primary action.

The UI and accessibility observations are human gates. The automated packaged
receipt uses random zeroizing values to exercise Keychain save, replacement,
restart resolution, and removal without contacting a literature provider; it
never performs a post-removal missing-item lookup that could trigger an
interactive Keychain authorization prompt.

## Repeatable macOS Preflight

Maintainers and clean-machine testers can verify one exact downloaded macOS
artifact without using Python, Node.js, or Rust:

```text
tooling/scripts/macos_native_acceptance.sh \
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
flags, bundle verification, signed-archive generation, DMG creation, and mount
verification without a production certificate or network submission. An
ad-hoc CI result is test evidence only and is never distributable release
trust. The Community Alpha lane may promote a separately regenerated ad-hoc App
only after the final asset is bound to Qiongli's release authority and its
Community Alpha ledger; this still does not create Developer ID trust.

R3P-B adds a separate `--community-alpha` mode. User-facing filenames remain
short and stable; the mode and `macos-ad-hoc-not-notarized` trust state are
recorded in the bound non-publishing receipt rather than encoded in the name.
It still cannot publish by itself. R3P-C builds the signed trust bundle and a
required-reviewer `community-alpha-publication` Environment authorizes the
exact release set. That job remains read-only and has no signing key; the
maintainer signs and publishes from the local machine so GitHub never receives
the private Ed25519 key.

Production signing is an explicit maintainer operation:

```text
QIONGLI_MACOS_SIGNING_IDENTITY=<Developer-ID-identity-in-Keychain> \
QIONGLI_MACOS_NOTARY_PROFILE=<existing-notarytool-Keychain-profile> \
QIONGLI_MACOS_EXPECTED_TEAM_ID=<10-character-Team-ID> \
tooling/scripts/macos_native_sign_notarize.sh \
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
staples and validates the application ticket, and requires a successful
Gatekeeper assessment. It retains the signed update ZIP for self-update and
also creates a two-entry drag-to-Applications DMG. Production mode signs,
notarizes, staples, mounts, and Gatekeeper-assesses that DMG independently.

The unsigned manifest inside the application remains the immutable pre-signing
source descriptor; it must not be interpreted as a post-signing executable
hash manifest. The sidecar signing receipt binds that descriptor and unsigned
archive to the final signed update ZIP, first-install DMG, and post-signing
executable hashes. Even a
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

Development CI source packages are unsigned; the macOS job additionally emits
explicitly labelled ad-hoc ZIP and DMG test artifacts. Every raw CI form remains
non-publishing. The separate R3P-B workflow can freshly rebuild and aggregate a
non-publishing three-target candidate after it is merged to `2.x`;
its first run is pending. The same run pauses at the protected Environment and
then emits a short-lived exact-set authorization. A public Community Alpha is a separately promoted
release set with matching source/artifact receipts, Qiongli Ed25519 metadata,
checksums, SBOM, provenance, target-native evidence, platform-trust warnings,
and explicit authorization. It does not claim macOS notarization or Windows
Authenticode.

Use only the operating system's bounded per-app continuation when it is
available. Do not disable Gatekeeper, Smart App Control, antivirus, enterprise
policy, or Linux integrity controls. A Windows host that blocks the unsigned
binary is outside Community Alpha support. The production lane continues to
require macOS Developer ID/notarization and Windows Authenticode.

The rolling Alpha.1 source contains the macOS CLI update engine and bundled
native replacement helper. Its transition fault matrix, receipt-owned
Skills/Codex/Claude Code reconciliation, reverse compensation, and packaged
ad-hoc-signed update/rollback journey are automated. The About Update card
now exposes Stable/Beta selection, signed checks, non-blocking preparation
progress, cancellation, typed install confirmation, restart persistence, and
fixed redacted recovery guidance. A source or ordinary CI package without
embedded production release authority reports update as unavailable. Public
automatic-update readiness is not claimed until the final exact-head
production signed/notarized fixture journey and publication ledger pass.
There is no Marketplace bypass or Desktop/cloud plugin injection in Alpha.1;
those capabilities must not be inferred from the existence of a desktop
package.
