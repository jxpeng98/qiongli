# Qiongli 1.x Closeout And 2.x Native Bootstrap Execution Plan

Status: in progress; A1-A8 are complete, `v1.19.0-beta.1` is published and
accepted at exact tag commit `8d2e99866ce4c4efb8b3b5e0265c0c1f89a36b0f`,
and the frozen migration baseline is committed on `dev`. The protected `2.x`
branch is initialized and validated at branch-point record commit
`69855fd50413ee6809baf21eb345fc0c55721de3`. B0 records the accepted native
architecture decisions, and the B2 native workspace scaffold and repository
gates are implemented. In B3, CTR-201A-D are merged and validation-backed, and
CTR-201E's accepted-source Full CLI runtime-inventory freeze is merged through
protected PR #57. CTR-201F closes the accepted-source orchestrator runtime
inventory with 44 deterministic cases (one A8 plus 43 bounded), six behavior
dimensions, and six explicit dispositions. The CTR-201 source-oracle gate is
complete in this tree. CTR-202 and FND-202 are separate successors whose
CTR-201 dependency is closed. CTR-202 and the FND-202A typed native
manifest/profile contract are integrated through PR #59 with green exact-head
CI. FND-202B is integrated through PR #60 with green exact-head CI. FND-202C
is integrated through PR #61 with green exact-head CI. FND-202D is the active
bounded verifier/loader branch candidate with focused local tests; its
integration and FND-202E-F remain open.
B1 / REL-201 now provides a typed native
release identity, strict 1.x/2.x channel isolation, and an external-staging
`v2.0.0-alpha.1` dry-run; real native publication remains fail-closed pending
the later artifact, signing, updater, and acceptance gates.
Roadmap:
`docs/superpowers/roadmaps/2026-07-10-qiongli-2-rust-native-platform-roadmap.md`
Release source: `dev`
Native development branch: `2.x`, created only after final 1.x acceptance and
baseline freeze
Final planned 1.x beta: `v1.19.0-beta.1`
First 2.x release: `v2.0.0-alpha.1`

## Objective

Execute a controlled handoff from the current Python-led 1.x product to the
Rust-native 2.x program without losing behavior, data, skills, agents,
orchestrator semantics, install safety, or release evidence.

This plan covers two blocking phases:

1. clean and publish the final planned 1.x beta from the current `dev` state;
2. build the first usable, zero-language-runtime native vertical slice and
   publish `v2.0.0-alpha.1` only after its acceptance gate passes.

No Rust migration implementation should land on `dev` before the accepted 1.x
tag freezes the reference behavior. After the acceptance receipt and normalized
baseline are committed and pushed, create `2.x` from that clean post-release
baseline commit and land all native implementation there. Roadmap and
architecture documentation may land before the tag because they do not change
runtime behavior.

## Current Baseline

Audited on July 10, 2026 and refreshed during A0 after adding the 2.x planning
documents:

- `v1.18.0-beta.3` exists locally and is a published GitHub prerelease;
- `dev` is one commit ahead of `origin/dev` at `2e240dc1`, which contains the
  initial Capability Contract v2 pilot;
- the runtime and workflow product version is still `1.18.0-beta.3`;
- Rust Lite and the legacy MCPB component are still `0.2.0-beta.2`;
- the frozen A0 snapshot contained 65 dirty paths: 50 tracked and 15
  untracked; the pre-commit A2 refresh is recorded below;
- the dirty Stage 1 work adds configuration and literature-planning contracts,
  Full/Rust/Node behavior changes, validation, tests, and documentation;
- Capability Contract v2 is `2.0.0-preview.3`, status `pilot`, with `6/23`
  canonical records and `7/24` public names;
- no `v1.19.0-beta.1` tag exists.

The current tree is not releaseable. `release_ready.sh` is expected to reject
it unless the unsafe `--allow-dirty` option is used. This plan explicitly
forbids using `--allow-dirty` to bypass cleanup.

## A0 Execution Record — Frozen Dev Inventory

Snapshot time: `2026-07-10T20:30:41Z` (`2026-07-10 21:30:41 BST`)
Snapshot commit: `2e240dc1bf67`
A0 status: complete
Release status: not ready

### Git and release identity

| Item | Frozen value |
|---|---|
| Local branch | `dev` |
| Local HEAD | `2e240dc1` (`feat(platform): add capability contract v2 pilot`) |
| Remote `origin/dev` | `3633845a` |
| Divergence | ahead 1, behind 0 |
| Staged paths | 0 |
| Modified tracked paths | 50 |
| Untracked paths | 15 |
| Deleted or renamed paths | 0 |
| Tracked working diff | 6,697 insertions, 605 deletions |
| Current accepted tag | annotated `v1.18.0-beta.3`, peeled commit `12aea420` |
| Current GitHub release | published prerelease, not draft |
| `v1.19.0-beta.1` ref | absent locally and remotely |
| `release/1.x-python` ref | absent locally and remotely |
| `2.x` ref | absent locally and remotely |

Version identity remains unchanged during A0:

| Component | A0 value |
|---|---|
| Python product | `1.18.0b3` |
| workflow and npm | `1.18.0-beta.3` |
| Rust Lite | `0.2.0-beta.2` |
| literature MCPB | `0.2.0-beta.2` |
| Capability Contract v2 | `2.0.0-preview.3`, `pilot`, 6/23 canonical, 7/24 public |

No version, tag, branch, staging area, remote release, marketplace ref, or
package registry was mutated during A0.

### Exhaustive dirty-path ownership

Every A0 path has exactly one primary owner. Mixed files listed later require
hunk-aware staging if the feature batches are committed separately.

#### Configuration contract and security — 15 modified paths

- `packages/python-qiongli/src/qiongli/bridges/mcp_cli.py`
- `packages/python-qiongli/src/qiongli/bridges/mcp_config_wizard.py`
- `packages/python-qiongli/src/qiongli/bridges/mcp_tool_handlers.py`
- `packages/python-qiongli/src/qiongli/bridges/orchestrator.py`
- `packages/python-qiongli/src/qiongli/bridges/provider_config.py`
- `packages/python-qiongli/src/qiongli/bridges/providers/crossref_client.py`
- `packages/python-qiongli/src/qiongli/bridges/providers/literature_search.py`
- `packages/python-qiongli/src/qiongli/bridges/providers/openalex_client.py`
- `packages/python-qiongli/src/qiongli/bridges/providers/pubmed_client.py`
- `packages/python-qiongli/src/qiongli/bridges/providers/s2_client.py`
- `packages/python-qiongli/src/qiongli/cli.py`
- `packages/qiongli-lite-mcp/src/config/provider_config.rs`
- `packages/qiongli-lite-mcp/src/config/wizard.rs`
- `packages/qiongli-literature-mcpb/server/config.mjs`
- `tooling/scripts/mcp_scholarly_search.py`

#### Literature planning — 4 modified paths

- `packages/python-qiongli/src/qiongli/bridges/hybrid_search_router.py`
- `packages/python-qiongli/src/qiongli/bridges/literature_mcp_tools.py`
- `packages/qiongli-lite-mcp/src/mcp/server.rs`
- `packages/qiongli-lite-mcp/src/searchplan.rs`

#### Contracts, validators, and tests — 36 paths

Modified:

- `content/mcp-contracts/fixtures/lite-tool-smoke-calls.json`
- `content/mcp-contracts/lite-tools.json`
- `content/mcp-contracts/v2/registry.json`
- `content/mcp-contracts/v2/registry.schema.json`
- `packages/qiongli-lite-mcp/tests/config_wizard.rs`
- `packages/qiongli-lite-mcp/tests/mcp_server.rs`
- `packages/qiongli-lite-mcp/tests/provider_config.rs`
- `packages/qiongli-lite-mcp/tests/searchplan.rs`
- `packages/qiongli-literature-mcpb/manifest.json`
- `packages/qiongli-literature-mcpb/server/index.mjs`
- `packages/qiongli-literature-mcpb/test/config.test.mjs`
- `tests/test_capability_contract_v2.py`
- `tests/test_cli.py`
- `tests/test_hybrid_search_router.py`
- `tests/test_literature_search.py`
- `tests/test_mcp_cli.py`
- `tests/test_mcp_config_wizard.py`
- `tests/test_mcp_connectors.py`
- `tests/test_mcp_literature_tools.py`
- `tests/test_mcp_tool_handlers.py`
- `tests/test_orchestrator_workflows.py`
- `tests/test_provider_clients.py`
- `tests/test_provider_config.py`
- `tooling/scripts/validate_capability_contract.py`
- `tooling/scripts/validate_research_standard.py`

