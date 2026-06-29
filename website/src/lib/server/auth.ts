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
	user: {
		additionalFields: {
			slackId: {
				type: 'string',
				required: false
			},
			verificationStatus: {
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
					scopes: ['openid', 'email', 'name', 'profile', 'verification_status', 'slack_id'],
					mapProfileToUser: (profile) => {
    return {
        // 1. You MUST return the standard fields, otherwise authentication fails!
        name: profile.name,
        email: profile.email,
        emailVerified: profile.email_verified === true,
        image: profile.picture, // or profile.image depending on Hack Club's exact response
        
        // 2. Add your custom fields
        slackId: profile.slack_id,
        verificationStatus: profile.verification_status
        
    } as any; // 3. Cast to 'any' to bypass the strict TypeScript return type
}
				}
			]
		}),
		sveltekitCookies(getRequestEvent) // make sure this is the last plugin in the array
	]
});
