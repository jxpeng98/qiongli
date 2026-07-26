import {
  QiongliAppClient,
  type AcademicGraphPathResult,
  type AcademicGraphPortfolioSnapshot,
  type AcademicGraphQueryResult,
  type AcademicGraphRevisionComparison,
  type AcademicGraphSnapshot,
  type ArtifactChangeSnapshot,
  type AppEvent,
  type AppIntent,
  type AppSnapshot,
  type CaptureAssignmentPage,
  type CaptureAssignmentPreview,
  type CaptureAssignmentView,
  type CaptureConsolidationPreview,
  type CaptureCoverageSnapshot,
  type CaptureDeliveryAcknowledgementPreview,
  type CaptureDeliveryPage,
  type CaptureDeliveryView,
  type CaptureInboxSnapshot,
  type CaptureIntakePreview,
  type CaptureResolutionPage,
  type CaptureResolutionPreview,
  type CaptureResolutionSelection,
  type CaptureResolutionView,
  type ContinuityOperationProgress,
  type OperationPreview,
  type OrchestrationRunList,
  type PortfolioDoctor,
  type PortfolioMaintenancePreview,
  type PortfolioMaintenanceResult,
  type PortfolioQueryResult,
  type PortfolioStatus,
  type ResearchCapture,
  type SemanticTimelineResult
} from '@qiongli/app-api';

import { i18n } from './i18n.svelte';

export interface AppNotice {
  tone: 'info' | 'success' | 'warning' | 'danger';
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
  orchestrationRuns = $state<OrchestrationRunList | null>(null);
  capture = $state<ResearchCapture | null>(null);
  captureIntakePreview = $state<CaptureIntakePreview | null>(null);
  captureConsolidationPreview = $state<CaptureConsolidationPreview | null>(null);
  captureDeliveries = $state<CaptureDeliveryPage | null>(null);
  captureDelivery = $state<CaptureDeliveryView | null>(null);
  captureDeliveryAcknowledgementPreview =
    $state<CaptureDeliveryAcknowledgementPreview | null>(null);
  captureAssignments = $state<CaptureAssignmentPage | null>(null);
  captureAssignment = $state<CaptureAssignmentView | null>(null);
  captureAssignmentPreview = $state<CaptureAssignmentPreview | null>(null);
  captureResolutions = $state<CaptureResolutionPage | null>(null);
  captureResolution = $state<CaptureResolutionView | null>(null);
  captureResolutionPlan = $state<CaptureResolutionPreview | null>(null);
  captureResolutionPreview = $state<CaptureResolutionPreview | null>(null);
  captureResolutionSelections = $state<CaptureResolutionSelection[]>([]);
  portfolioStatus = $state<PortfolioStatus | null>(null);
  portfolioQuery = $state<PortfolioQueryResult | null>(null);
  semanticTimeline = $state<SemanticTimelineResult | null>(null);
  portfolioDoctor = $state<PortfolioDoctor | null>(null);
  portfolioMaintenancePreview = $state<PortfolioMaintenancePreview | null>(null);
  continuityOperationProgress = $state<ContinuityOperationProgress | null>(null);
  portfolioMaintenanceResult = $state<PortfolioMaintenanceResult | null>(null);
  notice = $state<AppNotice | null>(null);
  loading = $state(false);
  bridgeReady = $state(true);
  closeRequested = $state(false);

  constructor(private readonly client = new QiongliAppClient()) {}