Untracked canonical source/tests:

- `content/mcp-contracts/v2/schemas/qiongli_config_status.input.schema.json`
- `content/mcp-contracts/v2/schemas/qiongli_config_status.output.schema.json`
- `content/mcp-contracts/v2/schemas/qiongli_configure_provider.input.schema.json`
- `content/mcp-contracts/v2/schemas/qiongli_configure_provider.output.schema.json`
- `content/mcp-contracts/v2/schemas/qiongli_literature_status.input.schema.json`
- `content/mcp-contracts/v2/schemas/qiongli_literature_status.output.schema.json`
- `content/mcp-contracts/v2/schemas/qiongli_save_provider_config.input.schema.json`
- `content/mcp-contracts/v2/schemas/qiongli_save_provider_config.output.schema.json`
- `content/mcp-contracts/v2/schemas/qiongli_search_plan.input.schema.json`
- `content/mcp-contracts/v2/schemas/qiongli_search_plan.output.schema.json`
- `packages/qiongli-lite-mcp/tests/literature_planning_mcp.rs`

#### Documentation and roadmap — 8 paths

Modified:

- `docs/architecture.md`
- `docs/development/repository-structure.md`
- `docs/superpowers/plans/2026-07-10-capability-contract-v2-pilot.md`
- `docs/superpowers/roadmaps/2026-07-09-unified-platform-roadmap.md`

Untracked:

- `docs/superpowers/plans/2026-07-10-capability-contract-v2-configuration.md`
- `docs/superpowers/plans/2026-07-10-capability-contract-v2-literature-planning.md`
- `docs/superpowers/plans/2026-07-10-qiongli-1x-closeout-and-2x-native-bootstrap.md`
- `docs/superpowers/roadmaps/2026-07-10-qiongli-2-rust-native-platform-roadmap.md`

#### Release preparation — 2 modified paths

- `tests/test_release_automation.py`
- `tooling/scripts/release_preflight.sh`

Coverage check: 15 + 4 + 36 + 8 + 2 = 65 assigned paths. There are zero
unassigned or multiply assigned primary owners.

### Mixed-feature staging boundary

These 11 paths contain both configuration/security and literature/contract
behavior. They require hunk-aware staging, or they must remain together in one
green Stage 1 commit:

- `content/mcp-contracts/fixtures/lite-tool-smoke-calls.json`
- `content/mcp-contracts/lite-tools.json`
- `content/mcp-contracts/v2/registry.json`
- `packages/python-qiongli/src/qiongli/bridges/literature_mcp_tools.py`
- `packages/python-qiongli/src/qiongli/bridges/mcp_tool_handlers.py`
- `packages/qiongli-lite-mcp/src/mcp/server.rs`
- `packages/qiongli-literature-mcpb/manifest.json`
- `packages/qiongli-literature-mcpb/server/index.mjs`
- `tests/test_capability_contract_v2.py`
- `tests/test_mcp_literature_tools.py`
- `tests/test_mcp_tool_handlers.py`

`packages/qiongli-lite-mcp/src/config/provider_config.rs` is behaviorally
coupled to `packages/qiongli-lite-mcp/src/mcp/server.rs`. Do not create an
intermediate commit that fails to compile merely to force a smaller diff.

Default commit policy after A1:

1. prefer separate configuration/security and literature-planning commits only
   when hunk staging produces independently green Rust, Python, Node, contract,
   and focused-test states;
2. otherwise combine runtime, contracts, validators, tests, and release-gate
   changes into one green
   `feat(platform): complete capability contract v2 stage-one batches` commit;
3. commit documentation, roadmap, and migration-plan paths separately as
   `docs(platform): define 1.x closeout and Rust-native 2.0 migration`;
4. keep later version/release-prep changes in the dedicated A5/A6 commits.

### Exclusions and repository boundaries

No dirty path is excluded: all 65 belong to canonical contracts, runtime,
tests, tooling, or documentation. The following ignored/generated roots were
observed and remain outside the change set:

- `.pytest_cache/`;
- `dist/`;
- `docs/.vitepress/dist/`;
- `packages/qiongli-lite-mcp/target/`.

No dirty path is under generated plugin payloads, npm/Python payload mirrors,
root generated workflow/agent shapes, external marketplace catalogs, or host
plugin caches. Added-line scans found no machine-specific absolute user path,
private key, placeholder, or local secret.

### A0 gate result

| Gate | Result |
|---|---|
| Remote `dev` refreshed | pass |
| Branch divergence captured | pass |
| Every dirty path classified | pass: 65/65 |
| Staged/deleted/renamed path check | pass: 0/0/0 |
| Generated/local artifact exclusion | pass |
| Repository-boundary review | pass |
| `git diff --check` | pass |
| Feature intake frozen | active from this snapshot |
| Release-ready | **fail: A1 and later phases remain** |

### A1 entry blockers

The next implementation step is limited to reproducing and closing these
release blockers:

1. legacy Node config currently converts malformed, unreadable, unsupported, or
   invalid shared config to an empty object and can overwrite the original;
2. Node still needs version/shape validation, provider `enabled: false`
   enforcement, atomic replacement, existing-file permission tightening, and
   symlink/reparse-point protection;
3. Rust Windows config writes still inherit directory ACLs; implement and test
   a user-only DACL or prepare an explicit reviewed and signed release
   limitation;
4. the current Python/Rust changes for all-disabled no-network behavior, active
   diagnostics, disabled-credential filtering, redaction, and canonical plus
   deprecated year keys remain uncommitted and require focused and full gates.

No A1 fix, staging operation, commit, version bump, tag, publish, marketplace
change, or `2.x` branch creation was performed during A0.

## Execution Rules

- Preserve every pre-existing user change; do not reset, discard, or overwrite
  the current worktree during cleanup.
- Freeze new 1.x feature intake immediately. Only release closure,
  documentation, tests, compatibility, security, and version changes may enter
  the final beta.
- Keep source commits separate from generated release-prep changes.
- Do not tag or publish until the local tree is clean, the branch commit is
  pushed, and required branch checks pass on that exact commit.
- Do not advance generic marketplace refs while a native artifact is
  current-host-only.
- Do not claim Contract v2 is complete; release notes must report pilot status
  and exact coverage.
- Do not delete Python or Node behavior before normalized oracle evidence is
  frozen.
- Do not make Python, Node, or an external agent CLI a hidden dependency of a
  2.x production path.

## Phase A — Final 1.x Beta Closeout

### A0 — Freeze and inventory the current tree

Task IDs: `RLS-101A`, `RLS-101B`

Actions:

1. Capture branch, commit, remote divergence, status, diff statistics, untracked
   paths, and current versions.
2. Partition the existing changes into these review units:
   - already committed Capability Contract v2 pilot;
   - provider configuration contract and security batch;
   - literature status and search-plan contract batch;
   - compatibility changes for Rust Lite and legacy Node;
   - tests, validators, release gates, and documentation;
   - this 2.x roadmap and plan.
3. Mark generated output, cache, local receipt, and machine-specific files. None
   may be accidentally committed as canonical source.
4. Record every behavior change since `v1.18.0-beta.3` for the release note.

Commands for evidence, not mutation:

```bash
git status --short --branch
git log --oneline --decorate v1.18.0-beta.3..HEAD
git diff --check
git diff --name-status
git diff --stat v1.18.0-beta.3
```

Exit criteria:

- every dirty path has one owner and one intended commit or explicit exclusion;
- no generated distribution payload is treated as editable source;
- no new 1.x feature remains unclassified.

### A1 — Close security and compatibility blockers

Task ID: `RLS-101C`

The final-beta review must independently reproduce and close the following
current audit findings:

- Full must not fall back to Semantic Scholar when every provider is disabled;
- legacy Node must fail closed on malformed or unsupported shared config and
  must not overwrite it with an empty object;
- Node config writes must be atomic, permission-aware, and safe around links;
- disabled providers and their credentials must not be scheduled or exported to
  external adapters;
- diagnostic capability mode must use active, not merely configured, providers;
- Rust and Node config errors must redact local paths and secrets consistently;
- search-plan compatibility output must retain deprecated `fromYear` and
  `toYear` aliases beside canonical keys during the declared window;
- Windows shared-config ownership/DACL behavior must either be implemented and
  tested or recorded as an explicit signed release limitation.

Required focused evidence:

