# Qiongli 2 Native Acceleration Design

Status: approved for roadmap integration on July 13, 2026

Base: `2.x` after PR #62, merge commit `ebd2d7bef651fcbd22a7310aa50f9945604fa9eb`

First rolling branch: `feat/2x-native-alpha1`

## Goal

Complete the Qiongli 2 Rust-native migration as quickly as practical without
trading speed for accumulated defects, unsafe side effects, or false release
claims.

The migration produces one native product whose CLI, desktop UI, MCP modes,
skills, agents, orchestrator, installer, updater, and local integrations run
without user-installed Rust, Python, or Node.js.

## Problem

The accepted 1.x baseline and the first native foundation are already complete:

- `v1.19.0-beta.1` is frozen as the final normal 1.x development baseline;
- the accepted architecture decisions and 2.x branch point exist;
- Contract v2 and the accepted 1.x source inventory are frozen;
- FND-202A-D provide the resource-pack contract, collector, deterministic
  writer, and bounded in-memory loader;
- exact-head CI for PR #62 is green on Linux, macOS, and Windows.

The remaining bottleneck is process shape. The existing plan still describes
many independently reviewed PRs and requires a legacy Python test tier whose
single Ubuntu unit-test step takes about 33-35 minutes. That protects a frozen
implementation which is no longer the product target and delays every native
feedback cycle.

At the same time, deferring all validation until the rewrite is complete would
move defects to the most expensive point in the program. The new process must
remove legacy blocking work while preserving fast Rust-native feedback.

## Decisions

### One active rolling Draft PR

There is exactly one active 2.x migration branch and one active migration PR at
any time.

For the first native vertical slice:

- branch: `feat/2x-native-alpha1`;
- base: current protected `2.x`;
- PR state: Draft until every alpha.1 exit criterion passes;
- integration target: `2.x`;
- branch lifetime: design rebaseline through `v2.0.0-alpha.1` readiness.

FND, CFG, MCP, UI, installer, integration, and packaging IDs remain planning
labels. They do not create branches or PRs. Work is recorded as cohesive
Conventional Commits on the rolling branch. The branch is pushed frequently so
the same Draft PR always shows the current integrated state.

After alpha.1 is ready, the rolling PR becomes Ready and merges. Only then may
the next milestone create its successor rolling branch. There are no parallel
feature PRs into the active migration branch and no direct implementation
commits to protected `2.x`.

### 1.x is frozen and non-blocking

Normal 1.x development stops. Python and Node sources remain read-only
references and frozen compatibility evidence until their production paths are
retired.

The 2.x migration does not:

- add Python or Node product features;
- regenerate new legacy behavior inventories;
- require the full Python unit suite to pass for native commits;
- optimize or shard the legacy Python suite;
- preserve undocumented implementation accidents merely because a Python test
  observes them.

An emergency 1.x security or release-breakage fix is a separate maintenance
decision. It does not reopen ordinary 1.x development and must not interrupt
the active 2.x rolling branch unless the same defect invalidates a shared
contract or native trust boundary.

### Rust-native feedback remains continuous

Dropping legacy blocking tests does not mean testing only at the end.

Every native change must keep the changed Rust workspace compiling and run the
smallest focused tests that prove its new behavior. The complete Rust workspace
test is currently fast enough to run on every PR push. Cross-platform and
release checks expand only when the relevant component or release claim is
present.

The required 2.x lanes are:

1. Rust formatting, check, Clippy, and workspace tests;
2. Linux, macOS, and Windows native build/test jobs;
3. focused boundary tests for touched state, path, process, network, secret,
   installer, or update behavior;
4. milestone-only packaging, signing, clean-machine, activation, and rollback
   acceptance.

Legacy Python and Node suites move to a manual compatibility workflow. They
may provide diagnostic evidence but cannot be required branch checks for the
rolling native PR.

### Preserve architecture, reduce physical scaffolding

ADRs 0201-0207 remain accepted. Their security, state, executable, content,
installer, UI, backend, and release boundaries are not weakened.

