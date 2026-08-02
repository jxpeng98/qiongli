import { describe, expect, it } from 'vitest';

import { load } from './+page';

describe('root route', () => {
  it('preserves the read-only fixture while redirecting to overview', () => {
    expect(() => load({
      url: new URL('http://127.0.0.1:1420/?fixture=source-read-only')
    })).toThrow(expect.objectContaining({
      status: 307,
      location: '/overview?fixture=source-read-only'
    }));
  });
});
