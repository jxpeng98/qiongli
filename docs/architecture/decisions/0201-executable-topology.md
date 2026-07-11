# ADR 0201: Canonical Native Executable Topology

- Status: Accepted
- Date: 2026-07-11
- Task ID: `ARC-201A`
- Owners: Qiongli maintainers
- Decision scope: Qiongli 2 product entry points, process modes, and packaging

## Context

Qiongli 2 must expose a full CLI, desktop application, Lite and Full MCP
profiles, agents, orchestration, installation, diagnostics, update, and
recovery without requiring a user-installed Rust, Python, or Node.js runtime.
These surfaces must use one implementation of contracts, state, providers,
policy, and rollback.

Separate CLI, desktop, and MCP executables can still share crates, but they
create independent version, signing, target, startup, update, and host-plugin
identities. They also make it possible to install a compatible UI beside an
incompatible MCP server. A mandatory background service would add another
lifecycle and recovery boundary before the first native vertical slice.

## Decision drivers

- one product version and artifact identity on each OS/architecture target;
- one service implementation for CLI, GUI, MCP, agent, and installer callers;
- no language-runtime bootstrap or external CLI requirement at startup;
- truthful target-specific plugin and desktop packaging;
- bounded process isolation for dangerous tools without a second product;
- simple clean-machine install, doctor, update, rollback, and support evidence.

## Decision

Qiongli 2 ships one canonical native executable per target, named `qiongli`
(`qiongli.exe` on Windows). The Rust workspace has one product application at
`packages/qiongli-native/apps/qiongli/`; domain behavior lives in service
crates and is not implemented in the application dispatcher.

The executable provides typed modes:

- normal CLI commands, including install, doctor, config, skills, agents,
  orchestration, update, and recovery;
- `qiongli ui` for the desktop application;
- `qiongli mcp serve --profile lite|full` for native MCP stdio or an explicitly
  enabled transport;
- public orchestration commands that call the same agent and domain services;
- reserved internal worker and ToolHost modes used only through authenticated,
  parent-created IPC.

The platform desktop entry invokes the same product bytes in UI mode. A CLI
link, shim, or application-bundle entry may point to those bytes. An operating
system may require a minimal launcher for registration or bundle conventions;
such a launcher contains no product services, contracts, mutable state,
provider logic, updater, or embedded content and is not a second Qiongli
runtime. Its target and digest remain bound to the same installer identity.

Local plugin packages that require a process carry the same target-specific
binary and a signed launch grant. The grant binds product version, target,
binary digest, resource-pack digest, integration scope, and the maximum allowed
profile. Runtime policy calculates the effective profile as the least
privileged of the binary capability ceiling, signed grant, and requested
profile. Arguments may lower that profile but cannot raise it. A Lite plugin
therefore cannot expose Full tools even though it reuses the canonical binary.
It does not contain a separately implemented Lite server. Content-only plugins
contain no binary or launch grant and must be identified as content-only.

Each installation transaction has an ID and receipt. Runtime handshakes report
that receipt plus product, target, binary, pack, grant, and effective-profile
identities. CLI, UI, MCP, and updater compatibility claims apply within one
managed installation; Qiongli does not assume a host-managed cache copy is the
same instance. Doctor enumerates documented registrations, identifies stale or
split-version copies, and gives a host-supported repair action. The updater
mutates only the instance and registrations owned by its receipt.

The alpha does not require a persistent daemon. Long-running work remains in
the foreground owner process or an explicitly spawned child whose lifecycle,
IPC identity, cancellation, and recovery receipt are visible. A future daemon
requires a new ADR covering authentication, multi-user isolation, upgrade,
crash recovery, and service management.

Every mode reports the same product version, source revision, resource-pack
identity, target identity, and release channel. Public modes are stable CLI
contracts. Reserved internal modes are not user APIs, but their inputs remain
strictly parsed and authenticated because obscurity is not a security boundary.

## Alternatives considered

### Separate full CLI, desktop, and MCP binaries

This creates familiar entry points, but multiplies artifact identity, update,
installation, and compatibility states. Shared crates reduce code duplication
but do not prevent mismatched installed binaries. Rejected for the product
topology.

### Two thin executables for CLI and desktop

This is workable and remains possible if an OS later imposes a hard packaging
constraint, but it still creates two signed launch artifacts and does not help
the first vertical slice. Rejected unless a superseding target-specific ADR
demonstrates the requirement.

### Mandatory local daemon with thin clients

