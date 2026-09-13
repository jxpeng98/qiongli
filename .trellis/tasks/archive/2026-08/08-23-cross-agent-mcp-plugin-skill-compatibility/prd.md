# Codex and Claude MCP, Plugin, and Skill compatibility

## Goal

Establish truthful Qiongli 2 compatibility for Codex and Claude Code: each
client must recognize the receipt-owned Qiongli Plugin and embedded Skill, and
the same bundle-contained native executable must successfully expose both Lite
and Full MCP profiles from isolated client configurations.

The user value is confidence that Qiongli is not merely an MCP server that works
in a direct protocol test: its Plugin, Skill, Lite MCP, and Full MCP surfaces
also fit the two supported Agent hosts and their conventional paths.

## Background and Confirmed Facts

- PLT-320 accepted the native `CLI -> Plugin/Skills -> Lite/Full MCP -> Zotero`
  replacement vertical, and PLT-321 accepted the App-to-Full-MCP self-test.
- Qiongli 2 has one native MCP executable with `lite`, `marketplace-lite`, and
  `full` profiles. Codex and Claude Code bundles already point at that
  executable; no agent-specific MCP implementation is needed.
- Native activation, discovery, repair, removal, receipts, and real-client
  tests already exist for Codex and Claude Code under disposable homes.
- The current real-client tests prove Plugin installation, embedded Skill
  discovery, client cache identity, and Full MCP startup. They do not yet prove
  a symmetric Lite-and-Full MCP matrix for both clients.
- The native client inventory already recognizes Codex user/project Skills at
  `~/.agents/skills` and `<project>/.agents/skills`, and Claude Code
  user/project Skills at `${CLAUDE_CONFIG_DIR:-~/.claude}/skills` and
  `<project>/.claude/skills`.
- The referenced `ui-ux-pro-max-skill` installer uses `.agents/skills`
  (plural) for Codex, Antigravity, and the universal Agent Skills convention;
  `.claude/skills` for Claude Code; `.cursor/skills` for Cursor; and
  `.gemini/skills` for Gemini. It generates platform-specific Skill projections
  from one source but does not define one common cross-agent Plugin or MCP
  protocol.
- The user selected Codex and Claude Code as the complete first compatibility
  matrix. Other agents are explicitly deferred.

## Requirements

### R1 — Freeze the two-client compatibility matrix

- Document the authoritative Qiongli 2 user/project Skill roots, managed Plugin
  source/registration locations, and MCP descriptor/profile behavior for Codex
  and Claude Code.
- State explicitly that `.agents` is plural, `.agent` is not a Qiongli 2 target,
  and legacy `.codex/skills` observations do not qualify the current Codex
  Skill path.
- Keep Host-owned cache version directories descriptive rather than treating
  them as stable write targets; Qiongli must not write client caches directly.

### R2 — Prove Plugin and Skill discovery through both clients

- Reuse the existing receipt-owned Codex personal-marketplace flow and Claude
  local-marketplace/user-scope flow.
- Require the real client CLI to list the installed Plugin at the exact version
  and enabled scope.
- Require the client-visible installed/cache bundle to contain and report the
  canonical `qiongli-workflow` Skill with the expected workflow-variant digest.
- Remove the test Plugin and verify absence before the isolated fixture ends.

### R3 — Prove Lite and Full MCP compatibility symmetrically

- For both Codex and Claude Code, use the official client CLI under a disposable
  home/config root to register or inspect bounded stdio MCP entries for the
  bundle-contained native executable.
- Exercise `initialize` and `tools/list` against both `lite` and `full` using
  `PATH`-empty isolated execution.
- Lite must expose exactly the Lite public tool registry and exclude Full-only
  project/Host tools.
- Full must expose the exact Lite + Full project + Full Host-orchestration
  registry and retain the Full profile-sensitive route behavior.
- Client configuration recognition is not an authenticated model-session claim.
  Where a client CLI health-checks MCP entries, capture that evidence; otherwise
  pair client inventory evidence with the exact protocol launch.

