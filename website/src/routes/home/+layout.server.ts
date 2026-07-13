import type { LayoutServerLoad } from './$types';
import { redirect } from '@sveltejs/kit';

export const load: LayoutServerLoad = async (event) => {
	if (!event.locals.user) {
		return redirect(308, '/');
	}

	return { user: event.locals.user };
};