The previous workspace diagram named fourteen possible service crates. The
alpha.1 implementation starts with seven physical library crates and one
application. Logical responsibilities remain modules until an independent
compile, release, reuse, or security boundary justifies another crate.

```text
packages/qiongli-native/
  apps/
    qiongli/
  crates/
    qiongli-content/
    qiongli-config/
    qiongli-runtime/
    qiongli-execution/
    qiongli-platform/
    qiongli-ui/
    qiongli-testkit/
```

Responsibilities:

| Crate | Responsibility |
|---|---|
| `qiongli-content` | Resource-pack collection, writing, verification, embedding, selection, and approved materialization |
| `qiongli-config` | Global/project state, schema migration, atomic writes, redaction, and secret-store facade |
| `qiongli-runtime` | Contract loading, providers, domain services, and Lite/Full MCP dispatch |
| `qiongli-execution` | Agent backends, native ToolHost, execution policy, and orchestrator |
| `qiongli-platform` | Install plans, client adapters, doctor, updater, repair, removal, and rollback |
| `qiongli-ui` | egui view models, components, accessibility, and typed service intents |
| `qiongli-testkit` | Native fixtures, fake backends, target-install harnesses, and acceptance helpers |
| `apps/qiongli` | The single multi-mode native executable and composition root |

`qiongli-config` retains exclusive ownership of Qiongli state mutation.
`qiongli-platform` owns host installation side effects. `qiongli-execution`
owns model/tool execution policy. The UI cannot directly write state, launch
processes, access secrets, or call providers.

The dependency direction is one-way:

```text
apps/qiongli -> qiongli-ui, qiongli-platform, qiongli-execution,
                qiongli-runtime
qiongli-ui -> typed service interfaces
qiongli-platform -> qiongli-config, qiongli-content
qiongli-execution -> qiongli-runtime, qiongli-config
qiongli-runtime -> qiongli-config, qiongli-content
qiongli-config -> qiongli-content contracts where required
```

No library depends on the application or UI. Canonical academic content stays
under `content/` and is embedded or materialized; it is not translated into
hand-maintained Rust source.

## Accelerated Migration Flow

### R0 — Native control plane

R0 creates the one rolling branch and Draft PR, makes the accelerated roadmap
authoritative, and replaces legacy required checks with native checks.

Exit criteria:

- one active Draft PR targets `2.x`;
- its required checks do not run the full Python or Node suites;
- the complete native workspace passes format, check, Clippy, and tests;
- Tier 1 native jobs report against the same exact head.

### R1 — Native foundation closure

R1 completes FND-202E/F and establishes the first state service.

Scope:

- atomic materialization only to a temporary or explicitly approved root;
- embedded pack generation and source-drift verification;
- v2 config-home resolution, schemas, atomic writes, redaction, and secret
  references;
- native content, config, status, and doctor commands.

Exit criteria:

- the binary can list and materialize embedded profiles without a checkout;
- a failed materialization or config mutation leaves prior bytes intact;
- startup and the supported commands work with an empty `PATH`.

### R2 — Shared Lite runtime

R2 extracts the existing Rust Lite provider and MCP behavior into the native
workspace instead of creating a second implementation.

Scope:

- provider configuration and bounded literature search;
- evidence export and supported Zotero operations;
- Contract v2 profile dispatch and stdio MCP;
- a thin compatibility entry for the old Rust Lite package while required.

Exit criteria:

- the native executable exposes the advertised Lite tools;
- no production process launches Python or Node;
- provider failures, credentials, timeouts, and unavailable capability states
  return typed, redacted results.

### R3 — Alpha.1 native product slice

R3 joins content, config, Lite runtime, platform integration, and a minimal
desktop manager.

Scope:

- typed `InstallPlan` preview, apply, verify, repair, remove, and rollback;
- supported Codex and Claude local adapters;
- egui views for Skills, MCP, Providers, Integrations, and Diagnostics;
- CLI and UI calls through the same service layer;
- current advertised target artifact and clean-machine startup evidence.

Exit criteria:

