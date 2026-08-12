import { isHttpError, type Handle, type HandleServerError } from '@sveltejs/kit';
import { randomUUID } from 'node:crypto';
import { building } from '$app/environment';
import { auth } from '$lib/server/auth';
import { svelteKitHandler } from 'better-auth/svelte-kit';
import { env } from '$env/dynamic/private';
import { base } from '$app/paths';
import { syncHackClubProfileIfStale } from '$lib/server/profile-sync';
import { internalErrorDetails, type InternalErrorDetails } from '$lib/server/error-logging';
import { withRequestContext } from '$lib/server/request-context';

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
	event.locals.requestId = randomUUID();
	return withRequestContext(event.locals.requestId, async () => {
		try {
			const response = await handleRequest(event, resolve);
			if (response.status >= 500) log5xx(event, response.status, event.locals.internalError);
			return response;
		} catch (error) {
			const status = isHttpError(error) ? error.status : 500;
			if (status >= 500) log5xx(event, status, internalErrorDetails(error));
			throw error;
		}
	});
};

export const handleError: HandleServerError = ({ error, event }) => {
	event.locals.internalError = internalErrorDetails(error);
};

async function handleRequest(
	event: Parameters<Handle>[0]['event'],
	resolve: Parameters<Handle>[0]['resolve']
) {
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
}

function log5xx(
	event: Parameters<Handle>[0]['event'],
	status: number,
	error?: InternalErrorDetails
) {
	console.error(
		'[http/5xx]',
		JSON.stringify({
			requestId: event.locals.requestId,
			method: event.request.method,
			path: event.url.pathname,
			route: event.route.id,
			status,
			...(error ? { error } : {})
		})
	);
}

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
