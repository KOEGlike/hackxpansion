import type { Handle } from '@sveltejs/kit';
import { building } from '$app/environment';
import { auth } from '$lib/server/auth';
import { svelteKitHandler } from 'better-auth/svelte-kit';
import { env } from '$env/dynamic/private';

export const handle: Handle = async ({ event, resolve }) => {
	let request = event.request;

	if (request.url.startsWith('http://') && env.ORIGIN?.startsWith('https://')) {
		request = new Request(request.url.replace('http://', 'https://'), request);
	}

	if (event.url.pathname.startsWith('/api/auth/')) {
		return auth.handler(request);
	}

	const session = await auth.api.getSession({ headers: request.headers });

	if (session) {
		event.locals.session = session.session;
		event.locals.user = session.user;
	}

	return svelteKitHandler({ event, resolve, auth, building });
};
