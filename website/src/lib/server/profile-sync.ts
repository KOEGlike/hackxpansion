import { and, eq, isNull, lt, or } from 'drizzle-orm';
import { auth } from '$lib/server/auth';
import { db } from '$lib/server/db';
import { user } from '$lib/server/db/auth.schema';
import { fetchHackClubProfile, mapHackClubProfile } from '$lib/server/hackclub-profile';
import { env } from '$env/dynamic/private';

const PROFILE_SYNC_INTERVAL_MS = 15 * 60 * 1_000;

export async function syncHackClubProfileIfStale(userId: string, headers: Headers) {
	const now = new Date();
	const staleBefore = new Date(now.getTime() - PROFILE_SYNC_INTERVAL_MS);
	const [claimedUser] = await db
		.update(user)
		.set({ profileCheckedAt: now })
		.where(
			and(
				eq(user.id, userId),
				or(isNull(user.profileCheckedAt), lt(user.profileCheckedAt, staleBefore))
			)
		)
		.returning({ id: user.id });

	if (!claimedUser) return null;

	try {
		return await refreshHackClubProfile(userId, headers);
	} catch (error) {
		await db.update(user).set({ profileCheckedAt: null }).where(eq(user.id, userId));
		console.warn('Could not refresh Hack Club profile', error);
		return null;
	}
}

export async function refreshHackClubProfile(userId: string, headers: Headers) {
	const { accessToken } = await auth.api.getAccessToken({
		headers,
		body: { providerId: 'hackclub' }
	});
	const profile = await fetchHackClubProfile(accessToken, env.SLACK_BOT_TOKEN);
	if (!profile) throw new Error('Hack Club userinfo request failed');

	const mapped = mapHackClubProfile(profile);
	const [updatedUser] = await db
		.update(user)
		.set({
			displayName: mapped.name,
			email: mapped.email?.toLowerCase(),
			emailVerified: mapped.emailVerified,
			image: mapped.image,
			slackId: mapped.slackId,
			verificationStatus: mapped.verificationStatus,
			yswsEligible: mapped.yswsEligible,
			pronouns: mapped.pronouns,
			profileCheckedAt: mapped.profileCheckedAt
		})
		.where(eq(user.id, userId))
		.returning({
			name: user.displayName,
			email: user.email,
			emailVerified: user.emailVerified,
			image: user.image,
			slackId: user.slackId,
			verificationStatus: user.verificationStatus,
			yswsEligible: user.yswsEligible,
			pronouns: user.pronouns,
			updatedAt: user.updatedAt
		});

	if (!updatedUser) throw new Error('Local user no longer exists');
	return updatedUser;
}
