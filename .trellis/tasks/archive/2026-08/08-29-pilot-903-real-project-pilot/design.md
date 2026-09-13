# Design: representative migrated real-project pilot

## System boundary

The pilot connects existing owners without adding a product layer:

```text
repository asset-pricing source
  -> native project migrate preview/apply (private temporary state)
  -> current-source Full MCP over stdio
  -> ephemeral Codex host + conventional Qiongli Skill
  -> revision-bound doctor/start/read/submit/next
  -> redacted receipt
  -> native migration rollback
```

The research source, native project service, Graph projector/query service,
orchestration checkpoint engine, Skill contract, and Codex client remain the
authorities. No new Graph model, Host adapter, or project mutation API is added.

## Isolation and identities

- Start from one clean committed product source and build the native binary once.
- Create one private temporary root containing the Qiongli config root, migrated
  project destination, project-local `.agents/skills/qiongli` materialization,
  temporary Codex output schema, and non-committed run output.
- Resolve the Codex executable/version before execution. Codex may use its own
  authentication, but `--ignore-user-config` prevents normal configuration from
  defining the pilot and `--ephemeral` prevents session-file persistence.
- Pass the temporary `QIONGLI_CONFIG_HOME` only to the pilot process and Full MCP
  child. The normal Qiongli registry and normal Plugin/cache paths are not used.
- Hash exact identities before cleanup; public evidence contains no temporary or
  absolute path.

## Host execution

The prompt is bounded to the existing asset-pricing project and instructs Codex
to invoke the discovered Qiongli Skill. The current-source Full MCP is registered
explicitly as a stdio server. The Host descriptor reports `single-agent` and,
when the output schema is active, `structured-output`; it does not report native
subagents. Registration, enablement, trust, activation, and Plugin states come
from observed pilot state rather than installed-version text.

Codex must:

1. list/read the exact migrated project and revision;
2. call host doctor and stop on a non-runnable result;
3. start a solo run and execute each returned handoff;
4. read evidence only through tools allowed by that handoff;
5. include unchanged evidence references and known-fact digests in submission;
6. call `next` with the new generation/document digest until terminal;
7. return only the bounded observation fields required for receipt composition.

The run must include a source-grounded project read and a non-empty Graph
snapshot/query. The exact model prose is not an acceptance assertion.

## Evidence and privacy

The derived receipt may contain:

- exact source commit and binary/Skill/source-inventory digests;
- Codex, adapter, and Full MCP protocol versions;
- migration plan/receipt digests without filesystem locations;
- projection digest, readiness, and bounded node/edge/relation counts;
- counts/digests of handoffs, authenticated reads, submissions, and terminal
  checkpoint state;
- required check IDs and the receipt digest.

It must not contain prompts, model responses, candidate text, tool results,
citations or research rows, Host conversation/session IDs, project paths,
temporary locations, environment values, or credentials. The private project
registry necessarily owns its temporary root while registered; rollback removes
that isolated state. Raw subprocess output is temporary working material, never
repository evidence.

## Failure and rollback

- A Lite route, missing Skill/tool, stale revision, rejected evidence reference,
  non-terminal checkpoint, empty Graph result, source drift, forbidden receipt
  field, or failed rollback leaves `PILOT-903=proposed`.
- If the failure is a reproducible current-source product defect, patch only its
  shared owner and leave one focused regression before rerunning the pilot.
- Always invoke supported migration rollback when an apply occurred. If rollback
  refuses drift, retain the private temporary root for diagnosis and report the
  blocker instead of deleting product-owned state manually.
- Normal user state is never a rollback target because the pilot does not mutate
  it.

## Test boundary

Use the smallest checks that prove this slice:

- one existing Full-route/host-handoff native test if the relevant code is
  unchanged, or one focused new regression if a defect is fixed;
- the single real Codex pilot and its receipt/privacy/rollback assertions;
- Program Ledger generation check and `git diff --check`;
- one exact-head Slice CI run before acceptance closeout.

Do not rerun the full native, Desktop, App API, or PLT-322 suites unless the
pilot forces a product change in one of those owners.
