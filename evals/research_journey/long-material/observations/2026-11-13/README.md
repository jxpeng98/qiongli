# Released CLI long-material and fresh-session observation

November 13, 2026. This uses the public **v2.6.1 / 25b6bf36f06b2f2ddb1b2395a88bb5dcd9b9cdb3**
Linux ARM64 archive and matching extracted Codex Plugin guidance. Both archives
match GitHub asset digests and the same release's SHA256SUMS. Codex 0.167.0 uses
the user's existing `gpt-6-astra` / `xhigh` settings in separate isolated profiles.
This is direct CLI execution with packaged guidance, not installed-Plugin/MCP
qualification. The later help repair was not substituted into either model run.

## Frozen task and authority

The [source packet](source-packet.json) is an invented 39,667-byte Chinese/escaped
JSON fixture. Relevant results and limitations occur in S01, S12, S22 and T1;
F1 is deliberately unavailable. It contains no real paper or scientific finding.
The [criteria](criteria.json) were frozen before either turn, outside model access.
The [first request](draft-task.md) asks for a bounded synthesis. Native
preview/approval/CAS first saves the source and historical handoff at revision 2.

The coordinator checks the [complete candidate](draft.md), retains it without
prose edits, reviews exact summary/handoff previews and approves the save at
revision 3. The [latest synthetic decision](current-handoff.md) appends a concise
project default, a next-paragraph-only explanatory override and unresolved issues
after 90 historical entries. The original candidate's “not yet saved/revision 2”
words remain historical content; the revision-3 receipt owns current saved status.
The [fresh request](resume-task.md) supplies the project ID and supported read
routes, with no previous conversation, saved prose or preference inline.

Both model turns are read-only and have tool-network access disabled. Effective
policy readback is retained in the two `*-policy.json` files. The model service
itself still uses the configured authenticated Host. Project/config/input/binary
fingerprints are equal before and after each turn; the complete extracted Plugin
file-hash map is represented by its digest and count. Model turns invoke no Graph
rebuild. No coordinator intervention occurs during either turn. Preparing the
fixture, source review and approved saving are coordinator work, not autonomous
model writes or measured human effort.

## Actual result

| Turn | Seconds | Native/shell calls | Failed calls | Outer truncations | Outer script failures |
| --- | ---: | ---: | ---: | ---: | ---: |
| Draft | 97.47 | 12 | 2 | 0 | 0 |
| Fresh recovery | 216.95 | 34 | 4 | 1 | 0 |

The draft receives the entire packet in three contiguous ranges and preserves the
numeric, causal, measurement and missing-image limits. Failed calls are missing
`project document` help topics. The [continuation](resume.md) recovers the current
preference, scoped override, original and unresolved issues, and distinguishes
historical prose from the current receipt. Coordinator source review finds no
substantive repair necessary. This is not independent expert review.

The context reader initially returns only 16,384 of 23,351 handoff bytes, with
`truncatedAfter=true`. Increasing its native budget succeeds, but batching that
response with long guidance truncates the **outer** return. A separate call then
returns the tail from line 275. The received prefix and tail overlap at byte
16,231 and cover the entire saved handoff; the 3,014-byte summary is also delivered
intact. [Coverage evidence](resume-coverage.json) binds these actual outer outputs,
their call IDs, and the exact saved digest/revision. It also proves failed help
calls retain successful sibling results in two batches. It does not exercise a
thrown wrapper exception.

Recovery delivers the required source fields using bounded reads, JSON pointers
and selected table/image metadata. It does **not** deliver every appendix body to
the model. Its full packet is assembled inside the tool wrapper, and the answer
correctly distinguishes that from actual reviewed coverage. The observation's
`whole_source_received=false` for recovery describes this distinction, not failure
to receive the required evidence. The draft's corresponding value is true.

The source and context recovery complete, but the outer truncation and extra help
calls remain costs. A single adaptive recovery does not establish truncation
prevention, net efficiency, quality improvement or reliable performance on other
materials. Input/cached/output tokens are recorded separately; actual billing and
human minutes remain unknown.

## Narrow repair and negative checks

Actual calls reproduce a missing help route: `project document --help` and its
list/read/search topics return `unknown help topic`; the short project page and
`help all` omit the same document owner. The candidate fixes only `cli_help.rs`,
reusing existing syntax, pagination and revision/digest explanations. Two CLI
integration tests and strict Rust 1.97 Clippy pass, including help with an unusable
config root, aliases, unknown-topic refusal and incomplete-read rejection. Readers,
MCP tools, model settings and write approvals do not change. No model rerun measures
the help repair's effect on call count or cost.

Separate [negative checks](negative-checks.json) reject stale revision, wrong
source hash and unapproved saving without changing project/config bytes. A
[wrong preview digest](stale-save-digest.json) also refuses. After both model turns,
a [source-change check](source-change-check.json) reads page one, changes only the
isolated synthetic packet, and confirms continuation with the old binding refuses.
The fixture is restored byte-for-byte; no user research is accessed.

Setup failures remain explicit: public web-cache/DNS access fails before the host
asset download; the default Python lacks PyYAML; the nested preflight fails on
socket directory mode while the unchanged inner read-only/no-network profile
passes in host context. The coverage audit initially assumes document field names
for a context artifact; using its actual `artifactPath`/`contentDigest` fields
fixes the audit. These are setup/audit attempts, not additional model runs.

## Inspect or replay

[observation.json](observation.json) binds the release URLs, assets, model/thread
identity, counts, token usage, raw local capture hashes and every exported evidence
file. `*-tools.jsonl` retains only actual tool calls/results from the rollout,
including the failed and truncated return. Reasoning, session messages and
credentials are excluded. `${OBSERVATION_ROOT}` and `${CODEX_BINARY}` replace only
local runtime paths; do not treat sanitized files as the raw rollout hash.
`*-commands.json` identifies the inner CLI commands and raw output hashes.
`*-read-delivery.json` derives received native views from untruncated outer results.
The coverage record separately handles the intentionally selected handoff/source
fields. Preview and apply records preserve save authority and artifact hashes.

To replay, download the bound public archives, create a fresh private synthetic
project through native create/capture preview and approval, and substitute its ID
and bindings into the retained requests. Run fresh read-only/no-network profiles
with the user's configured model; use the existing `codex_preflight.py`,
`antigravity_observation.run_process` and `plugin_baseline` trace/delivery owners.
Freeze new identities before running; record all attempts separately. Exact new
outputs are not expected to match this one observation. Original baseline scores
and program acceptance remain unchanged.
