# Qiongli 2.0 R3E Claude Code Local Design

Date: 2026-07-14  
Status: frozen for implementation  
Roadmap slice: `R3E / INT-202A`  
Scope: Claude Code personal skills-directory and local marketplace integration

## Outcome

R3E adds the first supported Claude Code local vertical over the same signed
artifact, verified embedded content, native Lite MCP executable, declarative
install plan, exact approval, receipt, and transactional filesystem boundaries
used by R3A through R3D.

The preferred installation is a personal skills-directory plugin. Qiongli may
place one verified native plugin package at
`<claude-config>/skills/qiongli`; Claude Code discovers that package in place
as `qiongli@skills-dir` without a marketplace installation step.

R3E also emits a local marketplace source containing the same verified package
bytes. Claude Code remains responsible for adding that marketplace, copying an
installed plugin into its versioned cache, and recording enablement. Qiongli
never writes Claude Code's marketplace registry, plugin cache, installed-plugin
registry, settings, or enablement state directly.

Both paths run the plugin-local Qiongli executable. Users do not need Python,
Node, Rust, Cargo, npm, pip, or a user-global `qiongli` command at runtime.

## Documented Claude Code Boundary

The implementation follows the current Claude Code plugin contract confirmed
on 2026-07-14:

- a plugin root may contain `.claude-plugin/plugin.json`;
- plugin components live at the plugin root, including `skills/`, `.mcp.json`,
  and `bin/`;
- a directory directly below `<claude-config>/skills/` with a plugin manifest
  is discovered in place and does not require marketplace installation;
- plugin MCP commands may use `${CLAUDE_PLUGIN_ROOT}` as the absolute installed
  plugin root;
- a local marketplace has `.claude-plugin/marketplace.json`, and relative
  plugin sources begin with `./` and remain inside that marketplace root;
- marketplace add and plugin install are separate Claude Code operations;
- marketplace-installed plugins are copied into a Claude-owned versioned
  cache; and
- `CLAUDE_CONFIG_DIR` relocates Claude Code settings, sessions, and plugins and
  is therefore the isolation boundary used by acceptance tests.

References:

- <https://code.claude.com/docs/en/plugins-reference>
- <https://code.claude.com/docs/en/plugin-marketplaces>
- <https://code.claude.com/docs/en/discover-plugins>
- <https://code.claude.com/docs/en/slash-commands>
- <https://code.claude.com/docs/en/env-vars>

Plugins are trusted executable components. R3E does not bypass client trust,
workspace policy, MCP approval, or host-controlled install and enable steps.

## Canonical Native Plugin Package

Each target-specific package has one root directory named `qiongli`:

```text
qiongli/
  .claude-plugin/plugin.json
  .mcp.json
  .qiongli-claude-plugin-bundle.json
  bin/qiongli              # Unix
  bin/qiongli.exe          # Windows
  skills/
    qiongli-workflow/
      SKILL.md
      ...Marketplace Lite content...
```

The canonical `.claude-plugin/plugin.json` resource is a metadata template.
The composer preserves descriptive metadata and deterministically sets the
artifact version, `skills` to `./skills/`, and `mcpServers` to `./.mcp.json`.

The MCP declaration is minimal and platform-specific only at the executable
suffix:

```json
{
  "mcpServers": {
    "qiongli": {
      "command": "${CLAUDE_PLUGIN_ROOT}/bin/qiongli",
      "args": [
        "mcp",
        "serve",
        "--profile",
        "marketplace-lite",
        "--transport",
        "stdio"
      ]
    }
  }
}
```

Windows uses `${CLAUDE_PLUGIN_ROOT}/bin/qiongli.exe`. The command contains no
shell interpolation beyond Claude Code's documented root variable and names no
language runtime, package manager, checkout path, or global executable.

The Marketplace Lite content projection is identical in meaning to R3D:
canonical `workflow/SKILL.md` becomes
`skills/qiongli-workflow/SKILL.md`, other `workflow/` resources lose that
prefix, and remaining profile resources retain their relative paths below the
workflow skill. Platform manifest templates are consumed as metadata and are
not copied into the skill payload.

## Package Trust And Receipt

Composition requires:

1. the verified Marketplace Lite resource pack;
2. a verified launch grant authorizing `LiteMcp` and `ClaudeCodeLocal`;
3. a current-target Lite `PluginBundle` artifact;
4. a regular executable whose SHA-256 matches the signed grant; and
5. an explicitly approved absolute target named `qiongli`.

`.qiongli-claude-plugin-bundle.json` uses canonical RFC 8785 JSON and binds the
signed artifact and grant, resource-pack identity, content root, target,
binary, Claude manifest, MCP declaration, every managed file's mode, size, and
digest, and a domain-separated package content root. It excludes only itself.

Verification rejects missing or extra paths, links and Windows reparse points,
hard links, path or permission drift, oversized input, non-canonical receipts,
and every identity or digest mismatch. Composition uses a private sibling
stage, a target lock, no-replace promotion, parent synchronization, and
post-commit verification. Existing targets are preserved and rejected.

The Claude package implementation is kept separate from the already accepted
Codex package in R3E. A later internal-only refactor may deduplicate mechanical
filesystem code after both host contracts are stable; that refactor is not an
R3E gate and may not weaken either receipt or verifier.

## Personal Skills-directory Path

The caller supplies an already resolved Claude config root from a trusted CLI,
UI, installer, release, or test boundary. Model output and MCP parameters may
not mint this path capability.

