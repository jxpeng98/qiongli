# Design: Codex and Claude cross-agent compatibility proof

## Problem

Qiongli already owns the required runtime and installation mechanisms, but the
accepted evidence is asymmetric: the real Codex and Claude tests prove Full MCP
inside a working Plugin/Skill install, while Lite is proven only at the native
stdio boundary. The minimum complete solution is to close that evidence gap at
the existing two Host tests and publish the exact path contract. It is not a
new installer or a new agent abstraction.

## Authority and boundaries

| Concern | Existing owner | Planned change |
|---|---|---|
| MCP registries and dispatch | `qiongli-runtime`, `apps/qiongli/src/mcp.rs` | Reuse unchanged unless the new matrix exposes a real profile defect. |
| Codex paths and lifecycle | `qiongli-platform::client_inventory`, Codex adapter/bundle | Reuse exact paths and receipt lifecycle; add focused assertions only where coverage is missing. |
| Claude paths and lifecycle | `qiongli-platform::client_inventory`, Claude adapter/bundle | Reuse exact paths and receipt lifecycle; add focused assertions only where coverage is missing. |
| Plugin/Skill content | embedded canonical `content/` pack | Reuse one source and existing workflow-variant digest. |
| Host compatibility evidence | existing ignored Codex/Claude bundle tests | Extend each test to cover both MCP profiles and client configuration recognition. |
| User documentation | `docs/alpha/install-2x.md` | Add the supported two-client path/profile matrix and explicit nonclaims. |
| Acceptance evidence | `docs/superpowers/acceptance/` | Add one redacted exact-source compatibility note after Slice CI. |

No App API, frontend schema, MCP schema, canonical Skill, or generated Plugin
tree change is planned. If a live test reveals a shared-owner defect, fix that
owner and add one focused regression before continuing.

## Compatibility matrix

| Surface | Codex | Claude Code |
|---|---|---|
| User Skill root | `<user-home>/.agents/skills` | `<claude-config>/skills`, default `<user-home>/.claude/skills` |
| Project Skill root | `<project-root>/.agents/skills` | `<project-root>/.claude/skills` |
| Managed Plugin source | `<user-home>/.qiongli/plugins/codex/qiongli-next` | `<user-home>/.qiongli/plugins/claude-code/qiongli-local/plugins/qiongli-next` |
| Registration | personal marketplace at `<user-home>/.agents/plugins/marketplace.json` | Qiongli-managed local marketplace at `<user-home>/.qiongli/plugins/claude-code/qiongli-local/.claude-plugin/marketplace.json`, installed at user scope |
| Skill inside Plugin | `skills/qiongli-workflow/SKILL.md` | `skills/qiongli-workflow/SKILL.md` |
| Full MCP Plugin entry | `.codex-plugin/plugin.json` -> `.mcp.json` -> native `--profile full` | `.claude-plugin/plugin.json` MCP entry -> native `--profile full` |
| Lite compatibility entry | isolated official client MCP registration -> same native binary with `--profile lite` | isolated official client MCP registration -> same native binary with `--profile lite` |
| Host cache | Host-owned versioned cache, verified but never directly written | Host-owned versioned cache, verified but never directly written |

`.agents` is intentionally plural. `.agent` is not a Qiongli 2 installation
target. `<codex-config>/skills/qiongli-workflow` remains legacy-observation only
and cannot qualify the current Codex Skill path.

## Test data flow

```text
canonical embedded content + copied native binary
  -> receipt-owned Agent-specific Plugin source
  -> official isolated Host CLI install
  -> Host-listed Plugin + Host-owned cache + embedded Skill identity
  -> isolated MCP entries for lite/full
  -> client inventory/health observation
  -> PATH-empty initialize + tools/list for each profile
  -> exact registry/profile assertions
  -> official Plugin/MCP removal + absence check
```

The Plugin remains the production Full integration. The Lite registration is a
fixture-only compatibility probe, so the task does not leave two Qiongli MCP
servers in a user's normal Host configuration.

## Host-specific proof

### Codex

- Keep personal-marketplace registration and `codex plugin add/list/remove`.
- Use `codex mcp add/get/list/remove` in the isolated `CODEX_HOME` for the Lite
  probe and inspect the exact command/arguments returned as JSON.
- Codex CLI inventory does not itself establish an MCP handshake, so pair it
  with the copied/cache binary's exact Lite and Full stdio exchanges.
- Verify the cached receipt and customized Skill bytes before removal.

### Claude Code

- Keep strict validation, skills-directory discovery, local marketplace add,
  user-scope Plugin install/details/uninstall, and marketplace removal.
- Use `claude mcp add/get/list/remove` in the isolated `CLAUDE_CONFIG_DIR` for
  the Lite probe. Current Claude `mcp get/list` health-checks approved entries;
  require positive health evidence without any model session.
- Verify `plugin details` still reports exactly the expected Skill and MCP
  component before testing the cached native binary.

### MCP assertions

For each client fixture:

- Lite `tools/list` names exactly `LITE_PUBLIC_TOOL_NAMES` and excludes
  `qiongli_task_run`.
- Full `tools/list` names exactly the ordered union of Lite, Full project, and
  Full Host-orchestration controls.
- Full calls `qiongli_orchestrator_route` and receives the Full route without
  Lite-only `preview_only`, `runtime_profile`, `recommended_runtime`, or
  `upgrade` fields.
- stderr and rendered evidence contain no private fixture path or secret
  canary.

## Compatibility, security, and rollback

- Existing unsupported-version, conflict, drift, malformed inventory, bounded
  command, and partial-batch tests remain unchanged and green.
- Tests clear/override `HOME`, `USERPROFILE`, `CODEX_HOME`,
  `CLAUDE_CONFIG_DIR`, `QIONGLI_CONFIG_HOME`, and runtime `PATH` as appropriate.
- No authentication files or normal user projects are copied into fixtures.
- Rollback is a normal revert of the two test extensions, the 2.x guide matrix,
  and the acceptance note. No user data migration or cleanup is required.

## Trade-offs and deferred work

- We deliberately avoid a shared multi-agent test framework; two existing test
  files already own the Host-specific commands and output formats.
- We do not claim authenticated prompt-to-tool execution. That belongs to a
  later candidate/live-Host gate with credentials and consent.
- Additional agents require path, Plugin, MCP, discovery, ownership, and live
  verification decisions; copying the Skill into another dot-directory alone
  is not support.
