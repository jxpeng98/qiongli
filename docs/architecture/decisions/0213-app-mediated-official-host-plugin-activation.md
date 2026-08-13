# ADR 0213: App-Mediated Official Host Plugin Activation

- Status: Accepted
- Date: 2026-08-13
- Task ID: `ARC-213`
- Owners: Qiongli maintainers
- Decision scope: local App execution of official Codex and Claude Code Plugin
  commands after an exact integration preview and approval
- Supersedes in part: ADR 0206's requirement to report a documented Host
  action and stop; ADR 0212's matching `awaiting-client-activation` behavior
  when the supported Host exposes an official command and fresh observation
- Retains: ADR 0206's typed plan, ownership, conflict, trust, policy, cache,
  local/remote, security, rollback, and distinct-state boundaries; ADR 0212's
  install-before-cleanup and recovery rules

## Context

The packaged App already materializes receipt-owned Qiongli Plugin sources,
registers supported marketplaces, previews the exact Host action, and probes
official client state. A confirmed clean install still stops at
`installed-host-action-required`, requiring the user to copy commands even
when the installed Codex or Claude Code client provides an official local CLI
for the same operation.

That stop is safe but leaves the App's primary installation path incomplete.
The user has approved one bounded extension: the existing confirmation may
authorize the App to run the fixed official command plan shown in the preview.
The Host continues to own policy, trust, installation, enablement, cache, and
runtime state. Command exit status alone is not evidence that the Plugin is
Ready.

## Decision

### One approval binds one fixed Host plan

The existing integration preview includes the packaged-product changes and a
native Host action plan for every selected target. Its semantic digest covers:

- target and discovered absolute Host executable;
- install or receipt-owned repair mode;
- exact Plugin identifier, version, scope, and managed source;
- ordered fixed argument vectors; and
- the observed state and packaged-product plan already protected by ADR 0206.

Confirmation authorizes only that digest. A change to executable identity,
target, mode, Plugin identity, version, scope, source, arguments, or observed
state invalidates the preview before any Host command runs.

The executable comes only from supported-client discovery. Arguments come only
from target/action constants in the native integration owner. Rendered UI
text, serialized command strings, model output, environment aliases, and user
input are never executable input. Display-only `$HOME` text is resolved to the
verified platform home without shell expansion.

### Only official target-specific commands execute

The allowlist is deliberately closed:

- Codex install requests `qiongli-next@personal` through the official Plugin
  CLI;
- Claude Code install registers the receipt-owned local marketplace and then
  installs `qiongli-next@qiongli-local` at user scope; and
- repair may use only the existing target-specific remove/install sequence,
  and only when the preview names a receipt-owned repair.

No generic command runner, arbitrary executable/argument API, shell, login
shell, script file, UI automation, or direct Host-cache mutation is added.

### Execution is bounded and fail closed

The App launches the discovered absolute client directly with:

- no stdin and no shell;
- the verified platform home and deterministic current directory/environment;
- fixed timeout and bounded stdout/stderr; and
- stable classification for spawn failure, timeout, non-zero exit, output
  overflow, decoding/parse failure, and observation mismatch.

Selected targets run serially in deterministic order. The first failure stops
later targets. If a repair command partially changes the selected Qiongli
Plugin, the App records that repair or verification is required. It does not
guess at compensation by editing Host caches or unrelated Plugin state.

### Fresh Host observation owns Ready

After every attempted mutating Host plan, prior Host observations are
discarded. After a successful plan, the App performs new bounded official
probes. Ready requires the same refreshed snapshot to prove all applicable
facts:

- receipt-owned packaged source, registration, managed bundle, and matching
  Host bundle/cache identity;
- exact Plugin identifier and version, installed and enabled in the required
  scope/source;
- enabled `qiongli-next` Full MCP; and
- for Claude Code, exactly one `qiongli-workflow` Skill and one
  `qiongli-next` MCP component.

Codex exposes Plugin and MCP inventory but no Plugin component-details command.
Its exact activated bundle identity proves the canonical Skill bytes are in
the bundle; it does not prove discovery or live invocation by a model session.

A successful command followed by missing, stale, malformed, or contradictory
evidence is non-Ready. A later explicit verify may clear that state only from a
new positive observation. Ready means usable by a new Host session; bundled
skills may require that new session before use.

### Host authority remains intact

Official clients may reject commands because of trust prompts, authentication,
workspace policy, administrator controls, marketplace review, or unsupported
versions. The App reports that result and remains non-Ready. Approval in
Qiongli never suppresses or substitutes for Host controls.

The operation remains local. It makes no claim about ChatGPT web, Codex cloud,
Claude cloud, another machine, or authenticated model execution.

## Alternatives considered

### Continue requiring copied commands

Rejected for the supported local clients because it leaves an avoidable manual
gap after the App already owns the exact preview, receipt, and verification
flow.

### Execute the rendered command text

Rejected because presentation text is not a safe execution contract and would
create a shell/argument injection boundary.

### Write the Plugin directly into Host caches

Rejected because caches are private Host state and cannot establish supported
installation, enablement, trust, or runtime activation.

### Treat a zero exit code as Ready

Rejected because Host policy, stale cache state, wrong versions, missing MCP,
or missing components can remain after command completion.

## Consequences

- One existing App confirmation can complete supported local Plugin setup.
- Preview and execution share one native plan owner, avoiding command drift.
- Installation may still finish non-Ready when the Host rejects or cannot
  prove the required state.
- Fresh probes add bounded latency after mutation but remove cached-success
  false positives.
- Existing CLI installation, removal, Host ownership, and remote boundaries do
  not change.

## Security and privacy

- Process launch accepts only discovered supported binaries and closed native
  argument templates.
- The executor inherits no stdin, uses bounded output/time, and never invokes a
  shell or reads executable arguments from UI/model/serialized display text.
- Paths are resolved under the verified platform home and revalidated before
  use; Host caches remain read-only observation inputs.
- Receipts and events contain stable reason codes and redacted identities, not
  raw Host output, profile paths, credentials, prompts, or account data.
- Preview expiry, digest, state, ownership, and conflict checks run before the
  first packaged or Host mutation.

## Rollback

Reverting the implementation restores the copied-command path. Plugin state
already created by an official client remains client-owned and is observed or
removed only through existing explicit Qiongli operations. The rollback never
deletes Host caches or unmanaged state. This accepted ADR remains historical
evidence and may be superseded by a later decision, not edited away.

## Acceptance tests

1. Codex and Claude Code install and receipt-owned repair plans produce only
   the fixed executable/argv/scope/source combinations and never use a shell.
2. A changed target, mode, executable, version, scope, source, argv, or observed
   state rejects the confirmation before Host launch.
3. Spawn, timeout, non-zero exit, overflow, decoding, parse, and probe mismatch
   failures return stable non-Ready results.
4. Multi-target execution is deterministic, stops at the first failure, and
   leaves later targets and unrelated Host state untouched.
5. Codex Ready requires exact Plugin/source/version/enablement, matching bundle
   identity, and enabled Full MCP from fresh official JSON probes.
6. Claude Code Ready additionally requires the exact user scope and one
   `qiongli-workflow` Skill plus one `qiongli-next` MCP component.
7. Command success with stale, malformed, missing, or contradictory evidence
   remains non-Ready until an explicit fresh verify passes.
8. Isolated temporary-home client tests prove the App path without mutating the
   normal Host profile or claiming authenticated model invocation.

## Primary references

- [OpenAI: Plugins in Codex](https://developers.openai.com/codex/plugins/)
- [Anthropic: Plugins reference](https://code.claude.com/docs/en/plugins-reference)

