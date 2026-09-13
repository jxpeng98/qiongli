# App API Contract

`packages/qiongli-app-api/src/schema.ts` is the TypeScript decoder for the
native App snapshot and intent/event protocol. The native fixture comes from
`app_api_contract_fixture_json()` in
`packages/qiongli-native/apps/qiongli/src/desktop.rs`.

[All Chat App contract v1](./all-chat-app-v1.md) is a separate Rust-owned
boundary for committed All Chat State. It does not extend or renumber the
frozen schema 19 protocol.

## Rules

- Decode unknown native data at this boundary; UI components consume typed data.
- Change the contract version only for a wire-shape change and update the Rust
  fixture plus `packages/qiongli-app-api/tests/client.test.ts` in the same slice.
- Do not invent frontend-only provider fields, actions, or readiness states.
- New write operations must expose preview and apply as distinct intents/events.

## Pre-Development Checklist

- Find every producer and consumer of the changed field or intent.
- Confirm backward/fail-closed behavior for missing or unknown data.

## Quality Check

- `pnpm --dir packages/qiongli-app-api test`
- Verify the Rust fixture decodes without casts or duplicated defaults.

## Scenario: App Full MCP self-test

### 1. Scope / Trigger

App API schema version 19 exposes the native bounded Full MCP self-test. Use
this contract whenever its intents, event, checks, or frontend state change.

### 2. Signatures

```typescript
type FullMcpSelfTestIntent =
  | { action: 'run-full-mcp-self-test' }
  | { action: 'poll-full-mcp-self-test' }
  | { action: 'cancel-full-mcp-self-test' };

type McpSelfTestEvent = {
  type: 'mcp-self-test-updated';
  selfTest: McpSelfTestView;
};
```

`McpSelfTestView.profile` is the literal `full`; its six check IDs are the exact
ordered tuple `embedded-contract`, `initialize`, `tool-registry`,
`full-dispatch`, `provider-readiness`, and `client-registration`.

### 3. Contracts

- Native stdio and the App self-test share `mcp::full_server`.
- `tool-registry` validates the exact ordered union of Lite, Full project, and
  Full Host-orchestration public tool constants; the count is derived from the
  same arrays.
- `full-dispatch` calls `qiongli_orchestrator_route` and requires
  `route=orchestrator_mcp`, `requires_full_runtime=true`, and no Lite `upgrade`
  or `preview_only` result.
- The test is offline, bounded, cancellable, and does not resolve credentials.
- A passed self-test reports embedded Full runtime health only. It never mutates
  or promotes integration readiness; fresh Host/receipt probes remain the
  authority for Ready.

### 4. Validation & Error Matrix

| Condition | Result |
| --- | --- |
| Profile is not `full` | App API decode fails closed |
| Check tuple is missing, reordered, or unknown | App API decode fails closed |
| Ready providers exceed enabled providers | App API decode fails closed |
| Registered clients exceed discovered clients | App API decode fails closed |
| Registry or Full-only dispatch differs | Native check is non-Ready and overall state is `failed` |
| Five-second bound expires | State is `timed-out` |
| User cancels a running test | State is `cancelled` |

### 5. Good / Base / Bad Cases

- Good: a connected Host integration stays Ready and a separate Full self-test
  passes with the exact combined registry and Full-only route.
- Base: provider/client checks can be non-Ready while the offline protocol and
  Full registry checks remain truthful.
- Bad: infer Full health from the Lite registry, execute every tool, read
  credentials, or change an integration from stale/error to Ready after a
  self-test.

### 6. Tests Required

- Native: exact combined registry, Full-only route, no credential reads,
  cancellation, and timeout.
- App API: Rust fixture and TypeScript decoder agree on schema version 19 and
  the strict six-check event.
- Desktop: state stores the event without changing snapshot integration
  evidence; Svelte check/tests/build pass.

### 7. Wrong vs Correct

Wrong: call `LiteMcpServer`, then label a successful Lite task-plan dispatch as
Full.

Correct: construct the shared `FullMcpServer`, compare all authoritative public
tool arrays, and exercise `qiongli_orchestrator_route` with its Full-only result
shape.

## Scenario: Alpha 3 provider fields, workflow variants, and project guidance

### 1. Scope / Trigger

App API schema version 18 adds native-declared literature-provider fields,
receipt-owned Workflow/Skill variants, bounded Plugin preview, and advisory
project-local guidance. Use this contract whenever any of these wire shapes
change.

### 2. Signatures

