import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';

import StatusBadge from './StatusBadge.svelte';

describe('StatusBadge', () => {
  it('renders a readable status label instead of color alone', () => {
    render(StatusBadge, { status: 'recovery-required' });
    expect(screen.getByText('Recovery required')).toBeInTheDocument();
  });

  it('accepts a product-specific label', () => {
    render(StatusBadge, { status: 'write-unsupported', label: 'Inspect only' });
    const label = screen.getByText('Inspect only');
    expect(label).toBeInTheDocument();
    expect(label.closest('.status')).toHaveAttribute('title', 'Inspect only');
  });
});
