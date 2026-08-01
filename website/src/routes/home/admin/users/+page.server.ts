import { error, fail, redirect } from '@sveltejs/kit';
import { resolve } from '$app/paths';
import type { Actions, PageServerLoad } from './$types';
import { requireUser } from '$lib/server/guards';
import {
	demoteUserFromAdmin,
	getAdminUsers,
	promoteUserToAdmin,
	requireAdmin,
	ShopError
} from '$lib/server/shop';

export const load: PageServerLoad = async ({ locals }) => {
	if (!locals.user) error(404, 'Page not found');

	try {
		await requireAdmin(locals.user.id);
		return { users: await getAdminUsers() };
	} catch (caught) {
		if (caught instanceof ShopError) error(caught.status, caught.message);
		throw caught;
	}
};

export const actions: Actions = {
	promote: async ({ locals, request }) => {
		const currentUser = requireUser(locals);
		const formData = await request.formData();
		const userId = formData.get('userId');
		if (typeof userId !== 'string' || !userId) {
			return fail(400, { success: false, message: 'Invalid user.' });
		}

		try {
			const promotedUser = await promoteUserToAdmin(currentUser.id, userId);
			return { success: true, message: `${promotedUser.name} is now an admin.` };
		} catch (caught) {
			if (caught instanceof ShopError) {
				return fail(caught.status, { success: false, message: caught.message, userId });
			}
			console.error('[admin/users] Unexpected promotion error', caught);
			return fail(500, { success: false, message: 'Could not promote the user.', userId });
		}
	},
	demote: async ({ locals, request }) => {
		const currentUser = requireUser(locals);
		const formData = await request.formData();
		const userId = formData.get('userId');
		if (typeof userId !== 'string' || !userId) {
			return fail(400, { success: false, message: 'Invalid user.' });
		}

		let demotedUser;
		try {
			demotedUser = await demoteUserFromAdmin(currentUser.id, userId);
		} catch (caught) {
			if (caught instanceof ShopError) {
				return fail(caught.status, { success: false, message: caught.message, userId });
			}
			console.error('[admin/users] Unexpected demotion error', caught);
			return fail(500, { success: false, message: 'Could not demote the user.', userId });
		}

		if (currentUser.id === userId) redirect(303, resolve('/home'));
		return { success: true, message: `${demotedUser.name} is no longer an admin.` };
	}
};
