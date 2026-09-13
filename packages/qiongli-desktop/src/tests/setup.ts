import '@testing-library/jest-dom/vitest';
import { cleanup } from '@testing-library/svelte';
import { afterEach, beforeAll } from 'vitest';

import { i18n } from '$lib/i18n.svelte';

class ResizeObserverStub {
  observe(): void {}
  unobserve(): void {}
  disconnect(): void {}
}

Object.defineProperty(globalThis, 'ResizeObserver', {
  configurable: true,
  value: ResizeObserverStub
});
if (typeof window !== 'undefined') {
  Object.defineProperty(window, 'ResizeObserver', {
    configurable: true,
    value: ResizeObserverStub
  });
}

beforeAll(async () => {
  await i18n.setLocale('en');
});

afterEach(async () => {
  cleanup();
  // Bits UI intentionally defers body-scroll-lock restoration for 24 ms so
  // same-tick overlays can hand off without flicker. Let that cleanup finish
  // before Vitest tears down jsdom and removes the global document.
  await new Promise((resolve) => window.setTimeout(resolve, 30));
});
