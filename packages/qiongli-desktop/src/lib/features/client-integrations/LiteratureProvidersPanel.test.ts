import { render, screen } from '@testing-library/svelte';
import { QiongliAppClient } from '@qiongli/app-api';
import { beforeEach, describe, expect, it } from 'vitest';

import { AppState } from '$lib/app-state.svelte';
import { sourceFixtureTransport } from '$lib/dev-transport';
import LiteratureProvidersPanel from './LiteratureProvidersPanel.svelte';

describe('LiteratureProvidersPanel', () => {
  let app: AppState;

  beforeEach(async () => {
    app = new AppState(new QiongliAppClient(sourceFixtureTransport()));
    await app.refresh();
  });

  it('renders exactly the configuration fields declared by native state', () => {
    render(LiteratureProvidersPanel, { appState: app });

    expect(screen.getByLabelText('OpenAlex API key')).toBeVisible();
    expect(screen.getByLabelText('OpenAlex Contact email')).toBeVisible();
    expect(screen.getByLabelText('Semantic Scholar API key')).toBeVisible();
    expect(screen.getByLabelText('Crossref Contact email')).toBeVisible();
    expect(screen.getByLabelText('PubMed API key')).toBeVisible();
    expect(screen.queryByLabelText('arXiv API key')).not.toBeInTheDocument();
    expect(screen.queryByLabelText('arXiv Contact email')).not.toBeInTheDocument();
  });
});
