# Qiongli PLT-320 Replacement Vertical Acceptance

Status: accepted at Slice tier

Date: August 23, 2026

Target branch: `2.x`

Pull request: `#140`

Publication allowed: false

## Exact identities

| Identity | Accepted value |
|---|---|
| Product source | `f3c2c0edea04479c423ba3801f2d835c20d8980a` |
| Evaluation Truth | run `32653636405`: success |
| Native CI | run `32653636419`: success |
| Governance dependency | merge source `41accaf0407510c971c596fa174f6f3527e03b30`; Native CI run `32651745050`: success |

The Native CI run passed the change-boundary gate, R2 Lite compatibility, and
the Linux, macOS, and Windows native foundation jobs. Candidate, packaged
product, non-publishing package, and promotion jobs were skipped by the
ordinary Slice boundary.

## Accepted replacement path

The accepted source reuses the existing native owners for this path:

`CLI -> Codex/Claude Plugin and Skills -> Lite/Full MCP -> Zotero`

The copied native binary completed Zotero discovery, search, default dry-run
upsert, malformed-receipt rejection, changed-plan rejection, and an explicit
receipt-bound apply against an isolated loopback fixture. The test asserts the
exact Connector and Companion request order. Full-profile routing continued to
return the Full orchestration contract rather than a Lite upgrade result.

No production API, schema, runtime owner, dependency, canonical content source,
or App-owned product logic changed. The only product-path change extends the
existing copied-binary stdio regression.

## Focused evidence

| Gate | Result |
|---|---|
| Zotero public MCP journey | search, preview, malformed receipt, changed plan, and approved apply passed |
| Full MCP routing | Full orchestration result passed without Lite profile drift |
| Official Host plans | fixed argv, state, and target-order regression passed |
| Codex | isolated real `codex-cli 0.147.0` install, enable, cache observation, empty-`PATH` launch, remove, and absence checks passed |
| Claude Code | isolated real `2.1.237` strict validation, local Plugin forms, Skills/MCP launch, cache observation, empty-`PATH` launch, and removal checks passed |
| Native stdio suite | 7 passed |
| Roadmap contract | generator current; 7 unit tests passed |

The real-client tests used disposable homes and removed their receipt-owned
test bundles. Normal Host profiles, credentials, prompts, responses, and user
Zotero libraries were not inputs. Zotero traffic remained loopback-only and the
write stayed fail-closed behind `dry_run=false`, `write_intent=apply`, and the
immediately preceding `zwr1_` receipt.

## Nonclaims

This Slice does not accept App setup/status/recovery (`PLT-321`), Graph v1
(`PLT-322`), candidate or packaged artifacts, an authenticated model session,
a real user Zotero library, signing, notarization, promotion, publication, or
release authorization. Any such claim requires its own exact-source Acceptance
tier evidence.
