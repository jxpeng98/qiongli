# Qiongli Architecture Decision Log

This directory is the reviewed source of architecture decisions for the
Rust-native Qiongli 2 line. `tooling/architecture/arc-201-decisions.json` is the
frozen ARC-201 bootstrap inventory, while
`tooling/architecture/current-decisions.json` is the complete current registry.
`python scripts/validate_arc_201_adrs.py` checks both records and their ADR
metadata. Every accepted ARC-201 decision also retains its required context,
decision, alternatives, consequences, security, rollback, and acceptance
evidence.
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
| `ARC-214` | [ADR 0214](0214-receipt-owned-local-workflow-variants.md) | Accepted | Editable Workflow/Skill Markdown remains derived, receipt-owned, explicitly reconciled, and exact at Ready |
| `PKG-202C` | [ADR 0215](0215-community-alpha-distribution-boundary.md) | Accepted | Community Alpha distribution remains separate from platform-trusted production distribution |
| `GOV-408` | [ADR 0216](0216-rust-owned-public-schema-authority.md) | Accepted | Rust owns changed public schemas; generated contracts and explicit compatibility classes govern consumers |
| `ARC-217` | [ADR 0217](0217-app-owned-acp-and-all-chat-state.md) | Accepted | App-owned ACP v1 sessions and Qiongli-owned All Chat State supersede the external-Host-only default |
| `ARC-218` | [ADR 0218](0218-cli-first-local-host-collaboration.md) | Accepted | CLI-first delivery and same-device Host collaboration supersede the App ACP default; retained GUI source and historical evidence remain |
| `CLI-410` | [ADR 0219](0219-cli-github-release-distribution.md) | Accepted | Standalone CLI Alpha GitHub assets are independent of App/Community Alpha promotion; managed-product trust remains unchanged |
| `CLI-410` | [ADR 0220](0220-three-platform-cli-registry-distribution.md) | Accepted | Three-platform native CLI installation with npm next and PEP 440 PyPI versions; Cargo deferred |
| `CLI-410` | [ADR 0221](0221-cargo-cli-publication.md) | Accepted | Cargo source publication and three-platform install checks; supersedes Cargo deferral |
| `CLI-403` / `CLI-405` | [ADR 0222](0222-user-approved-local-plugin-sources.md) | Accepted | User-approved local Plugin source lifecycle; signed product and Host authority remain separate |
| `CLI-410` | [ADR 0223](0223-bundled-marketplace-native-mcp.md) | Accepted | Platform-specific Marketplace Plugins bundle the native executable; no Node/npm startup dependency |
| `CLI-402` | [ADR 0224](0224-cli-local-plugin-registration.md) | Accepted | CLI exports bundled content and registers local Plugins through separately approved official Host commands |
| `CLI-402` | [ADR 0226](0226-confirmed-codex-plugin-source-migration.md) | Accepted | Confirmed Codex source migration preserves old Plugin files and changes only their enabled flags through the official configuration API |
| `CLI-410` | [ADR 0227](0227-native-main-cutover-and-stable-release-routing.md) | Accepted | Native main integration and stable release routing reuse the qualified CLI/registry pipeline; legacy maintenance stays separate |
| `CLI-410` | [ADR 0232](0232-linux-arm64-registry-distribution.md) | Accepted | Linux ARM64 joins the native pip/npm matrix; new packets require four targets while historical three-target packets remain verifiable |
| `CLI-409` | [ADR 0233](0233-antigravity-local-plugin-adapter.md) | Accepted | Native Antigravity Plugin projection and separately confirmed official registration; live Skill/MCP qualification remains explicit |
| `CLI-409` | [ADR 0234](0234-antigravity-compact-skill-entries.md) | Accepted | One AGY research entry with internal workflow routing and a separate reply-only entry; supersedes ADR 0233's public wrapper layout |
| `CLI-402` | [ADR 0235](0235-install-plugins-for-detected-hosts.md) | Accepted | One install-all entry selects detected Host CLIs and reuses separate Plugin approvals; explicit Host lists remain strict |
| `CLI-409` | [ADR 0236](0236-pi-local-package-adapter.md) | Accepted | Pi local package with compact Skills and native MCP, integrated into detected Host installation |
| `CLI-402` | [ADR 0237](0237-automatic-plugin-source-selection.md) | Accepted | Detected-all defaults and verified per-Host source selection without platform/path prompts |
| `CLI-402` | [ADR 0238](0238-resilient-plugin-installation.md) | Accepted | Per-Host progress, bounded diagnostics, continued batches and targeted recovery summaries |

## Decision lifecycle

1. New architectural choices receive the next ADR number and a task ID.
2. A proposed ADR must name an owner and include measurable acceptance tests.
3. Accepted ADRs are immutable in meaning. Material changes require a new ADR
   that names the decision it supersedes.
4. Implementation pull requests link the ADR and provide its acceptance
   evidence; an ADR is not evidence that the implementation already passes.
5. Generated payloads, host caches, and external marketplace catalogs remain
   outside this decision-log source boundary.
