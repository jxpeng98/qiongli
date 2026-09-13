import type { LayoutLoad } from './$types';

import { i18n } from '$lib/i18n.svelte';

export const ssr = false;

export const load: LayoutLoad = async () => {
  await i18n.initialize();
  return {};
};
