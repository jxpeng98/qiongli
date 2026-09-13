<script lang="ts">
  import type {
    ArticleProjectSummary,
    PortfolioQueryFilters,
  } from '@qiongli/app-api';
  import { Filter, RotateCcw, Search } from '@lucide/svelte';

  import { i18n } from '$lib/i18n.svelte';
  import { ActionGroup } from '$lib/components/app';
  import { Button } from '$lib/components/ui/button';
  import * as Card from '$lib/components/ui/card';
  import { Input } from '$lib/components/ui/input';
  import { NativeSelect } from '$lib/components/ui/native-select';

  let {
    projects,
    disabled,
    onApply,
    onReset
  }: {
    projects: ArticleProjectSummary[];
    disabled: boolean;
    onApply: (filters: PortfolioQueryFilters) => void;
    onReset: () => void;
  } = $props();

  let projectId = $state('');
  let stage = $state('');
  let evidenceSignal = $state('');
  let manuscriptSection = $state('');
  let sharedNodeType = $state('');
  let sharedCanonicalId = $state('');
  let captureSource = $state('');
  let captureDelivery = $state('');
  let deliveryState = $state('');
  let assignmentOutcome = $state('');
  let lineageId = $state('');
  let text = $state('');

  function apply(): void {
    const filters: PortfolioQueryFilters = {};
    if (projectId) filters.projectId = projectId as PortfolioQueryFilters['projectId'];
    if (stage) filters.stage = stage as PortfolioQueryFilters['stage'];
    if (evidenceSignal) {
      filters.evidenceSignal = evidenceSignal as PortfolioQueryFilters['evidenceSignal'];
    }
    if (manuscriptSection.trim()) filters.manuscriptSection = manuscriptSection.trim();
    if (sharedNodeType && sharedCanonicalId.trim()) {
      filters.sharedIdentity = {
        nodeType: sharedNodeType as 'paper' | 'concept' | 'method',
        canonicalId: sharedCanonicalId.trim()
      };
    }
    if (captureSource) {
      filters.captureSource = captureSource as PortfolioQueryFilters['captureSource'];
    }
    if (captureDelivery) {
      filters.captureDelivery = captureDelivery as PortfolioQueryFilters['captureDelivery'];
    }
    if (deliveryState) {
      filters.deliveryState = deliveryState as PortfolioQueryFilters['deliveryState'];
    }
    if (assignmentOutcome) {
      filters.assignmentOutcome =
        assignmentOutcome as PortfolioQueryFilters['assignmentOutcome'];
    }
    if (lineageId.trim()) filters.lineageId = lineageId.trim();
    if (text.trim()) filters.text = text.trim();
    onApply(filters);
  }

  function reset(): void {
    projectId = '';
    stage = '';
    evidenceSignal = '';
    manuscriptSection = '';
    sharedNodeType = '';
    sharedCanonicalId = '';
    captureSource = '';
    captureDelivery = '';
    deliveryState = '';
    assignmentOutcome = '';
    lineageId = '';
    text = '';
    onReset();
  }
</script>

