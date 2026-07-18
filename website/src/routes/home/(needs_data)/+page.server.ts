import { auth } from '$lib/server/auth';

import { redirect, type Actions } from '@sveltejs/kit';
import { resolve } from '$app/paths';

export const actions: Actions = {
	signOut: async (event) => {
		await auth.api.signOut({
			headers: event.request.headers
		});

		redirect(303, resolve('/'));
	}
};
