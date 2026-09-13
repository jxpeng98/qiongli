// @ts-expect-error Vitest runs this source contract in Node; the Desktop
// production bundle intentionally does not depend on Node type declarations.
import { readFileSync } from 'node:fs';
import { render, screen } from '@testing-library/svelte';
import { createRawSnippet } from 'svelte';
import { describe, expect, it } from 'vitest';

import { Button } from './button';
import * as Card from './card';

describe('shadcn-svelte foundation', () => {
  it('exposes the Desktop frontend dev server from the repository root', () => {
    const rootPackage = JSON.parse(
      readFileSync('../../package.json', 'utf8')
    ) as { scripts?: { dev?: string } };

    expect(rootPackage.scripts?.dev).toBe(
      'pnpm --dir packages/qiongli-desktop exec vite dev --host 127.0.0.1 --port 1421'
    );
  });

  it('locks component generation to the Nova and Neutral baseline', () => {
    const config = JSON.parse(
      readFileSync('components.json', 'utf8')
    ) as { style?: string; tailwind?: { baseColor?: string } };

    expect(config.style).toBe('nova');
    expect(config.tailwind?.baseColor).toBe('neutral');
  });

  it('renders the shared button contract with Qiongli semantic variants', () => {
    const children = createRawSnippet(() => ({ render: () => '<span>Continue</span>' }));

    render(Button, {
      children,
      variant: 'default',
      disabled: true
    });

    const button = screen.getByRole('button', { name: 'Continue' });
    expect(button).toBeDisabled();
    expect(button).toHaveAttribute('data-slot', 'button');
    expect(button).toHaveClass('bg-primary');
    expect(button).toHaveClass('rounded-[var(--radius-control)]');
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

  it('uses a high-contrast semantic highlight for selected tabs', () => {
    const tabsTrigger = readFileSync('src/lib/components/ui/tabs/tabs-trigger.svelte', 'utf8');

    expect(tabsTrigger).toContain('data-active:bg-primary');
    expect(tabsTrigger).toContain('data-active:text-primary-foreground');
    expect(tabsTrigger).toContain('motion-reduce:transition-none');
  });
});
