# ADR 0218: CLI-First Delivery And Local Host Collaboration

- Status: Accepted
- Date: 2026-09-06
- Task ID: `ARC-218`
- Owners: Qiongli maintainers
- Decision scope: product delivery, execution default, and local collaboration
- Supersedes: ADR 0217's App-owned ACP product default; the mandatory App
  delivery/launcher dependency in ADRs 0201, 0208, 0209 and 0213 for the new
  standalone CLI lane. Their existing desktop and published-package contracts
  remain applicable until an independently qualified replacement exists.

## Context

The maintainer has directed the project to close the App/ACP development stage,
integrate its completed source into `2.x`, and prioritize CLI-first delivery.
The September 5 local CLI proposal describes the desired outcome; its static
plan validation is not product acceptance. The existing main package still
combines native services, Tauri build dependencies, and window launch behavior.

## Decision

The primary delivery target is a Rust-native `qiongli` CLI and Lite/Full MCP
package. Installation, configuration, research work, approval, export and recovery
must work without opening or installing the Qiongli App. Consumers need no
source checkout, Cargo, Python or Node runtime for Qiongli itself. The selected
Host's dependencies and authentication remain separately declared.

Users run their chosen Hosts. Hosts own models, accounts and conversations;
Qiongli owns research state, sources, task packets, candidates, checkpoints,
tools and receipts. Ordinary work does not launch model workers or copy Host
conversations. Direct provider APIs and silent transport fallback remain excluded.

Deliver in this order: CLI execution/build separation; independent trusted
resources and installation; one Host's controlled research journey and same-device
handoff; then two real local Host sessions with task claims, candidate review,
conflict handling and recovery. Additional Hosts qualify individually. A CLI
baseline release need not wait for multi-Agent collaboration. Cross-device
synchronization follows local collaboration and requires a separate decision.

Reuse `ProjectStateService`, existing execution/handoff/checkpoint types,
ToolHost, Full MCP, content packs and install transactions. Separate window and
packaging code from shared services; do not delete modules based on their names.
A narrow desktop feature or separate target may perform the first split. CLI
normal, build and selected test dependencies must exclude the graphical stack.
The optional desktop lane retains Tauri/Svelte under ADR 0210.

All Chat/ACP source, schemas and historical observations are retained as deferred
development work. App expansion, embedded chat, React/Electron migration and a
general Agent daemon are outside the current horizon. This decision neither
accepts those journeys nor removes existing GUI support or user data.

## Alternatives considered

Continuing the App ACP journey would preserve its current integration path but
would delay the requested window-free product. Rewriting in Node/Python or
building a shared cross-repository framework would duplicate working native
owners. Deleting the GUI immediately would break mixed service and installation
boundaries before their replacements exist. Each is rejected for this horizon.

## Consequences

Completed source can be integrated without continuing investment in App features.
The extraction must prove actual selected dependencies and behavior, rather than
equating a CLI subcommand or a disabled default feature with GUI independence.
The main package remains mixed until that work passes; standalone delivery is
an approved target, not a claim about the current artifact.

Existing accepted ledger rows, frozen ADRs, release evidence and publication
decisions retain their exact historical scope. New package identity, installation
trust, compatibility and rollback need fresh source/target/byte evidence.

## Security and privacy

Keep verified resource locks, package identity and release trust. Source-built
inspection restrictions remain until the independent delivery verifier is
implemented; an ordinary config flag cannot establish package authority.

Project writes still require existing preview, approval, revision/digest checks,
CAS and recoverable transactions. A role, natural-language approval, tool auto-allow,
TTY or public digest alone is not a verified human approval channel. MCP stdout
remains protocol-only. Persisted candidates do not restore active approval tokens.

Local collaboration uses atomic claims, separate coordination generations and
research revisions, exact candidate-digest review, and short commit transactions.
Model execution holds no global project lock. Separate MCP processes do not share
in-memory authority; add a scoped native coordinator only if existing storage and
approval owners require it. Same-UID arbitrary shell access is outside the strong
isolation claim; VM/WSL/container environments require separate qualification.

## Rollback

Revert an extraction slice while retaining the explicit desktop build lane and
existing package trust rules. Never rewrite project files, Host credentials,
published tags or historical chat schemas as part of a source rollback. Reopening
App-owned execution as the product default requires a new superseding decision.

## Acceptance tests

1. The selected CLI builds and tests without Tauri/rfd, frontend output or GUI
   development libraries; no-argument/help/JSON/MCP paths open no window.
2. A verified standalone package reads its content and installs/repairs/removes
   a declared Host integration from an unrelated working directory. Tampering,
   path substitution and unmanaged config replacement fail closed.
3. One real Host completes a source-bound candidate, trusted human approval,
   commit, restart and same-device handoff without the Qiongli App. Stale or
   replayed approval cannot write.
4. Two actual local Host sessions claim work and review an exact candidate;
   competing claims, changed sources, duplicate commits, late cancelled results
   and crash recovery preserve existing project transaction guarantees.
5. CLI baseline and local-collaboration releases each qualify their own exact
   source, package, platform and Host claims under separate release authority.
