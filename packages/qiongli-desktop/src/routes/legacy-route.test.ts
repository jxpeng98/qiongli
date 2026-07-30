import { describe, expect, it } from 'vitest';

import { load as loadModelBackend } from './model-backend/+page';
import { load as loadWorkflowContent } from './workflow-content/+page';

describe('retired product routes', () => {
  it('moves standalone content into its single Integration owner without losing fixture state', () => {
    expect(() => loadWorkflowContent({
      url: new URL('http://127.0.0.1:1420/workflow-content?fixture=source-read-only')
    })).toThrow(expect.objectContaining({
      status: 307,
      location: '/client-integrations?fixture=source-read-only#workflow-content'
    }));
  });

  it('moves legacy backend cleanup into Integrations without reopening a model surface', () => {
    expect(() => loadModelBackend({
      url: new URL('http://127.0.0.1:1420/model-backend?fixture=source-read-only')
    })).toThrow(expect.objectContaining({
      status: 307,
      location: '/client-integrations?fixture=source-read-only#legacy-credential-cleanup'
    }));
  });
});
