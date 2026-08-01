import { error, fail, redirect } from '@sveltejs/kit';
import { resolve } from '$app/paths';
import type { Actions, PageServerLoad } from './$types';
import { requireUser } from '$lib/server/guards';
import { createShopOrder, getShopCatalog, ShopError } from '$lib/server/shop';

export const load: PageServerLoad = async ({ locals, params }) => {
	const currentUser = requireUser(locals);
	const catalog = await getShopCatalog(currentUser.id);
	const item = catalog.items.find((candidate) => candidate.id === params.itemId);
	if (!item) error(404, 'Shop item not found');

	return { item, balance: catalog.balance };
};

export const actions: Actions = {
	order: async ({ locals, params, request }) => {
		const currentUser = requireUser(locals);
		const formData = await request.formData();
		const notes = formData.get('notes');
		if (typeof notes !== 'string') {
			return fail(400, { success: false, message: 'Invalid order form.' });
		}

		try {
			await createShopOrder(currentUser.id, params.itemId, notes);
		} catch (caught) {
			if (caught instanceof ShopError) {
				return fail(caught.status, { success: false, message: caught.message, notes });
			}
			console.error('[shop] Unexpected order error', caught);
			return fail(500, {
				success: false,
				message: 'Could not place the order. Try again.',
				notes
			});
		}

		redirect(303, resolve('/home/shop/orders'));
	}
};
