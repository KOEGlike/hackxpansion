import type { PageServerLoad } from './$types';
import { redirect } from '@sveltejs/kit';
import { auth } from '$lib/server/auth';
import { resolve } from '$app/paths';

export const load: PageServerLoad = async ({ url }) => {
	const callbackURL = new URL(resolve('/demo/better-auth'), url.origin).toString();

	const res = await auth.api.signInWithOAuth2({
		body: {
			providerId: 'hackclub',
			callbackURL
		}
	});

	if (res.redirect) {
		redirect(302, res.url);
	}
};
