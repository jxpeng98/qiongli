import { fireEvent, render, screen } from '@testing-library/svelte';
import { QiongliAppClient } from '@qiongli/app-api';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { AppState } from '$lib/app-state.svelte';
import { sourceFixtureTransport } from '$lib/dev-transport';
import WorkflowContentPanel from './WorkflowContentPanel.svelte';

describe('WorkflowContentPanel', () => {
  let app: AppState;

  beforeEach(async () => {
    app = new AppState(new QiongliAppClient(sourceFixtureTransport()));
    await app.refresh();
  });

  it('keeps client plugins in the Integration control and exposes only standalone Skills targets', () => {
    render(WorkflowContentPanel, { appState: app });

    expect(screen.getByRole('heading', {
      name: 'Materialize workflow content without a client plugin'
    })).toBeVisible();

    const destination = screen.getAllByRole('combobox')[0] as HTMLSelectElement;
    expect(Array.from(destination.options, (option) => option.value)).toEqual([
      'qiongli-managed',
      'registered-project',
      'custom-folder'
    ]);
    expect(screen.queryByRole('option', { name: 'Detected Codex plugin' })).not.toBeInTheDocument();
    expect(screen.queryByRole('option', { name: 'Detected Claude Code plugin' })).not.toBeInTheDocument();
  });

  it('routes registered-project installation through a path-free project intent', async () => {
    const execute = vi.spyOn(app, 'execute').mockResolvedValue(null);
    render(WorkflowContentPanel, { appState: app });

    const destination = screen.getAllByRole('combobox')[0] as HTMLSelectElement;
    await fireEvent.change(destination, { target: { value: 'registered-project' } });
    expect(screen.getByRole('combobox', { name: 'Registered project' })).toHaveValue(
      'prj_018f4d5a3b2c71008a9b0c1d2e3f4051'
    );
    await fireEvent.click(screen.getByRole('button', { name: 'Preview installation' }));

    expect(execute).toHaveBeenCalledWith({
      action: 'preview-project-skills-materialization',
      profile: 'marketplace-lite',
      projectId: 'prj_018f4d5a3b2c71008a9b0c1d2e3f4051'
    });
  });

  it('does not render every missing project target in the managed details list', async () => {
    app.snapshot!.content.managedSkills.destinations.push({
      targetId: `skills-target-${'5'.repeat(64)}`,
      preset: 'current-project',
      symbolicPath: '<project>/.qiongli-skills',
      state: 'missing',
      status: 'missing',
      profile: null,
      productVersion: null,
      projectId: 'prj_118f4d5a3b2c71008a9b0c1d2e3f4052'
    });
    const { container } = render(WorkflowContentPanel, { appState: app });

    expect(container.querySelectorAll('.managed-destination')).toHaveLength(2);
    const destination = screen.getAllByRole('combobox')[0] as HTMLSelectElement;
    await fireEvent.change(destination, { target: { value: 'registered-project' } });
    const project = screen.getByRole('combobox', {
      name: 'Registered project'
    }) as HTMLSelectElement;
    expect(project.options).toHaveLength(2);
    expect(container.querySelectorAll('.managed-destination')).toHaveLength(3);
  });

  it('does not invent an installable preset when native state has no target context', () => {
    app.snapshot!.content.managedSkills.destinations =
      app.snapshot!.content.managedSkills.destinations.filter(
        (destination) => destination.preset === 'custom-folder'
      );
    render(WorkflowContentPanel, { appState: app });

    const destination = screen.getAllByRole('combobox')[0] as HTMLSelectElement;
    expect(Array.from(destination.options, (option) => option.value)).toEqual([
      'qiongli-managed',
      'custom-folder'
    ]);
    expect(screen.getByRole('button', { name: 'Preview installation' })).toBeDisabled();
  });

  it('selects a custom folder opaquely before allowing a new materialization preview', async () => {
    render(WorkflowContentPanel, { appState: app });

    const destination = screen.getAllByRole('combobox')[0] as HTMLSelectElement;
    await fireEvent.change(destination, { target: { value: 'custom-folder' } });
    expect(screen.getByRole('button', { name: 'Preview installation' })).toBeDisabled();

    await fireEvent.click(screen.getByRole('button', { name: 'Choose custom folder' }));

    expect(app.selectedCustomSkillsTargetId).toBe(`skills-target-${'3'.repeat(64)}`);
    expect(screen.getByRole('button', { name: 'Preview installation' })).toBeEnabled();
    expect(app.notice?.detail).toBe(
      'The native service validated an opaque custom target. Its absolute path was not added to App state.'
    );
  });

  it('manages every registered custom target by anonymous receipt identity', async () => {
    const execute = vi.spyOn(app, 'execute').mockResolvedValue(null);
    render(WorkflowContentPanel, { appState: app });

    await fireEvent.click(screen.getByRole('button', {
      name: 'Verify Custom folder …22222222'
    }));
    expect(execute).toHaveBeenCalledWith({
      action: 'verify-managed-skills-target',
      targetId: `skills-target-${'2'.repeat(64)}`
    });

    expect(screen.getByRole('button', {
      name: 'Update Custom folder …22222222'
    })).toBeDisabled();
    await fireEvent.click(screen.getByRole('button', {
      name: 'Remove Custom folder …22222222'
    }));
    expect(execute).toHaveBeenCalledWith({
      action: 'preview-remove-managed-skills-target',
      targetId: `skills-target-${'2'.repeat(64)}`
    });
  });

  it('locks an installed destination to its receipt profile instead of silently replacing it', async () => {
    const managed = app.snapshot!.content.managedSkills.destinations.find(
      (destination) => destination.preset === 'qiongli-managed'
    )!;
    Object.assign(managed, {
      state: 'current',
      status: 'ready',
      profile: 'skill-only',
      productVersion: '2.0.0-alpha.2'
    });
    render(WorkflowContentPanel, { appState: app });

    const profile = screen.getAllByRole('combobox')[1] as HTMLSelectElement;
    expect(profile).toBeDisabled();
    expect(profile.value).toBe('skill-only');
    expect(screen.getByRole('button', { name: 'Preview installation' })).toBeDisabled();
    expect(screen.getByRole('button', { name: 'Verify content' })).toBeEnabled();
    expect(screen.getByRole('button', { name: 'Preview removal' })).toBeEnabled();
  });

  it('keeps verification read-only when standalone Skills mutation authority is unavailable', () => {
    app.snapshot!.capabilities.skillsMaterialize = false;
    const managed = app.snapshot!.content.managedSkills.destinations.find(
      (destination) => destination.preset === 'qiongli-managed'
    )!;
    Object.assign(managed, {
      state: 'current',
      status: 'ready',
      profile: 'skill-only',
      productVersion: '2.0.0-alpha.2'
    });
    render(WorkflowContentPanel, { appState: app });

    expect(screen.getByRole('button', { name: 'Preview installation' })).toBeDisabled();
    expect(screen.getByRole('button', { name: 'Preview update' })).toBeDisabled();
    expect(screen.getByRole('button', { name: 'Verify content' })).toBeEnabled();
    expect(screen.getByRole('button', { name: 'Preview removal' })).toBeDisabled();
  });

  it('preserves drifted files while offering an explicit ownership detach', async () => {
    const managed = app.snapshot!.content.managedSkills.destinations.find(
      (destination) => destination.preset === 'qiongli-managed'
    )!;
    Object.assign(managed, {
      state: 'drifted',
      status: 'drifted',
      profile: 'skill-only',
      productVersion: '2.0.0-alpha.2'
    });
    const execute = vi.spyOn(app, 'execute').mockResolvedValue(null);
    render(WorkflowContentPanel, { appState: app });

    expect(screen.getByText(/removes only Qiongli ownership/i)).toBeVisible();
    expect(screen.getByRole('button', { name: 'Preview removal' })).toBeDisabled();
    await fireEvent.click(screen.getByRole('button', { name: 'Preserve and detach' }));
    expect(execute).toHaveBeenCalledWith({
      action: 'preview-detach-managed-skills-target',
      targetId: managed.targetId
    });
  });

  it('does not offer installation over a detached preset directory that still contains files', () => {
    const managed = app.snapshot!.content.managedSkills.destinations.find(
      (destination) => destination.preset === 'qiongli-managed'
    )!;
    Object.assign(managed, {
      state: 'unmanaged',
      status: 'conflict',
      profile: null,
      productVersion: null
    });
    render(WorkflowContentPanel, { appState: app });

    expect(screen.getByText(/contains retained files/i)).toBeVisible();
    expect(screen.getByRole('button', { name: 'Preview installation' })).toBeDisabled();
    expect(screen.getByRole('button', { name: 'Verify content' })).toBeDisabled();
    expect(screen.queryByRole('button', { name: 'Preserve and detach' })).not.toBeInTheDocument();
  });

  it('locks the destination and profile scope while a native operation is running', () => {
    app.loading = true;
    render(WorkflowContentPanel, { appState: app });

    const [destination, profile] = screen.getAllByRole('combobox');
    expect(destination).toBeDisabled();
    expect(profile).toBeDisabled();
    expect(screen.getByRole('region', {
      name: 'Materialize workflow content without a client plugin'
    })).toHaveAttribute('aria-busy', 'true');
  });
});
