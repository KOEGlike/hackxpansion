import type { auth } from '$lib/server/auth';
import type { InternalErrorDetails } from '$lib/server/error-logging';

type AuthSession = typeof auth.$Infer.Session;

// See https://svelte.dev/docs/kit/types#app.d.ts
// for information about these interfaces
declare global {
	namespace App {
		interface Locals {
			user?: AuthSession['user'];
			session?: AuthSession['session'];
			requestId?: string;
			internalError?: InternalErrorDetails;
		}

		// interface Error {}
		// interface PageData {}
		// interface PageState {}
		// interface Platform {}
	}
}

export {};
