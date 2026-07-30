import type { Actions, PageServerLoad } from './$types';
import { redirect } from '@sveltejs/kit';
import { auth } from '$lib/server/auth';
import { resolve } from '$app/paths';

export const load: PageServerLoad = async (event) => {
	if (event.locals.user) {
		redirect(302, resolve('/home'));
	}

	return {};
};

export const actions: Actions = {
	signIn: async ({ request, url }) => {
		const formData = await request.formData();
		const returnTo = formData.get('returnTo');
		const defaultCallbackURL = new URL(resolve('/home'), url.origin);
		const requestedCallbackURL =
			typeof returnTo === 'string' && returnTo.startsWith('/')
				? new URL(returnTo, url.origin)
				: defaultCallbackURL;
		const callbackURL =
			requestedCallbackURL.origin === url.origin
				? requestedCallbackURL.toString()
				: defaultCallbackURL.toString();

		const res = await auth.api.signInWithOAuth2({
			body: {
				providerId: 'hackclub',
				callbackURL
			}
		});

		if (res.redirect) {
			redirect(302, res.url);
		}
	}
};
