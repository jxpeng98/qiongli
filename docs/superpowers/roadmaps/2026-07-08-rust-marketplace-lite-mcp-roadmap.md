# Rust Marketplace Lite MCP Roadmap

> **Status: superseded on July 9, 2026.** The Rust skeleton and packaging
> milestones in this document have landed on `dev`. Functional closure and all
> subsequent convergence work are now tracked by
> `docs/superpowers/roadmaps/2026-07-09-unified-platform-roadmap.md` and
> `docs/superpowers/plans/2026-07-09-lite-mcp-functional-closure.md`. The stage
> labels below are retained as historical planning context and must not be used
> as current implementation status.

## Purpose

Qiongli Marketplace Lite should become a one-step local plugin install for
Codex, Claude Code, Claude Desktop, and other local MCP hosts. The installed
plugin should include a self-contained Rust MCP executable so users can run the
Literature Provider tools without installing Node, Python, npm, pip, Cargo,
Rust, or the Full CLI.

This roadmap turns the Rust Lite MCP decision into staged product and
engineering milestones. The detailed design and task-level execution plan live
in:

- `docs/superpowers/specs/2026-07-07-marketplace-lite-self-contained-mcp-design.md`
- `docs/superpowers/plans/2026-07-07-marketplace-lite-self-contained-mcp.md`

## Current Baseline

As of July 8, 2026:

- Marketplace and direct plugin artifacts can bundle a lightweight literature
  MCP server, but the current server entry is still Node-based.
- Standalone MCPB packaging exists, but it also uses the Node literature
  provider runtime.
- Python Full CLI owns the complete Qiongli runtime: installer, doctor,
  provider tools, orchestrator, task planning, task execution, project
  guidance, local agent coordination, release validation, and CLI/MCP serving.
- Codex, Claude Code, and Claude Desktop can launch local MCP subprocesses, but
  marketplace plugin installs must not assume `node`, `python`, or `qiongli`
  exists on the user's `PATH`.
- Zotero import-file generation can be done inside the Lite MCP runtime, while
  direct Zotero library search and writes still require the separate Zotero
  Companion because that code runs inside Zotero's extension boundary.

## Product Direction

Qiongli keeps two runtime profiles:

| Runtime profile | Primary install path | End-user runtime dependency | Role |
|---|---|---:|---|
| Marketplace Lite | Codex, Claude Code, Claude Desktop, MCPB, or other plugin/MCP package | None beyond the host app | Rust local MCP executable for literature provider setup, search, evidence export, Zotero import files, optional Zotero Companion bridge, and preview-only workflow planning |
| Full CLI | `qiongli install --profile full`, PyPI, or local source install | Python Qiongli runtime | Complete product runtime with full MCP tools, orchestrator, task execution, local agents, project guidance, doctor checks, release tooling, and validation workflows |

Marketplace Lite and Full CLI are two profiles of one product. Rust is selected
for the no-dependency Lite MCP runtime. Python remains the Full CLI runtime.

## Roadmap Principles

- End users of Marketplace Lite must not install Node, Python, Cargo, Rust, npm,
  pip, shell wrappers, or the Full CLI.
- The Lite MCP server runs locally over standard stdio MCP. Remote MCP hosting
  is not the default path for this roadmap.
- Rust Lite implements the marketplace-safe subset. It must not launch local
  agents or arbitrary shell commands.
- Full CLI remains Python through this roadmap. A Full CLI Rust migration would
  require a separate future spec.
- Compatibility is contract-first: shared schemas, golden fixtures, black-box
  MCP parity tests, and release gates define the product contract.
- Plugin artifacts are platform-specific where binaries require it. A single
  archive must not pretend to be no-dependency on every OS if it contains only
  one native executable.
- Zotero Companion remains a separate Zotero-side install for local library
  search and writes. Lite MCP can provide import-file workflows without it.
- Security defaults are local-first: loopback-only setup pages, secret
  redaction, owner-only config permissions where supported, and no API keys in
  manifests, logs, tests, or release artifacts.

## Stage 0: Decision And Design Lock

Status: in progress.

Primary outcome:

- The project commits to Rust for Marketplace Lite MCP and explicitly skips Go.
- Full CLI remains Python and is not pulled into the Lite MCP rewrite.

Scope:

