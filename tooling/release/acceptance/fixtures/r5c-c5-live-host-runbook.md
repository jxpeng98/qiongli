# R5C C5 Live Host Runbook

Use this runbook once for Codex and once for Claude Code. C5 deliberately uses
two different profile scopes:

1. the accepted package's isolated `manual-home` proves clean 2.x
   install/migration, restart discovery, Plugin, Skill, and MCP projection
   without authentication; and
2. the already authenticated system Host performs the live handoff after its
   Qiongli registration has been migrated to the same accepted 2.x Plugin
   content.

The Host owns authentication, reasoning, and conversation state; Qiongli
remains the installed workflow, project, Plugin, Skill, and Full MCP shell.
There is no Qiongli OAuth flow.

Do not copy credentials into the isolated home or into Qiongli. Do not launch a
model CLI from Qiongli. Do not place project IDs, paths, prompts, responses,
conversations, tool bodies, registration paths, or credentials in the final
observation or receipt.

## 1. Separate installation and live-Host preflight

First retain the already completed isolated proof:

- the accepted App installed `qiongli-next` `2.0.0-alpha.2` into
  `manual-home`;
- the isolated registrations survived App and client restart; and
- the prepared three-project fixture remains at semantic revision `2`.

Do not log the isolated Host into Codex or Claude Code. Authentication is not
part of the installation proof.

Then prepare the existing system Host:

1. Confirm that the normal Codex or Claude Code process is already
   authenticated through that Host's own account. If it is not, any login is
   the Host's login—not Qiongli OAuth.
2. In the accepted App, migrate or reinstall the selected system integration
   until it reports current `qiongli-next` `2.0.0-alpha.2`. A recognized 1.x
   source is migration input only and cannot satisfy this gate.
3. Confirm the host-specific system registration exists:
   - Codex:
     `<user-home>/.qiongli/plugins/codex/.qiongli-next-codex-registration.json`
   - Claude Code:
     `<user-home>/.qiongli/v2/integrations/claude-code/.qiongli-next-claude-registration.json`
4. Start a fresh normal Host process. Confirm the current Plugin, Skill, and
   Full MCP attachment and confirm it can observe the synthetic acceptance
   project at semantic revision `2`.
5. Use the host-visible Qiongli Skill and Full MCP tools for the remaining
   steps. Do not call a second model CLI as a child process.

The composer reads only the bounded 2.x registration document and compares its
version and Plugin content digest with the isolated accepted installation. It
rejects the isolated acceptance registration being reused as the system input
and does not read the Host's authentication store. Use one stable host
descriptor for the entire run:

```json
{
  "schemaVersion": 1,
  "family": "codex",
  "hostVersion": "<observed-host-version>",
  "adapterVersion": "2.0.0-alpha.2",
  "fullMcpProtocol": "qiongli-full-mcp/1",
  "capabilities": ["single-agent"],
  "pluginState": "ready",
  "registrationState": "ready",
  "enablementState": "ready",
  "trustState": "ready",
  "activationState": "ready"
}
```

Change `family` to `claude-code` for Claude Code. Do not infer `ready`: each
state must be observed in that fresh system Host process.

## 2. Rejection observations

Before the valid submission, observe all four fail-closed paths. After every
probe, call `qiongli_orchestration_runs` with revision `2` and confirm that the
run generation and document digest did not advance.

1. Call `qiongli_orchestration_doctor` with revision `1`; require a revision
   conflict.
2. After starting the valid run, call `qiongli_orchestration_next` with one
   changed hex character in `expectedDocumentSha256`; require a stale run
   reference.
3. Call `qiongli_orchestration_submit` with a candidate that is otherwise bound
   to the current handoff but contains one syntactically valid evidence
   reference that was not returned by `qiongli_orchestration_read`; require
   `host-candidate-evidence-unauthenticated`.
4. Call `qiongli_orchestration_next` with one additional
   `acceptanceProbe` field; require schema or argument rejection.

Record only one bounded count for each observed rejection. Do not record the
rejected candidate or tool arguments. Set `rejectionStateUnchanged` only after
all four post-probe run snapshots match their pre-probe generation and
document digest (or the same empty run set for the stale-revision doctor
probe).

## 3. Valid triad handoff

Call `qiongli_orchestration_start` at revision `2` with `executionMode:
"triad"` and the stable host descriptor.

For each returned primary, reviewer, and verifier handoff:

1. Preserve the exact run ID, generation, document digest, handoff digest,
   task ID, role, attempt, and candidate kind returned by the tool.
2. Call `qiongli_orchestration_read` for both allowed tools with
   `toolArguments: {"project_id":"<current-project-id>"}`:
   - `qiongli_project_graph_snapshot`
   - `qiongli_project_read`
3. Preserve each exact `_meta["qiongli/evidence"]` reference. Build
   `knownFactDigests` from the used evidence `resultSha256` values, sorted and
   deduplicated.
4. Produce a bounded candidate using only the observed graph and project
   evidence. Include an explicit `evidenceGaps` array and truthful
   `reviewResult`: `not-applicable` for the primary role and `pass` for the
   reviewer and verifier roles.
5. Submit the exact candidate. Record the returned
   `acceptedCandidateSha256`, generation, and document digest.

The receipt observation records the primary candidate digest and the following
continuous, directly observable chain:

| Role completed | Transition | Expected shape |
|---|---|---|
| Primary | `candidate-accepted` | generation `2` to `3` |
| Reviewer | `review-accepted` | generation `3` to `4` |
| Verifier | `checkpoint-persisted` | generation `4` to `6` |

Use the actual generations and digests returned by the host. Reject the run if
the chain is not continuous; never manufacture the unobservable initial
handoff transition.

After recording the checkpoint, cancel the still-active acceptance run with
the exact current generation and digest. This cleanup is not an acceptance
transition. Confirm App and copied-CLI parity and confirm that the project
semantic revision remains `2`.

## 4. Compose the package-bound receipt

Create a temporary observation that conforms to
`r5c-c5-host-observation.schema.json`. `evidenceResultSha256s` is the strictly
sorted, deduplicated set of result hashes from the evidence references used by
the accepted triad. The composer derives its count and audit digest. It also
derives the known-fact count and fact-set digest from the fixed fixture. Set
`hostProfileScope` to `system-existing`.

Canonicalize the observation as a single JSON value with no terminal newline,
then run one host-specific command:

```bash
bash scripts/compose_macos_acceptance_host_receipt.sh \
  --observation /absolute/path/to/canonical-observation.json \
  --system-registration \
    /absolute/path/to/.qiongli-next-codex-registration.json
```

The composer binds the receipt to the exact accepted product, binary, source
commit, isolated prepared fixture and installation, plus the current system
Host registration. It writes one private receipt inside the ignored acceptance
root and rejects an existing receipt with different content. Use the Claude
Code registration path for the Claude Code run.

Validate the generated receipt:

```bash
bash scripts/validate_macos_acceptance_host_receipt.sh \
  --receipt /absolute/path/to/generated-host-receipt.json \
  --system-registration \
    /absolute/path/to/.qiongli-next-codex-registration.json
```

Delete the temporary observation after validation. Commit only the
path-redacted receipt or its bounded acceptance record, never the host
conversation, system registration, or private acceptance root.
