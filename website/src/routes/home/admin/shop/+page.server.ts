import { error, fail } from '@sveltejs/kit';
import type { Actions, PageServerLoad } from './$types';
import { requireUser } from '$lib/server/guards';
import {
	createCatalogItem,
	getAdminShopItems,
	requireAdmin,
	ShopError,
	updateCatalogItem
} from '$lib/server/shop';
import {
	CatalogItemValidationError,
	parseCatalogItem,
	type CatalogItemFormValues
} from '$lib/shop/catalog';

export const load: PageServerLoad = async ({ locals }) => {
	if (!locals.user) error(404, 'Page not found');

	try {
		await requireAdmin(locals.user.id);
		return { items: await getAdminShopItems() };
	} catch (caught) {
		if (caught instanceof ShopError) error(caught.status, caught.message);
		throw caught;
	}
};

export const actions: Actions = {
	create: async ({ locals, request }) => {
		const currentUser = requireUser(locals);
		const values = catalogItemValues(await request.formData());

		try {
			const input = parseCatalogItem(values);
			await createCatalogItem(currentUser.id, input);
			return { success: true, message: `${input.name} was added.`, action: 'create' as const };
		} catch (caught) {
			return catalogItemFailure(caught, 'create', values);
		}
	},
	update: async ({ locals, request }) => {
		const currentUser = requireUser(locals);
		const values = catalogItemValues(await request.formData());

		try {
			const input = parseCatalogItem(values);
			await updateCatalogItem(currentUser.id, values.id, input);
			return {
				success: true,
				message: `${input.name} was updated.`,
				action: 'update' as const,
				itemId: input.id
			};
		} catch (caught) {
			return catalogItemFailure(caught, 'update', values);
		}
	}
};

function catalogItemValues(formData: FormData): CatalogItemFormValues {
	return {
		id: formString(formData, 'id'),
		name: formString(formData, 'name'),
		description: formString(formData, 'description'),
		price: formString(formData, 'price'),
		imageUrl: formString(formData, 'imageUrl'),
		sortOrder: formString(formData, 'sortOrder'),
		active: formData.get('active') === 'on'
	};
}

function formString(formData: FormData, key: string) {
	const value = formData.get(key);
	return typeof value === 'string' ? value : '';
}

function catalogItemFailure(
	caught: unknown,
	action: 'create' | 'update',
	values: CatalogItemFormValues
) {
	if (caught instanceof CatalogItemValidationError) {
		return fail(422, {
			success: false,
			message: caught.message,
			action,
			itemId: values.id,
			values
		});
	}
	if (caught instanceof ShopError) {
		return fail(caught.status, {
			success: false,
			message: caught.message,
			action,
			itemId: values.id,
			values
		});
	}
	console.error('[shop/admin] Unexpected catalog item error', caught);
	return fail(500, {
		success: false,
		message: 'Could not save the shop item.',
		action,
		itemId: values.id,
		values
	});
}
