import type { Handle } from '@sveltejs/kit';
import { building } from '$app/environment';
import { auth } from '$lib/server/auth';
import { svelteKitHandler } from 'better-auth/svelte-kit';
import { env } from '$env/dynamic/private';
import { base } from '$app/paths';
import { syncHackClubProfileIfStale } from '$lib/server/profile-sync';

const providerOwnedUserFields = new Set([
	'isAdmin',
	'slackId',
	'verificationStatus',
	'displayName',
	'yswsEligible',
	'pronouns',
	'name',
	'email',
	'emailVerified',
	'image',
	'profileCheckedAt'
]);

export const handle: Handle = async ({ event, resolve }) => {
	let request = event.request;

	if (request.url.startsWith('http://') && env.ORIGIN?.startsWith('https://')) {
		request = new Request(request.url.replace('http://', 'https://'), request);
	}

	if (event.url.pathname.startsWith(`${base}/api/auth/`)) {
		if (
			event.url.pathname === `${base}/api/auth/update-user` &&
			request.method === 'POST' &&
			(await updatesProviderOwnedField(request))
		) {
			return Response.json(
				{ message: 'Provider-owned profile fields cannot be updated directly' },
				{ status: 400 }
			);
		}
		return auth.handler(request);
	}

	const session = await auth.api.getSession({ headers: request.headers });

	if (session) {
		event.locals.session = session.session;
		event.locals.user = {
			...session.user,
			...(await syncHackClubProfileIfStale(session.user.id, request.headers))
		};
	}

	return svelteKitHandler({ event, resolve, auth, building });
};

async function updatesProviderOwnedField(request: Request) {
	try {
		const body = (await request.clone().json()) as unknown;
		return (
			typeof body === 'object' &&
			body !== null &&
			!Array.isArray(body) &&
			Object.keys(body).some((field) => providerOwnedUserFields.has(field))
		);
	} catch {
		return true;
	}
}
