import { fireEvent, render, screen, waitFor } from '@testing-library/svelte';
import { expect, it, vi } from 'vitest';
import AllChat from './AllChat.svelte';
import { chatFixtureTransport } from './dev-chat';

it('supports two labelled turns, permission decisions, cancellation and a new session', async () => {
  render(AllChat, { projectId: `prj_${'0'.repeat(32)}`, projectRevision: 1, sources: ['fixture-source'], onLoadSources: vi.fn(), transport: chatFixtureTransport() });
  const start = screen.getByRole('button', { name: 'Start session' });
  await waitFor(() => expect(start).toBeEnabled());
  await fireEvent.click(start);
  const textbox = screen.getByRole('textbox', { name: 'Message' });
  await waitFor(() => expect(textbox).toBeEnabled());
  await fireEvent.input(textbox, { target: { value: 'Compare the supplied context' } });
  await fireEvent.click(screen.getByRole('button', { name: 'Send' }));
  await waitFor(() => expect(screen.getByRole('heading', { name: 'Permission requested' })).toBeInTheDocument());
  expect(screen.getByRole('status')).toHaveTextContent('Permission requested');
  await fireEvent.click(screen.getByRole('button', { name: 'Deny once' }));
  await waitFor(() => expect(screen.getByText('Turn finished · Session remains open')).toBeInTheDocument());
  expect(textbox).toBeEnabled();
  await fireEvent.input(textbox, { target: { value: 'Try a second turn' } });
  await fireEvent.click(screen.getByRole('button', { name: 'Send' }));
  await waitFor(() => expect(screen.getByRole('button', { name: 'Stop this turn' })).toBeEnabled());
  await fireEvent.click(screen.getByRole('button', { name: 'Stop this turn' }));
  await waitFor(() => expect(screen.getByRole('button', { name: 'Start a new session' })).toBeEnabled());
  expect(screen.getByText('Try a second turn')).toBeInTheDocument();
  expect(textbox).toBeDisabled();
  await fireEvent.click(screen.getByRole('button', { name: 'Start a new session' }));
  await waitFor(() => expect(textbox).toBeEnabled());
  expect(screen.queryByText('Try a second turn')).not.toBeInTheDocument();
});

it('keeps controls disabled when the native response cannot be decoded', async () => {
  render(AllChat, { projectId: `prj_${'0'.repeat(32)}`, projectRevision: 1, onLoadSources: vi.fn(), transport: { async invoke<T>() { return { schemaVersion: 99 } as T; } } });
  await waitFor(() => expect(screen.getByRole('alert')).toBeInTheDocument());
  expect(screen.getByRole('button', { name: 'Start session' })).toBeDisabled();
  expect(screen.getByRole('button', { name: 'Send' })).toBeDisabled();
});


it('shows restored unfinished work as interrupted without reviving permission or send controls', async () => {
  const { default: fixtures } = await import('../../../../../qiongli-native/apps/qiongli/tests/fixtures/all-chat-control-v1.json');
  const restored = { ...fixtures[0], status: 'interrupted', error: 'all-chat-restart-interrupted' };
  render(AllChat, { projectId: restored.projectId, projectRevision: 1, onLoadSources: vi.fn(), transport: { async invoke<T>() { return restored as T; } } });
  await waitFor(() => expect(screen.getByText('Turn interrupted')).toBeInTheDocument());
  expect(screen.getByRole('button', { name: 'Send' })).toBeDisabled();
  expect(screen.queryByRole('button', { name: 'Allow once' })).not.toBeInTheDocument();
  expect(screen.getByRole('button', { name: 'Start a new session' })).toBeEnabled();
});
