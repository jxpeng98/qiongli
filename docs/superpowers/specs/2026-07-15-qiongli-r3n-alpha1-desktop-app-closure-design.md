# Qiongli R3N Alpha.1 Desktop Application Closure Design

Status: implementation in progress; macOS arm64 is the Alpha.1 publication target and R3O owns unified update closure

Date: July 15, 2026

Branch: `feat/2x-native-alpha1`

Rolling PR: `#63`

## Outcome

R3N converts the existing R3F/R3M native window from a read-oriented prototype
into the minimum useful Qiongli Lite Alpha.1 desktop application. It responds
to five observed acceptance failures:

1. Overview cannot edit the supported global configuration.
2. Skills cannot select and use a materialization destination.
3. MCP cannot run a health check from the window.
4. Ordinary source sessions do not discover local Codex and Claude Code
   integrations usefully.
5. The UI is a CLI mode, not a double-clickable desktop application.

These are Alpha.1 release blockers. R3M's signed candidate and lifecycle work
remain valid release-core evidence, but Alpha.1 is not ready until R3N passes.

## Product Boundary

R3N reuses the existing Rust application, UI crate, config store, embedded
content, Lite runtime, platform adapters, and signed-candidate authority. It
does not introduce a web frontend, JavaScript runtime, Python helper, Node
helper, second settings format, or second installer implementation.

The desktop package and CLI are two entry surfaces for the same native product:

```text
desktop activation -> native application entry -> qiongli-ui -> typed services
qiongli ui          -> native application entry -> qiongli-ui -> typed services
qiongli <command>   -> command entry            -> the same domain services
```

ADR 0208 records the target-specific launcher exception without modifying the
frozen ADR 0201 baseline.

A platform may require a thin launcher executable or bundle metadata. Such a
launcher may select desktop mode and report startup failure, but it may not own
UI state, configuration logic, installation logic, or content.

## Global Settings From Overview

Overview gains an `Edit global settings` action and a service-backed editor.
The first Alpha.1 editor supports only fields already represented by the
versioned global settings document:

- default profile;
- provider enabled/disabled state;
- public provider contact email where the provider contract supports it.

Secret values and secret references remain read-only readiness in R3N; adding a
new cross-platform credential store would expand this closure beyond the five
observed failures. The UI never shows a stored secret and never writes the
settings file itself.
Load returns a redacted editable model and revision. Save sends the expected
revision plus a typed patch to the config service. The service validates,
writes atomically, rereads, and returns the new redacted revision. A stale
revision, invalid field, unavailable secret store, insecure permissions, or
recovery-required state produces a fixed remediation result without a partial
write.

The Overview card must show whether settings are absent, ready, invalid,
insecure, busy, or recovery-required. Restarting the application must show the
persisted supported values.

## Skills Destination And Lifecycle

The Skills view gains a native folder picker and four bounded actions:

- select destination;
- preview materialization;
- materialize the selected embedded profile; and
- verify or remove a receipt-owned materialization.

The path originates only from an explicit human folder-picker action. It is
normalized and validated by the existing content/platform services; it is not
accepted from MCP input, model output, plugin metadata, clipboard automation,
or an untrusted config document. Preview shows a display-safe destination and
operation digest. Apply requires the unchanged digest and explicit filesystem
approval.

Materialization retains the existing traversal, symlink, ownership, atomic
write, and unrelated-file protections. Remove deletes only files bound by a
verified Qiongli receipt. A destination with ambiguous ownership or unrelated
conflicts fails closed and remains diagnosable.

## Lite MCP Self-Test

The MCP view gains `Run Lite MCP self-test`. The operation uses the same native
Lite registry and dispatcher as `qiongli mcp serve`; it does not start Python,
Node, Cargo, a shell, or an unbounded background server.

The bounded check reports:

1. embedded contract and profile availability;
2. MCP initialize compatibility;
3. exact public `tools/list` registry integrity;
4. provider readiness from redacted configuration;
5. one offline, non-mutating dispatch probe where available; and
6. client registration state for discovered local integrations.

Network calls and mutating Zotero operations are excluded from the default
self-test. The UI runs the check off the render loop with cancellation and a
fixed timeout. Results contain fixed codes and remediation, not raw paths,
environment values, request headers, credentials, or provider response bodies.

## Source-Session Integration Discovery

Discovery and installation authority are separate capabilities. An ordinary
source/development session must be able to read and report supported local
Codex and Claude Code state even though it cannot apply a release-candidate
installation.

Refresh evaluates, for each client:

- executable/application presence when detectable without launching it;
- supported config-root presence;
- Qiongli source/package state;
- registration state;
- managed receipt state; and
- a fixed reason when discovery is unavailable or ambiguous.

