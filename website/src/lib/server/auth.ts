import { env } from '$env/dynamic/private';
import { betterAuth } from 'better-auth/minimal';
import { drizzleAdapter } from 'better-auth/adapters/drizzle';
import { sveltekitCookies } from 'better-auth/svelte-kit';
import { getRequestEvent } from '$app/server';
import { db } from '$lib/server/db';
import { genericOAuth } from 'better-auth/plugins';

export const auth = betterAuth({
	baseURL: env.ORIGIN,
	secret: env.BETTER_AUTH_SECRET,
	database: drizzleAdapter(db, { provider: 'pg' }),
	emailAndPassword: { enabled: true },
	plugins: [
		genericOAuth({
			config: [
				{
					providerId: 'hackclub',
					discoveryUrl: 'https://auth.hackclub.com/.well-known/openid-configuration',
					clientId: process.env.HACKCLUB_CLIENT_ID ?? '',
					clientSecret: process.env.HACKCLUB_CLIENT_SECRET,
					scopes: ['openid', 'email', 'name', 'profile', 'verification_status', 'slack_id']
				}
			]
		}),
		sveltekitCookies(getRequestEvent) // make sure this is the last plugin in the array
	]
});