The only direct target is:

```text
<claude-config>/skills/qiongli
```

Discovery is read-only and reports symbolic paths only. A healthy direct
installation is an exact verified Claude plugin bundle. An absent target is
missing. Any unreceipted, linked, malformed, or drifted target is a conflict
and is never adopted or overwritten.

Fresh composition is the direct install transaction. Verify revalidates the
whole package. Removal first revalidates the exact receipt-covered tree, moves
it with no-replace semantics to a transaction-owned sibling quarantine,
revalidates it, deletes only that quarantine, and synchronizes the parent.
Failure after an ambiguous move retains recovery evidence and never deletes an
uncertain path. Repair and replacement remain later managed-upgrade work; R3E
does not overwrite an existing direct target.

Claude Code may require `/reload-plugins` or a new local session after install
or removal. That host action is reported, not simulated by undocumented file
writes or UI automation.

## Local Marketplace Path

The Qiongli-owned local marketplace root is fixed below an explicitly approved
managed-data root:

```text
<qiongli-managed>/plugins/claude-code/qiongli-local/
  .claude-plugin/marketplace.json
  plugins/qiongli/
    ...the verified native Claude package...
```

The catalog is canonical JSON with marketplace name `qiongli-local`, Qiongli
owner metadata, and exactly one entry:

```json
{
  "name": "qiongli",
  "source": "./plugins/qiongli"
}
```

The marketplace adapter's declarative `InstallPlan` is user-scoped,
`ClaudeCodeLocal`, Lite, and binds the verified package receipt digest, package
content root, exact catalog entry and document digests, observed state, and
ownership ID `qiongli-claude-code-user`. The plan requires exact filesystem,
client-config, and host-trust approvals and retains
`install-or-enable-plugin` as an outstanding host action.

Apply, verify, missing-catalog repair, remove, and rollback use private adapter
state, a root lock, canonical receipts, a bounded journal, compare-and-swap
document digests, atomic replacement, and post-commit verification. A surviving
journal blocks mutation with `recovery-required`. The adapter owns only this
Qiongli marketplace source; it does not merge into or mutate Claude Code's
client-owned registry.

Client activation uses the documented interface:

```text
claude plugin marketplace add <marketplace-root> --scope user
claude plugin install qiongli@qiongli-local --scope user
```

Those operations are performed only by Claude Code, either as an explicit user
action or in the isolated acceptance harness. Production source builds without
a release grant expose no mutating command that could bypass signing or host
trust.

## CLI Truthfulness

`qiongli install claude status` is read-only. It resolves the process-selected
Claude config root, reports the direct skills-directory state and supported
adapter contract using symbolic paths, and does not create directories or run
Claude Code.

`qiongli install status` may change `claude-code-local` from `contract-only` to
`adapter-engine-ready` only after the deterministic package and adapter tests
pass. Production `launch_grant`, preview, apply, client activation, public
marketplace, and release remain `unavailable` in the ordinary source build.

## Real-client Acceptance

The explicit acceptance harness uses a fresh temporary home and
`CLAUDE_CONFIG_DIR`; it never reads or writes the developer's normal Claude
configuration.

It proves both documented paths:

1. compose and strictly verify a direct skills-directory package with an
   ephemeral test signing key;
2. run `claude plugin validate --strict` and confirm real Claude Code discovers
   `qiongli@skills-dir`;
3. launch the direct plugin MCP with an empty `PATH` and complete MCP
   `initialize` and `tools/list`;
4. compose the same package into an isolated Qiongli local marketplace;
5. ask real Claude Code to add the marketplace and install the plugin;
6. confirm the plugin is listed and copied below the isolated Claude cache;
7. launch the cached MCP with an empty `PATH` and verify the canonical Lite
   tool list; and
8. ask Claude Code to uninstall the plugin and remove the marketplace, then
   prove the isolated client state no longer lists either.

Evidence records the Claude Code version, package receipt digest, symbolic
path shape, plugin ID, and exact Lite tool list. It records no absolute home,
credential, environment snapshot, private key, or normal user configuration.
The external-client test is ignored by default and runs only when the real
Claude CLI is available and explicit acceptance is requested.

## Non-claims

R3E does not prove or provide:

- Claude Desktop direct-plugin or MCPB installation;
- Claude web or cloud access to a local executable;
- public marketplace publication or review;
- cross-target binary selection from one generic marketplace entry;
- managed in-place plugin upgrade or replacement;
- production signing, installers, UI, updater, or release artifacts; or
- Full MCP, project-write, agent, ToolHost, or orchestrator execution.

Claude Desktop remains `INT-203`. Remote Claude sessions remain a separate
remote-service or host-upload program.

## Exit Gate

R3E is complete when:

1. the canonical embedded pack contains the Claude metadata template;
2. deterministic Claude package composition and complete-tree verification
   pass on Tier 1 targets;
3. direct discovery and exact verified removal preserve unmanaged or drifted
   paths;
4. marketplace preview/apply/verify/repair/remove/rollback bind the exact
   package and fail closed on conflicts, tampering, links, partial approval,
   and recovery states;
5. the bundled binary serves the canonical Lite MCP with an empty `PATH`;
6. real isolated Claude Code proves both skills-directory discovery and local
   marketplace install/cache/enable/remove behavior;
7. local native gates and exact-head CI pass; and
8. the rolling Draft PR states these local-only limits without claiming Claude
   Desktop, cloud, publication, Full runtime, release, or UI completion.
