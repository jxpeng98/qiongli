import type { FeatureDescriptor } from '../types';
import type { CaptureInboxEntry, StatusCode } from '@qiongli/app-api';

export const capturesFeature: FeatureDescriptor = {
  id: 'captures',
  label: 'Research Captures',
  route: '/captures',
  stage: 'r4b'
};

export function captureStatus(entry: CaptureInboxEntry): StatusCode {
  switch (entry.state) {
    case 'pending-review': return 'attention';
    case 'stale': return 'drifted';
    case 'conflicted': return 'conflict';
    case 'applied': return 'ready';
  }
}

export function canReviewCapture(entry: CaptureInboxEntry): boolean {
  return entry.state !== 'applied';
}
