import type { PageServerLoad } from './$types';
import { getUserShopOrders } from '$lib/server/shop';

export const load: PageServerLoad = async ({ locals }) => {
	return {
		signedIn: Boolean(locals.user),
		orders: locals.user ? await getUserShopOrders(locals.user.id) : []
	};
};
