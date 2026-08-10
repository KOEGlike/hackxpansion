import { env } from '$env/dynamic/private';
import { betterAuth } from 'better-auth/minimal';
import { drizzleAdapter } from 'better-auth/adapters/drizzle';
import { sveltekitCookies } from 'better-auth/svelte-kit';
import { getRequestEvent } from '$app/server';
import { db } from '$lib/server/db';
import { genericOAuth } from 'better-auth/plugins';
import { base } from '$app/paths';
import { fetchHackClubProfile, mapHackClubProfile } from '$lib/server/hackclub-profile';

export const auth = betterAuth({
	baseURL: env.ORIGIN,
	basePath: `${base}/api/auth`,
	trustedOrigins: [env.ORIGIN, env.ORIGIN.replace('https://', 'http://')],
	secret: env.BETTER_AUTH_SECRET,
	database: drizzleAdapter(db, { provider: 'pg' }),
	account: {
		encryptOAuthTokens: true
	},
	user: {
		additionalFields: {
			currency: {
				type: 'number',
				required: true,
				defaultValue: 0,
				input: false
			},
			isAdmin: {
				type: 'boolean',
				required: true,
				defaultValue: false,
				input: false,
				returned: false
			},
			slackId: {
				type: 'string',
				required: true
			},
			verificationStatus: {
				type: 'string',
				required: true
			},
			given_name: {
				type: 'string',
				required: false
			},
			yswsEligible: {
				type: 'boolean',
				required: true
			},
			pronouns: {
				type: 'string',
				required: false
			},
			profileCheckedAt: {
				type: 'date',
				required: false,
				returned: false
			},
			githubUsername: {
				type: 'string',
				required: false
			},
			addressLine1: {
				type: 'string',
				required: false,
				returned: false
			},
			addressLine2: {
				type: 'string',
				required: false,
				returned: false
			},
			addressCity: {
				type: 'string',
				required: false,
				returned: false
			},
			addressRegion: {
				type: 'string',
				required: false,
				returned: false
			},
			addressPostalCode: {
				type: 'string',
				required: false,
				returned: false
			},
			addressCountry: {
				type: 'string',
				required: false,
				returned: false
			}
		}
	},
	plugins: [
		genericOAuth({
			config: [
				{
					providerId: 'hackclub',
					discoveryUrl: 'https://auth.hackclub.com/.well-known/openid-configuration',
					clientId: env.HACKCLUB_CLIENT_ID,
					clientSecret: env.HACKCLUB_CLIENT_SECRET,
					overrideUserInfo: true,
					scopes: ['openid', 'email', 'name', 'profile', 'verification_status', 'slack_id'],
					getUserInfo: (tokens) =>
						tokens.accessToken ? fetchHackClubProfile(tokens.accessToken) : Promise.resolve(null),
					mapProfileToUser: (profile) => mapHackClubProfile(profile)
				}
			]
		}),
		sveltekitCookies(getRequestEvent) // make sure this is the last plugin in the array
	]
});