- Record the Rust runtime decision.
- Define Marketplace Lite versus Full CLI responsibilities.
- Define the no-runtime install target for Codex, Claude Code, Claude Desktop,
  standalone MCPB, and other local MCP hosts.
- Define the Zotero boundary and the optional Companion bridge.
- Keep existing Node MCP packaging as a temporary reference during migration.

Success criteria:

- The design document names Rust as the selected Lite MCP runtime.
- The implementation plan contains no Go migration path.
- The design says Full CLI stays Python during this project.
- The roadmap, spec, and implementation plan agree on the same runtime split.

Formal references:

- `docs/superpowers/specs/2026-07-07-marketplace-lite-self-contained-mcp-design.md`
- `docs/superpowers/plans/2026-07-07-marketplace-lite-self-contained-mcp.md`

## Stage 1: Contract Foundation

Status: planned.

Primary outcome:

- Rust Lite and Python Full are kept consistent through external contracts
  before implementation diverges.

Scope:

- Add canonical Lite MCP tool definitions.
- Add provider config, literature result, diagnostics, search-plan, and Zotero
  import-file schemas.
- Add golden provider fixtures for OpenAlex, Semantic Scholar, Crossref,
  PubMed, and arXiv.
- Add contract tests that validate the fixture shapes before either runtime is
  trusted.

Success criteria:

- `content/mcp-contracts/` exists with versioned schemas and fixtures.
- Python tests fail when a required Lite tool or result field is missing.
- Contract fixtures contain no secrets or machine-local paths.
- The Node runtime remains only a parity reference, not the future contract.

Formal execution tasks:

- Task 1 in `docs/superpowers/plans/2026-07-07-marketplace-lite-self-contained-mcp.md`

## Stage 2: Rust Lite MCP Skeleton

Status: planned.

Primary outcome:

- A Rust executable can start as a local stdio MCP server and answer core MCP
  lifecycle requests.

Scope:

- Add `packages/qiongli-lite-mcp/` as a Rust package.
- Implement JSON-RPC request and response handling.
- Implement `initialize`, `ping`, `tools/list`, and `tools/call` dispatch.
- Load tool definitions from the shared contract assets.
- Add basic CLI flags such as `--transport stdio` and `--version`.

Success criteria:

- `cargo test` passes for the Rust package.
- The binary responds to `initialize` and `tools/list` without Node or Python.
- Tool names come from the contract file, not duplicated hard-coded lists.
- Startup failure messages are sanitized and useful for marketplace diagnostics.

Formal execution tasks:

- Task 3 in `docs/superpowers/plans/2026-07-07-marketplace-lite-self-contained-mcp.md`

## Stage 3: Provider Configuration Parity

Status: planned.

Primary outcome:

- Marketplace Lite and Full CLI read and write the same provider configuration
  shape.

Scope:

- Implement provider config path resolution with `QIONGLI_CONFIG_HOME` support.
- Implement config status, provider configuration, config saving, and the local
  setup wizard.
- Preserve provider aliases and secret redaction behavior.
- Bind the setup wizard only to loopback addresses.

Success criteria:

- Lite MCP can report provider status with an empty config.
- Lite MCP can save provider config without returning raw secrets in output.
- Full CLI and Lite MCP agree on provider names, aliases, and config fields.
- Tests prove config files and diagnostics do not leak API keys.

Formal execution tasks:

- Task 4 in `docs/superpowers/plans/2026-07-07-marketplace-lite-self-contained-mcp.md`

## Stage 4: Literature Search Core

Status: planned.

Primary outcome:

- Marketplace Lite provides useful literature discovery without the Full CLI.

Scope:

- Implement `qiongli_literature_status`.
- Implement `qiongli_search_plan`.
- Implement `qiongli_literature_search`.
- Implement provider clients for OpenAlex, Semantic Scholar, Crossref, PubMed,
  and arXiv.
- Normalize provider responses into the shared literature result schema.
- Preserve diagnostics and partial-provider failure reporting.

Success criteria:

- Automated tests cover at least arXiv and one JSON provider without
  user-installed runtimes.
- Search responses match the normalized result contract.
- Provider failures are reported per provider without exposing credentials.
- Lite MCP keeps startup independent from configured API keys.

Formal execution tasks:

- Tasks 5 and 6 in `docs/superpowers/plans/2026-07-07-marketplace-lite-self-contained-mcp.md`

