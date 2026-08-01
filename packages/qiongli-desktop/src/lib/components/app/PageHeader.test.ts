import { fireEvent, render, screen } from '@testing-library/svelte';
import { createRawSnippet } from 'svelte';
import { describe, expect, it } from 'vitest';

import PageHeader from './PageHeader.svelte';

describe('PageHeader', () => {
  it('moves supporting copy into a focusable tip while exposing actions in one group', async () => {
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
    expect(container.querySelector('.description-sr')).toHaveTextContent(description);
    const descriptionTrigger = screen.getByRole('button', { name: 'More information' });
    await fireEvent.focus(descriptionTrigger);
    expect(await screen.findByRole('tooltip')).toHaveTextContent(description);
    expect(screen.getByRole('button', { name: 'Refresh discovery' })).toBeVisible();
    expect(container.querySelectorAll('.actions')).toHaveLength(1);
  });
});
