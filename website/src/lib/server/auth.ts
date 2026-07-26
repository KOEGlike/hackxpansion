import { env } from '$env/dynamic/private';
import { betterAuth } from 'better-auth/minimal';
import { drizzleAdapter } from 'better-auth/adapters/drizzle';
import { sveltekitCookies } from 'better-auth/svelte-kit';
import { getRequestEvent } from '$app/server';
import { db } from '$lib/server/db';
import { genericOAuth } from 'better-auth/plugins';
import { base } from '$app/paths';
import { fetchWithTimeout } from '$lib/server/http';

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
					getUserInfo: async (tokens) => {
						const res = await fetchWithTimeout('https://auth.hackclub.com/oauth/userinfo', {
							headers: {
								Authorization: `Bearer ${tokens.accessToken}`
							}
						});

						if (!res.ok) return null;

						const data = await res.json();
						let slackData = {};

						try {
							const slackResponse = await fetchWithTimeout(
								`https://cachet.dunkirk.sh/users/${encodeURIComponent(data.slack_id)}`
							);
							if (slackResponse.ok) slackData = await slackResponse.json();
						} catch {
							// Cachet enriches the profile but must not make OAuth unavailable.
						}

						return {
							...slackData,
							...data
						};
					},
					mapProfileToUser: (profile) => ({
						id: profile.sub,
						name: profile.name,
						email: profile.email,
						emailVerified: profile.email_verified === true,
						image: profile.imageUrl,
						slackId: profile.slack_id,
						verificationStatus: profile.verification_status,
						given_name: profile.given_name,
						yswsEligible: profile.ysws_eligible === true,
						pronouns: profile.pronouns
					})
				}
			]
		}),
		sveltekitCookies(getRequestEvent) // make sure this is the last plugin in the array
	]
});
