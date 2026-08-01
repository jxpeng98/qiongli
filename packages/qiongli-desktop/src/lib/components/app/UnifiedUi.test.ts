import { render, screen } from '@testing-library/svelte';
// @ts-expect-error Vitest runs this source contract in Node; the Desktop
// production bundle intentionally does not depend on Node type declarations.
import { readFileSync } from 'node:fs';
import { createRawSnippet } from 'svelte';
import { describe, expect, it } from 'vitest';

import MetricCard from './MetricCard.svelte';
import MetricGrid from './MetricGrid.svelte';
import InfoGrid from './InfoGrid.svelte';
import ContentGrid from './ContentGrid.svelte';
import DescriptionGrid from './DescriptionGrid.svelte';
import SectionHeader from './SectionHeader.svelte';
import PageLayout from './PageLayout.svelte';
import StatePanel from './StatePanel.svelte';

describe('unified UI primitives', () => {
  it('uses one page composition contract for every routed workspace', () => {
    const actions = createRawSnippet(() => ({ render: () => '<button type="button">Refresh</button>' }));
    const children = createRawSnippet(() => ({ render: () => '<section>Workspace content</section>' }));
    const { container } = render(PageLayout, {
      eyebrow: 'Overview',
      title: 'Research system',
      description: 'Shared page composition.',
      actions,
      children
    });

    expect(screen.getByRole('heading', { level: 1, name: 'Research system' })).toBeVisible();
    expect(screen.getByRole('button', { name: 'Refresh' })).toBeVisible();
    expect(container.querySelector('[data-slot="page-content"]')).toHaveTextContent('Workspace content');

    for (const route of [
      'overview',
      'about',
      'client-integrations',
      'research-library',
      'artifacts',
      'captures',
      'academic-graph',
      'portfolio',
      'timeline',
      'orchestrator'
    ]) {
      const page = readFileSync(`src/routes/${route}/+page.svelte`, 'utf8');
      expect(page, route).toContain('<PageLayout');
      expect(page, route).not.toContain('<PageHeader');
    }
  });

  it('renders section hierarchy, metadata, and actions through one header API', () => {
    const metadata = createRawSnippet(() => ({ render: () => '<span>Ready</span>' }));
    const actions = createRawSnippet(() => ({
      render: () => '<button type="button">Refresh</button>'
    }));

    render(SectionHeader, {
      eyebrow: 'Evidence',
      title: 'Academic graph',
      level: 3,
      description: 'Inspect connected research claims.',
      metadata,
      actions
    });

    expect(screen.getByRole('heading', { level: 3, name: 'Academic graph' })).toBeVisible();
    expect(screen.getByText('Ready')).toBeVisible();
    expect(screen.getByRole('button', { name: 'Refresh' })).toBeVisible();
  });

  it('announces dangerous states without relying on page-specific markup', () => {
    const { container } = render(StatePanel, {
      tone: 'danger',
      role: 'alert',
      title: 'Inspection failed',
      description: 'Try the inspection again.'
    });

    expect(screen.getByRole('alert')).toHaveTextContent('Inspection failed');
    expect(screen.getByRole('alert')).toHaveTextContent('Try the inspection again.');
    expect(container.querySelector('.state-panel')).toHaveClass('danger');
  });

  it('uses the same metric card and responsive grid contract everywhere', () => {
    const children = createRawSnippet(() => ({
      render: () => '<span>Metric content</span>'
    }));
    const grid = render(MetricGrid, { label: 'Summary', children });

    expect(screen.getByRole('region', { name: 'Summary' })).toBeVisible();
    grid.unmount();

    const { container } = render(MetricCard, {
      value: 12,
      label: 'Projects',
      tone: 'success'
    });
    expect(screen.getByText('12')).toBeVisible();
    expect(screen.getByText('Projects')).toBeVisible();
    expect(container.querySelector('.metric-card')).toHaveClass('success');
  });

  it('groups related facts into one restrained surface with internal dividers', () => {
    const children = createRawSnippet(() => ({
      render: () => '<div><article>Focal question</article><article>Working thesis</article></div>'
    }));

    const { container } = render(InfoGrid, {
      columns: 2,
      compact: true,
      'aria-label': 'Research summary',
      children
    });

    const grid = screen.getByLabelText('Research summary');
    expect(grid).toHaveAttribute('data-slot', 'info-grid');
    expect(grid).toHaveAttribute('data-columns', '2');
    expect(grid).toHaveAttribute('data-compact', 'true');
    expect(container.querySelectorAll('article')).toHaveLength(2);
  });

  it('shares responsive content and semantic description grids across features', () => {
    const content = createRawSnippet(() => ({
      render: () => '<article>One</article><article>Two</article><article>Three</article>'
    }));
    const grid = render(ContentGrid, {
      columns: 3,
      collapse: 'sm',
      lastSpan: 2,
      children: content
    });

    expect(grid.container.querySelector('[data-slot="content-grid"]')).toHaveAttribute('data-columns', '3');
    expect(grid.container.querySelector('[data-slot="content-grid"]')).toHaveAttribute('data-last-span', '2');
    grid.unmount();

    const facts = createRawSnippet(() => ({
      render: () => '<div><dt>Version</dt><dd>2.0</dd></div>'
    }));
    render(DescriptionGrid, { columns: 1, compact: true, children: facts });
    expect(screen.getByRole('term')).toHaveTextContent('Version');
    expect(screen.getByRole('definition')).toHaveTextContent('2.0');
  });
});
