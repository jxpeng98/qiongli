# Qiongli Architecture Decision Log

This directory is the reviewed source of architecture decisions for the
Rust-native Qiongli 2 line. `tooling/architecture/arc-201-decisions.json` is the
machine-readable inventory, and `python scripts/validate_arc_201_adrs.py` checks
that every accepted ARC-201 decision retains its required context, decision,
alternatives, consequences, security, rollback, and acceptance evidence.
After the initial B0 merge, `scripts/check_frozen_2x_architecture_baseline.py`
rejects byte changes to ADR 0201-0207 and their accepted inventory. A changed
decision must be recorded as a new superseding ADR.

| Task | ADR | Status | Decision |
|---|---|---|---|
| `ARC-201A` | [ADR 0201](0201-executable-topology.md) | Accepted | Executable topology |
| `ARC-201B` | [ADR 0202](0202-rust-native-ui-and-accessibility.md) | Accepted | Rust-native UI and accessibility |
| `ARC-201C` | [ADR 0203](0203-agent-backend-and-tool-host.md) | Accepted | Agent backend and native tool-host boundary |
| `ARC-201D` | [ADR 0204](0204-versioned-state-and-secret-storage.md) | Accepted | Versioned state and secret storage |
| `ARC-201E` | [ADR 0205](0205-deterministic-resource-pack.md) | Accepted | Deterministic embedded resource pack |
| `ARC-201F` | [ADR 0206](0206-declarative-install-plan-and-client-trust.md) | Accepted | Declarative install plan and client trust |
| `ARC-201G` | [ADR 0207](0207-release-channel-and-artifact-identity.md) | Accepted | Release channel and artifact identity |
| `ARC-208` | [ADR 0208](0208-target-specific-desktop-launcher.md) | Accepted | Target-specific desktop launcher |
| `ARC-209` | [ADR 0209](0209-macos-unified-update-and-v2-only-boundary.md) | Accepted | macOS unified update and Qiongli 2-only boundary |
| `ARC-210` | [ADR 0210](0210-tauri-svelte-desktop-presentation.md) | Accepted | Tauri and Svelte desktop presentation; supersedes ADR 0202 presentation choice |
| `ARC-211` | [ADR 0211](0211-host-driven-model-execution.md) | Accepted | Host-driven model execution; supersedes ADR 0203 direct-provider default |
| `ARC-212` | [ADR 0212](0212-qiongli-1x-replacement-migration.md) | Accepted | One-way Qiongli 1.x replacement migration and verified 2.x cutover |
| `ARC-213` | [ADR 0213](0213-app-mediated-official-host-plugin-activation.md) | Accepted | One approved App preview may run fixed official Host Plugin commands; fresh observation owns Ready |

## Decision lifecycle

1. New architectural choices receive the next ADR number and a task ID.
2. A proposed ADR must name an owner and include measurable acceptance tests.
3. Accepted ADRs are immutable in meaning. Material changes require a new ADR
   that names the decision it supersedes.
4. Implementation pull requests link the ADR and provide its acceptance
   evidence; an ADR is not evidence that the implementation already passes.
5. Generated payloads, host caches, and external marketplace catalogs remain
   outside this decision-log source boundary.
