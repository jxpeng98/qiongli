# ADR 0208: Target-Specific Desktop Launcher

- Status: Accepted
- Date: 2026-07-15
- Task ID: `ARC-208`
- Owners: Qiongli maintainers
- Decision scope: Qiongli 2 desktop activation and package layout
- Supersedes in part: ADR 0201's rejection of a second thin executable for the
  target-specific desktop activation boundary; the canonical runtime decision
  remains unchanged

## Context

ADR 0201 permits a minimal operating-system launcher but requires a
superseding target-specific decision before shipping two executable files.
R3N desktop acceptance now demonstrates that the exception is required.

On Windows, one executable cannot simultaneously retain the normal console CLI
contract and use the GUI subsystem needed for double-click activation without
a persistent console window. macOS application bundles also require a named
bundle executable, while Linux AppDir and AppImage layouts use `AppRun`.
Duplicating UI or product services for these entries would create the split
runtime that ADR 0201 rejects.

## Decision

Qiongli desktop packages may include one target-native thin launcher beside
the canonical `qiongli` product executable. The launcher performs exactly four
operations:

1. resolve its own executable directory;
2. select only the documented sibling canonical executable;
3. start that executable in explicit `ui` mode; and
4. display one fixed startup failure dialog when resolution, start, or exit
   fails.

The launcher contains no product services, configuration, embedded resource
pack, installer, updater, provider, MCP, agent, orchestrator, or mutable state.
The canonical executable remains the sole Qiongli runtime and the CLI entry.

Development builds name the launcher `qiongli-desktop`. Packages expose it as
`Qiongli` inside macOS `Contents/MacOS`, `Qiongli.exe` on Windows, and `AppRun`
in a Linux AppDir. The packaged canonical executable is named `qiongli-cli`
or `qiongli-cli.exe`; this avoids a case-insensitive collision with the Windows
launcher and keeps an explicit CLI entry in every package.

The Windows launcher uses the GUI subsystem and starts only the canonical
sibling with `CREATE_NO_WINDOW`. macOS and Linux may reuse the same service-free
launcher implementation for a uniform package boundary. A desktop package
manifest binds both executable digests to the same product version, source
commit, target, resource pack, and application identity. The launcher is not a
second artifact identity or a profile authority.

## Alternatives considered

### Convert the canonical Windows executable to the GUI subsystem

Rejected because normal CLI invocation would lose the expected console
attachment and standard output behavior.

### Put desktop logic in a second full executable

Rejected because it duplicates services and embedded content, increases the
package and signing surface, and permits CLI/UI version drift.

### Use a script, shell command, Python, Node.js, or installer-generated shim

Rejected because it violates dependency-free startup and introduces quoting,
path, policy, and host-runtime variation.

### Keep desktop activation as `qiongli ui` from a terminal

Rejected because it does not provide a normal double-clickable application.

## Consequences

- Desktop packages contain two native executable files but only one product
  runtime and one embedded content pack.
- Both executable digests must be inspected, signed where the target requires
  it, and included in package provenance and rollback evidence.
- The launcher can remain small and stable while UI, CLI, MCP, agents, and
  orchestration continue to evolve in the canonical executable.
- Startup errors are intentionally bounded; detailed diagnostics remain in the
  canonical product and managed receipts.

## Security and privacy

- Resolution never searches `PATH`, a registry, environment-provided command,
  package manager, or language runtime.
- Package assembly accepts only bounded regular native binaries for the exact
  target, records both hashes, and rejects symlink or archive-layout drift.
- The launcher passes only the fixed `ui` argument and does not forward model,
  config, credential, path, or network input.
- Failure messages use fixed public codes and never render filesystem paths,
  environment values, or child-process output.
- Signing keys and platform trust decisions remain outside the launcher and
  repository.

## Rollback

A package may remove the launcher and retain direct canonical CLI operation.
Rollback restores both launcher and canonical binary from one verified prior
package identity; it never mixes either executable across versions. If a
target later supports a single executable without breaking CLI behavior, a
new ADR may retire the launcher after packaged activation evidence passes.

## Acceptance evidence

1. Static dependency and size checks show that the launcher carries no product
   service crate or embedded resource pack.
2. Tests prove sibling-only resolution, fixed `ui` invocation, repeated
   activation, and fixed path-free failures.
3. Windows package tests prove GUI-subsystem activation and no persistent
   console; macOS and Linux test their normal bundle entries.
4. Package verification rejects missing, added, renamed, reordered, wrong-mode,
   wrong-target, or digest-mismatched launcher and canonical bytes.
5. Clean-machine startup succeeds without Rust, Python, Node.js, Cargo, npm,
   pip, a shell shim, or a source checkout.
