import { auth } from '$lib/server/auth';

import { redirect, type Actions } from '@sveltejs/kit';

export const actions: Actions = {
	signOut: async (event) => {
		await auth.api.signOut({
			headers: event.request.headers
		});

		return redirect(302, '/');
	}
};
