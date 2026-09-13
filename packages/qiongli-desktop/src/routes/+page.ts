import { redirect } from '@sveltejs/kit';

export function load({ url }: { url: URL }): never {
  redirect(307, `/overview${url.search}`);
}