  async refresh(): Promise<void> {
    this.loading = true;
    try {
      const snapshot = await this.client.snapshot();
      const libraryChanged = this.snapshot?.researchLibrary.revision
        !== snapshot.researchLibrary.revision;
      this.snapshot = snapshot;
      this.clearCaptureContinuity();
      if (libraryChanged) this.clearPortfolioContinuity();
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
    this.captureDeliveryAcknowledgementPreview = null;
    this.captureAssignmentPreview = null;
    this.captureResolutionPreview = null;
    this.captureResolutionSelections = [];
    this.portfolioMaintenancePreview = null;
  }

  private applyEvent(event: AppEvent): void {
    switch (event.type) {
      case 'snapshot':
        if (this.snapshot?.researchLibrary.revision !== event.snapshot.researchLibrary.revision) {
          this.clearPortfolioContinuity();
        }
        this.snapshot = event.snapshot;
        this.clearCaptureContinuity();
        break;
      case 'preview':
        this.preview = event.preview;
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
      case 'capture-deliveries':
        this.captureDeliveries = event.page;
        break;
      case 'capture-delivery-inspected':
      case 'capture-delivery-updated':
        this.captureDelivery = event.delivery;
        break;
      case 'capture-delivery-acknowledgement-preview':
        this.captureDeliveryAcknowledgementPreview = event.acknowledgement;
        this.captureAssignmentPreview = null;
        this.captureResolutionPreview = null;
        this.captureResolutionSelections = [];
        this.preview = event.preview;
        break;
      case 'capture-assignments':
        this.captureAssignments = event.page;
        break;
      case 'capture-assignment-inspected':
        this.captureAssignment = event.assignment;
        break;
      case 'capture-assignment-preview':
        this.captureAssignmentPreview = event.assignment;
        this.captureDeliveryAcknowledgementPreview = null;
        this.captureResolutionPreview = null;
        this.captureResolutionSelections = [];
        this.preview = event.preview;
        break;
      case 'capture-resolutions':
        this.captureResolutions = event.page;
        break;
      case 'capture-resolution-inspected':
        this.captureResolution = event.resolution;
        break;
      case 'capture-resolution-plan':
        this.captureResolutionPlan = event.resolution;
        break;
      case 'capture-resolution-preview':
        this.captureResolutionPreview = event.resolution;
        this.captureResolutionPlan = event.resolution;
        this.captureResolutionSelections = event.selections;
        this.captureDeliveryAcknowledgementPreview = null;
        this.captureAssignmentPreview = null;
        this.preview = event.preview;
        break;
      case 'portfolio-status': {
        const current = this.portfolioStatus;
        const catalogChanged = current === null
          || current.catalogId !== event.portfolio.catalogId
          || current.catalogGeneration !== event.portfolio.catalogGeneration
          || current.portfolioId !== event.portfolio.portfolioId
          || current.libraryRevision !== event.portfolio.libraryRevision;
        this.portfolioStatus = event.portfolio;
        if (catalogChanged || event.portfolio.state !== 'current') {
          this.portfolioQuery = null;
          this.semanticTimeline = null;
          this.portfolioDoctor = null;
        }
        break;
      }
      case 'portfolio-query':
        this.portfolioQuery = event.result;
        break;
      case 'semantic-timeline':
        this.semanticTimeline = this.portfolioStatus?.state === 'current'
          && this.portfolioStatus.catalogId === event.result.catalogId
          ? event.result
          : null;
        break;
      case 'portfolio-doctor':
        this.portfolioDoctor = event.doctor;
        break;
      case 'portfolio-maintenance-preview':
        this.portfolioMaintenancePreview = event.maintenance;
        this.preview = event.preview;
        break;
      case 'continuity-operation-progress':
        if (this.continuityOperationProgress?.operationId !== event.progress.operationId) {
          this.portfolioMaintenanceResult = null;
        }
        this.continuityOperationProgress = event.progress;
        break;
      case 'portfolio-maintenance-completed':
        this.portfolioMaintenanceResult = event.result;
        this.continuityOperationProgress = null;
        this.portfolioStatus = null;
        this.portfolioQuery = null;
        this.semanticTimeline = null;
        this.portfolioDoctor = null;
        this.portfolioMaintenancePreview = null;
        this.academicGraphPortfolio = null;
        this.notice = {
          tone: 'success',
          title: i18n.t('notice.portfolioCompleted'),
          detail: i18n.t('notice.portfolioCompletedDetail', {
            operation: i18n.label(event.result.operation)
          })
        };
        break;
      case 'project-directory-selected':
        this.notice = {
          tone: 'info',
          title: i18n.t('notice.locationSelected'),
          detail: i18n.t('notice.locationSelectedDetail', { label: event.rootLabel })
        };
        break;
      case 'project-migration-completed':
        this.snapshot = event.snapshot;
        this.captureInbox = null;
        this.captureCoverage = null;
        this.artifactChanges = null;
        this.academicGraph = null;
        this.academicGraphComparison = null;
        this.academicGraphQuery = null;
        this.academicGraphPath = null;
        this.academicGraphPortfolio = null;
        this.orchestrationRuns = null;
        this.capture = null;
        this.clearCaptureContinuity();
        this.clearPortfolioContinuity();
        this.closePreview();
        this.notice = event.qualification.deterministicRebuild
          ? {
              tone: 'success',
              title: i18n.t('notice.migrationCompleted'),
              detail: i18n.t('notice.migrationCompletedDetail')
            }
          : {
              tone: 'warning',
              title: i18n.t('notice.migrationRebuildRequired'),
              detail: i18n.reason(
                event.qualification.reasonCode ?? 'project-migration-graph-rebuild-required'
              )
            };
        break;
      case 'update-changed':
        if (this.snapshot) this.snapshot.update = event.update;
        this.closeRequested = event.closeRequested;
        break;
      case 'orchestration-loaded':
        this.orchestrationRuns = event.runs;
        break;
      case 'orchestration-run-updated':
        this.orchestrationRuns = event.runs;
        this.notice = {
          tone: event.run.status === 'cancelled' ? 'info' : 'success',
          title: i18n.t('notice.orchestrationUpdated'),
          detail: i18n.t('notice.orchestrationUpdatedDetail', {
            status: i18n.label(event.run.status)
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
        this.orchestrationRuns = null;
        this.capture = null;
        this.clearCaptureContinuity();
        this.clearPortfolioContinuity();
        this.closePreview();
        this.notice = event.code === 'project-migration-rolled-back'
          ? {
              tone: 'success',
              title: i18n.t('notice.migrationRolledBack'),
              detail: i18n.t('notice.migrationRolledBackDetail')
            }
          : {
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
        this.orchestrationRuns = null;
        this.capture = null;
        this.captureDeliveries = null;
        this.captureAssignments = null;
        this.captureResolutions = null;
        this.captureDelivery = event.delivery;
        this.captureAssignment = event.assignment;
        this.captureResolution = event.resolution;
        this.closePreview();
        this.captureResolutionPlan = null;
        this.clearPortfolioContinuity();
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

  private clearCaptureContinuity(): void {
    this.captureDeliveries = null;
    this.captureDelivery = null;
    this.captureDeliveryAcknowledgementPreview = null;
    this.captureAssignments = null;
    this.captureAssignment = null;
    this.captureAssignmentPreview = null;
    this.captureResolutions = null;
    this.captureResolution = null;
    this.captureResolutionPlan = null;
    this.captureResolutionPreview = null;
    this.captureResolutionSelections = [];
  }

  private clearPortfolioContinuity(): void {
    this.portfolioStatus = null;
    this.portfolioQuery = null;
    this.semanticTimeline = null;
    this.portfolioDoctor = null;
    this.portfolioMaintenancePreview = null;
    this.continuityOperationProgress = null;
    this.portfolioMaintenanceResult = null;
  }
}

function publicError(error: unknown): string {
  if (error instanceof Error && error.message.length <= 240) return error.message;
  if (typeof error === 'string' && error.length > 0 && error.length <= 240) return i18n.reason(error);
  return i18n.reason('native-bridge-unavailable');
}
