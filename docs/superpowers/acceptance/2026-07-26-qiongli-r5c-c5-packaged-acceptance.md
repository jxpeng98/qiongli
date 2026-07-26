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

1. seed or register the same three disposable projects in the host-visible
   acceptance profile, then restart the packaged App and confirm discovery;
2. start fresh Codex and Claude Code sessions and confirm the current Plugin,
   Skill, and Full MCP attachment;
3. complete one revision-bound handoff per host using declared project evidence;
4. reject a stale revision, mismatched checkpoint digest, undeclared evidence,
   and unknown handoff field without advancing state;
5. advance only after explicit artifact approval; and
6. return to App and copied CLI and confirm project revision and checkpoint
   parity while all conversation and provider material remains absent.

Until both live-host receipts exist, C5, R5C completion review, and all
publication claims remain open.
