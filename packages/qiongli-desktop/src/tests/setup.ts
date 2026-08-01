import '@testing-library/jest-dom/vitest';
import { beforeAll } from 'vitest';

import { i18n } from '$lib/i18n.svelte';

beforeAll(async () => {
  await i18n.setLocale('en');
});
