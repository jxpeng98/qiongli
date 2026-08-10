import {
  type AcademicGraphPathResult,
  type AcademicGraphPortfolioSnapshot,
  type AcademicGraphQueryResult,
  type AcademicGraphReadiness,
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
  type ManagedSkillsTargetId,
  type OperationPreview,
  type OrchestrationRunList,
  type PortfolioDoctor,
  type PortfolioMaintenancePreview,
  type PortfolioMaintenanceResult,
  type PortfolioQueryResult,
  type PortfolioStatus,
  type ProjectArtifactView,
  type ResearchCapture,
  type SemanticTimelineResult
} from '@qiongli/app-api';

import {
  deferredAppClient,
  type AppClient
} from './deferred-app-client';
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
  academicGraphReadiness = $state<AcademicGraphReadiness | null>(null);
  academicGraphComparison = $state<AcademicGraphRevisionComparison | null>(null);
  academicGraphQuery = $state<AcademicGraphQueryResult | null>(null);
  academicGraphPath = $state<AcademicGraphPathResult | null>(null);
  academicGraphPortfolio = $state<AcademicGraphPortfolioSnapshot | null>(null);
  projectArtifact = $state<ProjectArtifactView | null>(null);
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
  selectedCustomSkillsTargetId = $state<ManagedSkillsTargetId | null>(null);
  loading = $state(false);
  bridgeReady = $state(true);
  closeRequested = $state(false);
  private activeOperationCount = 0;

  constructor(private readonly client: AppClient = deferredAppClient()) {}

  async refresh(): Promise<void> {
    this.beginOperation();
    try {
      const snapshot = await this.client.snapshot();
      this.snapshot = snapshot;
      // A full authoritative refresh may follow a native-process restart.
      // Process-local previews, operation IDs, cursors, and derived catalog
      // observations must never survive that boundary, even when the Research
      // Library revision happens to be unchanged.
      this.selectedCustomSkillsTargetId = null;
      this.closePreview();
      this.clearCaptureContinuity();
      this.clearPortfolioContinuity();
      this.bridgeReady = true;
    } catch (error) {
      this.bridgeReady = false;
      this.notice = {
        tone: 'danger',
        title: i18n.t('notice.nativeUnavailable'),
        detail: publicError(error)
      };
    } finally {
      this.endOperation();
    }
  }

  async execute(
    intent: AppIntent,
    accept: (event: AppEvent) => boolean = () => true
  ): Promise<AppEvent | null> {
    this.beginOperation();
    try {
      const event = await this.client.execute(intent);
      if (!accept(event)) return null;
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
      this.endOperation();
    }
  }

  dismissNotice(): void {
    this.notice = null;
  }

  private beginOperation(): void {
    this.activeOperationCount += 1;
    this.loading = true;
  }

  private endOperation(): void {
    this.activeOperationCount = Math.max(0, this.activeOperationCount - 1);
    this.loading = this.activeOperationCount > 0;
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
        this.snapshot = event.snapshot;
        // Read-only native probes reuse the snapshot event but do not replace
        // the native service. Keep the approved process-local folder choice;
        // only refresh(), the actual reconnect boundary, invalidates it.
        this.closePreview();
        this.clearCaptureContinuity();
        this.clearPortfolioContinuity();
        break;
      case 'preview':
        this.preview = event.preview;
        break;
      case 'content-customization':
        break;
      case 'skills-destination-selected':
        this.selectedCustomSkillsTargetId = event.targetId;
        this.notice = {
          tone: 'info',
          title: i18n.t('notice.skillsDestinationSelected'),
          detail: i18n.t('notice.skillsDestinationSelectedDetail')
        };
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
        this.academicGraphReadiness = event.readiness;
        this.academicGraphComparison = event.comparison;
        this.academicGraphQuery = null;
        this.academicGraphPath = null;
        this.projectArtifact = null;
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
      case 'project-artifact-read':
        this.projectArtifact = event.artifact;
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
          if (this.portfolioMaintenancePreview) this.closePreview();
          this.portfolioQuery = null;
          this.semanticTimeline = null;
          this.portfolioDoctor = null;
        }
        break;
      }
      case 'portfolio-query':
        this.portfolioQuery = this.portfolioStatus?.state === 'current'
          && this.portfolioStatus.catalogId === event.result.catalogId
          ? event.result
          : null;
        break;
      case 'semantic-timeline':
        this.semanticTimeline = this.portfolioStatus?.state === 'current'
          && this.portfolioStatus.catalogId === event.result.catalogId
          ? event.result
          : null;
        break;
      case 'portfolio-doctor':
        this.portfolioDoctor = this.portfolioStatus
          && this.portfolioStatus.libraryRevision === event.doctor.libraryRevision
          ? event.doctor
          : null;
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
        this.academicGraphReadiness = null;
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
        this.selectedCustomSkillsTargetId =
          this.selectedCustomSkillsTargetId !== null
          && event.snapshot.content.managedSkills.destinations.some(
            (destination) => destination.targetId === this.selectedCustomSkillsTargetId
          )
            ? this.selectedCustomSkillsTargetId
            : null;
        this.snapshot = event.snapshot;
        this.captureInbox = null;
        this.captureCoverage = null;
        this.artifactChanges = null;
        this.academicGraph = null;
        this.academicGraphReadiness = null;
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
              detail: i18n.reason(event.code)
            };
        break;
      case 'capture-operation-completed':
        this.snapshot = event.snapshot;
        this.captureInbox = event.inbox;
        this.captureCoverage = event.coverage;
        this.artifactChanges = event.changes;
        this.academicGraph = null;
        this.academicGraphReadiness = null;
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
    this.projectArtifact = null;
  }
}

function publicError(error: unknown): string {
  if (error instanceof Error && error.message.length <= 240) return error.message;
  if (typeof error === 'string' && error.length > 0 && error.length <= 240) return i18n.reason(error);
  return i18n.reason('native-bridge-unavailable');
}
