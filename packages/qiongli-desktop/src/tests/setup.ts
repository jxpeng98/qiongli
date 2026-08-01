import '@testing-library/jest-dom/vitest';
import { beforeAll } from 'vitest';

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
