import type { PageServerLoad } from './$types';
import { getShopCatalog } from '$lib/server/shop';

export const load: PageServerLoad = async ({ locals }) => {
	return { ...(await getShopCatalog(locals.user?.id)), signedIn: Boolean(locals.user) };
};
