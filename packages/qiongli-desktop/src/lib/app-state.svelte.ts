import {
  QiongliAppClient,
  type AppEvent,
  type AppIntent,
  type AppSnapshot,
  type CaptureConsolidationPreview,
  type CaptureInboxSnapshot,
  type CaptureIntakePreview,
  type OperationPreview,
  type ResearchCapture
} from '@qiongli/app-api';

export interface AppNotice {
  tone: 'info' | 'success' | 'danger';
  title: string;
  detail: string;
}

export class AppState {
  snapshot = $state<AppSnapshot | null>(null);
  preview = $state<OperationPreview | null>(null);
  captureInbox = $state<CaptureInboxSnapshot | null>(null);
  capture = $state<ResearchCapture | null>(null);
  captureIntakePreview = $state<CaptureIntakePreview | null>(null);
  captureConsolidationPreview = $state<CaptureConsolidationPreview | null>(null);
  notice = $state<AppNotice | null>(null);
  loading = $state(false);
  bridgeReady = $state(true);

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
        title: 'Native service unavailable',
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
      return event;
    } catch (error) {
      this.notice = {
        tone: 'danger',
        title: 'Qiongli could not complete this action',
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
        break;
      case 'capture-inbox':
        this.captureInbox = event.inbox;
        this.capture = null;
        break;
      case 'capture-read':
        this.capture = event.capture;
        break;
      case 'capture-file-selected':
        this.notice = {
          tone: 'info',
          title: 'Research capture selected',
          detail: `${event.fileLabel} is ready for native validation. No absolute path was exposed.`
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
          title: 'Project location selected',
          detail: `${event.rootLabel} is ready for a native operation preview. No absolute path was exposed.`
        };
        break;
      case 'completed':
        this.snapshot = event.snapshot;
        this.captureInbox = null;
        this.capture = null;
        this.closePreview();
        this.notice = {
          tone: 'success',
          title: 'Operation completed',
          detail: event.code
        };
        break;
      case 'capture-operation-completed':
        this.snapshot = event.snapshot;
        this.captureInbox = event.inbox;
        this.capture = null;
        this.closePreview();
        this.notice = {
          tone: 'success',
          title: 'Capture operation completed',
          detail: event.code
        };
        break;
      case 'cancelled':
        this.closePreview();
        this.notice = { tone: 'info', title: 'Operation cancelled', detail: event.code };
        break;
      case 'validation-failed':
        this.notice = { tone: 'danger', title: 'Check the selected options', detail: event.code };
        break;
      case 'failed':
        this.notice = { tone: 'danger', title: 'Operation failed', detail: event.code };
        break;
    }
  }
}

function publicError(error: unknown): string {
  if (error instanceof Error && error.message.length <= 240) return error.message;
  return 'native-bridge-unavailable';
}
