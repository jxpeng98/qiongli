# Technical Design

## Boundary

Extend the existing path in place:

```text
integration snapshot
  -> packaged install/repair preview + fixed Host plan
  -> one existing confirmation token
  -> receipt-owned materialization/registration
  -> official Host CLI plan
  -> discard prior observation
  -> fresh official Host probes
  -> Ready or explicit non-Ready reason
```

The bundled CLI path is already complete and remains unchanged unless its
focused regression exposes a defect. No new CLI installer, command framework,
worker service, or public arbitrary-command API is introduced.

## Decision Authority

ADR 0213 records the user-approved exception to frozen ADR 0206: after an exact
preview and approval, the local App may invoke an allowlisted official Host CLI
to request Plugin installation/enablement. The Host still owns policy, trust,
cache, installed/enabled state, and final observation.

ADR 0213 supersedes only the old “report the Host action and stop” behavior and
the matching limited step in ADR 0212. All other install-plan, ownership,
conflict, rollback, cache, remote/cloud, and evidence rules remain in force.

## Fixed Host Plan

Use one small native target/action mapping as the source for both displayed
`hostAction` detail and execution. The mapping contains only:

- target (`Codex` or `ClaudeCode`);
- mode (`Install` or receipt-owned `Repair`);
- required scope and exact Plugin identifier;
- fixed argv templates;
- expected product version and concrete managed source resolved under the
  verified platform home.

The preview stores the selected per-target mode and combines it with the
packaged-product plan digest. Confirmation rejects target, mode, product,
version, scope, path, or state drift. The executor never consumes the App API's
rendered command strings.

Install and repair reuse the official commands already represented by
`app_host_action_for_target` / `app_host_refresh_action_for_target`. A literal
`$HOME` is presentation-only; direct process argv receives the contained
resolved path.

## Execution Owner

Refine the existing bounded Host process helper rather than adding another
runner. It launches only the discovered absolute `codex` or `claude` binary,
with no stdin, no shell, fixed home/current directory, `NO_COLOR=1`, timeout,
and the existing output cap.

Return a minimal typed result so callers can preserve stable causes for spawn,
timeout, exit, overflow, decoding, and parse failure. Read-only probes and
mutating fixed plans share process containment, while only the fixed plan owner
may request mutating argv.

Selected targets execute serially in the existing deterministic target order.
On failure, stop. Do not attempt speculative rollback through cache files. The
selected Qiongli target may require repair; all unrelated Host state and later
selected targets remain untouched.

## Ready Contract

Ready is a conjunction, not command exit code.

### Shared App evidence

- packaged source and canonical Skill content are receipt-owned and exact;
- marketplace/registration is Ready;
- managed and Host cache bundle receipts match.

### Codex observation

- `plugin list --json`: exact ID, version, installed/enabled, expected local
  source;
- cache receipt: exact activated bundle identity, including the canonical Skill
  manifest/content;
- `mcp list --json`: `qiongli-next` enabled.

Codex currently exposes no component-details command, so Ready makes no live
Skill-discovery or invocation claim.

### Claude observation

- `plugin list --json`: exact ID, version, user scope, enabled, expected cache
  path/MCP declaration;
- `plugin details qiongli-next@qiongli-local`: exactly one
  `qiongli-workflow` Skill and one `qiongli-next` MCP server;
- cache receipt matches the managed source.

Prior in-memory observations are cleared before probing. Any command or probe
failure forces a non-Ready observation until a later explicit verify succeeds.

## App/API Compatibility

Prefer the existing `preview-install-selected`, `preview-reconcile-integrations`,
`confirm-operation`, `hostAction`, and connection-state schemas. Change the
wire contract only if a stable execution result cannot be expressed through
the existing event/reason-code surface. UI work is limited to preview and
completion copy; command details remain visible for transparency.

No on-disk App migration is required. Official clients continue to own their
installed/enabled state.

## Security And Failure Rules

- Supported-client discovery and minimum versions gate preview.
- Containment checks resolve the managed source under the platform home.
- No PATH lookup after preview, shell expansion, inherited stdin, arbitrary
  environment, UI-supplied argv, cache write, or unrelated Plugin mutation.
- A non-zero command that happened to leave an old Plugin observable still
  fails the operation; a separate fresh verify is required to clear the error.
- Logs/events expose stable reason codes and bounded redacted facts, never raw
  profile paths, Host output, prompts, credentials, or account data.

## Rollback

Reverting the product commit restores the manual Host-action path. Any Plugin
state already created by official clients remains client-owned and can be
observed or removed through the existing explicit flows. Receipt-owned App
sources remain safe; no schema or user-project migration must be reversed.
ADR 0213 remains historical decision evidence and can be superseded, not erased.
