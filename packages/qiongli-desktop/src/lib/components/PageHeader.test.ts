import { render, screen } from '@testing-library/svelte';
import { createRawSnippet } from 'svelte';
import { describe, expect, it } from 'vitest';

import PageHeader from './PageHeader.svelte';

describe('PageHeader', () => {
  it('keeps the complete description accessible while exposing actions in one group', () => {
    const description = 'A long control-plane description that remains available to assistive technology even when the compact visual header limits its displayed lines.';
    const actions = createRawSnippet(() => ({
      render: () => '<button type="button">Refresh discovery</button>'
    }));

    const { container } = render(PageHeader, {
      eyebrow: 'Client integrations',
      title: 'Connect Qiongli',
      description,
      actions
    });

    expect(screen.getByRole('heading', { level: 1, name: 'Connect Qiongli' })).toBeVisible();
    expect(screen.getByText(description)).toHaveTextContent(description);
    expect(screen.getByRole('button', { name: 'Refresh discovery' })).toBeVisible();
    expect(container.querySelectorAll('.actions')).toHaveLength(1);
  });
});