### R4 — Preserve trust and ownership boundaries

- Use only temporary homes, config roots, workspaces, marketplaces, Plugin
  caches, and Qiongli config roots.
- Do not read or mutate normal Codex/Claude profiles, authentication, prompts,
  responses, or provider secrets.
- Preserve preview/digest approvals, receipt-bound repair/removal, official Host
  CLI allowlists, no-shell execution, output/time bounds, path redaction, and
  fail-closed conflict behavior.
- Add no dependency, second runtime, generic command runner, direct cache writer,
  or universal agent-path registry.

### R5 — Record bounded evidence

- Run focused deterministic MCP and client-inventory checks plus the two ignored
  real-client compatibility tests.
- Run the affected native Slice checks and exact-head CI after the product test
  change is frozen.
- Record a path-redacted acceptance note bound to the exact product commit,
  supported Codex/Claude versions, and test/CI results.
- Make no release, package, publication, authenticated model execution, or
  unsupported-agent claim.

## Acceptance Criteria

- [x] The 2.x compatibility matrix names Codex and Claude Code Plugin, Skill,
      Lite MCP, and Full MCP locations/entry points, including user/project
      scope and the `.agents` plural rule.
- [x] A current Codex CLI, under an isolated home, installs/lists/removes the
      receipt-owned Plugin, observes its MCP registration, preserves the
      embedded customized Skill, and recognizes isolated Lite and Full MCP
      configurations.
- [x] A current Claude Code CLI, under an isolated config root, validates and
      installs/lists/removes the receipt-owned Plugin, reports its Skill and MCP
      components, and recognizes/health-checks isolated Lite and Full MCP
      configurations.
- [x] In both client fixtures, Lite returns exactly the Lite registry and Full
      returns the exact combined registry; Lite excludes a representative
      Full-only tool and Full retains its non-Lite route response.
- [x] Conflicting, unmanaged, stale, unsupported, or partial Plugin/Skill state
      continues to fail closed, and removal touches only receipt-owned state.
- [x] Tests prove that normal `.agents`, `.claude`, `.codex`, credentials, and
      user projects were not inputs or mutation targets.
- [x] Focused checks, affected native Slice checks, and exact-head CI pass, and
      the acceptance note makes only the claims supported by that evidence.

## Out of Scope

- Cursor, Gemini, Antigravity, Windsurf, OpenCode, or any other additional
  Agent. Their path templates may inform later work but are not supported by
  this Slice.
- Simultaneously installing Lite and Full into a normal user profile, changing
  Full MCP as the Qiongli 2 Client Ready gate, or adding a new App mode picker.
- A second MCP runtime, remote MCP hosting, authenticated model conversations,
  arbitrary shell execution, or direct writes to Host caches.
- New MCP tools/schemas, new Plugin content, broad installer refactoring,
  candidate packaging, signing, promotion, publication, or release approval.

## Key Decisions

- Tier 1 contains only Codex and Claude Code; later agents require their own
  product decision and live-host contract.
- Existing native inventory, bundle composers, official Host CLI plans, and
  receipts remain the owners. This task adds evidence and fixes only a mismatch
  exposed by that evidence.
- Full remains the production integration-readiness profile. Lite compatibility
  is proven as a separately configured bounded profile in isolated clients; the
  task does not install two production servers side by side.
- No new machine-readable path registry is introduced. The existing native
  path inventory is authoritative; the 2.x installation guide publishes the
  human-readable matrix.

## References

- https://github.com/nextlevelbuilder/ui-ux-pro-max-skill#using-cli-recommended
- https://github.com/nextlevelbuilder/ui-ux-pro-max-skill/blob/main/cli/assets/templates/platforms/codex.json
- https://github.com/nextlevelbuilder/ui-ux-pro-max-skill/blob/main/cli/assets/templates/platforms/claude.json
- https://github.com/nextlevelbuilder/ui-ux-pro-max-skill/blob/main/cli/assets/templates/platforms/universal.json
