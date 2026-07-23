import {
  QiongliAppClient,
  type AcademicGraphPathResult,
  type AcademicGraphPortfolioSnapshot,
  type AcademicGraphQueryResult,
  type AcademicGraphRevisionComparison,
  type AcademicGraphSnapshot,
  type AgentRunResult,
  type ArtifactChangeSnapshot,
  type AppEvent,
  type AppIntent,
  type AppSnapshot,
  type CaptureConsolidationPreview,
  type CaptureCoverageSnapshot,
  type CaptureInboxSnapshot,
  type CaptureIntakePreview,
  type OperationPreview,
  type ResearchCapture
} from '@qiongli/app-api';

import { i18n } from './i18n.svelte';

export interface AppNotice {
  tone: 'info' | 'success' | 'danger';
  title: string;
  detail: string;
}

export class AppState {
  snapshot = $state<AppSnapshot | null>(null);
  preview = $state<OperationPreview | null>(null);
  captureInbox = $state<CaptureInboxSnapshot | null>(null);
  captureCoverage = $state<CaptureCoverageSnapshot | null>(null);
  artifactChanges = $state<ArtifactChangeSnapshot | null>(null);
  academicGraph = $state<AcademicGraphSnapshot | null>(null);
  academicGraphComparison = $state<AcademicGraphRevisionComparison | null>(null);
  academicGraphQuery = $state<AcademicGraphQueryResult | null>(null);
  academicGraphPath = $state<AcademicGraphPathResult | null>(null);
  academicGraphPortfolio = $state<AcademicGraphPortfolioSnapshot | null>(null);
  agentRun = $state<AgentRunResult | null>(null);
  capture = $state<ResearchCapture | null>(null);
  captureIntakePreview = $state<CaptureIntakePreview | null>(null);
  captureConsolidationPreview = $state<CaptureConsolidationPreview | null>(null);
  notice = $state<AppNotice | null>(null);
  loading = $state(false);
  bridgeReady = $state(true);
  closeRequested = $state(false);

  constructor(private readonly client = new QiongliAppClient()) {}

  async refresh(): Promise<void> {
    this.loading = true;
    try {
      this.snapshot = await this.client.snapshot();
      this.bridgeReady = true;
    } catch (error) {
      this.bridgeReady = false;
      this.notice = {
        tone: 'danger',
        title: i18n.t('notice.nativeUnavailable'),
        detail: publicError(error)
      };
    } finally {
      this.loading = false;
    }
  }

  async execute(intent: AppIntent): Promise<AppEvent | null> {
    this.loading = true;
    try {
      const event = await this.client.execute(intent);
      this.applyEvent(event);
      if (intent.action === 'confirm-operation') this.closePreview();
      return event;
    } catch (error) {
      // A failed confirmation invalidates its reviewed preview even when the
      // native bridge rejects the invoke instead of returning an AppEvent.
      if (intent.action === 'confirm-operation') this.closePreview();
      this.notice = {
        tone: 'danger',
        title: i18n.t('notice.actionFailed'),
        detail: publicError(error)
      };
      return null;
    } finally {
      this.loading = false;
    }
  }

  dismissNotice(): void {
    this.notice = null;
  }

  closePreview(): void {
    this.preview = null;
    this.captureIntakePreview = null;
    this.captureConsolidationPreview = null;
  }

  private applyEvent(event: AppEvent): void {
    switch (event.type) {
      case 'snapshot':
        this.snapshot = event.snapshot;
        break;
      case 'preview':
        this.preview = event.preview;
        if (event.preview.kind === 'agent-run') this.agentRun = null;
        break;
      case 'capture-inbox':
        this.captureInbox = event.inbox;
        this.capture = null;
        break;
      case 'capture-coverage':
        this.captureCoverage = event.coverage;
        break;
      case 'artifact-changes':
        this.artifactChanges = event.changes;
        break;
      case 'academic-graph':
        this.academicGraph = event.graph;
        this.academicGraphComparison = event.comparison;
        this.academicGraphQuery = null;
        this.academicGraphPath = null;
        break;
      case 'academic-graph-query':
        this.academicGraphQuery = event.result;
        break;
      case 'academic-graph-portfolio':
        this.academicGraphPortfolio = event.portfolio;
        break;
      case 'academic-graph-path':
        this.academicGraphPath = event.result;
        break;
      case 'academic-graph-artifact-opened':
        break;
      case 'capture-read':
        this.capture = event.capture;
        break;
      case 'capture-file-selected':
        this.notice = {
          tone: 'info',
          title: i18n.t('notice.captureSelected'),
          detail: i18n.t('notice.captureSelectedDetail', { label: event.fileLabel })
        };
        break;
      case 'capture-intake-preview':
        this.captureIntakePreview = event.intake;
        this.captureConsolidationPreview = null;
        this.preview = event.preview;
        break;
      case 'capture-consolidation-preview':
        this.captureConsolidationPreview = event.consolidation;
        this.captureIntakePreview = null;
        this.preview = event.preview;
        break;
      case 'project-directory-selected':
        this.notice = {
          tone: 'info',
          title: i18n.t('notice.locationSelected'),
          detail: i18n.t('notice.locationSelectedDetail', { label: event.rootLabel })
        };
        break;
      case 'update-changed':
        if (this.snapshot) this.snapshot.update = event.update;
        this.closeRequested = event.closeRequested;
        break;
      case 'agent-run-completed':
        this.agentRun = event.result;
        this.closePreview();
        this.notice = {
          tone: 'success',
          title: i18n.t('notice.agentRunCompleted'),
          detail: i18n.t('notice.agentRunCompletedDetail', {
            turns: event.result.modelTurns,
            tools: event.result.toolCalls
          })
        };
        break;
      case 'completed':
        this.snapshot = event.snapshot;
        this.captureInbox = null;
        this.captureCoverage = null;
        this.artifactChanges = null;
        this.academicGraph = null;
        this.academicGraphComparison = null;
        this.academicGraphQuery = null;
        this.academicGraphPath = null;
        this.academicGraphPortfolio = null;
        this.capture = null;
        this.closePreview();
        this.notice = {
          tone: 'success',
          title: i18n.t('notice.completed'),
          detail: event.code
        };
        break;
      case 'capture-operation-completed':
        this.snapshot = event.snapshot;
        this.captureInbox = event.inbox;
        this.captureCoverage = event.coverage;
        this.artifactChanges = event.changes;
        this.academicGraph = null;
        this.academicGraphComparison = null;
        this.academicGraphQuery = null;
        this.academicGraphPath = null;
        this.academicGraphPortfolio = null;
        this.capture = null;
        this.closePreview();
        this.notice = {
          tone: 'success',
          title: i18n.t('notice.captureCompleted'),
          detail: event.code
        };
        break;
      case 'cancelled':
        this.closePreview();
        this.notice = { tone: 'info', title: i18n.t('notice.cancelled'), detail: i18n.reason(event.code) };
        break;
      case 'validation-failed':
        this.closePreview();
        this.notice = { tone: 'danger', title: i18n.t('notice.checkOptions'), detail: i18n.reason(event.code) };
        break;
      case 'failed':
        this.closePreview();
        this.notice = { tone: 'danger', title: i18n.t('notice.failed'), detail: i18n.reason(event.code) };
        break;
    }
  }
}

function publicError(error: unknown): string {
  if (error instanceof Error && error.message.length <= 240) return error.message;
  if (typeof error === 'string' && error.length > 0 && error.length <= 240) return i18n.reason(error);
  return i18n.reason('native-bridge-unavailable');
}