A daemon can centralize state and long-running tasks, but adds service install,
authentication, port/IPC ownership, upgrade ordering, and recovery work. It is
not required for alpha.1 and is rejected as the default topology.

### Continue shipping the existing Python Full and Rust Lite executables

This preserves short-term behavior but violates the zero-language-runtime and
single-platform goals, keeps two implementations, and cannot become the 2.x
product identity. The accepted 1.x line remains an oracle and rollback target,
not a 2.x frontend.

## Consequences

Positive consequences:

- one canonical target binary component identifies the shared product runtime,
  while signed package manifests and grants identify each exposed profile;
- CLI, desktop, and MCP cannot silently drift to different service versions;
- installer, updater, doctor, SBOM, provenance, and rollback evidence have one
  primary executable per target;
- self-spawned worker modes can add bounded isolation while preserving one
  shipped product.

Costs and limitations:

- the dispatcher and feature graph must keep headless MCP startup from
  initializing the renderer or desktop-only services;
- the full product binary may be larger than a single-purpose Lite binary;
- an internal child-process protocol needs versioning, authentication, limits,
  and cancellation tests;
- an OS-required launcher is an additional signed file even though it is not a
  second product runtime.

## Security and privacy

- Mode selection is a closed enum. Unknown, malformed, or conflicting modes
  fail before config, network, renderer, or project initialization.
- Internal worker and ToolHost modes require a parent-created, short-lived,
  authenticated IPC capability and reject ordinary interactive invocation.
- MCP profile policy is enforced by the service layer, not by a filename,
  symlink name, environment variable, or hidden argument alone. Release MCP
  startup requires a valid artifact-bound launch grant.
- Headless modes do not initialize the UI, read UI persistence, or broaden
  filesystem/network access.
- Process arguments and diagnostics never carry secret values; secret
  references are resolved through the config boundary after policy checks.
- The executable never locates Python, Node.js, Cargo, or an external agent CLI
  as a required fallback. Optional adapters are explicit capabilities.

## Rollback

The updater keeps the prior verified target binary and resource pack as the
last-known-good slot. Failure in any mode restores the complete prior product
identity rather than mixing old and new frontends. Host registrations can be
transactionally repointed to the accepted 1.x artifact during the migration
window without modifying 1.x state.

If a target proves that one executable cannot satisfy its signed packaging or
accessibility requirements, retain the service crates and introduce the
smallest target-only launcher or frontend through a superseding ADR. Do not
fork contracts, providers, state, installer behavior, or MCP policy.

## Acceptance tests

1. Each native product payload contains exactly one canonical Qiongli product
   executable; any OS launcher is classified and proven service-free.
2. CLI, UI, Lite MCP, Full MCP, orchestrator, doctor, installer, and internal
   child modes report the same version, channel, target, source, and resource
   identities.
3. Clean-machine process-tree tests start every advertised public mode without
   Python, Node.js, Cargo, a development server, or an external agent CLI.
4. Headless startup tests prove MCP and CLI do not initialize the renderer or
   desktop persistence and stay within their declared profile capabilities.
5. Invalid mode, conflicting mode, direct internal-mode, forged IPC, expired
   capability, and parent-loss fixtures fail closed with stable redacted errors.
6. Cancellation and crash tests terminate owned child processes, preserve the
   transaction/audit receipt, and leave no orphan listener or mutable lock.
7. Installer and update tests cannot create mismatched CLI, UI, and MCP product
   identities and restore the complete previous identity on failure.
8. Target-specific plugin tests invoke the canonical binary with an allowlisted
   profile and reject an artifact for the wrong OS or architecture.
9. Grant tests reject missing, forged, wrong-digest, wrong-target, wrong-scope,
   and profile-escalating launches; arguments can only reduce effective scope.
10. Multi-instance tests detect a stale host cache or registration through the
    runtime handshake, keep updater ownership receipt-scoped, and never report
    a split installation as compatible.

## Follow-up tasks

- `FND-201`: scaffold one product app and the shared service-crate graph.
- `MCP-201`: expose Lite and Full MCP as profile-constrained product modes.
- `UI-201`: attach the desktop entry to UI mode without duplicating services.
- `AGT-203`: implement the authenticated internal ToolHost process boundary.
- `PKG-201`: package, identify, and start the canonical binary on Tier 1 targets.
- `PLT-201`: carry signed launch grants and installation identity in plans and
  receipts.
- `QAT-201`: audit installed payloads and process trees for forbidden runtimes.