```typescript
type ProviderConfigurationField = {
  field: 'api-key' | 'email';
  configured: boolean;
};

type ProviderView = {
  provider: 'openalex' | 'semantic-scholar' | 'crossref' | 'pubmed' | 'arxiv';
  enabled: boolean;
  readiness: 'disabled' | 'ready' | 'needs-secret' | 'needs-public-setting' | 'unavailable';
  configurationFields: ProviderConfigurationField[];
};

type ContentCustomizationIntent = {
  action: 'load-content-customization';
  profile: 'skill-only' | 'marketplace-lite' | 'full';
  projectId: string | null;
};

type ContentPreviewResource = {
  path: string;
  format: 'markdown' | 'json';
  editable: boolean;
  canonicalSha256: string;
  currentSha256: string;
  overridden: boolean;
  content: string;
};

type WorkflowVariantIntent = {
  action: 'preview-workflow-resource-replace' | 'preview-workflow-resource-reset';
  expectedRevision: number;
  expectedVariantSha256: string | null;
  path: string;
  expectedCurrentSha256: string;
  content?: string;
};

type ProjectGuidanceIntent = {
  action: 'preview-project-guidance';
  projectId: string;
  expectedSha256: string | null;
  content: string;
};
```

Native project storage remains owned by:

```rust
ProjectStateService::read_local_guidance(&ProjectId) -> Result<Option<String>, ProjectError>
ProjectStateService::replace_local_guidance(&ProjectId, Option<&str>, &str)
    -> Result<String, ProjectError>
```

### 3. Contracts

| Provider | Native `configurationFields` |
| --- | --- |
| OpenAlex | `api-key`, `email` |
| Semantic Scholar | `api-key` |
| Crossref | `email` |
| PubMed | `api-key` |
| arXiv | empty |

- The Desktop iterates `configurationFields`; it does not infer fields from a
  provider ID.
- `preview-provider-secret-change` accepts only native-supported API-key pairs.
- `preview-provider-settings.publicSettingChanges` accepts at most two changes;
  only OpenAlex and Crossref email are supported and duplicate providers fail.
- Content preview exposes editable `workflow/SKILL.md` and `skills/**/*.md`
  resources plus read-only Codex/Claude Plugin manifests. Every resource is at
  most 128 KiB and valid UTF-8.
- Replace/reset accepts only the editable Markdown paths and exact loaded
  revision, variant digest, and current resource digest. Confirmation writes
  only the private variant state; installed destinations require a separate
  explicit reconcile.
- Guidance exposes only `<project>/.qiongli/local_guidance.md`. Content is
  1..32768 UTF-8 bytes, permits only tab/newline control characters, and uses
  the loaded content SHA-256 as compare-and-swap input.
- Confirmation reuses `confirm-operation`; canonical embedded content is never
  a write target.

### 4. Validation & Error Matrix

| Condition | Result |
| --- | --- |
| Unsupported public-setting provider | `provider-public-setting-unsupported` |
| Duplicate public-setting provider | `provider-public-setting-change-duplicate` |
| Unsupported API-key provider | `provider-secret-unsupported` |
| Preview resource exceeds 128 KiB | `content-preview-too-large` |
| Preview resource is not UTF-8 | `content-preview-not-utf8` |
| Unsupported resource path or non-Markdown edit | `workflow-variant-reference-invalid` |
| Invalid UTF-8, control characters, or size bounds | `workflow-variant-content-invalid` |
| Variant revision/base/current digest changed | fail closed with the matching revision/digest conflict |
| Empty, oversized, or invalid-control guidance | `project-guidance-invalid` |
| Guidance SHA differs from loaded content | `project-guidance-revision-conflict` |
| Project is missing, archived, or unhealthy | fail closed before preview |
| File changes after preview | `revision-conflict` at project storage |

### 5. Good / Base / Bad Cases

- Good: render PubMed API Key from native state; preview secret-store write;
  confirm with the exact operation token.
- Good: preview an embedded Skill, save a receipt-owned local variant, then
  explicitly reconcile affected managed destinations.
- Good: write guidance to a ready registered project with the matching content
  digest; guidance remains separate from the shared variant.
- Base: arXiv renders no configuration input and remains testable when enabled.
- Bad: invent a provider field in Svelte, expose an absolute project path,
  overwrite bundled/installed Plugin files, or treat a saved variant as active
  before reconciliation.

### 6. Tests Required

- App API fixture: assert schema version 18, the exact provider-field table,
  dynamic bounded resources, and both variant preview intents.
- Desktop component: assert all five declared inputs render and arXiv renders
  none; assert guidance preview targets only the symbolic local-guidance path.
- Native provider tests: assert public-setting routing and PubMed secret
  save/restart/remove without raw-secret persistence.
- Project storage test: assert size/control bounds and stale-digest rejection.
- Native preview test: assert oversized and non-UTF-8 resources fail.

### 7. Wrong vs Correct

Wrong:

```svelte
{#if provider.provider === 'openalex'}<input type="password" />{/if}
```

Correct:

```svelte
{#each provider.configurationFields as field (field.field)}
  <Input type={field.field === 'api-key' ? 'password' : 'email'} />
{/each}
```

Wrong: write an edited `workflow/SKILL.md` back into installed content.

Correct: preview and confirm the private receipt-owned workflow variant with
revision/base/current digests, then use a separate approved reconciliation to
derive installed content. Project-only advice continues to use
`<project>/.qiongli/local_guidance.md` with its expected SHA-256.
