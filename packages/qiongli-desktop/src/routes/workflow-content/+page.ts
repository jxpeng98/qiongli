import { redirect } from '@sveltejs/kit';

export function load({ url }: { url: URL }): never {
  redirect(307, `/client-integrations${url.search}#workflow-content`);
}
