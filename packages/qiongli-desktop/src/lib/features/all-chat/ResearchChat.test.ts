import { fireEvent, render, screen, waitFor } from '@testing-library/svelte';
import { expect, it, vi } from 'vitest';
import type { AppTransport, ChatRequest, ChatSnapshot, ResearchSnapshot } from '@qiongli/app-api';
import { researchContractSchema, researchRequestSchema } from '@qiongli/app-api';
import fixture from '../../../../../qiongli-native/apps/qiongli/tests/fixtures/all-chat-research-v2.json';
import AllChat from './AllChat.svelte';

it('loads explicit excerpts, reviews two native candidates, rejects one and does not restore candidates after restart', async () => {
  const { manifest, candidate } = researchContractSchema.parse(fixture);
  let chat: ChatSnapshot | null = null;
  let research: ResearchSnapshot | null = null;
  const prompts: ChatRequest[] = [];
  const start = vi.fn();
  const review = vi.fn(async () => undefined);
  let researchReads = 0;
  const transport: AppTransport = { async invoke<T>(command: string, args?: Record<string, unknown>) {
    if (command === 'qiongli_all_chat_research') {
      const request = researchRequestSchema.parse(args?.request);
      if (request.type === 'read') researchReads++;
      if (request.type === 'start') {
        start(request);
        chat = { schemaVersion: 1, projectId: manifest.projectId, expectedProjectRevision: 1, runId: candidate.runId,
          agent: 'offline_demo', revision: 1, status: 'idle', nextTurn: 1, prompts: [], updates: [], error: null };
        research = { runId: candidate.runId, manifestDigest: candidate.manifestDigest, manifest, candidate: null, error: null };
      }
      if (request.type === 'dismiss' && research) research.candidate = null;
      return structuredClone(research) as T;
    }
    const request = args?.request as ChatRequest;
    if (request.type === 'prompt' && chat && research) {
      prompts.push(structuredClone(request));
      chat.prompts.push(request.prompt);
      chat.nextTurn++; chat.revision++;
      research.candidate = { ...candidate, turnId: request.expectedTurn };
    }
    return structuredClone(chat) as T;
  } };
  const props = { projectId: manifest.projectId, projectRevision: 1, sources: [manifest.sources[0].selection.artifactPath], onLoadSources: vi.fn(), onReviewCandidate: review, transport };
  const rendered = render(AllChat, props);
  await waitFor(() => expect(screen.getByRole('button', { name: 'Start session' })).toBeEnabled());
  await fireEvent.click(screen.getByText('Sources and context'));
  for (const [index, source] of manifest.sources.entries()) {
    await fireEvent.change(screen.getByRole('combobox', { name: `Excerpt ${index + 1}` }), { target: { value: source.selection.artifactPath } });
    await fireEvent.input(screen.getByRole('spinbutton', { name: `Excerpt ${index + 1} start line` }), { target: { value: source.selection.startLine } });
    await fireEvent.input(screen.getByRole('spinbutton', { name: `Excerpt ${index + 1} end line` }), { target: { value: source.selection.endLine } });
  }
  const startButton = screen.getByRole('button', { name: 'Start excerpt comparison demo' });
  const consent = screen.getByRole('checkbox', { name: 'Allow this local demo to read only these two selected excerpts and the selected method.' });
  expect(startButton).toBeDisabled();
  expect(consent).not.toBeChecked();
  expect(start).not.toHaveBeenCalled();
  await fireEvent.click(consent);
  expect(startButton).toBeEnabled();
  const startLine = screen.getByRole('spinbutton', { name: 'Excerpt 1 start line' });
  await fireEvent.input(startLine, { target: { value: manifest.sources[0].selection.startLine + 1 } });
  expect(startButton).toBeDisabled();
  expect(consent).not.toBeChecked();
  await fireEvent.input(startLine, { target: { value: manifest.sources[0].selection.startLine } });
  expect(consent).toBeChecked();
  expect(startButton).toBeEnabled();
  await fireEvent.click(startButton);
  await waitFor(() => expect(screen.getByRole('heading', { name: 'Loaded research excerpts' })).toBeInTheDocument());
  expect(start).toHaveBeenCalledWith({ type: 'start', projectId: manifest.projectId, expectedProjectRevision: 1, contextAccess: 'selected_excerpts', selections: manifest.sources.map((source) => source.selection) });
  expect(screen.getByText('Read access: 3 resources — two selected excerpts and the selected method.')).toBeInTheDocument();
  expect(screen.queryByText(manifest.readPaths[0])).not.toBeInTheDocument();
  expect(screen.getByText((_text, element) => element?.tagName === 'PRE' && element.textContent === manifest.sources[1].content)).toBeInTheDocument();
  expect(screen.getAllByText('Earlier lines omitted · Later lines omitted')).toHaveLength(1);
  expect(screen.getByText('Earlier lines omitted · Through the source end')).toBeInTheDocument();
  const textbox = screen.getByRole('textbox', { name: 'Message' });
  for (const [index, question] of ['Compare methods', 'Explain the limitations'].entries()) {
    await waitFor(() => expect(textbox).toBeEnabled());
    await fireEvent.input(textbox, { target: { value: question } });
    await fireEvent.click(screen.getByRole('button', { name: 'Send' }));
    await waitFor(() => expect(screen.getByRole('heading', { name: `Research note candidate · Turn ${index + 1}` })).toBeInTheDocument());
    expect(screen.getByRole('textbox', { name: 'Comparison 1' })).toHaveValue(candidate.draft.comparison.text);
    expect(screen.getByRole('textbox', { name: 'Methods 1' })).toHaveValue(candidate.draft.methods[0].text);
    expect(screen.getAllByText(candidate.draft.methods[0].citations[0]!.quote).length).toBeGreaterThan(0);
    if (index === 0) {
      const method = screen.getByRole('textbox', { name: 'Methods 1' });
      await fireEvent.input(method, { target: { value: 'Edited interpretation of the first method.' } });
      const priorReads = researchReads;
      await waitFor(() => expect(researchReads).toBeGreaterThan(priorReads));
      expect(method).toHaveValue('Edited interpretation of the first method.');
      await fireEvent.click(screen.getByRole('button', { name: 'Submit for Capture review' }));
      const edited = structuredClone(candidate);
      edited.turnId = 1; edited.draft.methods[0].text = 'Edited interpretation of the first method.';
      await waitFor(() => expect(review).toHaveBeenCalledWith(edited));
      expect(screen.getByText(/This candidate is not saved/)).toBeInTheDocument();
      await fireEvent.click(screen.getByRole('button', { name: 'Reject candidate' }));
      await waitFor(() => expect(screen.queryByRole('region', { name: 'Research note candidate' })).not.toBeInTheDocument());
    }
  }
  expect(prompts).toEqual(['Compare methods', 'Explain the limitations'].map((text, index) => ({ type: 'prompt', runId: candidate.runId,
    expectedTurn: index + 1, prompt: { text, context: '', sourceRefs: [] } })));
  rendered.unmount();
  if (chat) (chat as ChatSnapshot).status = 'interrupted';
  research = null;
  render(AllChat, props);
  await waitFor(() => expect(screen.getByText(/Research context and unsaved candidates are not restored/)).toBeInTheDocument());
  expect(screen.queryByRole('region', { name: 'Research note candidate' })).not.toBeInTheDocument();
  expect(screen.getByRole('button', { name: 'Send' })).toBeDisabled();
  expect(screen.getByRole('checkbox', { name: 'Allow this local demo to read only these two selected excerpts and the selected method.' })).not.toBeChecked();
});
