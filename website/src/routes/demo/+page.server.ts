import type { PageServerLoad } from './$types';
import { redirect } from '@sveltejs/kit';
import { auth } from '$lib/server/auth';

export const load: PageServerLoad = async () => {
	const res = await auth.api.signInWithOAuth2({
		body: {
			providerId: 'hackclub'
		}
	});

	if (res.redirect) {
		redirect(302, res.url);
	}
};