## Stage 5: Evidence Export, Zotero Boundary, And Preview Planning

Status: planned.

Primary outcome:

- Marketplace Lite supports practical literature workflows without claiming Full
  CLI execution capabilities.

Scope:

- Implement evidence export from normalized search results.
- Implement Zotero import-file generation for CSL JSON, RIS, BibTeX, and import
  reports.
- Implement Zotero Companion probing through loopback only.
- Return explicit `companion_missing` or `fallback_only` states when the
  Companion is absent.
- Add preview-only orchestrator route and task-plan tools.

Success criteria:

- Zotero import files can be generated without Zotero Companion.
- Direct Zotero local-library search and writes only work through the Companion.
- Preview tools do not launch agents, write project guidance, or call shell
  commands.
- Lite and Full CLI produce compatible overlapping result shapes.

Formal execution tasks:

- Tasks 7, 8, and 9 in `docs/superpowers/plans/2026-07-07-marketplace-lite-self-contained-mcp.md`

## Stage 6: Platform Binary Build And Marketplace Packaging

Status: planned.

Primary outcome:

- Marketplace plugin artifacts launch a plugin-local Rust binary instead of
  `node`.

Scope:

- Add build scripts for current-platform and target-platform Rust binaries.
- Stage binaries at a stable plugin-local path such as
  `bin/qiongli-literature-provider`.
- Update Codex Marketplace plugin MCP config.
- Update Claude Code marketplace plugin MCP config.
- Update Claude Desktop direct plugin MCP config.
- Add platform target metadata for `marketplace-lite-binary`.
- Validate binary names, executable permissions, manifest command paths, and
  forbidden runtime commands.

Success criteria:

- Marketplace manifests do not invoke `node`, `python`, `npm`, `pip`,
  `qiongli`, or shell wrapper scripts.
- Plugin artifacts contain the platform-specific Rust executable.
- Local install validation reports `marketplace-lite-binary` mode.
- A machine without Node and Python can run `initialize`, `tools/list`, and
  `qiongli_literature_status` against the bundled binary.

Formal execution tasks:

- Tasks 10, 11, and 12 in `docs/superpowers/plans/2026-07-07-marketplace-lite-self-contained-mcp.md`

## Stage 7: Python Full CLI Compatibility Gate

Status: planned.

Primary outcome:

- Rust Lite does not drift from Python Full CLI where the tools overlap, and it
  does not weaken the complete Full CLI runtime.

Scope:

- Add black-box parity tests for shared MCP tool names and shared result shapes.
- Keep Full CLI tests in every release gate.
- Validate that Rust Lite is a subset profile, not a replacement for
  `qiongli mcp serve --transport stdio`.
- Document differences between preview-only Lite tools and executable Full CLI
  tools.

Success criteria:

- Lite tool names are declared in the shared contract.
- Overlapping Lite and Full CLI tool outputs match shared schemas.
- Full CLI tests pass without depending on the Rust binary.
- Release validation clearly labels artifacts as `marketplace-lite-binary`,
  `python-full-runtime`, or `legacy-node-provider`.

Formal execution tasks:

- Task 13 and Task 16 in `docs/superpowers/plans/2026-07-07-marketplace-lite-self-contained-mcp.md`

## Stage 8: MCPB Migration And Node Runtime Sunset

Status: planned.

Primary outcome:

- Standalone MCPB users get the same self-contained Rust Lite MCP runtime as
  marketplace plugin users.

Scope:

- Update MCPB manifest and packaging to include the Rust Lite executable.
- Keep a clearly labeled legacy Node fallback for one release train if needed.
- Reuse the same contracts and binary staging path as marketplace artifacts.
- Remove or de-emphasize the Node provider only after parity and installation
  checks cover all supported targets.

Success criteria:

- MCPB manifest no longer declares `server.type: "node"` for the primary path.
- MCPB package launches the Rust Lite binary over stdio.
- Legacy Node MCPB, if retained, is explicitly marked as legacy.
- MCPB and marketplace artifacts share the same provider behavior contract.

Formal execution tasks:

- Task 14 in `docs/superpowers/plans/2026-07-07-marketplace-lite-self-contained-mcp.md`

## Stage 9: Documentation, Beta Rollout, And Feedback

Status: planned.

Primary outcome:

- Users and maintainers understand which runtime profile they installed and
  which capabilities are available.

Scope:

