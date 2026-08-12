# Bug Analysis: Full MCP returned the Marketplace Lite route

## 1. Root Cause Category

- **Category**: B/D — Cross-layer contract plus integration coverage gap.
- **Specific cause**: `FullMcpServer` composed `LiteMcpServer` and delegated
  every shared tool that was not a project/control tool. The shared name
  `qiongli_orchestrator_route` is profile-sensitive, so Full returned Lite's
  preview/upgrade payload even while Full host tools were active. Existing tests
  checked tool inventory and each profile separately, not the shared name's Full
  response.

## 2. Why Earlier Fix Directions Failed

1. Treating the symptom as Skill wording would still leave every Host caller
   receiving the wrong Full response.
2. Changing the frozen legacy Python/Contract v2 input surface expanded scope
   without fixing native delegation; that draft was reverted.
3. Intercepting the shared name at the native Full owner fixed all Full callers
   while preserving Lite validation and Lite behavior.

## 3. Prevention Mechanisms

| Priority | Mechanism | Specific action | Status |
|---|---|---|---|
| P0 | Architecture | Intercept profile-sensitive names before generic Lite delegation | DONE |
| P0 | Test coverage | Full copied-binary regression asserts the host tool sequence and absence of Lite upgrade fields | DONE |
| P0 | Documentation | Add the executable Full profile-routing code-spec and cross-layer checklist | DONE |
| P1 | Review | Audit any future shared Full/Lite name for profile-specific response schemas | DONE |

## 4. Systematic Expansion

- **Similar issues**: `qiongli_task_plan` still exposes a legacy Lite preview in
  native Full, but it is no longer in the canonical host-driven route. Treat it
  as Contract v2 governance work, not an Alpha 3 first-use blocker.
- **Design improvement**: Keep common validation reusable, but make profile
  selection explicit at the outer server boundary.
- **Process improvement**: For composed MCP profiles, test one call result per
  profile-sensitive shared name; `tools/list` parity alone is insufficient.

## 5. Knowledge Capture

- [x] Added `.trellis/spec/native/runtime/full-mcp-profile-routing.md`.
- [x] Updated `.trellis/spec/guides/cross-layer-thinking-guide.md`.
- [x] Added the copied Full binary regression and retained the Lite regression.
- [x] Updated canonical Skill routing and regenerated the embedded pack lock.
- [x] No spec template mirror exists in this project, so no template sync applies.
