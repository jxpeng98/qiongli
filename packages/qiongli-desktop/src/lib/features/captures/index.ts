import type { FeatureDescriptor } from '../types';
import type { CaptureInboxEntry, CaptureSourceCoverage, StatusCode } from '@qiongli/app-api';

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

export function coverageStatus(source: CaptureSourceCoverage): StatusCode {
  switch (source.state) {
    case 'pending-review': return 'attention';
    case 'current': return 'ready';
    case 'stale': return 'drifted';
    case 'conflicted': return 'conflict';
    case 'unbound': return 'blocked';
    case 'unknown': return 'missing';
  }
}
