# Qiongli PLT-321 App Full MCP Acceptance

Status: accepted at Slice tier

Date: August 23, 2026

Target branch: `2.x`

Pull request: `#141`

Publication allowed: false

## Exact identities

| Identity | Accepted value |
|---|---|
| Product source | `670478ccf7c5ed73b264128e1c790c58564117c4` |
| Evaluation Truth | run `32664187326`: success |
| Native CI | run `32664187366`: success |
| Replacement-path dependency | PLT-320 source `f3c2c0edea04479c423ba3801f2d835c20d8980a`; Native CI run `32653636419`: success |

The Native CI run passed the change-boundary gate, R2 Lite compatibility, and
the Linux, macOS, and Windows native foundation jobs. Candidate, packaged
product, non-publishing package, and promotion jobs were skipped by the
ordinary Slice boundary.

## Accepted App path

The accepted source reuses the existing native owners for this path:

`App integration setup/status/recovery -> shared Full MCP server -> bounded self-test`

The versioned App API now exposes typed run, poll, and cancel intents and one
strict Full result. The App enables the critical workflow after fresh native
integration status is connected, renders embedded Full MCP health separately,
and cannot promote stale or drifted Host evidence when the self-test passes.

The self-test uses the same `FullMcpServer` constructor as `qiongli mcp full`.
It validates the exact ordered 32-tool union: 14 Lite tools, 9 Full project
tools, and 9 Full Host-orchestration control tools. It then calls
`qiongli_orchestrator_route` and requires `route=orchestrator_mcp`,
`requires_full_runtime=true`, and no Lite `upgrade` or `preview_only` result.

## Focused evidence

| Gate | Result |
|---|---|
| Full MCP lifecycle | exact registry/route, no credential reads, cancel, and timeout: 3 passed |
| Native unit slice | 184 passed |
| Native stdio | 7 passed, including Full-only routing without Lite upgrade |
| Native UI | 33 passed |
| App API v19 | 32 passed, including wrong-profile and reordered-check rejection |
| Desktop | 249 passed; Svelte check 0 errors and 0 warnings; production build passed |
| Setup/status/recovery | selection, unsupported client, fresh probes, drift/reconciliation, fixed Host plans, and partial failure regressions passed |
| Roadmap contract | generator current; 7 unit tests passed |

The Full self-test remained offline, bounded, cancellable, credential-free, and
non-mutating. Normal Host profiles, credentials, prompts, responses, projects,
and user Zotero libraries were not inputs.

## Nonclaims

This Slice does not accept Graph v1 (`PLT-322`), candidate or packaged
artifacts, live authenticated Hosts, an authenticated model session, real user
projects or Zotero data, signing, notarization, promotion, publication, or
release authorization. PR `#141` remains stacked on unmerged dependency PR
`#140`; merging and release decisions remain separate maintainer actions.