- Update install docs, cross-platform MCP docs, plugin-first architecture docs,
  README, README_CN, and workflow skill runtime notes.
- Explain that Marketplace Lite is Rust-built and local.
- Explain that Full CLI is Python-backed and complete.
- Explain that Zotero Companion remains separate for direct Zotero library
  access.
- Publish beta release notes with supported platforms and fallback behavior.
- Collect startup failure reports by host, OS, architecture, and artifact type.

Success criteria:

- Docs no longer describe Marketplace Lite as Node-dependent.
- The no-runtime guarantee is scoped to Marketplace Lite provider tools.
- Full CLI install docs continue to describe Python requirements honestly.
- Release notes list the exact supported binary targets.

Formal execution tasks:

- Task 15 and Task 16 in `docs/superpowers/plans/2026-07-07-marketplace-lite-self-contained-mcp.md`

## Full CLI Migration Decision Gate

Status: future evaluation only.

The Full CLI should not migrate to Rust during the Marketplace Lite roadmap.
Revisit a Full CLI migration only after Rust Lite has shipped through stable
release trains and the project has evidence that a native complete runtime is
worth the cost.

Re-evaluation inputs:

- Rust Lite binary maintenance is stable across macOS, Linux, and Windows.
- Contract-first parity tests have reduced, rather than increased, maintenance
  cost.
- Marketplace users need more than preview-only planning without installing
  Full CLI.
- Python Full CLI distribution becomes a persistent user blocker, not only a
  marketplace-lite startup blocker.
- The team has a clear design for local agent execution, project writes,
  installer behavior, and doctor checks in a native runtime.

Possible future path if the gate is passed:

- Extract shared Rust crates from the Lite runtime only after their contracts
  are stable.
- Keep Python Full CLI as the product owner while Rust modules replace narrow
  subsystems one at a time.
- Migrate high-churn orchestration last, not first.
- Treat a full Rust CLI as a separate product migration with its own spec,
  acceptance criteria, rollback plan, and release train.

Default decision:

- Keep Full CLI in Python.
- Use Rust for the self-contained Marketplace Lite MCP runtime.
- Use contracts and black-box tests to keep both profiles consistent.

## Risk Register

| Risk | Impact | Mitigation |
|---|---|---|
| Rust maintenance burden | Slows future Lite MCP changes | Keep scope narrow, use small modules, add tests before behavior, and treat AI-assisted implementation as contract-driven code generation with human review |
| Platform binary mismatch | Plugin installs but cannot launch MCP | Build and validate target-specific artifacts; publish separate platform artifacts when the marketplace cannot select binaries |
| Full CLI drift | Lite and Full return incompatible shapes | Shared schemas, golden fixtures, and black-box MCP parity tests in release gates |
| Secret leakage | Provider keys appear in logs or artifacts | Redaction tests, fixture scans, no secrets in manifests, and owner-only config permissions where supported |
| Zotero expectation mismatch | Users expect direct library writes from the plugin alone | Keep import-file support built in; label direct search/write as Companion-backed only |
| Node fallback becomes permanent | Migration never converges | Set the Node runtime as a temporary reference and require explicit legacy labeling after Rust parity passes |

## Release Gates

Every beta or stable release that claims Marketplace Lite no-runtime support
must pass:

- Rust package tests for `packages/qiongli-lite-mcp`.
- Python contract fixture tests.
- Black-box MCP startup tests for the Rust binary.
- Marketplace artifact tests that reject Node, Python, npm, pip, qiongli, and
  shell-wrapper entrypoints.
- Provider config redaction tests.
- Zotero import-file tests.
- Full CLI regression tests for Python runtime compatibility.
- Platform target validation.
- Marketplace install validation.
- Secret, local-path, and diff hygiene scans.

## Definition Of Done

The roadmap is complete when:

- Codex Marketplace plugin, Claude Code plugin, Claude Desktop direct plugin,
  and standalone MCPB can install a Rust-built local Lite MCP server.
- The Lite MCP server starts without user-installed Node or Python.
- Lite provider setup, search, search planning, evidence export, Zotero
  import-file export, optional Companion bridge, and preview planning work over
  standard stdio MCP.
- Full CLI remains Python-backed and keeps complete orchestration and execution
  behavior.
- Release validation prevents accidental runtime regressions back to Node or
  Python entrypoints for Marketplace Lite.