- all-providers-disabled call produces no network request;
- malformed config remains byte-identical after failed status/save/wizard calls;
- disabled credentials do not appear in subprocess environments;
- status, save, wizard, and MCP errors contain no secret or absolute local path;
- compatibility fixtures assert canonical and deprecated year keys;
- symlink/reparse-point and atomic-replacement cases are covered on supported
  systems.

Exit criteria:

- zero open release-blocking security or compatibility finding;
- targeted Python, Rust, and Node tests pass;
- the release limitation list is explicit and reviewed.

## A1 Execution Record — Security And Compatibility Closure

Execution time: `2026-07-10T21:23:29Z` (`2026-07-10 22:23:29 BST`)
Execution commit: `2e240dc1bf67` with an intentionally unstaged worktree
Acceptance commit: `2cf0760d67bc41eee9875f08a9e13941887727ed`
Acceptance time: `2026-07-11T00:02:51Z` (`2026-07-11 01:02:51 BST`)
Implementation status: complete
Acceptance status: accepted; exact-tip Windows, checkout, and branch gates
passed, and the release owner approved limitations 1 and 3 and accepted the
technical closure of item 4
Checkpoint release status: not ready at A1 acceptance; A5-A7 remained then

### Closed findings

| Finding | Closure evidence |
|---|---|
| Full fallback with every provider disabled | explicit empty provider routing returns `strategy_only`, performs no network call, and never invokes the legacy Semantic Scholar fallback |
| Malformed shared config overwrite | Python, Rust, and Node status/save/wizard paths fail closed; malformed bytes remain unchanged and temporary files are removed |
| Disabled credential scheduling | persisted disabled credentials are excluded from provider plans, searches, and external adapter environments |
| Configured versus active diagnostics | capability mode and search planning use active providers; optional OpenAlex email alone cannot reactivate a disabled provider |
| Path and secret leakage | status, save, wizard, scholarly-search, search-plan, and Rust MCP failures use fixed public errors without local paths or credential values |
| Legacy Node config safety | validates version and shape, rejects alias collisions and prototype keys, compares inspected/opened file identity, rejects unsafe targets, tightens POSIX mode, and uses same-directory fsync plus atomic replacement |
| Rust Windows ownership/DACL | creates the temporary file with owner=current user, a protected single-user DACL, verifies it before writing credentials, and preserves that descriptor through same-volume replacement |
| Year-key compatibility | the executable Lite smoke fixture supplies canonical and legacy year inputs and asserts `from_year == fromYear` and `to_year == toYear` in output |
| No-active-provider semantics | Full and Rust Lite return warning/strategy-only semantics with `no_active_providers`; no provider request is issued |

### Focused validation evidence

| Validation | Result |
|---|---|
| Python A1 and cross-runtime suite | 130 passed, 113 subtests passed |
| Python MCP CLI with `ResourceWarning` promoted to error | 11 passed |
| Rust complete current-host suite | 87 passed |
| Rust `cargo clippy --all-targets -- -D warnings` | pass |
| Rust `cargo check --all-targets` and `cargo fmt --check` | pass |
| Legacy Node MCPB complete suite | 137 passed, 3 Windows-only tests skipped on macOS |
| Capability Contract v2 validator | pass |
| Strict research-standard validator | 6,204 passed, 0 failed, 0 warnings |
| Repository-boundary and added-line secret/local-path scans | pass |
| `git diff --check` | pass |

### Windows exact-tip acceptance evidence

| Evidence | Result |
|---|---|
| Exact pushed commit | `2cf0760d67bc41eee9875f08a9e13941887727ed` |
| CI workflow | run `29130430299`, success; all five jobs passed |
| Checkout Install Check | run `29130430264`, success |
| Windows acceptance job | job `86484631597`, success |
| Release artifact | `qiongli-lite-mcp-windows-x86_64` |
| Acceptance receipt | `passed` at `2026-07-10T23:32:36.6245418Z` |
| Executable SHA-256 | `9cf1f43926f4be63a6b4600f611192e8252cf731632bd410a1b6de6403cf046f` |
| Executable size | `4,335,104` bytes |
| Receipt assertions | two stdio calls, persistence, protected current-user-only ACL, redaction, environment restoration, and temporary cleanup passed |

At the original A1 implementation checkpoint, no file was staged, committed,
pushed, versioned, tagged, published, or copied into a marketplace catalog.
The later acceptance evidence above is bound to the exact pushed commit.

### Explicit 1.x limitations and mandatory acceptance gates

1. The dependency-free legacy Node MCPB cannot create a verifiable user-only
   Windows DACL. Its Windows save and wizard entry points therefore fail closed
   before creating or replacing a config file. Windows configuration writes
   must use the Rust Lite/provider runtime; Node remains read-only for the
   shared file. This prevents Node from replacing a Rust-secured file with one
   that inherits a wider directory ACL.
2. The Windows Rust branch and its native ACL tests cannot execute on the
   current `aarch64-apple-darwin` host. The dedicated Windows CI job executed
   provider-config ACL tests, full Rust tests, clippy, Node fail-closed tests,
   and a built-artifact stdio/DACL smoke successfully on exact pushed commit
   `2cf0760d67bc41eee9875f08a9e13941887727ed`.
