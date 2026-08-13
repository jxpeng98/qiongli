# Close App-bundled CLI and Plugin activation

## Goal

Make the packaged App's existing install surfaces produce two honest outcomes:

- the bundled `qiongli` CLI is installed, reachable from a fresh login shell,
  and reports the App's exact version; and
- one confirmed integration operation installs or repairs the selected bundled
  Plugin through the official Codex/Claude CLI, then reports Ready only from a
  fresh positive Host observation.

Ready means ready for use by the next fresh Host session. It does not claim that
a model session has already invoked the Skill.

## Dependency

Start only after the parent roadmap-realignment task is accepted and archived.
This P0 task must be accepted or explicitly deferred before the P1 Plugin-quality
task starts.

## Confirmed Baseline

- The App's About surface already implements CLI preview, receipt-owned
  install/update/remove, PATH configuration, and a fresh login-shell command and
  version test.
- The integration surface already has one `preview-install-selected` /
  `confirm-operation` flow, strict target/scope `hostAction` data, packaged
  Plugin materialization and registration, cache receipts, Host probes, and
  isolated real-client tests.
- The missing production link is execution: after materialization/registration,
  the App probes but does not run the displayed official Host commands, so a
  clean install remains `installed-host-action-required`.
- Codex exposes Plugin and MCP JSON inventory but no Plugin component-details
  command. Claude exposes Plugin JSON inventory and text component details,
  including `qiongli-workflow` and `qiongli-next` MCP.
- Accepted ADR 0206 forbids direct cache mutation, shell/UI bypass, and treating
  copied or registered bytes as active. It is frozen and currently leaves the
  Host action to the user, so this material product decision requires a new
  superseding ADR.

## Requirements

### A1. Preserve and prove the existing bundled CLI path

- Reuse `cli_install.rs` and the current About UI; do not add another installer.
- A confirmed CLI install/update remains receipt-owned and version-bound.
- PATH configuration must be idempotent and a fresh supported login shell must
  resolve the managed executable and exact App version.
- Existing remove/predecessor restoration behavior must remain unchanged.

### A2. Record the new trust-boundary decision

- Add ADR 0213 with Task ID `ARC-213` before product code changes.
- Supersede only ADR 0206's requirement that the installer stop after reporting
  a documented Host action, plus the matching limited wording in ADR 0212.
- Retain Host ownership, trust prompts, administrator policy, explicit preview
  and approval, receipt ownership, conflict behavior, direct-cache prohibition,
  no arbitrary shell, and separate materialized/registered/enabled/active
  states.
- Add ADR 0213 to the decision-log README; do not modify frozen ADR 0206.

### A3. Bind one approval to one fixed Host action plan

- The existing integration preview must state that the App will run the
  official client CLI and identify selected target, scope, managed source,
  expected Plugin version, and whether the action is install or receipt-owned
  repair.
- The confirmation token and plan digest must bind both packaged-product changes
  and the fixed Host action plan. A state or plan change invalidates the preview.
- Command plans are native constants selected from current verified state:
  - Codex install: official `plugin add` for `qiongli-next@personal`;
  - Claude install: official local `marketplace add`, then `plugin install` for
    `qiongli-next@qiongli-local` at user scope;
  - receipt-owned repair may use the existing target-specific remove/install
    sequence only when the preview explicitly names repair.
- Executables come only from supported client discovery. Arguments come only
  from the native plan; `$HOME` display text is replaced by a resolved managed
  path without invoking a shell.
- UI input, model output, serialized command text, environment aliases, and
  arbitrary executable/argument input must never reach process launch.

### A4. Execute with bounded, fail-closed behavior

- Run with no stdin, a fixed timeout, bounded stdout/stderr, deterministic
  environment, and the existing resolved platform home.
- Distinguish at least spawn failure, timeout, non-zero exit, oversized output,
  invalid UTF-8/JSON, and observation mismatch with stable reason codes.
- For a multi-target confirmation, use deterministic serial order and stop at
  the first failed target. A previously successful target remains observable;
  later targets remain untouched and retryable.
