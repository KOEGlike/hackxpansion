import { error, fail } from '@sveltejs/kit';
import type { Actions, PageServerLoad } from './$types';
import { AdminError, requireAdmin } from '$lib/server/admin';
import { requireUser } from '$lib/server/guards';
import { fulfillShopOrder, getAllShopOrders, ShopError } from '$lib/server/shop';
import { isUuid } from '$lib/projects/domain';

export const load: PageServerLoad = async ({ locals }) => {
	if (!locals.user) error(404, 'Page not found');

	try {
		await requireAdmin(locals.user.id);
		return { orders: await getAllShopOrders() };
	} catch (caught) {
		if (caught instanceof AdminError) error(caught.status, caught.message);
		throw caught;
	}
};

export const actions: Actions = {
	fulfill: async ({ locals, request }) => {
		const currentUser = requireUser(locals);
		const formData = await request.formData();
		const orderId = formData.get('orderId');
		const message = formData.get('message');

		if (typeof orderId !== 'string' || !isUuid(orderId) || typeof message !== 'string') {
			return fail(400, { success: false, message: 'Invalid fulfillment form.' });
		}

		try {
			await fulfillShopOrder(currentUser.id, orderId, message);
			return { success: true, message: 'Order marked as fulfilled.' };
		} catch (caught) {
			if (caught instanceof AdminError || caught instanceof ShopError) {
				return fail(caught.status, { success: false, message: caught.message, orderId });
			}
			console.error('[shop/admin] Unexpected fulfillment error', caught);
			return fail(500, { success: false, message: 'Could not fulfill the order.' });
		}
	}
};