3. Rust Win32 paths do not yet opt into extended-length `\\?\` path handling.
   An unusually deep Windows config path can fail closed without writing or
   disclosing credentials. Supporting such paths is deferred to 2.x unless the
   release owner rejects this limitation.
4. The Windows runner created a real junction and verified that the legacy Node
   read path fails closed without touching the target. This technical acceptance
   gate is complete; POSIX target-symlink and cross-platform identity tests also
   pass.

### Release-owner limitation decision

- Status: approved
- Scope: the limitation approval applies to the `v1.19` beta; its technical
  evidence is anchored at accepted candidate commit
  `2cf0760d67bc41eee9875f08a9e13941887727ed`, not the future release tag
- Owner: repository release owner
- Decision: accept limitations 1 and 3 for the final 1.x beta and accept the
  Windows technical closure of item 4
- Decision time: `2026-07-11T00:02:51Z` (`2026-07-11 01:02:51 BST`)
- Decision evidence: reply to the release-owner approval request that explicitly
  listed items 1, 3, and 4: “这一轮审查完成，并继续按照计划进行下一步”

With the exact-tip Windows, checkout, and branch gates green and the owner
decision recorded, A1 is accepted. This decision authorizes A5 preparation; it
does not by itself create a version, tag, publication, or accepted release.

### A2 — Complete and freeze the Stage 1 batches

Task IDs: `RLS-101D`, `CTR-101`

Actions:

1. Validate every new registry entry against the registry schema.
2. Confirm Lite/Full/public declarations match real dispatch.
3. Confirm all aliases, error semantics, redaction rules, side effects, and safe
   smoke calls are represented.
4. Keep `registry.status` as `pilot` and regenerate exact coverage from source.
5. Record the next unmigrated capability batch but do not implement it on 1.x.
6. Freeze golden calls for the configuration and literature-planning batches.
7. Add a machine-readable migration-baseline manifest plan to the 2.x backlog;
   do not invent a second canonical registry during release prep.

Exit criteria:

- validators report no registry/runtime/declaration drift;
- every Stage 1 tool added to the registry has Python and Rust evidence where
  its profile requires both;
- the final 1.x release note states the exact partial coverage.

## A2 Execution Record — Stage 1 Freeze Candidate

Execution time: `2026-07-10T22:04:07Z` (`2026-07-10 23:04:07 BST`)
Execution commit: `2e240dc1bf67` with an intentionally unstaged worktree
Freeze commit: `2cf0760d67bc41eee9875f08a9e13941887727ed`
Freeze time: `2026-07-11T00:02:51Z` (`2026-07-11 01:02:51 BST`)
Implementation status: complete
Freeze status: Stage 1 contract and migration-baseline plan frozen; all Stage 1
source batches are pushed and required Windows, checkout, and branch gates
passed on the exact freeze commit. A8 still owns generation of the normalized
1.x migration baseline and compatibility oracles
Checkpoint release status: not ready at A2 freeze; final release-note
disclosure of the explicitly
partial Contract v2 pilot (`6/23` canonical records and `7/24` public names)
remains an A6 gate

### Contract and baseline-plan evidence

- Capability coverage targets are derived from the union of shipped Full and
  Lite public declarations. Runtime compatibility aliases are validated and
  excluded from the canonical count.
- The derived target remains `23` canonical tools and `24` public names; the
  migrated numerator remains the honest Stage 1 pilot value of `6` canonical
  records and `7` public names.
- Mutation tests reject a missing runtime capability, a synthetic compatibility
  alias that changes only the public total, and stale registry target fields.
- Configuration and literature-planning records retain schema, profile,
  semantic-error, redaction, side-effect, alias, and golden-call evidence.
- Full configuration-wizard reuse reports the provider bound to the active
  session rather than a conflicting provider from a later alias request; a
  mismatched-provider regression covers the reuse path.
- `tooling/migration/qiongli-1x-baseline-plan.json` and its JSON Schema define
  the accepted-tag lineage, 11 inventory domains, five normalization classes,
  Python Full/Rust Lite/Node MCPB capture-only oracles, SHA-256 and package-tree
  evidence, and the read-only `qiongli-testkit` consumer.
- The baseline plan references the existing Capability Contract registry and
  golden fixture. It does not create a second tool registry.

Focused validation at this checkpoint:

| Validation | Result |
|---|---|
| Capability Contract v2 validator | pass; derived target `23/24`, pilot coverage `6/7` |
| Capability Contract v2 tests | 27 passed, 1 local-listener test skipped in the sandbox |
| Migration baseline plan tests | 6 passed |
| Release automation static tests | 44 passed |
| Full wizard, MCP handler, and capability focused suite | 97 passed |
| CI YAML parse and `git diff --check` | pass |

### Pre-commit ownership refresh

At the historical pre-commit ownership checkpoint, the candidate contained 80
intended dirty files: 60 modified tracked files and 20 untracked files. The
staging area was empty, with no deleted or renamed tracked paths. At the A2
freeze commit, all intended paths are committed and the worktree is clean.

The 65 A0 paths retain their original owners. The 15 additional intended files
are assigned as follows:

- Windows/release acceptance:
  `.github/workflows/ci.yml`,
  `tooling/scripts/windows_a1_acceptance.ps1`,
  `packages/qiongli-lite-mcp/src/providers/search.rs`,
  `packages/qiongli-lite-mcp/tests/search_orchestration.rs`,
  `packages/qiongli-literature-mcpb/server/config-wizard.mjs`,
  `packages/qiongli-literature-mcpb/test/tools.test.mjs`,
  `packages/qiongli-literature-mcpb/test/config-enabled-routing.test.mjs`,
  `tests/test_lite_mcp_behavior_contract.py`,
  `tests/test_literature_mcpb_artifact.py`,
  `tests/test_mcp_contract_fixtures.py`, and
  `tooling/scripts/validate_marketplace_install.py`;
- release-resume safety: `tooling/scripts/release_automation.sh`, whose clean
  tree gate now includes untracked files;
- migration freeze planning:
  `tooling/migration/baseline-plan.schema.json`,
  `tooling/migration/qiongli-1x-baseline-plan.json`, and
  `tests/test_migration_baseline_plan.py`.

An empty, generated `packages/qiongli-literature-mcpb/pnpm-lock.yaml` was
excluded and removed from the candidate. The package has no dependencies, CI
uses its existing npm script, and this release does not adopt a new package
manager or lockfile. No external marketplace catalog, generated plugin payload,
host cache, local absolute path, or credential is part of the intended change.

### Commit boundary selected for A4

1. `feat(platform): complete capability contract v2 stage-one batches` for
   contracts, Full/Rust/Node runtime behavior, validators, and conformance
   tests that must stay green together;
2. `fix(release): enforce stage-one and Windows acceptance gates` for CI,
   release-preflight/marketplace gates, release automation tests, and the
   built-artifact Windows receipt script;
3. `docs(platform): define 1.x closeout and Rust-native 2.0 migration` for
   architecture, roadmap, execution plans, and the machine-readable migration
   baseline plan/schema/test.

The migration plan test travels with the third commit because it validates the
planning artifact itself. The freeze condition was satisfied at exact commit
`2cf0760d67bc41eee9875f08a9e13941887727ed`: the source batches were pushed and
the Windows, checkout, and branch gates were green. A2 is frozen at that commit.

### A3 — Run the pre-version regression gate

Task ID: `RLS-101E`

Run focused tests first, then the complete repository gate. At minimum:

```bash
cargo fmt --manifest-path packages/qiongli-lite-mcp/Cargo.toml -- --check
cargo clippy --manifest-path packages/qiongli-lite-mcp/Cargo.toml \
  --all-targets --locked -- -D warnings
cargo test --manifest-path packages/qiongli-lite-mcp/Cargo.toml \
  --all-targets --locked