- If a receipt-owned repair partially changes the selected Qiongli Plugin and a
  later command fails, do not mutate Host caches or unrelated state to guess at
  compensation. Mark repair required and allow explicit retry/verify.

### A5. Require fresh Ready evidence

After every successful Host command sequence, discard prior observations and
run a fresh bounded probe for the affected target.

- Shared App evidence must show receipt-owned source, Skill bundle,
  marketplace/registration, and exact managed bundle receipt as Ready.
- Codex evidence must parse official JSON and observe:
  `qiongli-next@personal`, exact version, installed and enabled, the expected
  local source, a matching managed/cache bundle receipt, and enabled
  `qiongli-next` Full MCP. The matching exact bundle proves that the canonical
  Skill bytes are present; it does not prove live invocation.
- Claude evidence must parse official JSON/details and observe:
  `qiongli-next@qiongli-local`, exact version, user scope, enabled state,
  expected install/cache identity, exactly one `qiongli-workflow` Skill, and the
  `qiongli-next` Full MCP component.
- Ready/Connected is permitted only when every required App and Host observation
  is positive in the same refreshed snapshot.
- Command success followed by missing, stale, malformed, or contradictory probe
  evidence is an operation failure and remains non-Ready until a later explicit
  verify produces fresh positive evidence.

### A6. Keep the user path clear

- The existing command list may remain visible as approval/audit detail, but it
  must no longer instruct the normal user to copy commands after confirmation.
- Completion and failure copy must name whether the App is Ready, needs a fresh
  Host restart, or needs verify/retry/repair.
- Preserve keyboard/accessibility behavior and existing batch selection.

### A7. Leave deterministic evidence

- Add one focused native regression owner covering the fixed argv, approval
  binding, command result classification, fresh probes, Ready predicate, and
  partial failure behavior for both supported Hosts.
- Reuse isolated fake executables and the existing real-client temporary-home
  tests; never mutate the normal Host profile in automated tests.
- Keep App API/frontend tests only where the public snapshot or copy actually
  changes. Do not add an umbrella harness.

## Acceptance Criteria

- [ ] Existing CLI install/PATH/fresh-shell tests prove exact version, shadowed
      command, missing command, and version mismatch behavior without new CLI
      architecture.
- [ ] ADR 0213 records the user's one-confirmation official-CLI decision,
      supersedes only the conflicting part of ADR 0206/0212, and retains every
      cache, trust, ownership, conflict, and fail-closed boundary.
- [ ] One integration confirmation runs the exact fixed Codex install plan in an
      isolated home and reaches Ready only after fresh Plugin/cache/MCP evidence.
- [ ] One integration confirmation runs the exact fixed Claude install plan in
      an isolated home and reaches Ready only after fresh Plugin/Skill/MCP
      evidence.
- [ ] Preview digest or state changes prevent execution of a stale Host plan.
- [ ] Tests prove no shell is used and no model/UI-supplied executable or
      argument can be launched.
- [ ] Timeout, spawn/non-zero failure, output overflow, malformed JSON/text,
      wrong ID/version/scope/source, cache mismatch, missing Skill, and missing
      MCP all remain non-Ready with a stable retry/verify/repair reason.
- [ ] A batch failure stops later Host commands, preserves unrelated Host state,
      and does not perform direct cache compensation.
- [ ] App API, Desktop checks/tests/build, focused native tests, isolated
      real-client CLI tests, and one packaged-App vertical pass for the frozen
      product input.
- [ ] No completion claim extends to authenticated model use, public release,
      target-native qualification, or publication.

## Out of Scope

- A generic command runner, shell execution, undocumented Host configuration,
  direct Host-cache writes, or UI automation.
- New Host support or redesign of the native CLI lifecycle.
- Editing frozen ADR 0206 in place.
- Automatic removal semantics beyond the existing separately confirmed remove
  operation.
- Paid/model-dependent Skill invocation, research-result evaluation, signing,
  release publication, and target-native claim closure.