<Card.Root class="filters" aria-labelledby="portfolio-filters-title">
  <header>
    <div>
      <p class="eyebrow">{i18n.t('portfolio.filtersEyebrow')}</p>
      <h2 id="portfolio-filters-title">{i18n.t('portfolio.filtersTitle')}</h2>
    </div>
    <span><Filter size={15} aria-hidden="true" />{i18n.t('portfolio.nativeOrder')}</span>
  </header>

  <div class="primary-fields">
    <label>
      <span>{i18n.t('portfolio.textFilter')}</span>
      <Input bind:value={text} maxlength={256} placeholder={i18n.t('portfolio.textPlaceholder')} />
    </label>
    <label>
      <span>{i18n.t('portfolio.projectFilter')}</span>
      <NativeSelect class="portfolio-select" bind:value={projectId}>
        <option value="">{i18n.t('portfolio.allProjects')}</option>
        {#each projects as project}
          <option value={project.projectId}>{project.displayName}</option>
        {/each}
      </NativeSelect>
    </label>
    <label>
      <span>{i18n.t('portfolio.stageFilter')}</span>
      <NativeSelect class="portfolio-select" bind:value={stage}>
        <option value="">{i18n.t('portfolio.anyStage')}</option>
        {#each ['idea', 'framing', 'literature', 'design', 'analysis', 'writing', 'review', 'submission'] as value}
          <option value={value}>{i18n.label(value)}</option>
        {/each}
      </NativeSelect>
    </label>
    <label>
      <span>{i18n.t('portfolio.evidenceFilter')}</span>
      <NativeSelect class="portfolio-select" bind:value={evidenceSignal}>
        <option value="">{i18n.t('portfolio.anyEvidence')}</option>
        <option value="gap">{i18n.label('gap')}</option>
        <option value="contradiction">{i18n.label('contradiction')}</option>
      </NativeSelect>
    </label>
  </div>

  <details>
    <summary>{i18n.t('portfolio.advancedFilters')}</summary>
    <div class="advanced-fields">
      <label>
        <span>{i18n.t('portfolio.manuscriptSection')}</span>
        <Input bind:value={manuscriptSection} maxlength={512} />
      </label>
      <label>
        <span>{i18n.t('portfolio.sharedIdentityType')}</span>
        <NativeSelect class="portfolio-select" bind:value={sharedNodeType}>
          <option value="">{i18n.t('portfolio.anyIdentity')}</option>
          {#each ['paper', 'concept', 'method'] as value}
            <option value={value}>{i18n.label(value)}</option>
          {/each}
        </NativeSelect>
      </label>
      <label>
        <span>{i18n.t('portfolio.sharedCanonicalId')}</span>
        <Input
          bind:value={sharedCanonicalId}
          maxlength={512}
          disabled={!sharedNodeType}
        />
      </label>
      <label>
        <span>{i18n.t('portfolio.captureSource')}</span>
        <NativeSelect class="portfolio-select" bind:value={captureSource}>
          <option value="">{i18n.t('portfolio.anySource')}</option>
          {#each ['codex', 'claude-code', 'claude-desktop', 'other', 'unknown'] as value}
            <option value={value}>{i18n.label(value)}</option>
          {/each}
        </NativeSelect>
      </label>
      <label>
        <span>{i18n.t('portfolio.captureDelivery')}</span>
        <NativeSelect class="portfolio-select" bind:value={captureDelivery}>
          <option value="">{i18n.t('portfolio.anyDelivery')}</option>
          {#each ['connected', 'portable', 'manual', 'unknown'] as value}
            <option value={value}>{i18n.label(value)}</option>
          {/each}
        </NativeSelect>
      </label>
      <label>
        <span>{i18n.t('portfolio.deliveryState')}</span>
        <NativeSelect class="portfolio-select" bind:value={deliveryState}>
          <option value="">{i18n.t('portfolio.anyDeliveryState')}</option>
          {#each ['queued', 'delivering', 'delivered', 'acknowledged', 'retry-required', 'conflicted', 'cancelled'] as value}
            <option value={value}>{i18n.label(value)}</option>
          {/each}
        </NativeSelect>
      </label>
      <label>
        <span>{i18n.t('portfolio.assignmentOutcome')}</span>
        <NativeSelect class="portfolio-select" bind:value={assignmentOutcome}>
          <option value="">{i18n.t('portfolio.anyAssignment')}</option>
          {#each ['assigned', 'rejected', 'resolution-required'] as value}
            <option value={value}>{i18n.label(value)}</option>
          {/each}
        </NativeSelect>
      </label>
      <label>
        <span>{i18n.t('portfolio.lineageId')}</span>
        <Input bind:value={lineageId} maxlength={160} />
      </label>
    </div>
  </details>

  <ActionGroup class="actions" label={i18n.t('portfolio.filtersTitle')}>
    <Button disabled={disabled} onclick={apply}>
      <Search size={16} aria-hidden="true" />{i18n.t('portfolio.applyFilters')}
    </Button>
    <Button variant="outline" disabled={disabled} onclick={reset}>
      <RotateCcw size={16} aria-hidden="true" />{i18n.t('portfolio.resetFilters')}
    </Button>
  </ActionGroup>
</Card.Root>

<style>
  :global(.filters) { min-width: 0; padding: var(--ui-panel-padding); }
  header { display: flex; align-items: flex-start; justify-content: space-between; gap: 12px; }
  h2 { margin: 0; color: var(--color-ink-strong); font-size: 17px; }
  header > span {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    color: var(--color-muted);
    font-size: var(--font-size-label);
  }
  .primary-fields, .advanced-fields {
    display: grid;
    grid-template-columns: minmax(180px, 1.4fr) repeat(3, minmax(130px, 0.8fr));
    gap: 9px;
    margin-top: 13px;
  }
  label { display: grid; min-width: 0; gap: 5px; }
  label > span {
    color: var(--color-muted);
    font-size: var(--font-size-micro);
    font-weight: 750;
    text-transform: uppercase;
  }
  label :global([data-slot='input']), :global(.portfolio-select) { width: 100%; }
  details { margin-top: 10px; border-top: 1px solid var(--color-border); padding-top: 10px; }
  summary { cursor: pointer; color: var(--color-accent-strong); font-size: var(--font-size-supporting); font-weight: 750; }
  .advanced-fields { grid-template-columns: repeat(4, minmax(130px, 1fr)); }
  :global(.actions) { margin-top: 13px; }
  @media (max-width: 980px) {
    .primary-fields, .advanced-fields { grid-template-columns: 1fr 1fr; }
  }
  @media (max-width: 560px) {
    header { flex-direction: column; }
    .primary-fields, .advanced-fields { grid-template-columns: 1fr; }
  }
</style>
