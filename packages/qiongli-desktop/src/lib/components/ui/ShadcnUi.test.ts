// @ts-expect-error Vitest runs this source contract in Node; the Desktop
// production bundle intentionally does not depend on Node type declarations.
import { readFileSync } from 'node:fs';
import { render, screen } from '@testing-library/svelte';
import { createRawSnippet } from 'svelte';
import { describe, expect, it } from 'vitest';

import { Button } from './button';
import * as Card from './card';

describe('shadcn-svelte foundation', () => {
  it('locks component generation to the Rhea and Neutral baseline', () => {
    const config = JSON.parse(
      readFileSync('components.json', 'utf8')
    ) as { style?: string; tailwind?: { baseColor?: string } };

    expect(config.style).toBe('rhea');
    expect(config.tailwind?.baseColor).toBe('neutral');
  });

  it('renders the shared button contract with Qiongli semantic variants', () => {
    const children = createRawSnippet(() => ({ render: () => 'Continue' }));

    render(Button, {
      children,
      variant: 'default',
      disabled: true
    });

    const button = screen.getByRole('button', { name: 'Continue' });
    expect(button).toBeDisabled();
    expect(button).toHaveAttribute('data-slot', 'button');
    expect(button).toHaveClass('bg-primary');
    expect(button).toHaveClass('rounded-2xl');
  });

  it('composes card anatomy from the shared component boundary', () => {
    const children = createRawSnippet(() => ({
      render: () => '<section><h2>Research status</h2><p>Ready for review</p></section>'
    }));

    const { container } = render(Card.Root, { children });

    expect(screen.getByRole('heading', { level: 2, name: 'Research status' }))
      .toBeVisible();
    expect(screen.getByText('Ready for review')).toBeVisible();
    expect(container.querySelector('[data-slot="card"]')).toBeVisible();
  });
});