Discovery is read-only and does not require a signed candidate. Preview/apply
continues to require the accepted signed-candidate session, fixed install
plans, explicit approvals, and host-owned follow-up actions. The UI labels
`client not discovered`, `discovered but unmanaged`, `managed`, and
`candidate required for install` as distinct states.

Tests use isolated roots and real adapter logic. Publication additionally
requires a manual or automated receipt from supported real Codex and Claude
Code installations; a test double cannot be described as real-client
evidence.

## Application Packaging And Alpha.1 Scope

The common packaging work continues to produce non-publishing engineering
artifacts for each supported operating-system family. ADR 0209 narrows the
first published Alpha.1 to macOS arm64:

| Platform | Current artifact | Alpha.1 publication status |
|---|---|---|
| macOS arm64 | `.app` in a signed/notarized archive | Required; Finder activation opens the UI without Terminal |
| Windows | desktop GUI executable in a portable package | Deferred; CI artifact remains non-publishing |
| Linux | AppImage with desktop metadata and icon | Deferred; CI artifact remains non-publishing |

The packages include the canonical native executable, embedded resource pack,
icons/metadata, licenses, and only the native runtime libraries required by the
chosen target. The installed user needs no Rust, Python, Node.js, Cargo, npm,
or pip. Build and packaging jobs may use pinned build tools.

The CLI remains available. Invoking the CLI binary with explicit command
arguments preserves current command behavior. Desktop activation selects UI
mode through bundle metadata, a no-argument desktop entry, or a thin native
launcher appropriate to the platform. There is still one UI implementation
and one service composition.

Desktop assets receive their own digest-bound artifact descriptor linked to
the same product version, source commit, target, executable digest, and
embedded-pack digest as the R3M portable candidate. R3N must not silently add
unsigned bytes to the previously accepted three-file candidate contract.
After R3N, the release candidate and release notes are regenerated from the
final exact head.

Normal macOS Gatekeeper claims require maintainer-controlled Developer ID
signing and notarization evidence. Development and deferred-platform packages
must be labelled non-publishing. Private keys remain outside the repository,
product, logs, artifacts, and runtime environment output.

R3O adds the complete application updater as an Alpha.1 blocker. It replaces
the 1.x split software/content update with stable and beta update streams,
staged macOS application replacement, rollback, and receipt-owned content
reconciliation. Its detailed contract is recorded separately so update
authority does not leak into UI or packaging code.

## UI And Service Intents

R3N extends the UI boundary with a small set of typed operations:

- `LoadGlobalSettings` and `PreviewGlobalSettingsPatch`;
- `ApplyGlobalSettingsPatch` with expected revision and approval token;
- `SelectSkillsDestination`, `PreviewSkillsMaterialization`,
  `ApplySkillsMaterialization`, `VerifySkillsMaterialization`, and
  `RemoveSkillsMaterialization`;
- `RunLiteMcpSelfTest` and `CancelLiteMcpSelfTest`; and
- `RefreshIntegrationDiscovery`.

The UI may retain only display-safe models, opaque service tokens, and
transient zeroized secret input. Filesystem, environment, process, provider,
secret-store, client, signing, and installer access remain outside the UI
crate.

## Acceptance

R3N is complete only when all of the following pass on the exact rolling-PR
head:

- global settings can be read, edited, saved atomically, refreshed, and
  observed after restart, including stale-revision and recovery failures;
- a user-selected Skills destination can complete preview, apply, verify, and
  receipt-owned remove without touching unrelated files;
- the MCP self-test reports success and separately exercises timeout,
  cancellation, provider-unready, and contract-failure states;
- source and packaged sessions discover real supported Codex and Claude Code
  installations without gaining unauthorized apply authority;
- the macOS arm64 application launches through normal desktop activation on a
  clean machine without language runtimes;
- the packaged window passes navigation, keyboard, scale, basic screen-reader,
  restart-persistence, and startup-failure checks;
- the stable/beta unified updater accepts only 2.x-or-newer complete
  applications and restores the old application plus managed content on
  failure;
- every published artifact is identity/digest bound and publication claims match the
  signing and real-client evidence actually recorded; and
- the regenerated Alpha.1 candidate, notes, native CI, and publication ledger
  refer to the same final source and artifact set.

## Non-Goals

- Full MCP or a long-running MCP process supervisor in the UI;
- agents, ToolHost, direct model backends, or the orchestrator;
- Windows and Linux interactive acceptance, signing, updater execution, and
  publication for Alpha.1;
- Qiongli 1.x import, migration, compatibility reads, cleanup, or rollback;
- new credential-store backends or secret-reference editing;
- Claude Desktop, Codex Desktop/ChatGPT Marketplace bypass, cloud/web
  execution, or public Marketplace distribution;
- a second frontend or platform-specific business logic; or
- complete production packages for every CPU architecture in Alpha.1.
