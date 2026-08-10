import type { LayoutServerLoad } from './$types';
import { isUserAdmin } from '$lib/server/admin';

export const load: LayoutServerLoad = async ({ locals }) => {
	return {
		user: locals.user,
		isAdmin: locals.user ? await isUserAdmin(locals.user.id) : false
	};
};
