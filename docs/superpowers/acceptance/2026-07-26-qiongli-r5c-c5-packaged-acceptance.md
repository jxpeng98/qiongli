# Qiongli R5C C5 Packaged Acceptance Record

Status: partially accepted — packaged continuity and isolated host
installation/restart passed; live Codex and Claude handoffs are pending
user-controlled authentication

Date: July 26, 2026

Publication allowed: false

## Product identity

| Field | Accepted value |
|---|---|
| Product version | `2.0.0-alpha.2` |
| Source commit | `1673e1f6c1eb933c8033b6981df883b67d19c8d1` |
| Receipt schema | `2` |
| Receipt status | `accepted-ad-hoc-nonpublishing` |
| Canonical package SHA-256 | `1d103190d712e61cb0019f66b038b7ba784d832a9e36f745c797a09e127dea05` |
| Product-control SHA-256 | `772ae23bf7afa21ffd90d59ae20d59136a1528643b770ba8c0d7b9ecfc349610` |
| Signed archive SHA-256 | `05fc1ae9d8e4fc720b71e20c12f8c52b722b922bc7788cb65e017417274806bf` |
| Signing class | Local ad-hoc, non-publishing |
| Packaged authority | Verified packaged product |

The product-controlled acceptance command ran from the exact clean source
commit. Generated Apps, homes, logs, fixtures, project contents, and
screenshots remain ignored local evidence and are not part of this record.

## Automated packaged acceptance

All schema-2 product checks passed:

- canonical signature and packaged-product control;
- empty-`PATH` startup and embedded authority;
- embedded inventory and Lite MCP self-test;
- Skills materialize, verify, and refresh;
- current Codex and Claude Code install, verify, restart, repair, and removal;
- isolated nine-surface 1.x-to-2.x migration;
- packaged restart verification;
- provider Keychain lifecycle in the isolated fixture;
- three-project restart and continuity lifecycle; and
- App/CLI library parity plus Full MCP public library/portfolio parity.

The continuity receipt contains only bounded observations:

| Observation | Count or verdict |
|---|---:|
| Registered projects | 3 |
| Delivery records | 4 |
| Retry | 1 |
| Acknowledgement replay | 1 |
| Duplicate suppression | 1 |
| Assignment | 1 |
| Resolution | 1 |
| Explicit resolution items | 5 |
| Archive / restore | 1 / 1 |
| Derived-state deletion | 1 |
| Full rebuild | 2 |
| Matching query projects | 3 |
| Matching query lineage | 8 |
| Timeline events | 33 |
| Shared source / concept / method identities | 1 / 1 / 1 |
| Reviewed lineage | 1 |
| App/CLI library parity | true |
| Full MCP library/portfolio parity | true |
| Canonical artifacts unchanged by derived rebuild | true |
| Path redacted | true |

Full MCP parity is limited to its public project-list and academic-graph
portfolio contracts. The receipt does not imply Full MCP mutation or Timeline
tools that the public contract does not expose.

## Manual packaged App acceptance

The accepted App was opened without rebuilding and with the dedicated manual
acceptance home.

1. **About** showed `2.0.0-alpha.2`, the exact source commit, and
   `Verified packaged product`; no source-build authority warning appeared.
2. **Overview** reported 422 verified embedded entries and 12 Lite MCP public
   tools. The manual Research Library remained empty, confirming that the real
   user and automated acceptance libraries were not reused.
3. **Client Integrations** initially showed both managed 2.x projections as
   absent. The App previewed filesystem, client-configuration, and host-trust
   changes before confirmation.
4. The confirmed batch installed only current 2.x content. Codex and Claude
   Code each reported plugin version `2.0.0-alpha.2`; Source, Skills,
   Marketplace, and Registration became ready. Activation and MCP attachment
   remained explicitly host-controlled.
5. After terminating and reopening the same accepted App with the same
   isolated home, the packaged identity and both managed integration receipts
   were rediscovered. No 1.x source satisfied current readiness.

The automated three-project fixture uses a separate destructive-test home.
Therefore the empty manual library does not count as manual rediscovery of the
three projects. That observation remains part of the live-host completion
batch rather than being inferred from the automated receipt.

## Real host installation and restart observations

Each command ran as a new process against the same isolated manual profile.

