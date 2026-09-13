# Qiongli R2D Orchestration Preview And Dispatch Design

Status: approved for execution

Date: July 14, 2026

Scope: Marketplace Lite route/task-plan previews and typed Lite handler
dispatch in the shared Rust runtime

Branch: `feat/2x-native-alpha1`

PR: rolling Draft PR #63

## Decision

Move the accepted Marketplace Lite `qiongli_orchestrator_route` and
`qiongli_task_plan` behavior into `qiongli-runtime`. Keep both operations pure:
they validate bounded input and return advisory previews without launching an
agent, process, shell, network request, or filesystem write.

Add a domain-typed dispatch projection for every accepted Lite tool identity.
The projection maps one already resolved `LiteToolId` into a config,
literature, Zotero, or orchestration handler identity. The orchestration target
is executable by the shared preview dispatcher in this batch; the other domain
targets continue to compose their already shared services through the old Lite
adapter until the canonical MCP server is built.

This batch does not expose MCP mode in the canonical `qiongli` executable.

## Preview Contract

Both outputs retain the accepted Contract v2 Marketplace Lite invariants:

- `mode` is `preview` and `preview_only` is `true`;
- `runtime_profile` is `marketplace_lite`;
- agent, shell, and project-write permissions are all `false`;
- `recommended_runtime` remains `full_cli_for_execution`; and
- `upgrade` retains the accepted 1.x Full-runtime compatibility recommendation.

The `qiongli mcp serve --transport stdio` value is compatibility guidance for
the frozen 1.x Full runtime. It is not native 2.x capability discovery and must
not cause the canonical binary, roadmap, or PR to claim that native MCP mode is
available. A later vertical slice must replace capability assumptions with
binary-level initialize, tools/list, and tools/call evidence.

Route previews preserve the caller's bounded request text and normalize an
omitted platform to `unknown`. Task-plan previews preserve the current
compatibility behavior of trimming the three required fields before returning
them.

## Input Boundaries

| Field | Accepted value | Bound |
|---|---|---:|
| route `request` | required, nonblank UTF-8 string | 4,096 bytes |
| route `platform` | optional Contract v2 enum | fixed six values |
| task `task_id` | required, nonblank UTF-8 string after trim | 256 bytes |
| task `paper_type` | required, nonblank UTF-8 string after trim | 256 bytes |
| task `topic` | required, nonblank UTF-8 string after trim | 4,096 bytes |

Arguments must be objects and unknown fields fail closed. Bounds are enforced
inside `qiongli-runtime`, not only by an MCP envelope. Validation errors are
typed and render static messages without echoing request, topic, platform, or
unknown-key content.

## Typed Dispatch Boundary

`LiteToolId` remains the canonical public-name resolver. R2D adds one exhaustive
`LiteDispatchTarget` projection with domain-specific handler enums:

- config: status, save-provider, and configure-provider;
- literature: status, search-plan, search, and evidence export;
- Zotero: status and in-memory import-file export; and
- orchestration: route preview and task-plan preview.

The public `qiongli_open_config_wizard` alias still resolves to the single
configure-provider identity before dispatch. No string matching occurs after
resolution. Adding a new canonical identity will therefore require an explicit
dispatch decision at compile time.

`dispatch_lite_orchestration` accepts only the typed orchestration handler and
returns a typed route or task-plan preview. It cannot dispatch Full task-run,
doctor, agents, or arbitrary commands.

## Ownership Boundary

`qiongli-runtime` owns:

- route/task argument parsing, compatibility normalization, and bounds;
- platform validation and defaulting;
- immutable safety flags and deterministic preview construction;
- typed, sanitized orchestration errors;
- the exhaustive Lite domain-handler projection; and
- typed dispatch of the two pure orchestration previews.

`qiongli-lite-mcp` owns only:

- the JSON-RPC result/error envelope;
- existing compatibility config and wizard adapters;
- mapping the remaining shared provider/evidence/Zotero services into that
  envelope; and
- defense-in-depth output redaction.

The old `orchestrator::preview` module becomes a shared-runtime re-export so
existing Rust call sites retain their public module path without keeping a
second implementation.

## Privacy And Side Effects

Contract v2 marks route requests, task topics, and upgrade projections as
profile-sensitive. The runtime returns those fields only because they are the
documented tool result; it does not log, persist, inspect as a path, or place
them in error text. Tests use canaries to prove invalid input is not echoed.

The implementation adds no process API, shell API, filesystem API, network
client, environment lookup, clock, random source, or agent backend. Preview
construction is deterministic for a given input.

## Verification

Direct runtime tests cover every dispatch identity, alias behavior, accepted
platform, omitted-platform default, output safety flags, compatibility upgrade
projection, trimming behavior, missing/type/unknown/empty input, UTF-8 byte
bounds, and canary-free errors.

Focused old Lite tests cover both public tool calls and the compatibility
module path. The Linux compatibility job adds `orchestrator_preview` to its
explicit test list. Native format, locked workspace check, strict Clippy,
workspace tests, and Windows MSVC cross-target check/Clippy remain required.
Python and Node suites remain outside this migration batch.

## Nonclaims

R2D does not implement or claim:

- canonical binary MCP initialize, tools/list, tools/call, or stdio serve mode;
- Full task execution, agents, ToolHost, shell commands, or project writes;
- provider secret mutation, a native configuration wizard, or production
  secure-store availability;
- native capability discovery or automatic 1.x-to-2.x command rewriting;
- UI, host installation, marketplace activation, packaging, or release
  readiness; or
- completion of R2.

## Acceptance Criteria

R2D is complete only when:

1. direct runtime calls own and enforce both preview contracts;
2. all accepted Lite identities have one exhaustive domain-typed target;
3. the orchestration dispatcher accepts only its two typed preview handlers;
4. invalid or oversized input fails with static, canary-free errors;
5. old Lite route/task calls delegate to the shared implementation;
6. no duplicate Rust preview kernel remains in the compatibility package;
7. focused compatibility and all required native/Windows gates pass without
   Python, Node, live providers, or agent execution; and
8. exact-head GitHub jobs pass before the roadmap records completion.

## Approval Record

The user instructed continuation under the accepted accelerated roadmap on
July 14, 2026. This authorizes R2D on the existing rolling branch and Draft PR
without expanding public product claims.