- a clean machine can start the CLI and UI without Rust, Python, or Node;
- the user can inspect embedded skills, configure a provider, start Lite MCP,
  register one Codex and one Claude local surface, diagnose status, and remove
  the managed integration;
- release notes state the actual target and incomplete Full-runtime boundary;
- all required alpha.1 native checks pass on the merge candidate.

R3 completion allows the Draft PR to become Ready and permits
`v2.0.0-alpha.1` preparation.

### R4 — Full native runtime

The next rolling milestone adds Full MCP, project/domain services,
`AgentBackend`, ToolHost, and the orchestrator. At least one direct API backend
must complete an end-to-end bounded workflow without an external agent CLI.

R4 targets `v2.0.0-alpha.2` rather than reopening the alpha.1 branch.

### R5 — Native cutover and beta

The final migration milestone adds the complete Tier 1 artifact matrix,
state-import and rollback acceptance, signed update metadata, packaging
evidence, and removal of Python/Node production invocation.

R5 completion permits `v2.0.0-beta.1`. Beta hardening and stable promotion are
separate release decisions based on observed defects, not legacy parity-suite
completion.

## Long-Flow Development Loop

Each development session works through as many dependency-contiguous behaviors
as can be implemented and verified safely:

```text
read rolling PR state
-> select the next dependency-contiguous batch
-> implement one or more complete behaviors
-> run focused Rust tests
-> run the native workspace gate
-> create one or more atomic commits
-> push the same rolling branch
-> update the Draft PR ledger
-> continue
```

A task ID completing is not a stopping condition. Work pauses only for:

- required external authority such as signing credentials or a real client
  installation action;
- a product choice whose alternatives materially change behavior;
- a concrete security or data-loss blocker in the active boundary;
- an external platform limitation that prevents truthful acceptance.

## Draft PR Ledger

The rolling PR body maintains four short sections:

1. current native capabilities;
2. completed checkpoint commits and native evidence;
3. next dependency-contiguous batch;
4. known limitations and release nonclaims.

The PR description reports only tests that actually ran on the current head.
It never carries results from a superseded commit as exact-head evidence.

## Error And Security Policy

Speed comes from removing irrelevant work, not removing concrete safeguards.

Always-blocking defects remain:

- credential or private-data disclosure;
- path traversal or symlink escape;
- unauthorized state mutation, process launch, listener, or network behavior;
- corrupt or non-atomic migration/install/update behavior;
- fabricated academic evidence;
- a production dependency on Python or Node;
- failure to compile or run the changed native component;
- a public artifact or capability claim unsupported by current evidence.

Repository-wide governance programs, speculative hardening, and unrelated
legacy debt do not block the rolling PR. A real defect is fixed in its owning
module with a focused regression.

## Tradeoffs

The rolling PR is larger than the previous micro-PR model. Atomic commits, a
current ledger, and an always-green native head provide the rollback and review
boundaries that separate PRs previously supplied.

Removing legacy tests from required CI means undocumented Python behavior may
diverge. This is accepted because 1.x is frozen and the target is the documented
Rust-native product contract. Frozen source and fixtures remain available when
a specific parity question matters.

The seven-crate layout may later split. Prematurely creating every possible
crate is avoided because interface churn is more expensive than an evidence-led
split during alpha.

## Acceptance Criteria

This design is successfully adopted when:

- the accelerated roadmap is the authoritative execution source;
- `feat/2x-native-alpha1` is the only active native migration branch;
- one Draft PR tracks all alpha.1 implementation;
- Python and Node full suites are absent from required 2.x checks;
- every new native behavior compiles and has proportionate Rust evidence;
- accepted ADRs and canonical content remain intact;
- alpha.1 becomes Ready only after the R3 clean-machine native vertical slice
  passes on its exact merge candidate.

## Non-Goals

- continuing normal 1.x product development;
- porting Python files line by line;
- preserving every undocumented Python implementation detail;
- publishing alpha.1 before its stated native slice works;
- implementing remote MCP for hosted cloud surfaces during the local alpha;
- splitting work into a new branch or PR for every roadmap task.
