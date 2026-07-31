import { fail } from '@sveltejs/kit';
import type { Actions, PageServerLoad } from './$types';
import { createShopOrder, getShopCatalog, ShopError } from '$lib/server/shop';
import { requireUser } from '$lib/server/guards';

export const load: PageServerLoad = async ({ locals }) => {
	return { ...(await getShopCatalog(locals.user?.id)), signedIn: Boolean(locals.user) };
};

export const actions: Actions = {
	order: async ({ locals, request }) => {
		const currentUser = requireUser(locals);
		const formData = await request.formData();
		const itemId = formData.get('itemId');
		const notes = formData.get('notes');

		if (typeof itemId !== 'string' || typeof notes !== 'string') {
			return fail(400, { success: false, message: 'Invalid order form.' });
		}

		try {
			const result = await createShopOrder(currentUser.id, itemId, notes);
			return {
				success: true,
				message: `${result.itemName} was added to your orders.`,
				orderId: result.orderId
			};
		} catch (error) {
			if (error instanceof ShopError) {
				return fail(error.status, { success: false, message: error.message, itemId, notes });
			}
			console.error('[shop] Unexpected order error', error);
			return fail(500, { success: false, message: 'Could not place the order. Try again.' });
		}
	}
};
