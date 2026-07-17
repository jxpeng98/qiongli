# Qiongli R3L Client Activation And Desktop Intent Design

Status: frozen for implementation

Date: July 14, 2026

Scope: one target-at-a-time local activation coordination, desktop typed
confirmation, and packaged desktop startup preflight

Branch: `feat/2x-native-alpha1`

Rolling PR: `#63`

## Goal

Compose the accepted Codex and Claude Code registration adapters behind one
typed, rollback-aware service and let the native desktop manager invoke that
same service through an exact preview-and-confirm boundary.

R3L is deliberately an adapter batch. It does not merge the R3K portable native
payload format with the R3D/R3E plugin-bundle format. A caller must already hold
one verified target-specific plugin source and one verified launch grant for
that plugin bundle. R3M will assemble those inputs into the release candidate
and run the clean-machine journey.

## Unified Activation Service

`qiongli-platform` adds a closed `ClientActivationTarget` vocabulary containing
only `codex` and `claude-code`. Discovery accepts an already resolved user home,
an optional already resolved Claude config root, and one target selected at a
trusted CLI, UI, installer, release, or test boundary.

The returned handle contains the existing target-specific capability but never
renders the user path. Preview delegates to the accepted R3C or R3E planner,
then re-verifies the complete plan and signed grant against:

- the exact trusted public-key set and minimum generation supplied by the
  release boundary;
- the target-specific integration scope;
- Lite MCP mode;
- the grant-bound plugin-bundle artifact, binary, and resource pack; and
- the requested plan creation and expiry times.

The service exposes one redacted preview containing the target, effect, exact
semantic plan digest, three required approvals, and outstanding host action.
It accepts no arbitrary path, command, mode, profile, scope, key, or approval
from model or MCP input.

## Lifecycle Coordination

One coordinator instance owns exactly one discovered target. It delegates:

- apply and identical replay;
- read-only verify;
- receipt-backed repair;
- remove and terminal replay; and
- rollback and terminal replay.

All mutation requires an `ApprovedInstallPlan` carrying exactly
`filesystem-write`, `client-config-change`, and `host-trust`. The coordinator
rejects a preview created for another target. After apply or repair it performs
an immediate adapter verification; if that verification unexpectedly fails, it
attempts the accepted adapter rollback and returns a fixed failure code.

The coordinator does not activate both clients atomically. Each plan targets
one client, so a failure can never silently roll back or remove the other
client's state. Codex and Claude continue to own their caches, enablement,
installed-plugin registries, and runtime trust prompts.

## Desktop Typed Intent

`qiongli-ui` remains service-only. It gains no platform, filesystem, process,
network, config, or content dependency. An integration preview displays:

- the fixed target and summary;
- the exact lowercase plan digest;
- the three approval labels; and
- whether confirmation is enabled.

The application-layer desktop service may receive a prepared activation session
from a trusted release/installer boundary. Preview creates the verified plan
and stores it behind one unforgeable operation token. Confirming that token
creates the exact approval, calls the coordinator, clears the pending operation,
and refreshes the redacted snapshot. Cancelling clears it without mutation.
Wrong, stale, or cross-target tokens fail closed.

Ordinary source builds receive no activation session. They continue to show a
truthful blocked preview and `apply: false`; the UI cannot manufacture a grant,
source package, path capability, or approval.

## Packaged Desktop Startup Preflight

The canonical binary adds:

```text
qiongli ui --startup-check
```

This is a bounded, non-mutating, no-window diagnostic. It constructs the same
embedded content, application desktop service, validated snapshot, and UI app
state used by `qiongli ui`, then emits versioned JSON and exits. It accepts no
path or environment override and starts no subprocess.

A copied current-target artifact binary must pass the startup check outside the
checkout with an empty `PATH`. The normal `qiongli ui` command remains the
actual eframe window entrypoint. R3L does not automate an undocumented GUI or
claim clean-machine display-server, screen-reader, signing, or installer
acceptance; those remain R3M evidence.

## Output And Failure Contract

Coordinator and desktop failures use fixed reason codes. Public summaries and
Debug output contain no home, Claude config, plugin source, marketplace,
environment, key, signature, private input, or rejected argument bytes.

The startup-check JSON contains only schema version, command, product version,
target OS/architecture, snapshot status, and window-entrypoint availability.
It does not imply that an OS window was displayed.

## Acceptance

- Codex and Claude preview, exact approval, apply/replay, verify, remove, and
  rollback use the same coordinator API;
- wrong scope, generation, target, approval, token, drift, conflict, and
  recovery states fail closed with path-free errors;
- desktop previews show the exact digest and all approvals, cancellation is
  non-mutating, confirmation refreshes the snapshot, and source builds remain
  blocked;
- qiongli-ui retains its dependency boundary and AccessKit tests pass at the
  supported sizes and scales;
- a copied current-target binary passes `ui --startup-check` with an empty
  runtime `PATH`; and
- local Native, focused Lite, Windows MSVC, frozen-boundary, exact-head Native
  CI, and Cloudflare gates pass.

## Explicit Non-Claims

R3L does not create or discover managed payload roots, compose or download
plugin sources, convert a portable-archive grant into a plugin-bundle grant,
select production keys, handle private signing material, write Codex or Claude
caches/settings/enablement, invoke client CLIs, support Claude Desktop or cloud
surfaces, atomically activate both clients, display a clean-machine GUI, publish
an artifact or Marketplace entry, provide an updater, sign/notarize packages,
produce checksum/SBOM/provenance outputs, or publish `v2.0.0-alpha.1`.