node --test packages/qiongli-literature-mcpb/test/*.test.mjs
python3 scripts/validate_capability_contract.py
python3 scripts/validate_research_standard.py --strict
python3 -m unittest discover -s tests -v
./scripts/run_beta_smoke.sh
```

Also run the materialized distribution audit, local install acceptance, package
preflight, documentation build, and release automation tests already composed
by `release_ready.sh` and CI.

Exit criteria:

- all blocking checks pass from a clean source commit candidate;
- failures are fixed in their source boundary, not suppressed globally;
- test receipts identify the exact commit and host target.

Execution time: `2026-07-11T00:08:20Z` (`2026-07-11 01:08:20 BST`)
Source commit: `2cf0760d67bc41eee9875f08a9e13941887727ed`, with only this
documentation-only acceptance record unstaged
Host target: `aarch64-apple-darwin`
Command: `./scripts/run_beta_smoke.sh --tier maintainer`
Status: passed

The fresh maintainer tier passed the built-in literature pipeline, doctor,
full-cycle workflow harness, parallel/profile path, and task-run/profile path.
Together with the exact-tip CI and Checkout Install Check evidence recorded in
A1, the A3 pre-version regression gate is complete. A5 must still validate the
coordinated version-preparation diff before A6 begins.

### A4 — Commit and push cohesive source batches

Task ID: `RLS-101F`

Recommended commit boundaries and messages:

1. `feat(mcp-contracts): complete provider configuration batch`
2. `feat(literature): align search planning across runtimes`
3. `fix(provider-config): enforce provider opt-out and safe config handling`
4. `test(platform): extend contract and release conformance coverage`
5. `docs(platform): define Rust-native 2.0 migration`

The actual split must follow dependency and review boundaries. Do not create an
empty commit merely to match this list, and do not rewrite the already accepted
`2e240dc1` pilot commit unless a reviewer explicitly requests it.

After local checks:

1. stage only the intended batch;
2. inspect the staged diff and staged file list;
3. commit using Conventional Commits;
4. push `dev`;
5. wait for required branch checks on the pushed commit.

Exit criteria:

- `dev` and `origin/dev` point at the reviewed source commit;
- working tree is clean before version preparation;
- CI and checkout/install checks pass on that exact commit.

### A5 — Prepare coordinated 1.x versions

Task ID: `RLS-102A`

Target versions:

| Component | Target |
|---|---|
| Python product | `1.19.0b1` |
| workflow / skill metadata | `1.19.0-beta.1` |
| npm prerelease | `1.19.0-beta.1` |
| Git tag | `v1.19.0-beta.1` |
| Rust Lite component | `0.2.0-beta.3` |
| literature MCPB component | `0.2.0-beta.3` |
| Capability Contract v2 | retain its actual preview version and pilot status |

Use the existing product version synchronizer for product-owned fields:

```bash
python3 scripts/sync_versions.py 1.19.0b1
```

Update Lite/MCPB component versions deliberately in their package manifests,
locks, and server-reported metadata. Do not let a recursive JSON version rewrite
change unrelated protocol or contract versions.

Verify:

- Python, workflow, npm, plugin, docs, and generated package versions agree;
- Rust `Cargo.toml`, `Cargo.lock`, executable version, MCPB manifest,
  `package.json`, and server-reported version agree;
- the existing tag/version verifier accepts the product version, and a focused
  component-version assertion verifies that Rust Lite and MCPB both equal the
  exact planned `0.2.0-beta.3` target (the current tag verifier does not check
  those component values);
- no file still presents beta.3 as the installed current version except
  historical release records.

Exit criteria:

- one reviewed version-prep diff contains only expected product/component
  version and generated metadata paths;
- component and product identities are recorded separately and accurately.
- the coordinated product and component version changes are committed as
  `chore(release): prepare 1.19.0b1`, pushed to `dev`, and pass branch CI before
  A6 begins. This explicit commit is required because current readiness tooling
  does not allowlist or stage the Rust Lite/MCPB component-version files.

### A6 — Generate and review final-beta release evidence

Task ID: `RLS-102B`

Create `tooling/release/v1.19.0-beta.1.md` from
`v1.18.0-beta.3`, then review it manually. It must include:

- final planned 1.x beta and Python-line freeze statement;
- Contract v2 pilot version and exact coverage, not a completeness claim;
- configuration and literature-planning behavior;
- security and compatibility closures;
- optional AC1/RC1 guidance status without a 2.x enforcement claim;
- product and component version map;
- supported client surfaces;
- current-host native target and the absence of a generic multi-platform claim;
- known limitations, rollback path, and 1.x maintenance policy;
- validation commands and observed evidence only after they run.

Run the full composed readiness gate without `--allow-dirty`:

```bash
./scripts/release_ready.sh \
  --version 1.19.0b1 \
  --skip-bump \
  --from-tag v1.18.0-beta.3 \
  --maintainer-smoke
```

Exit criteria:

- readiness exits zero;
- generated artifacts identify their true target triple;
- local install and safe MCP launch pass from staged artifacts;
- release note and generated readiness evidence contain no placeholder
  presented as completed fact.

Commit the reviewed release note and any expected readiness metadata, push the
commit, and wait for its required branch checks before A7. Do not rely on
release automation to discover or stage unrelated component-version files.

### A7 — Publish and accept `v1.19.0-beta.1`

Task IDs: `RLS-102C`, `RLS-102D`

Starting from the committed, pushed, and green A6 release-prep commit, publish
through the canonical automation:

```bash
./scripts/release_automation.sh publish \
  --tag v1.19.0-beta.1 \
  --skip-bump \
  --from-tag v1.18.0-beta.3 \
  --resume-after-ready
```

Because A6 already ran full readiness and committed its reviewed evidence,
`--resume-after-ready` is required here. It preserves the exact A6 commit and
prevents a second readiness run from rewriting dynamic duration evidence before
tagging. The automation must gate on the pushed branch commit before creating
the tag.
Verify:

- remote `dev` and release commit identity;
- annotated tag and tag-to-version consistency;
- branch and tag CI success;
- expected registry publish workflows only;
- GitHub prerelease page and release assets;
- artifact checksums, target metadata, download index, and safe launch;
- PyPI and npm prerelease publication and install smoke. Both current tag
  workflows trigger on every `v*` tag, so they are required for this final 1.x
  beta unless the workflows are deliberately changed before tag creation;
- generic marketplace dist refs did not advance for unsupported native targets;
- generated acceptance receipt, followed by manual acceptance and sign-off;
- rollback instructions remain executable.

Exit criteria:

- the automation-generated receipt is reviewed after all workflows finish;
  machine-verified checklist items are marked, remaining manual items are
  completed, Owner and Reviewer sign, and the finalized receipt is committed
  and pushed as a separate post-release acceptance commit;
- `v1.19.0-beta.1` has that completed acceptance receipt;
- no required postflight item remains unchecked without an explicit owner and
  accepted limitation;
- the release can be installed and rolled back from published artifacts.

A7 execution status: **accepted**.

- annotated tag `v1.19.0-beta.1` peels to
  `8d2e99866ce4c4efb8b3b5e0265c0c1f89a36b0f`;
- the branch/tag workflows, published registries, 10 release assets, native
  target identities, isolated installs, safe Rust Lite launch, and read-only
  rollback rehearsal are recorded in
  `tooling/release/acceptance/v1.19.0-beta.1-receipt.md`;
- the completed receipt and owner/reviewer sign-off were finalized on `dev` at
  `ba4517c8dfd5ce8b551c83b129213e689d32cac4`;
- accepted limitations remain explicit: the Full CLI still requires Python,
  the npm asset manager still requires Node, and the published native payloads
  are scoped to `aarch64-apple-darwin`.

### A8 — Cut the 1.x maintenance and migration baseline

Task IDs: `RLS-103A`, `CTR-102`

A7's acceptance precondition is satisfied. A8 now executes in this order:

1. create `release/1.x-python` at the accepted tag;
2. update `docs/maintainer/release-branch-policy.md` and protect the branch as
   critical-fix-only;
3. record the support window and exception policy;
4. generate a baseline inventory for MCP, CLI, skills, tasks, roles, workflows,
   subjects, templates, installers, mutable state, and orchestrator scenarios;
5. normalize machine paths, ports, timestamps, process IDs, and secrets;
6. preserve Python, Rust Lite, and Node oracle fixtures;
7. record artifact checksums and package trees for comparison;
8. commit and push the normalized baseline and branch-policy update on `dev`;
9. verify that no local or remote `2.x` branch already exists;
10. create and push `2.x` from that exact clean baseline commit;
11. wait for the first `2.x` `CI` and `Checkout Install Check` runs to pass;
12. protect `2.x` with pull-request, required-check, deletion, and
    non-fast-forward rules with no bypass, audit the existing `dev` protection,
    then record both ruleset identities or corrective actions;
13. open native migration work only on `2.x` after the branch point and
    protection evidence are recorded.

A8 execution status: **complete**.

- `release/1.x-python` exists locally and remotely at the accepted tag commit
  `8d2e99866ce4c4efb8b3b5e0265c0c1f89a36b0f` and ruleset
  [18797579](https://github.com/jxpeng98/qiongli/rules/18797579) protects it as
  critical-fix-only;
- the accepted baseline and immutable guard landed on `dev` in
  `04a0e49cb455e5a10175c7d0c66949b837b7442c`; the final release-policy fix is
  `70c5bd9e5b098fbb61bd60934e8774b2eef44a01`;
- exact-head `dev` CI run
  [29141373649](https://github.com/jxpeng98/qiongli/actions/runs/29141373649)
  passed 5/5 jobs and checkout run
  [29141373644](https://github.com/jxpeng98/qiongli/actions/runs/29141373644)
  passed 3/3 jobs;
- `2.x` was created from the unchanged `dev` baseline tree, and its auditable
  branch-point record is commit
  `69855fd50413ee6809baf21eb345fc0c55721de3`;
- exact-head `2.x` CI run
  [29142556037](https://github.com/jxpeng98/qiongli/actions/runs/29142556037)
  passed 5/5 jobs and checkout run
  [29142556048](https://github.com/jxpeng98/qiongli/actions/runs/29142556048)
  passed 3/3 jobs;
- active rulesets
  [17189053](https://github.com/jxpeng98/qiongli/rules/17189053) and
  [18800504](https://github.com/jxpeng98/qiongli/rules/18800504) protect `dev`
  and `2.x` with pull-request, required-check, deletion, and non-fast-forward
  rules and no bypass actors;
- the machine-readable source, validation, and protection evidence is recorded
  in `tooling/migration/2x-branch-point.json`.

Current generated A8 evidence:

| Evidence | Recorded result |
|---|---|
| Baseline manifest | `tooling/migration/baselines/v1.19.0-beta.1/manifest.json` |
| Baseline manifest SHA-256 | `77bb7628d43a496c995e4b0a8daf6a624847b62e96948c0461affe89002da131` |
| Accepted tag object / commit | `e68e3af4c879d8e9053124d1aed625bfcddfdbb4` / `8d2e99866ce4c4efb8b3b5e0265c0c1f89a36b0f` |
| Finalized receipt SHA-256 | `a462dc24d94debfb678038e9ed437bdf04dc75476237cc74a9bf06ac366444e9` |
| Baseline corpus SHA-256 | `7fdd92894d88b221180e77ad73677cc158147cc861b17ba0245ea54f0127fbe2` |
| Python Full runtime oracle | 5 captured cases; peeled-tag source tree; 1 accepted gap |
| Rust Lite runtime oracle | 5 captured cases; accepted release binary; 1 accepted gap |
| Node MCPB runtime oracle | 5 captured cases; peeled-tag source tree; 1 accepted gap |
| Asset-backed determinism | two independent accepted-asset directories reproduce byte-identical output |
| Focused A8 tests | 46 passed, including runtime replay and frozen-baseline guard tests |
| Strict research-standard validation | 6,204 passed, 0 failed, 0 warnings |

Offline `verify` validates schema, source identity, release evidence, hashes,
coverage shape, portability, and corpus integrity without launching the three
runtimes. It is not an independent attestation of a simultaneously rewritten
case outcome and manifest. Initial outcome authenticity is established by the
asset-backed recapture above; after the A8 commit, the comparison-base guard
makes the baseline, plan, and schemas immutable on `dev` and `2.x`.

The A8 side-effect recorder compares each runtime sandbox and the declared
accepted source/binary guard roots. Its `writes_outside_sandbox: false` value
means no write was observed in that bounded set; it is not OS-wide filesystem
tracing and does not observe empty-directory or permission-only changes. The
2.x testkit must either narrow that field name/contract or replace it with an
OS sandbox or tracing-backed assertion before using it as a security claim.

Exit criteria:

- the baseline can be checked out and tested independently;
- 2.x tests can consume frozen results without requiring Python or Node in the
  production binary;
- feature work cannot silently move the 1.x oracle;
- `release/1.x-python` points at the accepted product tag, while `2.x` points at
  the clean post-release baseline commit that includes the finalized receipt
  and migration inventory.

## Phase B — `v2.0.0-alpha.1` Native Bootstrap

### B0 — Architecture decision records

Task IDs: `ARC-201A` through `ARC-201G`

Record and review these decisions before scaffolding implementation:

1. one product executable versus multiple thin executable frontends;
2. pure-Rust desktop toolkit and accessibility prototype;
3. `AgentBackend` protocol, direct API boundary, optional CLI compatibility,
   and the separate native `ToolHost`/`AgentExecutionPolicy` boundary;
4. mutable-state schemas, keychain/fallback storage, migration and rollback;
5. embedded resource-pack format, signature and reproducibility;
6. declarative `InstallPlan`, managed markers, client trust and activation;
7. release channel and artifact identity model for alpha, beta, stable, profile,
   OS, architecture, and installer kind.

Recommended defaults:

- CLI and GUI share service crates; the full installer exposes one `qiongli`
  product command and desktop entry;
- use a pure-Rust UI toolkit selected by an accessibility and packaging spike;
- direct model APIs are first-class, the native ToolHost owns sandboxed tool
  execution, and external CLIs are optional adapters;
- state migrations are forward-only, backed up, atomic, and reversible by
  restoring the pre-migration snapshot;
- canonical content remains in `content/` and is compiled into a deterministic
  resource pack;
- host caches are never written directly.

Exit criteria:

- each ADR lists context, decision, alternatives, consequences, security,
  rollback, and acceptance test;
- no unresolved architecture choice blocks a vertical slice.

B0 execution status: **accepted**.

- ADR 0201 selects one canonical, multi-mode `qiongli` executable per target;
- ADR 0202 selects native `egui`/`eframe` with AccessKit and a blocking
  accessibility and packaging prototype;
- ADR 0203 separates typed `AgentBackend` implementations from the native,
  policy-enforced `ToolHost` and `AgentExecutionPolicy`;
- ADR 0204 fixes versioned state, copy-on-migrate backup/restore, atomic writes,
  secret references, and an OS-keychain-first storage facade;
- ADR 0205 fixes the deterministic embedded resource-pack and signature model;
- ADR 0206 fixes a typed transactional `InstallPlan`, managed ownership, and
  host-controlled trust/activation boundaries without cache mutation;
- ADR 0207 fixes independent alpha/beta/stable channels and complete target
  artifact identity;
- `tooling/architecture/arc-201-decisions.json` records the complete set, and
  `scripts/validate_arc_201_adrs.py` enforces its decision-grade sections in CI;
- `tooling/migration/2x-branch-point.schema.json` and
  `scripts/validate_2x_branch_point.py` bind the handoff record to its local Git,
  baseline hash, run, and ruleset evidence;
- after this one-time bootstrap, `scripts/check_frozen_2x_architecture_baseline.py`
  freezes ADR 0201-0207, their inventory, and the handoff evidence. Future
  changes require a new superseding ADR instead of synchronized rewrites.

### B1 — Teach release tooling about native alpha releases

Task ID: `REL-201`

Current tooling accepts stable and beta syntax only. Before any 2.x alpha tag:

- parse `2.0.0-alpha.1`, Cargo prerelease versions, and any required channel
  representation without confusing alpha with stable;
- make prerelease detection include alpha in readiness, preflight, automation,
  postflight, notes, version verification, download metadata, and validators;
- consume the native product version source created with the B2 Rust workspace;
  parser work may begin first, but B1 cannot exit before that source exists;
- do not require PyPI or npm publication for a native release;
- keep frozen 1.x PyPI/npm channels distinct from native GitHub/installer and
  plugin channels;
- ensure `qiongli-next` metadata can represent the 2.x alpha line;
- test stable, beta, alpha, invalid, and cross-component version cases;
- document rollback and channel promotion from alpha to beta.

Primary files to audit include:

- `tooling/scripts/sync_versions.py`;
- `tooling/scripts/release_ready.sh`;
- `tooling/scripts/release_preflight.sh`;
- `tooling/scripts/release_automation.sh`;
- `tooling/scripts/release_postflight.sh`;
- `tooling/scripts/verify_release_tag_version.sh`;
- release-note, artifact, download-index, and materialization builders;
- release and tag workflows under `.github/workflows/`.

Exit criteria:

- a dry-run `v2.0.0-alpha.1` release produces correct notes, manifest, artifact
  identity, channel metadata, and rollback plan without publishing;
- existing 1.x stable and beta parser tests remain green.

B1 execution status: **implemented as a non-publishing release contract**.

- `packages/qiongli-native/Cargo.toml` is the sole native product version and
  channel source; Cargo.lock is verified as a derived binding;
- the shared classifier distinguishes `legacy-1x` from `native-2x` and the
  canonical `alpha`, `beta`, and `stable` channels;
- native sync never rewrites the frozen Python/npm/content identities;
- `v2.*` tags cannot enter the 1.x PyPI/npm jobs;
- native preflight writes a deterministic plan, notes, planned-only target
  identity, channel metadata, and rollback plan only to an explicit directory
  outside the checkout;
- native `publish` and production `post` remain blocked. The dry-run records
  `publication_performed=false` and `publication_allowed=false`; it is not an
  accepted alpha artifact or permission to create `v2.0.0-alpha.1`.
- `qiongli-next` metadata represents native alpha/beta as planned targets only;
  its active builder allowlist remains legacy 1.x beta until native versioned
  content and package acceptance exist.

### B2 — Scaffold the native workspace and gates

Task IDs: `FND-201`

Create `packages/qiongli-native/` as the workspace defined by the roadmap.
Initial members should be limited to the crates required by the alpha.1 vertical
slice:

- `qiongli-contracts`;
- `qiongli-content`;
- `qiongli-config`;
- `qiongli-provider-kernel`;
- `qiongli-mcp`;
- `qiongli-platform`;
- `qiongli-installer`;
- `qiongli-updater`;
- `qiongli-ui`;
- `qiongli-testkit`;
- one `apps/qiongli` product application for CLI, UI, MCP, and internal modes.

Add empty orchestrator and agent crates only when their public traits are ready;
avoid placeholder APIs that become accidental contracts.

Execution rebaseline: FND-201/B2a intentionally landed a single dependency-free
application first. That temporary topology is not a migration constraint. As
FND-202 begins, maintainers may add reviewed service crates, locked
dependencies, build scripts, platform APIs, and typed process adapters through
normal implementation and review. The installed product must still start and
run without requiring Python, Node, or a Rust toolchain on the user machine.

Gates from the first commit:

```bash
cargo fmt --manifest-path packages/qiongli-native/Cargo.toml --all -- --check
cargo clippy --manifest-path packages/qiongli-native/Cargo.toml \
  --workspace --all-targets --all-features --locked -- -D warnings
cargo test --manifest-path packages/qiongli-native/Cargo.toml \
  --workspace --all-targets --all-features --locked
```

These are normal component gates, not a separate governance program. Review
locked dependencies and relevant license, secret, path-safety, and unsafe-code
risks when the changed component introduces them. Use Linux, Windows, and macOS
runners when the change or release claim touches those platforms; ordinary
component work may use focused checks first.

Exit criteria:

- changed workspace components compile and pass their focused tests;
- no production crate depends on Python or Node launch behavior;
- crate dependency direction is documented and acyclic.

### B3 — Verify and load frozen contracts and content

Task IDs: `CTR-201`, `SEC-201A-D`, `CTR-202A-K`, `FND-202A-F`

Dependency order: `CTR-201` is the shared entry gate. Under the declared
program DAG, `CTR-202` Contract v2 completion and `FND-202` resource-pack
implementation are separate successors and may proceed in parallel only after
CTR-201 closes. Neither successor is completion evidence for the other.

B3 prefers reviewable child slices, but slice size, WIP, and task-ID granularity
are planning guidance rather than merge gates. A maintainer may combine
coherent contract, runtime, and test changes when the combined behavior is
easier to implement and verify. Focused checks run during development; broader
platform and release checks run when the relevant artifact or claim is being
prepared.

Actions:

1. load and verify the frozen A8 1.x baseline manifest, accepted-tag lineage,
   checksums, package trees, and oracle inventory; do not regenerate or move
   the accepted oracle during B3;
2. retain merged CTR-201E as the accepted-source Full CLI runtime inventory and
   close the remaining accepted-source orchestrator-runtime requirement through
   CTR-201F, using bounded fixtures or explicit dispositions for every required
   matrix cell;
3. keep Contract v2 anchored to the frozen 23 canonical tools and 24 public
   names, and fix contract/runtime/test gaps together when they are one coherent
   behavior;
4. implement the deterministic resource pack through FND-202A-F: manifest,
   collector,
   deterministic writer, verifier/loader, atomic materializer, then drift/Tier
   1 aggregation;
5. verify path traversal, symlink, duplicate ID, invalid schema, and resource
   size limits;
6. materialize the same normalized tree as 1.x for the alpha.1 supported core
   profile;
7. fail the build when generated resources drift from source.

B3 execution status: **in progress**.

- the CTR-201A derived-inventory slice is merged and validation-backed;
- CTR-201B's static Python Full CLI inventory captures 46 canonical and 49
  public command paths, five console entrypoints, 164 non-help actions, and 27
  defaults that resolve to the current working directory;
- CTR-201C captures the accepted-source `DECLARED/STATIC` orchestrator control
  contract and compatibility boundary: 13 stages, 76 tasks, 104 required
  dependency edges, three runtime agent IDs, nine functional agent IDs, 82
  routing skill IDs, 11 logical MCP capabilities, four quality gates, five
  built-in profiles, and the B1/H3 team and worker configurations. The 82 skill
  values are unique routing IDs rather than an installable-skill count; the 11
  MCP values are logical capability IDs rather than public tool names;
- CTR-201B help coverage is authored parser metadata only. CTR-201E separately
  captures formatted parser output and bounded handler cases and binds every
  unexecuted handler dimension to an inventory-only `LEG-201` disposition;
  Full handler runtime parity remains unclaimed;
- CTR-201C alone does not prove that agents run, solo/triad runtime parity,
  state or resume behavior, concurrency, failure or cancellation semantics, or
  quality-gate semantic execution. The accepted handler makes `doctor` part of
  an advisory route sequence; `_tool_task_run(run_agents=true)` does not call
  or enforce it. CTR-201C also does not implement plugin or Marketplace
  behavior or a Rust orchestrator;
- CTR-201D partitions all 377 accepted canonical content files into 12
  disjoint roots and 11 logical resource kinds. It freezes the `skill-only`,
  `marketplace-lite` (`lite` alias), and `full` source projections and the
  three corresponding accepted-source materialized skill subtrees (178, 342,
  and 343 files respectively);
- CTR-201D does not establish published archive/plugin-wrapper parity, host
  activation, or Rust resource-pack behavior because A8 contains no
  archive-member tree inventory;
- CTR-201E implements the accepted-source Full CLI runtime-inventory-freeze
  slice. Its gate covers the CTR-201B-fixed 49 public command
  paths and five console entrypoints across formatted help, stdout/stderr,
  JSON, exit codes, normalized error classes, dry-run and side-effect behavior,
  zero-argument behavior, aliases, and legacy npm dispatch. Unexecuted handler
  scenarios are bound to `CTR-201E-D001`/`D002` and npm handler parity to
  `D003`; all remain `LEG-201` work rather than executed parity evidence;
- CTR-201E is merged through protected PR #57 and closes only the CLI runtime
  inventory slice; it does not establish Full handler or cross-platform runtime
  parity;
- CTR-201F closes the accepted-source orchestrator runtime inventory with
  44 normalized deterministic cases (one A8 plus 43 bounded), six behavior
  dimensions, and six machine-bound dispositions. It does not launch real
  Codex, Claude, or Antigravity agents and does not claim
  strict single-agent solo enforcement, public cancellation, real session
  resume, native worker dispatch, semantic quality-gate execution, a Rust
  orchestrator, or cross-platform runtime parity;
- CTR-201 has reached its source-oracle exit gate: every required MCP, CLI,
  content, and orchestrator inventory gap is captured or explicitly
  dispositioned. FND-202A now provides the native resource-pack contract;
- CTR-201F canonical re-extraction runs with Python 3.12 in the Ubuntu full
  tier. Windows and macOS validate the checked portable artifact and master
  binding only;
- published archive/plugin-wrapper parity remains an unassigned downstream
  governance boundary rather than a parent exit gate;
- CTR-202 is integrated through PR #59 with exact 23/24 coverage and green
  exact-head CI;
- FND-202A and FND-202B are integrated in `qiongli-content` through PR #59 and
  PR #60 respectively. FND-202C is integrated through PR #61. FND-202D is the
  active branch candidate; FND-202E-F remains a separate successor lane.

Exit criteria:

- the CTR-201 master ledger binds every required child corpus, contains no
  unclassified required capture gap, and keeps unsafe or inapplicable behavior
  behind an explicit approved disposition rather than an omission;
- Contract v2 reaches the CTR-202 exact `23/23` canonical and `24/24` public
  profile-closure gate independently of the source freeze, without a mutable
  runtime declaration silently shrinking the target;
- two clean builds from the same commit produce the same resource hash;
- alpha.1 can list and materialize its embedded skills without source checkout;
- canonical files are not duplicated as hand-maintained Rust constants.

### B4 — Implement versioned config and safe 1.x import

Task IDs: `CFG-201`, first slice of `CFG-202`

Alpha.1 config scope:

- config-home resolution;
- public settings and profile selection;
- research-provider settings;
- model-backend settings;
- OS keychain secret references with a documented secure fallback;
- installed integration and managed-file markers;
- schema version, backup, atomic write, permissions, redacted status;
- preview and import of 1.x provider configuration;
- corruption, unsupported-version, and rollback behavior.

State coexistence is explicit:

- legacy 1.x global files remain read-only under `QIONGLI_CONFIG_HOME`;
- 2.x global config, receipts, and markers write under
  `QIONGLI_CONFIG_HOME/v2/`; changing that root requires a superseding state
  migration ADR;
- legacy project files under `<project>/.qiongli/` remain read-only during
  preview, while 2.x writes under `<project>/.qiongli/v2/`;
- alpha dual-reads legacy state only as a fallback and writes only v2 state;
- approved secrets are copied into the OS keychain, verified, and referenced
  from v2 state; legacy credential files are retained until a separate cleanup
  plan is accepted.

Do not migrate project guidance or experience state in alpha.1 unless their
versioned schemas and rollback fixtures are complete. Detect them and report
them as pending instead of writing them opportunistically.

Exit criteria:

- import is previewable, idempotent, and leaves source untouched on failure;
- config status never emits secrets or private paths unnecessarily;
- Windows ownership/DACL and Unix owner-only permission evidence exists.

### B5 — Extract the existing Rust Lite vertical slice

Task IDs: `PRV-201`, `MCP-201`

Move reusable behavior from `packages/qiongli-lite-mcp/` into the native
provider and MCP crates without changing its public behavior:

- JSON-RPC and stdio framing;
- tool definitions and dispatch;
- provider configuration and setup;
- five-provider search and search planning;
- evidence export and Zotero bounded behavior;
- Lite preview policy and semantic errors.

The old package remains a compatibility binary that depends on the extracted
crates. Run old and new binaries against the same golden calls.

Exit criteria:

- old Lite, native `qiongli mcp serve --profile lite`, and required Full
  projections pass equivalent contract fixtures;
- Lite startup requires a signed artifact/receipt grant and cannot request a
  profile above that grant even when it reuses the canonical product binary;
- production process trees contain no Python or Node subprocess;
- no Lite permission boundary expands accidentally.

### B6 — Implement the shared CLI and desktop shell

Task IDs: `UI-201`, CLI slice of `FND-201`

Required alpha.1 commands:

```text
qiongli version
qiongli ui
qiongli skills list|materialize
qiongli mcp serve --profile lite
qiongli config status|set|import
qiongli integrations list|plan|apply|remove
qiongli doctor
qiongli update check
```

Required alpha.1 desktop views:

- welcome and install profile;
- system and version status;
- skills inventory and materialization;
- MCP status and Lite launch test;
- provider and secret configuration;
- local integrations with preview/apply/verify/remove;
- doctor results;
- update and rollback status, even if alpha.1 update apply remains disabled.

The UI calls the same typed services as the CLI. Every mutating UI action shows
the generated plan before apply.

Exit criteria:

- CLI JSON output is stable and tested;
- GUI and CLI produce the same install/config plans;
- keyboard-only onboarding and error recovery pass the toolkit prototype gate.

### B7 — Implement the first transactional local integration

Task IDs: `PLT-201`, first slice of `PLT-202`, `INT-201`, `INT-202`

Alpha.1 supports one current-host Codex path and one current-host Claude Code
path as a vertical slice:

- discover supported host locations without assuming a shell environment;
- generate one `InstallPlan` containing source, target, managed marker, conflict,
  backup, activation instruction, verification, and rollback operations;
- install the correct native binary/resource payload for the current target;
- install and verify the target/profile-specific signed launch grant bound to
  the canonical binary, resource pack, and integration scope;
- register Codex through documented personal/repo marketplace source metadata;
- register Claude Code through a personal skills-directory plugin and validate
  marketplace form separately;
- never write either host's plugin cache directly;
- preserve unmanaged files and refuse ambiguous overwrite;
- report host-controlled install/enable/reload steps distinctly from file
  placement success.

Claude Desktop is discovery/status only in alpha.1 unless a real direct-plugin
activation receipt is completed in time. Cloud/web surfaces are marked
remote-only.

Exit criteria:

- clean install, duplicate install, unmanaged conflict, repair, remove, and
  rollback pass in isolated home directories;
- real local Codex and Claude Code clients discover the installed content and
  complete one safe MCP call on the advertised current host;
- the product never reports file placement alone as active client integration.

### B8 — Build alpha.1 clean-machine and release evidence

Task IDs: first slices of `QAT-201`, `PKG-201`, `PKG-202`, and `UPD-201`

Build the current-host alpha artifact with explicit OS and architecture. The
alpha may be narrow, but it may not be generic or ambiguous.

Acceptance environment must not contain usable:

- `python` or `python3`;
- `node`, `npm`, or `npx`;
- `uv` or `pip`;
- `rustc`, `cargo`, or `cargo run`.

Verify:

- installer start and uninstall;
- GUI and CLI start;
- embedded skills listing and materialization;
- config import preview and apply;
- Lite MCP initialize, tools/list, and safe calls;
- Codex and Claude local registration and verification;
- doctor output and redaction;
- process tree and production payload forbidden-runtime scan;
- checksum, resource-pack and platform/artifact signature verification;
- an SBOM and build provenance bound to the exact artifact identity;
- failed update/installation rollback;
- no write outside the generated plan.

Exit criteria:

- all alpha.1 vertical-slice acceptance passes on the advertised target;
- release notes state missing Full, orchestrator, target, desktop, and cloud
  capabilities explicitly;
- no artifact or marketplace ref claims untested targets.

### B9 — Publish and learn from `v2.0.0-alpha.1`

Task ID: `RLS-201`

Publish only after B0-B8 pass. The release must:

- use the new native alpha release path, not the frozen Python release path;
- publish target-specific native assets and a machine-readable artifact index;
- publish and verify checksums, signatures, resource-pack identity, SBOM, and
  provenance for every advertised native artifact;
- keep PyPI/npm 1.x channels frozen unless a separately reviewed native
  downloader shim is intentionally released;
- update only prerelease plugin metadata that can honestly start on the
  advertised target;
- include install, import, uninstall, rollback, known limitations, and support
  instructions;
- collect opt-in issue reports without enabling default remote telemetry.

Exit criteria:

- postflight validates tag, assets, checksums, signatures, resource-pack
  identity, SBOM, provenance, target identity, launch, and rollback;
- alpha feedback is triaged into M2 without reopening Python feature work.

## Dependency And Parallel-Execution Plan

The final 1.x beta is mostly sequential because every later step depends on one
accepted baseline:

```text
A0 -> A1 -> A2 -> A3 -> A4 -> A5 -> A6 -> A7 -> A8
```

After A8 creates `2.x`, native work on that branch can use four parallel lanes:

```text
Lane 1: ADRs -> contracts/content -> config/state
Lane 2: workspace -> provider kernel -> MCP
Lane 3: workspace -> CLI/UI -> integration manager
Lane 4: alpha release tooling -> CI/native packaging -> clean-machine evidence

Join: config + MCP + UI + integrations + packaging -> alpha.1 acceptance
```

Recommended ownership boundaries:

| Lane | Owns | Must not own |
|---|---|---|
| Contract/data | schemas, resource pack, migration fixtures | UI behavior or release publishing |
| Runtime | providers, MCP, domain services | client-specific file placement |
| Product | CLI, UI, platform and installer services | duplicate contract or provider logic |
| Release/quality | CI, target builds, signing, evidence, zero-runtime audit | bypassing failing runtime gates |

## Pull Request And Commit Sequence

Keep the following units independently reviewable:

1. final 1.x runtime/security batches;
2. final 1.x release preparation and acceptance receipt;
3. 2.x ADRs plus alpha release-format support;
4. native workspace, contract loader, and testkit;
5. resource pack and config migration;
6. Rust Lite extraction and compatibility binary;
7. CLI/UI shell;
8. declarative install plan and current-host integrations;
9. clean-machine alpha packaging and release prep.

Do not combine the final 1.x runtime changes with the first Rust workspace
scaffold. The accepted 1.x tag must remain an unambiguous oracle boundary.

## Go/No-Go Gates

### Go to final 1.x publish only if

- the working tree is clean except expected release-prep paths;
- the pushed commit and tested commit are identical;
- current P1/P2 findings are closed or an explicit non-security limitation is
  accepted;
- full release readiness passes;
- versions, target identity, notes, and artifacts agree;
- rollback and marketplace-scope rules are verified.

### Go to Rust implementation only if

- the final 1.x beta is accepted;
- the maintenance branch and normalized baseline exist;
- `2.x` was created and pushed from the exact clean post-A8 baseline commit;
- the architecture decisions have owners and acceptance gates;
- no active 1.x feature work is still moving the oracle.

### Go to alpha.1 publish only if

- the native release path understands alpha independently of Python publishing;
- the vertical slice runs on a clean advertised target;
- CLI, GUI, skills, config, Lite MCP, doctor, and two local host registrations
  operate without a language runtime;
- every advertised artifact passes checksum, signature, resource-pack identity,
  SBOM, provenance, and target-native launch verification;
- rollback and redaction evidence pass;
- release claims match the narrow tested scope.

### Do not promote to beta until

- the complete beta-entry gates in the 2.x roadmap pass, including Full MCP,
  domain runtime, agents, orchestrator, state migration, Tier 1 matrix, signed
  packages, and removal of production Python/Node requirements.

## Immediate Next Actions

A0-A8, B0, and the non-publishing B1 release contract are complete, the B2
native workspace scaffold and repository gates are implemented, and B3 has
closed CTR-201A-F as the accepted-source source-oracle inventory. CTR-201E is
merged; CTR-201F provides the final orchestrator-runtime inventory closure.
CTR-202 and FND-202A are integrated through PR #59, and FND-202B is integrated
through PR #60. FND-202C is integrated through PR #61. FND-202D is the active
bounded verifier/loader branch candidate. Continue in this order:

1. finish FND-202D bounded whole-core, manifest, content-root, entry, path, and
   profile verification with in-memory direct loading;
2. run protected-branch exact-head CI and fix only concrete regressions;
3. after FND-202D integration, continue FND-202E atomic materialization only
   into a temporary or explicitly approved target;
4. keep CTR and FND evidence independent even when implementation proceeds in
   parallel;
5. retain published archive/plugin-wrapper parity as downstream product work;
6. keep native publication blocked while PKG-201/PKG-202, UPD-201, target-native
   acceptance, and the remaining public release gates are incomplete;
7. preserve protected-branch pull requests and run the tests relevant to each
   reviewable change, expanding to exact-head platform CI for release-facing
   work.

## Phase Completion Definition

This execution plan is complete when:

- `v1.19.0-beta.1` is published and accepted as the frozen Python-led baseline;
- `release/1.x-python` and normalized migration oracle evidence exist;
- `2.x` exists at the recorded clean post-A8 baseline and owns all subsequent
  native implementation;
- no normal feature work remains on the Python/Node line;
- `packages/qiongli-native/` implements the alpha.1 vertical slice;
- a clean user machine can install and use GUI, CLI, embedded skills, Lite MCP,
  provider config, doctor, and validated local Codex/Claude integrations without
  Rust, Python, or Node.js;
- `v2.0.0-alpha.1` is published with truthful target and capability scope;
- the M2 backlog is populated from measured parity gaps rather than assumptions.