| Host | Client | Source/registration | Plugin and Skills | Full MCP | Live handoff |
|---|---|---|---|---|---|
| Codex | `0.144.6` | Personal marketplace discovered | `qiongli-next@personal` installed and enabled at `2.0.0-alpha.2` | `qiongli-next` registered and enabled | Pending authentication |
| Claude Code | `2.1.216` | `qiongli-local` marketplace added | `qiongli-next@qiongli-local` enabled at `2.0.0-alpha.2`; one workflow Skill visible | Plugin MCP health reported connected | Pending authentication |

Both isolated profiles reported that they were not logged in. No credential
was copied from the real user home, and no live model call was attempted.

## Host-visible fixture preparation

Commit `6192cd20` added a package-bound preparation and validation flow for the
remaining live sessions. Commit `9884f494` tightened it to use only evidence
that the live Full MCP contract exposes: graph-backed facts, three continuous
post-handoff transitions, and four fail-closed rejection observations. Neither
flow rebuilds the accepted product or writes to the real user home.

The preparation command validated the exact product receipt and copied App
binary before creating the same bounded three-project continuity lifecycle in
the isolated manual profile:

| Field | Prepared value |
|---|---|
| Fixture | `r5c-c5-host-driven-v1` |
| Fixture SHA-256 | `1a2b2b8252418161cd3c35a54546e613502cd576ad4980899a9866956296a12d` |
| Product acceptance receipt SHA-256 | `b163f413b7032a8ec1e1a5ac68a68b0cef15ad1d861050851f26a0525ae2998e` |
| Preparation receipt SHA-256 | `86e70951141fc2d05a577252fcffc75afd455ee301640d5138a07b943e86c8d0` |
| Host project revision | `2` |
| Registered projects | `3` |
| App/CLI library parity | true |
| Full MCP library/portfolio parity | true |
| Path redacted | true |
| Publication allowed | false |

The source fixture contains two synthetic fact and anchor digests for records
that are visible through `qiongli_project_graph_snapshot` and
`qiongli_project_read`. Preparation verifies those facts against the actual
prepared Evidence Atlas graph before refreshing or accepting its receipt. The
local preparation receipt contains only product and fixture hashes, bounded
counts, one project ordinal and revision, and verdicts. It contains no project
identifier, path, prompt, response, credential, conversation, or tool body.
Running the preparation command after `9884f494` validated the existing
projects and refreshed only the fixture binding; it did not rebuild the App,
duplicate a project, or change project revision.

The package-bound receipt composer and validator were also exercised with an
explicitly synthetic temporary observation. The composer derived the evidence
audit and fixed fact-set digests, and the validator accepted the exact product
binary, source commit, prepared revision, and Codex Plugin content digest as
one bound set. That synthetic check is not a live-host acceptance claim and was
not retained as evidence.

## Privacy, isolation, and model boundary

- All product, Plugin, Skills, marketplace, cache, receipt, and fixture writes
  remained inside the dedicated acceptance roots.
- The real Codex, Claude, Qiongli, and project homes were not modified.
- Qiongli stored no model credential and issued no provider request.
- Qiongli stored no prompt, response, conversation, candidate body, or tool
  body.
- Qiongli did not launch Codex, Claude, or another model CLI as its model
  backend.
- The committed record contains no absolute path, project identifier, source
  text, artifact body, provider credential, prompt, or response.
- No broad cybersecurity scan was run. The focused checks covered package
  identity, path containment, current-product ownership, receipt binding, and
  absence of Qiongli-owned model transport.

## Open completion items

C5 remains open until user-controlled authentication is available in both
isolated host profiles and the following observations pass:

1. restart the packaged App and confirm the prepared three projects remain
   visible at the declared revision;
2. start fresh authenticated Codex and Claude Code sessions and confirm the
   current Plugin, Skill, and Full MCP attachment;
3. complete one revision-bound handoff per host using declared project evidence;
4. reject a stale revision, mismatched checkpoint digest, undeclared evidence,
   and unknown handoff field without advancing state;
5. record the continuous primary, reviewer, and verifier checkpoint chain and
   cancel the remaining acceptance run using its exact final binding;
6. compose and validate each path-redacted receipt against the exact package,
   prepared fixture, and host-specific installed plugin digest; and
7. return to App and copied CLI and confirm project revision and checkpoint
   parity while all conversation and provider material remains absent.

The operator sequence and observation boundary are fixed in
`tooling/release/acceptance/fixtures/r5c-c5-live-host-runbook.md`.

Until both live-host receipts exist, C5, R5C completion review, and all
publication claims remain open.
