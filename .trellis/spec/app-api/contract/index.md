# App API Contract

`packages/qiongli-app-api/src/schema.ts` is the TypeScript decoder for the
native App snapshot and intent/event protocol. The native fixture comes from
`app_api_contract_fixture_json()` in
`packages/qiongli-native/apps/qiongli/src/desktop.rs`.

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

## Scenario: Alpha 3 provider fields and project guidance

### 1. Scope / Trigger

App API schema version 17 adds native-declared literature-provider fields and
bounded Plugin/Skill preview with advisory project-local guidance. Use this
contract whenever either wire shape changes.

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
- Content preview exposes only `workflow/SKILL.md`,
  `.codex-plugin/plugin.json`, and `.claude-plugin/plugin.json`, each at most
  128 KiB and valid UTF-8.
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
| Empty, oversized, or invalid-control guidance | `project-guidance-invalid` |
| Guidance SHA differs from loaded content | `project-guidance-revision-conflict` |
| Project is missing, archived, or unhealthy | fail closed before preview |
| File changes after preview | `revision-conflict` at project storage |

### 5. Good / Base / Bad Cases

- Good: render PubMed API Key from native state; preview secret-store write;
  confirm with the exact operation token.
- Good: preview embedded Skill text; write guidance to a ready registered
  project with the matching content digest.
- Base: arXiv renders no configuration input and remains testable when enabled.
- Bad: invent a provider field in Svelte, expose an absolute project path, or
  overwrite bundled Plugin/Skill files.

### 6. Tests Required

- App API fixture: assert schema version 17 and the exact provider-field table.
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

Correct: preview and confirm only
`<project>/.qiongli/local_guidance.md` with its expected SHA-256.
