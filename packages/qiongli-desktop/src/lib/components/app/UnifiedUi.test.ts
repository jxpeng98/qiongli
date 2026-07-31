import { render, screen } from '@testing-library/svelte';
import { createRawSnippet } from 'svelte';
import { describe, expect, it } from 'vitest';

import MetricCard from './MetricCard.svelte';
import MetricGrid from './MetricGrid.svelte';
import InfoGrid from './InfoGrid.svelte';
import SectionHeader from './SectionHeader.svelte';
import StatePanel from './StatePanel.svelte';

describe('unified UI primitives', () => {
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
});
